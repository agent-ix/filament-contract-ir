---
id: SR-20261116051302
title: "gap-analysis of IR-1550 FR-640 join checks (PR #1029)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1029 at 966952c3 (base origin/main 9155a127); FR-369 AC-6 to AC-12, TC-476, spec/checked_package/matrix/tests.md; tests/it/checked_package_v2_probabilistic.rs, tests/conformance_fspec/probabilistic_delay.rs"
review_set: subset
---

## Summary

This is a planless gap analysis of filament-contract-ir PR #1029. Ticket: IR-1550. Plan completion was not assessed. It reads the FR-369 criteria AC-6 to AC-12 against the tests tagged `#[trace("TC-476", "FR-369-AC-n")]` and the code in `probabilistic_join.rs` and `probabilistic_read.rs`.

`quire coverage --strict` puts FR-369 at 12/12. Its 61 unbacked rows equal those on `origin/main`.

Eleven env-gated mutants of the join checks were run. The probabilistic `it` tests killed all eleven, so every implemented rule has a test that fails without it.

## Examined scope

- FR-369-AC-6 (examined). `the_public_reader_refuses_a_bad_operation_reference` covers random parameters, rewards and delays: a non-model `declaring` and a missing operation, plus an admitted package.
- FR-369-AC-7 (examined). The test covers a misnamed parameter and a text value on an Integer parameter.
- FR-369-AC-8 (examined). The test covers omit, repeat, reorder, misname and a non-model `model`.
- FR-369-AC-9 (examined). The test covers a Boolean type, another owner, no parameters, another type, and `pre_read`.
- FR-369-AC-10 (examined). The untimed refusal is in `it`. The dense admission is in `conformance_fspec`.
- FR-369-AC-11 (examined). Random parameter, workload and reward are in `it`; delay is in `conformance_fspec`.
- FR-369-AC-12 (examined). Random parameter, reward and workload are in `it`; delay is in `conformance_fspec`.
- TC-476 and the tests.md rows (examined). The status names the three Planned refinements and is consistent with FR-369.

## Verdict

Changes requested. Each implemented rule is covered by a test that fails without it. The gaps are: no test admits a declared or quantity-typed random parameter or reward, which is where the code-review's two high findings live; part of the evidence is outside the CI test run; and two ACs are only partly asserted.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No test admits a random parameter or a reward whose type is a declared value type (such as `Int[lo, hi]`) or a quantity type (`scalar_type`/`unit`). Every admitted case uses native `Integer`, so the refusals of FR-640-AC-1's `Service` shape go unseen. | tests/it/checked_package_v2_probabilistic.rs:24-32 |
| FND-002 | medium | The FR-369-AC-10 admission, the AC-11 delay conflict and the AC-12 delay read are tested only in `tests/conformance_fspec`. Neither `cargo test --workspace` nor CI's `ci` target runs that suite; CI only compiles it. The two tests pass when run directly. | tests/conformance_fspec/probabilistic_delay.rs:79-114 |
| FND-003 | low | The FR-369-AC-9 test reaches only a `pre_read`. The other six forbidden kinds have no test: a `state` node, `presence_read`, `value_read`, `deref`, `query` and `reachability`. Deleting any one of them from the match in `reads_state` survives the suite. | tests/it/checked_package_v2_probabilistic.rs:622-640 |
| FND-004 | low | The FR-369-AC-12 test asserts a reward's name only, although the AC states its name, type and function. For workloads it checks only the weight count and the first weight, and it reads only a `discrete` delay. | tests/it/checked_package_v2_probabilistic.rs:684-713 |

## Dispositions

Round 1 checks the fix commit 20f4f457. The probabilistic tests (18 in `it`) pass at the head with `FILAMENT_SPECIFICATION_DIR` set.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | `the_public_reader_admits_declared_and_quantity_typed_parameters_and_rewards` admits `d: Millis` (`Int[1, 12]`) with support 1, 3 and 12, a `Quantity<ms>` parameter, and rewards of a unit type and an integer-range type. The no-unit/integer-range mutant and the no-slot-type mutant both fail it. |
| FND-002 | fixed | The delay cases now also live in `tests/it/checked_package_v2_probabilistic_delay.rs`, which `make test` runs. CI's Rust job sets `FILAMENT_SPECIFICATION_DIR` (ci.yml:57). Without it the tests skip, and with `FSPEC_CONFORMANCE` set they fail. The dense-model and conflict mutants are killed in `it`. |
| FND-003 | fixed | `the_public_reader_admits_only_the_reads_fr_405_allows` reaches each of the six expressions, a `state` node, a `model` node and an unlisted parameter, and admits a constant. All eight per-kind mutants are killed. |
| FND-004 | fixed | `the_public_reader_exposes_the_typed_read` now asserts the reward's `value_type` and `function`, and every workload row's operation, weight and declaring node. The delay test reads `discrete`, `uniform_interval` and `exponential` with their exact numbers and units. |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | Three join mutants survive the tests. Removing the parameter-list match in `reward` survives, because the unlisted-parameter walk refuses the "no parameters" case first, so the list check is never the only rule a test breaks. Removing the cross-declaring repeated-name check (`named.contains`) survives. So does dropping the visit FR-369 states for each supertype edge. | crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs:367-388,423,489 |
