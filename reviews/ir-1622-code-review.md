---
id: SR-2026112600143100000001
title: "code-review and rust-review of IR-1622 budgeted-read charge ordering"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1189 at 96e076e3ab870014c879524f1c63f0755a2166db (base origin/main 4baf7c931); crates/filament-contract-model/src/checked_package/v2/{claim_names.rs,document_de.rs,intake.rs,model_fields.rs,model_fields/tests.rs,schedulability.rs,term_view.rs}"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in, for filament-contract-ir PR #1189. Ticket: IR-1622. Head reviewed: 96e076e3ab870014c879524f1c63f0755a2166db, which contains current origin/main (4baf7c931, 0 commits behind).

Command run by the reviewer (through the lead-reqs cargo wrapper, `CARGO_TARGET_DIR=/mnt/build/target/ir-1622-read-budget`, `-j 4`, `--locked`): `cargo test -p filament-contract-model --lib -- pays_for_each_list_before_requesting_it are_paid_for_before_they_are_requested tc_472_field_tables_allocate_exactly_what_they_request`, 6 passed, 0 failed, exit 0 (log gates-lead-reqs/ir-1622-review-lib.log). Coder logs read: ir-1622-final-allfeatures.log (626 passed, all features, four of the five ticket tests), ir-1622-cc-modules.log (619 passed, conformance-cli), ir-1622-final-model.log and ir-1622-model-lib-default.log (138 passed each), ir-1622-gate-96e076e3a.log (workspace check exit 0), ir-1622-probe-five.log (five ticket tests on the probe tree, no mismatch logged), ir-1622-allfeatures-modules.log (first fix only: refinement_tables passed, refinement_local_dependencies failed).

## Examined scope

- document_de.rs `exact_vec` / `SEQUENCE_TOKEN` path (examined): the array's whole block is charged in `deserialize_newtype_struct` before the visitor runs, and the sequence is handed on with `prepaid: true`, so `Sequence::next_element_seed` does not charge it again. Charged once, before `Vec::with_capacity`. A non-array value charges nothing and fails in the visitor with the same "a sequence" expectation as serde's `Vec` visitor.
- claim_names.rs `qualified_name`, schedulability.rs `release_bindings` and `mode_switch_bindings` (examined): now `exact_vec`, so each block is charged before it is requested. These were the late charges (serde's `VecVisitor` reserves from `size_hint` before its first `next_element`).
- intake.rs `operation_effects` / `operation_bodies` (examined): `present_rows` replaced by `exact_nonnull_optional_vec`. Absent still defaults to `None`; explicit null, `[]` and `false` still refuse `malformed_wire` at `/operation_effects` (`explicit_null_and_empty_carrier_refuse_at_member` passed in the 619/626 runs). The encode envelope in mod.rs keeps `present_rows`; it is not decoded under a budget, so it rightly stays.
- term_view.rs `decode_term` Frame arm (examined): the four lists now go through `decode_vec_field`, like the other list members of that function. Same locus, same `MalformedMember` mapping.
- model_fields.rs `spell` (examined): `budget.vec` (charge then allocate) now comes before `request(fresh)`, which is followed at once by the `Arc::from` allocations. A denial at the vector no longer leaves `fresh` charged and never requested.
- model_fields.rs `build_component` (examined): `field_type_charged` charges and allocates for itself, so it now runs before the entry's request, and the request is followed at once by the `Arc`, `Box` and `decl.clone()` it pays for. Same form `F`, same row locus, same refusal code; only the amount consumed at a denial moves.
- Sibling sites (examined): model_fields.rs operation-types table (lines 1430-1460), model_members.rs `operation_binders` (3075-3100), state.rs, step_relation.rs and operations.rs callers of `slot_type_charged`/`field_type_charged`. Each charge is followed at once by its own allocation; none has the early-charge gap.
- Other `Deserialize` structs with plain `Vec` members (examined): refinement objects, `SourceOperationFrame`, `CheckedApplicationOperationRecord` and the catalog index are decoded by hand or under an unlimited budget. `DomainKey.path` is inside a tagged enum (see gap-analysis SR).
- model_fields/tests.rs `tc_472_field_tables_allocate_exactly_what_they_request` (examined): it now steps to each request boundary (`consumed + first_unpaid_read_storage`) and refuses charged-but-not-requested bytes. Stronger than the old fixed 53-byte step. Red on 1c12b9b9 without the second fix (ir-1622-red-fields.log).
- New document_de.rs and term_view.rs tests (examined): each one sweeps every byte allowance from 0 to the requirement and asserts allocated == charged and accepted only at the exact requirement, measured by the counting allocator. That is an independent oracle. They were red without the production change (ir-1622-red-sidecars.log, ir-1622-red-frame.log).

## Verdict

Correct. On every path the PR changes, each charge now comes right before its allocation and is made once. Refusal codes, loci and pointers do not move for callers. Admitted totals are unchanged. Only the bytes consumed at a denial inside the field tables move, and no test pins those. The findings are coverage and process items. None is a defect in the change.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Two exact-replay integration tests over the changed paths were not run with test-support on the final head: refinement_tables_are_charged_to_the_read_budget (the ticket's fifth test) and claim_carrier_tables_and_order_are_charged_exactly (reads the schedulability rows whose lists moved to exact_vec). CI does not compile the replay, so nothing else will run them | tests/it/checked_package_v2_refinement.rs:10638, tests/it/checked_package_v2_schedulability.rs:1273 |
| FND-002 | low | The late-charge fallback in Sequence::next_element_seed still accepts a plain Vec wire member without complaint. A new member that leaves out exact_vec brings back the defect, and the only detector (the test-support replay) is not in CI | crates/filament-contract-model/src/checked_package/v2/document_de.rs:900-909 |
| FND-003 | low | The four new document_de tests are tagged FR-038-AC-499 only. The property they assert (no request is made for a denied charge, allocated equals charged at every allowance) is the FR-038-AC-511 clause | crates/filament-contract-model/src/checked_package/v2/document_de.rs:1326-1397 |
