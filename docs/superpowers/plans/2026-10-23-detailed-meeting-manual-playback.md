# Detailed Meeting Summaries and Manual Playback Implementation Plan

> **For agentic workers:** Native, task-by-task execution in this session is the
> preserved preference. Read the spec first. Approved on 2026-10-23, including
> additional DeepSeek calls and real-file acceptance. Do not start subagents
> or publication work.

**Goal:** Make detailed meeting summaries easy to request/read, load media
paused, and simplify the seek controls without changing their five-second action.

**Architecture:** Keep the three summary formats and extend meeting mode with
a persisted detail policy used throughout the existing direct AI pipeline.
Store that policy with results so historical layouts remain correct. Enforce
paused loading at the Rust player boundary, not by racing a frontend Pause.

**Tech Stack:** Existing Vue3, Pinia, TypeScript, Tauri2, Rust, libmpv,
Vitest and Rust loopback/opt-in native tests; no new dependencies.

**Spec:** `docs/superpowers/specs/2026-10-23-detailed-meeting-manual-playback-design.md`

## Global Constraints

- Approved local implementation and real-file AI acceptance; no push/release.
- Preserve all existing dirty changes on `codex/long-summary-recovery`.
- Native sequential execution; no worktree reset or unrelated refactor.
- Keep `meeting`, `key_points`, `custom`; no Evidence Ledger or new retry loop.
- `summary.meeting_detail` is `standard | detailed`, default `standard`.
- Standard/provider limits, source-ID validation and atomic cache safety remain.
- Options persist across launches and apply only to future explicit runs.
- Never overwrite a real source file, raw transcript, old summary or API key.
- UI strings in typed en/th locale files; use current theme/font/modal system.
- Cloud approval is specific to the unchanged voice_214468.aac transcript,
  existing DeepSeek destination/key/model, and these four acceptance variants.

## Review Focus

1. Replacing a playing file cannot emit a short sound before Pause reaches mpv.
2. Detail must survive map/reduction/review, not just the final instruction.
3. A legacy result must not be relabelled detailed when next-run settings change.
4. A large Thai draft and its recovery request must still fit the configured window.
5. A short/narrow modal must retain unsaved text and reachable keyboard actions.

## Files and boundaries

- Native loading: `src-tauri/src/player/{manager,loading,mod}.rs`.
- Detail persistence: `src-tauri/src/storage/settings_store.rs`,
  `src-tauri/src/summary/{types,pipeline}.rs`, `src/types/{settings,summary}.ts`,
  `src/stores/settingsStore.ts`.
- Generation policy: `src-tauri/src/summary/prompt.rs`; no transport rewrite.
- Options/reader: `src/components/transcript/{SummaryPromptEditor,SummaryTab}.vue`,
  `src/utils/summaryMarkdown.ts`, `src/locales/{en,th}.ts`.
- Transport appearance: `src/components/player/ReadingTransport.vue`.
- Tests: existing summary/settings/player suites plus focused files below.

## Task 1 — Load paused and remove visible seek numerals

**Interfaces**

- Preserve IPC `player_load_file(path, startTime?)` and TypeScript
  `loadFile(path: string, startTime?: number): void`.
- Add internal `loading::load_paused(core: &MpvCore, path: &str,
  start_time: Option<f64>) -> Result<()>`, called by `PlayerManager::load_file`.
  This small extraction lets headless tests exercise the production load path
  without a Tauri window. Keep current modern/legacy mpv resume syntax fallback.

- [ ] Write failing native tests in `player/loading.rs` for fresh load, saved
  position and replacing playingA withB: B stays paused; its position does not
  advance while paused; explicit unpause advances it. Use real mpv with null
  video output, muted audio, finite event deadlines and `VELO_TEST_MEDIA`.
- [ ] Add `tests/unit/manualPlayback.test.ts`: importing/selecting/loading
  never invokes `player_play`/`player_toggle_pause`; opening a panel and seeking
  do not invoke Play. Explicit Play still does. Adapt `libraryPlayback.test.ts`
  and `openMedia.test.ts` to the paused-load contract without changing queues.
- [ ] Add `tests/unit/readingTransport.test.ts`: no visible seek numeral, both
  accessible names/tooltips retain5 seconds, buttons dispatch exactly-5/+5,
  compact/full layouts and keyboard activation work. Do not delete the speed
  label or the playback-time digits when asserting numeral removal.
- [ ] Run these tests and record the intended failures before implementation.
- [ ] Extract the load helper, set pause before load, retain resume handling,
  and avoid requesting sleep prevention for paused loading. Remove the two
  numeral spans and add localized titles in ReadingTransport; no new autoplay
  toggle or EOF feature. Preserve existing Play/Pause command behavior.
- [ ] Run focused tests and native fixture checks; verify no audible burst in
  Preview when opening AAC/MP4 and replacing an actively playing recording.

## Task 2 — Persist and generate detailed meeting summaries

**Interfaces**

- Rust `MeetingDetail { Standard, Detailed }`, serde snake_case, default and
  unknown fallbackStandard. Add `SummarySettings.meeting_detail: MeetingDetail`.
- TypeScript `MeetingDetail = 'standard' | 'detailed'`,
  `SummarySettings.meeting_detail: MeetingDetail`,
  `Summary.meeting_detail?: MeetingDetail`; Rust results default old caches.
- Extend `saveSummaryOptions`'s existing argument object with
  `meeting_detail?: MeetingDetail`; preserve serialized save-before-commit.
- `prompt::detailed_meeting(settings: &SummarySettings) -> bool` is true only
  for meeting+detailed. Existing prompt public signatures remain unchanged.

- [ ] Write failing settings/cache tests: missing/unknown→standard, detailed
  roundtrip, unrelated values unchanged, failed save leaves saved options
  unchanged, old summary bytes remain untouched. Extend `summaryOptions.test.ts`.
- [ ] Write failing request-boundary tests in `summary_long.rs`: detailed policy
  reaches map, reduction, final and review/repair; key points/custom do not gain
  meeting headings when the stored meeting detail isDetailed. Exercise the
  actual emitted source/instruction requests, not a mock-only quality oracle.
- [ ] Add synthetic1/2/3-hour detailed cases: all source rows included, original
  clocks retained, initial/recovery sizes fit, cancellation/limits preserve
  prior cache, and loops remain bounded. Keep the existing three-format matrix.
- [ ] Run targeted tests RED. Implement enum/defaults, atomic settings fields,
  saved-result metadata and the spec's localized detailed prompt at every
  generation stage. Leave output caps and repair counts unchanged.
- [ ] Run `cargo test --test summary_long --test summary_direct` plus relevant
  lib tests and `npx vitest run tests/unit/summaryOptions.test.ts`. Confirm GREEN.

## Task 3 — Simplify summary options and expose detailed reading

**Interfaces**

- Preserve `SummaryPromptEditor` props `{ open: boolean; initialDraft?: string }`
  and `close` event. Save format, meeting_detail and instructions together.
- `groupSummarySections(markdown, format, sources?, meetingDetail = 'standard')`
  keeps standard behavior by default; detailed meeting keeps original section
  order. SummaryTab passes the cached result's detail, never next-run settings.
- Reader display: meeting+detailed shows “สรุปประชุม · ละเอียด”; discussion is
  initially expanded. Existing key-point/custom render contracts stay intact.

- [ ] Write failing controls tests for three format cards, meeting-only detail
  radio group, static expected-output outline, advanced editor disclosure and
  immediate editor focus for custom/chat-proposed prompts.
- [ ] Test that switching format/detail does not overwrite text; applying an
  example over nonempty text requires confirmation; Cancel does not save;
  Save errors preserve drafts; no options interaction calls summary/chat APIs.
- [ ] Add keyboard tests: radio arrows, Tab wrap, Shift-Tab, Escape and focus
  restoration. Opening a hidden advanced field must precede focusing it.
- [ ] Add reading tests: savedDetailed shows discussion expanded; savedStandard
  stays folded/outcome-first even with Detailed selected for the next run;
  legacy cache, copy and show/hide timestamps stay compatible.
- [ ] Run focused tests RED, implement the spec using current BaseModal wide
  layout and a local scrollable-body/footer arrangement. Avoid changing all
  shared modals solely for this screen. Add typed en/th copy.
- [ ] Run `npx vitest run tests/unit/summaryFormatControls.test.ts
  tests/unit/summaryReading.test.ts tests/unit/summaryMarkdown.test.ts` and
  options/player tests. Verify both themes, long Thai text, narrow760px and
  short550px windows, zoom200%, keyboard-only navigation in native Preview.

## Task 4 — Integrated acceptance and handoff

- [ ] Run all six gates, not only focused tests:

  ```bash
  cd src-tauri
  cargo fmt
  cargo fmt --check
  cargo clippy --all-targets -- -D warnings
  cargo test
  cd ..
  npm run typecheck
  npm run lint
  npx vitest run
  git diff --check
  ```

  Environment-gated audio/provider cases are not counted as real passes when
  skipped. Run native file fixtures explicitly with `VELO_TEST_MEDIA`.
- [ ] Build an isolated updater-disabled Preview bundle using the existing
  Preview configuration procedure. No production bundle/data replacement.
- [ ] After explicit API execution approval, use the unchanged cached transcript
  of `voice_214468.aac`, the existing Keychain key/model/context and new isolated
  output directories. Run standard meeting, detailed meeting, key points and
  the existing custom prompt on the same frozen build. Do not copy source audio
  to a provider or silently alter user settings to obtain a pass.
- [ ] Compare standard vs detailed against original excerpts: detailed retains
  material reasons/options/conditions, not just more words. Check explicit
  agreements vs proposals, assignments/dates, later corrections and unclear
  ASR terms. A duplicated clock alone does not prove the adjacent claim.
- [ ] Require complete terminal responses and correct format/reference checks.
  Any failure retains the old cache. Stop to diagnose; no unbounded paid retries
  or weakened validator, and no claim of perfect AI/transcription fidelity.
- [ ] Verify native opening/import/drop/library/resume remains paused, including
  replacement while playing. Click an unambiguous source clock: seek works
  while remaining paused. Test explicit Play/Space separately. Restore Preview
  to a paused reader state without changing production data.
- [ ] Update `PLAN.md`, `design-qa.md` and Thai `CHANGELOG.md` with actual results,
  failures/skips and limitations. Open Preview for the user only after acceptance;
  no push or release. Review this plan before implementation begins.
