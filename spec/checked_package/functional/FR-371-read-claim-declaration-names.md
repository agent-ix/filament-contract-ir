---
id: FR-371
title: "Read the claim_names rows of state clauses, refinements and non-temporal claims"
type: FR
relationships:
  - target: ix://agent-ix/filament-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/filament-specification/FR-322
    type: references
  - target: ix://agent-ix/filament-specification/FR-322-AC-457
    type: references
  - target: ix://agent-ix/filament-specification/FR-322-AC-458
    type: references
  - target: ix://agent-ix/filament-specification/FR-322-AC-459
    type: references
  - target: ix://agent-ix/filament-specification/FR-322-AC-200
    type: references
  - target: ix://agent-ix/filament-specification/FR-322-AC-460
    type: references
  - target: ix://agent-ix/filament-specification/FR-322-AC-461
    type: references
---

# FR-371: Read the claim_names rows of state clauses, refinements and non-temporal claims

## Description

The closed V2 package schema SHALL admit FSpec FR-322's `claim_names` member,
and the strict reader SHALL admit its `ClaimNameRow` rows only under the join,
uniqueness, coverage, refinement-name, one-namespace and declaration-region
rules of "Claim declaration names" and "Clause declaration regions" in FSpec
FR-322, so that a consumer holding only package
bytes resolves a claim a request names to one node and occurrence.

## Inputs

A `filament.checked-package/v2` document, supplied to the reader of
[FR-038](./FR-038-consume-checked-package-v2.md), and the optional typed rows
supplied to the canonical writer.

## Outputs

The admitted package exposing each row by reference and a lookup from a
qualified path to its row, or the typed located refusal.

## Behavior

The member `claim_names` is absent exactly when the graph has no
`state`/`state_clause`, `relation`/`refinement` or `claim` node; otherwise it
is a non-empty array of closed rows
`{declaration, occurrence, name, qualified_name, declaration_region}`. `occurrence` is the exact
`{role, ordinal}` key of one `declaration` or `claim` occurrence of the node
`declaration` names. The rows are outside every node identity: they change no
`node_id`, node or `source_map` entry.

The reader SHALL refuse as `malformed_wire` a row with a missing or extra
member at the row, a `name` that is not an identifier at `/name`, a
`qualified_name` that is empty or holds a non-identifier segment at
`/qualified_name`, an `occurrence.role` other than `declaration` or `claim` at
`/occurrence/role`, and an `occurrence.ordinal` that is not a non-negative
integer at `/occurrence/ordinal`. The reader SHALL finish this closed-schema
check of every row before it visits any row, so a malformed later row wins over
a dangling earlier one.

Every `claim_names` row and every `temporal_clause_names` row carries
`declaration_region`, exactly one source region (`source`, `start`, `end`): the
half-open byte interval of the whole source declaration that introduced the
occurrence. The reader SHALL refuse a row without `declaration_region`, or with
one that is not a source region, as `malformed_wire` at
`/claim_names/{i}/declaration_region` or
`/temporal_clause_names/{i}/declaration_region`, and SHALL refuse it at the
member itself, not at a part of it.

The reader SHALL then visit rows in array order and finish every check of a row
before the next:

1. a `declaration` naming no node, or a node that is not a state clause, a
   refinement or a claim, refuses `invalid_package`/`invalid-value` at
   `/claim_names/{i}/declaration`;
2. an `occurrence` that is not one of that node's `declaration` or `claim`
   occurrences refuses the same pair at `/claim_names/{i}/occurrence`;
   directly after the join, a `declaration_region` whose source differs from
   the source of the occurrence's `source_map` region, or which does not contain
   every byte of that region, refuses the same pair at
   `/claim_names/{i}/declaration_region`;
3. a `qualified_name` whose last segment is not `name` refuses the same pair at
   `/claim_names/{i}/qualified_name`, and a refinement row whose `name` differs
   from its node's body `name` refuses the same pair at
   `/claim_names/{i}/name`;
4. a repeated (`declaration`, `occurrence`) key refuses
   `invalid_package`/`conflicting-definition` at the later row, and a row out of
   order by structural `NodeRef` then occurrence key refuses
   `invalid_package`/`invalid-value` at the later row;
5. a `qualified_name` equal to an earlier row's refuses
   `ambiguous_declaration`/`ambiguous-name` at
   `/claim_names/{i}/qualified_name`, and the refusal names the earlier row's
   `/claim_names/{j}/qualified_name` as its package-document position, so both
   rows are named.

After every row has passed, a `declaration` or `claim` occurrence of a state
clause, refinement or claim node with no row, or a missing member in a graph
holding such a node, refuses `invalid_package`/`invalid-value` at
`/claim_names`.

The `temporal_clause_names` rows follow the same region rule: after a row's
occurrence join and before its repeated-key check, a region that is under
another source than the occurrence's `source_map` region, or does not contain
every byte of it, refuses `invalid_package`/`invalid-value` at
`/temporal_clause_names/{i}/declaration_region`. An occurrence with no
`source_map` entry is left to the package-wide source-map check. The region is
provenance: it changes no `node_id` and no node.

Once the rows have passed, `claim_names` and `temporal_clause_names` are two
tables of one claim namespace (FR-322-AC-459). The reader SHALL refuse the
first `claim_names` row whose `name` equals the `name` of a
`temporal_clause_names` row as `ambiguous_declaration`/`ambiguous-name` at
`/claim_names/{i}/name`, and the refusal names the first
`temporal_clause_names` row with that `name`, `/temporal_clause_names/{j}/name`,
as its package-document position.

The reader SHALL charge the finite reader budget one validation visit for each
row in the closed-schema check, one for each occurrence it compares while
joining a row to its node, one for each occurrence the coverage walk inspects,
one for each source-map region the declaration-region check compares, and one
for each row in the one-namespace check.

The refinement body `name` is a required text identifier (FR-322-AC-200). The
reader SHALL refuse a refinement body without a `name`, or with a `name` that
is not text, as a body-schema error at the node body, and SHALL re-encode the
`name` it admits. The reader SHALL admit a refinement row only when its `name`
equals the body `name`. The reader SHALL refuse the later of two refinement
nodes of one package that share a `name` as
`ambiguous_declaration`/`ambiguous-name` at
`/semantic_graph/nodes/{n}/body/name`, and the refusal names the earlier
node's `body/name` as its package-document position.

The reader SHALL expose the admitted rows as `Option<&[ClaimNameRow]>` and a
lookup from a qualified path to its row, and the canonical writer SHALL accept
optional borrowed rows, emit them in the supplied order and omit the member for
`None`. A package read, written and read again SHALL preserve every row.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-371-AC-1 | A package of a unit with the invariant state clause `NotAboveThree` and the state-graph claim `ReachThree` admits with one row each, `qualified_name` `["NotAboveThree"]` and `["ReachThree"]`, in `NodeRef` then occurrence order; two structurally equal state clauses share one node and carry two rows with distinct occurrences; a package with none of these nodes has no member and admits; the rows are exposed by reference, resolved by `claim_name_for_path`, and preserved by write and re-read. | Test (TC-477; FSpec FR-322-AC-457) |
| FR-371-AC-2 | A row with an extra or missing member, a `name` or `qualified_name` segment that is no identifier, an empty `qualified_name`, an `occurrence.role` other than `declaration` or `claim`, or a negative `occurrence.ordinal` refuses `malformed_wire` at the member FR-371 names; the member `[]` or `null` refuses `malformed_wire` at `/claim_names`; a malformed later row wins over a dangling earlier row. | Test (TC-477; FSpec FR-322-AC-458) |
| FR-371-AC-3 | A row whose `declaration` names no node or a `temporal_clause` node refuses `invalid-value` at `/claim_names/{i}/declaration`; an `occurrence` the node lacks refuses at `/claim_names/{i}/occurrence`; a `qualified_name` whose last segment is not `name` refuses at `/claim_names/{i}/qualified_name`; a refinement row whose `name` differs from the body `name` refuses at `/claim_names/{i}/name`. | Test (TC-477; FSpec FR-322-AC-458) |
| FR-371-AC-4 | Two rows with one `qualified_name` refuse `ambiguous_declaration`/`ambiguous-name` at the later row's `qualified_name` and name the earlier row's `qualified_name` as the document position; a repeated key refuses `conflicting-definition` at the later row; a row out of order refuses `invalid-value` at the later row; the repeated key wins over the equal name within one row pair. | Test (TC-477; FSpec FR-322-AC-458) |
| FR-371-AC-5 | A claim-bearing node occurrence with no row, and a package holding such a node with no member, each refuse `invalid-value` at `/claim_names`; a row defect wins over missing coverage. | Test (TC-477; FSpec FR-322-AC-458) |
| FR-371-AC-6 | The closed-schema check charges one validation visit per row, the row join one per occurrence compared, the coverage walk one per occurrence inspected, and the one-namespace check one per row: the declaration-region check one per source-map region compared: the smallest work limit that decides a package differs by exactly these counts between packages refused at different rows. | Test (TC-477; FSpec FR-322-AC-458, FR-322-AC-459, FR-322-AC-460) |
| FR-371-AC-7 | A `claim_names` row renamed to the `name` of a `temporal_clause_names` row, with `qualified_name` `[name]` or `["Scope", name]`, refuses `ambiguous_declaration`/`ambiguous-name` at `/claim_names/{i}/name` and names `/temporal_clause_names/{j}/name`; with distinct names the package admits, the claim name resolves by `claim_name_for_path` and the clause name resolves to its `temporal_clause_names` row. | Test (TC-477; FSpec FR-322-AC-459) |
| FR-371-AC-8 | A refinement body without `name` refuses as a body-schema error; the body `name` survives write and re-read; two refinement nodes of one package with an equal `name` refuse `ambiguous_declaration`/`ambiguous-name` at the later node's `/semantic_graph/nodes/{n}/body/name` and name the earlier node's `body/name`. | Test (TC-477; FSpec FR-322-AC-200) |
| FR-371-AC-9 | A name row of either table carries `declaration_region`: a region wider than the name token admits, is exposed beside the name, survives write and re-read and moves no node; a row without it, or with a region that is not a source region, refuses `malformed_wire` at `/{table}/{i}/declaration_region`; a region under another source, or not containing the occurrence's `source_map` region, refuses `invalid-value` at that member after the occurrence join and before the repeated-key check; an occurrence with no `source_map` entry is left to the source-map check. | Test (TC-477; FSpec FR-322-AC-460, FR-322-AC-461) |

## Compatibility

This change is a breaking public-API change of the `filament-contract-model`
crate and SHALL ride the next contract-ir tag as a breaking version bump, not a
patch. `CheckedPackageSidecarsRef` gains the field `claim_names`, and
`CheckedTerm::Refinement` gains the required field `name`; `TemporalClauseName`
and the new `ClaimNameRow` gain the required field `declaration_region`
(QSL-2433, FSpec FR-322-AC-460 and FR-322-AC-461). The FSL package
writer (QSL-2402) SHALL write `claim_names`, the refinement body `name` and the
`declaration_region` of every name row from the checked declaration's span
before any consumer reads its packages under the new tag, because the strict
reader refuses a package with a claim-bearing node and no `claim_names` member.
A consumer that re-encodes a package it read SHALL pass `claim_names()` back to
the writer, or its own re-read is refused.

Consumer sites that construct `CheckedPackageSidecarsRef` and fail to compile at
their next bump, as listed by the review of this change at their origin/main:

- filament-contract-codegen: `src/oracle/equality/mod.rs` (2 sites),
  `src/oracle/function/owner_path.rs`, `tests/it/admitted_composite_equality.rs`.
- filament-spec-language: `fsl-package/src/emit.rs` (2 sites),
  `fsl-package/src/emit/monitor_plan.rs`, `src/population_fixtures.rs`,
  `tests/it/emit_checked_v2.rs`, `fsl-package/src/emit/tests/operation_effects.rs`.

filament-driver builds no sidecar literal. A consumer that builds a
`TemporalClauseName` literal also fails to compile for the new field. FSpec
publishes no `claim_names` rows, no `declaration_region` on its
`temporal_clause_names` rows, and no TC-2011, TC-2012 or TC-2015 vectors in its
checked-package fixtures; the conformance harnesses of this repository derive
the rows and regions for a published package refused only for what it lacks,
and that derivation is removed when FSpec publishes them.

## Dependencies

- FR-038 owns strict admission; FSpec FR-322 owns the wire shape and rules.
