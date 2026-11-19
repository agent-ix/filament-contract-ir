---
id: SR-17940486709564847048074
title: "code-review and rust-review of QSL-2400 TemporalBinding trace member"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1051 at 93e09980 (base origin/main 71914a07); crates/filament-contract-model/src/checked_package/v2/temporal_binding.rs, tests/it/checked_package_v2_temporal.rs, tests/it/checked_package_v2_canonical_encoding.rs, tests/it/support/checked_package.rs, tests/conformance_fspec/timed_clock_no_row.rs"
review_set: subset
---

## Summary

Code review, with the rust-review lane folded in, for filament-contract-ir PR #1051. Ticket: QSL-2400. Requirement source, read only: filament-specification origin/main 0bf58005. Sections read: FR-322 "Hyper clause rows", FR-322-AC-294 and FR-322-AC-453, TC-1680, FR-402 "Body wrapper" and "Hyperproperty step", and the `TemporalBinding` def in `proposals/checked-package-v2/schema.json`. The FR-322 text the coder read at f4583226 is unchanged for these ACs at 0bf58005. The only FR-322 diff between them is in FR-789/790 and AC-461.

The coordinator relayed a draco:QSL:arch ruling (FSpec FR-322-AC-463/464, TC-2027/2028, W52). Under it, every hyper row is keyed on the hyperproperty node, finite-domain hyper clauses keep per-trace rows, and a new case admits two different `sequence_authority` values. That ruling replaces `hyper_wrappers`/`traces_of`, the wrapper keying, and the coder's two reported gaps. This review covers only what holds independent of those cases. A delta pass on the new head follows.

Gates, all run in the scratch worktree CIR-1051-review at 93e09980 (FILAMENT_SPECIFICATION_DIR at FSpec 0bf58005, QUIRE_LOCK_CHECKER quire-0.52):
- `cargo fmt --all -- --check`: clean.
- `cargo clippy --locked --workspace --all-targets --features conformance-cli -- -D warnings`: clean.
- `cargo clippy -p filament-contract-model -- -D warnings`: clean.
- `cargo test --workspace --all-targets --features conformance-cli --no-fail-fast -- --include-ignored`: 2193 passed, 16 failed.
- The same run on clean main 71914a07: 2185 passed, 15 failed.
- The failing sets are identical except for one new head failure (FND-001). The 15 shared failures (probabilistic_delay x5, whole_read_accounting x3, timed_model x6, temporal_binding_public x1) and the 4 shared `make conformance-fspec` selected_closure_walk failures fail identically on clean main. The ones sampled refuse `UnknownMember` at `/claim_names`, an FSpec member that open PR #1046 adds. They are pre-existing, not this PR.
- Doctests: pass on both.

## Examined scope

- `TemporalBinding.trace` and its serde attributes (examined). The `present_trace` deserializer (`Box::<str>::deserialize(..).map(Some)`) with `default` and `skip_serializing_if` gives the schema's optional `Nonempty`: absent maps to `None`; null and non-string are rejected; a duplicate member is rejected by serde derive under `deny_unknown_fields`. Hand mutant M01 is killed.
- `Encode for TemporalBinding` (examined). `trace` is written only when present. The canonical-bytes test compares against quire-canonical (M02 killed).
- `validate_shape` empty-trace check (examined): empty refuses `malformed_wire` at `trace` (M03 killed).
- The row loop's trace match, key `(position, occurrence, trace_at)`, conflicting-definition and order checks (examined). They match FR-322 "Hyper clause rows" and AC-294's loci, and M04 to M07 and M11 are killed. The out-of-order locus `/trace` is an IR choice; see spec-review FND-002.
- Missing-row coverage with `rows_needed` (examined). M08 and M09 are killed.
- `hyper_wrappers` (examined, rewritten by the ruling): FND-001, FND-002, FND-004. M12 to M14 survive but are equivalent: other claim forms carry no `quantifiers`/`body` bindings, a non-clause body target cannot carry a row (refused at `declaration`), and a non-text literal has no `as_str`.
- `refresh_temporal_bindings` test helper (examined): flat-maps occurrence by trace in quantifier order, and the stable sort keeps the trace order.
- Public API (examined): FND-003.
- Probes (scratch only, removed): repeated trace variable (FND-002). A traceless pre-PR producer row on a wrapper refuses `timed-node-reference` at `/temporal_bindings/0/trace` (FND-003). A zero-quantifier hyperproperty with a traceless row refuses at `trace` before the hyperproperty step's `invalid_semantic_graph`. That follows FR-322-AC-294's stated order (coverage precedes body steps), so it is not a finding.

## Verdict

The reader, writer and serde changes for `trace` are correct and well tested: 11 of the 14 hand mutants are killed and 3 are equivalent. fmt and clippy are clean. The PR is not mergeable at 93e09980:

- FND-001 (high): the string-edge gate fails, so `make lint` fails and the test suite gains a failure that clean main does not have. The coder's uncommitted rework in the PR worktree (`hyper_nodes`) still trips the same rule at temporal_binding.rs:537.
- FND-002: a repeated trace variable is silently deduplicated and admitted with one row. It needs a draco ruling (see spec-review FND-004) and then a test.
- FND-003: the downstream break needs a breaking tag landed with QSL-2277's FSL writer.

The wrapper-keying parts are superseded by the W52 ruling and get the delta pass.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The string-edge gate fails at head. `hyper_wrappers` gates a branch on the raw string comparison `AsRef::<str>::as_ref(name) == "variable"` with no `#[ix_trace_rs::string_edge]` marker and no rules entry. `fsl-string-edge` reports "1 unmarked, unlisted occurrence(s): temporal_binding.rs:531 in hyper_wrappers (branch-gating)". So `make lint` fails, and `string_edge_gate::actual_source_roots_are_clean_under_the_authoritative_scanner` fails at head but passes on clean main 71914a07. It is the only new failure in the full workspace run. The coder's uncommitted rework trips it again at :537 in `hyper_nodes`. Reuse the hyperproperty step's typed Group reading (temporal.rs:2468 checks `["variable", "quantifier", "alias"]`) or mark the helper as the other v2 modules do (dispatch_projection.rs:446) | crates/filament-contract-model/src/checked_package/v2/temporal_binding.rs:529-531 |
| FND-002 | medium | A hyperproperty whose two quantifier Groups bind one variable (`s`, `s`) is admitted with one row (probe P4b: ADMITTED). With the two rows FR-322 asks for, it refuses `conflicting-definition` at `/temporal_bindings/1` (P4). `variable.filter(\|name\| !variables.contains(name))` silently collapses the Groups. FR-322 requires "exactly one row per quantifier Group, keyed by its variable", which no row set can satisfy for a repeated variable, and the hyperproperty step does not refuse the repeat. The reader picks an outcome the spec does not state, and no test pins it (mutant M10, removing the dedupe, survives). Needs the draco ask in spec-review FND-004, then a test of whichever outcome is ruled | crates/filament-contract-model/src/checked_package/v2/temporal_binding.rs:544 |
| FND-003 | medium | Breaking public API flag day. `TemporalBinding` is not `#[non_exhaustive]` and gains `pub trace`. filament-spec-language origin/main builds it by struct literal at fsl-package/src/emit.rs:8475, 8547, 8610, 8678, plus fsl-package emit tests (plan_subject_selection.rs:200, retained_rows.rs:143, 161) and fsl-eval tests. It destructures it exhaustively at fsl-package/src/emit/monitor_plan.rs:1115 and 1140. filament-contract-codegen tests/checked_package_support/temporal.rs:63 also builds it. All fail to compile at the next contract-ir pin bump (both pin v0.1.5). Semantically, FSL's hyper emitter (emit.rs:8486-8553) writes one traceless row per wrapper occurrence, which this reader refuses at `/temporal_bindings/{i}/trace` (probe P5), so every FSL event-position hyper package is refused after the bump. Ship it as a breaking tag landed together with the QSL-2277 FSL writer, and record it in FR-019 or the PR | crates/filament-contract-model/src/checked_package/v2/temporal_binding.rs:66-85 |
| FND-004 | low | Unmetered quadratic scans. `traces_of` linearly scans every wrapper for each row and again for each node occurrence in the coverage loop, so the cost is O(rows × hyperproperties). `variables.contains` is O(groups²). Neither is charged to the `WorkMeter`, though the surrounding occurrence loop charges `binding_work` per visit (FR-358 / TC-472 exact charging). The W52 rework keys rows on the hyperproperty node, so it can index the trace list by node position (one lookup per row) and should charge the Group scan | crates/filament-contract-model/src/checked_package/v2/temporal_binding.rs:568-573,688 |

## New findings (disposition pass 1)

Round 1, head f7cf238c (origin/main a15bf349 merged). FSpec origin/main is 57574468, which contains W52/W54 f92839de.

Gates at f7cf238c:
- `make string-edge`: clean.
- fmt and both clippy lanes: clean.
- Full workspace test: 2252 passed, 18 failed. Clean main a15bf349: 2234 passed, 18 failed. The failing sets are identical: all 18 are `monitor_plan::reached_model_read`.
- `make conformance-fspec`: the same 5 failures at head and on main.
- quire validate: the same 10 grammar findings as main.
- `coverage --strict`: 70 unbacked rows and 0 contradicted, as on main. AC-561 has 18 trace claims.

Hand mutants M01 to M26, run against `--test it tc_483` and the lib round-trip test:
- 19 killed.
- 4 survive as equivalent on spec-valid input: M12 (any claim form), M13 (wrapper target kind), M14 (non-text variable), M23 (unreadable wrapper counts as configured).
- 3 survive as real test gaps: M18, M19, M24 (see gap-analysis).

Probes, scratch only:
- Behaviours `s`, `s` with one row is admitted. That is the expected interim outcome until FR-402-AC-10 lands.
- A body that references the formula instead of a clause refuses `timed-node-reference` at the clause's body, before the hyperproperty step. That follows FR-322-AC-294's order, so it is not a finding.
- A finite first or middle row with a different `clock_name` refuses at that row's `/clock_name`.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | Stale field docs on the public `TemporalBinding`. `trace` reads "For a hyper clause body wrapper's row, the exact name of the quantified trace variable", but under W52 a body wrapper has no row and the row is keyed on the `claim`/`hyperproperty` node. `declaration` still reads "Exact temporal clause or compensation declaration ID" and does not name the `hyperproperty` node (or the await) that FR-322 now keys rows by. Rustdoc is the public contract for FSL and codegen | crates/filament-contract-model/src/checked_package/v2/temporal_binding.rs:70,78-79 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |
| FND-004 | fixed | - |
| FND-005 | still-open | New this round: the doc text was not updated for W52 keying |
| FND-005 | fixed | - |

## Disposition pass 2 notes

Round 2 (disposition pass 2), head 8cded29d: fix commit 0b8cf26e, plus a clean merge of origin/main 9edbd391. FSpec is origin/main ebfcb277. Gates at 8cded29d:
- `make string-edge`, fmt and both clippy lanes: clean.
- Full workspace test: 2251 passed, 21 failed. Clean main 9edbd391: 2231 passed, 21 failed. The failing sets are identical: 18 `monitor_plan::reached_model_read` and 3 `public_interface`.
- `make conformance-fspec`: the same 6 failures on both.
- quire validate and `coverage --strict`: identical to main (89 unbacked rows, 0 contradicted). AC-561 has 20 trace claims.
- Hand mutants M01 to M26 at the new head: 21 killed, now including M19 and M24. M12, M13, M14 and M23 remain equivalent survivors. M18 survives, deferred.

## Disposition pass 3 notes

Round 3 (disposition pass 3); no finding of this SR is open, so no row is added, head 71892dd4. The delta since 8cded29d is fix commit 9d4bcfa7, a merge of origin/main (including #1045 `validate_clock_configuration`), and the 71892dd4 `lock_tables` test literal `trace: None`. Per the arch ruling, a finite-domain hyperproperty always requires its rows. Under a lock whose selected `temporal_profile` is not clock-configured, it refuses `invalid_package`/`timed-node-reference` at `/semantic_graph/nodes/{N}/body`. Test `tc_483_a_finite_domain_node_under_a_profile_without_a_clock_refuses_at_its_body` and the AC-561 note cover this. FSpec is FSPEC-9a181ace.

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
