---
id: SR-2026111422264100000013
title: "Code review of IR-96 formalization coverage spec"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; spec/core/functional/FR-071-formalization-coverage-evidence-records.md"
review_set: all
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-071
    type: references
---

## Summary

The PR changes Markdown requirements only; no executable code or tests are changed. The reviewed spec duplicates existing nested type field definitions already owned by FR-052, FR-012, and Engineering Assurance.

## Verdict

**FAIL** — the duplicate type shape definitions must be removed or replaced with references to their owners.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-071 re-states the member shapes of `SourceAuthoredCriterionId`, `EvidenceEnvelope`, `SourceSpan`, and `SourceLocation`, which are already owned by FR-052, Engineering Assurance, and FR-012; this creates a second definition that can drift. | spec/core/functional/FR-071-formalization-coverage-evidence-records.md:54 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | FR-071 now names existing nested types and points to FR-052, FR-011, and FR-012 for their definitions; the duplicate field shapes are removed. |

After excerpt:

> `ClauseAddress` has exactly these members:
>
> | Field | Type | Meaning |
> | --- | --- | --- |
> | `package_identity` | `PackageId` | Identity of the package containing the clause, as defined by FR-011. |
> | `criterion_id` | `SourceAuthoredCriterionId` | The Quire-authored criterion identity defined by FR-052. |
> | `clause_id` | `ClauseId` | Identity of the clause as defined by FR-012. |
> | `source_span` | `SourceSpan` | The admitted clause source range defined by FR-012. |
