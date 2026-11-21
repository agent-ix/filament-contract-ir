---
id: SR-967175026302197768900474181941935084517
title: "QSL-2743 PR 1117 spec integrity analysis of the changed IR rows"
type: SpecReview
analysis: integrity
scope: "agent-ix/filament-contract-ir; QSL-2743; PR 1117 head 31a74a3c6d48f8f272f82092cd28ba9e377d2516; FR-352-AC-31, FR-358-AC-15, FR-069-AC-11, FR-019 inventory"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-352", type: references }
  - { target: "ix://agent-ix/filament-contract-ir/FR-358", type: references }
  - { target: "ix://agent-ix/filament-contract-ir/FR-069", type: references }
---

## Summary

Integrity analysis (completeness, consistency, atomicity) of the changed IR rows. Ticket: QSL-2743. Both new FR-352 and FR-358 criteria are compound. The consistency defects in their wording are already recorded as code-review FND-012 and are not duplicated here.

## Verdict

**CONDITIONAL** — medium and low findings only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-352-AC-31 is not atomic: it bundles row retention, tuple-first ordering, census and per-case checks, payload side-input joins, full and reached coverage codes, direct, record-field and object-field projection, foreign and missing refusals and receiver immutability. One tag claims all of them, which is how the untested clauses (record/object field, receiver, foreign) passed as covered. | spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md:627 |
| FND-002 | low | FR-358-AC-15 bundles three separately failing obligations (charge order of common pass, clone and prior copy; exact and one-below limits with no partial publish; zero budget before retention) in one criterion. | spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md:256 |

## Machine-readable data

```yaml
{
  "clean": false,
  "scope": [
    {
      "id": "FR-352-AC-31",
      "path": "spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md",
      "role": "examined",
      "excerpt": "Implemented (QSL-2743). Joining the selected model tuple and model union witness rows (FSpec FR-830) returns the complete rows keyed by structural node, retaining each type handle, settled key, case rank, position, name, original case, member and payload as the producer's opaque identities. Tuple rows are admitted first, then union identity, case census, per-case checks, payload original type against the side-input witness table, and coverage: a full scope refuses an uncovered union with `IncompleteIdentityWitness` and a reached scope with `UnavailableIdentityWitness`. Direct projection and a "
    },
    {
      "id": "FR-358-AC-15",
      "path": "spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md",
      "role": "examined",
      "excerpt": "Implemented (QSL-2743). The model tuple and model union row join charges one shared `NativeJoinBudget`: every row of both channels is charged in the common pass before allocation, then the selected clone and the prior-copy publish. The measured exact cumulative work and storage succeed and one unit below either refuses at the first unpaid kind and locus with no partial publish; a zero budget refuses before any row is retained."
    },
    {
      "id": "FR-069-AC-11",
      "path": "spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md",
      "role": "examined",
      "excerpt": "Implemented (QSL-2743). A model union variant payload naming an object type of the selected package is admitted as a `Reference` payload, so a union such as `Mix.Held(Holder)` passes intake instead of being refused for an undeclared payload type."
    },
    {
      "id": "FR-019-inventory",
      "path": "spec/model/functional/FR-019-rust-library-interface.md",
      "role": "examined",
      "excerpt": "| `checked_package` (V2 reader) | types `AdmissionBinding`, `CheckedCollectionKind`, `CheckedDeclaration`, `CheckedDependencyClockSelection`, `CheckedDependencySelection`, `CheckedDiagnosticCause`, `CheckedDiagnosticCode`, `CheckedDiagnosticStage`, `CheckedDiagnosticV2`, `CheckedDiagnosticsV2`, `CheckedJson`, `CheckedJsonError`, `CheckedMemberType`, `CheckedModelField`, `CheckedModelFieldsError`, `CheckedModelObjectFields`, `CheckedModelRecordField`, `CheckedModelRecordFields`, `CheckedModelRecordFieldsError`, `CheckedModelVariantBody`, `CheckedModelVariantCase`, `CheckedVariantPayloadType`, `"
    }
  ],
  "findings": [
    {
      "fnd": "FND-001",
      "severity": "medium",
      "confidence": "high",
      "method": "spec-review/integrity",
      "check": "compound",
      "artifact_id": "FR-352-AC-31",
      "related_ids": [],
      "path": "spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md",
      "lines": "627",
      "excerpt": "Implemented (QSL-2743). Joining the selected model tuple and model union witness rows (FSpec FR-830) returns the complete rows keyed by structural node, retaining each type handle, settled key, case rank, position, name, original case, member and payload as the producer's opaque identities. Tuple rows are admitted first, then union identity, case census, per-case checks, payload original type against the side-input witness table, and coverage: a full scope refuses an uncovered union with `IncompleteIdentityWitness` and a reached scope with `UnavailableIdentityWitness`. Direct projection and a ",
      "finding": "FR-352-AC-31 is not atomic: it bundles row retention, tuple-first ordering, census and per-case checks, payload side-input joins, full and reached coverage codes, direct, record-field and object-field projection, foreign and missing refusals and receiver immutability. One tag claims all of them, which is how the untested clauses (record/object field, receiver, foreign) passed as covered."
    },
    {
      "fnd": "FND-002",
      "severity": "low",
      "confidence": "high",
      "method": "spec-review/integrity",
      "check": "compound",
      "artifact_id": "FR-358-AC-15",
      "related_ids": [],
      "path": "spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md",
      "lines": "256",
      "excerpt": "Implemented (QSL-2743). The model tuple and model union row join charges one shared `NativeJoinBudget`: every row of both channels is charged in the common pass before allocation, then the selected clone and the prior-copy publish. The measured exact cumulative work and storage succeed and one unit below either refuses at the first unpaid kind and locus with no partial publish; a zero budget refuses before any row is retained.",
      "finding": "FR-358-AC-15 bundles three separately failing obligations (charge order of common pass, clone and prior copy; exact and one-below limits with no partial publish; zero budget before retention) in one criterion."
    }
  ],
  "bindings": []
}
```
