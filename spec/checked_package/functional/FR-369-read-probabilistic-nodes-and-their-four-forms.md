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
  - target: ix://agent-ix/filament-contract-ir/FR-367
    type: depends_on
  - target: ix://agent-ix/filament-specification/FR-610
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
  states, including a member row of `discrete` that does;
- a time quantity of a `uniform_interval` or `discrete` delay that is negative
  (FSpec FR-640 rule 5 with FR-420's non-negative delays), at the quantity's
  `value` member; a quantity whose spelling is no exact rational stays
  `malformed_wire`.

A workload weight of 0 is admitted. The strict V2 reader SHALL charge one
validation visit of the caller's budget for each node and for each support,
weight or delay member, SHALL make that charge ahead of inspecting the member,
and SHALL stop at the first uncharged member with no partial view.

A `node_tag` outside the closed vocabulary keeps the decoder's existing
`unsupported_node_tag` refusal.

When the owner join has admitted a package, the strict V2 reader SHALL check each `probabilistic`
node in ascending node-id order against the nodes it joins and refuse the first
failure with `invalid_package` and cause `invalid-value`, at the failing member:

- an operation reference whose `declaring` node is neither a `model` node of an
  object type, process or state machine of a selected model nor a
  source-declared `model`/`object_type` node of the package (FSpec FR-322
  "Source-declared object types"), or which names no operation that node
  declares (a node absent from the graph is already refused by the graph's
  dependency check). A selected model's declaring node resolves through the
  retained models, as before; a source-declared one has no domain document, so
  the strict read checks only its kind and `SourceOwner`, and the operation is
  resolved at the identity-row join below;
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
  integer or quantity type, or, on a selected model's declaring node, whose
  `expression` is not a `function` node owned by the model operation member, that lists exactly the operation's
  parameters in declared order in its `parameters` binding, and whose own type
  is the `value_type`;
- a `reward` whose `declaring` node is source-declared and whose `expression`
  is not a `function` node that carries the declaring node's `SourceOwner`
  (the same `identity`), the `declaration` `[<type>, <operation>, "reward",
  <name>]` of FSpec FR-322 "Source-declared object types" item 5, `<type>`
  the declaring node's own `declaration` segment, `<operation>` the reward's
  `operation` text and `<name>` its `name` text, the clause kind `reward` in
  its one `clause` binding, and the `value_type` as its own type. A source
  operation has no domain document, so its parameter list is checked at the
  identity-row join below; the walk of the next item admits the parameters
  the function lists;
- a `reward` whose `expression` reads anything but the operation's parameters
  and constants, as FSpec FR-405 states it (a reward "reads only the
  operation's parameters, random parameters included ... reads no state");
- a `delay_distribution` whose `declaring` model is not the model of a
  `timed_model` root whose time source has kind `dense`, or, for a
  source-declared `declaring` node, for which no `timed_model` root whose
  `model` is a source-declared node of the same `SourceOwner` identity has a
  `dense` source: one source time member times every source object type of
  the unit (FSpec FR-640 Reader rule 5, FR-415), and the delay's time
  quantities are then typed at that source's unit.

After the strict read admits a package and the source-object identity rows are
admitted ([FR-367](./FR-367-join-source-object-rows-and-field-reads.md),
`SelectedSourceObjects`), the reader SHALL join every operation reference of a
`probabilistic` node whose `declaring` node, or a workload's `model`, is
source-declared, in ascending node-id order, through the identity row whose
`structural_node` is that node (FSpec FR-610 "Identity rows"), and refuse the
first failure with `invalid_package` and cause `invalid-value` at the same
member the strict read would name:

- an `operation` name that no `operations` entry of the row carries (the entry's
  `identity` is `<type>::<operation>`); for a workload, the operations are the
  entries of the row and of the rows of its `supertypes`, followed
  transitively, and the workload rules of the strict read apply to them;
- a `random_parameter` whose `parameter` names no entry of the operation
  entry's `parameters`, or a support value whose literal kind or membership is
  not that of the entry's type node by the rules the strict read applies to a
  selected parameter; a clock entry has no literal kind and refuses;
- a `reward` whose `expression` function's `parameters` binding does not list
  exactly the entries of the operation entry's `parameters`, by name, in
  declared order and with none left out, the check the strict read applies
  to a selected operation's parameters; the refusal is at the reward's
  `expression`.

The source reward's function is a declared node keyed by its `declaration` at
ordinal 0 (FSpec FR-640 "Reward identity"), so it is never matched by a
lowering ordinal, and the owner join admits it beside the operation's other
source clause functions under the one-holder-per-qualified-name rule
([FR-366](./FR-366-admit-source-operation-dispatch-tables.md)); a function
whose declaration names another type, operation or reward, or whose `clause`
is another kind, belongs to no reward and is refused at the `expression`
that names it, as a selected reward's function owned by another operation is.

The join SHALL charge one validation visit per operation reference joined, per
operation entry and supertype edge read, per support value checked and per
reward function parameter compared, ahead of the work, so a bound too small
for the join stops the read as incomplete.

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

The strict V2 reader SHALL refuse, with `invalid_package` and cause
`invalid-value` at the value, a `random_parameter` support value that is no
member of its parameter's declared type, as FSpec FR-322 states membership and
literal spelling and not as FSpec FR-141 evaluates at run time:

- an integer, or a rational, or the magnitude of a quantity over a rational,
  that is not the one canonical spelling FR-322 gives it (`+5`, `05`, `2/4`,
  `3/1` and `01` refuse), so that two spellings of one number cannot both stand
  in a support. A misspelt rational or quantity refuses at the literal's `value`
  member, as FR-322 "Scalar literal values" places it; every other refusal here,
  a misspelt integer included (FR-322 names no pointer for one), refuses at the
  literal;
- an integer outside `Int[lo, hi]`; a rational, or the magnitude of a quantity
  over a rational, whose numerator is outside `[n1, n2]` or whose positive
  denominator is outside `[d1, d2]` of `Rational[n1, n2; d1, d2]`; and
- a text that, under `nfc`, `nfd`, `nfkc` or `nfkd`, does not already equal its
  own normalization (the reader applies no normalization, FR-322 "Text literal
  membership"), or whose length is outside `[min, max]` of
  `Text[min, max; profile]`. The length is the scalar count of the value as
  written, and under `binary-utf8` its UTF-8 byte count.

The strict V2 reader SHALL refuse, with `invalid_package` and cause
`invalid-value` at the quantity, a time quantity of a `delay_distribution` that
is not typed at the unit of the `dense` time source of the model's `timed_model`
root, as FSpec FR-640 states "Time quantities are in the timed model's time
unit".

The join SHALL charge one validation visit, at the value's pointer and ahead of
inspecting it, for each support value and for each delay parameter member it
checks.

A `reward` whose function has a `body` binding is refused `ill_typed` and cause
`type-mismatch` at the function when the body root has another `semantic_type`
than the function, by the reader's general result comparison of FSpec FR-322
"Results"; the join adds no second check of the body.

A support value of a Boolean parameter is a `boolean` literal whose `value` is a
JSON boolean, and one of a parameter of a declared enumeration (a variant type
none of whose cases carries a payload) is an `enum` literal typed at that
declaration whose `value` names one of its cases (FSpec FR-640 admits a random
parameter of any parameter type); a literal of another kind, a non-boolean
value, a literal typed at another declaration and an unknown or differently
cased case name refuse at the value. A support value of a declared record,
tuple, option or collection type, or of a variant type with a payload case (a
union, for its nullary cases too), has no literal kind: FSpec has no literal
form for these, and the reader's general literal check refuses it as
`ill_typed`/`operator-ineligible` at the literal's `type`, before this join.

### Planned

IR reads FSpec FR-322 "Results" as leaving a function with no `body` binding (a
clause function) outside the comparison of a body root with the function's
type, and FSpec FR-640 as silent on whether a reward function has a `body`
binding. Under that reading the reader admits a reward function with no `body`,
which is the shape FR-640 describes, and the type its expression computes is
never checked: only the function's own `semantic_type` is compared with
`value_type` (FR-369-AC-9). Whether a reward function SHALL carry a `body`
binding, so that its computed type can be checked, is a spec question for
QSL-2310; until FSpec states it, this check is open. Owner: titan:IR:code after
QSL-2310.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-369-AC-1 | `probabilistic` and each of its four forms decode, `CheckedNodeKind::all` lists them, a node of each form (each delay family included) reads and re-encodes to the document it was read from, and the public reader admits a package holding a node of each form. | Test |
| FR-369-AC-2 | A probabilistic node with a missing member, an extra member, members out of order, a `declaration`, an `owner`, a `recursion_group`, no or two occurrences, an occurrence of another role, a binding of another term kind, or a short or extended support row or workload weights row refuses `malformed_wire` at the failing member. | Test |
| FR-369-AC-3 | An empty support, a repeated support value (a Boolean one included), a zero support or discrete weight, a zero or negative rate, a negative workload weight, a negative delay time quantity, a weight not in lowest terms, an unknown delay family and delay parameters that do not match the family refuse `invalid_package`/`invalid-value`; a zero workload weight is admitted. | Test |
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
| FR-369-AC-18 | A support value that is no member of its parameter's declared type refuses `invalid_package`/`invalid-value` (at the literal, and at the literal's `value` member for a misspelt rational or quantity): outside `Int[lo, hi]` or `Rational[n1, n2; d1, d2]` (also for a quantity over one), a non-canonical integer or rational spelling (`+5`, `05`, `2/4`, `3/1`, `+1`, `01`, `1/0`), a text not in the normal form of `nfc`, `nfd`, `nfkc` or `nfkd`, or a text whose length is outside `Text[min, max; profile]` (scalars as written, bytes under `binary-utf8`); the bounds themselves and a text in normal form with a length in range are admitted, and two spellings of one value refuse at the second. | Test |
| FR-369-AC-19 | A `reward` whose function has a `body` binding whose root has another type than `value_type` refuses `ill_typed`/`type-mismatch` at the function by FR-322 "Results"; a body of that type is admitted. The AC pins that existing comparison for a reward with a `body`; it does not cover the bodyless clause function FR-640 describes (see Planned). | Test |
| FR-369-AC-20 | The join charges one validation visit for each support value at the value's pointer: a budget exists that stops the read as incomplete at exactly the value of each row. | Test |
| FR-369-AC-21 | A delay quantity, of `uniform_interval` or `discrete`, typed at another unit than the `dense` source's refuses `invalid_package`/`invalid-value` at the quantity; quantities in the source's unit are admitted. | Test |
| FR-369-AC-22 | The join charges one validation visit for each delay quantity of a `discrete` list at the quantity's pointer: a budget exists that stops the read as incomplete at exactly the delay of each row. | Test |
| FR-369-AC-23 | A `random_parameter` of a Boolean parameter with `boolean` support literals `true` and `false`, and one of a declared enumeration parameter with `enum` support literals typed at that declaration and naming its cases, are admitted and read back with their supports. | Test |
| FR-369-AC-24 | A Boolean parameter's support value given as the text `"true"`, as the number `1`, as an integer literal or as an `enum` literal, and an enumeration parameter's support value that names an unknown case, differs from a case in letter case, is typed at another enumeration, is a Boolean literal or is a text literal refuse `invalid_package`/`invalid-value` at the value. A support value of a variant type with a payload case (a union), for its payload case and its nullary case alike, refuses `ill_typed`/`operator-ineligible` at the literal's `type`, by the reader's general literal check before the join. | Test |
| FR-369-AC-25 | A package of a unit with no selected model that declares `object O { d: Clock; operation step() modifies self.d; }`, `object Other { n: Integer; operation poke(k: Int[1, 3]) modifies self.n; }` and `time O dense unit ms;`, holding a `delay_distribution` on `O::step`, a `discrete` one on `Other::poke` with times typed at `ms`, a `random_parameter` of `Other::poke`'s `k` with support `1`, `3`, a `workload` on `O`, and a `reward` `cost` on `Other::poke` returning `k` with `value_type` the `Int[1, 3]` node, its `expression` the function carrying the unit's `SourceOwner`, `declaration` `["Other", "poke", "reward", "cost"]` at ordinal 0, `clause` `"reward"` and `parameters` `[k]`, each operation reference's `declaring` (the workload's `model`) a source `model`/`object_type` node, is admitted by the strict read and by the identity-row join, and the typed read gives each declaration keyed by its node, the reward with its name `cost`, its value type and that function. The package and that typed read are the checked-in golden pair of TC-484: the reward's function id must equal the declared-node id `{package, ["Other", "poke", "reward", "cost"], 0}` built independently of the reader, and the typed read must equal the golden one. | Test (TC-484) |
| FR-369-AC-26 | Over the AC-25 package each refuses `invalid_package`/`invalid-value` at the member: at the strict read, a `declaring` that is a source `relation`/`population` node, and a delay whose source model has no `dense` time member; at the identity-row join, an `operation` `nudge` that `Other`'s row lists no entry for, a `parameter` `j` that `Other::poke`'s `parameters` lack, a support value `4` outside `Int[1, 3]`, and a workload that omits `O::step`. For the reward `cost`, each refuses `invalid_package`/`invalid-value` at the reward's `expression` (`/semantic_graph/nodes/{i}/body/members/4/value/target`): at the strict read, its function's `declaration` rewritten to `["Other", "poke", "reward", "load"]`, to `["Other", "step", "reward", "cost"]` and to `["O", "poke", "reward", "cost"]`, its `clause` rewritten to `"precondition"`, and its owner replaced by a `SourceOwner` of another `identity`; at the identity-row join, its function's `parameters` listing a parameter `j` (read by the body) in place of `k`, and listing none with a constant body `1`. The reward's `operation` rewritten to `nudge` refuses at `operation` at the identity-row join, as for the other forms. A budget one visit short of the join's charges, the reward's parameter comparison included, stops the read as incomplete. | Test (TC-484) |

## Dependencies

- FR-038 owns strict admission; FSpec FR-640 owns the wire shapes.
