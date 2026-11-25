---
id: SR-2026112568770457
title: "spec-review integrity of QSL-3138 contract-ir owner operand edits"
type: SpecReview
analysis: integrity
scope: "agent-ix/filament-contract-ir; PR #1176 at 3e6232a9; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/model/functional/FR-048-derive-structural-and-group-candidates.md"
review_set: subset
---

## Summary

Ticket: QSL-3138. Spec review with the integrity sub-analysis of the three edited spec files, checked against FSpec FR-322 at filament-specification origin/main 7de9a209 and against the reader. EARS grammar was not re-run, because no requirement statement changed form.

## Examined scope

- FR-038 path paragraph (examined): "a declared node's declaration path is its `declaration.qualified_name`, followed, for a `protocol_clause` with an `owner_model` edge, by one owner-model operand segment". Agrees with FSpec FR-322.
- FR-038 stale paragraph (examined): "If that segment is absent or differs from the edge, the reader SHALL refuse `invalid_package`/`stale-node-key` at `/semantic_graph/nodes/{i}/owner_model`". Agrees with FR-322 ("Any mismatch between the owner edge and the final path operand refuses ...").
- FR-038-AC-629 (examined): the "Known limitation" sentence is removed. Absent, retargeted and two-segment paths are stated.
- TC-048 step (examined), FR-048 Behavior (examined).

## Verdict

Two defects. AC-629 and TC-048 name a refusal for the dependency-position case that FSpec assigns otherwise. FR-048 keeps a sentence the edit contradicts.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-038-AC-629 and the TC-048 step say a final operand that is a dependency position refuses `invalid_package`/`stale-node-key`. FSpec FR-322-AC-521 refuses it earlier, as `malformed_wire`. A dependency position in the row of any node other than a `component_composition` refuses at `/semantic_graph/operand_rows/{i}/{r}` (NI-38). An own-`node_id` slot that names a dependency position refuses at the slot (NI-33). The reader follows FR-322 (node_index.rs:1410-1442). Fix: state `malformed_wire` at the row entry, or drop the clause from AC-629, TC-048 and the PR body's table. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6984; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1656-1663 |
| FND-002 | medium | FR-048 Behavior now says a declared node's path is followed, for an owned `protocol_clause`, by the owner-model operand, "including when several declared nodes ... share one recursion-group label". The same paragraph still says "No grouped path embeds operands." A grouped owned clause, and each anonymous member of a group whose sole declared member is that clause (which takes its path under FSpec FR-322), both hold an operand in their path. Fix: limit the sentence to the derived-operand rule (a grouped path is not derived from the member's own operands), or name the exception. | spec/model/functional/FR-048-derive-structural-and-group-candidates.md:46 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 7a4cda78 AC-629: "An owner operand slot that names a dependency position never reaches the owner comparison: the operand checks refuse it as `malformed_wire`, with no cause, at `/semantic_graph/nodes/{i}/node_id/path/{n}/operand` (FSpec FR-322-AC-521)". TC-048 is aligned. This matches FR-322-AC-521 NI-33 and node_index.rs step 2. |
| FND-002 | fixed | 7a4cda78 FR-048: "A grouped path embeds no derived operands. The one exception is the owner-model operand of an owned `protocol_clause`, held by that clause, grouped or not, and by each anonymous member that takes its path as the sole declared member's." FR-038 was aligned in the same way at both group-path sentences. |
