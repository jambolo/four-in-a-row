# Four In A Row

A two-player, drop-a-disc grid game for the desktop, built with Tauri.

Two players take turns dropping discs into a vertical grid; discs fall to the
lowest open cell in the chosen column. First to line up four of their own
discs — horizontally, vertically, or diagonally — wins.

## Status

Skeleton only — the Tauri shell, compute core, and frontend scaffolding are
wired together via a sample `greet` command, but no grid, disc-drop, or
win-detection logic has been implemented yet.

## Architecture

- `src-core/` — `four-in-a-row-core`, a UI-agnostic Rust crate. This is
  where the grid state, move validation, and win detection will live. No
  Tauri dependency, so its tests run headless.
- `src-tauri/` — `four-in-a-row`, the thin Tauri shell. Owns the window and
  exposes core functions as Tauri commands over a small JSON protocol
  (camelCase, serde structs).
- `src/` — the frontend. `api.ts` is the only module that talks to the
  Tauri IPC layer; `main.ts` renders the grid and handles input.

## Development

```bash
pnpm install
pnpm tauri dev
```

```bash
pnpm build       # typecheck + build frontend
pnpm test        # frontend unit tests (Vitest)
cargo test --workspace   # Rust unit tests (core + shell)
```

## License

MIT — see [LICENSE](LICENSE).
