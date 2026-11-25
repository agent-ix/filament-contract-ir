---
id: SR-202611253381
title: "gap-analysis of QSL-3380 PR #1179 (CertificateMember zero_weight)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1179 at 1a4f0498c4; crates/filament-contract-model/src/certificate_member.rs; FR-070"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-070
    type: references
---

## Summary

Ticket: QSL-3380. Planless gap analysis limited to the diff and its owning requirement FR-070. `quire matrix --scope . --format json` (quire 0.52.1) binds FR-070-AC-1 to the renamed unit test and the conformance test, AC-2 and AC-3 to `model_certificate_members_match_fspec_schema`, and AC-4 to `members_appear_once_with_distinct_plain_spellings`. No criterion is untagged. The new variant has an owner (FR-070, depending on FSpec FR-331 and FR-412-AC-33). No stub or inflated tag.

## Verdict

PASS (repository claim only). The `unmarked-trace-id` notes on the doc comments of the AC-1 and AC-4 tests are the repo-wide doc-line pattern and not introduced by this diff. AC-3 (order) is enforced only by the conformance lane outside CI, as FR-070 and the Makefile already state.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No gaps found | - |

## Coverage

FR-070-AC-1..AC-4 examined, all tagged. Semantic review: skipped (not requested). Plan completion: not assessed
