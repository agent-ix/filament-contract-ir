---
id: SR-967175026302197768900474181941935084594
title: "QSL-2497 PR 1169 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR #1169 at 0c5ab9a57279c7c52b83f8ddee6a7569482d32c6; crates/filament-contract-model/src/checked_package/v2/temporal.rs, tests/it/checked_package_v2_temporal.rs"
review_set: subset
---

## Summary

Ticket: QSL-2497. Code review with the rust-review lane of PR #1169 (head
0c5ab9a57279c7c52b83f8ddee6a7569482d32c6) against FSpec FR-402 "Reader order"
step 1 (Shape), FR-402-AC-10, TC-346 HI-18, FSpec FR-322 "Body steps" and
"Temporal clock bindings", and FR-038-AC-630 / FR-038-AC-561.

## Verdict

**PASS with one low finding.** The refusal matches FR-402-AC-10 and step 1:
code `ambiguous_declaration`, cause `ambiguous-name`, pointer
`/semantic_graph/nodes/{n}/body/members/3/value/members/{g}/members/0` at the
later Group, locus the hyperproperty node key, both domains, before the
Subjects step (alias test). The binding-before-hyperproperty precedence is the
only reading consistent with all of FSpec FR-322: line 5165 puts the
clock-binding checks before the FR-370 clause-body joins, FR-322-AC-294 puts
row coverage before the per-node joins, and "Body steps" runs FR-370 before
FR-402. The "follow it" clause and FR-322-AC-464's "then the rows" also allow
another grammatical reading, but that reading contradicts "the reader runs the
whole binding check before that step". TC-346 HI-18 gives the same result under
both readings. The changed tests are not weaker. The fairness test's second
Group `b` removes a repeat the spec now forbids and keeps the admitted oracle,
and the repeated-key test's one-row half now asserts the exact refusal where it
used to assert only "not a missing row". The FSpec "naming the variable" clause
has no IR carrier. The refusal struct has no name field, and FR-038-AC-630
leaves the clause out, consistently. The sorted `Vec` insert is O(n^2), but the
1 MiB byte limit bounds the Group count to a few thousand, so it is not a
resource defect.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The repeat check runs inside the per-Group loop, so Groups `s`, `s` and a third Group whose layout is broken (or whose `quantifier` is `some`) refuse `ambiguous-name` at Group 1. Running the step-1 layout sentence over every Group first would refuse `invalid_semantic_graph` at Group 2. Neither FSpec FR-402 step 1 nor FR-038-AC-630 fixes this order inside Shape, and no test pins it. The per-Group interleave matches the existing `invalid-value` handling, so pin it in FR-038-AC-630 with one test, or ask FSpec to state the order. | crates/filament-contract-model/src/checked_package/v2/temporal.rs:2893-2946 |

## Coverage

- FSpec FR-402 Reader order step 1 (examined): "Two Groups whose `variable` texts are equal (Groups `s` and `s`) refuse `ambiguous_declaration`/`ambiguous-name` at the later Group's `variable` binding, `B/members/3/value/members/{g}/members/0` with `g` that Group's index, naming the variable."
- FSpec FR-402-AC-10 (examined): behaviours `s`,`s` and finite `t`,`t` refuse at Group 1's `variable` before the Subjects step; `s`,`n` admitted.
- FSpec FR-322 Temporal clock bindings checks (1)-(4) and line 5165, FR-322-AC-294, FR-322-AC-464 (examined): precedence of the binding step over the hyperproperty step.
- FR-038-AC-630 (examined); FR-038-AC-561 (examined).
- Tests examined: tc_483_a_repeated_quantifier_variable_is_a_repeated_key_not_a_missing_row, tc_483_a_repeated_behaviours_variable_refuses_at_the_later_group, tc_483_a_repeated_variable_refuses_before_the_subjects_step, tc_483_a_repeated_finite_domain_variable_refuses_at_the_later_group, tc_346_a_fairness_reference_in_a_quantifier_group_is_admitted.
- Plan completion: not assessed.
