---
id: TC-485
title: "Source operation parameters in the identity rows"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-367
    type: verifies
---
# TC-485: Source operation parameters in the identity rows

## Description

Verify that a source-object identity row's operation entries carry their parameters as member entries and that a row whose `clock_parameters` disagree with its clock parameter entries, or whose parameter entries are malformed, refuses (FR-367-AC-8; FSpec FR-610-AC-9).

## Test Procedure

1. Join the row of `O` for `object O { d: Clock; operation arm(x: Clock, k: Int[1, 3]) modifies self.d; }` and take the typed read of `O::arm`.
2. Edit the entry's `clock_parameters` to `[]`, then to `["k"]`; give `x` both a `type_node` and a `member_type`; add a second entry `x`. Join each.

## Expected Results

- Step 1: joined; `parameters` are the clock entry `x` then the data entry `k`, `clock_parameters` `["x"]`.
- Step 2: each `UnknownIdentityWitness` at the `Package` locus, with no partial view.
