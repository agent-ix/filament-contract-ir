---
id: SR-6454
title: "gap-analysis of IR-1614 owner operand narrowing and red-test repairs"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1173 at 414e28ad; FR-038-AC-629, FR-038-AC-374, FR-038-AC-367, FR-040-AC-18, FR-063-AC-7, crates/filament-contract-model/src/checked_package/v2/{protocol_body.rs,identity.rs}, tests/it"
review_set: subset
---

## Summary

Gap analysis scoped to the PR diff. Ticket: IR-1614. Plan completion: not assessed. The matrix was computed with `quire matrix --scope . --format json` on this head; every criterion the diff touches is `tagged`: FR-038-AC-629 (four tests, three of them new), FR-038-AC-374 (three), FR-038-AC-367, FR-040-AC-18 and FR-063-AC-7. Semantic review was not run as a separate pass; the code-review artifact covers intent for the changed tests.

## Examined scope

- FR-038-AC-629 (examined): retarget refusal, updated-path admission, declaration-only admission, longer-path refusal and the published FSpec fixture admission each have a test that fails if its branch is removed.
- FR-038-AC-374 (examined): numerator pointers for `root_scale` are tested; see FND-001 for the denominator.
- FR-038-AC-367, FR-040-AC-18, FR-063-AC-7 (examined): the changed tests assert the AC text literally.
- Reverse gap (examined): no new production behaviour without an owning criterion, except the undeclared-clause case reported in the code-review artifact.
- Stubs and inflation (examined): none in the diff.

## Verdict

CONDITIONAL. One low gap. The four tests left red (FR-038-AC-80 tag literal, three FR-019 inventory tests) are spec defects reported in the spec-review artifact, not coverage gaps.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The changed constructor check now reports a nonpositive `root_scale` denominator at `/root_scale/denominator` (FR-038-AC-374 lists a nonpositive rational denominator), but no test covers the `root_scale` denominator through `try_dimension_equivalence`; only the `scale` denominator and the `root_scale` numerator are tested | crates/filament-contract-model/src/checked_package/v2/identity.rs:617, tests/it/checked_package_v2_nominal_api.rs:447-460 |

## Coverage

- Criteria examined: 5; tagged: 5; untagged: 0.
- Plan completion: not assessed
