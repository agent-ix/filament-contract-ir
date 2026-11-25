---
id: SR-2026112569086487
title: "code-review of QSL-3138 contract-ir unconditional owner operand"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR #1176 at 3e6232a9 (base fa9ed76e); crates/filament-contract-model/src/checked_package/v2/{mod.rs,protocol_body.rs,monitor_plan/reached_model_read.rs}, tests/it/*, tests/fixtures/qsl-2645-golden/package.json"
review_set: subset
---

## Summary

Ticket: QSL-3138. Code review with the rust-review lane, read only. Nothing was compiled or run for this review. Basis: FSpec FR-322 "Protocol clause owner identity", FR-322-AC-538 and FR-322-AC-521 (NI-33, NI-38) at filament-specification origin/main 7de9a209.

## Examined scope

- The owner check in `validate_protocol_nodes` (protocol_body.rs:7139-7199). It runs in this order: missing target `missing_declaration`, kind `invalid_model_binding`, selection or owner recovery, then one `node_work` charge, then `stale-node-key` when the path holds no owner operand or the operand names another node. Every earlier refusal keeps its precedence, and the work charge for an admitted owned clause is unchanged.
- `validate_declaration_names` (mod.rs:3651-3683): a final segment that is a dependency position passes only when `owner_model` is present.
- Golden package: clause node 9 and source_map entry 9 both carry `{"node": 21}`. Node 21 is `Order`, the clause's `owner_model` and dependency. No other id in the package names the clause. `source_package()` now builds exactly this form, and `settle` rebuilds the source map. typed-read.json holds no clause id, so it does not change. The equality test decides; nothing suggests the file was only hand-edited to pass.
- Fixture builders (frame_entries, refinement, source_objects, monitor_plan, `add_minimal_protocol_named`) now add the operand. Every refusing case at an existing `owner_model` site still refuses at an earlier step (missing, kind or selection) before the stale check. The FSpec published fixtures already carry the operand.
- Compile reading: the imports (`admitted_dependency`, `read_with_dependencies`, `v2_nominal`) exist in tests/it/support, the removed `clause_path_strings` has no remaining caller, and `resolve_node_held` exists. I predict no compile error.
- PR body and diff checked for private text: none.

## Verdict

Not mergeable. The unconditional check is right for local owners and refuses no valid package in the corpus. The new dependency-position test asserts a refusal the reader never reaches, and an ownerless clause with an operand is still admitted.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `protocol_owner_dependency_position_operand_refuses` will fail. The reader's operand-row pass refuses a dependency-position row entry in any row that is not a `correspondence`/`component_composition` row, as `malformed_wire` at `/semantic_graph/operand_rows/{i}/{r}`. That pass runs before graph validation (node_index.rs:1410-1442, FSpec FR-322-AC-521 NI-38). So the clause's `{"package", "node"}` entry is refused there, and the test's `invalid_package`/`stale-node-key` assertion fails. The new `Dependency` arms in `validate_declaration_names` and `owner_path_operand` cannot be reached for a protocol clause. Fix: assert `MalformedWire` at the row-entry pointer, or drop the case and the dead arms. | tests/it/checked_package_v2_protocol_body.rs:3435-3462; crates/filament-contract-model/src/checked_package/v2/mod.rs:3667-3672; crates/filament-contract-model/src/checked_package/v2/protocol_body.rs:72-79; crates/filament-contract-model/src/checked_package/v2/node_index.rs:1410-1442 |
| FND-002 | medium | An ownerless `protocol_clause` whose path is its declaration path plus one local operand is admitted. `has_protocol_owner_operand` accepts an `Index`/`Node` final segment for every protocol clause, whatever its `owner_model`, and the owner check runs only when `owner_model` is `Some`. FSpec FR-322 says "A `protocol_clause` without `owner_model` retains its exact `declaration.qualified_name` path". This PR's own FR-038 sentence says a declared path "that carries any other segment, refuses at its `node_id`". No test covers the case. Fix: require `node.owner_model.is_some()` for the `Index`/`Node` arm too, so the case refuses `declaration-nominal-mismatch`, and add the case (for example over `Boolean`). | crates/filament-contract-model/src/checked_package/v2/mod.rs:3662-3678 |
| FND-003 | low | `source_package()` and `concrete_protocol_refinement_with_family` add the owner operand by editing the clause's `node_id` in place. They do not call `reassign_node_id`, as frame_entries.rs and reached_model_read.rs do. This is correct only while nothing else names the clause before `settle`. A later builder that adds a reference to the clause first would leave a stale id. Fix: use `reassign_node_id`, as the other builders do. | tests/it/checked_package_v2_source_objects.rs:289-296; tests/it/checked_package_v2_refinement.rs:6953-6958 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 7a4cda78: the dead `Dependency` arms are removed from `validate_declaration_names` and `owner_path_operand`. The test now sets the clause's row entry at its own `node_id` slot to `{"package", "node"}` on the wire form, and expects `MalformedWire`, no cause, at `/semantic_graph/nodes/{clause}/node_id/path/2/operand`. The coder's pointer is right and the FND's (row entry) was not: node_index.rs step 2 ("Placement") refuses a `node_id` slot that names a dependency position before step 3 checks row entries, as FSpec FR-322-AC-521 NI-33/NI-39 state. Body slots do not trigger step 2, so the `node_id` site is the one reported. |
| FND-002 | rejected | not a defect. protocol_body.rs:7063 (`owner_model.is_none() && owner_path_operand(node).is_some()`, on main since #1173) refuses this case as `invalid_package`/`declaration-nominal-mismatch` at `/semantic_graph/nodes/{i}/node_id/path`, and the loop visits every protocol clause, so no path admits it. The review missed that check. Refusing it in `validate_declaration_names` instead would turn the presence refusal (`malformed_wire` at the clause, which FSL's "the edge is missing" case asserts) into a nominal mismatch, so keeping the check where it is is right. |
| FND-003 | fixed | 7a4cda78: `source_package()` and `concrete_protocol_refinement_with_family` now rekey each clause through `reassign_node_id(&mut package, &unowned, &owned)`. |

## New findings (disposition pass 3)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | high | Merging this head turns main CI red. `.github/workflows/ci.yml` checks out FSpec at 01f8dd31 (#807), which predates FSpec #843. At that ref, `positive-all-families.json` and its siblings carry the owned `Handshake` clause on its declaration path alone, `["Example", "Handshake"]` with `owner_model`. That form was admitted under IR-1614; this PR's unconditional check now refuses it `invalid_package`/`stale-node-key`. `make test` runs every target with `--include-ignored` and `FILAMENT_SPECIFICATION_DIR` set, so each FSpec conformance test that needs those fixtures admitted is predicted to fail. That includes `fspec_object_owned_clause_admits_on_its_owner_operand_path` and the AC-112 adverse base. Moving the ref to FSpec main 7de9a209 does not help: there those fixtures refuse `invalid_source_map` (see the dispositions below). Fix: first an FSpec PR that rewrites the clause's source_map entry in the eight #843 fixtures to the operand form. Then move this PR's ci.yml FSpec ref to that commit. | .github/workflows/ci.yml:14-19; Makefile (test target); tests/it/checked_package_v2_protocol_body.rs (fspec_object_owned_clause_admits_on_its_owner_operand_path) |

## Dispositions (pass 3)

| FND | outcome | reason |
| --- | --- | --- |
| FND-004 | still-open | New this pass. Closes only when CI's FSpec ref names a commit whose owned-clause fixtures carry the operand in both the node and its source_map entry. |

## Dispositions (pass 4)

| FND | outcome | reason |
| --- | --- | --- |
| FND-004 | fixed | de8f4136: commit 78b03dbf moves the ci.yml FSpec ref to a6968fb8 (FSpec #846). At that ref all eight owned-clause fixtures hold the operand in the clause node id and in its source_map entry: positive-all-families source_map[43] `["Example", "Handshake", {"node": 44}]` with `operand_rows[9][3]` = 44, and likewise in the other seven. `make test` does not build `conformance_fspec` (`test = false`). The FSpec-reading `it` and lib groups that CI runs report 327/0 and 41/0, and the five C2 failures are the read-budget reds already on main. So the merge adds no CI red. |
