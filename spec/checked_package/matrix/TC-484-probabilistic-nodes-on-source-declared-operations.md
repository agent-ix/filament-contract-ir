---
id: TC-484
title: "Probabilistic nodes on source-declared operations"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-369
    type: verifies
---
# TC-484: Probabilistic nodes on source-declared operations

## Description

Verify the probabilistic join of operation references whose `declaring` node is a source-declared object type: the strict read's kind and timed-root checks and the identity-row join's operation, parameter and workload checks (FR-369-AC-25, FR-369-AC-26; FSpec FR-640-AC-6).

## Test Procedure

1. Read FR-369-AC-25's package, admit its source-object identity rows and run the join; take the typed read.
2. Edit a `declaring` to a source `relation`/`population` node; read a variant whose source model has no `dense` time member.
3. Over the admitted package, edit the `operation` to `nudge`, the `parameter` to `j`, a support value to `4`, and drop `O::step` from the workload; join each.
4. Join step 1's package with a budget one visit short of the join's charges.

## Expected Results

- Step 1: admitted by both stages; each declaration keyed by its node.
- Step 2: `invalid_package`/`invalid-value` at `declaring`, at the strict read.
- Step 3: `invalid_package`/`invalid-value` at the edited member, at the identity-row join.
- Step 4: incomplete at the first uncharged visit.
