# File-Scoped Content Chat Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add grounded, streaming discussion of the current file and reviewable summary-prompt drafts inside the summary reader.

**Architecture:** Rust commands load transcript/summary snapshots and credentials, then give an independent chat module owned inputs. A shared resource lease serializes chat with summarization; keyed cancellation/events prevent old turns reaching a new file. Pinia owns in-memory conversation presentation, never secrets or HTTP.

**Tech Stack:** Existing Vue/Pinia/Tauri/Rust, reqwest, tokio sync, existing provider dialects and OS credentials; no new AI provider, tokenizer, vector database or frontend HTTP library.

**Spec:** [Approved design](../specs/2026-10-21-reading-tools-chat-design.md).

## Global Constraints

- Plan 2 of 3; depends on [workspace](2026-10-21-summary-workspace.md) interfaces; followed by [dictation](2026-10-21-composer-dictation.md). Approved for native task-by-task execution, preserving the existing branch and work.
- Preserve dirty work, the current 0.8.0 version, existing summary tests and all provider-specific thinking-off behavior.
- UI bilingual; English source/typed Thai strings; English commits/comments, Thai changelog. Use the approved inline composer, not a separate right column.
- Chat starts closed; only manual Send invokes inference. Opening chat may read local status/cache, never probe/download/infer. Transcript required, summary optional.
- One in-memory conversation per canonical media path for this app session; no disk chat archive, media upload, raw HTML, tools or external-link execution.
- User message at most 8 KiB; at most six complete history pairs sent; answer at most 2048 tokens. Count complete serialized prompt bytes with the existing conservative heuristic, not Thai character count.
- Show destination before Send, never return/log credentials or sensitive source text, never fall back to another host. No paid/live call in automated QA.
- Use exact neutral UI copy from spec; saved prompt replaces global `summary.instructions` only after review and never regenerates automatically.
- All six repository checks plus bundle/version/release-script checks at the final plan-set gate. No push, PR, release or installed-app mutation.

## Review Focus

1. A server never sends its first byte: Stop closes the request promptly instead of waiting for its timeout (Task 1).
2. HTTP redirect/error payload echoes credentials or transcript text: none reaches a second host, webview or logs (Task 1).
3. Long Thai/no-space input or one oversized segment: bounded, UTF-8-valid excerpts with honest coverage (Task 2).
4. Rapid file/provider changes, retries and repeated events: no duplicate user turn or cross-file answer (Tasks 3 and 4).
5. A fabricated citation or multiple prompt blocks: safe readable text, no unsafe seek or silent settings write (Task 4).

---

## File Structure

- Create `src-tauri/src/jobs.rs`: resource leases and cancellable request signal, independent of feature modules; enable tokio `time` explicitly in Cargo.toml for bounded cancellation tests/timers and update Cargo.lock if needed.
- Create `src-tauri/src/chat/{mod,types,context,prompt}.rs`: bounded source selection, job lifecycle and prompts.
- Create `src-tauri/src/commands/chat.rs`; modify command exports/lib state registration and summary/transcript start boundaries to pass resource leases.
- Extend `src-tauri/src/summary/transport.rs` with conversation requests while retaining `ChatRequest` and `stream_chat` compatibility.
- Create `src/{types/chat.ts,services/chatService.ts,stores/chatStore.ts,utils/chatContent.ts}`.
- Create `src/components/chat/{InlineChat,ChatMessage,ChatComposer}.vue`; wire the workspace chat slot into SummaryTab/TranscriptPanel and lifecycle into App.
- Extend `SummaryPromptEditor.vue` with an optional initial draft; add locales and optional mic action slot for plan 3.
- Create `src-tauri/tests/chat_pipeline.rs`, `tests/unit/{chatService,chatStore,chatContent,inlineChat}.test.ts`; extend existing summary service/pipeline/prompt tests.

### Task 1: Resource leases and safe reusable streaming

**Files:** jobs module, transport, lib, summary/transcript commands and job signatures; `summary_pipeline.rs`, `chat_pipeline.rs`.

**Interfaces:**

- `JobResource::{ChatModel, Whisper}` and `JobGate::try_acquire(resource, owner: &str) -> Result<JobLease>`; a lease releases its own resource on Drop, including spawn failures, never a newer owner's lease.
- `JobCancel::new()`, `cancel(&self)`, `cancelled(&self) -> impl Future<Output=()>`, `flag(&self) -> Arc<AtomicBool>`; flag supports Whisper's existing callback, notification interrupts async waits.
- Summary `start(app: AppHandle, input: JobInput, lease: JobLease) -> Result<()>`; transcript `start(app: AppHandle, media_path: String, language: String, prompt: String, lease: JobLease) -> Result<()>`. Commands acquire before start, tasks own the lease to termination. Model removal also acquires the Whisper resource.
- Keep existing summary `ChatRequest` and `stream_chat` callers source-compatible. Add `ConversationRequest { model: String, messages: Vec<WireMessage>, context_tokens: u32, max_tokens: u32 }`, `WireMessage { role: WireRole, content: String }`, roles System/User/Assistant.
- `stream_conversation(dialect, base_url, api_key, req: &ConversationRequest, cancel: &JobCancel, on_text, on_thinking) -> Result<String>` uses the shared wire machinery, requiring explicit terminal completion. No silent EOF completion for chat.

- [ ] **Step 1: Add failing tests:** `model_lease_excludes_summary_and_chat`, `failed_spawn_releases_own_lease`, `whisper_lease_excludes_file_and_dictation`, `cancel_before_headers_finishes_within_250ms`, `cancel_between_frames_finishes_within_250ms`, `redirect_does_not_forward_bearer`, `error_body_is_not_user_visible`, `terminal_frame_keeps_its_final_text`. For redirects, two loopback mock servers assert the second receives zero requests. For a stalled server, signal cancellation after accept and use an outer timeout independent of provider response.
- [ ] **Step 2: Run red:** `cargo test jobs::` and `cargo test --test chat_pipeline` in `src-tauri`. Confirm missing/new behavior fails rather than a missing native dependency.
- [ ] **Step 3: Implement the interfaces.** Add a body adapter replacing the dialect-built `messages` array, preserving its model/options/thinking flags. Share parsing/UTF-8 buffering and return final text from terminal frames before finishing. Select cancellation against connection/header reads and every stream read. Disable HTTP redirects; sanitize status messages without raw response bodies/URLs carrying credentials. Bound an accumulated line to 256 KiB and answer text to 64 KiB, rejecting overflows. Existing summary output limits stay unchanged. Apply the same no-redirect/sanitized-error client behavior to discovery without adding automatic probes. Remove sensitive path/text logging in the touched summary/transcript job boundaries and clear their active job on spawn failure as well as releasing its lease.
- [ ] **Step 4: Run green:** those tests and `cargo test --test summary_pipeline`; verify old request bodies, reasoning suppression and split-Thai behavior remain covered for all providers. No external host is used.
- [ ] **Step 5: Commit** task-owned paths with `feat: add cancellable conversation transport and model leases`; stage only inspected changes.

### Task 2: Bounded source grounding and coverage

**Files:** chat types/context/prompt modules with unit tests; no UI or network in these pure tests.

**Interfaces:**

- `ChatHistoryMessage { role: UserOrAssistant, content: String }`; validate alternating complete pairs, at most 12 messages, user <=8192 bytes, assistant <=65536 bytes. Reject system roles, invalid order or oversize inputs before HTTP.
- `ChatSource { start: f64, end: f64 }`, `ChatCoverage { transcript: FullOrExcerpts, summary: FullTruncatedOrAbsent, omitted_pairs: usize }` with lowercase serialized enum values.
- `ChatContext { messages: Vec<WireMessage>, sources: Vec<ChatSource>, coverage: ChatCoverage }`.
- `build_context(segments: &[TranscriptSegment], summary: Option<&str>, history: &[ChatHistoryMessage], message: &str, context_tokens: u32, language: &str) -> Result<ChatContext>`; all source intervals finite, ordered and nonnegative.

- [ ] **Step 1: Write failing tests** `full_transcript_when_it_fits`, `thai_bytes_not_characters`, `huge_single_segment_stays_valid_utf8`, `six_pair_limit_and_oldest_eviction`, `tiny_context_returns_error_without_underflow`, `explicit_time_range_prioritized`, `no_overlap_duplication`, `source_instruction_not_system_message`. Pin budget with:

```rust
let context = build_context(&segments, None, &history, "งานถัดไป", 8192, "th")?;
assert!(serde_json::to_vec(&context.messages)?.len() <= (8192 - 4096) * 3);
assert!(context.sources.iter().all(|s| s.start.is_finite() && s.end >= s.start));
```

- [ ] **Step 2: Run red:** `cargo test chat::context::` and `cargo test chat::prompt::`. Expected missing implementation or failing boundary assertions.
- [ ] **Step 3: Implement deterministic selection.** Reserve 4096 tokens (2048 answer plus safety); require positive remaining budget and measure the fully serialized message array. Evict old complete pairs first. Prefer full transcript; otherwise score segment overlap using lowercase Latin words/Thai character trigrams from current question and latest user context, limited to 512 unique query terms. Prioritize explicit mm:ss/hh:mm:ss ranges, then highest score (ties by source order), include one neighbor each side, deduplicate and emit chronologically. Split only oversized source text at UTF-8 boundaries, retain source intervals, and budget summary explicitly; label any truncation. When no lexical match exists, use available summary plus bounded beginning/end excerpts and disclose the limitation. System instructions distinguish quoted source data from commands, prohibit invented facts, require citations/limits, and use one fenced `prompt` block only when drafting is requested. Do not add an inference/map step or claim excerpt coverage is complete.
- [ ] **Step 4: Run green:** those unit filters and full `cargo test --test chat_pipeline`. Include silence/empty transcript and malicious quoted instructions; assertions test constructed role/context boundaries, not the model's ability to obey them.
- [ ] **Step 5: Commit** with `feat: ground file chat in bounded transcript context`.

### Task 3: File identity, streaming command lifecycle and isolation

**Files:** chat module/state/commands, command registration and typed service/types; `chat_pipeline.rs`, `chatService.test.ts`.

**Interfaces:**

- `chat_status(path) -> ChatStatus { path, has_transcript, provider, model, base_url, remote, configured, has_key }`: canonical path and local configuration/cache reads only; never reads key into response or probes a provider. Opening no-file chat does not call this command.
- `chat_send(path: String, request_id: String, message: String, history: Vec<ChatHistoryMessage>) -> ChatStarted { path, request_id, sources, coverage }`; validate IDs to 1..128 ASCII bytes and reject an active duplicate rather than starting it twice.
- `chat_cancel(request_id: String) -> Result<()>`: matching request only; idempotent if absent.
- Events `velo://chat-delta {path,request_id,sequence,text}`, `velo://chat-ready {path,request_id,text,sources,coverage}`, `velo://chat-error {path,request_id,code,message,cancelled}`. Sequence starts at 1 and increases. Codes are stable enum strings, display text localized in Vue.
- Error codes: `not_configured`, `missing_transcript`, `auth`, `connection`, `rate_limit`, `provider`, `context`, `timeout`, `empty`, `incomplete`, `busy`, `cancelled`, `invalid_input`. `chatService.status(path)`, `send(path, requestId, message, history)` and `cancel(requestId)` return the corresponding IPC result types. `newRequestId(): string` uses 16 `crypto.getRandomValues` bytes encoded as hex, not `crypto.randomUUID` unavailable on older supported WebKit; retry uses a fresh request ID while retaining the user-message ID.
- Tauri command arguments use its default camelCase (`requestId`); event/returned serde payload fields stay snake_case (`request_id`). Service tests must assert that distinction; do not send an unrecognized argument spelling.
- `run_turn(input: ChatInput, cancel: JobCancel, emit: Arc<dyn Fn(ChatEvent) + Send + Sync>) -> Result<ChatAnswer>` is integration-testable without a real Tauri window; `ChatInput` owns provider settings, source snapshots, validated history and optional key, never borrows stores.

- [ ] **Step 1: Add failing tests** `canonical_aliases_share_identity`, `switch_cancel_drops_old_turn`, `late_completion_cannot_release_new_job`, `no_transcript_rejects_before_http`, `summary_without_cloud_key_remains_readable`, `status_does_not_probe`, `chat_never_writes_summary_or_transcript_cache`. Test the real runner against each mock dialect; its final metadata must match chosen source intervals.
- [ ] **Step 2: Run red:** `cargo test --test chat_pipeline` and `npx vitest run tests/unit/chatService.test.ts`.
- [ ] **Step 3: Implement commands/state/service.** Command layer loads snapshots, resolves response language against transcript, obtains credentials through the existing write-only summary credential access, builds context and acquires ChatModel lease. Register state/listeners at app startup, not inside a mountable chat panel. Clear active state conditionally by request ID after worker termination. Cancellation/error preserves incomplete answer as incomplete, never emits ready. Suppress sensitive logs and keep source/path identifiers out of user-facing failure strings.
- [ ] **Step 4: Run green:** both commands, `cargo test --test summary_pipeline`, typecheck and lint. Assert all mock HTTP requests are text-only and none contains an audio/base64 payload.
- [ ] **Step 5: Commit** with `feat: add isolated file chat commands and events`.

### Task 4: Inline conversation, safe sources and prompt handoff

**Files:** chat frontend files, App lifecycle, workspace/SummaryTab integration, SummaryPromptEditor, locales; chatStore/chatContent/inlineChat/summaryOptions tests.

**Interfaces:**

- `ChatMessage { id: string; role: 'user' | 'assistant'; text: string; status: 'streaming' | 'complete' | 'failed' | 'cancelled' }`; `useChatStore()` exposes `messages: ChatMessage[]` for the current canonical path, `draft: string`, `isRunning`, `errorCode`, `coverage`, `sources`; methods `prepare(path): Promise<void>`, `send(): Promise<void>`, `cancel(): Promise<void>`, `retry(): Promise<void>`, `setDraft(text): void`, `appendDraft(text): boolean`, `initListeners/cleanupListeners`. `setDraft` permits editing overlong input while showing its validation error; `appendDraft` returns false without mutation if newline + text would exceed 8192 bytes, never truncates text silently.
- `ChatComposer` props `{ modelValue: string; disabled: boolean; sending: boolean; remoteHost: string | null }`, emits `update:modelValue`, `send`, `cancel`; named `dictation` slot reserved for plan 3. No fake working mic before that plan.
- `parseChatText(text: string, sources: ChatSource[], showTimestamps: boolean): ChatNode[]` yields text/paragraph/list/code/citation nodes; renders through Vue escaped nodes, no raw HTML. Citations are enabled only if time lies in a supplied source interval; use existing player seek.
- `promptDraft(text: string): string | null`: exactly one closed fenced `prompt` block <=8192 bytes; ambiguous/multiple/unclosed blocks produce null.
- `SummaryPromptEditor` optional `initialDraft?: string`; on open snapshot that value or saved instructions. It never overwrites an already-open edited draft. Saving keeps existing all-files scope warning and requires an explicit submit.

- [ ] **Step 1: Add failing tests** `duplicate_delta_ignored`, `sequence_gap_waits_for_authoritative_final`, `old_file_event_not_current_answer`, `retry_keeps_one_user_message`, `provider_change_cancels_inflight`, `hide_reopen_keeps_thread`, `starter_cannot_silently_replace_draft`, `html_and_external_links_are_text`, `unsupported_citation_not_seekable`, `two_prompt_blocks_not_auto_applied`, `save_prompt_does_not_generate`. Example:

```ts
await chat.send();
receiveDelta({ path, request_id: id, sequence: 1, text: "คำตอบ" });
receiveDelta({ path, request_id: id, sequence: 1, text: "คำตอบ" });
expect(chat.messages.at(-1)?.text).toBe("คำตอบ");
expect(summaryService.generate).not.toHaveBeenCalled();
```

Define test-local `receiveDelta(payload)` through the mocked Tauri `listen` callback registered by `chat.initListeners()`; do not bypass the store by editing rendered text. Mount real InlineChat/Composer handlers using plan 1's helper; test draft typing, click, IME composition, Shift+Enter newline, Enter Send only outside composition, and Send disabled for empty/missing context/busy summary. Add `eight_complete_pairs_report_two_client_omissions` in addition to backend budget eviction tests.

- [ ] **Step 2: Run red:** `npx vitest run tests/unit/chatStore.test.ts tests/unit/chatContent.test.ts tests/unit/inlineChat.test.ts tests/unit/summaryOptions.test.ts`.
- [ ] **Step 3: Implement the interfaces and approved UI.** File/provider watchers cancel the right request; UI hiding does not. History sent is six most recent complete pairs, while displayed in-memory history retains partial error context; display omitted history as client-trimmed pair count plus server `coverage.omitted_pairs`. Ignore duplicate/out-of-order deltas; if sequence is missing, show the partial safely and replace it with authoritative ready text (never append final twice). Display session-only retention and full/excerpt/omitted-history labels. Auto-scroll only when already at bottom; otherwise expose a jump-to-latest action. Shared timestamp preference hides source chips; plain text stays readable. Prompt action opens a review draft, not settings-save or regenerate. Keep ordinary composer usable without a mic/model for dictation.
- [ ] **Step 4: Run green:** those tests, all summary tests, typecheck/lint. Use synthetic mock transport for native UI QA; do not put a real key in frontend fixtures or assert live model factual accuracy.
- [ ] **Step 5: Commit** with `feat: add grounded inline content conversations`.

### Task 5: Part-2 regression and visual gate

**Files:** the above tests/components and `design-qa.md`; test-only fixtures for injected responses, never hardcoded production conversations.

**Interfaces:** consume the actual workspace/chat services; no new public API.

- [ ] **Step 1: Add failing regressions** for a cached transcript with no summary, removed speech model with text chat working, failed provider after successful transcript, 640x400 IME editing and reopening a hidden streaming chat. Include at least one question about non-meeting content so neutral labels persist without changing the summary's five-section backend contract.
- [ ] **Step 2: Run red:** `npx vitest run tests/unit/inlineChat.test.ts tests/unit/chatStore.test.ts`; confirm targeted failures.
- [ ] **Step 3: Fix only those regression gaps**, preserving summaries/transcripts, keyboard ownership and source data.
- [ ] **Step 4: Run** the six repository checks, `npm run version:check`, `node --test tests/release/*.check.mjs` and a macOS app bundle build. On the isolated app inspect welcome -> open synthetic file -> summary -> chat -> retry -> prompt review/cancel/save -> transcript -> hidden player. Compare capture with approved inline mock at the same viewport; record mocks/live limitations and skips in QA report.
- [ ] **Step 5: Commit** with `test: verify content chat isolation and prompt review`; no push/release. Deliverable is usable text chat and prompt refinement; mic remains explicitly unavailable until plan 3.

## Handoff

Read the shared spec and all three plans before approval/execution-method selection. The plan is testable without a real AI account; paid/live quality evaluation needs separate user-approved input and a deliberate request.
