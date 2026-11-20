---
id: SR-17940486709564847048124
title: "gap-analysis of QSL-2726 FIFO key walk into model records"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; branch code/fifo-key-model-record at 0ce5cc0d (git diff origin/main...HEAD); FSpec FR-726-AC-2, FR-726-AC-3, FR-322 equality-stable types, FR-465 reader steps 2-3; IR FR-372-AC-2, FR-372-AC-3; crates/filament-contract-model/src/checked_package/v2/protocol_body.rs, tests/it/checked_package_v2_protocol_body.rs"
review_set: subset
---

## Summary

This is a planless gap analysis (Plan completion: not assessed). Ticket: QSL-2726. It was done by hand, scoped to the change: the Quoin matrix run was not used for this one-function diff.

It checks each path by which FSpec FR-726 lets a float leaf sit under a model-owned record, tuple or variant against what `model_value_holds_ieee_leaf` reaches and what the two new tests assert.

The tests are tagged `#[trace("TC-1770", "FR-726-AC-2", "FR-372-AC-2")]` (admit) and `#[trace("TC-1770", "FR-726-AC-3", "FR-372-AC-3")]` (refuse). Both bindings are correct: the admit test asserts admitted keys whose leaves are stable, and the refuse test asserts `ill_typed`/`operator-ineligible` at `/body/arguments/3/value/members/2/value`.

The tests are not tautological. With the call to `model_value_holds_ieee_leaf` removed, the refuse test fails, because every model record was admitted before this change.

| Path to a float leaf | Code reaches | Test |
| --- | --- | --- |
| own record field, `float64`/`float32` | yes | yes |
| nested record, sequence (multiplicity) of record, sequence of float | yes | yes |
| `Option` value definition | yes | yes |
| tuple position (via record field / variant payload) | yes | yes |
| variant payload (native, record, tuple) | yes | yes |
| tuple as the key root | yes | no |
| inherited record field (supertype) | **no** | no |
| map member value | **no** | no |
| model value type with a float scalar | **no** | no |
| clock leaf (not equality-stable) | **no** | no |
| work or storage limit inside the walk | yes | no |

## Verdict

**GAPS.** FR-726-AC-3 ("a record with a `float64` field" refuses) is not met for a `float64` field the record inherits, or one reached through a map or a float-scalar value type. The reviewer probes in the code-review SR show all three admitted. The tested paths are covered and their bindings are correct.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-726-AC-3 is unmet for an inherited float field: a key record whose supertype declares `value: Float64` is admitted. The tests have no supertype case, so the AC's coverage claim is wider than the implementation | crates/filament-contract-model/src/checked_package/v2/protocol_body.rs:1910-1920; tests/it/checked_package_v2_protocol_body.rs:11438 |
| FND-002 | high | FR-726 ("no `float32` or `float64` leaf at any depth") is unmet for a float reached through a map member value or a float-scalar model value type: both are admitted, and neither has a test | crates/filament-contract-model/src/checked_package/v2/protocol_body.rs:1915-1940 |
| FND-003 | medium | The tuple-rooted key path and the walk's limit refusals have no test. The tuple root is unreachable only because the reader's `carries` check omits `tuple_value_type`, which contradicts FSpec FR-465 step 2 | tests/it/checked_package_v2_protocol_body.rs:11370-11540; crates/filament-contract-model/src/checked_package/v2/protocol_body.rs:3408-3412 |

## Dispositions

Round 2, at head ae150b09. The head's own tests ran in a snapshot with its own target dir: `--test it -- fifo protocol_channel_` gave 21 passed (20 PR and pre-existing tests plus the reviewer probe).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-726-AC-3 now holds for inherited fields. `protocol_channel_fifo_refuses_model_members_it_cannot_prove_stable` has "inherited float field" and "float two supertypes up", and `protocol_channel_admits_fifo_inherited_and_tuple_model_keys` has a stable inherited field. The "ancestors skipped" mutant is killed. after_excerpt: "for declaration in std::iter::once(name).chain(ancestors.iter().copied()) {" |
| FND-002 | fixed | Map keys and values, float-scalar value types (float32 and float64), and an Option over each now refuse, and each has a test. The mutants "map values skipped", "map keys skipped" and "ieee value types ignored" are killed. after_excerpt: "SelectedTypeDefinition::Map(map) => { budget.push(&mut pending, Item::Ref(&map.keys), MODEL)?; budget.push(&mut pending, Item::Ref(&map.values), MODEL)?; } ... } else if self.value_types.contains_key(name) { if self.ieee_value_types.contains(name) { return Ok(true); }" |
| FND-003 | fixed | The tuple-rooted key is tested ("tuple key", "tuple key with a float position"), and the walk's work charge is tested by `protocol_channel_fifo_model_walk_is_charged_to_the_work_meter`. `TupleValueType` was added to `carries`. after_excerpt: "CheckedNodeKind::Model(ModelForm::RecordValueType \| ModelForm::TupleValueType \| ModelForm::EventType \| ModelForm::VariantType)" |
