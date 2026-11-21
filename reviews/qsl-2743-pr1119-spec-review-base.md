---
id: SR-967175026302197768900474181941935084532
title: "QSL-2743 PR 1119 spec-review base checklist of the new and changed IR criteria"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; QSL-2743; PR 1119 head 5e7f682c7321a400fe693d649cd555095af19b5a; FR-352-AC-38..44; FR-358-AC-17; FR-358-AC-19; FR-358 statement; context FSpec FR-830 (read-only checkout FSPEC-QSL2743-model-unions)"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-352", type: references }
  - { target: "ix://agent-ix/filament-contract-ir/FR-358", type: references }
---

## Summary

Base checklist review of the IR spec rows PR 1119 adds or changes. Ticket: QSL-2743. `quire validate --scope .` on both changed specs returned rc=0. Its only warnings sit on lines this PR does not touch (FR-352 line 631, FR-358 line 231). No hash, digest or pin was introduced. No AssuranceProfile exists under spec/.

FR-358-AC-17 governing prose. FR-358's own statement has no sentence that names a locus for a reached-closure charge. Its `NativeJoinLocusSource` paragraphs list `MonitorAuthority`, `AdapterRow`, `AuthorityRow`, `PreparationOutput`, `PreparationSelection` and `PackageRead`, and never mention `GraphNode` or `ModelUnionRow`. The only nearby prose is FR-358 lines 116-117: "Charge order follows the admitted and input row order used by the join; an unrelated graph node is not visited solely to consume budget." FR-358 prose therefore supports neither the old ModelUnionRow locus nor the new GraphNode locus.

The PR body attributes its sources honestly. It quotes FSpec FR-830 (lines 290-293: "The denial's compact locus names the channel, the sidecar row index and the form of the charge, the first unpaid coordinate in the native policy's compact form; which native locus source carries it is the budget owner's choice and is not preselected here."). It also quotes the FR-358 lines above, and says FR-830, not FR-358, is the permission.

The code (model_union.rs:193-207, `at_closure_node`) and budget_case_b02 (model_union_witness.rs:968-1053, origins Pair x2 and Shape x2 at `GraphNode`, then a `ModelTupleRow` row-join denial) match the new GraphNode wording.

## Verdict

**CONDITIONAL — MERGEABLE=FALSE.** FR-352-AC-38..44 are clean against the test and code at the record-field and payload level (see SR-967175026302197768900474181941935084531 for the low adverse-strength findings). FR-358-AC-17's locus now matches the code, but two medium defects remain in the FND-008 remainder.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-358-AC-17 still states the edge set wrongly. It says the closure charges one edge per typed member of every reached union, but the reader follows and charges an edge for every variant case, tuple position, record field and object field of each visited declaration (model_union.rs:418-485). Its own bound test, budget_case_b02, asserts two edges whose origin is the tuple `Pair`. The PR rewrote this criterion to state what the reader does but kept a scope clause that neither the code nor the B02 origins it now cites match. | spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md:258; crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:418-485; tests/conformance_fspec/model_union_witness.rs:993-1015 |
| FND-002 | medium | The budget owner's GraphNode choice has no FR-358 prose. FR-830 leaves the locus source to the budget owner, which is FR-358 for `NativeJoinBudget`. But FR-358's statement never records that choice or why it is made. The same FR-830 sentence the PR quotes says the denial's compact locus names the channel, the sidecar row index and the form. The changed FR-358-AC-19 exempts closure-edge charges from that coordinate, which narrows the quoted FSpec obligation with only criterion text behind it. FR-358 needs a sentence beside its `NativeJoinLocusSource` paragraphs: a reached-closure node or edge charge has no sidecar row and is located at `GraphNode` of its origin node, with form `model_union_closure_node` or `model_union_closure_edge`, as the budget owner's choice that FR-830 leaves open. If that departure from FR-830's channel and sidecar-index coordinate is not accepted, it goes to the FSpec owner. | spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md:63-86, 260 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | The new FR-358 closure paragraph lists the closure's charges inaccurately. It says each closure node pays work and storage of form `model_union_closure_node`. The reader charges node work at `model_union_closure_node` but node storage at `model_union_closure` (model_union.rs:346-355). It also charges the start node's storage at `model_union_closure` (:398-399) and each member-type resolution at `model_union_closure_type` (:424, 445). The SHALL locus (GraphNode of the node expanded or whose edge is examined) does hold for all of these charges, but the paragraph names only two forms. | spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md:213-226; crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:346-355, 398-399, 424, 445 |
| FND-004 | low | The new FR-358 paragraph misstates FR-830. It says "FSpec FR-830 requires the compact locus of a sidecar-row charge to name the channel, the original row index and the form". FR-830 (lines 290-293) says, without that qualifier, "The denial's compact locus names the channel, the sidecar row index and the form of the charge". FR-830 states no closure exception; the exception is FR-358's own owner choice. The paragraph should state FR-830's sentence as written and say that a closure charge has no sidecar row to name, rather than attribute the limitation to FR-830. The PR body's own FR-830 quote is accurate. | spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md:219-221 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Disposition pass 1, head c9b9a2415. FR-358-AC-17 (line 273) now states the edge set the reader walks: each variant case payload, tuple position and record or object field with a typed member. This matches model_union.rs:418-485 and B02's Pair x2 and Shape x2 origins. The code and B02 are unchanged. |
| FND-002 | fixed | Disposition pass 1, head c9b9a2415. FR-358 Behavior (lines 213-226) now records the budget owner's choice normatively: closure charges are located at NativeJoinLocusSource::GraphNode of the node expanded or whose edge is examined, because a closure charge names no sidecar row. It cites FR-830's "budget owner's choice" permission and keeps common-pass and row-join loci. The PR body quotes this FR-358 text and, separately, FR-830. Residual wording defects are new findings FND-003 and FND-004 (low). |

## Disposition round 1 verdict

**CONDITIONAL.** FND-001 and FND-002 (medium) are fixed. Two new low wording findings (FND-003 and FND-004) are in the new FR-358 paragraph. No medium or high finding remains open.
