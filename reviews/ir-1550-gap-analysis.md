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
