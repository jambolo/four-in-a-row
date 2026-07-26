// Behavioural tests for the UI controller (main.ts) against a mocked backend.
// main.ts is DOM-driven glue, so these run in jsdom with `./api` replaced by a
// mock but the real `./view` renderer, exercising the actual rendered board:
// clicks and key presses forward the right column to the backend, hover
// previews a ghost disc, and the status line reflects the returned state
// (including the won/draw/error cases) without main.ts re-deriving any of it.

import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { CellValue, GameState } from './api';

const mockApi = vi.hoisted(() => ({
  newGame: vi.fn<() => Promise<GameState>>(),
  dropDisc: vi.fn<(col: number) => Promise<GameState>>(),
  getState: vi.fn<() => Promise<GameState>>(),
}));

vi.mock('./api', () => ({ api: mockApi }));

/** Let all pending promise chains (event -> invoke -> render) settle. */
const flush = () => new Promise<void>((resolve) => setTimeout(resolve, 0));

const SHELL = `
  <main id="app">
    <h1>Four In A Row</h1>
    <p id="status" class="status" role="status" aria-live="polite"></p>
    <div id="board" class="board" data-locked="false" aria-label="Game board"></div>
    <p id="hint" class="hint">Click a column to drop a disc, or press 1-7.</p>
    <div class="controls">
      <button id="new-game" class="button" type="button">New game</button>
      <button id="play-again" class="button button--primary" type="button" hidden>Play again</button>
    </div>
  </main>
`;

/** Reproduce the pinned `index.html` shell in `document.body`. */
function shell(): void {
  document.body.innerHTML = SHELL;
}

/** Build a fresh, empty in-progress game state, with any overrides applied. */
function makeState(overrides?: Partial<GameState>): GameState {
  return {
    board: Array.from({ length: 7 }, () => Array.from({ length: 6 }, () => 'empty' as CellValue)),
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

/** Boot `main.ts` fresh against `initial` as the resolved boot state. */
async function boot(initial: GameState): Promise<void> {
  shell();
  mockApi.getState.mockResolvedValueOnce(initial);
  vi.resetModules();
  await import('./main');
  await flush();
}

const board = () => document.getElementById('board') as HTMLElement;
const status = () => document.getElementById('status') as HTMLElement;
const columns = () => board().querySelectorAll('.column');
const cells = () => board().querySelectorAll('.cell');
const cell = (col: number, row: number) => board().querySelector(`.cell[data-col="${col}"][data-row="${row}"]`) as HTMLElement;

beforeEach(() => {
  vi.clearAllMocks();
});

describe('boot', () => {
  it('calls api.getState and renders 7 columns and 42 cells', async () => {
    await boot(makeState());
    expect(mockApi.getState).toHaveBeenCalledTimes(1);
    expect(columns()).toHaveLength(7);
    expect(cells()).toHaveLength(42);
  });

  it('shows a helpful error when getState rejects', async () => {
    vi.spyOn(console, 'error').mockImplementation(() => {});
    shell();
    mockApi.getState.mockRejectedValueOnce(new Error('no backend'));
    vi.resetModules();
    await import('./main');
    await flush();
    expect(status().textContent).toBe('Could not reach the backend. Is the app running under Tauri?');
  });
});

describe('clicking a column', () => {
  it('drops in the clicked column (0-indexed) and renders the result', async () => {
    await boot(makeState());
    mockApi.dropDisc.mockResolvedValueOnce(
      makeState({
        board: Array.from({ length: 7 }, (_, col) =>
          Array.from({ length: 6 }, (_, row) => (col === 3 && row === 0 ? 'p1' : 'empty') as CellValue),
        ),
        toMove: 'p2',
        lastMove: { col: 3, row: 0 },
        moveCount: 1,
      }),
    );
    cell(3, 0).click();
    await flush();
    expect(mockApi.dropDisc).toHaveBeenCalledWith(3);
    expect(status().textContent).toBe("Yellow's turn");
  });

  it('does nothing when the column is not in legalMoves', async () => {
    await boot(makeState({ legalMoves: [0, 1, 2, 4, 5, 6] }));
    cell(3, 0).click();
    await flush();
    expect(mockApi.dropDisc).not.toHaveBeenCalled();
  });
});

describe('keyboard', () => {
  it('key 1 drops in column 0', async () => {
    await boot(makeState());
    mockApi.dropDisc.mockResolvedValueOnce(makeState());
    document.dispatchEvent(new KeyboardEvent('keydown', { key: '1' }));
    await flush();
    expect(mockApi.dropDisc).toHaveBeenCalledWith(0);
  });

  it('key 7 drops in column 6', async () => {
    await boot(makeState());
    mockApi.dropDisc.mockResolvedValueOnce(makeState());
    document.dispatchEvent(new KeyboardEvent('keydown', { key: '7' }));
    await flush();
    expect(mockApi.dropDisc).toHaveBeenCalledWith(6);
  });

  it('ignores keys outside 1-7', async () => {
    await boot(makeState());
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'a' }));
    document.dispatchEvent(new KeyboardEvent('keydown', { key: '0' }));
    await flush();
    expect(mockApi.dropDisc).not.toHaveBeenCalled();
  });
});

describe('game over', () => {
  it('locks the board and shows the winner, without allowing further drops', async () => {
    const winningCells = [
      { col: 0, row: 0 },
      { col: 1, row: 0 },
      { col: 2, row: 0 },
      { col: 3, row: 0 },
    ];
    await boot(
      makeState({
        status: 'won',
        winner: 'p1',
        winningCells,
        legalMoves: [],
      }),
    );
    expect(board().getAttribute('data-locked')).toBe('true');
    cell(4, 0).click();
    await flush();
    expect(mockApi.dropDisc).not.toHaveBeenCalled();
    expect(status().textContent).toBe('Red wins!');
    for (const { col, row } of winningCells) {
      expect(cell(col, row).classList.contains('cell--win')).toBe(true);
    }
  });

  it('shows a draw message', async () => {
    await boot(makeState({ status: 'draw', legalMoves: [] }));
    expect(status().textContent).toBe("It's a draw!");
  });
});

describe('new game / play again', () => {
  it('clicking play-again calls api.newGame and renders the result', async () => {
    await boot(makeState({ status: 'draw', legalMoves: [] }));
    mockApi.newGame.mockResolvedValueOnce(makeState());
    document.getElementById('play-again')!.dispatchEvent(new MouseEvent('click', { bubbles: true }));
    await flush();
    expect(mockApi.newGame).toHaveBeenCalledTimes(1);
    expect(status().textContent).toBe("Red's turn");
  });

  it('clicking new-game calls api.newGame', async () => {
    await boot(makeState());
    mockApi.newGame.mockResolvedValueOnce(makeState());
    document.getElementById('new-game')!.dispatchEvent(new MouseEvent('click', { bubbles: true }));
    await flush();
    expect(mockApi.newGame).toHaveBeenCalledTimes(1);
  });
});

describe('hover ghost', () => {
  it('adds cell--ghost to the lowest empty cell of a hovered legal column', async () => {
    await boot(makeState());
    cell(2, 5).dispatchEvent(new MouseEvent('mouseover', { bubbles: true }));
    await flush();
    expect(cell(2, 0).classList.contains('cell--ghost')).toBe(true);
  });
});
