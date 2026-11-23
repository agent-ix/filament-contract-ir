---
id: SR-17193
title: "QSL-2985 PR 1148 spec review of FR-038-AC-394"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir#1148; spec/checked_package/functional/FR-038-consume-checked-package-v2.md FR-038-AC-394"
review_set: base
---

# SR-17193: QSL-2985 PR 1148 spec review of FR-038-AC-394

## Summary

Ticket: QSL-2985. Question: is the AC stale, or has the reader regressed? The
AC is stale. FR-038-AC-144 says the selected-owner charge is "charged once, at
the final selected owner, after graph validation", and that the read's M - 1
returns `incomplete` at that final selected-owner charge. FR-038-AC-30 says
that a read with a selected domain package, one below its work, stops at the
root's `/semantic_graph/nodes/<i>/owner`. FR-038-AC-508 freezes the existing
charge order. The old AC-394 ("one below ... at its final probabilistic-claim
row") contradicted AC-144 and AC-30 for any fixture with a selected model, and
the IR-constructed PC fixture has one at node 24.

FR-038 body text (rows charged before inspection) and FSpec FR-322 at 7e9600df
(each inspected row charged before inspection; exhaustion at the first
uncharged locus, no partial view) define only the order inside the sidecar.
They do not require the sidecar to be the last stage of the whole read. The
FSpec vector `work_limit` (one-less at the final probabilistic claim row) is
likewise a sidecar-stage equation. The amended AC keeps that order: the largest
limit below the owner visits stops at claim row 1. No other FR-038 AC or body
text contradicts the new row. AC-398 (schedulability, "first uncharged row") is
a different carrier.

`quire validate --scope . spec/checked_package/functional/FR-038-consume-checked-package-v2.md`
(quire 0.52.1) exited 0. It gave four pre-existing grammar warnings, none on
the AC-394 line.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The amended AC-394 names both refusals `resource_exhausted`. The reader returns `Incomplete` with `limit_kind = Work`, and FR-038-AC-30/AC-144 call this `incomplete` for `work`. A reader of the now-exact clause could look for a refusal code that does not exist. Say `incomplete` for `work` (consumed = limit + 1). | FR-038-AC-394; FR-038-AC-30; FR-038-AC-144 |
| FND-002 | low | AC-394 is compound. It covers by-reference fixture construction, adverse-vector comparison, no-copy policy and now two distinct work-boundary outcomes. The amendment adds a fourth independent clause, so one failure cannot be localized to a criterion. Consider splitting the work boundary into its own AC. | FR-038-AC-394 |

## Verdict

PASS with two low wording findings. The amendment is correct, testable and
states exact values (both loci and the ordering relative to the sidecar).
