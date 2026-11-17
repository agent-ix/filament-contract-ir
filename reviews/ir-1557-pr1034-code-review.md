---
id: SR-17940486709564847048038
title: "code-review and rust-review of IR-1557 function node check_mode"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1034 at 829e7c07 (base origin/main b17056bb); crates/filament-contract-model/src/checked_package/v2/{mod.rs,intake.rs,owner.rs,encode.rs,contract_package.rs}, tests/it/checked_package_v2_check_mode.rs, tests/it/checked_package_v2_owners.rs, tests/it/complete_v1_contract_package.rs, tests/conformance_fspec/{check_mode_vectors.rs,selection_vectors.rs}, test builders and fixtures"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in, for filament-contract-ir PR #1034. Ticket: IR-1557. Reference: filament-specification origin/main 669da4fb (scratch worktree, contains c2e24844), FR-770, FR-322 "Function check mode", FR-330, TC-1860, `proposals/checked-package-v2/schema.json`, `check-mode-vectors.json` CM-01 to CM-04, `fixtures/adverse.json` check-mode entries.

Gate at 829e7c07 with FILAMENT_SPECIFICATION_DIR at that worktree: `cargo fmt --check` clean; `make string-edge` with the CI scanner (fsl-string-edge 01eb30e1) clean; `cargo clippy --locked --workspace --all-targets --features conformance-cli -- -D warnings` and `-p filament-contract-model` clean; `cargo deny check` ok; Cargo.lock unchanged by the diff; `quire validate` rc 0; `quire coverage --strict` 61 unbacked rows (baseline); `cargo test --locked --workspace --features conformance-cli --no-fail-fast` 2295 passed, 7 failed: the 2 `lock_coherence` environment failures plus 5 `string_edge_gate` tests that need the scanner on PATH, which pass (6/6) when rerun with it on PATH; `--test conformance_fspec -- --test-threads=1` 460 passed, 0 failed.

Mutation run (each applied alone, the check_mode, kernel-clause, vectors, diamond, adverse and imported-unit tests rerun): presence check removed is killed (AC-556, AC-557, CM vectors); kernel-clause check removed is killed (AC-559); imported-node comparison removed is killed by the unit test only; `CheckedSemanticNodeV2` encoder emission removed is killed (AC-555, CM vectors); V1 value validation removed is killed; reader collapsing kernel to linked is killed (AC-555, AC-559, diamond, CM vectors); the `CheckedNodeProjectionV2` encoder emission removed SURVIVES; the `check_mode` arm added to `classify_wire_fault` removed SURVIVES. Restoring the old storage window `732_400..733_100` in `a_denied_relationship_destination_read_is_refused_not_a_double_borrow` fails at 829e7c07, so the window move follows the larger package and the test keeps its assertions (legitimate, not weakened).

## Examined scope

- mod.rs `CheckMode` (examined): closed two-value enum, lowercase serde, `as_str`; public and listed in the FR-019 table.
- intake.rs `BorrowedNode.check_mode` decode and `RawSemanticNodeV2` carry (examined): the wire value reaches the typed node.
- owner.rs `validate_owner_schema` presence rule (examined): function kind iff `check_mode` present, else `malformed_wire` at `/semantic_graph/nodes/{i}/check_mode`. Matches FR-770 Behavior bullets 2 and 3 and CM-03/CM-04 loci. `null` reads as absent and refuses the same.
- owner.rs `validate_owner_joins` kernel rule (examined): reached only after the `Model` owner join succeeds; refuses `kernel` on an `invariant` or `precondition` clause function with `invalid_package`/`invalid-value` at the member; `body` clause, helper and declaration-less functions are not refused. Clause functions are only ever `Model`-owned (FR-322 owner join), so no other owner path needs the rule.
- mod.rs `imported_node_matches` (examined): compares `check_mode` with the other node members.
- encode.rs both encoders (examined): emit `check_mode` after `owner_model`, before `body`, only when present.
- contract_package.rs V1 node shape (examined): `check_mode` optional, value checked when present. The spec side of this is in the gap-analysis SR.
- Builders and fixture rewrites (examined): `carry_check_mode` and the four JSON fixtures add `linked` to function nodes only.
- No pins, compatibility layers or tool versions were added. Cargo.lock is unchanged.

## Verdict

Correct on the main path. The refusal codes and loci are the ones FR-770, CM-03/CM-04 and adverse.json state, and FR-770's clause-kind rule is implemented exactly. Two writer and intake edits are not pinned by any test (FND-001, FND-002). The imported-copy comparison is pinned by a unit test only (FND-003). Mergeable after FND-001. FND-002 and FND-003 are low.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `CheckedNodeProjectionV2` encoder's `check_mode` emission (and its `From` copy) has no test: removing it survives every test, so a public canonical encoder can drop an identity-bearing member unnoticed | crates/filament-contract-model/src/checked_package/v2/encode.rs:163-165 |
| FND-002 | low | The `"check_mode"` arm added to `classify_wire_fault` owner_shape is not exercised: removing it changes no test outcome, since the member is now a known field and every bad value already refuses `malformed_wire` at the member | crates/filament-contract-model/src/checked_package/v2/intake.rs:382 |
| FND-003 | low | Imported-copy `check_mode` comparison is covered only by a crate unit test; the diamond test passes with it removed (closure conflict comes from derived equality), and no integration read of a declaration-less imported function copy differing only in `check_mode` exists | crates/filament-contract-model/src/checked_package/v2/mod.rs:3432-3434 |
