---
id: SR-17940486709564847048086
title: "code-review and rust-review of QSL-2535 reward clause kind and Recommend binder"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1065 at 061abd3d (base origin/main c7b62038; first pass at c96d7a37 over c9e52417, identical delta); crates/filament-contract-model/src/checked_package/v2/{owner.rs,probabilistic_join.rs,refinement.rs,model_members.rs,model_members/tests.rs,model_fields.rs}, tests/it/checked_package_v2_probabilistic.rs, tests/it/checked_package_v2_refinement.rs"
review_set: subset
---

## Summary

Code review, with the rust-review lane folded in, for filament-contract-ir PR #1065. Ticket: QSL-2535. Requirement source: FSpec W76 99226882 (FR-322 "Owner joins" and `operation_bodies` "Check order", FR-640 `reward` and Reader rule 4, FR-640-AC-7, FR-322-AC-471, FR-770-AC-2, FR-432 "The binder record of an abstract node", FR-432-AC-23/24, FR-375 class catalog).

The first pass ran at c96d7a37. The head then became 061abd3d, which merges origin/main c7b62038 with no conflicts. The PR delta (`git diff origin/main...HEAD`) is byte-identical across the two heads. Gates at 061abd3d, in the scratch worktree CIR-1065-review with its own target, FILAMENT_SPECIFICATION_DIR=FSpec W76:

- `cargo fmt --all -- --check`: clean.
- `make string-edge`: clean.
- `cargo clippy --workspace --all-targets --features conformance-cli -- -D warnings`: clean.
- Full workspace `cargo test --features conformance-cli --no-fail-fast`: 2413 passed, 23 failed.
- The same run on clean origin/main c7b62038, in its own worktree and target: 2410 passed, 23 failed.
- The two failing sets are identical: 18 `monitor_plan::reached_model_read`, 3 `public_interface` and 2 `lock_coherence`. The `lock_coherence` failures are environmental: `quire check-lock` is not a subcommand of the installed quire.
- The 3 `public_interface` failures also fail on clean main. Their set difference is exactly one item, `CheckedTerminalDuplicate`: it is exported, but FR-019 no longer lists it. W80 (#1070, c7b62038) removed it from FR-019's `checked_package` row. The PR touches neither lib.rs nor FR-019, and its new items are `pub(super)` or private: `CatalogOperator` is a private enum, and `is_reward_clause` and the `*_with` functions are `pub(super)`.

## Examined scope

- `OwnerClauseKind::Reward`, `from_wire`, and `model_kind_matches` (examined). A `reward` function joins only an operation member of the declared object type. An object type, field member, undeclared member, `invariant`, `postcondition`, `Reward` or empty clause refuses `missing_declaration`/`missing-selection` at `node_id`. Mutants M03 and M05 are killed.
- Kernel check_mode refusal (examined). Reward is added to the linked-only set (FR-770). M04 is killed.
- `probabilistic_join::reward`, rule 4 (examined). The expression must be an `is_reward_clause` function. `precondition` and another operation's member refuse at the expression target. M01 and M02 are killed.
- Check order (examined). In mod.rs:2211-2230, `operation_effects` runs, then `operation_bodies`, then `probabilistic_join`, then `checked_term_data`. A `body` clause with `no_body` rows refuses at `/operation_bodies/{i}/body`, so rule 4 is not reached (FR-322-AC-471). `operation_bodies` keys only on `is_body_clause`, so a reward demands no body row.
- `CatalogOperator::Recommend` (examined). It is split from `Event`, anchored on the `operation` binding like `Attempt`, with `with_result=false`. It shares the bare-path member class with `Event`. The old Event mapping no longer leaks: `[B]` is refused. M06, M07, M09 and M11 are killed.
- `model_operation_binder_types_with` and the result filter (examined). M08 is killed by both the lib test and the it test. The capacity expression `with_result && ...` is equivalent, because it only changes allocation.
- `RetainedModels::operation_binder_types`, the External signature path (examined): FND-001.
- Test doc comments (examined): FND-002.
- Pre-existing, not introduced here (examined, no finding): `dispatch_projection` accepts any function node on the model route as `operation_body`. That includes a precondition clause function and now also a reward clause function. This predates the PR.

## Verdict

The reader and catalog changes are correct against W76. The owner, ordering, rule 4 and kernel loci all match FR-322-AC-471 and FR-640-AC-7 (model half). 10 of 11 hand mutants are killed. fmt, string-edge and clippy are clean. The failing set equals clean main's, and the public_interface reds are main's W80 inventory drift (`CheckedTerminalDuplicate`), not this PR.

One medium test gap remains (FND-001): the abstract-side catalog of a dependency (External) protocol, which FR-432-AC-24 names explicitly, is never exercised with `with_result=false`. Add that test in this PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Mutant M10 survives: passing `true` instead of `with_result` in `RetainedModels::operation_binder_types` (the `CatalogSignatures::External` path, used for the abstract protocol of a selected dependency package, refinement.rs:3155) fails no test, so a dependency-abstract `recommend` class would silently require the result type. FR-432-AC-24 states the abstract-side `class_catalog` entry `{class: "A::Adv", binder_types: [<Cell ref>, <Int[0, 3]>]}`; the only recommend test (`refinement_catalog_maps_a_recommend_as_a_node_row_class`) covers the concrete side (Local signatures) only. Add an abstract-side recommend over a dependency protocol whose operation has a result, asserting the result-free binder admits and the with-result binder refuses | crates/filament-contract-model/src/checked_package/v2/model_fields.rs:1542; crates/filament-contract-model/src/checked_package/v2/refinement.rs:3397; tests/it/checked_package_v2_refinement.rs:8101 |
| FND-002 | low | The doc comment says the refused owners are "an object type, a field member or an operation of another type". The test actually uses `Account` (the object type), `Account/balance` (a field) and `Account/missing` (an undeclared member); no operation of another type is tried. Correct the comment, or add the case | tests/it/checked_package_v2_probabilistic.rs:1897-1900 |

## Dispositions

Round 1 was checked at head 0faf1c2b. That head carries fix commit 40e101e9 and merges origin/main 358e9e2d, which includes #1071 QSL-2391 and #1075 W83e. Gates at that head:

- fmt, `make string-edge` and clippy (`--workspace --all-targets --features conformance-cli -D warnings`): clean.
- Full workspace test: head 2401 passed, 37 failed. Clean origin/main 358e9e2d, run in its own worktree and target: 2397 passed, 37 failed.
- The failing sets are identical, so the fix round adds no regression. They are:
  - 18 `reached_model_read`
  - 3 `public_interface` (the `CheckedTerminalDuplicate` FR-019 drift from W80)
  - 2 `lock_coherence` (environmental)
  - 14 await tests: 13 in `checked_package_v2_protocol_body`, plus `refinement_catalog_joins_timeout_on_await_node`
- The await failures refuse `invalid_package`/`timed-node-reference` at `/semantic_graph/nodes/33/body` on clean main exactly as on head. They come from main's QSL-2391 await change (#1071), not from this PR.

Mutant M10 was re-applied at 0faf1c2b and is now killed by `refinement_catalog_gives_an_external_abstract_recommend_no_result_binder`.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `refinement_catalog_gives_an_external_abstract_recommend_no_result_binder` (tests/it/checked_package_v2_refinement.rs:9242) reads a selected dependency protocol through the External path. The abstract entry `[receiver, Integer]` admits, and the with-result, `[B]` and receiver-only entries refuse at `class_catalog/abstract/{i}`. M10 is killed |
| FND-002 | fixed | The doc comment now reads "owned by an object type, by a field member or by an undeclared member", which matches the three owners the test runs |
