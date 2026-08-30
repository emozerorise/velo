export interface TranscriptSegment {
  start: number;
  end: number;
  text: string;
}

export interface Transcript {
  path: string;
  language: string;
  model: string;
  /** The vocabulary prompt this run used. */
  prompt: string;
  created_at: number;
  segments: TranscriptSegment[];
}

export type TranscriptStage = 'extracting' | 'transcribing';

export interface TranscriptProgress {
  path: string;
  stage: TranscriptStage;
  /** 0..1, or -1 while the stage cannot report a fraction. */
  progress: number;
}

export interface TranscriptError {
  path: string;
  message: string;
}

export interface TranscriptEngineStatus {
  /** whisper is compiled in; only the model can be missing. */
  ready: boolean;
  model_path: string | null;
  model_name: string;
  model_bytes: number;
  downloading: boolean;
  /** True when the model is the app's own copy, and so is the app's to delete. */
  model_managed: boolean;
}

export interface ModelProgress {
  received: number;
  total: number;
}
