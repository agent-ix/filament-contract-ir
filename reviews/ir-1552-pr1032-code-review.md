---
id: SR-20261116190003
title: "code-review and rust-review of IR-1552 Clock tuple positions and union payloads"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1032 at 7852391e (base origin/main 90b90fc7); crates/filament-contract-model/src/checked_package/{shared.rs,v2/contract_package.rs,v2/model_fields.rs,v2/model_members.rs,v2/operations.rs,v2/timed_model.rs}, tests/conformance_fspec/{clock_payload_tuple.rs,main.rs,model_tuple_union.rs}, tests/it/checked_package_v2_temporal.rs"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in, for filament-contract-ir PR #1032. Ticket: IR-1552. Oracle (read only): filament-specification origin/main 959a1982, FR-322 "Clock positions and payloads", the `resolved_path` row definition, FR-322-AC-399, TC-1361 (CP-01..CP-07), `proposals/checked-package-v2/schema.json` (`TimedTuplePathStep`, `TimedUnionPathStep`) and `model-tuple-union-vectors.json` (TK-08, TK-09).

Gate rerun at 7852391e in a separate review worktree, CARGO_TARGET_DIR inside it, `--locked`, serialized on `/tmp/cargo-slot.lock`: `cargo fmt --all -- --check` clean; `cargo clippy --locked --workspace --all-targets --features conformance-cli -- -D warnings` clean; `cargo deny check` ok; `quire validate` of spec/plan/reviews exit 0; `make string-edge` clean with `fsl-string-edge` installed as CI does into a scratch root on PATH; `quire coverage --scope . --strict` 61 unbacked rows, equal to origin/main; whole workspace and single-threaded `conformance_fspec` results are in the Verdict. Cargo.lock unchanged.

Mutation runs (each reverted after, focused `clock_payload_tuple` and `model_tuple_union` tests, 18 tests):

- M1 join disabled (`timed_model.rs` `if !joined.is_some_and(..)` never taken): 7 of the 9 forged cases admit. "data position as the terminal" and "set bound different from the field's" still refuse through the existing Clock-terminal and slot-bound checks, which is correct.
- M1b join without the identity comparison: "another position's identity" and "another case's identity" admit.
- M1c join without the member-type comparison: all 18 tests pass (FND-001).
- M3 `selected_union_members` Clock branch removed: the union_value and reset tests fail.
- M4 `check_tuple_position` Clock refusal removed: the projection test fails.
- M5 `arity` reverted to `payload.is_some()`: the admission test fails.
- M6a `frame_writes_place` always true: the reset test fails at its unwritten case, so the existing reset frame rule is exercised by the new test. M6b always false: the reset test fails at its written case.
- M7 map key refusal removed: the map-key test fails.
- M8 / M9 Clock re-refused as tuple position / variant payload at intake: TK-08/TK-09 replay and all five dependent tests fail.

## Examined scope

- model_members.rs intake (examined): `Clock` removed from the refused tuple-position list and added to the admitted payload list; Rational, Decimal and Text stay refused (FR-322 refusal list).
- model_members.rs `aggregate_step_member` (examined): tuple identity `<tuple identity>/p<index>` is FR-154 / FR-322 / FR-208's position identity, and matches every published domain-package vector (`ix://acme/shapes/Pair/p0` etc.); the variant case identity is the selected document's case `identity`, as field rows use the field's. Budgeted concat; `#[string_edge]` marked.
- timed_model.rs `check_resolved_path` (examined): the old text forms `declared/tuple/<slot>` and `declared/union/<case>` are gone. They were never published by any vector and no workspace consumer uses them (grep). The tuple step index is read as an integer literal, matching `TimedTuplePathStep` (`TimedIntegerLiteral`); the old `text_member` read could never have matched a schema-valid step.
- operations.rs `selected_union_members` (examined): a Clock case gets a zero-length payload; `check_tuple_position` refuses a Clock position with `invalid_timed_model`/`clock-read-outside-constraint` at the application.
- shared.rs / contract_package.rs (examined): new cause `ClockReadOutsideConstraint` admitted only with `InvalidTimedModel`; both `cause_word` tables updated.
- model_fields.rs `arity` (examined): zero for a Clock payload.
- tests/conformance_fspec/clock_payload_tuple.rs, all six tests (examined): each can fail under at least one mutation above; each is `#[trace]`-tagged; no stub or tautology. The no-presence check is weak by construction (closed schema allows only `model`, `path`, `resolved_path`, and the test also asserts three members), which is adequate.
- Structural (examined): no pins, hashes, compat layers or tool versions in the diff.

## Verdict

The code is correct for the ticket bullets a to g, and every new check except one comparison can be shown to fail its test. One medium finding: the join's member-type comparison is never exercised. The completeness finding on CP-07 is in the spec-review SR.

Gate at 7852391e: whole `cargo test --locked --workspace --features conformance-cli --no-fail-fast`: 2289 passed, 7 failed: the 2 `lock_coherence` environment failures (`quire check-lock` absent) and 5 `string_edge_gate` failures caused only by the scanner not being on PATH for that run, which pass (6/6) when rerun with `fsl-string-edge` on PATH; so 2294 passed and 2 environment failures, as the coder claimed. `cargo test --locked --features conformance-cli --test conformance_fspec -- --test-threads=1`: 454 passed, 0 failed. fmt, clippy -D warnings, cargo deny, quire validate and make string-edge clean; Cargo.lock unchanged.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The join's member-type comparison (`member_type == Some(rendered.as_str())`) survives mutation: with it replaced by `true`, all 18 focused tests pass. No forged case has a tuple or union row whose identity is the selected one but whose member type is not, on a row that the Clock-terminal check does not already catch (a non-terminal tuple/union row, e.g. a position typed by a record with a clock field). FR-038 names "a member type ... that is not the declared one" as a refusal | crates/filament-contract-model/src/checked_package/v2/timed_model.rs:1282-1286; tests/conformance_fspec/clock_payload_tuple.rs:646-809 |
