---
id: SR-17940486709564847048111
title: "code-review of QSL-2726 PR #1099 (CertificateMember 51 to 61 schema names)"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR #1099 at 764127e4; crates/filament-contract-model/src/certificate_member.rs"
review_set: subset
---

## Summary

Ticket: QSL-2726. One file changes. The `certificate_members!` table gains ten variants (`non_zeno`, `conditional`, `denominator`, `ef_nodes`, `form`, `mode`, `numerator`, `post`, `reset`, `state_graph`) and a new unit test `schema_added_members_spell_their_names`. The rust-review lane is folded in here.

## Examined scope

- `certificate_members!` table (examined). The 61 spellings, in table order, equal the FSpec `$defs.CertificateRejected.oneOf[15].properties.at.properties.member.enum` sequence at FSPEC-main-ci cd89fcb3 element by element, and also equal the FR-331 "closed 61-name set" list in order. Checked by script.
- Macro (context_only). One table generates the enum, `ALL` and `as_str`, so a variant cannot be missing from `ALL`.
- Other exhaustive users (examined). No `match` on `CertificateMember` and no variant list exists anywhere else in the repo. `tests/it/public_interface.rs` lists only the owner, `ALL` and `as_str`, so it needs no edit.
- Gates run (examined), flock on /tmp/cargo-slot.lock, CARGO_TARGET_DIR in the worktree. `conformance_fspec certificate_member` with FILAMENT_SPECIFICATION_DIR=FSPEC-main-ci: 2 passed, `model_certificate_members_match_fspec_schema` among them. Model lib `certificate_member`: 1 passed. `it public_interface` with `--features conformance-cli`: 25 passed. Clippy on the model crate with all targets is clean, and `cargo fmt --check` is clean.
- New unit test (examined). It is not fully tautological. It checks that each added variant appears exactly once in `ALL`, that `ALL.len()` is 61, and that near-miss spellings are absent. The `(variant, literal)` pairs restate the same table, though. The real oracle is the conformance test, which this unit test duplicates in part.

## Verdict

PASS on correctness. The ordered vocabulary matches the schema exactly, and the conformance test is green. Two low findings are about the new test: its trace binding and its maintenance cost. Neither blocks merge.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | `schema_added_members_spell_their_names` is tagged FR-070-AC-3, the ordered-sequence AC, but asserts no order. It checks membership, uniqueness and length only, so the tag inflates AC-3 coverage. Its doc comment names only AC-4. | crates/filament-contract-model/src/certificate_member.rs:97-98 |
| FND-002 | low | The test copies the literal spellings from the same table and hard-codes `ALL.len() == 61`. Both repeat the conformance oracle, so every future schema add must edit this test too. The doc comment's "after the original 51" history will go stale. | crates/filament-contract-model/src/certificate_member.rs:97-124 |
