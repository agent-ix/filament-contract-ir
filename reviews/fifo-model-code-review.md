---
id: SR-17940486709564847048123
title: "code-review of QSL-2726 FIFO key walk into model records (code/fifo-key-model-record)"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; branch code/fifo-key-model-record at 0ce5cc0d (git diff origin/main...HEAD, main 3a4bc36d; PR not yet opened); crates/filament-contract-model/src/checked_package/v2/protocol_body.rs, tests/it/checked_package_v2_protocol_body.rs, spec/checked_package/functional/FR-372-admit-bounded-channels-population-captures-reference-carriers-and-post-test-repeats.md"
review_set: subset
---

## Summary

Ticket: QSL-2726, from the commit message. The branch name has no ticket id, and gh and linear are unauthenticated here, so nothing was posted. This is a code review with the rust-review lane folded in.

The change: `equality_stable` now takes `owners: &ModelOwners`. For a `model` key node of form `record_value_type`, `tuple_value_type` or `variant_type`, it calls the new `model_value_holds_ieee_leaf`. That function looks up the declaration's owner, then walks the selected model document's member types: record fields through `DomainModel::slot_type`, tuple positions and variant payloads through `element_type`. It goes through `Option`, `Collection` and nested `Record`/`Tuple`/`Variant`, keeps a visited set of `(package identity, node)`, and returns true at the first `Float32`/`Float64`.

Authorities checked:

- FSpec FR-726 (FIFO-key-stable).
- FSpec FR-322 "Equality-stable types".
- FSpec FR-465 reader steps 2 and 3.
- IR FR-372.

Tests: one cargo slot through `flock /tmp/cargo-slot.lock`, `-j 4`, `--features conformance-cli`, FILAMENT_SPECIFICATION_DIR = FSPEC-main-ci.

- `cargo test --test it -- protocol_channel_ fifo` in the worktree at 0ce5cc0d: 17 passed, 0 failed. This includes both new tests.
- Reviewer probes ran in a scratch copy of the tree, with no repo edits. They reuse the PR's `fifo_model_key_result` fixture plus a `value`-form declaration helper.

| Probe key | Result | Expected (FR-726) |
| --- | --- | --- |
| `Derived` (supertypes `[Reading]`, own `count: Integer`), `Reading.value: Float64` | ADMITTED | refuse |
| record field of a map type `{kind: map, keys: String, values: Float64}` | ADMITTED | refuse |
| record field of a model value type `{scalar: float64}` | ADMITTED | refuse |
| record field typed `Option<map of Float64>` | ADMITTED | refuse |
| record field typed `ix://filament/native/Clock` | ADMITTED | refuse (not equality-stable) |
| control: `Derived` with own `Float64` field | REFUSED ill_typed/operator-ineligible | refuse |
| control: `Derived` with undeclared supertype | REFUSED missing_declaration | refuse |

The controls show that the fixture reads `supertypes` and that the walk works on own fields, so the first row is a real gap in the walk.

Mutants: 8 reviewer mutants were run against the two new tests, and all 8 were KILLED:

- float32 arm dropped;
- `Option` arm dropped;
- `Collection` arm dropped;
- `Tuple` not followed;
- `Variant` not followed;
- visited set ignored (killed by the `recursive record` case, through the work limit);
- call site removed;
- variant expansion dropped.

The tests are not tautological, and the coder's "8 killed" claim holds for this set.

The probe and mutant builds shared the worktree's `target/`, because the tree paths collide. The worktree binary was rebuilt afterwards, and the two new tests re-ran: 2 passed, with no probe test present.

quire 0.36.2 was used for validation.

## Verdict

**NOT MERGEABLE.** There are two high soundness findings and two medium findings.

The walk is correct for what it reaches:

- own record fields, tuple positions and variant payloads;
- `Option`/`Sequence` layers of declared value definitions;
- multiplicity collections;
- recursion, which the visited set handles (the `Tree` case terminates);
- `Reference` members, which are correctly left unwalked: a reference compares by identity, and `option_types::read` only admits object or `Reference<Clock>` targets.

Metering is present on every list growth and on every pop. The visited key `(package identity, node)` is right, because declaration identities are unique within a document and the identity separates packages.

It still lets float leaves through, because:

- inherited fields are never read (FND-001);
- every member type that `element_type`/`slot_type` maps to `None` is silently treated as stable. That covers map-typed members and float-scalar model value types (FND-002).

Separately, the default arm treats `Clock` as a stable leaf, although FR-322 refuses equality over clock-bearing values (FND-003).

The over-refusal check is clean. Nothing the walk refuses is FIFO-key-stable: it refuses only on a float32/float64 leaf, and does not follow references or enums.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Inherited record fields are not walked. `expand_model_value` reads `package.record_types[node].fields`, but `FieldBearingTypeDecl.fields` holds only the declaration's own fields: `semantic_ir_object_type` reads `fields` and keeps `supertypes` separately (model_members.rs:6240-6276), and `check_supertype` admits record supertypes. A key record `Derived` whose supertype `Reading` declares `value: Float64` is ADMITTED (reviewer probe), while the same field declared on `Derived` itself is refused. The commit message, the doc comment and FR-372 all claim "own and inherited". Fix: expand `owner.package.ancestors(node)` (model_members.rs:1841), or the lock-stage field tables (`package.fields.of(node)`), not just `record.fields`. Add a refuse case with the float on a supertype, and one two levels up | crates/filament-contract-model/src/checked_package/v2/protocol_body.rs:1910-1920 |
| FND-002 | high | A member type that `slot_type`/`element_type` cannot map is skipped as though it were stable. `element_type` returns `None` for a declared map (`SelectedTypeDefinition::Map`, model_members.rs:2170), for `SelectedTypeRef::Map`, for an unmapped model value type (`value_types[name] == None`, which every `scalar` other than integer, rational and string gives, including `float64`/`float32`, model_members.rs:3797-3838), and for `Option`/`Sequence` over any of these. Record field admission refuses none of these: only event, process, state-machine, persistence-interface and namespace types are forbidden. Probes: record fields of `Map<String, Float64>`, `Option<Map<String, Float64>>` and a `scalar: float64` value type are each ADMITTED as a FIFO key. Fix: treat an unmappable member as not FIFO-key-stable (`return Ok(true)` / refuse) when `slot_type`/`element_type` gives `None` for a present `typeRef`, or walk map values and value-type scalars explicitly. Add a refuse case for each | crates/filament-contract-model/src/checked_package/v2/protocol_body.rs:1915-1919; crates/filament-contract-model/src/checked_package/v2/protocol_body.rs:1923-1925; crates/filament-contract-model/src/checked_package/v2/protocol_body.rs:1934-1940 |
| FND-003 | medium | The walk's default arm `_ => {}` admits a `MemberType::Clock` leaf. FSpec FR-726 defines FIFO-key-stable as an FR-322 equality-stable type, and FR-322 refuses `=` over a clock-bearing value (`DomainModel::bears_clock`'s contract). A record with a `Clock` field is ADMITTED as a FIFO key (probe). Before this PR every model record was admitted, so this is not a regression, but the walk now enumerates the leaves and passes over this one. Either refuse on `Clock` (and on `Option`/collection of `Clock`) here, or reuse `ModelOwners::bears_clock`. The default arm should list the stable `MemberType`s explicitly rather than accept every future variant | crates/filament-contract-model/src/checked_package/v2/protocol_body.rs:1893 |
| FND-004 | medium | No test drives the new root forms and limits that the code adds. (a) A `tuple_value_type` key at the root is never tested. The test note says a tuple "is not an admitted `carries` type", but FSpec FR-465 step 2 admits `tuple_value_type` as `carries`, and the reader's `carries` match omitting it is itself a defect (protocol_body.rs:3395-3413, see the spec-review SR). (b) No work-limit or storage-limit refusal inside `model_value_holds_ieee_leaf` is tested, although every `meter` call is new code. (c) There is no inherited-field, map or value-type case (FND-001, FND-002) | tests/it/checked_package_v2_protocol_body.rs:11370-11540 |
| FND-005 | low | Metering is incomplete and placed at the wrong site. `slot_type`/`element_type` are the unbudgeted variants: the collection-chain walk, its `seen` vector and the boxed `MemberType` chain they build are not charged, although `slot_type_runtime`/`element_type_runtime` exist. `meter.push` charges only the outer `pending` slot, not the boxed depth. `node_work` is charged at the key type node (`position`) while the rest of `equality_stable` charges at the channel (`self.position`), so a work-limit refusal from this walk names a different site from one in the surrounding walk. The total stays bounded by document size, so this is not a correctness failure | crates/filament-contract-model/src/checked_package/v2/protocol_body.rs:1879; crates/filament-contract-model/src/checked_package/v2/protocol_body.rs:1916; crates/filament-contract-model/src/checked_package/v2/protocol_body.rs:1923; crates/filament-contract-model/src/checked_package/v2/protocol_body.rs:1938 |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | `protocol_channel_fifo_model_walk_is_charged_to_the_work_meter` asserts only that a 12-record chain needs more work than a 1-record chain. The document read itself scales with the chain length, so the test does not pin the walk's own charge: with `budget.charge(1)?` removed from `holds_key_unstable_leaf`, the mutant SURVIVED (21 passed). The walk is still bounded through the charged `budget.push`/`visited.insert` calls, so this is a test-strength gap only. To pin it, compare two packages with the same document, one with a `fifo` key and one `unordered`, or a key at the chain head versus the tail | tests/it/checked_package_v2_protocol_body.rs:11829-11880; crates/filament-contract-model/src/checked_package/v2/model_members.rs:1863 |
| FND-007 | low | The `Item::Ref(SelectedTypeRef::Map(keys, values))` arm pushes the raw `Map<K, V>` key and value texts as `Item::Name`. A native such as `ix://filament/native/Integer` names no table entry, so it would reach the fail-closed `!object_types.contains_key` branch and refuse a stable inline `Map<String, Integer>`. Today this is unreachable: a record field, tuple position, variant payload, option inner and map value each refuse an inline `Map<..>` `typeRef` (probe: the record field refuses invalid_model_binding/malformed-declaration at the owner). Resolve `keys`/`values` through `SelectedTypeRef::from_admitted_name`, or mark the arm unreachable like the other guards | crates/filament-contract-model/src/checked_package/v2/model_members.rs:1878-1882 |

## Dispositions

Round 2, at head ae150b09 (fix commits be6ed5f9 and ae150b09).

Verification:

- The head ran in a `git archive` snapshot with its own CARGO_TARGET_DIR, under the flock, `-j 4`, `--features conformance-cli`, FSPEC-main-ci. `--test it -- fifo protocol_channel_` gave 21 passed (the PR and pre-existing tests, plus the reviewer probe).
- The reviewer probe `probe_fifo_round2` covered 20 cases, and every result matches FR-726.
  - Refused ill_typed/operator-ineligible at the key:
    - a float on a supertype, and two supertypes up;
    - map Float64 values;
    - float64 and float32 scalar value types;
    - an Option of a float map;
    - a Clock field;
    - a Set (unordered unique multiplicity) of Float64;
    - a tuple root with a Float32 position.
  - Admitted:
    - a Set of Integer;
    - a map of Integer;
    - native Decimal, Rational, Text, UUID, Boolean, Bytes, Timestamp and String;
    - an object reference field;
    - an enum (a variant with no payloads);
    - boolean, string, integer, decimal and uuid value types;
    - Duration;
    - a tuple root `(Boolean, Integer)`.
  - An inline `Map<..>` typeRef is refused at document read (invalid_model_binding), so it never reaches the walk.
- Event types cannot be record fields (`record_field_forbidden`) or variant payloads. An event or other unknown name reaching the walk fails closed.
- `Reference` definitions compare by identity and are not followed. That is correct, because `option_types::read` admits only object or `Reference<Clock>` targets.
- Generics are not part of the document model, so there is no generic path.
- The visited set is keyed by name within one document. The walk never leaves `self`, so the key is sufficient.
- Mutants against the head: 11 run, 9 KILLED and 2 SURVIVED.
  - KILLED: Clock not refused; float32 not refused; ancestors skipped; ieee value types ignored; map values skipped; map keys skipped; seq/option not followed; tuple positions skipped; visited ignored.
  - SURVIVED: "unresolved fail-open", which is the documented unreachable guard and acceptable; and "budget charge removed" (FND-006).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `holds_key_unstable_leaf` walks own and every ancestor record's fields. Both supertype probes refuse, and the "ancestors skipped" mutant is killed. after_excerpt: "let ancestors = self.ancestors(name, budget)?; for declaration in std::iter::once(name).chain(ancestors.iter().copied()) { let Some(record) = self.field_owner(declaration) else { continue; }; for field in &record.fields { match field.slot.type_ref.as_ref() { Some(type_ref) => { budget.push(&mut pending, Item::Ref(type_ref), MODEL)?; }" |
| FND-002 | fixed | The walk now reads the document's `SelectedTypeRef` tables directly, so no member type is lost to a `None` mapping. Map keys and values are walked, float32/float64 scalar value types are recorded in `ieee_value_types`, and an unresolved name fails closed. Native Decimal, Rational and Text, and non-float value types, stay stable (probes admit them). after_excerpt: "SelectedTypeDefinition::Map(map) => { budget.push(&mut pending, Item::Ref(&map.keys), MODEL)?; budget.push(&mut pending, Item::Ref(&map.values), MODEL)?; } } } else if self.value_types.contains_key(name) { if self.ieee_value_types.contains(name) { return Ok(true); }" |
| FND-003 | fixed | A Clock leaf refuses: the Clock-field probe refuses, and the "Clock not refused" mutant is killed. The default arm is gone: every `SelectedTypeRef` variant and every table is handled explicitly. after_excerpt: "Item::Ref(SelectedTypeRef::Native(native)) => { if matches!(native, NativeTypeRef::Float32 \| NativeTypeRef::Float64 \| NativeTypeRef::Clock) { return Ok(true); } continue; }" |
| FND-004 | fixed | New tests cover the gaps: `protocol_channel_fifo_refuses_model_members_it_cannot_prove_stable` (inherited, two up, alias, map keys and values, option over a map, float value types, Clock, tuple key with a float position), `protocol_channel_admits_fifo_inherited_and_tuple_model_keys` (stable inherited field, tuple key), and `protocol_channel_fifo_model_walk_is_charged_to_the_work_meter`. `TupleValueType` was added to `carries` (protocol_body.rs:3341). The meter test's strength is a separate new finding (FND-006). after_excerpt: "fn protocol_channel_admits_fifo_inherited_and_tuple_model_keys() { ... \"tuple key\", ... \"Pair\"" |
| FND-005 | fixed | The walk no longer uses `slot_type`/`element_type`. Every step, push and visited insert goes through `Budget::node(meter, self.position)`, so the charge is located at the channel like the rest of `equality_stable`. after_excerpt: "let mut budget = Budget::node(meter, self.position); owner.package.holds_key_unstable_leaf(owner.node, &mut budget)" |
