---
id: SR-17940486709564847048118
title: "gap-analysis of QSL-2726 seq binder level at temporal clause leaves"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; branch code/seq-binder-level at b2cc62a3e (git diff origin/main...HEAD); FSpec FR-322 Parameter rule, FSpec FR-370 clause body, FSpec FR-402 body wrapper, FSL FR-337 activation/capture scopes, FSL FR-093 let/fold lowering; IR TC-048; crates/filament-contract-model/src/checked_package/v2/operations.rs, tests/it/checked_package_v2_temporal.rs"
review_set: subset
---

## Summary

This is a planless gap analysis (Plan completion: not assessed). Ticket: QSL-2726. It was done by hand, scoped to the change, because the Quoin matrix run was not used for this one-test diff. It checks each place FSpec and FSL let an expression appear in a temporal clause against what the code reaches and what the test asserts.

The authority is the arch ruling as relayed in the brief: a `seq` binder is a binder like any other, and the FR-322 general binder rule governs wherever an expression may appear.

| Leaf | Scope per FSL lowering and FR-337 | Code at HEAD | Test at HEAD |
| --- | --- | --- | --- |
| `holds` operand and window-aggregate terms | `over`, trigger, every capture | base 1 + trigger + captures: right | covered (0/1 captures, trigger) |
| activation guard | `over` and trigger only | base includes the captures: wrong (FND-002) | covered with 0 captures only |
| capture initializer k | `over`, trigger, captures before k | not a leaf: refused (FND-003) | not covered |
| hyper body wrapper formula | every atom slot (k) | base 1 (FND-004) | not covered |
| a `let` inside a leaf | value outside the binder, body inside it | inverted (FND-001) | asserts the inverted level |

## Verdict

**Gaps found.** The test is not tautological. It has admitted and refused twins for each axis it covers, so mutating the base, the trigger count, the capture count, the guard root or the conflict check each fails it, and the clause's formula is a two-node chain (`eventually` to `holds`), so the transitive walk is exercised too. Some of what it asserts is wrong, though, and three leaf kinds the arch ruling covers are missing.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The test's anchor case asserts that `let s = seq(0..0: i) in B` at a clause leaf puts `i` at level 2 and that level 1 refuses. Under FR-322's rule ("position among the binders in scope where it is bound") and FSL's lowering (the `let` value is lowered before `s` is bound), `i` is at level 1. The test codifies the inverted `let` depth of `expression_scope_depth` (code-review SR FND-001) | tests/it/checked_package_v2_temporal.rs:5145-5150; tests/it/checked_package_v2_temporal.rs:5300-5303; tests/it/checked_package_v2_temporal.rs:5362-5372 |
| FND-002 | medium | No test puts a `seq` in the guard of a clause with captures. FSL FR-337 keeps captures out of the guard's scope, and the code counts them (code-review SR FND-002) | tests/it/checked_package_v2_temporal.rs:5316-5326 |
| FND-003 | medium | Capture initializers are expression positions of the clause (FR-370 argument 4), and the ruling covers them. HEAD neither implements them nor tests them. The uncommitted worktree draft has a test for them, which fails because its base never advances | crates/filament-contract-model/src/checked_package/v2/operations.rs:1644-1646 |
| FND-004 | medium | A hyper body wrapper (FR-402) is a `temporal_clause` whose leaf scope is its atom slots. No test covers a `seq` in a hyper clause leaf, and the code's base 1 is wrong when there are two or more slots | crates/filament-contract-model/src/checked_package/v2/operations.rs:1630 |
| FND-005 | low | The trace binding `FR-322-AC-455` does not state the rule under test, and no IR TC-048 step at HEAD describes it. The TC-048 text exists only as an uncommitted edit | tests/it/checked_package_v2_temporal.rs:5297 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | medium | The TC-048 step's "a `fold` or `reduce` step is inside its accumulator and element binders (two further)" has no test, and its mutant survives. The coder's unreachability argument is wrong, because a `seq` nested inside a step expression reaches the branch (code-review SR FND-007) | crates/filament-contract-model/src/checked_package/v2/operations.rs:1515-1527 |

## Dispositions

Round 1, at head b55c841e0.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The anchor case now admits `let s = seq(0..0: i) in B` at level 1, the level FSL writes, and a `seq` in a nested `let` body at 2, each refused one level either side through `admits_only_at` |
| FND-002 | fixed | The guard is covered with 0 and 2 captures, admitted at 2 and refused at 1 and 3 |
| FND-003 | fixed | Capture initializers 0 and 1, with and without a trigger, are admitted at 1, 2, 2 and 3 and refused one level either side |
| FND-004 | deferred | The hyper-wrapper leaf is now tested as a refusal at levels 0 to 3, which pins the fail-closed behaviour. Admission waits on an FSpec rule for the wrapper's atom-slot scope (code-review SR FND-003) |
| FND-005 | fixed | The test carries `#[trace("TC-048")]`, and the committed TC-048 step states the rule it verifies |

Round 2, at head 48075eccc (test-only delta).

| FND | outcome | reason |
| --- | --- | --- |
| FND-006 | fixed | The `fold`/`reduce` step rule of the TC-048 step is now tested: a `seq` nested in each step is admitted at level 3 and refused at 2 and 4 |
