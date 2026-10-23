# Complete-Part Long Summaries Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:executing-plans to implement this plan task-by-task. Native
> sequential execution in this session is the user's preserved choice, not
> subagent-driven development. Read the approved spec and this whole plan first.

**Goal:** Finish long summaries as complete source-owned parts, preserve original
citations and cross-part conditions, and save only a fully assembled result.

**Architecture:** Reuse the direct transport/reference/review rules behind one
per-job request budget. Write from original source intervals, review distant
source against bounded candidate batches, replace affected parts once, and
assemble locally after a second stability sweep. No Evidence Ledger or global
full-answer model rewrite.

**Tech Stack:** Existing Rust/Tauri2, Vue3/Pinia/TypeScript, serde_json, Rust
loopback tests and Vitest; no new dependencies.

**Spec:** `docs/superpowers/specs/2026-10-23-complete-part-summary-design.md`

**Execution boundary:** Approved for Native sequential implementation on
2026-10-23, including additional API calls on the unchanged voice_214468.aac
after local gates and Preview after acceptance. No automatic commit, push,
release, version change or premature Preview/cache replacement.

## Global Constraints

- Preserve the dirty `codex/long-summary-recovery` checkout and all prior work.
  No reset, stash, automatic commit, unrelated refactor or task delegation.
- Every original row has exactly one owner; no text truncation, skipped rows,
  renumbered source IDs or generated notes substituting for original body input.
- Soft initial source target: `24 * 1024` UTF-8 bytes; exact mandatory request
  fit determines the actual partition. Up to two whole context rows per side.
- At most `32` terminal owning parts (including empty parts), split depth `3`
  per initial interval, and `128` physical provider requests for the whole job.
- Exactly two complete cross-part sweeps, at most one replacement per flagged
  part, no post-sweep splitting, no third sweep or convergence loop.
- Routing/layout: maximum `1024` output tokens, no retry on malformed,
  incomplete or output-limit replies; at most `32` routed IDs per batch.
- Keep existing prose caps/recovery: DeepSeek initial final cap `4096`, one
  fresh output-limit retry up to `max(initial, min(8192, context_tokens / 4))`.
  Ollama/OpenAI caps stay unchanged. Never continue or reuse truncated output.
- Keep single supported citation repair and original-source prose review;
  unknown IDs, gapped ranges and model clocks remain invalid.
- Sequential HTTP only. All attempts, including failures/recovery/metadata,
  count against the same job budget. Never reset the counter on fallback.
- Keep three formats and cached `meeting_detail`; custom has no compulsory
  meeting headings. Do not rewrite the saved user prompt or migrate caches.
- Parts, routing and coverage exist only in memory. No persisted fact ledger,
  claims API, partial cache, new renderer or credential exposure.
- No success/summary-ready event or chat summary-context replacement before
  final validation and atomic save. Failure/cancel preserves the old result.
- UI strings use typed `src/locales/en.ts` and `th.ts`; retain font/theme,
  old-chat separation, timestamp toggle and no-autoplay behavior.
- Only new isolated output directories may receive live artifacts. Never alter
  the audio, transcript, production settings/key/cache or updater publication.

## Review Focus

1. Thai text with embedded newlines, fake source IDs and JSON escapes must keep
   exact row identity and fit every request, including recovery (Tasks 1–2).
2. A later statement about a different subject, or a fact absent from one
   window, must not erase a valid earlier statement (Task 3).
3. An empty/filler source part and a valid empty routing list differ from a
   broken response; none may silently remove source coverage (Tasks 2–4).
4. An old-window async event or cancellation racing a completed save must not
   corrupt the active file or display a false completion/cancellation (Tasks 4–5).
5. Custom quotas and repeated/nested headings must not multiply across parts,
   detach conditions or falsely promise layout compliance (Tasks 2, 4–5).

## Files and responsibilities

- Create `src-tauri/src/summary/generation.rs`: measured request builders,
  shared physical-call budget, existing prose completion/review and controls.
- Create `src-tauri/src/summary/parts.rs`: stable ownership, immutable quoted
  source packets, boundary context, bounded splits and coverage checks.
- Create `src-tauri/src/summary/layout.rs`: validated layout-only contract,
  Markdown part structure and deterministic format-preserving assembly.
- Create `src-tauri/src/summary/part_writer.rs`: complete final-form parts,
  initial split queue and one-shot whole-part replacement.
- Create `src-tauri/src/summary/consistency.rs`: measured candidate batches,
  source-window coverage, routing parse and the two sweeps.
- Create `src-tauri/src/summary/multipart.rs`: long-job orchestration and
  small overview composition; returns Markdown, never writes the cache.
- Modify `summary/{mod,pipeline,prompt,validation,types}.rs`: register private
  modules, retain the public runner, reusable policy and event documentation.
  Preserve `store.rs` and transport dialect behavior unless a failing test
  identifies an actual atomic-save/terminal-finish defect.
- Test pure/private helpers beside their modules. Add
  `src-tauri/tests/summary_parts.rs` and `summary_consistency.rs`; extend
  `summary_long.rs`, `summary_direct.rs`, `summary_pipeline.rs` and
  `support/summary_model.rs` for production-runner loopback assertions.
  Register that loopback double as a `#[cfg(test)]` private module in
  `summary/mod.rs` for sibling unit tests as well; do not expose helpers to IPC.
- Modify `src/types/summary.ts`, `src/stores/summaryStore.ts`,
  `src/components/transcript/SummaryTab.vue`, `src/locales/{en,th}.ts`;
  add `tests/unit/summaryPartProgress.test.ts`, extend `summaryReading.test.ts`
  and `summaryProviders.test.ts`. Reuse the existing event-mock test patterns.
- Final acceptance: `summary_direct_live.rs`, new
  `scripts/check-summary-parts-acceptance.mjs` and
  `tests/release/summary-parts-acceptance.check.mjs`; update `PLAN.md` and
  `design-qa.md` with observed outcomes, not predicted passes.

## Task 1 — Source ownership and one bounded generation session

**Interfaces** (all new Rust interfaces are `pub(super)`, not public IPC):

- `parts::SourceSnapshot::new(rows: &[TranscriptSegment]) -> Result<Self>`;
  store unchanged rows/quoted text, validate the current timeline rules once.
- `SourcePart { id: PartId, owned: Range<usize>, context: Vec<Range<usize>>,
  depth: u8 }`, with `PartId(String)` deriving order/equality.
  Root IDs are `P000001` etc.; children append `.0` / `.1`, never source IDs.
- `SourcePacket { rows: Vec<usize>, text: String, labels: BTreeSet<String> }`;
  `SourceSnapshot::packet(ranges: &[Range<usize>]) -> Result<SourcePacket>`
  sorts/unions complete rows and derives allowed IDs only from those rows.
- `parts::partition(snapshot: &SourceSnapshot,
  fits: impl Fn(&SourcePacket) -> bool) -> Result<Vec<SourcePart>>`;
  `parts::split(snapshot: &SourceSnapshot, part: &SourcePart,
  terminal_count: usize, fits: impl Fn(&SourcePacket) -> bool)
  -> Result<[SourcePart; 2]>`;
  `parts::validate_coverage(parts: &[SourcePart], row_count: usize) -> Result<()>`.
- `generation::Generation<'a>::new(input: &'a JobInput,
  cancel: &'a AtomicBool) -> Self`;
  `input(&self) -> &JobInput` exposes only the frozen job to sibling helpers;
  `requests(&self) -> usize`; `reserve_future(&mut self, minimum: usize)
  -> Result<()>` reserves known mandatory work outside the active operation.
- `Generation::fits_cited(&self, system: &str, source: &str, notes: bool,
  labels: &BTreeSet<String>) -> bool` shares actual mandatory builders.
- `Generation::cited(&mut self, system: String, source: String, notes: bool,
  labels: &BTreeSet<String>, allow_empty: bool) -> Result<Option<String>>`;
  `None` is allowed only for the exact reviewed no-content marker.
- `Generation::control(&mut self, request: ChatRequest, phase: &str)
  -> Result<String>` checks `max_tokens <= 1024`, fits and explicit finish,
  performs one physical call and never content-retries.

- [ ] **Write RED unit tests** beside `parts.rs` and `generation.rs`. Pin
  `SOURCE_TARGET_BYTES == 24 * 1024`, `MAX_PARTS == 32`,
  `MAX_SPLIT_DEPTH == 3`, `MAX_REQUESTS == 128`, `CONTROL_TOKENS == 1024`.
  For 93 rows assert the flattened owned ranges equal `(0..93).collect()`;
  splitting preserves that exact vector, rejects depth 4 / terminal count 33,
  and does not split a one-row owner. Check first/last source IDs and quotes
  around `"text\n[S999999] fake"`; that embedded ID is not authorized.
  Core assertions in the prepared fixtures:

  ```rust
  assert_eq!(owned_rows, (0..93).collect::<Vec<_>>());
  assert_eq!(owned_rows_after_split, owned_rows);
  assert!(!packet.labels.contains("S999999"));
  assert_eq!(session.requests(), 128);
  assert_eq!(model.requests.lock().unwrap().len(), 128);
  ```
- [ ] **Write RED request regressions** through the existing loopback model:
  `control_budget_counts_failed_requests_and_denies_129`,
  `reserved_review_is_not_spent_on_optional_recovery`, and
  `retry_discards_unfinished_text`. Assert attempt 129 never reaches the
  server; failed/auth attempts count; no following request contains
  `UNFINISHED RESPONSE`; no metadata request retries. Check worst-case JSON
  escaping/256-character diagnostic and both initial/recovery fits.
  Extend the existing loopback Reply double with a nonempty EOF-without-finish
  case; assert it never returns a completed part or valid control response.
- [ ] **Run RED:** from `src-tauri`, `cargo test --lib summary::parts` and
  `cargo test --lib summary::generation`. Expected: the new assertions fail
  or new modules/functions are missing; record that before implementation.
- [ ] **Implement the ownership/session interfaces.** Extract the existing
  builders and `complete`/`complete_cited` mechanics without changing prompt
  grounding, range validation or provider dialects. Count immediately before
  HTTP, after the cancel/size/budget guards; reserve mandatory internal review
  before spending an optional repair/recovery. Preserve source-only followup
  when the optional complete candidate cannot fit. Share this session with
  the existing short/legacy path, not a second uncounted transport entry point.
- [ ] **Implement deterministic packing/splitting.** Pack whole owned rows
  up to the soft target/exact fit; add whole left then right context rows only
  when they fit. Children rebuild boundary context using the same fit rule.
  Store the chosen context so replacement cannot silently change it. Split
  at the midpoint row boundary, increment depth, reject all size/count gaps.
  Oversized indivisible rows fail without network requests or source cutting.
- [ ] **Run GREEN:** the two unit filters and
  `cargo test --test summary_direct --test summary_long --test summary_pipeline`.
  Existing direct behavior must still pass; no paid test. Review the focused
  diff and record the task outcome locally; do not commit the dirty worktree.

## Task 2 — Layout contract and complete-part writing with bounded splitting

**Interfaces:** consumes Task 1 session/snapshot/parts.

- `layout::LayoutContract { Meeting, KeyPoints, SingleParagraph, Paragraphs,
  Bullets, Sections(Vec<String>) }` (enum, not a persisted setting).
- `layout::resolve(session: &mut Generation<'_>) -> Result<LayoutContract>`;
  called only for multipart writing. Built-in shape resolves locally; custom
  uses exactly one layout-only control call. Built-ins with nonempty extra
  instructions use one metadata feasibility check for global constraints,
  without allowing that response to change the built-in shape.
- `layout::validate_part(markdown: &str, layout: &LayoutContract)
  -> Result<()>`; `layout::local_overview(markdown: &str) -> Option<String>`.
- `part_writer::CompletePart { source: SourcePart, markdown: Option<String> }`.
- `part_writer::write_parts(session: &mut Generation<'_>,
  snapshot: &SourceSnapshot, initial: Vec<SourcePart>, layout: &LayoutContract,
  progress: &impl Fn(&str, usize, usize)) -> Result<Vec<CompletePart>>`.
- `part_writer::replace(session: &mut Generation<'_>, snapshot: &SourceSnapshot,
  part: &CompletePart, extra: &[Range<usize>], layout: &LayoutContract)
  -> Result<CompletePart>` never splits or changes ownership/context.
- Prompt helpers: `prompt::part_system(settings: &SummarySettings,
  vocabulary: &str, layout: &LayoutContract) -> String` and
  `prompt::layout_request(settings: &SummarySettings) -> ChatRequest`.

- [ ] **Write RED tests** for `single_paragraph_rejects_heading_or_bullet`,
  `custom_layout_is_not_meeting`, `layout_control_is_strict_metadata`,
  `filler_marker_requires_source_review`, and
  `initial_split_discards_parent_and_retains_complete_sibling`. For layout
  use strict JSON `{"kind":"single_paragraph"}` / `paragraphs` / `bullets` /
  `{"kind":"sections","headings":["Reasons","Options"]}`; reject unknown
  fields/duplicate or blank section names, empty content, fences and
  `{"kind":"unsupported"}`. The latter is a deliberate unsupported-layout
  error, not permission to rewrite the user's instructions. For a built-in
  feasibility check accept only `{"kind":"builtin"}` or the unsupported
  marker; a model cannot replace meeting/key-point shape with its own layout.
  Pin final writer assertions in those fixtures:

  ```rust
  assert!(layout::validate_part(
      "## Unexpected\nText [S000001]", &LayoutContract::SingleParagraph
  ).is_err());
  assert_eq!(children[0].source.owned.end, children[1].source.owned.start);
  assert!(!all_requests.contains("UNFINISHED RESPONSE"));
  assert!(empty_part.markdown.is_none());
  ```
- [ ] **Pin global-limit handling.** Existing “five key ideas”/concise examples
  must not produce five items *per part*. The metadata feasibility prompt must
  classify strict global selection/count/word-limit constraints as unsupported
  in multipart mode; fail with a localized layout message rather than add a
  global content-selection stage. Keep saved examples/text unchanged, and test
  paragraph and compare-perspectives examples continue to resolve as supported.
  Apply the same fail-explicitly rule to built-in extra instructions such as
  the executive example's global per-section quota, not just custom mode.
- [ ] **Run RED:** `cargo test --lib summary::layout` and
  `cargo test --lib summary::part_writer`. Expected: new assertions fail.
- [ ] **Implement layout parse and prompts.** Custom metadata gets only the
  saved instructions/language, no transcript or facts, no compulsory headings.
  Built-in feasibility checks receive the fixed selected format and the extra
  instructions only; they never change the selected format or saved text.
  Retain source-fidelity/detail policy in part/replacement draft, review and
  repair. Expose/reuse existing localized built-in headings rather than copy
  a new inconsistent list. Parts with supported built-in body content supply
  a short cited local overview; omission of mandatory structure fails clearly.
- [ ] **Implement complete writing.** Use `Generation::cited(..., false, ...,
  true)` with exact `[[NO_RELEVANT_CONTENT]]` as the only empty marker.
  Require original-source review even after an empty draft; mixed marker and
  prose is invalid. Validate each nonempty final part's layout/citations before
  accepting it. On exhausted output limit only, discard the parent and queue
  both children in source order; recompute terminal coverage and call reserves.
  All other terminal failures stop. Replacement uses the whole original owner,
  stored context and every flagged window, with a complete old candidate only
  as optional untrusted context. Oversize replacement context fails, not splits.
- [ ] **Run GREEN** for both unit filters; add production-runner integration
  cases to `summary_parts.rs` when Task 4 connects the path. Confirm each
  `cited` call keeps the original source rows and unknown/gapped IDs still fail.
  Review locally and record results; no commit or paid request.

## Task 3 — Cross-window routing, replacement and stability coverage

**Interfaces:** consumes Tasks 1–2. Source windows are the terminal owning
parts' stored original packets, frozen after initial writing/splits.

- `consistency::ReviewFlags = BTreeMap<PartId, BTreeSet<usize>>`; set members
  index those frozen source windows, not model-nominated source IDs.
- `consistency::sweep(session: &mut Generation<'_>, snapshot: &SourceSnapshot,
  parts: &[CompletePart], overview: Option<&str>, round: u8,
  progress: &impl Fn(&str, usize, usize)) -> Result<ReviewFlags>`.
  `round` is only 1 or 2; overview uses reserved ID `O000001` in round 2.
- `consistency::replace_flagged(session: &mut Generation<'_>,
  snapshot: &SourceSnapshot, parts: &[CompletePart], flags: &ReviewFlags,
  layout: &LayoutContract) -> Result<Vec<CompletePart>>` calls Task 2 replace
  once per flagged body part; no in-sweep mutation or recursive scheduling.

- [ ] **Write RED unit/integration fixtures:**
  `distant_completion_condition_replaces_early_start_claim`,
  `late_explicit_reversal_is_not_two_final_decisions`,
  `answered_question_is_not_left_open`,
  `unrelated_later_statement_does_not_override`, and
  `absence_in_one_window_is_not_disproof`. Record source-window/candidate
  pairs and assert complete Cartesian coverage in each sweep; a needed late
  row remains supplied even when it had no initial draft citation.
- [ ] **Add RED safety assertions:** `[]` inside the exact JSON envelope
  `{"parts":[]}` is valid, empty/truncated JSON is not. Unknown IDs, duplicate
  IDs, extra fields, source IDs pretending to be part IDs and more than 32
  routed IDs fail. A routing flag does not directly edit text. Second-sweep
  flags return `summary_review_unstable`, with no third sweep and no save.
  Assertions on the loopback fixtures:

  ```rust
  assert_eq!(visited_pairs, expected_window_candidate_pairs);
  assert_eq!(replacement_owned, original_owned);
  assert_eq!(completed_sweeps, 2);
  assert_eq!(error.to_string(), "summary_review_unstable");
  assert_eq!(store.load(path).unwrap().markdown, previous.markdown);
  ```
- [ ] **Run RED:** `cargo test --lib summary::consistency`; initially missing
  behavior must fail. Use loopback-only fixtures, not real API credentials.
- [ ] **Implement exact batching and coverage.** For each original window,
  pack whole frozen candidate texts that fit control input, at most 32 IDs per
  batch (32 body parts plus overview therefore require at least two batches).
  Never split/truncate candidate text; if one candidate/window pair cannot
  fit, return context error. Draft text/IDs are untrusted, not source. Do not
  drop candidates or source context to reduce cost. Preflight all known pair
  counts against remaining request budget before starting each sweep.
- [ ] **Implement routing/replacement.** Prompt flags only explicit material
  qualifications/resolutions/conflicts, never absence or stylistic changes;
  state that newest does not automatically win. Collect all windows for each
  flag, union full source rows in chronological order and replace each part
  once after sweep 1 finishes. Keep unaffected material conditions. Sweep 2
  reads the frozen replaced body plus final overview; any flag fails safely.
- [ ] **Run GREEN** unit filter and later `cargo test --test summary_consistency`.
  Mock prose/corrections prove control flow, not an AI semantic guarantee;
  retain source-inspection assertions even when mock routing returns no flags.

## Task 4 — Deterministic assembly, production runner and atomic completion

**Interfaces:** consumes Tasks 1–3; keep public
`pipeline::run(store, input, cancel, progress) -> Result<Summary>` unchanged.

- `layout::assemble(parts: &[CompletePart], overview: Option<&str>,
  layout: &LayoutContract) -> Result<String>` returns canonical-ID Markdown.
- `multipart::run(session: &mut Generation<'_>, snapshot: &SourceSnapshot,
  layout: &LayoutContract, initial: Vec<SourcePart>,
  progress: &impl Fn(&str, usize, usize)) -> Result<String>` does not save.
- New summary error markers: `summary_work_limit`, `summary_review_unstable`,
  `summary_review_invalid`, `summary_layout_unsupported`; retain
  `summary_output_limit`, `summary_context_limit`, incomplete/provider errors.

- [ ] **Write RED assembly tests:** one-paragraph children join with spaces,
  no added connective facts; an uncited child fails *before* joining even if
  another child has a citation. Named sections follow the frozen order,
  subheading/bullet blocks remain attached; only identical text + canonical
  ID set + destination section deduplicates. Different times/conditions stay.
  Built-in overview appears once and standard/detailed metadata stays correct.
- [ ] **Write RED production-runner tests** in `summary_parts.rs` and
  `summary_consistency.rs` using `support::summary_model::Model`:
  prior-cache unchanged at every failed phase, all-empty job fails, filler-only
  owners count, replacement preserves unrelated conditions, small direct
  output-limit fallback shares the original call counter, no complete parent
  or `UNFINISHED RESPONSE` is included in children or final output.
- [ ] **Add cancellation/write tests:** cancel during draft, control, between
  sweeps, before final save and after successful commit. Assert
  `store.load(path).markdown == old.markdown` for acknowledged pre-commit cancel
  and every failure; successful committed run returns `Ok`, not Cancelled.
  Force atomic rename failure in a temporary store and assert old cache bytes
  remain. Never introduce test hooks into public IPC or touch user data.
  In failure/success fixtures respectively:

  ```rust
  assert_eq!(std::fs::read(&cache_path).unwrap(), previous_cache_bytes);
  assert!(result.is_ok()); // cancellation after the successful atomic save
  assert_eq!(summary.source_segments, original_rows.len());
  assert!(!summary.markdown.contains("S000001")); // IDs rendered as clocks
  ```
- [ ] **Run RED:** `cargo test --test summary_parts --test summary_consistency`
  plus `cargo test --lib summary::layout`. Expected: intended new assertions
  fail before runner integration.
- [ ] **Implement the state machine** in `multipart.rs`:
  write/split all initial parts -> freeze windows/version A -> sweep 1 ->
  replace affected parts/version B -> compose one small overview from updated
  local overviews for built-ins -> sweep 2 -> validate each child -> assemble
  -> validate combined canonical IDs. No whole-body AI merge or notes reduction.
  Overview uses the existing cited review, requires complete fitting local
  overview input, never recursively reduces, and is checked against original
  source in sweep 2. Custom has no auto-inserted overview.
- [ ] **Connect the public runner.** Resolve language/settings snapshot once;
  create one session and pre-partition with the existing direct system/guard.
  One owner takes the existing small direct path without a metadata call;
  strict global quotas can still use that existing whole-source response.
  Multiple owners resolve layout and repartition using the actual part-system
  guard before writing. Exhausted direct output recovery enters multipart
  only if the owned source can split: resolve layout, repack and force the
  first bounded split if repacking still yields one owner. Already spent calls
  count. Remove unreachable old direct-map/reduce helpers only if necessary
  for Clippy; do not refactor or revive archived evidence_pipeline behavior.
- [ ] **Keep one final commit point.** The public runner checks cancellation,
  renders validated IDs from original rows, constructs the existing Summary
  with effective format/detail and calls the existing atomic store save once.
  Do not recheck cancellation after successful save and turn it into an error.
  No part deltas/summary-ready/old-chat summary-context update mid-job.
- [ ] **Run GREEN:** `cargo test --test summary_parts --test summary_consistency
  --test summary_long --test summary_direct --test summary_pipeline` and related
  unit filters. Update obsolete map/reduce-specific *direct* mock expectations
  to assert new coverage/finite behavior without weakening source checks.

## Task 5 — Honest progress, localized failures and preserved reader state

**Interfaces:** add Rust stage strings and TS `SummaryStage` values
`writing_parts`, `checking_consistency`, `replacing_parts`, `assembling_summary`;
retain existing `checking_references` and legacy values. Event shape stays
`{ path, stage, done, total }`, zero-based `done` while working; for cross-check
stages encode completed pair count, not an invented overall percentage.

- [ ] **Write RED Vitest tests** in `summaryPartProgress.test.ts` for changing
  part totals after a split, chained-run step 4, indeterminate overall progress,
  cancellation in every new stage and old-path event filtering. A prior result
  and chat summary context stay unchanged until summary-ready for the current
  path; switching files and receiving late events must not replace that file's
  content. Extend `summaryReading` for timestamp toggle/copy of assembled prose.
  Core store assertions after prepared mock events:

  ```ts
  expect(store.summary?.markdown).toBe(previous.markdown);
  expect(store.progress).toBe(-1);
  expect(store.isRunning).toBe(true);
  expect(store.step).toBe(4);
  ```
- [ ] **Add RED bilingual SSR assertions** to `summaryProviders.test.ts` and
  the progress test: work-limit/review/layout/context/output/incomplete errors
  use locale keys, with “สรุปใหม่ไม่สำเร็จ กำลังแสดงฉบับเดิม” only when an old
  result exists. First-run failures say no new summary was saved. Assert
  custom quotas fail explicitly, not meeting fallback or silent truncation.
- [ ] **Run RED:** `npx vitest run tests/unit/summaryPartProgress.test.ts
  tests/unit/summaryProviders.test.ts tests/unit/summaryReading.test.ts`.
- [ ] **Implement stage handling** in TS store/type and SummaryTab, plus update
  Rust progress documentation. All new stages count as running/cancellable and
  the chained summary step; progress remains `-1` overall. Add locale keys
  `summary.writingParts`, `summary.checkingConsistency`,
  `summary.replacingParts`, `summary.assemblingParts`, `summary.workLimit`,
  `summary.reviewUnstable`, `summary.reviewInvalid`,
  `summary.layoutUnsupportedParts`. Do not reuse evidence-certification copy.
  Update context-limit wording so it does not claim this path condensed notes.
- [ ] **Run GREEN** focused Vitest, `npm run typecheck`, `npm run lint`.
  Verify a loopback Preview at narrow/short sizes with keyboard Cancel/retry,
  old-summary warning and long one-paragraph timestamp buttons. No real API
  invocation or closing the user's existing Preview for this fixture QA.

## Task 6 — Full matrix, gated real-file acceptance and Preview handoff

**Interfaces:** production runner and UI above; live test remains opt-in.

- [ ] **Write RED matrix/acceptance-verifier tests.** Extend `summary_long.rs`
  for 1/2/3 hours x meeting-standard/meeting-detailed/key-points/custom with
  dense Thai/escaped rows, exact ownership, original source clocks and every
  actual request measured at initial/recovery caps. Add corrupt fixtures to
  `summary-parts-acceptance.check.mjs`: a missing variant, unknown clock,
  leaked internal ID, wrong metadata, multiline custom, failed/incomplete
  case, changed build/source hash and missing source-comparison notes must
  all fail `check-summary-parts-acceptance.mjs`, never report all-pass.
- [ ] **Run RED:** `cargo test --test summary_long` from `src-tauri` and
  `node --test tests/release/summary-parts-acceptance.check.mjs` from repo root.
  Record the failing new assertions, not gated-test skips as RED evidence.
- [ ] **Implement fixtures/verifier and dry-run gate.** Numeric-only diagnostics
  record calls/phase/counts/bytes/caps. Mock tests enforce all exact spec limits,
  metadata caps, time navigation beyond one hour and no partial reference leak.
  Compute planned and observed control/prose counts before live work; fail
  safely if required coverage cannot fit the 128-call job budget. The verifier
  checks artifacts and recorded review, not truth by itself.
- [ ] **Update the ignored real harness** in `summary_direct_live.rs`: add
  `deepseek_complete_parts_variants` ordered custom -> standard -> key points
  -> detailed, on one unchanged snapshot/build and new output directory.
  Require explicit opt-in `VELO_DIRECT_LIVE=deepseek`; preserve Keychain-only
  key access, numeric diagnostics and the exact existing custom test prompt.
  Do not use the old skip-detailed shortcut or reuse any old successful output.
  On first runtime/layout/reference failure, write failure metadata and stop
  billing later cases. Record outcome/call counts for each attempted variant.
- [ ] **Run all local gates before live acceptance:**

  ```bash
  # src-tauri/
  cargo fmt
  cargo fmt --check
  cargo clippy --all-targets -- -D warnings
  cargo test
  # repository root
  npm run typecheck
  npm run lint
  npx vitest run
  node --test tests/release/*.check.mjs
  npm run version:check
  git diff --check
  ```

  Expected: all required checks exit 0; report counts and ignored/gated cases
  separately. Inspect the changed modules and spec coverage in Native mode.
  No automatic commit, push/tag, version bump or release workflow.
- [ ] **Stop for the real-test authorization gate.** Do not infer a new paid
  test approval from this planning turn. After execution/API approval, verify
  the unchanged `voice_214468.aac` transcript has 2322 rows and SHA256
  `cf41864409f61b2ab126a953e1e3037d1530ac1e4963846400d9b674576dfe3b`.
  Confirm saved provider/model/context without exposing the key; record a
  frozen build hash, transcript hash and temporary output root. If the source
  changed, stop rather than overwrite or transcribe a replacement automatically.
- [ ] **Run the authorized four-case test once.** From `src-tauri` use
  `cargo test --test summary_direct_live deepseek_complete_parts_variants
  -- --ignored --exact --nocapture` with opt-in/settings/transcript/output path
  variables (paths only, never a key). Let the user unlock any Keychain dialog;
  do not read/password-fill it. Fail-fast, no automatic job rerun or cap bump.
- [ ] **Review actual prose against original excerpts** including distant
  conditions, late decisions, answered questions, defaults, requirement/bug
  tracing, change-impact conditions and closing instructions. Check cited
  original rows support the adjacent meaning, not just clock membership.
  Qualify uncertain ASR names/dates. If any required variant fails these
  criteria, report which/why and stop; do not open a new Preview as accepted.
- [ ] **Build/verify the isolated aarch64 Preview** using a temporary config
  with `identifier=com.velo.preview`, separate product name,
  `plugins.updater.endpoints=[]`, `bundle.createUpdaterArtifacts=false`:
  `npm run tauri build -- --config <temporary-config> --bundles app
  --target aarch64-apple-darwin`. Verify bundled resources locally. No new
  native dependency or production app replacement is planned.
- [ ] **Only after all gates pass, hand off Preview.** Reuse the user's
  conditional approval to close/reopen old Preview and back up its old summary
  before showing the new tested detailed result. Back up to a new validated
  private path; do not overwrite the old backup/production cache. Verify the
  opened build, file, cached transcript, tested result/detail label, clock
  seeking, font/layout, current controls and paused playback without another
  API call. If the authorization/state no longer matches, ask before replacing
  data or closing a different app. No updater/Latest/release action.
- [ ] **Record truthful final outcomes** in `design-qa.md` and append the actual
  new pipeline boundary in `PLAN.md`. Keep earlier failed runs/history and
  old task-4 status until its outstanding integrated gates genuinely pass.
  Report local checks, real generation, semantic limitations and Preview QA
  separately; no claim of perfect AI fidelity or duration-only guarantees.

## Native plan self-review and handoff

Coverage checked: ownership/request bounds (Task 1), complete output/layout/
empty parts/splits (Task 2), distant conditions and two frozen sweeps (Task 3),
overview/assembly/atomic save (Task 4), honest reader state (Task 5), synthetic
and actual acceptance (Task 6). The five Review Focus inputs have tests in
their owning tasks. Interfaces consistently share one Generation, PartId,
SourceSnapshot, SourcePart, CompletePart and LayoutContract; source IDs never
double as routing IDs. All new helper modules remain private to the summary.

No implementation, tests or paid requests have run for this plan. The saved
spec approval covers the design; this plan still needs review before Native
task-by-task execution. At each task gate record RED/GREEN evidence and inspect
the diff locally; the user's no-publication boundary overrides automatic commit
steps. No subagent dispatch is part of this plan.

## Approved replacement-layout follow-up (2026-10-23)

The user authorized fixing the observed replacement-format failure, local
verification first, then a new real voice_214468 run in three formats, with
additional DeepSeek calls and Preview only after acceptance. No push/release.
The earlier failed four-format run remains recorded, not reused as a pass.

Local reproduction identifies two distinct defects: the frozen per-part scope
and layout were not repeated after an untrusted draft, and a plain cited
multi-paragraph response failed before the existing one-paragraph whitespace
join could run. The old rejected prose was not retained, so do not claim its
exact rejected structure is known. Regressions must cover these classes before
changes, preserving source/citation validation and rejecting headings/lists.

The scoped fix uses the exact per-part ending for real request preflight,
review and citation repair. Only already-cited plain paragraphs can join by
whitespace; words and IDs remain identical, with no format-only paid retry.
Generated wrong-shape errors are separate from unsupported user-layout errors;
rejection diagnostics contain structural counts only, never source or keys.

Local gates rerun on this implementation. Live scope is explicitly custom,
standard meeting and key points on the same frozen source/build; detailed
meeting keeps local coverage but is not represented as newly live-certified.
Missing/implicit variants cannot pass acceptance. Native author self-review
is retained, not represented as independent review. Keep the previous cache
and running Preview until all requested real gates pass.
