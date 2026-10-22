# Source-Bound Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans for the user-selected Native execution, task by task with RED→GREEN evidence and one fresh final review. Steps use checkbox (`- [ ]`) syntax for tracking. The user approved this written plan on 22 October 2026; resume its Native ledger rather than repeating completed tasks.

**Goal:** Preserve protected source phrases through typed extraction references without weakening semantic review, exact citations or the original nine-job acceptance gate.

**Architecture:** Decode strict model-facing templates into the existing canonical `DraftPoint` inside the evidence module. Replace only the extraction wire seam, then reuse whole-point layout, placement review and local composition/save without another model stage or production fallback.

**Tech Stack:** Rust, existing serde/serde_json, Cargo and HTTP fixtures; existing Vue/Vitest and Node acceptance checks. No new dependency.

**Spec:** `docs/superpowers/specs/2026-10-22-source-bound-extraction-design.md`, written approval confirmed 22 October 2026; inherits unchanged requirements from `docs/superpowers/specs/2026-10-21-simple-grounded-summaries-design.md`. Native is already selected. This plan addresses the remaining extraction/acceptance blocker, not the completed unrelated tasks of the earlier plan. Current baseline: no successful semantic round.

## Global Constraints

- Preserve all dirty/untracked work on `codex/summary-prompt-controls`. No push/PR/merge/tag/release, version bump, installed-app replacement or closure of the running Preview; keep version 0.8.0.
- Keep provider/model/destination, credentials, permissions, source bytes/order/fractional times, production preferences/caches, UI, font, chat, microphone and player unchanged.
- Wire points contain exactly `parts`, `evidence`, `required`; parts are exactly `{ "text": string }` OR `{ "required": unsigned integer }`. Every declared required slot is referenced exactly once within its own point. No legacy-text fallback, regex substitution, fuzzy match or silent insertion/drop.
- Canonical points remain `DraftPoint { text, evidence, required }`. Exact unique anchors must belong to supplied raw sources and required anchors must be covered by evidence under the same source ID. Duplicate anchors within either array fail; the same anchor may appear once in each array.
- Concatenate literals and resolved raw spans in order, without adding separators, translations, corrections or labels. Quotes locate text; they do not certify meaning. Keep every existing semantic rejection and whole-occurrence review.
- Extraction remains at most twelve segments and 3000 serialized-source UTF-8 bytes per packet, up to two-segment overlap with forward progress, and at most six points. No truncation of sources, points or conditions.
- Check the complete actual request against `(context_tokens - max_tokens) * 3`, including the largest bounded feedback envelope. Preserve output/transport policies and indivisible-source/catalog errors.
- One initial extraction call plus one structural repair; layout/review slots unchanged, review follow-up OR schema repair. Logical ceiling `2E + 2V + 2`; no semantic rewrite/retry-until-success loop.
- Public commands, cache/Markdown shape, optional omission metadata, progress, cancellation/job lease, exact seek/copy and single atomic save remain unchanged. Failed/exhausted jobs preserve the old summary.
- Keep keys only in Rust memory; no key/header/JobInput artifacts. Capture raw QA output only in explicitly opted-in private directories. Keep all failed outputs/backups and normal evidence only in job memory.
- Require three consecutive complete three-format rounds on one unchanged build/source/frozen-settings identity. No existing failed job counts, relaxed criterion or omitted difficult topic. A known meaning error surviving review stops further live calls.

## Review Focus

1. A literal that resembles a slot, source ID, clock, heading or instruction must remain data; identical numeric indices in different points must resolve locally. Task1 pins this.
2. Thai/multilingual framing, adjacent parts and split literals must preserve exact protected bytes without invented punctuation, duplicated slots or changed deduplication. Task1 pins byte assembly; task2 pins canonical deduplication.
3. Correct quoted phrases inside invented assertions, misleading decision headings or late-answered questions must not bypass complete-statement review. Task2 pins this; real entailment remains task3's human gate.
4. Multiple simultaneous structural faults, oversized output and a tight repair budget must fail deterministically without a third call, partial publication or truncation. Tasks1–2 pin this.
5. Observation, cancellation, late-file events or changing runtime/source/settings must not leak credentials, overwrite an old/newer job or certify mixed acceptance runs. Task2 retains the regressions; task3 verifies provenance.

---

## File Ownership and Execution Setup

- Create `src-tauri/src/summary/evidence/extraction.rs`: private wire types, pure decode/assembly, bounded structural feedback. Register through `evidence/mod.rs`; no module outside summary evidence imports its wire types.
- Modify `evidence/stages.rs` and `prompts.rs`: connect the adapter and new extraction schema. Keep `source.rs`, `model.rs`, layout/admit/compose and `pipeline.rs` behavior unchanged; adjust only canonical test imports if the obsolete wire wrapper is retired.
- Modify `tests/support/evidence_fixtures.rs`, `tests/summary_evidence_pipeline.rs`, `tests/summary_review.rs` and evidence module tests: port only extraction responses, preserve canonical/reviewer fixtures and add adapter-boundary regressions.
- Retain `tests/support/evidence_acceptance.rs`, `tests/summary_fidelity_live.rs`, `scripts/check-evidence-acceptance.mjs` and its Node tests unless a demonstrated compatibility failure requires a scoped tooling change. No new artifact event, metadata field or QA-only semantic path is needed.
- Retain frontend product source. Verify `tests/unit/{summaryChain,summaryEvidenceStatus,summaryReading,summaryMarkdown,summaryNavigation}.test.ts`; change source only for a demonstrated regression within this contract.
- Task3 updates `PLAN.md`, Thai `CHANGELOG.md` and English `design-qa.md` with actual results, never private recording prose or keys.
- At execution start, resolve this plan's own ignored workspace with the Native skill. Record HEAD/branch/index state, interfaces and rulings there. Save and verify a recoverable archive of the current product sources/tests/untracked files plus the dirty diff; exclude Git, execution workspaces, targets, node_modules, production user data and credentials. Do not touch sibling plan workspaces or their backups. Reuse this checkout per the user's existing choice.
- Do not stage mixed pre-existing edits. Each task ends in a separable commit or verified uncommitted checkpoint with a ledgered reason. The current untracked evidence files contain prior user work; a whole-file add is not automatically separable.

## Task 1: Pure template decoding and exact source assembly

**Files:** create `src-tauri/src/summary/evidence/extraction.rs`; modify `src-tauri/src/summary/evidence/mod.rs`. Unit tests live in the new module. Keep the adapter test-only until task2 connects it.

**Interfaces:**

- Consumes existing `types::{Anchor, DraftPoint}` and `source::{SourceSnapshot, validate_point}`. `SourceSnapshot::resolve(&Anchor) -> Result<ResolvedAnchor>` supplies byte bounds; `segments` contains immutable raw text and `check_job(&str)` checks identity.
- Produces `pub(super) fn decode_packet(answer: &str, sources: &SourceSnapshot, supplied: &BTreeSet<String>) -> std::result::Result<Vec<DraftPoint>, ExtractionFailure>`.
- Produces opaque `pub(super) struct ExtractionFailure` with `pub(super) fn feedback(&self) -> serde_json::Value`, plus `pub(super) fn feedback_reservations() -> Vec<serde_json::Value>` containing a largest-field envelope for every feedback category.
- Private wire types are `ExtractionPacket { job: String, points: Vec<ExtractionPoint> }`, `ExtractionPoint { parts: Vec<TextPart>, evidence: Vec<Anchor>, required: Vec<Anchor> }` and `TextPart` with the two exclusive object forms in Global Constraints. Parse directly into strict typed structures, not through a Value that loses duplicate JSON fields.
- Feedback has a closed `category`, static `instruction` and only relevant numeric `point`/`part`/`required`/`anchor`/`actual`/`limit` fields, plus static `anchor_set` (`evidence` or `required`) for anchor location. No raw response, quote or credential echo. Categories are `schema`, `job_mismatch`, `point_limit_exceeded`, `invalid_anchor`, `required_not_covered`, `empty_parts`, `required_index_out_of_range`, `duplicate_required_reference`, `missing_required_reference`, `empty_text`, `canonical_point_invalid`. `limit` is 6. Reserve usize::MAX for numeric metadata using the same serializer/instructions as real feedback; actual job/source-ID lengths are already in the actual request body.

- [ ] **Write assembly goldens and rejection tables first.** `assembles_source_bytes_without_delimiters` uses a source at12.75 saying `Only when inspection finishes can the item be removed.`, required `Only when inspection finishes`, and parts reference0 + literal `, the item can be removed.`. Add Thai framing/combining characters, multiple required spans, literal-only ordinary points and same-index/two-point sources. Assert exact canonical bytes and unchanged source segments; the module never adds clocks.

```rust
assert_eq!(points[0].text, "Only when inspection finishes, the item can be removed.");
assert_eq!(points[0].required[0].quote, "Only when inspection finishes");
assert_eq!(sources.segments[0].start, 12.75);
```

`rejects_nonlocal_missing_or_duplicate_slots` declares required0 but supplies a paraphrased literal instead: expect `missing_required_reference`, required0, point0. Cover duplicate ref0, out-of-range ref1, negative/fractional/null references, mixed/unknown/duplicate JSON keys, absent required/parts, old `text` packets and attempted cross-point keys. An incidental matching literal does not satisfy a missing reference. Literal `{"required":0}`, fake `[S000099] [00:10]`, Markdown and instructions remain unchanged strings.

`preserves_anchor_and_failure_priority` covers unsupplied/unknown/ambiguous/duplicate anchors, uncovered required phrases, wrong jobs, empty parts, empty/whitespace-only assembled text and seven-point packets; same anchor once in each array remains valid. Assert first failure: schema → job → point count → points in wire order, then anchors/coverage → parts/slots → empty assembled text → defensive canonical validation. Empty filler packets remain valid; final no-content failure stays in the runner.

```rust
assert_eq!(failure.feedback()["category"], "missing_required_reference");
assert_eq!(failure.feedback()["required"], 0);
assert_eq!(oversized.feedback()["category"], "point_limit_exceeded");
assert_eq!(oversized.feedback()["actual"], 7);
```

`feedback_reservations_cover_actual_failures` constructs every category, including maximum numeric locations, and compares actual serialized feedback against its matching reservation. Assert bounded known keys, static instructions, limit6 only for overflow, and no response/quote/source-text fields. Every actual feedback envelope must fit its category reservation; different categories need their own lengths accounted for.

- [ ] **Observe RED.** Register `#[cfg(test)] mod extraction;`; run `cargo test --lib summary::evidence::extraction` from `src-tauri`. Expected: new assertions fail before implementation. If declarations are missing, add only testable signatures/stubs and observe behavioral failures; retain the log. No production runner switch yet.
- [ ] **Implement the pure adapter.** Enforce the exact priority and category metadata above. Resolve each protected span from the original source string at valid byte bounds, concatenate in supplied order, then call `validate_point` before returning any packet. Never add missing refs, discard bad points, normalize bytes or expose wire types downstream. Feedback reservations enumerate all categories with bounded maximum metadata, not an unbounded list of errors.
- [ ] **Verify GREEN.** Run `cargo test --lib summary::evidence::extraction`, `cargo test` and `cargo clippy --all-targets -- -D warnings`. Expected: adapter cases and existing suites pass, no new warnings; opt-in skips are recorded, not described as exercised model tests.
- [ ] **Checkpoint only separable work.** Suggested commit `feat: assemble protected summary phrases from source references`; otherwise verified archive + ledger ruling under the setup constraint. Do not include unrelated user edits.

## Task 2: Connect the single extraction seam and prove compatibility

**Files:** modify `src-tauri/src/summary/evidence/{mod,stages,prompts,types,source}.rs`, `src-tauri/tests/support/evidence_fixtures.rs`, `src-tauri/tests/{summary_evidence_pipeline,summary_review}.rs`. `types.rs`/`source.rs` changes are limited to retiring the obsolete packet wrapper/test imports. Consume task1's adapter; retain downstream production behavior.

**Interfaces:**

- Keep `extract_points(&SourceSnapshot, &EvidenceModel<'_>) -> Result<Vec<SourcePoint>>`, `prompts::points(&SummarySettings, &str, &str) -> String`, `pipeline::{run,run_observed}`, observer events and public Summary unchanged.
- `wire::point_packet() -> Value` becomes a new-wire literal-only packet yielding the same `They requested a group option.` canonical text. Add `wire::canonical_point() -> Value` for tests needing an internal DraftPoint and `wire::protected_point_packet() -> Value` using the task1 conditional fixture. Keep `point_plan()`, `point_report()`, `server()` and `server_gated()` signatures unchanged.
- `stages` uses `decode_packet` for each reply and `failure.feedback()` for its one repair. Its preflight tries every `feedback_reservations()` envelope in the actual extraction body with `EvidenceModel::fits`. Exact canonical deduplication/ID assignment remains in stages.

- [ ] **Write wire and integration failures before switching production.** `assembled_phrases_reach_planner_reviewer_and_cache` supplies the protected template, no-heading custom paragraph and supported raw-source report; use start12.75. Assert exactly3 requests, canonical text/required anchors in planner and reviewer, original raw text in review, and final unchanged sentence + ` [00:12]`. Assert no `parts` field survives into canonical points.

`extraction_repairs_missing_slot_once_without_dropping_protection` ports the old missing-verbatim-copy test: use `Please add a group option.`, required `Please add`, bad literal-only parts and corrected ref0 + literal ` a group option.`. Assert exactly4 calls and second payload repair category `missing_required_reference`, point0/required0. Seven-point output must get `point_limit_exceeded`/actual7/limit6; a second invalid response returns error with old-cache bytes intact and exactly2 extraction calls, never a first-six subset.

```rust
assert_eq!(requests.len(), 4);
assert_eq!(payload(&requests[1])["repair"]["category"], "missing_required_reference");
assert_eq!(summary.markdown, "Please add a group option. [00:01]");
assert_eq!(store.load("/synthetic.wav").unwrap().markdown, summary.markdown);
```

Use a paragraph-only layout for that exact golden. Port the old job-repair assertion from a string to the new category object, but retain existing layout/review feedback behavior.

`assembled_quotes_do_not_certify_invented_outcomes` puts a protected conditional/question inside an invented agreement, receives an unsupported placement verdict, and preserves the old cache. Also retain one supported overview plus rejected decision occurrence (omitted0 for that selected point), all-rejected failure, distant answered question/raw follow-up and no rewritten text. Synthetic verdicts establish routing only, not semantic truth.

`extraction_reserves_every_feedback_before_http` replaces the obsolete source-sized duplicate-quote reserve test. With an indivisible source, choose a small integer context via the existing completion_request output budget so the initial request fits but the largest candidate repair body does not; assert no HTTP/request artifact and a context error. A neighboring fitting context must run the real repair and stay within the serialized byte budget. Keep twelve-segment/3000-byte overlap, tail coverage and unsupplied-source regressions.

- [ ] **Write transport/observer compatibility assertions before the switch.** Port the existing nine provider/format cases, cancellation inside each awaited HTTP call and before save, bounded repairs, unsupported layout and cache tests. Retain Ollama think=false, DeepSeek thinking disabled and existing output limits. Extend observer/default equivalence to a protected template: identical requests after job normalization, identical composed output, raw response containing parts and canonical points containing text/anchors only. Preserve E/V set counting, exact metadata allowlist and absent dummy credential sentinel; no new event is needed. In `extraction_overlap_deduplicates_only_identical_text_and_anchors`, split the second overlapping response's literal into two parts with the same canonical text/evidence/required as the first: expect2 points across the original3 packets, not3, with stable P000001/P000002 IDs.
- [ ] **Observe RED.** Run `cargo test --test summary_evidence_pipeline --test summary_review`, `cargo test --lib summary::evidence::stages` and `cargo test --features summary-evidence-qa --test summary_evidence_pipeline`. Expected: the old wire rejects typed references or feedback/observer assertions fail. Keep the evidence; do not accept the old prose format as a fallback to turn tests green.
- [ ] **Switch extraction atomically and port fixtures.** Register extraction for production, use the adapter/reservation interfaces and teach `prompts::points` the actual current job, exclusive part objects, local zero-based slots, exactly-once protection and no automatic delimiters. Keep custom steering out of extraction/review. Remove `required_quote_repair` and extraction-only `REPAIR_POINT`; retain shared schema/job/layout/review constants where still consumed. Retire `PointPacket` only after its runtime/test references migrate; canonical DraftPoint tests remain. Replace fixture mutations of wire `text` with `parts`; use `canonical_point()` in catalog/layout unit tests. No old-wire parser or second runner remains.
- [ ] **Run compatibility GREEN.** Run `cargo test --lib summary::evidence`, `cargo test --test summary_evidence_pipeline --test summary_review`, `cargo test --features summary-evidence-qa --test summary_evidence_pipeline --test summary_review`, default `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo clippy --features summary-evidence-qa --all-targets -- -D warnings`. From root run `npx vitest run tests/unit/{summaryChain,summaryEvidenceStatus,summaryReading,summaryMarkdown,summaryNavigation}.test.ts`, `node --test tests/release/*.mjs`, typecheck/lint and full Vitest. Expected: all exercised assertions pass, unchanged exact12.75 seek/copy, stage4/4/cancel/late-file and omission-count behavior. Do not change frontend product code without a demonstrated regression.
- [ ] **Checkpoint.** Suggested separable commit `refactor: use source-bound extraction in the summary pipeline`; otherwise verified uncommitted checkpoint with the same preservation ruling. Inspect that the only product runtime change is the extraction adapter/seam.

## Task 3: Fresh gates, default bundle and original-file 3×3 acceptance

**Files:** update `PLAN.md`, `CHANGELOG.md` (Thai) and `design-qa.md` after actual verification. Consume tasks1–2 and the existing opt-in live harness/checker; private provenance, checklists and outputs remain outside Git.

**Interfaces:** unchanged `live_deepseek_all_three_formats`, explicit `VELO_FIDELITY_LIVE=deepseek`, transcript/settings/output guards, saved-key read in Rust memory and manual records `{ round, build, source, settings, formats }` consumed by `assessRuns`. No special semantic runtime or automatic accuracy oracle.

- [ ] **Run fresh complete gates before any provider request.** From `src-tauri` run `cargo fmt`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo clippy --features summary-evidence-qa --all-targets -- -D warnings` and `cargo test --features summary-evidence-qa --test summary_evidence_pipeline --test summary_review`. From root run `npm run typecheck`, `npm run lint`, `npx vitest run`, `node --test tests/release/*.mjs`, `npm run version:check`, `git diff --check`. Supply the installed local Whisper model and the existing verified non-silent synthetic WAV through `VELO_WHISPER_MODEL`/`VELO_TEST_MEDIA`; do not retranscribe or replace the acceptance cache. Expected: exit0, version0.8.0, actual counts recorded; separate early-return/ignored network download, Ollama and signed-archive checks from real exercised cases. Local TCP bind denials require the same test under allowed loopback, not product patches.
- [ ] **Build the default-feature app separately.** In this plan's ignored workspace create `preview.json` with only `{"productName":"Velo Source Bound Preview","identifier":"com.velo.source-bound-preview"}` using apply_patch. Run `npm run tauri build -- --bundles app --target aarch64-apple-darwin --config "$velo_source_bound_config"` from root, with that variable the verified absolute config path. Expected: default-feature bundle succeeds, stays unopened/uninstalled, existing running/installed app untouched. QA-only compilation is insufficient.
- [ ] **Freeze private provenance and inputs.** Create a fresh root with `mktemp -d /private/tmp/velo-source-bound.XXXXXX`. Resolve the production transcript with cache suffix85d6d140adb2920e: require53 segments and SHA256 `47d6de4119b81350585d54c4e8e0b52ce853f8b212d52f4fcd026e67189d5a15`, same offered224.433333-second recording. Save a private settings snapshot with saved DeepSeek/deepseek-flash/context32768 and current meeting/key-point focus; no API key property/export. Record source/settings/summary/app hashes. Distinguish unrelated `updates.last_checked_at` activity from summary-setting changes without restoring production preferences.

Compute the dirty runtime/test/build/assets identity from sorted tracked plus nonignored untracked paths and bytes, not HEAD alone. Use the existing scope: `src/`, `src-tauri/`, `public/`, `tests/`, `scripts/`, `index.html`, `.cargo/`, package/package-lock and tsconfig/vite/vitest/tailwind/postcss/eslint configs. Hash path + NUL + byte length + NUL + bytes; record per-file digests. Exclude docs, Git, targets, dependencies and ignored execution scratch. Freeze the exact settings-byte hash separately. A documentation commit is not a new runtime identity; a changed source/test/config file is.

- [ ] **Run round1 and inspect it completely before round2, then round3.** For each not-yet-created `round-1`, `round-2`, `round-3` child of that root, set `VELO_TEST_TRANSCRIPT` to the original verified cache, `VELO_TEST_SETTINGS` to the frozen snapshot, `VELO_FIDELITY_OUTPUT` to that child and `VELO_FIDELITY_LIVE=deepseek`. Run from `src-tauri`:

```bash
cargo test --features summary-evidence-qa --test summary_fidelity_live live_deepseek_all_three_formats -- --ignored --nocapture
```

Expected for each job: actual production inner runner, valid bounded manifest and composed output for human inspection, never a fabricated pass from test exit0. Verify identities again before each subsequent round. The user handles any Keychain dialog; do not export/read the key through shell, ask for secrets in chat or start duplicate jobs while one waits.

Meeting/key-point use the frozen current focus. Custom uses exactly `สรุปเป็นย่อหน้าสั้น ๆ ไม่เกิน 150 คำ ไม่ต้องมีหัวข้อหรือรายการ`. Compare original raw passages, raw wire templates, canonical assembled points, selected headings/placements/verdicts and every final statement/citation. All formats must retain requested person/group choice, person as Default rather than development order, completed-review removal with its original question/uncertainty, and backend read/unread retention without unused-now inference. Meeting/key-point also retain the shared inbox and actual-reviewer record. No inferred agreement, status, owners/dates, permission scope or unresolved question. Unclear mechanisms may be omitted, not guessed; required difficult topics may not be omitted to pass.

Every application clock must support its statement and navigate to the original fractional start. Custom must be a single paragraph, no heading/list, at most150 Thai word-like units for this fixture: independently count Intl.Segmenter('th',{granularity:'word'}) entries with isWordLike after removing application citations, and record Node/ICU. Do not make a general runtime word-count guarantee.

- [ ] **Enforce the unchanged gate.** Record every criterion under existing names in each private acceptance.json, plus E/V/logical requests/ceiling/time and all failures. Run `node scripts/check-evidence-acceptance.mjs` with the three records in order. Expected: consecutive3/passedtrue only for nine successful jobs on the same build/source/settings, with human source comparison complete. Failed rounds reset the sequence; retain them. Do not automatically repeat failed jobs for a lucky result. Investigate locally before any bounded in-scope fix; a fix requires relevant full gates/bundle and a fresh unchanged-build sequence. A known meaning error surviving review stops further live calls for a precise report, not another prompt experiment or weaker checklist.
- [ ] **Verify preservation and record the real outcome.** Recheck source/summary/preferences/app hashes, allowing only independently identified unrelated updater timestamp activity, and confirm no production write/launch replacement. Record actual counts/skips, failures, artifact paths and unresolved limits in durable docs. Keep task3 incomplete if 3×3 fails; do not mark it done because code tests pass. No old failed run, schema success or canned reviewer verdict is semantic acceptance.
- [ ] **Checkpoint the verified result.** Complete task3 only after the current-identity acceptance checker and preservation checks pass. Use the checker command with the three verified private record paths as the Native task-done verification; it must report consecutive3/passedtrue. Keep overlapping documentation/user work uncommitted with a ledgered checkpoint if it cannot be separated. This is not permission to skip the final gate below.

## Native Final Gate (after tasks 1–3)

- [ ] **Complete one fresh whole-worktree review.** Use requesting-code-review's template through executing-plans, the most capable available reviewer model explicitly, and this plan/spec/Review Focus/ledger rulings. No implementer/per-task agents. The review package must include actual dirty/untracked product files and provenance, not only a docs-only committed diff. Review the whole working tree, preserve unrelated work, re-grade by user effect and fix Important/Critical once with RED→GREEN and full verification. No second reviewer loop.
- [ ] **Revalidate any fix before claiming completion.** Record acceptance invalidation in the ledger before a product fix. Such a fix invalidates earlier nine-job acceptance: rerun relevant full gates/default bundle and a fresh same-build3×3 sequence, without reimplementing completed tasks. Old task-complete evidence is historical, not current semantic sign-off. Keep the final gate open until the current build passes; a known surviving meaning error still stops calls. Retain backups while recovery depends on them, with a ledger ruling rather than destructive cleanup. No push/release.

## Plan Self-Review and Handoff

- Adapter schema/assembly/errors and Review Focus1–2 map to task1; single-runner integration/budget/privacy/cancel/review and Focus3–5 map to task2; gates/default build/provenance/nine-job comparison/preservation/final review map to task3. Existing downstream behavior is verified, not redesigned.
- Canonical DraftPoint is the shared boundary. Task2 consumes only task1's decode/feedback/reservation interfaces; task3 uses unchanged run/harness/checker interfaces. Synthetic and live evidence remain explicitly different.
- Nine successful jobs demonstrate repeatability on this offered fixture, not general model accuracy or a guarantee for arbitrary recordings.
- Native is already selected. Review this written plan and confirm it captures the approved amendment before execution. No need to ask the execution-method question again, and no implementation/provider call is part of this planning turn.
