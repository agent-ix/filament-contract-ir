---
id: TC-491
title: "Function body location rows read and refused at the V2 reader"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: verifies
  - target: ix://agent-ix/filament-contract-ir/FR-019
    type: verifies
  - target: ix://agent-ix/filament-contract-ir/FR-051
    type: verifies
  - target: ix://agent-ix/filament-specification/FR-322-AC-490
    type: references
---
# TC-491: Function body location rows read and refused at the V2 reader

## Description

Verify FR-038-AC-581, FR-038-AC-597, FR-019-AC-29 and FR-051-AC-5 at the public V2 reader and encoder over FSpec's
`checked-term-data-vectors.json` `body_locations` cases, read by reference
from the `FILAMENT_SPECIFICATION_DIR` checkout: the body walk is computed
from the admitted graph, a captured path is kept, and each defect refuses at
its own locus.

## Test Procedure

1. Read BL-01, BL-02, BL-04 (record value) and BL-05 (tuple value) and list the typed `body_locations` rows.
2. Read BL-03, then BL-03 with its occurrence-1 `+` row removed.
3. Read BL-R1 to BL-R6.
4. Read BL-R9, BL-R10 and BL-R11, BL-S5 and BL-S6, and the additional missing-up, unsafe-up and mixed-format location cases.
5. Build BL-01's and BL-03's `body_locations` rows as typed public values, encode them with `encode_checked_package_v2` beside the other four arrays, and read the bytes back.
6. Read the 100,000-term BL-D1 and BL-D2 constructions under linear work limits; compare the least admitting work for n and 2n rows of both location arrays, and read each at that exact limit and one unit short.

## Expected Results

- Step 1: both admitted; BL-01's `sqrt` rows read `[0]` and `[1]`, BL-02's
  changed row reads `[0]`. BL-04 and BL-05 retain paths `[0, 0]`, `[0, 1]` and `[0]` across composite components, with no rows for the composite values.
- Step 2: admitted with two rows for the one `+` node, at its `expression` occurrences 0 and 1, each `term_ordinal` 0; then refused
  `invalid_package`/`invalid-value` at `/checked_term_data/body_locations`.
- Step 3: `invalid_package`/`invalid-value` at, in order,
  `/checked_term_data/body_locations`, the row's `application`, the last row's
  `name`, the last row's `index`, the row's `function`, the added row's
  `application`.
- Step 4: BL-R9, BL-R10 and BL-R11 refuse `invalid_package`/`invalid-value` at the row's `location/up`; all five malformed relative locations refuse `malformed_wire` at the row's `location`.
- Step 5: the encoder writes all five arrays in producer order and the strict
  reader returns the same typed rows.
- Step 6: both deep bodies are admitted with 100,000 rows. Doubling the row count doubles work up to fixed overhead. The exact limit admits and one unit short is incomplete. The public typed view keeps compact `up`/`down` under the architectural ruling, without a whole-path accessor.
