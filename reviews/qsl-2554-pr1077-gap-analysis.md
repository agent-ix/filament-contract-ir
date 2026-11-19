---
id: SR-17940486709564847048093
title: "gap-analysis of QSL-2554 probabilistic claim node read and refusals"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1077 at e0e47c8a (merge base 383857b6); FR-377-AC-1..AC-4, TC-489 against crates/filament-contract-model/src/checked_package/v2/{probabilistic_join/claims.rs,probabilistic_claim_read.rs,temporal.rs,flat_wire.rs}, tests/it/checked_package_v2_probabilistic_claim.rs, tests/conformance_fspec/probabilistic_claim_timed.rs, spec/checked_package/matrix/tests.md"
review_set: subset
---

## Summary

Gap analysis for PR #1077. Ticket: QSL-2554. This analysis is planless, so plan completion was not assessed. Upstream: FSpec origin/main FR-640-AC-8 to AC-11, FR-370-AC-22 and FR-322-AC-474.

`quire matrix` at the head shows FR-377-AC-1 to AC-4 as tagged:

- AC-1: `the_public_read_gives_each_admitted_claim`, `the_read_survives_a_write_and_a_graph_reorder` and `each_identity_change_gives_that_claim_another_entry`.
- AC-2: `each_defect_refuses_at_its_member`, `the_read_sets_are_exact_and_the_subject_is_a_model_root` and `typed_literals_untimed_time_forms_and_path_rewrites`.
- AC-3: `the_claim_checks_charge_each_binding_in_body_order`.
- AC-4: `a_formula_reference_stands_only_at_the_event_formula`, `a_fairness_node_at_the_event_formula_refuses_at_the_claim`, and the two `probabilistic_claim_timed` tests.

Every binding points at the right AC. The matrix row moves from Planned to Partial (QSL-2543, QSL-2554).

Failing set against clean origin/main 2ec296a1, each in its own worktree and target:

- Lib: the same 18 `reached_model_read` reds on both.
- `it`: the same 14 await reds on both. `public_interface` is green on both. It is red only at the unmerged PR head, whose base predates #1076's `CheckedTerminalDuplicate` removal.
- `conformance_fspec`: the same 8 reds on main and on the merged PR. They are:
  - `certificate_member::model_certificate_members_match_fspec_schema`
  - `monitor_plan::monitor_plan_refuses_a_clock_read_of_a_hybrid_subject_timed_clause`
  - `operation_effects::tc_977_published_operation_effect_vectors`
  - `protocol_mutation_manifest::current_fspec_appended_await_layout_remains_admitted`
  - `probabilistic_claims::claim_row_coverage_and_exact_work_boundary`
  - `temporal_aggregate::duration_bounds_match_every_occurrence_clock_unit`
  - `temporal_aggregate::published_temporal_aggregate_arguments_match_strict_reader`
  - `timed_model::timed_clause_published_vectors_match_reader`

  That is 8, not the brief's "about 7". `claim_row_coverage_and_exact_work_boundary` fails on both at its exact-work locus assertion: the PR reports `/semantic_graph/nodes/24/owner` where the test expects the sidecar row, so the PR does not turn it green.

## Examined scope

- FR-377-AC-1 (examined). Covered: the public read of `P95` (quantile, accumulate, integer `within`, quantity bound at the reward unit, confidence), `Avail` (formula event), `Cost` (expected_reward with a rational bound), `Fair` (every scheduler); the writer round trip; graph reorder; and seven identity changes. Not covered (FND-003): the read of every other variant, and part of FR-640-AC-10's identity list.
- FR-377-AC-2 (examined). Covered: form kind `median`, level/measure swap, a fraction measure under a quantile, direction `below`, a wrong-kind bound, level 1, within −1, indifference 1/20 against 19/20, the θ − ι boundary and its neighbours, indifference 0, confidence without indifference, a formula slot naming a predicate, empty and non-empty reward, a trigger naming a literal, a text `over`, scheduler a reward node, fairness member twice, out of order and naming no operation, the rule-6 sets, subject an object type, an `integer_range`-typed `within`, `elapsed` and `expected_elapsed` over an untimed model, the path rewrite, and the rule-6-before-rule-7 node. Several FR-640-AC-11 mutations that AC-2 names are not tested (FND-001).
- FR-377-AC-3 (examined). The least admitting budget is searched, and the stop one visit short of the reader's count is at `P95`'s `indifference`, with the next visit outside `P95`'s body. The order of the top-level bindings is asserted. FR-376's part is not implemented (QSL-2543 Planned), and the matrix row states this honestly. Charge order inside `form` is not pinned (FND-002).
- FR-377-AC-4 (examined). Covered: the formula reference admitted at the event formula, refused at `measure` `from`, and a fairness node at the event formula refused at the claim node; tick and dense `elapsed`/`expected_elapsed` admitted with quantities at `ms`; `delay` weighting refused over tick; an `integer_range`-typed `within` refused.
- matrix/tests.md FR-377 and TC-489 rows (examined). See FND-003.
- FSpec FR-322-AC-474 (context_only). The ticket lists it as IR-owned. No test exercises two claims named `A` that share a path, or a claim literal spelled `2/4`.

## Verdict

The tests are substantive, and they kill the boundary, placement, time, set and charge mutants the coder listed: 15 of 17. The two survivors are `c12`, a charge-order mutant, and `c17`, an equivalent mutant.

The AC-2 list is not fully covered: nine reviewer mutants on rule 7 survive. The matrix row's "Partial" note names only the FR-376 and foreign-model gaps, so it understates what remains. Three medium findings and one low.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-377-AC-2 names FR-640-AC-11's mutations, and several of them have no test. Surviving mutants prove the gaps: quantity typed at the unit node of `s` (unit check dropped, r24 survives); level spelled `38/40` (FR-322's refusal; no claim-literal spelling test); `from` referencing `Steady` (passes only in a reviewer probe; a predicate slot that admits any target, r23b, survives); `over` referencing `Steady` (the existing test uses a text `over`, which is a shape refusal, not `malformed-declaration`); a `rational` typed at a non-rational node (r38); a `steps` bound's integer type (r37); a level or bound at or below 0 (r32); `weighted by R` requiring a reward (r30); an extra binding after `kind` `origin` (r21); a workload of another model (r35, acknowledged in the matrix) | tests/it/checked_package_v2_probabilistic_claim.rs:631-890 |
| FND-002 | medium | θ ± ι is tested only through a quantile's `level`. Dropping `self.theta = Some(value)` for probability-bounded forms (r33) survives, so `probability`, `mean_of_fraction`, `long_run_fraction` and `reach` confidence checks are unguarded. A reviewer probe (`Avail` with ι = 1/2) shows the code is right today. Charge order within `form` is not pinned either: charging `form` after its nested bindings (c12, one of the coder's own mutants, reported as killed) survives | tests/it/checked_package_v2_probabilistic_claim.rs:722-760,897-978; crates/filament-contract-model/src/checked_package/v2/probabilistic_join/claims.rs:737-738,1044-1047 |
| FND-003 | medium | The matrix row says FR-377-AC-1..4 are implemented except FR-376's visits and a foreign-model workload. But the public read of `Reach`, `Steps`, `MeanOfFraction`, a `Compare` event, a `Fraction` measure, `Each` activation and `over: Some` has no test. A wrong `eventually`/`always` mapping in `probabilistic_claim_read.rs` would pass. AC-1's FR-640-AC-10 identity changes `on each`, `from` to `Idle`, `Avail`'s event and form changes, and `Cost`'s `until` change are also absent. And `whole_read_accounting` registers no claim package, which hides code-review FND-001. List these as Planned in the row, or add the tests | spec/checked_package/matrix/tests.md:26 |
| FND-004 | low | No test reaches a reader locus deeper than 18 JSON Pointer steps (a `CAPACITY = 18` mutant survives), so the capacity raise to 20 is not test-pinned. The 19-step `fraction`-in-`compare` `delay` weighting is the case to add | crates/filament-contract-model/src/checked_package/v2/stage_storage.rs:49 |

## Dispositions

Round 1, at head 4c56c181 (origin/main 2ec296a1 merged), with FILAMENT_SPECIFICATION_DIR set to FSpec origin/main 1d205b5b.

Failing set against clean origin/main 2ec296a1 (CIR-1077-main, own target, same FSpec):

- Workspace: 32 failures on both, identical: 18 `reached_model_read` and 14 await tests. `public_interface` and `claim_names` are green on both.
- `conformance_fspec`, full binary: 8 failures on the PR head and the same 8 on clean main, against the same FSpec (main: 459 passed, 8 failed; PR: 463 passed, 8 failed).

The coder's count of 7 does not reproduce here: the full binary, run unfiltered against FSpec origin/main, gives 8. The eighth is `probabilistic_claims::claim_row_coverage_and_exact_work_boundary`, which the brief already names, so a filtered or partial run is the likely source of the 7.

Mutant re-run against the claim suites:

- Killed: `c12` (by `the_claim_checks_charge_each_binding_in_body_order`), `r21`, `r23b`, `r32`, `r37` (by `each_remaining_fr640_mutation_refuses_at_its_member`), `r24` (by `a_quantity_typed_at_another_node_refuses`), `r25` (by the deepest-locus test), `r30` and `r33` (by `the_confidence_bounds_hold_for_every_probability_form`).
- Survives, deferred: `r35`.
- Survives, live: `r38`.

`r38` is not an equivalent mutant. The generic literal check in operations.rs `literal_defect` skips a literal whose type has no family (`if family.is_none() && !matches!(kind, Float32Bits | Float64Bits | Quantity) { continue; }`), so it never sees a `rational` typed at a familyless node. A reviewer probe set `P95`'s `level` to the `rational` `19/20` typed at the `workload` node `Steady`:

- At the head, the claims-side `typed(ScalarTypeForm::Rational)` check refuses it `invalid_package`/`invalid-value` at `/body/members/5/value/members/1/value`.
- With `r38` applied, the package is admitted.

The check is live, and it needs a killing test. A rational typed at the integer node is refused earlier by the generic check (`ill_typed` at `/type`), which is why the coder's cases did not kill the mutant.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | Every AC-2 item listed is now tested and its mutant killed (unit `s`, `38/40`, `from` and `over` naming `Steady`, the `steps` type, the bounds at or below 0, the reward weighting, the extra `origin` binding, and the predicate slot), except `r38`. `r38` is a live check, not dead code: a `rational` exact value typed at a familyless node (e.g. `Steady`) is admitted when it is removed. Add that refusal case (`invalid-value` at the literal). `r35` (foreign-model workload) stays deferred in the matrix row |
| FND-002 | fixed | `the_confidence_bounds_hold_for_every_probability_form` kills `r33`: θ = ι and θ + ι = 1 refuse, and their neighbours are admitted, for `probability`, `mean_of_fraction`, `long_run_fraction` and `reach`. `the_claim_checks_charge_each_binding_in_body_order` now pins `form` before its nested bindings and kills `c12` |
| FND-003 | fixed | The matrix row is rewritten with the honest remainder: FR-376's step, the foreign-model workload and FSL's producer spelling and ordinal vectors. `the_public_read_covers_every_form_event_measure_and_activation` covers `Reach` (each target), `Steps`, `MeanOfFraction`, the `Compare` event, the `Fraction` measure, the weightings, `Each` and `over: Some`. `the_remaining_identity_changes_give_that_claim_another_entry` covers the AC-10 changes. The claim package is registered in `whole_read_accounting` |
| FND-004 | fixed | The deepest-locus test reaches the 19-step `fraction`-in-`compare` `delay` weighting and kills `r25` (`CAPACITY = 18`) |

Round 2, at head 838dfcbe. The delta from 4c56c181 is one test, `a_literal_typed_at_a_familyless_node_refuses_at_the_literal`, plus the copied SR files. fmt, string-edge and clippy `-D warnings` are clean. `checked_package_v2_probabilistic_claim` and `public_interface` are green. Mutant `r38` (rational type node unchecked) is now killed by the new test.

`whole_read_accounting` fails three registry tests against FSpec origin/main 04109664, on the `tuple-union/wire` vectors TW-29 to TW-33. Those vectors are unrelated to claims and come from FSpec #758 (QSL-2589), which landed after round 1. At the same head with FSpec 1d205b5b, `whole_read_accounting` is green (11 of 11), so the failure is FSpec drift that clean main inherits too, not a defect of this PR.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `a_literal_typed_at_a_familyless_node_refuses_at_the_literal` types `P95`'s `level` `rational` `19/20`, and its `within` `integer` `10`, at the familyless `workload` node `Steady`. Each refuses `invalid_package`/`invalid-value` at the literal. The `r38` mutant now fails it. `r35` stays deferred in the matrix row |
