---
id: SR-967175026302197768900474181941935089
title: "Gap analysis — Contract IR repository audit for IR-1582"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; repository spec, source, and tests; IR-1582 changed behavior"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir", type: references }
---

## Summary

Planless repository audit of the Contract IR spec and computed traceability matrix; plan completion was not assessed. The workspace-wide matrix reports untagged criteria, an ignored-only criterion, and untracked test trace tags; the changed requirement-record behavior itself is owned by FR-376 and has tagged tests.

## Verdict

**FAIL** — the workspace matrix contains untagged criteria and one criterion backed only by an ignored test.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | 115 live criteria have no test carrying their criterion id; see the Coverage list for all ids | Computed matrix: untagged criteria |
| FND-002 | high | The only test carrying FR-028-AC-4 is ignored, so this criterion has no running test | FR-028-AC-4, tests/it/cycle_free_model.rs::production_default_all_feature_and_msrv_builds |
| FND-003 | medium | 315 test symbols carry trace ids that match no declared criterion | quire coverage: untracked_symbols |

## Coverage

- Reconciliation: `quoin matrix --repo . --json` (quoin 0.36.8, quire 0.52.0, engine 0.62.2); `quire coverage --scope . --json` inspected for untracked symbols and diagnostics.
- Plan completion: not assessed
- Criteria: tagged 1207 · untagged 115 · tagged-by-ignored-test 1 · method-without-symbol 26 · retired 8
- Evidence: no run evidence 1357 · suspect 0 · stale 0. The evidence store has no run records; Titan's exact-head runs are recorded in the code-review gate results, not this store.
- Stale/untracked tags: 315 symbols from `quire coverage`; diagnostics included 2011 `trace-id-outside-marker` records.
- Untagged criterion ids: FR-019-AC-14, FR-019-AC-27, FR-030-AC-7, FR-036-AC-1, FR-036-AC-2, FR-036-AC-3, FR-036-AC-4, FR-036-AC-5, FR-038-AC-383, FR-038-AC-384, FR-038-AC-385, FR-038-AC-386, FR-038-AC-576, FR-038-AC-577, FR-038-AC-578, FR-038-AC-579, FR-038-AC-580, FR-038-AC-135, FR-038-AC-281, FR-038-AC-282, FR-038-AC-284, FR-038-AC-285, FR-038-AC-287, FR-038-AC-289, FR-038-AC-290, FR-038-AC-291, FR-038-AC-292, FR-038-AC-324, FR-038-AC-326, FR-038-AC-327, FR-038-AC-357, FR-038-AC-359, FR-038-AC-360, FR-038-AC-361, FR-038-AC-362, FR-038-AC-493, FR-038-AC-517, FR-045-AC-1, FR-045-AC-2, FR-045-AC-3, FR-045-AC-4, FR-045-AC-5, FR-045-AC-6, FR-051-AC-2, FR-051-AC-3, FR-052-AC-7, FR-052-AC-9, FR-052-AC-10, FR-052-AC-11, FR-052-AC-26, FR-052-AC-27, FR-057-AC-2, FR-057-AC-3, FR-057-AC-4, FR-057-AC-5, FR-057-AC-6, FR-057-AC-7, FR-057-AC-8, FR-057-AC-10, FR-068-AC-1, FR-068-AC-2, FR-068-AC-3, FR-068-AC-5, FR-069-AC-1, FR-069-AC-2, FR-069-AC-7, FR-071-AC-1, FR-071-AC-2, FR-071-AC-3, FR-071-AC-4, FR-071-AC-5, FR-071-AC-6, FR-071-AC-7, FR-350-AC-1, FR-350-AC-2, FR-350-AC-3, FR-350-AC-4, FR-350-AC-5, FR-350-AC-6, FR-350-AC-8, FR-350-AC-9, FR-350-AC-10, FR-350-AC-11, FR-352-AC-6, FR-352-AC-11, FR-353-AC-1, FR-353-AC-2, FR-353-AC-3, FR-353-AC-4, FR-353-AC-6, FR-353-AC-7, FR-353-AC-8, FR-357-AC-5, FR-357-AC-6, FR-357-AC-7, FR-357-AC-9, FR-357-AC-11, FR-357-AC-16, FR-357-AC-17, FR-358-AC-7, FR-358-AC-11, NFR-001-AC-2, NFR-001-AC-3, NFR-001-M-1, NFR-001-M-2, NFR-001-M-3, NFR-002-M-1, NFR-002-M-2, NFR-003-M-1, NFR-003-M-2, NFR-003-M-3, NFR-003-M-4, NFR-006-M-1, NFR-006-M-2, NFR-006-M-3.
- Reverse gap/stub inspection: the changed validator behavior maps to FR-376 and its existing TC-488 tests; no stub or unowned behavior was found in the changed files. A full manual inventory of all 388 source/test files was not completed.
- Semantic review: skipped (optional; not requested).
