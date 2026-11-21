---
id: SR-17952600200001
title: "QSL-2743 fourth-window code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; QSL-2743; crates/filament-contract-model/src/checked_package/v2/exact_projection.rs, crates/filament-contract-model/src/checked_package/v2/exact_projection/complete_witness.rs, crates/filament-contract-model/src/checked_package/v2/exact_projection/model_enum_tests.rs, crates/filament-contract-model/src/checked_package/v2/mod.rs, crates/filament-contract-model/src/checked_package/v2/model_enum_witness.rs, crates/filament-contract-model/src/checked_package/v2/monitor_plan/reached_model_read.rs, crates/filament-contract-model/src/checked_package/v2/runtime_type_projection.rs, crates/filament-contract-model/src/lib.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir", type: references }
---

## Summary

Ticket: QSL-2743. Independent review of the fourth-window enum-only reader partial. No spec, wire or dependency changes. Payload-bearing model variants remain typed UnsupportedExactType (selected_union). QSL-2743 remains open for the model-union carrier, FSL producer integration, real bare/source value parity and unused enum emission. These are required handoffs, not newly introduced reader defects.

Validation: existing supplied logs report 4 model_enum_runtime tests and 1 reached_enum_shape test passing; git diff --check passes. No Cargo/build commands or source edits were run by the reviewer. Broad lint/workspace gates were not rerun under the dispatch restriction. No applicable AssuranceProfile found. No vendored code or new stub was found in the diff.

## Verdict

FAIL. Two concrete reader findings require a fix round.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Model enum shape authentication accepts a noncanonical case permutation. Reversing both cases and shape.variants for unordered Tier leaves every existing membership/census check true, so the new public admission installs Standard before Gold and exposes incorrect ranks. Validate unordered UTF-8 case order, and ordered case positions against the retained declaration facts, before installation. | crates/filament-contract-model/src/checked_package/v2/model_enum_witness.rs:619-634 |
| FND-002 | high | Enum-valued field projection deep-clones the complete EnumShape after charging only one storage unit and no per-variant clone work. EnumShape owns a Vec<VariantId> and every VariantId owns its case String in the selected Exact dependency. M fields sharing an N-case enum therefore copy O(M*N) elements/text outside the cumulative budget. Preflight and charge the complete shape before this clone, as the direct projection does. | crates/filament-contract-model/src/checked_package/v2/exact_projection.rs:3408-3409 |

## Coverage

Plan completion: not assessed

Checked complete case census, declaration/member/shape correspondence, owner labels, identity conflicts, rejoin equality, alias field/direct equality, package binding, atomic errors and cumulative budget boundaries. One-shot budget tests cannot detect undercharged clones because they measure the implementation itself. Reversed shape+case permutations are not exercised by the supplied tests.

```yaml
scope:
  - id: FR-352-AC-30
    path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md
    role: examined
    excerpt: "Direct projection of an admitted selected `model/variant_type` with
      its complete installed model enum shape returns `ValueType::Enum` equal to
      that shape, retaining ordering and every opaque variant identity. A
      package clone agrees. The reached-model join and following projection
      share one `NativeJoinBudget`: their measured exact cumulative work and
      storage succeed, and one unit below either refuses at the first unpaid
      kind and locus without a partial projection. An absent shape returns
      `UnavailableIdentityWitness`, and identities installed for an independent
      admission refuse `ForeignNode`. A "
  - id: FR-354-AC-1
    path: spec/checked_package/functional/FR-354-project-admitted-runtime-context.md
    role: examined
    excerpt: With the FR-352 and additional FSL-stage rows for selected tuples and
      source-owned units, and the required FSL producer facts for object
      operations, a public IR-model read supplies complete record, tuple and
      object declarations, effective fields, ancestry, own object operations and
      units from which FSL constructs the expected `TypeEnvironment`; no source
      or JSON is reparsed. Before a required producer fact exists, the
      corresponding request refuses rather than claiming a complete view.
  - id: FR-354-AC-2
    path: spec/checked_package/functional/FR-354-project-admitted-runtime-context.md
    role: examined
    excerpt: A missing, foreign, duplicated or conflicting selected declaration,
      field, parent or unit identity, including an additional tuple or
      source-unit row, returns a named typed outcome before any partial
      type-environment view; matching display names do not make a join.
  - id: FR-354-AC-6
    path: spec/checked_package/functional/FR-354-project-admitted-runtime-context.md
    role: examined
    excerpt: With FSL-stage enum declaration and member rows, an ordered and an
      unordered selected enum each yield exact case order, member identity and
      position sufficient for `EnumMemberIndex`; absent, duplicate,
      mismatched-case or foreign member witnesses refuse before any partial
      index.
  - id: FR-354-AC-7
    path: spec/checked_package/functional/FR-354-project-admitted-runtime-context.md
    role: examined
    excerpt: Repeated bounded reads of an admitted package and its clone agree; a
      selective admitted monitor-entry callable read returns its entry identity,
      ordered parameter handles and expression body handle with charged success
      work, without scanning unrelated graph nodes or claiming complete type
      closure. The exact successful work limit admits; a limit one below reports
      charged work and locus without a partial view, source recheck, wire
      reparse or stack growth with term depth.
  - id: FR-354-AC-8
    path: spec/checked_package/functional/FR-354-project-admitted-runtime-context.md
    role: examined
    excerpt: Public API and dependency inspection finds the runtime projections in
      IR model, no FSL `Scope`, `CheckedGraph`, evaluator, supplied-state
      admission or mirror expression type in IR, and no claimed success for a
      producer decision absent from the package.
  - id: FR-358-AC-3
    path: spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md
    role: examined
    excerpt: Every IR-owned row, key, handle, container and map/set entry retained
      or cloned by the budgeted path, and each byte of copied dynamic payload,
      is charged by the defined logical-storage rule before the corresponding
      reserve, clone or insert. On a budget denial the complete owned diagnostic
      floor is charged before any owned diagnostic allocation; at zero storage
      the refusal allocates no owned diagnostic. The tests make a prior nonempty
      map require an `Arc::make_mut` copy and show that a limit one below that
      copy's charge refuses before the new row appears. A borrowed inspection
      incurs no storage
  - id: crates/filament-contract-model/src/checked_package/v2/exact_projection.rs
    path: crates/filament-contract-model/src/checked_package/v2/exact_projection.rs
    role: examined
    excerpt: |-
      // SPDX-License-Identifier: AGPL-3.0-or-later
      //! Read-only projection of admitted checked types into Exact value types.

      use super::extension::integer_range_bounds;
  - id: crates/filament-contract-model/src/checked_package/v2/exact_projection/complete_witness.rs
    path: crates/filament-contract-model/src/checked_package/v2/exact_projection/complete_witness.rs
    role: examined
    excerpt: >-
      // SPDX-License-Identifier: AGPL-3.0-or-later

      //! FR-352: atomic package-local validation of every retained identity
      channel.


      use super::super::model_members::DeclarationForm;
  - id: crates/filament-contract-model/src/checked_package/v2/exact_projection/model_enum_tests.rs
    path: crates/filament-contract-model/src/checked_package/v2/exact_projection/model_enum_tests.rs
    role: examined
    excerpt: >-
      // SPDX-License-Identifier: AGPL-3.0-or-later

      // Copyright (C) 2026 Agent-IX


      //! Genuine FSL emission evidence for direct selected model enum
      projection.
  - id: crates/filament-contract-model/src/checked_package/v2/mod.rs
    path: crates/filament-contract-model/src/checked_package/v2/mod.rs
    role: examined
    excerpt: >-
      //! Strict reader for FSpec I04 `filament.checked-package/v2`, the sole
      admitted

      //! `CheckedPackage` contract.

      //!

      //! Implements the I04 contract described by AD-006. The wire shape this
  - id: crates/filament-contract-model/src/checked_package/v2/model_enum_witness.rs
    path: crates/filament-contract-model/src/checked_package/v2/model_enum_witness.rs
    role: examined
    excerpt: >-
      // SPDX-License-Identifier: AGPL-3.0-or-later

      // Copyright (C) 2026 Agent-IX


      //! FR-354: join FSL's model-owned enum identities to admitted selected
      cases.
  - id: crates/filament-contract-model/src/checked_package/v2/monitor_plan/reached_model_read.rs
    path: crates/filament-contract-model/src/checked_package/v2/monitor_plan/reached_model_read.rs
    role: examined
    excerpt: |-
      // SPDX-License-Identifier: AGPL-3.0-or-later
      // Copyright (C) 2026 Agent-IX

      //! Reached model and unit facts for the selected monitor graph walk.
  - id: crates/filament-contract-model/src/checked_package/v2/runtime_type_projection.rs
    path: crates/filament-contract-model/src/checked_package/v2/runtime_type_projection.rs
    role: examined
    excerpt: >-
      // SPDX-License-Identifier: AGPL-3.0-or-later

      // Copyright (C) 2026 Agent-IX


      //! FR-354: admitted selected type declarations for a sealed evaluator
      adapter.
  - id: crates/filament-contract-model/src/lib.rs
    path: crates/filament-contract-model/src/lib.rs
    role: examined
    excerpt: >-
      //! Versioned, implementation-language-independent semantic contract
      model.

      //!

      //! This package is the cycle-free substrate shared by Filament owner
      crates and

      //! the `filament-contract-ir` bounded-Kani crate. It intentionally has no
bindings:
  - test_id: genuine_tier_projects_the_authenticated_installed_producer_shape
    ac_id: FR-352-AC-30
    trace: correct
  - test_id: model_enum_runtime_full_view_matches_direct_field_and_literal_types_with_cumulative_budget
    ac_id: FR-352-AC-30
    trace: correct
  - test_id: model_enum_runtime_rejects_missing_conflicting_foreign_and_incomplete_witnesses
    ac_id: FR-352-AC-30
    trace: correct
  - test_id: model_enum_runtime_requires_unused_enum_and_keeps_same_case_declarations_distinct
    ac_id: FR-352-AC-30
    trace: correct
  - test_id: model_enum_runtime_full_view_matches_direct_field_and_literal_types_with_cumulative_budget
    ac_id: FR-354-AC-1
    trace: correct
  - test_id: model_enum_runtime_requires_unused_enum_and_keeps_same_case_declarations_distinct
    ac_id: FR-354-AC-1
    trace: correct
  - test_id: model_enum_runtime_rejects_missing_conflicting_foreign_and_incomplete_witnesses
    ac_id: FR-354-AC-2
    trace: correct
  - test_id: model_enum_runtime_requires_unused_enum_and_keeps_same_case_declarations_distinct
    ac_id: FR-354-AC-2
    trace: correct
  - test_id: model_enum_runtime_payload_variant_refuses_without_becoming_an_enum
    ac_id: FR-354-AC-2
    trace: correct
  - test_id: model_enum_runtime_full_view_matches_direct_field_and_literal_types_with_cumulative_budget
    ac_id: FR-354-AC-6
    trace: correct
  - test_id: model_enum_runtime_rejects_missing_conflicting_foreign_and_incomplete_witnesses
    ac_id: FR-354-AC-6
    trace: correct
  - test_id: model_enum_runtime_requires_unused_enum_and_keeps_same_case_declarations_distinct
    ac_id: FR-354-AC-6
    trace: correct
  - test_id: model_enum_runtime_payload_variant_refuses_without_becoming_an_enum
    ac_id: FR-354-AC-6
    trace: correct
  - test_id: model_enum_runtime_full_view_matches_direct_field_and_literal_types_with_cumulative_budget
    ac_id: FR-354-AC-7
    trace: correct
  - test_id: model_enum_runtime_payload_variant_refuses_without_becoming_an_enum
    ac_id: FR-354-AC-8
    trace: correct
```
