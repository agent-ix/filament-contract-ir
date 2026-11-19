---
id: SR-2026111905212900000002
title: "gap-analysis of QSL-2496/QSL-2503 per-declaration claim names"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1058 at 31e982b2; spec/checked_package/functional/FR-371-read-claim-declaration-names.md (AC-1, AC-6, AC-10, AC-11), FR-038-AC-286, TC-477; tests/it/checked_package_v2_claim_names.rs, tests/it/checked_package_v2_temporal.rs, tests/it/checked_package_v2_protocol_body.rs, tests/conformance_fspec/claim_names.rs"
review_set: subset
---

## Summary

Ticket: QSL-2496. A manual check that each acceptance criterion is backed by a test. The Quoin matrix was not run for this pass, because the cargo-slot queue was saturated. The tags and assertions were read directly.

## Examined scope

- FR-371-AC-10 (examined). It is backed by `the_rows_of_one_declaration_share_its_name_and_are_no_duplicates`, by `a_name_on_two_nodes_resolves_to_the_node_whose_declaration_set_nests`, and by the CL-P7/CL-P8/CL-R5/CL-R16/CL-R17 replay with published `lookups`.
- FR-371-AC-11 (examined). It is backed by `a_hyper_clauses_body_wrapper_has_no_temporal_name_row_and_its_node_is_named` and by the CL-R18 replay.
- FR-038-AC-286 capacity 0 and negative capacity (examined). It is backed by `protocol_queue_admits_zero_and_positive_literal_capacity_and_round_trips` and `protocol_queue_rejects_negative_literal_capacity`.
- FR-371-AC-1 (examined). The twin-clause test asserts two rows with distinct names. It never looks up the second name, so it lets FND-001 of the code-review SR through.
- FR-371-AC-6 (examined): see FND-001.

## Verdict

One gap. The own-node work charge that FR-371-AC-6 now claims is asserted by no test. The FR-371-AC-1 twin lookup gap is the test side of the code-review high finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-371-AC-6 now claims the own-node check costs two visits per row plus one per declaration-set element compared. No test measures this charge: the existing work tests compare packages refused before the own-node pass, or packages whose own-node cost is the same, so dropping any of the three `stage_storage::work` calls in `own_nodes` would go unnoticed. | crates/filament-contract-model/src/checked_package/v2/claim_names.rs:409-455 |
| FND-002 | medium | The FR-371-AC-1 twin test (two structurally equal clauses `A` and `B` on one node) asserts the rows but never asserts `claim_name_for_path` for each name, so the wrong-row lookup (code-review SR FND-001) passes the suite. | tests/it/checked_package_v2_claim_names.rs:233-291 |

## Dispositions

Round 1, head f3ab09af.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open: the element charge is now pinned, but the per-row charge is not | `the_own_node_check_costs_the_declaration_set_elements_it_compares` pins the `inner + outer` element charge: the mutant that removes it is killed by that test. No test pins the two per-row own-node charges (node sets and grouping). Both packages in every work test have the same row count, and the fix commit says the per-row test "follows". The mutants that remove those charges were still running at the verdict. |
| FND-002 | fixed | The twin test now looks up both names with `claim_name_for_path` and asserts that each returns its own row. |
