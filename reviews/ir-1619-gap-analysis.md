---
id: SR-5761732
title: "gap-analysis of IR-1619 event-reach operand refusal"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1177 at 9a275b2109622e73721e5f673688709916741bfd, delta over 9961b272b2be2750ea365e9d57b20b53ad3c94d5; tests/it/checked_package_v2_model_fields.rs, tests/it/checked_package_v2_recursive_leaves.rs"
review_set: subset
---

## Summary

Manual acceptance-criteria-to-test check, scoped to the PR delta. Ticket: IR-1619. The full `quire matrix` run was not performed for this stacked delta: this is a scoped manual check, not a full gap audit. No tests were run (host rule), so every binding below comes from reading the code.

## Examined scope

- FR-038-AC-633, examined. It binds to `tc_048_an_operation_over_an_event_typed_parameter_is_ineligible` and `tc_048_a_binder_operand_holding_an_event_typed_parameter_is_ineligible`. Both edits are correct (see SR-5761730).
- FR-038-AC-634, examined. It binds to `tc_048_an_operand_whose_type_holds_event_elements_is_ineligible`, `tc_048_an_operand_composite_holding_an_event_type_is_ineligible`, `tc_048_a_model_record_holding_an_event_sequence_is_an_ineligible_operand`, `tc_048_a_literal_or_declaration_operand_reaching_an_event_type_is_ineligible`, `tc_048_a_binder_operand_reading_a_composite_of_events_is_ineligible` and `tc_048_the_event_type_walk_ends_on_a_recursive_compared_type`. Each asserts the code, the cause and the exact argument locus.
- FR-038-AC-635, examined. It binds to `tc_281_equality_over_a_type_reaching_no_event_type_stays_admitted` and the admitted half of `tc_048_the_event_type_walk_ends_on_a_recursive_compared_type`.
- The ticket's six reproduction cases, examined. All six are covered: Option, Set and Sequence equality, `option.value` (refused at the producer's own operand), and record equality and projection. The record cases use a structural `composite_type`/`record`.
- Expected counts, by counting `#[test]` items. The six `--test it` filters give 532: model_fields 104 (91 in the file plus 9 in operation_census and 4 in option_types; 6 new), recursive_leaves 27 (1 new), temporal 121, protocol_body 237, owners 34, composite_operands 9. `checked_package::v2::operations::tests::` gives 105, unchanged. All of these hold only once SR-5761730 FND-001 is fixed.

## Verdict

The criteria are bound to tests that pin the code, cause and locus. The domain-walk branches that FR-038 names are mostly untested (FND-001).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038 says the domain walk follows inherited record fields, tuple positions, variant payloads, option and map definitions, and stops at reference and object types. Tests reach only own field → domain sequence. No test covers an inherited event-holding field, a domain tuple, a variant payload, an option or map definition, or a domain reference or object field that must stay admitted. | crates/filament-contract-model/src/checked_package/v2/model_members.rs:2057-2130; tests/it/checked_package_v2_model_fields.rs:10804 |
