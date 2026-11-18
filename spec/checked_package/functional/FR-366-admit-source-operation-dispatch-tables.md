---
id: FR-366
title: "Admit dispatch tables over source-declared operations"
type: FR
relationships:
  - target: ix://agent-ix/filament-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/filament-contract-ir/FR-362
    type: depends_on
  - target: ix://agent-ix/filament-contract-ir/FR-367
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/ADR-013
    type: references
  - target: ix://agent-ix/filament-spec-language/FR-569
    type: references
  - target: ix://agent-ix/filament-specification/FR-322
    type: references
---

# FR-366: Admit dispatch tables over source-declared operations

## Description

The reader validates a `checked_term_data.dispatch_tables` row against
selected model owners. It SHALL also admit a row whose receiver `context` and
`operation.declaring` are source-declared object type nodes, whose candidate
`selection_key` is the source declaration key of a source-declared type node, and whose candidates'
`operation_body` and `effective_precondition` name source clause function nodes
(FSL ADR-013 QC-30 items 2d, 2f and 4; the row shape is FSpec FR-322).

## Inputs

A dispatch-table row `{application, occurrence, operation, context,
candidates}` whose `context` is a source-declared type, and the source clause
function nodes.

## Outputs

The admitted row through the existing dispatch read, or one typed located
refusal.

## Behavior

The strict reader SHALL join `context` and `operation.declaring` through their
`SourceOwner`, not through a selected model owner. It SHALL require the call's
member declaration to equal `context` as it does for a model row, every
candidate's `selection_key` to be the source declaration key of a
source-declared type of the context's `SourceOwner`,
each `operation_body` to name a clause function node with `declaration`
`[<type>, <operation>, "body"]` and each `effective_precondition`, when
present, to name one with `[<type>, <operation>, "effective_precondition"]`,
every such node carrying the same `SourceOwner`, and candidate order and
witnesses to follow the model row's checks. The refusal code, cause and locus
of each failure SHALL be those of the matching model-row check.

A source candidate's `selection_key` is the source declaration key of
FSpec FR-610 "Declaration keys" and FR-788: `{owner_effective_type: null,
original: {package: I, node: "ix://I/<type>"}}`, with `I` the `identity` of
the context's `SourceOwner` (the rule [FR-365](./FR-365-read-source-declared-operation-effects.md)
applies to `document`) and `<type>` the final `declaration` segment of a
source-declared object type under that same `SourceOwner`. The key is the
exact producer `EffectiveId` of FR-150 (FSpec FR-322-AC-290), never derived
from a bare name: the reader recomputes `ix://I/<type>` from `package` and the
source type it names and compares it with `node` exactly. A bare name, the
checked package's name, an unknown owner or type, and a non-null
`owner_effective_type` each refuse `invalid_package`/`invalid-value` at
`selection_key`, as the model check does. The strict reader checks the
`<operation>` and clause kind of a clause function and its owner, and does not
check `<type>`: `<type>` is the type that declares the effective operation, and
the package carries no operation list or `extends` information from which to
know it. The check of `<type>` and the family check run in the source-object
read of [FR-367](./FR-367-join-source-object-rows-and-field-reads.md).

The candidates of a source context are the context type or one of its
subtypes, since a redefining subtype is a valid candidate. The source-object
read of FR-367 holds the supertype rows and the package's dispatch tables. For
each dispatch table whose `context` is a source-declared type it SHALL map each
candidate's `selection_key.original.node`, whose final segment after `ix://I/` is a `source_identity`, to the row of that
`source_identity` under the context row's authority, and SHALL require that
row to be the context's row or to reach it through `supertypes`. It SHALL
also require the `<type>` of the candidate's `operation_body` and
`effective_precondition`, when present, to name a row that the candidate's row
is, or reaches through `supertypes`. Otherwise it SHALL refuse with
`MismatchedIdentityWitness`, the FR-352 outcome for a stale or mismatched
stage row. This check needs the complete row set, so it ranks after the
missing outcome of FR-367. A row mixing a source-declared `context` and a
model-owned `operation.declaring`, or the reverse, SHALL refuse as a model row
whose context and declaring family differ does. The reader SHALL admit the
three clause-function `declaration` forms of one package, with the
authored-precondition form `[<type>, <operation>, "precondition", <label>]`,
as sibling clause functions under the existing one-holder-per-qualified-name
rule, with no change to the qualified-name identifier rule.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-366-AC-1 | A package with a source type `Order`, a redefining subtype `Sub` and a dispatched call on `Order` whose candidates are `Order` and `Sub` admits, and the dispatch read returns the context, candidate order and clause identities in the model row's order. | Test |
| FR-366-AC-2 | The three clause-function declaration forms for one operation, with an authored-precondition label, admit together and each qualified-name segment is an identifier. | Test |
| FR-366-AC-3 | A candidate `selection_key` whose `original.package` is not the context `SourceOwner`'s `identity`, whose `original.node` is a bare name, the checked package's name or `ix://I/<type>` for a `<type>` that names no source-declared type of that owner, or whose `owner_effective_type` is not null, an `operation_body` or `effective_precondition` naming a node of another declaration form or another operation, and a clause node with a different `SourceOwner` each refuse with the matching model-row check's code, cause and locus; the key cases refuse `invalid_package`/`invalid-value` at `selection_key`. | Test |
| FR-366-AC-4 | A row mixing a source context and a model-owned declaring family, and the reverse, refuses. | Test |
| FR-366-AC-5 | A model-owned dispatch table reads unchanged when a source table is in the same package. | Test |
| FR-366-AC-6 | In the FR-367 read with its supertype rows and the package's dispatch tables, a candidate that is neither the context nor a transitive subtype of it, and a clause function declared under a type that the candidate neither is nor reaches through `supertypes`, each refuse with `MismatchedIdentityWitness`; `Sub` for context `Order` admits, also with `Order`'s clause functions; and a missing row ranks first. | Test |

## Dependencies

- [FR-367](./FR-367-join-source-object-rows-and-field-reads.md) supplies the supertype rows for the family check and the `<type>` check.
- FSpec FR-322 owns the dispatch row.
