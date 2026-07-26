use four_in_a_row_core::{Game, MoveError, Player, Status};

fn play(cols: &[usize]) -> Game {
    let mut game = Game::new();
    for &col in cols {
        game.drop_disc(col).expect("scripted move must be legal");
    }
    game
}

#[test]
fn horizontal_win_is_detected() {
    let game = play(&[0, 0, 1, 1, 2, 2, 3]);

    assert_eq!(game.status(), Status::Won(Player::P1));
    assert_eq!(game.last_move(), Some((3, 0)));
    assert_eq!(game.move_count(), 7);
    let cells = vec![(0, 0), (1, 0), (2, 0), (3, 0)];
    assert_eq!(game.winning_cells(), cells);
    for &(col, row) in &cells {
        assert_eq!(game.cell(col, row), Some(Player::P1));
    }
}

#[test]
fn vertical_win_is_detected() {
    let game = play(&[0, 1, 0, 1, 0, 1, 0]);

    assert_eq!(game.status(), Status::Won(Player::P1));
    assert_eq!(game.last_move(), Some((0, 3)));
    assert_eq!(game.move_count(), 7);
    let cells = vec![(0, 0), (0, 1), (0, 2), (0, 3)];
    assert_eq!(game.winning_cells(), cells);
    for &(col, row) in &cells {
        assert_eq!(game.cell(col, row), Some(Player::P1));
    }
}

#[test]
fn diagonal_up_right_win_is_detected() {
    let game = play(&[0, 1, 1, 2, 3, 2, 2, 3, 3, 6, 3]);

    assert_eq!(game.status(), Status::Won(Player::P1));
    assert_eq!(game.last_move(), Some((3, 3)));
    assert_eq!(game.move_count(), 11);
    let cells = vec![(0, 0), (1, 1), (2, 2), (3, 3)];
    assert_eq!(game.winning_cells(), cells);
    for &(col, row) in &cells {
        assert_eq!(game.cell(col, row), Some(Player::P1));
    }
}

#[test]
fn diagonal_down_right_win_is_detected() {
    let game = play(&[6, 5, 5, 4, 0, 4, 4, 3, 0, 3, 0, 3, 3]);

    assert_eq!(game.status(), Status::Won(Player::P1));
    assert_eq!(game.last_move(), Some((3, 3)));
    assert_eq!(game.move_count(), 13);
    let cells = vec![(3, 3), (4, 2), (5, 1), (6, 0)];
    assert_eq!(game.winning_cells(), cells);
    for &(col, row) in &cells {
        assert_eq!(game.cell(col, row), Some(Player::P1));
    }
}

#[test]
fn five_in_a_row_reports_all_five_cells() {
    let game = play(&[0, 6, 1, 6, 3, 6, 4, 5, 2]);

    assert_eq!(game.status(), Status::Won(Player::P1));
    assert_eq!(game.last_move(), Some((2, 0)));
    assert_eq!(game.move_count(), 9);
    let cells = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)];
    assert_eq!(game.winning_cells(), cells);
    for &(col, row) in &cells {
        assert_eq!(game.cell(col, row), Some(Player::P1));
    }
}

#[test]
fn win_on_the_final_cell_is_detected() {
    let mut game = play(&[
        2, 3, 2, 4, 4, 3, 4, 4, 2, 6, 6, 1, 0, 1, 3, 6, 5, 1, 6, 4, 5, 6, 1, 6, 4, 0, 3, 0, 0, 0, 3, 1, 1, 5, 0, 5, 5, 2, 5, 2, 2,
        3,
    ]);

    assert_eq!(game.status(), Status::Won(Player::P2));
    assert_eq!(game.last_move(), Some((3, 5)));
    assert_eq!(game.move_count(), 42);
    let cells = vec![(3, 5), (4, 4), (5, 3), (6, 2)];
    assert_eq!(game.winning_cells(), cells);
    for &(col, row) in &cells {
        assert_eq!(game.cell(col, row), Some(Player::P2));
    }

    assert!(game.legal_moves().is_empty());
    assert_eq!(game.drop_disc(0), Err(MoveError::GameOver));
}

#[test]
fn no_win_before_the_line_completes() {
    let game = play(&[0, 0, 1, 1, 2, 2]);

    assert_eq!(game.status(), Status::InProgress);
    assert!(game.winning_cells().is_empty());
    assert_eq!(game.legal_moves(), vec![0, 1, 2, 3, 4, 5, 6]);
}
