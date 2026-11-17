---
id: SR-17940486709564847048035
title: "QSL-2332 PR 1033 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR #1033 at 7fe13fe1eb66f8fe322b67d25d7695af385fb83e vs 90b90fc7; crates/filament-contract-model/src/checked_package/v2/encode.rs; tests/it/checked_package_v2_check_mode.rs; tests/it/checked_package_v2_owners.rs; tests/it/checked_package_v2_dependency_selections.rs; tests/it/checked_package_v2_canonical_encoding.rs; FSpec FR-770/TC-1860; QSL-2332"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: reviews
---
# SR-17940486709564847048035: QSL-2332 PR 1033 code and Rust review

## Summary

Ticket QSL-2332; reviewed frozen PR head `7fe13fe1eb66f8fe322b67d25d7695af385fb83e` against base `90b90fc7`. Rust production and test changes were reviewed against FSpec FR-770/TC-1860 and the ticket acceptance. No code was changed. GitHub Rust Checks reports the workspace `make test` failure in `tc_048_closed_envelope_bytes_equal_independent_canonical_json` (`tests/it/checked_package_v2_canonical_encoding.rs:281`, fixture `v2_all_families`). I ran only that focused test at the PR head and the same test at base: it fails at PR head and passes at base. No aggregate gate was rerun.

## Coverage

| Unit | Role | Excerpt |
| --- | --- | --- |
| FSpec FR-770/AC-1 | examined | `check_mode` is a closed, identity-bearing `linked` or `kernel` member on function nodes; unequal modes at the same package/path definition conflict. |
| FSpec FR-770/AC-2 | examined | `check_mode` is required on function nodes and is forbidden on non-function nodes; `kernel` is forbidden for invariant and precondition clause functions after owner join. |
| `checked_package_v2_check_mode.rs::linked_and_kernel_modes_admit_as_distinct_node_identity` | examined | Both modes admit on a function with the same node id; admitted graph modes and projections differ. |
| `checked_package_v2_check_mode.rs::check_mode_shape_is_closed_and_function_only` | examined | Invalid, absent, and misplaced values are refused at the expected path/code. |
| `checked_package_v2_owners.rs::kernel_mode_is_refused_for_invariant_and_precondition_clause_functions` | examined | Kernel invariant and precondition clause functions refuse after owner join; a body-clause function in kernel mode admits. |
| `checked_package_v2_dependency_selections.rs::diamond_with_unequal_function_check_modes_refuses` | examined | The same selected leaf package/path with unequal function modes refuses as a conflicting definition. |
| `checked_package_v2_canonical_encoding.rs::tc_048_closed_envelope_bytes_equal_independent_canonical_json` | examined | Canonical node encoding matches independently canonicalized JSON for `v2_all_families`. |
| `crates/filament-contract-model/src/checked_package/v2/encode.rs::Encode for CheckedSemanticNodeV2` | examined | Typed checked semantic node serialization emits all identity-bearing node fields. |

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | `Encode for CheckedSemanticNodeV2` omits `check_mode`, so serializing an admitted function node drops its required identity-bearing mode. The focused canonical-encoding test fails on this PR head and passes at base. Add the field in the same canonical position used by `ProjectionFields`. | `crates/filament-contract-model/src/checked_package/v2/encode.rs:195-210`; `tests/it/checked_package_v2_canonical_encoding.rs:249-281` | missing-requirement |

## Verdict

**FAIL.** FND-001 blocks merge. In `encode.rs`, `CheckedSemanticNodeV2::encode_into` writes `semantic_form` and then `semantic_type` without writing `check_mode`; the adjacent `ProjectionFields::encode_into` does write it. Because graph encoding delegates node encoding to this implementation, serialization loses the mode. The targeted test failed on the PR head and passed on base, confirming a PR-caused regression. `git diff --check` passed. No additional aggregate gate was run.
