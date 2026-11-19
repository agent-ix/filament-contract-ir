---
id: TC-488
title: "Requirement records read from an admitted package"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-376
    type: verifies
---

# TC-488: Requirement records read from an admitted package

## Description

Verify [FR-376](../functional/FR-376-admit-the-requirement-records-of-a-checked-package.md):
the reader admits FSpec FR-825's `requirement_items` rows and
`claim`/`value_validity` nodes, refuses each defect with FR-825's code, cause
and pointer in FR-322's check order, and exposes the records through
`requirement_items()` and `value_validity_records_of()` in package order.
Scope: FR-376-AC-1 to FR-376-AC-3.

## Test Procedure

1. Read FSpec TC-2053's packages of FR-825-AC-1 to FR-825-AC-3 through the
   public reader; call `requirement_items()` and `value_validity_records_of()`
   for `f`, `g` and `inc`. Write each admitted package, read the output and
   call the reads again. Reorder the graph nodes, rebuild operand rows and
   identities, and read again.
2. Apply each mutation of FSpec FR-825-AC-4 and FR-825-AC-5 and build
   FR-322-AC-473's refusing packages; read each. Read FSpec FR-825-AC-5's package with no `claim_names` and FR-322-AC-473's package emitted with records, and compare the latter with its golden written without records.
3. Build the package with a probabilistic rule-6 defect and a record defect,
   and the package with a record defect and a reversed `probabilistic_claims`
   row; read FR-825-AC-1's package with exactly its charged count and with a budget one visit short of its
   charged count, and record the reach visits charged for `f`.

## Expected Results

- Step 1: the rows and records FR-376-AC-1 states, equal after write and
  re-read and after graph reordering; `value_validity_records_of(f)` is
  `[0, 1]` and `g` gives the empty slice.
- Step 2: each refusal has the exact code, cause and pointer FR-825 or
  FR-322 gives, and no refusing package is admitted; the no-`claim_names` package
  and the package with records are admitted, every node of the golden
  present byte-identical.
- Step 3: the refusal at the probabilistic defect, then at the record
  defect; admission at exactly the charged count and `incomplete` at
  `/requirement_items/1` one visit short; `f`'s body reach
  visits charged once for its two records.
