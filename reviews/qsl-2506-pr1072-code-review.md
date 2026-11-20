---
id: SR-17940486709564847048083
title: "Code review of QSL-2506 qualified source_class join, PR #1072"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR #1072 at a1cca1ba (merge base 8a24e2fe); crates/filament-contract-model/src/checked_package/v2/refinement.rs; crates/filament-contract-model/src/checked_package/v2/source_object.rs; tests/it/checked_package_v2_refinement.rs"
review_set: subset
---

## Summary

Ticket: QSL-2506. Code review with the rust-review lane, scoped to `git diff origin/main...a1cca1ba`. The diff is taken against merge base 8a24e2fe; the FR-061 spec change landed separately on main in #1073.

The head makes three changes:

- `require_declared_operation_class` resolves a catalog-free `source_class` before any `source_node` check. A bare `op` refuses `invalid_model_binding`/`malformed-declaration` at `source_class`. A `T::op` with no no-point anchor of a type named `T` refuses `missing_declaration`/`missing-name` at `source_class`.
- `source_class_names_anchor` joins the resolved class to the anchor named by `source_node`: same operation, no execution point, and the context type is `T`. A mismatch refuses at `source_node`.
- `declares_type_name` compares `T` against the model-owned node name through the existing marked helper `source_object::source_type_identity`, which is now `pub(super)`.

Gates at a1cca1ba, each run in its own worktree with its own target directory:

- `cargo fmt --check`: clean.
- `make string-edge`: clean, with no new marker.
- `clippy --workspace --all-targets --features conformance-cli -D warnings`: clean.
- `--test it refinement`: 96 passed, 1 failed. The failure is `refinement_catalog_joins_timeout_on_await_node`, which also fails on clean main 8a24e2fe.
- Full workspace (`--features conformance-cli --no-fail-fast`) on head and on 8a24e2fe: 35 failures on each, the same set. They are the 18 `monitor_plan::reached_model_read` tests, 13 `protocol_body` await tests, `refinement_catalog_joins_timeout_on_await_node` and 3 `public_interface` tests. The head passes 3 more tests than main (2402 vs 2399).

Mutants were run against `--test it refinement`. A failure only in the pre-red await test does not count as a kill.

- Killed: N6 (the join ignores the operation) and N7 (a bare class refuses as a missing class).
- Survived:
  - N1: class resolution ignores `execution_point`.
  - N2: the join ignores `execution_point`.
  - N3: `declares_type_name` ignores the identity check.
  - N4 and N8: the scan charge is 0 or 1 instead of `terms.len()`.
  - N5: the join ignores the type.

Interim findings at 5cc70671 that were not posted:

- Fixed: the per-row unmetered `String` allocations, the `/`-joined owner-path spelling, and the new `string_edge` marker. The comparison no longer allocates and reuses `source_type_identity`. Model-owned type nodes are `ix://<identity>/<name>` with one flat name, and FSL emits flat names (for example `Cell::increment`).
- Fixed in #1073: FR-061's stale bare-op wording.

## Verdict

CHANGES REQUESTED. The ruling is implemented. A bare `op` is refused at `source_class` with `invalid_model_binding`/`malformed-declaration`. The order is: not `T::op`, then missing-name at `source_class`, then a `source_node` mismatch. Refusal codes and loci match FR-061-AC-15. Work is charged before the scan.

Four defects remain:

- Class resolution does not restrict `T` to the selected concrete model.
- The per-row whole-graph scan makes work quadratic.
- Several refusals the AC names have no killing test, including the two point-qualified checks and the another-type anchor.
- One modified test lost the source-node case it used to check.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Class resolution accepts any local no-point anchor whose context's own node identity matches its owner. It does not require the selected concrete model. A `T::op` naming a type of the local abstract model, or of another selected model, that declares `op` therefore resolves, and then refuses `invalid_model_binding` at `source_node`. FR-061 requires `missing_declaration`/`missing-name` at `source_class` for a `T` that does not declare `op` in the selected concrete model subject. | crates/filament-contract-model/src/checked_package/v2/refinement.rs:1887-1910,1936-1945 |
| FND-002 | medium | `require_declared_operation_class` scans every term of the graph for each catalog-free step row and charges `terms.len()` each time. Read work is therefore rows × terms. A valid model refinement with many rows over a large package can exhaust `CheckedPackageReadLimits::bounded()` work (1,000,000) where the neighbouring joins use an index. Build the (type name, op) set of no-point anchors once per refinement, metered, and look each row up in it. | crates/filament-contract-model/src/checked_package/v2/refinement.rs:1897-1904 |
| FND-003 | medium | Refusals the tagged tests claim are not exercised: mutants N1, N2, N5, N3, N4 and N8 survive. There is no test for a point-qualified anchor, either as the only anchor of `(T, op)` or as `source_node` next to a no-point anchor. There is no test for a `source_node` that is the anchor of another type declaring the same `op` (FR-375-AC-31 "an anchor of another operation or type"). A point fixture is feasible: push `binding("execution_point", {"kind":"pre","operation":"save"})` onto the anchor members, as `pointed_state_package` does. The exact charge of the scan is also unasserted. | tests/it/checked_package_v2_refinement.rs:4257-4377 |
| FND-004 | low | `refinement_catalog_presence_precedes_declaration_backed_step_joins` used to assert that a step whose `source_node` is the state-machine declaration node refuses at `source_node`. That is the "model-owner node" case of FR-375-AC-31. The step's class is the bare `operation`, so the test now stops at `source_class` and that `source_node` case is no longer covered. Give the step a resolvable `T::op` so it still reaches `source_node`. | tests/it/checked_package_v2_refinement.rs:7218-7242 |

## Dispositions

Disposition pass 1 at head 767b4eee (fix commit bfe3fa75, merge base 312f3752).

Gates at 767b4eee:

- fmt, `make string-edge` and clippy (`--workspace --all-targets --features conformance-cli -D warnings`): clean.
- `--test it refinement`: 99 passed, 0 failed.
- Full workspace on head and on 312f3752: 18 failures on each, the same set. The head passes 4 more tests (2448 vs 2444).

Mutants at 767b4eee:

- Killed: P1 (the table ignores the execution point), P2 (the join ignores the execution point), P3 (the table ignores the selected model) and P7 (a bare class refuses as a missing class).
- Survived: P4 (the join ignores the type, which is the old N5), P5 (the table is not sorted) and P6 (the scan charge is 0, the old N4/N8).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `declared_operations` admits only anchors whose context's `Model` owner identity equals the selected `model_root` identity. A foreign-model `T::deposit` now refuses missing-name at `source_class`; the test was added and mutant P3 is killed. |
| FND-002 | fixed | The (type name, operation) table is built once per refinement, with `terms.len()` work charged and the table's bytes charged through `stage_storage::vec` before allocation. Each row is a `binary_search`. |
| FND-003 | still-open | The point-qualified checks are now tested (`catalog_free_operation_step_refuses_point_qualified_anchor`, P1 and P2 killed). P4/N5 still survives: no test has a `source_node` that is a selected-model anchor of another type declaring the same `op`. That is FR-375-AC-31's "anchor of another ... type" branch, and the type comparison is its only guard. The fixture is feasible: clone `types[0]` under a new identity with `rewrite_identity_prefix`, as the inheritance builder does, and point an `add_anchor_variant` clone's context at it. The exact scan charge (P6, N4/N8) is accepted with no change. FR-038 and FR-061 fix no per-step work amount, the charge-before-work rule holds by inspection, and an exact-constant test would only pin an implementation detail. |
| FND-004 | fixed | The test now uses `catalog_free_operation_package("state_machine")`, whose class `Machine::toggle` resolves, so the model-owner node `source_node` again refuses `invalid_model_binding` at `source_node`. |
