---
id: SR-4272
title: "code-review and rust-review of IR-1614 protocol clause owner operand narrowing"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1173 at 414e28ad (base 5a4b720b); crates/filament-contract-model/src/checked_package/v2/{protocol_body.rs,identity.rs}, tests/it/{checked_package_v2_protocol_body.rs,checked_package_v2_canonical_encoding.rs,checked_package_v2_frame_entries.rs,checked_package_v2_model_members.rs,checked_package_v2_refinement.rs,support/checked_package.rs}"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in, for filament-contract-ir PR #1173. Ticket: IR-1614. References read: FR-038-AC-629, FR-038-AC-367, FR-038-AC-374, FR-063-AC-7, FR-040-AC-18 to AC-20 in this repository, and filament-specification origin/main FR-322 "Structural node identity" and FR-322-AC-538. Workspace check (`cargo check --locked --workspace --all-targets --all-features`) exited 0 on this head. No tests were run by the reviewer.

## Examined scope

- protocol_body.rs `owner_path_operand` / `owner_operand_names` and the two call sites (examined). The operand is the last segment of a path whose length is the declaration path plus one and whose last segment is an index or node segment. `validate_declaration_names` (mod.rs, run earlier in `validate_graph`) already refuses every declared node whose path is not the declaration path or, for a protocol clause, the declaration path plus one index or node segment. So for a declared clause only two shapes reach the owner step: declaration path alone (now admitted without comparison) and declaration path plus operand (compared as before). A name segment, an operand-slot segment or a longer path is still refused earlier with `declaration-nominal-mismatch`; the new test pins the longer case.
- Unowned-operand check (examined): `owner_model.is_none() && owner_path_operand(node).is_some()` is the same predicate as the removed inline one, with the same charge and locus.
- Owner step order and charge (examined): presence/type, missing target, object-type and owner-recovery refusals still precede the comparison; the one `node_work` charge now applies only when a comparison happens, so a declaration-only owned clause no longer pays one unit. No limit or ordering for any other refusal changed.
- identity.rs `check_kind` root-scale check (examined): it now uses the shared `valid_rational` helper, so a public `try_dimension_equivalence` refusal points at `/root_scale/numerator` or `/root_scale/denominator` (FR-038-AC-374, body-relative offending member) instead of `/root_scale`. The charged reader refuses root-scale defects at the equivalence node in its own `rational` closure before `check_kind` runs (FR-063-AC-7), so reader behaviour is unchanged. The other `check_kind` caller (the charged reader) and the `scale`/`offset` paths are unchanged.
- identity.rs unit test (examined): it now calls the wire `from_value` with a body pointer and expects the node pointer, which is what FR-063-AC-7 requires; the earlier expectation could not hold for that function.
- canonical_encoding retired-identity scan (examined): replaced "no file names a retired identifier" with an exact list of three files naming only `StaleNodeKey`. A new occurrence in any other file, or any other retired name, still fails. FR-038-AC-367 itself keeps the one stale-node-key refusal for AC-629.
- frame_entries tc_056 (examined): expectation restored to `invalid_semantic_graph`, no cause, at the entry, which is FR-040-AC-18's literal text.
- model_members enum stage rows (examined): every limit moved by exactly one charge, the success boundary (7) is paired with a new one-short refusal (6 -> WorkLimit) and the existing one-short case moved 2 -> 3; no assertion was dropped.
- refinement step target (examined): the mutated step now lists its new target type as a dependency so the refusal under test is the step-row refusal, not a dependency-closure refusal; the asserted refusal is unchanged.
- support `refresh_temporal_bindings` (examined): returns early when the package locks no source; such packages had no lock source for a row to name.
- New AC-629 tests (examined): `protocol_owner_declaration_only_path_admits` fails on the base (StaleNodeKey) and asserts the admitted path and owner edge literally; `protocol_owner_path_longer_than_declaration_must_name_the_owner` keeps the operand-mismatch refusal and pins the decl+2 refusal; the FSpec fixture test runs only with the FSpec checkout, as the other conformance tests in this file do.
- Public API (examined): no `pub` item, serialized field or identity rule changed.

## Verdict

Approve with one medium finding. The narrowing is exactly the lead's option A for declared clauses, and no malformed declared path is newly admitted: the declared-path check refuses every shape other than "declaration path" and "declaration path plus one operand" before the owner step. The accepted, recorded consequence is that a retargeted `owner_model` on a declaration-only path admits (STD-727). The finding is that the predicate also requires a declaration, so an owned clause without one is no longer compared at all. No test was weakened to pass.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `owner_path_operand` returns `None` for a clause with no `declaration`, so an owned `protocol_clause` without a declaration now skips the owner comparison entirely, whatever its path holds; before, its last segment had to name the owner. Option A and the new FR-038-AC-629 sentence only cover the declaration-only path; nothing states or tests the undeclared case | crates/filament-contract-model/src/checked_package/v2/protocol_body.rs:66-77, crates/filament-contract-model/src/checked_package/v2/protocol_body.rs:7185-7196 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | rejected: not a defect | Verified at 4993b028: flat_wire.rs (unchanged since base) refuses `malformed_wire` at the node for every protocol form except queue and reference whose `declaration` presence is wrong, so an owned clause without a declaration never reaches the owner step. `protocol_owner_clause_without_declaration_refuses_at_the_wire` pins this for an owner operand, another node's operand and a name; the FR-038-AC-629 text now states it. |
