---
id: SR-17940486709564847048097
title: "spec-review of IR-1576 FR-369/TC-484 matrix rows and the identity-row join contract"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1082 at fafbdc18 (merge base origin/main); spec/checked_package/matrix/tests.md (FR-369 AC-25/26 row, TC-484 and TC-485 rows); spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md and FR-367-join-source-object-rows-and-field-reads.md as context"
review_set: subset
---

## Summary

Spec review of the one spec change in PR #1082: the FR-369 AC-25/AC-26 row of `spec/checked_package/matrix/tests.md`, moved from Planned to "Partly implemented". Ticket: IR-1576 (also covering QSL-2465's code half; QSL-2465 is now Duplicate of IR-1576).

The coder states that the identity-row join half of FR-369 was not implemented because the specs state no entry point for it. That claim was checked against FR-367 and FR-369. It holds for the join's member-located, metered refusals: FR-369 asks for a join that runs after the strict read has admitted the package and after `SelectedSourceObjects` has been admitted, that refuses `invalid_package`/`invalid-value` at the probabilistic node's member, and that is charged so that a small bound "stops the read as incomplete". FR-367 defines `SelectedSourceObjects::from_stage_rows` over an already-admitted `CheckedPackageV2`, with no meter, and refusing only with `SelectedSourceObjectErrorCode` at the `Package` locus. No stage in the specs can give the FR-369 result shape. It does not hold for FR-367-AC-8 (the `parameters` member and its `UnknownIdentityWitness` refusals). That part is fully specified, and the arch ruling on QSL-2465 placed it in this PR's scope.

## Examined scope

- tests.md FR-369 AC-25/AC-26 row (examined). It is honest about what is missing, but it names a closed owner and the TC row disagrees with it (FND-001, FND-002).
- tests.md TC-484 row (examined). It still reads "Planned" (FND-001).
- tests.md FR-367-AC-8 / TC-485 row (examined). It is Planned under QSL-2465, which is Duplicate (FND-002).
- FR-369 "After the strict read admits a package ... the reader SHALL join" and its charge paragraph (examined). It has no stage that can deliver it (FND-003).
- FR-369 delay item "for which no `timed_model` root whose `model` is a source-declared node of the same `SourceOwner` identity has a `dense` source" (examined). The code's "any dense root of that owner, scanning past non-dense ones" reading is the correct one. The wording does not say which unit applies when two `dense` roots of one owner have different units. FSpec FR-415's "one source time member times every source object type of the unit" excludes that case, so this is not a defect.
- FR-367 `SelectedSourceObjects` contract, lines 55-92 and FR-367-AC-8 (context_only).

## Verdict

The row's prose is accurate about the delivered and missing halves. Two bookkeeping defects make the deferred half look owned when it is not. A real spec gap is upstream of the PR: FR-369's identity-row join has no entry point, result type or meter in FR-367's API. That gap justifies deferring the join's member-located refusals. It does not justify deferring FR-367-AC-8, which is gap-analysis FND-001.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The FR-369 AC-25/26 row now says "Partly implemented (IR-1576, QSL-2465)", but the TC-484 row in the TC table still reads "🚧 Planned (QSL-2465)". The two rows for the same TC disagree | spec/checked_package/matrix/tests.md:19,87 |
| FND-002 | medium | The remainder rows name QSL-2465 as the tracker. QSL-2465 is Duplicate (closed) of IR-1576, so once this PR merges and IR-1576 closes, nothing owns the identity-row join, the golden pair or FR-367-AC-8 (TC-485). The readiness sweep on QSL-2465 already noted that FR-367-AC-8 has no other ticket. Name a live owning ticket in both rows | spec/checked_package/matrix/tests.md:19-20 |
| FND-003 | medium | Upstream contradiction (send to arch). FR-369 requires the identity-row join to refuse `invalid_package`/`invalid-value` at the probabilistic member and to be charged so that a small bound "stops the read as incomplete". It also runs only after the strict read admitted the package and `SelectedSourceObjects` was admitted. FR-367's `SelectedSourceObjects::from_stage_rows` takes an admitted package, charges nothing and refuses only at the `Package` locus. No stated stage can produce FR-369's result, so FR-369-AC-25/26's join half and TC-484 steps 3, 5 and 6 cannot be built as written. State the entry point: either the join is part of a read that takes the identity rows as input, or it is a metered, member-located refusal of `from_stage_rows` | spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md:139-178 |

## Dispositions

Round 1, at head 44ce41ca.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The TC-484 row now reads "FR-369-AC-25, FR-369-AC-26, FR-369-AC-27, FR-369-AC-28 \| 🚧 Partly implemented (IR-1576): strict-read steps 1 (without the join), 2 and 4; steps 3, 5, 6 and the golden pair are Planned (QSL-2645)", which agrees with the FR-369 rows |
| FND-002 | fixed | The deferred rows name QSL-2645 ("contract-ir: source-declared identity-row join ...", open, Backlog), which owns the join, the golden pair, the Clock variant and the budget. FR-367-AC-8/TC-485 moved to Implemented (IR-1576) |
| FND-003 | deferred | This is an upstream spec contradiction (the join's entry point, locus and meter versus FR-367's unmetered `Package`-locus `from_stage_rows`). QSL-2645 records it as blocked on an arch ruling, and FR-369 "Planned" now states "the identity-row join above has no entry point, result type or meter in the FR-367 API yet". Out of this PR's scope |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | Upstream ambiguity, not a PR defect. FR-367 "Member entries" names `collections: Vec<SelectedClockCollectionLayer { kind, minimum, maximum }>` without giving the field types. The PR chose `Box<str>` for all three, with the bounds as canonical decimal text or `unbounded`, per its reading of FSpec FR-789 "Rendering". A producer could read them as an enum and integers instead. Arch should pin the types in FR-367 (QSL-2645 or the FR-374/QSL-2347 owner) before FSL builds rows against this public type | spec/checked_package/functional/FR-367-join-source-object-rows-and-field-reads.md:104-115; crates/filament-contract-model/src/checked_package/v2/source_object.rs:55-63 |
