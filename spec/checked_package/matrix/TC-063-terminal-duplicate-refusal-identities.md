---
id: TC-063
title: "Terminal duplicate refusal carries both typed identities"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-062
    type: verifies
---

# TC-063: Terminal duplicate refusal carries both typed identities

## Description

Withdrawn with [FR-062](../functional/FR-062-carry-terminal-duplicate-identities.md):
FSpec FR-476 admits several terminal nodes on one model node and several
`claim` occurrences on one terminal node, so there is no terminal-duplicate
refusal whose identities this case could check. TC-487 verifies the
admission (FR-060-AC-8).

## Test Procedure

None. The id is retained and never reused.

## Expected Results

None.
