---
id: SR-967175026302197768900474181941935084520
title: "code-review and rust-review of QSL-2739 AC10 composition declaration names and member coordinates"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR #1120 at ddfb8854e (base 743b5e5f1); crates/filament-contract-model/src/checked_package/v2/{composition_coordinates.rs,claim_names.rs,declaration_region.rs,guarantee_composition.rs,mod.rs,structural.rs}, tests/it/checked_package_v2_guarantee_composition.rs, tests/it/support/checked_package.rs"
review_set: subset
---

## Summary

Ticket: QSL-2739 (AC10 composition coordinates only). This is the code review with the rust-review lane (and the rust-style defaults) folded into one artifact. The standard is FSpec origin/main, with PR #797 at 73e81dce and PR #798 at 9add8a0f. That covers FR-322 "Composition declaration names" (FR-322-AC-514/515) and FR-491 "Composition member occurrences" (FR-491-AC-7/8), plus the PR #798 ruling that an empty interval is generic `invalid_source_map` at `/source_map/{i}/regions/{j}` before any coordinate check. The reader logic, the check order and the refusal loci match the standard on the main path. There are three medium defects: an unmetered quadratic sorted insert, two untested branches (the end-sharing boundary and the composition occurrence-key check), and a test tagged FR-371-AC-12 that computes its own oracle. There are also five low findings.

## Examined scope

- composition_coordinates.rs `validate`, `strictly_inside` (examined). The node's entries are visited in `source_map` array order, through the positions sorted from `of_node`. The first invalid entry refuses `invalid_package`/`invalid-value` at `/source_map/{i}`. The missing pair refuses at `/source_map` only after every entry passes. The check is strict at both ends, against the same source. No source text is read and the declaration region is never inherited. Shuffled entries do not change attribution. Defects: FND-001, FND-007.
- claim_names.rs (examined). The composition is claim-bearing. The row must be the declaration at 0. Its region must equal the entry region (`is_exactly`). The `qualified_name` is compared with `declaration.qualified_name` after the existing last-segment check. Other correspondence forms are not claim-bearing. Defect FND-003 is the test reach of the new key check.
- declaration_region.rs `is_exactly`, `of_node`, `at` (examined). These borrow the whole entries and keep every region. Lookups are O(log n) over the existing charged key table. The empty and missing cases are deferred to the generic check. No unmetered BTree or new public API.
- guarantee_composition.rs (examined). Guarantees are all checked first, so the FR-490 order is retained. Compositions run shape/identity/order/targets, then the generic source-map closure (once, lazily, at the first composition to reach it), then coordinates, then the static checks 2 to 5. Empty compositions reach the closure. There is no budget reset and no cached bypass. Defects: FND-006, FND-008.
- mod.rs `declaration_forbidden`, the name-index trigger and the closure (examined). Only `component_composition` becomes declaration-eligible. Defect FND-009.
- structural.rs (examined). Composition is removed from the declaration-only role mask. Other correspondence forms are unchanged.
- Global precedence (examined). For a package with a composition that reaches the coordinate step, a package-wide generic source-map defect on any node is now reported before later graph-stage defects. Those later stages are claim_terminal, step relations, hyperproperties and refinement step rows. FR-491 requires this ("After FR-322's generic source-map checks"), and it mirrors the FR-375 refinement precedent. It is recorded here, not raised as a finding. The generic duplicate-key, unknown-node, total-occurrence and source-priority refusals are unchanged.
- Tests (examined). They use literal independent regions with the guarantees authored in reverse canonical order. They cover the reversed source map, write and re-read, and these mutations: out-of-range pair in both, other locked source with refreshed identity, outside, equal, start-sharing, overlap, multi-region, `expression` role in both, missing pair in both, first-invalid in array order, empty-interval precedence, repeated key, order/dependency before coordinates, and coordinates before static checks. Every mutation runs against a positive control. Refusals assert the exact code, cause, full pointer and node locus. Defects: FND-002, FND-003, FND-004, FND-005.
- Composition owner and its precedence (examined). `source_owner` on the fixture's composition is required by FR-322 ("A node with a `declaration` carries a `SourceOwner`"). It is not invented. The reader enforces this through the generic owner-schema rule. owner.rs:76 maps `declaration.is_some()` to `OwnerRequirement::Source`, and `check_one` refuses `malformed_wire` at the node, or at `/owner`, before any graph stage. So a composition with a declaration and no owner, or with an owner and no declaration, is refused by the existing early owner check. There is no composition-specific owner path, no fallback and no lack-of-declaration legacy admit: an old-shape composition with no declaration occurrence is refused by the coordinate check at `/source_map`.
- Gates (examined): `rustfmt --check` on the 8 changed .rs files rc=0. Clippy and full gates were not run (brief override).

Focused receipts (own WT target, `/tmp/ir-dynamic-build`, cpu 18-31, `--locked --features conformance-cli`):
- `--test it checked_package_v2_guarantee_composition`: 29 passed, 0 failed, rc=0.
- `--test it checked_package_v2_claim_names -- --skip published_refinement_body_name_vectors_replay`: 36 passed, rc=0.
- `FILAMENT_SPECIFICATION_DIR=<retained FSPEC-QSL2743-model-unions, vector file identical to 9add8a0f>` `... published_refinement_body_name_vectors_replay --exact`: 1 passed, rc=0.
- `-p filament-contract-model --lib checked_package::v2:: guarantee_composition claim_names declaration structural source_map`: 638 passed, 1 ignored, rc=0.

Mutant M-end: `member.end < declaration.end` weakened to `<=`. It survives by static reasoning: no case has a member end equal to the declaration end with its start inside (FND-002). It was not executed, because the brief allows no code edits.

## Verdict

**CONDITIONAL**: mergeable after FND-001 to FND-004 are fixed in this PR. The low findings should be fixed in the same round.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The disjointness check uses `Vec::insert` into a sorted vector, charging 1 work per entry while shifting O(n) elements. A composition whose member entries are listed by descending start does O(n²) unmetered moves (up to about 5e9 at 100,000 occurrences). The coder report's "no N^2" claim is false. | crates/filament-contract-model/src/checked_package/v2/composition_coordinates.rs:105 |
| FND-002 | medium | No case puts a member's end on the declaration's end with its start inside. A mutant that weakens the end bound to `<=` survives. The equal case is killed by its start alone, and there is no end-sharing control. | tests/it/checked_package_v2_guarantee_composition.rs:1984 |
| FND-003 | medium | The "a claim-role row" case is refused by the existing occurrence join, because the node has no claim occurrence. The new composition key check (role must be the declaration at 0) is never reached, and deleting it leaves every test green. | crates/filament-contract-model/src/checked_package/v2/claim_names.rs:218, tests/it/checked_package_v2_guarantee_composition.rs:1880 |
| FND-004 | medium | `composition_source_order_comes_from_declaration_regions_not_node_ids` is tagged FR-371-AC-12, but AC-12 has no source-order clause. The test sorts the rows itself and asserts its own sort result, so it gives tautological coverage of FR-322-AC-514 source-order selection. | tests/it/checked_package_v2_guarantee_composition.rs:2248 |
| FND-005 | low | The helper `admitted_or_report` and the case label "two distinct names would admit" both assert a refusal, so their names contradict the actual premise. | tests/it/checked_package_v2_guarantee_composition.rs:1909 |
| FND-006 | low | Every composition package charges the package-wide source-map pass twice: at the composition closure and again at mod.rs:2300. Where the MonitorPlan pre-check (mod.rs:4318) already passed the same pass, the rerun is fully redundant. A package near its work limit can now become BudgetIncomplete. | crates/filament-contract-model/src/checked_package/v2/mod.rs:4359 |
| FND-007 | low | When there is no single-region declaration-at-0 entry, the overlap check is skipped as well as the inside check. Overlapping members then refuse at `/source_map` instead of at the first overlapping entry's `/source_map/{i}`. FR-491 lists overlap as an entry defect that does not depend on the declaration. | crates/filament-contract-model/src/checked_package/v2/composition_coordinates.rs:93 |
| FND-008 | low | `CompositionSources` was inserted between `validate`'s doc comment and `fn validate`. "The guarantee step… Implements: FR-038 …" now documents the struct, and `validate` has lost its doc and its Implements tag. | crates/filament-contract-model/src/checked_package/v2/guarantee_composition.rs:388 |
| FND-009 | low | The `validate_declaration` doc still says `declaration` is forbidden on `correspondence` nodes. This PR makes `component_composition` the exception. | crates/filament-contract-model/src/checked_package/v2/mod.rs:2732 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-010 | low | FND-006's fix is the package-level once-flag (mod.rs:4332) plus the final-stage skip (mod.rs:2305), and no test exercises either. The AC-612 test injects its own counting closure into `guarantee_composition::validate`, so it checks only the per-loop flag that already existed at ddfb8854e. Deleting the `source_map_checked` short-circuit or the final skip restores the double charge and every test stays green. | crates/filament-contract-model/src/checked_package/v2/guarantee_composition.rs:1451, crates/filament-contract-model/src/checked_package/v2/mod.rs:2305 |

## Dispositions

Disposition pass 1 covers fix commit 009c94728 at head 009c9472805e4a80d4851ccf0d0ff2fe57f061b8. Merge da71ade54 brings in only origin/main a03c86b3b on top of the reviewed change.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Disjointness now uses a charged Fenwick rank index. There are three tables: positions, candidates (capacity = node entries) and the tree. All are precharged through `stage_storage::vec`, with 1 + 6·steps work per member and steps = bits(n)+1. That bound covers the binary search, two prefix sums, two selections and one insert, and there is no shifting insert. The nearest-neighbour test is sufficient because the accepted set stays pairwise disjoint and is grouped by source. The O(n log n) candidate sort follows the repo's charged-storage-only precedent and is covered by the per-member charge. AC-611 unit tests check exact storage (from size_of) and exact work, refuse one unit short of either, and refuse at the first overlapping entry among 64 descending entries. |
| FND-002 | fixed | AC-609 adds a member ending at the declaration end (refused at its entry) with a one-unit-inside control that admits. AC-608 is the start-side pair. The coder's mutant `<=` on the end bound turned that test red (/tmp/qsl2739-mA.log), the source was restored, and the head tree is clean. |
| FND-003 | fixed | AC-13 gives the composition a real `claim` occurrence and entry, plus a row whose region equals the entry. The row passes the join and only the new guard refuses it, at `/claim_names/{i}/occurrence`. The guard-deletion mutant turned it red (/tmp/qsl2739-mB.log), and the source was restored. |
| FND-004 | fixed | The test was replaced by FR-371-AC-15. It asserts the literal regions per canonical path under both source_map orders and the canonical node-id row order, with no self-computed sort. FR-371 now states that the reader SHALL NOT select or order source units, and that the consumer owns this. |
| FND-005 | fixed | The helper is renamed `refused_as_ambiguous`, the case label states the refusal, and a positive control (distinct names admit with 2 rows) was added. |
| FND-006 | fixed | The double charge is removed. The package-wide pass runs through one Cell-guarded closure, set true only after success, over the immutable `wire.semantic_graph` and `source_map`. MonitorPlan, composition and refinement share it, and the final package stage skips the pass after that success. A package with neither a composition nor a MonitorPlan keeps its old single final pass and precedence. Refusal outcomes and precedence are unchanged, because skipping happens only after the pure check has succeeded, and charges can only fall. The missing test for this is FND-010. |
| FND-007 | fixed | The overlap check no longer depends on the declaration. Only the inside test is gated by `declaration.is_some_and(..)`. The AC-613 unit test covers an absent anchor: an overlap refuses at `/source_map/1` and disjoint members refuse at `/source_map`. |
| FND-008 | fixed | The doc comment and Implements tag are back on `fn validate`. `CompositionSources` keeps its own doc. |
| FND-009 | fixed | The `validate_declaration` doc now names `component_composition` as the one correspondence exception. |
