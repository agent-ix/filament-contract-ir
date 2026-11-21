---
id: SR-17952600200002
title: "QSL-2743 fourth-window gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; QSL-2743; spec/, src/, crates/, tests/"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir", type: references }
---

## Summary

Ticket: QSL-2743. Independent review of the fourth-window enum-only reader partial. No spec, wire or dependency changes. Payload-bearing model variants remain typed UnsupportedExactType (selected_union). QSL-2743 remains open for the model-union carrier, FSL producer integration, real bare/source value parity and unused enum emission. These are required handoffs, not newly introduced reader defects.

Validation: existing supplied logs report 4 model_enum_runtime tests and 1 reached_enum_shape test passing; git diff --check passes. No Cargo/build commands or source edits were run by the reviewer. Broad lint/workspace gates were not rerun under the dispatch restriction. No applicable AssuranceProfile found. No vendored code or new stub was found in the diff.

## Verdict

FAIL. Repository assurance is not green; baseline missing bindings prevent PASS. No additional unowned behavior or stub was found in the changed surface.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The repository-wide computed matrix contains 115 untagged criteria and 1 ignored-only criterion, so an engine-backed repository PASS is unavailable. These are baseline assurance debt outside this enum-only reader diff; the six examined runtime/enum criteria have applicable tagged or inspection status. | spec/:1 |

## Coverage

Plan completion: not assessed

Reconciliation: quire matrix (quire 0.52.0, engine 0.62.2); no run evidence read. Repository counts: {"tagged":1155,"untagged":115,"method-without-symbol":26,"retired":8,"tagged-by-ignored-test":1}. Coverage command completed with 1972 diagnostics and 306 untracked symbols; Rust binding census: 2921 candidates, 2856 tagged, 2755 bound. Existing symbol-id collisions, unmarked trace claims and missing NFR headings are retained as limitations, not silently counted as successful evidence. New tests have no untracked symbols. Public surface inventory found 761 pub fn declarations; reverse-gap ownership inspection covered the changed APIs and adjacent callers under FR-352/354/358. Full semantic ownership inspection of all 761 public functions was not performed in this bounded PR review. Source scan found no todo!/unimplemented! stubs; no false-completion test stub was found in the changed tests. Optional repository-wide semantic review skipped; code-review inspected concrete requirement/test/code behavior. The baseline counts do not claim 115 new regressions.

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

## Dispositions

Round 1. Current repository gap verdict remains FAIL; baseline FND-001 is accepted without change for this PR only. This is not a repository assurance PASS.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | accepted-no-change | Explicit dispatching lead authorization for this scoped enum-only reader partial: 115 untagged criteria and one ignored-only criterion are preexisting repository assurance debt outside this PR. No engine-backed repository PASS is claimed. Global remediation or new tickets are not required by this bounded review. QSL-2743 remains open for producer integration, actual bare/source value parity, unused enum emission and the model-union carrier. |

Evidence: supplied fix-round RED log reports the two new tests failing on the old implementation (permutation admitted; three enum field clones charged work 0/storage 3 versus independently expected 48/54). Final logs report six model_enum_runtime tests and one reached_enum_shape budget test passing. The reviewer inspected implementation and tests, checked diff whitespace and confirmed committed original SR files were byte-identical to the original scratch artifacts. No Cargo/build gate or source edit was run.

```yaml
dispositions:
  - fnd: FND-001
    outcome: accepted-no-change
    reason: "Explicit dispatching lead authorization for this scoped enum-only
      reader partial: 115 untagged criteria and one ignored-only criterion are
      preexisting repository assurance debt outside this PR. No engine-backed
      repository PASS is claimed. Global remediation or new tickets are not
      required by this bounded review. QSL-2743 remains open for producer
      integration, actual bare/source value parity, unused enum emission and the
      model-union carrier."
```
