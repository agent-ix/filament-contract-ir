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
