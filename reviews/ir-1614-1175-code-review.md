---
id: SR-6268
title: "code-review and rust-review of the selected-model dimension-equivalence test update"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1175 at b9ba2e1e (base origin/main fa9ed76e); crates/filament-contract-model/src/checked_package/v2/composite_operands.rs"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in, for filament-contract-ir PR #1175. Ticket: IR-1614. The diff is one commit that changes one unit test, `dimension_equivalence_resolves_selected_model_package_operands`, in `composite_operands.rs`. No production code, spec or dependency changed.

No cargo command was run for this review. The verdict comes from reading the test and tracing the reader code at the PR head. The test has not run since #1164: on main, the `it` binary failed first and `make test` stopped there. Also, #1160, which added this test, merged with Rust Checks failing, so no CI record shows that any part of this test has ever passed.

## Examined scope

- FR-063-AC-7 (examined): "a strict-reader input with the `root_scale` member omitted, or with either `numerator` or `denominator` a zero or negative integer string, is refused as `malformed_wire` at the equivalence node during wire validation, before graph checks."
- FR-063-AC-8 (examined): "With a schema-valid positive scale `numerator: "2000"`, `denominator: "2"`, the strict reader admits the wire shape, then refuses the unreduced value as `invalid_semantic_graph` at that equivalence node during graph admission."
- `0/1` and `-1/1` (examined). The strict read decodes the body with `NominalBody::from_document` (identity.rs:1592 and mod.rs:3116, with `node_at = node_pointer(position)`). `positive_integer` rejects `0` and `-1`, and the `root_scale` branch refuses `malformed_wire` at `node_path()` (identity.rs:955), which is `/semantic_graph/nodes/{i}`. `flat_wire` has no root_scale rule, so nothing refuses earlier. This matches AC-7. The `it` test `malformed_equivalence_shape_refuses_at_its_node` asserts the same code and pointer through a full read.
- `root_scale` omitted (examined). The second `dimension_equivalence` arm refuses `malformed_wire` at `owner_node_pointer(at)` (identity.rs:1091). This removes the `/body` suffix and gives `/semantic_graph/nodes/{i}`. This matches AC-7.
- `2000/2` (examined). Both components are positive, so the decode admits the shape. `check_kind`'s `valid_rational` does not check that the scale is reduced. In graph admission, `validate_dimension_equivalence` calls `validate_rational`, and the `coprime` check refuses `invalid_semantic_graph` at `/semantic_graph/nodes/{i}`. `validate_equivalence_graph` gives the same code and pointer. This matches AC-8.
- #1173 exoneration (examined). #1173 changed only the post-parse `check_kind`. It replaced an inline positive-integer test with `valid_rational(.., true)`, which applies the same predicate and the same `malformed_wire` code. A strict read refuses a non-positive component in `from_document` before `check_kind` runs. The claim is correct. The stale expectation dates from #1164, which changed `invalid_semantic_graph` to `malformed_wire`, and #1165, which moved the missing-member locus to the node.
- Later assertions that have not run since #1164 (examined). Their fixture and the reader paths they use have not changed since #1160. `("acme/missing", ..)`: the operand loop in `validate_dimension_equivalence` finds neither a selected model nor a dependency, so it refuses `missing_declaration`/`missing-selection`. `("acme/flow", <rpc node>)`: `acme/flow` is selected but has no such dimension, so it refuses `missing_declaration`/`missing-name`. Structural keys are rebuilt from `node_id` paths, not from body operands, so these mutations do not fail a key check first. `no_selection`: the test accepts any refusal. The inconsistent cycle case refuses `invalid_semantic_graph` at `/semantic_graph/nodes/20`: there are 9 base nodes, 9 model nodes and 2 equivalences, so the appended node is at position 20. That case passes for a reason other than the cycle; see FND-001.
- Weakened or lost assertions (examined). None. Each case still asserts the exact refusal code, and each now also asserts the refusal pointer, which is stronger. Only the debug text in one panic message was dropped (FND-002).

## Verdict

Approve. Each new expectation is what FR-063-AC-7 and AC-8 require, with the code and the `/semantic_graph/nodes/{i}` locus. The trace predicts that the whole test passes. That prediction depends on the happy-path admission at the top of the test, which has no CI record of passing and was read here, not run. The #1173 exoneration is correct. FND-001 is a test-intent defect that predates this PR. It does not block the red-main fix.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The "inconsistent dimension equivalence cycle" block never reaches the cycle-product check. The appended node copies `semantic_type` from the ordinal-0 node, so `validate_dimension_equivalence` refuses it as a non-self semantic_type at nodes/20 (identity.rs:2154). The node also repeats the same operand pair, which the repeated-pair check refuses (identity.rs:1989). The assertion passes, but not for the reason the panic message states. This predates the PR (#1160). | crates/filament-contract-model/src/checked_package/v2/composite_operands.rs:1889-1930 |
| FND-002 | low | The missing-root_scale case now uses `let ... else { panic!("missing root scale must refuse") }`, which drops the `{refused:?}` debug output that the old `assert!` message printed. An unexpected admission would print no result. | crates/filament-contract-model/src/checked_package/v2/composite_operands.rs:1841-1843 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, head d40be75e43b1082e0ab43aafcb89f21fbed68e02. The cycle block now appends a third equivalence over a new pair, (acme/clock, acme/flow), at position 20. The node has a self `semantic_type`, `[]` dependencies, an anchor ordinal of 2 and a source start of 20. Each per-node check passes: ascending ids, owner, a reduced scale, a self type, no repeated pair, and in-order anchor ordinals 0, 1, 2. In the base-only `equivalence_potentials` pass, the walk starts at the clock dimension, because the package `acme/clock` sorts first. The edges are in node-id order: clock-rpc 1000, flow-rpc 1, closing. The closing edge is the last one checked: F→C needs F times r to equal 1 with F = 1/1000, so 1000 admits and 2 fails `require` at node_pointer(20). That gives `invalid_semantic_graph`, no cause, at /semantic_graph/nodes/20, raised by the cycle check itself. `ExactRatio::multiply` reduces the product, so 1000/1000 compares equal to 1. |
| FND-002 | fixed | Round 1, head d40be75e43. The missing-scale case and the cycle case now use `match` with `other => panic!("...: {other:?}")`, so an unexpected result prints its debug output. |
