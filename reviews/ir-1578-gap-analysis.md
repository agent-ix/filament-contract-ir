---
id: SR-967175026302197768900474181935084589
title: "IR-1578 PR 1129 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; crates/filament-contract-model/src/checked_package/v2/runtime_type_projection.rs; tests/it/nominal_reads.rs"
review_set: subset
---

## Summary

Ticket: IR-1578. Planless repository gap analysis for the exact PR diff at `bd876f8f8da5fc7fc5fde706283ad8cf14750976`. Optional semantic review was not requested and was skipped.

## Verdict

**PASS** — the introduced source-enum projection behavior has owning criteria and a trace-tagged regression; no new reverse gap or stub was found in the changed production/test paths. Plan completion: not assessed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

`quoin matrix` and `quire matrix` both identify the newly added `tests/it/nominal_reads.rs::source_enum_runtime_type_projection_requires_authenticated_row` as the test binder for `FR-354-AC-6` and `FR-360-AC-4`; both criteria are tagged. Quoin reports no local run evidence; the separate Titan receipts supplied by the lead cover the exact focused test and broader checks. No production or test stubs/TODO markers were found in the changed paths. The new code behavior is owned by the two criteria above; no uncovered new behavior was identified.

The repository-wide strict Quire matrix exits nonzero with existing aggregate diagnostics: 115 untagged criteria, one criterion tagged only by ignored tests, and 88 design gaps. The normal computed matrix also emits existing repository-wide declaration/symbol diagnostics. These are not attributable to this PR; the changed behavior’s two trace links are present in both computed matrices. No applicable AssuranceProfile was found. No plan was supplied; plan completion was not assessed.

Examined:

- `FR-354-AC-6`, `spec/checked_package/functional/FR-354-project-admitted-runtime-context.md`: “With FSL-stage enum declaration and member rows, an ordered and an unordered selected enum each yield exact case order, member identity and position sufficient for `EnumMemberIndex`; absent, duplicate, mismatched-case or foreign member witnesses refuse before any partial index.”
- `FR-360-AC-4`, `spec/checked_package/functional/FR-360-charge-runtime-type-projection-with-shared-budget.md`: “At sufficient limits, representative source and model runtime type reads, selected nominal reads, and selected unit-definition reads return the same complete values, original identities, order, and semantic refusal causes as the existing behavior. Equal input, rows, and limits produce equal values, cumulative charges, deltas, or first refusals for the original and a clone of the admitted package.”

Titan exact-head receipts supplied by the lead: target compilation passed, the focused exact-name test passed exactly one test, and `cargo check --locked --workspace` passed. No local builds/tests were run. Plan completion: not assessed.
