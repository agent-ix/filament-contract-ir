---
id: SR-17940486709564847048115
title: "gap-analysis of QSL-2726 PR #1100 (read_storage_alloc dependencies_10000 tuple fixture)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1100 at 7a091b60; tests/it/support/read_storage_inputs.rs, tests/read_storage_alloc.rs (context)"
review_set: subset
---

## Summary

Ticket: QSL-2726. Plan completion: not assessed. This is a focused manual gap check, limited to the diff. No production code changes. Only the adverse input feeding `whole_read_node_with_10000_dependencies` changes. This record checks that the AC this test backs keeps the same coverage.

## Examined scope

- FR-038-AC-511 (examined), via matrix row `spec/checked_package/matrix/tests.md:45` and TC-472. The row lists "node dependencies" among the adverse shapes `tests/read_storage_alloc.rs` builds and bounds. The test is still tagged `#[trace("TC-472", "FR-038-AC-511")]`. It still admits, and it still asserts the unpaid-heap bound. Before this fix the test refused (`IllTyped`/`TypeMismatch`), so the AC row was red on main. The fix restores that coverage and does not inflate it.
- TC-472 (context_only), `spec/checked_package/matrix/TC-472-strict-v2-read-exact-budget-charging.md`. It requires the adverse shapes to come from the repository's public fixture builders, and they still do.
- Stub and tautology check (examined). The fixture still builds a real 10000-dependency node through `flat_value()`, and the test asserts against the real public read. No assertion was removed or weakened.
- Code without an owning requirement (examined). There is no new production code.

## Verdict

No gaps. Coverage of FR-038-AC-511's node-dependencies shape is restored, and nothing is inflated.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
