---
id: SR-2026112305284800000001
title: "code-review and rust-review of IR-1586 finite hyperproperty body temporal_profile join"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1136 at e7b81bbb; crates/filament-contract-model/src/checked_package/{common.rs,v2/mod.rs,v2/temporal_binding.rs,v2/term_json.rs,v2/term_view.rs}, tests/it/checked_package_v2_temporal.rs, tests/it/support/checked_package.rs, tests/conformance_fspec/temporal_clock_bindings.rs"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in, for filament-contract-ir PR #1136. Ticket: IR-1586. Governing spec: filament-specification PR #811 (merged), FR-322 "Hyper clause rows" finite-domain paragraph, FR-322-AC-464, TC-2028, `temporal-clock-binding-vectors.json` `finite_hyper_profile_cases` TCB-14..16, `schema.json` `HyperpropertyBody`.

Commands run on titan with `FILAMENT_SPECIFICATION_DIR` at filament-specification origin/main 95708ac3: `cargo test --features conformance-cli --test it -- checked_package_v2_temporal` (110 passed), `cargo test --features conformance-cli --test it -- hyper finite compensation await` (76 passed), `cargo test --features conformance-cli,filament-contract-model/test-support --test conformance_fspec -- temporal_clock_bindings finite_hyper_profile_vectors` (7 passed).

## Examined scope

- temporal_binding.rs `finite_profile` (examined). Joins the body `temporal_profile` to the lock: counts `profile_selections` with role `temporal_profile` and equal definition, requires exactly 1, refuses `invalid_package`/`operation-law-unselected` at `/semantic_graph/nodes/{N}/body/temporal_profile` otherwise, and checks every finite row's clock kind against the named definition. Matches FR-322 and TCB-15/16.
- temporal_binding.rs compensation and await paths (examined). Still use `selected_profile(lock)`; unchanged.
- temporal_binding.rs behaviours-hyper guard (examined). Refuses `invalid-value` at the member; the spec says only that the member is absent there and gives no code, so the choice is not contradicted.
- common.rs and term_json.rs closed-shape widening (examined). Both gate the extra member on `is_body_root`, so nested aggregates still refuse.
- term_view.rs widening (examined). Not gated, but views are built only after the walk has validated the body.
- mod.rs `validate_body` kind guard (examined). Refuses the member on every non-hyperproperty node before the term walk.
- Tests (examined). Reverting `finite_profile` to the first lock selection fails TCB-14, TCB-15, and the absent and twice cases; reverting the closed-shape widening fails every finite fixture.

## Verdict

The finite branch is correct and joins the body definition to exactly one temporal-role lock selection. Compensation, await and behaviours paths are unchanged. Two medium defects: a malformed member is reported as a lock-join refusal, and the non-hyperproperty kind guard is untested. There are also three low items.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A shape-invalid body `temporal_profile` (extra member, non-object, empty text) is refused `operation-law-unselected` instead of as a shape defect; schema.json `HyperpropertyBody` gives it the exact `DefinitionRef` shape | crates/filament-contract-model/src/checked_package/v2/temporal_binding.rs:476-485 |
| FND-002 | medium | The mod.rs guard refusing `temporal_profile` on non-hyperproperty nodes is untested: the clause test hits the application closed-shape refusal (same code and locus) first, so deleting the guard admits the member on any aggregate-bodied node with no test failing | crates/filament-contract-model/src/checked_package/v2/mod.rs:3195-3203 |
| FND-003 | low | term_json.rs widening is dead: its only caller (validate_diagnostics) passes `is_body_root: false` | crates/filament-contract-model/src/checked_package/v2/term_json.rs:407-409 |
| FND-004 | low | TCB-14 compares the admitted first hyper row's clock with `package["temporal_bindings"][0]`, which is not necessarily that hyper row, and asserts the input package's kind rather than the reader's | tests/it/checked_package_v2_temporal.rs:3660-3667 |
| FND-005 | low | conformance test only asserts the published vector file's ids/codes/loci equal literals; it exercises no reader code and passes with the source change reverted | tests/conformance_fspec/temporal_clock_bindings.rs:939-990 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | high | `impl Encode for CheckedTerm` writes an aggregate as `term` and `members` only (`Self::Aggregate { members, .. }`), so the typed `temporal_profile` is dropped. The public `encode_checked_package_v2` writes bodies through this Encode, so writing back an admitted finite-hyper package emits a body with no member, which the reader then refuses (`operation-law-unselected`, TCB-17). The new round-trip test covers serde only, not the canonical writer. | crates/filament-contract-model/src/checked_package/v2/typed_terms.rs:3564-3573 |
| FND-007 | medium | `finite_hyper_profile_vectors_drive_the_reader` panics whenever `TCB_FINITE_HYPER_PACKAGE` is unset, and it lives in the shared `conformance_fspec` binary. `make conformance-fspec` runs `cargo test --test conformance_fspec` without that variable, so the target always fails. The cited precedent, IR1236_FSL_PACKAGE, has its own `test = false` binary (`conformance_fsl_ir1236`), so it never breaks a shared run. Move the driver to its own binary, or `#[ignore]` it with a named Makefile line that sets the variable. | tests/conformance_fspec/temporal_clock_bindings.rs:1062-1069 |
| FND-008 | low | The driver keeps a `set-hyperproperty-domain-behaviours` mutation arm that no published case uses. Spec PR 815 at b87c2a87 removed that op, so the arm is dead test code. | tests/conformance_fspec/temporal_clock_bindings.rs:995-1013 |

## Dispositions

Round 1, against 8fc3d38b (fix delta e7b81bbb..8fc3d38b). Spec reference: filament-specification PR 815 head b87c2a87, checked out read-only in a temporary worktree. Runs on titan:
- `it -- checked_package_v2_temporal checked_package_v2_typed_terms hyper finite`: 127 passed.
- `conformance_fspec -- temporal_clock_bindings` with `TCB_FINITE_HYPER_PACKAGE=/tmp/tcb-finite-hyper.json`: 7 passed. The driver ran TCB-14..18 and TCB-20..24 against the reader.

Branch tree: `git ls-tree -r HEAD` has no `$HOME`, `.cache`, `target` or `.rlib` paths. The 72-file delta is the typed member, `..` pattern updates for the public enum change, tests, and the two SR files.

TCB-19 skip (`UNGENERATED`, QSL-2887): acceptable. QSL-2887 is Backlog and records that the FR-402-AC-11 object-free hyper source fails to parse, so the base package for TCB-19 cannot be produced yet. The skip names its ticket, which is also its expiry. The same refusal is asserted in-repo by `a_behaviours_hyper_body_carries_no_profile_member_and_other_nodes_may_not`. No gate is weakened.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | still-open: the term_json.rs widening is unchanged and still dead (only caller passes is_body_root false) | |
| FND-004 | fixed | - |
| FND-005 | fixed | - |
