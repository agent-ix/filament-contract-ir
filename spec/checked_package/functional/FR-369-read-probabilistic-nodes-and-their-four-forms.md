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

A workload weight of 0 is admitted. The strict V2 reader SHALL charge one
validation visit of the caller's budget for each node and for each support,
weight or delay member, SHALL make that charge ahead of inspecting the member,
and SHALL stop at the first uncharged member with no partial view.

A `node_tag` outside the closed vocabulary keeps the decoder's existing
`unsupported_node_tag` refusal.

When the owner join has admitted a package, the strict V2 reader SHALL check each `probabilistic`
node in ascending node-id order against the nodes it joins and refuse the first
failure with `invalid_package` and cause `invalid-value`, at the failing member:

- an operation reference whose `declaring` node is not a `model` node of an
  object type, process or state machine of a selected model, or which names no
  operation that node declares (a node absent from the graph is already
  refused by the graph's dependency check);
- a `random_parameter` whose `parameter` names no parameter of the operation,
  or holds a support value whose literal kind is not that of the parameter's
  type, the type FSpec FR-322 gives the parameter (an integer or integer-range
  type is `integer`, a quantity or carried-rational type `quantity`, a native
  or declared rational `rational`, text `text`, a declared enumeration `enum`),
  or, for a quantity type, whose literal is not typed at the parameter's unit;
- a `workload` whose `model` is not a `model` node, whose weights are not in
  strictly ascending order of (`declaring` node id, `operation` name), whose
  row names an operation that is neither the model's own nor inherited through
  its supertypes, followed transitively, or repeats an operation name another row
  already names, or that omits an operation the model
  declares or inherits;
- a `reward` whose `value_type` is none of an `integer`, a `carried_quantity`,
  a `bounded_quantity` or a unit (`Quantity<u>`, `Duration`) type node, a `compound_unit` node (a
  quantity over a compound unit), or an
  `integer_range` node (`Int[lo, hi]`), the type nodes FSpec FR-322 gives an
  integer or quantity type, or whose `expression` is not a `function` node
  owned by the model operation member, that lists exactly the operation's
  parameters in declared order in its `parameters` binding, and whose own type
  is the `value_type`;
- a `reward` whose `expression` reads anything but the operation's parameters
  and constants, as FSpec FR-405 states it (a reward "reads only the
  operation's parameters, random parameters included ... reads no state");
- a `delay_distribution` whose `declaring` model is not the model of a
  `timed_model` root whose time source has kind `dense`.

The strict V2 reader SHALL decide the read of a reward by following the
dependencies of its function through functions, values and the expressions
`reference`, `call`, `unary`, `binary`, `conditional`, `case`, `let`,
`quantify`, `collection` and `conversion`; it SHALL admit in them a parameter
the function lists, a constant and a type node, and SHALL refuse a node of any
other kind, among them every `state` node, the `pre_read`, `presence_read`,
`value_read`, `deref`, `query` and `reachability` expressions, a `model` node
and a parameter the function does not list.

The strict V2 reader SHALL charge, ahead of the work and as the validation visits of the
caller's budget, one visit for each `probabilistic` node, one for each
workload row, one for each supertype edge followed to find a workload's
inherited operations, one for each node the walk of a reward's function
reaches, and one for each node of the graph scanned for the root of a delay's
model, so that a bound too small for the join stops the read as incomplete
rather than being skipped.

The reader SHALL refuse with `invalid_package` and cause `conflicting-definition`,
at the later node, a second `random_parameter` of one operation parameter, a
second `workload` of one name, a second `reward` of one name on one operation
and a second `delay_distribution` of one operation.

The admitted package SHALL give, by the operation or model node, each
`random_parameter` with its support and the exact probability of each value (the
weight over the sum of the weights, in lowest terms), each `reward` with its
`value_type` and function, each `workload` with its weights as exact rationals,
and each `delay_distribution` with its family and, for the delays, bounds and
rates, their exact rationals and the unit node of each time quantity, each
keyed by its node.

The rules read the node's own bindings, so the position of each refusal is the
member of the node that fails.

### Planned

The checks of FSpec FR-640 that need more than the nodes joined above are not yet
made, and a package that breaks them is admitted: that a support value of a
bounded or declared parameter type lies inside that type (the literal kind and,
for a quantity, its unit are checked, not the value's range), that a delay's
quantities are in the time unit of the model's `dense` source (the reader
exposes each quantity's unit but does not compare it), and that a `reward`'s
function body computes a value of `value_type` (its own type is checked, not
its expression). A support value of a declared record, tuple, variant, option or
collection type has no literal kind and is refused. Owner: titan:IR:code.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-369-AC-1 | `probabilistic` and each of its four forms decode, `CheckedNodeKind::all` lists them, a node of each form (each delay family included) reads and re-encodes to the document it was read from, and the public reader admits a package holding a node of each form. | Test |
| FR-369-AC-2 | A probabilistic node with a missing member, an extra member, members out of order, a `declaration`, an `owner`, a `recursion_group`, no or two occurrences, an occurrence of another role, a binding of another term kind, or a short or extended support row or workload weights row refuses `malformed_wire` at the failing member. | Test |
| FR-369-AC-3 | An empty support, a repeated support value, a zero support or discrete weight, a zero or negative rate, a negative workload weight, a weight not in lowest terms, an unknown delay family and delay parameters that do not match the family refuse `invalid_package`/`invalid-value`; a zero workload weight is admitted. | Test |
| FR-369-AC-4 | An unknown `node_tag`, and a form outside the probabilistic family, still fail decode with the existing error. | Test |
| FR-369-AC-5 | The reader charges one work unit for each node and for each row of a support, a workload weights list or a discrete delay list, and a budget one unit short of the total is denied at the first uncharged row. | Test |
| FR-369-AC-6 | An operation reference of each form whose `declaring` node is no `model` node, or which names an operation that node does not declare, refuses `invalid_package`/`invalid-value` at the member; a package whose references all name declared operations is admitted. | Test |
| FR-369-AC-7 | A `random_parameter` whose `parameter` is not a parameter of the operation, or whose support value has another literal kind than the parameter's type, refuses `invalid_package`/`invalid-value` at the member. | Test |
| FR-369-AC-8 | A `workload` that omits an operation of its model, repeats one (also through another declaring model), lists its rows out of order, names an operation the declaring node lacks, or names a `model` that is no model node refuses `invalid_package`/`invalid-value` at the member; a workload covering every operation in order is admitted. | Test |
| FR-369-AC-9 | A `reward` whose `value_type` is a Boolean type, whose `expression` function is owned by another operation, lists no parameters or the operation's parameters in another count or order, or has another type than `value_type` refuses `invalid_package`/`invalid-value` at the member; a reward whose function is owned by its operation with its parameters is admitted. | Test |
| FR-369-AC-10 | A `delay_distribution` on a model that has no `timed_model` root refuses `invalid_package`/`invalid-value` at `declaring`; one on the published timed model's `dense` source is admitted. | Test |
| FR-369-AC-11 | A second `random_parameter` of one operation parameter, `workload` of one name, `reward` of one name on one operation and `delay_distribution` of one operation refuse `invalid_package`/`conflicting-definition` at the later node. | Test |
| FR-369-AC-12 | An admitted package gives the support of a random parameter with the exact probability of each value, a reward with its name, type and function, a workload with all its weights in order, and a delay distribution of each family with its exact numbers and the unit of each time quantity, each keyed by its node. | Test |
| FR-369-AC-13 | A `random_parameter` of a declared value type (`Int[1, 12]`) with integer support, and one of a `Quantity<u>` type with quantities in `u`, are admitted; a `reward` whose `value_type` is a unit type, a compound-unit type or an `integer_range` type is admitted. | Test |
| FR-369-AC-14 | A support value of a declared integer type given as text, a quantity in another unit than the parameter's, and a quantity given as an integer refuse `invalid_package`/`invalid-value` at the value. | Test |
| FR-369-AC-15 | A `workload` is admitted when it names the model's own operations and those it inherits through supertypes of supertypes; one that omits the model's own operation although the model has supertypes, one that omits an inherited operation, and one that names a model that is no ancestor refuse `invalid_package`/`invalid-value` at the member. | Test |
| FR-369-AC-16 | A `reward` whose function reaches a `pre_read`, `presence_read`, `value_read`, `deref`, `query` or `reachability` expression, a `state` node, a `model` node, or a parameter it does not list refuses `invalid_package`/`invalid-value` at `expression`; one that reaches a constant is admitted. | Test |
| FR-369-AC-17 | The join charges one validation visit for each node scanned for a delay's `dense` root and one for each supertype edge followed for a workload: the work grows by at least the nodes added before the root and the edges added to the chain. | Test |

## Dependencies

- FR-038 owns strict admission; FSpec FR-640 owns the wire shapes.
