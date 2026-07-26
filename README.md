# Four In A Row

A two-player, drop-a-disc grid game for the desktop, built with Tauri.

## How to play

- Two human players share one window: Player 1 plays red discs, Player 2 plays yellow.
- The grid is 7 columns by 6 rows.
- Click anywhere in a column to drop a disc there; it falls to the lowest empty cell in that column. Keys `1-7` drop into the matching column.
- Hovering a column that still has room previews the landing spot with a translucent ghost disc. A full column shows no ghost and a not-allowed cursor.
- The first player to line up four or more of their discs — horizontally, vertically, or diagonally — wins; the winning line is highlighted and a banner announces the winner.
- If all 42 cells fill with no line of four, the game ends in a draw.
- A single button under the board starts a fresh game at any time. It reads "Restart" while a game is ongoing and "Play Again" once the game ends.

## Architecture

- `src-core/` — crate `four-in-a-row-core`, a UI-agnostic Rust crate with no dependencies. Holds the whole rules engine: grid state, move legality, gravity drop, win detection, and draw detection. No Tauri dependency, so its tests run headless.
- `src-tauri/` — crate `four-in-a-row`, the thin Tauri shell. Owns the window and exposes the core over three commands (`new_game`, `drop_disc`, `get_state`) using a small camelCase JSON protocol built with serde. No game logic lives here.
- `src/` — the frontend, vanilla TypeScript with Vite (no framework). `api.ts` is the only module that talks to the Tauri IPC layer; `view.ts` renders the game state into the DOM; `main.ts` is the controller wiring user input to `api` and `view`; `styles.css` holds all styling. The frontend never decides legality or wins — the Rust core is the single source of truth.

## Development

```bash
pnpm install
pnpm tauri dev
```

```bash
pnpm build
pnpm test
pnpm lint
pnpm format:check
cargo test --workspace
```

`pnpm build` must run before `cargo test --workspace`, because the Tauri shell embeds the built frontend at compile time.

## License

MIT — see [LICENSE](LICENSE).
