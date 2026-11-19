---
id: SR-17940486709564847048075
title: "gap-analysis of QSL-2400 TemporalBinding trace member"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1051 at 93e09980 (base origin/main 71914a07); FR-038-AC-561, TC-480 against crates/filament-contract-model/src/checked_package/v2/temporal_binding.rs and tests/it/{checked_package_v2_temporal.rs,checked_package_v2_canonical_encoding.rs}"
review_set: subset
---

## Summary

Gap analysis for PR #1051. Ticket: QSL-2400. Planless: plan completion was not assessed. Upstream: FSpec origin/main 0bf58005, FR-322-AC-294, FR-322-AC-453, TC-1680 and FR-322 "Hyper clause rows".

The W52 ruling (FSpec FR-322-AC-463/464, TC-2027/2028) changes three cases: hyperproperty-node keying, finite-domain rows, and a distinct-`sequence_authority` positive. Those are left to the delta pass. This review covers the parts that stay.

`quire trace --id FR-038-AC-561` shows 9 structural claims: one lib test, one canonical-encoding test and seven `tc_480_*` integration tests. `quire coverage --strict` reports 79 unbacked rows and 0 contradicted statuses, the same at head and on main, and AC-561 is backed.

Hand mutants (cargo-mutants was not used; 14 hand-applied to temporal_binding.rs, each run against `--test it tc_480` and the lib `trace_member_round_trips` test, source restored after):
- killed: M01 null as absent, M02 writer drops trace, M03 empty trace, M04 trace on non-wrapper, M05 absent wrapper trace, M06 unknown trace, M07 order check, M08 one row per occurrence, M09 any to all, M11 key ignores trace.
- survived: M10 (variable dedupe removed), a real survivor (FND-001).
- survived, equivalent: M12 (any claim form), M13 (body target kind), M14 (non-text variable literal). All three are in `hyper_wrappers`, which the W52 rework replaces.

## Examined scope

- FR-038-AC-561, the reader/writer/schema half (examined): covered by `trace_member_round_trips_and_refuses_violating_documents` and `tc_480_binding_trace_member_is_written_only_when_present`.
- FR-038-AC-561, the refusal list (examined): absent and unknown trace, trace on a non-wrapper row, out of order, repeated key, missing row, and malformed. Each has a `tc_480_*` test, and the mutants above are killed. The locus assertions are weaker than `expect` (FND-002).
- FR-322-AC-453, the refusal vectors (examined): a removed `n` row, `trace` `x` at `/temporal_bindings/1/trace`, a third repeated `s` row, and a trace on a single-trace row. All four are reproduced.
- FR-322-AC-453 positive, two models / distinct configurations (context_only): superseded by the W52 ruling case 3. Deferred to the delta pass.
- Full workspace failing set (examined): identical to clean main 71914a07 except the string-edge gate (code-review FND-001).

## Verdict

The refusal set AC-561 states is fully covered, and 10 of the 11 non-equivalent mutants are killed. Two gaps remain. A repeated quantifier variable is not tested (it waits on a spec ruling). The trace-locus tests check the pointer but not the refusal's locus node. Neither blocks once the spec-review FND-004 ruling is in and its test is added.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Unkilled: the quantifier-variable dedupe in `hyper_wrappers`. Hand mutant M10 removes `.filter(\|name\| !variables.contains(name))` and survives every `tc_480_*` test. No case quantifies one variable in two Groups, and that case's outcome is unspecified (code-review FND-002, spec-review FND-004). Add the case once draco rules | crates/filament-contract-model/src/checked_package/v2/temporal_binding.rs:544; tests/it/checked_package_v2_temporal.rs:3418-3594 |
| FND-002 | low | Weak locus assertions. `refusal_at_declaration_trace` checks code, cause and pointer but not `refusal.locus`, which the file's own `expect` helper asserts against FR-038's "Path and locus" table. `tc_480_an_empty_or_null_trace_is_malformed_wire` asserts only that the pointer starts with `/temporal_bindings/0`, so a malformed refusal at another member of row 0 (for example `clock`) also passes. Assert the exact pointer and the locus node | tests/it/checked_package_v2_temporal.rs:3512-3518,3574-3594 |

## New findings (disposition pass 1)

Round 1, head f7cf238c. TC-483 now carries 14 steps and 18 tagged tests, and the full suite's failing set is identical to clean main a15bf349. Hand mutants M01 to M26 (see the code-review SR): 19 killed, 4 equivalent (M12, M13, M14, M23), 3 real survivors, listed below.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | Unkilled: the finite-domain clock-name agreement is scoped to one occurrence. Hand mutant M24 drops `&& rows[row_index - 1].occurrence == row.occurrence`, so a row is compared with the previous row of any occurrence, and it survives. FSpec FR-322 "Hyper clause rows" (finite domain) says "two occurrences of one node may carry different roles, and the reader checks the rows of one occurrence for agreement". Every finite fixture has one `claim` occurrence. Add a finite node with two `claim` occurrences whose rows carry roles `t` and `u`, expected admitted | crates/filament-contract-model/src/checked_package/v2/temporal_binding.rs:769-775; tests/it/checked_package_v2_temporal.rs:3550-3600 |
| FND-004 | low | Unkilled: a behaviours node under a `timed` wrapper profile has no rows. Hand mutant M19 (`configured_profile(wrapper, false, ..)`, so Timed counts as configured) survives. FR-322 says rows exist only when the wrapper law selects one of the three clock-configured profiles. The only unconfigured case pinned is the infinite-trace profile (`tc_483_a_behaviours_node_under_an_unconfigured_wrapper_profile_has_no_rows`) | crates/filament-contract-model/src/checked_package/v2/temporal_binding.rs:700; tests/it/checked_package_v2_temporal.rs:3736-3753 |
| FND-005 | low | Unkilled: a finite node under a lock whose `temporal_profile` selection is not clock-configured. Hand mutant M18 (finite always configured) survives. The code requires rows only when `selected_profile(lock)` is clock-configured. FR-322 says a finite-domain clause "always has rows" because every finite-trace profile is clock-configured. Pin the reader's outcome for a finite node under a non-clock-configured lock, whichever stage refuses it | crates/filament-contract-model/src/checked_package/v2/temporal_binding.rs:696-697 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | still-open | New this round: no two-occurrence finite case |
| FND-004 | still-open | New this round: no timed-profile behaviours case |
| FND-005 | still-open | New this round: no finite case under an unconfigured lock |
| FND-003 | fixed | - |
| FND-004 | fixed | - |
| FND-005 | deferred | Sent to draco:QSL:arch for a ruling on a finite node under a lock whose `temporal_profile` is not clock-configured (admitted without rows here, against FR-322's "always has rows"); the coordinator will pin it in a follow-up commit once the ruling arrives. M18 survives until then |
| FND-005 | fixed | - |

## Disposition pass 2 notes

Round 2 (disposition pass 2), head 8cded29d: fix commit 0b8cf26e, plus a clean merge of origin/main 9edbd391. FSpec is origin/main ebfcb277. Gates at 8cded29d:
- `make string-edge`, fmt and both clippy lanes: clean.
- Full workspace test: 2251 passed, 21 failed. Clean main 9edbd391: 2231 passed, 21 failed. The failing sets are identical: 18 `monitor_plan::reached_model_read` and 3 `public_interface`.
- `make conformance-fspec`: the same 6 failures on both.
- quire validate and `coverage --strict`: identical to main (89 unbacked rows, 0 contradicted). AC-561 has 20 trace claims.
- Hand mutants M01 to M26 at the new head: 21 killed, now including M19 and M24. M12, M13, M14 and M23 remain equivalent survivors. M18 survives, deferred.

## Disposition pass 3 notes

Round 3 (disposition pass 3), head 71892dd4. The delta since 8cded29d is fix commit 9d4bcfa7, a merge of origin/main (including #1045 `validate_clock_configuration`), and the 71892dd4 `lock_tables` test literal `trace: None`. Per the arch ruling, a finite-domain hyperproperty always requires its rows. Under a lock whose selected `temporal_profile` is not clock-configured, it refuses `invalid_package`/`timed-node-reference` at `/semantic_graph/nodes/{N}/body`. Test `tc_483_a_finite_domain_node_under_a_profile_without_a_clock_refuses_at_its_body` and the AC-561 note cover this. FSpec is FSPEC-9a181ace.

Focused gates, single slot:
- `make string-edge`, fmt and both clippy lanes: clean.
- `--test it -- tc_483 tc_346`: 32 passed.
- model lib `temporal_binding`: 6 passed.
- `temporal_binding_public`: 1 passed.
- AC-561: 21 trace claims.

Hand mutants on the delta are all killed:
- N1: finite not forced to configured.
- N2: finite configured only under a clock-configured lock, the old M18 behaviour.
- M17, M19, M21, M24, M25 (re-run).

No full suite this round; the coordinator gates it.
