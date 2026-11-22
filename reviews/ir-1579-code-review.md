---
id: SR-967175026302197768900474181935084588
title: "Code review — IR-1579 node-index dependency rows"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR #1130 head 93a3eb1c453ee728d47eb371bb9091acc10f9030; crates/filament-contract-model/src/checked_package/v2/node_index.rs; tests/conformance_fspec/node_index.rs; tests/it/support/fspec_inputs.rs; FR-038-AC-521, FR-038-AC-539, FR-038-AC-540; FSpec FR-322-AC-521"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir", type: references }
---

## Summary

Reviewed PR #1130 at `93a3eb1c453ee728d47eb371bb9091acc10f9030` against the local operand-row criteria and FSpec FR-322-AC-521. The reader now rejects non-composition dependency rows and dependency positions absent from direct lock selections, preserves NI-39 placement precedence, and the exact published 32-vector test passes.

## Verdict

**PASS** — the only review finding was a rustfmt failure on the prior head; it was fixed, and formatting plus Titan's exact-head qualification pass.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `cargo fmt --check` failed on import ordering in the three changed Rust files at the pre-format head `e26d26ea`; the final head passes after the formatting-only commit. | crates/filament-contract-model/src/checked_package/v2/node_index.rs:7; tests/conformance_fspec/node_index.rs:5; tests/it/support/fspec_inputs.rs:12 |

## Review scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-038-AC-521 (`spec/checked_package/functional/FR-038-consume-checked-package-v2.md`) | examined | The strict reader requires operand rows and applies the published node-index vectors in the stated check order. |
| FR-038-AC-539 (`spec/checked_package/functional/FR-038-consume-checked-package-v2.md`) | examined | A `component_composition` row may name a supplied dependency node through its own operand. |
| FR-038-AC-540 (`spec/checked_package/functional/FR-038-consume-checked-package-v2.md`) | examined | Invalid dependency positions and dependency objects in path segments refuse at the row or segment. |
| FSpec FR-322-AC-521 at `6a7611d8003de1cdc45e3e427495376afc5fb0b5` | examined | NI-38 refuses a dependency row under node 3; NI-39 reports node 4's own operand slot before its dependency row defect; NI-40 refuses a supplied but unlocked package. |
| TC-048 `published_operand_vectors_refuse_in_the_fr322_check_order` | examined | The test asserts the closed 32-case set and checks each vector's outcome, cause and exact locus. |
| IR-1579 acceptance | examined | Enforce NI-38 and NI-40 in the checked-package reader and run the full published vector set. |

## Review methods

- Code and Rust review covered the diff, traversal scopes, budgeted site collection, owner/direct-lock checks, refusal precedence, test setup, trace tags, and assertion strength. No production-code correctness or safety finding remained at the final head.
- `cargo fmt --check` passes at `93a3eb1c`; `git diff --check` passes. The earlier formatting failure is retained as FND-001 and its disposition is recorded below.
- No local build or tests were run. Titan qualified exact head `93a3eb1c453ee728d47eb371bb9091acc10f9030` with: compile-only `cargo test --locked -p filament-contract-ir --features test-support --test conformance_fspec --no-run` (exit 0); exact named test `node_index::published_operand_vectors_refuse_in_the_fr322_check_order` (1 passed, all 32 vectors including NI-38/39/40); and `cargo check --locked --workspace` (exit 0). Titan used `CARGO_TARGET_DIR=/tmp/titan-ir1579-cargo-target`, FSpec head `6a7611d8003de1cdc45e3e427495376afc5fb0b5`, and logs `/tmp/titan-ir1579-93a3eb1c-logs/` on Titan.
- No CI workflow files changed. No applicable AssuranceProfile was found under `spec/`.

## Dispositions

Round 1. Rechecked the final head `93a3eb1c453ee728d47eb371bb9091acc10f9030`, which adds only rustfmt output over the qualified code/test head. `cargo fmt --check` passes.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Commit `93a3eb1c` applies workspace rustfmt to all three changed Rust files; the final formatting check passes. |
