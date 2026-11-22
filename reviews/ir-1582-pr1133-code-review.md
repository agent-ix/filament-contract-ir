---
id: SR-967175026302197768900474181941935084588
title: "Code review — IR-1582 PR #1133 named-record index"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; IR-1582; PR #1133; crates/filament-contract-model/src/checked_package/v2/requirement_records.rs and tests/conformance_fspec/requirement_records.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir", type: references }
---

## Summary

Reviewed PR #1133 at `1b075fe3902652fcbb83f63e711834efc25b1854` against FR-376-AC-3 and its 100,000-record test. The production change replaces both repeated named-record scans with a deterministic `BTreeSet<usize>` and preserves the first missing record's structural selection and refusal path. No change to the owner-group sort is needed; it remains an O(n log n) sort with comparator work over node keys. Titan exact-head compile-only, focused tests, and scale test passed; formatting passed. The 15-minute scale threshold does not detect the specific quadratic named-record scan this PR fixes. `cargo deny check` also remains red on a pre-existing dev dependency advisory.

## Verdict

**CONDITIONAL** — the implementation is correct, but the acceptance test would still pass with the quadratic naming scan present, and `cargo deny check` reports a pre-existing advisory.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The 100,000-record test does not detect a regression to the quadratic named-record scan | tests/conformance_fspec/requirement_records.rs:677-696 |
| FND-002 | medium | `cargo deny check` fails on pre-existing dev-only `url` 2.5.8 (RUSTSEC-2026-0592) | Cargo.lock:183, Cargo.toml:127 |

## Scope

- FR-376, role: context_only. Excerpt: “When a V2 checked package carries `requirement_items` rows or `claim`/`value_validity` nodes, the Contract IR reader SHALL admit exactly FSpec FR-825's rows, node body and joins, or refuse the whole package at the first defect in FR-825's reader order.”
- FR-376-AC-3, role: examined. Excerpt: “and an owner with two records is walked once (its reach visits are charged once).”
- Inspected the PR diff in `requirement_records.rs`, the unchanged TC-488 scale assertion, and FR-376-AC-3's bindings. The set conversion preserves the previous first missing record selection because it still scans `records` in the same structural order and tests set membership before constructing the same refusal.
- No applicable AssuranceProfile was found.

## Gate Results

- `cargo fmt --all -- --check`: passed.
- Titan exact-head compile-only: passed at `1b075fe3902652fcbb83f63e711834efc25b1854`.
- Titan requirement-record module filter: 18 passed; one documented origin-main known red skipped; one ignored scale test.
- Titan exact ignored 100,000-record test: passed in 224.81 seconds.
- The prior head `05e17dceb096091ccbf733d02594ff50863527ec`, which still had the nested named-record scans, passed the same test in 239.92 seconds. The current limit is 900 seconds; the test's WorkMeter bound also passed on both heads because container comparisons are not charged to it.
- `cargo deny check` with a writable temporary Cargo home: failed on RUSTSEC-2026-0592 (`url` 2.5.8 through dev-only `jsonschema`); bans, licenses, and sources passed.
- Workspace check was not run by reviewer; it is pending the lead's gate. No builds or tests were run locally.

## Finding Details

### FND-001

The 100,000-record test asserts admission, a 25.6-million WorkMeter cap, and a 15-minute wall-clock limit. The previous exact head, with the two earlier reach-membership fixes but both named-record `.iter().any()` scans still present, completed in 239.92 seconds. The current indexed head completed in 224.81 seconds. Thus the test passes when this quadratic path is present, so neither its work bound nor time bound guards this optimization. A comparison-counting or other bound that exercises named-record matching is needed if the gate must fail when this path regresses.

### FND-002

The changed PR does not alter dependencies. The security gate fails on `url` 2.5.8 via dev-only `jsonschema`; this is pre-existing and outside the PR diff.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The changed test exercises `first_unnamed_position` on 100,000 entries and bounds each production helper comparison through its callback; a linear-scan implementation that reports its comparisons exceeds the bound. |
| FND-002 | deferred | The advisory remains in unchanged Cargo.lock through dev-only jsonschema; dependency remediation is outside PR #1133. |
