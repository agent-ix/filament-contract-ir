---
id: SR-17940486709564847048098
title: "code-review and rust-review of QSL-2635 reader join stage 5 (model declaration occurrences)"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1083 at 3585ced2 (merge base 312f3752); crates/filament-contract-model/src/checked_package/v2/{model_occurrence.rs,mod.rs}, tests/it/support/checked_package.rs, tests/conformance_fspec/{model_tuple_union.rs,clock_payload_tuple.rs,extension.rs}, tests/it/{checked_package_v2_reader.rs,checked_package_v2_whole_read_accounting.rs,complete_v1_checked_package.rs,checked_package_v2_owners.rs,checked_package_v2_operand_families.rs,checked_package_v2_refinement.rs,checked_package_v2_model_members/inherited_dispatch.rs}"
review_set: subset
---

## Summary

Code review, with the rust-review lane folded in, for filament-contract-ir PR #1083. Tickets: QSL-2635 and QSL-2619. Spec authority: FSpec origin/main 45b5e1b6 (descendant of 04109664) FR-322 "Model declaration occurrences", "One total order of the join stages" (FR-322-AC-275), FR-322-AC-485 and FR-322-AC-486; contract-ir FR-038-AC-582 (added here) and FR-019.

The PR adds `v2/model_occurrence.rs`, a private module called from `validate_graph` immediately after `validate_selected_type_bounds`. For every `model` or `relation` node that carries a `ModelOwner`, it requires exactly one `source_map` entry: role `generated`, ordinal 0, with regions equal to the `generated` ordinal-zero entry of the `model_root` that has the same identity. It refuses `invalid_package`/`invalid-value` at `/source_map/{i}` for the first failing entry, and otherwise at `/source_map` for the first node with no entry.

Gates. All runs were in the scratch worktree QSL-2635-review with its own target, serialized through `cargo-slot` (single slot), name-filtered, and with FILAMENT_SPECIFICATION_DIR set to a private FSpec worktree at 45b5e1b6:

- `rustfmt --check` on the changed files: clean.
- `quire validate` on the three changed spec files: no errors.
- `conformance_fspec -- model_tuple_union model_declaration_occurrence`: 13 of 13 pass, including `published_model_tuple_union_wire_cases_match_reader_refusals` (TW-29 to TW-34) and the new stage-5 test.
- `it --features conformance-cli` filtered to whole_read_accounting, TC-048 injected faults, TC-044 strict mutations, `checked_package_v2_owners`, the monitor-base fold, inherited dispatch and orphan history: 57 of 57 pass. This includes the `whole_read_accounting` TW-29 to TW-33 registry red named in the ticket.
- `conformance_fspec -- clock_payload_tuple extension`: 25 of 25 pass.
- Not run here, to respect thermal limits: full workspace, clippy, mutants. The known baseline reds were not re-measured.

Reviewer probes, run locally only and never pushed:

- P1, the stage-4 baseline. The TW-28 dangling `semantic_type` on node 14, applied alone, refuses `ill_typed`/`operator-ineligible` at `/semantic_graph/nodes/31`. That is a reader-stage refusal, not a stage-4 graph refusal.
- P2. The same dangling edge plus TW-29 refuses `invalid_package`/`invalid-value` at `/source_map`. Stage 5 is reported ahead of the stage-4 defect.
- P3. The same dangling edge plus a TW-32-style region refuses at `/source_map/3`.
- P4, a relation declaration node. In `positive-all-families.json`, the relationship `Order-order-Order` is node 7 and its entry is `/source_map/6`. Giving that entry its own region, or role `expression`, refuses at `/source_map/6`. Removing the entry refuses at `/source_map`. The unmodified base is admitted. The relation path is therefore correct, but no test covers it (see gap-analysis).

## Examined scope

- `model_occurrence.rs` `is_declaration`, `validate` (examined): the root table, the entry pass, the missing pass and the meter charges.
- `mod.rs` `validate_graph` call site at line 4107 (examined). This includes the ordering against the end-of-graph edge resolution at line 4458 and the `structural`/owner/derived-shape/selected-bound stages before it.
- `tests/it/support/checked_package.rs` (examined): `align_model_declarations`, `drop_a_non_model_source_entry`, `generate_model_declaration_occurrences`, `select_model`, `source_map_for`, and the calls from `refresh_identity`, `index_fixture_node_ids` and `rebuild_source_map`.
- Per-test edits (examined). The "incomplete source map" cases in TC-044, TC-048 and whole-read accounting now drop a non-model entry and still assert `invalid_source_map` at `/source_map`. The monitor-base and D10 producer fixtures, and the owners, refinement, clock-payload and extension readers, are aligned at load time.
- FR-019 public-item inventory (context_only). The module is private: `pub(super)` only, with no crate-root export. No inventory row is needed.

## Verdict

The stage is correct on the published vectors and on the relation path (P4). It is metered: one work unit per root entry, per declaration entry and per declaration node checked, with `stage_storage::vec` and `meter.filled` for its two tables. It is placed where it can be both after stages 2 and 3 and before every declaration, frame, state and operation refusal.

Stage 4 is a problem. The edge existence check is not implemented as a graph stage at all: P1 shows a dangling `semantic_type` reaching a reader-stage `ill_typed`. So no placement of stage 5 can satisfy "after stage 4". Moving stage 5 after the end-of-graph edge loop would break the other half of the spec sentence. The coder's placement is the right choice, but the deviation must be recorded and a stage-4 pass ticketed (FND-003).

The central fixture changes do not weaken other tests. Every "incomplete source map" case still removes a real correspondence and asserts the same code and locus. `align_model_declarations` only repairs model declaration entries, which the new rule makes mandatory. Two medium findings and two low ones.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The root-table loop stops once `roots.len() == model_roots`. A repeated `generated` ordinal-zero entry of one root therefore fills a slot, and a later root's entry is never collected. With two selected models whose first root entry is duplicated before the second root's entry, every declaration of the second identity finds `selecting == None` and is refused `invalid_package`/`invalid-value` at its own entry. Its regions do equal its root's, so stage 5 should pass it, and the duplicate root entry should be left to the later source-map stage. Collect at most one entry per root node (or skip identities already held) instead of counting to capacity | crates/filament-contract-model/src/checked_package/v2/model_occurrence.rs:54-72 |
| FND-002 | low | `roots.sort_by` is a stable merge sort. Above 20 elements it allocates a scratch buffer that is not charged to the read meter. FR-038-AC-582 says the pass's storage is exactly its two tables. The repo idiom is `sort_unstable` with a position tie-break (declaration_region.rs:38). Sort by `(identity, entry position)` with `sort_unstable_by` | crates/filament-contract-model/src/checked_package/v2/model_occurrence.rs:74-75 |
| FND-003 | medium | FSpec FR-322 requires stage 5 to run "after the edge existence check of stage 4". The reader has no stage-4 pass: dangling `semantic_type`/`dependencies` edges are resolved only in the end-of-graph loop (mod.rs:4458), and P1 shows a dangling `semantic_type` reaching `ill_typed` at `/semantic_graph/nodes/31`. A package with both a dangling stage-4 edge and a stage-5 defect therefore reports stage 5 (P2, P3) where FSpec requires the stage-4 refusal. The placement before declaration refusals is right. The fix is a stage-4 pass between `validate_selected_type_bounds` and `model_occurrence::validate`, for the stage-4 tags, either in this PR or as a ticketed follow-up named in the AC/matrix row | crates/filament-contract-model/src/checked_package/v2/mod.rs:4105-4107 |
| FND-004 | low | Test support is inconsistent about which nodes are model declarations. `align_model_declarations` and `generate_model_declaration_occurrences` filter on `node_tag` `model`/`relation`. `select_model` and `source_map_for` rewrite every node whose owner kind is `model`, which includes model-owned `function` (operation) nodes such as those in `positive-all-families.json`. Those nodes silently get the root's region in synthetic fixtures. Apply the same tag filter | tests/it/support/checked_package.rs:886-904,2996-3011 |
