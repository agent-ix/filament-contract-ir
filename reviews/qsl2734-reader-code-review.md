---
id: SR-2734100001
title: "code-review and rust-review of QSL-2734 dimension-equivalence borrowed view (PR 1116)"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1116 at 844467f5 (base origin/main 24653ef4); crates/filament-contract-model/src/checked_package/v2/identity.rs, tests/it/checked_package_v2_dimension_equivalence.rs"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in, for filament-contract-ir PR #1116. Ticket: QSL-2734 (IR slice only; the FSL BareWorld half is outside this PR).

The production change is one method, `NominalBody::dimension_equivalence_details`, which matches the private `NominalBodyKind::DimensionEquivalence` and returns shared borrows of the stored `[CheckedDimensionOperand; 2]` and `CheckedRational`. Every other kind returns `None`. It runs in constant time, does not allocate, charges no budget, and does not expose `NominalBodyKind`. It adds no constructor or `&mut` path, so the local/package operand distinction stays as the reader admitted it.

Evidence: coder logs `/tmp/ir-qsl2734-eq.log` (12 passed, exit 0, written before the mutant), `/tmp/ir-qsl2734-nom.log` (8 passed, exit 0) and `/tmp/ir-qsl2734-pub.log` (tc_058 passed, exit 0). The mutant log `/tmp/ir-qsl2734-mut.log` shows the package view test failing with the accessor forced to `None`. The last binary the coder built was the mutant, so the reviewer reran the exact accessor filter on the clean committed tree: `flock /tmp/cargo-slot.lock cargo test -j 4 -p filament-contract-ir --features conformance-cli --test it equivalence_view`. Result: 2 passed, exit 0, log `/tmp/ir-qsl2734-reviewer-eqview.log`. No gates or broad runs.

## Examined scope

- identity.rs `dimension_equivalence_details` (examined): it borrows only, keeps the opaque kind, keeps authored order and returns `None` for other forms.
- `local_equivalence_view_borrows_exact_operands_and_ratio` (examined): asserts the exact operand array and the exact 3/2 ratio, and resolves both operands as dimensions.
- `package_equivalence_view_resolves_in_selected_dependency` (examined): asserts Package then Local, the package name and node id, and resolution in the selected dependency. It also asserts the foreign node is absent from the importer's `term`, that a reference naming another package resolves nowhere, and that the operand serialization equals the wire. The mutant shows this oracle can fail.
- `wrong_kind_operand_refuses_and_is_not_a_dimension` (examined): asserts only the refusal code. Precise cause and pointer for a wrong-kind operand are already asserted by `foreign_operand_wrong_kind_and_self_minted_local_refuse_at_operand`.
- FR-019 `SelectedModelEnumRows` inventory repair (examined): it adds one name in alphabetical order, the type is exported at lib.rs:246, and nothing else changed.

## Verdict

CONDITIONAL, with low findings only. The implementation is correct. One test's negative loop covers fewer nominal forms than TC-048 claims (FND-001).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The "every other nominal form" loop never passes a unit or enum body to the accessor: nodes 1 and 2 are dimensions, and node 0 is a boolean empty-aggregate node that the let-else drops before the accessor runs. TC-048 says the view returns none for dimension, unit and enum bodies. The wrong-kind test's doc comment also calls node 0 a unit. | tests/it/checked_package_v2_dimension_equivalence.rs:581-584,639-646; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:2797 |
