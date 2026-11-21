---
id: SR-967175026302197768900474181941935084521
title: "Gap analysis of QSL-2739 AC10 composition coordinates (PR #1120)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1120 at ddfb8854e; FR-371-AC-12, FR-038-AC-602..605, TC-474, TC-477; crates/filament-contract-model/src/checked_package/v2/, tests/it/checked_package_v2_guarantee_composition.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-038", type: references }
  - { target: "ix://agent-ix/filament-contract-ir/FR-371", type: references }
---

## Summary

Ticket: QSL-2739. This is a planless, repository-driven audit of the PR's spec, test and code chain. The matrix was computed by `quoin matrix` and stale tags were read from `quire coverage`. All five owned criteria are tagged, and every new code path has an owning criterion. No stubs were found. Two coverage gaps against the published FSpec contract remain. Plan completion was not assessed.

## Verdict

**CONDITIONAL**: only medium and low findings. There is no untagged owned criterion and no high finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Three FSpec FR-491 orderings have no test. Generic unknown-node and total-occurrence (occurrence without entry) defects must keep precedence over a composition coordinate defect, and a generic defect on another node must still precede one. Only an empty interval and a repeated key are combined with a coordinate defect. | FR-038-AC-605, tests/it/checked_package_v2_guarantee_composition.rs::generic_source_map_checks_precede_and_body_checks_follow_composition_coordinates |
| FND-002 | low | FR-322-AC-514's "another correspondence form remains ineligible for … a claim-name row" has no test. No test gives an `implementation_guarantee` a `claim_names` row and expects `/claim_names/{i}/declaration`. AC-603 covers only the declaration metadata. | FR-371-AC-12, FR-038-AC-603 |

## Coverage

- Reconciliation: quoin matrix (quoin 0.29.1, quire 0.36.2 engine 0.50.2)
- Plan completion: not assessed
- Owned criteria: tagged 5 · untagged 0 · tagged-by-ignored-test 0 · method-without-symbol 0. FR-038-AC-602 has 2 binders, AC-603 has 1, AC-604 has 2, AC-605 has 1, FR-371-AC-12 has 3.
- Repository-wide (context, pre-existing, not from this diff): 1311 criteria. Tagged 1208, untagged 76, tagged-by-ignored-test 1, method-without-symbol 26. `quire coverage` has 28 diagnostics and no untracked symbols in the changed test files.
- Evidence: no run evidence (the store has not been fed). The executed focused receipts are in SR-967175026302197768900474181941935084520.
- Untraced behaviors / stubs: 0. The new `composition_coordinates.rs`, `is_exactly`, `of_node`, `at` and the claim-row checks are all owned by FR-038-AC-602..605 and FR-371-AC-12.
- Bindings judged: the AC-12 binding of `composition_source_order_comes_from_declaration_regions_not_node_ids` is wrong (SR-967175026302197768900474181941935084520 FND-004). The AC-12 claim-role case does not reach its branch (SR-967175026302197768900474181941935084520 FND-003). The AC-604 boundary set lacks an end-sharing case (SR-967175026302197768900474181941935084520 FND-002).
- Semantic review: ran over FR-371-AC-12 and FR-038-AC-602..605 against FSpec FR-322 and FR-491 at 9add8a0f.

## Dispositions

Disposition pass 1 covers fix commit 009c94728 at head 009c9472805e4a80d4851ccf0d0ff2fe57f061b8. Recomputed with `quoin matrix` (quoin 0.29.1): all 16 owned criteria are tagged (FR-038-AC-602..613, FR-371-AC-12..15). Repo-wide untagged is still 76, none of them owned here.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AC-605 now combines an earlier coordinate defect with each of: an entry naming an unknown node, an entry moved to another node that has no such occurrence, and an occurrence left without an entry. The first two refuse `invalid_source_map` at the entry and the third at `/source_map`. Each passes on the head. |
| FND-002 | fixed | FR-371-AC-14: a row for an `implementation_guarantee` refuses at `/claim_names/0/declaration`, and the same package without the row admits. |
