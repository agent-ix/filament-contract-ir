---
id: SR-2026111422264100000007
title: "Integrity review of IR-96 formalization coverage records"
type: SpecReview
analysis: integrity
scope: "agent-ix/filament-contract-ir; FR-071 and dependencies"
review_set: all
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-071
    type: references
---

## Summary

Checked FR-071 for complete state, evidence, and acceptance-criteria semantics. The criterion states distinguish failure outcomes, but the required reason for non-success states has no field or defined source in the record.

## Verdict

**CONDITIONAL** — define how state reasons are retained and validated.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-5 requires the applicable reason for rejected, inconclusive, unknown, and unavailable outcomes, but the record defines no reason member or rule binding those states to a reason in EvidenceEnvelope. | spec/core/functional/FR-071-formalization-coverage-evidence-records.md:117 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The record adds a formalization-specific `reason` field and requires a non-empty value for rejected, inconclusive, unknown, unavailable, and not-applicable states. It distinguishes this from shared evidence fields. |

After excerpt:

> When a coverage-state record has state `rejected`, `inconclusive`, `unknown`,
> `unavailable`, or `not-applicable`, the record SHALL include a non-empty
> `reason`. When its state is `eligible`, `authored`, `parsed`, `typechecked`,
> `lowered`, `backend_supported`, or `proved`, the record SHALL omit `reason`.
