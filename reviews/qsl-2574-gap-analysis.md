---
id: SR-17940486709564847048083
title: "Gap analysis — filament-contract-ir QSL-2574 await fixture refresh"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; Quoin requirement/test matrix at PR head b65d6525 compared with its change base 358e9e2d"
review_set: subset
---

## Summary

Ticket: QSL-2574. The PR changes test fixtures only and does not change production behavior or specifications. The two await temporal-binding criteria asserted by the changed test remain tagged; the repository matrix also contains pre-existing untagged criteria outside this PR's delta.

## Verdict

**FAIL** — the full repository matrix has 130 untagged criteria; the same 130 were present at the PR change base, so this finding is pre-existing and not introduced by QSL-2574.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | `quoin matrix --repo . --json` reports 130 untagged criteria on the PR head, including FR-038-AC-383 and FR-376/FR-377 criteria. The change base has the same count and the PR adds no specifications or removes trace tags; this is an existing repository coverage gap, not a QSL-2574 regression. | Quoin matrix at b65d6525 and baseline 358e9e2d; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:30 | correct-requirement-no-evidence |

## Scope

- **examined — FR-038-AC-414**, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`: “A missing required compensation or await row (`CB-03` / `AW-03`) refuses `invalid_package`/`timed-node-reference` at that node's `/semantic_graph/nodes/{N}/body`… A valid await row is admitted and round-trips.”
- **examined — FR-038-AC-415**, same file: “For a resolved compensation or await row, a changed `clock_name` (`CB-05` / `AW-05`) refuses `invalid_package`/`timed-node-reference` at `/temporal_bindings/{i}/clock_name`…”.
- **examined — Quoin matrix**, 1,276 criteria at both the PR head and change base; 130 are untagged at both revisions. FR-038-AC-414/415 are tagged on both.
- **examined — PR delta**, only `tests/it/checked_package_v2_protocol_body.rs` and `tests/it/support/checked_package.rs` change; no specification or trace-tag removal is in the diff.

## Coverage

- Static matrix: 1,276 criteria; 130 untagged. The two criteria traced by the changed await test, FR-038-AC-414 and FR-038-AC-415, are tagged by `await_clock_row_is_admitted_and_round_trips`.
- Baseline comparison: 1,276 criteria and 130 untagged at change base 358e9e2d. No untagged criterion was added by the PR diff.
- Reverse gap: no production code changed; the new logic is test-fixture support for FR-038's await temporal-binding rows.
- Evidence axis: the matrix has no run-evidence bindings for these criteria. The targeted run logs on the exact head record 13/13 await-filter tests and 1/1 refinement test passing.
- Plan completion: not assessed.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | The 130 untagged criteria were identical on the original comparison base `358e9e2d` and head `b65d6525`; QSL-2574 changes no specification or trace tags, so the repository-wide gap is outside this ticket. On rebased base `3febe56e` and head `1e019bd3`, the updated count is 127 at both revisions. FR-038-AC-414 and FR-038-AC-415 remain tagged. |

## Disposition pass 1 verification

Ran `quoin matrix --repo` against rebased base `3febe56e` and PR head `1e019bd3`: both report 1,284 criteria, 127 untagged; FR-038-AC-414/415 are tagged at both. The PR diff touches only test files and changes no specification or trace tag. The original repository-level finding is deferred as out of scope for QSL-2574.
