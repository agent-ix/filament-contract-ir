---
id: TC-482
title: "Source operation-body coverage joins the identity rows"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-375
    type: verifies
---
# TC-482: Source operation-body coverage joins the identity rows

## Description

Verify FR-375-AC-1 through `SelectedSourceObjects::from_stage_rows` over
FSpec FR-790-AC-1's package with identity rows listing the operations.

## Test Procedure

1. Join the complete carrier.
2. Remove `Order.bump`'s row; add an `Order.ghost` row; join each.

## Expected Results

Step 1 joins; each edit is admitted by the read and refuses `InvalidValue`
at the stated locus. Every test binds its criterion with `#[trace]`.

## Status

Implemented (QSL-2347).
