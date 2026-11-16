---
id: SR-20261115234917
title: "code-review and rust-review of IR-1549 probabilistic node tag and four forms (PR #1027)"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1027 at ed31ba20 (base origin/main 80e3f67f); crates/filament-contract-model/src/checked_package/v2/{probabilistic,flat_wire,identity,lower,mod,operations,structural,vocabulary}.rs, crates/filament-contract-model/src/lib.rs"
review_set: subset
---

## Summary

This is the code review of filament-contract-ir PR #1027, with the rust-review lane folded in. Ticket: IR-1549. The reference text is FSpec FR-640 and FR-322 at filament-specification `origin/main` c143e2b5. It was read with `git show` and extracted read-only with `git archive`. The local checkout is at 17d49a98, which is behind `origin/main`.

Gates were run in a dedicated worktree under `flock /tmp/cargo-slot.lock`:

- These passed: `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets --features conformance-cli -- -D warnings`, `cargo clippy --locked -p filament-contract-model -- -D warnings`, the three doctest lanes of `make test`, the `conformance-fspec-compile` build, `make corpus`, `cargo deny check` and `cargo audit`.
- The whole `cargo test --locked --workspace --all-targets --features conformance-cli -- --include-ignored` first ran with no scanner and no `FILAMENT_SPECIFICATION_DIR`: 1454 passed and 11 failed. Nine of the 11 need the environment: 5 `string_edge_gate` tests and 4 `whole_read_accounting` tests. All 9 pass when rerun with the scanner on PATH and `FILAMENT_SPECIFICATION_DIR` set to the `origin/main` extract. The other 2 are `lock_coherence`, which fail because the installed `quire` 0.36.2 has no `check-lock`. That is the environment, not the PR: Cargo.lock is unchanged, and the PR's CI "Supply-chain Checks" job, which runs `make deny` with the released checker, passed.
- `make conformance-fspec`'s `conformance_fspec` suite is not in `ci`. Against the FSpec `origin/main` extract it gives 435 passed and 8 failed on both the PR and `origin/main` 80e3f67f: the same 8 tests each time. Six of the 8 are `monitor_plan` tests, which pass in isolation on the PR. The other 2, `model_tuple_union` and `timed_clock_no_row`, fail on both. So the PR introduces no new failure.
- The `fsl-string-edge` scanner was built from FSL `origin/main` 3c2710f3, the same source CI installs with `cargo install --git`. It reports both the PR and `origin/main` clean.

Sixteen mutations were run on a copy of `probabilistic.rs`. A scratch probe ran the public `CheckedPackageV2::read` over packages built with the repository's own test support. Neither was committed.

## Examined scope

- `vocabulary.rs`: `CheckedNodeTag::Probabilistic` and `ProbabilisticForm` (examined). There is one tag, `probabilistic`, and four forms, `random_parameter`, `workload`, `reward` and `delay_distribution`, exactly as FR-640 states. The PR adds no fifth form. `decode`, `tag`, `form_wire` and `all` are extended in every arm.
- `probabilistic.rs` `check_node` binding order (examined). Each form is checked against the FR-640 Properties text:
  - `random_parameter` binds declaring, operation, parameter and support, and each support row binds value and weight.
  - `workload` binds model, name and weights, and each weights row binds declaring, operation and weight.
  - `reward` binds declaring, operation, name, value_type and expression.
  - `delay_distribution` binds declaring, operation, family and parameters. The family parameters are none, lower/upper, rate, or discrete rows of delay and weight.
  - All of these match FR-640 in name and order.
- `probabilistic.rs` outer record (examined). `declaration` is refused, as FR-640 states. `owner` and `recursion_group` are refused too, which mirrors the non-root `timed_model` forms (`timed_model.rs:411`). There must be exactly one occurrence, with role `declaration`, as FR-640 states.
- `probabilistic.rs` value rules (examined):
  - An empty support refuses `invalid-value`.
  - A support weight, a discrete weight or a rate that is not above 0 refuses `invalid-value`.
  - An unknown family refuses `invalid-value` at the family literal.
  - A workload weight of 0 is admitted (FR-640: "greater than or equal to 0").
- `probabilistic.rs` budget (examined). One visit is charged per node and one per support, weight or delay row, before the row is inspected.
- `structural.rs` (examined). The new form reuses the Hybrid/TimedModel rule: `semantic_type` is the node's own id, there is no `recursion_group`, and `dependencies` equal the referenced targets in structural order. The role `declaration` is fixed. FR-640 states the occurrence role and that the node is a derived node. It is silent on `semantic_type`. The spec-review SR covers that question.
- `flat_wire.rs`, `mod.rs`, `identity.rs`, `lower.rs`, `operations.rs` (examined). The new tag is routed in every exhaustive match: no nominal kind, no quantity chain, no bound, no operand family, not type-shaped, and declaration forbidden. The `unreachable!` arm in `flat_wire.rs` copies the existing timed and hybrid arms. Its kind is fixed by `body_grammar`, so malformed input cannot reach it.
- Panic surface (examined). The reader has no `unwrap` or `expect`. `Path::holder`'s `unreachable!` is an internal invariant: every `Path` is built by `node_path`. The deepest path is 14 steps, within the 16-slot array.
- `lib.rs` and the FR-019 inventory (examined). `ProbabilisticForm` is exported and listed in the V2 vocabulary row.
- Pins (examined). No Cargo.toml or Cargo.lock change, and no hash, digest or pin was introduced.
- Tests in `probabilistic.rs` (examined). They carry four `#[trace("TC-476", "FR-369-AC-n")]` tags, one per AC, and all four pass. Mutations of the empty-support, support-weight, discrete-weight, rate, declaration, unknown-family and workload-zero rules are each killed. The rest are listed in FND-003.

Probe of the public strict reader (`CheckedPackageV2::read`):

- Admitted: a workload, a random parameter and a `uniform` delay.
- Refused `malformed_wire`: a negative workload weight, a negative rate, a `uniform` delay with a parameter, and a node with an `owner`.
- Admitted, which FR-640 forbids: out-of-order workload members, a weight `2/2` that is not in lowest terms, a repeated support value, and operation references to a non-`model` node. The gap-analysis SR covers these.

## Verdict

Changes requested. The tag, the four forms, the binding order and the outer record match FR-640, and the vocabulary is extended everywhere it must be. Two value rules refuse with the wrong code: a negative weight or rate (FND-001), and parameters that do not match the family (FND-002). Several guards and the budget charge have no test that can fail (FND-003). No committed test drives the public reader (FND-004).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | A negative workload weight or negative rate is refused `malformed_wire` with no cause. FR-640 Reader rules 3 and 5 and FR-640-AC-4 require `invalid_package`/`invalid-value`, and FR-369 itself says a weight or rate that is not an exact rational refuses `invalid-value`. `weight()` maps an unparsable or negative spelling to `wire`. | crates/filament-contract-model/src/checked_package/v2/probabilistic.rs:304-325 |
| FND-002 | medium | Delay `parameters` that do not match the family (a `uniform` delay with members, an `exponential` delay without exactly `rate`, an interval without lower/upper) refuse `malformed_wire`. FR-640 Reader rule 5 lists "parameters that do not match the family" as `invalid_package`/`invalid-value`. | crates/filament-contract-model/src/checked_package/v2/probabilistic.rs:205-228 |
| FND-003 | medium | Guards with no test that can fail: the owner, recursion_group, occurrence-count and occurrence-role checks; `uniform` with parameters; the reward value_type/expression term kinds; the quantity kind; and the per-node and per-row budget charges. Mutations M04-M07, M10, M11, M13, M15 and M16 each survive. The charge rule is stated in FR-369 Behavior but has no AC or test, and FR-640 requires `incomplete` at the first uncharged member. | crates/filament-contract-model/src/checked_package/v2/probabilistic.rs:118-232 |
| FND-004 | medium | Every TC-476 test calls the private `flat_wire::check`. None reads a package through the public strict reader (`CheckedPackageV2::read` / `read_checked_package`), so the structural.rs dependency/role rule and admission through every later stage are untested. FSL's `a_random_parameter_is_emitted_and_read_back` (IR-1549 AC 4) depends on exactly that whole read. The scratch probe shows it admits today. | crates/filament-contract-model/src/checked_package/v2/probabilistic.rs:459-469 |

## Dispositions

Round 1 checks the fix commits 6d0eb621 and d0c796b0 at head d0c796b0. I re-ran the mutation set on a copy of the new `probabilistic.rs`: the original 16 mutations, adapted to the new signatures, and 8 more for the new rules. 22 of the 24 are killed. Two survive: M16 (the discrete-row `delay` quantity kind) and M23 (the new negative-quantity refusal).

A scratch probe read the public `CheckedPackageV2::read` over unit-typed quantities. It admits `uniform`, `uniform_interval 1..4` and `discrete {1: 90, 3: 10}`.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | `weight()` now refuses `invalid-value` for a non-exact spelling, a negative value and a value not in lowest terms. The unit test covers -1, -1/2, -0, 2/2, 0/5, 01, 1/0 and x, plus a rate of -1. The public reader refuses a workload weight of -1 as `invalid_package`/`invalid-value`. Mutations M17 and M22 are killed. |
| FND-002 | fixed | `family_bind` maps a family-parameter shape failure to `invalid-value`, and `uniform` with members refuses `invalid-value`. The tests cover uniform, interval, exponential, lambda and a discrete-row case, plus a public-reader `uniform`+rate case. Mutations M13, M20 and M21 are killed. |
| FND-003 | still-open | The owner, recursion_group, occurrence-count and occurrence-role guards, the reward term kinds and the node and row charges are now killed (M04-M07, M10, M11a-c, M15, M24). I accept the unit-level budget test, which checks exact counts and a `Budget` failure at the first uncharged row: the public `incomplete` mapping is the shared stage machinery. One survivor remains, M16: removing `quantity(r[0])` from the discrete row. No test gives a `discrete` row's `delay` another term kind; only the `uniform_interval` lower bound is tested. |
| FND-004 | fixed | `tests/it/checked_package_v2_probabilistic.rs` reads each of the four forms through `CheckedPackageV2::read` (AC-1) and covers missing/extra/owner (AC-2), value refusals (AC-3) and the unknown tag (AC-4). The scratch probe also confirmed the public reader admits `uniform_interval` and `discrete` with unit-typed quantities. |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | The fix adds a negative-quantity refusal (`quantity()` now matches `Some((false, _, _))`), but no test gives a time quantity a negative spelling. Mutation M23, which admits a negative quantity, survives. | crates/filament-contract-model/src/checked_package/v2/probabilistic.rs:418-424 |
