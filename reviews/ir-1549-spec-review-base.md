---
id: SR-20261115234919
title: "spec-review of IR-1549 FR-369 and TC-476 (PR #1027)"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1027 at ed31ba20 (base origin/main 80e3f67f); spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md, spec/checked_package/matrix/TC-476-probabilistic-node-reader.md, spec/checked_package/matrix/tests.md, spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

This is the base spec review of the spec changes in filament-contract-ir PR #1027. Ticket: IR-1549. The reference text is FSpec FR-640 and FR-322 at filament-specification `origin/main` c143e2b5.

FR-369 and TC-476 are new ids. Neither id exists on `origin/main` 80e3f67f, and no other open PR adds either; #1027 is the only open PR. `quire validate` is clean apart from two new grammar warnings at FR-369 line 38 (FND-003).

## Examined scope

- FR-369 Description (examined). One tag and four forms, as FR-640 states. This is correct.
- FR-369 Behavior, record paragraph (examined). "no `declaration`" and "exactly one occurrence, with role `declaration`" are FR-640 text. "no `owner` and no `recursion_group`" are inferred. FR-640 calls the node a derived node of FR-322. FR-322's derived nodes carry no `declaration`, and the owner-presence rule lists no probabilistic form. The non-root `timed_model` forms follow the same rule in this reader. That inference is the strict reading: a node outside every owner-bearing form has no owner kind it could join. So refusing `owner` does not reject output FR-640 allows. The text should say where the rule comes from (FND-002).
- FR-369 Behavior, error-kind paragraph (examined). This is FND-001.
- FR-369 exclusion paragraph (examined). The gap-analysis SR reports the missing planned record.
- FR-369-AC-1 to AC-4 (examined). Each AC is testable and can fail, and each is backed by one tagged test.
- TC-476 (examined). Its procedure steps map one to one onto the ACs. Status: Implemented.
- tests.md rows for FR-369 and TC-476 (examined). Both are consistent with FR-369's own scope.
- FR-019 V2 vocabulary row (examined). `ProbabilisticForm` was added in alphabetical order.

## Verdict

Changes requested. FR-369 restates FR-640's shapes faithfully. But it assigns `malformed_wire` to defects that FR-640's Reader assigns to `invalid_package`/`invalid-value` (FND-001). The `owner` and `semantic_type` rules are sound inferences, but they need a stated source (FND-002).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-369's error split contradicts FR-640's Reader. FR-640 refuses "parameters that do not match the family" and a negative workload weight with `invalid_package`/`invalid-value` (rules 3 and 5, AC-4). FR-369 classes any form shape outside the bindings, family parameters included, as `malformed_wire`, and is silent on a negative weight. FR-369 also says a weight or rate "that is not an exact rational" refuses `invalid-value`, which the code does not do. | spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md:36-49 |
| FND-002 | low | "no `owner` and no `recursion_group`", and the reused rule that `semantic_type` is the node's own id, are not FR-640 text. They are IR's reading of FR-322's derived-node and owner-presence rules. FR-369 states them as SHALL without naming that source. A reader of FR-369 cannot tell they are inferred, or what would change them if FSpec later gives probabilistic nodes an owner. | spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md:36-37 |
| FND-003 | low | The FR-369 Behavior sentence "Its body SHALL be an aggregate of exactly the bindings ..." raises the new quire grammar warnings `ears:missing-subject` and `ears:unclassifiable` at line 38. The sentence before it, "A `probabilistic` node SHALL carry ...", has the same shape. Neither SHALL names the system that acts; for example: "the strict V2 reader SHALL refuse a `probabilistic` node whose body is not ...". | spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md:36-41 |

## Dispositions

Round 1 checks FR-369, TC-476 and tests.md at head d0c796b0. `quire validate` now reports 10 grammar findings, the same count as `origin/main`; FR-369 adds none.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | FR-369 Behavior now assigns `invalid_package`/`invalid-value` to a negative workload weight, a weight or rate not an exact rational in lowest terms, and `parameters` that do not hold exactly the members the family states. This matches FR-640 Reader rules 3 and 5. |
| FND-002 | fixed | FR-369 now cites FR-322's derived-node shape and owner-presence rule, and the non-root `timed_model` precedent. It says the `owner`, `recursion_group` and `semantic_type` rules are IR's reading of FR-322, not FR-640 text. |
| FND-003 | fixed | Every SHALL now names "The strict V2 reader", and the FR-369 grammar warnings are gone. |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | The new FR-369-AC-5 says "a budget one unit short of the total is denied at the first uncharged row". `one_visit_is_charged_per_node_and_per_member` instead denies a budget of 1, which covers the node visit only, at the first row. No test reads with the total minus one. For a total-minus-one budget, the first uncharged charge is the last row's weight, not a row. So the AC as written is neither what the test asserts nor a well-defined row. Either test total-minus-one or reword the AC to "a budget that covers the node and no row is denied at the first row". | spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md:98 |
