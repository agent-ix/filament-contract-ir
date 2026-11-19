---
id: SR-17940486709564847072
title: "Gap analysis — filament-contract-ir QSL-2333/QSL-2319 batch"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; spec/, crates/filament-contract-model/src/checked_package/v2/, tests/it/checked_package_v2_catalogue_differential.rs, tests/it/checked_package_v2_typed_terms.rs"
review_set: subset
---

## Summary

Ran the repository matrix for this planless, repository-driven audit and inspected the changed behavior against the current FR-772 and FR-322 criteria. The repository matrix has inherited untagged criteria, and the changed FR-772 quantifier-domain test is not tagged; plan completion was not assessed.

## Verdict

**FAIL** — the matrix reports 110 untagged criteria and the new FR-772 consumer-exclusion test has no trace tag.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The repository matrix reports 110 of 1,237 local criteria as untagged; this is a repository-wide baseline gap and is not limited to this PR's changed criteria. | spec/, tests/, crates/ |
| FND-002 | high | The changed test `fr772_option_slots_is_only_a_quantifier_domain` lacks a trace attribute. It exercises the FR-772-AC-2 restriction, so the check does not bind as evidence for that requirement. | crates/filament-contract-model/src/checked_package/v2/operations.rs:13137 |

## Coverage

- Reconciliation: `quoin matrix --repo . --json` (quoin 0.36.7, quire 0.52.0 / engine 0.62.2).
- Plan completion: not assessed
- Criteria: tagged 1,096 · untagged 110 · tagged-by-ignored-test 1 · method-without-symbol 26 · retired 4
- Evidence: no run evidence for 1,237 criteria
- Untraced behaviors / stubs: no unowned public behavior or placeholder found in the changed source; the behavior is specified by FR-772 and FR-322.
- Semantic review: skipped

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | Round 1: this was an existing repository-wide matrix gap, not introduced by PR #1059; current rebased tree then reported 121/1,248 untagged criteria. |
| FND-002 | fixed | Round 1: the consumer-exclusion test gained `#[trace("TC-1862", "FR-772-AC-2")]`. |

## Disposition pass 2 recheck

- On head `9a733e625456679f6d881859b1599b80dbdf9618`, `quoin matrix` reports 122/1,257 untagged criteria. The PR changes no spec criteria; the existing repository-wide baseline remains outside its scope.
- FND-002 remains fixed: `fr772_option_slots_is_only_a_quantifier_domain` retains `#[trace("TC-1862", "FR-772-AC-2")]`.

## Disposition pass 3 recheck

- Head/base: `265ce2bc1ecfe36e7f62f03251dc302100b84d2b` / `c6dc5c3e5abf3d245a91ac79546d8bafda1e2e56`.
- FND-001 remains deferred as the existing repository-wide baseline; the current matrix reports 122 of 1,257 criteria untagged. No spec changes are in the PR.
- FND-002 remains fixed. The broad catalog and typed-terms census no longer claim FR-772 AC coverage; dedicated FR-772 tests remain tagged and pass.
- `quire coverage --scope . --json` failed on pre-existing `symbol-id-collision` reports in files outside this PR. No new gap finding was identified in the changed diff.
