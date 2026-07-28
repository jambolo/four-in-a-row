// Behavioural tests for the UI controller (main.ts) against a mocked backend.
// main.ts is DOM-driven glue, so these run in jsdom with `./api` replaced by a
// mock but the real `./view` renderer, exercising the actual rendered board:
// clicks and key presses forward the right column to the backend, hover
// previews a ghost disc, and the status line reflects the returned state
// (including the won/draw/error cases) without main.ts re-deriving any of it.

import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { CellValue, Config, GameState } from './api';

const mockApi = vi.hoisted(() => ({
  newGame: vi.fn<(config: Config) => Promise<GameState>>(),
  dropDisc: vi.fn<(col: number) => Promise<GameState>>(),
  getState: vi.fn<() => Promise<GameState>>(),
  setPaused: vi.fn<(paused: boolean) => Promise<GameState>>(),
  onAiMove: vi.fn<(handler: (next: GameState) => void) => Promise<() => void>>(),
}));

vi.mock('./api', () => ({ api: mockApi }));

/** Let all pending promise chains (event -> invoke -> render) settle. */
const flush = () => new Promise<void>((resolve) => setTimeout(resolve, 0));

const SHELL = `
  <main id="app">
    <h1>Four In A Row</h1>
    <p id="status" class="status" role="status" aria-live="polite" data-thinking="false"></p>
    <div id="board" class="board" data-locked="false" aria-label="Game board"></div>
    <p id="hint" class="hint">Click a column to drop a disc, or press 1-7.</p>
    <div class="settings">
      <label class="setting" for="p1-kind">
        <span class="swatch" data-player="p1"></span>
        <select id="p1-kind" class="select">
          <option value="human">Human</option>
          <option value="computer">Computer</option>
        </select>
      </label>
      <label class="setting" for="p2-kind">
        <span class="swatch" data-player="p2"></span>
        <select id="p2-kind" class="select">
          <option value="human">Human</option>
          <option value="computer">Computer</option>
        </select>
      </label>
      <label class="setting" for="depth">
        <span class="setting__label">Depth</span>
        <input id="depth" class="number" type="number" min="1" max="42" step="1" value="7" />
      </label>
    </div>
    <div class="controls">
      <button id="restart" class="button" type="button">Restart</button>
      <button id="pause" class="button" type="button" hidden>Pause</button>
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
    players: { p1: 'human', p2: 'human' },
    searchDepth: 7,
    thinking: false,
    paused: false,
    generation: 0,
    ...overrides,
  };
}

/** Every handler `main.ts` has passed to `api.onAiMove`, newest last. */
const aiMoveHandlers: Array<(next: GameState) => void> = [];

/** Boot `main.ts` fresh against `initial` as the resolved boot state. */
async function boot(initial: GameState): Promise<void> {
  shell();
  mockApi.getState.mockResolvedValueOnce(initial);
  aiMoveHandlers.length = 0;
  mockApi.onAiMove.mockImplementation((handler) => {
    aiMoveHandlers.push(handler);
    return Promise.resolve(() => {});
  });
  vi.resetModules();
  await import('./main');
  await flush();
}

const board = () => document.getElementById('board') as HTMLElement;
const status = () => document.getElementById('status') as HTMLElement;
const columns = () => board().querySelectorAll('.column');
const cells = () => board().querySelectorAll('.cell');
const cell = (col: number, row: number) => board().querySelector(`.cell[data-col="${col}"][data-row="${row}"]`) as HTMLElement;
const pauseButton = () => document.getElementById('pause') as HTMLButtonElement;
const p1Kind = () => document.getElementById('p1-kind') as HTMLSelectElement;
const p2Kind = () => document.getElementById('p2-kind') as HTMLSelectElement;
const depthInput = () => document.getElementById('depth') as HTMLInputElement;

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
    aiMoveHandlers.length = 0;
    mockApi.onAiMove.mockImplementation((handler) => {
      aiMoveHandlers.push(handler);
      return Promise.resolve(() => {});
    });
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

describe('restart / play again', () => {
  const restart = () => document.getElementById('restart') as HTMLElement;

  it('clicking it after game end calls api.newGame and renders the result', async () => {
    await boot(makeState({ status: 'draw', legalMoves: [] }));
    expect(restart().textContent).toBe('Play Again');
    mockApi.newGame.mockResolvedValueOnce(makeState());
    restart().dispatchEvent(new MouseEvent('click', { bubbles: true }));
    await flush();
    expect(mockApi.newGame).toHaveBeenCalledTimes(1);
    expect(status().textContent).toBe("Red's turn");
    expect(restart().textContent).toBe('Restart');
  });

  it('clicking it mid-game calls api.newGame', async () => {
    await boot(makeState());
    expect(restart().textContent).toBe('Restart');
    mockApi.newGame.mockResolvedValueOnce(makeState());
    restart().dispatchEvent(new MouseEvent('click', { bubbles: true }));
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

describe('settings controls', () => {
  it('boot puts the backend session into the controls', async () => {
    await boot(makeState({ players: { p1: 'human', p2: 'computer' }, searchDepth: 12 }));
    expect(p1Kind().value).toBe('human');
    expect(p2Kind().value).toBe('computer');
    expect(depthInput().value).toBe('12');
  });

  it('restart sends the controls values', async () => {
    await boot(makeState());
    p2Kind().value = 'computer';
    depthInput().value = '9';
    mockApi.newGame.mockResolvedValueOnce(makeState());
    document.getElementById('restart')?.dispatchEvent(new MouseEvent('click', { bubbles: true }));
    await flush();
    expect(mockApi.newGame).toHaveBeenCalledWith({ p1: 'human', p2: 'computer', searchDepth: 9 });
  });

  it('clamps depth above the maximum down to 42', async () => {
    await boot(makeState());
    depthInput().value = '99';
    mockApi.newGame.mockResolvedValueOnce(makeState());
    document.getElementById('restart')?.dispatchEvent(new MouseEvent('click', { bubbles: true }));
    await flush();
    expect(mockApi.newGame).toHaveBeenCalledWith({ p1: 'human', p2: 'human', searchDepth: 42 });
  });

  it('clamps depth below the minimum up to 1', async () => {
    await boot(makeState());
    depthInput().value = '0';
    mockApi.newGame.mockResolvedValueOnce(makeState());
    document.getElementById('restart')?.dispatchEvent(new MouseEvent('click', { bubbles: true }));
    await flush();
    expect(mockApi.newGame).toHaveBeenCalledWith({ p1: 'human', p2: 'human', searchDepth: 1 });
  });
});

describe('guarding human input', () => {
  it('does not drop on a computer turn', async () => {
    await boot(makeState({ players: { p1: 'computer', p2: 'human' } }));
    cell(3, 0).click();
    document.dispatchEvent(new KeyboardEvent('keydown', { key: '1' }));
    await flush();
    expect(mockApi.dropDisc).not.toHaveBeenCalled();
  });

  it('does not drop while a search is thinking', async () => {
    await boot(makeState({ thinking: true }));
    cell(0, 0).click();
    await flush();
    expect(mockApi.dropDisc).not.toHaveBeenCalled();
  });

  it('shows the thinking status text', async () => {
    await boot(makeState({ thinking: true }));
    expect(status().textContent).toBe('Red is thinking…');
  });
});

describe('ai-move subscription', () => {
  it('subscribes exactly once at startup', async () => {
    await boot(makeState());
    expect(mockApi.onAiMove).toHaveBeenCalledTimes(1);
    expect(aiMoveHandlers).toHaveLength(1);
  });

  it('re-renders when the subscribed handler is called', async () => {
    await boot(makeState());
    aiMoveHandlers[0](
      makeState({
        board: Array.from({ length: 7 }, (_, col) =>
          Array.from({ length: 6 }, (_, row) => (col === 3 && row === 0 ? 'p1' : 'empty') as CellValue),
        ),
        moveCount: 1,
        lastMove: { col: 3, row: 0 },
      }),
    );
    await flush();
    expect(cell(3, 0).getAttribute('data-value')).toBe('p1');
  });

  it('keeps a pushed win when the reply to the move before it arrives late', async () => {
    const players = { p1: 'human', p2: 'computer' } as const;
    await boot(makeState({ players }));

    // Hold the reply to the human's move back so the search's pushed state
    // lands first — the ordering that used to erase the computer's win.
    let settle: (next: GameState) => void = () => {};
    mockApi.dropDisc.mockReturnValueOnce(
      new Promise<GameState>((resolve) => {
        settle = resolve;
      }),
    );
    cell(3, 0).click();
    await flush();

    aiMoveHandlers[0](makeState({ players, status: 'won', winner: 'p2', moveCount: 2, legalMoves: [] }));
    await flush();
    settle(makeState({ players, toMove: 'p2', moveCount: 1, thinking: true }));
    await flush();

    expect(status().textContent).toBe('Yellow wins!');
  });

  it('ignores a state pushed by the game that a new game replaced', async () => {
    const players = { p1: 'computer', p2: 'computer' } as const;
    await boot(makeState({ players, generation: 1, moveCount: 5 }));

    mockApi.newGame.mockResolvedValueOnce(makeState({ players, generation: 2 }));
    document.getElementById('restart')?.dispatchEvent(new MouseEvent('click', { bubbles: true }));
    await flush();

    // An event emitted just before the new game started still arrives.
    aiMoveHandlers[0](makeState({ players, generation: 1, status: 'won', winner: 'p1', moveCount: 6 }));
    await flush();

    expect(status().textContent).not.toBe('Red wins!');
  });
});

describe('pause button', () => {
  it('is visible with label Pause when a computer is playing', async () => {
    await boot(makeState({ players: { p1: 'human', p2: 'computer' } }));
    expect(pauseButton().hidden).toBe(false);
    expect(pauseButton().textContent).toBe('Pause');
  });

  it('is hidden when both sides are human', async () => {
    await boot(makeState({ players: { p1: 'human', p2: 'human' } }));
    expect(pauseButton().hidden).toBe(true);
  });

  it('toggles pause via api.setPaused and shows Resume once paused', async () => {
    await boot(makeState({ players: { p1: 'human', p2: 'computer' } }));
    mockApi.setPaused.mockResolvedValueOnce(makeState({ players: { p1: 'human', p2: 'computer' }, paused: true }));
    pauseButton().dispatchEvent(new MouseEvent('click', { bubbles: true }));
    await flush();
    expect(mockApi.setPaused).toHaveBeenCalledWith(true);
    expect(pauseButton().textContent).toBe('Resume');
  });
});
