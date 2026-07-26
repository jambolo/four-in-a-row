// UI controller. Holds no application logic of its own — it forwards user
// intent through `api` and renders what the backend returns. The Rust core is
// the single source of truth for grid state, move legality, and win
// detection; this module only ever guards on `state.status` / `legalMoves`
// to avoid firing requests the backend would reject anyway.

import { api } from './api';
import { clearGhost, render, showGhost } from './view';
import type { GameState } from './api';

const board = document.getElementById('board') as HTMLElement;
const status = document.getElementById('status') as HTMLElement;
const restart = document.getElementById('restart') as HTMLElement;

const elements = { board, status, restart };

let state: GameState | null = null;

function apply(next: GameState): void {
  state = next;
  render(elements, next);
}

function fail(err: unknown): void {
  console.error(err);
  status.textContent = 'Could not reach the backend. Is the app running under Tauri?';
}

function drop(col: number): void {
  if (state === null || state.status !== 'inProgress' || !state.legalMoves.includes(col)) {
    return;
  }
  api.dropDisc(col).then(apply).catch(fail);
}

function start(): void {
  api.newGame().then(apply).catch(fail);
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

api.getState().then(apply).catch(fail);
