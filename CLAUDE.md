# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

A two-player, drop-a-disc grid game for the desktop, built with Tauri (Rust backend + TypeScript/Vite frontend, pnpm). The two-player game is implemented; the next planned step is a computer player using the `game-player` crate, which is vendored as a submodule and already wired in as a dependency of `src-core` (nothing uses it yet).

`docs/rules.md` is the authoritative rules reference (7×6 board, gravity drops, 69 possible win lines, win/draw conditions, move notation). Base game only — the variants in its §10 are out of scope unless explicitly requested.

## Commands

```bash
git submodule update --init --recursive   # fetch vendor/game-player (required before any cargo command)
pnpm install             # install frontend deps
pnpm tauri dev           # run the app (dev, hot reload)
pnpm build               # typecheck + build frontend into dist/
pnpm test                # frontend unit tests (Vitest)
pnpm vitest run src/api.test.ts   # single frontend test file
pnpm lint                # ESLint
pnpm format:check        # Prettier check (format with pnpm format)
cargo test --workspace   # Rust tests (core + shell)
cargo test -p four-in-a-row-core <name>   # single Rust test
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Build order matters: `tauri::generate_context!` embeds `dist/` at compile time, so `pnpm build` must run before any `cargo build`/`clippy`/`cargo test` on a clean checkout.

## Architecture

Three layers, strict separation. The Rust core is the single source of truth for legality and wins; the frontend never decides them.

- `src-core/` — `four-in-a-row-core`: UI-agnostic Rust crate holding the whole rules engine (`Game` in `game.rs`: grid state, move legality, gravity drop, win/draw detection). Its only dependency is `game-player` (see below). No Tauri dependency; tests run headless. Rust edition 2024.
- `src-tauri/` — `four-in-a-row`: thin Tauri shell. Owns the window and the single `Mutex<Game>` managed state; exposes exactly three commands — `new_game`, `drop_disc`, `get_state` — each returning the full `GameStateDto`. No game logic here. Shell tests pin the wire shapes, not logic.
- `src/` — frontend, vanilla TypeScript (no framework). `api.ts` is the *only* module that imports `@tauri-apps/api`; its types mirror the shell's serde structs. `view.ts` renders game state into the DOM; `main.ts` is the controller wiring input to `api` and `view`; `styles.css` holds all styling.

### Vendored `game-player`

`vendor/game-player` is a git submodule (`https://github.com/jambolo/game-player.git`) providing the AI-player scaffolding for the planned computer player. It is third-party code — do not edit it from this repo; change it upstream and bump the submodule pointer.

- The workspace `exclude`s it, so `cargo test --workspace`, `cargo fmt --all`, and `cargo clippy --workspace` skip it. It still compiles as a path dependency of `src-core`.
- CI/CD checkouts use `submodules: recursive`; a clean checkout without it fails to build.
- Bump it with `git submodule update --remote vendor/game-player`, then commit the new pointer.

### Wire protocol

Defined by the shell's serde structs, mirrored by hand in `src/api.ts` — change both together, and update the shell's serialization tests that pin the JSON shapes.

- camelCase JSON; board is column-major: `board[col][row]`, col 0 = leftmost, row 0 = bottom.
- Cells/players are string codes: `"empty" | "p1" | "p2"`.
- `drop_disc` rejects with error-code strings: `invalidColumn | columnFull | gameOver`.

## Releases and versioning

The version lives in four files and must stay in sync: `package.json`, `src-tauri/Cargo.toml`, `src-core/Cargo.toml`, `src-tauri/tauri.conf.json` (plus `Cargo.lock` via a cargo build).

Branch model: work lands on `develop` (or `release/**`), releases go to `master`. CD triggers on a `package.json` change pushed to `master`: it tags `v<version>` and merges `master` back into `develop`. Coverage runs only on `develop`.
