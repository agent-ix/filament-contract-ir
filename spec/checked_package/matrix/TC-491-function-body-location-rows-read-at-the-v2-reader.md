---
id: TC-491
title: "Function body location rows read and refused at the V2 reader"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: verifies
  - target: ix://agent-ix/filament-specification/FR-322-AC-490
    type: references
---
# TC-491: Function body location rows read and refused at the V2 reader

## Description

Verify FR-038-AC-581 at the public V2 reader over FSpec's
`checked-term-data-vectors.json` `body_locations` cases, read by reference
from the `FILAMENT_SPECIFICATION_DIR` checkout: the body walk is computed
from the admitted graph, a captured path is kept, and each defect refuses at
its own locus.

## Test Procedure

1. Read BL-01 and BL-02 and list the typed `body_locations` rows.
2. Read BL-03, then BL-03 with its second `+` row removed.
3. Read BL-R1 to BL-R6.
4. Read BL-01 with its `checked_term_data` removed, and BL-01 with one row's
   origin `kind` set to `type_declaration`.

## Expected Results

- Step 1: both admitted; BL-01's `sqrt` rows read `[0]` and `[1]`, BL-02's
  changed row reads `[0]`.
- Step 2: admitted with two rows for the one `+` node; then refused
  `invalid_package`/`invalid-value` at `/checked_term_data/body_locations`.
- Step 3: `invalid_package`/`invalid-value` at, in order,
  `/checked_term_data/body_locations`, the row's `application`, the last row's
  `name`, the last row's `index`, the row's `function`, the added row's
  `application`.
- Step 4: refused at `/checked_term_data/body_locations`; `malformed_wire` at
  that `kind`.
