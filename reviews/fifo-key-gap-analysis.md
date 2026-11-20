---
id: SR-17940486709564847048121
title: "Gap analysis — QSL-2726 FIFO-key-stable keys against FR-372 and FSpec FR-726"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; branch code/fifo-key-quantity at fbcddd32; spec/checked_package/functional/FR-372-admit-bounded-channels-population-captures-reference-carriers-and-post-test-repeats.md; crates/filament-contract-model/src/checked_package/v2/protocol_body.rs; tests/it/checked_package_v2_protocol_body.rs"
review_set: subset
---

## Summary

Ticket: QSL-2726. This is a planless gap analysis scoped to the criteria the PR touches. It covers local FR-372-AC-1/AC-2/AC-3, each of which "Implements FSpec FR-726-AC-n", and FSpec FR-726-AC-1..3 with TC-1770. Evidence is `quire matrix --scope .` (quire 0.36.2) at head plus the FR-372 text. Plan completion: not assessed.

## Examined scope

- FR-372-AC-1 (examined): tagged by tests/it/checked_package_v2_protocol_body.rs:10277. It implements FR-726-AC-1 and is unchanged by this PR.
- FR-372-AC-2 (examined): the matrix binders are lines 10514, 10546, 10619, 11039 and 11142. The new test `protocol_channel_admits_fifo_unit_quantity_keys` (10996) is not a binder.
- FR-372-AC-3 (examined): the matrix binders are lines 11197, 11254 and 11359. The new test `protocol_channel_fifo_refuses_a_float_leaf_beside_a_quantity` (11016) is not a binder.
- FR-726-AC-1..3 / TC-1770 (examined): these are FSpec ids. They resolve to no local criterion, so the two new tags bind nothing in this repository's matrix.
- Code against intent (examined): the walk matches FR-372 "FIFO-key-stable keys" and FR-726 "Behavior". The only drift is in the prose (FND-002).

## Verdict

**PASS with findings.** Every criterion the PR touches has executing tests. The PR's own tests miss the local criterion, and the local prose lags the code.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new tests carry only `#[trace("TC-1770", "FR-726-AC-2")]` and `("TC-1770", "FR-726-AC-3")`. These are FSpec ids that bind to no local criterion, so the matrix does not credit FR-372-AC-2 or FR-372-AC-3 (the local criteria implementing FR-726-AC-2/3) with the new bare-unit, bytes, json_object and float-beside-quantity coverage. Add `FR-372-AC-2` and `FR-372-AC-3` to the tags. | tests/it/checked_package_v2_protocol_body.rs:10994, tests/it/checked_package_v2_protocol_body.rs:11014 |
| FND-002 | low | FR-372 "FIFO-key-stable keys" lists the stable leaves as `bounded_quantity`, `bytes`, `json_object`, an enum and a reference, plus `carried_quantity` via its carrier. It does not name the bare `scalar_type`/`unit` node (`Quantity<m>`) or `compound_unit`, which the code admits. The Status section still says the negative cases of AC-3 have no tests, but lines 11197, 11254, 11359 and 11016 test them. | spec/checked_package/functional/FR-372-admit-bounded-channels-population-captures-reference-carriers-and-post-test-repeats.md:85-87, spec/checked_package/functional/FR-372-admit-bounded-channels-population-captures-reference-carriers-and-post-test-repeats.md:180-187 |
