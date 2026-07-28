//! Game core — all non-UI logic lives here.
//!
//! This crate knows nothing about Tauri, the webview, or any UI. That
//! separation is deliberate: it lets the rules be unit-tested headless,
//! without a GUI. The grid, move rules, and win detection for Four In A Row
//! live here; the `src-tauri` shell exposes them through a thin command
//! layer.

mod ai;
mod game;
pub use ai::{Evaluator, Generator, Position, choose_move};
pub use game::{COLS, Game, MoveError, Player, ROWS, Status};
