---
id: SR-967175026302197768900474181941935084516
title: "QSL-2743 PR 1117 spec-review base checklist of the changed IR rows"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; QSL-2743; PR 1117 head 31a74a3c6d48f8f272f82092cd28ba9e377d2516; FR-352-AC-31, FR-358-AC-15, FR-069-AC-11, FR-019 inventory"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-352", type: references }
  - { target: "ix://agent-ix/filament-contract-ir/FR-358", type: references }
  - { target: "ix://agent-ix/filament-contract-ir/FR-069", type: references }
---

## Summary

Base checklist review of the IR spec rows PR 1117 adds or changes. Ticket: QSL-2743. Two rows name a TC id in their Verification cell; FR-358-AC-15 and FR-352-AC-31 are weaker or vaguer than the FR-830 contract they implement; FR-069-AC-11 has no error path. No hash, digest or pin use was found. No AssuranceProfile exists under spec/, so none applies.

## Verdict

**CONDITIONAL** — medium and low findings only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The Verification cell reads 'Test (FSpec FR-830, TC-2112)'; the checklist requires the cell to name a method only, with no TC- id. | spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md:627 |
| FND-002 | low | The Verification cell reads 'Test (FSpec FR-830, TC-2112)'; the checklist requires the cell to name a method only, with no TC- id. | spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md:256 |
| FND-003 | medium | FR-358-AC-15 is weaker than the contract it implements: it requires only 'the first unpaid kind and locus', while FR-830-AC-11 and Work and storage require the full attempted amount, the channel (model_tuples vs model_unions) and the sidecar row index, the closure-edge boundary (MUW-B-02), charges kept after either channel refuses, and accumulation across joins. Two implementers could both pass this AC and disagree with FR-830. | spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md:256 |
| FND-004 | medium | FR-352-AC-31 does not state the payload scope: it says 'payload original type against the side-input witness table' but not that a data payload is any admitted type but Clock (option, collection, map, bounded, carried-quantity and enum included), nor the Clock vs nullary distinction. The code refuses several admitted forms UnsupportedExactType and still satisfies the AC as written. | spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md:627 |
| FND-005 | low | FR-069-AC-11 states only the admission; it has no error-path clause for a payload naming an object type outside the selected package or document, so the refusal boundary of the intake change is unstated and untestable. | spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md:166 |

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
      "severity": "low",
      "confidence": "high",
      "method": "spec-review/base",
      "check": "other",
      "artifact_id": "FR-352-AC-31",
      "related_ids": [],
      "path": "spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md",
      "lines": "627",
      "excerpt": "Implemented (QSL-2743). Joining the selected model tuple and model union witness rows (FSpec FR-830) returns the complete rows keyed by structural node, retaining each type handle, settled key, case rank, position, name, original case, member and payload as the producer's opaque identities. Tuple rows are admitted first, then union identity, case census, per-case checks, payload original type against the side-input witness table, and coverage: a full scope refuses an uncovered union with `IncompleteIdentityWitness` and a reached scope with `UnavailableIdentityWitness`. Direct projection and a ",
      "finding": "The Verification cell reads 'Test (FSpec FR-830, TC-2112)'; the checklist requires the cell to name a method only, with no TC- id."
    },
    {
      "fnd": "FND-002",
      "severity": "low",
      "confidence": "high",
      "method": "spec-review/base",
      "check": "other",
      "artifact_id": "FR-358-AC-15",
      "related_ids": [],
      "path": "spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md",
      "lines": "256",
      "excerpt": "Implemented (QSL-2743). The model tuple and model union row join charges one shared `NativeJoinBudget`: every row of both channels is charged in the common pass before allocation, then the selected clone and the prior-copy publish. The measured exact cumulative work and storage succeed and one unit below either refuses at the first unpaid kind and locus with no partial publish; a zero budget refuses before any row is retained.",
      "finding": "The Verification cell reads 'Test (FSpec FR-830, TC-2112)'; the checklist requires the cell to name a method only, with no TC- id."
    },
    {
      "fnd": "FND-003",
      "severity": "medium",
      "confidence": "high",
      "method": "spec-review/base",
      "check": "soundness",
      "artifact_id": "FR-358-AC-15",
      "related_ids": [
        "FR-830-AC-11"
      ],
      "path": "spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md",
      "lines": "256",
      "excerpt": "Implemented (QSL-2743). The model tuple and model union row join charges one shared `NativeJoinBudget`: every row of both channels is charged in the common pass before allocation, then the selected clone and the prior-copy publish. The measured exact cumulative work and storage succeed and one unit below either refuses at the first unpaid kind and locus with no partial publish; a zero budget refuses before any row is retained.",
      "finding": "FR-358-AC-15 is weaker than the contract it implements: it requires only 'the first unpaid kind and locus', while FR-830-AC-11 and Work and storage require the full attempted amount, the channel (model_tuples vs model_unions) and the sidecar row index, the closure-edge boundary (MUW-B-02), charges kept after either channel refuses, and accumulation across joins. Two implementers could both pass this AC and disagree with FR-830."
    },
    {
      "fnd": "FND-004",
      "severity": "medium",
      "confidence": "high",
      "method": "spec-review/base",
      "check": "ambiguous",
      "artifact_id": "FR-352-AC-31",
      "related_ids": [
        "FR-830-AC-4",
        "FR-830-AC-9"
      ],
      "path": "spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md",
      "lines": "627",
      "excerpt": "Implemented (QSL-2743). Joining the selected model tuple and model union witness rows (FSpec FR-830) returns the complete rows keyed by structural node, retaining each type handle, settled key, case rank, position, name, original case, member and payload as the producer's opaque identities. Tuple rows are admitted first, then union identity, case census, per-case checks, payload original type against the side-input witness table, and coverage: a full scope refuses an uncovered union with `IncompleteIdentityWitness` and a reached scope with `UnavailableIdentityWitness`. Direct projection and a ",
      "finding": "FR-352-AC-31 does not state the payload scope: it says 'payload original type against the side-input witness table' but not that a data payload is any admitted type but Clock (option, collection, map, bounded, carried-quantity and enum included), nor the Clock vs nullary distinction. The code refuses several admitted forms UnsupportedExactType and still satisfies the AC as written."
    },
    {
      "fnd": "FND-005",
      "severity": "low",
      "confidence": "medium",
      "method": "spec-review/base",
      "check": "coverage",
      "artifact_id": "FR-069-AC-11",
      "related_ids": [],
      "path": "spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md",
      "lines": "166",
      "excerpt": "Implemented (QSL-2743). A model union variant payload naming an object type of the selected package is admitted as a `Reference` payload, so a union such as `Mix.Held(Holder)` passes intake instead of being refused for an undeclared payload type.",
      "finding": "FR-069-AC-11 states only the admission; it has no error-path clause for a payload naming an object type outside the selected package or document, so the refusal boundary of the intake change is unstated and untestable."
    }
  ],
  "bindings": []
}
```
