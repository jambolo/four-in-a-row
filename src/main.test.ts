// Behavioural tests for the UI controller (main.ts) against a mocked backend.
// main.ts is DOM-driven glue, so these run in jsdom with `./api` replaced by a
// mock: submitting the form forwards the input to the backend and renders the
// response, and failures degrade to a readable message.

import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { Greeting } from './api';

const mockApi = vi.hoisted(() => ({
  greet: vi.fn<(name: string) => Promise<Greeting>>(),
}));

vi.mock('./api', () => ({ api: mockApi }));

/** Let all pending promise chains (submit -> invoke -> render) settle. */
const flush = () => new Promise<void>((resolve) => setTimeout(resolve, 0));

/** Build the DOM main.ts expects, then (re-)import it fresh. */
async function boot(): Promise<void> {
  document.body.innerHTML = `
    <form id="greet-form">
      <input id="greet-input" />
      <button type="submit">Greet</button>
    </form>
    <p id="greet-output"></p>
  `;
  vi.resetModules();
  await import('./main');
}

const input = () => document.getElementById('greet-input') as HTMLInputElement;
const output = () => document.getElementById('greet-output')!;
const submit = () => (document.getElementById('greet-form') as HTMLFormElement).requestSubmit();

beforeEach(() => {
  vi.clearAllMocks();
});

describe('greeting', () => {
  it('sends the input to the backend and renders the response', async () => {
    await boot();
    mockApi.greet.mockResolvedValueOnce({ message: 'Hello, world, from the Rust core!' });
    input().value = 'world';
    submit();
    await flush();
    expect(mockApi.greet).toHaveBeenCalledWith('world');
    expect(output().textContent).toBe('Hello, world, from the Rust core!');
  });

  it('shows a helpful error when the backend is unreachable', async () => {
    await boot();
    mockApi.greet.mockRejectedValueOnce(new Error('no backend'));
    submit();
    await flush();
    expect(output().textContent).toBe('Could not reach the backend. Is the app running under Tauri?');
  });
});
