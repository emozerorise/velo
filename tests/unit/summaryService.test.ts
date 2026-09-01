import { beforeEach, describe, expect, it, vi } from 'vitest';

const invoke = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (command: string, args?: unknown) =>
    args === undefined ? invoke(command) : invoke(command, args),
}));

import { summaryService } from '@/services/summaryService';

describe('summary API key IPC', () => {
  beforeEach(() => invoke.mockReset());

  it('sends a key without exposing a read operation', async () => {
    await summaryService.setApiKey('sk-secret');

    expect(invoke).toHaveBeenCalledWith('summary_set_api_key', { key: 'sk-secret' });
    expect('getApiKey' in summaryService).toBe(false);
  });

  it('clears only the saved key', async () => {
    await summaryService.clearApiKey();

    expect(invoke).toHaveBeenCalledWith('summary_clear_api_key');
  });
});
