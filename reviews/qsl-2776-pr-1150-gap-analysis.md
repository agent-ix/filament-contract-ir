---
id: SR-967175026302197768900474181941935084593
title: "gap-analysis of QSL-2776 PR #1150 package-wide requirement occurrences"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1150 at 3ed709288ef742ed267c8727f205266f985f2341; changed requirement-record and refinement-reader paths, conformance and integration tests"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-contract-ir
    type: references
---

## Summary

Ran the engine-backed repository matrix, coverage report, and reverse-gap/stub inspection, with findings scoped to the PR's changed behavior. The matrix has 1,213 tagged criteria, 116 untagged criteria, 1 criterion tagged only by an ignored test, 26 methods without a symbol, and 8 retired criteria; FR-376-AC-4 is the directly relevant untagged criterion. No stubs or newly unspecified behavior were found in the changed files. Plan completion: not assessed.

Ticket: QSL-2776. Matrix: `quoin matrix --repo /home/peter/dev/worktrees/review-qsl2776 --json`; coverage: `quire coverage --scope /home/peter/dev/worktrees/review-qsl2776 --json`. The coverage command also reports pre-existing duplicate-symbol diagnostics; this pass did not attribute those baseline diagnostics to the PR.

## Verdict

FAIL: FR-376-AC-4 has no trace-tagged test, and the PR adds no test for its 1,000-record / 1,000-frame bounded-storage case.

## Examined scope

- FR-376-AC-1 through FR-376-AC-4 (examined): requirement-row admission/refusal, reader order and charging, and the linear-storage stress case.
- Changed `requirement_records.rs` behavior (examined): package-wide numbering, path conditions, one owner traversal, and dynamic index allocation.
- Changed FSpec conformance and integration tests (examined): new cases exercise package-wide numbering, empty and missing branches, shared nodes, and history-target validation. The matrix shows the FSpec criteria quoted by the new conformance tests are untracked in this repository; local FR-376-AC-1 through AC-3 have other tags.
- Changed source and test files (examined): no stubs, placeholder implementations, or new behavior without an owning local requirement found.
- Whole-repository computed matrix (context_only): 1,213 tagged, 116 untagged, 1 ignored-only, 26 method-without-symbol, and 8 retired criteria; no run evidence is present in the evidence store.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-376-AC-4 is untagged and the PR adds no test measuring storage for 1,000 value-validity records under 1,000 shared frames, leaving the linear-storage bound unverified. | FR-376-AC-4 |

## Coverage

The computed matrix is engine-backed. This review reports the direct gap for FR-376-AC-4; the matrix also reports 115 other untagged criteria outside the direct PR criterion, one ignored-only criterion, 26 methods without a symbol, and 8 retired criteria. The changed behavior has local ownership in FR-376 and FR-061. No changed-file source/test stubs or reverse-spec gaps were identified. Semantic review was skipped because it was not explicitly requested. Plan completion: not assessed.
