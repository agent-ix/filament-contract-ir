---
id: SR-17940486709564847048096
title: "gap-analysis of IR-1576 FR-369-AC-25/26 and FR-367-AC-8 against PR #1082"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1082 at fafbdc18 (git diff origin/main...HEAD); FR-369-AC-25, FR-369-AC-26, FR-367-AC-8, TC-484, TC-485, FR-377-AC-1; crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs, tests/it/checked_package_v2_source_objects/probabilistic.rs, tests/it/checked_package_v2_whole_read_accounting.rs, spec/checked_package/matrix/tests.md"
review_set: subset
---

## Summary

Gap analysis for filament-contract-ir PR #1082. Ticket: IR-1576 (covers QSL-2465's code half; QSL-2465 is Duplicate of IR-1576). Plan completion: not assessed. The ticket's arch rulings on QSL-2465 (comments c3d245ee and 3daefcce) are read as the scope statement.

Computed matrix (`quire matrix --format tsv` at the PR head). FR-369-AC-25 and FR-369-AC-26 now read `tagged`, bound to `tests/it/checked_package_v2_source_objects/probabilistic.rs`. FR-369-AC-5, AC-11 and AC-17 gain bindings from the same file. FR-367-AC-8 is unchanged.

Mutants: 15 hand mutants on the new paths (see code-review SR-17940486709564847048095): 11 killed, 1 equivalent (M3), 3 surviving (M11, M13, M14), recorded as FND-004 and FND-005.

## Examined scope

- FR-369-AC-25 (examined). The strict-read half and the typed read are tested in `source_declared_probabilistic_nodes_are_admitted_and_read`, including the independently built declared-node id `{package, ["Other","poke","reward","cost"], 0}`. "Admitted ... by the identity-row join" and "the checked-in golden pair" are absent (FND-002).
- FR-369-AC-26 (examined). Tested at the strict read: the source relation/population `declaring` (dimension and population), no `dense` member, the three declaration rewrites, `clause` `precondition`, and the second `SourceOwner`. The identity-row join refusals are absent: `nudge`, `j`, support `4`, a workload omitting `O::step`, the reward `parameters` `j` and none, the `Clock` variant, and the budget one visit short (FND-002).
- FR-367-AC-8 (examined). Not implemented: `SelectedSourceObjectOperation` has no `parameters` (FND-001).
- TC-484 steps 1-6 (examined). Step 1 is partial (no join, no golden). Steps 2 and 4 are done. Steps 3 and 5 are absent. Step 6 is absent: the "strict part of 6" in the matrix row is the per-row and per-scanned-node charges, which are FR-369-AC-5/AC-17 strict charges, not TC-484's join count.
- `TC048/source_probabilistic` registry case (examined). It holds the source reward function, the workload and the dense-root scan, so the new `listed` and `named` buffers are inside the whole-read bound. It drops `delay_poke` and `random_k` because their rational parses are paid by stages this registry does not yet hold. The source branches of those two forms allocate nothing new, so no new allocation is outside the registry.
- FR-377-AC-1 regression test (examined). Clean; binding `TC-489`/`FR-377-AC-1` is correct.

## Verdict

The strict-read half that was delivered is tested at the right loci, and the tests are honestly labelled "strict-read half". There are three gaps. First, the arch ruling placed FR-367-AC-8 in this PR, and it is fully specified, but it was not delivered and no other ticket owns it (FND-001). Second, the matrix reports FR-369-AC-25/26 as `tagged` while half of each AC is unbuilt (FND-002). Third, four tests carry a TC id that does not list their AC (FND-003). Two strict-read behaviors lack a killing test (FND-004, FND-005).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-367-AC-8 is ruled in scope and not delivered. The QSL-2465 arch ruling (comment c3d245ee, "FR-367-AC-8 ... has no ticket and must land in this PR") and the readiness sweep make the `parameters` member of `SelectedSourceObjectOperation` (clock and data entries, the `UnknownIdentityWitness` refusals at the `Package` locus) part of this ticket. It is fully specified in FR-367 and does not depend on the missing join entry point (spec-review FND-003). The PR does not add it, and QSL-2465 is Duplicate, so it would be left without an owner | crates/filament-contract-model/src/checked_package/v2/source_object.rs:78-87; spec/checked_package/matrix/tests.md:20 |
| FND-002 | medium | Coverage inflation: the computed matrix shows FR-369-AC-25 and FR-369-AC-26 as `tagged` (fully bound), but the identity-row join half of each AC is absent. That half is the admission "by the identity-row join", the golden pair, the seven join refusals, the `Clock` variant and the budget, i.e. TC-484 steps 3, 5 and 6. The matrix row's prose says so, but the tooling-visible status does not. Split the ACs, or keep the row and add a tracked Planned marker the matrix reads | tests/it/checked_package_v2_source_objects/probabilistic.rs:422-424,477-479,523-525,615-617,767-769,802-804,913-915,924-926 |
| FND-003 | low | Trace: four tests tag `TC-484` with criteria TC-484 does not verify. TC-484 lists only FR-369-AC-25/26; FR-369-AC-5, AC-11 and AC-17 belong to TC-476. The tests are `a_source_workload_charges_one_visit_for_each_row` (AC-5), `a_second_source_declared_definition_is_a_conflicting_definition` (AC-11), `source_delays_of_different_operations_are_no_conflict` (AC-11) and `a_source_delay_charges_each_node_scanned_for_the_root` (AC-17). Tag them `TC-476`, or add the ACs to TC-484's row | tests/it/checked_package_v2_source_objects/probabilistic.rs:679,867,895,941 |
| FND-004 | low | A source `declaring` node whose `operation` binding is not text is refused at `operation` (`Join::operation`, source branch), but no test reaches that branch. Mutant M11 (accept a missing name as `""`) survives. Add the missing-text case to `source_declared_declaring_nodes_refuse_at_the_member` | crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs:335-337 |
| FND-005 | low | The storage charges of the two new buffers are correct in the code but are not pinned by a test. These are `source_reward`'s `listed` (one slot per listed parameter) and `source_workload`'s `named` (one per row). Replacing either with an unmetered `Vec::with_capacity` survives (M13, M14), because `TC048/source_probabilistic` has one reward parameter and one workload row, far under the registry's 4096-byte `UNPAID_BOUND`. Add a registry or `read_storage_alloc` case whose reward lists, or whose workload holds, enough entries to exceed the bound when unpaid | tests/it/checked_package_v2_source_objects/probabilistic.rs:353-367; crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs:613-617,900-904 |

## Dispositions

Round 1, at head 44ce41ca. The fix commit is 91ad84dd, merged with origin/main c2146579, and the merge delta is only the PR's files. The filtered `--test it` gate and 9 mutants were run at 91ad84dd, and the gate was re-run at 44ce41ca.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-367-AC-8 is implemented. `SelectedSourceObjectOperation` gains `pub parameters: Vec<SelectedSourceObjectMember>`. `SelectedSourceObjectMember` gains `type_node: Option<CheckedNodeId>`, `member_type` and `collections`, as FR-367 "Member entries" states. `operation_holds` and `entry_shape_holds` refuse `UnknownIdentityWitness` for both or neither of `type_node` and `member_type`, a non-type `type_node`, repeated names and a `clock_parameters` list that is not exactly the native-`Clock` entries in order. These are tested in `parameters.rs` (TC-485), and mutants N1, N2, N3, N6 and N7 are killed. The FR-367-AC-7 clock-member checks it needs are also covered (N4 and N5 killed) |
| FND-002 | still-open | The matrix rows and ACs are split correctly: FR-369-AC-27/28 hold the strict-read half, and AC-25/26 are Planned under QSL-2645. But `quire matrix` at the head still reports FR-369-AC-25 and FR-369-AC-26 as `tagged`. They are bound at `probabilistic.rs:475` and `:820` (AC-25) and `:530`, `:576` and others (AC-26), because quire harvests the doc-comment citations `/// FR-369-AC-25 (strict-read half)` (lines 468 and 815) and `/// FR-369-AC-26 ...` (lines 526, 571 and 851), plus the module doc at line 6. Reword those six comments to cite FR-369-AC-27/28 so the computed status of AC-25/26 matches the Planned row |
| FND-003 | fixed | The AC-5, AC-11 and AC-17 tests now carry `#[trace("TC-476", ...)]`: `a_source_workload_charges_one_visit_for_each_row`, `a_second_source_declared_definition_is_a_conflicting_definition`, `source_delays_of_different_operations_are_no_conflict` and `a_source_delay_charges_each_node_scanned_for_the_root` |
| FND-004 | rejected | The branch is unreachable from a package. The wire stage decodes `operation` as a text literal first and refuses a non-text term as `malformed_wire` at `/body/members/1/value`, so the join's `invalid-value` branch is defensive only, and M11 is equivalent. The new test `a_source_operation_reference_without_a_text_operation_refuses_at_the_operation` pins the refusal that actually happens, for delay, random parameter and reward |
| FND-005 | fixed | M13 (an unmetered `listed`) is now killed twice: `TC048/source_probabilistic` lists 520 parameters ("unpaid read memory 7592 exceeds its independent bound 4096"), and `the_source_buffers_are_charged_before_they_are_made` expects one request each of 8×22 and 16×13 bytes at the probabilistic node. M14 (an unmetered `named`) is killed by the latter. The coordinator's mutant Q, the selected-model workload's `named` buffer, is pre-existing code outside this diff and not a finding here |
