# Source-bound phrases in summary extraction

Status: proposed written amendment for user review, 22 October 2026. The user
approved approach 1 and writing this spec, not its implementation. No new
provider calls, product-code changes, push or release are authorized by this
document. Written-spec approval precedes an implementation plan and its review.

## Outcome and scope

Produce readable summaries that preserve the recording's meaning and navigate
to the right original times. Remove one observed extraction failure: asking
the model to copy a protected phrase both into an anchor and into free prose,
then rejecting the packet when those two copies differ.

This is a scoped amendment to
[`2026-10-21-simple-grounded-summaries-design.md`](2026-10-21-simple-grounded-summaries-design.md).
If approved, it replaces only that spec's model-facing extraction point shape
and its conversion into the existing internal point. The internal
`DraftPoint { text, evidence, required }`, whole-point layout, placement review,
composition, atomic save and all semantic acceptance requirements remain.
It does not reinstate the full Evidence Ledger or create a second runner.

Keep the current provider/model/destination, transcript bytes/order/times,
settings, caches, credentials, UI, Thai font, chat, microphone and player.
No dependency, permission, version or installed-app change is required.

## Evidence and decision

The repaired live attempt at
`/private/tmp/velo-simple-grounding-repair.KZO648/round-1/` made eight logical
extraction calls across meeting, key-point and custom modes. All supplied
anchors matched their raw source passages. Required phrases nevertheless
remained absent verbatim from the repaired point text: five in meeting,
two in the second key-point packet, and two in custom. One initial key-point
response also returned eleven points rather than at most six. None reached
layout, review or composition; there is no accepted final text.

Specific `required_quote_missing` feedback did not resolve the duplication
problem. This evidence establishes a structural failure, not proof that the
model's interpretations would pass a semantic reviewer.

Two approaches were presented:

- **Typed references to protected source phrases, selected by the user:**
  the model supplies prose around references; Rust inserts exact raw spans.
  This removes the duplicate-copy requirement while retaining concise prose
  and the existing reviewer. It adds a small extraction adapter, not another
  generation stage. It cannot guarantee that the model selected enough
  protection or interpreted the source correctly.
- **Verbatim excerpts only:** fewer paraphrasing opportunities, but noisier
  output resembling a transcript and less flexible custom summaries. Not
  selected. More prompt-only retries or looser quote checks are not fallbacks.

## Model-facing extraction contract

The packet still has exactly `job` and `points`. A model-facing point now has
exactly `parts`, `evidence` and `required`; a model-supplied `text` is invalid.
Both anchor arrays keep the existing strict `{ source, quote }` shape.
`evidence` is nonempty and `required` is present even when empty.

Each entry in the nonempty `parts` array is exactly one of:

- `{ "text": "ordinary wording" }`: a literal string, treated as untrusted
  model/source data, not instructions or markup.
- `{ "required": 0 }`: an unsigned integer indexing this same point's
  `required` array, zero-based. This is a local slot reference, not a new
  semantic ID, source offset or reference to another point.

Reject mixed-key entries, unknown keys, nulls, wrong types, negative/fractional
indices and out-of-range references. Every declared required slot must be
referenced exactly once. Duplicate references and unreferenced slots fail;
an incidental matching string in a literal part does not satisfy coverage.
Ordinary points with no protected phrases may have `required: []` and literal
parts only. This permission does not waive protection for meaning-critical
conditions, negation, limitations, questions or uncertainty.

A synthetic example, not acceptance-recording content:

```json
{
  "job": "J42-1",
  "points": [
    {
      "parts": [
        { "required": 0 },
        { "text": ", the item can be removed." }
      ],
      "evidence": [
        {
          "source": "S000001",
          "quote": "Only when inspection finishes can the item be removed."
        }
      ],
      "required": [
        { "source": "S000001", "quote": "Only when inspection finishes" }
      ]
    }
  ]
}
```

Given that exact unique passage under S000001, Rust assembles:
`Only when inspection finishes, the item can be removed.` No model-generated
second copy of the protected phrase is required. The example job token is
illustrative; actual prompts and responses must use the current job token.

## Deterministic conversion and ownership

The extraction adapter belongs inside the existing summary evidence module.
Its only input is the parsed packet plus the immutable current source snapshot
and supplied-source set; its output is the existing internal point shape.
It does not read the player, transcript store, credentials or production cache.

Before accepting any packet:

1. Enforce the strict schema, current job and at-most-six-point limit. Validate
   nonempty point content and all anchors against the raw sources actually
   supplied to this packet. Preserve rejection of duplicate anchors within
   either anchor array, ambiguous quotations and invented or unsupplied source
   IDs. The same anchor may appear once in evidence and once in required.
2. Require every protected anchor to be covered by an evidence quotation under
   the same original source ID. Validate local slot coverage as above. A source
   ID not supplied to this packet is invalid even if supplied elsewhere; an
   overlapping source actually supplied to both packets remains valid.
3. Concatenate parts in their supplied order. Literal strings are copied as
   data. For a reference, copy the resolved span from the immutable raw source,
   not a model-provided replacement. Do not normalize, translate or correct
   protected bytes. Do not add separators, quotation marks, punctuation,
   status labels or explanatory prose; literal parts supply any delimiters.
4. Produce `DraftPoint { text, evidence, required }` and run the existing
   `validate_point` boundary, including verbatim inclusion and evidence
   coverage. No partially valid packet is admitted.
5. Deduplicate only identical canonical internal point data and assign Rust
   point IDs as before. Different literal-part splits yielding identical
   canonical data do not create extra points. Do not semantically merge points.

There is no regex/token substitution in prose, fuzzy match, silent insertion
of an omitted reference, automatic deletion of an invalid point, or repair
of source wording. Literal strings resembling slot syntax remain literal data.
Discard the template after conversion; downstream code consumes only the
canonical point. This replaces the extraction wire contract, not adds a
legacy-text fallback or alternative production runtime.

Slot indices always resolve within the current point. There is no cross-point
reference syntax: an entry adding a point identifier is invalid, while index
0 in two different points means each point's own first protected anchor. Rust
cannot infer an unstated intention to reference a different point; complete
statement review still checks the resulting meaning. Empty point packets
retain the existing behavior for filler; they do not certify source coverage,
and no substantive surviving content still makes the whole job fail.

## Meaning and source-time guarantees remain separate

Rust can establish that each declared protected span occurs unchanged at its
chosen position. It cannot establish that the selected phrases are sufficient
or that surrounding prose expresses their relationship correctly.

For example, inserting a correct conditional phrase does not make a fabricated
decision or added negation true. A model might also incorrectly return no
protected phrases. The existing placement reviewer must still assess every
complete assembled statement with its selected heading against raw passages,
including later clarifications. It must reject missing conditions, request-to-
fact promotion, question-to-certainty promotion, inferred roles/permissions,
development-order inventions and unsupported open questions. Rejected content
is removed as a whole occurrence; it is not rewritten or relabeled locally.

Keep a conditional effect and trigger in one inseparable point. Do not infer a
person or action from phonetic recognition such as a misheard Default; ordinary
spelling is normalized only when supplied evidence or confirmed vocabulary
supports it in context. Protected quotations themselves remain unedited.

Layout still selects whole canonical point IDs, without new prose. Review
still sees the full raw snapshot when it fits, otherwise the existing focused
raw evidence/neighbours/catalog and one bounded source follow-up. Neither
the template nor a generated note substitutes for original evidence. No new
semantic status axes, field reports or relationship graph are introduced.

Times still resolve from original source IDs to the unchanged fractional
segment starts. Models cannot supply clocks or offsets. A synthetic source
starting at 12.75 seconds must still seek to 12.75, even if displayed as 00:12.
Only composition adds application citations. Literal source headings/clocks
and embedded instructions remain escaped data, never navigation commands.
Human acceptance must check that each cited passage supports its statement,
not merely that the clock exists somewhere in the recording.

## Bounds, repair and failure behavior

Keep at most twelve segments and 3000 serialized-source UTF-8 bytes per
extraction packet, with up to two-segment overlap and forward progress.
Keep at most six points per packet; never take the first six of an oversized
response. Keep the actual complete request budget, indivisible-source errors
and output/transport policies. Extra template structure is not permission to
truncate sources, shorten a condition or raise output/context limits.

Each packet still has one initial request and one structural repair. Feedback
identifies the actual structural category, including point-count overflow,
invalid part/index, missing/duplicate slot reference, job/schema mismatch or
invalid/uncovered anchor. Include bounded location/count metadata as relevant;
ask for one complete corrected packet from the same raw source rows. Do not
publish surviving fragments, drop protections to pass, or start semantic
rewrites. Before the first request, reserve the largest possible serialized
feedback envelope for the categories used, including actual job/source-ID
lengths. The complete actual repair request must fit too; retain conservative
headroom rather than claiming a context saving without measurement.

Report one deterministic first failure: schema, current job, point count, then
points in wire order; within a point validate anchors/coverage, parts/slot
coverage and nonempty assembled text in that order. Reference-only failures
identify the first offending part or missing slot, not an unbounded list or
echo of the full response. This bounds feedback without changing the one-repair
limit or ignoring other invalid data on the next validation pass.

No new completion stage is added. Layout and review retain their existing
slots, including review follow-up OR schema repair. For E extraction packets
and V occurrence batches, the ceiling remains `2E + 2V + 2` logical calls.
Transport retries retain their existing separate policy and accounting.
There is no unbounded retry-until-success loop.

Cancellation, invalid/exhausted repair, overflow, provider failure, unsupported
layout and no supported substantive content preserve the preceding summary.
Nothing streams to the reader before final local composition and the single
atomic save. Public commands, stage values, Markdown cache schema, omission
count semantics, timestamp toggle/copy and navigation remain unchanged.

The QA observer uses the same runner. Existing allowlisted request/response
artifacts contain the new wire packet, while point artifacts contain canonical
assembled text. Keep credentials/headers/JobInput out of artifacts and normal
evidence only in job memory. Private acceptance outputs remain outside Git.

## Verification and unchanged acceptance

Use hand-derived synthetic fixtures, not hardcoded acceptance-recording prose.
Required regression coverage includes:

- Exact source-byte assembly with Thai/English framing and different UTF-8
  lengths, multiple protected spans and plain points without protected spans.
- Missing, duplicate, negative/fractional and out-of-range references, attempted
  cross-point reference keys and point-local resolution of the same index;
  mixed or extra keys; obsolete text packets; wrong jobs; empty assembled text;
  unknown/unsupplied/ambiguous anchors and missing evidence coverage. Each
  failure preserves the preceding cache; no local silent repair is allowed.
- A paraphrased literal cannot substitute for a missing typed reference. Part
  order is deterministic. Literal fake IDs/clocks/headings and instructions do
  not gain reference, markup or execution meaning.
- Canonical deduplication, all tail sources covered, the existing packet and
  request budgets, category-specific repair feedback and an exhausted second
  response making no third call. Over-six-point output is never truncated.
- Review of the complete assembled text plus heading, including correct quotes
  inside an unsupported assertion, absent critical protection, late answers
  and partial occurrence rejection. Canned verdicts prove orchestration, not
  actual model entailment. Retain exact fractional seek and escaped-clock copy.
- The same real inner runner and observer/default request equivalence, privacy
  allowlists, all three formats/providers, cancellation during actual awaits,
  late-file events and old-cache preservation. Observer E/V identities still
  use original packet source sets and occurrence sets, not template variants.

Run all six repository commands fresh after implementation: `cargo fmt`,
`cargo clippy --all-targets -- -D warnings`, `cargo test`, `npm run typecheck`,
`npm run lint` and `npx vitest run`. Also run fmt check,
default/QA warning-denying Clippy, QA pipeline tests, Node acceptance/release
checks, version/whitespace checks and a separate default-feature Apple Silicon
bundle. Keep version 0.8.0 and the user's running/installed app untouched.
Record opt-in skips separately from actually exercised Whisper/model cases.

Then require the original 53-segment cache, SHA-256
`47d6de4119b81350585d54c4e8e0b52ce853f8b212d52f4fcd026e67189d5a15`,
associated with the same offered 224.433333-second recording, through the
saved DeepSeek destination. Use isolated stores and fixed private settings,
not production preference/cache writes. Preserve meeting/key-point focus and
the exact custom prompt:
`สรุปเป็นย่อหน้าสั้น ๆ ไม่เกิน 150 คำ ไม่ต้องมีหัวข้อหรือรายการ`.

The unchanged minimum checklist remains mandatory in every format:

- Requested person/group choice; person as Default, not a development sequence.
- Completed review as the removal trigger, retaining its original question/
  uncertainty, not pressing or starting review as the trigger.
- Backend read/unread retention without inventing that it is unused now.
- No invented agreement, status, owner, date, permission scope or open issue;
  every application clock supports the actual nearby statement.
- Meeting/key-point also retain the shared inbox and actual-reviewer record.
  Custom remains one short paragraph, no headings/list and at most 150 Thai
  word-like units for this fixture. Independently count `Intl.Segmenter('th',
  {granularity:'word'})` entries with `isWordLike`, excluding application
  citations, and record Node/ICU identity. This is not a general runtime
  exact-word-count guarantee.

Inspect original raw passages, wire templates, assembled points, chosen
placements/verdicts and composed output before accepting each round. Require
three consecutive complete rounds of all three formats on one unchanged
build/source/frozen-settings identity: nine successful jobs. A failed format
resets the sequence; any code change requires a fresh sequence and appropriate
gates. Do not reuse older failed jobs, skip difficult required topics or
change checker criteria. A known meaning error surviving review stops further
live calls for a precise report, not more retries or weaker wording.

Retain failed artifacts and record E/V/logical requests/ceilings/time. Record
source, summary and settings hashes before/after, distinguishing unrelated
updater timestamp activity from summary-preference changes without restoring
production preferences. Read the key only in Rust memory; the user handles
any Keychain dialog. Keep backups until recovery no longer depends on them.
Final independent whole-worktree review remains required, and any subsequent
product fix invalidates earlier nine-job acceptance. No push/PR/merge/tag/
release or installed-app replacement is included.

## Review and handoff

This document describes a design, not a completed fix or passing live result.
The previous implementation plan remains blocked at task5; this amendment
does not reopen or reimplement its completed unrelated tasks.

Review this written amendment first. Once approved, write a focused Native
implementation plan for the extraction wire adapter and its regression/
acceptance work, reusing the existing pipeline. Implementation waits for
review of that written plan; do not infer approval from this spec-writing
request. No provider call is needed to review this document.
