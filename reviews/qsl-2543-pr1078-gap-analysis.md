---
id: SR-17940486709564847048090
title: "QSL-2543 PR 1078 requirement-record gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir#1078@d500ef77026001048f60c27f361414aabadf5afa vs 383857b64ac16dd172d86afcfb8369db339349cd; spec/checked_package/functional/FR-376-admit-the-requirement-records-of-a-checked-package.md; spec/checked_package/matrix/TC-488-requirement-records-read-from-an-admitted-package.md; crates/filament-contract-model/src/checked_package/v2/requirement_records.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-376
    type: references
---
# SR-17940486709564847048090: QSL-2543 PR 1078 requirement-record gap analysis

## Summary

Ticket: QSL-2543. Plan completion: not assessed. The review was limited to the requirement-record behavior in this PR. `quire matrix --scope . --format json` reports FR-376-AC-1, FR-376-AC-2, and FR-376-AC-3 as `untagged`, with no binders. The changed module has three tests for wire enum decoding and row encoding but no public package-reader tests for TC-488. No repository-wide reverse-gap claim is made.

## Verdict

**FAIL.** All three acceptance criteria for the new reader behavior are untagged and the stated TC-488 test is absent. The only supplied focused run passed 3 tests, none exercising those criteria.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-376-AC-1 through AC-3 are untagged in the computed matrix and no TC-488 public-reader test exists in this change. The admission/typed-read path, required refusal cases, stage ordering, and budget behavior therefore have no acceptance-bound test. | spec/checked_package/matrix/tests.md:25, spec/checked_package/matrix/tests.md:84; crates/filament-contract-model/src/checked_package/v2/requirement_records.rs:1178-1294 |

## Coverage

Plan completion: not assessed. Scope examined: FR-376-AC-1 through FR-376-AC-3 and TC-488. Matrix result for each FR-376 criterion: untagged. The repository-wide reverse-gap and stub audit was excluded to honor the PR-specific review scope. Optional semantic review was skipped.

Reviewed fix head `fd9a48164b203bff94e5010546cb6f02c01f0a6f`. The computed matrix now marks all FR-376-AC-1/2/3 criteria `tagged`, with binders in `tests/conformance_fspec/requirement_records.rs`; TC-488 public-reader tests exist. Their focused run is explicitly pending and is not counted as passing evidence.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | TC-488 coverage remains partial: it omits the specified probabilistic/requirement-record stage-precedence cases and several FR-825 refusal mutations. The budget test derives the expected work from an ample-budget read of the same implementation, so it does not independently establish the required visit count or one body walk per owner. | tests/conformance_fspec/requirement_records.rs:439-477, 587-754 |

## Dispositions

Round 1 outcomes:

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The matrix has tagged tests for all three FR-376 criteria, and the public-reader TC-488 suite is present. Focused execution remains pending. |
| FND-002 | still-open: required acceptance cases remain untested | The suite omits the specified precedence cases and several refusal mutations. Its budget test derives the expected work from the same implementation's ample-budget run, so it does not independently assert the required count or one body walk per owner. The focused test run is also still pending. |
