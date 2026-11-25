---
id: SR-2026112558271996
title: "gap-analysis of QSL-3138 contract-ir unconditional owner operand"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1176 at 3e6232a9; FR-038-AC-629, crates/filament-contract-model/src/checked_package/v2/{mod.rs,protocol_body.rs}"
review_set: subset
---

## Summary

Ticket: QSL-3138. Gap analysis scoped to the PR diff. Plan completion: not assessed. The criterion-to-test check was done by hand; `quire matrix` was not run (not run for this review).

## Examined scope

- FR-038-AC-629 (examined): "has as its `node_id.path` the declaration path followed by exactly one owner-model operand segment, and is admitted when that segment resolves to the same `M` ... Retargeting ... refuses `invalid_package`/`stale-node-key` ... Updating the path segment to resolve to `N` admits. The same refusal applies when `node_id.path` is the declaration path alone, with the edge unchanged or retargeted, and when the final operand is a dependency position ... A path with two or more segments past the declaration path refuses `invalid_package`/`declaration-nominal-mismatch`".
  - Admit, stale retarget, updated path admits: `protocol_owner_retarget_with_stale_key_path_refuses_and_updated_path_admits`, with literal code, cause, pointer and locus.
  - Declaration path alone after a retarget: the same test (`bare`). With the edge unchanged: `protocol_owner_declaration_only_path_refuses`.
  - Dependency position: a test exists but asserts a refusal the reader cannot reach (SR-2026112569086487 FND-001).
  - Two segments: the existing test at protocol_body.rs:3540-3558.
  - The FSpec published owned clause: `fspec_object_owned_clause_admits_on_its_owner_operand_path` (only with FILAMENT_SPECIFICATION_DIR).
- FSL FR-383-AC-27 for a package the producer writes: the FSL side (PR #3920) carries it. Nothing in this repository exercises an FSL-emitted package.

## Verdict

AC-629 is covered except for the dependency-position branch (wrong oracle). One reader branch has no criterion or test.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No criterion or test covers an ownerless `protocol_clause` whose path carries an operand. FR-038 (prose) says a declared path "that carries any other segment, refuses at its `node_id`", and FSpec FR-322 says such a clause "retains its exact `declaration.qualified_name` path". AC-629 states only the owned case, and no test reads an ownerless clause with an operand. The reader admits it (SR-2026112569086487 FND-002), and nothing would catch that. Fix: add the ownerless case to AC-629 or a sibling AC, with a test that expects `declaration-nominal-mismatch` at `/semantic_graph/nodes/{i}/node_id/path`. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:665-667; crates/filament-contract-model/src/checked_package/v2/mod.rs:3662-3678 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 7a4cda78: AC-629 gains "A `protocol_clause` without `owner_model` keeps its declaration path: one with an operand after that path refuses `invalid_package`/`declaration-nominal-mismatch` at `/semantic_graph/nodes/{i}/node_id/path`", and TC-048 gains the matching step. `protocol_ownerless_clause_with_an_operand_refuses` first asserts the ownerless families package admits, then adds `{"node": boolean}` through `reassign_node_id`. It asserts the literal code, cause and pointer, and the refusal it reaches is protocol_body.rs:7063. The package is not settled after the rekey; nothing in it orders ids by the clause key (the source map follows graph order), so the case still reaches that refusal. |
