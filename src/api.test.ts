// Verifies the api module's side of the wire protocol: that each helper
// invokes the right Tauri command with the right argument shape, and passes
// the backend's response through untouched. Uses Tauri's official IPC mocks,
// so no backend is needed. The Rust side of the same contract is pinned down
// in `src-tauri/src/main.rs`'s tests.

import { clearMocks, mockIPC } from '@tauri-apps/api/mocks';
import { afterEach, describe, expect, it } from 'vitest';
import { api } from './api';
import type { GameState } from './api';

afterEach(() => {
  clearMocks();
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
};

describe('api', () => {
  it('newGame invokes the new_game command', async () => {
    const calls: Array<{ cmd: string; args: unknown }> = [];
    mockIPC((cmd, args) => {
      calls.push({ cmd, args });
      return freshState;
    });
    const result = await api.newGame();
    expect(calls.map((call) => call.cmd)).toEqual(['new_game']);
    expect(result).toEqual(freshState);
  });

  it('getState invokes the get_state command', async () => {
    const calls: Array<{ cmd: string; args: unknown }> = [];
    mockIPC((cmd, args) => {
      calls.push({ cmd, args });
      return freshState;
    });
    const result = await api.getState();
    expect(calls.map((call) => call.cmd)).toEqual(['get_state']);
    expect(result).toEqual(freshState);
  });

  it('dropDisc invokes the drop_disc command with the 0-indexed column', async () => {
    const calls: Array<{ cmd: string; args: unknown }> = [];
    const stateAfterMove: GameState = {
      ...freshState,
      board: freshState.board.map((column, index) => (index === 3 ? ['p1', 'empty', 'empty', 'empty', 'empty', 'empty'] : column)),
      toMove: 'p2',
      lastMove: { col: 3, row: 0 },
      moveCount: 1,
    };
    mockIPC((cmd, args) => {
      calls.push({ cmd, args });
      return stateAfterMove;
    });
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
});
