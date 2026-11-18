---
id: FR-374
title: "Join source timed-model clock places in two stages"
type: FR
relationships:
  - target: ix://agent-ix/filament-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/filament-contract-ir/FR-367
    type: depends_on
  - target: ix://agent-ix/filament-contract-ir/FR-352
    type: depends_on
  - target: ix://agent-ix/filament-specification/FR-322
    type: references
  - target: ix://agent-ix/filament-specification/FR-610
    type: references
  - target: ix://agent-ix/filament-specification/FR-789
    type: references
  - target: ix://agent-ix/filament-spec-language/FR-569
    type: references
---

# FR-374: Join source timed-model clock places in two stages

## Description

A source timed model is a `timed_model` root whose `model` is a
source-declared object type node (FSpec FR-322 "Source-declared object
types" item 8). `CheckedPackageV2::read` SHALL join its `clock_place` rows,
and its `parameter` constraints and resets, to everything the package alone
shows, and `SelectedSourceObjects::from_stage_rows` SHALL join them to the
member entries of the identity rows, as FSpec FR-789 divides the two stages.
Neither the read's wire nor its API changes: the read never receives the
identity rows, and the source join of the identity rows runs only after the
read has admitted the package.

## Inputs

A V2 package holding a source timed model; and, for the identity-row join,
the FR-367 source-object rows together with the source record, tuple and
union rows of the same identity file, each member, position and payload
position in the member-entry form of FR-367 "Member entries".

## Outputs

From the read, the admitted package or one located refusal. From the
identity-row join, the joined `SelectedSourceObjects` or one
`SelectedSourceObjectError`.

## Behavior

### At the read

For a source timed model with root `SourceOwner` `identity` `I`,
`check_resolved_path` SHALL NOT recover a `ModelOwner` for the root and
SHALL check, refusing `invalid_package`/`malformed-clock-place` at the row
`/semantic_graph/nodes/{i}/body/members/2/value/members/{j}` (the path-step
refusals of FR-038 keep their locus):

1. the row shape, `path_step`, the first-step and terminal `Clock` rules and
   every `slot` rule, unchanged from a selected place;
2. row `0`'s `declaring_type` `ix://I/<root type>`, and the chain rule;
3. a `field`, `tuple` or `union` row's `declaring_type` `ix://I/<T>` naming a
   source-declared object type, native record, tuple or union node `T` of a
   `SourceOwner` with `identity` `I`, or a product rendering after a `union`
   row, and its member identity spelling (FSpec FR-789 "Rendering");
4. a `field(name)` row whose declaring node's body binds `name`: its
   `member_type` against the rendering of the bound type node (`ix://I/<T>`
   for a source declaration, `ix://filament/native/<Name>` for a native, the
   collection wrappers over a `composite_type` node), `Option` exactly for an
   `optional` binding, and a following `slot` row's kind and bounds against
   the type node's collection and `collection_bounds` nodes;
5. a `field(name)` row with no binding, and a `tuple` or `union` row, only
   for a well-formed rendering; their members are left to the identity-row
   join.

For a `parameter` constraint or reset of a source timed model the read SHALL
NOT resolve the operation through `ModelOwners`; it SHALL keep the anchor
and `text` checks and, for a `parameter` reset, refuse
`invalid_package`/`timed-node-reference` at `/body/members/1/value/target`
when the anchor's frame names no `Field` entry of the root model whose name
has no binding in the root node's body (`frame_writes_clock` reads the
bindings for a source root). A selected root's checks are unchanged.

### At the identity-row join

After the FR-367 classification (foreign, unknown, conflicting, missing,
mismatched) admits every row, the join SHALL visit the package's source
timed models, their `clock_place` nodes in ascending node-id order and each
place's rows in path order, and SHALL check each row the read left to it
against the member entry it names, by FSpec FR-789 "What the identity-row
join checks": a clock field's entry, presence, `member_type` and collection
layers; a tuple position's rendering; a union member's payload rendering.
It SHALL then check each `parameter` constraint and reset against the
`clock_parameters` of the anchor's operation entry in the root type's
`operations` (or the nearest supertype's in `supertypes` order), and each
`parameter` reset's frame for a clock entry of the root row whose
`member_type` is `ix://filament/native/Clock`.

A failure SHALL be `SelectedSourceObjectErrorCode::MalformedClockPlace` for
a row or `SelectedSourceObjectErrorCode::TimedNodeReference` for a clause,
reported last in the reason order, after `MismatchedIdentityWitness`, with
the `CheckedPackageRefusalLocation` the strict read would give the same
failure (the row, the constraint's `/body/members/2`, the reset's
`/body/members/1/value/target`); `SelectedSourceObjectError` gains that
`locus`, `None` for every earlier reason, whose locus stays `Package`. The
first failing row in visit order is reported, with no partial view and no
meter charge.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-374-AC-1 | Over FSpec FR-789-AC-1's package (`record W { c: Clock; }`, `object O { w: W; d: Clock; }`, `time O dense unit ms;`), `CheckedPackageV2::read` admits the package with no identity row, and `from_stage_rows` with `O`'s row (data entry `w`, clock entry `d`) and `W`'s record row (clock entry `c`) joins it. | Test (TC-481) |
| FR-374-AC-2 | Each FSpec FR-789-AC-2 edit refuses at the read `invalid_package`/`malformed-clock-place` at the edited row, with no identity row supplied. | Test (TC-481) |
| FR-374-AC-3 | Each FSpec FR-789-AC-3 and FR-789-AC-4 edit is admitted by the read and refused by `from_stage_rows` with `MalformedClockPlace` at the stated row; a supply that also holds a missing type reports `MissingIdentityWitness`. | Test (TC-481) |
| FR-374-AC-4 | Over FSpec FR-789-AC-5's package, the read refuses the `touch` reset `invalid_package`/`timed-node-reference` at `/body/members/1/value/target`, and `from_stage_rows` refuses the constraint naming `k`, and the `x` constraint with `arm`'s `clock_parameters` empty, with `TimedNodeReference` at `/body/members/2`; a selected timed model's parameter and frame checks read unchanged. | Test (TC-481) |

## Dependencies

- [FR-367](./FR-367-join-source-object-rows-and-field-reads.md) owns the
  source-object rows, their member entries and the reason order.
- FSpec FR-789 owns the stage split and the rendering; FSL FR-569 and FR-531
  own the identity file that carries the rows.
