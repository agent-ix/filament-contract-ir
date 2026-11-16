---
id: SR-20261116051303
title: "spec-review base of IR-1550 FR-369 join rules (PR #1029)"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1029 at 966952c3 (base origin/main 9155a127); spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md, spec/checked_package/matrix/TC-476-probabilistic-node-reader.md, spec/checked_package/matrix/tests.md, spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

This is the base spec review of the FR-369, TC-476, tests.md and FR-019 changes in filament-contract-ir PR #1029. Ticket: IR-1550. The rules are compared with FSpec FR-640 (Reader 1-5, Typed read), FR-405 (Rewards) and FR-322 (the type rows for `Quantity<u>`, `Duration` and `Int[lo, hi]`) at filament-specification `origin/main` 959a1982.

`quire validate` exits 0.

## Examined scope

- The FR-369 Behavior join paragraph (examined). The operation reference, `random_parameter`, `workload`, `reward` and `delay_distribution` clauses are read against FR-640 Reader 1-5.
- The FR-369 `conflicting-definition` paragraph (examined). It matches the four FR-640 conflict rules, refusing at the later node.
- The FR-369 typed-read paragraph (examined). It matches FR-640 "Typed read": by operation for random parameters and rewards, by model for workloads and delays, keyed by node, with probabilities in lowest terms.
- The FR-369 Planned paragraph (examined). See FND-002.
- FR-369-AC-6 to AC-12 (examined). Each one is a testable Test-method criterion.
- The TC-476 steps 6-12 and the status (examined). They are consistent with the ACs.
- The tests.md FR-369 and TC-476 rows (examined). They are consistent.
- The FR-019 probabilistic typed-read row (examined). It matches the `lib.rs` re-exports.

## Verdict

Changes requested. FR-369 restates FR-640's reward value type more narrowly than FR-640 and FR-322 allow (FND-001). The Planned paragraph does not list every rule that is still unenforced, and misdescribes one of them (FND-002). The forbidden-read set of Rule 4 is a reasonable, minimal reading that FR-369 documents. It is not an unauthorized invention, but it should cite its FR-405 source (FND-003). The join's budget is not stated (FND-004).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-369 requires a reward `value_type` to be "an `integer`, a `carried_quantity` [or] a `bounded_quantity` type node". FR-640 says "integer or quantity type". FR-322 encodes `Quantity<u>` and `Duration` as the `scalar_type`/`unit` node and `Int[lo, hi]` as `bounded_domain`/`integer_range`. So FR-369 refuses rewards FR-640 admits, including FR-640-AC-1's `duration: Millis`. | spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md:89-94 |
| FND-002 | medium | The Planned paragraph is incomplete and partly inaccurate. It says "only the literal kind is checked", but for a declared parameter type the reader assumes kind `enum`. It omits the unit of a `Quantity<u>` support value. It says a supertype model's inherited operations go unjudged, when its own operations go unjudged too. It does not say that only direct supertypes are recognised. | spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md:112-120 |
| FND-003 | low | FR-640 does not define "reads state". FR-369's set (a `state` node, or a `pre_read`, `presence_read`, `value_read`, `deref`, `query` or `reachability` expression reached through functions, expressions and values) is a documented, minimal IR reading. It cites no source, and it is a deny-list where FR-405 gives a positive rule: the reward "reads only the operation's parameters". | spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md:89-94 |
| FND-004 | low | FR-369 states one validation visit per node and per member, but says nothing about the join stage's own charges: a re-visit per node and per workload row, the reward walk, and the scan for a timed root. The join's total budget therefore has no stated formula and no AC. | spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md:66-69,74-110 |
