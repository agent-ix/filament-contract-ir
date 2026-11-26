---
id: SR-967175026302197768900474181941935084782
title: "gap analysis of QSL-3359 location ordinal grouping"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1193 at 2f7abfa4; crates/filament-contract-model/src/checked_package/v2/{protocol_locations.rs,body_locations.rs}"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: references
---

## Summary

Gap analysis limited to the criteria this PR touches. Ticket: QSL-3359. quire matrix ran over the repository; only FR-038-AC-581, FR-038-AC-597 and FR-322-AC-291 were examined. No whole-repository audit.

## Verdict

CONDITIONAL on this method alone; the code-review artifact blocks the merge. The new test has hand-written expected values from the spec, and it fails on the base reader. No stub and no unspecified behaviour in the changed production code.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The added test calls validate_rows only: no test asserts the public reader or cursor admits and pairs two functions sharing (application, occurrence) at ordinals 0 and 1, and the duplicate case asserts neither code nor row locus | crates/filament-contract-model/src/checked_package/v2/body_locations.rs:436-512 |
| FND-002 | low | The added test is tagged FR-038-AC-597 (compact up/down paths, BL-04..BL-06), none of which it asserts | crates/filament-contract-model/src/checked_package/v2/body_locations.rs:438 |

## Coverage

Plan completion: not assessed. Semantic review: skipped (not requested). FR-322-AC-291 and the FR-038 body_locations shared-rule prose examined. FR-038 has no criterion row stating the cross-function key; FR-322-AC-291 owns it.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 3199f6a3 rewrites shared_k_call_plus_one (public reader and cursor, ordinals 0 and 1) and the interleaved duplicate test (code, cause, row locus) |
| FND-002 | fixed | 3199f6a3 removes the FR-038-AC-597 tag from the added test |
