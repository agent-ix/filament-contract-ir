---
id: SR-2026112002305500000001
title: "spec-review of QSL-2645 FR-369 identity-row join entry point, meter and FR-367 layer types"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1085 at bd142116; spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md, spec/checked_package/functional/FR-367-join-source-object-rows-and-field-reads.md, spec/checked_package/matrix/TC-484-probabilistic-nodes-on-source-declared-operations.md"
review_set: subset
---

## Summary

Spec-review of the QSL-2645 arch ruling's encoding. Ticket: QSL-2645. Checked against the ruling comment on QSL-2645, FR-374 (read never receives identity rows; locus shape), FR-375 (`InvalidValue`), FR-358 and FR-352 (`NativeJoinBudget`, first unpaid kind/amount/locus) and FSL FR-531 `collections` values at filament-spec-language origin/main. `quire validate --scope .` exits 0 on all three files (one pre-existing grammar warning at FR-369 line 336).

## Examined scope

- FR-369 identity-row join paragraph (examined): entry point `join_probabilistic(&self, &mut NativeJoinBudget) -> Result<(), SelectedSourceObjectError>`, run after `from_stage_rows`, not by `CheckedPackageV2::read`; `InvalidValue` with `locus` `Some`. Matches ruling A and FR-374/FR-375.
- FR-369 meter paragraph (examined): FR-358 work on the shared budget, no second budget API, `Incomplete` at the first uncharged member, first unpaid kind/amount/locus, no partial view, `Ok(())` meaning the four typed reads are joined. Matches ruling A.
- FR-369 Planned paragraph (examined): consistent.
- FR-369-AC-26 (examined): all seven refusals, the clock-parameter variant and the budget stop retained; the budget stop now names `Incomplete` and can fail.
- FR-369-AC-25 (examined): unchanged.
- FR-367 unmetered sentence and `SelectedClockCollectionLayer` types (examined): match ruling B and FSL FR-531 (`kind` set, decimal `minimum`, decimal-or-`unbounded` `maximum`).
- TC-484 step 6 (examined): a join that skips reward parameter charges fits the one-short budget and returns `Ok(())`, so the step fails it.
- FR-369 strict-read meter paragraph (line 209-215) and FR-369-AC-20/AC-22 (examined): substance is strict-read charging, correctly left alone.

## Verdict

Mergeable. The text states the ruling and nothing beyond it, and is consistent with FR-374, FR-375, FR-358, FR-352 and FSL FR-531. No behavior or AC was dropped; no id collisions. Two low wording findings: "join" in strict-read meter text now collides with the named `join_probabilistic`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-369-AC-20 and AC-22 say "The join charges" for strict-read charges; with `join_probabilistic` also charging per support value, "join" is ambiguous. Replace "The join charges" with "The strict read charges" in both rows. | spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md:329,331 |
| FND-002 | low | Strict-read meter paragraph says "a bound too small for the join stops the read as incomplete"; replace "for the join" with "for the read". | spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md:214 |
