# Simpler source grounded summaries

Status: written design approved by the user, 21 October 2026. Native planning,
additional DeepSeek calls for the offered recording, and the same three-format,
three-round acceptance target were confirmed. The implementation plan is also
approved; Native tasks 1–4 are complete. Task 5 live acceptance is blocked by
repeated extraction-contract failures after its bounded repair, confirmed on
22 October 2026. There are zero accepted rounds; no final text was produced.
No push or release.

## User outcome and scope

Produce readable meeting, key-point and custom summaries that preserve what
the recording says and link back to exact source times. The user needs that
outcome, not a comprehensive claim-classification system. Keep the approved
Thai font, chat, microphone, player, settings, provider and existing caches.

Replace the full Evidence Ledger runtime proposed in
`2026-10-21-evidence-ledger-design.md`; do not add another production pipeline
or silently fall back to the earlier free-prose audit. Keep the existing
public summary command, Markdown result, optional omission count, job gate,
transport policies and atomic save. No dependency, credential, permission,
version or installed-app change is required.

## Evidence and alternatives

The retained trial sent all 53 source segments in one extraction packet while
allowing six claims. Its key-point result omitted important later material.
Detailed field verification also accepted unsupported scope and question-state
labels, which the renderer then added to the prose. Meeting and custom output
failed the large nested schema. Passing code tests did not establish semantic
acceptance. Private trial evidence remains outside Git at
`/private/tmp/velo-ledger-acceptance.bmffnd/`.

Three approaches were considered:

- **Short source-backed points and review of their final placement** is the
  recommended replacement. Keep direct evidence and protected phrases, remove
  per-field classification machinery, and check the text the user will read.
- **Retain the full ledger and reduce packet sizes** would help coverage but
  keep the nested statuses, special content variants and relationship graph.
- **Selected verbatim passages only** would remove most paraphrasing, but
  reproduce recognition noise and read more like excerpts than summaries.

The simpler design does not guarantee that a model interprets evidence
correctly. Its value must be demonstrated by the unchanged acceptance cases,
not inferred from fewer modules or a valid schema.

## Minimum retained data

Keep the immutable transcript snapshot, job token, stable source IDs and exact
unique `source` plus `quote` anchors. Transcript words, order and fractional
times never change. Rust assigns point IDs after parsing each packet.

Packets, plans and review reports carry the exact current job token and use
strict schemas. Extraction anchors must belong to the passages actually
supplied to that packet. Models do not supply offsets or clock labels;
unknown jobs, invented IDs and ambiguous quotations fail locally.

A point contains only:

- `text`: one concise statement or coupled conditional statement in the
  requested language. It must preserve requests, tentative wording and scope.
- `evidence`: nonempty exact source anchors supporting that statement.
- `required`: exact anchored phrases that must remain in the statement,
  including meaning-critical triggers, negation, limitations and uncertainty.
  This array is present even when empty. A conditional effect is not a second
  independently selectable point.

Every required quotation must occur verbatim in `text` before admission and
rendering. Quote presence alone does not prove that the statement is faithful;
the semantic review must check how those phrases relate. Critical quoted
phrases remain in the source language when the framing is translated.

Remove the runtime speech-act/scope/certainty axes, event/permission/default/
question union, owner/deadline fields, answer/dependency graph and spelling-edit
chain. Preserve those distinctions in ordinary wording and in review tests,
not as statuses automatically prepended to every point. Do not infer missing
owners, dates, delivery order, implementation status or open questions.

Spelling may be normalized in ordinary wording only when the supplied
recording or vocabulary clearly supports it. Prefer a clear later mention to
a phonetic guess. Protected quotations and the raw transcript are not edited;
no substring replacement chain is introduced.

## Processing and review

1. **Prepare short points from bounded raw passages.** Use at most twelve
   segments and 3000 UTF-8 bytes of serialized source rows per packet, with
   two-segment overlap. Check the complete actual request against the provider
   context budget as well. These limits divide the offered transcript into
   six packets in an offline count; context overhead may split further. An
   indivisible oversized segment produces a context error, never truncation.
   Keep up to six points per packet and deduplicate only exactly identical
   text plus anchored evidence. Do not give extraction the custom layout
   prompt. Unknown fields, invalid anchors and missing required keys fail;
   the one structural repair receives the specific validation category.
2. **Select whole points into the requested layout.** Reuse ID-only paragraph,
   list and neutral heading blocks. The planner receives the selected format
   and custom focus, but cannot write replacement prose, factual headings or
   select part of a point. No arbitrary free-text writer follows it.
3. **Review the complete visible statement and its heading together.** Give
   each placement an application-assigned occurrence ID. A compact report
   returns that ID, supported/unsupported/unclear and exact raw evidence,
   never replacement text or dozens of field verdicts. Require exactly one
   result for every occurrence. A decision heading is itself a claim of
   agreement; an open-question heading requires explicit unresolved/deferred
   evidence, not missing detail or a question mark. Check source-to-text
   entailment, conditions, roles, uncertainty, spelling and later answers.
   No generated note substitutes for original passages.
4. **Compose locally and save once.** Render only supported occurrences, using
   their complete unchanged point text. Omit empty sections; do not relabel or
   shorten a rejected point into a new claim. Escape literal Markdown before
   adding application-owned headings/bullets and exact source-clock labels.
   Check cancellation immediately before the single atomic save.

Supported review evidence must cover the point's declared evidence and required
phrases under the same original source IDs. Missing, duplicate or unknown
occurrence IDs invalidate the report. A supported quote match remains only a
structural check; the model must still assess the complete statement and its
heading. Neither drafts nor partial reviewed batches stream into the reader.

For the offered small transcript, the reviewer sees the entire raw snapshot
when it fits. For longer inputs, batch up to four placements, supply their raw
passages with adjacent segments and a compact catalog of all source-backed
points. One context follow-up may fetch explicit original source IDs. If that
still cannot establish support, mark the occurrence unclear. Context/catalog
overflow stops with the prior summary intact; unlimited-length support is not
part of this replacement.

Each extraction packet allows an initial call and one structural repair.
Layout allows an initial call and one structural repair. Each review batch
allows an initial call and either one context follow-up or one schema repair.
For E packets and V review batches, retain the logical ceiling `2E + 2V + 2`.
There is no semantic rewrite/retry loop. Smaller inputs can mean more model
requests even though each report is simpler; record actual requests and time.

## Formats and compatibility

Meeting mode retains overview/topics and supported decisions/actions/open
questions, omitting empty categories. Key-point mode has no meeting outcome
headings. Custom mode supports whole-point selection, ordering, paragraphs,
lists and optional neutral headings, including the no-heading short-paragraph
case. Arbitrary narrative/letter rewrites remain unsupported and keep the old
summary. Natural-language length limits steer selection, not removal of a
condition or an exact runtime word-count guarantee.

Existing Markdown, timestamp toggles/copy and exact fractional seek remain
unchanged. The renderer adds no inferred existing/planned/unresolved labels.
Count omitted unique points only when they have no supported placement left;
the omission notice is not certification of retained content. If nothing
substantive remains, fail visibly instead of saving an empty summary.

Keep existing UI stage values: preparation includes extraction and ID-only
layout, evidence checking covers the final-placement review, and composition
is local rendering. No new UI panel or public data model is required.

## Verification and acceptance

Use synthetic, hand-derived fixtures for permanent tests. Cover dropped
completion/negation, request-to-existing promotion, inferred open questions,
unclear roles, confirmed spellings, later answers, factual headings, duplicate
occurrences, escaped source markup, exact clocks, context ceilings, malformed
reports, cancellation and late-event/cache preservation. Structural tests are
not a substitute for model-quality evaluation.

Run all six repository checks fresh, format check, version/whitespace checks,
the acceptance-checker tests and a separate default-feature Apple Silicon
bundle. Preserve prior useful regressions and local backups when retiring the
superseded runtime; do not delete unrelated user changes or keep two active
production summarizers.

Then run the same original 53-segment cache through the saved DeepSeek
destination in isolated temporary stores. Use the same meeting/key-point
focus settings and the same custom prompt requesting one short paragraph,
no headings/list and at most 150 Thai words. The previous fixed checklist is
not weakened: every format retains requested person/group selection, person
as the default, completed review as the removal trigger with its modality,
and backend read/unread retention. Meeting/key-point modes also retain the
shared review box and actual-reviewer record. Unclear mechanism/negative
permission comparisons may be omitted, not guessed.

Inspect raw source, points, placement reports and composed text. Every clock
must match its supporting original segment. Count custom words independently
with Thai `Intl.Segmenter` after removing application citations and record
Node/ICU identity. Require three consecutive complete three-format rounds
on one unchanged build. A failed format resets the sequence; retain failures
and restart after any code change. If a known meaning error survives review,
stop and report the exact boundary rather than extending calls or softening
the checklist. This is repeatability on the offered fixture, not general
accuracy certification.

Keep raw acceptance artifacts outside Git, credentials only in memory, and
normal evidence only in job memory. Verify production transcript, summary and
settings hashes remain unchanged. Do not close or replace the user's running
Preview, push, open a PR, merge, tag or release.

## Approval and next step

The written replacement and Native implementation plan at
`../plans/2026-10-21-simple-grounded-summaries.md` are approved. Tasks 1–4 and
automated gates are implemented locally; retain the unchanged nine-job
acceptance boundary and fresh final review before claiming completion.
