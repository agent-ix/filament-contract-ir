---
id: TC-483
title: "Strict V2 reader keys a hyperproperty's clock rows by trace"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: verifies
---
# TC-483: Strict V2 reader keys a hyperproperty's clock rows by trace

## Description

Verify FR-038-AC-561 through the public strict V2 reader over event-position
packages holding `claim`/`hyperproperty` nodes, in the behaviours and the
finite domain.

## Test Procedure

1. Read a hyperproperty quantifying traces `s` and `n`; expect two rows keyed
   on the hyperproperty node, `trace` `s` then `n`, and no row for its body
   wrapper. Read a one-trace node; expect admission.
2. Give the two rows different `sequence_authority` values; expect admission.
3. Read a finite-domain hyperproperty with two traces; expect two rows; drop
   one; expect `timed-node-reference` at the node's body.
4. Key a row on the body wrapper; expect `timed-node-reference` at its
   `declaration`.
5. Read two hyperproperty nodes sharing one wrapper with different
   quantifiers; expect each node's own rows.
6. Remove the `n` row, and the `s` row; expect
   `invalid_package`/`timed-node-reference` at the node body.
7. Set the second row's `trace` to `x`; remove the first row's `trace`; expect
   `timed-node-reference` at the row's `trace`.
8. Append a row repeating `s`; expect `conflicting-definition` at the later row.
9. Swap the two rows; expect `timed-node-reference` at the later row's `trace`.
10. Give the second row another clock name; expect `timed-node-reference` at
    its `clock_name`.
11. Add `trace` to a single-trace clause's row; expect
    `timed-node-reference` at its `trace`.
12. Set `trace` to the empty string, `null` and a number; expect `malformed_wire`.
13. Round-trip a row with and without `trace` through the row type.

## Expected Results

Each admitted package reads its rows back by key; each defective one refuses
with the stated code, cause and locus.

## Status

Implemented.
