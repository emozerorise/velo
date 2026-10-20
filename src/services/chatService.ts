import { invoke } from '@tauri-apps/api/core';
import type { ChatHistoryMessage, ChatStarted, ChatStatus } from '@/types/chat';

export function newRequestId(): string {
  const bytes = new Uint8Array(16);
  crypto.getRandomValues(bytes);
  return Array.from(bytes, (byte) => byte.toString(16).padStart(2, '0')).join(
    '',
  );
}

export const chatService = {
  status: (path: string): Promise<ChatStatus> =>
    invoke('chat_status', { path }),
  send: (
    path: string,
    requestId: string,
    message: string,
    history: ChatHistoryMessage[],
  ): Promise<ChatStarted> =>
    invoke('chat_send', { path, requestId, message, history }),
  cancel: (requestId: string): Promise<void> =>
    invoke('chat_cancel', { requestId }),
};
