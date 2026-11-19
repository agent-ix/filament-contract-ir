---
id: FR-062
title: "Carry both terminal duplicate identities in a typed reader refusal"
type: FR
relationships:
  - target: ix://agent-ix/filament-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/filament-contract-ir/FR-060
    type: depends_on
  - target: ix://agent-ix/filament-specification/FR-476
    type: references
---

# FR-062: Carry both terminal duplicate identities in a typed reader refusal

## Description

Withdrawn. This requirement described the typed payload of two reader
refusals: a second `temporal`/`terminal` node on one model node, and a second
`claim` occurrence on one terminal node. FSpec FR-476 no longer refuses
either. A package names a model by its selection identity, so two source
aliases that select one model identity reference one model node, and each
alias may carry its own `terminal` declaration; the reader admits both and
exposes them as separate terminal rows ([FR-060](./FR-060-admit-model-terminal-declarations.md)
"Public terminal rows"). With no duplicate refusal there is nothing for a
typed duplicate payload to carry, so `CheckedTerminalDuplicate` and
`CheckedPackageRefusal::terminal_duplicate` are removed rather than kept
unused. The id is retained and never reused.

## Inputs

None. No reader input produces a terminal-duplicate refusal.

## Outputs

None. `CheckedPackageRefusal` carries no terminal-specific field.

## Behavior

The V2 reader SHALL NOT refuse a package for holding several terminal nodes
on one model node or several `claim` occurrences on one terminal node
(FR-060-AC-8), and SHALL expose no `CheckedTerminalDuplicate` type or
`terminal_duplicate` refusal field.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-062-AC-1 | Withdrawn: FSpec FR-476 admits several terminal nodes on one model node; FR-060-AC-8 covers the admission. This id is retained and never reused. | Withdrawn |
| FR-062-AC-2 | Withdrawn: FSpec FR-476 admits several `claim` occurrences on one terminal node; FR-060-AC-8 covers the admission. This id is retained and never reused. | Withdrawn |
| FR-062-AC-3 | Withdrawn with the duplicate refusals it ordered. This id is retained and never reused. | Withdrawn |
| FR-062-AC-4 | Withdrawn with the public duplicate payload type. This id is retained and never reused. | Withdrawn |

## Dependencies

- [FR-060](./FR-060-admit-model-terminal-declarations.md) owns terminal
  admission and the terminal rows.
- `ix://agent-ix/filament-specification/FR-476` owns the rule that several
  terminal nodes and occurrences on one model node are admitted.

## Status

Withdrawn. Any implemented duplicate refusal and its payload type are removed
with FR-060-AC-8.
