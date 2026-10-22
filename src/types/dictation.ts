export type DictationErrorCode =
  | 'permission_denied' | 'permission_restricted' | 'device_unavailable'
  | 'unsupported_format' | 'engine_busy' | 'missing_model' | 'capture_failed'
  | 'silence' | 'transcription_failed' | 'cancelled' | 'invalid_input' | 'too_long';
export type DictationPhase = 'idle' | 'requesting_permission' | 'recording' | 'transcribing' | 'review';
export interface DictationIdentity { path: string; session_id: string }
export interface DictationProgress extends DictationIdentity {
  phase: 'requesting_permission' | 'recording' | 'transcribing';
  elapsed_ms: number;
  paused_playback: boolean;
}
export interface DictationReady extends DictationIdentity { text: string }
export interface DictationFailure extends DictationIdentity { code: DictationErrorCode; cancelled: boolean }
