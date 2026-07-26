# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

A two-player, drop-a-disc grid game for the desktop, built with Tauri (Rust backend + TypeScript/Vite frontend, pnpm).

Plan:

1. Implement two-player game of Four In A Row.
2. Add computer player using the `game-player` crate.

## Commands

```bash
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

Build order matters: `tauri::generate_context!` embeds `dist/` at compile time, so `pnpm build` must run before any `cargo build`/`clippy` on a clean checkout.

## Architecture

Three layers, strict separation:

- `src-core/` — `four-in-a-row-core`: UI-agnostic Rust crate. All game logic (grid state, move rules, win detection) belongs here. No Tauri dependency; tests run headless. Rust edition 2024.
- `src-tauri/` — `four-in-a-row`: thin Tauri shell. Owns the window, exposes core functions as Tauri commands, contains no game logic. Translates core types to a JSON protocol (camelCase via serde). Heavy work goes through `tauri::async_runtime::spawn_blocking`. Shell tests pin the wire shapes, not logic.
- `src/` — frontend. `api.ts` is the *only* module that imports `@tauri-apps/api`; every backend call goes through it, and its interfaces mirror the shell's serde structs. `main.ts` renders and forwards user intent; it holds no game logic — the Rust core is the single source of truth.

The current `greet` command is skeleton wiring proving the round trip; replace it with real grid/move commands as the game is implemented.

## Releases and versioning

The version lives in four files and must stay in sync: `package.json`, `src-tauri/Cargo.toml`, `src-core/Cargo.toml`, `src-tauri/tauri.conf.json` (plus `Cargo.lock` via a cargo build).

Branch model: work lands on `develop` (or `release/**`), releases go to `master`. CD triggers on a `package.json` change pushed to `master`: it tags `v<version>` and merges `master` back into `develop`. Coverage runs only on `develop`.
