---
id: SR-17191
title: "QSL-2985 PR 1148 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir#1148; tests/conformance_fspec/probabilistic_claims.rs; crates/filament-contract-model/src/checked_package/v2/mod.rs (context)"
review_set: subset
---

# SR-17191: QSL-2985 PR 1148 code and Rust review

## Summary

Ticket: QSL-2985. Head d4bff7828f8e51c969e44e3a3a5eb713071e3519 over
origin/main d5b1858c. The PR changes only the conformance test
`claim_row_coverage_and_exact_work_boundary` and the FR-038-AC-394 row; the
reader is unchanged. Rust lane folded in.

Root cause verified: `validate` in `checked_package/v2/mod.rs` calls
`models.charge_selected_owner_visits(&mut meter)` after
`dispatch_projection::validate_checked_term_data`. It was added deliberately in
#1040 (commits "Restore checked package owner budget order" and "Fix selected
owner work charge ordering"), and FR-038-AC-144 requires it: "M - 1 returns
`incomplete` at the final selected-owner charge (`/semantic_graph/nodes/<i>/owner`),
which follows graph validation". FR-038-AC-30 says the same. The reader is
correct and the old AC-394 locus was stale.

Test soundness. `exhausted(low - 1)` must equal the owner locus before the
scan, so the span cannot be empty. The scan walks down while the locus stays at
the owner. It then requires the next lower limit to hit claim row 1 and the one
after that to hit claim row 0. Each step asserts `limit_kind == Work` and
`consumed == limit + 1`. The test fails if the reader goes back to charging the
sidecar last (first assert), if the owner visits move before the sidecar (first
assert), if a stage is put between the owner visits and the claim rows (row-1
assert), or if a row charges more than one unit (row-0 assert). The u64
decrement cannot wrap silently, because a debug-build underflow panics.
"No partial typed sidecar view" holds by construction:
`CheckedPackageIncomplete` carries only `limit_kind`, `limit`, `consumed` and
`path`, and the test matches the `Incomplete` variant.

Run on titan, in the coder's worktree, whose tree is byte-identical to the PR
head: `cargo test --locked --features conformance-cli,test-support --test
conformance_fspec probabilistic_claims` gave 5 passed against FSpec 7e9600df,
and 5 passed against the ~/dev/filament-specification checkout at 57c662b1.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | `assert!(below_visits < low - 1, "the owner visits are charged")` can never fail: the preceding `assert_eq!(exhausted(low - 1), owner_visits)` already forces at least one loop step. The real guard is the assert_eq; drop the dead assert or fold its message into the assert_eq. | tests/conformance_fspec/probabilistic_claims.rs:435 |

## Verdict

PASS with one low nit. The diagnosis is correct and the test is a sound, exact
and non-vacuous boundary probe. Mergeable.
