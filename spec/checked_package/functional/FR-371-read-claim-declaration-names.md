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
rules of "Claim declaration names", "Rows of one declaration" and "Clause
declaration regions" in FSpec FR-322, so that a consumer holding only package
bytes resolves a claim a request names to one declaration, its own node and its
claim row.

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
occurrence. The reader SHALL refuse a row without `declaration_region` as
`malformed_wire` at the row, `/claim_names/{i}` or
`/temporal_clause_names/{i}`, as the closed schema refuses any missing member,
and SHALL refuse a `declaration_region` that is not a source region as
`malformed_wire` at the member, `/claim_names/{i}/declaration_region` or
`/temporal_clause_names/{i}/declaration_region`, not at a part of it.

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
5. rows are told apart by declaration, not by row: a row's declaration key is
   its `declaration_region`, compared as (`source`, `start`, `end`). A row whose
   declaration key equals an earlier row's and whose `name` or `qualified_name`
   differs from it refuses `invalid_package`/`invalid-value` at
   `/claim_names/{i}/name`: one declaration has one name;
6. a `qualified_name` equal to the `qualified_name` of an earlier row of
   another declaration (a different declaration key) refuses
   `ambiguous_declaration`/`ambiguous-name` at
   `/claim_names/{i}/qualified_name`, and the refusal names the earlier row's
   `/claim_names/{j}/qualified_name` as its package-document position, so both
   rows are named. Rows of one declaration are never duplicates of each other.

After every row has passed, a `declaration` or `claim` occurrence of a state
clause, refinement or claim node with no row, or a missing member in a graph
holding such a node, refuses `invalid_package`/`invalid-value` at
`/claim_names`.

Then, for each `qualified_name` in the order of its first row, a name whose rows
name two or more nodes needs an own node: the node, among those its rows name,
whose declaration set (the declaration keys of its rows) is strictly contained
in the declaration set of every other of them. A name with none refuses
`ambiguous_declaration`/`ambiguous-name` at the `qualified_name` of the name's
last row, `/claim_names/{i}/qualified_name`, and names its first row's
`/claim_names/{j}/qualified_name` as its package-document position. A name whose
rows name one node has that node as its own node.

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

A hyper clause is named by its `hyperproperty` node: the `temporal_clause` node
that a `claim`/`hyperproperty` node's `body` binding references is the clause's
body wrapper (FSpec FR-402), and it has no `temporal_clause_names` row. The
reader SHALL require no row for a body wrapper's `claim` occurrences, and SHALL
refuse a `temporal_clause_names` row whose `declaration` is a body wrapper as
`invalid_package`/`invalid-value` at `/temporal_clause_names/{i}/declaration`.
The hyper clause's name rows are the `claim_names` rows of its `hyperproperty`
node, one per `claim` occurrence, and the one-namespace rule below holds
whatever the regions of two such rows.

The reader SHALL charge the finite reader budget one validation visit for each
row in the closed-schema check, one for each occurrence it compares while
joining a row to its node, one for each occurrence the coverage walk inspects,
one for each source-map region the declaration-region check compares, two for
each row in the own-node check (one to build the node sets and one to group the
rows by name) and one for each element of the declaration sets it compares, and
one for each row in the one-namespace check.

The refinement body `name` is a required text identifier (FR-322-AC-200). The
reader SHALL refuse a refinement body without a `name` as a body-schema error
at the node body, and a `name` that is not text, is empty, or is not an
identifier (a qualified name such as `Impl::CasRefinesCounter` is none) as a
body-schema error at `/semantic_graph/nodes/{n}/body/name`, and SHALL re-encode the
`name` it admits. The reader SHALL admit a refinement row only when its `name`
equals the body `name`. The reader SHALL refuse the later of two refinement
nodes of one package that share a `name` as
`ambiguous_declaration`/`ambiguous-name` at
`/semantic_graph/nodes/{n}/body/name`, and the refusal names the earlier
node's `body/name` as its package-document position.

The reader SHALL expose the admitted rows as `Option<&[ClaimNameRow]>` and a
lookup from a qualified path to the claim row of the one declaration that
carries it: among the rows of the declaration's own node that carry the path
(structurally equal declarations share one node, and each keeps its own rows),
the row of least occurrence key whose role is `claim`, or else the least row,
never a row on a conjunction node that also names the declaration. The
canonical writer SHALL accept optional borrowed rows, emit them in the supplied order and omit the member for
`None`. A package read, written and read again SHALL preserve every row.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-371-AC-1 | A package of a unit with the invariant state clause `NotAboveThree` and the state-graph claim `ReachThree` admits with one row each, `qualified_name` `["NotAboveThree"]` and `["ReachThree"]`, in `NodeRef` then occurrence order; two structurally equal state clauses share one node and carry two rows with distinct occurrences; a package with none of these nodes has no member and admits; the rows are exposed by reference, resolved by `claim_name_for_path`, and preserved by write and re-read. | Test (TC-477; FSpec FR-322-AC-457) |
| FR-371-AC-2 | A row with an extra or missing member, a `name` or `qualified_name` segment that is no identifier, an empty `qualified_name`, an `occurrence.role` other than `declaration` or `claim`, or a negative `occurrence.ordinal` refuses `malformed_wire` at the member FR-371 names; the member `[]` or `null` refuses `malformed_wire` at `/claim_names`; a malformed later row wins over a dangling earlier row. | Test (TC-477; FSpec FR-322-AC-458) |
| FR-371-AC-3 | A row whose `declaration` names no node or a `temporal_clause` node refuses `invalid-value` at `/claim_names/{i}/declaration`; an `occurrence` the node lacks refuses at `/claim_names/{i}/occurrence`; a `qualified_name` whose last segment is not `name` refuses at `/claim_names/{i}/qualified_name`; a refinement row whose `name` differs from the body `name` refuses at `/claim_names/{i}/name`. | Test (TC-477; FSpec FR-322-AC-458) |
| FR-371-AC-4 | Two rows of distinct declarations (different `declaration_region`) with one `qualified_name` refuse `ambiguous_declaration`/`ambiguous-name` at the later row's `qualified_name` and name the earlier row's `qualified_name` as the document position; a repeated key refuses `conflicting-definition` at the later row; a row out of order refuses `invalid-value` at the later row; the repeated key wins over the equal name within one row pair. | Test (TC-477; FSpec FR-322-AC-458) |
| FR-371-AC-5 | A claim-bearing node occurrence with no row, and a package holding such a node with no member, each refuse `invalid-value` at `/claim_names`; a row defect wins over missing coverage. | Test (TC-477; FSpec FR-322-AC-458) |
| FR-371-AC-6 | The reader charges six kinds of work: one validation visit per row for the closed-schema check, one visit per occurrence compared for the row join, one per occurrence inspected for the coverage walk, one per row for the one-namespace check, one per source-map region compared for the declaration-region check, and two per row and one per declaration-set element compared for the own-node check; the smallest work limit that decides a package differs by exactly these counts between packages refused at different rows. | Test (TC-477; FSpec FR-322-AC-458, FR-322-AC-459, FR-322-AC-460) |
| FR-371-AC-7 | A `claim_names` row renamed to the `name` of a `temporal_clause_names` row, with `qualified_name` `[name]` or `["Scope", name]`, refuses `ambiguous_declaration`/`ambiguous-name` at `/claim_names/{i}/name` and names `/temporal_clause_names/{j}/name`; with distinct names the package admits, the claim name resolves by `claim_name_for_path` and the clause name resolves to its `temporal_clause_names` row. | Test (TC-477; FSpec FR-322-AC-459) |
| FR-371-AC-8 | A refinement body without `name` refuses as a body-schema error at the body, and a `name` that is not text, is empty, or is not an identifier refuses as a body-schema error at `body/name`; the body `name` survives write and re-read; two refinement nodes of one package with an equal `name` refuse `ambiguous_declaration`/`ambiguous-name` at the later node's `/semantic_graph/nodes/{n}/body/name` and name the earlier node's `body/name`. | Test (TC-477; FSpec FR-322-AC-200) |
| FR-371-AC-9 | A name row of either table carries `declaration_region`: a region wider than the name token admits, is exposed beside the name, survives write and re-read and moves no node; a row without it refuses `malformed_wire` at the row `/{table}/{i}`, and a row with a region that is not a source region refuses `malformed_wire` at `/{table}/{i}/declaration_region`; a region under another source, or not containing the occurrence's `source_map` region, refuses `invalid-value` at that member after the occurrence join and before the repeated-key check; an occurrence with no `source_map` entry is left to the source-map check. | Test (TC-477; FSpec FR-322-AC-460, FR-322-AC-461) |
| FR-371-AC-10 | Rows with one `declaration_region` are one declaration: a second row of one declaration on the same node, or on another (conjunction) node, repeating its `name` and `qualified_name`, admits and is no duplicate name; the lookup of its name returns the row on its own node, the node whose declaration set is strictly contained in that of the other, of least occurrence key whose role is `claim`, else its least row, never the row on the other node; the same row renamed refuses `invalid-value` at its `/claim_names/{i}/name`; a name whose two nodes have declaration sets that do not nest refuses `ambiguous_declaration`/`ambiguous-name` at its last row's `qualified_name` and names its first row's. | Test (TC-477; FSpec FR-322-AC-465) |
| FR-371-AC-11 | A hyper clause's body wrapper has no `temporal_clause_names` row and the package admits without one; a `temporal_clause_names` row whose `declaration` is a body wrapper refuses `invalid-value` at `/temporal_clause_names/{i}/declaration`; a `claim_names` row and a `temporal_clause_names` row with one `name` refuse `ambiguous_declaration`/`ambiguous-name` at `/claim_names/{i}/name` even when their `declaration_region`s are equal. | Test (TC-477; FSpec FR-322-AC-466) |

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
(filament-specification #702) publishes `declaration_region` on the
`temporal_clause_names` rows of its fixtures and in `schema.json`, the vectors
`claim-name-vectors.json` (TC-2011, TC-2012, TC-2015 step 4, replayed by
`tests/conformance_fspec/claim_names.rs`) and `refinement-body-name-vectors.json`
(replayed by `published_refinement_body_name_vectors_replay`). Its fixtures
carry their `claim_names` rows (filament-specification #709) and the vectors'
`base_patch` only renames the temporal clause row, so the conformance harness
derives nothing: each vector is read as published, with its published
expectation.
A removed `declaration_region` refuses at the row in every published vector
(`CN-S9`, `CL-S4`, `CL-S5`, FR-322-AC-461, as of filament-specification #708),
the same locus as `CN-S1` to `CN-S3`.

## Dependencies

- FR-038 owns strict admission; FSpec FR-322 owns the wire shape and rules.
