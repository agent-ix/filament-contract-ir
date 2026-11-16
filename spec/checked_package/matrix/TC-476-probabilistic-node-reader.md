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
delay join over the published timed model.

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
   parameters, another type, and a function that reaches a `pre_read`; expect
   `invalid-value` (FR-369-AC-9).
10. Put a delay distribution on a model with no timed root; expect
    `invalid-value`; admit one on a `dense` source (FR-369-AC-10).
11. Declare a second parameter distribution, workload name, reward and delay;
    expect `conflicting-definition` at the later node (FR-369-AC-11).
12. Read the admitted declarations back by node, with the exact probability of
    each support value (FR-369-AC-12).

## Expected Results

Each shape reads and re-encodes; each damaged node refuses at its first failing
member with the stated code and cause; a valid package is admitted.

## Status

Partial (IR-1550): a bounded parameter type's range, a delay quantity's time unit and a supertype model's inherited workload operations are Planned in FR-369.
