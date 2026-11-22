---
id: SR-967175026302197768900474181941935086
title: "Code review — QSL-2785 requirement rows"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; crates/filament-contract-model/src/checked_package/v2/requirement_records.rs, tests/conformance_fspec/requirement_records.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-376", type: references }
---

## Summary

Reviewed PR #1127 at exact head `dc83cb40ae270ab76a2f0c2abe26944e0926756b`, including the Rust implementation and tests against FR-376. Titan supplied exact-head focused test and workspace-check evidence; no local build or test was run.

## Verdict

**FAIL** — the new production helper duplicates operand-slot resolution, the budget test no longer protects FR-376-AC-3's required first uncharged pointer, and a new unit test is untraced.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `resolve_holder_id` duplicates the existing operand-slot resolution rule | crates/filament-contract-model/src/checked_package/v2/requirement_records.rs:1061-1077 |
| FND-002 | medium | Budget test permits omission of the required `/requirement_items/1` pointer | tests/conformance_fspec/requirement_records.rs:630-631 |
| FND-003 | low | New holder-row unit test has no trace binding | crates/filament-contract-model/src/checked_package/v2/requirement_records.rs:1226-1268 |

## Coverage

Reviewed all changed files: `crates/filament-contract-model/src/checked_package/v2/mod.rs`, `crates/filament-contract-model/src/checked_package/v2/requirement_records.rs`, and `tests/conformance_fspec/requirement_records.rs`. Compared public row behavior and its test bindings with FR-376/FR-376-AC-1 through AC-3 and inspected existing operand-slot conversions in `term_view.rs` and `node_index.rs`.

Titan evidence supplied by dispatch: focused exact-head tests passed (four named tests); `cargo check --locked --workspace` passed. Initial featureless conformance invocations failed to compile because the test-support feature was required; corrected feature-enabled invocations passed. Paired FSL consumer failures at separate FSL heads are scoped to QSL-2807 producer work and are not treated as an IR defect.

## Verdict details

FND-001: `term_view.rs` and `node_index.rs` already convert holder-row operand slots into resolved graph indices or dependency IDs. The new helper repeats that same rule; a change to this mapping would require editing multiple implementations.

FND-002: FR-376-AC-3 requires a one-visit-short read to return `incomplete` at `/requirement_items/1`. The revised assertions accept `path: None` and any consumption below the full count, so this required behavior can regress while the test remains green.

FND-003: `value_record_references_resolve_through_the_actual_holder_row` checks the new record-reference mapping but has no `#[trace]` annotation for its owning criterion.
