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

Verify the probabilistic join of operation references whose `declaring` node is a source-declared object type: the strict read's kind, timed-root and reward-function checks and the identity-row join's operation, parameter, workload and reward-parameter checks (FR-369-AC-25, FR-369-AC-26; FSpec FR-640-AC-6, FR-640-AC-7).

The golden pair is a checked-in fixture directory holding FR-369-AC-25's package and the expected typed read of its declarations. The test reads the package, compares the typed read with the golden one, and compares the reward function's id with the declared-node id `{package, ["Other", "poke", "reward", "cost"], 0}` the test builds itself, so a reader that resolves the reward through another node, keys it by an ordinal, or drops it from the typed read fails.

## Test Procedure

1. Read FR-369-AC-25's golden package, admit its source-object identity rows and run the join; take the typed read and compare it with the golden typed read; build the reward function's declared-node id and compare it with the function the reward names.
2. Edit a `declaring` to a source `relation`/`population` node; read a variant whose source model has no `dense` time member.
3. Over the admitted package, edit the `operation` to `nudge`, the `parameter` to `j`, a support value to `4`, and drop `O::step` from the workload; join each.
4. Over the admitted package, rewrite the reward function's `declaration` to `["Other", "poke", "reward", "load"]`, `["Other", "step", "reward", "cost"]` and `["O", "poke", "reward", "cost"]`; rewrite its `clause` to `"precondition"`; give it a `SourceOwner` of another `identity`; read each strictly.
5. Over the admitted package, give the reward function the parameter `j` (read by the body) in place of `k`, and no parameter with the constant body `1`; rewrite the reward's `operation` to `nudge`; join each.
6. Join step 1's package with a budget one visit short of the join's charges, the reward's parameter comparison included.

## Expected Results

- Step 1: admitted by both stages; each declaration keyed by its node; the typed read equals the golden one, the reward `cost` with value type `Int[1, 3]` and the function whose id equals the built declared-node id.
- Step 2: `invalid_package`/`invalid-value` at `declaring`, at the strict read.
- Step 3: `invalid_package`/`invalid-value` at the edited member, at the identity-row join.
- Step 4: each `invalid_package`/`invalid-value` at the reward's `/body/members/4/value/target`, at the strict read.
- Step 5: the two parameter edits each `invalid_package`/`invalid-value` at the reward's `/body/members/4/value/target`, and the `nudge` edit at the reward's `operation`, at the identity-row join.
- Step 6: incomplete at the first uncharged visit.
