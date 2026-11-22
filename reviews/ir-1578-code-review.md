---
id: SR-967175026302197768900474181935084588
title: "IR-1578 PR 1129 code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; crates/filament-contract-model/src/checked_package/v2/runtime_type_projection.rs; tests/it/nominal_reads.rs"
review_set: subset
---

## Summary

Ticket: IR-1578. Reviewed exact head `bd876f8f8da5fc7fc5fde706283ad8cf14750976` against `3163f15c04a11bc9d6871cb74d8fc2e561fe607e`. Rust review lane included. No source edits or local builds/tests.

## Verdict

**PASS** — no defects found in the changed projection path or its focused regression test. Titan exact-head evidence reports the focused test, target compilation, and workspace check passed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

The source-owned enum branch now invokes `project_exact_type` within `ctx.exact_read`, sharing the `NativeJoinBudget`; it returns a typed refusal if `source_enum_view()` is unavailable. The test supplies the authenticated source enum row, checks success and positive charges, repeat/clone equality, exact measured limits, one-below work/storage refusal, and missing-row `UnavailableIdentityWitness` without a success delta. The branch remains restricted to source-owned enum scalar declarations; all other unsupported types retain their refusal route.

Examined criteria:

- `FR-354-AC-6` (`spec/checked_package/functional/FR-354-project-admitted-runtime-context.md`): “With FSL-stage enum declaration and member rows, an ordered and an unordered selected enum each yield exact case order, member identity and position sufficient for `EnumMemberIndex`; absent, duplicate, mismatched-case or foreign member witnesses refuse before any partial index.” The new test links this acceptance criterion; the full FSL enum index remains downstream scope.
- `FR-360-AC-4` (`spec/checked_package/functional/FR-360-charge-runtime-type-projection-with-shared-budget.md`): “At sufficient limits, representative source and model runtime type reads, selected nominal reads, and selected unit-definition reads return the same complete values, original identities, order, and semantic refusal causes as the existing behavior. Equal input, rows, and limits produce equal values, cumulative charges, deltas, or first refusals for the original and a clone of the admitted package.” The new regression checks source-enum success, clone determinism, and budget boundaries.

Titan exact-head receipts supplied by the lead: `cargo test --locked -p filament-contract-ir --features conformance-cli --test it --no-run` passed; the exact focused test matched and passed one test; `cargo check --locked --workspace` passed. I did not run builds/tests locally. Repository is public; findings were not posted to GitHub.
