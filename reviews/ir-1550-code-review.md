---
id: SR-20261116051301
title: "code-review and rust-review of IR-1550 FR-640 join checks (PR #1029)"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1029 at 966952c3 (base origin/main 9155a127); crates/filament-contract-model/src/checked_package/v2/{probabilistic_join,probabilistic_read,mod,operation_effects}.rs, crates/filament-contract-model/src/lib.rs, tests/it/checked_package_v2_probabilistic.rs, tests/conformance_fspec/probabilistic_delay.rs"
review_set: subset
---

## Summary

This is the code review of filament-contract-ir PR #1029, with the rust-review lane folded in. Ticket: IR-1550. The scope is `git diff origin/main...HEAD` only. The reference text is FSpec FR-640, FR-405 and FR-322 at filament-specification `origin/main` 959a1982, read with `git show`. The local spec checkout is one commit behind and does not hold FR-640 in its working tree.

Gates were run in a dedicated review worktree under `flock /tmp/cargo-slot.lock`, with `--locked` and the target directory inside that worktree:

- These passed: `cargo fmt --all --check`, `cargo clippy --locked --workspace --all-targets --features conformance-cli -- -D warnings`, `cargo deny --locked check` (advisories, bans, licenses and sources ok), and `quire validate` over spec, plan and reviews (exit 0).
- `quire coverage --scope . --strict` exits 1 with 61 unbacked rows. `origin/main` 9155a127 gives the same 61, so the PR adds none, and FR-369 is 12/12.
- The whole `cargo test --locked --workspace --features conformance-cli` gave 1465 passed and 7 failed in `it`, and every other target passed. The 7 are the environment-only tests: 2 `lock_coherence` (the installed quire 0.36.2 has no `check-lock`) and 5 `string_edge_gate` (no scanner on PATH). With the CI-installed scanner, `make string-edge` itself is clean on the head (below).
- The `conformance_fspec` target is not run by the workspace command. Its two new `probabilistic_delay` tests pass when run directly with `FILAMENT_SPECIFICATION_DIR` set.
- `fsl-string-edge` was installed the way CI installs it (`cargo install --git` of filament-spec-language, which resolved to 95fbb139) into a scratch root. `make string-edge` is clean on the PR head.
- Cargo.toml and Cargo.lock are unchanged. The PR adds no pin, hash, digest, compatibility layer or tool version.

Eleven env-gated mutants of `probabilistic_join.rs` were built once and run against the probabilistic `it` tests and `conformance_fspec probabilistic_delay`: literal kind, workload order, workload coverage, workload misname, reward value type, function owner, function type, function parameters, state read, dense model, and conflict. The `it` suite killed all eleven. The conformance suite killed the conflict mutant as well. The mutants were reverted and nothing was committed.

## Examined scope

- Rule 1, operation reference, `Join::operation` (examined). A missing or non-`model` `declaring` node, an owner that is not an object type, process or state machine, or an operation the node does not declare each refuse `invalid-value` at the member, as FR-640 Reader 1 states.
- Rule 2, `random_parameter` (examined). A parameter the operation lacks refuses. Support values are checked by literal kind only: see FND-002 and FND-005.
- Rule 3, `workload` (examined). The rows must be strictly ascending by (declaring node id, operation name), which also refuses a repeated row. A misnamed operation refuses. A missing operation refuses only when the model has no supertype: see FND-003 and FND-004.
- Rule 4, `reward` (examined). The function must be owned by the operation member, list exactly the operation's parameters in order, have `value_type` as its type, and reach no state. See FND-001 and FND-006.
- Rule 5, `delay_distribution` (examined). The delay must be on the model of a `timed_model` root whose source is `dense`.
- `conflicting-definition` (examined). The keys are (form, declaring, operation, name), checked in ascending node-id order, so the refusal lands at the later node. The keys match FR-640: one parameter, one workload name per package, one reward name per operation, and one delay per operation.
- Typed read in `probabilistic_read.rs` (examined). Results come in graph order. Each probability is a weight over the sum, reduced by the gcd, so it is in lowest terms: 9 and 7/10 give 90/97 and 7/97. The FR-019 row lists the seven new types and four methods, matching the `lib.rs` re-exports.
- `own_operation` visibility widened to `pub(super)` (examined). It is used only by the join.
- Panic surface (examined). There is no `unwrap`, `expect` or indexing on untrusted positions beyond positions already bounded by `by_id`.
- `#[ix_trace_rs::string_edge]` markers (examined). See FND-008.

## Verdict

Changes requested. The join's structure, ordering, conflict keys and typed-read arithmetic are sound, and every implemented rule has a test that fails when the rule is removed. But the reward value-type check (FND-001) and the support literal-kind check (FND-002) refuse packages that FR-640 admits, including the shape of FR-640-AC-1's own `Service` example (`Millis = Int[1, 12]`). Workload coverage for models with supertypes is weaker than the Planned text says (FND-003, FND-004). The quantity-unit gap (FND-005) is not recorded as Planned. The join's work charges are not specified or tested (FND-007).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Rule 4 admits a reward `value_type` only as a `scalar_type` `integer`, `carried_quantity` or `bounded_quantity` node. FR-322 types `Quantity<u>` and `Duration` by the `scalar_type`/`unit` node, and an `Int[lo, hi]` value type by `bounded_domain`/`integer_range`. So a plain quantity reward, or FR-640-AC-1's `duration` typed `Millis = Int[1, 12]`, is refused `invalid-value`, although FR-640 admits any integer or quantity type. | crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs:284-296 |
| FND-002 | high | `literal_kind` maps every `SelectedTypeRef::Declared` parameter type to `LiteralKind::Enum`. A random parameter of a declared value type, such as `Int[lo, hi]` (FR-640-AC-1's `d: Millis` with support 1, 3, 12), a Rational value type or a Text-bounded type, has integer, rational or text support literals, so each one is refused `invalid-value`. | crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs:131-156 |
| FND-003 | medium | `Join::inherits` accepts a workload row whose `declaring` node is only a direct supertype of the model. A row for an operation inherited from a grandparent is refused at `declaring`, although that operation is an operation of the model. `model_members.rs` already has a proper-ancestors walk (line 1820). | crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs:408-410,437-455 |
| FND-004 | medium | For a model that declares any supertype, the coverage loop is skipped entirely. A workload that omits the model's own operations, even one with an empty `weights`, is admitted. FR-369's Planned text says only the inherited operations are not judged. | crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs:419-431 |
| FND-005 | medium | A `Quantity<u>` parameter's support value is checked for literal kind `quantity` only, never for its unit. FR-640 requires "a quantity literal in the parameter's unit". FR-369's Planned text records the range and delay-unit gaps but not this one, so a package with a support value in another unit is silently admitted. | crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs:152,522-528 |
| FND-006 | medium | `reads_state` is a deny-list. It walks only `function`, `expression` and `value` nodes and flags only `state` nodes and six expression forms. A reached `model`, `relation` or other node, for example a `reference` expression to a model member, is not refused. FR-405 states the rule positively: the reward "reads only the operation's parameters", and constants. | crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs:229-270 |
| FND-007 | medium | The join stage's work is not specified and not tested. It charges a second visit per `probabilistic` node and per workload row, one visit per node the reward walk reaches, and storage for `by_id`, `seen` and `named`. `is_dense` scans every graph node per delay node with no charge. FR-369 says one visit per node and per member, and the AC-5 test covers `check_wire` only. | crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs:217-224,243,374,495 |
| FND-008 | low | `#[ix_trace_rs::string_edge]` on `probabilistic_join::validate` is not needed: the CI scanner (FSL 95fbb139) reports the tree clean with it removed. The marker exempts the whole function from the string-edge gate. The marker on `delay_distributions` is legitimate: removing it gives 4 branch-gating hits at probabilistic_read.rs:261-269. | crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs:460 |
| FND-009 | low | The typed read gives weights, probabilities, rates and delay quantities as `&str` and `Box<str>`, although the crate already exports the exact `CheckedRational`. Delay quantities are given without their unit, so each consumer has to parse them again. | crates/filament-contract-model/src/checked_package/v2/probabilistic_read.rs:15-103 |

## Dispositions

Round 1 checks the fix commit 20f4f457 against the review head 966952c3. Gate at 20f4f457:

- fmt, clippy `-D warnings`, `cargo deny --locked check` and `quire validate` pass.
- `quire coverage --strict` has the same 61 unbacked rows as `origin/main`, and FR-369 is 17/17.
- `make string-edge` with the CI-installed scanner (FSL 0dce20ba) is clean.
- The whole `cargo test --locked --workspace --features conformance-cli`, with `FILAMENT_SPECIFICATION_DIR` set to an `origin/main` extract, has only the 2 `lock_coherence` environment failures.

Twenty-eight env-gated mutants of `probabilistic_join.rs` were run against the probabilistic `it` tests. They were the eleven round-0 rule removals adapted to the new code, `params` among them, plus seventeen for the new checks. Twenty-five were killed. The three survivors, `params`, `repeat` and `lineagecharge`, are gap-analysis New finding FND-005. (This count was corrected in round 2: the round-1 text and comment said 29 run and 26 killed.)

`conformance_fspec` was run against the same `origin/main` 959a1982 spec extract. In parallel, the head gives 436 passed and 9 failed and contract-ir `origin/main` 9155a127 gives 435 passed and 8 failed. Run serially (`--test-threads=1`), both give only the same 2 failures, `model_tuple_union` and `timed_clock_no_row`. The `monitor_plan` failures are `repeated allocation requests` drift between threads in parallel runs: they pass serially on both, and their count varies from run to run. PR #1029 causes none of these failures.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | `reward` now admits `scalar_type` `integer`, `unit`, `carried_quantity`, `bounded_quantity` and `bounded_domain`/`integer_range`. The FR-369-AC-13 test admits a reward of the `ms` unit and of an `Int` range, and removing the unit and integer-range arms fails it. |
| FND-002 | fixed | `literal_kind` takes the kind from `DomainModel::slot_type` (an integer or integer range gives `integer`, a quantity gives `quantity`, and so on). It falls back to `enum` only for a declared type with no member type. The FR-369-AC-13 test admits `d: Millis` (`Int[1, 12]`) with support 1, 3 and 12, and removing `slot_type` fails 6 tests. |
| FND-003 | fixed | `Join::lineage` follows supertypes transitively, and each row's `declaring` node must be in that lineage within the same package. The FR-369-AC-15 test admits Leaf/Mid/Base. Restricting to direct supertypes fails it, and so does accepting any ancestor. |
| FND-004 | fixed | Coverage now loops over every node in the lineage for every model. FR-369-AC-15 refuses a model that leaves out its own operation, and one that leaves out the grandparent's. Removing the coverage check fails 2 tests. |
| FND-005 | fixed | A support value of a quantity type must have its literal typed at the parameter's unit (`literal_unit`). FR-369-AC-14 refuses a value in `s` on a `ms` parameter, and removing the check fails it. |
| FND-006 | fixed | `reads_beyond_parameters` is now an allow-list that cites FR-405. It admits type nodes, listed parameters, values, functions and ten value-computing expressions, and refuses every other kind. FR-369-AC-16 has one case per forbidden form, plus a `state` node, a `model` node and an unlisted parameter. Each per-form mutant is killed. |
| FND-007 | fixed | FR-369 now states the join's charges, and `is_dense` charges one visit per node it scans. FR-369-AC-17 (`the_join_charges_the_dense_root_scan`) measures the differential cost, and dropping the scan charge fails it. The supertype-edge charge has no test: see gap-analysis New finding FND-005. |
| FND-008 | fixed | The marker on `validate` is removed, and the marker on `delay_distributions` is kept. `make string-edge` is clean. |
| FND-009 | fixed | Weights, probabilities and rates are `CheckedRational`. Delay quantities are `CheckedDelayQuantity { magnitude, unit }`. The new type is in `lib.rs` and the FR-019 row. |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-010 | medium | The reward value-type allow-list still leaves out `scalar_type`/`compound_unit`, which FR-322 uses for a quantity formed over a compound unit. FR-405 admits any quantity type (FR-142), so a reward typed by a compound-unit quantity is refused. FR-369 Behavior has the same omission. | crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs:335-345 |

## Dispositions (round 2)

Round 2 checks the fix commit 0ef9e0ec. Gate at 0ef9e0ec:

- fmt, clippy `-D warnings`, `cargo deny --locked check` and `quire validate` pass.
- `quire coverage --strict` has the same 61 unbacked rows as `origin/main`, and FR-369 is 17/17.
- `make string-edge` with the CI-installed scanner (FSL 61afe659) is clean.
- The whole `cargo test --locked --workspace --features conformance-cli`, with `FILAMENT_SPECIFICATION_DIR` set to an `origin/main` 959a1982 extract and the scanner on PATH, gives 2288 passed and 2 failed. The 2 are the `lock_coherence` environment failures.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-010 | fixed | `reward` now admits `ScalarTypeForm::CompoundUnit`. FR-369 Behavior and FR-369-AC-13 name the `compound_unit` node. Removing the arm fails `the_public_reader_admits_a_reward_over_a_compound_unit` and nothing else. |
