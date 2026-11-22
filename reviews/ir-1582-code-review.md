---
id: SR-967175026302197768900474181941935088
title: "Code review — Contract IR requirement-record reachability"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; IR-1582; crates/filament-contract-model/src/checked_package/v2/requirement_records.rs, tests/conformance_fspec/requirement_records.rs, FR-376-AC-3"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir", type: references }
---

## Summary

Reviewed PR #1132 at `05e17dceb096091ccbf733d02594ff50863527ec` against FR-376-AC-3, its owner-reach requirement, and the changed validator and scale fixture. The code change preserves validation order while replacing linear membership scans with deterministic ordered sets; no applicable AssuranceProfile was found. Titan compile-only, the focused requirement-record tests, and the ignored 100,000-record test passed; formatting passed. `cargo deny check` reports a pre-existing RustSec advisory in dev-only `url` 2.5.8; the workspace check remains pending.

## Verdict

**CONDITIONAL** — no defect was found in the changed code; the required deny gate reports the existing dev-dependency advisory RUSTSEC-2026-0592.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `cargo deny check` fails because dev-only `jsonschema` resolves vulnerable `url` 2.5.8 (RUSTSEC-2026-0592); this is pre-existing and outside the PR diff | Cargo.lock:183, Cargo.toml:127 |

## Scope

- FR-376, role: context_only. Excerpt: “The owner reachability of rules 4 and 8 SHALL walk each owner function's body once, from its body root and not its measure, and reuse the reached set for every record of that owner.”
- FR-376-AC-3, role: examined. Excerpt: “and an owner with two records is walked once (its reach visits are charged once).”
- Inspected the production diff in `requirement_records.rs` and the changed fixture/test in `tests/conformance_fspec/requirement_records.rs`; the matching `TC-488` traces are correct.

## Gate Results

- `cargo fmt --all -- --check`: passed.
- Titan exact-head compile-only: passed.
- Titan `requirement_records::` filter: 18 passed; one documented origin-main known red was skipped; one ignored test.
- Titan exact ignored 100,000-record test: passed in 239.92 seconds.
- `cargo deny check` with a writable temporary Cargo home: failed on RUSTSEC-2026-0592 (`url` 2.5.8 via dev-only `jsonschema`); `bans`, `licenses`, and `sources` passed.
- Workspace check not run by reviewer; it is pending the lead's post-review Titan gate. No local builds or tests were run.
