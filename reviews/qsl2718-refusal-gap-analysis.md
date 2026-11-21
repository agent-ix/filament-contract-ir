---
id: SR-2026112112042500000002
title: "gap-analysis of QSL-2718 terminal model_root binding and object_type refusal"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1115 at f4c8c2b5; FR-060-AC-2, FR-060-AC-5, TC-061, crates/filament-contract-model/src/checked_package/v2/claim_terminal.rs, tests/it/checked_package_v2_claim_terminal.rs"
review_set: subset
---

## Summary

Gap analysis scoped to the PR diff. Ticket: QSL-2718. Plan completion: not assessed. The full matrix was not run, because the brief allows only focused, name-filtered work. Each changed branch was checked against a test that fails when the branch is changed, and the deleted assertions were checked against the TC procedures they served.

## Examined scope

- FR-060-AC-2 (examined): "wrong model (a non-model node, or a `model`/`object_type` node, with and without `self`, refusing `invalid_model_binding`/`malformed-declaration` at the `model` target while the `model_root` node admits and its writer output reads back equal)". Every clause has an asserting test: `terminal_model_must_be_the_model_root_not_an_object_type` covers the `object_type` no-self and self cases and the root admit with read-back, and the existing `terminal_when_and_mode_refusals` covers the non-model case. The trace `FR-060-AC-2` on the new test is correct.
- FR-060-AC-5 (examined): IR-owned vectors through the public V2 reader and writer. The new test uses only `CheckedPackageV2::read` and the canonical writer form. The trace is correct.
- TC-061 "model identity control" (examined): "select two distinct models `M` and `N` ... Start from an admitted `any` terminal on `M` ... Change only the terminal body's `model` reference to `N` ... The model identity control admits both packages, retains distinct complete terminal bodies under structural ids and preserves each through read/write/read." See FND-001.
- Code branches (examined): the `ModelRoot` acceptance and the non-root refusal both have tests that fail. The deleted `object_type` arm has no remaining caller.

## Verdict

The ticket's acceptance, the `object_type` refusal on both paths with the root admitted, is covered by meaningful tests. One coverage regression came from the migration (FND-001).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The PR deleted the `first_model`/`second_model` block in `terminal_identity_and_several_terminals_per_model`. That block was the only realization of TC-061's model identity control: two complete packages that differ only in the terminal's `model` reference, both admitted, with distinct bodies. It now admits an invalid `object_type` target, so deleting it was right, but no replacement moves the control to two distinct selected `model_root` nodes. `terminal_rows_order_a_prefix_model_before_its_extension` admits two such terminals in one package, but it does not compare two packages and does not check read/write/read for each. | tests/it/checked_package_v2_claim_terminal.rs:1224-1367, spec/checked_package/matrix/TC-061-model-terminal-reader-writer.md:34-40 |

## Dispositions

Round 1, at d13bcfcb. The fix adds `terminal_on_selected_root` and a two-package block at the end of `terminal_identity_and_several_terminals_per_model`.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | still-open | The restored control does not yet hold the one variable fixed. `terminal_on_selected_root("test/families")` selects only `test/families`, but `terminal_on_selected_root("test/families-other")` also adds a second `model_root` node and selection. The two packages therefore differ in their selected models as well as in the terminal's `model` reference, against TC-061 ("retain both nodes and selections in each compared package"). So `assert_ne!(encoded(..))` holds from the extra node alone, and `assert_ne!` on the bodies compares hand-built inputs. The per-package check asserts only `terminal_rows().len() == 1`, not that the row's `model` is the bound root. Still missing: select both identities in both packages, and assert that each admitted row's `model` equals its own root key |
