import { invoke } from '@tauri-apps/api/core';

export const dictationService = {
  start: (path: string, sessionId: string): Promise<void> => invoke('dictation_start', { path, sessionId }),
  stop: (sessionId: string): Promise<void> => invoke('dictation_stop', { sessionId }),
  cancel: (sessionId: string): Promise<void> => invoke('dictation_cancel', { sessionId }),
};
