---
id: SR-17940486709564847048137
title: "Spec review — TC-464 explicit-lane wording and FSL checkout note"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1105 at 57f2659d; spec/checked_package/matrix/TC-464-model-operation-census.md; Makefile FILAMENT_SPEC_LANGUAGE_DIR comment and conformance-fsl-ir760 message"
review_set: subset
---

## Summary

Ticket: QSL-2711. The PR changes one sentence of TC-464 and adds a Makefile comment and error message about the FSL checkout.

## Examined scope

- The TC-464 sentence (examined) reads: "that test is run explicitly and is not part of CI (`test = false`, no Makefile target); it must be run explicitly before its evidence is credited." It is accurate:
  - Cargo.toml declares `conformance_fsl_ir1236` with `test = false`.
  - `grep ir1236 Makefile .github` finds no target or workflow.
  - tests/conformance_fsl_ir1236/main.rs reads `IR1236_FSL_PACKAGE`/`IR1236_FSL_MODEL`, which the TC names.
- The Makefile note (examined) says FILAMENT_SPEC_LANGUAGE_DIR must be current FSL origin/main, built with its own CARGO_TARGET_DIR, with no default. This is accurate. The test runs FSL's `population_v2_fixtures` example from that checkout, so a stale checkout emits an older wire. The message is consistent with the `test -n` guard.

## Verdict

**Clean apart from one wording nit.** The doc change is accurate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The sentence says the test "is run explicitly" and then "must be run explicitly" a second time. Drop one, for example "that test is not part of CI (`test = false`, no Makefile target) and must be run explicitly before its evidence is credited." | spec/checked_package/matrix/TC-464-model-operation-census.md:29 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | edc9b65b changes the sentence to "that test is not part of CI (`test = false`, no Makefile target); it must be run explicitly before its evidence is credited." |
