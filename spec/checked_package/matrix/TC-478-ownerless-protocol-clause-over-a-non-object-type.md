---
id: TC-478
title: "Strict V2 reader admits an ownerless protocol clause over a non-object type"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-064
    type: verifies
---
# TC-478: Strict V2 reader admits an ownerless protocol clause over a non-object type

## Description

Verify FR-064-AC-7 and FR-064-AC-8 through the public strict V2 reader over
protocol packages built in the test: a clause whose `over` type reaches no
object type is admitted without `owner_model`, and a clause whose `over` type
reaches an object type still requires the edge.

## Test Procedure

1. Retype the `over` binder of an admitted clause to `Boolean`, `Int[0, 3]`,
   a model record, a tuple, `Option<Integer>` and `Sequence<Integer>`, remove
   `owner_model`, and read; expect admission with no object-type node in the
   clause's dependencies (FR-064-AC-7).
2. Add a clause over `M::T` beside one over `Int[0, 3]` and reconstruct the
   refinement class catalog; expect only the clause over `M::T` in it
   (FR-064-AC-7).
3. Remove the edge from clauses over `M::T`, `Option<M::T>` and
   `Sequence<M::T>`; expect `malformed_wire` at the clause node. Restore it to
   `M::T` and retarget it to `M::U`; expect admission (FR-064-AC-8).
4. Add an `owner_model` edge to each ownerless clause of step 1; expect
   `malformed_wire` at the clause node (FR-064-AC-8).

## Expected Results

Each ownerless clause over a non-object type is admitted and contributes no
catalog class; each clause over an object type without its edge, and each
non-object clause with one, refuses `malformed_wire` at the clause node.

## Status

Partial: an ownerless clause over a bounded integer is admitted and a clause
over an object type without its edge refuses; the reference, `option` and
collection reach and the added-edge refusal are not yet tested.
