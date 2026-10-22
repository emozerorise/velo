# Velo source understanding

Velo turns recordings into readable transcripts, summaries and grounded
discussion. These terms distinguish what a recording says from what a reader
or model might infer.

## Language

**Source evidence**:
The original transcript words and their position in a particular recording.
Evidence may contain recognition errors; it is not independent proof of truth.
_Avoid_: Cleaned source, verified truth

**Source claim**:
One assertion, request, suggestion or question attributable to source evidence,
with the qualifications needed to preserve its meaning.
_Avoid_: Fact merely because a timestamp exists

**Evidence ledger**:
The collection of source claims and their supporting passages for one recording.
_Avoid_: Summary, meeting minutes

**Speech act**:
Whether a source claim observes, requests, proposes, agrees or asks something.
An agreement about a design does not establish that the design is implemented.
_Avoid_: Implementation state

**Implementation scope**:
Whether described behaviour is existing, planned or unspecified in the source.
Conditions can qualify either existing or planned behaviour.
_Avoid_: Agreement, delivery order

**Required qualifier**:
A condition, negation, limitation or uncertainty without which a source claim
would change meaning.
_Avoid_: Optional detail, removable context

**Event bundle**:
A described trigger and its effects, together with their required qualifiers.
Starting an action and completing it are distinct events.
_Avoid_: A result independent of its trigger

**Explicit open issue**:
A question the speakers explicitly leave unanswered or defer, rather than a
detail the recording happens not to mention.
_Avoid_: Missing information

**Unclear detail**:
A role, term or relationship that the available evidence does not establish
clearly enough to repeat as an assertion.
_Avoid_: Most plausible interpretation

**Summary layout**:
The choice, grouping and order of source claims for a particular reading goal.
_Avoid_: Permission to rewrite a claim's conditions or certainty
