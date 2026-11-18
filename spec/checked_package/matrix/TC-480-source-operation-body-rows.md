---
id: TC-480
title: "Checked-package reader admits operation-body rows of source-declared types"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-373
    type: verifies
---
# TC-480: Checked-package reader admits operation-body rows of source-declared types

## Description

Verify FR-373-AC-1 through FR-373-AC-5 at the public V2 reader boundary over
the in-repo source-object package of TC-475 with its `Order`/`Sub` dispatch
table. Build every package in the test.

## Test Procedure

1. Add the `operation_bodies` carrier of FR-373-AC-1 and read it.
2. Edit one member per case (FR-373-AC-2 to FR-373-AC-4) and compare the code,
   cause and locus.
3. Point `Sub`'s candidate at `Order`'s body clause function and drop `Sub`'s
   row (FR-373-AC-5).

## Expected Results

The AC-1 package and the AC-5 package admit; each edit refuses at its stated
locus. Every test binds its criterion with `#[trace]`.

## Status

Planned (QSL-2347).
