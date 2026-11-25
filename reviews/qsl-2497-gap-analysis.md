---
id: SR-967175026302197768900474181941935084595
title: "QSL-2497 PR 1169 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1169 at 0c5ab9a57279c7c52b83f8ddee6a7569482d32c6; spec/checked_package (FR-038, TC-483, matrix/tests.md), temporal.rs check_hyper_shape, tests/it/checked_package_v2_temporal.rs"
review_set: subset
---

## Summary

Ticket: QSL-2497. Planless gap analysis of PR #1169, limited to the diff. The
computed matrix (`quire matrix`) shows FR-038-AC-630 tagged by three tests
(checked_package_v2_temporal.rs:4153, 4178, 4198), and all three pass. The new
code in `check_hyper_shape` belongs to FR-038-AC-630, and no stub or tautology
stands behind the tags. `quire validate` lists the same five structural
failures on base and head, including matrix/tests.md:92 (TC-491). The added
matrix row is valid. Semantic review was not run separately; the code-review SR
covers intent against code.

## Verdict

**CONDITIONAL**: two low spec-traceability findings in FR-038, both already on
main.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The second FR-038-AC-531 row (Group reference to a non-fairness node, FSpec FR-402 step 4 check 1) cites "FSpec FR-402-AC-10". FSpec #714 added FR-402-AC-10 as the repeated-variable criterion that FR-038-AC-630 now consumes, so two IR criteria claim one FSpec criterion for different behaviour. No FSpec AC covers step 4 check 1, so the citation should name the step only. This is a one-cell fix in the FR-038 table this PR already edits. Fix it in this PR. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:7157 |
| FND-002 | low | The id FR-038-AC-531 is declared twice in FR-038, once for the TC-474 implementation_guarantee admission (line 7087) and once for the TC-346 non-fairness Group reference (line 7157). Tests at checked_package_v2_guarantee_composition.rs:442 and 1218 and at checked_package_v2_temporal.rs:4694 bind to one id, so the matrix merges the coverage of two criteria. The duplicate predates this PR. Renumber the second row and retag temporal.rs:4694, here together with FND-001 or in a follow-up ticket. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:7087, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:7157 |

## Coverage

- FR-038-AC-630 (examined): tagged by 3 tests, all pass (35/35 in the focused tc_483_/tc_346_ run).
- FR-038-AC-561 (examined): rewritten repeated-variable sentence agrees with AC-630 and FSpec FR-322; still tagged.
- TC-483 steps 12, 15, 16 (examined): match the tests.
- matrix/tests.md FR-038-AC-630 row (examined).
- FR-038-AC-531, both rows (examined).
- Reverse gap: the new code (`text_literal`, the variable repeat check) belongs to FR-038-AC-630. No unowned behaviour.
- Semantic review: covered by the code-review SR, not run as a separate pass.
- Plan completion: not assessed
