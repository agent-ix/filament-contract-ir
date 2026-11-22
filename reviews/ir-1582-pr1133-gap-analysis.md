---
id: SR-967175026302197768900474181941935084589
title: "Gap analysis — IR-1582 PR #1133"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; repository spec, source, and tests; IR-1582 PR #1133 changed behavior"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir", type: references }
---

## Summary

Planless repository audit using the current computed traceability matrix and coverage report. FR-376-AC-3 has tagged running tests, including TC-488's 100,000-record scale lane; the changed named-record index is owned by FR-376. The repository-wide matrix has untagged and ignored-only criteria plus undeclared test trace ids. Reverse-gap inspection confirmed the changed behavior is specified, but did not manually inventory every source and test symbol.

## Verdict

**FAIL** — the workspace matrix has 115 live untagged criteria and one criterion backed only by an ignored test; coverage reports 315 untracked symbols.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | 115 live criteria have no test carrying their criterion id | Computed matrix: untagged criteria |
| FND-002 | high | FR-028-AC-4 is backed only by an ignored test | FR-028-AC-4, tests/it/cycle_free_model.rs::production_default_all_feature_and_msrv_builds |
| FND-003 | medium | 315 test symbols carry trace ids that name no declared criterion | quire coverage: untracked_symbols |

## Coverage

- Reconciliation: `quoin matrix --repo . --json` (quoin 0.36.8; quire 0.52.0, engine 0.62.2) and `quire coverage --scope . --json`.
- Plan completion: not assessed.
- Criteria: tagged 1207 · untagged 115 · tagged-by-ignored-test 1 · method-without-symbol 26 · retired 8; total 1357.
- Evidence: no run evidence 1357 · suspect 0 · stale 0. The evidence store has no run records; Titan runs are not registered in it.
- Stale/untracked tags: 315 symbols; coverage diagnostics 2049.
- Untagged criteria: FR-019-AC-14, FR-019-AC-27, FR-030-AC-7, FR-036-AC-1, FR-036-AC-2, FR-036-AC-3, FR-036-AC-4, FR-036-AC-5, FR-038-AC-383, FR-038-AC-384, FR-038-AC-385, FR-038-AC-386, FR-038-AC-576, FR-038-AC-577, FR-038-AC-578, FR-038-AC-579, FR-038-AC-580, FR-038-AC-135, FR-038-AC-281, FR-038-AC-282, FR-038-AC-284, FR-038-AC-285, FR-038-AC-287, FR-038-AC-289, FR-038-AC-290, FR-038-AC-291, FR-038-AC-292, FR-038-AC-324, FR-038-AC-326, FR-038-AC-327, FR-038-AC-357, FR-038-AC-359, FR-038-AC-360, FR-038-AC-361, FR-038-AC-362, FR-038-AC-493, FR-038-AC-517, FR-045-AC-1, FR-045-AC-2, FR-045-AC-3, FR-045-AC-4, FR-045-AC-5, FR-045-AC-6, FR-051-AC-2, FR-051-AC-3, FR-052-AC-7, FR-052-AC-9, FR-052-AC-10, FR-052-AC-11, FR-052-AC-26, FR-052-AC-27, FR-057-AC-2, FR-057-AC-3, FR-057-AC-4, FR-057-AC-5, FR-057-AC-6, FR-057-AC-7, FR-057-AC-8, FR-057-AC-10, FR-068-AC-1, FR-068-AC-2, FR-068-AC-3, FR-068-AC-5, FR-069-AC-1, FR-069-AC-2, FR-069-AC-7, FR-071-AC-1, FR-071-AC-2, FR-071-AC-3, FR-071-AC-4, FR-071-AC-5, FR-071-AC-6, FR-071-AC-7, FR-350-AC-1, FR-350-AC-2, FR-350-AC-3, FR-350-AC-4, FR-350-AC-5, FR-350-AC-6, FR-350-AC-8, FR-350-AC-9, FR-350-AC-10, FR-350-AC-11, FR-352-AC-6, FR-352-AC-11, FR-353-AC-1, FR-353-AC-2, FR-353-AC-3, FR-353-AC-4, FR-353-AC-6, FR-353-AC-7, FR-353-AC-8, FR-357-AC-5, FR-357-AC-6, FR-357-AC-7, FR-357-AC-9, FR-357-AC-11, FR-357-AC-16, FR-357-AC-17, FR-358-AC-7, FR-358-AC-11, NFR-001-AC-2, NFR-001-AC-3, NFR-001-M-1, NFR-001-M-2, NFR-001-M-3, NFR-002-M-1, NFR-002-M-2, NFR-003-M-1, NFR-003-M-2, NFR-003-M-3, NFR-003-M-4, NFR-006-M-1, NFR-006-M-2, NFR-006-M-3.
- Reverse gaps and stubs: inspected the changed validator and its mapped TC-488 tests; the changed behavior maps to FR-376-AC-3 and no stub or unowned behavior was found there. A full manual inventory of all source/test symbols was not completed; this audit does not claim repository-wide reverse-gap completeness.
- Semantic review: skipped (optional; not requested).

## Findings Details

### FND-001

The matrix reports 115 live criteria with `static_status: untagged`, meaning no test carries each criterion id. These gaps are repository-wide and do not include FR-376-AC-3.

### FND-002

The matrix marks FR-028-AC-4 `tagged-by-ignored-test`; its only binder is `tests/it/cycle_free_model.rs::production_default_all_feature_and_msrv_builds`, which is ignored.

### FND-003

Quire coverage reports 315 test symbols with trace ids that name no declared criterion. The first sampled symbol is `crates/filament-contract-model/src/checked_package/v2/body_locations.rs::tests::a_row_rebuilds_its_path_from_the_previous_one` tagged `FR-322-AC-510` at line 371.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | deferred | The untagged criteria are pre-existing repository-wide matrix gaps unrelated to the requirement-record index in PR #1133. |
| FND-002 | deferred | FR-028-AC-4's ignored-only test is outside the requirement-record change and no related test was modified in PR #1133. |
| FND-003 | deferred | The untracked trace symbols are pre-existing and outside the changed requirement-record source/test files. |
