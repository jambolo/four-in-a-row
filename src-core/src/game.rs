//! The Four In A Row rules engine.
//!
//! This module owns the board, turn order, move legality, and win/draw
//! detection. It is the single source of truth for the rules: the Tauri
//! shell and the frontend must never re-implement any of this logic.
//!
//! Coordinates are 0-indexed as `(col, row)`, with `row 0` at the bottom.
//! Columns run `0..COLS` left to right; rows run `0..ROWS` bottom to top.

/// Number of columns on the board.
pub const COLS: usize = 7;
/// Number of rows on the board.
pub const ROWS: usize = 6;

/// The four independent axes checked for a four-in-a-row run.
const DIRECTIONS: [(isize, isize); 4] = [(1, 0), (0, 1), (1, 1), (1, -1)];

/// One of the two players in a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    /// The first player to move.
    P1,
    /// The second player to move.
    P2,
}

impl Player {
    /// Returns the other player.
    pub fn opponent(self) -> Player {
        match self {
            Player::P1 => Player::P2,
            Player::P2 => Player::P1,
        }
    }
}

/// The current outcome of a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The game is ongoing.
    InProgress,
    /// The named player has completed a line of four or more and won.
    Won(Player),
    /// The board is full with no winner.
    Draw,
}

/// Reasons a requested move cannot be applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveError {
    /// The column index is out of range.
    InvalidColumn,
    /// The column has no free row left.
    ColumnFull,
    /// The game has already finished.
    GameOver,
}

/// The full state of a Four In A Row game.
#[derive(Debug, Clone)]
pub struct Game {
    board: [[Option<Player>; ROWS]; COLS],
    heights: [usize; COLS],
    to_move: Player,
    status: Status,
    last_move: Option<(usize, usize)>,
    move_count: u32,
}

impl Game {
    /// Creates a new game: empty board, `Player::P1` to move.
    pub fn new() -> Game {
        Game {
            board: [[None; ROWS]; COLS],
            heights: [0; COLS],
            to_move: Player::P1,
            status: Status::InProgress,
            last_move: None,
            move_count: 0,
        }
    }

    /// Returns the occupant of a cell, or `None` if empty or out of range.
    pub fn cell(&self, col: usize, row: usize) -> Option<Player> {
        if col >= COLS || row >= ROWS {
            return None;
        }
        self.board[col][row]
    }

    /// Returns the player whose turn it is.
    pub fn to_move(&self) -> Player {
        self.to_move
    }

    /// Returns the current game status.
    pub fn status(&self) -> Status {
        self.status
    }

    /// Returns whether dropping a disc into `col` is currently legal.
    pub fn is_legal(&self, col: usize) -> bool {
        self.status == Status::InProgress && col < COLS && self.heights[col] < ROWS
    }

    /// Returns all currently legal columns, in ascending order.
    pub fn legal_moves(&self) -> Vec<usize> {
        (0..COLS).filter(|&c| self.is_legal(c)).collect()
    }

    /// Drops a disc for the player to move into `col`.
    ///
    /// On success, updates the board, advances or ends the turn, and records
    /// the move. On failure, the game state is left completely unchanged.
    pub fn drop_disc(&mut self, col: usize) -> Result<(), MoveError> {
        if self.status != Status::InProgress {
            return Err(MoveError::GameOver);
        }
        if col >= COLS {
            return Err(MoveError::InvalidColumn);
        }
        if self.heights[col] >= ROWS {
            return Err(MoveError::ColumnFull);
        }
        let row = self.heights[col];
        let player = self.to_move;
        self.board[col][row] = Some(player);
        self.heights[col] += 1;
        self.last_move = Some((col, row));
        self.move_count += 1;
        if self.wins_through(col, row, player) {
            self.status = Status::Won(player);
        } else if self.move_count as usize == COLS * ROWS {
            self.status = Status::Draw;
        } else {
            self.to_move = player.opponent();
        }
        Ok(())
    }

    /// Returns the `(col, row)` of the most recently placed disc, if any.
    pub fn last_move(&self) -> Option<(usize, usize)> {
        self.last_move
    }

    /// Returns the number of discs placed so far, `0..=42`.
    pub fn move_count(&self) -> u32 {
        self.move_count
    }

    /// Returns every cell belonging to a winning run through the last move,
    /// deduplicated and sorted ascending by `(col, row)`. Empty unless the
    /// game has been won.
    pub fn winning_cells(&self) -> Vec<(usize, usize)> {
        let Status::Won(player) = self.status else {
            return Vec::new();
        };
        let Some((col, row)) = self.last_move else {
            return Vec::new();
        };
        let mut cells: Vec<(usize, usize)> = Vec::new();
        for &(dc, dr) in DIRECTIONS.iter() {
            let forward = self.run_len(col, row, dc, dr, player) as isize;
            let backward = self.run_len(col, row, -dc, -dr, player) as isize;
            if 1 + forward + backward >= 4 {
                for k in -backward..=forward {
                    let c = col as isize + dc * k;
                    let r = row as isize + dr * k;
                    cells.push((c as usize, r as usize));
                }
            }
        }
        cells.sort_unstable();
        cells.dedup();
        cells
    }

    fn run_len(&self, col: usize, row: usize, dc: isize, dr: isize, player: Player) -> usize {
        let mut k = 0usize;
        let mut c = col as isize;
        let mut r = row as isize;
        loop {
            c += dc;
            r += dr;
            if c < 0 || c >= COLS as isize || r < 0 || r >= ROWS as isize {
                return k;
            }
            if self.board[c as usize][r as usize] != Some(player) {
                return k;
            }
            k += 1;
        }
    }

    fn wins_through(&self, col: usize, row: usize, player: Player) -> bool {
        DIRECTIONS
            .iter()
            .any(|&(dc, dr)| 1 + self.run_len(col, row, dc, dr, player) + self.run_len(col, row, -dc, -dr, player) >= 4)
    }
}

impl Default for Game {
    fn default() -> Self {
        Game::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_is_empty_and_p1_to_move() {
        let game = Game::new();
        for col in 0..COLS {
            for row in 0..ROWS {
                assert_eq!(game.cell(col, row), None);
            }
        }
        assert_eq!(game.to_move(), Player::P1);
        assert_eq!(game.status(), Status::InProgress);
        assert_eq!(game.move_count(), 0);
        assert_eq!(game.last_move(), None);
        assert_eq!(game.winning_cells(), Vec::new());
        assert_eq!(game.legal_moves(), vec![0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn drop_lands_at_bottom_and_alternates_turn() {
        let mut game = Game::new();
        assert_eq!(game.drop_disc(3), Ok(()));
        assert_eq!(game.drop_disc(3), Ok(()));
        assert_eq!(game.cell(3, 0), Some(Player::P1));
        assert_eq!(game.cell(3, 1), Some(Player::P2));
        assert_eq!(game.to_move(), Player::P1);
        assert_eq!(game.move_count(), 2);
        assert_eq!(game.last_move(), Some((3, 1)));
    }

    #[test]
    fn cell_out_of_range_is_none() {
        let game = Game::new();
        assert_eq!(game.cell(7, 0), None);
        assert_eq!(game.cell(0, 6), None);
    }

    #[test]
    fn horizontal_win_is_detected() {
        let mut game = Game::new();
        for col in [0, 0, 1, 1, 2, 2, 3] {
            assert_eq!(game.drop_disc(col), Ok(()));
        }
        assert_eq!(game.status(), Status::Won(Player::P1));
        assert_eq!(game.winning_cells(), vec![(0, 0), (1, 0), (2, 0), (3, 0)]);
    }

    #[test]
    fn opponent_is_involutive() {
        assert_eq!(Player::P1.opponent(), Player::P2);
        assert_eq!(Player::P2.opponent(), Player::P1);
    }

    #[test]
    fn default_matches_new() {
        assert_eq!(Game::default().status(), Game::new().status());
        assert_eq!(Game::default().to_move(), Game::new().to_move());
    }
}
