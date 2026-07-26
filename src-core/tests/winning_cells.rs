use four_in_a_row_core::{Game, Player, Status};

fn play(cols: &[usize]) -> Game {
    let mut game = Game::new();
    for &col in cols {
        game.drop_disc(col).expect("scripted move must be legal");
    }
    game
}

#[test]
fn empty_while_the_game_is_in_progress() {
    let game = play(&[0, 0, 1, 1, 2, 2]);
    assert_eq!(game.status(), Status::InProgress);
    assert!(game.winning_cells().is_empty());
    assert!(Game::new().winning_cells().is_empty());
}

#[test]
fn empty_on_a_draw() {
    let game = play(&[
        2, 3, 2, 4, 4, 3, 4, 4, 2, 6, 6, 1, 0, 1, 3, 6, 5, 1, 6, 4, 5, 6, 1, 6, 4, 0, 3, 0, 0, 0, 3, 1, 1, 5, 0, 2, 5, 5, 5, 2, 2,
        3,
    ]);
    assert_eq!(game.status(), Status::Draw);
    assert!(game.winning_cells().is_empty());
}

#[test]
fn horizontal_win_cells_are_sorted_by_column() {
    let game = play(&[0, 0, 1, 1, 2, 2, 3]);
    assert_eq!(game.winning_cells(), vec![(0, 0), (1, 0), (2, 0), (3, 0)]);
}

#[test]
fn vertical_win_cells_are_sorted_by_row() {
    let game = play(&[0, 1, 0, 1, 0, 1, 0]);
    assert_eq!(game.winning_cells(), vec![(0, 0), (0, 1), (0, 2), (0, 3)]);
}

#[test]
fn a_run_of_five_reports_all_five_cells() {
    let game = play(&[0, 6, 1, 6, 3, 6, 4, 5, 2]);
    assert_eq!(game.winning_cells(), vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]);
    assert_eq!(game.winning_cells().len(), 5);
}

#[test]
fn crossing_lines_are_unioned_and_deduplicated() {
    let game = play(&[0, 5, 5, 2, 4, 5, 6, 2, 3, 0, 1, 2, 3, 3, 6, 6, 5, 4, 0, 2]);
    assert_eq!(game.status(), Status::Won(Player::P2));
    assert_eq!(game.last_move(), Some((2, 3)));
    assert_eq!(game.move_count(), 20);
    assert_eq!(
        game.winning_cells(),
        vec![(2, 0), (2, 1), (2, 2), (2, 3), (3, 2), (4, 1), (5, 0)]
    );
    assert_eq!(game.winning_cells().len(), 7);
}

#[test]
fn every_winning_cell_holds_the_winner() {
    let five_run = play(&[0, 6, 1, 6, 3, 6, 4, 5, 2]);
    let crossing = play(&[0, 5, 5, 2, 4, 5, 6, 2, 3, 0, 1, 2, 3, 3, 6, 6, 5, 4, 0, 2]);

    for game in [&five_run, &crossing] {
        let winner = match game.status() {
            Status::Won(player) => player,
            _ => panic!("expected a won game"),
        };
        for (col, row) in game.winning_cells() {
            assert_eq!(game.cell(col, row), Some(winner));
        }
    }
}
