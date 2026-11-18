---
id: FR-064
title: "Bind each checked protocol clause to its exact model owner"
type: FR
relationships:
  - target: ix://agent-ix/filament-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/filament-contract-ir/FR-051
    type: depends_on
  - target: ix://agent-ix/filament-contract-ir/FR-061
    type: references
  - target: ix://agent-ix/filament-specification/FR-322-AC-130
    type: references
  - target: ix://agent-ix/filament-specification/FR-375-AC-23
    type: references
  - target: ix://agent-ix/filament-specification/FR-464-AC-16
    type: references
  - target: ix://agent-ix/filament-specification/FR-464-AC-20
    type: references
  - target: ix://agent-ix/filament-specification/FR-727
    type: references
  - target: ix://agent-ix/filament-spec-language/FR-880-AC-7
    type: references
---

# FR-064: Bind each checked protocol clause to its exact model owner

## Description

When a source clause is declared over model `M`, its producer SHALL set
`owner_model` to the object-type node of `M`, the object type of the clause's
`over` input, a model owned by a selected declaration. The Contract IR model and
typed writer SHALL carry the supplied local `NodeRef`. The strict reader SHALL
require it to name a selected local model declaration, or a source-declared object type under [FR-363](./FR-363-join-protocol-owner-model-to-source-declared-type.md). The reader uses the edge
as the authoritative clause-to-model ownership for refinement class catalogs.
IR preserves the FSpec FR-322 and FR-464 wire, graph, identity and protocol
reader order. The reader SHALL NOT infer or cross-check source ownership from
the `over` parameter's type, the clause name, or a refinement step row.

A clause whose `over` type reaches no object type is ownerless: it SHALL carry
no `owner_model` edge, and the reader SHALL admit it without one (FSpec
FR-727, FSL FR-880-AC-7). The `over` type *reaches* object type `T` when it is
`T` itself, a reference to `T`, or an `option`, `sequence`, `set` or `bag`
whose element is `T` or a reference to `T`. A scalar, a bounded domain, a
record, a tuple, or an `option` or collection of those reaches none. A
protocol over a job number or a request record is a common shape; requiring
an owner for it would force the producer to invent one, such as a model that
one of its roles names, and would add the clause to a refinement class
catalog it does not belong to. An absent edge states that the clause is over
no model object. An ownerless clause contributes no class to any model's
refinement class catalog; its roles and relationships still name their own
model nodes, and its `over` binder, stage reads and slots are the same as for
an owned clause. The reader reads the `over` type only to decide whether the
edge must be present; when it is present, its target is checked as below and
never compared with the `over` type.

## Inputs

A typed V2 graph supplied to the public encoder of
[FR-051](./FR-051-encode-checked-package-v2-envelope.md), or untrusted V2
bytes, selections, evidence and limits supplied to the strict reader of
[FR-038](./FR-038-consume-checked-package-v2.md). The authoritative edge
shape and structural id are FSpec FR-322, FR-464 and the published
`proposals/checked-package-v2/schema.json` and
`schema.json` at the selected FSpec source.

## Outputs

Canonical bytes and an admitted typed graph retaining the owner edge and
identities, or one typed, located refusal for the first failing check, with
no partial admitted package.

## Behavior

The public `CheckedSemanticNodeV2` SHALL carry `owner_model` as a typed local
node reference, required exactly when `node_tag` is `protocol`,
`semantic_form` is `protocol_clause` and the clause's `over` type reaches an
object type, and absent on every other node, an ownerless clause included. Its raw
reader form and typed graph SHALL retain the same edge. The
public V2 writer SHALL encode the edge once in the graph node, without adding another envelope or a
second graph. The reader SHALL retain the admitted edge through typed
read/write/read. The edge is distinct from the existing structural `owner`
member and from `semantic_type`.

For a clause, the closed node SHALL carry `owner_model` as a local structural node id; other application nodes omit this edge. Its exact local dependencies include the edge target and every body target once each in structural node-id order. The reader validates edge shape, selected owner, dependency membership and body semantics without computing or comparing a content-derived key. A valid edge retargeting changes the complete body but can retain the same structural id.

The source producer SHALL set `owner_model` to the exact local object-type
node of the model that the clause's source `over` type names, which is owned by a
selected declaration. The public typed
encoder is a byte-construction API: it retains the supplied edge and has no
source declaration with which to check this producer obligation.

The reader SHALL apply FSpec's closed wire presence and `NodeRef` shape
checks before graph or meaning checks. An absent `owner_model` on a clause
whose `over` type reaches an object type, a present edge on a clause whose
`over` type reaches none, or a present edge on another node, SHALL refuse as
`malformed_wire` at the
offending node/member; an external package key in the local-only edge SHALL
also refuse under the wire shape. The graph-shape stage SHALL join a
model/object-type target to its selected declaration at the target's own
`node_id`, refusing `missing_declaration`/`missing-selection` there if it
cannot join. It SHALL require the target in exact dependencies, refusing
`invalid_semantic_graph` at `/semantic_graph/nodes/{n}/dependencies` before
key and body checks. After the earlier wire, graph, key, declaration, state,
refinement-step and protocol member checks, the clause step SHALL check the
typed edge first. A target naming no local node SHALL refuse
`missing_declaration`/`missing-name` at `owner_model`; a local target that is
not a selected object-type declaration SHALL refuse `invalid_model_binding`/
`malformed-declaration` there, before the same clause's activation, capture or
slot checks. The reader SHALL use the admitted edge without a source-level
`over` comparison; no general key stage SHALL invent or repair the edge.

The refinement's concrete `semantic_type` SHALL resolve to the canonical
`model_root` of its selected concrete model package. That root identifies the
selected package as a whole; it is not an object-type declaration node. For
each admitted clause, the reader SHALL resolve `owner_model` through the
selected declaration and its package to an exact object-type node owned by
that declaration. The refinement class-catalog graph join SHALL select every
admitted clause whose resolved owner belongs to the refinement's selected
concrete declaration, including a clause with no named step row, and SHALL
exclude clauses whose edge names a model owned by another declaration or an
unselected package. It SHALL NOT compare the root node id directly to the
owner node id, treat a matching name as ownership, or accept a selected
declaration's non-object-type node as an owner.
It SHALL compare the complete selected model class inventory under
[FR-038](./FR-038-consume-checked-package-v2.md) and FSpec FR-375. A
refinement-step row/catalog defect SHALL precede the protocol step; a
protocol-body defect SHALL precede the later class-catalog graph join. A
catalog made complete only by a foreign clause or by an inferred `over` type
refuses under the existing class-catalog rules, with no synthesized
class or default owner.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-064-AC-1 | An external consumer constructs a complete typed V2 package with two selected local model declarations and one protocol clause whose `owner_model` names the exact model over which it was declared, calls `encode_checked_package_v2`, and reads the bytes through the public strict reader. The admitted typed graph retains the edge as a local `NodeRef`, exact dependencies include it, the occurrence-free projection includes it, and canonical read/write/read retains the same structural clause id and package name. | Test |
| FR-064-AC-2 | Withdrawn by the IR-1141 structural-identity decision under FSpec #517: the reader does not recompute identity from body content or refuse a retained structural id solely because a valid body changed. This id is retained and never reused. | Withdrawn |
| FR-064-AC-3 | A missing `owner_model` on a protocol clause whose `over` type reaches an object type (FR-064-AC-8 states those cases), a present `owner_model` on a non-clause node, or an external `{package, node}` value in the local-only edge refuses at closed wire validation before graph or protocol checks. A clause whose otherwise valid local edge target is omitted from, duplicated in, or out of order in `dependencies` refuses `invalid_semantic_graph` at `/semantic_graph/nodes/{n}/dependencies` before clause-meaning checks. | Test |
| FR-064-AC-4 | With wire, graph and structural ids valid, a genuinely unselected `model`/`object_type` target refuses `missing_declaration`/`missing-selection` at the target's own `node_id` in the owner join before any body step. A local edge naming no node refuses `missing_declaration`/`missing-name` at `owner_model` in the clause step. A non-model node, a source-local declaration that is not a source-declared object type ([FR-363](./FR-363-join-protocol-owner-model-to-source-declared-type.md) admits that one), a selected declaration node of another form (`state_machine`, `process`, `record_value_type` or a relation node), or the selected package's `model_root` refuses `invalid_model_binding`/`malformed-declaration` at `owner_model` as the clause's first check. An earlier wire, graph, refinement-step or protocol member defect wins when paired with a typed edge defect; the typed edge defect wins over the same clause's activation, capture or slot defect. A source producer given a clause declared `over` model A emits the edge to A's object-type node even when selected model B exists; a candidate B edge fails producer conformance, without claiming an IR reader source-level comparison. The IR reader admits a selected B edge as a changed complete clause body under its structural id and does not infer or cross-check it from the `over` parameter. | Test |
| FR-064-AC-5 | A protocol refinement whose concrete `semantic_type` is its selected package's canonical `model_root` and whose complete concrete catalog includes classes from an admitted clause whose distinct `owner_model` is the object-type node of a model owned by its selected concrete declaration is admitted even when no step row names those classes. A clause whose edge names a model owned by another declaration or an unselected package contributes no class to that catalog, regardless of similar declaration names or binder types. A catalog missing an owned clause's class or including a foreign clause's class refuses under FR-038-AC-325 after protocol-body checks; an earlier refinement-step row/catalog defect still wins. | Test |
| FR-064-AC-6 | The public typed writer and reader carry `owner_model` through `CheckedSemanticNodeV2`, its raw form and typed graph in the existing V2 graph and envelope. No other node accepts the member, and no copied FSpec schema, fixture, parallel untyped writer, compatibility reader or fallback owner inference is added. | Inspection |
| FR-064-AC-7 | Protocol clauses whose `over` binder has type `Boolean`, `Int[0, 3]` (a `bounded_domain` `integer_range` node), a model record `Parts::Handoff`, a tuple, `Option<Integer>` and `Sequence<Integer>` are each admitted with no `owner_model` member and with exact dependencies that hold no model object-type node (FSpec FR-727-AC-1, FSL FR-880-AC-7). A model refinement whose package also holds an owned clause over `M::T` reconstructs a class catalog that includes only the owned clause (FSpec FR-727-AC-3). | Test |
| FR-064-AC-8 | A clause over `M::T`, over `Option<M::T>` and over `Sequence<M::T>`, each with its `owner_model` edge removed, refuses `malformed_wire` at the clause node before graph or protocol checks; with the edge to `M::T`'s object-type node each is admitted, and with the edge naming another selected object type `M::U` each is still admitted, the edge being authoritative and not compared with `over` (FSpec FR-727-AC-2). Each ownerless clause of FR-064-AC-7 with an added `owner_model` edge refuses `malformed_wire` at the clause node (FSpec FR-727-AC-1). | Test |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns strict V2 admission,
  check order, identity and refinement class-catalog graph joins.
- [FR-051](./FR-051-encode-checked-package-v2-envelope.md) owns the public
  typed writer; [FR-061](./FR-061-admit-refinement-nodes.md) owns the closed
  refinement node. Its Behavior "Keys, graph and identity" and AC-1/AC-3/AC-7
  require the same canonical selected `model_root` for `semantic_type` and
  `abstract`; the clause's object-type `owner_model` remains distinct.
- FSpec FR-322-AC-130, FR-375-AC-23 and FR-464-AC-16 own the published
  wire and structural node shapes, exact owner relation and catalog semantics.
- FSpec FR-727 owns the ownerless clause whose `over` type reaches no object
  type, and the element object type an `option` or collection `over` names.

## Status

Partially implemented. The typed `owner_model` carrier, writer,
selected-declaration owner join and class-catalog reconstruction are present.
The source producer's choice of edge from the source `over` type still needs
independent producer-conformance evidence; the IR byte-construction API cannot
make that source-level comparison. The reader admits an ownerless clause and
refuses one whose `over` binder's type is itself an object type; deciding
reachability through a reference, `option` or collection element, and
refusing an edge on a clause whose `over` type reaches no object type, are not
yet implemented.
