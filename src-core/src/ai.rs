//! Computer-player support for Four In A Row.
//!
//! This module adapts the rules engine in [`crate::game`] to the search
//! interfaces of the `game-player` crate. The board is re-encoded as a
//! bitboard [`Position`]: 7 bits per column, so the bit index of `(col, row)`
//! is `col * 7 + row`, and bit `col * 7 + 6` is an always-zero gap bit that
//! keeps shifted win masks from wrapping between columns.

use crate::{COLS, Game, Player, ROWS, Status};
use game_player::minimax::{self, ResponseGenerator};
use game_player::{PlayerId, State, StaticEvaluator};

/// A Four In A Row position in the bitboard encoding used by the search.
#[derive(Clone, Debug)]
pub struct Position {
    /// `masks[0]` holds P1's discs, `masks[1]` holds P2's discs.
    masks: [u64; 2],
    /// Next free row per column, `0..=6`.
    heights: [u32; COLS],
    /// Number of discs placed, `0..=42`.
    move_count: u32,
    /// Whether the player who made the LAST move completed a line.
    won: bool,
}

/// Returns whether a single player's disc mask contains four in a line.
///
/// The shifts are, in order: vertical (1), horizontal (7), diagonal up-right
/// (8) and diagonal down-right (6), in the 7-bit-stride column-major layout.
fn has_win(m: u64) -> bool {
    for shift in [1u64, 7, 6, 8] {
        let x = m & (m >> shift);
        if x & (x >> (2 * shift)) != 0 {
            return true;
        }
    }
    false
}

/// The splitmix64 finalizer, used to mix disc masks into a fingerprint.
fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

impl State for Position {
    type Action = usize;

    fn fingerprint(&self) -> u64 {
        // The masks alone identify the position: the side to move follows from
        // the disc count and the heights follow from the masks.
        splitmix64(self.masks[0] ^ splitmix64(self.masks[1]))
    }

    fn whose_turn(&self) -> PlayerId {
        if self.move_count.is_multiple_of(2) {
            PlayerId::Alice
        } else {
            PlayerId::Bob
        }
    }

    fn is_terminal(&self) -> bool {
        self.won || self.move_count as usize == COLS * ROWS
    }

    fn apply(&self, action: &usize) -> Position {
        let col = *action;
        let side = (self.move_count % 2) as usize;
        let mut next = self.clone();
        next.masks[side] |= 1u64 << (col as u32 * 7 + self.heights[col]);
        next.heights[col] += 1;
        next.move_count += 1;
        next.won = has_win(next.masks[side]);
        next
    }
}

impl From<&Game> for Position {
    fn from(game: &Game) -> Position {
        let mut masks = [0u64; 2];
        let mut heights = [0u32; COLS];
        for (col, height) in heights.iter_mut().enumerate() {
            for row in 0..ROWS {
                match game.cell(col, row) {
                    Some(Player::P1) => masks[0] |= 1u64 << (col * 7 + row),
                    Some(Player::P2) => masks[1] |= 1u64 << (col * 7 + row),
                    None => break,
                }
                *height = (row + 1) as u32;
            }
        }
        Position {
            masks,
            heights,
            move_count: game.move_count(),
            won: matches!(game.status(), Status::Won(_)),
        }
    }
}

/// The value returned when P1 (Alice) has won; its negation means P2 (Bob) won.
const WIN_VALUE: f32 = 10_000.0;

/// Score of a line by how many of one player's discs it holds, when the other
/// player has none in it.
const SCORE: [f32; 4] = [0.0, 1.0, 8.0, 64.0];

/// Bit masks of the 69 four-cell lines that can win the game.
const LINES: [u64; 69] = build_lines();

/// Returns the mask of the single cell `(col, row)`.
const fn bit(col: usize, row: usize) -> u64 {
    1u64 << (col * 7 + row)
}

/// Builds the win-line masks: 24 horizontal, 21 vertical, 12 up-right diagonal
/// and 12 down-right diagonal.
const fn build_lines() -> [u64; 69] {
    let mut lines = [0u64; 69];
    let mut n = 0usize;
    let mut col = 0usize;
    while col + 3 < COLS {
        let mut row = 0usize;
        while row < ROWS {
            lines[n] = bit(col, row) | bit(col + 1, row) | bit(col + 2, row) | bit(col + 3, row);
            n += 1;
            row += 1;
        }
        col += 1;
    }
    col = 0;
    while col < COLS {
        let mut row = 0usize;
        while row + 3 < ROWS {
            lines[n] = bit(col, row) | bit(col, row + 1) | bit(col, row + 2) | bit(col, row + 3);
            n += 1;
            row += 1;
        }
        col += 1;
    }
    col = 0;
    while col + 3 < COLS {
        let mut row = 0usize;
        while row + 3 < ROWS {
            lines[n] = bit(col, row) | bit(col + 1, row + 1) | bit(col + 2, row + 2) | bit(col + 3, row + 3);
            n += 1;
            row += 1;
        }
        col += 1;
    }
    col = 0;
    while col + 3 < COLS {
        let mut row = 3usize;
        while row < ROWS {
            lines[n] = bit(col, row) | bit(col + 1, row - 1) | bit(col + 2, row - 2) | bit(col + 3, row - 3);
            n += 1;
            row += 1;
        }
        col += 1;
    }
    lines
}

/// The static evaluation function used by the search.
pub struct Evaluator;

impl StaticEvaluator for Evaluator {
    type State = Position;

    fn evaluate(&self, state: &Position) -> f32 {
        if state.won {
            // `won` refers to the player who made the last move, so an odd
            // move count means P1 (Alice) completed the line.
            return if state.move_count % 2 == 1 {
                self.alice_wins_value()
            } else {
                self.bob_wins_value()
            };
        }
        if state.move_count as usize == COLS * ROWS {
            return 0.0;
        }
        let mut total = 0.0;
        for line in LINES {
            let a = (state.masks[0] & line).count_ones() as usize;
            let b = (state.masks[1] & line).count_ones() as usize;
            if a == 4 {
                return self.alice_wins_value();
            }
            if b == 4 {
                return self.bob_wins_value();
            }
            if a > 0 && b > 0 {
                continue;
            }
            total += SCORE[a] - SCORE[b];
        }
        total
    }

    fn alice_wins_value(&self) -> f32 {
        WIN_VALUE
    }

    fn bob_wins_value(&self) -> f32 {
        -WIN_VALUE
    }
}

/// The order columns are offered to the search. Central columns belong to more
/// win lines, so trying them first makes alpha-beta pruning cut more branches.
const MOVE_ORDER: [usize; COLS] = [3, 2, 4, 1, 5, 0, 6];

/// Supplies the search with the legal columns of a position.
pub struct Generator;

impl ResponseGenerator for Generator {
    type State = Position;

    fn generate(&self, state: &Position, _depth: u32) -> Vec<usize> {
        // The search treats an empty result as its only signal that the game
        // has ended, so this must be empty exactly when the state is terminal.
        if state.is_terminal() {
            return Vec::new();
        }
        MOVE_ORDER
            .into_iter()
            .filter(|&col| state.heights[col] < ROWS as u32)
            .collect()
    }
}

/// Returns the column the computer plays for the side to move in `game`, or
/// `None` if the game is already over.
///
/// `depth` is a hard search limit in plies; deep values can take a long time.
pub fn choose_move(game: &Game, depth: u32) -> Option<usize> {
    minimax::search(&Evaluator, &Generator, &Position::from(game), depth)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a bitboard mask from explicit `(col, row)` cells.
    fn mask_from(cells: &[(usize, usize)]) -> u64 {
        let mut m = 0u64;
        for &(col, row) in cells {
            m |= 1u64 << (col * 7 + row);
        }
        m
    }

    /// Replays a column sequence into a fresh `Game`, asserting every move is legal.
    fn game_from(cols: &[usize]) -> Game {
        let mut game = Game::new();
        for &col in cols {
            game.drop_disc(col).expect("move should be legal");
        }
        game
    }

    #[test]
    fn has_win_detects_all_four_directions() {
        assert!(has_win(mask_from(&[(0, 0), (0, 1), (0, 2), (0, 3)])));
        assert!(has_win(mask_from(&[(0, 0), (1, 0), (2, 0), (3, 0)])));
        assert!(has_win(mask_from(&[(0, 0), (1, 1), (2, 2), (3, 3)])));
        assert!(has_win(mask_from(&[(0, 3), (1, 2), (2, 1), (3, 0)])));

        assert!(!has_win(mask_from(&[(0, 0), (1, 0), (2, 0), (4, 0)])));
        assert!(!has_win(0));
        assert!(!has_win(mask_from(&[(0, 3), (0, 4), (0, 5), (1, 0)])));
    }

    #[test]
    fn position_round_trips_game() {
        let mut game = Game::new();
        for &col in &[3, 3, 4, 2, 5, 1, 0] {
            game.drop_disc(col).expect("move should be legal");
            let pos = Position::from(&game);

            for col in 0..COLS {
                for row in 0..ROWS {
                    let bit = 1u64 << (col * 7 + row);
                    assert_eq!(pos.masks[0] & bit != 0, game.cell(col, row) == Some(Player::P1));
                    assert_eq!(pos.masks[1] & bit != 0, game.cell(col, row) == Some(Player::P2));
                }
            }

            assert_eq!(pos.move_count, game.move_count());
            assert_eq!(pos.is_terminal(), game.status() != Status::InProgress);
            if game.status() == Status::InProgress {
                assert_eq!(pos.whose_turn() == PlayerId::Alice, game.to_move() == Player::P1);
            }
        }
    }

    #[test]
    fn position_apply_matches_game_drop() {
        let game = game_from(&[0, 6, 1, 6, 2, 5]);
        let pos = Position::from(&game);
        assert!(!pos.is_terminal());

        let won = pos.apply(&3);
        assert!(won.is_terminal());
        assert_eq!(won.move_count, 7);

        let not_won = pos.apply(&4);
        assert!(!not_won.is_terminal());

        assert!(!pos.is_terminal());
        assert_eq!(pos.move_count, 6);

        let mut engine_game = game.clone();
        engine_game.drop_disc(3).expect("move should be legal");
        let engine_pos = Position::from(&engine_game);
        assert_eq!(engine_pos.fingerprint(), won.fingerprint());
    }

    #[test]
    fn fingerprint_is_deterministic_and_position_dependent() {
        let a = game_from(&[3, 4, 3]);
        let b = game_from(&[3, 4, 3]);
        assert_eq!(Position::from(&a).fingerprint(), Position::from(&b).fingerprint());

        let c = game_from(&[3, 4]);
        let d = game_from(&[4, 3]);
        assert_ne!(Position::from(&c).fingerprint(), Position::from(&d).fingerprint());

        let e = game_from(&[3]);
        let f = game_from(&[3, 4]);
        assert_ne!(Position::from(&e).fingerprint(), Position::from(&f).fingerprint());
    }

    #[test]
    fn line_table_covers_all_69_lines() {
        assert_eq!(LINES.len(), 69);

        let mut seen = std::collections::HashSet::new();
        for &mask in LINES.iter() {
            assert_eq!(mask.count_ones(), 4);
            assert!(seen.insert(mask), "duplicate line mask {mask:#x}");
            for col in 0..COLS {
                assert_eq!(mask & (1u64 << (col * 7 + 6)), 0, "gap bit set for col {col}");
            }
        }
    }

    #[test]
    fn evaluator_returns_exact_win_values() {
        let game = game_from(&[0, 6, 1, 6, 2, 5, 3]);
        assert_eq!(Evaluator.evaluate(&Position::from(&game)), 10_000.0);

        let game = game_from(&[0, 1, 0, 2, 0, 3, 5, 4]);
        assert_eq!(Evaluator.evaluate(&Position::from(&game)), -10_000.0);

        assert_eq!(Evaluator.alice_wins_value(), 10_000.0);
        assert_eq!(Evaluator.bob_wins_value(), -10_000.0);
    }

    #[test]
    fn evaluator_returns_zero_on_drawn_board() {
        let cols = [
            6, 6, 4, 5, 4, 6, 3, 4, 4, 3, 2, 1, 6, 5, 2, 6, 3, 6, 2, 2, 0, 1, 4, 0, 0, 0, 0, 5, 2, 3, 2, 4, 3, 3, 5, 1, 1, 5, 0, 1,
            5, 1,
        ];
        let game = game_from(&cols);
        assert_eq!(game.status(), Status::Draw);

        let pos = Position::from(&game);
        assert!(pos.is_terminal());
        assert_eq!(Evaluator.evaluate(&pos), 0.0);
    }

    #[test]
    fn evaluator_scores_center_above_edge_for_alice() {
        let empty = Game::new();
        assert_eq!(Evaluator.evaluate(&Position::from(&empty)), 0.0);

        let center = game_from(&[3]);
        let edge = game_from(&[0]);

        let center_value = Evaluator.evaluate(&Position::from(&center));
        let edge_value = Evaluator.evaluate(&Position::from(&edge));

        assert!(center_value > 0.0);
        assert!(edge_value > 0.0);
        assert!(center_value > edge_value);
    }

    #[test]
    fn generator_is_empty_exactly_when_terminal() {
        let game = Game::new();
        let pos = Position::from(&game);
        assert_eq!(Generator.generate(&pos, 1), vec![3, 2, 4, 1, 5, 0, 6]);

        let game = game_from(&[3, 3, 3, 3, 3, 3]);
        let pos = Position::from(&game);
        let generated = Generator.generate(&pos, 1);
        assert!(!generated.contains(&3));
        assert_eq!(generated.len(), 6);

        let game = game_from(&[0, 6, 1, 6, 2, 5, 3]);
        let pos = Position::from(&game);
        assert!(Generator.generate(&pos, 1).is_empty());
    }

    #[test]
    fn choose_move_takes_immediate_win() {
        let game = game_from(&[0, 6, 1, 6, 2, 5]);
        assert_eq!(choose_move(&game, 4), Some(3));
    }

    #[test]
    fn choose_move_blocks_opponent_win() {
        let game = game_from(&[0, 6, 1, 6, 2]);
        assert_eq!(choose_move(&game, 4), Some(3));
    }

    #[test]
    fn choose_move_returns_none_when_game_is_over() {
        let game = game_from(&[0, 6, 1, 6, 2, 5, 3]);
        assert_eq!(game.status(), Status::Won(Player::P1));
        assert_eq!(choose_move(&game, 4), None);

        let cols = [
            6, 6, 4, 5, 4, 6, 3, 4, 4, 3, 2, 1, 6, 5, 2, 6, 3, 6, 2, 2, 0, 1, 4, 0, 0, 0, 0, 5, 2, 3, 2, 4, 3, 3, 5, 1, 1, 5, 0, 1,
            5, 1,
        ];
        let game = game_from(&cols);
        assert_eq!(game.status(), Status::Draw);
        assert_eq!(choose_move(&game, 4), None);
    }

    #[test]
    fn choose_move_depth_one_returns_a_legal_column() {
        let sequences: [&[usize]; 4] = [&[], &[3], &[3, 3, 4, 2, 5, 1, 0], &[0, 1, 2, 3, 4, 5, 6, 0, 1, 2]];
        for cols in sequences {
            let game = game_from(cols);
            let col = choose_move(&game, 1).expect("game should not be over");
            assert!(game.is_legal(col));
        }
    }
}
