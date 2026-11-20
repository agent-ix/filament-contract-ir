---
id: SR-17940486709564847048095
title: "code-review and rust-review of IR-1576 source-declared declaring nodes in the probabilistic join"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1082 at fafbdc18 (git diff origin/main...HEAD); crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs, tests/it/checked_package_v2_source_objects.rs, tests/it/checked_package_v2_source_objects/probabilistic.rs, tests/it/checked_package_v2_probabilistic_claim.rs, tests/it/checked_package_v2_whole_read_accounting.rs"
review_set: subset
---

## Summary

Code review, with the rust-review lane folded in, for filament-contract-ir PR #1082. Ticket: IR-1576, which also covers QSL-2465's code half and adds the IR-1575 regression test. Spec authority: contract-ir FR-369 (strict read items for source-declared `declaring` nodes, AC-25, AC-26), TC-484, FR-367 and FR-377-AC-1.

Gates, in the scratch worktree `/home/peter/dev/worktrees/IR-1576-review` (its own target, serialized through `cargo-slot` with one slot):

- `cargo fmt --all -- --check`: clean.
- `cargo test --features conformance-cli --test it` filtered to `checked_package_v2_source_objects::probabilistic`, `checked_package_v2_probabilistic`, `an_empty_fairness_set` and `whole_read_accounting`: 74 run: the 13 TC-484 tests, the FR-377-AC-1 regression test and every `checked_package_v2_probabilistic*` test pass. In `whole_read_accounting` only the known baseline 3 fail (QSL-2635, FSpec `TW-29` refusal code); `TC048/source_probabilistic` is read 5th in the registry, before `TW-29`, and its accounting passes.
- Clippy and the full workspace gate were not run (thermal limit; the aggregate gate runs once before merge).

Mutants: 15 hand mutants on the new code paths of `probabilistic_join.rs`. 11 killed (M1 first-root-decides, M2 dense-root owner, M4 reward owner, M5 reward kind, M6 declaration type prefix, M7 listed entry kind, M8 FR-405 walk, M9 workload row kind, M10 workload row owner, M12 reward own type, M15 per-row visit). 4 survived: M3 (drop `is_source_object` on the dense-root model; equivalent, since a `timed_model`'s `model` that carries the same `SourceOwner` is a source object type), M11 (a missing `operation` text on a source node no longer refused), M13 and M14 (the `listed`/`named` buffers taken off `stage_storage`; the registry fixture has one parameter and one row, far under the 4096-byte `UNPAID_BOUND`). The coverage consequences are in the gap-analysis artifact.

## Examined scope

- `Operation` enum and `Join::operation`, source branch (examined). It accepts a `model`/`object_type` node with a `declaration` and a `SourceOwner` (`is_source_declared_object_type`), and refuses a missing `operation` text at `operation`. A source dimension or population node falls through to the selected branch and is refused at `declaring`. This matches FR-369's first strict item.
- `Join::dense_unit` with `accepts`/`any` and `Join::delay` (examined). On the selected path `any = false` keeps the old behavior: the first root of the model decides. On the source path the scan passes non-`dense` roots and returns the first `dense` root whose model is a source-declared node of the same `SourceOwner`. That is FR-369's "no root ... has a `dense` source" refusal read correctly: refuse only when no such root is `dense`. Each scanned node is still charged one visit, which `a_source_delay_charges_each_node_scanned_for_the_root` pins (+2 for two added nodes).
- `Join::reward` dispatch and `Join::source_reward` (examined). The value-type check is shared. The source function check covers `SourceOwner` equality, the declaration prefix and the `[op, "reward", name]` tail, `semantic_type == value_type`, `is_reward_clause`, listed entries being `value/parameter` nodes, and the FR-405 walk. The `listed` buffer is allocated through `stage_storage::vec`, so it is charged.
- `Join::random_parameter`, extracted unchanged from `validate` (examined). The diff is a pure move apart from `join` → `self`, and the selected-path tests (`checked_package_v2_probabilistic*`) stay green.
- `Join::workload` routing and `Join::source_workload` (examined). The row visit, the charged `named` buffer, row order, a repeated name and the row's `declaring` being a source object of the model's `SourceOwner` mirror the selected path. Coverage and operation existence are deferred.
- `validate` conflict keys for the source variant (examined). They are the same `(form, model, operation, name)` keys, and two `conflicting-definition` tests pin them.
- Test module wiring and the `TC048/source_probabilistic` registry case (examined). See FND-002 and the gap-analysis.
- FR-377-AC-1 regression test `an_empty_fairness_set_in_the_scheduler_binding_is_admitted` (examined). Clean: the scheduler binding `fairness -> []` is admitted, and it is tagged `TC-489`/`FR-377-AC-1`, which TC-489 lists.

## Verdict

The strict-read half is careful and mirrors the selected path. Every new allocation goes through `stage_storage`, and every new loop is charged per row or per node, so the strict-read code itself has no defect that the tests or the review found. The defect is in what the PR leaves the reader doing: a source-declared probabilistic node is now admitted and served by the typed read with no operation, parameter, support, coverage or reward-parameter check anywhere (FND-001). There is one nit (FND-002).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The reader goes from fail-closed to fail-open. Before this PR, every source-declared `declaring` node was refused at `declaring`. Now `CheckedPackageV2::read` admits it, and `delay_distributions`/`random_parameters`/`rewards`/`workloads` serve it, with the FR-369 identity-row checks never run by any stage. Concretely, these are all admitted: an operation `Other` does not declare (`nudge`), a parameter `j`, a support value `4` outside `Int[1, 3]`, a workload that omits `O::step`, and a reward function whose `parameters` differ from the operation's. The PR's own tests show it: `a_source_workload_charges_one_visit_for_each_row` admits a workload of `op0`..`op2` on `O`, which declares only `step`, and `source_delays_of_different_operations_are_no_conflict` admits a delay on `other_operation`. FR-369-AC-26 requires each of these packages to be refused. Either gate the typed read of source-declared probabilistic nodes until the join exists, or state in FR-369/FR-019 and the rustdoc that an admitted package's source probabilistic nodes are unjoined, with a live owning ticket for the join | crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs:331-343,981-1023; tests/it/checked_package_v2_source_objects/probabilistic.rs:867-892,941-954 |
| FND-002 | low | `pub(crate) mod probabilistic;` sits between two `use` blocks, after `use ix_trace_rs::trace;` and before `use quire_canonical::Limits;`. That splits the import list, and rustfmt will not move it. Place the `mod` declaration after the imports | tests/it/checked_package_v2_source_objects.rs:25-28 |
