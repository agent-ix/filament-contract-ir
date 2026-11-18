---
id: FR-367
title: "Join source-object identity rows and resolve fields on source object nodes"
type: FR
relationships:
  - target: ix://agent-ix/filament-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/filament-contract-ir/FR-362
    type: depends_on
  - target: ix://agent-ix/filament-contract-ir/FR-352
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/ADR-013
    type: references
  - target: ix://agent-ix/filament-spec-language/FR-569
    type: references
  - target: ix://agent-ix/filament-specification/FR-322
    type: references
---

# FR-367: Join source-object identity rows and resolve fields on source object nodes

## Description

A source-declared object type node has the body `aggregate` of one
record-style field binding per field of its field view. Today a `field`
member whose declaring node has the body `aggregate{[]}` is refused as
`ill_typed`/`operator-ineligible`, and no reader joins a source-object
identity row. The reader SHALL resolve a `field` member on a source-declared
node by the binding of its name, and SHALL join the source-object identity
rows to their nodes (FSL ADR-013 QC-30 items 2a and 4; the row is FSL
FR-569's "Source-object identity rows").

## Inputs

A `field` member whose declaring node is a source-declared type; and a
producer input row `SelectedSourceObjectRow` with `structural_node`,
`source_authority`, `source_identity`, `abstract`, `supertypes`,
`identity_fields`, `members` and `populations`, supplied beside the package as
FR-352 does for its stage rows. The IR public inventory (FR-019) gains the row
type and its typed read.

## Outputs

The field's type from the binding, and the joined rows through a typed read, or
one typed located refusal.

## Behavior

The reader SHALL resolve a `field` member on a source-declared node to the
binding of its name in the node body, including an inherited field the
field view lists, and SHALL refuse a name with no binding, and any field
read on an `aggregate{[]}` source type, as it refuses an absent field on a
model-owned node.

The reader SHALL classify every supplied row before it reports. The reader reports
the first reason in the reason order of FSL FR-390-AC-67, at the `Package`
locus with no partial view and no meter charge. Foreign comes first: a row
whose `source_authority` is the authority of no `SourceOwner` in the package.
Unknown comes next: a row whose `structural_node` is not a source-declared
object type, or whose `source_identity` disagrees with that node's
`declaration`, or whose `members` or `supertypes` fail the checks below.
Conflicting follows: two rows for one (`source_authority`, `source_identity`)
key, even with equal contents, as FR-352 returns `ConflictingIdentityWitness`.
Missing follows, after all rows are classified: a source-declared type with
no row. Mismatched comes last, because it needs the complete row set: the
family check of [FR-366](./FR-366-admit-source-operation-dispatch-tables.md)
over the package's dispatch tables, which refuses with
`MismatchedIdentityWitness`. Each outcome is the FR-352 outcome of the same
name, `MismatchedIdentityWitness` being the FR-352 outcome for a stale or
mismatched stage row.

The key (`source_authority`, `source_identity`) identifies one type. The
`declaration` of a source-declared type names it within the package whatever
its `SourceOwner`, so two `SourceOwner`s of one authority cannot each declare a
type of one name: the package already refuses the second node as a repeated
declaration, and no two rows can then share a key across owners.

The row `members` check compares, in order, each data entry's name, field
type node and optional presence with the node body's bindings: the binding
name, the bound type node reference and its `optional` group.

### Member entries

A row's `members` list every field of the effective view (FSpec FR-610
"Identity rows"), so a clock field, which has no binding, is a member too.
`SelectedSourceObjectMember` SHALL be `{name, optional, type_node:
Option<CheckedNodeId>, member_type: Option<Box<str>>, collections:
Vec<SelectedClockCollectionLayer { kind, minimum, maximum }>}`: a **data
entry** has `type_node` and no `member_type` and empty `collections`; a
**clock entry** has no `type_node`, `member_type` its FSpec FR-789
rendering with presence, and one layer per collection of its type,
outermost first. The data entries, in order, SHALL equal the bindings; a
clock entry SHALL share no name with a binding and its `member_type` SHALL
be a rendering whose terminal is `ix://filament/native/Clock` with no node
(`Clock`, `Option<Clock>`, a collection or map of them). An entry with both
or neither of `type_node` and `member_type`, or failing these, is Unknown.
`SelectedSourceRecordMemberRow` and `SelectedTuplePositionRow` SHALL carry
the same optional `type_node` (with `value_type` absent exactly for a clock
entry, which no Exact projection reads) and `member_type` and `collections`,
and `SelectedUnionMemberRow` SHALL gain `clock_payload`, the clock entries of
its payload positions by ordinal. Each
`SelectedSourceObjectOperation` SHALL also carry `clock_parameters:
Vec<Box<str>>`, the names of the operation's `Clock` parameters in
declaration order. These rows feed the clock join of
[FR-374](./FR-374-join-source-timed-model-clock-places-at-the-identity-row-join.md)
and the coverage check of
[FR-375](./FR-375-check-source-operation-body-coverage-at-the-identity-row-join.md). The `supertypes`
check requires each entry to name an admitted source-declared type node of the
package that is not the row's own node, and the supertype relation of all rows
to be acyclic. `identity_fields` and `populations` are producer facts read only
from a joined row; the package has no `extends`, identity or population
information against which to check them.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-367-AC-1 | `self.total` on a source type with a field `total` resolves to the type of the binding, and an inherited field resolves through the field view. | Test |
| FR-367-AC-2 | A field name with no binding, and any field read on an `aggregate{[]}` source type, refuse `ill_typed`/`operator-ineligible` at the member. | Test |
| FR-367-AC-3 | A supply holding a foreign row and an unknown row reports foreign whatever their order; one holding an unknown row and a duplicate-key pair reports unknown; one holding a duplicate-key pair and an omitted type reports conflicting; one omitting a type reports missing; one whose dispatch candidate lies outside its context's family reports mismatched only when no type is missing. A second type of the same name under another `SourceOwner` of the same authority refuses at admission. | Test |
| FR-367-AC-4 | A row whose `members` differ from the body bindings in name, type node, optional presence or order refuses unknown. A row whose `supertypes` name a non-source node, the row's own node, or form a cycle refuses unknown. | Test |
| FR-367-AC-5 | A complete row set exposes abstractness, supertypes, identity fields, members and populations for each source type through a typed read. | Test |
| FR-367-AC-6 | A field read on a model-owned node reads unchanged in a package that also holds a source type. | Test |
| FR-367-AC-7 | A row of `O` (`object O { w: W; d: Clock; }`) with the data entry `w` then the clock entry `d` (`member_type` `ix://filament/native/Clock`) joins; a clock entry `d` carrying a type node, an entry with neither `type_node` nor `member_type`, a clock entry named `w`, and a clock entry whose `member_type` is `Integer` each refuse unknown. | Test (TC-481) |

## Dependencies

- [FR-352](./FR-352-project-admitted-exact-value-types.md) owns the stage-row convention and its closed outcomes.
- FSL FR-569 owns the row content, FR-531 the file that carries it, and FR-390-AC-67 the reason order.
