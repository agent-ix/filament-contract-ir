---
id: SR-17940486709564847048070
title: "spec-review of QSL-2410 FR-371 claim declaration names"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1046 at 11029d36; spec/checked_package/functional/FR-371-read-claim-declaration-names.md, spec/checked_package/matrix/TC-477-claim-declaration-names-reader.md, spec/checked_package/matrix/tests.md, spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

Spec review of the new FR-371, TC-477, the matrix rows and the FR-019 public-items row in PR #1046. Ticket: QSL-2410. Upstream source, read only: filament-specification origin/main f16e1701. The sections read were FR-322 "Claim declaration names", "One claim namespace" and the `claim_names` member bullet, FR-322-AC-200, FR-322-AC-457 to AC-459, and `schema.json` `ClaimNameRow`/`RefinementBody`. `make spec` gives the same 68 unbacked rows as clean main; FR-371 is 6/6 backed. FR-371 and TC-477 collide with no id in this repository or in FSpec.

## Examined scope

- FR-371 Description, Inputs, Outputs (examined): the reader and writer allocation is stated. The FR-322, FR-322-AC-457 and FR-322-AC-458 references are present.
- FR-371 Behavior, closed-schema paragraph (examined): matches the FR-322 closed-schema sentence and pointers.
- FR-371 Behavior, rules 1-4 (examined): match FR-322 rules 1-4, including the AC-200 refinement clause.
- FR-371 Behavior, rule 5 (examined): see FND-002.
- FR-371 Behavior, coverage and metering paragraph (examined): see FND-003.
- FR-371 refinement-body paragraph (examined): see FND-004.
- FR-371-AC-1..6 (examined): each is a direct verification statement with code, cause and pointer. AC-1 names the AC-457 example (see FND-004 of the gap-analysis SR).
- TC-477 (examined): its procedure is mapped to ACs. Steps 4 and the locus expectation are not implemented (gap-analysis FND-001, FND-003).
- FR-019 public-items row (examined): it lists `ClaimNameRow`, the borrowed read, the lookup and the encoder input. It omits that `CheckedPackageSidecarsRef` and `CheckedTerm::Refinement` gained fields (code-review FND-005).
- FSpec FR-322-AC-459 (examined, context: same section): see FND-001.

## Verdict

FR-371 is a faithful consumer of FR-322-AC-457/458, except for three things:

- It omits the one-namespace rule (AC-459), which merged before the PR's commit.
- It drops the "naming both rows" clause.
- It states the refinement body `name` as optional, against FSpec's required member, without recording the departure or an owner.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-371 omits FSpec FR-322-AC-459 ("One claim namespace"). Once both members pass, a `claim_names` row whose `name` equals a `temporal_clause_names` row's `name` refuses `ambiguous_declaration`/`ambiguous-name` at `/claim_names/{i}/name`, naming both rows, one visit per row. FR-371 has no rule, AC or relationship for it, and does not record it as deferred with an owner | spec/checked_package/functional/FR-371-read-claim-declaration-names.md:1-98 |
| FND-002 | medium | FR-371 rule 5 drops FR-322's "naming both rows" from the duplicate-name refusal, so the spec now matches code that names only the later row. It narrows the upstream requirement without saying so | spec/checked_package/functional/FR-371-read-claim-declaration-names.md:75-77 |
| FND-003 | low | FR-371 says only "Each visited row consumes one validation visit". The code also charges one visit per occurrence scanned in the row join and per occurrence inspected by the coverage walk, and FR-371-AC-6's tests assert those exact deltas. The spec understates the metering its tests pin | spec/checked_package/functional/FR-371-read-claim-declaration-names.md:79-83,98 |
| FND-004 | medium | FR-371 says "The reader SHALL admit the member `name` of a refinement body", which makes `name` optional. FSpec `RefinementBody` requires `name`, and FR-322-AC-200 refuses a body without it at the schema and refuses equal refinement names `ambiguous-name` at `body/name`. FR-371 neither requires the member nor records this repository's departure from AC-200 with an owner and follow-up | spec/checked_package/functional/FR-371-read-claim-declaration-names.md:85-86 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | FR-371-AC-6 is garbled. It reads "...and the one-namespace check one per row: the declaration-region check one per source-map region compared: the smallest work limit...", so the region charge reads as an explanation of the namespace charge. It should be one list of five charges, followed by the measured consequence | spec/checked_package/functional/FR-371-read-claim-declaration-names.md:153 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-371 now states the one-namespace rule with its pointer, the document position naming the first temporal row, and the per-row charge. FR-371-AC-7 and an `ix://` reference to FSpec FR-322-AC-459 were added. |
| FND-002 | fixed | Rule 5 now says the refusal names the earlier row's `/claim_names/{j}/qualified_name` as its package-document position, "so both rows are named". |
| FND-003 | fixed | The metering paragraph now lists every charge: per row in the schema check, per occurrence in the join, per occurrence in the coverage walk, per source-map region in the region check, and per row in the namespace check. |
| FND-004 | fixed | FR-371 now says the refinement body `name` is a required text identifier (FR-322-AC-200). A missing or non-text name refuses at the node body, and equal names refuse `ambiguous-name` at the later node's `body/name` naming the earlier one. FR-371-AC-8 was added. |
| FND-005 | fixed | FR-371-AC-6 now lists the five charges as one sentence (per row for the schema check, per occurrence for the join, per occurrence for the coverage walk, per row for the namespace check, per region for the region check), followed by the measured consequence. FR-371-AC-9 now says a missing `declaration_region` refuses at the row, matching FSpec #708. |
