---
id: SR-2026111422264100000002
title: "gap-analysis of STD-661 operation_bodies family candidate join"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1024 at 5c735908; FR-038-AC-541, crates/filament-contract-model/src/checked_package/v2/operation_bodies.rs, tests/conformance_qspec/operation_bodies.rs"
review_set: subset
---

## Summary

Gap analysis scoped to the PR diff. Ticket: STD-661. The analysis traces each branch of the carrier's candidate join to a test that fails if that branch is removed. Plan completion: not assessed. Vectors are from filament-specification origin/main 17d49a98, read by reference.

## Examined scope

- FR-038-AC-541 (examined). "The reader reads FSpec TC-987's positive cases OBC-01 to OBC-07 ... and refuses ... its 21 reader cases OBR-01 to OBR-20 and OBR-22 ... and its dispatch-join case OBD-01 at the dispatch join". Every listed case is asserted at its published oracle (5 tests pass).
- A join through `selected_operation` alone, or one that always uses the anchor: OBC-07 fails, because the FastService and SafeService candidates disagree with the Service row.
- A join that always uses the receiver itself: Plain has no row and would be skipped. The new inheriting-candidate test then fails, because the input is admitted.
- The carrier's `operation_in_family` guard (operation_bodies.rs:248): nothing exercises it. See FND-001.

## Verdict

One medium coverage gap. Removing the Premium tests lost the only test of the carrier's family guard. Everything else in the AC is covered.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Nothing tests the carrier's `operation_in_family` skip any more. The removed test `a_candidate_outside_the_tables_family_is_refused_by_dispatch_validation` used receiver Invoice, which declares its own `settle` and has a `no_body` row. Without the guard, that candidate joins the Invoice row and refuses at `/operation_bodies/1/body`; with the guard, it refuses at `.../candidates/1`. That test told the two apart. Its replacement, OBD-01, uses receiver Stray, which has no `run`. `effective_operation_owner` returns `None` for Stray, so the candidate is skipped even if `operation_in_family` is deleted. Deleting the guard at operation_bodies.rs:248 passes every remaining test. Fix: add a test (or ask FSpec for a vector) whose out-of-family receiver has its own same-named operation and row, and assert the dispatch-join locus. | tests/conformance_qspec/operation_bodies.rs:487-521; crates/filament-contract-model/src/checked_package/v2/operation_bodies.rs:248 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Checked at 1b757c1e. The test `an_out_of_family_receiver_with_its_own_operation_is_left_to_the_dispatch_join` builds on OBC-04: receiver Invoice, which declares its own `settle` and has a `no_body` row, in a table whose selected operation is Order.settle. It asserts the refusal at `/checked_term_data/dispatch_tables/0/candidates/0/selection_key`. Re-proved in a throwaway detached worktree, since removed: with `operation_in_family` short-circuited (`if false && ...`), this test alone fails, with left `/operation_bodies/1/body`, and the other 5 pass. At the PR head all 6 pass. |
