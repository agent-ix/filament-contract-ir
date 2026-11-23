---
id: SR-17192
title: "QSL-2985 PR 1148 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir#1148; FR-038-AC-394; tests/conformance_fspec/probabilistic_claims.rs"
review_set: subset
---

# SR-17192: QSL-2985 PR 1148 gap analysis

## Summary

Ticket: QSL-2985. Scoped to the one changed AC and its test. Plan completion:
not assessed. FR-038-AC-394 is traced by `claim_row_coverage_and_exact_work_boundary`
(`#[trace("TC-233", "FR-038-AC-391", "FR-038-AC-394")]`) and by
`published_probabilistic_claim_vectors_join_and_refuse`. Every clause of the
amended AC is asserted:

- exact admit: the binary search, with an asserted admit at the bounded limit;
- one below lands at `/semantic_graph/nodes/24/owner`;
- the largest limit below the owner visits lands at `/checked_term_data/probabilistic_claims/1`;
- neither refusal has a partial view: the `Incomplete` variant carries no package.

The test also asserts row 0 one lower, which goes beyond the AC. Ordering is
consistent with FR-038-AC-144 and AC-30, which pin the selected-owner charge as
the final charge. The PR adds no production code, so no reverse gaps and no
stubs. The FSpec `checked-term-data-vectors.json` `work_limit` block (30 units;
one-less refusal at the final probabilistic claim row) is a sidecar-stage
equation over the full vector set. IR did not consume it before this PR and
does not consume it now. The new test still shows that, within the sidecar,
the final charge is the last claim row. This was a pre-existing scope choice,
not a gap this PR introduced.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS. No uncovered criterion in scope.
