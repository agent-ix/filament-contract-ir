---
id: SR-17940486709564847048082
title: "Code review — filament-contract-ir QSL-2574 await fixture refresh"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; tests/it/checked_package_v2_protocol_body.rs, tests/it/support/checked_package.rs, FR-038-AC-414 and FR-038-AC-415"
review_set: subset
---

## Summary

Ticket: QSL-2574. Reviewed the fixture refresh and await clock-row regression test. The test exercises the expected admission and refusal paths, but the change adds a forwarding `settle` helper even though the same helper already exists in test support.

## Verdict

**FAIL** — the new helper duplicates an existing function, violating the repository's no-duplication rule.

## Scope

- **examined — FR-038-AC-414**, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`: “A missing required compensation or await row (`CB-03` / `AW-03`) refuses `invalid_package`/`timed-node-reference` at that node's `/semantic_graph/nodes/{N}/body`… A valid await row is admitted and round-trips.”
- **examined — FR-038-AC-415**, same file: “For a resolved compensation or await row, a changed `clock_name` (`CB-05` / `AW-05`) refuses `invalid_package`/`timed-node-reference` at `/temporal_bindings/{i}/clock_name`…”.
- **examined — changed code**, `tests/it/checked_package_v2_protocol_body.rs` and `tests/it/support/checked_package.rs`; traced fixture construction through `refresh_identity` into `refresh_temporal_bindings`.
- **context only — repository convention**, `AGENTS.md` / `CLAUDE.md` and the reviewer no-duplication rule.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The new local `settle` wrapper only forwards to the existing `support::checked_package::settle`, which this file previously imported directly. Remove the wrapper and restore the import; this is a duplicate helper under the P0 no-duplication rule. | tests/it/checked_package_v2_protocol_body.rs:32; tests/it/support/checked_package.rs:2463 |

## Gate evidence

- `cargo fmt --all -- --check`: passed.
- `git diff --check origin/main...HEAD`: passed.
- Targeted tests on this exact head, per the run logs: 13/13 await-filter tests passed; `refinement_catalog_joins_timeout_on_await_node` passed 1/1. Logs: `/tmp/qsl2574-evidence/qsl-2574-final-await-broad.log`, `/tmp/qsl2574-evidence/qsl-2574-refinement-await.log`.
- GitHub Rust Checks were pending at review time; Supply-chain Checks and CLA gate passed.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The wrapper is removed and the shared helper is imported directly; after excerpt: `rename_node, select_model, selected_interface_model_node, settle, source_owner, structural_id,`. |

## Disposition pass 1 verification

Inspected PR head `1e019bd3084a73da3cbb506e5d7fc192d8703597`: the import at `tests/it/checked_package_v2_protocol_body.rs:12` includes `settle`, and no local `fn settle` remains. The finding is fixed.
