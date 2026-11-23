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
