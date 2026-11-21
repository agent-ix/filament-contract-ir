---
id: SR-967175026302197768900474181941935084551
title: "QSL2271 PR 1121 gap-analysis of FR-378 and TC-493"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; QSL-2271; PR 1121 head 50ba1ace53b6388ff1e41b329b6c0bfb33e25dcb; spec/checked_package/functional/FR-378-read-protocol-correlation-provenance-of-a-checked-package.md, spec/checked_package/matrix/TC-493-protocol-correlation-provenance-read-from-an-admitted-package.md, spec/checked_package/matrix/tests.md; crates/filament-contract-model/src/checked_package/v2/protocol_correlation.rs; tests/conformance_fspec/protocol_correlation.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-378", type: references }
  - { target: "ix://agent-ix/filament-contract-ir/TC-493", type: references }
---

## Summary

Planless gap analysis of the obligations PR 1121 adds. Ticket: QSL-2271. I computed the Test Matrix with `quire matrix --scope . --format json` (quire 0.36.2, engine 0.50.2, rc=0). All five FR-378 criteria are tagged. The semantic check of intent, test and code was requested by the brief and done inline for every FR-378 criterion. It found two hollow parts in the evidence and one reverse gap.

## Verdict

**CONDITIONAL**: two medium findings and one low finding. No criterion is untagged.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Part of the FR-378-AC-1 evidence is hollow. The clause "a root without a member over a dependency with one" (CO-P11) is never exercised. read_case always supplies PC-DEP admitted without a member, so the case reads a root and a dependency that both lack a member. Non-inheritance from a dependency that does carry the member is not tested. | tests/conformance_fspec/protocol_correlation.rs:149-164,283-285 |
| FND-002 | medium | The FR-378-AC-4 storage oracle comes from the code under test. view_bytes takes G from the reader's own first denied view amount. The published storage_units.groups=6 is never asserted, so CO-B08 to CO-B11 would still pass if the reader charged the wrong number of group records. | tests/conformance_fspec/protocol_correlation.rs:615-628,674-678 |
| FND-003 | low | Reverse gap: the public error ProtocolCorrelationReadError::NotAnEligibleControl has no owning FR-378 text and no test. Neither has the public lookup ProtocolCorrelationTable::group(control, ordinal). | crates/filament-contract-model/src/checked_package/v2/protocol_correlation.rs:201-216,250-251,300-305 |

## Coverage

- **Matrix, FR-378:**
  - AC-1 is tagged by correlation_positive_vectors_read_as_published, correlation_dependency_is_read_from_its_own_member and correlation_group_carries_every_region_of_its_source_map_entry.
  - AC-2 is tagged by correlation_reader_refusals_name_code_cause_and_locus and correlation_schema_decisions_are_malformed_wire_at_the_first_error.
  - AC-3 is tagged by correlation_consumer_refuses_an_absent_member_over_nonempty_related_only.
  - AC-4 is tagged by correlation_budget_vectors_admit_exactly_and_deny_the_first_unpaid_charge.
  - AC-5 is tagged by tests::writer_preserves_correlation_rows_and_their_order and correlation_member_is_the_only_difference_between_invariance_pairs.
  - All 5 are tagged. The TC-493 row lists FR-378-AC-1 to AC-5.
- **Case counts checked by the tests:** 12 positive, 21 refusal, 12 schema_invalid plus 3 schema_valid, 16 budget, the consumer cases and 3 invariance pairs. The tests compare against the published literals: code, cause, locus, the typed-read tables, and the work offsets 1884, 542, 543, 1883, 103, 323, 116, 1179 and 1180.
- **Results:** reviewer rerun of the 8 tests passed (rc=0). Writer unit test: 1 passed.
- **AC-4 exact storage:** the CO-B11 reinterpretation is recorded as code-review FND-001 (SR-...550) and spec-review/base FND-002, not here.
- **Stubs:** none. No retained scratch probes, debug exports or test-support branches in production code.
- Plan completion: not assessed
