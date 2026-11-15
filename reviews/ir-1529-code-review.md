---
id: SR-20261115010004
title: "code-review and rust-review of IR-1529 source-declared object reader (PR #1025)"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1025 at 3615bd2f (base origin/main ce1970fb); crates/filament-contract-model/src/checked_package/v2/{callable,dispatch_projection,mod,operation_effects,operations,protocol_body,source_object,source_operation,structural}.rs, crates/filament-contract-model/src/lib.rs, tests/it/checked_package_v2_source_objects.rs, tests/it/main.rs"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in, for filament-contract-ir PR #1025. Ticket: IR-1529. Scope is `git diff origin/main...HEAD` only (12 files, all under `crates/filament-contract-model` and `tests/it`). The diff edits no spec file, no Makefile, and no ir1384 test lane, and touches nothing outside filament-contract-ir/filament-contract-model.

Public API: no `pub` item of filament-contract-model is renamed or removed. The additions are the `source_object` and `source_operation` re-exports, `CheckedCallableIdentity::SourceOperation`, `CheckedOperationEffectOrigin::SourceDeclarationOperationFrame` and `CheckedPackageV2::source_operation_callable`. Both enums are exhaustive and not `#[non_exhaustive]`, so a new variant breaks exhaustive matches downstream. No exhaustive match exists in this repo or in quire-spec-language `origin/main`, which only constructs `ModelDocumentOperationFrame`. The addition is therefore additive in practice.

A package without source objects keeps its old path. `operation_effects::validate` takes the old model path when no row has the source origin and no node is a source-declared type. `check_field_member` charges new work only for a source-declared declaring node. The population exceptions in `validate_declaration` and `validate_derived_type_shapes` apply only to a `SourceOwner`-owned population. No untrusted package input reaches an `unwrap`, `expect` or panicking index: every index comes from a validated position.

Tests run: `cargo test --all-features --test it checked_package_v2_source_objects` gave 31/31 passed.

## Verdict

Two defects block merge. `source_operation_callable` can return another source type's body function (FND-001). Its refusals report zero work after it has charged work (FND-002). Everything else is low. The FR-366 gaps are recorded in the gap-analysis SpecReview of this PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `source_operation_callable` takes the first `function` node under the context's `SourceOwner` whose declaration matches `[_, <operation>, "body"]` and never checks the `<type>` segment. All source types of one unit share one `SourceOwner`, so when `Order` and a redefining `Sub` both carry a `deposit` body, the read for one of the two anchors returns the other type's body function. The FR-366 form is `[<type>, <operation>, "body"]`. Require `qualified_name[0]` to equal the final `declaration` segment of the anchor's context, the same segment the row's `source_identity` check uses, and add a two-type test. | crates/filament-contract-model/src/checked_package/v2/source_operation.rs:216-233 |
| FND-002 | medium | Every refusal in `source_operation_callable` after the first charge comes from `let error = \|code\| Budget::refusal(code, anchor, 0)`, so `MalformedBody`, `MissingSelectedJoin` (after charging one unit per scanned node), `MissingBody` and `MissingFrame` report `work: 0`. `CheckedCallableReadError.work` is documented as "Charged work", and every other callable read builds its refusal with `budget.error`, which reports the charged work. | crates/filament-contract-model/src/checked_package/v2/source_operation.rs:188 |
| FND-003 | low | `SelectedSourceObjects::from_stage_rows` scans every graph node once per supplied row (`owners_authority`), O(rows × nodes), and nothing charges for it. FR-367 forbids a meter charge, so collect the `SourceOwner` authorities into a set once. | crates/filament-contract-model/src/checked_package/v2/source_object.rs:148-158 |
