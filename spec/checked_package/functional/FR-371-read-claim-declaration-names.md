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
---

# FR-371: Read the claim_names rows of state clauses, refinements and non-temporal claims

## Description

The closed V2 package schema SHALL admit FSpec FR-322's `claim_names` member,
and the strict reader SHALL admit its `ClaimNameRow` rows only under the join,
uniqueness, coverage and refinement-name rules of "Claim declaration names" in
FSpec FR-322, so that a consumer holding only package bytes resolves a claim a
request names to one node and occurrence.

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
`{declaration, occurrence, name, qualified_name}`. `occurrence` is the exact
`{role, ordinal}` key of one `declaration` or `claim` occurrence of the node
`declaration` names. The rows are outside every node identity: they change no
`node_id`, node or `source_map` entry.

The reader SHALL refuse as `malformed_wire` a row with a missing or extra
member at the row, a `name` that is not an identifier at `/name`, a
`qualified_name` that is empty or holds a non-identifier segment at
`/qualified_name`, an `occurrence.role` other than `declaration` or `claim` at
`/occurrence/role`, and an `occurrence.ordinal` that is not a non-negative
integer at `/occurrence/ordinal`.

The reader SHALL visit rows in array order and finish every check of a row
before the next:

1. a `declaration` naming no node, or a node that is not a state clause, a
   refinement or a claim, refuses `invalid_package`/`invalid-value` at
   `/claim_names/{i}/declaration`;
2. an `occurrence` that is not one of that node's `declaration` or `claim`
   occurrences refuses the same pair at `/claim_names/{i}/occurrence`;
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
   `/claim_names/{i}/qualified_name`.

After every row has passed, a `declaration` or `claim` occurrence of a state
clause, refinement or claim node with no row, or a missing member in a graph
holding such a node, refuses `invalid_package`/`invalid-value` at
`/claim_names`. Each visited row consumes one validation visit from the
caller's finite reader budget.

The reader SHALL admit the member `name` of a refinement body, one text identifier, and re-encode it. The reader SHALL admit a refinement row only when the body carries a `name` equal to the row's `name`.

The reader SHALL expose the admitted rows as `Option<&[ClaimNameRow]>` and a
lookup from a qualified path to its row, and the canonical writer SHALL accept
optional borrowed rows, emit them in the supplied order and omit the member for
`None`. A package read, written and read again SHALL preserve every row.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-371-AC-1 | A package of a unit with the invariant state clause `NotAboveThree` and the state-graph claim `ReachThree` admits with one row each, `qualified_name` `["NotAboveThree"]` and `["ReachThree"]`, in `NodeRef` then occurrence order; two structurally equal state clauses share one node and carry two rows with distinct occurrences; a package with none of these nodes has no member and admits; the rows are exposed by reference, resolved by `claim_name_for_path`, and preserved by write and re-read. | Test (TC-477; FSpec FR-322-AC-457) |
| FR-371-AC-2 | A row with an extra or missing member, a `name` or `qualified_name` segment that is no identifier, an empty `qualified_name`, an `occurrence.role` other than `declaration` or `claim`, or a negative `occurrence.ordinal` refuses `malformed_wire` at the member FR-371 names. | Test (TC-477; FSpec FR-322-AC-458) |
| FR-371-AC-3 | A row whose `declaration` names no node or a `temporal_clause` node refuses `invalid-value` at `/claim_names/{i}/declaration`; an `occurrence` the node lacks refuses at `/claim_names/{i}/occurrence`; a `qualified_name` whose last segment is not `name` refuses at `/claim_names/{i}/qualified_name`; a refinement row whose `name` differs from the body `name` refuses at `/claim_names/{i}/name`. | Test (TC-477; FSpec FR-322-AC-458) |
| FR-371-AC-4 | Two rows with one `qualified_name` refuse `ambiguous_declaration`/`ambiguous-name` at the later row's `qualified_name`; a repeated key refuses `conflicting-definition` at the later row; a row out of order refuses `invalid-value` at the later row; the repeated key wins over the equal name within one row pair. | Test (TC-477; FSpec FR-322-AC-458) |
| FR-371-AC-5 | A claim-bearing node occurrence with no row, and a package holding such a node with no member, each refuse `invalid-value` at `/claim_names`; a row defect wins over missing coverage. | Test (TC-477; FSpec FR-322-AC-458) |
| FR-371-AC-6 | Each visited row consumes one validation visit: a work limit one below the exact allowance of a package with rows refuses incomplete, and the exact allowance admits. | Test (TC-477; FSpec FR-322-AC-458) |

## Dependencies

- FR-038 owns strict admission; FSpec FR-322 owns the wire shape and rules.
