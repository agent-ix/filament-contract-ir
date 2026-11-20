---
id: SR-2026112013214800000002
title: "gap-analysis of QSL-2709 PR #1095 integer containment at result positions"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1095 at cf214d40; FR-038-AC-586..589 and FR-347 against crates/filament-contract-model/src/checked_package/v2/operations.rs and tests/it/checked_package_v2_operation_arguments.rs; upstream filament-specification b90d84ec FR-322-AC-495..502, TC-2099"
review_set: subset
---

## Summary

This gap analysis is planless. Plan completion: not assessed. Ticket: QSL-2709. `quire matrix --scope .` reports FR-038-AC-586, 587, 588 and 589 as `tagged`. AC-586 is bound to `tc_2099_a_function_result_admits_a_contained_integer_root` and `tc_2099_an_expression_root_admits_a_contained_integer_result`. AC-587 is bound to `tc_2099_a_call_argument_keeps_the_exact_match`. AC-588 is bound to the `if`, `case` and `let` tests, and AC-589 to the two `let` binder tests. Every new production branch has an owning AC (FR-038-AC-586..589, FR-347). The upstream ACs FR-322-AC-495..502 were each mapped to a test case:

- IC-01..IC-04 and IC-08: the function-result test.
- IC-05..IC-07: the call test.
- IC-09..IC-13 and IC-16..IC-18: the `if` and `case` tests.
- IC-11, IC-14, IC-15, IC-19 and IC-20: the `let` tests.

## Examined scope

- FR-322-AC-495, 496, 497 (IC-01..IC-04). Admitted or `type-mismatch` at the function node, as specified. Edge cases are added: equal and lower bounds, raised bounds, Boolean. Clean.
- FR-322-AC-498 (IC-05..IC-07). Admitted with a convert, and `operator-ineligible` at `body/arguments/1` with a bare reference. Clean.
- FR-322-AC-500 and 502 (IC-09..IC-13, IC-16..IC-18). Each branch is checked, the hull case is admitted, and a hull that misses either bound refuses. Clean.
- FR-322-AC-501 (IC-14, IC-15, IC-19, IC-20). The listed cases are covered. The masking case is not (FND-001).
- FR-322-AC-499 (IC-08). See FND-002.

## Verdict

Changes required: one coverage gap behind a real defect (code-review FND-001), plus low-severity test-intent items.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No test pins the `let` binder to the let's own binder. Every `let_case` package holds exactly one parameter of the binder's name, so the graph-wide "any same-named parameter admits" scan passes every case. A scratch test (IC-15 plus an unrelated parameter `y: Integer`) is admitted, where FR-322 requires a refusal (code-review FND-001). Add that case, and the shadowing case `let y = a in let y = b in y` | tests/it/checked_package_v2_operation_arguments.rs:1890-1956 |
| FND-002 | low | IC-08 (FR-322-AC-499) is specified as `Sequence<Int[0, 9]>[0, 3]` at `Sequence<Int[0, 10]>[0, 3]`, but the test builds `Ty::Set(.., 0, 3)` (the `Ty` enum has no sequence) and still labels it "IC-08". The constructor rule is the same, but the test is not the vector it names. Add a `Sequence` type to `Ty`, or relabel it and add the `Sequence` case | tests/it/checked_package_v2_operation_arguments.rs:1773-1777 |
| FND-003 | low | No test reaches the `named == false` path, where a `let` binding names no `value`/`parameter` node and is admitted with no binder check (code-review FND-002, scratch probe P3). Mutants M10 and M17, an unresolved branch or value, survive too, but they are equivalent in practice: an untyped branch is refused earlier at the operand check | crates/filament-contract-model/src/checked_package/v2/operations.rs:3964 |

## Dispositions

Round 1, at head 515f738a (fix commit 515f738a4 over cf214d40). The private review worktree was used with a private target dir, via cargo-slot. `it` (`--features conformance-cli`), name-filtered (operation_arguments, lowering, union, catalogue_differential): 57 pass and 0 fail, with no compiler warnings. `quire validate`: no errors, and the FR-038-AC-588 grammar warning is gone. TC-492 does not collide with main or with open PR #1094. The disposition of the `let` binder check follows the arch ruling, posted as the fix-round comment on QSL-2709: the wire gives the `let` application no node-id link to its binder, a binder is never found by name, and FR-322 is to pin such a link. The check was removed rather than weakened. FR-038-AC-589, the FR-347 paragraph, TC-492 step 4 and the trace row of `tests.md` (Partial) now state that IC-15 is not enforced. No AC, doc comment or trace row claims a binder check. That is an honest deferral. No FSpec ticket id is cited for the id link yet, so the lead should file one and cite it.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | Arch ruling (QSL-2709 fix-round comment): the `let` application carries no node-id link to its binder and a binder is never found by name, so the binder check is removed until FSpec FR-322 pins an id link; FR-038-AC-589 and the trace row state IC-15 is admitted (Partial). Cite the FSpec follow-up ticket; no masking test applies while no binder check is made |
| FND-002 | fixed | - |
| FND-003 | deferred | Arch ruling (QSL-2709 fix-round comment): the `let` application carries no node-id link to its binder and a binder is never found by name, so the binder check is removed until FSpec FR-322 pins an id link; FR-038-AC-589 and the trace row state IC-15 is admitted (Partial). Cite the FSpec follow-up ticket; the untested `named == false` path was removed with the check |
