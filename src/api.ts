// The single door between the UI and the Rust backend.
//
// Every backend interaction goes through this module: nothing else in the
// front end imports from `@tauri-apps/api` directly. That keeps the Rust<->JS
// boundary in one place, so as the game protocol (grid state, moves, win
// results, the computer player's search) grows, there is exactly one file to
// update. The shapes below mirror the shell's serde structs by hand; the Rust
// side pins the same JSON in `src-tauri/src/main.rs`'s tests.

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

/** One board square on the wire. */
export type CellValue = 'empty' | 'p1' | 'p2';

/** Which player a value refers to. */
export type PlayerId = 'p1' | 'p2';

/** The outcome of the game so far. */
export type GameStatus = 'inProgress' | 'won' | 'draw';

/** Who plays a side: the person at the keyboard, or the search. */
export type PlayerKind = 'human' | 'computer';

/** A board coordinate: 0-indexed, col left-to-right, row bottom-to-top. */
export interface Cell {
  col: number;
  row: number;
}

/** Which kind of player controls each side. */
export interface Players {
  p1: PlayerKind;
  p2: PlayerKind;
}

/** The settings a game is started with, mirrored from the shell's `ConfigDto`. */
export interface Config {
  p1: PlayerKind;
  p2: PlayerKind;
  /** Search depth in plies; the backend accepts 1 to 42. */
  searchDepth: number;
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
  /** Who plays each side in this session. */
  players: Players;
  /** The search depth this session was started with. */
  searchDepth: number;
  /** True while a search is running; no human move is accepted then. */
  thinking: boolean;
  /** True while further searches are held off. */
  paused: boolean;
  /**
   * Which game this state belongs to; bumped by the backend on every new game.
   * States arrive over two unordered channels — command replies and `ai-move`
   * events — so `(generation, moveCount)` is what orders them.
   */
  generation: number;
}

/** The error codes `dropDisc` can reject with. */
export type DropError = 'invalidColumn' | 'columnFull' | 'gameOver' | 'notHumanTurn';

/** The error codes `newGame` can reject with. */
export type NewGameError = 'invalidDepth';

export const api = {
  /** Start a fresh game with `config`. Rejects with a `NewGameError` string. */
  newGame(config: Config): Promise<GameState> {
    return invoke<GameState>('new_game', { config });
  },

  /** Drop a disc into `col` (0-indexed). Rejects with a `DropError` string. */
  dropDisc(col: number): Promise<GameState> {
    return invoke<GameState>('drop_disc', { col });
  },

  /** Read the current state without changing it. */
  getState(): Promise<GameState> {
    return invoke<GameState>('get_state');
  },

  /** Hold off further searches, or resume them. */
  setPaused(paused: boolean): Promise<GameState> {
    return invoke<GameState>('set_paused', { paused });
  },

  /**
   * Subscribe to the state the backend pushes after a search plays a move.
   * Resolves to the function that cancels the subscription.
   */
  onAiMove(handler: (state: GameState) => void): Promise<UnlistenFn> {
    return listen<GameState>('ai-move', (event) => handler(event.payload));
  },
};
