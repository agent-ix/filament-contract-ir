---
id: SR-5761729
title: "spec-review/integrity of IR-1615 FR-038 event-typed parameter amendment"
type: SpecReview
analysis: integrity
scope: "agent-ix/filament-contract-ir; PR #1174 at eff94c5d093c6908f8dd5a51984494940398b825; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (parameter paragraph, FR-038-AC-632, FR-038-AC-633)"
review_set: subset
---

## Summary

Spec-review/integrity of the FR-038 parameter paragraph amendment and the two new ACs, checked against filament-specification origin/main fd90d643: FR-322 (parameter node, event equality paragraph, FR-322-AC-68), TC-281 EV-01 to EV-06, FR-370 "Properties" (`over` argument) and FR-208-AC-9. Ticket: IR-1615. `quire validate --scope .` on FR-038 reports no error and no warning on the changed lines. EARS/shape was checked inline here rather than as a separate sub-analysis: the change is one sentence pair and two ACs, both direct assertions.

## Examined scope

- Parameter paragraph (examined): "a `model`/`event_type` declaration node that carries a `ModelOwner`" matches the code (owner variant `Model`) and FSpec FR-322-AC-68's owner rule.
- New paragraph (examined): "No operation over a parameter typed by an event type is admitted ... The one exception is the first operand of a temporal clause ... Field projection of an event value (FSpec FR-322-AC-68) is therefore not yet admitted, and neither is record conformance between event types." States the deliberate divergence from FR-322-AC-68 / EV-03 openly.
- FR-038-AC-632, FR-038-AC-633 (examined): atomic enough to test, literal codes, causes and loci.
- FSpec FR-322 event equality (context): "`filament.op.structural.eq` and `filament.op.structural.ne` refuse an application whose operand 0 type node is, or contains at any depth through option, tuple, union and collection element positions, a `model`/`event_type` node ... FR-208 gives an event type no equality, and no record, field or parameter holds an event (FR-208-AC-9)".

## Verdict

Acceptable with two findings. The amendment is honest about what it does not admit, but it rests on a reading of FSpec that FSpec does not state, and its ACs carry no FSpec citation.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038 now admits an event-typed `value`/`parameter` node, while FSpec FR-322's event-equality paragraph says "no record, field or parameter holds an event (FR-208-AC-9)". The IR-1615 ticket reads "parameter" there as model data, not a clause binder, and calls that "a reading, not a stated rule"; FR-208-AC-9 itself speaks only of field, population, frame and relationship `typeRef`s. FR-038 records no FSpec ticket or ADR for the reading, so the two specs disagree on their face; an FSpec clarification (clause and quantifier binders may be event-typed) should be filed and cited | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3266-3285 |
| FND-002 | low | FR-038-AC-632 and FR-038-AC-633 have Verification `Test` with no FSpec citation, unlike their neighbours (`Test (TC-048; FSpec FR-322-AC-...)`). AC-632's owner-join half restates FSpec FR-322-AC-68 / TC-281 EV-02, and AC-633 refuses what TC-281 EV-03 admits (`record.project` over an event value); both relations should be cited so the divergence is traceable | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:7179-7180 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 8397db689: FR-038 now states the divergence and cites STD-728 ("FSpec FR-322: state that a clause binder may be typed by an event type"), filed and related to IR-1615. |
| FND-002 | fixed | 8397db689: AC-632 cites `Test (TC-048; FSpec FR-322-AC-68, TC-281 EV-01, EV-02)`, AC-633 cites `Test (TC-048; FSpec TC-281 EV-03, EV-04)` and states the EV-03 divergence. |
