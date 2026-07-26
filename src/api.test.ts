// Verifies the api module's side of the wire protocol: that each helper
// invokes the right Tauri command with the right argument shape, and passes
// the backend's response through untouched. Uses Tauri's official IPC mocks,
// so no backend is needed. The Rust side of the same contract is pinned down
// in `src-tauri/src/main.rs`'s tests.

import { clearMocks, mockIPC } from '@tauri-apps/api/mocks';
import { afterEach, describe, expect, it } from 'vitest';
import { api } from './api';

afterEach(() => {
  clearMocks();
});

describe('api', () => {
  it('greet invokes the greet command with the name', async () => {
    const calls: Array<{ cmd: string; args: unknown }> = [];
    mockIPC((cmd, args) => {
      calls.push({ cmd, args });
      return { message: 'Hello, world, from the Rust core!' };
    });
    const result = await api.greet('world');
    expect(calls).toEqual([{ cmd: 'greet', args: { name: 'world' } }]);
    expect(result).toEqual({ message: 'Hello, world, from the Rust core!' });
  });

  it('rejects when the backend rejects', async () => {
    mockIPC(() => {
      throw 'backend error';
    });
    await expect(api.greet('world')).rejects.toBe('backend error');
  });
});
