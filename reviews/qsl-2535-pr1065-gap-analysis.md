---
id: SR-17940486709564847048087
title: "gap-analysis of QSL-2535 reward clause kind and Recommend binder"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1065 at 061abd3d (base origin/main c7b62038); FR-038-AC-571, FR-038-AC-572, FR-038-AC-573, FR-369 rule 4 against FSpec W76 FR-322-AC-471, FR-640-AC-7, FR-770-AC-2, FR-432-AC-23, FR-432-AC-24 and the changed tests"
review_set: subset
---

## Summary

Gap analysis of the ACs that PR #1065 declares, checked against the W76 FSpec criteria they consume and against the tagged tests. Ticket: QSL-2535. Quoin 0.29.1. Trace tags at head:

- `the_reader_admits_the_reward_clause_kind_and_demands_no_operation_body`: tagged FR-038-AC-571.
- `a_reward_clause_function_joins_only_its_operation_member_and_is_linked`: tagged FR-038-AC-571.
- `a_reward_expression_of_another_clause_kind_refuses`: tagged FR-038-AC-572.
- `refinement_catalog_maps_a_recommend_as_a_node_row_class`: tagged FR-038-AC-570 and FR-038-AC-573.
- `inherited_operation_binders_use_named_receiver_and_retained_result` (lib): tagged FR-038-AC-573.

Each binding matches what the test asserts. FR-038-AC-571 to AC-573 are new. No open PR and no commit on origin/main c7b62038 uses those numbers. TC-048 `relationships` and the tests.md row are added. `quire validate` on the four changed spec files gives the same 4 grammar warnings as main and no errors.

## Examined scope

- FSpec FR-322-AC-471 (examined). Covered:
  - the `reward` clause, the operation-member owner and the `no_body` row;
  - the object-type owner and the field-member owner, each refusing `missing-selection`;
  - the `precondition` clause refusing rule 4;
  - the `body` clause refusing at `/operation_bodies/{i}/body`, with no rule-4 refusal.
  "The precondition clause functions read for Server::attempt hold no reward function" has no contract-ir consumer: no API reads preconditions from clause functions. It is N/A here.
- FSpec FR-640-AC-7 (examined). The model half is covered: the `precondition` clause and `Server/reset` (here `withdraw`) both refuse at the expression. The source half (`Other::poke`, a `SourceOwner` with declaration `["Other","poke","reward","cost"]`) is not covered: see FND-001.
- FSpec FR-770-AC-2, reward `kernel` (examined): covered.
- FSpec FR-432-AC-23/24 and FR-375 catalog (examined). On the concrete side, the recommend binder `[receiver, k]` admits, and `[B]`, `[]`, `[receiver]` and `[receiver, k, result]` refuse. The abstract external side is not exercised: see code-review FND-001 (SR-17940486709564847048086). The step-row decisions of AC-23 (operation row versus node row, product exploration) are FSL-side checks, not this reader's.
- FR-038-AC-571, AC-572 and AC-573 (examined). Each is backed by its tagged tests, as above.

## Verdict

The model-owned half of the W76 reward clause kind and the concrete-side Recommend binder are fully covered and tagged. One coverage gap is deferred without an owner (FND-001).

On whether the SourceOwner reward case belongs in this PR: it does not. It needs the source-declared probabilistic join, which is still Planned (FR-369-AC-25/26 and TC-484 are "Planned" in tests.md:19 and 80, and QSL-2465 is in Backlog). This reader therefore cannot reach a reward on a source-declared `declaring` node at all. It is a separate, spec-owned piece of work. However, neither QSL-2465 (whose scope is delay, random_parameter and workload), FR-369-AC-25/26 nor TC-484 mentions a reward, so the deferral that FR-038-AC-571 states has no owner.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FSpec FR-640-AC-7's source half is neither implemented nor tracked: a reward `cost` on `Other::poke` whose function carries the unit's `SourceOwner` and the declaration `["Other","poke","reward","cost"]` is admitted, and the declaration `[...,"load"]` refuses rule 4 at `/body/expression`. FR-038-AC-571 defers it to "FR-369-AC-25", but that AC, TC-484 and QSL-2465 cover only delay, random_parameter and workload. Add the reward SourceOwner declaration check to QSL-2465's scope and to FR-369-AC-25/26 (or a new Planned AC) so the deferral has an owner. It is not required in this PR | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6716; spec/checked_package/matrix/tests.md:19 |

## Dispositions

Round 1, at head 0faf1c2b, with origin/main 358e9e2d merged.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The case now has an owner. #1075 (W83e, on main) adds the source reward function to FR-369-AC-25/26 and TC-484 as Planned under QSL-2465 (tests.md:19), and QSL-2465 carries the scope addition. FR-038-AC-571 now points at FR-369's source-declared `reward` rule under QSL-2465. Implementation stays out of this PR, as judged in the review pass |
