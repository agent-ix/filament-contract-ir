---
id: SR-2734100002
title: "Gap analysis of QSL-2734 dimension-equivalence borrowed view (PR 1116)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1116 at 844467f5; FR-038-AC-601, TC-048 section, TC-058, FR-019 nominal-body and Exact-projection rows"
review_set: subset
---

## Summary

A diff-scoped repository audit for PR #1116. Ticket: QSL-2734. Planless.

- Matrix: `quire trace --id FR-038-AC-601` resolves three structural claims. They are the local view, package view and wrong-kind tests in tests/it/checked_package_v2_dimension_equivalence.rs. The AC is not untagged.
- Reverse gap: the one new public method is owned by FR-038-AC-601 and the FR-019 nominal-body row.
- Stubs: none. The production method is a real match. The tests assert concrete values, and a mutant shows they can fail.
- Spec files: `quire validate --scope .` on the three changed spec files exits 0. Its 4 warnings are on pre-existing lines.

## Verdict

CONDITIONAL, with low trace findings only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-038-AC-601 cites TC-058, and the three new tests carry a TC-058 tag, but TC-058 has no section for this accessor. Compare the enum-details section, which states the external-consumer read and compile-fail probes that reject mutation or unchecked construction. So the AC's "no mutable output or unchecked constructor" clause has no TC-058 test-case text and no oracle. | spec/model/matrix/TC-058-model-crate-public-interface.md:112-125; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6421 |
| FND-002 | low | TC-048 says "The view round-trips through the canonical writer." The tests tagged FR-038-AC-601 only serde-serialize the borrowed operands and compare them with the input wire. None calls `encode_checked_package_v2`. The writer round-trip is in `selected_dimension_operand_admits_and_retains_package_form`, which is tagged TC-280/FR-038-AC-387 and not FR-038-AC-601. | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:2799-2800; tests/it/checked_package_v2_dimension_equivalence.rs:631-636,139-185 |

## Coverage

FR-038-AC-601: 3 tagged tests, structural claims, and the exact accessor tests rerun at the frozen head (2 passed). FR-019 inventory: tc_058 passed (coder log). Plan completion: not assessed. Semantic review: done inline by the reviewer per the dispatch brief (oracle strength); no subagent fan-out.
