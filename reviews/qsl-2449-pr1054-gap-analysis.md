---
id: SR-17940486709564847048078
title: "gap-analysis of QSL-2449 role-body boundary admission against the ticket and FSpec W46"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1054 at 12eab4ef (base origin/main a15bf349); FR-038-AC-564, FR-038-AC-565, TC-048 role boundary section, tests/it/checked_package_v2_protocol_body.rs"
review_set: subset
---

## Summary

Gap analysis of filament-contract-ir PR #1054. Ticket: QSL-2449. Plan completion: not assessed.

QSL-2449's acceptance has two parts. (1) "a contract-ir test reads and writes a role body with boundary=observe-only and an unknown value is refused". (2) "draco:QSL:code's strict-read test for QSL-2211 passes against the pinned contract-ir". Its stated outcome is that the reader and writer admit the member "as specified by W46".

The QSL-2211 strict-read test is `the_package_records_the_observe_boundary` in filament-spec-language PR #2932 (head fb8b0768), `fsl-package/src/emit/tests/protocol_observe.rs`. It emits `observe role Auditor`, `run observe sequence Review { check ...; recommend Advise by Auditor ...; }`, and reads the result through `read_checked_package` of filament-contract-model. It asserts that both the `Auditor` role node and the `Review` control node carry the last argument `boundary` `observe-only`, and that `Advise` is a `control` of operator `recommend`.

## Examined scope

- FR-038-AC-564 (examined): Implemented (QSL-2449). The public reader admits an FSpec FR-465 role whose last argument is the `boundary` binding, the text `observe-only`, after `model_member` when both are present and directly after `replication` otherwise; the role then reads `protocol_role_boundary` `Some(ObserveOnly)`, the typed writer re-encodes it, and the readback is equal. Backed by `observe_only_role_boundary_admits_and_round_trips_after_the_member` (passes; mutants M2, M7 and M15 killed).
- FR-038-AC-565 (examined): A role `boundary` whose text is not `observe-only` (including the empty text), whose value is not a text, that precedes `replication` or `model_member`, or that repeats refuses `malformed_wire` with no cause at `/semantic_graph/nodes/{n}`. Backed by `role_boundary_refuses_unknown_text_and_misplaced_member_malformed_wire` (passes). The non-text branch is covered only with `model_member` present (code-review FND-001).
- QSL-2449 acceptance part 1 (examined): met by the two tests above.
- QSL-2449 acceptance part 2 (examined): not met (FND-001 below).
- FSpec FR-465-AC-14 (context_only): An observe-only role carries the last argument `boundary` `observe-only`, after `model_member` on a role on a Port ...; with an `attempt` control whose `role` is the observe-only role added to the graph, the reader refuses `ill_typed`/`operator-ineligible` at the attempt node, retaining `action` `attempt` and `boundary` the role node.
- FSpec FR-466-AC-19 (context_only): A `recommend` control is admitted ...; a `sequence` carrying the last argument `boundary` `observe-only` and holding a `check`, a `receive` and a `recommend` is admitted; ... a `boundary` text other than `observe-only` refuses `malformed_wire`.
- FSpec FR-322-AC-462 (context_only): ... a `boundary` member on any node that is not a `protocol`/`role` or `protocol`/`control` node refuses `malformed_wire`.
- Full workspace (examined): `cargo test --workspace --features conformance-cli --no-fail-fast`, run on head 12eab4ef and on clean main a15bf349, each with its own target dir. The failing sets are identical: 18 `filament-contract-model` lib `monitor_plan::reached_model_read` tests on both. See the reviewer report for the exact comparison.

## Verdict

FAIL on ticket acceptance. The role-only admission is correct and tested. But the QSL-2211 strict-read test the ticket names as acceptance needs the control-node `boundary` and the `recommend` operator, and neither is in this PR. The coder's carve-out is honest in the AC text and the PR body, but it leaves QSL-2449 unsatisfiable as written and QSL-2211 still blocked.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | QSL-2449 acceptance part 2 cannot pass. draco's QSL-2211 strict-read test emits `observe sequence Review` and a `recommend` control as well as `observe role Auditor`. At 12eab4ef a control-node `boundary` `observe-only` refuses `malformed_wire` at the control node (probe: `/semantic_graph/nodes/22`, a `sequence`), and `layout()` has no `recommend` control operator, so the package still refuses and QSL-2211 stays blocked. | crates/filament-contract-model/src/checked_package/v2/protocol_body.rs:370-392,1074-1076 |
| FND-002 | medium | The W46 reader obligations this PR leaves out have no owning IR criterion or ticket. They are FR-465 check 6's `operator-ineligible` refusals behind an observe-only role, FR-466 check 7 (control-node `boundary`), FR-466-AC-19's `recommend` layout, and FR-322-AC-462's refusal of `boundary` on a non-protocol node (a probe appended `boundary` to a `function`/`pure_function` node and the package was admitted). tests.md says they "remain open" but names no ticket. | spec/checked_package/matrix/tests.md:182; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6701 |

## New findings (disposition pass 1)

Head 51166c4b. The full workspace (`--features conformance-cli --no-fail-fast`, own target dirs, single slot, FSpec 9a181ace) was run on head and on clean main e0327d7c. Each first run showed 28 failures, because a shared /tmp tmpfs hit its quota. The 13 non-`reached_model_read` names were re-run serially on both trees and all passed. The real failing set on both is the 18 model-lib `monitor_plan::reached_model_read` tests, identical. Head passes 11 more tests than main.

QSL-2211 check: in a scratch worktree, filament-spec-language #2932 (fb8b0768) was merged with FSL origin/main bdb9a2ef, and `filament-contract-model` was pinned to rev 51166c4b. Compiling needed three shims for struct fields that other contract-ir PRs (#1045, #1049) added: `operations: Vec::new()` on `SelectedSourceObjectRow` twice, and `dependency_clock_selections: None` on `CheckedPackageLockV2`. The test's own `argument_names` helper then panicked on a node with no `arguments` array (`expect("arguments")`). With that helper made tolerant, `the_package_records_the_observe_boundary` passes: the strict reader admits the marked and unmarked packages.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | `refinement::catalog_operator` maps no `recommend` control, so a recommend node contributes no class to the checked class catalog. FSpec FR-426 defines a `recommend` step, but FR-432's class and abstract-path tables list no `recommend` class. This is an FSpec gap: it needs an FSpec ruling and a ticket. It is not a defect in this PR. | crates/filament-contract-model/src/checked_package/v2/refinement.rs:3876-3892 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Control-node `boundary` and the `recommend` layout are implemented (AC-566, AC-567). draco's QSL-2211 strict-read test passes against contract-ir 51166c4b once FSL main is merged; the only changes needed were FSL-side compile shims and a test-helper fix, none of them in IR |
| FND-002 | fixed | The carved-out obligations now have IR criteria and are implemented: AC-566 (recommend), AC-567 (control boundary), AC-568 (check 6 and check 7 refusals), AC-569 (FR-322-AC-462 non-protocol boundary, flat-wire refusal; mutant N10 killed) |
