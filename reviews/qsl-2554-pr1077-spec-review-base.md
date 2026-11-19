---
id: SR-17940486709564847048094
title: "spec-review of QSL-2554 FR-377 grammar paragraph, FR-019 row and matrix row"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1077 at e0e47c8a (merge base 383857b6); spec/checked_package/functional/FR-377-admit-the-probabilistic-claim-node-and-what-it-states.md, spec/model/functional/FR-019-rust-library-interface.md, spec/checked_package/matrix/tests.md (FR-377 and TC-489 rows); FSpec origin/main FR-640 rules 6/7 and FR-322 Body grammar as context"
review_set: subset
---

## Summary

Spec review of the spec changes in PR #1077. Ticket: QSL-2554. The changes are:

- a new FR-377 section, "Body grammar of the claim node";
- a new FR-019 row for the probabilistic claim read;
- the FR-377/TC-489 matrix rows, moved from Planned to Partial.

The upstream text is FSpec origin/main FR-640 ("Probabilistic claim node", Reader rules 6 and 7) and FR-322 "Body grammar". That grammar has no row for the claim body; arch has been asked. That gap is recorded here as a finding and does not block. The ambiguity in FR-640's refusal locus is recorded here too and has been sent to arch. It does not fail the PR.

## Examined scope

- FR-377 "Body grammar of the claim node" (examined).
  - The grammar is consistent with FR-640's bindings. The claim body is a Leaf, an `aggregate` of such terms, or a `binding` of one, and an `application` is refused. That covers every FR-640 shape: references, literals, empty aggregates, and the aggregates of bindings in `scheduler`, `form`, `event`, `measure`, `target`, `bound`, `weighting`, `activation` and `confidence`.
  - FR-640 needs no other term kind, and the paragraph leaves the nine-binding shape to rules 6 and 7.
  - The depth claim is wrong (FND-001).
- FR-377 "Public claim read" (examined). The code block is stale (FND-002).
- FR-019 new row (examined). It lists all 13 new public types backticked. It names the method `probabilistic_claims` without backticks, as the `random_parameters, rewards, workloads and delay_distributions` row does for its methods. That form is legitimate. The inventory parser reads backticked words as crate-root items, and methods are not items, so no item is hidden.
- matrix/tests.md FR-377 and TC-489 rows (examined). Their honesty is recorded once, as gap-analysis FND-003.
- FSpec FR-640 rule 6 locus "at the failing member" (context_only). See FND-003.
- FSpec FR-322 "Body grammar" (context_only). See FND-004.

## Verdict

The grammar text the coder wrote into FR-377 is consistent with FR-640's bindings, and the FR-019 row is complete. Two low defects are in this PR's spec text: the depth figure and the stale type name in the read block. Two low upstream items are recorded for arch and do not block the PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The grammar paragraph says the deepest member the reader names is a `from` predicate at seventeen JSON Pointer steps, "and the reader's pointer builder holds twenty". The deepest named locus is actually nineteen steps: the `kind` of a `weighting` inside a `fraction` measure of a `compare` event, where rule 7's time check refuses a `delay` weighting. State 19, so the stated margin is true | spec/checked_package/functional/FR-377-admit-the-probabilistic-claim-node-and-what-it-states.md:111-115 |
| FND-002 | low | The "Public claim read" block declares `pub form: ProbabilisticForm` and `pub enum ProbabilisticForm { Probability … }`. That name is taken by the existing node-form enum, and the code and FR-019 export `ProbabilisticClaimForm`. The PR edits FR-377 but leaves this block contradicting the shipped API | spec/checked_package/functional/FR-377-admit-the-probabilistic-claim-node-and-what-it-states.md:131-144 |
| FND-003 | low | Upstream (FSpec FR-640 rule 6 and FR-640-AC-9): "`invalid-value` at the failing member" and "at `subject`" / "at `scheduler`" do not say whether the pointer is the binding's `value` or the reference's `target`. The PR refuses set and shape failures at `…/members/N/value` and reference failures at `…/value/target`. Two readers could differ, so the conformance vectors cannot be exact. Sent to arch; not a PR defect | spec/checked_package/functional/FR-377-admit-the-probabilistic-claim-node-and-what-it-states.md:49-60 |
| FND-004 | low | Upstream (FSpec FR-322 "Body grammar"): its five strata have no production for the `claim`/`probabilistic_claim` body, whose bindings nest aggregates of bindings below the Tuple stratum. The IR spec now carries its own grammar for it, in FR-377. FSpec should own that row, so that other readers agree. Arch has been asked; not a PR defect | spec/checked_package/functional/FR-377-admit-the-probabilistic-claim-node-and-what-it-states.md:100-115 |
