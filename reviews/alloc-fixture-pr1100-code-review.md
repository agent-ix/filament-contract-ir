---
id: SR-17940486709564847048114
title: "code-review of QSL-2726 PR #1100 (read_storage_alloc dependencies_10000 tuple fixture)"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR #1100 at 7a091b60 (base origin/main aa580eb7); tests/it/support/read_storage_inputs.rs"
review_set: subset
---

## Summary

Ticket: QSL-2726. One test-support file changes: `dependencies_10000` in `tests/it/support/read_storage_inputs.rs`. The rust-review lane is folded in here. Before the change, the tuple type node `allocation-tuple/type` declared 10000 positions that all referenced the template's `semantic_type`, which is `boolean`, with `dependencies: [boolean]`. After the change, position i references flat node i (`allocation-flat/{i:05}`) and `dependencies` lists the same 10000 ids. The tuple value node `allocation-tuple/value` is unchanged: 10000 dependencies, and member i references flat node i.

## Examined scope

- Reader (examined). `git diff origin/main...HEAD -- crates/` is empty, so the reader is not touched and not loosened.
- Root cause (examined) in `operations.rs` `tuple_value_defect` (#1097) and `referenced_operand_type`. The flat nodes are cloned from the `expression`/`reference` template. `is_type_shaped(K::Expression(ExpressionForm::Reference))` is `true`, so the operand type of a reference to flat node i is flat node i itself. Under the old fixture the expected type was `boolean` and the actual type was flat node i, so `conforms_to_type` returned false and the reader refused with `IllTyped`/`TypeMismatch`. That is the main-red failure. Under the new fixture expected == actual at every position, so the structural-id compare passes. The loop visits all 10000 positions and does not stop early: `typed_reference` succeeds and every target is indexed.
- Same measurement (examined). The test `whole_read_node_with_10000_dependencies` (tests/read_storage_alloc.rs:322-327, trace TC-472 / FR-038-AC-511) is unchanged and still calls `whole_read_adverse`, which is `Expected::Admitted` with `ReadViewScope::Core`. The required outcome class is the same (Admitted). The allocation accounting is the same `read_allocation_proof::assert_read` unpaid-heap bound. The value node still carries exactly 10000 dependencies, and the `assert_eq!(..., 10_000)` guard is kept.
- Shape delta (examined). The tuple type node now also carries 10000 distinct dependencies, and its body references 10000 distinct nodes instead of one node 10000 times. The input therefore grows: about 10000 more dependency ids and a larger aggregate body. The oracle is relative (unpaid heap beyond the admitted read storage, within a fixed bound), so a bigger admitted input does not weaken it. It adds a second wide node, which makes the adverse shape stronger, not weaker. The type node's `dependencies` and body references agree, as they did before.
- Comment (examined). The new comment says the flat nodes are type-shaped `reference` nodes, so each one is its own operand type, and cites FR-347 position conformance. This is accurate.
- Other tuple-value fixtures in tests/it/support (examined). `monitor_plan.rs::constant` builds `tuple_value` members as references to `value` nodes typed by the declared field type. Value forms are not type-shaped, so the operand type is the member's `semantic_type` = the declared field type, which is well-typed by construction. `fspec_option_inputs.rs` emits `tuple_value_type` model nodes only, with no `tuple_value` graph nodes. No other support file builds `tuple_value`. No latent mismatch of the same kind was found.
- Rust lane (examined). The change uses no new `unwrap`. `values` is reused by reference for both bodies. Formatting is consistent with rustfmt (coder ran fmt and clippy -D warnings).
- Gate run by this reviewer (examined): `flock /tmp/cargo-slot.lock cargo test --features conformance-cli --test read_storage_alloc whole_read_node_with_10000_dependencies` with CARGO_TARGET_DIR in the worktree and FILAMENT_SPECIFICATION_DIR=FSPEC-main-ci. Result: 1 passed, 0 failed (37.4 s). The coder reported the full read_storage_alloc binary 32/32.

## Verdict

PASS. This is a fixture-only correction that makes the adverse input well-typed under #1097's FR-347 tuple position check. The reader is unchanged, the test's outcome class and allocation oracle are unchanged, and it still has a node with 10000 dependencies. No defects were found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
