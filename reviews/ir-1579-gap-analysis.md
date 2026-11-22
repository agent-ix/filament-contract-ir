---
id: SR-967175026302197768900474181935084589
title: "Gap analysis — IR-1579 node-index dependency rows"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; repository matrix and coverage; PR #1130 changed behavior; FR-038-AC-521, FR-038-AC-539, FR-038-AC-540; FSpec FR-322-AC-521"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir", type: references }
---

## Summary

This planless audit checked the repository's computed matrix and coverage data, then traced IR-1579's reader behavior and test against the local operand-row criteria and FSpec FR-322-AC-521. The changed behavior is specified and exercised, but the repository matrix has 115 untagged criteria and 308 untracked symbols; the reverse-gap pass was limited to the PR's changed behavior rather than a manual inventory of all public declarations.

## Verdict

**FAIL** — the repository-wide matrix has untagged criteria, and the reverse-gap pass did not inventory every public declaration in the repository.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The computed repository matrix reports 115 untagged criteria, including obligations across FR-019, FR-030, FR-036, FR-038, FR-045, FR-051, FR-052, FR-057, FR-068, FR-069, FR-071, FR-350, FR-352, FR-353, FR-357, FR-358 and NFR-001 through NFR-006. | FR-019-AC-14, FR-019-AC-27, FR-030-AC-7, FR-036-AC-1, FR-038-AC-383, FR-045-AC-1, NFR-006-M-1 |
| FND-002 | medium | Quire coverage reports 308 untracked symbols whose trace IDs do not bind to a declared criterion; examples include stale FR-322-AC-510 and test-only TC-447/TC-473 tags. | crates/filament-contract-model/src/checked_package/v2/body_locations.rs::tests::a_row_rebuilds_its_path_from_the_previous_one; crates/filament-contract-model/src/checked_package/v2/exact_projection.rs::tests::numeric_map_comparison_handle_is_charged_before_its_single_allocation |
| FND-003 | high | The reverse-gap procedure was scoped to IR-1579's changed behavior and did not trace all 1,567 public declaration matches across the 170 Rust source files. | crates/filament-contract-model/src/checked_package/v2/node_index.rs::check |

## Coverage

- Repository: `agent-ix/filament-contract-ir`; spec root `spec/`; source `src/` and `crates/`; tests `tests/`; planless.
- Reconciliation: `quoin matrix --repo . --json` (quoin 0.36.8) and `quire coverage --scope . --json` (quire 0.52.0, engine 0.62.2); no grep fallback.
- Plan completion: not assessed
- Criteria: 1,207 tagged; 115 untagged; 1 tagged only by ignored test; 26 method-without-symbol; 8 retired (1,357 total). The affected local criteria FR-038-AC-521, AC-539 and AC-540 are tagged by TC-048/TC-474.
- Evidence: no run evidence in the matrix. Separately, Titan's exact-head run passed the named 32-vector test and workspace check at `93a3eb1c453ee728d47eb371bb9091acc10f9030`; logs are `/tmp/titan-ir1579-93a3eb1c-logs/` on Titan.
- Coverage diagnostics: 308 untracked symbols and 2,049 diagnostics; these are repository-wide existing scan results. Example reasons include `trace-id-outside-marker`, `uncatalogued-verification-method`, and `section-matches-nothing`.
- Reverse-gap/stub check: IR-1579's dependency-row constraints are owned by the local operand-row requirements and FSpec FR-322-AC-521; the 32-vector test exercises the reader and checks exact refusals. No `todo!`, `unimplemented!`, `TODO`, `FIXME`, or `XXX` markers were found under `src/` or `crates/`. The full public API surface was not manually traced; the source search counted 1,567 public declaration matches across 170 files. Test-stub inspection was limited to the changed test and support helper.
- Semantic review: skipped (not requested).

## Dispositions

Round 1. Compared the matrix and coverage results at the reviewed branch with `origin/main` at `0e41bcda99c0a3317055f9989d60ff0eb4529bc6`. Both sides report 1,207 tagged, 115 untagged, 1 ignored-only, 26 method-without-symbol, 8 retired; 308 untracked symbols; and 2,049 diagnostics. FR-038-AC-521, AC-539 and AC-540 are tagged by TC-048/TC-474 on `origin/main` as well. No IR-1579 change introduces these repository-wide findings.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | deferred | All 115 untagged criteria are present with the same status on `origin/main`; the three IR-1579 criteria remain tagged and the change adds no untagged obligation. This is separate repository-level matrix remediation for the Contract IR spec/evidence owners (`terra:IR:codex`); no single follow-up ticket for this broad set was found. IR-1580 only covers FR-070 verification metadata. |
| FND-002 | deferred | All 308 untracked symbols and 2,049 diagnostics are present with the same counts on `origin/main`; the IR-1579 trace binding is not among the untracked symbols. Route repository-wide stale-tag reconciliation to the Contract IR spec/evidence owners (`terra:IR:codex`) as separate tracked work; no single follow-up ticket for the full set was found. |
| FND-003 | deferred | The 1,567 public declaration matches across 170 Rust files predate IR-1579; this finding records that the review's reverse-gap pass was intentionally limited to the assigned change. A full repository surface audit is separate work for the Contract IR spec/evidence owners (`terra:IR:codex`), not an implementation change in PR #1130. |
