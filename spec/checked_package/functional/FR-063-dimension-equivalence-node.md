---
id: FR-063
title: "Encode and admit checked-package dimension equivalence nodes"
type: FR
relationships:
  - target: ix://agent-ix/filament-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/filament-contract-ir/FR-051
    type: depends_on
  - target: ix://agent-ix/filament-specification/FR-322-AC-51
    type: references
  - target: ix://agent-ix/filament-specification/FR-322-AC-52
    type: references
---
# FR-063: Encode and admit checked-package dimension equivalence nodes

## Description

When a checked-package V2 graph contains a `scalar_type` node with semantic
form `dimension_equivalence`, the Contract IR typed writer SHALL encode its
complete FSpec structural id and graph members, and the strict reader SHALL
admit it only under FSpec FR-322's dimension-equivalence and owner rules. For an invalid equivalence, the strict
reader SHALL refuse the package at the first applicable FSpec FR-322 wire,
graph or semantic check, with no partial admitted package. IR
neither infers an equivalence from names nor rewrites a producer's pair or
scale.

## Inputs

The typed V2 graph supplied to the encoder of
[FR-051](./FR-051-encode-checked-package-v2-envelope.md), or untrusted V2
bytes, selected package evidence, lock and limits supplied to the strict
reader of [FR-038](./FR-038-consume-checked-package-v2.md). The authoritative
form is FSpec `ix://agent-ix/filament-specification/FR-322` and the
published `proposals/checked-package-v2/schema.json`; IR carries no schema
copy.

## Outputs

Canonical checked-package V2 bytes retaining the typed equivalence node, or
an admitted package retaining that node and its occurrence, or one typed,
located refusal with no partial admitted package.

## Behavior

The typed V2 vocabulary SHALL include `dimension_equivalence` as a closed
`scalar_type` form with the closed body `{term: "dimension_equivalence", dimension_node_ids, root_scale}`. The node carries one top-level `SourceOwner`, not an owner inside its body. `dimension_node_ids` contains exactly two dimension node ids and `root_scale` is a positive rational. The writer preserves the supplied graph, structural id and
occurrence through canonical encoding under [FR-051](./FR-051-encode-checked-package-v2-envelope.md);
the reader retains the admitted node in its typed graph.

The reader SHALL require the pair to name distinct `scalar_type`/`dimension`
nodes in structural-id order, with no second equivalence node for that pair.

The node SHALL be self-typed, have no `declaration`, and have a `dependencies`
list containing only locally declared endpoint `NodeId`s, in the order in
which those endpoints occur in `dimension_node_ids`; the list is empty when
both endpoints are `Package` operands. Foreign endpoints are represented and
validated solely through the ordered `dimension_node_ids` `Package`
references and selected-model evidence. The node SHALL have one `anchor`
occurrence with its source-order ordinal among that source's equivalence
declarations; the top-level source owner joins the lock. Reversing the source
declaration's operand spelling preserves the normalized pair and scale. A
valid pair or scale edit changes the retained body without causing a stale-id
refusal.

The reader SHALL validate the authored structural id for shape and uniqueness
without deriving the path from the pair or scale. The path on this
declaration-free node is a producer-assigned identity label; declared-path
equality applies only where a declaration exists.

The closed V2 wire schema SHALL require `root_scale` with positive-integer
`numerator` and `denominator` strings, exactly two `dimension_node_ids`, and
no `declaration` member. A missing scale, zero or negative scale component,
one-entry pair or present declaration SHALL refuse `malformed_wire` at the
node before graph checks. This wire-shape classification is the default for a
missing or non-positive `root_scale`; a positive, schema-valid value proceeds
to graph admission, where the reader SHALL require `root_scale` to be reduced,
with the exact ratio determined by the two base
roots. For equivalences
between derived dimensions, it SHALL require equal normalized maps and
the ratio fixed by those base roots. Across a connected equivalence graph,
it SHALL require every cycle's root-scale product to equal one exactly.
It SHALL apply FSpec FR-322's wire, graph and semantic refusal
order; the encoder is a byte-construction API and does not substitute for
reader admission.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-063-AC-1 | PLANNED/UNRUN (IR-1141; FSpec #529). An external consumer encodes a typed dimension_equivalence node with the closed three-member body and top-level SourceOwner, then reads it through the public strict V2 reader. The admitted package retains the structural id, exact pair, positive reduced scale, self semantic_type, absent declaration, owner and one anchor occurrence. For two local endpoints, `dependencies` contains both local endpoint `NodeId`s in `dimension_node_ids` order; for two selected-model `Package` endpoints it is `[]`, and the foreign endpoints are validated through their ordered `dimension_node_ids` references and selected-model evidence. Canonical re-encoding retains the same complete node. | Test |
| FR-063-AC-3 | PLANNED/UNRUN (IR-1141; FSpec FR-322-AC-268, XP-01). Three independently authored equivalence packages retain endpoint pairs in ascending canonical node-key order: `(clock_time, flow_time)`, `(clock_time, rpc_time)` and `(flow_time, rpc_time)`, with `root_scale` respectively `1/1000`, `1/1000` and `1/1`. For the second pair, the source equation `1 R::s = 1000 G::ms` therefore stores `root_scale` `1/1000` when `clock_time` precedes `rpc_time`; `root_scale` is the ratio of the canonical root of the first ordered endpoint to that of the second. Reversing the authored equation yields the same ordered pair and stored ratio. One canonical root of the first endpoint equals the stored ratio times the canonical root of the second. The reader admits their consistent three-package graph as one equivalence class with anchor ordinals in source declaration order; a selected dependency reference resolves to its named package and structural node id, never to a same-path local node. As an additive control, three equivalence nodes in one selected package graph with the same consistent ratios also admit as one class. Withdrawn hash-based TC-233 MU-22 through MU-24 are not evidence for either current structural control. | Test |
| FR-063-AC-4 | PLANNED/UNRUN (IR-1141; FSpec #529). Each independently authored adverse pair or graph is refused `invalid_semantic_graph` at its equivalence node: descending ids, equal ids, a unit id, a repeated pair, unequal derived-dimension normalized maps, a scale inconsistent with base roots, or a cycle whose exact scale product differs from one. No malformed node is skipped or treated as an unknown form. | Test |
| FR-063-AC-5 | PLANNED/UNRUN (IR-1141; FSpec #529). Missing or invalid scale, one-entry pair, missing top-level SourceOwner and present declaration refuse `malformed_wire` at the node before graph checks. Unreduced scale, missing dependency or non-self semantic_type refuses `invalid_semantic_graph` at its exact node. A well-formed SourceOwner absent from `lock.sources` refuses `missing_declaration`/`missing-selection` at the node's `node_id` in the owner join before equivalence-body checks. No stale-key refusal or vector hash comparison occurs. | Test |
| FR-063-AC-6 | The writer and reader use the typed dimension_equivalence form in the V2 graph and envelope. Unknown scalar forms remain closed; no untyped alternate path or copied FSpec schema or fixture is added. | Inspection |
| FR-063-AC-7 | PLANNED/UNRUN (QSL-3120; FSpec FR-322). With each other package member valid, a strict-reader input with the `root_scale` member omitted, or with either `numerator` or `denominator` a zero or negative integer string, is refused as `malformed_wire` at the equivalence node during wire validation, before graph checks. This is the wire-shape default: a missing or non-positive scale component is not reclassified as a semantic graph error. | Test (TC-048) |
| FR-063-AC-8 | PLANNED/UNRUN (QSL-3120; FSpec FR-322). With a schema-valid positive scale `numerator: "2000"`, `denominator: "2"`, the strict reader admits the wire shape, then refuses the unreduced value as `invalid_semantic_graph` at that equivalence node during graph admission, with no partially admitted package. | Test (TC-048) |
| FR-063-AC-9 | PLANNED/UNRUN (QSL-3120; FSpec FR-322-AC-268). The strict reader receives three base-dimension pairs in ascending endpoint order: A–B with `root_scale` `1/1`, B–C with `1/1`, and A–C with `999/1`. The equivalence-node identities sort by node id as E-AB (`path: ["01-ab"]`), E-BC (`["02-bc"]`), then E-AC (`["03-ac"]`); each edge otherwise has valid endpoints, ownership and occurrences. Traversing A→B→C→A multiplies `1 × 1 × 1/999 = 1/999`, not one. The reader refuses `invalid_semantic_graph` with no cause at E-AC, the later equivalence node in node-id order, and returns no admitted package. | Test (TC-048) |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns strict admission,
  structural identity and refusal precedence.
- [FR-051](./FR-051-encode-checked-package-v2-envelope.md) owns the typed
  envelope writer.
- FSpec FR-322's dimension-equivalence narrative owns the pair, scale and graph
  meaning. FSpec #529 proposes top-level SourceOwner presence and a closed
  body; FR-322-AC-51 explicitly admits SourceOwner on this declaration-free
  form, and FR-322-AC-52 owns the owner join.
  FSpec FR-322-AC-268 and its XP-01..XP-10 vectors in the #530 draft specify
  cross-package equivalence; the three-package FR-063-AC-3 obligation remains live.
  The former FR-322-AC-118/139
  and TC-233 equivalence vectors are withdrawn; IR's controls above require
  fresh, independently authored fixtures and claim no coverage from those vectors.

## Status

Planned. The current IR typed V2 vocabulary does not contain this form.
