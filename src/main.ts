// UI controller. Holds no application logic of its own — it forwards user
// intent through `api` and renders what the backend returns. The Rust core is
// the single source of truth for grid state, move legality, and win
// detection; this module only guards on what the backend already told it
// (`isHumanTurn`, `legalMoves`) to avoid firing requests the backend would
// reject anyway.

import { api } from './api';
import { clearGhost, isHumanTurn, render, showGhost } from './view';
import type { Config, GameState, PlayerKind } from './api';

const board = document.getElementById('board') as HTMLElement;
const status = document.getElementById('status') as HTMLElement;
const restart = document.getElementById('restart') as HTMLElement;
const pause = document.getElementById('pause') as HTMLElement;
const p1Kind = document.getElementById('p1-kind') as HTMLSelectElement;
const p2Kind = document.getElementById('p2-kind') as HTMLSelectElement;
const depth = document.getElementById('depth') as HTMLInputElement;

const elements = { board, status, restart, pause };

/** The deepest search the backend accepts, one ply per cell on the board. */
const MAX_DEPTH = 42;

let state: GameState | null = null;

/**
 * Whether `next` is at least as recent as what is already rendered.
 *
 * The backend answers over two channels with no ordering between them: command
 * replies and pushed `ai-move` events. A search that finishes quickly can emit
 * its state before the reply to the move that triggered it arrives, and that
 * older reply would otherwise overwrite it — losing the computer's winning move
 * from the display. `generation` separates games, `moveCount` orders moves
 * within one; equal pairs differ only in `thinking`/`paused`, where last-write
 * is fine.
 */
function isCurrent(next: GameState): boolean {
  if (state === null) {
    return true;
  }
  if (next.generation !== state.generation) {
    return next.generation > state.generation;
  }
  return next.moveCount >= state.moveCount;
}

function apply(next: GameState): void {
  if (!isCurrent(next)) {
    return;
  }
  state = next;
  render(elements, next);
}

function fail(err: unknown): void {
  console.error(err);
  status.textContent = 'Could not reach the backend. Is the app running under Tauri?';
}

/** The settings currently showing in the controls, depth clamped to what the backend accepts. */
function readConfig(): Config {
  const requested = Math.round(Number(depth.value));
  const searchDepth = Number.isFinite(requested) ? Math.min(MAX_DEPTH, Math.max(1, requested)) : 1;
  return {
    p1: p1Kind.value as PlayerKind,
    p2: p2Kind.value as PlayerKind,
    searchDepth,
  };
}

/** Put the session the backend is actually running back into the controls. */
function showConfig(next: GameState): void {
  p1Kind.value = next.players.p1;
  p2Kind.value = next.players.p2;
  depth.value = String(next.searchDepth);
}

/** Take a state that may have come from a different session than the controls show. */
function accept(next: GameState): void {
  if (!isCurrent(next)) {
    return;
  }
  showConfig(next);
  apply(next);
}

function drop(col: number): void {
  if (state === null || !isHumanTurn(state) || !state.legalMoves.includes(col)) {
    return;
  }
  api.dropDisc(col).then(apply).catch(fail);
}

function start(): void {
  api.newGame(readConfig()).then(accept).catch(fail);
}

board.addEventListener('click', (event) => {
  const column = (event.target as Element).closest('.column');
  if (column === null) {
    return;
  }
  const col = Number(column.getAttribute('data-col'));
  drop(col);
});

board.addEventListener('mouseover', (event) => {
  const column = (event.target as Element).closest('.column');
  if (column === null) {
    return;
  }
  const col = Number(column.getAttribute('data-col'));
  if (state !== null) {
    showGhost(board, col, state);
  }
});

board.addEventListener('mouseleave', () => {
  clearGhost(board);
});

document.onkeydown = (event) => {
  if (event.key >= '1' && event.key <= '7') {
    drop(Number(event.key) - 1);
  }
};

restart.addEventListener('click', () => {
  start();
});

pause.addEventListener('click', () => {
  if (state === null) {
    return;
  }
  api.setPaused(!state.paused).then(apply).catch(fail);
});

// One subscription for the whole run: the backend pushes a state every time a
// search plays a move, which is also what drives computer-versus-computer play.
api.onAiMove(apply).catch(fail);

api.getState().then(accept).catch(fail);
