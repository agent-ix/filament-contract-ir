---
id: SR-17940486709564847048091
title: "spec-review of QSL-2560 matrix rows for FR-060-AC-6/7/8"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1076 at 8e69e047 (merge base c7b62038); spec/checked_package/matrix/tests.md (FR-060 rows, TC-487 row)"
review_set: subset
---

## Summary

Spec review of the only spec change in PR #1076: three status cells in `spec/checked_package/matrix/tests.md` flip from Planned to Implemented. Ticket: QSL-2560. The cells cover FR-060-AC-6/AC-8 (TC-487) and FR-060-AC-7 (TC-061). The requirement text (FR-060, FR-062, TC-063, TC-487, FR-019) was merged with W80 (#1070) and is unchanged here. It was checked against FSpec origin/main FR-476 as context. `quire validate` on tests.md reports no errors, only module-registry warnings that also appear on main.

## Examined scope

- tests.md row FR-060 | FR-060-AC-6, FR-060-AC-8 | TC-487 (examined). It names QSL-2521, QSL-2558 and QSL-2560, and the tests are tagged. The AC-8 MT-19 clause is untested; that gap is recorded once, as gap-analysis FND-001.
- tests.md row FR-060 | FR-060-AC-7 | TC-061 (examined). Backed by `terminal_admits_a_boolean_literal_condition`, and the literal-admission mutant kills it.
- tests.md row TC-487 (examined). Its status is consistent with the FR row.
- FR-060 "Public terminal rows" and Behavior (context_only). They agree with FSpec FR-476 rules 4/5, terminal row order and the ordinal.
- FR-062 and TC-063, Withdrawn (context_only). They agree with the code removal.

## Verdict

The matrix edits are consistent with the requirement text and the tagged tests. The only gap behind the Implemented status is recorded in the gap analysis, so it is not repeated here.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
