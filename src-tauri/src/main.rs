//! Tauri shell — the thin adapter layer.
//!
//! This binary owns the window and exposes the Four In A Row game core to
//! the web UI as a small set of commands. It contains no game logic — that
//! lives in the core crate (grid state, move rules, win detection). What it
//! does own is the session: who plays each side, how deep the computer
//! searches, and whether a search is running or paused.

// Hide the console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use four_in_a_row_core::{COLS, Game, MoveError, Player, ROWS, Status, choose_move};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

// ---------------------------------------------------------------------------
// Protocol — the JSON shapes the front end consumes (mirrored in src/api.ts).
// ---------------------------------------------------------------------------

/// Who plays a side: the person at the keyboard, or the search.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum PlayerKind {
    Human,
    Computer,
}

/// Which kind of player controls each side.
#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct PlayersDto {
    p1: PlayerKind,
    p2: PlayerKind,
}

/// The settings a game is started with (mirrored in `src/api.ts`).
#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct ConfigDto {
    p1: PlayerKind,
    p2: PlayerKind,
    search_depth: u32,
}

impl Default for ConfigDto {
    fn default() -> Self {
        ConfigDto {
            p1: PlayerKind::Human,
            p2: PlayerKind::Human,
            search_depth: 7,
        }
    }
}

/// A board coordinate on the wire: 0-indexed, col left-to-right, row bottom-to-top.
#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct CellDto {
    col: usize,
    row: usize,
}

/// The full game state the front end renders (mirrored in `src/api.ts`).
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct GameStateDto {
    board: Vec<Vec<&'static str>>,
    to_move: &'static str,
    status: &'static str,
    winner: Option<&'static str>,
    winning_cells: Vec<CellDto>,
    legal_moves: Vec<usize>,
    last_move: Option<CellDto>,
    move_count: u32,
    players: PlayersDto,
    search_depth: u32,
    thinking: bool,
    paused: bool,
    /// Which game this state belongs to; see [`App::generation`]. The front end
    /// receives states over two unordered channels (command replies and
    /// `"ai-move"` events), so it needs `(generation, move_count)` to tell
    /// which of two states it holds is the newer one.
    generation: u64,
}

/// The wire code for a player.
fn player_code(player: Player) -> &'static str {
    match player {
        Player::P1 => "p1",
        Player::P2 => "p2",
    }
}

/// The wire code for a rejected move.
fn error_code(error: MoveError) -> &'static str {
    match error {
        MoveError::InvalidColumn => "invalidColumn",
        MoveError::ColumnFull => "columnFull",
        MoveError::GameOver => "gameOver",
    }
}

/// Check a requested search depth in plies, yielding the wire error code.
///
/// The upper bound is the number of cells on the board: no game can run
/// longer, so a deeper search would only re-tread terminal positions.
fn validate_depth(depth: u32) -> Result<(), &'static str> {
    if (1..=42).contains(&depth) {
        Ok(())
    } else {
        Err("invalidDepth")
    }
}

// ---------------------------------------------------------------------------
// Session state
// ---------------------------------------------------------------------------

/// Everything this window owns: the game plus the session around it.
struct App {
    game: Game,
    config: ConfigDto,
    thinking: bool,
    paused: bool,
    /// Bumped whenever a new game starts, so an in-flight search can tell
    /// that its result belongs to a game that no longer exists.
    generation: u64,
}

impl Default for App {
    fn default() -> Self {
        App {
            game: Game::new(),
            config: ConfigDto::default(),
            thinking: false,
            paused: false,
            generation: 0,
        }
    }
}

impl App {
    /// Who plays the given side in this session.
    fn kind(&self, player: Player) -> PlayerKind {
        match player {
            Player::P1 => self.config.p1,
            Player::P2 => self.config.p2,
        }
    }
}

/// Whether the person at the keyboard is allowed to drop a disc right now.
fn human_may_move(app: &App) -> bool {
    !app.thinking && app.kind(app.game.to_move()) == PlayerKind::Human
}

impl From<&App> for GameStateDto {
    fn from(app: &App) -> Self {
        let game = &app.game;
        let board = (0..COLS)
            .map(|col| {
                (0..ROWS)
                    .map(|row| game.cell(col, row).map_or("empty", player_code))
                    .collect()
            })
            .collect();
        let (status, winner) = match game.status() {
            Status::InProgress => ("inProgress", None),
            Status::Won(player) => ("won", Some(player_code(player))),
            Status::Draw => ("draw", None),
        };
        GameStateDto {
            board,
            to_move: player_code(game.to_move()),
            status,
            winner,
            winning_cells: game
                .winning_cells()
                .into_iter()
                .map(|(col, row)| CellDto { col, row })
                .collect(),
            legal_moves: game.legal_moves(),
            last_move: game.last_move().map(|(col, row)| CellDto { col, row }),
            move_count: game.move_count(),
            players: PlayersDto {
                p1: app.config.p1,
                p2: app.config.p2,
            },
            search_depth: app.config.search_depth,
            thinking: app.thinking,
            paused: app.paused,
            generation: app.generation,
        }
    }
}

/// The one session this window runs, guarded for command-handler access.
type AppState = Mutex<App>;

/// Start a background search if it is a computer's turn and nothing blocks it.
///
/// Returns as soon as the thread is spawned. When the search finishes, the move
/// is applied, the new state goes out as an `"ai-move"` event, and this is
/// called again — which is what makes a computer-versus-computer game play
/// itself to the end.
fn maybe_start_search(app_handle: &AppHandle) {
    let (game, depth, generation) = {
        let state = app_handle.state::<AppState>();
        let mut app = state.lock().expect("app state mutex poisoned");
        if app.thinking || app.paused || !matches!(app.game.status(), Status::InProgress) {
            return;
        }
        if app.kind(app.game.to_move()) != PlayerKind::Computer {
            return;
        }
        app.thinking = true;
        (app.game.clone(), app.config.search_depth, app.generation)
    };

    let handle = app_handle.clone();
    std::thread::spawn(move || {
        // The search runs on a clone with no lock held: it can take a long time.
        let chosen = choose_move(&game, depth);
        let dto = {
            let state = handle.state::<AppState>();
            let mut app = state.lock().expect("app state mutex poisoned");
            if app.generation != generation {
                // A new game started while this ran; the result is stale.
                return;
            }
            app.thinking = false;
            let Some(col) = chosen else {
                // The position was already terminal.
                return;
            };
            let _ = app.game.drop_disc(col);
            GameStateDto::from(&*app)
        };
        let _ = handle.emit("ai-move", dto);
        maybe_start_search(&handle);
    });
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Start a fresh game with `config` and return the state after the reset.
#[tauri::command]
fn new_game(config: ConfigDto, app_handle: AppHandle, state: tauri::State<'_, AppState>) -> Result<GameStateDto, String> {
    validate_depth(config.search_depth).map_err(|code| code.to_string())?;
    {
        let mut app = state.lock().expect("app state mutex poisoned");
        app.game = Game::new();
        app.config = config;
        app.thinking = false;
        app.paused = false;
        app.generation = app.generation.wrapping_add(1);
    }
    maybe_start_search(&app_handle);
    let app = state.lock().expect("app state mutex poisoned");
    Ok(GameStateDto::from(&*app))
}

/// Drop a disc for the player to move into `col`, on behalf of a human.
#[tauri::command]
fn drop_disc(col: usize, app_handle: AppHandle, state: tauri::State<'_, AppState>) -> Result<GameStateDto, String> {
    {
        let mut app = state.lock().expect("app state mutex poisoned");
        if !human_may_move(&app) {
            return Err("notHumanTurn".to_string());
        }
        app.game.drop_disc(col).map_err(|error| error_code(error).to_string())?;
    }
    // The human has moved; a computer opponent may now be to move.
    maybe_start_search(&app_handle);
    let app = state.lock().expect("app state mutex poisoned");
    Ok(GameStateDto::from(&*app))
}

/// Return the current state without changing it.
#[tauri::command]
fn get_state(state: tauri::State<'_, AppState>) -> GameStateDto {
    let app = state.lock().expect("app state mutex poisoned");
    GameStateDto::from(&*app)
}

/// Hold off further searches, or resume them.
#[tauri::command]
fn set_paused(paused: bool, app_handle: AppHandle, state: tauri::State<'_, AppState>) -> GameStateDto {
    {
        let mut app = state.lock().expect("app state mutex poisoned");
        app.paused = paused;
    }
    maybe_start_search(&app_handle);
    let app = state.lock().expect("app state mutex poisoned");
    GameStateDto::from(&*app)
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::new(App::default()))
        .invoke_handler(tauri::generate_handler![new_game, drop_disc, get_state, set_paused])
        .run(tauri::generate_context!())
        .expect("error while running the Tauri application");
}

// The command handlers are thin wrappers over the core crate (where the logic
// and its tests live); what is worth pinning down here is the protocol — the
// exact JSON shapes the front end deserializes (`src/api.ts` mirrors them).
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn play(cols: &[usize]) -> Game {
        let mut g = Game::new();
        for &c in cols {
            g.drop_disc(c).unwrap();
        }
        g
    }

    /// A default session (both sides human, depth 7) around an existing game.
    fn app_with(game: Game) -> App {
        App { game, ..App::default() }
    }

    /// The wire JSON for a game in a default session.
    fn dto(game: Game) -> serde_json::Value {
        serde_json::to_value(GameStateDto::from(&app_with(game))).unwrap()
    }

    #[test]
    fn empty_game_serializes_to_the_wire_shape_the_frontend_expects() {
        let value = dto(Game::new());
        assert_eq!(
            value,
            json!({
                "board": [
                    ["empty", "empty", "empty", "empty", "empty", "empty"],
                    ["empty", "empty", "empty", "empty", "empty", "empty"],
                    ["empty", "empty", "empty", "empty", "empty", "empty"],
                    ["empty", "empty", "empty", "empty", "empty", "empty"],
                    ["empty", "empty", "empty", "empty", "empty", "empty"],
                    ["empty", "empty", "empty", "empty", "empty", "empty"],
                    ["empty", "empty", "empty", "empty", "empty", "empty"]
                ],
                "toMove": "p1",
                "status": "inProgress",
                "winner": null,
                "winningCells": [],
                "legalMoves": [0, 1, 2, 3, 4, 5, 6],
                "lastMove": null,
                "moveCount": 0,
                "players": { "p1": "human", "p2": "human" },
                "searchDepth": 7,
                "thinking": false,
                "paused": false,
                "generation": 0
            })
        );
    }

    #[test]
    fn the_board_is_column_major_with_row_zero_at_the_bottom() {
        let value = dto(play(&[3]));
        assert_eq!(value["board"][3], json!(["p1", "empty", "empty", "empty", "empty", "empty"]));
        for col in [0, 1, 2, 4, 5, 6] {
            assert_eq!(
                value["board"][col],
                json!(["empty", "empty", "empty", "empty", "empty", "empty"])
            );
        }
        assert_eq!(value["lastMove"], json!({ "col": 3, "row": 0 }));
        assert_eq!(value["toMove"], json!("p2"));
        assert_eq!(value["moveCount"], json!(1));
    }

    #[test]
    fn a_win_serializes_status_winner_and_winning_cells() {
        let value = dto(play(&[0, 0, 1, 1, 2, 2, 3]));
        assert_eq!(value["status"], json!("won"));
        assert_eq!(value["winner"], json!("p1"));
        assert_eq!(value["toMove"], json!("p1"));
        assert_eq!(value["moveCount"], json!(7));
        assert_eq!(value["legalMoves"], json!([]));
        assert_eq!(
            value["winningCells"],
            json!([
                { "col": 0, "row": 0 },
                { "col": 1, "row": 0 },
                { "col": 2, "row": 0 },
                { "col": 3, "row": 0 }
            ])
        );
    }

    #[test]
    fn a_full_board_serializes_as_a_draw() {
        let moves = [
            2, 3, 2, 4, 4, 3, 4, 4, 2, 6, 6, 1, 0, 1, 3, 6, 5, 1, 6, 4, 5, 6, 1, 6, 4, 0, 3, 0, 0, 0, 3, 1, 1, 5, 0, 2, 5, 5, 5, 2,
            2, 3,
        ];
        let value = dto(play(&moves));
        assert_eq!(value["status"], json!("draw"));
        assert_eq!(value["winner"], json!(null));
        assert_eq!(value["winningCells"], json!([]));
        assert_eq!(value["legalMoves"], json!([]));
        assert_eq!(value["moveCount"], json!(42));
        assert_eq!(value["lastMove"], json!({ "col": 3, "row": 5 }));
        let board = value["board"].as_array().unwrap();
        for column in board {
            for cell in column.as_array().unwrap() {
                let cell = cell.as_str().unwrap();
                assert!(cell == "p1" || cell == "p2", "unexpected cell value {cell}");
            }
        }
    }

    #[test]
    fn move_errors_map_to_the_wire_error_codes() {
        assert_eq!(error_code(MoveError::InvalidColumn), "invalidColumn");
        assert_eq!(error_code(MoveError::ColumnFull), "columnFull");
        assert_eq!(error_code(MoveError::GameOver), "gameOver");

        assert_eq!(Game::new().drop_disc(7), Err(MoveError::InvalidColumn));
        assert_eq!(play(&[0, 0, 0, 0, 0, 0]).drop_disc(0), Err(MoveError::ColumnFull));
        assert_eq!(play(&[0, 0, 1, 1, 2, 2, 3]).drop_disc(4), Err(MoveError::GameOver));
    }

    #[test]
    fn config_dto_deserializes_from_the_camel_case_wire_shape() {
        let config: ConfigDto = serde_json::from_value(json!({
            "p1": "computer",
            "p2": "human",
            "searchDepth": 21
        }))
        .unwrap();
        assert_eq!(config.p1, PlayerKind::Computer);
        assert_eq!(config.p2, PlayerKind::Human);
        assert_eq!(config.search_depth, 21);

        assert_eq!(ConfigDto::default().p1, PlayerKind::Human);
        assert_eq!(ConfigDto::default().p2, PlayerKind::Human);
        assert_eq!(ConfigDto::default().search_depth, 7);

        assert!(
            serde_json::from_value::<ConfigDto>(json!({
                "p1": "human",
                "p2": "human",
                "search_depth": 7
            }))
            .is_err()
        );
    }

    #[test]
    fn player_kinds_serialize_as_lowercase_wire_codes() {
        assert_eq!(serde_json::to_value(PlayerKind::Human).unwrap(), json!("human"));
        assert_eq!(serde_json::to_value(PlayerKind::Computer).unwrap(), json!("computer"));
    }

    #[test]
    fn the_state_dto_carries_players_depth_thinking_and_paused() {
        let app = App {
            game: play(&[3]),
            config: ConfigDto {
                p1: PlayerKind::Computer,
                p2: PlayerKind::Human,
                search_depth: 12,
            },
            thinking: true,
            paused: true,
            ..App::default()
        };
        let value = serde_json::to_value(GameStateDto::from(&app)).unwrap();
        assert_eq!(value["players"], json!({ "p1": "computer", "p2": "human" }));
        assert_eq!(value["searchDepth"], json!(12));
        assert_eq!(value["thinking"], json!(true));
        assert_eq!(value["paused"], json!(true));
        assert_eq!(value["toMove"], json!("p2"));
        assert_eq!(value["moveCount"], json!(1));

        let base = dto(Game::new());
        assert_eq!(base["players"], json!({ "p1": "human", "p2": "human" }));
        assert_eq!(base["searchDepth"], json!(7));
        assert_eq!(base["thinking"], json!(false));
        assert_eq!(base["paused"], json!(false));
    }

    #[test]
    fn the_state_dto_carries_the_session_generation() {
        // The front end orders states by (generation, moveCount), so the
        // counter that tells one game from the next has to reach it.
        let app = App {
            game: play(&[3]),
            generation: 4,
            ..App::default()
        };
        let value = serde_json::to_value(GameStateDto::from(&app)).unwrap();
        assert_eq!(value["generation"], json!(4));
        assert_eq!(dto(Game::new())["generation"], json!(0));
    }

    #[test]
    fn depth_validation_accepts_one_to_fortytwo_and_rejects_the_rest() {
        assert_eq!(validate_depth(1), Ok(()));
        assert_eq!(validate_depth(7), Ok(()));
        assert_eq!(validate_depth(42), Ok(()));
        assert_eq!(validate_depth(0), Err("invalidDepth"));
        assert_eq!(validate_depth(43), Err("invalidDepth"));
        assert_eq!(validate_depth(u32::MAX), Err("invalidDepth"));
    }

    #[test]
    fn a_human_may_move_only_on_a_human_turn_with_no_search_running() {
        assert!(human_may_move(&app_with(Game::new())));

        assert!(!human_may_move(&App {
            thinking: true,
            ..app_with(Game::new())
        }));

        let computer_vs_human = ConfigDto {
            p1: PlayerKind::Computer,
            p2: PlayerKind::Human,
            search_depth: 7,
        };
        assert!(!human_may_move(&App {
            config: computer_vs_human,
            ..app_with(Game::new())
        }));
        assert!(human_may_move(&App {
            config: computer_vs_human,
            ..app_with(play(&[3]))
        }));
    }
}
