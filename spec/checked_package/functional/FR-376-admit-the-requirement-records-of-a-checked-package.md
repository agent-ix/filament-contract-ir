---
id: FR-376
title: "Admit the requirement records of a checked package"
type: FR
relationships:
  - target: ix://agent-ix/filament-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/filament-specification/FR-825
    type: references
  - target: ix://agent-ix/filament-specification/FR-322
    type: references
---

# FR-376: Admit the requirement records of a checked package

## Description

When a V2 checked package carries `requirement_items` rows or
`claim`/`value_validity` nodes, the Contract IR reader SHALL admit exactly
FSpec FR-825's rows, node body and joins, or refuse the whole package at the
first defect in FR-825's reader order. The canonical writer SHALL retain the
admitted rows and nodes unchanged. An admitted package SHALL expose the
records through one typed read, so a consumer that holds only the package
bytes (FSL's package route for `call.requirements`) reads a called function's
requirement items, their request indices and every fact of each item from
the admitted package and never re-walks the graph.

## Inputs

The untrusted `filament.checked-package/v2` bytes, evidence and limits of
[FR-038](./FR-038-consume-checked-package-v2.md). The rows, the node form and
the reader rules are specified by
`ix://agent-ix/filament-specification/FR-825`, and the identity, graph, check
order and refusal ordering by `ix://agent-ix/filament-specification/FR-322`.

## Outputs

An admitted package retaining every row and record node, with their exact
bodies, dependencies, Boolean type and `generated` occurrence; or one typed
refusal with code, cause when defined and RFC 6901 pointer. No partial
package.

## Behavior

The reader SHALL add `value_validity` to the closed `claim` form vocabulary
and `requirement_items` to the closed top-level members, while continuing to
refuse every unknown `claim` form as `invalid_semantic_graph` at
`/semantic_graph/nodes/{n}/semantic_form` and every unknown top-level member
as `malformed_wire`.

The reader SHALL run FR-322's strict-wire, graph-shape and structural-id
checks, the owner joins, the `operation_effects` and `operation_bodies`
carriers and FSpec FR-640's probabilistic step before the requirement-record
step, and the `checked_term_data` step after it (FSpec FR-322 "Check
order"). In this crate that step SHALL run in `v2/mod.rs` after
`probabilistic_join::validate` and before `checked_term_data` validation.
Within the step it SHALL validate FSpec FR-825's `path_condition_frames`
table first; then its `claim`/`value_validity` nodes in ascending structural
node-id order; then its `requirement_items` rows in array order; then record
coverage and generated ordinals. It SHALL refuse the first failure with
exactly the code, cause and pointer FR-825 gives. One owner-body walk from its
body root (not its measure) SHALL derive all scalar-operation path conditions
for that owner, validate each frame's parent and required branch polarity,
and compare every record's terminal frame id with the independently derived
chain for the record's application occurrence. A mismatch is refused; the
reader SHALL NOT trust the emitter's table without this comparison. The reader
SHALL charge the caller's reader budget before each FR-825 validation item.
When the budget is exhausted, the reader SHALL return `incomplete` at the
first uncharged member, with no partial package.

The reader SHALL compare the record node's identity terminal frame id with
its body binding and SHALL derive/validate the path-condition chain as FR-825
requires. It SHALL NOT recompute an extent, re-derive an owner, or refuse a
package because it lacks a record the producer's checker might have written.

### Public requirement rows

An admitted package SHALL expose its requirement records through
`CheckedPackageV2::requirement_items(&self) -> &[RequirementItemRow]` and
`CheckedPackageV2::value_validity_records_of(&self, owner: &CheckedNodeId)
-> &[usize]`, the indices into `requirement_items()` of the rows whose
record's owner is `owner`, ascending:

```rust
pub struct RequirementItemRow {
    pub claim: CheckedNodeId,          // the node the record is keyed at
    pub occurrence: Occurrence,        // its role and ordinal
    pub kind: CapabilityKind,          // FSpec FR-290 kind
    pub extent: RequirementExtent,     // Bounded | Unbounded(Vec<(DomainKey, DomainKind)>)
    pub record: Option<ValueValidityRecord>,
}
pub struct ValueValidityRecord {
    pub node: CheckedNodeId,           // the claim/value_validity node
    pub owner: CheckedNodeId,          // the owning function node
    pub application: CheckedNodeId,
    pub occurrence: u64,               // the application's expression ordinal
    pub result_bound: CheckedNodeId,   // the bound's type node
    pub conversion: Option<(CheckedNodeId, u64)>, // narrow node and its ordinal
    pub path_condition_frame: Option<PathConditionFrameId>, // terminal frame id
    pub extent_roots: Vec<CheckedNodeId>,         // ascending node id
    pub objects: Vec<CheckedNodeId>,              // ascending node id
}
pub struct PathConditionFrameId(pub usize); // package-local row index
pub struct PathConditionFrame {
    pub parent: Option<PathConditionFrameId>,
    pub guard: CheckedNodeId,
    pub occurrence: u64,
    pub holds: bool,
}
pub struct PathConditionView<'a> {
    // Borrowed from the admitted package; iterates outermost first.
}
```

`CheckedPackageV2::path_condition_frames(&self) -> &[PathConditionFrame]`
returns the admitted shared table. `CheckedPackageV2::path_condition(&self,
record: &ValueValidityRecord) -> PathConditionView<'_>` returns a borrowed
view over that table and the record's optional terminal id. The view iterates
`RequirementGuard { guard, occurrence, holds }` values outermost first without
allocating or storing a per-record vector. An empty terminal id gives an empty
view. Frame ids are package-local indices and cannot be used with another
admitted package.

- The rows SHALL be in the package's array order, which the reader has
  checked is FSpec FR-825's package order, so a row's index is its record's
  request index. Graph order orders nothing: two packages holding the same
  nodes in different graph orders return equal rows.
- Every id SHALL be in the resolved-graph form the admitted package exposes,
  and ids compare by structural key (FSpec FR-322 "Rebuilding the key").
- `record` is `Some` exactly for a `value-validity` row; every field is read
  from the admitted node's body in FR-825's binding order, `conversion`
  `None` for the empty aggregate, and `path_condition_frame` is `None` for its
  empty aggregate or the sole terminal frame id otherwise.
- `value_validity_records_of` returns the empty slice for a function that
  owns no record and for a node that is not a function. A package with no
  `requirement_items` returns the empty slice from both reads.
- The rows, frames and path-condition views SHALL allocate nothing per read
  call and return only references/indices into what admission retained.
- Retained storage for all requirement rows, value-validity records and
  path-condition frames SHALL be O(R + G), where R is the number of records
  and G is the number of unique frame rows. Admission's owner-body traversal
  SHALL run once per owner and SHALL NOT retain a full path vector per record.

The writer SHALL write `requirement_items`, `path_condition_frames` and the
record nodes exactly as admitted, so a read of the writer's output returns
equal rows, frames and path-condition views in equal order.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-376-AC-1 | The reader admits TC-2053's packages of FR-825-AC-1 to AC-3. For AC-1, `requirement_items()` returns two rows keyed at the `+` node's expression occurrences 0 and 1, both `value-validity` and bounded; their records retain `f`, their separate bounds and conversions, roots `[x]` and no objects. The first record has no terminal frame; the second has the terminal id of the `g(x + 1)` guard frame, whose view yields that guard with holds true. `path_condition_frames()` returns the shared table and `path_condition(record)` returns a borrowed outermost-first view. `value_validity_records_of(f)` returns `[0, 1]` and the read for `g` is empty. The writer round trip and a package with graph nodes in another order yield equal rows, frames and views. AC-3's non-value-validity rows have no record; an empty package returns empty rows, frames and owner indices. | Test (TC-488) |
| FR-376-AC-2 | Each refusing mutation of FSpec FR-825-AC-4 and FR-825-AC-5, and each refusing package of FR-322-AC-473 (the two check-order packages and the `claim_names` row naming a `value_validity` node), refuses with FR-825's or FR-322's exact code, cause and pointer, observed separately through the public reader, and none admits a partial package; the admitted cases of those ACs are FR-376-AC-1's and FR-376-AC-3's; an unknown `claim` form still refuses at `semantic_form`. | Test (TC-488) |
| FR-376-AC-3 | The requirement-record step runs after `probabilistic_join::validate` and before `checked_term_data` validation: a package with a probabilistic defect and a record defect refuses at the probabilistic defect, and one with a record defect and a reversed probabilistic row refuses at the record defect. The reader charges each frame, record and owner-body node once; a budget one validation visit short returns `incomplete` at the first uncharged item and no partial package. An owner with multiple records is walked once. | Test (TC-488) |
| FR-376-AC-4 | Admit a package with 1,000 value-validity records under a 1,000-deep nested guard body and 1,000 shared condition frames. The retained requirement-record, frame and view storage is at most 1,000 record entries plus 1,000 frame entries and O(1) indexes per entry (linear in records plus unique guards); the measured reader storage charge for those records is linear in the frame and record counts, and no record retains a 1,000-element guard vector. | Test, Analysis (TC-488) |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns the V2 reader, writer,
  identity and shared refusal machinery.
- `ix://agent-ix/filament-specification/FR-825` owns the rows, the node body,
  the reader rules and their refusals; `ix://agent-ix/filament-specification/FR-322`
  owns the enclosing package and the check order.

## Status

Planned. The current reader refuses `claim`/`value_validity` at its closed
form gate and `requirement_items` as an unknown top-level member.
