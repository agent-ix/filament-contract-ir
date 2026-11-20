---
id: SR-17940486709564847048117
title: "code-review of QSL-2726 seq binder level at temporal clause leaves (code/seq-binder-level)"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; branch code/seq-binder-level at b2cc62a3e (git diff origin/main...HEAD, main 6f097b90a; no PR); crates/filament-contract-model/src/checked_package/v2/operations.rs, tests/it/checked_package_v2_temporal.rs"
review_set: subset
---

## Summary

Ticket: QSL-2726. This is a code review with the rust-review lane folded in. No PR exists and gh is unauthenticated. The review covers the committed diff at b2cc62a3e only.

The worktree `/home/peter/dev/worktrees/QSL-seq-binder` also holds uncommitted changes to `operations.rs`, `checked_package_v2_temporal.rs` and the TC-048 matrix. Someone is working on capture initializers there. Those changes were not reviewed as part of the head, and the tests below ran against a `git archive b2cc62a3e` snapshot, not the dirty tree.

The change: `sequence_function_level` gains a pass over `temporal`/`temporal_clause` nodes through `clause_leaf_levels`. The base is 1 for `over`, plus 1 for a trigger, plus 1 per capture. The leaves are the activation guard and every formula node reachable from argument 5, and the level is base + `expression_scope_depth`.

Authorities checked:

- FSpec FR-322 "Parameter" (level = position among the binders in scope where it is bound).
- FSpec FR-370 (clause body arguments).
- FSpec FR-402 (hyper body wrapper).
- FSL FR-093's lowering table and AC-5/AC-15.
- FSL FR-337 (activation and capture scopes).
- FSL's lowering in `fsl-semantics/src/check/lowering/{temporal.rs,hyper.rs,lowering.rs}` at FSL b1157384c.
- All eight FSpec published clause fixtures, which give `over` level 0.

FSpec has no published vector with a `seq`, a trigger or a capture inside a clause.

Base arithmetic that matches FSL:

- `over` is 0, so the first leaf binder is at 1, and base = 1 is right.
- The trigger is at 1 (temporal.rs:278).
- Captures follow from `scope.len()` (temporal.rs:290-303).
- `holds` and window-aggregate leaves are lowered over the full scope (temporal.rs:358).

So the formula-leaf base is right.

Tests (one cargo slot through `flock /tmp/cargo-slot.lock`, `-j 4`, `--features conformance-cli`, FILAMENT_SPECIFICATION_DIR = FSpec main cd89fcb3, run on a b2cc62a3e snapshot that adds two reviewer probe tests, with a private CARGO_TARGET_DIR inside the snapshot): `cargo test --locked -j 4 --features conformance-cli --test it -- tc_048_a_seq_binder reviewer_probe` gave 3 passed, 0 failed.

- `tc_048_a_seq_binder_in_a_clause_leaf_resolves_its_level_from_the_binders_in_scope` passes at HEAD.
- `reviewer_probe_fsl_let_value_level_refused` passes. It shows that the level FSL emits for `let s = seq(0..0: i) in B` (1) is refused.
- `reviewer_probe_guard_counts_captures` passes. It shows that a guard `seq` with one capture is admitted at 4 and refused at 3: the guard counts the capture.

A first attempt shared the worktree's `target/` and ran the dirty tree's binary instead of the snapshot. Cargo hashes workspace members by their relative path, so the two trees collide. That run failed the uncommitted draft's `capture 1 initializer, level 3` case.

quire-0.52 cannot start (a `spec-artifacts-process` 0.2.0 manifest error: `traceability.trace_tags: unknown field legacy`), so this SR was validated with quire 0.36.2, exit 0.

## Verdict

**NOT MERGEABLE.** There is one high finding and three medium findings.

The base for formula leaves is right. The `let` depth is inverted, though, and the new test asserts the inverted level, so it admits a package FSL never emits and refuses the one FSL does emit (FND-001).

Three cases are wrong for packages FSL emits:

- the guard counts captures that are not in its scope (FND-002);
- hyper body wrappers get base 1 instead of their atom-slot count (FND-003);
- capture initializers are not leaves at all (FND-004).

Termination is sound: a `visited` set over formula positions guards every formula-reference cycle. A formula node shared between clauses cannot cause a false conflict, because a different level means a different binder node, so a different seq node and formula node. The one exception is guard against body inside one clause (FND-002).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `expression_scope_depth` adds 1 for the *value* of a `binding` argument and 0 for a plain argument. A `filament.op.control.let` is `[binding{name: x, value: ref(v)}, ref(b)]` (FSL FR-093 lowering table). The bound value `v` is outside `x`'s scope, and the body `b` is inside it: FSL lowers `v` before it binds `x` (lowering.rs:4667-4674, 6566-6583), FSL FR-093-AC-15 gives the same, and so does FSpec FR-322's "position among binders in scope where it is bound". The new pass therefore puts `let s = seq(0..0: i) in B` at a clause leaf at level 2, where FSL writes `i` at level 1. The new test asserts the inverted value: the doc comment says "the `let` binds `s` at level 1, so `i` sits at level 2", `admitted("seq binder at level 2", ...)`, and `for wrong in [0, 1, 3]` expects the FSL-correct level 1 to refuse. A `seq` in a `let` body is under-counted by one. `fold`/`reduce` (`binding{acc, aggregate{binding{x, step}}}`) is under-counted too, at +1 where it should be +2. The depth rule predates this change (it is on main for function bodies), but the new pass and test build on it and make it look correct. Fix: depth +0 for a `let`'s operand-0 binding value and +1 for its body; +1 for each nested binding of `fold`/`reduce`. Rewrite the fixture so the `seq` sits in the `let` body, or expect level 1 | crates/filament-contract-model/src/checked_package/v2/operations.rs:1497-1507; tests/it/checked_package_v2_temporal.rs:5145-5150; tests/it/checked_package_v2_temporal.rs:5298-5372 |
| FND-002 | medium | The activation guard is read at `base`, which includes the capture count. FSL lowers the guard over `guard_scope`, which is `over` and the trigger only, taken before the captures (temporal.rs:289, 461-463). FSL FR-337 states it too: "A capture is ... not in the scope of the activation's `when` guard". A `seq` in the guard of a clause with captures is refused at the FSL level and admitted at the FSL level + captures. The probe `reviewer_probe_guard_counts_captures` confirms this at HEAD. The new test exercises the guard only with 0 captures, so this is invisible to it | crates/filament-contract-model/src/checked_package/v2/operations.rs:1644-1651; tests/it/checked_package_v2_temporal.rs:5317 |
| FND-003 | medium | `clause_leaf_levels` uses base 1 for every `temporal_clause` node. FSL's hyper body wrapper (FSpec FR-402) is also a `temporal`/`temporal_clause` node. Its `over` is the first atom slot, and its formula is lowered over every atom slot, at levels 0..k-1 (hyper.rs:291-349), so its leaf base is k, not 1. A `seq` in a hyper clause with two or more atom slots is refused at its FSL level. Every hyperproperty over two traces has at least two slots. Neither the code nor the test distinguishes the wrapper | crates/filament-contract-model/src/checked_package/v2/operations.rs:1630 |
| FND-004 | medium | Capture initializers (argument 3's binding values) are not leaves. A `seq` that occurs only in a capture initializer is never reached, so `found` stays `None` and the reader refuses a package FSL emits: FSL lowers initializer k over `over`, the trigger and captures 0..k-1 (temporal.rs:292-300), so its base is 1 + trigger + k. The arch ruling ("wherever an expression may appear, incl. a temporal clause leaf") covers it. The uncommitted worktree edit starts on this, but `scope = scope.saturating_add(0)` never advances, so every initializer gets base 1 + trigger, and its own `(1, 3)` case would fail | crates/filament-contract-model/src/checked_package/v2/operations.rs:1644-1646 |
| FND-005 | low | The new traversal's `visited.contains(&position)` is an unmetered linear scan per pop, so the work is O(F^2) with F formula nodes. `expression_scope_depth` charges each comparison in its `visited` scan (operations.rs:1482-1483). For every `seq` application, every clause also reserves three `graph.nodes.len()` vectors and runs `expression_scope_depth` per root. That is O(seq x clauses x (3n + roots x 2n)) of cumulative FR-358 storage, which can exhaust a budget on a large valid package. Charge the scan, and either compute the clause bases once per package or stop at the first clause that reaches the target | crates/filament-contract-model/src/checked_package/v2/operations.rs:1647-1664 |
| FND-006 | low | The test is tagged `#[trace("TC-048", "FR-322-AC-455")]`. FSpec FR-322-AC-455 covers the `only`/`seq`/`history` catalog entries and `only` in a clause function body and a refinement row. It says nothing about a `seq` binder's level at a temporal clause leaf, and the uncommitted TC-048 edit itself says "FSpec holds no acceptance criterion with this numbering". The binding is wrong. Tag an IR TC-048 step and AC that states the rule, or get the FSpec AC first | tests/it/checked_package_v2_temporal.rs:5297 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | medium | The new `fold`/`reduce` branch of `expression_scope_depth`, where the step sits at depth + 2, has no test, and the coder reports that the `at(2)` to `at(1)` mutant survives. The coder argues it is unreachable ("a fold step that is also a seq is not type-correct"), but the step need not *be* the `seq`. `expression_scope_depth` follows the step's `reference` into any expression node, so a step such as `acc + size(seq(0 .. 3: j))` reaches the branch, and FSL writes `j` two levels inside the fold's enclosing scope. The surviving mutant would admit a `seq` binder one level short. Add one focused test with a `seq` nested in a `fold` (or `reduce`) step, admitted at the level FSL writes and refused one level either side. A crate-internal unit test of `expression_scope_depth` over a hand-built graph is an acceptable alternative | crates/filament-contract-model/src/checked_package/v2/operations.rs:1515-1527 |

## Dispositions

Round 1, at head b55c841e0 (commits a68190b5b and b55c841e0). Focused run on the clean worktree at HEAD (one cargo under the flock, `-j 4`, `--features conformance-cli`, FSpec main cd89fcb3): `tc_048_a_seq_binder_in_a_clause_leaf_resolves_its_level_from_the_binders_in_scope` and `collection_seq_requires_bounded_inputs_matching_binder_and_sequence` both passed (2 passed, 0 failed). I checked the binary's strings to confirm it was built from HEAD.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `expression_scope_depth` now pushes a `let`'s binding value at `depth` and every argument after it at `depth + 1`. I re-checked this against FSL lowering.rs:4667-4674 (`NodeKind::Let` descends into `value` before any bind) and 6566-6583 (`bind`, then descend `body`, then `pop`): the value is outside the binder and the body is inside. The fixture now admits `let s = seq(0..0: i) in B` at 1 and a `seq` in a nested `let` body at 2, each refused one level either side. That matches FSL FR-093-AC-15 (`let v = x in ((let w = x in w) + ...)`, `v` at 4 and `w` at 5). `fold`/`reduce` steps are at +2, which is right but untested (FND-007) |
| FND-002 | fixed | The guard is pushed at `guard_scope`, taken after the trigger and before the captures. The test admits a guard `seq` at 2 with 0 and with 2 captures, refusing 1 and 3 |
| FND-003 | deferred | Refusing is acceptable for this PR. The wrapper's leaf base is its atom-slot count k (FSL hyper.rs:291-349, FR-487 "Atom slots"), and the wire does not give k for a wrapper: slot parameters carry no owner, and rebuilding FR-487's alias pairing from the hyperproperty's bindings has no FSpec rule. Refusing is fail-closed (no wrong-level admission), the TC-048 step documents it, and the test pins the refusal at levels 0 to 3. Admitting needs an FSpec AC that fixes the wrapper's scope, either on the wire or as a rule. The code lead should file that with spec-review FND-002's FSpec ticket; until then this is a known under-admission of FSL output |
| FND-004 | fixed | Each capture initializer is a root at `1 + trigger + k` (`scope` is pushed before it advances). The test covers captures 0 and 1, with and without a trigger, at levels 1, 2, 2 and 3, each refused one level either side |
| FND-005 | fixed | `ClauseLeaves` reserves its scratch once per `seq` application through the meter (`meter.vec` and `meter.filled`). A per-clause stamp replaces the unmetered `visited.contains` scan, so each check is O(1). The wrapper scan is charged per element through `work_at_path`. No work-limit test was added, which matches the other passes of `sequence_function_level` |
| FND-006 | fixed | The test is now `#[trace("TC-048")]` alone, and the TC-048 step says no FSpec AC states the numbering yet |

Round 2, at head 48075eccc. The delta from b55c841e0 is `tests/it/checked_package_v2_temporal.rs` only (78+/6-). The focused run of `tc_048_a_seq_binder_in_a_clause_leaf_resolves_its_level_from_the_binders_in_scope` on the clean worktree at HEAD (one cargo under the flock, `-j 4`, `--features conformance-cli`, FSpec main cd89fcb3) gave 1 passed, 0 failed, and the binary was confirmed built from HEAD.

| FND | outcome | reason |
| --- | --- | --- |
| FND-007 | fixed | The test now puts `size(seq(...))` in a `fold` step (`[ref(set), binding{acc, aggregate{binding{j, ref(size)}}}, literal 1]`) and in a `reduce` step, each the bound value of the outer `let`. Each is admitted at 3 and refused at 2 and 4 through `admits_only_at`. FSL writes 3 here: `over` at 0, then `acc` at 1 and `j` at 2, with the `let` value outside `s`. The `at(2)` to `at(1)` mutant would admit 2, so it now dies |
