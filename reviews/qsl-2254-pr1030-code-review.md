---
id: SR-17940486709564847048032
title: "code-review and rust-review of QSL-2254 timed ClockConfiguration kind"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1030 at 68d9732e (base origin/main 8eb5201f); crates/filament-contract-model/src/checked_package/v2/{intake.rs,mod.rs,monitor_plan.rs,temporal_binding.rs,timed_model.rs}, tests/conformance_fspec/timed_clock_no_row.rs, tests/it/checked_package_v2_temporal.rs, tests/it/support/checked_package.rs"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in, for filament-contract-ir PR #1030. Ticket: QSL-2254. References (read only): filament-specification origin/main 959a1982, FR-322 "Temporal clock bindings" and `proposals/checked-package-v2/timed-clock-no-row-vectors.json` (version 2, TN-01..TN-09).

Whole gate rerun at 68d9732e, CARGO_TARGET_DIR in the review worktree, `--locked`, serialized on `/tmp/cargo-slot.lock`: `cargo fmt --all -- --check` clean; `cargo clippy --locked --workspace --all-targets --features conformance-cli -- -D warnings` clean; `cargo deny check` ok (advisories, bans, licenses, sources); `quire validate --scope . 'spec/**/*.md' 'plan/**/*.md' 'reviews/**/*.md' --summary` exit 0; `make string-edge` clean with `fsl-string-edge` installed as CI does (`cargo install --locked --git .../filament-spec-language.git fsl-string-edge`, at 40f224a4, into a scratch root on PATH); whole `cargo test --locked --workspace --features conformance-cli --no-fail-fast`: only the 2 `lock_coherence` failures (`quire check-lock` subcommand absent from the installed quire: environment), plus 5 `string_edge_gate` failures caused only by the scanner not being on PATH for that run, which pass (6/6) when rerun with it on PATH; `cargo test --locked --features conformance-cli --test conformance_fspec -- --test-threads=1`: 444 passed, 1 failed (`model_tuple_union::published_model_tuple_union_intake_cases_refuse_at_exact_member`, TK-08, not touched by this diff). Cargo.lock unchanged.

## Examined scope

- temporal_binding.rs `ClockConfiguration::Timed { unit }` (examined): serialize via the derived internally tagged form, deserialize via both the map and enum visitor paths with `deny_unknown_fields`; `validate_shape` refuses an empty unit at `/clock/unit`. The closed-record storage test includes the new kind.
- temporal_binding.rs `configured_profile(position, model_subject, ..)` and the coverage loop (examined): a timed clause is clock-configured only over a trace subject, so a missing trace row refuses at the clause body (TN-02).
- temporal_binding.rs first-pass refusal of a row naming a timed clause (examined): now conditioned on `is_model_subject_clause`, so TN-04/TN-07 still refuse at the whole row.
- temporal_binding.rs per-row check (examined): after the `clock_name` comparison and before `clock_matches`, a timed trace row whose `clock` is not `timed` or whose name is `model-time`/`model-steps` refuses at the whole row (TN-08, TN-09). The order matches FR-322 ("compares clock_name before checking the configuration kind").
- timed_model.rs `is_model_subject_clause`, `hybrid_subject`, `model_subject_of` (examined): `clause_model_subject` is a pure refactor onto `model_subject_of`; `validate_models` now skips the `model-time` name check for trace subjects only.
- monitor_plan.rs clock read (examined): `TraceTime` removed; a timed trace clause now reads its `Configured(Timed)` row. See FND-001 for the hybrid case.
- Public API (examined): `CheckedMonitorClock` is not `#[non_exhaustive]`. No consumer of `TraceTime` exists in the workspace or in sibling repos at origin/main (filament-spec-language `fsl-analyze` uses `let ModelTime {..} = clock else`, filament-driver uses only the error types; filament-contract-codegen, -runtime, -integration and others have no use). FR-019's public-interface table lists types, not variants, and needs no change. Crate version is 0.1.0 with git-branch consumers. Removal is appropriate: no compat layer is warranted. Adding `ClockConfiguration::Timed` will require an arm in filament-integration's exhaustive match in `src/temporal/projection/owners.rs` when it next updates (it still depends on the old `quire-contract-model` name); that is the intended spec change, not a defect of this PR.
- intake.rs `invalid_clock_field` (examined): a non-text `timed` unit is now located at `/clock/unit`, as for `fixed_sample`; correct. Its missing test is FND-002 of the gap-analysis SR.
- tests/it fixture helpers (examined): `select_profile` renames the timed clause role to `fixture-clock` and `refresh_temporal_bindings` gives a timed trace clause a `timed` row, so the operator-shape tests keep testing operator shape under the new row rule. The TN cases that need no row (TN-02) remove the row explicitly, so no case is hidden.
- Structural (examined): `#[trace]` tags on the new and changed tests; the deprecated `/// Trace:` line on the stub was dropped; no pins, compat layers or tool versions.

## Verdict

Mergeable, with findings. The serde form, the closed-shape refusals, the trace-subject row requirement and the whole-row refusals are implemented as FR-322 and the TN vectors state, and mutation of each check makes a test fail (see the gap-analysis SR). One non-blocking finding: the hybrid-subject monitor clock read changed without a statement or test.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A timed clause over a hybrid-model subject now has no row (is_model_subject_clause counts hybrid) and no ModelTime join (admitted_model_time resolves only Model/Reference kinds), so clock_for_claim on it returns MissingClock where it returned TraceTime before; no spec text or test covers it | crates/filament-contract-model/src/checked_package/v2/monitor_plan.rs:466-488; crates/filament-contract-model/src/checked_package/v2/timed_model.rs:2713-2783,2800-2826 |
