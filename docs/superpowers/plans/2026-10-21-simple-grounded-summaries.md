# Simpler Grounded Summaries Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans for the user-selected Native execution, task by task with red/green evidence and one fresh final review. Steps use checkbox (`- [ ]`) syntax for tracking. This written plan was reviewed and approved; resume from the Native progress ledger, not from unchecked historical steps.

**Goal:** Replace the full ledger with short source-backed points while preserving conditions, exact citations and the unchanged nine-job semantic acceptance gate.

**Architecture:** Extract small point packets, select whole point IDs into a layout, review each visible occurrence together with its heading, then render Markdown locally and save once. Reuse the existing source snapshot, transport, job gate, reader and cache interfaces; remove the superseded field taxonomy and relationship graph after integration.

**Tech Stack:** Rust, existing serde/serde_json and provider transports; Vue 3/Pinia, Cargo, Vitest and the existing Node acceptance checker. No new dependency.

**Spec:** `docs/superpowers/specs/2026-10-21-simple-grounded-summaries-design.md`, approved 21 October 2026. Native, this implementation plan and additional DeepSeek calls for the original recording were explicitly approved. Tasks 1–4 are complete; task 5 automated gates/build pass, but both live attempts failed extraction. The post-repair attempt finished after Keychain authorization on 22 October, with zero accepted rounds; do not rerun it as though authorization were still pending. See the Native ledger and `design-qa.md` for the blocking result.

## Global Constraints

- Preserve all existing work on `codex/summary-prompt-controls`, including Thai font, chat, microphone, player and summary controls. Reuse this checkout per the user's explicit preference. No push, PR, merge, tag, release or version bump; keep 0.8.0.
- Preserve public commands, Markdown cache shape, optional omission count, stage values, cancellation, model lease and atomic replacement. Do not change decoding, source text/order/fractional times, provider/model/destination, credentials or permissions.
- A point has only `text`, nonempty `evidence`, and present `required` arrays. Rust assigns point and occurrence IDs. All wire objects use strict schemas and the current job token.
- Required quotations remain verbatim inside the complete point. Matching quotes prove location, not meaning. No runtime status axes, special content union, answer/dependency graph, owner/date fields or spelling-edit chain.
- Extraction uses at most twelve segments and 3000 UTF-8 bytes of serialized source rows per packet, two-segment overlap where feasible, and up to six points. Packets shorter than three segments advance at least one segment to avoid a loop; no source is skipped.
- Account for the complete request using the existing `(context_tokens - max_tokens) * 3` policy, including schema/catalog/feedback. Indivisible source or catalog overflow fails without truncation or a fallback.
- At most `2E + 2V + 2` logical completions: two per extraction packet, two for layout, two per review batch. Review's second slot is context follow-up OR schema repair. No semantic rewrite loop.
- Extraction has no custom layout steering. Layout has only whole-point references and closed headings. Review sees the complete text and its selected heading, raw evidence and later clarifications, not the custom steering.
- All HTTP remains in Rust against the configured single destination, with existing thinking/output policies. No key/header/JobInput logging or export. Evidence persists only in explicitly opted-in private QA artifacts outside Git.
- A failed report, unsupported custom form, cancellation, provider failure or overflow preserves the preceding summary. Partial rejection removes complete occurrences only; no replacement prose or silent relabeling.
- Keep the running Preview undisturbed. Build a separately named default-feature Apple Silicon app; do not install, launch over or close the user's session without checking its unsent/session data first.
- Use hand-derived synthetic fixtures permanently. Keep all failed private runs. Require three consecutive complete three-format rounds on one unchanged build and fixed source/settings; never assemble a pass across builds or weaken the minimum content checklist.

## Review Focus

1. A point containing fake source IDs, clocks, headings or embedded instructions must remain quoted data; only application-owned references may navigate. Test in tasks 1–2.
2. A short custom paragraph or translated framing must retain every required trigger/negation/uncertainty phrase, without adding a status label or turning completion into starting. Test in tasks 1–2.
3. The same point in overview and a decision/open-question section needs separate placement verdicts; one supported occurrence cannot certify the other or inflate the omitted-detail count. Test in task 2.
4. An answer far after a question must be checked against original passages, including a bounded follow-up when the full recording cannot fit; missing context cannot manufacture an open issue. Test in task 3.
5. Cancellation during an actual HTTP await, cross-file late events, observer capture and changing source/settings/build identities must not publish partial output, reset a newer job, leak keys or certify mixed runs. Test in tasks 3–4.

---

## File Ownership and Migration

- Reuse `src-tauri/src/summary/evidence/{types,source}.rs` for the lean point schema, existing exact anchors and packet boundaries. Do not create another source registry.
- Reuse `evidence/layout.rs` for closed layout validation and occurrence IDs; `admit.rs` for whole-occurrence review reports and the private reviewed-plan boundary; `compose.rs` for escaped Markdown. These filenames do not retain the old semantic field system.
- Reuse `evidence/{prompts,stages,model}.rs`; `model.rs` keeps its budget/transport/observer wrapper. `pipeline.rs` remains the single production runner. No second runtime module or feature-selected fallback.
- Update `src-tauri/tests/{summary_evidence_pipeline,summary_review}.rs` and `tests/support/{evidence_fixtures,evidence_acceptance}.rs`. Keep transport/chat regressions. Add only test utilities that an actual test consumes.
- Preserve frontend source unless a compatibility regression demonstrates a required change. Relevant tests are `tests/unit/{summaryChain,summaryEvidenceStatus,summaryReading,summaryMarkdown,summaryNavigation}.test.ts`.
- Tasks 1–2 introduce private, unit-tested lean helpers alongside the only active old runner. Compile migration-only helpers under `#[cfg(test)]` until task 3 connects them; remove those temporary gates and obsolete production types in that atomic switch. Default behavior must not route into unfinished helpers.
- At execution start, save a verified local archive of current summary sources/tests and the complete dirty diff, including untracked files, in this plan's ignored workspace. Keep the earlier ledger workspace/backups untouched. Never use broad clean/reset commands.
- Stage only separable changes belonging to this plan. If an overlapping file cannot be safely separated from prior work, retain an uncommitted checkpoint and record the reason in the Native ledger instead of committing user edits.

## Task 1: Lean points and complete source packets

**Files:** modify `evidence/{types,source}.rs`; unit tests in those modules.

**Interfaces:**
- `DraftPoint { text: String, evidence: Vec<Anchor>, required: Vec<Anchor> }` and `PointPacket { job: String, points: Vec<DraftPoint> }` derive strict Deserialize/Serialize; no model-supplied point ID.
- `SourcePoint { id: String, point: DraftPoint }` is application-created, Serialize only. IDs are `P000001`, `P000002`, etc., assigned to unique points in source traversal order in task 3.
- `source::validate_point(snapshot: &SourceSnapshot, point: &DraftPoint, supplied: &BTreeSet<String>) -> Result<()>` checks nonempty text/evidence, exact unique anchors, supplied IDs, every required phrase in text and coverage by an evidence quote under the same source ID.
- `SourceSnapshot::packet_ranges(&self, fits: impl Fn(&str) -> bool) -> Result<Vec<Range<usize>>>` returns segment ranges; `fits` receives serialized source rows and lets task 3 include the full request envelope. Keep existing `resolve`, `lines`, `job` and `check_job` interfaces.

- [ ] **Write strict-schema and boundary tests.** Use synthetic S000001 at 10.25 saying `When inspection finishes, remove the item.` and S000002 at 12.75 saying `Other reviewers no longer see it.`. A valid point retains `When inspection finishes`; the same point with that phrase removed fails even when its citations are valid. Reject missing `required`, extra `scope`, a model `id`, invented source IDs, repeated quotes, UTF-8 ambiguity and anchors outside the supplied packet. Separately assert `check_job` rejects a packet from another job even if its source IDs happen to match. Literal fake clock/heading text is valid data, not parsed structure.

```rust
assert!(validate_point(&sources, &without_trigger, &supplied).is_err());
assert!(serde_json::from_value::<PointPacket>(missing_required).is_err());
assert!(serde_json::from_value::<PointPacket>(extra_scope).is_err());
assert_eq!(sources.packet_ranges(|_| true).unwrap(), vec![0..12, 10..22, 20..25]);
```

The last assertion uses 25 distinct short synthetic segments. Add Thai-byte fixtures where each individual source fits but combining sources exceeds 3000 bytes, plus a closure forcing an earlier context split. Assert original tail coverage, ordered progress and no HTTP call. A single oversized source returns a context error.

- [ ] **Observe RED.** Run from `src-tauri`: `cargo test --lib summary::evidence::source` and `cargo test --lib summary::evidence::types`. Missing declarations may be the first compile failure; add only minimal test-scoped declarations/stubs and rerun until the behavioral assertions fail for the intended boundary. Keep the red logs.
- [ ] **Implement the declared types and validation.** Preserve original bytes/times and existing exact-anchor protections. Greedily extend ranges under both limits and `fits`, then reuse up to two preceding segments without losing forward progress. Do not silently default a missing required field or semantically merge similar points.
- [ ] **Verify GREEN.** Run both unit filters, then `cargo test`. Expected: new boundary cases and existing default suite pass; live opt-ins retain their explicit skips. Do not describe packet coverage as proof that a model extracted every substantive topic.
- [ ] **Checkpoint.** Review the scoped diff and archive a task checkpoint. If separable, commit only this task with `refactor: add lean source-backed summary points`; otherwise record an uncommitted checkpoint in the ledger.

## Task 2: Placement review and deterministic rendering

**Files:** modify `evidence/{layout,admit,compose}.rs`; unit fixtures/tests in their owning modules. Consume task 1's `DraftPoint` and `SourcePoint`.

**Interfaces:**
- Reuse strict `LayoutPlan`, `LayoutBlock` and `HeadingKind`, including the existing private `claims: Vec<String>` wire key for point references. `layout::validate_points_plan(plan: LayoutPlan, points: &[SourcePoint], format: SummaryFormat, sources: &SourceSnapshot) -> Result<PointPlan>` validates structure, formats, whole IDs and duplicate placement. It does not infer agreement/question status before review.
- `Occurrence { id: String, point_id: String, heading: Option<HeadingKind>, block_index: usize }` is application-created. `PointPlan` privately owns the plan and ordered occurrences, assigning `O000001` etc., with `layout(&self) -> &LayoutPlan` and `occurrences(&self) -> &[Occurrence]`. Each repeated point placement gets a different occurrence ID.
- `OccurrenceCheck { occurrence: String, verdict: Verdict, evidence: Vec<Anchor> }` and strict `PointReviewReport { job: String, checks: Vec<OccurrenceCheck>, need_sources: Vec<String> }` replace field reports. Retain the supported/unsupported/unclear enum.
- `admit::validate_occurrence_report(sources: &SourceSnapshot, points: &[SourcePoint], occurrences: &[Occurrence], report: &PointReviewReport) -> Result<()>` checks final empty `need_sources`, complete unique coverage and exact evidence covering the point's declared evidence/required anchors.
- `admit::reviewed_plan(sources: &SourceSnapshot, points: &[SourcePoint], plan: PointPlan, report: PointReviewReport) -> Result<ReviewedPointPlan>` is the only reviewed-plan constructor. Its private data is exposed through `plan(&self) -> &PointPlan`, `check(&self, occurrence: &str) -> Option<&OccurrenceCheck>` and `omitted(&self) -> usize`. Composition errors if a known occurrence has no check. The count includes selected unique points with no supported placement, not points intentionally unselected for brevity.
- `compose::compose_points(reviewed: &ReviewedPointPlan, points: &[SourcePoint], sources: &SourceSnapshot, language: &str) -> Result<String>` renders unchanged complete point text with neutral paragraph/list separators, closed headings and exact clocks. References include point evidence/required anchors and that occurrence's checked supporting evidence.

- [ ] **Write literal goldens and review-boundary tests.** Cover meeting/key-point/custom layouts and English framing containing protected Thai text. The valid custom result is exactly `When inspection finishes, remove the item; other reviewers no longer see it. [00:10] [00:12]`, with no automatic status prefix. Hand-build separate O000001 overview and O000002 decision checks for one point; reject only the latter as unsupported and preserve the former. Add complete rejection, unknown/missing/duplicate occurrence IDs, old-job reports, fabricated evidence and unsupported custom forms.

```rust
assert_eq!(compose_points(&reviewed, &points, &sources, "en").unwrap(),
    "When inspection finishes, remove the item; other reviewers no longer see it. [00:10] [00:12]");
assert_eq!(reviewed.omitted(), 0); // One supported placement of this point survives.
assert!(validate_occurrence_report(&sources, &points, &occurrences, &missing_check).is_err());
```

Use separate fixtures for these assertions. Also reject duplicate body references, effect-fragment IDs and planner text/factual headings. A raw source `## heading [00:10]` remains escaped literal text next to an application reference resolving to 12.75. Reject a late-answer question under an open-question heading through its placement verdict; use an actual question and later explicit answer, not a renamed ordinary request. Verify two rejected placements of one selected point count once; unselected points do not inflate the notice.

- [ ] **Observe RED.** Run `cargo test --lib summary::evidence::layout`, then the `admit` and `compose` filters. Record intended failures before implementations; scoped declarations remain test-only during migration.
- [ ] **Implement the reviewed-plan boundary and renderer.** Whole-occurrence rejection never edits a point or promotes it to another heading. Omit empty sections and fail if nothing substantive survives. Flatten source newlines, escape literal Markdown, and resolve only typed application references; do not scan/replace source text for citation tokens or spelling edits.
- [ ] **Verify GREEN and mutation coverage.** Run `cargo test --lib summary::evidence` and `cargo test`. Confirm literal goldens in every format catch trigger removal, negation/uncertainty loss, insertion of an existing/planned label and accidental heading inheritance. A synthetic supported verdict establishes orchestration only, not entailment.
- [ ] **Checkpoint.** Review only task changes; use a separable commit `refactor: review complete summary placements` or a documented local checkpoint.

## Task 3: Bounded model stages and the single production runner

**Files:** modify `evidence/{prompts,stages,mod,model}.rs`, `summary/pipeline.rs`, `tests/support/evidence_fixtures.rs`, `tests/{summary_evidence_pipeline,summary_review}.rs`; retire obsolete runtime definitions in `evidence/{types,layout,admit,compose}.rs` only after wiring the replacement.

**Interfaces:**
- Keep `EvidenceModel::fits`, `complete`, `check_cancel`, public `pipeline::run(...)` and feature-gated `run_observed(...)` unchanged. Preserve `JobInput`, Summary and the command/job lease seams.
- `prompts::points(settings: &SummarySettings, vocabulary: &str, job: &str) -> String`, `points_layout(settings: &SummarySettings, job: &str) -> String`, and `placement_review(job: &str) -> String` produce the lean contracts, using actual job tokens and explicit allowed values rather than copyable placeholders.
- `stages::extract_points(sources: &SourceSnapshot, model: &EvidenceModel<'_>) -> Result<Vec<SourcePoint>>`; `plan_points(sources: &SourceSnapshot, points: &[SourcePoint], model: &EvidenceModel<'_>) -> Result<PointPlan>`; `review_points(sources: &SourceSnapshot, points: &[SourcePoint], plan: PointPlan, model: &EvidenceModel<'_>) -> Result<ReviewedPointPlan>`.
- Extend the synthetic HTTP utility with `server_gated(answers: Vec<Value>, pause_request: usize) -> (String, Receiver<Value>, tokio::sync::oneshot::Receiver<()>, Sender<()>)`. It pauses after reading the selected 1-based request body until the test releases it, permitting condition-driven cancellation without fixed sleeps.

- [ ] **Write wire/cache regressions before the runtime switch.** In all formats, one-packet happy paths issue extraction → layout → placement review, exactly three logical requests, then save one literal expected document. Extraction/review exclude private custom steering; layout includes it. The review body contains the actual chosen heading and point text, original raw passages and required phrases, with no field-taxonomy report. Exercise all nine combinations of Ollama/OpenAI-compatible/DeepSeek × meeting/key-points/custom, retaining each transport's thinking/output policy. Reject obsolete field-verdict report shapes; do not parse them as a fallback.

```rust
assert_eq!(requests.len(), 3);
assert_eq!(request_stages, ["extracting", "planning", "checking"]);
assert_eq!(summary.markdown, "## Overview\n\nThey requested a group option. [00:01]");
assert_eq!(store.load("/synthetic.wav").unwrap().markdown, summary.markdown);
```

Add actual gated cancellation inside each HTTP await; bound channel waits with a Tokio timeout so a broken fixture fails rather than hangs. Cancel after final review/before save through the existing composition progress callback. Require cancelled error, unchanged old-cache bytes and no later request. Keep cross-file job ownership tests. A wrong job gets only the existing structural repair. Exhaust extraction/layout repair slots and both review slot orders (follow-up→malformed, malformed→follow-up, repeated follow-up); no third call or prose fallback.

Use a long synthetic recording with each source below 3000 bytes, enough total raw context to force focused review, and a distant explicit answer. The first review requests the answer's source ID; the second supplies its original text with neighbours and rejects the misleading open-question placement. Assert the `sources` payload itself, not the whole request/catalog string. Also test full-snapshot review when it fits, indivisible-source/catalog/layout overflow before HTTP, unknown fetched IDs and truncated output preserving the old summary.

- [ ] **Observe RED.** Run `cargo test --test summary_evidence_pipeline --test summary_review` against the new wire fixtures. Expected: old runtime cannot consume the lean protocol/order and the new lifecycle assertions fail. Keep the red output; do not weaken assertions to keep the old pipeline passing.
- [ ] **Implement the bounded stages and atomic integration.** Use task 1's ranges, actual request sizes and specific bounded structural feedback; reserve room for the longest known feedback envelope. Deduplicate exact point data only, assigning stable Rust IDs. Plan whole points, then review up to four occurrences per batch: try the full raw snapshot first, otherwise focused raw passages + neighbours/catalog. Fetch requested raw IDs once; invalid final reports fail, and an exhausted valid request quarantines that batch. Resolve auto language before building prompts. Emit `extracting_evidence` through preparation/layout, `checking_evidence` for review, then `composing_summary` for local render/save. Never stream drafts.
- [ ] **Retire the old runtime and verify GREEN.** Remove the migration-only test gates, field taxonomy, special content variants, graph traversal and spelling replacement from production. Port useful regressions to the new point/report boundary rather than keeping a second implementation. Preserve the unrelated legacy audit tests only if still useful; do not retain obsolete code merely to maintain a test count. Run `cargo test` and `cargo clippy --all-targets -- -D warnings`; require no new warnings and no old-runtime fallback. Inspect captures for actual context budgets and the `2E + 2V + 2` ceiling, excluding bounded transport retries from logical counts.
- [ ] **Checkpoint.** Inspect the complete summary diff and snapshot current tests. Commit only separable task work as `refactor: simplify grounded summary pipeline`, or ledger why the overlapping changes remain uncommitted.

## Task 4: Reader compatibility and trustworthy acceptance tooling

**Files:** modify `tests/unit/{summaryChain,summaryEvidenceStatus,summaryReading,summaryMarkdown,summaryNavigation}.test.ts`, `src-tauri/tests/{summary_fidelity_live,summary_evidence_pipeline}.rs`, `tests/support/evidence_acceptance.rs`, `scripts/check-evidence-acceptance.mjs`, `tests/release/evidence-acceptance.check.mjs`. Change frontend source only for a demonstrated compatibility failure.

**Interfaces:**
- Preserve the existing stages, Summary schema, unsupported-layout sentinel, timestamp/copy/navigation behavior and localized omission copy. Do not add a ledger UI or a new locale taxonomy.
- Keep opt-in `summary-evidence-qa` and the same inner runner for observed/default jobs. Record only allowlisted point/plan/occurrence/report/composed artifacts. Identify E by original packet source IDs and V by occurrence ID sets, not differing follow-up context or feedback bodies.
- Extend manual acceptance records to `{ round, build, source, settings, formats }`; `build`, `source`, `settings` are nonempty fixed identities. `assessRuns` resets on any identity change, nonconsecutive round or failed/missing format/check. Preserve existing checklist names and the optional two secondary checks in custom mode.

- [ ] **Write compatibility and checker regressions.** Drive new backend stages through registered event handlers, retain old content during errors, and assert step4/4/cancel routing. Test another file's late ready/error events while a newer run is active. Verify absent/null/zero omission metadata shows no warning and positive metadata does. Task 2 tests the producer's unique-point count; do not recreate that algorithm in the frontend. Keep exact 12.75 seek and escaped source-clock copy tests.

```javascript
assert.equal(assessRuns([round1, round2, round3]).passed, true);
assert.equal(assessRuns([round1, failedRound2, round3]).consecutive, 1);
assert.equal(assessRuns([round1, changedSettingsRound2, round3]).passed, false);
```

Use literal synthetic records, including unknown/missing checks and source/build changes. Observer-on/off jobs must produce identical requests and text. A dummy credential sentinel must be absent from every artifact, with no credential/header/JobInput properties outside the allowlist; ordinary transcript wording about an API key remains valid source data. A requested context follow-up must not increase V or its ceiling as if it were another batch.

- [ ] **Observe RED where changes are required.** Run focused Vitest files, `node --test tests/release/evidence-acceptance.check.mjs`, and `cargo test --features summary-evidence-qa --test summary_evidence_pipeline`. Watch the new identity/counting assertions fail before modifying tooling. Existing compatible behavior may remain green; document it rather than fabricating a red.
- [ ] **Adapt the tooling, not product behavior.** Capture the lean stages/points/occurrences and preserve the ignored harness's consent/destination/new-output-directory guards. Keep the saved key only in Rust memory. The manual checker trusts human comparison and is not a keyword accuracy oracle. Do not change production preferences to run tests.
- [ ] **Verify GREEN.** Run `npm run typecheck`, `npm run lint`, `npx vitest run`, the Node checker tests, default `cargo test`, and QA-feature pipeline tests. Expected: all default suites pass; observer equivalence and identity/count safeguards are exercised, not skipped. Record unrelated opt-in skips honestly.
- [ ] **Checkpoint.** Review compatibility/tooling changes; use a separable `test: preserve grounded summary acceptance invariants` commit or a ledgered local checkpoint.

## Task 5: Full gates and unchanged real-file acceptance

**Files:** update `PLAN.md`, `CHANGELOG.md` (Thai), `design-qa.md` with actual results; private output/checklists remain outside Git. Consume tasks 1–4's production runner and harness.

**Interfaces:** Keep `live_deepseek_all_three_formats` and its explicit `VELO_FIDELITY_LIVE=deepseek`, transcript/settings paths and new output-directory guards. The harness must exercise the same production inner runner, not a special semantic path.

- [ ] **Run all gates fresh before cloud acceptance.** From the appropriate repo directory run `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `npm run typecheck`, `npm run lint`, `npx vitest run`; also fmt check, QA-feature all-target clippy, QA-feature pipeline tests, Node checker tests, `npm run version:check`, and `git diff --check`. Supply the installed local Whisper model and non-silent synthetic audio where present. Expected: exit0, version0.8.0; explicitly distinguish ignored/early-return download, Ollama and archive tests from actual exercised tests.
- [ ] **Build the default application separately.** In the ignored execution workspace, use `apply_patch` to create `preview.json` containing only `{"productName":"Velo Simple Grounding Preview","identifier":"com.velo.simple-grounding-preview"}`. From the repository root run `npm run tauri build -- --bundles app --target aarch64-apple-darwin --config "$velo_preview_config"`, where `velo_preview_config` is the verified absolute path to that file. Expected: default-feature bundle succeeds; the existing app/session and `/Applications/Velo.app` remain untouched. A QA-feature-only build is insufficient.
- [ ] **Freeze provenance and private inputs.** Use `mktemp -d /private/tmp/velo-simple-grounding.XXXXXX` for a fresh acceptance root. Resolve the original production transcript by its existing `85d6d140adb2920e` cache suffix, verify 53 segments and SHA-256 `47d6de4119b81350585d54c4e8e0b52ce853f8b212d52f4fcd026e67189d5a15`, and confirm the same 3:44 offered recording path. Do not use the diagnostic re-transcription. Snapshot current settings/summary hashes, preserve meeting/key-point focus and saved DeepSeek destination/model/context, and compute a dirty-code identity from sorted runtime/test/config paths plus their bytes, not HEAD alone. Store all provenance privately; none of these checks invokes the provider.
- [ ] **Run and inspect complete round1, then round2, then round3.** Set `VELO_FIDELITY_LIVE=deepseek`, `VELO_TEST_TRANSCRIPT` to the verified original cache, and `VELO_TEST_SETTINGS` to the frozen private settings snapshot. For each round set `VELO_FIDELITY_OUTPUT` to the not-yet-created `round-1`, `round-2` or `round-3` child of the new acceptance root, then run from `src-tauri`:

```bash
cargo test --features summary-evidence-qa --test summary_fidelity_live live_deepseek_all_three_formats -- --ignored --nocapture
```

Use only the saved `api.deepseek.com` key in memory. If macOS requests authorization, the user handles that prompt; never export/read it through shell tooling. Meeting/key-point use the frozen current focus. Custom uses exactly `สรุปเป็นย่อหน้าสั้น ๆ ไม่เกิน 150 คำ ไม่ต้องมีหัวข้อหรือรายการ`. No production cache/settings write or UI automation that closes the user's session.

Compare every selected point, required phrase, placement verdict and final paragraph/bullet against the original raw source before counting the round. All formats must retain requested person/group choice, person as the default, completed review as the removal trigger with its question/uncertainty, and backend read/unread retention. Meeting/key-point also retain the shared review box and actual-reviewer record. Reject implementation-order/status/owner/date/permission inventions and inferred open questions. A vague result omitting difficult required topics fails.

Verify every application clock against its supporting source, no internal ID leaks, and no heading/list in the custom paragraph. Count custom words with `Intl.Segmenter('th', {granularity:'word'})`, `isWordLike` only, after removing application citations; record Node/ICU. Require <=150 for this fixture, not a general runtime word-count claim. Record each job's E/V/logical requests/ceiling/time and all failures, not an accuracy percentage.

- [ ] **Enforce the acceptance boundary.** The manual checker must report three consecutive complete same-build/source/settings rounds, nine successful jobs. A failure resets to zero; a code change requires relevant gates and a fresh sequence. Retain failed outputs. A known meaning error surviving review stops further live calls for a precise report; do not extend calls, alter the checklist or combine different builds. A harness exit0 is necessary but not semantic sign-off.
- [ ] **Verify preservation and final handoff.** Recheck production hashes and app session, inspect the omission/error/progress states without another unapproved cloud request, and record exactly which checks were native, browser-only or mocked. Update durable docs with actual code identity, counts, skips, outcomes and unresolved limits; mark the superseded ledger runtime/history clearly. Retain private artifacts/backups until recovery no longer depends on them.
- [ ] **Complete one fresh final review after all tasks.** Native's required reviewer examines the actual final working tree, spec, this plan, ledger rulings and the private acceptance evidence. Do not dispatch implementer/per-task agents. Re-grade findings by user effect; fix Important/Critical once with RED→GREEN and the full suite. Any final code change invalidates earlier nine-job acceptance and requires a new unchanged-build sequence before a completion claim. No second review loop, push or release.

## Plan Self Review and Handoff

- Source/schema/packet requirements map to task1; formats, required phrases, placement eligibility and literal rendering to task2; model/context/budget/atomic-save behavior to task3; compatibility/privacy/provenance counting to task4; real nine-job acceptance and preservation to task5.
- All five Review Focus cases have explicit owner tests. Synthetic verdicts test local enforcement; only the original-recording comparison can establish this fixture's semantic acceptance.
- The user has already selected Native. Review this written plan for scope and correctness; after confirmation preserve that method and execute tasks 1–5 in order. Plan approval is not an automatic release or permission to weaken a failed semantic gate.
