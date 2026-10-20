export type ChatErrorCode =
  | 'not_configured'
  | 'missing_transcript'
  | 'auth'
  | 'connection'
  | 'rate_limit'
  | 'provider'
  | 'context'
  | 'timeout'
  | 'empty'
  | 'incomplete'
  | 'busy'
  | 'cancelled'
  | 'invalid_input';
export interface ChatHistoryMessage {
  role: 'user' | 'assistant';
  content: string;
}
export interface ChatSource {
  start: number;
  end: number;
}
export interface ChatCoverage {
  transcript: 'full' | 'excerpts';
  summary: 'full' | 'truncated' | 'absent';
  omitted_pairs: number;
}
export interface ChatStatus {
  path: string;
  has_transcript: boolean;
  provider: string;
  model: string;
  base_url: string;
  remote: boolean;
  configured: boolean;
  has_key: boolean;
}
export interface ChatStarted {
  path: string;
  request_id: string;
  sources: ChatSource[];
  coverage: ChatCoverage;
}
export interface ChatDelta {
  path: string;
  request_id: string;
  sequence: number;
  text: string;
}
export interface ChatReady extends ChatStarted {
  text: string;
}
export interface ChatError {
  path: string;
  request_id: string;
  code: ChatErrorCode;
  message: string;
  cancelled: boolean;
}
export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant';
  text: string;
  status: 'streaming' | 'complete' | 'failed' | 'cancelled';
  replyTo?: string;
  sources?: ChatSource[];
  coverage?: ChatCoverage;
  clientOmittedPairs?: number;
}
