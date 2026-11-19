---
id: TC-489
title: "Probabilistic claim node read from an admitted package"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-377
    type: verifies
---

# TC-489: Probabilistic claim node read from an admitted package

## Description

Verify [FR-377](../functional/FR-377-admit-the-probabilistic-claim-node-and-what-it-states.md):
the reader admits FSpec FR-640's nine-binding `claim`/`probabilistic_claim`
body, refuses each defect with FR-640 rule 6 or rule 7's code, cause and
pointer in that order, charges the reader budget per binding, and exposes each
claim through `probabilistic_claims()`. Scope: FR-377-AC-1 to FR-377-AC-3.

## Test Procedure

1. Read FSpec TC-1563's packages of FR-640-AC-8 and FR-640-AC-10 through the
   public reader and call `probabilistic_claims()`; read each spelling and
   each identity-change package of FR-640-AC-10 and compare the entries and
   their `node`s with the first package's. Write each admitted package, read
   the output and call the read again. Reorder the graph nodes, rebuild
   operand rows and identities, and read again.
2. Apply each mutation of FSpec FR-640-AC-9 and FR-640-AC-11 and read each;
   read FR-640-AC-11's path-rewrite package.
3. Build a package with a rule-7 defect and a requirement-record defect
   (FSpec FR-825-AC-1's function added); read FR-640-AC-8's package with
   exactly its charged count and with a budget one visit short.

## Expected Results

- Step 1: the entries FR-377-AC-1 states, equal across spellings, write and
  re-read and graph reordering, and with another `node` for each identity
  change.
- Step 2: each refusal has the exact code, cause and pointer FR-640 or FR-322
  gives, and no refusing package is admitted; the path rewrite is admitted.
- Step 3: the refusal at the rule-7 defect; admission at exactly the charged
  count and `incomplete` at `P95`'s `confidence` `indifference` member one
  visit short.
