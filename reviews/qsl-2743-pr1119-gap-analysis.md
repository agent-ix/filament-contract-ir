---
id: SR-967175026302197768900474181941935084531
title: "QSL-2743 PR 1119 planless gap-analysis with semantic review of the new and changed criteria"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; QSL-2743; PR 1119 head 5e7f682c7321a400fe693d649cd555095af19b5a; FR-352-AC-32, FR-352-AC-38..44, FR-358-AC-17, FR-358-AC-19; tests/it/checked_package_v2_model_fields.rs; tests/conformance_fspec/model_union_witness.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-352", type: references }
  - { target: "ix://agent-ix/filament-contract-ir/FR-358", type: references }
---

## Summary

Planless gap analysis of the criteria PR 1119 adds or changes, using the computed Test Matrix from `quire matrix --scope . --format tsv` (quire 0.36.2). Ticket: QSL-2743. Plan completion: not assessed. The optional semantic review (intent, test and code) was run as the brief directs.

Matrix: FR-352-AC-38, -39, -40, -41, -42, -43 and -44 are all `tagged`, each to tests/it/checked_package_v2_model_fields.rs:9203:1 (the `#[trace(...)]` attribute of the one parameterised test). FR-358-AC-17 is tagged to model_union_witness.rs:601, 968 (budget_case_b02) and 1266. FR-358-AC-19 is tagged to model_union_witness.rs:732, 1157 and 1203. The counts cover only these targeted criteria; no repository-wide claim is made.

Reverse gap: none. The PR adds no production code. Stubs: none in the changed test; there is no `todo!`, `unimplemented!`, `#[ignore]` or tautological assertion.

## Verdict

**CONDITIONAL** — two low findings on the strength of the adverse cases.

Semantic review, held clean:
- All seven record-field positives are independent and faithful. Each kind is strict-admitted from its own document and graph and projected through the record entry `project_exact_type(record).record_view()`. The result is compared with a test-built Exact type covering width and strict mode, bounds and profile, or unit and carrier domain.
- The adverse refusal at `/model_unions/0/cases/1/payload/original_type` does exercise the shared record-field projector. `model_union_expected_type` calls `project_wrapped_record_field_type` (model_union.rs:1470-1492), and the comparison at model_union.rs:1417 refuses when the supplied type differs. So for Float32, Float64, Bytes, JsonObject and carried Rational, the adverse can fail on its own if the projector maps the kind wrongly. The M2 mutant (Float32 mapped to binary64) would also have turned the adverse into a passing join.
- These refusals are union-join decisions taken before any record-field projection. FR-352-AC-38..44 describe them truthfully, as model union payload refusals. They are not presented as refusals of the record-field entry.
- Of the 14 per-kind oracles, 12 discriminate the kind or its parameters: 7 positives and the adverse cases for Float32, Float64, Bytes, JsonObject and carried Rational. The 2 adverse cases for bounded Rational and bounded Text are generic (FND-001).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The adverse cases for bounded Rational (FR-352-AC-42) and bounded Text (FR-352-AC-43) take the wrong original type `ValueType::Boolean`, shared from the older forms map. Those cases cannot fail for any interval, denominator or profile defect in the shared projector; only the positive equality guards the bounds the criteria name. A same-kind wrong type, such as a `RationalDomain` with another numerator bound or a `TextType` with another length or profile, would make each adverse discriminate the record-field obligation. | tests/it/checked_package_v2_model_fields.rs:9380-9392; spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md:638-639 |
| FND-002 | low | FR-352-AC-44 says an original type naming another carrier domain or unit refuses. The carried-quantity adverse varies only the domain (`domain(8)`, same `second` unit). Only the Duration `Quantity` form varies the unit, so no test refuses a carried quantity with the wrong unit. The missing-unit-row refusal (`UnavailableIdentityWitness` at the unit node) is likewise taken only in the union join, never at the record-field entry the criterion is about. | tests/it/checked_package_v2_model_fields.rs:9356-9368, 9514-9533; spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md:640 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Disposition pass 1, head c9b9a2415. The wrong witnesses are now same-kind and built independently (checked_package_v2_model_fields.rs:9407-9412, 9524-9537): integer range 0..999; text upper bound 65 and profile nfd; rational numerator upper 8 and numerator lower -4; option Option(Boolean). Each is refused MismatchedIdentityWitness at the payload node, form /model_unions/0/cases/1/payload/original_type. Mutant m3 (text upper bound +1 in project_record_field_type) was killed and restored. |
| FND-002 | fixed | Disposition pass 1, head c9b9a2415. A carried-quantity wrong-unit case was added (`unit_exact("minute")`, same domain). The new test a_carried_quantity_record_field_needs_the_declared_unit_identity_row (:9583-9696) uses record and field rows only, with no union stage. Its positive (with the DeclaredUnit row) asserts an independently built CarriedQuantity through project_exact_type(record).record_view(). Removing only the unit row then refuses UnavailableIdentityWitness at the unit node. FR-352-AC-44 states that field-path refusal. |

## Disposition round 1 verdict

**PASS.** Both findings are fixed and the pass adds no new finding.
- All 7 kinds have an independent positive through the record entry.
- Every adverse case is now kind-specific: Float32 and Float64 the other width; Bytes and JsonObject swapped; Rational two bound variants; Text a bound and a profile; carried Rational a domain and a unit; Duration a unit.
- The carried record-field missing-unit refusal is taken on the record-field path itself.
- Matrix: FR-352-AC-44 now has two binders.
