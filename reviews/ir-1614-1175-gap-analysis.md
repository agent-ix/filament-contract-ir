---
id: SR-9320
title: "gap analysis of the selected-model dimension-equivalence test trace"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1175 at b9ba2e1e (base origin/main fa9ed76e); crates/filament-contract-model/src/checked_package/v2/composite_operands.rs; spec/checked_package/functional/FR-063-dimension-equivalence-node.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
---

## Summary

A scoped gap analysis for filament-contract-ir PR #1175. Ticket: IR-1614. The diff is test-only, so there is no new production code to audit for missing owners. This analysis covers only the requirement binding of the changed test, `dimension_equivalence_resolves_selected_model_package_operands`, and the FR-063 criteria it now asserts. No cargo, quire matrix or quoin matrix command was run. The binding was checked by reading the files with git at the PR head and on filament-spec-language origin/main.

## Examined scope

- FR-063-AC-7 (examined): a missing or non-positive `root_scale` component "is refused as `malformed_wire` at the equivalence node during wire validation, before graph checks." TC-048 lists it. In this repo, the `it` tests in `checked_package_v2_dimension_equivalence.rs` already trace it.
- FR-063-AC-8 (examined): an unreduced positive scale `2000/2` is refused "as `invalid_semantic_graph` at that equivalence node during graph admission." TC-048 lists it. In this repo, the `it` tests already trace it.
- FR-063-AC-1 (context_only): "for two selected-model `Package` endpoints it is `[]`, and the foreign endpoints are validated through their ordered `dimension_node_ids` references and selected-model evidence."
- The test's tag `#[trace("TC-419", "FR-094-AC-13")]` (examined). Neither id exists in this repo's spec. Both come from filament-spec-language: TC-419 is "quantity type nodes and compound units", and its FR-094-AC-13 describes this exact fixture (`acme/rpc`, `acme/flow`, `acme/clock`, `1/1` and `1000/1`). The final clause of that AC reads: "IR's checked-package/v2 reader admits the package ... refusing `invalid_semantic_graph` the same package with a `root_scale` of `0/1`, `-1/1`, `2000/2` or none".

## Verdict

The new expectations close the gap for FR-063-AC-7 and AC-8 in this test. The test's own trace tag is wrong (FND-001). The tag predates this PR, but the PR makes it contradictory: the test now asserts `malformed_wire` for three of the four inputs, while the AC it claims to verify says `invalid_semantic_graph` for all four. The local spec, which follows FSpec FR-322, is the authority, and the new assertions follow it. The fix in this repo changes only the attribute. The stale clause in FR-094-AC-13 belongs to the filament-spec-language spec and is outside this PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The test is tagged `#[trace("TC-419", "FR-094-AC-13")]`, which names filament-spec-language ids that do not exist in this repo's spec. That foreign AC says the IR reader refuses `invalid_semantic_graph` for `0/1`, `-1/1`, `2000/2` or a missing scale. The test now asserts `malformed_wire` for three of these, as FR-063-AC-7 requires, so the test contradicts the AC it claims. The correct local tag is `#[trace("TC-048", "FR-063-AC-1", "FR-063-AC-7", "FR-063-AC-8")]`. | crates/filament-contract-model/src/checked_package/v2/composite_operands.rs:1721-1723 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, head d40be75e43b1082e0ab43aafcb89f21fbed68e02. The tag is now `#[trace("TC-048", "FR-063-AC-7", "FR-063-AC-8")]`, and both ids exist in this repo's TC-048. FR-063-AC-1 was left out on purpose. The test does not assert node `dependencies`, so AC-7 and AC-8 are a sound binding. The new `1/0` and `1/-1` cases add the denominator half of AC-7: `positive_integer` rejects both in `from_document`, which refuses `malformed_wire` at /semantic_graph/nodes/{i}. The stale clause in filament-spec-language FR-094-AC-13 stays outside this repo. |
