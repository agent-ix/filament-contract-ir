---
id: SR-17940486709564847048106
title: "spec-review of QSL-2653 FR-038-AC-511 matrix row sentence"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1089 at 1dc26097b (git diff origin/main...HEAD); spec/checked_package/matrix/tests.md (FR-038-AC-511 row)"
review_set: subset
---

## Summary

Ticket: QSL-2653. The only spec change is one sentence added to the FR-038-AC-511 row of the TC-472 matrix. It names `tests/it/checked_package_v2_whole_read_accounting.rs`, a source-declared and a model-owner workload of 20, 300 and 600 rows (the first with a `discrete` delay and a random parameter), the equal-unpaid-within-the-bound assertion, and that the registry holds both over 256 rows.

Each claim was checked against the code. The fixtures are `accounting_fixture_of(rows)` in both modules, and `wide(520, rows)` keeps `delay_poke`/`random_k`. The assertions are `<= proof::UNPAID_BOUND` and `windows(2)` equality. The registry rows are 257 and 300. The new test does not need FSpec, so the existing "those four tests fail" count still holds. The row status stays 🚧. `quire validate` on the file reports registry warnings only, and FR-038-AC-511's text is unchanged.

## Verdict

**Clean.** The sentence is accurate and atomic, and it does not overclaim. Examined: spec/checked_package/matrix/tests.md FR-038-AC-511 row (examined), FR-038-AC-511 (context_only).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
