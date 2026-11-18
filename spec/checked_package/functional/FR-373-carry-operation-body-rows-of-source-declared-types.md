---
id: FR-373
title: "Admit operation-body rows of source-declared types"
type: FR
relationships:
  - target: ix://agent-ix/filament-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/filament-contract-ir/FR-365
    type: depends_on
  - target: ix://agent-ix/filament-contract-ir/FR-366
    type: depends_on
  - target: ix://agent-ix/filament-specification/FR-322
    type: references
  - target: ix://agent-ix/filament-specification/FR-790
    type: references
---

# FR-373: Admit operation-body rows of source-declared types

## Description

The `operation_bodies` carrier of [FR-038](./FR-038-consume-checked-package-v2.md)
joins each row to a selected model owner's operation member. The reader SHALL
also admit a row whose `declaring` is a source-declared object type node, as
FSpec FR-322 "Source-declared object types" item 9 and FR-790 state it, joining
the row through the node's `SourceOwner` and the clause function's
`declaration` and not through a `ModelOwner` member identity. The model-owned
rules are unchanged.

## Inputs

An `operation_bodies` row `{document, declaring, operation, body}` whose
`declaring` names a source-declared object type, the package's source clause
function nodes, and its dispatch tables.

## Outputs

The admitted row through the existing borrowed typed read, or one typed located
refusal.

## Behavior

For a source row the reader SHALL:

1. require `document` to equal the `identity` of the declaring node's
   `SourceOwner`; a string that is the `identity` of no `SourceOwner` of the
   package refuses `missing_declaration`/`missing-selection` at `document`, and
   another owner's identity refuses `invalid_package`/`invalid-value` at
   `document`;
2. require `body` to be `function` exactly when the package holds a `function`
   node with that `SourceOwner` and the `declaration` `[<type>, <operation>,
   "body"]`, `<type>` being the final segment of the declaring node's
   `declaration` (`invalid_package`/`invalid-value` at `body` otherwise), and
   `body.node` to be that node (`invalid_package`/`invalid-value` at
   `body/node` for a precondition clause function, another operation's body or
   another owner's function);
3. join each source dispatch candidate to the row of the type that its
   `operation_body` is declared under (the first segment of that clause
   function's `declaration`) and the table's operation: the candidate's
   `operation_body` SHALL equal that row's `body.node` and the row SHALL exist
   (`invalid_package`/`invalid-value` at `/operation_bodies` when it does not,
   `invalid_package`/`invalid-value` at `body` when the bodies differ);
4. keep the carrier order and the repeated-key refusal of the model rows.

The package carries no list of a source type's operations, so
`CheckedPackageV2::read` SHALL NOT require a row for an operation that no
dispatch candidate joins and SHALL NOT refuse a row naming an operation the
package holds no evidence of. Coverage is checked at the identity-row join
(FR-375),
which has the `operations` of each source-object row. A candidate whose clause
function is not a source body clause function of its operation is left to the
dispatch validation of FR-366.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-373-AC-1 | A package with source types `Order` (operation `deposit` with body clause function `["Order", "deposit", "body"]`, operation `quote` without a body) and `Sub` (`deposit` redefined, body `["Sub", "deposit", "body"]`), a dispatch table over `Order` whose candidates are `Order` and `Sub`, and an `operation_bodies` carrier of the rows `Order.deposit`, `Order.quote` (`no_body`) and `Sub.deposit`, each `document` the `SourceOwner` `identity`, admits; the typed read returns each row by its key and the rows serialize as written. | Test (TC-480) |
| FR-373-AC-2 | Over the AC-1 package, `Order.deposit` with `no_body`, and `Order.quote` with a `function`, each refuse `invalid_package`/`invalid-value` at `body`. | Test (TC-480) |
| FR-373-AC-3 | Over the AC-1 package, `Order.deposit`'s `body.node` naming its effective precondition clause function, and naming `Sub.deposit`'s body, each refuse `invalid_package`/`invalid-value` at `body/node`. | Test (TC-480) |
| FR-373-AC-4 | Over the AC-1 package, the `Sub.deposit` row removed refuses `invalid_package`/`invalid-value` at `/operation_bodies`; `Order.deposit`'s `document` of an identity of no `SourceOwner` refuses `missing_declaration`/`missing-selection` at `document`; a repeated row refuses `invalid_package`/`conflicting-definition` at the row. | Test (TC-480) |
| FR-373-AC-5 | Over the AC-1 package with `Sub`'s candidate naming `Order.deposit`'s body clause function and the `Sub.deposit` row removed, the package admits: the candidate joins the row of the type its body is declared under. | Test (TC-480) |

## Dependencies

- [FR-365](./FR-365-read-source-declared-operation-effects.md) states the
  `document` rule applied here.
- [FR-366](./FR-366-admit-source-operation-dispatch-tables.md) states the
  source clause function forms and the dispatch row.
- FSpec FR-322-AC-397 and FR-790 own the carrier.
