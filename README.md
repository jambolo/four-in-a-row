# Four In A Row

[![CI](https://github.com/jambolo/four-in-a-row/actions/workflows/ci.yml/badge.svg)](https://github.com/jambolo/four-in-a-row/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/jambolo/four-in-a-row/branch/develop/graph/badge.svg)](https://codecov.io/gh/jambolo/four-in-a-row)

A two-player, drop-a-disc grid game for the desktop.

## How to play

- Both players share one window: Player 1 plays red discs and moves first, Player 2 plays yellow, and each player can be a human or the computer.
- The grid is 7 columns by 6 rows.
- Click anywhere in a column to drop a disc there; it falls to the lowest empty cell in that column. Keys `1-7` drop into the matching column.
- Hovering a column that still has room previews the landing spot with a translucent ghost disc. A full column shows no ghost and a not-allowed cursor.
- The first player to line up four or more of their discs — horizontally, vertically, or diagonally — wins; the winning line is highlighted and a banner announces the winner.
- If all 42 cells fill with no line of four, the game ends in a draw.
- A single button under the board starts a fresh game at any time. It reads "Restart" while a game is ongoing and "Play Again" once the game ends.
- A dropdown for each player lets you choose "Human" or "Computer." The chosen settings take effect the next time a new game is started.
- A "Depth" box sets how many moves ahead a computer player looks, from 1 to 42, defaulting to 7. Higher numbers play stronger but take longer to move, and there is no time limit, so a high depth can make a turn take a long while.
- While a computer player is choosing its move, the status line says that player is thinking, and the board ignores clicks and the number keys until it is a human player's turn again.
- A "Pause" button appears whenever at least one player is a computer. It toggles to "Resume" and stops the next computer move from starting, though a move already being worked out still lands when it finishes.
- With two computer players, the game plays itself through to the end, with Pause available the whole time.

## Development

See [DEVELOPMENT.md](DEVELOPMENT.md) for architecture, environment setup, and instructions for building, running, debugging, and testing.

## License

MIT — see [LICENSE](LICENSE).
