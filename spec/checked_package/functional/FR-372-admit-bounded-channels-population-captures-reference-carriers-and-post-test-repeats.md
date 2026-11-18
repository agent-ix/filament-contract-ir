---
id: FR-372
title: "Admit bounded channel types, stable FIFO keys, population captures, protocol reference carriers and post-test repeats"
type: FR
relationships:
  - target: ix://agent-ix/filament-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/filament-contract-ir/FR-064
    type: references
  - target: ix://agent-ix/filament-specification/FR-726
    type: references
  - target: ix://agent-ix/filament-specification/FR-728
    type: references
  - target: ix://agent-ix/filament-specification/FR-729
    type: references
  - target: ix://agent-ix/filament-specification/FR-464
    type: references
  - target: ix://agent-ix/filament-spec-language/FR-880-AC-3
    type: references
  - target: ix://agent-ix/filament-spec-language/FR-880-AC-8
    type: references
  - target: ix://agent-ix/filament-spec-language/FR-880-AC-9
    type: references
  - target: ix://agent-ix/filament-spec-language/FR-880-AC-10
    type: references
  - target: ix://agent-ix/filament-spec-language/FR-881-AC-2
    type: references
  - target: ix://agent-ix/filament-spec-language/FR-883-AC-1
    type: references
  - target: ix://agent-ix/filament-spec-language/FR-883-AC-2
    type: references
---

# FR-372: Admit bounded channel types, stable FIFO keys, population captures, protocol reference carriers and post-test repeats

## Description

In the protocol step of [FR-038](./FR-038-consume-checked-package-v2.md),
the strict V2 reader SHALL admit every protocol form that a source language
writes for a checked protocol: a channel that carries a bounded type, a FIFO
channel keyed by a FIFO-key-stable type, a model population read in a slot
that reads model state, a `protocol_reference` carrier in any protocol slot,
and a post-test `repeat`. Each form keeps the refusals that FSpec states for
its malformed or ineligible variants.

## Inputs

A `filament.checked-package/v2` document with protocol nodes, supplied with
its selections, evidence and limits to the strict reader of
[FR-038](./FR-038-consume-checked-package-v2.md).

## Outputs

The admitted package, or the typed, located refusal for the first failing
check, with no partial admitted package.

## Behavior

### Bounded channel message types

A channel's `carries` target MAY be a `bounded_domain` node of form
`integer_range`, `rational_range`, `decimal_range`, `float_rounding`,
`text_bounds` or `collection_bounds` (FSpec FR-726). A source language lowers
`Int[0, 9]`, `Text[0, 8; nfc]` and a bounded `Sequence` to such nodes, and a
protocol author names them as message types as often as native types. The
bound is a premise every sent message satisfies; it changes neither how the
channel queues messages nor how the queue is stepped, so the reader admits the
type exactly as written and the package keeps the bound for every consumer
that checks a send against it. A `carries` target of any other node kind that
is not a type, such as an `expression` node, still refuses
`invalid_model_binding`/`malformed-declaration` at the target.

### FIFO-key-stable keys

A FIFO channel orders the messages that carry equal key values, which is well
defined only when key equality is an equivalence relation. The reader SHALL
admit a FIFO `key` whose type is *FIFO-key-stable*: a FSpec FR-322
equality-stable type with no `float32` or `float64` leaf at any depth (FSpec
FR-726). The reader decides this by a structural walk of the key's type node
through `option`, collection, record, tuple and variant members to every leaf,
charging the protocol step's work meter for each node it visits. A
`bounded_domain` node other than `float_rounding` and `model_population`
counts as a leaf of its base type's family; a `carried_quantity` is walked
through its carrier; a `bounded_quantity`, `bytes`, `json_object`, an enum and
a reference are stable leaves; a model-owned `value_type`,
`tuple_value_type`, `record_value_type` or `variant_type` is walked through
its members. IEEE `numeric_equal` is not reflexive on NaN and equates `+0`
and `-0`, so a key type with a float leaf, a `float_rounding` bounded domain
included, refuses `ill_typed`/`operator-ineligible` at the key, retaining the
operator `=` and the type. A type with an `event_type` leaf is not
equality-stable and refuses the same way.

### Model population reads in protocol slots

The protocol slots that read model state are the protocol, registration and
activation captures, the `send` and `event` constraints with their related
relationship ends, the `finish` constraint and the `terminal when` condition
(FSpec FR-464 stage table). In those slots, the binder-scope check SHALL
treat a `relation`/`population` node, and a model-owned `value`/`parameter`
node, of any selected model as a model-state read in scope, not as an
out-of-scope binder. The read is not restricted to the model of the clause's
`owner_model` edge, so an ownerless clause reads populations the same way
and a capture may read the domain package's population (FSpec FR-464
"Binders"). A capture such as
`n: Integer = size(allInstances<M::Order>(M::orders))` therefore admits; it
reads the population that the evaluated protocol state holds, by the same
rule a state clause body uses (FSL FR-883). A read of a binder that is
outside the slot's scope still refuses `missing_declaration`/`missing-name`
at the read. A slot that bars model state (an activation or trigger guard,
the retry relation, the recovery predicate, a `choice` or `repeat` guard, a
`visible` entry, a `check` condition or a FIFO key) keeps refusing an
`allInstances` or `lookup` read `ill_typed`/`operator-ineligible` at the
read, before binder scope (FSpec FR-464 "Slots that bar model state").

### `protocol_reference` carriers in binder scope

When the reader finds what a protocol expression reads, the dependency
closure SHALL stop at a `protocol_reference` carrier, as it stops at a
`function` node (FSpec FR-728). A carrier denotes the referenced protocol as a
whole; the referring slot never steps into the referenced clause, so that
clause's finish binder, captures and `over` input are not reads of the
referring slot. Walking through the carrier would charge the slot with reads
it does not make and refuse every protocol body that names a protocol.
Stopping at the carrier leaves recursion-group handling unchanged when the
reference closes a cycle. A carrier is therefore admitted in a `check` or
`send` constraint, an `each` guard, a `repeat` or `choice` guard, a
`terminal when` condition, a capture initializer and a `finish` constraint,
wherever a value of its type is admitted.

### Post-test `repeat`

The `repeat` control layout SHALL accept an optional eighth binding `test`,
a text literal, after `exhausted` (FSpec FR-729). Its only value is `post`.
A `repeat` without `test` is pre-test and keeps the bytes and node id it had
before the member existed; a post-test `repeat` and the pre-test `repeat` of
the same body and guard have different node ids. A post-test body runs once
before its guard is first read, so its guard MAY read the binders its body
binds on every path to the body's exit. A retry loop whose guard reads the
body's result is post-test, and without this form a source language could
check such a loop but never write it. In both forms, the `repeat`'s `visible`
entries and its `proof` `invariant` and `variant` MAY read the binders its
body binds on every path to the body's exit. These back-edge reads read the
value from the most recently completed iteration; in a pre-test `repeat` they
have no value at entry, so at entry such a `visible` entry is not a decision
fact and such an `invariant` or `variant` is not evaluated. Only a post-test
guard reads body binders. A `test` value other than `post`
refuses `malformed_wire` at the member. A post-test `repeat` with `maximum`
0 refuses `invalid_package`/`invalid-value` at the node. A pre-test guard
that reads a body binder, and any slot that reads a binder bound on only some
paths to the body's exit, refuse `missing_declaration`/`missing-name` as an
out-of-scope read.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-372-AC-1 | A channel whose `carries` names a `bounded_domain` node of each form `integer_range` (`Int[0, 5]`), `text_bounds` (`Text[0, 8; nfc]`), `decimal_range`, `rational_range`, `float_rounding` (`Float64[nearest-even]`) and `collection_bounds` (`Sequence<Boolean>[0, 3]`) is admitted, and the admitted channel names that node as `carries`. A `carries` that names an `expression` node refuses `invalid_model_binding`/`malformed-declaration` at the target. Implements FSpec FR-726-AC-1 and FSL FR-880-AC-10. | Test |
| FR-372-AC-2 | A FIFO channel whose `key` has type `Int[0, 99]`, a model value type that is a bounded integer, `Quantity<m>`, `Option<Int[0, 5]>`, `Sequence<Quantity<m>>`, a record with a bounded `Decimal` field, a model `variant_type`, or a model `record_value_type` whose leaves are FIFO-key-stable is admitted with its key. Implements FSpec FR-726-AC-2, FSL FR-880-AC-10 and FSL FR-881-AC-2. | Test |
| FR-372-AC-3 | A FIFO channel whose `key` has type `float64`, `Float64[nearest-even]`, `Option<float32>`, a record with a `float64` field, or a type holding an `event_type` leaf refuses `ill_typed`/`operator-ineligible` at the key, retaining `=` and the type; the same channel declared `unordered` is admitted. Implements FSpec FR-726-AC-3 and FSL FR-881-AC-2. | Test |
| FR-372-AC-4 | A protocol capture `n: Integer = size(allInstances<M::Order>(M::orders))` is admitted in a clause owned by model `M`, in an ownerless clause and in a clause owned by another selected model, and the same population read is admitted in a registration capture, an activation capture, a `send` and an `event` constraint and a `finish` constraint. Replacing the initializer with a read of a parameter that no binder of the slot introduces refuses `missing_declaration`/`missing-name` at the read. The same `allInstances` read in an activation guard, a `choice` guard or a `check` condition refuses `ill_typed`/`operator-ineligible` at the read. Implements FSL FR-883-AC-1, FR-883-AC-2 and FR-883-AC-4 under the FSpec FR-464 stage table. | Test |
| FR-372-AC-5 | A protocol whose `check` constraint, `each` guard, `repeat` guard, `terminal when` condition, capture initializer and `finish` constraint each hold a `protocol_reference` carrier to another protocol's clause, whose clause has a finish binder and reads `over`, is admitted with no `BinderScope` or `ModelStateRead` refusal for those slots. In the same slots, a read of the referenced clause's finish binder without the carrier refuses `missing_declaration`/`missing-name` at the read, and a read of `over` beside the carrier in a slot that bars model state refuses `ill_typed`/`operator-ineligible`. Implements FSpec FR-728-AC-1, FR-728-AC-2 and FSL FR-880-AC-3. | Test |
| FR-372-AC-6 | A `repeat` whose body binds `got` on every path, with `test` `post`, `visible (got.ok)` and guard `not got.ok`, is admitted with no `maximum`, with `maximum` 3 and with `maximum` 1. With `maximum` 0 it refuses `invalid_package`/`invalid-value` at the node; with `test` `pre` or `middle` it refuses `malformed_wire`; without `test`, the same guard refuses `missing_declaration`/`missing-name` as an out-of-scope read. A pre-test `repeat` without `test` has the same node bytes and node id as before the member existed, and differs in node id from the post-test `repeat` of the same body and guard. Implements FSpec FR-729-AC-1, FR-729-AC-3 and FSL FR-880-AC-8. | Test |
| FR-372-AC-7 | A post-test `repeat` whose `invariant` and `variant` read the body's every-path binder `got` is admitted, and the same `repeat` whose `visible` entry reads a binder that only one branch of a body `choice` binds refuses `missing_declaration`/`missing-name` as an out-of-scope read. A pre-test `repeat` with guard `true` and `visible (got)`, and one whose `invariant` and `variant` read `got`, are each admitted without `test`, the `visible` entry being an `expression` node over `got`. Implements FSpec FR-729-AC-4, FR-729-AC-5 and FSL FR-880-AC-9. | Test |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns the protocol step, its
  check order and its binder-scope walk.
- [FR-064](./FR-064-bind-protocol-clause-to-owner-model.md) owns the
  `owner_model` edge whose model scopes the population reads.
- FSpec FR-726, FR-728 and FR-729 own the channel, binder-closure and
  `repeat` rules; FSpec FR-464 owns the stage table of slots that read or bar
  model state.

## Status

Partially implemented. Bounded `carries`, the FIFO-key-stable walk, the
population reads in the model-state slots, the carrier stop and the `test`
`post` member are implemented for their admitted cases. The reader still
admits `test` `pre`, which this requirement refuses, and admits population
reads only of the clause's owner model. The post-test
`maximum` 0 refusal, the post-test guard's scope over the body's binders, and
the negative cases of AC-3 to AC-5 and the back-edge reads of AC-7 have no
tests yet.
