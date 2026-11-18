---
id: SR-2026111422264100000008
title: "Dependency review of IR-96 formalization coverage records"
type: SpecReview
analysis: dependency
scope: "agent-ix/filament-contract-ir; FR-071, Quoin FR-204 and FR-205"
review_set: all
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-071
    type: references
---

## Summary

Checked the prerequisite edges and the downstream Quoin mapping against FR-071's state vocabulary. One serialized state token conflicts with the Quoin consumer's exact token.

## Verdict

**CONDITIONAL** — align the downstream token or state an explicit mapping.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-071 defines `not_applicable`, while Quoin FR-205 names the IR-96 state `not-applicable`; the producer and consumer currently disagree on the exact enum token. | spec/core/functional/FR-071-formalization-coverage-evidence-records.md:61 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | FR-071 now uses the canonical `not-applicable` token used by Quoin FR-205. |

After excerpt:

> `FormalizationState` is the closed set `eligible`, `authored`, `parsed`,
> `typechecked`, `lowered`, `backend_supported`, `proved`, `inconclusive`,
> `rejected`, `unknown`, `unavailable`, and `not-applicable`.
