# Evidence Ledger Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans for Native execution or superpowers:subagent-driven-development if the user selects delegation. Execute tasks in order with red/green evidence. Do not implement before the user reviews this plan and selects the execution method.

**Goal:** Preserve what the recording actually says, especially request/current distinctions and completion conditions, across meeting, key-point and custom summaries.

**Architecture:** Replace the free-prose draft/review/audit route with source-claim extraction, field verification, ID-only layout and deterministic Markdown composition. Keep the existing job gate, provider transport, exact citation lookup and atomic summary cache. Evidence is job-local; partial uncertainty is visible rather than guessed away.

**Tech Stack:** Rust, existing serde/serde_json and transports, Vue 3/Pinia, typed English/Thai locales, Cargo/Vitest.

**Spec:** `docs/superpowers/specs/2026-10-21-evidence-ledger-design.md`; glossary: `CONTEXT.md`; existing contracts: `PLAN.md` §31.

**Status:** Approved for Native task-by-task execution and additional DeepSeek calls on 21 October 2026. Local implementation and code gates exist, but real-file acceptance is incomplete: the first protocol-fixed three-format round failed, accepted complete rounds **0/3**. Further live calls stopped at the known-meaning-error boundary. No push, release or readiness claim is authorized by passing code checks.

## Global Constraints

- Preserve all existing work on `codex/summary-prompt-controls`, including the approved Thai font. No push, PR, merge, tag, version bump or release.
- Change the summary pipeline, not transcript decoding/cache identity, chat, dictation, playback, provider/model choice or credentials. No new native dependency or public command.
- Keep transcript text, original order and fractional times unchanged. The ledger records source claims, not objective truth.
- Use the configured single destination with existing thinking/output policies. Never log/export credentials, raw evidence or model bodies during normal operation.
- Extract/verify without custom summary instructions; those instructions reach only layout planning. Vocabulary supplies spelling evidence, not facts or permissions.
- Once admitted, meaning-critical trigger, effect, role, negation and uncertainty fragments cannot be rewritten or omitted by layout. A conditional effect cannot be referenced separately.
- Keep Markdown cache names and old-cache readability. Add only optional omission metadata and progress-stage values; no automatic regeneration or certification badge.
- Custom mode supports selection, order, paragraphs, lists and neutral predefined headings, not arbitrary prose/letter rewrites or newly worded factual headings. Unsupported layouts preserve the old summary and explain the limitation.
- Preserve conditions when shortening. Natural-language word counts remain steering; the Thai 150-word acceptance check uses `Intl.Segmenter`, not Rust whitespace splitting.
- At most `2E + 2V + 2` logical completions for E extraction packets and V verification batches: one extraction schema repair per packet, one verification follow-up OR schema repair per batch, one layout repair. No semantic retry loop or fallback to the old prose reducer.
- Keep the running Preview open and undisturbed. Building a separate bundle is allowed; restarting the user's running app requires checking for unsent/session data first.

## Review Focus

1. Source text containing Markdown, instructions or fake `[S...]` labels must remain quoted data and cannot manufacture citations/headings (tasks 1, 3).
2. An identical source ID/quote from another job, repeated quote, Thai combining marks or ambiguous spelling warrant must not validate accidentally (tasks 1, 2).
3. A long recording may have an answer far after its question; packet adjacency alone cannot establish an agreement or unresolved status (tasks 2, 4).
4. Short custom output and translated framing must not remove event conditions, erase uncertainty or imply protected excerpts were translated (tasks 3, 6).
5. Cancellation, overflow, malformed output or late events while an old summary is visible must not replace it, leak data or unlock a newer job (tasks 4, 5).

## File Ownership and Integration Boundaries

- New `src-tauri/src/summary/evidence/` modules: `mod.rs` (internal exports), `types.rs` (wire/admitted types), `source.rs` (immutable sources and anchors), `admit.rs` (field/link validation), `layout.rs` (plan validation), `compose.rs` (Markdown), `prompts.rs` (structured contracts), `model.rs` (budgeted completions), `stages.rs` (extraction/verification).
- Existing `summary/pipeline.rs` orchestrates those modules and saves once. `summary/mod.rs` retains the public job/transport seam. Do not grow the old `prompt.rs` into another orchestration module.
- Existing `summary/validation.rs` retains exact ID-to-clock resolution; expose its checked lookup for typed application-owned references rather than scanning source quotations for tokens. No nearest-time matching. Retire old production draft/review/audit calls, but preserve existing local files and useful regressions rather than discarding unrelated work.
- New `src-tauri/tests/support/evidence_fixtures.rs` holds synthetic wire responses and hand-derived expected documents only. The offered recording and its generated artifacts stay outside Git.
- Update existing `summary_review.rs` orchestration tests to the new wire contract while preserving cancellation/context/cache regressions. Keep `summary_pipeline.rs` transport tests.
- UI changes are limited to additive summary metadata, progress labels, omission notice, unsupported-layout copy and honoring escaped literal Markdown in the summary reader/copy helper. No typography/layout redesign.

## Task 1: Immutable sources, strict claim schema and protected anchors

**Files:** new `evidence/{mod,types,source}.rs`; modify `summary/mod.rs`; module unit tests.

**Interfaces:**
- `SourceSnapshot::new(path: &str, segments: &[TranscriptSegment]) -> Result<SourceSnapshot>` owns an immutable clone, stable `S000001` IDs and a process-local monotonic job token; no new dependency or persistent token.
- `SourceSnapshot::resolve(anchor: &Anchor) -> Result<ResolvedAnchor>` requires a known ID and one exact nonempty quote match, deriving UTF-8 byte offsets locally. `Anchor` contains `source` and `quote`; the enclosing packet/ledger carries the current job token.
- `CandidatePacket { job, claims: Vec<CandidateClaim> }` is strict JSON (`deny_unknown_fields` on nested objects). Claims have packet-local IDs namespaced by Rust before aggregation.
- `CandidateClaim` separates `SpeechAct` (observation/request/proposal/agreement/question), `ImplementationScope` (existing/planned/unspecified) and `Certainty` (asserted/tentative/unclear), with anchors for each meaningful field.
- `ClaimContent` is a tagged union: ordinary concise clause, event bundle, permission relationship, default-setting request or explicit question. Event trigger/phase/effects, permission roles/comparison and default selected-value are protected anchored fields; no alternate standalone replacement sentence.
- Required qualifiers carry kind (condition/negation/scope/uncertainty) and anchors. Optional owner/deadline and claim links require their own evidence. Spelling correction contains original anchor, replacement and independent recording/vocabulary warrant; it never replaces the raw source.

- [ ] Write unit regressions `exact_anchor_is_job_scoped_and_unique`, `thai_offsets_are_derived_not_model_supplied`, `wrong_source_and_ambiguous_quotes_fail`, and `event_and_permission_fields_cannot_be_prose_shortcuts`.
  Hand fixture: source S1 at 10.25 says `เมื่อตรวจเสร็จ เอกสารออกจากกล่อง`; S2 at 12.75 says `คนอื่นไม่เห็นเอกสารแล้ว`. The trigger resolves only to `เมื่อตรวจเสร็จ` in S1; resolving the effect under S1 or injecting a model offset fails. A second job with the same text rejects the first job token.
- [ ] Run the new unit filter before implementation; observe the missing source/schema boundary fail. Keep the red output as local test evidence.
- [ ] Implement the declared types and exact-anchor resolver. Escape no text yet; retain raw anchors and validate finite ordered source times. Missing critical fields, duplicate IDs and unknown schema keys are errors.
- [ ] Run `cargo test summary::evidence::source` and `cargo test summary::evidence::types`; require all new cases green, including repeated matches, leading/trailing quote bytes, combining marks and source text that contains fake reference tokens.
- [ ] Review the scoped diff before the next task. Any checkpoint commit must exclude unrelated pre-existing changes; otherwise leave local work uncommitted. Never push.

## Task 2: Field-specific admission and bounded evidence stages

**Files:** new `evidence/{admit,model,prompts,stages}.rs`; synthetic fixtures under `src-tauri/tests/support/`; module tests.

**Interfaces:**
- `EvidenceModel::complete(stage: EvidenceStage, system: String, user: String) -> Result<String>` wraps the existing `completion_request`/`transport::stream_chat`; it owns no destination/key other than borrowed `JobInput`. Stage is extracting/checking/planning. No application log includes request/response bodies.
- `extract(snapshot: &SourceSnapshot, model: &EvidenceModel<'_>) -> Result<Vec<CandidateClaim>>` uses bounded packets with two-segment overlap and original source IDs. Packet output is capped at six candidates, requests reserve the existing provider answer allowance. Split input packets at segment boundaries before sending, never truncate or hide trailing content. Output truncation is a failure, not a partial ledger.
- `verify(snapshot: &SourceSnapshot, candidates: &[CandidateClaim], model: &EvidenceModel<'_>) -> Result<AdmittedLedger>` batches up to four candidates, splitting earlier if the actual serialized request cannot fit. `AdmittedLedger` has private admitted-claim constructors and an omitted-uncertain count; the planner cannot construct it.
- A strict verification report covers every assertion, speech act, scope, certainty, content field, qualifier, correction and relationship exactly once. Findings are supported/unsupported/unclear or an explicit request for more source IDs, never replacement prose. `admit` checks coverage, raw anchors, dependency/cycle integrity and job identity before constructing admitted claims.
- First-pass verification sees each claim's passages plus one adjacent source segment on either side and a compact catalog of all candidates (IDs/kind/anchors/links). One follow-up may fetch requested raw IDs with the same adjacency; an oversized request is a context error preserving the old cache, while a fitting follow-up that still cannot establish support quarantines the affected claim. A schema repair consumes that same second-call slot.
- Ordinary clauses remain model-assisted: verification must independently classify protected content present in them; a candidate ordinary clause found to be conditional/role-bearing is quarantined, not silently rewritten. Cross-packet answers/agreements need supported links, not an isolated yes.

- [ ] Write `request_is_not_implementation_or_agreement`, `future_use_is_not_an_unasked_question`, `default_spelling_is_not_development_order`, `ambiguous_permission_does_not_destroy_independent_clear_claim`, `later_answer_resolves_only_its_linked_question`, and `missing_field_verdict_cannot_admit_claim`.
  Synthetic assertions: request + planned remains requested/planned; agreement + planned remains agreed/planned; unlinked `ใช่` cannot approve two requests; `เก็บข้อมูลไว้เผื่อใช้ภายหลัง` yields no open issue; an answered question cannot remain open. A supported independent positive permission claim survives only as its own candidate.
- [ ] Run those cases red against the missing admission/stage boundary. Add loopback request captures to assert custom instructions never enter extraction/verification and raw evidence, not model notes, supplies quotes. These mocks test orchestration, not factual accuracy.
- [ ] Implement extraction, focused verification and admission with one schema repair or one context follow-up as specified. Deduplicate only claims with identical anchored content and classifications; conflicting event phases stay separate. Propagate quarantined dependencies without creating substitute claims.
- [ ] Verify actual serialized request bytes against `(context_tokens - max_tokens) * 3`, including system/schema/catalog/candidate/context/fences/reminders. Reject an indivisible oversized unit or catalog before HTTP; use existing output caps and transport retries without adding a new semantic retry route.
- [ ] Run `cargo test summary::evidence`; require duplicate/missing verdicts, unknown cross-packet IDs, unsupported corrections, dependency cycles, output truncation and exhausted second calls to fail safely. Inspect captured requests for both local/OpenAI and DeepSeek policies without real credentials.

## Task 3: ID-only layout and deterministic composition in all formats

**Files:** new `evidence/{layout,compose}.rs`; extend `evidence/prompts.rs`; module tests and synthetic golden fixtures.

**Interfaces:**
- `LayoutPlan { job, status, blocks }`: status is supported/unsupported; supported blocks are `Heading(HeadingKind)`, `Paragraph(Vec<ClaimId>)` or `List(Vec<ClaimId>)`. Unsupported has no rendered blocks. Unknown keys including `text`, titles, prose, offsets and claim fragments are rejected.
- `HeadingKind` enumerates overview/topics/decisions/actions/open-questions/key-points/details. Custom may use only neutral overview/key-points/details headings, optionally none. Meeting uses supported outcome categories; key-points uses overview/key-points only.
- `validate_plan(plan: LayoutPlan, ledger: &AdmittedLedger, format: SummaryFormat) -> Result<ValidatedPlan>` validates category eligibility, whole-claim references, dependencies and placement. A claim appears at most once in overview and once in one body section; no duplicates inside a paragraph or competing body sections.
- `compose(plan: &ValidatedPlan, ledger: &AdmittedLedger, sources: &SourceSnapshot, language: &str) -> Result<String>` returns clock-cited Markdown. Internally keep literal text and application-owned source-reference spans separate; resolve only the latter through the checked exact clock lookup during serialization. Never run a token-replacement pass over raw source quotations. The composer owns neutral separators, speech-act/scope/certainty labels and headings; it cannot generate a new factual sentence or causal connector.
- `ComposeError::UnsupportedLayout` maps to the fixed public message sentinel `unsupported_summary_layout`; use localized UI explanation without changing the failure-event shape. A valid but unsupported custom request is not retried into an approximation.

- [ ] Write `effect_only_selection_is_impossible_in_every_format` and literal golden-output tests. The valid event output in each format must contain `เมื่อตรวจเสร็จ`, both original effects and exact references `[00:10]`/`[00:12]` for S1/S2; omission of the trigger fails, never yields an effect-only summary.
- [ ] Write `planner_cannot_supply_prose_or_factual_headings`, `category_cannot_promote_request`, `duplicate_body_placement_fails`, `planned_agreement_stays_planned`, `source_markup_cannot_become_layout`, and `custom_paragraph_keeps_qualifiers_without_headings`. Assert against hand-written strings/structures, not values generated by the renderer itself.
- [ ] Observe red before composition exists, then implement the closed layout schema and renderer. Backslash-escape literal Markdown punctuation (including brackets, backslashes, asterisks and line-leading heading/list markers) before serialization, and flatten source-internal hard line breaks to spaces so they cannot create blocks. Unsupported factual organization is rejected, not hidden as a caption or heading. Task 5 teaches the small reader/copy parser to honor those escapes without treating source-like clock text as a seek button.
- [ ] Preserve explicit question state and uncertainty with localized framing; decisions require agreement evidence, actions explicit requested/assigned work, open issues explicit unresolved/deferred question evidence. Do not invent owner/date placeholders unless requested by the user.
- [ ] Run `cargo test summary::evidence`. Mutation-check the completion trigger, planned scope and mandatory qualifiers: changing to starting review or dropping a qualifier must break the golden cases in every format. Run cross-language cases: English framing may contain protected Thai quotations and must not claim full translation.

## Task 4: Replace orchestration without losing lifecycle guarantees

**Files:** modify `summary/{pipeline,mod,types}.rs`, `src-tauri/tests/{summary_review,summary_pipeline,chat_pipeline}.rs`; add `src-tauri/tests/summary_evidence_pipeline.rs`; adjust Rust Summary literals for additive metadata.

**Interfaces:**
- Keep `pipeline::run(store: &SummaryStore, input: &JobInput, cancel: &AtomicBool, progress: impl Fn(&str, usize, usize)) -> Result<Summary>` unchanged.
- Add `Summary.omitted_uncertain_claims: Option<usize>` with serde default and skip-when-none. Old caches keep `None`; a completed ledger job may record `Some(0)` but receives no assurance badge. Update the Markdown comment to describe deterministic composition, not model-written prose.
- Emit stages `extracting_evidence`, `checking_evidence`, `composing_summary`; retain old stage values for compatibility. Save and emit ready only after complete composition/reference validation and a final cancel check.

- [ ] Add loopback end-to-end tests `all_formats_use_extract_verify_plan_without_prose_writer`, `partial_quarantine_saves_notice_but_all_quarantined_preserves_cache`, `each_stage_cancel_preserves_previous_bytes`, and `repair_ceilings_stop_without_fallback`.
  For one packet/one batch: exactly three happy-path completions and at most six including bounded repairs. An all-quarantined ledger stops before layout. Invalid final layout never invokes the old five-completion route.
- [ ] Observe red against the current `pipeline::run`, then wire tasks 1–3 through the unchanged job entry point. Remove production calls to the old draft/review/audit/reducer; retain existing files and useful regression coverage, clearly distinguishing legacy helper tests from the active pipeline.
- [ ] Rewrite old orchestration fixtures for the new stages rather than weakening their original expectations. Preserve old cache on malformed/truncated JSON, semantic quarantine, invalid links, context overflow, transport failure and cancellation at every await/save boundary.
- [ ] Exercise a long synthetic transcript with a later answer across packet boundaries, overlapping source IDs, duplicate claims and an oversized source/catalog/layout. No unchecked notes, raw-source truncation or automatic fallback. Failed/cancelled jobs release only their own model lease; run existing chat/job tests.
- [ ] Run `cargo test --test summary_evidence_pipeline`, `cargo test --test summary_review`, `cargo test --test summary_pipeline` and `cargo test --test chat_pipeline`. Confirm source transcript bytes are unchanged and summary replacement remains atomic.

## Task 5: Bilingual progress, omission notice and backward-compatible UI

**Files:** `src/types/summary.ts`, `src/stores/summaryStore.ts`, `src/components/transcript/SummaryTab.vue`, `src/utils/summaryMarkdown.ts`, `src/locales/{en,th}.ts`; `tests/unit/{summaryChain,summaryReading,summaryMarkdown,summaryNavigation}.test.ts`; new `tests/unit/summaryEvidenceStatus.test.ts`.

**Interfaces:** Add optional `omitted_uncertain_claims?: number | null` and the three stage values to TypeScript. Keep existing command names and event bodies. Map the unsupported-layout sentinel to localized copy; retain other errors as currently handled.

- [ ] Write mounted regressions for each new stage in standalone/chained runs, cancel routing for all three, and cross-file late events preserving the current result. Expect step 4/4 throughout the three summary stages, not a blended whole-job percentage.
- [ ] Write reading tests: omitted count >0 shows a localized status notice; absent/null/zero shows no notice or certification. An unsupported custom form leaves the cached Markdown visible with an actionable message; old caches load without mutation.
- [ ] Write `escaped_source_stamp_is_text_not_navigation` and copy tests with a source literal `\\[00:10]` beside an application reference `[00:12]`: the first displays literal `[00:10]` without a seek button, while the second seeks to exactly 12.75. Timestamp-hidden copy retains the source literal and removes only the application reference. Escaped heading/list/bold text remains text; old ordinary Markdown still renders as before.
- [ ] Observe those cases red, then add the labels/notice/error mapping, stage handling and a small escape-aware inline/copy scanner. Unescape literal punctuation only after structural/citation recognition; do not enable HTML or arbitrary links. Use English source-of-truth keys plus Thai translations. Omission copy says details were excluded because evidence was unclear, not that retained details are verified truth.
- [ ] Run `npm run typecheck`, `npx vitest run tests/unit/summaryChain.test.ts tests/unit/summaryReading.test.ts tests/unit/summaryEvidenceStatus.test.ts`, then all frontend tests. Keep timestamp hide/copy/exact fractional seek, prompt review, chat, microphone and the approved font regressions passing.

## Task 6: Full verification and repeatable real-file acceptance

**Files:** extend `src-tauri/tests/summary_fidelity_live.rs`; add `src-tauri/tests/support/evidence_acceptance.rs`, `scripts/check-evidence-acceptance.mjs`; add default-off feature `summary-evidence-qa` in `src-tauri/Cargo.toml` only for artifact observation; update `PLAN.md`, `CHANGELOG.md`, `design-qa.md` with actual results.

**Interfaces:** Production `pipeline::run` remains unchanged. A feature-gated `pipeline::run_observed(..., observer: &dyn EvidenceObserver)` uses the same internal runner and read-only typed artifact callbacks. The default app uses a no-op observer and never persists a ledger. Callback payloads are whitelisted evidence/report/layout/composed data, never JobInput, headers or keys. Artifact capture requires explicit harness opt-in and a new local output directory.

- [ ] Write harness regressions proving observation changes neither requests nor rendered output, artifacts exclude credential fields, and the repeatability counter resets to zero when any format/checklist item fails. The acceptance checker reads hand-reviewed per-run checklist results, rejects missing/unknown checks and records Node/Intl runtime; it is not a keyword-based semantic oracle.
- [ ] Run all six repository commands fresh: `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `npm run typecheck`, `npm run lint`, `npx vitest run`. Also run `cargo fmt --check`, `npm run version:check` and `git diff --check`. Supply the installed local Whisper model and non-silent synthetic test audio where available; report opt-in skips accurately.
- [ ] Build a separate default-feature Apple Silicon app with `npm run tauri build -- --bundles app --target aarch64-apple-darwin --config <temporary preview config>`. Keep version 0.8.0 and the installed/running Velo untouched. Do not hide a default-build failure behind a QA-feature build.
- [ ] Snapshot the currently authorized transcript/settings hashes for the same 3:44 recording. Use the original 53-segment cache, not the diagnostic re-transcription. Run the existing `live_deepseek_all_three_formats` with `--features summary-evidence-qa -- --ignored --nocapture` and the existing opt-in environment guards. It must use only the saved DeepSeek destination/key in memory and a new temporary store; do not write production preferences or caches.
- [ ] For meeting/key-points preserve the current focus settings. For custom use exactly `สรุปเป็นย่อหน้าสั้น ๆ ไม่เกิน 150 คำ ไม่ต้องมีหัวข้อหรือรายการ`. Compare all candidates, field verdicts, layout and final text against the raw transcript. Log per-job E/V/request counts, failures and elapsed times, not a fabricated accuracy score.
- [ ] Check the minimum content boundary in **every** format: requested person/group choice; person as default rather than development order; completion of review as the removal trigger with its original question/uncertainty where unresolved; backend read/unread retention without invented current-non-use or open issue. Meeting/key-points additionally keep the described shared review box and actual-reviewer recording without invented permission ownership. A custom paragraph may omit only those last two secondary details.
- [ ] Reject added owners/dates, fabricated permission comparisons, start/completion substitution, unanswered-question inventions and factual headings. Confirm every clock label comes from the original source set. Count the custom paragraph with `Intl.Segmenter('th', {granularity:'word'})`, `isWordLike` only after removing application citations; record the Node/ICU version. No headings/lists and <=150 word-like segments for this fixture.
- [ ] Require **three consecutive complete three-format rounds** (nine successful jobs) on one unchanged build and fixed checklist. A failed format resets the sequence; retain all failed artifacts. If a code change is needed, rerun the relevant automated gates and restart the sequence. Do not assemble a pass from different builds. The per-job ceiling remains `2E + 2V + 2`, so actual API count depends on packet/batch count, not merely nine requests. Do not extend the budget or soften acceptance to manufacture a pass.
- [ ] If extraction/verification still admits a known meaning error, stop and report the exact source/claim boundary and retained artifacts; do not declare completion because IDs/layout tests passed. If additional authority or an expanded experiment budget is needed, request it before further runs.
- [ ] Inspect progress/omission/unsupported-layout in the separate app without triggering an unapproved cloud request. If opening a new Preview would discard current chat/draft state, ask before closing the old app. Verify the current transcript, summary and settings hashes are unchanged after the isolated harness.
- [ ] Update docs with actual commands, counts, skipped checks, remaining limits and acceptance verdict. Keep private artifacts outside Git. Review the complete local diff with the selected execution method, then hand off without push or release.

## Plan Self-Review and Next Gate

- Spec acceptance cases 1–8 map to tasks 1–3; lifecycle/budget/atomic-save case 9 maps to task 4; omission/legacy case 10 maps to tasks 4–5; custom/language/navigation case 11 maps to tasks 3, 5–6.
- All five review-focus inputs have explicit tests above. The ledger verification is still model-assisted; deterministic validation/composition does not establish semantic truth.
- This plan deliberately supersedes the old free-prose runtime while preserving provider/lifecycle/reader interfaces and prior unrelated local work. No new platform, model, signing or cache-fingerprinting task is included.
- Before product edits, ask the user to review this plan and choose Native task-by-task execution or delegated task/review execution. Recommend Native for continuity with the existing local work; retain an independent final review. No API call occurs during this planning gate.
