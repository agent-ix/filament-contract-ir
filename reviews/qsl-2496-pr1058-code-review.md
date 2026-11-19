---
id: SR-2026111905212900000001
title: "code-review and rust-review of QSL-2496/QSL-2503 per-declaration claim names, body wrapper, queue capacity 0"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1058 at 31e982b2 (base origin/main 9edbd391); crates/filament-contract-model/src/checked_package/v2/{claim_names.rs,mod.rs,temporal_clause_names.rs,protocol_body.rs}, tests/it/{checked_package_v2_claim_names.rs,checked_package_v2_temporal.rs,checked_package_v2_protocol_body.rs,checked_package_v2_frame_entries.rs,support/checked_package.rs}, tests/conformance_fspec/claim_names.rs"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in, for filament-contract-ir PR #1058. Ticket: QSL-2496 (also QSL-2410, QSL-2503). Reference: a private FSpec worktree at origin/main 9a181ace (contains W58, #717 57574468), FR-322 "Rows of one declaration", "Claim declaration names" steps 1-6 and the own-node pass, "Temporal clause names", "One claim namespace", FR-322-AC-465/466, and `proposals/checked-package-v2/claim-name-vectors.json` (CL-P7, CL-P8, CL-R16, CL-R17, CL-R18).

Commands run: `cargo fmt --all --check` (clean); `make string-edge` (clean; removing the `#[ix_trace_rs::string_edge]` marker on `body_wrappers` makes it fail at claim_names.rs:359, so the marker is required). A probe assertion added to `renaming_a_claim_changes_no_node_and_one_node_carries_two_occurrence_rows` in a scratch worktree (`cargo test --features conformance-cli --test it -- renaming_a_claim_changes_no_node`) failed: `claim_name_for_path(["B"])` returned the row named `A` (FND-001). Clippy, the full workspace test run and the mutant sweep were queued behind a saturated cargo-slot and had not run when this verdict was reached; they belong to the disposition pass.

## Examined scope

- claim_names.rs `validate` steps 5 and 6 (examined). The declaration key is (`source`, `start`, `end`) of `declaration_region`. A row whose key matches an earlier row but whose `name` or `qualified_name` differs refuses `invalid-value` at `/claim_names/{i}/name`. A repeated `qualified_name` is ambiguous only when the earlier row has a different key. Both checks run after the key-repeat and order checks, in FSpec's order. Comparing with the first row of the declaration is equivalent to comparing with any earlier row, because an earlier row that differed would already have refused.
- claim_names.rs `own_nodes` (examined). Declaration sets cover all of a node's rows, not only the rows of the name. The unique smallest set must be a subset of every other set. A tie at the smallest size has no own node. A unique smallest set that is a subset of a larger set is a strict subset. Names are visited in order of their first row. The refusal is at the last row's `qualified_name` and names the first row. This matches FR-322.
- Ordering: own node before one namespace (examined). FR-322 places the own-node pass inside "Claim declaration names", after coverage. "One claim namespace" runs "once both members have passed their own row checks". So the coder's order follows the FSpec text.
- `body_wrappers` and temporal_clause_names join and coverage skip (examined). A wrapper is the `temporal_clause` target of a hyperproperty `body` binding, as in FR-322 "Temporal clause names". A row naming a wrapper refuses `invalid-value` at its `declaration`. The coverage walk skips wrappers. `temporal_bindings` is untouched, so hyper rows keyed on a body wrapper stay with #1051.
- protocol_body.rs queue capacity (examined): `< 0` refuses, so 0 is admitted. This matches FSpec TC-947 QU3/QU7 and FR-465 "non-negative integer". A negative capacity still refuses as before.
- Work charge in `own_nodes` (examined): 1 per row for the node sets, 1 per row for grouping, and `inner + outer` per node pair compared. FSpec is silent on this charge. The IR-local wording in FR-371 describes the code. No test measures it (gap-analysis SR).
- Test helper `fill_declaration_regions` widening (examined). It widens a later, differently named row's region only while it equals an earlier row's region. This models two distinct declarations. Refusals built by renaming after the fill keep their regions equal and still refuse (CL-R16 replay, `the_rows_of_one_declaration_share_its_name_and_are_no_duplicates`). The published vectors do not use the helper. No masked defect was found.
- `claim_name_for_path` (examined): FND-001.

## Verdict

Not mergeable. The reader's validation matches FSpec W58. The lookup regresses, though: on a node that carries two structurally equal declarations, a name can resolve to the other declaration's row. The fix is local: filter the own node's rows by the requested path (or declaration key), and add the lookup assertion to the twin-clause test.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `claim_name_for_path` picks the own node and then returns the first `claim` row on that node, whatever declaration it belongs to. When the own node carries two structurally equal declarations (FR-371-AC-1 twins `A` and `B`), the lookup of `B` returns `A`'s row. Reproduced. On main this lookup returned the correct row. | crates/filament-contract-model/src/checked_package/v2/mod.rs:2042-2045 |

## Dispositions

Round 1, head f3ab09af (fix abbfb0fc; merge of origin/main e0327d7c). Checks: `cargo fmt --all --check` clean; `make string-edge` clean; `cargo clippy --locked --workspace --all-targets --features conformance-cli -- -D warnings` clean; `--test it` filtered to claim_names, temporal and protocol_body: 304 passed; `--test conformance_fspec claim_names`: 4 passed, with FILAMENT_SPECIFICATION_DIR at FSpec 9a181ace.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `on_own` now filters `&row.declaration == own && carries(row)`. The twin test asserts that each twin's path resolves to its own row, and that `["B"]` resolves to ordinal 1. That assertion is the reviewer's probe, which failed on 31e982b2. |
