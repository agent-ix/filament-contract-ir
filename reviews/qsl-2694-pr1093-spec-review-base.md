---
id: SR-2026112009294800000003
title: "spec-review/base of QSL-2694 PR #1093 stage-4 edge existence spec edits"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1093 at 24ab77d9; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (FR-038-AC-582, FR-038-AC-272, FR-038-AC-275, FR-038-AC-280, population prose), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md, spec/model/functional/FR-019-rust-library-interface.md; checked against filament-specification origin/main 47f1f73b FR-322 'One total order of the join stages' (FR-322-AC-275), FR-322-AC-73, FR-322-AC-299"
review_set: subset
---

## Summary

Spec review of the IR spec edits in PR #1093 (tickets QSL-2694, QSL-2666, QSL-2644), checked against FSpec FR-322 at origin/main 47f1f73b. Ticket: QSL-2694. `quire validate --scope .` passes on all four changed files with 0 errors. It reports 4 pre-existing `ac:non-canonical-shape` grammar warnings, none of them on the edited rows.

## Examined scope

- FR-038-AC-582 (examined). The rewritten row orders stage 4 before stage 5 and states the tags, the edge order, the code and locus, the exemption and the charge. The tags, the order and "invalid_semantic_graph with no cause" agree with FSpec FR-322 (4). FSpec gives no code or locus for stage 4, and the row says so. The exemption of `dimension`/`unit`/`dimension_equivalence` dependencies has no FSpec basis (FND-002).
- FR-038-AC-272, FR-038-AC-275, FR-038-AC-280 and the FR-038 population prose ("An absent node is `ill_typed` and not `missing_declaration`") (examined). The PR does not edit them, but stage 4 now contradicts them (FND-001).
- TC-048 stage-4 paragraph and the tests.md FR-038-AC-582 row (examined). Both agree with the code. The TC-048 claim that a dangling `dependencies` entry is "already refused by the stage 1 `dependencies` comparison, so stage 4 reads it only as a backstop" was disproved by a mutant run (FND-003).
- FR-019 public-items row for the FR-376 requirement records (examined). It is a stale-catalogue fix that `tests/it/public_interface.rs` enforces. The row is correct.
- FSpec FR-322-AC-275 / FR-322-AC-73 `reaches-population-names-absent-node` (context_only). FSpec contradicts itself here (FND-004).

## Verdict

Changes required in the IR spec text. Three FR-038 ACs and the population prose remain marked Implemented with outcomes that the code no longer produces. The exemption needs either an FSpec ruling or an explicit deviation note. FSpec has its own internal conflict, which should be decided upstream.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-038-AC-272, AC-275 and AC-280 (Implemented) and the population prose still say that a listed population naming no node refuses ill_typed/operator-ineligible at population (or, for AC-280, at the earlier operand). Stage 4 now refuses invalid_semantic_graph at the dependencies edge. The tests tagged to these ACs were changed, but the ACs were not | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4249-4255, 6621, 6624, 6629 |
| FND-002 | medium | The FR-038-AC-582 exemption (dimension/unit/dimension_equivalence dependencies are left to the nominal join) is not in FSpec FR-322 stage 4, which covers every scalar_type node. It is presented as conformant reader behaviour. It is not marked as a deviation and has no FSpec ticket | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6360 |
| FND-003 | medium | The TC-048 note says stage 4 reads dangling dependencies "only as a backstop" behind the stage 1 comparison. This is false. With the dependencies loop disabled, 4 `it` tests fail: a listed absent population, and a record/set type whose dependency names no node now reach the operation stage and refuse ill_typed. Stage 4 is the stage that refuses them | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:3045-3049 |
| FND-004 | medium | Spec question for FSpec: FR-322-AC-275 stage 4 refuses a listed population that names no node before the operation stage, but FR-322-AC-73 / TC-233 vector reaches-population-names-absent-node expects ill_typed/operator-ineligible. FSpec must decide whether stage 4 exempts member-held population ids or the vector changes | filament-specification spec/objects/interfaces/FR-322-checked-package-artifact.md:689-715; spec/test-cases/TC-233-published-checked-package-and-scalar-contract-vectors.md:344-350 |

## Dispositions

Round 1, head 1f40c78e.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Verified in the fix commit. |
| FND-002 | deferred | The exemption is now recorded as an explicit DEVIATION in FR-038-AC-582 and TC-048, pending an arch/FSpec decision (draco:QSL:arch); the code is unchanged until that ruling, which is outside this PR. |
| FND-003 | fixed | Verified in the fix commit. |
| FND-004 | deferred | Deviation recorded in the FR-038 population prose and FR-038-AC-272 (follows FR-322-AC-275; vector replay fails until FSpec decides); the FSpec decision is pending with draco:QSL:arch and is outside this PR. |
