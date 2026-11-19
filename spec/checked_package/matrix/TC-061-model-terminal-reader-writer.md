---
id: TC-061
title: "Read and write checked-package model terminal declarations"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-060
    type: verifies
---

# TC-061: Read and write checked-package model terminal declarations

## Description

Verify [FR-060](../functional/FR-060-admit-model-terminal-declarations.md):
the public V2 reader and writer carry each FSpec terminal mode intact and
refuse malformed declarations in FSpec's order.

## Test Procedure

Use FSpec TC-961 definitions MT-01 through MT-17 as the published oracle.
Build IR-owned V2 wire vectors for `when` with
and without `self`, and `any`, under one selected model. Read each through
the public reader, write the admitted package, then read its output. For
each adverse vector, edit a valid package, rebuild dependencies and
identities except where the vector tests their mismatch, and read it again.
Include FSpec TC-961 MT-17: a `when` condition that is the Boolean
`value`/`literal` node of `true`, then of `false`, and the same condition
retargeted to the `self` parameter node and to an integer literal `3`
(FR-060-AC-7). A second node for one model and a second claim occurrence
on one node are admitted, not refused, and are TC-487's (FR-060-AC-8). Add
two-defect vectors at each reader-order boundary. Do not place any
FSpec file, schema or copied fixture in this repository.

For the model identity control, select two distinct models `M` and `N` and
retain both nodes and selections in each compared package. Start from an
admitted `any` terminal on `M`, which
has no `self` parameter or condition target. Change only the terminal body's
`model` reference to `N`; rebuild its dependency list, node id and package
identity, leaving the mode, other body bindings and occurrences equal. Read
and write both complete packages independently.

## Expected Results

The constant `true` and `false` conditions are admitted and preserved, the
`self`-parameter condition refuses `ill_typed`/`operator-ineligible` and the
integer literal `ill_typed`/`non-boolean-root`, each at the target. The positive modes preserve the four-binding body, node identities,
dependencies and claim occurrences through read/write/read. The adverse
forms refuse with FR-476's exact code, cause and pointer, with earlier graph
and body-step defects winning over terminal defects. The model identity
control admits both packages, retains distinct complete terminal bodies under structural ids and preserves each through read/write/read.
