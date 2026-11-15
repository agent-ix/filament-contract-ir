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

Verify every AC of FR-369 at the flat wire check of the strict V2 reader over
packages built in the test.

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

## Expected Results

Each shape reads and re-encodes; each damaged node refuses at its first failing
member with the stated code and cause.

## Status

Implemented (IR-1549).
