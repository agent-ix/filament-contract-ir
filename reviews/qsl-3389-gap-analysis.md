---
id: SR-967175026302197768900474181941935084780
title: "Gap analysis — filament-contract-ir repository"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; spec/, crates/, src/, tests/"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir", type: references }
---

## Summary

This planless, repository-driven audit used the computed trace matrix and checked the PR's changed reader behavior against its criterion, callers, and tests. The changed behavior is tagged, but the repository matrix reports 122 untagged criteria, one criterion covered only by an ignored test, and 411 untracked symbol records.

## Verdict

**FAIL** — the repository-wide matrix has high-severity trace gaps. The changed criterion, FR-038-AC-597, is tagged; the listed gaps are broader repository findings and are not introduced by PR #1186.

## Findings

| ID      | Severity | Summary                                                                 | Refs |
| ------- | -------- | ----------------------------------------------------------------------- | ---- |
| FND-001 | high     | The computed matrix reports 122 live criteria with no tagged test.       | Matrix output: 122 untagged criteria |
| FND-002 | high     | FR-028-AC-4 is bound only to an ignored build-lane test.                 | FR-028-AC-4 |
| FND-003 | medium   | The coverage report contains 411 untracked symbol records; several use external FSpec ids and require owner classification. | quire coverage: untracked_symbols |

## Coverage

- Reconciliation: quoin matrix (quoin 0.36.8, quire 0.52.0 / engine 0.62.2).
- Plan completion: not assessed.
- Criteria: tagged 1233 · untagged 122 · tagged-by-ignored-test 1 · method-without-symbol 26 · retired 8. Matrix total: 1390.
- Evidence: no run evidence for all 1390 matrix entries; no bound, stale, suspect, or undischarged evidence status was reported.
- Untracked symbols: 411 records, 108 unique trace ids. The records include external FSpec ids such as FR-322-AC-510, so they are reported as a classification queue rather than asserted to be local typos.
- Reverse-gap inventory: 1,850 public function declaration candidates and 944 public type declaration candidates across `src/` and `crates/`; 145 syntactically matched bare `#[implements(...)]` attributes and 2,958 `#[trace(...)]` test tags were found. These counts are inventory signals, not a one-to-one ownership map. Whole-tree trace-by-trace classification was not completed, so the confirmed untraced behavior count is 0 and the unclassified count remains undetermined; no whole-tree reverse-gap PASS is claimed. In the changed paths, `protocol_locations::validate_rows` and `body_locations::validate` both implement FR-038-AC-597, which has 14 test binders, and no unowned behavior was found.
- Stub scan: 0 explicit source candidates for `todo!`, `unimplemented!`, `TODO:`, or `not implemented`; 1 tiny source candidate (`checked_package/v2/raw_body.rs`, 2 lines) was inspected and is an inert, undeclared module doc stating that carriers are implemented in `model_members`, not executable behavior. No confirmed source stub. The test tree has 238 test-path candidates and 14 `#[ignore]` annotations; changed-feature assertions were inspected, but the full test tree was not exhaustively adjudicated for hollow assertions, so confirmed test-stub count is 0 and unclassified count remains undetermined. No whole-tree stub PASS is claimed.
- `quoin matrix` and `quire trace --file` exited 0 and emitted pre-existing symbol-id-collision diagnostics in unrelated checked-package files. No diagnostics pointed to the two changed validators.
- Semantic review: skipped; it was not requested.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | The 122 untagged criteria are repository-wide and outside this location-scan PR's two production files; this docs-only follow-up adds no code or test changes that affect them. They remain a repository assurance follow-up, not a blocker for this PR's scoped behavior. |
| FND-002 | deferred | FR-028-AC-4 concerns optional/dev/historical dependency isolation and its ignored build-lane test, unrelated to the location-row scan. No changed file or behavior in this PR can resolve it; it remains a repository assurance follow-up. |
| FND-003 | deferred | The 411 untracked symbol records are repository-wide, and the reported external FSpec ids require owner classification unrelated to the location-row scan. This docs-only follow-up does not alter those traces; they remain a repository assurance follow-up. |

Disposition pass 1 reviewed PR #1186 at `5082ad46448ee1d22e056cd7e6b95f722a98c33c`. Since the originally reviewed code head `5128e8c1dcac06ead600dd063b7ce93ab32d4f99`, only the two review artifacts were added. The matrix findings are deferred as repository-wide issues outside the location-scan diff; they do not block this PR's scoped code change. The code-review artifact has only its no-findings placeholder and is not dispositioned.
