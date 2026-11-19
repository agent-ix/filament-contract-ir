---
id: SR-17940486709564847048041
title: "QSL-2360 code and Rust review of contract-ir reader tests"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR #1063 @ 95961535635b814db2c8b6ae2871bd2ae5b16b21; tests/it/checked_package_v2_catalogue_differential.rs; tests/it/checked_package_v2_refinement.rs; QSL-2360 acceptance"
review_set: subset
---

## Summary

Reviewed PR #1063's test-only changes against QSL-2360 and the relevant reader requirements. The three focused filters pass; the tests assert admission, writer/readback graph equality, and operand-specific typed refusal for all three catalog rows.

## Verdict

**PASS** — no code, Rust-style, oracle-strength, coverage, or traceability defect was found in the reviewed changes. The hosted aggregate Rust check remains red on six unrelated unpaid-memory tests already present in this area; it does not invalidate these focused results.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- Scope: reviewed both changed integration-test files, QSL-2360 acceptance, FR-038-AC-251, and FR-061-AC-5.
- `collection.only`: test admits the catalog fixture, writes and reads it back with graph equality, then substitutes a scalar literal for the binder operand and asserts `IllTyped` / `OperatorIneligible`, the operand JSON pointer, and application locus.
- `collection.seq`: existing bounded-input controls remain; the new writer/readback assertion covers its admitted fixture, and a scalar literal at the binder argument is refused with the expected typed operand location.
- `refinement.history`: a valid joined-history fixture writes and reads back with graph equality; replacing its reference operand with an integer literal produces the expected typed refusal at the application.
- Trace tags: the new `collection.only` test binds TC-048 and FR-038-AC-251; the history test binds TC-048, FR-038-AC-251, and FR-061-AC-5. The sequence test's existing trace remains attached to the expanded public-reader test.
- Targeted checks: all three requested `cargo test --locked -p filament-contract-ir --test it --features conformance-cli <filter>` commands passed locally (1/1 each).
- No production or catalog files changed. No applicable AssuranceProfile was found.
- The Rust Checks aggregate on PR #1063 fails six unpaid-memory tests in unrelated test modules. CLA and Supply-chain Checks pass. The aggregate failure is not caused by this diff.
