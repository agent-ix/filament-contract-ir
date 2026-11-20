---
id: SR-17940486709564847048084
title: "gap-analysis of QSL-2506 catalog-free T::op class join against FR-061 and FSpec FR-375"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1072 at a1cca1ba (merge base 8a24e2fe); FR-061-AC-14, FR-061-AC-15, FR-061-AC-16, TC-062, FSpec FR-375-AC-31; tests/it/checked_package_v2_refinement.rs"
review_set: subset
---

## Summary

Gap analysis of filament-contract-ir PR #1072. Ticket: QSL-2506. Plan completion: not assessed. The FR-061 text this PR implements landed on main in #1073 (a89ad36a) and is in the merge base. The arch ruling relayed by the code lead is: only `T::op`; refusal order is not-`T::op`, then missing-name at `source_class`, then a `source_node` mismatch. That matches FR-061 as it now stands.

The ticket's acceptance was "qualified matching accepted; wrong type rejected; wrong op rejected; QSL-2247 AC-4 reaches liveness". The first three are met by tests that pass at a1cca1ba. The QSL-2247 liveness check is FSL-side and was not run in this review.

## Examined scope

- FR-061-AC-14 (examined): a `T::op` class with the operation's anchor admits; it is also admitted under inheritance and survives read/write/read. Backed by `catalog_free_operation_step_qualified_source_class_is_admitted` and the updated fixtures. Mutant N6 is killed.
- FR-061-AC-15 (examined). Coverage per refusal:
  - Bare `save` refusing at `source_class`: covered; mutant N7 is killed.
  - `Other::save`, `Account::other` and the prefix/suffix spellings, plus the inherited `SubAccount::save`, refusing missing-name at `source_class` before `source_node`: covered.
  - `Order::cancel`-style declared op with another operation's anchor refusing at `source_node`: covered by `toggleAgain`.
  - Another type's anchor: not covered (FND-001).
- FR-061-AC-16 (examined): the earlier stages win, and with two row defects the first row wins (`catalog_free_operation_steps_join_in_received_order`, `catalog_free_operation_step_respects_earlier_reader_stages`). Covered.
- FSpec FR-375-AC-31 (examined): the point-qualified anchor and anchor-of-another-type refusals are tagged but unexercised (FND-001). The model-owner-node `source_node` case lost its test (code-review FND-004).
- TC-062 (context_only): matrix home of these tests; the trace tags `TC-062`, `FR-061-AC-14/15`, `FR-375-AC-31` resolve.
- Full workspace (examined): head and clean merge base fail the same 35 tests (see the code-review SR). No regression.

## Verdict

PARTIAL. The behaviour the ruling asks for is implemented and tested on its main paths. Two AC-named refusal branches have trace tags but no test that can fail.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-375-AC-31 and FR-061-AC-15 refusals tagged by the new tests are not exercised: a point-qualified `source_node` (and a `(T, op)` with only a point-qualified anchor), and a `source_node` that is the anchor of another type declaring the same `op`. Mutants N1, N2 and N5 survive. | tests/it/checked_package_v2_refinement.rs:4257-4377 |

## Dispositions

Disposition pass 1 at head 767b4eee.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | The point-qualified `source_node` and the only-point-qualified `(T, op)` are now exercised (mutants P1 and P2 killed). The `source_node` anchor of another selected-model type declaring the same `op` (FR-375-AC-31, FR-061-AC-15) is still unexercised: mutant P4 survives. |
