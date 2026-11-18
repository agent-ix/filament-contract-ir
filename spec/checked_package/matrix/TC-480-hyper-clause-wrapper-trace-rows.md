---
id: TC-480
title: "Strict V2 reader keys a hyper clause wrapper's clock rows by trace"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: verifies
---
# TC-480: Strict V2 reader keys a hyper clause wrapper's clock rows by trace

## Description

Verify FR-038-AC-561 through the public strict V2 reader over event-position
packages whose one temporal clause is the body wrapper of a
`claim`/`hyperproperty` node.

## Test Procedure

1. Read a wrapper quantifying traces `s` and `n`; expect two rows, `trace`
   `s` then `n`, equal declaration, occurrence, clock name and configuration.
   Read a one-trace wrapper; expect admission.
2. Remove the `n` row, and the `s` row; expect
   `invalid_package`/`timed-node-reference` at the wrapper's node body.
3. Set the second row's `trace` to `x`; remove the first row's `trace`; expect
   `timed-node-reference` at the row's `trace`.
4. Append a third row repeating `s`; expect
   `invalid_package`/`conflicting-definition` at the later row.
5. Swap the two rows; expect `timed-node-reference` at the later row's `trace`.
6. Add `trace` to the row of a single-trace clause; expect
   `timed-node-reference` at its `trace`.
7. Set `trace` to the empty string, `null` and a number; expect `malformed_wire`.
8. Round-trip a row with and without `trace` through the row type.

## Expected Results

Each admitted package reads its rows back by key; each defective one refuses
with the stated code, cause and locus.

## Status

Implemented.
