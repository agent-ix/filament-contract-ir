---
id: SR-5761731
title: "spec-review (integrity) of IR-1619 FR-038 event-reach rule"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1177 at 9a275b2109622e73721e5f673688709916741bfd, delta over 9961b272b2be2750ea365e9d57b20b53ad3c94d5; spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: subset
---

## Summary

Spec review (integrity sub-analysis) of the FR-038 changes in PR #1177. Ticket: IR-1619. Read against filament-specification origin/main: FR-322 event equality paragraph and family table, FR-322-AC-65, FR-322-AC-68, FR-208-AC-9, TC-281 EV-03..EV-06.

## Examined scope

- FR-038 event-reach paragraph ("No operation over a value whose type reaches an event type is admitted ..."), examined.
- FR-038 divergence paragraph ("This diverges on purpose from FSpec TC-281 EV-03 ..."), examined.
- FR-038-AC-633 (edited `contains` locus), examined, clean.
- FR-038-AC-634, examined.
- FR-038-AC-635, examined, clean. It matches TC-281 EV-06.
- The question of `record.project` of a non-event field of a record that holds an event, examined. FSpec has no such record. FR-208-AC-9 and FR-322-AC-65 refuse an event-typed field at intake (`invalid_model_binding`/`malformed-declaration`), and FR-322 says "no record, field or parameter holds an event". Refusing the projection therefore refuses nothing that FSpec admits. The FSpec refusal belongs at intake, with a different code, and that is the intake ticket's work.
- Admitting type and parameter nodes that hold events while refusing at the operation, examined, clean. TC-281 EV-05 itself takes `Option<OrderPlaced>` and `Set<OrderPlaced>` operands and expects the refusal at the `eq` application, so these nodes are admitted in FSpec.

## Verdict

FR-038 now refuses more than FSpec does, and records only part of that as divergence (FND-001). The AC-634 model-record case depends on an intake gap (FND-002).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038 refuses every operation over a value whose type reaches an event: `option.value`, a `forall` over `Set<OrderPlaced>`, `contains`, `ne`, and `record.project` over a record holding one. Its divergence paragraph names only TC-281 EV-03 and record conformance. FSpec FR-322 gives `model`/`event_type` family `record` and limits the event exception to `structural.eq`/`ne`, so it admits the rest. The broader divergence and its STD ticket are not recorded. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3280-3322 |
| FND-002 | low | FR-038-AC-634's case "eq over a model record value type whose field is a domain package sequence of it" asserts behaviour for a document that FSpec (FR-322 "no record, field or parameter holds an event"; FR-208-AC-9) refuses at intake. The AC and its test do not say that this rests on an intake gap still open in the reader. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:7216 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | The new FR-038-AC-635 clause pins as admitted an `eq` over a model record whose field names an object type. FSpec FR-208-AC-9 refuses a field `typeRef` naming any business model meaning other than a record value type at intake, so the AC asserts admission of a document FSpec refuses. The AC does not cite the open intake gap (IR-1620) the way AC-634 now does. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:7237 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The divergence paragraph now names FR-322's eq/ne-only rule and the `record` family, lists project, option.value, forall, contains and every other collection operation, and cites STD-728. |
| FND-002 | fixed | AC-634 now says that its model-record cases rest on a model intake gap (FR-208-AC-9 and FR-322), and that closing the gap changes those cases. |
