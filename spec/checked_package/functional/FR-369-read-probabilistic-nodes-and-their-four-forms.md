---
id: FR-369
title: "Read the FR-640 probabilistic node tag and its four forms"
type: FR
relationships:
  - target: ix://agent-ix/filament-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/filament-specification/FR-640
    type: references
  - target: ix://agent-ix/filament-specification/FR-322
    type: references
---

# FR-369: Read the FR-640 probabilistic node tag and its four forms

## Description

The closed node vocabulary SHALL carry the `probabilistic` node tag and its four
semantic forms `random_parameter`, `workload`, `reward` and `delay_distribution`
of FSpec FR-640, and the strict V2 reader SHALL admit a node of each form only
in the closed record and ordered body bindings FR-640 states.

## Inputs

A `filament.checked-package/v2` document with `probabilistic` nodes, supplied to
the reader of [FR-038](./FR-038-consume-checked-package-v2.md).

## Outputs

The admitted package, or the typed located refusal.

## Behavior

A `probabilistic` node SHALL carry no `declaration`, no `owner` and no
`recursion_group`, and exactly one occurrence, with role `declaration`. Its body
SHALL be an aggregate of exactly the bindings FSpec FR-640 lists for its form,
in that order, each of the term kind FR-640 gives it. A missing member, an extra
member, a member out of order or a member of another term kind refuses
`malformed_wire` at the failing member.

A body that is well shaped but holds a value outside the rules that need no
other node refuses `invalid_package` with cause `invalid-value` at that value:
an empty support, a support, `discrete` delay or `exponential` rate weight that
is not above 0, an exact weight or rate that is not an exact rational, and a
`family` outside `uniform`, `uniform_interval`, `exponential` and `discrete`. A
workload weight of 0 is admitted. The reader charges one validation visit per
node and per support, weight or delay member before inspecting it.

A `node_tag` outside the closed vocabulary keeps the decoder's existing
`unsupported_node_tag` refusal.

The checks of FR-640 that join other nodes (operation references, parameter
types, reward functions, duplicates across nodes, workload order and the typed
read) are not part of this requirement.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-369-AC-1 | `probabilistic` and each of its four forms decode, `CheckedNodeKind::all` lists them, and a node of each form (each delay family included) reads and re-encodes to the document it was read from. | Test |
| FR-369-AC-2 | A probabilistic node with a missing member, an extra member, members out of order, a `declaration`, or a short or extended nested support, weight or delay row refuses `malformed_wire` at the failing member. | Test |
| FR-369-AC-3 | An empty support, a zero support or discrete weight, a zero exponential rate and an unknown delay family refuse `invalid_package`/`invalid-value`; a zero workload weight is admitted. | Test |
| FR-369-AC-4 | An unknown `node_tag`, and a form outside the probabilistic family, still fail decode with the existing error. | Test |

## Dependencies

- FR-038 owns strict admission; FSpec FR-640 owns the wire shapes.
