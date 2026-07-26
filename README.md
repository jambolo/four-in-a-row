# Four In A Row

[![CI](https://github.com/jambolo/four-in-a-row/actions/workflows/ci.yml/badge.svg)](https://github.com/jambolo/four-in-a-row/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/jambolo/four-in-a-row/branch/develop/graph/badge.svg)](https://codecov.io/gh/jambolo/four-in-a-row)

A two-player, drop-a-disc grid game for the desktop, built with Tauri.

## How to play

- Two human players share one window: Player 1 plays red discs, Player 2 plays yellow.
- The grid is 7 columns by 6 rows.
- Click anywhere in a column to drop a disc there; it falls to the lowest empty cell in that column. Keys `1-7` drop into the matching column.
- Hovering a column that still has room previews the landing spot with a translucent ghost disc. A full column shows no ghost and a not-allowed cursor.
- The first player to line up four or more of their discs — horizontally, vertically, or diagonally — wins; the winning line is highlighted and a banner announces the winner.
- If all 42 cells fill with no line of four, the game ends in a draw.
- A single button under the board starts a fresh game at any time. It reads "Restart" while a game is ongoing and "Play Again" once the game ends.

## Development

See [DEVELOPMENT.md](DEVELOPMENT.md) for architecture, environment setup, and instructions for building, running, debugging, and testing.

## License

MIT — see [LICENSE](LICENSE).
