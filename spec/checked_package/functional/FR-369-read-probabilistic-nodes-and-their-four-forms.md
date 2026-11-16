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

The strict V2 reader SHALL refuse a `probabilistic` node that carries a
`declaration`, an `owner` or a `recursion_group`, or that does not have exactly
one occurrence, with role `declaration`. FSpec FR-640 states the missing
`declaration` and the single `declaration` occurrence. It states that the node
has the derived-node path shape of FSpec FR-322; FR-322 gives a derived node no
`declaration`, `owner` or `recursion_group`, its owner-presence rule lists no
probabilistic form, and the reader treats the non-root `timed_model` forms the
same way. The `owner` and `recursion_group` refusals, and the reader's reuse of
the derived-node rule that `semantic_type` is the node's own id and
`dependencies` are its referenced targets, are therefore IR's reading of
FR-322, not FR-640 text.

The strict V2 reader SHALL refuse with `malformed_wire`, at the failing member,
a node whose body is not an aggregate of exactly the bindings FR-640 lists for
its form, in that order, each bound to the term kind FR-640 gives it: a missing
member, an extra member, a member out of order, a `reference` or literal of
another kind, or a damaged support or workload weight row.

The strict V2 reader SHALL refuse with `invalid_package` and cause
`invalid-value`, at the failing value, a well-shaped body that holds a value
outside the rules of FSpec FR-640 that need no other node:

- an empty support, a support value that repeats an earlier one, a support
  weight, `discrete` delay weight or `exponential` rate that is not above 0, a
  workload weight that is negative, and a weight or rate that is not an exact
  rational in lowest terms;
- a `family` outside `uniform`, `uniform_interval`, `exponential` and
  `discrete`, and `parameters` that do not hold exactly the members the family
  states, including a member row of `discrete` that does.

A workload weight of 0 is admitted. The reader SHALL charge one validation
visit of the caller's budget per node and per support, weight or delay member
before inspecting it, and SHALL stop at the first uncharged member with no
partial view.

A `node_tag` outside the closed vocabulary keeps the decoder's existing
`unsupported_node_tag` refusal.

### Planned

The strict reader SHALL also give the FR-640 checks that join other nodes, as
a planned follow-up that the planner assigns; this requirement is therefore
PARTIAL until they land. They are: rule 1 (an operation reference's `declaring`
node is present, local and a `model` node that names the operation), rule 2
(`parameter` names a parameter of the operation and each support value is of
its type), rule 3 (a workload omits, repeats or misnames no operation of its
model, and its members are in ascending order), rule 4 (a reward `value_type`
is an integer or quantity type, and its `expression` is a clause function of
the operation that reads no state), rule 5 (a delay distribution is on a `time
dense` model, and quantities are in the model's time unit), the
`conflicting-definition` refusals between two nodes, and the typed read of
FSpec FR-640 "Typed read". Until then a package that breaks those rules is
admitted. Owner: planner to assign.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-369-AC-1 | `probabilistic` and each of its four forms decode, `CheckedNodeKind::all` lists them, a node of each form (each delay family included) reads and re-encodes to the document it was read from, and the public reader admits a package holding a node of each form. | Test |
| FR-369-AC-2 | A probabilistic node with a missing member, an extra member, members out of order, a `declaration`, an `owner`, a `recursion_group`, no or two occurrences, an occurrence of another role, a binding of another term kind, or a short or extended support row or workload weights row refuses `malformed_wire` at the failing member. | Test |
| FR-369-AC-3 | An empty support, a repeated support value, a zero support or discrete weight, a zero or negative rate, a negative workload weight, a weight not in lowest terms, an unknown delay family and delay parameters that do not match the family refuse `invalid_package`/`invalid-value`; a zero workload weight is admitted. | Test |
| FR-369-AC-4 | An unknown `node_tag`, and a form outside the probabilistic family, still fail decode with the existing error. | Test |
| FR-369-AC-5 | The reader charges one work visit per node and per support, weight or delay member, and a budget one unit short is denied at the first uncharged member. | Test |

## Dependencies

- FR-038 owns strict admission; FSpec FR-640 owns the wire shapes.
