---
id: SR-17940486709564847048136
title: "Gap analysis — QSL-2711 red-main test bindings"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1105 at 57f2659d; tests touched by the diff and their trace tags: FR-053-AC-3/TC-453, FR-038-AC-390/392/395/TC-233, FR-376-AC-2/TC-488, FR-038-AC-484/547/TC-048, TC-977/FR-322-AC-303/FR-038-AC-406-408/FR-354-AC-11"
review_set: subset
---

## Summary

Ticket: QSL-2711. This was a manual AC-to-test check, limited to the tests the diff touches. The full `quoin matrix` gap-analysis was not run, because the PR adds no production code and no new requirement. No production code changed, so no code lacks an owning requirement.

## Examined scope

- FR-053-AC-3 (examined). The AC says: "A population binding with a second body member fails V2 admission as `invalid_package`/`invalid-derived-shape` at that node rather than producing a projection outcome." `duplicate_max_population_body_is_refused_by_the_reader` now asserts that clause. Before, it asserted the opposite (admit, then `NotPopulation`). The binding is now correct. The pointer strength is covered in SR-17940486709564847048135 FND-001.
- The FR-053-AC-3 `NotPopulation` / `StaleBinding` / `MissingOccurrence` / `ForeignPackage` clauses (context_only) are still covered by `crates/filament-contract-model/tests/selected_population_public.rs`, which references `NotPopulation`.
- FR-038-AC-390 (examined). The AC requires "exactly five required arrays ... `body_locations`". The renamed `five_array_...` test now removes each of the five. The binding is correct and stronger.
- FR-376-AC-2 (examined). It covers "each refusing mutation of FSpec FR-825-AC-4 ... and the `claim_names` row naming a `value_validity` node". The split-out declaration-member test, the ordinal-swap case and the claim_names test all keep this tag. The split-out test is red pending FSpec, which the PR reports. The binding itself is correct.
- FR-038-AC-484/547 (TC-048) monitor clock, and TC-977 operation effects (examined). The tags are unchanged and the tests pass.
- Every trace id resolves to an ID in the local spec (examined).

## Verdict

**Clean.** Every touched test is bound to the AC it asserts. No coverage was removed, and FR-038-AC-390 gained a case.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
