---
id: SR-17940486709564847048119
title: "spec-review of the QSL-2726 seq-binder level rule as applied to temporal clause leaves"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; branch code/seq-binder-level at b2cc62a3e (no spec file in the committed diff); IR spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md (uncommitted worktree edit, read as context); FSpec FR-322 lines 393-412 and FR-322-AC-455, FSpec FR-370, FSpec FR-402; FSL FR-093, FSL FR-337"
review_set: subset
---

## Summary

Ticket: QSL-2726. The committed diff changes no spec file. This review checks the level rule the code and the test claim against the specifications. It also covers the uncommitted TC-048 step that the worktree is drafting, so the next fix round does not commit it as written.

FSpec FR-322 says a `value`/`parameter` node's `level` is "its position among the binders in scope where it is bound (FSL FR-092)". FSpec FR-370 lists the clause's arguments and, for the window atom, says "Expression nodes are in the clause's binder scope (its `over` parameter and captures), as a `holds` operand is". FSpec states no level or scope for the guard, the trigger or the capture initializers. FSL FR-337 and FSL's lowering do:

- the trigger is at slot 1;
- the captures follow at the next slots;
- a capture is out of scope in the guard, in its own initializer and in earlier initializers;
- holds conditions are lowered over all of these.

All eight FSpec clause fixtures give `over` level 0. FSpec publishes no vector with a `seq`, a trigger or a capture in a clause.

## Verdict

**Changes needed.** The `over` level, trigger and capture numbering the brief describes agrees with FSL. The draft TC-048 step contradicts FSL FR-337 on the guard, and the `let` example the test and the draft rely on contradicts FR-322's own rule. FSpec should state the clause-leaf scope rule, because the IR currently derives it from FSL alone.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The draft TC-048 step says "The activation guard and every formula leaf are read over all of the clause's binders". FSL FR-337 says a capture "is not in the scope of the activation's `when` guard", and FSL lowers the guard over `over` and the trigger only (temporal.rs:289, 461-463). The step should give the guard base 1 + trigger | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:3130-3131 |
| FND-002 | medium | FSpec FR-322 and FR-370 give no level rule for clause leaves: not the guard, not the capture initializers, and not the hyper body wrapper, whose leaf scope is every atom slot (FSL hyper.rs:291-349). The IR rule rests on FSL's lowering and on an arch ruling. The draft step's own last line ("FSpec holds no acceptance criterion with this numbering: FSpec needs the AC text") records the gap. File an FSpec ticket for an FR-322 or FR-370 AC that states the clause-leaf scopes, the hyper wrapper's included | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:3134-3135 |
| FND-003 | low | The draft step and the test cite FR-322-AC-455, which is about the `only`/`seq`/`history` catalog entries and `only` placements, not binder levels. The citation should go until an AC that states the rule exists | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:3124-3125 |

## Dispositions

Round 1, at head b55c841e0.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The committed TC-048 step now reads: "the activation guard over `over` and the trigger only (a capture is not in the guard's scope, FSL FR-337); the initializer of capture `k` over `over`, the trigger and the captures declared before `k`" |
| FND-002 | deferred | This is an FSpec gap that the IR repo cannot close. The step records it ("No FSpec acceptance criterion states this numbering yet ... FSpec needs the AC text"). The code lead should file the FSpec ticket for an FR-322 or FR-370 AC covering the clause-leaf scopes and the hyper wrapper's atom-slot scope. That ticket also unblocks code-review FND-003 |
| FND-003 | fixed | The step now says "FR-322-AC-455 covers the catalog entries only, so the test is tagged TC-048 alone", and the test tag is `TC-048` |
