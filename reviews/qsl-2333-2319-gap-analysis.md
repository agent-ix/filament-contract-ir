---
id: SR-17940486709564847072
title: "Gap analysis — filament-contract-ir QSL-2333/QSL-2319 batch"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; spec/, crates/filament-contract-model/src/checked_package/v2/"
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
