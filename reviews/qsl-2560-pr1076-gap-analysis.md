---
id: SR-17940486709564847048090
title: "gap-analysis of QSL-2560 terminal literal condition and terminal rows"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1076 at 8e69e047 (merge base c7b62038); FR-060-AC-6, FR-060-AC-7, FR-060-AC-8, FR-062-AC-1..4, TC-061, TC-063, TC-487 against crates/filament-contract-model/src/checked_package/v2/claim_terminal.rs and tests/it/checked_package_v2_claim_terminal.rs"
review_set: subset
---

## Summary

Gap analysis for PR #1076. Ticket: QSL-2560. Planless: plan completion was not assessed. Upstream: FSpec origin/main FR-476 (rules 4/5, AC-5/6).

`quire matrix` at the PR head shows FR-060-AC-6, AC-7 and AC-8 as tagged:

- AC-6: `terminal_rows_retain_structural_order_and_round_trip`.
- AC-7: `terminal_admits_a_boolean_literal_condition`.
- AC-8: `terminal_rows_order_by_model_key_before_terminal_key`, `terminal_identity_and_several_terminals_per_model` and `three_terminal_nodes_on_one_model_get_ordinals_in_key_order`.

All of these bindings are correct. FR-062-AC-1/2/4 are untagged, which is correct for Withdrawn. FR-062-AC-3 is still tagged; see code-review FND-001.

## Examined scope

- FR-060-AC-6 (examined). Covered: rows are in model-key order, opposite to graph order, with the key order asserted first. Ordinal 0 on each model. No row for K. Writer round-trip and key-consistent graph order are checked. MT-03 is `When { parameter: None }`. The empty slice is checked. A second terminal on M gives M two rows with ordinals 0 and 1.
- FR-060-AC-7 (examined). Covered: `true` and `false` literal conditions are admitted as `When { parameter: None, condition: literal }`, and the writer's re-read rows are equal. A `self` parameter condition refuses `ill_typed`/`operator-ineligible` at the target. An integer literal `3` refuses `ill_typed`/`non-boolean-root` at the target. Removing the literal admission kills the test.
- FR-060-AC-8 (examined). MT-18 is covered: when and any on one model, ordinals 0 and 1 in key order, both graph orders equal, written rows equal. MT-16 is covered: two and then three claim occurrences give one row with ordinal 0, and the written source_map and rows are equal. Three terminals give ordinals 0, 1 and 2. Model-before-terminal ordering is covered with `aa/` against the family key. The MT-19 clause is not covered (FND-001).
- FR-062-AC-1..4 and TC-063 (context_only). Withdrawn; no tests expected.

## Verdict

AC-6 and AC-7 are fully covered and AC-8 nearly so. One AC-8 clause, the MT-19 prefix-ordered identities, has no test, although the matrix row now reads Implemented.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-060-AC-8's "Over MT-19, rows on the `test/vault` model come before rows on the `test/vault-other` model" and TC-487 step 7 have no test. The only model-ordering test uses `aa/before-selected-model` against the family key, which differ in their first byte. The proper-prefix case that MT-19 exists to pin is never exercised. Add MT-19 (terminal on `N` selecting `test/vault-other` listed before one on `V` selecting `test/vault`, expected rows `test/vault` then `test/vault-other`, each ordinal 0) | tests/it/checked_package_v2_claim_terminal.rs:776-799; spec/checked_package/matrix/TC-487-terminal-rows-read-from-an-admitted-package.md:45 |
| FND-002 | low | FR-060-AC-7 says the literal terminal is "written and read back unchanged", but the test compares only `terminal_rows()` of the re-read. It does not compare the re-read graph or node (body, dependencies, occurrences) with the admitted one. Add `assert_eq!(reread.graph(), read.graph())` or an equivalent | tests/it/checked_package_v2_claim_terminal.rs:1756 |
