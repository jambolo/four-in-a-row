// Pure DTO-to-DOM renderer for the game board, status line and restart
// button. This module never talks to the backend and never decides legality
// or win conditions — it only turns a `GameState` the Rust core already
// produced into DOM, and back out again (the hover ghost) into query params
// the core understands. Keeping rendering here, isolated from `main.ts`'s
// event wiring, is what makes it unit-testable with plain jsdom fixtures.

import type { GameState, PlayerId } from './api';

const COLUMNS = 7;
const ROWS = 6;

/** The three elements this module renders into. Owned by `index.html`. */
export interface ViewElements {
  board: HTMLElement;
  status: HTMLElement;
  restart: HTMLElement;
}

const PLAYER_NAMES: Record<PlayerId, string> = {
  p1: 'Red',
  p2: 'Yellow',
};

/** The exact status-line text for the current game state. */
export function statusText(state: GameState): string {
  if (state.status === 'draw') {
    return "It's a draw!";
  }
  if (state.status === 'won') {
    // `winner` is non-null exactly when status is 'won' (api.ts contract).
    return `${PLAYER_NAMES[state.winner as PlayerId]} wins!`;
  }
  return `${PLAYER_NAMES[state.toMove]}'s turn`;
}

/** Build one `.cell` div (with its `.disc` child) for `col`/`row`. */
function buildCell(state: GameState, col: number, row: number): HTMLDivElement {
  const cell = document.createElement('div');
  cell.className = 'cell';
  cell.setAttribute('data-col', String(col));
  cell.setAttribute('data-row', String(row));
  cell.setAttribute('data-value', state.board[col][row]);

  if (state.winningCells.some((winCell) => winCell.col === col && winCell.row === row)) {
    cell.classList.add('cell--win');
  }
  if (state.lastMove !== null && state.lastMove.col === col && state.lastMove.row === row) {
    cell.classList.add('cell--drop');
  }

  const disc = document.createElement('div');
  disc.className = 'disc';
  cell.appendChild(disc);

  return cell;
}

/** Build one `.column` button (with its 6 cells) for `col`. */
function buildColumn(state: GameState, col: number): HTMLButtonElement {
  const column = document.createElement('button');
  column.type = 'button';
  column.className = 'column';
  column.setAttribute('data-col', String(col));

  const legal = state.status === 'inProgress' && state.legalMoves.includes(col);
  column.setAttribute('data-legal', String(legal));
  column.setAttribute('aria-disabled', String(!legal));
  column.setAttribute('aria-label', `Drop in column ${col + 1}`);

  // Visual top of the column first: descending row order.
  for (let row = ROWS - 1; row >= 0; row -= 1) {
    column.appendChild(buildCell(state, col, row));
  }

  return column;
}

/** Rebuild `el.status`'s children: an optional swatch, then the status text. */
function renderStatus(status: HTMLElement, state: GameState): void {
  status.replaceChildren();

  if (state.status !== 'draw') {
    const swatch = document.createElement('span');
    swatch.className = 'swatch';
    const player = state.status === 'won' ? (state.winner as PlayerId) : state.toMove;
    swatch.setAttribute('data-player', player);
    status.appendChild(swatch);
  }

  status.appendChild(document.createTextNode(statusText(state)));
}

/**
 * Render `state` into `el`. Rebuilds the board from scratch every call — a
 * fresh set of DOM nodes restarts the CSS drop animation on the new
 * last-move cell, which a targeted attribute update would not do.
 */
export function render(el: ViewElements, state: GameState): void {
  const columns: HTMLButtonElement[] = [];
  for (let col = 0; col < COLUMNS; col += 1) {
    columns.push(buildColumn(state, col));
  }
  el.board.replaceChildren(...columns);
  el.board.setAttribute('data-locked', String(state.status !== 'inProgress'));

  renderStatus(el.status, state);

  // One button serves both roles: mid-game it is an escape hatch, at game end
  // it is the primary call to action.
  const over = state.status !== 'inProgress';
  el.restart.textContent = over ? 'Play Again' : 'Restart';
  el.restart.classList.toggle('button--primary', over);
}

/** Remove any hover-preview ghost disc from `board`. */
export function clearGhost(board: HTMLElement): void {
  board.querySelectorAll('.cell--ghost').forEach((cell) => {
    cell.classList.remove('cell--ghost');
    cell.removeAttribute('data-ghost');
  });
}

/**
 * Preview where a disc would land in `col` if the current player dropped
 * one now. No-op when the column isn't legal or the game is over.
 */
export function showGhost(board: HTMLElement, col: number, state: GameState): void {
  clearGhost(board);

  if (state.status !== 'inProgress' || !state.legalMoves.includes(col)) {
    return;
  }

  const row = state.board[col].indexOf('empty');
  if (row === -1) {
    return;
  }

  const cell = board.querySelector(`.cell[data-col="${col}"][data-row="${row}"]`);
  if (cell === null) {
    return;
  }
  cell.classList.add('cell--ghost');
  cell.setAttribute('data-ghost', state.toMove);
}
