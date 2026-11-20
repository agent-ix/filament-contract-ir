---
id: SR-17940486709564847048101
title: "code-review of QSL-2645 metered probabilistic identity-row join (PR #1086)"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1086 at 7b30d82da (git diff origin/main...HEAD); crates/filament-contract-model/src/checked_package/v2/source_join.rs, source_object.rs, probabilistic_join.rs, probabilistic_read.rs, mod.rs; tests/it/checked_package_v2_source_objects/probabilistic.rs, probabilistic/join.rs; tests/fixtures/qsl-2645-golden"
review_set: subset
---

## Summary

Ticket: QSL-2645. This is a code review with the rust-review lane folded in. It checks PR #1086 against the arch ruling comment on QSL-2645 and the merged spec (#1085: FR-369, FR-367, FR-374/375, FR-358).

The ruling's shape is followed. `join_probabilistic(&self, &mut NativeJoinBudget) -> Result<(), SelectedSourceObjectError>` is a separate method, `from_stage_rows` keeps its signature and stays unmetered, there is no second budget API, `Incomplete` returns no partial view, and the read does not run the join. `SelectedSourceObjectError.locus` is `None` for every `from_stage_rows` reason. `source_region: None` matches the strict read: `CheckedPackageRefusal::new` and `push_path` never set a source region.

Three things do not conform:

- Two refusal loci are one level above the member the strict read names.
- The join charges no FR-358 storage, uses unmetered big-integer parses, and records no success delta.
- Its own scans over the graph and the rows are uncharged.

Tests run (one cargo slot, a private CARGO_TARGET_DIR, FSpec origin/main 2b9645be): `cargo test --locked --test it checked_package_v2_source_objects::probabilistic`. Result: 27 passed, 0 failed (`cargo test --locked --features conformance-cli --test it checked_package_v2_source_objects::probabilistic`, one cargo slot, a private CARGO_TARGET_DIR, FSpec origin/main 2b9645be); `rustfmt --check` is clean on the two new files.

## Verdict

**NOT MERGEABLE.** There are two high findings.

FND-001: the operation and parameter refusals are located at `/body/members/{1,2}/value`. The strict read names `/body/members/{1,2}/value/value` (`probabilistic_join.rs:324,648`, and pinned by `tests/it/checked_package_v2_probabilistic.rs:746,765`). The tests assert the join's own path, so they pass.

FND-002: FR-358 storage is not charged. A budget with a storage limit of 0 admits the whole join, and `a_budget_one_visit_short_stops_the_join_as_incomplete` runs at `NativeJoinBudget::new(limit, 0)`. The `order`, `lineage`, `declared` and `named` Vecs, the `JsonPointer` builds, the `Membership::Text` box and the unreserved `Integer::parse` and `gcd` are all unpaid heap. The strict read meters these through `stage_storage::vec` and `exact_integer` / `canonical_rational`.

What went right: the clock-entry handling, the transitive lineage, the support-membership port, the charge-ahead ordering and the resume semantics are correct. The tests are well targeted.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The `operation` and `parameter` refusals are located at `/semantic_graph/nodes/{i}/body/members/1/value` and `/body/members/2/value` (`member_at(holder, 1)` and `member_at(holder, 2)`). The strict read names `binding_at(..).key("value")`, which is `/body/members/1/value/value` and `/body/members/2/value/value` (`probabilistic_join.rs:324,648`, pinned by `tests/it/checked_package_v2_probabilistic.rs:746,765`). FR-369 requires `locus` `Some` of the same member the strict read would name. The join tests build the same wrong path with `member_at`, so they pass. Fix `entry()` and `random_parameter()` to add `.key("value")`, and assert against the strict read's path | crates/filament-contract-model/src/checked_package/v2/source_join.rs:117-123,253-290; tests/it/checked_package_v2_source_objects/probabilistic/join.rs:118-123,244,251,757 |
| FND-002 | high | No FR-358 storage is charged. FR-358 says "IR SHALL compute and admit the required storage charge before every corresponding reserve, clone or insertion", and temporary sets and buffers follow the same rule. The `order` Vec (one entry per probabilistic node), `lineage`, `declared` and `named` are never charged, and neither are the `JsonPointer` loci built per node and row, the `Membership::Text` `Box<str>`, or the error locus. `canonical_rational` / `integer_of` use the unreserved `Integer::parse` and `gcd`. The strict read meters the same parses through `exact_integer` / `canonical_rational` (`graph_storage.rs:552-591`). As a result the heap grows with literal length and is unpaid, which is the same class QSL-2653 is fixing in the strict read. The join also passes at storage limit 0 (`join.rs:410,415`). FR-352's joins (`nominal_reads.rs:100-161`), which the ruling names as the model, charge Storage | crates/filament-contract-model/src/checked_package/v2/source_join.rs:45,69-79,199,376-433,442-452,477-485 |
| FND-003 | medium | The join never records a success delta. It calls `budget.resume()`, which clears `last_success_delta`, but never calls `begin()`/`finish(before)`, so after `Ok(())` the budget's `last_success_delta()` is `None`. FR-358 says "Each successful operation ... records its work and storage delta". FR-352's joins call `begin`/`finish` (`nominal_reads.rs:101,138`). No test checks the delta | crates/filament-contract-model/src/checked_package/v2/source_join.rs:470-474,517 |
| FND-004 | medium | Some scans are uncharged work. These are the filter over every graph node and the `sort_by` on node ids that build `order`, the `lineage.contains`, `declared.contains` and `named.contains` scans (quadratic in rows and entries), and the text scans in `operation_of` (`rsplit_once` over the identity) and `text_length`. FR-358 says "A scan over caller-controlled text or a sequence charges each examined byte or element before advancing". FR-352 charges identity text by length (`nominal_reads.rs:301-311`). The rows' `identity` text is caller-supplied | crates/filament-contract-model/src/checked_package/v2/source_join.rs:33-36,376-433,477-485 |
| FND-005 | medium | The breaking API change is not declared. `SelectedSourceObjectError` gains a public `locus` field and loses `Copy`, which breaks struct-literal construction and `Copy` users. `SelectedSourceObjectErrorCode` (not `#[non_exhaustive]`) gains `InvalidValue` and `Incomplete`, which breaks exhaustive matches. The FR-019 row only describes the method, and neither the PR body nor FR-019 says that this is a breaking change riding the next contract-ir tag, which FR-019 states for comparable changes (for example, the V2 claim-names row) | crates/filament-contract-model/src/checked_package/v2/source_object.rs:177-217; spec/model/functional/FR-019-rust-library-interface.md:332 |
| FND-006 | low | The support-membership, canonical-rational and workload rules are duplicated from `probabilistic_join.rs` (`Membership`/`not_member`, `canonical_rational`, the lineage walk). QSL-2653 is editing the strict read's workload and rational paths, so its fixes will not reach `source_join.rs`, and the two copies will drift. Share the strict read's metered helpers (`exact_integer`/`canonical_rational` through a meter, `not_member`) instead of a second copy | crates/filament-contract-model/src/checked_package/v2/source_join.rs:38-60,208-251,376-433,442-452 |
| FND-007 | low | When `QSL_2645_WRITE_GOLDEN` is set, the golden test writes both files and then compares them with what it just wrote, so it passes whatever the reader does. Writing a regression into the golden pair is silent. Make the write path fail the test after writing (or return early with a panic message), so a rewrite can never also count as a pass | tests/it/checked_package_v2_source_objects/probabilistic/join.rs:145-156 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-008 | medium | The join charges its lists to FR-358 logical storage in bytes: `push` charges `extra * size_of::<T>()`, and `AC25_STORAGE` is `4 * 8 + 4 * 24 + 4 * 16`. FR-358 defines a storage unit as "a logical ownership unit, not an allocator byte count", namely one unit per newly owned container or element plus one per byte of copied owned text, and says "Each counter therefore has one unit and its limit keeps one meaning". The new charge gives the logical counter a second unit and ties the test's count to the platform's `usize` size. Charge one unit for the container and one per element slot reserved. The 256-unit refusal-pointer charge is a deterministic upper bound on the bytes of owned pointer text (the longest pointer the join builds is about 150 bytes), so it conforms | crates/filament-contract-model/src/checked_package/v2/source_join.rs:141-160; tests/it/checked_package_v2_source_objects/probabilistic/join.rs:595-598 |
| FND-009 | low | Three diagnostic gaps remain. First, `incomplete(at)` builds the stop's `JsonPointer` without a charge, including after a denied logical-storage charge. FR-358 keeps only the compact denial when "diagnostic storage is unaffordable". Second, the read-storage stop's `first_unpaid_locus` is the strict read's `PackageRead(Graph, holder)` locus (`meter.at_node`), not the join's `GraphNode`. Third, a successful join calls `finish`, not `finish_read`, so `last_success_read_storage_delta()` stays `None` although the join charged read storage. FR-369 now says "the budget's last success delta SHALL be what the join charged" | crates/filament-contract-model/src/checked_package/v2/source_join.rs:86-93,130-138,354,609-626 |

## Dispositions

Round 1, at head 0c9d47aea (fix 94821764f, survivor tests d4e58d837, merge of origin/main). Focused run: `cargo test --locked --features conformance-cli --test it probabilistic` (one slot, private target dir, FSpec origin/main 2b9645be). Result: 85 passed, 0 failed (the TC-476, TC-484, delay and claim suites).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `entry()` and `random_parameter()` now refuse at `member_at(holder, 1).key("value")` and `member_at(holder, 2).key("value")`. `the_join_names_the_loci_the_strict_read_names` derives the expected path from the strict read's own refusal over a selected node |
| FND-002 | fixed | The lists are charged ahead of growth (`push` with `try_reserve_exact`), the refusal pointer is charged 256 ahead of being built, the `Membership::Text` box is gone (the profile is borrowed), and the exact parses go through the shared metered `bounded_not_member`/`exact_integer` via `WorkMeter::over`. A storage-short or read-storage-short budget now stops as `Incomplete`. The unit and counter used are recorded as FND-008 and spec-review FND-004 |
| FND-003 | fixed | `join_probabilistic` takes `before = budget.consumed()` after `resume()` and calls `budget.finish(before)` on `Ok`. `a_joined_package_records_its_charges_as_the_success_delta` pins the work and storage delta, and every short budget asserts `last_success_delta() == None`. The read-storage delta is FND-009 |
| FND-004 | fixed | `run()` charges one unit per node met in `node_ids_ascending()`, which replaces the uncharged filter and sort. The lineage, declared and named lookups charge per element met (`lineage_has`, `declared_has`, `named_has`), and `charge_text` adds one unit per byte of the caller text compared. This judgement is consistent with FR-358: the per-node visit is a real kind inspection required by the sequence-scan rule, not "visited solely to consume budget", and the per-byte and per-element charges are FR-358's scan rule |
| FND-005 | fixed | The FR-019 row now ends "A breaking change that rides the next contract-ir tag: the error type gains the public field locus and is no longer Copy, and its code enum, which is not non-exhaustive, gains two variants", and the PR body has a "Breaking API" section listing both |
| FND-006 | fixed | Membership is shared: `Bounded` and `bounded_not_member` in `probabilistic_join.rs` serve both the strict read and the join. The duplicated `canonical_rational` is gone, and parses use the strict read's metered helpers. The row-based lineage walk necessarily differs from the strict read's domain walk. QSL-2653's fixes to the shared helpers now reach the join |
| FND-007 | fixed | The write path now panics after writing: "the golden pair is rewritten: review its diff, then run without QSL_2645_WRITE_GOLDEN" |
