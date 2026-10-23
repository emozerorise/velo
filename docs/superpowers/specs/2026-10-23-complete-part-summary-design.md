# Complete-Part Long Summaries — Proposed Design

Status: approved by the user on 2026-10-23 for a Native sequential
implementation plan. This approval is not permission to implement or run new
paid tests yet. No cache replacement, Preview restart, commit, push or release
in the specification/planning task.

## Intent and current evidence

Produce a useful long summary without requiring the model to write, repair
or review the entire final document in one response. Keep the unchanged
transcript, original source clocks, the selected format and material conditions.
Never continue an interrupted generation or present an incomplete job as saved.

The current direct pipeline reads source chunks, writes/reduces working notes,
then asks for one global answer. Exact request sizing removed an unnecessary
reduction loop, but the real `voice_214468.aac` custom case still reached both
4096 and 8192 output limits during citation repair. Increasing the input window
does not remove that output bottleneck. See the final checkpoint in
`design-qa.md`; the earlier detailed-meeting plan's task 4 remains incomplete.
The previously generated detailed result is not acceptance of this new design.

## Constraints and non-goals

- Keep the direct model-written path; do not restore Evidence Ledger,
  claim extraction, claim verdicts or a quotation-only composer.
- Every original transcript row must be assigned to a source-owning part.
  Summarising is selective, but the application must not silently skip source
  rows, cut their text or replace them with generated notes as the only input.
- Preserve proposals versus agreements, defaults versus implementation order,
  conditions versus effects, uncertainty and later explicit changes. Missing
  information is not automatically an unresolved question.
- Keep provider dialects, credentials, context settings and existing output
  ceilings. The DeepSeek recovery ceiling stays at most 8192, not unlimited.
- Keep `meeting` (standard/detailed), `key_points` and `custom`. Do not force
  meeting headings into custom output or modify the user's saved prompt.
- Retain existing player/UI work, no-autoplay behavior, old cache compatibility
  and isolated Preview/production data. No new renderer, dependencies, public
  claim schema, resumable-job storage or chat/library redesign.
- Reading all source and validating clock membership are mechanical properties,
  not proof that every material point was selected or every sentence is true.
  AI review remains fallible; real semantic acceptance is still required.

## Approaches considered

1. **Source-ordered complete parts, application assembly — selected.** Coverage
   is mechanically trackable and each response has bounded writing scope.
   Distant corrections need an explicit cross-part review; repeated topics may
   remain in more than one chronological location.
2. **Topic-routed writing.** Can produce smoother thematic prose, but adds a
   global topic-assignment step and opportunities to omit/misroute source.
   The user selected approach 1 instead.
3. **A larger final response or another full-document AI merge.** Retains the
   same output-limit failure and is not a fallback in this design.

## Source snapshot and partitioning

Freeze the transcript, effective settings, vocabulary and language once per
job. Give rows their existing stable `S000001` identities; never renumber a
later excerpt. Quote row text exactly as today so embedded newlines cannot
create source IDs or instructions. Do not change audio or transcript caches.

Each part owns a disjoint contiguous row interval. The ordered union of those
intervals must equal the whole snapshot, with no gaps or double ownership.
Include up to two complete rows on each side as boundary context when they
fit. Context rows are not owned a second time. A part writes about its owned
material; it may cite context necessary to explain that material.

Initial packing uses a soft 24 KiB quoted-source target, reduced when the exact
mandatory request builders require less room. It is not a duration boundary.
Fit checks include system instructions, layout, allowed IDs, review/repair
instructions, JSON escaping and recovery output allowance. A single oversized
row is never cut: fail clearly if it cannot fit whole in a required request.

For an input requiring multiple parts, write final-form parts directly from
these original excerpts: do not first condense the whole recording into one
set of notes. A small input may keep the existing direct path. If that path
exhausts output recovery, discard its candidate and enter partitioned writing
only if its owned interval can actually be split.

## Writing and accepting one complete part

A part is an internal source interval plus generated prose, not a fact ledger.
The model receives the selected instructions, original owned/context rows,
the exact available IDs and a request for a self-contained, finished part.
No introductory repetition or ending that requires a later continuation.

1. Draft the part with the existing final-answer output cap.
2. Apply at most the existing one citation-format repair if required.
3. Review against the same original excerpt, not just the generated draft.
4. Require an explicit successful provider finish, valid references, supported
   Markdown structure and the selected part layout before accepting it.

A source interval containing only irrelevant/filler material may return one
explicit no-content marker instead of invented prose. The same source-based
review must confirm that outcome; it is an internal empty part, not an omitted
source interval or a factual statement shown to the reader. It still counts
toward part/work limits and original-source sweep coverage. If every part is
empty, fail with no supported summary rather than overwrite a real cache.

Each of those prose requests retains at most one fresh output-limit retry.
An output-limit reply is never parsed as a complete part, cached, appended to
the next request or supplied as a continuation prefix. Nonempty EOF is not a
successful finish. A terminal marker alone does not certify semantic coverage.

If output recovery still fails during initial part writing, discard the
parent's generated material, bisect its owned interval at a row boundary and
write both children afresh from original source. Keep complete siblings only
inside this job. Maximum split depth is three per initial interval. Recheck
ownership/coverage and all request budgets after splitting. A one-row part,
depth/part limit or unsplittable required context fails the job; never solve
it by dropping source or saving the successful siblings alone.

Authentication/network errors and terminal citation/layout failures do not
trigger source splitting or an unrelated retry loop. Resolve all initial
splits before cross-part review so the review operates on stable part IDs.

## Cross-part conditions and later changes

Adjacent overlap alone cannot catch a decision changed an hour later. Add a
bounded review sweep over the **original source**, including distant windows.
Do not rely on keyword matching, cited rows alone, a last-few-minutes heuristic
or a compact global summary to select which source gets read.

### First sweep: identify the parts needing replacement

- Freeze the complete draft parts as version A. For each original source
  window, examine all draft parts in batches sized by the exact request guard.
  Every source-window/draft-batch pair must finish; the application tracks
  this coverage independently of model replies. The source windows cover every
  original row, including rows the drafts did not cite.
- Supply draft text as untrusted candidates, not evidence. Ask only whether
  that source window requires a material correction to a candidate: an omitted
  condition, a later explicit change, an answered question wrongly left open,
  or an unsupported interpretation. Do not flag mere stylistic preferences.
  Absence from this window is not disproof: a different window may support
  the candidate. A flag needs relevant source content that qualifies, answers
  or conflicts with it, not merely failure to find all its claims locally.
- Return a small structural routing response containing only supplied part
  IDs needing re-examination, or an explicit empty list. No extracted claims,
  evidence scores, fact records or approval certificates. Reject unknown IDs,
  malformed data and incomplete responses. A routing response cannot delete
  prose or change a citation by itself.
- Associate flagged part IDs locally with the full original source window
  that caused the flag. The model cannot nominate an unseen time/row as a
  replacement. Union the flags from the complete sweep before rewriting.

For each affected part, write one fresh complete replacement from its original
owned/context excerpt **and all flagged original windows**, with their actual
chronology. Review and validate that replacement using the same supplied source.
Do not retain a truncated replacement or edit only a sentence using a guessed
clock. If this union cannot fit whole, stop with a context error rather than
silently cutting context or selecting a convenient subset of flags.

Replacements preserve ownership and layout. They may cite nonlocal rows that
support a correction. Retain unaffected material reasons and conditions;
replacement is not permission to condense away the rest of an owned interval.
Mark an earlier view as earlier when later material changes it; do not leave
the old view as an unqualified final decision. State
unresolved conflict only where the source genuinely leaves it unresolved, and
cite both sides when needed. A later statement wins only when it explicitly
changes the same matter, not merely because its timestamp is newer.

### Second sweep: stability check, not another rewrite loop

Freeze the replacements as version B. Repeat the complete source-window/batch
coverage against version B, including the final overview described below.
If any material correction is still flagged, fail without saving. There is
no third sweep, new rewrite cycle or automatic paid rerun. A replacement that
exhausts its existing output/citation recovery also fails; no new part splits
are introduced after the first sweep.

This is a finite direct-prose review with source-window routing, not Evidence
Ledger. It still adds model calls and cannot guarantee detecting every remote
condition. An empty routing list means the model requested no change, not that
the application proved the draft true. Tests and manual source comparison must
expose false negatives, false positives and missed later resolutions.

## Format-preserving assembly

All accepted content stays complete prose. Assembly is local, deterministic
and tested; there is no final AI call that rewrites the full document.

**Meeting:** each part uses the existing supported sections, omitting unsupported
ones. The application groups exact section identities once and preserves source
order within each section. Detailed discussion stays expanded; standard reader
ordering stays unchanged. Do not invent headings, decisions or empty tasks to
fill the template. Section membership is an AI judgment, not a local fact check.

**Key points:** collect complete point blocks in source order without adding
meeting-only outcomes. Do not impose a new fixed point count.

**Overview:** meeting/key-point parts may provide short cited local overviews.
After first-sweep replacements, compose the small global overview once from
the updated complete local overviews, with the existing review/reference
contract. It is not a rewrite of the body. Its complete source must fit; no
recursive overview reduction. Include it in the second original-source sweep.
Failure leaves the job unsaved;
do not silently omit a requested overview to fit. Notes here are an overview
input only, never a substitute for reading original body source.

**Custom:** establish a small layout-only contract once, before writing, using
the user's unchanged instructions. A bounded model response may select a
single paragraph, paragraphs, bullets or an ordered set of named Markdown
sections; it contains layout metadata only, no meeting facts or mandatory
meeting categories. Parse/validate it locally. Unrepresentable or ambiguous
requirements fail explicitly rather than silently switching to meeting mode.
Do not add table/HTML rendering or promise arbitrary word-count enforcement.

- A one-paragraph contract requires each part to contain finished sentences
  without headings/lists. Join with spaces, retaining inline references at
  their original sentence positions. Never insert connecting factual prose.
- Paragraph/list contracts preserve complete blocks. Named sections use the
  one frozen order; omit unsupported sections and never create extra headings.
- Instructions about global brevity or counts must not silently become a
  per-part quota. The implementation plan must test the existing examples;
  if a strict global requirement cannot be fulfilled without another global
  content-selection mechanism, reject it clearly, not falsely claim compliance.

Remove a duplicate factual block only when its text and canonical supporting
ID set and destination section are identical. Do not semantically deduplicate
distinct times, silently delete a later change or use generated paraphrases to
decide two conditions are equivalent. Repeated topics are an accepted
readability tradeoff.

Validate references per part, after replacements and after assembly. Joining
a custom paragraph must not allow an uncited child to hide behind a different
child's citation. Expand ranges only when every member was actually supplied
to that request. Resolve each final clock from the original snapshot row;
never use nearest-time matching, renumber parts or accept model-written clocks.

## Finite work and cost boundaries

The following are proposed engineering defaults, not performance claims:

- At most 32 terminal source-owning parts, including empty ones, and split
  depth 3 per initial interval.
- One first cross-part sweep, one replacement per affected part, one second
  sweep; no convergence loop. Routing responses carry at most 32 part IDs.
- Routing/layout requests have a 1024-token ceiling and no automatic retry
  on malformed, incomplete or output-limit responses. Pack smaller routing
  batches by measured input size before sending, never truncate a response.
- Prose requests retain the existing draft/repair/review rules and one fresh
  output-limit recovery each, including overview/replacement requests.
- At most 128 physical provider requests per job, counting every failed
  request, recovery, routing call, layout call and overview. No hidden reset
  when entering partitioned mode. Check cancellation and remaining budget
  before every request; reserve enough for the known mandatory remaining work.
  If a new split/review schedule cannot finish inside that budget, stop.
- Sequential execution; no model-call parallelism, change of provider,
  context/output-cap escalation or automatic job restart to get a lucky pass.

Routing cost can grow with source windows times candidate batches. Measure
and report planned/actual call counts in local tests before live acceptance.
These limits may fail dense 1–3-hour material safely: duration alone cannot
guarantee success, cost or response time. Budget failure must not be reported
as a completed summary. Revising these limits requires a recorded decision,
not an ad hoc retry while billing the real-file test.

## Job state, UI and persistence

Keep source intervals, part versions, layout and review coverage in memory
only. No new persisted ledger or partial-summary cache. Numeric diagnostics
may record phase/count/byte/cap information, not transcript text or secrets.

Show localized progress for writing part i/N, checking cross-part consistency,
assembling and checking references. Counts may change after an initial split;
do not pretend progress is a fixed percentage or that parts are saved results.
Retain cancellation and the previous-chat separation work already approved.

Do not publish part deltas as the completed summary or change the chat's saved
summary context mid-job. The old complete result stays active until final
assembly, both sweeps and reference/layout validation succeed. On failure with
an old result say, for example, “สรุปใหม่ไม่สำเร็จ กำลังแสดงฉบับเดิม”; without
one say the new summary was not saved. Distinguish output, context, review,
work-budget and provider errors in Thai/English without implying factual proof.

Check cancellation before/after HTTP, between parts and immediately before
the existing atomic summary save. Cancellation acknowledged before commit
leaves the old cache untouched; after a successful commit report success, not
a false cancellation. Save one normal `Summary` with effective format/detail
metadata only at the end. Old summaries remain readable and are not migrated
or regenerated on opening the app. A failed write must retain the old result.

## Acceptance and later execution boundary

This document authorizes no new execution or paid test. Following approval,
write a Native sequential implementation plan before product changes. Preserve
the dirty worktree and earlier tasks; do not mark their integrated handoff done.

Local acceptance must exercise the production runner with a loopback model:

- 1-, 2- and 3-hour fixtures across standard meeting, detailed meeting, key
  points and custom one-paragraph output. Assert exact owned-row coverage,
  unchanged source text and measured draft/review/repair/routing request fits.
- Output limits during draft, repair, review and replacement; fresh retries
  never include partial text. Splits are gap-free, finite and discard parents;
  one-row/oversized context/depth/part/call limits fail without losing source.
- A condition across adjacent boundaries and another several windows away;
  a late reversal, a later answer to an early question, a genuine unresolved
  contradiction and a later unrelated statement that must not override it.
  Assert full cross-review coverage, exact source inputs and corrected prose.
- Routing invalid IDs, malformed/truncated/empty responses, false flags and
  second-sweep instability; an explicit valid empty list differs from an empty
  response. Absence in one window must not invalidate claims supported in
  another. No accepted partial sweep or unbounded retries.
- Filler-only parts retain source coverage without invented content; an
  all-empty job cannot replace a saved result. Replacements retain unrelated
  material conditions as well as the corrected issue.
- Global overview versus updated body, exact-only deduplication, nested
  headings, custom no-heading/one-paragraph assembly, existing prompt examples,
  independent child citations and hour-scale source-clock navigation.
- Cancellation at each transition and cache write failure with/without an
  existing result; no complete event or previous-summary/chat mutation early.
- Legacy metadata, localized progress/error copy, timestamp off/on and copy,
  narrow-window/keyboard controls and the existing no-autoplay behavior.

Run `cargo fmt`, strict all-target Clippy, `cargo test`, `npm run typecheck`,
`npm run lint` and `npx vitest run`; retain release-policy/version checks and
build the isolated Preview before any handoff. Mock passes prove orchestration,
not model quality. Report gated/skipped work separately.

After explicit execution/API approval, test the unchanged 2322-row cache of
`voice_214468.aac` in fresh isolated directories with the saved provider/model.
Run custom first (the known failure), then standard meeting, key points and
detailed meeting once each on the same frozen summary implementation. Fail
fast and stop billing on a failed case; do not silently omit a required case,
reuse an old-format success or rerun repeatedly. Record requests, outcome,
source/build hashes, layout and original-clock validation separately from
manual review of material conditions and later changes. Dense/unclear ASR
must remain qualified; clock membership alone is not semantic acceptance.

Open the new Preview only after all required gates pass. Any Preview cache
backup/replacement remains subject to the user's existing conditional handoff
approval; do not touch production data. No push, release, Latest or updater
publication is included.

### Targeted diagnostic implementation clarification

Within each of the same two sweeps, a partial-window routing signal is
provisional. Confirm it once per affected part against complete original
supporting/owned rows and whole flagged windows before replacement or rejection.
The overview uses its cited original rows with boundary context, not generated
notes. Confirmation cannot truncate source, spend caller-reserved requests,
exceed the shared128 physical-call ceiling or accept invalid routing. It does
not introduce a third rewrite/retry loop; a confirmed material flag in sweep2
still fails immediately. Empty confirmations must not bypass remaining pair
coverage. This scoped repair follows the user-approved targeted diagnosis;
captured unrelated-window false positives are not a reason to lower fidelity.

## Native design self-review

Reviewed against the selected approach, current pipeline/validator/store and
the last real failure. Native sequential work is retained; no delegated
implementation or reviewer was dispatched. The approved engineering choices
are the two bounded cross-part sweeps, the layout-only custom
contract, the finite limits above and failing safely instead of dropping
required context. Internal type names, module split and fixture organisation
are left to the implementation plan. None of the proposed behavior is claimed
implemented, tested or semantically certified by this document.
