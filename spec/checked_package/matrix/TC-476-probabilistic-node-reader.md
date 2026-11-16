---
id: TC-476
title: "Checked-package reader carries the probabilistic node tag and four forms"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-369
    type: verifies
---
# TC-476: Checked-package reader carries the probabilistic node tag and four forms

## Description

Verify every AC of FR-369 at the flat wire check and through the public strict V2
reader (`CheckedPackageV2::read`) over packages built in the test, and the
delay join over the published timed model (the `it` target reads it from
`FILAMENT_SPECIFICATION_DIR`, as CI sets it; the `conformance_fspec` target
repeats the delay cases).

## Test Procedure

1. Decode the tag and four forms; read and re-encode one node of each form and
   each delay family (FR-369-AC-1).
2. Remove, add and reorder one member per form, add a `declaration`, and damage
   a nested row; expect `malformed_wire` at the member (FR-369-AC-2).
3. Give an empty support, zero weights and rate, and an unknown family; expect
   `invalid_package`/`invalid-value`; admit a zero workload weight
   (FR-369-AC-3).
4. Change the tag and a form to words outside the vocabulary; expect the
   existing decode failures (FR-369-AC-4).
5. Count the work charged per node and per member, and deny a budget one unit
   short (FR-369-AC-5).
6. Point an operation reference at a node that is no model node and at an
   operation the model lacks; expect `invalid-value` at the member
   (FR-369-AC-6).
7. Name a parameter the operation lacks and give a support value of another
   literal kind; expect `invalid-value` (FR-369-AC-7).
8. Omit, repeat, reorder and misname a workload operation; expect
   `invalid-value` at the row (FR-369-AC-8).
9. Give a reward a Boolean `value_type`, a function of another operation, no
   parameters, and another type; expect `invalid-value` (FR-369-AC-9).
10. Put a delay distribution on a model with no timed root; expect
    `invalid-value`; admit one on a `dense` source (FR-369-AC-10).
11. Declare a second parameter distribution, workload name, reward and delay;
    expect `conflicting-definition` at the later node (FR-369-AC-11).
12. Read the admitted declarations back by node: the exact probability of each
    support value, a reward's type and function, every workload weight, and
    each delay family with its exact numbers and units (FR-369-AC-12).
13. Admit a parameter of a declared `Int[1, 12]` type and of a `Quantity<ms>`
    type, and a reward of a unit type and of an integer-range type
    (FR-369-AC-13).
14. Give a declared integer parameter a text value, and a quantity parameter a
    quantity in another unit and an integer; expect `invalid-value`
    (FR-369-AC-14).
15. Cover a model's own and transitively inherited operations; omit its own,
    omit an inherited one, and name a non-ancestor (FR-369-AC-15).
16. Make a reward's function reach each of the six state-reading expressions, a
    `state` node, a `model` node and a parameter it does not list; expect
    `invalid-value`; admit a constant (FR-369-AC-16).
17. Pad the graph before a delay's timed root and compare the work the delay
    costs; expect it to grow by at least the padding (FR-369-AC-17).

## Expected Results

Each shape reads and re-encodes; each damaged node refuses at its first failing
member with the stated code and cause; a valid package is admitted.

## Status

Partial (IR-1550): a bounded parameter type's range, a delay quantity's time unit and a reward function's computed value type are Planned in FR-369.
