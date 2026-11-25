---
id: SR-2026110602354625
title: "QSL-2958 PR #1156 code review — contract-model semantic-value pin"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR #1156; crates/filament-contract-model/Cargo.toml; Cargo.lock; QSL-2958"
review_set: subset
---

## Summary

Reviewed frozen PR head `2acdf53bb30ec0c5d62fcd574088af6b41c89580`. It changes only the existing `filament-semantic-value` revision in the contract-model manifest and its corresponding lockfile source; no implementation, tests, or requirement artifacts changed. No applicable AssuranceProfile was found under `spec/`.

## Verdict

**PASS** — both manifest and lock select the merged semantic-value code revision `689f259eab592b62cb362f5e649dba16adf874f7`; the dependency tree contains one semantic-value package through `filament-contract-model` and the filtered test passes. No defect was found in the PR diff.

## Coverage

- `crates/filament-contract-model/Cargo.toml` (examined): the existing semantic-value dependency changes from `4274b4eb4abeae4ccd457a6de681625acb8fc57d` to `689f259eab592b62cb362f5e649dba16adf874f7`.
- `Cargo.lock` (examined): the sole semantic-value package source changes to the same exact revision.
- No acceptance-criterion artifact or test code is changed by this dependency-only PR; AC coverage and trace bindings were not in scope.

## Verification

- `cargo tree --workspace -e normal -i filament-semantic-value`: passed; one semantic-value package at `689f259e` is selected through `filament-contract-model`.
- `cargo check --workspace -j 4`: passed.
- `cargo test --workspace public_term_deserialization_retains_node_ids_from_text_and_buffered_values -j 4`: passed, 1 test; other tests filtered.
- `git diff --check origin/main...HEAD`: passed.
- `cargo fmt --all -- --check`: failed on pre-existing formatting in the untouched `crates/filament-contract-model/src/lib.rs` export list; this file is byte-identical to `origin/main`.
- `cargo clippy --workspace --all-targets --all-features -j 4 -- -D warnings`: failed on warnings in unchanged model/test files (unused imports/variable, `items_after_test_module`, and `useless_conversion`).
- `cargo deny check`: failed on the existing `url 2.5.8` advisory RUSTSEC-2026-0592 and unmatched `ix-cli-kit` source warning; neither dependency is changed by this diff.
- `gh pr view`: PR is OPEN and MERGEABLE at the reviewed head.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
