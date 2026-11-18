---
id: SR-2026111422264100000005
title: "Base review of IR-96 formalization coverage records"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; FR-071 and spec/spec.md"
review_set: all
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-071
    type: references
---

## Summary

Reviewed FR-071's exact record shapes, state vocabulary, clause address, and six acceptance criteria against FR-011, FR-012, FR-045, FR-052, and the Quoin consumer. One missing invariant can permit a proved record without its clause address.

## Verdict

**CONDITIONAL** — add an explicit state-to-address presence rule before implementation.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-071 does not require a clause address for states that imply a formal clause exists, so a lowered, backend-supported, or proved record can omit the deterministic address. | spec/core/functional/FR-071-formalization-coverage-evidence-records.md:56 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The state-aware invariant requires an address for every state that implies a formal clause, and for unknown/unavailable when a clause exists. |

After excerpt:

> When a coverage-state record has state `authored`, `parsed`, `typechecked`,
> `lowered`, `backend_supported`, `proved`, `rejected`, or `inconclusive`, the
> record SHALL include `clause_address`. When a coverage-state record has state
> `unknown` or `unavailable` and a formal clause exists in the selected package,
> the record SHALL include `clause_address`; when no such clause is present, the
> record SHALL omit it. Every present formal clause SHALL therefore have its
> deterministic address even when its coverage state is `unknown` or
> `unavailable`.
