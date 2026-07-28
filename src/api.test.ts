// Verifies the api module's side of the wire protocol: that each helper
// invokes the right Tauri command with the right argument shape, subscribes
// to the right event, and passes the backend's response through untouched.
// Uses Tauri's official IPC mocks for the commands and a module mock for the
// event channel, so no backend is needed. The Rust side of the same contract
// is pinned down in `src-tauri/src/main.rs`'s tests.

import { clearMocks, mockIPC } from '@tauri-apps/api/mocks';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { api } from './api';
import type { Config, GameState } from './api';

const eventMock = vi.hoisted(() => ({
  listen: vi.fn<(name: string, handler: (event: { payload: GameState }) => void) => Promise<() => void>>(),
}));

vi.mock('@tauri-apps/api/event', () => eventMock);

afterEach(() => {
  clearMocks();
  vi.clearAllMocks();
});

const freshState: GameState = {
  board: [
    ['empty', 'empty', 'empty', 'empty', 'empty', 'empty'],
    ['empty', 'empty', 'empty', 'empty', 'empty', 'empty'],
    ['empty', 'empty', 'empty', 'empty', 'empty', 'empty'],
    ['empty', 'empty', 'empty', 'empty', 'empty', 'empty'],
    ['empty', 'empty', 'empty', 'empty', 'empty', 'empty'],
    ['empty', 'empty', 'empty', 'empty', 'empty', 'empty'],
    ['empty', 'empty', 'empty', 'empty', 'empty', 'empty'],
  ],
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
};

const config: Config = { p1: 'human', p2: 'computer', searchDepth: 12 };

/** Record every IPC call and answer them all with `response`. */
function recordIPC(response: unknown): Array<{ cmd: string; args: unknown }> {
  const calls: Array<{ cmd: string; args: unknown }> = [];
  mockIPC((cmd, args) => {
    calls.push({ cmd, args });
    return response;
  });
  return calls;
}

describe('api', () => {
  it('newGame invokes new_game with the config payload', async () => {
    const calls = recordIPC(freshState);
    const result = await api.newGame(config);
    expect(calls).toEqual([{ cmd: 'new_game', args: { config } }]);
    expect(result).toEqual(freshState);
  });

  it('newGame rejects with the backend depth error code', async () => {
    mockIPC(() => {
      throw 'invalidDepth';
    });
    await expect(api.newGame({ ...config, searchDepth: 0 })).rejects.toBe('invalidDepth');
  });

  it('getState invokes the get_state command', async () => {
    const calls = recordIPC(freshState);
    const result = await api.getState();
    expect(calls.map((call) => call.cmd)).toEqual(['get_state']);
    expect(result).toEqual(freshState);
  });

  it('dropDisc invokes the drop_disc command with the 0-indexed column', async () => {
    const stateAfterMove: GameState = {
      ...freshState,
      board: freshState.board.map((column, index) => (index === 3 ? ['p1', 'empty', 'empty', 'empty', 'empty', 'empty'] : column)),
      toMove: 'p2',
      lastMove: { col: 3, row: 0 },
      moveCount: 1,
    };
    const calls = recordIPC(stateAfterMove);
    const result = await api.dropDisc(3);
    expect(calls).toEqual([{ cmd: 'drop_disc', args: { col: 3 } }]);
    expect(result).toEqual(stateAfterMove);
  });

  it('dropDisc rejects with the backend error code', async () => {
    mockIPC(() => {
      throw 'columnFull';
    });
    await expect(api.dropDisc(0)).rejects.toBe('columnFull');
  });

  it('dropDisc rejects with notHumanTurn when it is not a human turn', async () => {
    mockIPC(() => {
      throw 'notHumanTurn';
    });
    await expect(api.dropDisc(0)).rejects.toBe('notHumanTurn');
  });

  it('setPaused invokes set_paused with the flag', async () => {
    const calls = recordIPC({ ...freshState, paused: true });
    const result = await api.setPaused(true);
    expect(calls).toEqual([{ cmd: 'set_paused', args: { paused: true } }]);
    expect(result.paused).toBe(true);
  });

  it('onAiMove subscribes to the ai-move event and forwards the payload', async () => {
    const unlisten = vi.fn();
    let captured: ((event: { payload: GameState }) => void) | undefined;
    eventMock.listen.mockImplementation((_name, handler) => {
      captured = handler;
      return Promise.resolve(unlisten);
    });

    const handler = vi.fn<(state: GameState) => void>();
    const result = await api.onAiMove(handler);

    expect(eventMock.listen).toHaveBeenCalledWith('ai-move', expect.any(Function));
    expect(result).toBe(unlisten);

    captured?.({ payload: freshState });
    expect(handler).toHaveBeenCalledWith(freshState);
  });
});
