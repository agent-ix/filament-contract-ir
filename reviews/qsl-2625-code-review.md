---
id: SR-2026112005294500000001
title: "code-review (rust-review lane) of QSL-2625 contract-ir body_locations reader"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR #1088 at d0dc990983c032e3f5f5a748db7712742378fc60; crates/filament-contract-model/src/checked_package/v2/{body_locations.rs,checked_term_data.rs,protocol_locations.rs,dispatch_projection.rs,mod.rs}, crates/filament-contract-model/src/lib.rs, tests/it/**, tests/read_storage_alloc.rs"
review_set: subset
---

## Summary

Ticket: QSL-2625. Reviewed the `body_locations` reader, typed rows, encoder and test fixtures against FSpec FR-322 (body_locations paragraph, merged #760) and contract-ir FR-038 / FR-019 (merged #1081). Read only; no builds. The branch was rebased; both commits are patch-identical to the earlier head ed9c2cb1 (`git range-diff`).

## Examined scope

- Reader rules (examined, match). `body_locations::validate` runs the shared `protocol_locations` row rules for every row first (`validate_rows`), then per row: `function` (local function node with a `body` binding), `name`/`index` against the function's first row, shared `index` across two functions, row count beyond the walk's terms at the application; then the missing-predecessor check, then the missing-term check at `/checked_term_data/body_locations`. This is FR-038's stated order. The absent-carrier path calls `validate` with no rows. Unknown members stay `malformed_wire` through `deny_unknown_fields`.
- Body walk (examined, matches FR-322). Pre-order from the `body` binding target, follows `reference` leaves to local `expression` nodes in members/arguments order, revisits counted, `let`/`if`/`case` applications and reference expressions are not terms, callee functions and other packages end a branch. Every visit and every followed reference is charged to the meter.
- Types and encoder (examined). `CheckedBodySemanticLocation`, `CheckedBodyLocation`, `CheckedBodyLocationOrigin` are public through `v2::*` and `lib.rs`, with the FR-019 fields. `CheckedTermDataV2.body_locations` is a required field. The encoder writes the array in the given order, as for `protocol_locations`.
- Tests (examined). No assertion removed, weakened or ignored: the 18 fixture edits add `"body_locations": []` or a call to `rebuild_body_locations`; `refinement::writer_bytes` now passes the fixture's `checked_term_data` instead of `None`.
- CI (examined). Run 44680779771 on this head fails `make lint` at `string-edge`, before clippy runs.

## Verdict

Not mergeable until FND-001 is fixed: the PR's own string-edge lint is red. The reader logic matches the spec. FND-002 and FND-003 can be fixed in the same round.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `make lint` fails at its `string-edge` step on this head (CI run 44680779771): `protocol_locations.rs:93 in CheckedBodySemanticLocation::check_origin (branch-gating)` is unmarked. The protocol counterpart is a free `#[ix_trace_rs::string_edge]` function (`intake_protocol_origin`). Move the `index > MAX_SAFE` check into a marked `intake_body_origin` function and call it from `check_origin` | crates/filament-contract-model/src/checked_package/v2/protocol_locations.rs:83-101 |
| FND-002 | medium | The body origin `name` is admitted when empty and is never charged. The FSpec schema gives `BodyLocationOrigin.name` `Nonempty`, and the protocol origin refuses an empty name (`malformed_wire`) and charges `name.len()`. `body_locations.rs:207` also compares names with no charge. Charge `name.len()` and refuse an empty name as `malformed_wire` in the body origin check, matching `intake_protocol_origin` | crates/filament-contract-model/src/checked_package/v2/protocol_locations.rs:83-101 |
| FND-003 | low | `rebuild_source_map` (and so `settle`) now always calls `rebuild_body_locations`, which replaces any hand-written `body_locations` rows and adds a carrier when there was none. A later test that writes its own rows and then settles loses them without notice. Call it from `settle` only, or keep rows that are already present | tests/it/support/checked_package.rs:2374 |
