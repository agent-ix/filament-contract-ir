---
id: SR-20261116190004
title: "gap-analysis of IR-1552 Clock tuple positions and union payloads"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1032 at 7852391e (base origin/main 90b90fc7); FR-038-AC-548..553, FR-069-AC-9; tests/conformance_fspec/clock_payload_tuple.rs, tests/conformance_fspec/model_tuple_union.rs"
review_set: subset
---

## Summary

Planless, PR-diff-scoped gap analysis. Ticket: IR-1552. Plan completion: not assessed. Criteria examined: FR-038-AC-548, -549, -550, -551, -552, -553 and FR-069-AC-9, against the six new tests in `tests/conformance_fspec/clock_payload_tuple.rs` and the re-tagged TK replay test, with mutation evidence in the code-review SR. `quire coverage --scope . --strict`: 61 unbacked rows at head, 61 at origin/main (no regression; the new tests.md rows use AC ranges, so the per-AC gap below is not visible to it).

Gate at 7852391e: whole `cargo test --locked --workspace --features conformance-cli --no-fail-fast`: 2289 passed, 7 failed: the 2 `lock_coherence` environment failures (`quire check-lock` absent) and 5 `string_edge_gate` failures caused only by the scanner not being on PATH for that run, which pass (6/6) when rerun with `fsl-string-edge` on PATH; so 2294 passed and 2 environment failures, as the coder claimed. `cargo test --locked --features conformance-cli --test conformance_fspec -- --test-threads=1`: 454 passed, 0 failed. fmt, clippy -D warnings, cargo deny, quire validate and make string-edge clean; Cargo.lock unchanged.

## Examined scope

- FR-038-AC-548 (examined): `clock_tuple_position_and_union_payload_places_admit_without_a_clock_node` and `forged_tuple_and_union_clock_place_rows_refuse_at_the_differing_row`; 7 of 9 forged cases depend on the new join, 2 on pre-existing checks; all 9 assert the row pointer.
- FR-038-AC-549 (examined): `union_value_of_a_clock_case_carries_the_empty_payload`, fails with the Clock payload branch removed.
- FR-038-AC-550 (examined): `writing_a_clock_union_case_is_a_reset_of_its_place`; both the written and the unwritten case fail under the matching frame mutation, so the existing reset rule is really exercised for a union-case place.
- FR-038-AC-551 (examined): `projecting_a_clock_tuple_position_refuses_clock_read_outside_constraint`, with a position-1 admit control.
- FR-038-AC-552 (examined): behaviour is covered (the `Set<Slot>[0, 2]` place admits inside the AC-548 admission test with a row per step; the differing slot maximum refuses at row 0 inside the AC-548 forged test). That is enough evidence for the behaviour, but no test carries the AC-552 tag (FND-001).
- FR-038-AC-553 (examined): `map_keyed_by_a_clock_bearing_type_refuses_at_the_key`, Slot, Phase and a Text control; fails with the key refusal removed.
- FR-069-AC-9 (examined): TK-08/TK-09 admit through `published_model_tuple_union_intake_cases_refuse_at_exact_member` (fails when Clock is re-refused at intake); arity and unchanged declarations asserted in the AC-548 admission test.

## Verdict

Every new criterion has a test that can fail; one criterion has no trace tag. The CP-07 remainder is recorded in the spec-review SR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-552 (`Set<Slot>[0, 2]` clock collection and the differing-maximum refusal) has no `#[trace]`-tagged test: its evidence sits inside the AC-548 admission and forged tests, tagged only FR-038-AC-548 (and FR-069-AC-9). The tests.md range row "FR-038-AC-548 through FR-038-AC-553" hides the gap from `quire coverage --strict` | tests/conformance_fspec/clock_payload_tuple.rs:545,644; spec/checked_package/matrix/tests.md:169 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At b869112b `set_of_clock_bearing_tuples_is_a_clock_collection_bound_to_the_field` is tagged `#[trace("TC-048", "FR-038-AC-552")]`, asserts three rows, the field row's `set`/0/2, and the differing-maximum refusal at row 0; tests.md now has one row per AC, FR-038-AC-548 through FR-038-AC-554. `quire coverage --strict` at 3e8b9913: 61 unbacked, unchanged from main. |
