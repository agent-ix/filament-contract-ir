---
id: TC-486
title: "Body wrapper over a model root"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: verifies
---
# TC-486: Body wrapper over a model root

## Description

Verify that the strict V2 reader admits a hyper body wrapper whose `over` is the `model_root` node of its first subject when the wrapper's formula reads no slot parameter, and refuses a model-root `over` everywhere else (FR-038-AC-562, FR-038-AC-563; FSpec FR-402-AC-11).

## Test Procedure

1. Read a package holding the object-free `Whole` with its wrapper `over` the vault `model_root` node.
2. Edit the `NonInterference` wrapper's `over` to the model root; edit `Whole`'s to the root of a selected package no subject names; edit a single-trace clause's `over` to a model root; read each.
3. Edit `Whole`'s `over` to a node outside its `dependencies` and read.

## Expected Results

- Step 1: admitted; the hyper rows read back keyed by the `hyperproperty` node.
- Step 2: each `invalid_model_binding`/`malformed-declaration` at `/semantic_graph/nodes/{w}/body/arguments/0`: the first and third edits at the temporal step, the second at the hyperproperty step (FSpec FR-402 "Reader order" step 3).
- Step 3: `missing_declaration`/`missing-name` at the same pointer.
