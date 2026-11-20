---
id: SR-17940486709564847048103
title: "spec-review of QSL-2645 FR-369 Planned note, TC-484, matrix rows and FR-019 row"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1086 at 7b30d82da (git diff origin/main...HEAD); spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md, spec/checked_package/matrix/TC-484-probabilistic-nodes-on-source-declared-operations.md, spec/checked_package/matrix/tests.md, spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

This is a spec review of the PR's four spec edits. Ticket: QSL-2645. They were checked against the arch ruling comment on QSL-2645 and the merged #1085 text (FR-369, FR-367, FR-374, FR-375, FR-358).

The edits are:

- a replacement for FR-369's "Identity-row join" Planned paragraph, which becomes "Source-declared entry types";
- the TC-484 description naming the method and fixture directory;
- both FR-369 TC-484 matrix rows set to ✅ Implemented, and the TC-484 summary row likewise;
- the FR-019 probabilistic typed-read row naming `join_probabilistic`.

`quire validate` exits 0 on this SR. The spec files are unchanged from #1085 apart from these four edits.

## Verdict

**Changes needed (medium).** The TC-484 description and the FR-019 method wording are accurate. Three problems remain:

- The new Planned paragraph frames an ownerless "spec question for QSL-2645's owner", and it lumps a join-specific input gap (source enumerations) together with gaps the strict read shares (bounded and int/decimal carried quantities).
- The matrix row claims full implementation, refusals "at the strict read's member", and the parts of AC-26 that are not built.
- FR-369-AC-27 and FR-369-AC-28 still say "the join half is Planned, QSL-2645".

The FR-019 breaking-change omission is recorded as FND-005 in the code-review SR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new FR-369 Planned paragraph ends "whether the join SHALL admit those is a spec question for QSL-2645's owner": an open question with no ticket and an owner that this PR closes. It also lists the source enumeration, which is a join-only gap because `join_probabilistic` lacks the FR-352 source enum rows, beside the bounded and int/decimal carried quantities, which the strict read's selected-parameter rule also leaves without a literal kind (`member_literal_kind`). Split it: file an arch ticket for the enumeration input, and state the quantity forms as shared with FR-369's strict-read rule, with their own owner | spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md:283-292 |
| FND-002 | medium | The FR-369-AC-25/26 matrix row is ✅ Implemented and says "refusals `InvalidValue` at the strict read's member". The operation and parameter refusals are one level above the strict read's member (code-review SR FND-001), and TC-484 step 5's body variants are not built (gap-analysis SR FND-001). The TC-484 summary row is also ✅. Keep the rows 🚧 Partial, naming what is open, until both land | spec/checked_package/matrix/tests.md:20; spec/checked_package/matrix/tests.md:90 |
| FND-003 | low | FR-369-AC-27 and FR-369-AC-28 still read "(the join half is Planned, QSL-2645)" although this PR implements the join, and AC-27 still says "the operation, parameter, support and coverage are unjoined". Reword them as the strict-read halves whose join half is FR-369-AC-25/26 | spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md:335-336 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | high | FR-369, as amended in this PR, says the join charges "the exact numbers it parses to its read storage as the strict read charges them". FR-358 says the read-storage counter is "used only by the strict V2 checked-package read of FR-038 ... and no other operation charges read storage", so the two requirements contradict each other. The consequence is that a caller following FR-358 (`NativeJoinBudget::new(w, s)`, whose read-storage limit is 0) is stopped as `Incomplete` on a parse that needs a limb block, and the read-storage denial is invisible to `first_unpaid()`. Either charge the parse heap to logical storage as copied owned payload, one unit per byte, which conforms to FR-358 unchanged, or have arch amend FR-358 to admit `join_probabilistic` as a second read-storage user | spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md:198-201; spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md:141-147 |

## Dispositions

Round 1, at head 0c9d47aea.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The Planned paragraph is now "Source enumeration entries (QSL-2655)". It names the missing input (the identity rows carry no enumeration's cases), states that the bounded and int/decimal carried quantities have no literal kind "there either, as in the strict read", and gives "Owner: QSL-2655" |
| FND-002 | fixed | Both rows are now 🚧 Partial: the FR-369-AC-25/26 row names the refusal "at the member the strict read names for the same edit" and the QSL-2655 gap, and the TC-484 row reads "every step is implemented; entries typed by a source enumeration are open on QSL-2655" |
| FND-003 | fixed | AC-27 and AC-28 now read "(the join half is the identity-row join of QSL-2645)". The remaining "unjoined" in AC-27 describes the strict read without identity rows, which is accurate |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | FR-369-AC-26 now says that "one unit of logical storage short of the failure pointer, the lists it keeps or the exact numbers it parses, stops the join with `Incomplete` at the first uncharged member". Behavior (FR-369:201) and TC-484 step 6 say that a budget that cannot pay for the failure pointer stops "with `Incomplete` and no locus", and the test asserts `error.locus.is_none()`. Move "the failure pointer" out of that list, and say it stops with no locus | spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md:347 |

Round 2, at head 63e40db63. Focused run: 86 passed, 0 failed.

| FND | outcome | reason |
| --- | --- | --- |
| FND-004 | fixed | FR-369 now reads "The join charges no read storage, which only the strict read of FR-038 uses". Exact parses are FR-358 logical storage units (one for the number and one per limb), which agrees with FR-358:141-147. The tests assert `read_storage_consumed() == 0` and `first_unpaid_read_storage().is_none()`, and they run with `NativeJoinBudget::new(w, s)` (read-storage limit 0) |

## New findings (disposition pass 3)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | Both TC-484 matrix rows are still 🚧 Partial, but neither names anything open. The FR-369-AC-25/26 row now describes the enumeration check as implemented, and the TC-484 row reads "🚧 Partial (IR-1576, QSL-2645, QSL-2655): every step is implemented". Either set them ✅ Implemented, or name what remains | spec/checked_package/matrix/tests.md:20,90 |

Round 3, at head bb4be1211 (diff e4f345785..HEAD: ae47b5702, 87d55115e, and main merges that touch only other lanes and Cargo pins). Focused run: 88 passed, 0 failed.

| FND | outcome | reason |
| --- | --- | --- |
| FND-005 | fixed | FR-369-AC-26 now reads "... the failure pointer, which is paid before the join starts, stops it with `Incomplete` and no locus", which agrees with Behavior and TC-484 step 6 |
