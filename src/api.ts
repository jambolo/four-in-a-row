// The single door between the UI and the Rust backend.
//
// Every backend interaction goes through this module: nothing else in the
// front end imports from `@tauri-apps/api` directly. That keeps the Rust<->JS
// boundary in one place, so as the game protocol (grid state, moves, win
// results) grows, there is exactly one file to update.
//
// `greet` below is the skeleton's wiring sample, not game logic — it proves
// the round trip works. It will be replaced by the real move/grid-state
// commands once src-core implements the game.

import { invoke } from '@tauri-apps/api/core';

/** A greeting from the Rust core, mirrored from the shell's `Greeting`. */
export interface Greeting {
  message: string;
}

export const api = {
  /** Ask the Rust core for a greeting. */
  greet(name: string): Promise<Greeting> {
    return invoke<Greeting>('greet', { name });
  },
};
