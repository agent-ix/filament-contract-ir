---
id: SR-958004
title: "gap-analysis of QSL-2776 clock-reference Exact projection (PR #1145)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1145 at b9d663da; FR-352-AC-45, FR-352-AC-46, TC-494, FR-352-AC-32"
review_set: subset
---

## Summary

Gap analysis scoped to the PR diff, matching FR-352-AC-45, FR-352-AC-46 and TC-494 clause by clause against the tests at b9d663da. Ticket: QSL-2776. Plan completion: not assessed. No spec files changed in this PR. The spec came from #1142 and was read at origin/main.

## Examined scope

- FR-352-AC-45 (examined). Covered with exact values:
  - Record `r` returns ClockReference, `maybe` returns ClockReference with Presence::Optional, and `refs` returns Sequence<ClockReference>[0, 3]. All three project with no identity row for the clock reference.
  - A clone gives an equal view.
  - The measured allowance succeeds, and one unit less returns ExhaustedWork with configured_bound set to the allowance minus 1.
  - A union payload of Reference<Clock> joins and returns ClockReference. The record field `payload` over the object type Node still returns Reference(EffectiveId), which covers the `peer` clause.
  - The tuple position returns ClockReference, but only as `(Boolean, Reference<Clock>)` (FND-001).
  - The unit test covers wrapped Option and Sequence[0, 3] payloads.
  - `ra` through an alias is covered only at member-type level by tc_234 (FND-004).
- FR-352-AC-46 (examined). Covered: MismatchedIdentityWitness at the payload node, form `/model_unions/0/cases/1/payload/original_type`, in both directions (Reference(EffectiveId) for a clock-reference payload, ClockReference for an object payload). Not covered: the Clock field locus (FND-002). The Timed(Clock) retained payload falls under AC-32's existing tests. The "no place" clause holds by construction, because the projection takes no value input.
- TC-494 (examined): steps 1, 2 and 4 are covered as above. Step 3 is covered only partly (FND-002). Step 1's tuple deviates (FND-001).
- The intake discriminator `clock_references` (examined). Its negative side has no test (FND-003).
- tests/conformance_fsl_ir1384/Cargo.toml (examined, FND-005).

## Verdict

Three medium gaps:

- The TC-494 tuple fixture cannot be met as written, the PR silently substitutes a different fixture, and plain Clock at a tuple position is untested.
- AC-46's Clock field locus is untested, and the code puts it somewhere else.
- The new intake set has no negative test.

The other clauses are covered with exact values.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-45 and TC-494 name a tuple `(Clock, Reference<Clock>)` and expect position 1 to return ClockReference. That cannot be met, because the selected tuple projection is all or nothing. A plain Clock position (admitted at intake, model_members.rs:5517-5531, Native Clock maps to true) refuses the whole tuple with UnsupportedExactType at the tuple node, form `tuple_position`. The test silently uses `(Boolean, Reference<Clock>)`, and no test asserts the Clock-position refusal. Fix in this PR: correct the AC-45 and TC-494 tuple text, and add a test that `(Clock, Reference<Clock>)` refuses UnsupportedExactType at the tuple node, form tuple_position. | crates/filament-contract-model/src/checked_package/v2/runtime_type_projection.rs:1342-1365; tests/it/checked_package_v2_model_fields.rs:9886-9932 |
| FND-002 | medium | AC-46 says a field typed `Clock` returns UnsupportedExactType "at that field's type node". The code refuses at the requested record node (`refuse!(UnsupportedExactType, Some(requested))`), and the existing test `admitted_record_with_unsupported_field_refuses_without_view` asserts `at == record node`. The PR traces AC-46 but does not test this clause in the clock-reference model. Fix: align the AC-46 wording with the record-node locus (or change the locus), and assert it in the clock-reference record test. | crates/filament-contract-model/src/checked_package/v2/exact_projection.rs:3502-3507; tests/it/checked_package_v2_model_fields.rs:4087-4099 |
| FND-003 | medium | Nothing tests that a `reference` type over an object (not Clock) is still refused as a tuple position and as a union payload. If every ReferenceType were inserted into `clock_references`, every test would still pass. Fix: add a negative intake test for both positions, and one with the ClockRef declared after its user, to pin declaration-order independence. | crates/filament-contract-model/src/checked_package/v2/model_members.rs:5107-5118 |
| FND-004 | low | The `ra: Reference<A>` (`type A = Clock`) clause of AC-45 is not projected. The alias path is proved only at member-type level by tc_234. Add `ra` to the clock-reference record test. | tests/it/checked_package_v2_model_fields.rs:9741-9760 |
| FND-005 | low | This PR moves the model crate to filament-exact 08a5a176 and quire-canonical v0.3.2. tests/conformance_fsl_ir1384 is a separate workspace that pins filament-exact b26a8e4c and quire-canonical v0.3.1 and uses `filament_exact` types directly, so that lane resolves two copies of each. The lane had already diverged on main (b26a8e4c against a58474cc), so this is not a regression. Realign the two pins in this PR, since it moves them anyway. | tests/conformance_fsl_ir1384/Cargo.toml:15-16 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At acc267fb, AC-45 and TC-494 name `(Boolean, Reference<Clock>)` for position 1 and add `(Clock, Reference<Clock>)` (field `mixed`), which refuses UnsupportedExactType at the tuple node, form `tuple_position`. The new test `a_plain_clock_position_or_field_refuses_beside_a_clock_reference` asserts the code, at and form, and passes. quire validate exits 0. |
| FND-002 | fixed | AC-46 and TC-494 step 3 now say the Clock field refuses at the record node, form `model/record_value_type`. Amending the wording rather than the locus is right: FR-352 prose says a plain Clock member SHALL keep its existing outcomes, and that both refusals SHALL retain the requested node. The same new test asserts at == record node and form == model/record_value_type beside a clock-reference field. |
| FND-003 | fixed | The new test `only_a_reference_type_over_clock_is_admitted_as_a_tuple_position_and_a_union_payload` checks two things. A reference over an object refuses as a tuple position (InvalidModelBinding/MalformedDeclaration at /types/2/positions/0) and as a union payload (UnsupportedConstruct/DeclarationForm at /types/2/variants/0/payloadType). ZClockRef, declared and sorted after its users, admits in both positions, directly over Clock and through an alias of Clock. |
| FND-004 | fixed | `ra` (AliasRef over alias A = Clock) is now projected and asserts ClockReference with Presence::Required. The allowance assertions now pin (11, 10) and first unpaid GraphNode(5), form record_field_result. |
| FND-005 | fixed | The lane now pins filament-exact 08a5a176 and quire-canonical v0.3.2, matching the model crate. The lane also failed to build on origin/main fe49a2a2, before this PR: `cargo check --manifest-path tests/conformance_fsl_ir1384/Cargo.toml --tests` in a clean scratch worktree exits 101. It shows two filament-exact copies (a58474cc via FSL main, b26a8e4c direct), E0308, and 16 errors in fsl-package (ValueValidityRecord.path_condition, RequirementRecord fields). So the realignment does not break a lane that was building. |
