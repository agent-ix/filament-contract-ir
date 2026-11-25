---
id: SR-5761727
title: "code-review and rust-review of IR-1615 event-typed parameter admission and operand guard"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1174 at eff94c5d093c6908f8dd5a51984494940398b825 (base b81baf144); crates/filament-contract-model/src/checked_package/v2/{structural.rs,operations.rs}, tests/it/{checked_package_v2_model_fields.rs,checked_package_v2_temporal.rs}"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in, for filament-contract-ir PR #1174. Ticket: IR-1615 (blocks QSL-3198). Lead scope: admit an `event_type`-typed `value`/`parameter` node in `parameter_defect` and refuse, `ill_typed`/`operator-ineligible`, every application operand that references such a parameter, except the temporal clause's `over` operand. Event field tables, record projection and conformance stay on IR-1615. References read: FSpec (filament-specification origin/main fd90d643) FR-322 "Operation families, groups and constraints" event-equality paragraph, FR-322-AC-68, TC-281 EV-01 to EV-06, FR-370 "Properties", FR-208-AC-9; operation catalog filament-verification-contracts e6eebb6 (the locked rev). Build and tests NOT run: cargo is blocked on this host for reviewers; everything below is by reading.

## Examined scope

- `parameter_defect` (structural.rs:1606-1650, examined): the only change adds `ModelForm::EventType` to the model-owned `VariantType` arm, so an event-typed binder is admitted only when its type node carries a `ModelOwner`; the owner join (owner.rs:553) then requires that owner to name a selected event type. `parameter_defect` is the sole check for every `value`/`parameter` node (structural.rs:759). On main every parameter whose present type node is an `event_type` node was refused here, so the operand guard can fire only on packages main refused: nothing admitted on main stops being admitted.
- `check_event_parameter_operands` (operations.rs:3393-3439, examined): runs in `operation_defect` before the place checks and `check_operands`, after the variant/clock/carried-sum checks; none of those earlier checks admits early. It scans top-level `arguments`, and refuses at the first `Reference` whose target is a `value`/`parameter` node whose `semantic_type` node has kind `model`/`event_type`. Kind is read from the type node directly; owner is not re-checked (parameter_defect already required a `ModelOwner`).
- Temporal exception width (examined): `reads_over` holds only when `entry.operator == Temporal` and catalog operand 0 has class `reference`. In the locked catalog only `filament.op.temporal.clause` has operator `temporal`, and its operand 0 is `reference`. Only argument 0 is skipped; arguments 1-5 are still scanned. This is exactly FR-370's `over` slot and no wider.
- Why the guard is needed (examined): `operand_family` maps `Model(EventType)` to no family (operations.rs:2933) and `resolve_family_with_selected` returns `NoFamily`; `check_operands` skips an operand whose family is `None`, and `SameType` passes when both operands have the same type node. Without the guard `structural.eq`, `boolean.not` and `collection.contains` over event-typed parameters would be admitted. The new comment on the function states this correctly.
- Routes for an event-typed parameter to reach an operator (examined): direct reference in any operand or rest position of any family (refused); `control.if` branch (direct reference, refused); `control.let` body `x` (direct reference to the let binder, which is itself an event-typed parameter, refused); `function.call` rest argument (refused); `collection.set` element (refused); quantifier binder used in a body expression node (the binder is a `value`/`parameter` node, refused at that node). Not covered: a reference that sits inside a `binding` or `aggregate` argument (FND-001), and any operand that is an expression node of event type rather than a parameter (FND-002, pre-existing).
- Protocol and temporal nested slots (examined): the protocol `over` binding, temporal `activation.trigger`, captures and the compensation `attempts`/`retry` binders sit inside `binding`/`aggregate` arguments, so the guard does not touch them; this is what keeps FR-828-AC-8 style event-typed triggers admissible.
- Tests (examined): both model_fields tests and the temporal test assert literal codes, causes and JSON-pointer loci; each fails on main (the base package is refused at the parameter's `semantic_type`), and the AC-633 cases would be admitted, or refused at another locus, without the guard.
- rust-review lane (examined): no `unsafe`, no panic path (`?` on every lookup, `skip(usize::from(bool))`), no new allocation, no integer conversion at a wire boundary, no public item changed; the guard is unmetered but bounded by the argument count, like the neighbouring `check_carried_member`. Doc comment explains the code without history.

## Verdict

Request changes on one medium soundness gap, with one high pre-existing finding to ticket separately. The admission change is minimal and correctly owner-gated, the `over` exception is exactly one argument of exactly one operation, and nothing main admits is newly refused. The guard, however, scans only top-level `reference` arguments, so a bare reference to an event-typed parameter nested in a `binding` value (a quantifier, fold or case body) is not refused, and no reader check types those bodies (FND-001). The coder's "other routes" note is true by reading: on main, operations over event-typed values that are not parameters are admitted, contrary to FSpec TC-281 EV-04/EV-05 (FND-002, pre-existing, out of this PR's scope).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `check_event_parameter_operands` inspects only top-level `CheckedTerm::Reference` arguments. A reference to an event-typed parameter nested in a `binding` value escapes: e.g. `filament.op.collection.forall(all: Set<OrderPlaced>, e => e)`, whose body is the bare reference to the event-typed binder `e`, or a `case` arm / `fold` step whose body is such a reference. `check_operands` enforces no `bound_family`/body type (operations.rs:3726-3739 lists `BoundFamily` as not enforced) and no other check types a quantifier body, so by reading this is admitted at this head; on main `e` itself was refused by `parameter_defect`. FR-038 states "No operation over a parameter typed by an event type is admitted". A fix must keep the temporal `activation.trigger`, captures and protocol binder slots admissible (they also nest such references), e.g. refuse when a quantifier/fold/case body binding value is a reference to an event-typed parameter | crates/filament-contract-model/src/checked_package/v2/operations.rs:3413-3431 |
| FND-002 | high | Pre-existing on main b81baf144, not introduced here: operations over event-typed values that are not parameters are admitted. `composite_type` option/set/sequence element is any well-formed reference (structural.rs:967-977), so `Option<OrderPlaced>`/`Set<OrderPlaced>` parameters are admitted; `structural.eq` over two of them fits `structural_kind` and `same_type`, and no reader check refuses an event type at any depth (only `bears_clock` and the variant family are checked, operations.rs:2443-2486). `option.value` of an `Option<OrderPlaced>` gives an expression node of event type whose use as an operand resolves `NoFamily` and is skipped by `check_operands`, so `boolean.not` over it is admitted. A `composite_type`/`record` may hold an `event_type` field (tests/it/checked_package_v2_protocol_body.rs:12819-12833 builds one). FSpec FR-322 event equality and TC-281 EV-04/EV-05 refuse these `ill_typed`/`operator-ineligible`. Needs its own ticket (IR-1615 item 2) | crates/filament-contract-model/src/checked_package/v2/operations.rs:2443-2486, crates/filament-contract-model/src/checked_package/v2/structural.rs:967-977 |

## New findings (disposition pass 1)

Re-reviewed at 8397db68968fdd4b3869d36689025473ff01507a (fix commit 8397db689 on eff94c5d0). Build and tests still unverified.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | A temporal clause's `activation` is an `aggregate` operand (catalog class not `binder`), so the new walk does not enter it, and the bare `reference` check does not match an aggregate. An `each` activation whose `guard` binding value is a bare reference to its event-typed `trigger` parameter (the event read as the Boolean guard) is therefore not refused, and no check in operations.rs or temporal.rs types the guard (the only use, operations.rs:1786-1806, collects it for binder-level scoping). On main the event-typed trigger itself was refused. The trigger and captures must stay admitted; the guard value is a read and can be scanned like a binder body | crates/filament-contract-model/src/checked_package/v2/operations.rs:3416-3427 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 8397db689: a `binder`-class operand (operand or rest) is walked through `binding` and `aggregate` terms by `binder_reads_event_parameter`; a `case` arm skips its slot aggregate. Covers let bound value, forall/exists, map/filter/count/only/sum/seq bodies, fold/reduce steps and case arm bodies; trigger, captures and protocol binder slots are not `binder` class and stay admitted. Tested by `tc_048_a_binder_operand_holding_an_event_typed_parameter_is_ineligible`. |
| FND-002 | deferred | Moved to its own IR ticket by planner order (pre-existing on main, out of this PR's scope); that ticket was not yet filed or linked on IR-1615 when this pass ran. |
