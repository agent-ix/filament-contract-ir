---
id: SR-17940486709564847048120
title: "Code review — QSL-2726 FIFO keys admit quantity, bytes and json_object"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; branch code/fifo-key-quantity at fbcddd32; crates/filament-contract-model/src/checked_package/v2/protocol_body.rs; tests/it/checked_package_v2_protocol_body.rs"
review_set: subset
---

## Summary

Ticket: QSL-2726. Reviewed `git diff origin/main...HEAD` at fbcddd32 (two files, +177/-0) against FSpec FR-726 and the FR-322 "Equality-stable types" paragraph, with the Rust review lane folded in. The change adds `ScalarTypeForm::Unit`, `Bytes` and `JsonObject` to the stable-leaf arm of `MemberCheck::equality_stable` (protocol_body.rs:1885-1888) and adds two TC-1770 tests.

## Examined scope

- `equality_stable` call sites (examined): the function is private and has exactly one caller, the FIFO key check at protocol_body.rs:3334. No map key, set element, equality-operator or DB-key path reaches it, so the widening changes only FIFO key admission. The equality operator, map-key and comparison checks live elsewhere (operations.rs, composite_operands.rs, model_members) and are untouched.
- `ScalarTypeForm::Unit` (examined): this is the bare `scalar_type`/`unit` node, which FR-322 "Derived type forms" names as the type `Quantity<u>` (family `quantity`, operations.rs:2558-2562 maps `Unit`, `CarriedQuantity` and `BoundedQuantity` to `quantity`). `Dimension` and `DimensionEquivalence` are separate vocabulary members (vocabulary.rs:143-144) and still fall to `_ => Ok(false)`. `CompoundUnit` was already admitted (QSL-2548) and is FR-322's formed quantity node. `Float32`/`Float64` still fall to `_`, `float_rounding` still returns false, and `carried_quantity` is still walked through its carrier.
- Depth walk (examined): `CompositeType(_)` (option, sequence, set, bag, ordered_set, map, record, tuple, union) and model `value_type`/`tuple_value_type`/`record_value_type`/`variant_type` push every child through `collect_type_children`, and every popped node is classified again, so a float leaf at any depth under any of these refuses. The walk is bounded by the `WorkMeter` and the visited set; no new allocation or panic path.
- Carries (examined): the `carries` predicate already admits any `ScalarType(_)` and `CompositeType(_)` (protocol_body.rs:3301-3320), so a unit, bytes or json_object message type needed no change. The claim holds.
- Tests (examined): mutants were run on a scratch worktree, `cargo test --test it -- fifo`. Removing `Unit`, `Bytes` or `JsonObject` each fails `protocol_channel_admits_fifo_unit_quantity_keys`. Admitting `Float64` fails both `protocol_channel_fifo_refuses_a_float_leaf_beside_a_quantity` and `protocol_channel_fifo_refuses_float_keys_in_nested_types`. Admitting `Dimension` and `DimensionEquivalence` **survives** (FND-001). The new tests are not tautological: the AC-2 test fails on origin/main.
- Gates: 12/12 FIFO-filtered tests pass at head (`--features conformance-cli`, `-j 4`).

## Verdict

**PASS with findings.** The production change is correct, scoped to FIFO keys and consistent with FR-322 and FR-726. The findings are test-coverage gaps, none blocking.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No test pins that a FIFO key typed by a `scalar_type`/`dimension` or `dimension_equivalence` node refuses. Admitting both in the widened arm survives every FIFO test, so a later edit to this same match arm can let a non-value type in unnoticed. | crates/filament-contract-model/src/checked_package/v2/protocol_body.rs:1872-1889 |
| FND-002 | low | The AC-3 test only nests the float under record and sequence/option. Tuple, union/variant and model `record_value_type` nesting is untested, and `sequence_option_float64` carries no quantity, despite the test's name `..._beside_a_quantity`. | tests/it/checked_package_v2_protocol_body.rs:11016-11017 |
