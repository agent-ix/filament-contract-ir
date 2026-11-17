---
id: SR-17940486709564847048039
title: "gap-analysis of IR-1557 function node check_mode"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1034 at 829e7c07; FR-038-AC-555 to FR-038-AC-560, FSpec FR-770, crates/filament-contract-model/src/checked_package/v2/{owner.rs,contract_package.rs}, tests/conformance_fspec/check_mode_vectors.rs, tests/it/{checked_package_v2_check_mode.rs,complete_v1_contract_package.rs}"
review_set: subset
---

## Summary

Gap analysis scoped to the PR diff. Ticket: IR-1557. Plan completion: not assessed. FSpec reference: filament-specification origin/main 669da4fb. Each new AC was traced to a tagged test, and each test was checked by mutation (see the code-review SR). `quire coverage --scope . --strict` reports 61 unbacked rows (baseline). It also reports that `check_mode_vectors_read_as_published` traces to `TC-1860`, `FR-770-AC-1` and `FR-770-AC-2`, which match no declared row.

FSpec search for the V1 question: `grep -rl` over FSpec `spec/` and `proposals/` for `check_mode`, `ContractPackageWireV1` and `contract-package/v1`. FR-770, FR-322, TC-1860 and the proposal files mention `check_mode`. FR-330 is the only spec of `filament.contract-package/v1`. It never mentions `check_mode`, and FSpec has no V1 schema. FR-330 Properties say the V1 `semantic_graph` "node array uses FR-322's closed node-tag vocabulary" and that its joins "retain FR-322's validation and identity rules". FR-330 Dependencies say FR-322 "owns the admitted input's closed node ... forms reused here". FR-770 is scoped to "every `function` node of a `filament.checked-package/v2` package". No vector covers a V1 package. Locally, FR-035 and TC-047 say nothing about `check_mode`.

## Examined scope

- FR-038-AC-555 (examined): `check_mode_is_typed_identity_bearing_and_written`, killed by the encoder and reader-collapse mutants.
- FR-038-AC-556 (examined): `a_function_node_without_a_valid_check_mode_refuses_malformed_wire`, killed by the presence mutant.
- FR-038-AC-557 (examined): `check_mode_on_a_non_function_node_refuses_malformed_wire`, killed by the presence mutant.
- FR-038-AC-558 (examined): `a_diamond_whose_leaves_differ_only_in_function_check_mode_conflicts` (TC-226) and the unit test `an_imported_function_node_matches_only_at_the_selected_check_mode`.
- FR-038-AC-559 (examined): `kernel_is_refused_on_invariant_and_precondition_clause_functions_only`, killed by the kernel-clause mutant.
- FR-038-AC-560 (examined): no test carries the tag.
- adverse.json check-mode-unknown-value/empty/not-a-string/null (examined): run by `tc_048_fspec_adverse_mutations_refuse_as_recorded`, none on the expected-failure list.
- FSpec FR-330 V1 node forms (context_only).

## Verdict

All six ACs are implemented, and the five with tests fail when their code is removed. Two trace defects and one undocumented behaviour change remain. FND-002 is the V1 open question for the planner.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-560 has no tagged test: the CM vectors test traces FSpec ids (TC-1860, FR-770-AC-1/2) that match no local row instead of TC-048/FR-038-AC-560 | tests/conformance_fspec/check_mode_vectors.rs:48 |
| FND-002 | medium | The V1 ContractPackage reader now admits an optional `check_mode` on any node, but no local or FSpec requirement states it. FR-330 reuses FR-322's closed node forms, which would make it required on function nodes and forbidden elsewhere, so "optional on any node" is invented. Planner ruling needed. | crates/filament-contract-model/src/checked_package/v2/contract_package.rs:2450-2454 |
| FND-003 | low | `v1_package_carries_function_check_mode_and_refuses_an_invalid_value` is tagged FR-038-AC-555 (the V2 reader/writer AC) under TC-047; it tests V1 lowering/reading, which no AC owns, and asserts only the cause, not the refusal code | tests/it/complete_v1_contract_package.rs:123-125 |
| FND-004 | low | The AC-555 test sets `kernel` on one declared function only, though AC-555 names a declared function and a declared helper; the two-node case is only in CM-02, whose test is not tagged AC-555 | tests/it/checked_package_v2_check_mode.rs:79-87 |
