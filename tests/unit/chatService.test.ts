import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { chatService, newRequestId } from '@/services/chatService';
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

describe('file chat IPC', () => {
  beforeEach(() => vi.clearAllMocks());
  afterEach(() => vi.unstubAllGlobals());
  it('uses camelCase command arguments while preserving snake_case results', async () => {
    const result = {
      path: '/canonical.wav',
      request_id: 'abc',
      sources: [],
      coverage: { transcript: 'full', summary: 'absent', omitted_pairs: 0 },
    };
    vi.mocked(invoke).mockResolvedValueOnce(result);
    expect(await chatService.send('/alias.wav', 'abc', 'Next?', [])).toEqual(
      result,
    );
    expect(invoke).toHaveBeenCalledWith('chat_send', {
      path: '/alias.wav',
      requestId: 'abc',
      message: 'Next?',
      history: [],
    });
    await chatService.cancel('abc');
    expect(invoke).toHaveBeenLastCalledWith('chat_cancel', {
      requestId: 'abc',
    });
  });
  it('status makes only the explicit local IPC request', async () => {
    await chatService.status('/clip.wav');
    expect(invoke).toHaveBeenCalledExactlyOnceWith('chat_status', {
      path: '/clip.wav',
    });
  });
  it('creates a cryptographic ID without randomUUID', () => {
    const id = newRequestId();
    expect(id).toMatch(/^[0-9a-f]{32}$/);
    expect(newRequestId()).not.toBe(id);
  });
  it('works when WebKit exposes getRandomValues but no randomUUID', () => {
    vi.stubGlobal('crypto', {
      getRandomValues: (bytes: Uint8Array) => bytes.fill(0xab),
    });
    expect(newRequestId()).toBe('ab'.repeat(16));
  });
});
