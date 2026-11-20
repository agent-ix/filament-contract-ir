---
id: TC-481
title: "Source timed-model clock places join in two stages"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-374
    type: verifies
  - target: ix://agent-ix/filament-contract-ir/FR-367
    type: verifies
---
# TC-481: Source timed-model clock places join in two stages

## Description

Verify FR-374-AC-1 through FR-374-AC-4 and FR-367-AC-7 at the public V2
reader and `SelectedSourceObjects::from_stage_rows` over the packages and
identity rows of FSpec `proposals/checked-package-v2/source-timed-model-vectors.json`,
built in the test.

## Test Procedure

1. Read the `TM-S1` package with no identity row, then join its rows.
2. Read each `TM-SR*` edit; compare code, cause and locus.
3. Read and then join each `TM-SJ*` edit; compare the join's code and locus.
4. Join the FR-367-AC-7 member-entry variants.

## Expected Results

`TM-S1`, `TM-S2` and `TM-S3` read and join; each `TM-SR*` refuses at the
read and each `TM-SJ*` at the join, at the stated locus; each malformed
member entry refuses unknown. Every test binds its criterion with `#[trace]`.

## Status

Partial (QSL-2347); see the matrix row.
