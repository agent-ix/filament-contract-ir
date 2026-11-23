---
id: SR-967175026302197768900474181941935084590
title: "Code review — filament-contract-ir PR #1146 (QSL-2891 slot-domain maximum)"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; crates/filament-contract-model/src/checked_package/v2/operations.rs"
review_set: subset
---

## Summary

Ticket: QSL-2891. Reviewed PR #1146 head `5bf75837` against `origin/main` (`git diff origin/main...HEAD`), with the rust-review lane folded in. The change makes `slot_domain_result_matches` read the expected collection maximum from the `resolved_path` row before the terminal slot step. The claim holds against `timed_model.rs` `check_resolved_path`: row `i` carries `collection_kind`/`minimum`/`maximum` equal to step `i+1`'s bounds only when step `i+1` is a slot (timed_model.rs:1405-1414); every other row, the terminal slot row included, must spell `none`, `""`, `""` (timed_model.rs:1416). A place whose first step is a slot is refused `malformed-clock-place` (timed_model.rs:1178-1183), so the slot is never step 0 in an admitted place. Multi-level paths (`field ts, field ys, slot`, or nested `field q, slot, slot`) also put the terminal slot's bounds on row `last - 1`. Before the fix, the two-step test case reads row 1's `""` maximum, `parse::<u64>()` fails, and the function returns false, so `read_slot_domain("2", true)` would return the refusal instead of `Ok(true)`: the new case is a real regression test (mutation checked by reading). Focused tests pass on terra: `fr772_option_slots_checks_place_owner_and_bounded_domain` and `timed_model::tests::map_slot_spells_the_declared_bound_of_its_map_type`.

## Verdict

**CONDITIONAL**: the production fix is correct. The PR's own test hunks are not rustfmt-clean, which fails `make fmt-check`, and the retained one-step fixture asserts admission for a place shape the reader refuses.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `rustfmt --check` (repo rustfmt.toml, edition 2021) reports diffs only in this PR's hunks: the six-parameter `row` closure (one-line closure header over 100 columns) and the `let Err(...) = read_slot_domain("3", field_step) else {` split. `make fmt-check`, part of `make ci`, fails. origin/main's copy of the file is clean. | crates/filament-contract-model/src/checked_package/v2/operations.rs:14518,14664 |
| FND-002 | medium | The one-step branch (`field_step = false`) asserts `Ok(true)` for a place whose only step is a slot and whose slot row carries `sequence`/`0`/`2`. `check_resolved_path` refuses both: a first-step slot (timed_model.rs:1178) and a slot-terminal row that is not `none`/`""`/`""` (timed_model.rs:1416). This is the fixture shape that hid the bug. The branch should be dropped or turned into a refusal, and the loop should run the real two-step shape only. | crates/filament-contract-model/src/checked_package/v2/operations.rs:14545-14550,14663 |
| FND-003 | low | The expected maximum is located by index arithmetic (`last_step_index.saturating_sub(1)`). The terminal slot step's own `maximum` binding (`bound(last_step, 4)`) is joined equal to that row by timed_model.rs:1405-1414, and reading it directly would remove the index coupling. It would also remove the `saturating_sub` that maps an unreachable index 0 to the slot's own row. | crates/filament-contract-model/src/checked_package/v2/operations.rs:6071-6073 |
