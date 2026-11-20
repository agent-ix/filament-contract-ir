---
id: SR-2026112013214800000001
title: "code-review (rust-review lane) of QSL-2709 PR #1095 integer containment at result positions"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR #1095 at cf214d40, git diff origin/main...HEAD; crates/filament-contract-model/src/checked_package/v2/operations.rs (integer_contains, admits_at_result, check_control_result, union_arms_hold, body_root_defect); tests/it/checked_package_v2_operation_arguments.rs (tc_2099_*); checked against filament-specification b90d84ec FR-322 'Integer containment', FR-322-AC-495..502, TC-2099"
review_set: subset
---

## Summary

This is the code-review of PR #1095, with the rust-review lane folded into this file. Ticket: QSL-2709, under the amended ruling (integer containment at result positions only). It was checked against FSpec FR-322 "Integer containment", "Results", "The `if` check" and "The `let` binder check" at filament-specification b90d84ec. All runs used a private review worktree (QSL-2709-review) and a private target dir, with cargo-slot, one job at a time:

- tests/it (`--features conformance-cli`), name-filtered: `checked_package_v2_operation_arguments`, `lowering`, `catalogue_differential`, `union`, `reader`, `typed_terms`, `whole_read_accounting`, `budgeted_read`, `ieee_admission`, `frame_bodies`, `operand_families`, `term_data_writer`, `admitted_result_context` and `admitted_guard_context`. 183 pass and 0 fail. The if/let checks cause no regression in these modules, and the per-`let` node scan does not move any exact work-accounting pin.
- conformance_fspec, full: 477 pass and 16 fail. The failing set is identical, name by name, to the main baseline (certificate_member, monitor_plan, operation_effects, probabilistic_claims, requirement_records, temporal_aggregate, timed_model), and no FSpec positive fixture newly refuses under the if/let checks.
- Local scratch `it` tests, each reverted and never pushed: (P1) IC-15 plus an unrelated parameter `y: Integer` is admitted, which reproduces FND-001. (P2) An `if` whose then-branch is an `aggregate` term is refused earlier, at `body/arguments/1` (operand check). (P3) A `let` whose binding name `z` names no parameter node, while its body reads an `Int[0, 999]` parameter over an `Int[0, 1000]` value, is admitted (FND-002).

## Examined scope

- `integer_contains`. It matches FR-322 exactly. Rule 1: `E` is the builtin `ScalarType(Integer)` node and `T` is a `BoundedDomain(IntegerRange)` node. Rule 2: both are `IntegerRange` nodes and `lo_E <= lo_T && hi_T <= hi_E`. The bounds come from the existing `integer_range_bounds` (exactly `min`, `max`) and are parsed as `Integer`. A range whose bounds do not parse is not contained. A carried quantity is a `scalar_type`/`carried_quantity` node, so a unit is never widened. Rational, decimal, IEEE, constructors and `Integer` at `Int[..]` all fall to `false`. Clean.
- Positions. `admits_at_result` is used at exactly these places: `body_root_defect` (function and expression roots), `union_arms_hold` arm bodies (the binder check `binders_hold` stays exact), both `if` branches (operands 1..3), the `let` body (operand 1), and the `let` binder. `conforms_to_type` stays alone at call arguments, union payloads, tuple positions and slots. IC-05..IC-07 and the existing payload/tuple tests pin this (FR-149-AC-31). Clean.
- Public API: no `pub` item is added or changed. The new functions are private, so no FR-019 inventory change is needed. Clean.
- The hull-typed `if`: main has no `if` or `let` result check at all. `operand:<n>` result forms are not enforced as type identity anywhere in the reader; only families are checked. A hull-typed `if` whose function result equals its hull is therefore admitted on main. This explains why the coder could not reproduce the refusal named in the ticket comment. Main refuses only when the function's declared result differs from the hull (for example `Integer` over an `Int[0, 20]` if). The PR fixes that through the body-root containment.

## Verdict

Changes required: one finding to fix in this PR (FND-001). The containment predicate and the position set match the amended ruling and FR-322 exactly.

- Mutant M10 (an unresolved `if` branch refuses) is equivalent in practice. A branch with no type node (an `aggregate` term) is already refused at the operand check (P2). A dangling reference is refused by the earlier edge and term stages.
- Mutant M17 (an unresolved `let` value refuses) is equivalent on the same grounds for the bound value.
- Mutant M18 (non-parameter nodes scanned) is equivalent under the FR-322 node bodies. Only a `value`/`parameter` body is `aggregate{[binding "name" = text, ...]}`, so no other node kind can match the name test. It is still a defence that costs nothing to keep.
- The merge conflict expected with open PR #1094 in FR-038 (AC numbering 583..585 against 586..589) and in matrix/tests.md is noted only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The `let` binder is found by name across the whole graph, and the `let` is admitted when any `value`/`parameter` node of that name admits the bound value: `for (position, kind) in kinds.iter().enumerate() { ... if admits_at_result(&nodes[position].semantic_type, Some(bound), ..)? { return Ok(None); } }`. A parameter of another function, or an outer binder with the same name, therefore masks a bad binder. P1: IC-15 (`y: Int[0, 999]` bound to `x: Int[0, 1000]`) plus any parameter `y: Integer` elsewhere is admitted. That package holds a value outside its binder's type. It breaks FR-322 "The `let` binder check" ("A binder whose `semantic_type` is none of them refuses") and the closing guarantee that nothing admitted holds a value outside the type its node names. Common names (`x`, `a`, `n`) make the mask likely in real graphs. Tightest rule, following this file's own `check_quantifier_slot_binder` precedent: the binder is a `value`/`parameter` node of the binding's name that the `let` body (operand 1) reaches by reference (`expression_scope_depth(body, binder)`). Every such node must admit the bound value (all, not any). With no such node there is no read binder, so there is no check. If the shadowing corner `let y = a in let y = b in y` must be exact, also match the binder `level` against the scope level, as `sequence_function_level` does. Add P1 and the shadowing case as tests | crates/filament-contract-model/src/checked_package/v2/operations.rs:3930-3964 |
| FND-002 | low | A `let` whose binding names no `value`/`parameter` node takes no binder check (`Ok(named.then(refuse))` is `Ok(None)` when `named` is false). P3 is admitted although its body reads an `Int[0, 999]` parameter that no binder of this `let` binds over an `Int[0, 1000]` value. Producers never write this, and fixing FND-001 with the body-reach rule closes it the same way (the body's same-named binders are checked), so this needs only a test | crates/filament-contract-model/src/checked_package/v2/operations.rs:3964 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | The `ic_vectors` doc comment says "A vector file is generated from this list: `Case.package` is the wire package of each", but nothing in the repo serializes the list or writes a vector file. It should say that the list can be serialized (each `Case.package` is a complete wire package), or the generator should be added | tests/it/checked_package_v2_operation_arguments.rs:2002-2004 |
| FND-004 | low | IC-15 (`Expect::Unenforced => {}`) asserts nothing, so the known gap is invisible to the test run. Pin today's outcome (`case.admits`, labelled as the FR-322 gap) so that the FR-322 id-link change flips this test deliberately instead of passing silently | tests/it/checked_package_v2_operation_arguments.rs:2066-2070, 2114 |

## Dispositions

Round 1, at head 515f738a (fix commit 515f738a4 over cf214d40). The private review worktree was used with a private target dir, via cargo-slot. `it` (`--features conformance-cli`), name-filtered (operation_arguments, lowering, union, catalogue_differential): 57 pass and 0 fail, with no compiler warnings. `quire validate`: no errors, and the FR-038-AC-588 grammar warning is gone. TC-492 does not collide with main or with open PR #1094. The disposition of the `let` binder check follows the arch ruling, posted as the fix-round comment on QSL-2709: the wire gives the `let` application no node-id link to its binder, a binder is never found by name, and FR-322 is to pin such a link. The check was removed rather than weakened. FR-038-AC-589, the FR-347 paragraph, TC-492 step 4 and the trace row of `tests.md` (Partial) now state that IC-15 is not enforced. No AC, doc comment or trace row claims a binder check. That is an honest deferral. No FSpec ticket id is cited for the id link yet, so the lead should file one and cite it.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | Arch ruling (QSL-2709 fix-round comment): the `let` application carries no node-id link to its binder and a binder is never found by name, so the binder check is removed until FSpec FR-322 pins an id link; FR-038-AC-589 and the trace row state IC-15 is admitted (Partial). Cite the FSpec follow-up ticket |
| FND-002 | deferred | Arch ruling (QSL-2709 fix-round comment): the `let` application carries no node-id link to its binder and a binder is never found by name, so the binder check is removed until FSpec FR-322 pins an id link; FR-038-AC-589 and the trace row state IC-15 is admitted (Partial). Cite the FSpec follow-up ticket; the `named == false` path no longer exists |
