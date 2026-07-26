// The single door between the UI and the Rust backend.
//
// Every backend interaction goes through this module: nothing else in the
// front end imports from `@tauri-apps/api` directly. That keeps the Rust<->JS
// boundary in one place, so as the game protocol (grid state, moves, win
// results) grows, there is exactly one file to update.

import { invoke } from '@tauri-apps/api/core';

/** One board square on the wire. */
export type CellValue = 'empty' | 'p1' | 'p2';

/** Which player a value refers to. */
export type PlayerId = 'p1' | 'p2';

/** The outcome of the game so far. */
export type GameStatus = 'inProgress' | 'won' | 'draw';

/** A board coordinate: 0-indexed, col left-to-right, row bottom-to-top. */
export interface Cell {
  col: number;
  row: number;
}

/** The full game state, mirrored from the shell's `GameStateDto`. */
export interface GameState {
  /** 7 columns (index 0 = leftmost) of 6 squares (index 0 = bottom row). */
  board: CellValue[][];
  toMove: PlayerId;
  status: GameStatus;
  /** Non-null exactly when `status` is `'won'`. */
  winner: PlayerId | null;
  /** Every cell of the winning line; empty unless `status` is `'won'`. */
  winningCells: Cell[];
  /** Columns that currently accept a disc, ascending. */
  legalMoves: number[];
  /** The most recently placed disc, or null before the first move. */
  lastMove: Cell | null;
  moveCount: number;
}

/** The error codes `dropDisc` can reject with. */
export type DropError = 'invalidColumn' | 'columnFull' | 'gameOver';

export const api = {
  /** Start a fresh game and return the state after the reset. */
  newGame(): Promise<GameState> {
    return invoke<GameState>('new_game');
  },

  /** Drop a disc into `col` (0-indexed). Rejects with a `DropError` string. */
  dropDisc(col: number): Promise<GameState> {
    return invoke<GameState>('drop_disc', { col });
  },

  /** Read the current state without changing it. */
  getState(): Promise<GameState> {
    return invoke<GameState>('get_state');
  },
};
