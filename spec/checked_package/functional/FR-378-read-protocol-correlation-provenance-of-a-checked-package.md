---
id: FR-378
title: "Read the protocol correlation provenance of a checked package"
type: FR
relationships:
  - target: ix://agent-ix/filament-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/filament-contract-ir/FR-358
    type: depends_on
  - target: ix://agent-ix/filament-specification/FR-831
    type: references
  - target: ix://agent-ix/filament-specification/FR-322
    type: references
---

# FR-378: Read the protocol correlation provenance of a checked package

## Description

When a V2 checked package carries the optional `protocol_correlation_occurrences`
member, the Contract IR reader SHALL admit exactly FSpec FR-831's rows and
joins, or refuse the whole package at the first defect in FR-831's reader
order.

The Contract IR canonical writer and the typed read of an admitted package are
specified under Behavior.

## Inputs

The untrusted `filament.checked-package/v2` bytes, evidence and limits of
[FR-038](./FR-038-consume-checked-package-v2.md), and the caller's native join
budget. The row form, the joins and the coverage rule are specified by
`ix://agent-ix/filament-specification/FR-831`; the enclosing package and the
check order by `ix://agent-ix/filament-specification/FR-322`.

## Outputs

An admitted package retaining every row, or one typed refusal with code, cause
when defined and RFC 6901 pointer, or the reader's budget result. No partial
package.

## Behavior

The member is an optional top-level array, never `null`. Each row is closed:
`control`, `occurrence`, `canonical_index`, `alias` and `source_ordinal`.

### Row pass

The reader SHALL take each row, in array order, through its charge, its joins,
its key and its ordinal, in that order, and SHALL refuse at the first defect:

- the charge is `1 + B + L + Bp` work, made on the one shared native join
  budget before the row is inspected, where `B` is the row's control bytes,
  `L` the row's alias bytes and `Bp` the previous row's control bytes;
- the joins are, in order: the control exists, the control's kind and operator
  are eligible (`send`, `receive`, `attempt`, `effect`, `event`, `recommend`),
  the `occurrence` names a declaration occurrence with a `source_map` entry,
  and `canonical_index` is below the length of the control's `related`
  argument;
- the key `(control, occurrence.ordinal, source_ordinal)` is strictly
  ascending in node-id order, and an equal key is a conflicting definition,
  checked before the order;
- the first row of a group has `source_ordinal` 0 and each next row of the group
  is the previous plus one.

### View and coverage pass

After the last row passes, the reader SHALL charge `read_storage` for one
block of `G` bytes (the number of distinct `(control, occurrence)` groups times
the native group record, which holds the row span and the position of the
whole `source_map` entry) before requesting it; a member with no row requests
none. It SHALL then walk every `protocol`/`control` node in ascending node-id
order over the node index the strict read already retains, charging `work`
before each node: `1 + 2·Bn` for a node that is not eligible and
`1 + 2·Bn + O + m` for an eligible one, and for an eligible node with `m >= 1`
charging `m` bytes of `read_storage` for its coverage flags before requesting
them. The reader builds no scan, index or table of its own outside these passes.

The reader SHALL refuse, at `/protocol_correlation_occurrences` and retaining
the control, occurrence and first uncovered `canonical_index`, every eligible
control occurrence with `m >= 1` whose canonical positions `0..m` are not all
named by the rows of its group, a group with no row included. A present `[]` is
therefore valid exactly when every eligible control occurrence has `m = 0`.

### Budget order

A package that carries the member reserves the strict read's result box
before the member's step, so that the step's storage limits, stated over a
baseline that already holds it, admit on the public strict read with no growth
of the limit. A package without the member keeps the result box as the read's
last storage request. A denied charge returns the reader's `incomplete`
result naming the counter and the full amount attempted, at
`/protocol_correlation_occurrences/{i}` for a row charge and at
`/protocol_correlation_occurrences` for the view, node and flag charges; an
absent member charges nothing.

### Absent member and dependencies

An absent member is not an empty provenance. A present `[]` over controls with
no `related` records is a complete empty table. The reader SHALL read a
dependency package from the dependency's own member and `source_map`, never the
root's, and a dependency without a member is unavailable whatever the root
carries.

### Writer

The canonical writer SHALL write `None` as no member, `Some(&[])` as `[]` and
`Some(rows)` as exactly the rows it is given, in the order it is given them,
without sorting, deduplicating or deriving rows.

### Public read

The typed read of an admitted package SHALL distinguish a complete table from
a typed `unavailable_correlation_provenance` capability:

```rust
CheckedPackageV2::protocol_correlation_provenance(&self)
    -> ProtocolCorrelationProvenance<'_>
// Complete(ProtocolCorrelationTable) | Unavailable(UnavailableCorrelationProvenance)
CheckedPackageV2::protocol_correlation_clauses(&self, control: &CheckedNodeId)
    -> Result<&[ProtocolCorrelationRow], ProtocolCorrelationReadError>
ProtocolCorrelationTable::group(&self, control: &CheckedNodeId, ordinal: u64)
    -> Option<ProtocolCorrelationGroup<'_>>
```

A `ProtocolCorrelationTable` yields its groups in key order. Each group names
the control and occurrence, borrows the control's whole `source_map` entry
with every region, and borrows its clauses in `source_ordinal` order.
`group` returns `None` for a control and ordinal that name no group.
`protocol_correlation_clauses` returns `ProtocolCorrelationReadError::
NotAnEligibleControl` for a node the package does not hold, a node of another
package and a node that is not a `protocol`/`control` of an eligible operator.
A consumer needing clause order for a control with a nonempty `related`
argument and no member gets `unavailable_correlation_provenance`; a control
with empty `related` needs none.

### Limits

The reader checks that the rows are consistent with the package. It cannot
authenticate the spelling of an alias, which names no relationship, and it
cannot detect trailing duplicate clauses a producer omitted. Neither is a
refusal. This requirement makes no claim that a reader without it admits a
package carrying the member: such a reader refuses the member as an unknown
top-level member.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-378-AC-1 | The reader admits FSpec TC-2113's twelve positive packages and the typed read returns each case's published table or `unavailable_correlation_provenance`: an absent member over empty and nonempty `related`, a present `[]`, every one of the six related-bearing forms, identical clauses three times, two aliases of one record, a repeated alias over two records, reversed source order, an omitted trailing duplicate (admitted, since the reader cannot detect it), a dependency answering from its own member, a root without a member over a dependency with a member (the dependency reads its own table and the root is unavailable), and a root with a member over a dependency without. A group's `source_map` entry carries every region of the entry. | Test (TC-493) |
| FR-378-AC-2 | Each of FSpec TC-2113's twenty-one reader refusals refuses with its exact code, cause and pointer, in the joins' order, with the coverage refusal at `/protocol_correlation_occurrences` retaining the control, occurrence and first uncovered index, for an eligible occurrence with `m >= 1` whether its group has no row or too few; each schema decision (`null`, non-array, unknown row member, non-integer or oversized integer, non-identifier alias) refuses `malformed_wire` at the first error; no refusal admits a partial package. | Test (TC-493) |
| FR-378-AC-3 | A control with a nonempty `related` argument and no member yields `unavailable_correlation_provenance` from the clause read; a control with empty `related` and no member yields the empty clause list. `protocol_correlation_clauses` returns `NotAnEligibleControl` for an absent node, a foreign-package node and each ineligible operator, `group` returns `None` for them and for an unnamed occurrence ordinal, and an eligible control returns its clauses in source order. | Test (TC-493) |
| FR-378-AC-4 | The sixteen published budget vectors run on the public strict read with their limits stated as the baseline plus the published offset, `G` being the published group count times the native group record, with no growth of any limit: each admits at the stated limit, including CO-B11 at `B_storage + G + 7`, and each denial names the published first unpaid charge, amount and locus. A package without the member keeps the result box as the read's last storage request. | Test (TC-493) |
| FR-378-AC-5 | The writer writes `None` as no member, `Some(&[])` as `[]` and `Some(rows)` as exactly those rows in exactly that order, with no sort, deduplication or derivation, and the rows read back equal. For each of FSpec TC-2113's invariance pairs, the packages differ in the member alone: every other member is equal, the package bytes differ, and only the provenance read differs. | Test (TC-493) |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns the V2 reader, writer,
  identity and shared refusal machinery.
- [FR-358](./FR-358-compose-native-identity-join-budget.md) owns the native join budget the charges are made on.
- `ix://agent-ix/filament-specification/FR-831` owns the row form, the joins,
  the coverage rule and the budget charges.

## Status

Implemented by the QSL2271 correlation provenance reader.
