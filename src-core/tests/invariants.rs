//! Integration tests that assert the full set of Four In A Row state
//! invariants after every move of several scripted games and of many
//! deterministic pseudo-random games.

use four_in_a_row_core::{COLS, Game, MoveError, Player, ROWS, Status};

/// Number of discs belonging to `player` currently on the board.
fn count(game: &Game, player: Player) -> usize {
    let mut n = 0;
    for col in 0..COLS {
        for row in 0..ROWS {
            if game.cell(col, row) == Some(player) {
                n += 1;
            }
        }
    }
    n
}

/// Whether `player` has a line of four (or more) anywhere on the board.
fn has_line_of_four(game: &Game, player: Player) -> bool {
    const DIRECTIONS: [(isize, isize); 4] = [(1, 0), (0, 1), (1, 1), (1, -1)];
    for col in 0..COLS {
        for row in 0..ROWS {
            for &(dc, dr) in DIRECTIONS.iter() {
                let mut all_match = true;
                for k in 0..4isize {
                    let c = col as isize + dc * k;
                    let r = row as isize + dr * k;
                    if c < 0 || c >= COLS as isize || r < 0 || r >= ROWS as isize {
                        all_match = false;
                        break;
                    }
                    if game.cell(c as usize, r as usize) != Some(player) {
                        all_match = false;
                        break;
                    }
                }
                if all_match {
                    return true;
                }
            }
        }
    }
    false
}

/// Asserts the full set of state invariants hold for `game`. `context`
/// identifies which game and which move produced this state, so a failure
/// pinpoints exactly where the invariant broke.
fn assert_invariants(game: &Game, context: &str) {
    let p1 = count(game, Player::P1);
    let p2 = count(game, Player::P2);

    // 1. P1 has played as many discs as P2, or exactly one more.
    assert!(
        p1 as i64 - p2 as i64 >= 0 && p1 as i64 - p2 as i64 <= 1,
        "invariant 1 (disc count parity) violated at {context}: p1={p1} p2={p2}"
    );

    // 2. Neither player can have placed more discs than columns * rows / 2 rounded up.
    assert!(p1 <= 21, "invariant 2 (max discs P1) violated at {context}: p1={p1}");
    assert!(p2 <= 21, "invariant 2 (max discs P2) violated at {context}: p2={p2}");

    // 3. Only meaningful while the game is still in progress: the terminal
    // move ends the game without advancing `to_move`.
    if game.status() == Status::InProgress {
        assert_eq!(
            game.to_move() == Player::P1,
            p1 == p2,
            "invariant 3 (to_move matches parity) violated at {context}: p1={p1} p2={p2} to_move={:?}",
            game.to_move()
        );
    }

    // 4. Gravity: within a column, no empty cell sits below an occupied one.
    for col in 0..COLS {
        let mut seen_empty = false;
        for row in 0..ROWS {
            let occupied = game.cell(col, row).is_some();
            if seen_empty {
                assert!(!occupied, "invariant 4 (gravity) violated at {context}: col={col} row={row}");
            }
            if !occupied {
                seen_empty = true;
            }
        }
    }

    // 5. Legal moves consistency with status and is_legal.
    let legal = game.legal_moves();
    if game.status() != Status::InProgress {
        assert!(
            legal.is_empty(),
            "invariant 5 (legal_moves empty when over) violated at {context}: legal={legal:?}"
        );
        for c in 0..COLS {
            assert!(
                !game.is_legal(c),
                "invariant 5 (is_legal false when over) violated at {context}: col={c}"
            );
        }
    } else {
        assert!(
            !legal.is_empty(),
            "invariant 5 (legal_moves non-empty in progress) violated at {context}"
        );
        for w in legal.windows(2) {
            assert!(
                w[0] < w[1],
                "invariant 5 (legal_moves strictly ascending) violated at {context}: {legal:?}"
            );
        }
        let expected: Vec<usize> = (0..COLS).filter(|&c| game.is_legal(c)).collect();
        assert_eq!(
            legal, expected,
            "invariant 5 (legal_moves matches is_legal) violated at {context}"
        );
    }

    // 6. At most one player has a line of four anywhere on the board.
    let p1_line = has_line_of_four(game, Player::P1);
    let p2_line = has_line_of_four(game, Player::P2);
    assert!(
        !(p1_line && p2_line),
        "invariant 6 (at most one line of four) violated at {context}"
    );

    // 7. Move count matches disc counts and is bounded.
    assert_eq!(
        game.move_count() as usize,
        p1 + p2,
        "invariant 7 (move_count matches disc counts) violated at {context}"
    );
    assert!(
        game.move_count() <= 42,
        "invariant 7 (move_count bound) violated at {context}: move_count={}",
        game.move_count()
    );

    // 8. winning_cells consistency with status.
    let winning_cells = game.winning_cells();
    match game.status() {
        Status::Won(p) => {
            assert!(
                !winning_cells.is_empty(),
                "invariant 8 (winning_cells non-empty when won) violated at {context}"
            );
            assert!(
                winning_cells.len() >= 4,
                "invariant 8 (winning_cells length >= 4) violated at {context}: {winning_cells:?}"
            );
            for w in winning_cells.windows(2) {
                assert!(
                    w[0] < w[1],
                    "invariant 8 (winning_cells strictly ascending) violated at {context}: {winning_cells:?}"
                );
            }
            for &(c, r) in &winning_cells {
                assert_eq!(
                    game.cell(c, r),
                    Some(p),
                    "invariant 8 (winning_cells hold the winner) violated at {context}: cell ({c}, {r})"
                );
            }
        }
        _ => {
            assert!(
                winning_cells.is_empty(),
                "invariant 8 (winning_cells empty unless won) violated at {context}: {winning_cells:?}"
            );
        }
    }
}

const WIN_SCRIPT: [usize; 42] = [
    2, 3, 2, 4, 4, 3, 4, 4, 2, 6, 6, 1, 0, 1, 3, 6, 5, 1, 6, 4, 5, 6, 1, 6, 4, 0, 3, 0, 0, 0, 3, 1, 1, 5, 0, 5, 5, 2, 5, 2, 2, 3,
];

const DRAW_SCRIPT: [usize; 42] = [
    2, 3, 2, 4, 4, 3, 4, 4, 2, 6, 6, 1, 0, 1, 3, 6, 5, 1, 6, 4, 5, 6, 1, 6, 4, 0, 3, 0, 0, 0, 3, 1, 1, 5, 0, 2, 5, 5, 5, 2, 2, 3,
];

#[test]
fn invariants_hold_on_a_new_game() {
    assert_invariants(&Game::new(), "new game");
}

#[test]
fn invariants_hold_through_a_win_game() {
    let mut game = Game::new();
    assert_invariants(&game, "win game before move 1");
    for (i, &col) in WIN_SCRIPT.iter().enumerate() {
        let move_number = i + 1;
        assert_eq!(
            game.drop_disc(col),
            Ok(()),
            "win game move {move_number} (col {col}) should be legal"
        );
        assert_invariants(&game, &format!("win game after move {move_number}"));
    }
    assert_eq!(game.status(), Status::Won(Player::P2));
    assert_eq!(game.move_count(), 42);
}

#[test]
fn invariants_hold_through_a_draw_game() {
    let mut game = Game::new();
    assert_invariants(&game, "draw game before move 1");
    for (i, &col) in DRAW_SCRIPT.iter().enumerate() {
        let move_number = i + 1;
        assert_eq!(
            game.drop_disc(col),
            Ok(()),
            "draw game move {move_number} (col {col}) should be legal"
        );
        assert_invariants(&game, &format!("draw game after move {move_number}"));
    }
    assert_eq!(game.status(), Status::Draw);
    assert_eq!(game.move_count(), 42);
}

#[test]
fn invariants_hold_after_rejected_moves() {
    let mut game = Game::new();
    for &col in &[3, 3, 3] {
        assert_eq!(game.drop_disc(col), Ok(()));
    }
    assert_invariants(&game, "rejected moves setup");

    assert_eq!(game.drop_disc(COLS), Err(MoveError::InvalidColumn));
    assert_invariants(&game, "after drop_disc(COLS)");

    assert_eq!(game.drop_disc(usize::MAX), Err(MoveError::InvalidColumn));
    assert_invariants(&game, "after drop_disc(usize::MAX)");

    assert_eq!(game.move_count(), 3);
}

#[test]
fn invariants_hold_across_many_deterministic_games() {
    fn next(state: &mut u64) -> u64 {
        *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *state >> 33
    }

    let mut finished = 0;
    for seed in 0..200u64 {
        let mut state = seed;
        let mut game = Game::new();
        assert_invariants(&game, &format!("seed {seed} start"));
        while game.status() == Status::InProgress {
            let legal = game.legal_moves();
            assert!(!legal.is_empty(), "seed {seed}: legal_moves empty while in progress");
            let col = legal[(next(&mut state) as usize) % legal.len()];
            game.drop_disc(col).expect("chosen column must be legal");
            assert_invariants(&game, &format!("seed {seed} move {}", game.move_count()));
        }
        assert!(
            matches!(game.status(), Status::Won(_) | Status::Draw),
            "seed {seed}: game did not end in a won or drawn state"
        );
        assert!(
            game.move_count() >= 7,
            "seed {seed}: move_count {} below minimum possible win length",
            game.move_count()
        );
        assert!(
            game.move_count() <= 42,
            "seed {seed}: move_count {} above board capacity",
            game.move_count()
        );
        finished += 1;
    }
    assert_eq!(finished, 200);
}
