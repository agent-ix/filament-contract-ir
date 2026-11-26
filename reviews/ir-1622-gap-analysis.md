---
id: SR-2026112600143100000002
title: "gap-analysis of IR-1622 budgeted-read charge ordering"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1189 at 96e076e3ab870014c879524f1c63f0755a2166db; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (FR-038-AC-499, AC-502, AC-504, AC-511); crates/filament-contract-model/src/checked_package/v2"
review_set: subset
---

## Summary

A gap check of PR #1189 against the criteria it cites. Ticket: IR-1622. Head reviewed: 96e076e3ab870014c879524f1c63f0755a2166db. `quire trace --id FR-038-AC-511` lists 110 claims. `quire trace --file .../model_fields.rs` lists none: the PR adds no `implements` markers, and the PR body says why (each changed function realises only part of a criterion). The audit covered only the criteria the PR cites. It did not walk the whole matrix.

## Examined scope

- FR-038-AC-499 (examined): "Every owned container is reserved at its exact final length". The claim-name, schedulability, operation-effect and operation-body lists are now exact. Covered by four new lib tests that run under default features, so CI runs them too.
- FR-038-AC-502 (examined): the field tables are the denied request at every request boundary, with no charged-but-unrequested bytes. Covered by the strengthened `tc_472_field_tables_allocate_exactly_what_they_request`.
- FR-038-AC-504 (examined): frame term lists. Covered by `frame_term_lists_are_paid_for_before_they_are_requested`.
- FR-038-AC-511 (examined): "no request is observed for a denied charge" and "a fixed bound that does not depend on input size". This holds for the vectors the PR fixes. It fails for serde-derived tagged or untagged wire enums (finding below).

## Verdict

The PR delivers its part. AC-511 stays partly unmet for a separate cause, which the PR says under "Not covered". That cause needs its own ticket. It does not belong in this PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Serde-derived tagged or untagged wire enums decoded under the budget (requirement-item extent, DomainKey with serde(tag = "type")) buffer through serde's content representation with uncharged allocations: 1342 bytes allocated against 388 charged, even under an unlimited allowance. This grows with row count, so it breaks FR-038-AC-511's fixed bound. The failing test is saved only outside the tree (gates-lead-reqs/ir-1622-requirement-extent-red-test.rs) and has no ticket yet | crates/filament-contract-model/src/checked_package/v2/requirement_records.rs:93-112, crates/filament-contract-model/src/checked_package/v2/intake.rs:105 |
