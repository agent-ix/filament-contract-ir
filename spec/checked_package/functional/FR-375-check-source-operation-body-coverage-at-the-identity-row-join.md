---
id: FR-375
title: "Check source operation-body coverage at the identity-row join"
type: FR
relationships:
  - target: ix://agent-ix/filament-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/filament-contract-ir/FR-367
    type: depends_on
  - target: ix://agent-ix/filament-specification/FR-790
    type: references
---

# FR-375: Check source operation-body coverage at the identity-row join

## Description

The strict read joins each source `operation_bodies` row and each source
dispatch candidate to the package (the source operation-body admission of
FSpec FR-790), and cannot check coverage, because the package lists no
source type's operations. The source-object identity rows list them
(`SelectedSourceObjectRow::operations`, FR-367). When the package carries
`operation_bodies`, `SelectedSourceObjects::from_stage_rows` SHALL require,
for every source-declared object type, exactly one carrier row whose
`declaring` is the type's node for each entry of its row's `operations`,
with `operation` the entry's `<operation>` segment, and no other row with
that `declaring` (FSpec FR-790 "Stages").

## Inputs

The admitted package's `operation_bodies` carrier and the joined
source-object rows.

## Outputs

The joined rows, or one `SelectedSourceObjectError`.

## Behavior

The check SHALL run after the FR-374 clock join, in the reason order's last
place. A missing row SHALL be `SelectedSourceObjectErrorCode::InvalidValue`
with locus `/operation_bodies`; a row naming an operation its type's row does
not list SHALL be the same code with locus `/operation_bodies/{k}/operation`;
the first failure in (type node ascending, entry order) then row order is
reported. A package with no `operation_bodies` carrier, and a model-owned
row, are unchanged.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-375-AC-1 | Over FSpec FR-790-AC-1's package with identity rows listing `Order` `size` then `bump` and `Sub` `size`, `from_stage_rows` joins; with `Order.bump`'s row removed (the read admits it) it refuses `InvalidValue` at `/operation_bodies`, and with an added `Order.ghost` `no_body` row at `/operation_bodies/{k}/operation`. | Test (TC-482) |

## Dependencies

- [FR-367](./FR-367-join-source-object-rows-and-field-reads.md): the
  `operations` member and the reason order.
- FSpec FR-790 owns the carrier and the stage split.
