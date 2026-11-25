---
id: TC-483
title: "Strict V2 reader keys a hyperproperty's clock rows by trace"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: verifies
  - target: ix://agent-ix/filament-contract-ir/FR-038-AC-630
    type: verifies
---
# TC-483: Strict V2 reader keys a hyperproperty's clock rows by trace

## Description

Verify FR-038-AC-561 and FR-038-AC-630 through the public strict V2 reader over
packages holding `claim`/`hyperproperty` nodes, in the behaviours and the finite
domain: event-position packages with clock rows, and infinite-trace packages
without them.

## Test Procedure

1. Read a hyperproperty quantifying traces `s` and `n`; expect two rows keyed
   on the hyperproperty node, `trace` `s` then `n`, and no row for its body
   wrapper. Read a one-trace node; expect admission.
2. Give the two rows different `sequence_authority` values; expect admission.
3. Read a finite-domain hyperproperty with traces `a` and `b`; expect two rows
   with the role `t`; drop one, set a trace to `c`, set the second row's
   `clock_name` to `u`, give a row a `fixed_sample` configuration; expect
   `timed-node-reference` at the node's body, the row's `trace`,
   `/temporal_bindings/1/clock_name` and the row's `clock/kind`.
4. Key a row on the body wrapper; expect `timed-node-reference` at its
   `declaration`.
5. Read two hyperproperty nodes sharing one wrapper with different
   quantifiers, one the other's reverse with a third trace; expect each node's
   own rows in its own order; write the reverse's rows in the other's order,
   and omit the third trace's row; expect the stated refusals.
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
12. Give two Groups one variable; expect `conflicting-definition` at the
    second row, and, with only one row present, `ambiguous-name` at the later
    Group's `variable` binding in place of a missing-row refusal.
13. Set `trace` to the empty string, `null` and a number; expect `malformed_wire`.
14. Round-trip a row with and without `trace` through the row type.
15. Without rows, read Groups `s` and `n` (admitted), `s` and `s`, and `s`,
    `n` and `s`; with one event-position row, read behaviours Groups `s` and
    `s` and finite-domain Groups `t` and `t`; expect
    `ambiguous_declaration`/`ambiguous-name` at the later Group's `variable`
    binding.
16. Give the later Group an alias naming no subject, with a distinct variable
    and then a repeated one; expect `missing-name` at the alias, then
    `ambiguous-name` at the variable binding.

## Expected Results

Each admitted package reads its rows back by key; each defective one refuses
with the stated code, cause and locus.

## Status

Implemented.
