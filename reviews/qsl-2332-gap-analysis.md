---
id: SR-17940486709564847048036
title: "QSL-2332 PR 1033 acceptance coverage gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1033 at 7fe13fe1eb66f8fe322b67d25d7695af385fb83e vs 90b90fc7; FSpec FR-770/TC-1860; tests/it/checked_package_v2_check_mode.rs; tests/it/checked_package_v2_owners.rs; tests/it/checked_package_v2_dependency_selections.rs; tests/it/checked_package_v2_canonical_encoding.rs; QSL-2332"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: reviews
---
# SR-17940486709564847048036: QSL-2332 PR 1033 acceptance coverage gap analysis

## Summary

Ticket QSL-2332; manually mapped each supplied acceptance criterion (FSpec FR-770/TC-1860) to the changed code and focused integration tests at frozen PR head `7fe13fe1eb66f8fe322b67d25d7695af385fb83e`. The installed `quoin gap-analysis` command is unavailable (`command gap-analysis not found`), so this review records the direct criterion-to-test mapping and the existing focused canonical-encoding failure. No aggregate gate was run.

## Coverage

| Unit | Role | Excerpt |
| --- | --- | --- |
| FSpec FR-770/AC-1 | examined | `check_mode` is a closed, identity-bearing `linked` or `kernel` member on function nodes; unequal modes at the same package/path definition conflict. |
| FSpec FR-770/AC-2 | examined | `check_mode` is required on function nodes and is forbidden on non-function nodes; `kernel` is forbidden for invariant and precondition clause functions after owner join. |
| `checked_package_v2_check_mode.rs::linked_and_kernel_modes_admit_as_distinct_node_identity` | examined | Both modes admit on a function with the same node id; graph values and identity projections retain different modes. |
| `checked_package_v2_check_mode.rs::check_mode_shape_is_closed_and_function_only` | examined | Invalid, absent, and non-function placement cases exercise the closed shape and function-only rule. |
| `checked_package_v2_owners.rs::kernel_mode_is_refused_for_invariant_and_precondition_clause_functions` | examined | Owner-joined invariant and precondition function cases refuse kernel mode; body-clause kernel mode admits. |
| `checked_package_v2_dependency_selections.rs::diamond_with_unequal_function_check_modes_refuses` | examined | Unequal modes for the same selected leaf name refuse with `ConflictingDefinition`. |
| `checked_package_v2_canonical_encoding.rs::tc_048_closed_envelope_bytes_equal_independent_canonical_json` | examined | Existing canonical encoding test catches omission of `check_mode` from typed node serialization. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

**PASS WITH IMPLEMENTATION DEFECT REPORTED BY CODE REVIEW.** Every supplied acceptance criterion maps to a focused test, including a canonical-encoding assertion that detects the implementation defect reported in the code-review artifact. I found no uncovered acceptance criterion. The gap-analysis CLI was unavailable, so the mapping was performed manually; this verdict is limited to the direct criterion-to-test review, not a generated quoin gap report.
