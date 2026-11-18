---
id: SR-17940486709564847048069
title: "gap-analysis of QSL-2410 claim_names reader"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1046 at 11029d36; FR-371-AC-1..6, TC-477, FSpec FR-322-AC-457, FR-322-AC-458, FR-322-AC-459, FR-322-AC-200; crates/filament-contract-model/src/checked_package/v2/{claim_names.rs,intake.rs,refinement.rs}, tests/it/checked_package_v2_claim_names.rs, tests/conformance_fspec/main.rs"
review_set: subset
---

## Summary

Gap analysis scoped to the PR diff. Ticket: QSL-2410 (admit `claim_names` and the ClaimNameRow reader rules: dangling, duplicate and uncovered names refuse, and a refinement row's name equals `body.name`, plus vectors). Plan completion: not assessed. Requirement source: filament-specification origin/main f16e1701, exported fresh and read only.

Reviewer mutation run: 33 mutants, each built and run on its own, with the source restored after each and the worktree verified clean at the end. Each mutant ran first against `--test it claim_names`. Survivors were rerun against all 1188 `checked_package_v2` tests, with FILAMENT_SPECIFICATION_DIR set, and compared with that suite's 5 baseline failures at head.

Killed (29):

- M01-M04: the schema checks on name, empty path, path segments and row role.
- M05: the kind filter is removed.
- M06: temporal clauses count as claim-bearing everywhere.
- M07: the occurrence ownership check.
- M08: the last path segment check.
- M09: the refinement name check.
- M10: the refinement check is skipped when the body has no name.
- M11 and M15: the duplicate key check and its cause.
- M12: the order check.
- M13 and M14: the duplicate name check and its code.
- M16: coverage.
- M17: the coverage role filter.
- M18-M20: each work charge.
- M21-M23: each claim-bearing arm.
- M25: the `validate` call.
- M26: envelope encode.
- M29: term_view name decode.
- M30: body name encode.
- M31: lookup length.
- M32: the wire-fault arm.

This agrees with the coder's 21-mutant claim.

Surviving the whole checked_package_v2 suite (4):

- M24: the refusal's node locus dropped.
- M27: the `check_lossless_source` comparison of `claim_names` removed.
- M28: refinement.rs `decode_body` always yields `name: None`.
- M33: the row join also accepts a `temporal` node.

## Examined scope

- FR-371-AC-1 / FR-322-AC-457 (examined): admission, rows per occurrence, two equal clauses on one node with two rows, no member without claim-bearing nodes, lookup, write/read round trip. See FND-004.
- FR-371-AC-2 / FR-322-AC-458 schema half (examined): 9 damaged-row cases at exact pointers. `[]` and `null` refuse `malformed_wire` at `/claim_names` (reviewer probe; no test pins them).
- FR-371-AC-3 / FR-322-AC-458 join half (examined): dangling, wrong-kind, missing occurrence, wrong role, last segment, refinement name with present, renamed and absent body names. See FND-001.
- FR-371-AC-4 (examined): duplicate name at the later row's `qualified_name`, repeated key `conflicting-definition` at the row, descent `invalid-value` at the later row.
- FR-371-AC-5 (examined): first and last row removed, member removed, row defect wins over coverage, and a claim node with declaration and claim occurrences.
- FR-371-AC-6 (examined): exact per-row, per-scan and per-walk work deltas measured by bisection of the smallest deciding work limit.
- FR-322-AC-459 (examined): no requirement, test or code (FND-002 of the code-review SR; FND-001 of the spec-review SR).
- FR-322-AC-200 (examined): partial (FND-006 of the code-review SR).
- Test-side derivation in conformance_fspec, `fspec_inputs::read_json` and the temporal_binding_public fixture (examined). It is honest, but it has no owner and no removal ticket (FND-005). Whole-read accounting misses one path (FND-001 of the code-review SR).
- Trace tags (examined): all 17 new tests carry `#[trace("TC-477", "FR-371-AC-n")]`. FR-371 and TC-477 do not collide with any id in this repository or in FSpec's spec tree; FSpec FR-370 is real, and FR-371 has no FSpec file.

## Verdict

Coverage of FR-322-AC-457/458 is strong: every reader rule has a killing test and exact pointers. The gaps are as follows:

- One explicit AC clause is untested: the row naming a `temporal_clause` node.
- One production guard is untested: the lossless compare of the member.
- A locus that TC-477 claims to check is never asserted.
- The AC-457 example omits its state-graph claim.
- FSpec ships no vectors, and no ticket owns that.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-322-AC-458 / FR-371-AC-3 "a row whose `declaration` names ... a `temporal_clause` node refuses at `/declaration`" has no test. The "wrong kind" case uses the first non-claim-bearing node of the state fixture, not a temporal clause, and the step listed in TC-477 ("Point a row at ... a temporal clause") is not implemented. Mutant M33, where the row join also admits `temporal` nodes, survives all 1188 checked_package_v2 tests | tests/it/checked_package_v2_claim_names.rs:369-389; crates/filament-contract-model/src/checked_package/v2/claim_names.rs:144-152 |
| FND-002 | medium | The production lossless check of `claim_names` (`compare_optional_rows(root, "claim_names", ..)`) has no test. Removing it (M27) leaves every checked_package_v2 test green, so a raw member that differs from its typed re-encoding is never shown to refuse | crates/filament-contract-model/src/checked_package/v2/intake.rs:690 |
| FND-003 | low | TC-477 Expected Results claims each refusal carries "the structural node locus of the row's declaration". No test asserts `refusal.locus`; `assert_refusal` checks only code, cause and `loci[0].path`. M24 (drop `.at_node(declaration)`) survives. Separately, the oracle decoder refinement.rs `decode_body` reads `name` but the lib differential fixture (term_view.rs:2085) carries none, so M28 survives the it suite | tests/it/checked_package_v2_claim_names.rs:105-121; spec/checked_package/matrix/TC-477-claim-declaration-names-reader.md:36-38 |
| FND-004 | low | FR-371-AC-1 and FR-322-AC-457 name a state-graph claim `ReachThree` (`possible`). The test renames a precondition state clause to `ReachThree`; the package has no state-graph claim node, so the AC example is not reproduced. Claim-node rows are covered only through the step-relation claim test | tests/it/checked_package_v2_claim_names.rs:41-72 |
| FND-005 | medium | FSpec publishes no `claim_names` vectors (TC-2011) and no fixture rows. 8 of the 20 published fixture packages at f16e1701 hold claim-bearing nodes and no member, so the strict reader refuses them, and three test harnesses derive synthetic rows instead. The derivation is honest (FR-322 makes the member mandatory), but nothing records an FSpec ticket to publish the rows and vectors, or a removal ticket for the shims. The ticket's "plus vectors" deliverable is therefore not met upstream | tests/conformance_fspec/main.rs:263-279; tests/it/support/fspec_inputs.rs:25-30; crates/filament-contract-model/tests/temporal_binding_public.rs:138-154 |
