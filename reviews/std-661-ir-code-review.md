---
id: SR-2026111422264100000001
title: "code-review and rust-review of STD-661 operation_bodies owner compare and TC-987 harness"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1024 at 5c735908 (base origin/main 58fb1d86); crates/filament-contract-model/src/checked_package/v2/{operation_bodies.rs,mod.rs}, tests/conformance_qspec/operation_bodies.rs"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in, for filament-contract-ir PR #1024. Ticket: STD-661. Reference (read only, extracted to a scratchpad and deleted after): filament-specification origin/main 17d49a98, FR-322 "Operation-body carrier" (Check order, Candidate join), TC-987 and `proposals/checked-package-v2/operation-body-vectors.json`.

Commands run with `FILAMENT_SPECIFICATION_DIR` at the extract: `cargo test --all-features --test conformance_qspec operation_bod` (5 passed), `cargo clippy --all-features --test conformance_qspec -- -D warnings` (18 errors, all in files the diff does not touch, so they are already on origin/main: main.rs 2, protocol_vectors.rs 4, node_index.rs 2, ../it/support/read_observation.rs 5, dimension_equivalence.rs, entry_keys.rs, reaches_population.rs, grouped_ordinal.rs, node_ordinal.rs 1 each), `cargo clippy --all-features -p filament-contract-model --lib -- -D warnings` (clean), `cargo fmt --check` (clean).

## Examined scope

- operation_bodies.rs `owned_by` and the `owned` list (examined). The owner is now compared by `node` only. FR-322 "Check order" says the carrier compares the owner `node` only, because the member identity names its document. The owner join (owner.rs `validate_owner_joins`) admits a `ModelOwner` only when its `identity` document declares `node` as an operation. `member_identity` forces an operation identity to be `<object id>/<name>`, `classify` forces an object id to be `ix://<package>/<object id>`, and neither part may hold a `/`. So equal member identities mean equal documents. STD-660 FND-003 holds and no refusal is weakened. OBR-17 and OBR-19 still refuse at `/operation_bodies/2/body/node`.
- operation_bodies.rs `validate` candidate join (examined, not changed): `operation_in_family` then `effective_operation_owner`. This matches FR-322 "Candidate join". `has_body` is derived from the graph, and the check order is graph stages, then `operation_effects`, then the carrier, then `checked_term_data` (mod.rs). Claim 1 holds.
- mod.rs and operation_bodies.rs comments (examined): they cite FR-322 and OBD-01, with no stale OBR-20 or OBR-21 text.
- tests/conformance_qspec/operation_bodies.rs (examined). `extends` and `redefines` are read from the sketch. The `zip` with `expand_document` is 1:1 because that function maps operations in order. Candidates are built from `selection_key`, and every published candidate carries one (9 checked). The counts asserted are 7/13/21/1. Removing the `foreign_owner` filter and the OBR-21 branch is correct: OBC-04's foreign node is now owned by acme/billing.
- the new test `an_inheriting_receivers_candidate_must_carry_the_inherited_rows_body` (examined): Plain inherits Service.run, so it joins row 2 (Service), and FastService's body refuses there. The test is correct.
- Public API (examined): no `pub` item changed. Only the private `owned_by` signature changed. The diff does not touch the ir1384 lane or the Makefile.

## Verdict

Clean. The production change is the one FR-322 states, and the reasoning that it weakens nothing traces through the reader's own identity checks. The test-coverage gap left by the removed Premium tests is reported in the gap-analysis SR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
