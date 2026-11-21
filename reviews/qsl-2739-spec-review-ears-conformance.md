---
id: SR-967175026302197768900474181941935084524
title: "spec-review EARS conformance of QSL-2739 AC10 composition reader requirements"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/filament-contract-ir; PR #1120 at ddfb8854e; FR-038 composition paragraph (section 'Implementation guarantee and component composition bodies'), FR-371 Behavior composition paragraph"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-038", type: references }
  - { target: "ix://agent-ix/filament-contract-ir/FR-371", type: references }
---

## Summary

Ticket: QSL-2739. This is the EARS requirement-grammar check of the two normative statements the PR adds. The FR-371 composition paragraph uses ubiquitous form with SHALL ("For such a row the reader SHALL refuse ..."). The FR-038 composition paragraph states the same normative reader behaviour in the indicative ("The reader runs ...", "It examines ...") with no SHALL. Its host section already uses that mood (0 SHALL in the section), so this is one low inconsistency.

## Examined scope

- FR-371 Behavior paragraph "A `correspondence`/`component_composition` node is claim-bearing ..." (examined): it has a SHALL, a named actor and a condition, and it conforms.
- FR-038 composition paragraph "A `component_composition` node adopts FSpec FR-491 ..." (examined): see FND-001.
- FR-038-AC-602..605 and FR-371-AC-12 (context_only): these are acceptance criteria, not requirement statements, so EARS does not apply.

## Verdict

**CONDITIONAL**: one low finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The FR-038 composition paragraph states mandatory reader behaviour (the member-coordinate check, its order and its refusals) with no SHALL. FR-371's paired paragraph uses SHALL, so the two owned statements of one rule use different requirement grammars. | FR-038 |
