---
id: SR-967175026302197768900474181941935084781
title: "code-review and rust-review of QSL-3359 location ordinal grouping"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1193 at 2f7abfa4 (base b7854d33); crates/filament-contract-model/src/checked_package/v2/{protocol_locations.rs,body_locations.rs}; tests/it/support/checked_package.rs"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in. Ticket: QSL-3359. Contract: FSpec FR-322 (body_locations and protocol_locations prose, FR-322-AC-291) at origin/main, contract-ir FR-038, arch ruling QSL-3498 comment 440767b3.

Commands: cargo fmt --all --check (clean); cargo clippy -p filament-contract-model --all-targets -D warnings (clean); lib filter body_locations and protocol_locations (9 passed); conformance_fspec body_locations, protocol_locations, location_scaling (passed); it checked_package_v2_term_data_writer (passed); it checked_package_v2_body_walk and checked_package_v2_operand_families and conformance_fspec requirement_records (9 failures, all pass with the base reader). cargo deny and whole suites not run. FSL writer read at filament-spec-language origin/main a5ab2593, not executed.

## Examined scope

- protocol_locations.rs validate_rows: the duplicate and consecutive keys are (application, occurrence, term_ordinal) and (application, occurrence). This is FR-322 for both arrays. Row-level refusal order and the array-level order of FR-038 are unchanged.
- body_locations.rs validate: pairing slots stay keyed (function, application) in array order, so rank in (occurrence, term_ordinal) stays per function. body_location_read.rs filters by function and sorts by (application, row index), also correct.
- A temporary probe through body_locations::validate with two functions: ordinals 0,1 and the reverse assignment admit; 0,0 refuses conflicting-definition at row 1; 0,2 refuses at the array. The base reader gave the opposite on the first three. Probe reverted.
- FSL fsl-package emit/body_locations.rs and protocol_locations.rs ordered: the final pass numbers per (application, occurrence) across all functions, origins by name then visit, so each function's rank order is its walk order. It agrees with the new reader.
- FSpec published vectors: no body_locations vector shares an (application, occurrence) between two functions.

## Verdict

Not mergeable. The production change is correct against FR-322. The in-repo test producer still writes the old per-function numbering, and 9 tests fail.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | rebuild_body_locations still restarts term_ordinal per function, and two body_walk tests assert that rule; 9 tests that pass on base now fail with conflicting-definition or the wrong locus | tests/it/support/checked_package.rs:2831-2834 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 3199f6a3 declares visits once across function walks and rewrites the two body_walk tests for package-wide ordinals |
