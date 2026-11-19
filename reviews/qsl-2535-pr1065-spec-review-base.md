---
id: SR-17940486709564847048088
title: "spec-review of FR-038-AC-570..573 and FR-369 rule 4 for QSL-2535"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1065 at 061abd3d (base origin/main c7b62038); spec/checked_package/functional/FR-038-consume-checked-package-v2.md (AC-559, AC-570 to AC-573), spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md (rule 4), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md"
review_set: subset
---

## Summary

Spec review of the contract-ir requirement text that PR #1065 adds or changes, checked against FSpec W76 99226882 as the authority. Ticket: QSL-2535. `quire validate` on each changed file: no errors, and the same 4 FR-038 grammar warnings (`ac:non-canonical-shape`) as on main, so nothing new.

## Examined scope

- FR-038-AC-559 (examined). It adds the reward `kernel` refusal and matches FSpec FR-770-AC-2 (W76).
- FR-038-AC-570 (examined). The "binder field list is not read (W75)" clause is replaced by a pointer to AC-573. This is consistent.
- FR-038-AC-571 (examined). Owner, `no_body` and `kernel` all match FSpec FR-322 "Owner joins", FR-322-AC-471 and FR-770-AC-2. The source-declared deferral is FND-001.
- FR-038-AC-572 (examined). The `precondition`, `body` (with or without rows), `invariant`/unknown and other-operation cases match FR-640 rule 4 and FR-322-AC-471. The locus `/body/members/4/value/target` is how this repo spells FSpec's `/body/expression`; FR-369's existing rule-4 ACs use the same convention, so it is not a defect. The no-`operation_bodies` case (rule 4 at the expression) is a stated extension that FSpec does not contradict.
- FR-038-AC-573 (examined). The receiver plus one field per parameter, with no result and not `B`, matches FR-432 "The binder record of an abstract node" and FR-375.
- FR-369 rule 4 (examined). It adds the `reward` clause-kind condition and agrees with FSpec FR-640 rule 4.
- TC-048 section and the tests.md row (examined). They agree with the ACs.

## Verdict

The new and changed text is faithful to W76 and testable. One low wording defect: the deferral pointer in AC-571 names an AC that does not cover the deferred case.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-038-AC-571 says a source-declared `reward` clause function "is outside this reader until source-declared probabilistic nodes join (FR-369-AC-25)". But FR-369-AC-25 (and AC-26, and TC-484) states only delay, random_parameter and workload, so the pointer names an AC that will not bring the reward case in. Extend the FR-369 Planned ACs with FSpec FR-640-AC-7's SourceOwner reward declaration (`[<type>,<op>,"reward",<name>]`), or cite a new Planned AC | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6716 |

## Dispositions

Round 1, at head 0faf1c2b, with origin/main 358e9e2d merged.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-038-AC-571 now reads "is the case FR-369's source-declared `reward` rule states; its owner and reader are planned under QSL-2465 and are not part of this criterion". With #1075, FR-369-AC-25/26 state the source reward function, so the pointer resolves |
