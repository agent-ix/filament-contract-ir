---
id: SR-17940486709564847048107
title: "code-review of QSL-2711 PR #1096 (ir1052 resolved-graph lookup, FR-375 retag)"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR #1096 at 3fcf6a01 (merge base af45a983); tests/conformance_fsl_ir1052.rs, tests/it/checked_package_v2_refinement.rs"
review_set: subset
---

## Summary

Ticket: QSL-2711. Test-only PR, two commits. eda933f8 changes `tests/conformance_fsl_ir1052.rs` to look reference targets up in `CheckedPackageV2::resolved_graph()` instead of `graph()`. 3fcf6a01 removes the `FR-375-AC-31`/`FR-375-AC-32` trace ids from six tests in `tests/it/checked_package_v2_refinement.rs`. No production code changes. The rust-review lane is folded in here.

## Examined scope

- `tests/conformance_fsl_ir1052.rs::node` and the `transitions` filter (examined). `resolved_graph()` is documented at `crates/quire-contract-model/src/checked_package/v2/mod.rs:1815-1820` as "the graph with every node-held operand resolved to the index segment of the node it names, the form the reader checks and the typed views borrow from". `graph()` (1712-1717) is the slot form, where node-held operands are slots. A `CheckedTerm::Reference { target }` taken from an admitted node body carries the index form, so only `resolved_graph()` node ids can match it byte for byte. The transition is taken from the same graph, so its body and the lookup use the same id form. Many other tests already use this pattern, for example `tests/it/checked_package_v2_refinement.rs` and `tests/conformance_qspec/transition.rs`.
- Assertion strength (examined). Every `assert!`/`assert_eq!`/`let-else panic` in both tests is unchanged: one transition, an authentic claim, a typed StateTransition application, Boolean result, binding names `anchor, subject, inputs, pre, post`, a subject projection of `on`, one `parameter` input, and a Boolean `state_clause` post. Only the graph accessor changed. The test is not weakened.
- Reproduction (examined). The FSL checkout is `/home/peter/dev/worktrees/FSL-QSL2711` at f040d334, with `emit_checked_v2` built after that commit. With main's version of the test, `fsl_boolean_transition_readback_retains_typed_subject_inputs_and_result` panics at `tests/conformance_fsl_ir1052.rs:119` (`admitted reference target`). With the PR's version, both tests pass (2 passed, 0 failed).
- `tests/it/checked_package_v2_refinement.rs` retag (examined). The removed ids name FSpec's FR-375. This repository has its own `FR-375`, "Check source operation-body coverage at the identity-row join", which declares only `FR-375-AC-1`, so the local `FR-375-AC-31`/`AC-32` resolve to nothing. An earlier gap analysis already flagged `FR-375-AC-32` as an untracked trace id (`reviews/SR-17940486709564847048081-qsl-2548-gap-analysis.md` FND-284). Each of the six tests keeps its `TC-062` and `FR-061-AC-5/14/15` tags, and FR-061 is the local requirement that cites FSpec FR-375. No assertion changed.

## Verdict

PASS. The fix is causal and does not weaken the test. The retag removes trace ids that cannot resolve and keeps the correct bindings. No defects.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
