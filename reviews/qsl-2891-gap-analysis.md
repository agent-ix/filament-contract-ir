---
id: SR-967175026302197768900474181941935084591
title: "Gap analysis — filament-contract-ir PR #1146 (QSL-2891 slot-domain maximum)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; crates/filament-contract-model/src/checked_package/v2/operations.rs, spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: subset
---

## Summary

Ticket: QSL-2891. This audit is scoped to the PR #1146 diff. `quire matrix --scope <repo> --format json` ran. FR-038-AC-552 (slot bounds on the field row) is tagged by `set_of_clock_bearing_tuples_is_a_clock_collection_bound_to_the_field` (tests/conformance_fspec/clock_payload_tuple.rs:2173). The changed rule belongs to FSpec FR-772 "The slot-domain node" (agent-ix/filament-specification): `result_type` is `Set<D>`, or the `collection_bounds` `[0, N]` node when the collection's declared maximum is `N`. This repo's spec does not mention FR-772 or `filament.op.option.slots` anywhere. The test's trace ids FR-772-AC-1, FR-772-AC-2 and TC-1862 therefore name no criterion of this repo. FSpec FR-772 gives no refusal code, cause or pointer for a `[0, N]` result whose `N` differs from the declared maximum. The code's `ill_typed`/`operator-ineligible` at `/body/result_type` is unspecified in both repos.

## Verdict

**CONDITIONAL**: there are two medium findings and no untagged criterion in the changed scope.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Spec gap: FR-038 has no reader clause or acceptance criterion for the FSpec FR-772 slot-domain result bound. It does not say that the `collection_bounds` max must equal the collection maximum spelled on the row before the terminal slot step, or that a mismatch refuses `ill_typed`/`operator-ineligible` at `/body/result_type`. FSpec FR-772-AC-2 also lacks this negative. The behaviour this PR fixes has no owning criterion in this repo; an FR-038 AC alongside AC-552 citing FSpec FR-772 is needed, and an FR-772-AC-2 negative upstream. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1737-1776 |
| FND-002 | medium | No admitted full-package FR-772-AC-1 control runs a real `field q` + `slot` place through `check_resolved_path` and then `check_place_slot_domain`. tests/it/checked_package_v2_typed_terms.rs:557-560 states that none exists. The fix is proven only by a unit fixture with hand-built rows, which is how the off-by-one row read shipped. | crates/filament-contract-model/src/checked_package/v2/operations.rs:14498-14675 |

## Coverage

- Matrix: computed by `quire matrix`; FR-038-AC-552 tagged; FR-772-AC-1/AC-2 are foreign ids that this repo does not declare.
- Reverse gap: `slot_domain_result_matches` bound check has no owning criterion in this repo (FND-001).
- Stubs: none in the diff.
- Semantic review: skipped (not requested).
- Plan completion: not assessed

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-038-AC-614 added (with its TC-048 matrix row): result `Set<D>` or `collection_bounds` `[0, N]` with `N` the terminal slot step's maximum, joined equal to the row before the slot (AC-552); `field q`, `slot(sequence, 0, 2)` admits `[0, 2]` and `[0, 3]` refuses `ill_typed`/`operator-ineligible` at `/body/result_type`. Consistent with FSpec FR-772 and FR-038-AC-552. The FSpec FR-772-AC-2 negative is upstream in agent-ix/filament-specification and not part of this repo. |
| FND-002 | fixed | `slot_domain_over_a_field_and_slot_place_admits_its_declared_maximum_only` (tests/conformance_fspec/clock_payload_tuple.rs) reads a real `field clocks` + `slot(set, 0, 2)` package through the strict reader: `[0, 2]` admits, `[0, 3]` refuses at `/body/result_type`. It passes at head and fails on the pre-fix code (`[0, 2]` refused), verified on titan. |
