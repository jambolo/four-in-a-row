use four_in_a_row_core::{COLS, Game, MoveError, Player, ROWS, Status};

fn play(cols: &[usize]) -> Game {
    let mut game = Game::new();
    for &col in cols {
        game.drop_disc(col).expect("scripted move must be legal");
    }
    game
}

type Snapshot = (Vec<Option<Player>>, Player, Status, u32, Option<(usize, usize)>);

fn snapshot(game: &Game) -> Snapshot {
    let mut cells = Vec::new();
    for col in 0..COLS {
        for row in 0..ROWS {
            cells.push(game.cell(col, row));
        }
    }
    (cells, game.to_move(), game.status(), game.move_count(), game.last_move())
}

#[test]
fn new_game_initial_state() {
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
    assert!(game.winning_cells().is_empty());
    assert_eq!(game.legal_moves(), vec![0, 1, 2, 3, 4, 5, 6]);

    for col in 0..COLS {
        assert!(game.is_legal(col));
    }
    assert!(!game.is_legal(COLS));
}

#[test]
fn discs_stack_from_the_bottom() {
    let game = play(&[3, 3, 3]);

    assert_eq!(game.cell(3, 0), Some(Player::P1));
    assert_eq!(game.cell(3, 1), Some(Player::P2));
    assert_eq!(game.cell(3, 2), Some(Player::P1));
    assert_eq!(game.cell(3, 3), None);
}

#[test]
fn last_move_and_move_count_track_drops() {
    let mut game = Game::new();

    game.drop_disc(3).expect("legal move");
    assert_eq!(game.last_move(), Some((3, 0)));
    assert_eq!(game.move_count(), 1);

    game.drop_disc(3).expect("legal move");
    assert_eq!(game.last_move(), Some((3, 1)));
    assert_eq!(game.move_count(), 2);

    game.drop_disc(0).expect("legal move");
    assert_eq!(game.last_move(), Some((0, 0)));
    assert_eq!(game.move_count(), 3);
    assert_eq!(game.to_move(), Player::P2);
}

#[test]
fn legal_moves_shrink_as_a_column_fills() {
    let game = play(&[0, 0, 0, 0, 0, 0]);

    assert_eq!(game.status(), Status::InProgress);
    assert_eq!(game.move_count(), 6);
    assert!(!game.is_legal(0));
    assert_eq!(game.legal_moves(), vec![1, 2, 3, 4, 5, 6]);
}

#[test]
fn invalid_column_is_rejected() {
    let mut game = Game::new();

    assert_eq!(game.drop_disc(COLS), Err(MoveError::InvalidColumn));
    assert_eq!(game.drop_disc(usize::MAX), Err(MoveError::InvalidColumn));
}

#[test]
fn column_full_is_rejected() {
    let mut game = play(&[0, 0, 0, 0, 0, 0]);

    assert_eq!(game.drop_disc(0), Err(MoveError::ColumnFull));
}

#[test]
fn game_over_is_rejected_after_a_win() {
    let mut game = play(&[0, 0, 1, 1, 2, 2, 3]);

    assert_eq!(game.status(), Status::Won(Player::P1));
    assert_eq!(game.drop_disc(3), Err(MoveError::GameOver));
    assert_eq!(game.drop_disc(9), Err(MoveError::GameOver));
    assert!(game.legal_moves().is_empty());
    assert!(!game.is_legal(0));
}

#[test]
fn rejected_moves_leave_the_state_unchanged() {
    // Invalid column on a fresh game.
    let mut game = Game::new();
    let before = snapshot(&game);
    assert_eq!(game.drop_disc(COLS), Err(MoveError::InvalidColumn));
    let after = snapshot(&game);
    assert_eq!(before, after);

    // Full column after the fill script.
    let mut game = play(&[0, 0, 0, 0, 0, 0]);
    let before = snapshot(&game);
    assert_eq!(game.drop_disc(0), Err(MoveError::ColumnFull));
    let after = snapshot(&game);
    assert_eq!(before, after);

    // Any drop after the horizontal-win script.
    let mut game = play(&[0, 0, 1, 1, 2, 2, 3]);
    let before = snapshot(&game);
    assert_eq!(game.drop_disc(3), Err(MoveError::GameOver));
    let after = snapshot(&game);
    assert_eq!(before, after);
}

#[test]
fn draw_when_the_board_fills() {
    let mut game = play(&[
        2, 3, 2, 4, 4, 3, 4, 4, 2, 6, 6, 1, 0, 1, 3, 6, 5, 1, 6, 4, 5, 6, 1, 6, 4, 0, 3, 0, 0, 0, 3, 1, 1, 5, 0, 2, 5, 5, 5, 2, 2,
        3,
    ]);

    assert_eq!(game.status(), Status::Draw);
    assert_eq!(game.move_count(), 42);
    assert_eq!(game.last_move(), Some((3, 5)));
    assert_eq!(game.to_move(), Player::P2);
    assert!(game.winning_cells().is_empty());
    assert!(game.legal_moves().is_empty());
    assert_eq!(game.drop_disc(0), Err(MoveError::GameOver));

    for col in 0..COLS {
        for row in 0..ROWS {
            assert!(game.cell(col, row).is_some());
        }
    }
}
