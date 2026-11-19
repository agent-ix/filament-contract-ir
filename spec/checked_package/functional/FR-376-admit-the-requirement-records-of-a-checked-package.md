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
Within the step it SHALL apply FSpec FR-825's rules 1 to 10 to each
`claim`/`value_validity` node in ascending structural node-id order, then
rules 11 to 13 to the rows in array order, then rule 14, refusing the first
failure with exactly the code, cause and pointer FR-825 gives. The owner
reachability of rules 4 and 8 SHALL walk each owner function's body once,
from its body root and not its measure, and reuse the reached set for every
record of that owner. The step SHALL charge the caller's reader budget as
FR-825 "Reader" states, before inspecting each charged item, and on
exhaustion return the reader's `incomplete` result at the first uncharged
member with no partial package.

The reader SHALL NOT compare a record node's path with its body, recompute
an extent, re-derive an owner or a guard, or refuse a package because it
lacks a record the producer's checker might have written (FSpec FR-825
"Reader").

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
    pub path_condition: Vec<RequirementGuard>,    // outermost first
    pub extent_roots: Vec<CheckedNodeId>,         // ascending node id
    pub objects: Vec<CheckedNodeId>,              // ascending node id
}
pub struct RequirementGuard {
    pub guard: CheckedNodeId,
    pub occurrence: u64,
    pub holds: bool,
}
```

- The rows SHALL be in the package's array order, which the reader has
  checked is FSpec FR-825's package order, so a row's index is its record's
  request index. Graph order orders nothing: two packages holding the same
  nodes in different graph orders return equal rows.
- Every id SHALL be in the resolved-graph form the admitted package exposes,
  and ids compare by structural key (FSpec FR-322 "Rebuilding the key").
- `record` is `Some` exactly for a `value-validity` row; every field is read
  from the admitted node's body in FR-825's binding order, `conversion`
  `None` for the empty aggregate.
- `value_validity_records_of` returns the empty slice for a function that
  owns no record and for a node that is not a function. A package with no
  `requirement_items` returns the empty slice from both reads.
- Both reads SHALL allocate nothing per call and return what admission
  retained.

The writer SHALL write `requirement_items` and the record nodes exactly as
admitted, so a read of the writer's output returns equal rows in equal order.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-376-AC-1 | The reader admits FSpec TC-2053's packages of FR-825-AC-1 to FR-825-AC-3. For FR-825-AC-1's package `requirement_items()` returns two rows, both `value-validity`, `Bounded`, keyed at the `+` node's `expression` occurrences 0 and 1, with records whose `owner` is `f`, `result_bound` the `Int[0, 10]` and `Int[0, 20]` nodes, `conversion` `Some` of each narrow and its ordinal, `path_condition` `[]` and `[{guard: the g(x + 1) node, occurrence, holds: true}]`, `extent_roots` `[x]` and `objects` `[]`; `value_validity_records_of(f)` returns `[0, 1]` and `value_validity_records_of(g)` the empty slice. For FR-825-AC-3's package the `inc` row's index equals its array position, `Bounded`'s and `BelowThree`'s rows have `record` `None`. The writer's output read back returns equal rows in equal order, and the same package with its nodes in another graph order returns equal rows. | Test (TC-488) |
| FR-376-AC-2 | FSpec FR-825-AC-4's and FR-825-AC-5's mutations, and FR-322-AC-473's packages, each refuse with FR-825's or FR-322's exact code, cause and pointer, observed separately through the public reader, and none admits a partial package; an unknown `claim` form still refuses at `semantic_form`. | Test (TC-488) |
| FR-376-AC-3 | The requirement-record step runs after `probabilistic_join::validate` and before `checked_term_data` validation: a package with a probabilistic rule-6 defect and a record defect refuses at the probabilistic defect, and one with a record defect and a reversed `probabilistic_claims` row refuses at the record defect; a reader budget one visit short of FR-825-AC-1's package's charged count returns `incomplete` at the first uncharged member, and an owner with two records is walked once (its reach visits are charged once). | Test (TC-488) |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns the V2 reader, writer,
  identity and shared refusal machinery.
- `ix://agent-ix/filament-specification/FR-825` owns the rows, the node body,
  the reader rules and their refusals; `ix://agent-ix/filament-specification/FR-322`
  owns the enclosing package and the check order.

## Status

Planned. The current reader refuses `claim`/`value_validity` at its closed
form gate and `requirement_items` as an unknown top-level member.
