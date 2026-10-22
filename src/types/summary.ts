import type { SummaryFormat } from './settings';

export interface Summary {
  /** Older caches predate format selection and use the meeting layout. */
  format?: SummaryFormat;
  path: string;
  model: string;
  language: string;
  /** Composed Markdown; legacy model-written results remain readable. */
  markdown: string;
  omitted_uncertain_claims?: number | null;
  created_at: number;
  /** Segment count of the transcript it was made from, so staleness is detectable. */
  source_segments: number;
}

export interface SummaryStatus {
  provider: string;
  base_url: string;
  model: string;
  configured: boolean;
  /** The request would leave this machine. */
  remote: boolean;
  /** The current destination host has a key in the OS keychain. */
  has_key: boolean;
}

export type SummaryStage = 'mapping' | 'reducing' | 'reviewing'
  | 'extracting_evidence' | 'checking_evidence' | 'composing_summary'
  | 'writing_summary' | 'checking_references';

/** Every step the one-click run passes through, in order. */
export type ChainStage =
  'checking' | 'extracting' | 'transcribing' | SummaryStage;

export interface SummaryProgress {
  path: string;
  stage: SummaryStage;
  done: number;
  total: number;
}

export interface SummaryDelta {
  path: string;
  text: string;
}

export interface SummaryFailure {
  path: string;
  message: string;
}
