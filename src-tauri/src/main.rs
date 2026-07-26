//! Tauri shell — the thin adapter layer.
//!
//! This binary owns the window and exposes the Four In A Row game core to
//! the web UI as a small set of commands. It contains no game logic — that
//! lives in the core crate (grid state, move rules, win detection). All it
//! does is translate between core types and the JSON protocol the front end
//! renders.

// Hide the console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use four_in_a_row_core::{COLS, Game, MoveError, Player, ROWS, Status};
use serde::Serialize;

// ---------------------------------------------------------------------------
// Protocol — the JSON shapes the front end consumes (mirrored in src/api.ts).
// ---------------------------------------------------------------------------

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

impl From<&Game> for GameStateDto {
    fn from(game: &Game) -> Self {
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
        }
    }
}

/// The one game this window plays, guarded for command-handler access.
type GameState = Mutex<Game>;

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Start a fresh game and return the state after the reset.
#[tauri::command]
fn new_game(state: tauri::State<'_, GameState>) -> GameStateDto {
    let mut game = state.lock().expect("game state mutex poisoned");
    *game = Game::new();
    GameStateDto::from(&*game)
}

/// Drop a disc for the player to move into `col`.
#[tauri::command]
fn drop_disc(col: usize, state: tauri::State<'_, GameState>) -> Result<GameStateDto, String> {
    let mut game = state.lock().expect("game state mutex poisoned");
    match game.drop_disc(col) {
        Ok(()) => Ok(GameStateDto::from(&*game)),
        Err(error) => Err(error_code(error).to_string()),
    }
}

/// Return the current state without changing it.
#[tauri::command]
fn get_state(state: tauri::State<'_, GameState>) -> GameStateDto {
    let game = state.lock().expect("game state mutex poisoned");
    GameStateDto::from(&*game)
}

fn main() {
    tauri::Builder::default()
        .manage(GameState::new(Game::new()))
        .invoke_handler(tauri::generate_handler![new_game, drop_disc, get_state])
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

    #[test]
    fn empty_game_serializes_to_the_wire_shape_the_frontend_expects() {
        let value = serde_json::to_value(GameStateDto::from(&Game::new())).unwrap();
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
                "moveCount": 0
            })
        );
    }

    #[test]
    fn the_board_is_column_major_with_row_zero_at_the_bottom() {
        let value = serde_json::to_value(GameStateDto::from(&play(&[3]))).unwrap();
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
        let value = serde_json::to_value(GameStateDto::from(&play(&[0, 0, 1, 1, 2, 2, 3]))).unwrap();
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
        let value = serde_json::to_value(GameStateDto::from(&play(&moves))).unwrap();
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
}
