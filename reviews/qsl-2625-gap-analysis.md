---
id: SR-2026112005294500000002
title: "gap-analysis of QSL-2625 contract-ir body_locations reader"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1088 at d0dc990983c032e3f5f5a748db7712742378fc60; FR-038-AC-581, FR-019-AC-29, FR-051-AC-5, TC-491; tests/it/checked_package_v2_operand_families.rs, tests/it/checked_package_v2_term_data_writer.rs, tests/it/public_interface.rs"
review_set: subset
---

## Summary

Ticket: QSL-2625. Manual check of acceptance criteria against tests (Quoin matrix not run; no builds per brief). Plan completion: not assessed.

## Examined scope

- FR-019 inventory (examined). The three `public_interface` tests compare FR-019's table with the crate's exports; the three listed types now exist and are re-exported from `lib.rs`, so they pass by construction.
- FR-038-AC-581 (examined, partly covered). `body_location_rows_are_read_and_refused_at_their_own_pointers` runs over the Integer-fold fixture with rows from `rebuild_body_locations`. It covers the missing row, `name`, `index`, a non-function `function`, a row beyond the count, an unknown occurrence and the absent carrier.
- FR-019-AC-29, FR-051-AC-5 (examined, not covered). No test builds a typed row, encodes it and reads it back.
- `rebuild_body_locations` (examined). It reimplements FR-322's wire walk over JSON on its own; it does not call or copy the reader. It always writes path `[]` and the node's first `expression` occurrence.

## Verdict

The reader is mergeable for the lint fix once SR-2026112005294500000001 FND-001 is fixed. FND-001 below is not blocking for this PR, but TC-491 must not be marked done until it is closed: the matrix row correctly still reads Planned.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The test traces FR-038-AC-581 but does not read FSpec's BL vectors, and leaves the AC's distinguishing cases untested: BL-02 (captured path kept and returned by the typed read), BL-03 and BL-03 without its occurrence-1 row (a node reached twice is counted twice), BL-R2 (a `value` node as `application`), two functions sharing one `index`, a `type_declaration` origin `kind` (`malformed_wire` at `kind`), and a present carrier without `body_locations`. Every fixture row has path `[]` and occurrence 0, so a reader that counted a revisit once, or walked `dependencies`, would pass. After FSpec #765 merges, add `tests/conformance_fspec/body_locations.rs` that reads the BL vectors by reference, as `protocol_locations.rs` does | tests/it/checked_package_v2_operand_families.rs:2037 |
| FND-002 | medium | FR-019-AC-29 and FR-051-AC-5 have no test: nothing constructs a `CheckedBodySemanticLocation` from typed values, encodes it with `encode_checked_package_v2`, and checks that the strict read returns the same values. The writer tests only add an empty `body_locations` array. Add a round-trip test in `checked_package_v2_term_data_writer.rs` with a non-empty row (BL-01-shaped) | tests/it/checked_package_v2_term_data_writer.rs:67 |
