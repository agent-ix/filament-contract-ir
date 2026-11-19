---
id: SR-17940486709564847048089
title: "QSL-2543 PR 1078 code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir#1078@d500ef77026001048f60c27f361414aabadf5afa vs 383857b64ac16dd172d86afcfb8369db339349cd; crates/filament-contract-model/src/checked_package/v2/{requirement_records.rs,mod.rs,intake.rs,claim_names.rs,identity.rs,lower.rs,operations.rs,structural.rs,vocabulary.rs}; crates/filament-contract-model/src/lib.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-376
    type: references
---
# SR-17940486709564847048089: QSL-2543 PR 1078 code review

## Summary

Ticket: QSL-2543. Reviewed frozen PR head `d500ef77026001048f60c27f361414aabadf5afa` against parent `383857b64ac16dd172d86afcfb8369db339349cd`. The record validation and typed read path are in scope. The supplied post-rebase `requirement_records::tests::` run passed 3/3, but those tests cover wire vocabulary and encoding only; they do not exercise package admission or the public record reads.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new tests never exercise the public reader for requirement records. The three module tests cover enum decoding and row encoding, leaving FR-376-AC-1 through AC-3's admission, refusal pointers, ordering, budgets, body reach, and typed reads without a regression test. | crates/filament-contract-model/src/checked_package/v2/requirement_records.rs:1178-1294 |

## Verdict

**FAIL WITH FINDINGS.** The implementation adds the reader and public accessors, but its core behavior has no test exercising the admission path. The available 3/3 focused run does not cover that behavior. No aggregate gate was run, per review brief.

Examined units:

- FR-376-AC-1: "The reader admits FSpec TC-2053's packages of FR-825-AC-1 to FR-825-AC-3" and exposes typed rows/records, package order, writer round-trip, graph reordering, no-record, and claim_names cases.
- FR-376-AC-2: "Each refusing mutation of FSpec FR-825-AC-4 and FR-825-AC-5, and each refusing package of FR-322-AC-473" returns exact code, cause and pointer through the public reader, without partial admission.
- FR-376-AC-3: "The requirement-record step runs after probabilistic_join::validate and before checked_term_data validation" and checks precedence, exact budget, one-short budget and single owner reach walk.
- FSpec FR-825-AC-1 through AC-5: examined as the record shape, reader rule, malformed-input and budget details behind the three FR-376 acceptance criteria.
- Changed Rust paths listed in `scope`; `mod.rs` admission order and public reads, plus the new module validation, retained rows and tests.

Reviewed fix head `fd9a48164b203bff94e5010546cb6f02c01f0a6f`. The new `tests/conformance_fspec/requirement_records.rs` exercises the public reader, and the computed matrix binds FR-376-AC-1 through AC-3. The focused suite is still pending, so execution is not verified green.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | TC-488 coverage remains partial: it does not exercise the specified probabilistic/requirement-record stage-precedence cases or all FR-825 refusal mutations. Its budget test derives the expected work from the same implementation's ample-budget run, so it does not independently assert the specified charge count or that the owner's body reach is charged once. | tests/conformance_fspec/requirement_records.rs:439-477, 587-754 |

## Dispositions

Round 1 outcomes:

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Public-reader/admission regression tests were added and are trace-bound to FR-376. Their focused run remains pending. |
| FND-002 | still-open: required acceptance cases remain untested | The suite omits the specified precedence cases and several refusal mutations. Its budget test derives the expected work from the same implementation's ample-budget run, so it does not independently assert the required count or one body walk per owner. The focused test run is also still pending. |
