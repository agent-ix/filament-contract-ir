---
id: SR-958003
title: "code-review of QSL-2776 clock-reference Exact projection (PR #1145)"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR #1145 at b9d663da; Cargo.toml, Cargo.lock, crates/filament-contract-model/Cargo.toml, crates/filament-contract-model/src/checked_package/v2/exact_projection.rs, exact_projection/model_union.rs, model_members.rs, native_join_storage.rs, tests/it/checked_package_v2_model_fields.rs"
review_set: subset
---

## Summary

Code review with the Rust lane folded in, scoped to `git diff origin/main...b9d663da`. Ticket: QSL-2776.

## Examined scope

- exact_projection.rs `project_record_field_type`: the new `MemberType::ClockReference => (ValueType::ClockReference, 1)` arm (examined). It charges storage 1 through `record_field_value`. Plain `Clock`, `Option` and `Collection` leaves still refuse `UnsupportedExactType`.
- native_join_storage.rs `preflight_value_type` and `value_type` (examined). `ClockReference` is in the scalar arm of both exhaustive matches.
- model_members.rs, the pre-pass `clock_references` set (examined). It resolves the `target` through `option_types::reference`, the same alias resolver the definition reader uses. The alias table is built before the pre-pass, so the result does not depend on declaration order. Alias loops end at `followed > aliases.len()`. The set feeds the union payload check and both tuple position checks.
- Cargo.toml, crate Cargo.toml and Cargo.lock (examined). `cargo tree -i` shows one filament-exact (08a5a176) and one quire-canonical (v0.3.2 f00840e4). The lock diff covers only the four moved sources plus quire-canonical-derive.
- Focused runs on titan: `-p filament-contract-model --lib exact_projection` passed 20 of 20. `--features conformance-cli --test it checked_package_v2_model_fields` passed 93 of 93, including the 3 new or changed tests.
- Known red on main, reproduced in a clean detached worktree at fe49a2a2 with the same failures: lib tests `tc_048_a_reference_to_a_refused_node_leaves_that_nodes_own_refusal` (tests.rs:1328, left Ok(()), right Err(InvalidModelBinding/MalformedDeclaration)) and `tc_048_semantic_ir_declaration_defects_refuse_with_their_fr_154_cause` (tests.rs:1204), and `cargo check --workspace --all-targets` E0432 at tests/it/source_obligation_filament.rs:22. This PR does not cause them.

## Verdict

The logic is correct and idiomatic. There is no panic path and no new allocation outside the budget. One formatting defect would fail the fmt lint.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | `cargo fmt --all --check` fails. rustfmt joins `MemberType::Clock \| MemberType::Option(_)` and `\| MemberType::Collection { .. } => {` onto one line. The fmt lint goes red until `cargo fmt` runs. | crates/filament-contract-model/src/checked_package/v2/exact_projection.rs:3502-3503 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | low | The fix commit acc267fb rewrote `NativeJoinLocus {` to `filament_contract_model::NativeJoinLocus {` in two unrelated tests but kept the local `use filament_contract_model::{NativeJoinLocus, ...}` lines. The build now warns `unused import: NativeJoinLocus` at both places, and `make lint` (clippy `--all-targets --features conformance-cli -D warnings`) fails. Revert the churn, or drop `NativeJoinLocus` from both `use` lines. | tests/it/checked_package_v2_model_fields.rs:3168; tests/it/checked_package_v2_model_fields.rs:8679 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At acc267fb the arm reads `MemberType::Clock \| MemberType::Option(_) \| MemberType::Collection { .. } => {` on one line, and `cargo fmt --all --check` exits 0. |
| FND-002 | still-open | Added in this round. The fix commit introduced two unused `NativeJoinLocus` imports (tests/it/checked_package_v2_model_fields.rs:3168 and 8679), which fail clippy -D warnings. Not yet fixed. |
