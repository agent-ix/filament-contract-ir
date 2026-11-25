---
id: SR-5761728
title: "gap-analysis of IR-1615 event-typed parameter admission and operand guard"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1174 at eff94c5d093c6908f8dd5a51984494940398b825; FR-038-AC-632, FR-038-AC-633, crates/filament-contract-model/src/checked_package/v2/{structural.rs,operations.rs}, tests/it/{checked_package_v2_model_fields.rs,checked_package_v2_temporal.rs}"
review_set: subset
---

## Summary

Gap analysis scoped to the PR diff. Ticket: IR-1615. Plan completion: not assessed. The matrix was computed with `quire matrix --scope . --format json` on this head: FR-038-AC-632 is `tagged` by `tc_048_an_event_typed_parameter_is_admitted_only_over_a_selected_event` and `tc_048_a_temporal_clause_admits_an_event_typed_over_parameter`; FR-038-AC-633 is `tagged` by `tc_048_an_operation_over_an_event_typed_parameter_is_ineligible`. Reverse gap: both production hunks (the `parameter_defect` arm and `check_event_parameter_operands`) are owned by FR-038's amended parameter paragraph. No stubs, no `todo!`, no ignored tests. Tests were not run (cargo blocked for reviewers); pass/fail is unverified.

## Examined scope

- FR-038-AC-632 (examined): the model_fields test asserts `Admitted` for an `OrderPlaced`-typed parameter and literal `missing_declaration`/`missing-selection` at `/semantic_graph/nodes/{n}/node_id` for the event node re-owned by `Money`, `Order` and an absent node; the temporal test asserts `Admitted` with the fixture's `over` parameter retyped to an event type. Both fail on main (parameter refused at `semantic_type`).
- FR-038-AC-633 (examined): four cases (`structural.eq`, `boolean.not`, `record.project`, `collection.contains`), each asserting literal `ill_typed`, `operator-ineligible` and the argument pointer, including argument 1 for `contains`. Independent literal oracles; would fail without the guard.
- Binding accuracy (examined): all three trace tags name TC-048 and the AC the test proves; no test claims more than it asserts.

## Verdict

Coverage of both new ACs is real and the tests are independent; one low gap. The `over` exception is proved only positively: no test shows that a temporal clause's other arguments, or a non-temporal operation whose operand 0 is a `reference`, still refuse an event-typed parameter, so widening the exception would pass every test.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The exception-width half of FR-038-AC-633 ("the temporal clause's `over` operand excepted") has no negative test: nothing fails if the skip is widened to every argument of a temporal clause or to any operation whose operand 0 is `reference` | tests/it/checked_package_v2_model_fields.rs:10348-10428, tests/it/checked_package_v2_temporal.rs:6745-6795 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 8397db689 adds `tc_048_a_temporal_clause_operand_after_over_may_not_read_an_event_typed_parameter` (bare event-typed reference as a temporal clause's argument 3 refuses at `/body/arguments/3`) and a `filament.op.model.deref` case (operand 0 of class `reference`, not temporal, refuses at argument 0), so widening the skip in either direction now fails a test. |
