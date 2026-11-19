---
id: SR-17940486709564847048042
title: "QSL-2360 gap analysis of catalog-row reader evidence"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1063 @ 95961535635b814db2c8b6ae2871bd2ae5b16b21; QSL-2360 catalog-row acceptance; FR-038-AC-251; FR-061-AC-5; tests/it/checked_package_v2_catalogue_differential.rs; tests/it/checked_package_v2_refinement.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-038", type: references }
  - { target: "ix://agent-ix/filament-contract-ir/FR-061", type: references }
---

## Summary

The diff-scoped traceability and evidence review covers QSL-2360's three catalog-row cases and their owning reader criteria. Each row now has an admitted writer/readback control and a wrong-shaped operand refusal; the focused runs passed.

## Verdict

**PASS** — the changed test evidence closes the specified reader-test gaps without adding production behavior or leaving a new code-to-requirement gap.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No gaps found | - |

## Coverage

- Reconciliation: `quoin matrix --repo /tmp/qsl2360-pr1063-review --json` (quoin 0.36.7; quire 0.52.0 / engine 0.62.2); matrix showed FR-038-AC-251 and FR-061-AC-5 tagged, with no run evidence attached to matrix rows. Focused test runs supply this review's local execution evidence.
- Scope selection: PR diff contains only integration tests, so reverse-gap and stub checks were limited to the changed tests and the directly related acceptance criteria; no production behavior was added.
- QSL-2360 acceptance: all three catalog rows are admitted and round-trip through writer/readback, and each has a wrong-shaped operand refusal assertion.
- Criteria: FR-038-AC-251 and FR-061-AC-5 tagged and bound to the relevant integration tests; no untagged criterion introduced by this diff.
- Evidence: focused tests passed 1/1 for `collection_only_round_trips_and_refuses_a_non_binder_operand`, `collection_seq_requires_bounded_inputs_matching_binder_and_sequence`, and `refinement_history_round_trips_and_refuses_a_non_reference_operand`.
- Untraced behaviors / stubs: 0 in the changed test surface.
- Plan completion: not assessed
- Semantic review: skipped
