---
id: SR-5761730
title: "code-review and rust-review of IR-1619 event-reach operand refusal"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1177 at 9a275b2109622e73721e5f673688709916741bfd, delta over 9961b272b2be2750ea365e9d57b20b53ad3c94d5 (PR #1174); crates/filament-contract-model/src/checked_package/v2/{operations.rs,model_members.rs}"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in, for filament-contract-ir PR #1177. Ticket: IR-1619. The PR is stacked on #1174 (IR-1615); only `git diff 9961b272b2..HEAD` was reviewed. Reference: filament-specification origin/main (read with `git show`): FR-322 event equality paragraph and family table, FR-322-AC-65, FR-322-AC-68, FR-208-AC-9, TC-281 EV-01..EV-06.

No cargo build, check, clippy or test was run on this host (planner order). Every test result is unverified; the code was read for compile correctness.

## Examined scope

- operations.rs `EventReach` (`new`, `term_reaches_event`, `walk`, `expand`), examined. The walk covers every `CompositeTypeForm` except `reference`, `alias` and every `bounded_domain` form through `semantic_type`, and model record, tuple and variant nodes through the selected declaration. It terminates: each graph position is expanded at most once per walk through the `eventless` marks. Each expansion and each body member is charged with `work_at_path`, and the mark vector is charged through `meter.filled`. A walk that finds an event clears the marks of every position it expanded, so a stale "eventless" mark never survives a positive walk. A walk that fails on the budget aborts the read, or leaves a limit refusal as the outcome. No panic paths were found: all indexing uses `get`.
- operations.rs `check_event_operands`, `guard_reads_event`, `bound_value_reads_event`, examined. Reference, literal and application arguments are walked through `operand_type_node_typed`, which maps an application to its `result_type` and a reference to an expression node to its semantic type, so a chained `option.value` → `boolean.not` refuses at the consumer. The temporal `over` operand, the captures and triggers, and the protocol binder slots stay exempt, as in #1174.
- model_members.rs `DomainModel::holds_event` and `ModelOwners::{selects_events, holds_event}`, examined. It follows own and inherited fields, tuple positions, variant payloads, sequence, option and map definitions and inline maps, and stops at reference and object types. It uses a per-call visited set, and every step is charged to the `Budget`.
- Remaining soundness routes, examined. No route was found by which an operation consumes a value whose type reaches an event type through a reference, literal or application argument, a binder body or an activation guard. One residual is pre-existing and unverified: a non-binder `aggregate` or `binding` argument outside the temporal activation is not walked. A catalog operand of that shape, other than the exempt temporal captures and protocol binder slots, was not identified.
- The two edited #1174 tests in tests/it/checked_package_v2_model_fields.rs, examined. The `collection.contains` locus moved from argument 1 to argument 0. This is correct: the `Set<OrderPlaced>` operand now reaches the event first. The forall and fold cases now range over `Set<Boolean>` instead of `Set<OrderPlaced>`. This keeps those cases on the binder-body route, which the earlier argument-0 refusal would otherwise mask, and the assertion still pins argument 1. Neither test is weakened.

## Verdict

Not mergeable as pushed: the crate does not compile (FND-001). With FND-001 fixed, the design is sound for the routes in the ticket. The walk is bounded, charged and terminates on recursive types. Expected counts after the fix (by counting `#[test]` items, unverified): 532 for the six `--test it` filters (model_fields 104, recursive_leaves 27, temporal 121, protocol_body 237, owners 34, composite_operands 9) and 105 for `checked_package::v2::operations::tests::`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `const TEMPORAL_ACTIVATION_OPERAND` is defined twice at module scope (kept at 3399, re-added at 3570): E0428, so filament-contract-model does not compile and no test can run | crates/filament-contract-model/src/checked_package/v2/operations.rs:3397-3399, 3568-3570 |
| FND-002 | low | `DomainModel::holds_event` fails open where the sibling `holds_key_unstable_leaf` fails closed: a field without `typeRef`, and a declared name that is in no table, both return no event | crates/filament-contract-model/src/checked_package/v2/model_members.rs:2057-2130 |
| FND-003 | low | A positive walk caches nothing and clears every mark it set. Every later application over the same event-holding type walks its whole closure again, including a fresh domain walk, so a package with many such applications can end on the work limit instead of `ill_typed` | crates/filament-contract-model/src/checked_package/v2/operations.rs:3433-3460 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | `holds_event` treats an inline `Map<K, V>` field `typeRef` as unreachable and fails closed. `References::check_type_ref` appears to admit `Map<Text, V>` at the document stage (`CheckedTypeRefInput::MapValue`). If such a field survives the read, `eq` over its record is refused as reaching an event in any package that declares one. Confidence is low: reachability was not checked through the whole read. | crates/filament-contract-model/src/checked_package/v2/model_members.rs:2075-2079, 5921, 5982 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The second const was removed in 992e3f07; one definition remains, at operations.rs:3399. |
| FND-002 | fixed | `holds_event` now fails closed: a field with no typeRef, an inline map, or a name outside the value, record, tuple, variant, collection and object tables returns true. |
| FND-003 | fixed | The tri-state `EventMark` caches both answers. `keep_reaching` marks only the root-to-event holder chain as Reaches and resets the rest; `Eventless` is kept only after a walk completes with no event. |
