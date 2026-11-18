---
id: TC-477
title: "Checked-package reader admits claim_names and refuses every dangling, duplicate and uncovered row"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-371
    type: verifies
  - target: ix://agent-ix/filament-specification/TC-2011
    type: references
  - target: ix://agent-ix/filament-specification/TC-2012
    type: references
  - target: ix://agent-ix/filament-specification/TC-2015
    type: references
---
# TC-477: Checked-package reader admits claim_names and refuses every dangling, duplicate and uncovered row

## Description

Verify every AC of FR-371 through the public strict V2 reader
(`CheckedPackageV2::read`) and canonical writer over packages built in the test
from the shared V2 fixture. No FSpec vector is copied into this repository.

## Test Procedure

1. Read a package with a state clause, a state-graph claim and a refinement,
   check the rows and the lookup, and write and read the package again
   (FR-371-AC-1).
2. Read two structurally equal state clauses sharing one node with two rows,
   and a package with no claim-bearing node and no member (FR-371-AC-1).
3. Damage one row at a time with an extra member, a missing member, a
   non-identifier name or segment, an empty path, another occurrence role and a
   negative ordinal; add the member as `[]` and as `null`; put a dangling
   row before a malformed one (FR-371-AC-2).
4. Point a row at a missing node, a temporal clause and a missing occurrence;
   end the path in another segment; rename the refinement row
   (FR-371-AC-3).
5. Repeat a name, repeat a key, reverse two rows (FR-371-AC-4).
6. Remove a row and remove the member (FR-371-AC-5).
7. Measure the smallest deciding work limit of packages refused at different
   rows and during coverage (FR-371-AC-6).
8. Rename a claim row to a temporal clause name with a one- and a two-segment
   path, and read distinct names (FR-371-AC-7).
9. Remove a refinement body `name`, read it back, and give two refinement nodes
   one `name` (FR-371-AC-8).
10. Widen, remove, damage, shrink, shift and re-source the `declaration_region`
    of a claim row and of a temporal clause row, and remove the occurrence's
    `source_map` entry (FR-371-AC-9).

## Expected Results

- Each case admits or refuses with the code, cause and primary pointer FR-371
  states, and the structural node locus of the row's declaration where a node
  is named.
- Every reader rule is mutation-checked: deleting its check fails a test.
