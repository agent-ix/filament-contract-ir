---
id: SR-2026112105511900000003
title: "spec-review base for QSL-2718 PR #1109 (no spec change)"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1109 at de9833fe; spec/ unchanged; spec/checked_package/functional/FR-060-admit-model-terminal-declarations.md"
review_set: subset
---

## Summary

Ticket: QSL-2718. PR #1109 changes no file under `spec/` or `plan/`. This record checks the IR spec FR-060 against FSpec FR-476 (main 6ad873d5; the #778 branch 06e768a1 is identical for FR-476 and FR-366) for the `model` binding rule that the PR implements.

## Examined scope

- FR-060 Behavior, "`model` references the selected model node" and "A non-model `model` target refuses `invalid_model_binding`/`malformed-declaration`" (examined). FSpec FR-476 Properties and Reader order step 4 say instead: "a `model` target that is not a `model`/`model_root` node refuses". FR-060 is weaker than its oracle, and it permits the leniency the code has (FND-001).
- FR-060 Behavior, `self` "typed `Reference<M::T>` for the referenced model's FR-366 object type" (examined). This agrees with FR-476 and FR-366. It needs no edit.
- FR-060-AC-2 (examined): it names MT-10, whose second case is the `object_type` target. The AC is correct as written. The code does not meet it.
- Ticket numbering (context_only): QSL-2718 cites "FR-411", which is the FSL requirement. The contract-ir oracle is FSpec FR-476.

## Verdict

No spec file is touched by the PR. FR-060 Behavior lags FSpec FR-476 on the `model` target kind and should be tightened together with the reader fix (code-review FND-001).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-060 Behavior says "a non-model `model` target refuses". FSpec FR-476 requires refusal for any target that is not `model`/`model_root`. FR-060 under-specifies the oracle, which lets the reader admit `object_type` targets. | spec/checked_package/functional/FR-060-admit-model-terminal-declarations.md:45-46, spec/checked_package/functional/FR-060-admit-model-terminal-declarations.md:72-73 |

## Dispositions

Round 1, at b201ff1d. No spec change, as the ruling requires.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | deferred | arch ruling: #1109 admits both ModelRoot and ObjectType; the object_type refusal is follow-up PR scope remaining on QSL-2718, after the FSL emitter emits model_root; tighten FR-060 Behavior in that follow-up PR |
