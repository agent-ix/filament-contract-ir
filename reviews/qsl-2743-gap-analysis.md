---
id: SR-967175026302197768900474181941935084515
title: "QSL-2743 PR 1117 gap-analysis of the changed obligations"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; QSL-2743; PR 1117 head 31a74a3c6d48f8f272f82092cd28ba9e377d2516; spec/checked_package/functional/FR-352, FR-358, FR-069; spec/model/functional/FR-019; tests/conformance_fspec/model_union_witness.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-352", type: references }
  - { target: "ix://agent-ix/filament-contract-ir/FR-358", type: references }
  - { target: "ix://agent-ix/filament-contract-ir/FR-069", type: references }
---

## Summary

Planless gap analysis of the obligations PR 1117 changes (FR-352-AC-31, FR-358-AC-15, FR-069-AC-11, FR-019 inventory) against the computed Test Matrix, tagged tests and source. Ticket: QSL-2743. Two of the three new criteria are untagged, and the third is tagged but has hollow evidence for several of its clauses.

## Verdict

**FAIL** — untagged criteria FR-069-AC-11 and FR-358-AC-15 (high).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-069-AC-11 is untagged in quire matrix (quire 0.36.2). The only test naming it carries a space-separated Trace line (model_union_witness.rs:383, 'Trace: FR-352-AC-31 FR-069-AC-11 ...'); the repo binds comma-separated lists, so only the first id binds. | spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md:166; tests/conformance_fspec/model_union_witness.rs:383 |
| FND-002 | high | FR-358-AC-15 is untagged in quire matrix. Its only mention is the space-separated Trace line 'Trace: FR-830-AC-11 FR-358-AC-15.' (model_union_witness.rs:448), whose first id is an FSpec criterion; nothing binds FR-358-AC-15. | spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md:256; tests/conformance_fspec/model_union_witness.rs:448 |
| FND-003 | medium | Four #[trace] attributes name FSpec criteria (FR-830-AC-1 twice, FR-830-AC-9, FR-830-AC-11) that this repository does not declare; quire coverage lists them as untracked_symbols. They bind nothing here and leave the IR criteria they test to the broken Trace lines. | tests/conformance_fspec/model_union_witness.rs:386, 404, 452, 495 |
| FND-004 | medium | FR-352-AC-31 is tagged but its evidence is hollow for several clauses: no test projects a record or object field of an admitted union, checks a refused join leaves the receiver unchanged, refuses a foreign selection with UnknownNode, or asserts that reached scope retains only closure rows. The binders cover the join codes, loci and direct projection only. | spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md:627 |
| FND-005 | medium | Reverse gap: new public behaviour with no owning IR criterion and no test. SelectedTypeDeclarations::unions() output, the model_unions row threading through SelectedMonitorClosureRows into the reached read, and the UnknownNode reclassification in selected_declaration_error are named by no IR acceptance criterion (FR-830 is the FSpec contract, not an IR criterion). | crates/filament-contract-model/src/checked_package/v2/runtime_type_projection.rs:285-289, 1211-1220 |

## Coverage

Reconciliation: quire matrix (quire 0.36.2, engine 0.50.2), no run evidence read. quoin matrix was not run because no evidence store is fed for this branch. Changed criteria: FR-352-AC-31 tagged (3 binders, model_union_witness.rs:387, 405, 496), FR-358-AC-15 untagged, FR-069-AC-11 untagged. quire coverage: 4 untracked symbols in model_union_witness.rs (FR-830-AC-1 x2, FR-830-AC-9, FR-830-AC-11). The scope is the changed obligations and their closure, not the whole repository; the pre-existing untagged FR-019-AC-27 is context only. Semantic review: not run as a separate fan-out; intent-to-test-to-code checks for these criteria are in the code-review SR. Plan completion: not assessed

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
      "id": "FR-352-AC-28",
      "path": "spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md",
      "role": "examined",
      "excerpt": "Implemented (IR-1360). A default-feature external consumer calls the checked public `with_admitted_type_rows` and single `validate_and_rebind` signatures with `&mut NativeJoinBudget`. The selected-type declaration projection uses the same checked admitted-type join; the old unchecked copy path is absent. The public surface exposes no private-map getter, unchecked rebind, completeness flag, wrapper or second validator. Full finite package-local checks use native admitted facts, terminate iteratively, perform no source parse, and acquire no FSL/runtime dependency or pin. Missing emitted declarat"
    },
    {
      "id": "FR-352-AC-29",
      "path": "spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md",
      "role": "examined",
      "excerpt": "Implemented (IR-1360). The checked admitted-type acquisition joins a genuine selected model tuple and source-owned declared unit, preserving their complete private provenance and other witness families. Foreign nodes, wrong selected document/source, wrong tuple/unit form, changed owner/original declaration, compound `UnitId`, duplicate rows and cross-family handle conflicts refuse with the typed first native cause and no partial extension. Exact measured work and storage allowances succeed; one unit below either requirement refuses at the first unpaid kind, amount and locus before the correspo"
    },
    {
      "id": "FR-352-AC-30",
      "path": "spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md",
      "role": "examined",
      "excerpt": "Direct projection of an admitted selected `model/variant_type` with its complete installed model enum shape returns `ValueType::Enum` equal to that shape, retaining ordering and every opaque variant identity. A package clone agrees. The reached-model join and following projection share one `NativeJoinBudget`: their measured exact cumulative work and storage succeed, and one unit below either refuses at the first unpaid kind and locus without a partial projection. An absent shape returns `UnavailableIdentityWitness`, and identities installed for an independent admission refuse `ForeignNode`. A "
    }
  ],
  "findings": [
    {
      "fnd": "FND-001",
      "severity": "high",
      "confidence": "high",
      "method": "gap-analysis",
      "check": "trace",
      "artifact_id": "FR-069-AC-11",
      "related_ids": [
        "FR-352-AC-31"
      ],
      "path": "spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md",
      "lines": "166; tests/conformance_fspec/model_union_witness.rs:383",
      "excerpt": "Implemented (QSL-2743). A model union variant payload naming an object type of the selected package is admitted as a `Reference` payload, so a union such as `Mix.Held(Holder)` passes intake instead of being refused for an undeclared payload type.",
      "finding": "FR-069-AC-11 is untagged in quire matrix (quire 0.36.2). The only test naming it carries a space-separated Trace line (model_union_witness.rs:383, 'Trace: FR-352-AC-31 FR-069-AC-11 ...'); the repo binds comma-separated lists, so only the first id binds."
    },
    {
      "fnd": "FND-002",
      "severity": "high",
      "confidence": "high",
      "method": "gap-analysis",
      "check": "trace",
      "artifact_id": "FR-358-AC-15",
      "related_ids": [
        "FR-830-AC-11"
      ],
      "path": "spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md",
      "lines": "256; tests/conformance_fspec/model_union_witness.rs:448",
      "excerpt": "Implemented (QSL-2743). The model tuple and model union row join charges one shared `NativeJoinBudget`: every row of both channels is charged in the common pass before allocation, then the selected clone and the prior-copy publish. The measured exact cumulative work and storage succeed and one unit below either refuses at the first unpaid kind and locus with no partial publish; a zero budget refuses before any row is retained.",
      "finding": "FR-358-AC-15 is untagged in quire matrix. Its only mention is the space-separated Trace line 'Trace: FR-830-AC-11 FR-358-AC-15.' (model_union_witness.rs:448), whose first id is an FSpec criterion; nothing binds FR-358-AC-15."
    },
    {
      "fnd": "FND-003",
      "severity": "medium",
      "confidence": "high",
      "method": "gap-analysis",
      "check": "trace",
      "artifact_id": "tests/conformance_fspec/model_union_witness.rs::published_positive_vectors_are_admitted_with_their_rows_returned",
      "related_ids": [
        "FR-352-AC-31",
        "FR-358-AC-15"
      ],
      "path": "tests/conformance_fspec/model_union_witness.rs",
      "lines": "386, 404, 452, 495",
      "excerpt": "/// Every published positive is admitted by the strict read and the native join.\n#[trace(\"FR-830-AC-1\")]\n#[test]\nfn published_positive_vectors_are_admitted_with_their_rows_returned() {",
      "finding": "Four #[trace] attributes name FSpec criteria (FR-830-AC-1 twice, FR-830-AC-9, FR-830-AC-11) that this repository does not declare; quire coverage lists them as untracked_symbols. They bind nothing here and leave the IR criteria they test to the broken Trace lines."
    },
    {
      "fnd": "FND-004",
      "severity": "medium",
      "confidence": "high",
      "method": "gap-analysis",
      "check": "coverage",
      "artifact_id": "FR-352-AC-31",
      "related_ids": [
        "FR-830-AC-10"
      ],
      "path": "spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md",
      "lines": "627",
      "excerpt": "Implemented (QSL-2743). Joining the selected model tuple and model union witness rows (FSpec FR-830) returns the complete rows keyed by structural node, retaining each type handle, settled key, case rank, position, name, original case, member and payload as the producer's opaque identities. Tuple rows are admitted first, then union identity, case census, per-case checks, payload original type against the side-input witness table, and coverage: a full scope refuses an uncovered union with `IncompleteIdentityWitness` and a reached scope with `UnavailableIdentityWitness`. Direct projection and a ",
      "finding": "FR-352-AC-31 is tagged but its evidence is hollow for several clauses: no test projects a record or object field of an admitted union, checks a refused join leaves the receiver unchanged, refuses a foreign selection with UnknownNode, or asserts that reached scope retains only closure rows. The binders cover the join codes, loci and direct projection only."
    },
    {
      "fnd": "FND-005",
      "severity": "medium",
      "confidence": "medium",
      "method": "gap-analysis",
      "check": "coverage",
      "artifact_id": "crates/filament-contract-model/src/checked_package/v2/runtime_type_projection.rs::SelectedTypeDeclarations::unions",
      "related_ids": [
        "FR-352-AC-31",
        "FR-019-AC-27"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/runtime_type_projection.rs",
      "lines": "285-289, 1211-1220",
      "excerpt": "    /// Complete selected model unions, each with its original declaration,\n    /// case and payload identities, in structural order.\n    pub fn unions(&self) -> &[super::SelectedModelUnionRow] {\n        &self.unions\n    }",
      "finding": "Reverse gap: new public behaviour with no owning IR criterion and no test. SelectedTypeDeclarations::unions() output, the model_unions row threading through SelectedMonitorClosureRows into the reached read, and the UnknownNode reclassification in selected_declaration_error are named by no IR acceptance criterion (FR-830 is the FSpec contract, not an IR criterion)."
    }
  ],
  "bindings": []
}
```
