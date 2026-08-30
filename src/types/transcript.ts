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
  whisper_bin: string | null;
  model_path: string | null;
  ready: boolean;
}
