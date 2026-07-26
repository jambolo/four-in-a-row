// Unit tests for the pure DOM renderer in view.ts. Runs in jsdom against
// hand-built GameState fixtures — no backend, no main.ts wiring — so these
// pin down exactly what render()/statusText()/showGhost()/clearGhost() do
// with a DTO, independent of how main.ts calls them.

import { describe, expect, it } from 'vitest';
import type { CellValue, GameState } from './api';
import { clearGhost, render, showGhost, statusText, type ViewElements } from './view';

/** A fresh 7-column x 6-row empty board. */
function emptyBoard(): CellValue[][] {
  return Array.from({ length: 7 }, () => Array.from({ length: 6 }, () => 'empty' as CellValue));
}

/** A baseline in-progress, empty-board GameState fixture; override fields per test. */
function makeState(overrides: Partial<GameState> = {}): GameState {
  return {
    board: emptyBoard(),
    toMove: 'p1',
    status: 'inProgress',
    winner: null,
    winningCells: [],
    legalMoves: [0, 1, 2, 3, 4, 5, 6],
    lastMove: null,
    moveCount: 0,
    ...overrides,
  };
}

/** Build the shell view.ts renders into, matching index.html's pinned ids. */
function makeElements(): ViewElements {
  document.body.innerHTML = `
    <p id="status"></p>
    <div id="board"></div>
    <button id="play-again" hidden></button>
  `;
  return {
    status: document.getElementById('status') as HTMLElement,
    board: document.getElementById('board') as HTMLElement,
    playAgain: document.getElementById('play-again') as HTMLElement,
  };
}

function columns(board: HTMLElement): HTMLElement[] {
  return Array.from(board.querySelectorAll('.column'));
}

function cellAt(board: HTMLElement, col: number, row: number): HTMLElement {
  return board.querySelector(`.cell[data-col="${col}"][data-row="${row}"]`) as HTMLElement;
}

describe('render: board structure', () => {
  it('produces 7 columns in ascending data-col order and 42 cells, each column descending from row 5', () => {
    const el = makeElements();
    render(el, makeState());

    const cols = columns(el.board);
    expect(cols).toHaveLength(7);
    expect(cols.map((column) => column.getAttribute('data-col'))).toEqual(['0', '1', '2', '3', '4', '5', '6']);
    expect(el.board.querySelectorAll('.cell')).toHaveLength(42);

    for (const column of cols) {
      const rows = Array.from(column.querySelectorAll('.cell')).map((cell) => cell.getAttribute('data-row'));
      expect(rows).toEqual(['5', '4', '3', '2', '1', '0']);
    }
  });

  it('renders a p1 cell with data-value="p1" at the right col/row', () => {
    const el = makeElements();
    const board = emptyBoard();
    board[3][0] = 'p1';
    render(el, makeState({ board, lastMove: { col: 3, row: 0 } }));

    expect(cellAt(el.board, 3, 0).getAttribute('data-value')).toBe('p1');
    expect(cellAt(el.board, 3, 1).getAttribute('data-value')).toBe('empty');
    expect(cellAt(el.board, 2, 0).getAttribute('data-value')).toBe('empty');
  });
});

describe('statusText', () => {
  it('returns the exact phrase for each state, matching #status after render', () => {
    const cases: Array<[Partial<GameState>, string]> = [
      [{ status: 'inProgress', toMove: 'p1' }, "Red's turn"],
      [{ status: 'inProgress', toMove: 'p2' }, "Yellow's turn"],
      [{ status: 'won', winner: 'p1', winningCells: [{ col: 0, row: 0 }] }, 'Red wins!'],
      [{ status: 'won', winner: 'p2', winningCells: [{ col: 0, row: 0 }] }, 'Yellow wins!'],
      [{ status: 'draw', toMove: 'p1', legalMoves: [] }, "It's a draw!"],
    ];

    for (const [overrides, expected] of cases) {
      expect(statusText(makeState(overrides))).toBe(expected);

      const el = makeElements();
      render(el, makeState(overrides));
      expect(el.status.textContent).toBe(expected);
    }
  });
});

describe('render: status swatch', () => {
  it('shows a swatch for the player to move while in progress', () => {
    const el = makeElements();
    render(el, makeState({ status: 'inProgress', toMove: 'p2' }));

    const swatch = el.status.querySelector('.swatch');
    expect(swatch).not.toBeNull();
    expect(swatch?.getAttribute('data-player')).toBe('p2');
  });

  it('shows a swatch for the winner on a win', () => {
    const el = makeElements();
    render(el, makeState({ status: 'won', winner: 'p1', winningCells: [{ col: 0, row: 0 }] }));

    const swatch = el.status.querySelector('.swatch');
    expect(swatch).not.toBeNull();
    expect(swatch?.getAttribute('data-player')).toBe('p1');
  });

  it('has no swatch on a draw', () => {
    const el = makeElements();
    render(el, makeState({ status: 'draw', legalMoves: [] }));

    expect(el.status.querySelector('.swatch')).toBeNull();
  });
});

describe('render: winning cells and last move', () => {
  it('applies cell--win to exactly the cells in winningCells', () => {
    const el = makeElements();
    const board = emptyBoard();
    const winningCells = [
      { col: 0, row: 0 },
      { col: 1, row: 0 },
      { col: 2, row: 0 },
      { col: 3, row: 0 },
    ];
    for (const { col, row } of winningCells) {
      board[col][row] = 'p1';
    }
    render(el, makeState({ status: 'won', winner: 'p1', board, winningCells }));

    const winCells = Array.from(el.board.querySelectorAll('.cell--win'));
    expect(winCells).toHaveLength(4);
    for (const { col, row } of winningCells) {
      expect(cellAt(el.board, col, row).classList.contains('cell--win')).toBe(true);
    }
    expect(cellAt(el.board, 4, 0).classList.contains('cell--win')).toBe(false);
  });

  it('applies cell--drop to the lastMove cell', () => {
    const el = makeElements();
    const board = emptyBoard();
    board[5][2] = 'p2';
    render(el, makeState({ board, toMove: 'p1', lastMove: { col: 5, row: 2 } }));

    expect(cellAt(el.board, 5, 2).classList.contains('cell--drop')).toBe(true);
    expect(el.board.querySelectorAll('.cell--drop')).toHaveLength(1);
  });
});

describe('render: locked / legality when the game is over', () => {
  it('marks #board data-locked and every column illegal and aria-disabled when not inProgress', () => {
    const el = makeElements();
    render(el, makeState({ status: 'won', winner: 'p1', winningCells: [{ col: 0, row: 0 }], legalMoves: [] }));

    expect(el.board.getAttribute('data-locked')).toBe('true');
    for (const column of columns(el.board)) {
      expect(column.getAttribute('data-legal')).toBe('false');
      expect(column.getAttribute('aria-disabled')).toBe('true');
    }
  });

  it('marks #board data-locked="false" and legal columns while inProgress', () => {
    const el = makeElements();
    render(el, makeState({ legalMoves: [0, 1, 2, 3, 4, 5, 6] }));

    expect(el.board.getAttribute('data-locked')).toBe('false');
    expect(columns(el.board).every((column) => column.getAttribute('data-legal') === 'true')).toBe(true);
  });

  it('marks a column not in legalMoves as data-legal="false" while in progress', () => {
    const el = makeElements();
    render(el, makeState({ legalMoves: [1, 2, 3, 4, 5, 6] }));

    const col0 = columns(el.board).find((column) => column.getAttribute('data-col') === '0');
    expect(col0?.getAttribute('data-legal')).toBe('false');
    expect(col0?.getAttribute('aria-disabled')).toBe('true');
  });
});

describe('render: play-again visibility', () => {
  it('hides play-again while inProgress and shows it otherwise', () => {
    const el = makeElements();
    render(el, makeState({ status: 'inProgress' }));
    expect(el.playAgain.hidden).toBe(true);

    render(el, makeState({ status: 'draw', legalMoves: [] }));
    expect(el.playAgain.hidden).toBe(false);
  });
});

describe('showGhost / clearGhost', () => {
  it('adds cell--ghost and data-ghost to the lowest empty cell of a legal column', () => {
    const el = makeElements();
    const board = emptyBoard();
    board[2][0] = 'p1';
    board[2][1] = 'p2';
    const state = makeState({ board, toMove: 'p1', legalMoves: [0, 1, 2, 3, 4, 5, 6] });
    render(el, state);

    showGhost(el.board, 2, state);

    const ghostCell = cellAt(el.board, 2, 2);
    expect(ghostCell.classList.contains('cell--ghost')).toBe(true);
    expect(ghostCell.getAttribute('data-ghost')).toBe('p1');
    expect(el.board.querySelectorAll('.cell--ghost')).toHaveLength(1);
  });

  it('adds nothing for an illegal column', () => {
    const el = makeElements();
    const state = makeState({ legalMoves: [0, 1, 2, 3, 4, 5] });
    render(el, state);

    showGhost(el.board, 6, state);

    expect(el.board.querySelectorAll('.cell--ghost')).toHaveLength(0);
  });

  it('adds nothing when the game is finished', () => {
    const el = makeElements();
    const state = makeState({ status: 'won', winner: 'p1', winningCells: [{ col: 0, row: 0 }], legalMoves: [] });
    render(el, state);

    showGhost(el.board, 0, state);

    expect(el.board.querySelectorAll('.cell--ghost')).toHaveLength(0);
  });

  it('clearGhost removes cell--ghost and data-ghost', () => {
    const el = makeElements();
    const state = makeState();
    render(el, state);

    showGhost(el.board, 0, state);
    expect(el.board.querySelectorAll('.cell--ghost')).toHaveLength(1);

    clearGhost(el.board);
    const cell = cellAt(el.board, 0, 0);
    expect(el.board.querySelectorAll('.cell--ghost')).toHaveLength(0);
    expect(cell.hasAttribute('data-ghost')).toBe(false);
  });
});
