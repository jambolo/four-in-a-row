# Development

Everything needed to set up, build, run, debug, and test Four In A Row.

## Architecture

Three layers with strict separation. The Rust core is the single source of truth for legality and wins; the frontend never decides them.

- `src-core/` — crate `four-in-a-row-core`, a UI-agnostic Rust crate holding the whole rules engine: grid state, move legality, gravity drop, win detection, and draw detection (`Game` in `src-core/src/game.rs`). It also holds the computer player in `src-core/src/ai.rs`: a bitboard `Position`, an `Evaluator` scoring the 69 win lines, a `Generator` yielding legal columns center-first for better pruning, and the entry point `choose_move(game, depth) -> Option<usize>`, which runs the vendored `game-player` crate's alpha-beta minimax search at a fixed depth (no time limit, no iterative deepening) and returns the column to play, or `None` when the game is already over. Its only dependency is the vendored `game-player` crate. No Tauri dependency, so its tests run headless.
- `src-tauri/` — crate `four-in-a-row`, the thin Tauri shell. Owns the window and a single `Mutex<App>` of managed state, where `App` holds the `Game`, a `Config` (per-player `Human`/`Computer` plus the search depth), a `thinking` flag, a `paused` flag, and a `generation` counter. It exposes the core over four commands (`new_game`, `drop_disc`, `get_state`, `set_paused`) using a small camelCase JSON protocol built with serde. When it is a computer's turn, the shell clones the `Game` and runs `choose_move` on a background thread — the lock is never held across the search; when the search finishes, the shell discards the result if the captured `generation` no longer matches (this is how starting a new game cancels an in-flight search), otherwise applies the move and emits an `ai-move` event carrying the full game state. No game logic lives here; its tests pin the wire shapes.
- `src/` — the frontend, vanilla TypeScript with Vite (no framework). `api.ts` is the only module that talks to the Tauri IPC layer, including subscribing to the `ai-move` event; `view.ts` renders the game state into the DOM; `main.ts` is the controller wiring user input to `api` and `view`; `styles.css` holds all styling.

### Wire protocol

Defined by the shell's serde structs and mirrored by hand in `src/api.ts` — change both together, and update the shell's serialization tests that pin the JSON shapes.

- camelCase JSON; the board is column-major: `board[col][row]`, col 0 = leftmost, row 0 = bottom.
- Cells and players are string codes: `"empty" | "p1" | "p2"`.
- `drop_disc` rejects with error-code strings: `invalidColumn | columnFull | gameOver | notHumanTurn` — `notHumanTurn` covers both "it is a computer's turn" and "a search is in flight".
- `new_game` takes `{ p1, p2, searchDepth }`, where `p1`/`p2` are `"human" | "computer"` and `searchDepth` is a ply count in `1..=42`; an out-of-range depth is rejected with the error code `invalidDepth`.
- `GameStateDto` also carries `players` (an object `{ p1, p2 }` of `"human" | "computer"`), `searchDepth`, `thinking`, `paused`, and `generation`, alongside the older board/turn/status fields.
- `generation` is the session counter the shell bumps on every new game, and it is on the wire for a reason: command replies and `ai-move` events reach the front end over two channels with no ordering between them, so a fast search can push its state before the reply to the move that triggered it arrives. The front end orders states by `(generation, moveCount)` and ignores any that is older than the one it already holds. Anything that changes when a state is produced has to keep that pair monotonic.
- The shell pushes an `ai-move` event carrying a full `GameStateDto` after it applies a computer move; the front end subscribes to it once at startup. This event payload is part of the hand-mirrored protocol too — a protocol change has to keep it in sync, not just the command return values.

### Vendored `game-player`

`vendor/game-player` is a git submodule (`https://github.com/jambolo/game-player.git`) providing the AI-player scaffolding. It now backs the shipped computer player in `src-core/src/ai.rs`, whose minimax search and transposition table are used as published. It is third-party code — do not edit it from this repo; change it upstream and bump the submodule pointer.

- The workspace `exclude`s it, so `cargo test --workspace`, `cargo fmt --all`, and `cargo clippy --workspace` skip it. It still compiles as a path dependency of `src-core`.
- Bump it with `git submodule update --remote vendor/game-player`, then commit the new pointer.

### Reference docs

- `docs/rules.md` — authoritative rules reference (7×6 board, gravity drops, 69 possible win lines, win/draw conditions, move notation).
- `docs/implementation.md` — implementation notes.

## Environment setup

### Prerequisites

| Tool | Version | Notes |
| --- | --- | --- |
| Rust | stable, edition 2024 | Install via [rustup](https://rustup.rs). |
| Node.js | LTS | Install via [nvm](https://github.com/nvm-sh/nvm) or the official installer. |
| pnpm | 10.20.0 (see `packageManager`) | `corepack enable` picks up the pinned version automatically. |

Platform toolchains required by Tauri:

- **Windows** — Visual Studio Build Tools with the "Desktop development with C++" workload, and WebView2 (preinstalled on Windows 11).
- **macOS** — Xcode Command Line Tools (`xcode-select --install`).
- **Linux** — the packages CI installs:

  ```bash
  sudo apt-get update
  sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
  ```

### First checkout

```bash
git clone https://github.com/jambolo/four-in-a-row.git
cd four-in-a-row
git submodule update --init --recursive   # fetch vendor/game-player — required before any cargo command
pnpm install
pnpm build                                # populate dist/ before any cargo build
```

`git submodule update --init --recursive` is not optional: without it `vendor/game-player` is an empty directory and every `cargo` command fails to resolve the path dependency.

### Recommended editor setup

- **VS Code** — `rust-analyzer`, `tauri-vscode`, `ESLint`, `Prettier`.
- Point `rust-analyzer` at the workspace root; `vendor/game-player` is excluded from the workspace and is analyzed only as a path dependency.

## Build order

`tauri::generate_context!` embeds the `frontendDist` directory (`dist/`) at compile time, so **`pnpm build` must run before any `cargo build`, `cargo test`, `cargo clippy`, or `cargo llvm-cov`** on a clean checkout. If `dist/` is missing, the Rust build fails with a context-generation error rather than a useful message.

`pnpm tauri dev` and `pnpm tauri build` handle this themselves via `beforeDevCommand` / `beforeBuildCommand` in `src-tauri/tauri.conf.json`.

## Running

```bash
pnpm tauri dev          # full app: Vite dev server on :5173 + Tauri window, hot reload
pnpm dev                # frontend only, in a browser (Tauri IPC calls will fail)
pnpm tauri build        # release bundle (installers under src-tauri/target/release/bundle/)
pnpm preview            # serve the built dist/ in a browser
```

`pnpm tauri dev` starts Vite first (fixed port 5173, `strictPort: true` — free the port if it is taken), then launches the shell pointed at `devUrl`. Frontend edits hot-reload; Rust edits trigger a recompile and a window restart.

## Testing

```bash
pnpm test                          # frontend unit tests (Vitest, jsdom)
pnpm vitest run src/api.test.ts    # a single frontend test file
pnpm vitest run -t "drops a disc"  # a single test by name
pnpm vitest                        # watch mode
pnpm coverage                      # frontend coverage → coverage/lcov.info

cargo test --workspace                        # all Rust tests (core + shell)
cargo test -p four-in-a-row-core              # core only
cargo test -p four-in-a-row-core <name>       # a single Rust test
cargo test -- --nocapture                     # show println! output from tests
```

Test layout:

- `src/*.test.ts` — Vitest, jsdom environment, matched by `include: ['src/**/*.test.ts']` in `vite.config.ts`. `api.test.ts` mocks `@tauri-apps/api`; `view.test.ts` asserts DOM output; `main.test.ts` covers the controller wiring.
- `src-core/src/game.rs` — unit tests for the rules engine, headless and fast.
- `src-core/src/ai.rs` — unit tests for the computer player: bitboard round-trip against `Game`, evaluator terminal values, and `choose_move` behaviour, headless like the rules tests.
- `src-tauri/src/main.rs` — tests that pin the JSON wire shapes, not game logic.

Rust coverage (as CI runs it):

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --workspace --lcov --output-path lcov.info
```

## Linting and formatting

```bash
pnpm lint            # ESLint
pnpm format          # Prettier, write
pnpm format:check    # Prettier, check only
pnpm typecheck       # tsc --noEmit

cargo fmt --all                # format Rust
cargo fmt --all --check        # check only
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Clippy compiles the Tauri shell, so `dist/` must exist first.

Run before pushing:

```bash
pnpm build && pnpm test && pnpm lint && pnpm format:check
cargo fmt --all --check && cargo clippy --workspace --all-targets --all-features -- -D warnings && cargo test --workspace
```

## Debugging

### Frontend

- Open devtools in the Tauri window: **F12**, **Ctrl+Shift+I** (Windows/Linux), or **Cmd+Option+I** (macOS). Available in debug builds; release builds need the `devtools` Cargo feature on `tauri`.
- `console.log` from the webview lands in devtools, not the terminal.
- Vite does not clear the screen (`clearScreen: false`), so Rust logs stay visible alongside frontend output in the same terminal.
- To debug view or controller logic without the shell, run `pnpm dev` and drive the DOM directly — only `api.ts` needs Tauri.

### Rust

- `println!` / `eprintln!` from commands go to the terminal running `pnpm tauri dev`.
- Backtraces: `RUST_BACKTRACE=1 pnpm tauri dev` (PowerShell: `$env:RUST_BACKTRACE=1; pnpm tauri dev`).
- Debugger: start Vite separately with `pnpm dev`, then launch `src-tauri` under the debugger (VS Code + CodeLLDB on macOS/Linux, or the MSVC debugger from Visual Studio on Windows) so the shell attaches to the already-running dev server.
- The rules engine has no Tauri dependency — reproduce a bug as a `#[test]` in `src-core` and debug it headless before touching the shell.

### IPC

- A command that returns an error surfaces in the frontend as a rejected promise carrying one of the error-code strings.
- If a call fails with "command not found", check that the command is registered in the `invoke_handler` in `src-tauri/src/main.rs` and that the name in `src/api.ts` matches.
- Serialization mismatches (a renamed field, a changed enum code) show up as a runtime type error in the frontend, not a compile error — the wire protocol is mirrored by hand.
- A computer move arrives as an `ai-move` event, not as a command return value — if the board stops updating after a computer's turn, check that the front end's event subscription is live (it is set up once at startup in `src/api.ts` / `src/main.ts`).

### Common problems

| Symptom | Cause | Fix |
| --- | --- | --- |
| `cargo` fails resolving `game-player` | Submodule not fetched | `git submodule update --init --recursive` |
| Rust build fails in `generate_context!` | `dist/` missing | `pnpm build` |
| `Port 5173 is already in use` | Stale dev server | Kill the process holding the port (`strictPort` prevents a fallback) |
| App window blank | Vite not ready or a CSP violation | Check the terminal and devtools console; the CSP is set in `tauri.conf.json` |
| Stale frontend in the packaged app | `dist/` not rebuilt | `pnpm build`, then rebuild |

## Continuous integration

`.github/workflows/ci.yml` runs on pushes to `master`, `develop`, `release/**`, and on every pull request.

- **Build & Test** — matrix over `ubuntu-latest`, `windows-latest`, `macos-latest`: install, `pnpm build`, `pnpm test`, `cargo build --workspace --all-targets`, `cargo test --workspace`.
- **Lint & Format** — pull requests only: `cargo fmt --all --check`, clippy with `-D warnings`, `pnpm lint`, `pnpm format:check`.
- **Coverage** — `develop` only: `cargo llvm-cov` plus `pnpm coverage`, uploaded to Codecov.

All checkouts use `submodules: recursive`.

## Releases and versioning

The version lives in four files and must stay in sync: `package.json`, `src-tauri/Cargo.toml`, `src-core/Cargo.toml`, `src-tauri/tauri.conf.json` (plus `Cargo.lock`, refreshed by a cargo build).

Branch model: work lands on `develop` (or `release/**`); releases go to `master`. `.github/workflows/cd.yml` triggers on a `package.json` change pushed to `master`: it builds and tests, tags `v<version>`, and merges `master` back into `develop`.
