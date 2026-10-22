# Evidence-ledger summaries with protected conditions

Status: user-approved design and Native implementation plan, 21 October 2026.
Additional DeepSeek calls for the offered recording were approved. Local code
checks pass, but semantic acceptance remains incomplete (0/3 complete rounds).
Keep the work unshipped; the approval does not waive the acceptance boundary
or authorize push/release.

## Intent and success boundary

Keep Velo's three summary choices: meeting, key points and custom prompt.
Prevent a formatting/writing pass from turning requested behaviour into an
existing capability, inventing open issues, substituting unclear permission
roles, or changing an effect's trigger from completing to starting an action.

This design changes the internal summary pipeline, not transcription, chat,
dictation, playback, providers or credentials. Preserve all existing local
work on `codex/summary-prompt-controls`. The previous experimental audit is
not accepted as a source-fidelity solution and must not be treated as ready
for release merely because its code checks pass.

An evidence ledger records **source claims**, not objective truth. Extraction
and interpretation remain model-assisted. The deterministic promise is
narrower: once a claim's evidence and qualifications are admitted, arranging
the summary cannot rewrite or discard those qualifications. Live acceptance
must separately establish whether extraction admitted the right claims.

## Evidence behind this change

The 53-segment, 3:44 recording used in the previous run exposed two separate
failure surfaces:

- Correctly cited free prose could still change meaning. In the latest
  meeting output, a completed-review effect became an effect of pressing
  review; the audit approved it.
- The audit sometimes rejected supported material or failed to remove a noisy
  permission comparison in its single repair. More audit prompt instructions
  did not provide consistent acceptance across the three formats.

The report is `/private/tmp/velo-fidelity-audit.yUXWZt/report.txt`; it is local
diagnostic material, not a required runtime file or a durable repository test
fixture. Do not commit the private recording, transcript, generated outputs
or Keychain material. Permanent automated fixtures use synthetic equivalents.

## Alternatives and selected direction

| Approach | Benefit | Cost / limitation |
| --- | --- | --- |
| **Evidence ledger + constrained layout + deterministic composition (recommended)** | Makes conditions and stance explicit before writing; prevents the layout pass from altering admitted content | Less fluid wording; initial extraction can still be wrong and needs focused verification |
| Evidence ledger + unrestricted prose writer + another semantic audit | More natural paraphrases, translations and arbitrary custom structures | Reintroduces the observed condition-loss surface; another model verdict is not a structural guarantee |
| Selected verbatim excerpts only | Minimizes new wording and exposes noisy recognition directly | Reads more like excerpts than a summary; limited rephrasing and translation |

Choose the first approach for the next implementation proposal. Use verbatim
source fragments for meaning-critical fields within it, not an unrestricted
writer with additional audit rules. Do not add a second provider/model,
enable reasoning, or silently fall back to the old pipeline.

## Processing boundary

1. Take an immutable transcript snapshot for the job and assign stable source
   IDs, preserving original text, order and fractional times.
2. Extract small candidate source claims with evidence for each meaningful
   field. Do not write meeting sections or a final summary at this stage.
3. Verify candidate fields and relationships against raw source context.
   Admit, quarantine or explicitly request more source context for each claim.
4. Ask a layout planner to select and arrange admitted claim IDs according
   to the selected format and user focus. It cannot return replacement prose.
5. Validate the plan and compose Markdown from the admitted content and
   application-owned connective/heading templates. Resolve source IDs to
   existing clock labels only after composition.
6. Publish/save only the complete composed result. Retain the previous
   summary on cancellation, invalid output or provider/context failure.

The ledger stays private to the Rust job. No new public command or editable
fact-management screen is required. Do not persist raw ledger artifacts in
normal operation or make them an additional source of truth for chat.

Extraction and verification receive source evidence, resolved output language
and spelling vocabulary, not the user's custom summary focus/layout prompt.
That prompt reaches only layout planning. Treat recording text and vocabulary
as quoted data rather than commands; vocabulary can confirm spelling, not
facts or permissions. No model stage may choose a new destination, invoke a
tool or request credentials. The layout planner's schema, not user steering,
defines which transformations the application will accept.

## Claim model

Use the glossary in `CONTEXT.md`. A candidate has one claim, not an arbitrary
paragraph whose unrelated clauses receive one blanket verdict. Its fields
include:

- Local claim ID and evidence anchors for the assertion and every protected
  field. An anchor names a supplied source ID and an exact quoted substring.
- Speech act: observation, request, proposal, agreement or question.
- Implementation scope: existing, planned or unspecified. Conditions are a
  separate qualifier and can apply to existing or planned behaviour alike.
- Certainty: asserted, tentative or unclear. This is source wording, not a
  self-reported model confidence score.
- Content: an ordinary supported point, an event bundle, a permission
  relationship, a default-setting request, or an explicit question.
- Required qualifiers: conditions, negation, scope and uncertainty that must
  travel with the content. A claim can have no qualifier only when the source
  actually supports an unconditional point.
- Optional owner/deadline evidence; absence is not an assignment or an open
  issue. A default-setting request has no development order unless a separate
  source claim explicitly establishes one.

Keep speech act and implementation scope separate. An explicit agreement
about a planned feature is still planned, not implemented. A meeting decision
may therefore be an agreement with planned scope. Do not retain the previous
single-axis `requested` versus `agreed` classification as the whole model.

An agreement can reference the request it acknowledges only when its raw
reply context supports that link. An isolated yes cannot approve every nearby
proposal. Questions need evidence of being answered, explicitly deferred or
explicitly left unanswered before receiving those labels. Missing future-use
details are not a question to classify.

### Meaning-critical content

For event bundles, keep trigger, event phase, conditions, effect and affected
roles together. For permission relationships, distinguish document origin,
group membership and personal permissions; do not substitute one for another.
For defaults, distinguish a selected value from a development sequence.

In the first implementation, critical trigger/negation/role fragments come
from exact source excerpts, with only independently supported spelling
corrections allowed. The model never calculates text offsets or timestamps:
it copies a source ID and quote; Rust locates a unique match and derives any
internal byte offsets. Ambiguous matches require a longer exact quote or
quarantine. Each correction retains the original anchor and explicit spelling
warrant; a plausible ASR expansion is not a warrant.

For an event bundle, the critical trigger and effect fragments both use raw
anchors; a normalized sentence cannot quietly introduce a different event.
For a permission relationship, its meaningful roles and negated comparison
also require their own clear anchors, not just a citation on the general topic.
The renderer, not a prose model, joins the protected trigger and effect. It
must not accept an effect-only reference to an event bundle. A normalized
standalone sentence cannot serve as an alternate route around this rule.
Ordinary content may use a checked concise clause, but verification must
classify and protect any conditions or roles present in that clause first.

This does not make event/role extraction infallible. An incorrect trigger
selected during extraction remains a semantic failure even when the renderer
faithfully preserves it. The release gate tests that boundary explicitly.

### Concrete regression example

The recording contains a completion condition around 02:13 and the effect on
the other reviewers around 02:17. For a test ledger, that is one event bundle:

- Scope: a described/design workflow, not proof of an existing implementation.
- Trigger: completion of review, anchored to the completion phrase.
- Effects: the document leaves the review box and other reviewers no longer
  see it, with separate source anchors where necessary.

Selecting that bundle always includes its completion condition and source
references, whether shown as a meeting bullet or inside a custom paragraph.
There is no layout operation that changes it to pressing or starting review.
The raw question/uncertainty must also be preserved where not resolved by
surrounding evidence; this example is not permission to erase that modality.

## Verification and uncertainty

Local validation checks schema, unique IDs, exact evidence matches, complete
verification coverage, valid links, qualifier dependencies and cycles, and
that every output reference resolves inside the current job. It does not
claim that matching quotes prove entailment.

The focused verifier sees candidate fields, their raw supporting passages,
adjacent context and a bounded catalog of other candidate claims/references
from the same recording. It evaluates source-to-claim mappings before layout
instructions are introduced. It returns field-specific findings, not a
rewritten summary or an overall approval.

- Admit only a claim whose meaningful fields and relationships are supported.
- Quarantine unclear or unsupported claims. Do not guess a replacement role,
  turn lack of detail into a question, or promote a proposal to an agreement.
- An independent safe point may survive only as its own separately extracted
  and verified candidate, never by deleting a required qualifier from another
  claim. The verifier cannot create replacement prose/cards. If extraction did
  not produce that independent candidate, omit it for this job rather than
  add an unbudgeted semantic rewrite/re-extraction loop.
- A verifier can request additional explicit source IDs once. Rust fetches
  those original passages with adjacent context and verifies the complete
  request size before the second call. No generated summary substitutes for
  raw evidence. If the connection still cannot be established, quarantine it.
- If all substantive content is quarantined, stop with an actionable error
  and preserve the old summary. Otherwise summarize admitted content and show
  a small localized notice when uncertain content was omitted.

Do not invent a global accuracy percentage. The omission notice means that
some detail was excluded, not that everything remaining is guaranteed true.
An `omitted_uncertain_claims` count is additive optional summary metadata;
older caches default to no known notice, not certification. No raw ledger or
quotes need to be added to IPC or the normal cache to support this notice.

## Layout and rendering contract

The planner receives admitted content and IDs, format, language, and user
focus/layout instructions. It returns an ordered layout of references and
application-defined paragraph/list/heading choices. Unknown keys such as
replacement `text`, unknown IDs, invalid duplicate placement, missing qualifier
dependencies and unsupported category mappings are invalid.

A claim can appear once in the overview and once in a single body section.
It cannot repeat within a paragraph/section or across competing body sections.
This permits an overview without treating repeated mentions as new evidence.

All rendered factual content, including the overview, comes from admitted
claims. No free-text opening, conclusion, caption or factual heading provides
a hidden route for another model-written assertion. The renderer owns neutral
connectors and localized scope labels and validates the final document shape.
Do not add causal connectors such as therefore unless that relation is itself
an admitted part of the claim.

- **Meeting:** overview/topics select admitted claims; decisions require an
  evidenced agreement; work items require explicitly requested or assigned
  work; open issues require an explicit unresolved/deferred question. Empty
  sections are omitted. Missing owner/date metadata may annotate a supported
  work item only when requested.
- **Key points:** use overview and key points, preserving requested/current
  and tentative distinctions. Do not add meeting outcome categories.
- **Custom:** no required headings or meeting categories. Support selection,
  ordering, short paragraphs, lists, and neutral application-defined headings.
  The existing short-paragraph/no-heading example remains supported. Arbitrary
  newly worded headings, narrative/letter forms or prose transformations that
  cannot be expressed without rewriting admitted content return a clear
  unsupported-layout message and keep the existing summary. Do not silently
  approximate these requests or weaken protected content to satisfy them.

This custom-mode limit is an intentional trade-off for the proposed first
version and requires user acceptance with this spec. It is not full free-form
ChatGPT-style rewriting. The existing chat remains available for discussion
but is outside this pipeline's acceptance claim.

Length requests select fewer whole claims rather than trimming qualifiers.
The planner does not return an estimated word count as proof. Paragraph/list
counts can be structural constraints; natural-language word limits remain
length steering in this first version, not a new exact runtime guarantee.
The existing 150-word Thai acceptance case is independently checked after
removing citations with `Intl.Segmenter('th', { granularity: 'word' })`, counting
only `isWordLike` segments and recording the test runtime version. This is a
test convention, not an equivalent Rust whitespace count. Do not add a Thai
word-segmentation subsystem just to imply broader limit enforcement. If a
length target competes with a complete condition, preserve the condition or
omit the whole claim, never shorten it into a different claim.

Output language still follows existing settings. Localized framing and checked
ordinary clauses use that language; critical verbatim excerpts may remain
quoted in the source language. Do not imply a fully monolingual translation
guarantee for those excerpts. Accept more source-like wording initially;
natural paraphrase of protected fields is a separate future design decision.

## Long recordings, budgets and job lifecycle

Keep raw source IDs stable across the existing overlapping chunks. Extract
small bounded packets rather than emitting the whole Thai ledger in one JSON
answer. Deduplicate identical anchored claims; do not merge conflicting event
phases or create a new agreement from several suggestions.

Verification can follow references across packets using raw passages. Later
clarifications can answer earlier questions, but chronological proximity alone
is not proof of that relationship. Unresolved cross-packet relationships stay
unclear; they are not automatically open questions.

Keep the existing provider output caps and conservative UTF-8 context checks.
Account for the actual serialized schema, source, catalog and candidate fields
in each request. Input packets may split at source-segment boundaries before
extraction; no evidence or qualifier is silently truncated. An indivisible
oversized source unit or a catalog/layout that cannot fit causes a context
error, not fallback to the old unchecked reducer. Unlimited-length support or
a separate hierarchical layout planner is outside this first version.

For E extraction packets and V verification batches, allow at most:

- Two logical completions per extraction packet: initial extraction and one
  schema repair. Failure after that stops the job.
- Two per verification batch: first verification, then either one requested
  context follow-up or one schema repair, never both in a loop.
- Two for final layout: initial plan and one structural repair.

The ceiling is `2E + 2V + 2` logical completions, with no final free-prose
writer/auditor. A one-packet/one-batch happy path uses three; this is a design
budget, not a latency or price estimate. Existing bounded transport retries
remain separate and must not create an unbounded semantic retry loop.

The current job gate, cancellation, in-memory credentials, configured single
destination, and atomic summary replacement stay in place. No generation
starts from opening media or changing options. The source snapshot and ledger
are job-scoped and discarded on cancellation; ledger reuse across different
jobs/formats requires a separate cache-invalidation design and is not added.

Progress should describe extracting evidence, checking evidence and composing
the summary as separate stages, not a blended percentage. Add bilingual stage,
omission-notice and unsupported-layout strings using the existing typed locale
pattern. Render only after the complete plan and references validate. Legacy
Markdown remains readable, with no automatic regeneration or assurance badge.

## Responsibilities and compatibility

- `summary/` owns evidence anchors, claim/qualifier types, focused verification,
  layout validation, deterministic composition and orchestration. Keep these
  responsibilities in focused modules rather than growing `prompt.rs` further.
- The command layer still supplies transcript/settings/key data. The summary
  module must not reach into transcription or player state.
- Transports remain provider-neutral and retain existing thinking/output
  policies. No new provider, model selection, permission or native dependency
  is part of this design.
- Keep the existing Markdown summary field and file naming. Only optional
  omission metadata and progress-stage values are additive schema changes.
  Update Rust/TypeScript/locales together when implementing; old caches and
  defaults must not be rewritten on load.
- Exact citation lookup, fractional seek navigation, copy/timestamp toggles
  and all prior user work remain unchanged.

## Acceptance and verification

Before implementation, define synthetic golden ledgers and hand-derived
expected output; do not let the generator compute its own expected answers.
The implementation plan must include red/green cases for:

1. Selecting an event's effect without its completion condition is rejected
   in every format. Rendering the valid bundle retains the condition and both
   supporting references. A start/completion mutation must fail.
2. Agreement + planned scope does not render as implemented. A request alone
   cannot appear as a decision, and an explicitly requested task needs no
   invented owner or due date.
3. A confirmed Default spelling does not create development order or a name.
4. Keeping data for possible future use is retained without adding an open
   issue or a claim that the data is not currently used.
5. An ambiguous owner/permission comparison is quarantined; a clearly stated
   independent positive relationship is retained only with its own evidence.
6. An earlier question answered later, including across a packet boundary,
   is not still presented as open. An explicitly deferred question can be.
7. Nonexistent, ambiguous, wrong-source, Unicode-boundary and cross-job
   evidence anchors fail. Quotes are checked against raw source, not notes.
8. Planner-supplied replacement text, factual headings, duplicate/unknown IDs,
   category promotion and qualifier removal fail before rendering/saving.
9. Malformed/truncated structured output, exhausted call budgets, oversized
   input, cancellation during any stage and provider failure preserve the old
   summary and do not leak keys or start another job.
10. Partial uncertainty produces the omission notice; wholly unsupported
    content does not save an empty summary. Legacy caches still load.
11. Custom no-heading paragraphs, supported length limits, translated framing
    and timestamp copy/hide behavior remain correct; unsupported custom forms
    report their limit rather than silently changing the requested structure.

After the six repository checks and a local Apple Silicon bundle, run the
offered recording through all three formats using the same code and approved
DeepSeek account. Capture ledger/field-verdict/layout/composed-output artifacts
only in an explicit local acceptance harness. Do not emit raw evidence in
normal application logs or commit private acceptance artifacts.

The fixed real-file checklist has a minimum content boundary so a short,
vague output cannot pass merely by omitting every difficult point. All three
formats must retain the requested person/group choice, person as default (not
development order), the completed-review condition for removal, and keeping
read/unread data in the backend. Meeting and key-point modes must additionally
retain the described shared review box and recording the actual reviewer,
without inventing permission ownership. The short custom paragraph may omit
those two secondary details, but not alter them if included. Unclear mechanism
names and negative permission comparisons need not be retained. Preserve
modality and planned scope on every retained point.

Inspect both extraction and final text, including the 02:13/02:17 completion
condition. Retain every failed run; do not assemble a pass from different code
versions or mistake valid IDs for correct meaning. Require three consecutive
complete three-format runs with no known meaning error in the fixed checklist;
any failure resets that sequence. This evaluates repeatability on the offered
fixture, not a guarantee for every recording. Keep opt-in live tests separate
from normal CI and obtain appropriate authorization for any new data/destination.

## Review and next gate

Review the constrained custom-mode behavior, source-like critical wording,
partial-uncertainty policy and API-call ceiling before implementation planning.
These are proposed choices, not behavior already shipped. Once the user
approves this written spec, write and review an implementation plan before
any product code changes. Do not push or release without an explicit request.
