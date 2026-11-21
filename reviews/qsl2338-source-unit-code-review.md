---
id: SR-17940486709564847048129
title: "code-review of QSL-2338 clause source_unit row member (code/qsl-2338-clock-unit-owner)"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; branch code/qsl-2338-clock-unit-owner at dff7dfd7b (git diff origin/main...HEAD, main c353c3f38); crates/filament-contract-model/src/checked_package/v2/{temporal_binding.rs,temporal.rs,lock_tables.rs}; tests/conformance_fspec/{temporal_aggregate.rs,temporal_clock_bindings.rs,timed_clock_no_row.rs}; tests/it/{checked_package_v2_protocol_body.rs,support/checked_package.rs}; tests/fixtures/{qsl-2645-golden/package.json,qsl-2705-ieee-operand/temporal-holds-let.json}; against FSpec PR #777 head 74e91f2e"
review_set: subset
---

## Summary

Ticket: QSL-2338. This is a code-review with the rust-review lane folded in. It covers the typed `TemporalBinding.source_unit` member, the strict-reader row check, the canonical writer, and the change from inferring the clock unit's owner from the subject to reading it from the row.

What was examined:

- `temporal_binding.rs:676-695`, the row check. The arm order is declaration join, timed-model row, occurrence join, then `source_unit`, then trace, repeated key and row order. That matches FR-322's row visit ("first its declaration join, then its occurrence join, then its `source_unit` ..., then whether its key repeats"). The binding stage (`mod.rs:4114`) runs before `validate_temporal` (`mod.rs:4290`), so SU-04 refuses before any unit resolution, as FR-778-AC-11 requires. The new compensation refusal sits at the row, not at `/clock/kind`, so the deferral of a missing protocol temporal selection (`mod.rs:4126-4138`) does not swallow it.
- Lock-source identity. `lock.sources.contains(unit)` uses the derived `PartialEq` on `CheckedSourceRef { authority, identity }`. That is exact equality of both fields, with no substring and no case folding. The test probes `{other, example}` and `{agent-ix, Example}` and both refuse.
- The writer. `Encode` writes `source_unit` after `clock` and before `trace`. `temporal_clock_writer_retains_a_configured_clause_row` round-trips the row, and so does `published_temporal_occurrence_vectors_bind_exact_read_and_write`.
- `temporal.rs:1916-1938` and `1972-2040`. Each row's `timestamp_unit` is now resolved in that row's own `source_unit`, and the subject is no longer read.
- Breaking API. `TemporalBinding` is a public struct without `#[non_exhaustive]`, and FR-019 and FR-038-AC-590 record the new field.

Tests run (name-filtered, `flock /tmp/cargo-slot.lock`, `-j 4`, `--features conformance-cli`, `FILAMENT_SPECIFICATION_DIR` = the FSpec PR #777 checkout):

- `conformance_fspec`, filtered on source_unit, clock-unit, clock-binding, timed-no-row and timed_slots: 21 of 22 pass. The one red is `duration_bounds_match_every_occurrence_clock_unit`.
- `it`, filtered on compensation/await clock, temporal_binding, probabilistic::join (which covers the qsl-2645 golden fixture) and timed: 37 of 37 pass.
- `filament-contract-model --lib`, temporal / temporal_binding / lock_tables: 25 of 25 pass.
- `temporal_binding_public`: 1 of 1 passes.

Red comparison for the touched groups (temporal, clock, compensation, operation_effects, probabilistic, requirement_records), each run in a `git archive` snapshot with its own `CARGO_TARGET_DIR`:

- HEAD with FSpec #777: 14 red.
- origin/main with FSpec #777: 35 red. That set contains all 14, plus the reds caused by the new `source_unit` member, which main does not read.
- origin/main with FSpec main cd89fcb3, the three temporal reds only (`duration_bounds_match_every_occurrence_clock_unit`, `published_temporal_aggregate_arguments_match_strict_reader` with AG-07/AR-19/AR-20, and `monitor_plan_refuses_a_clock_read_of_a_hybrid_subject_timed_clause`): the same three fail with the same messages.

So the diff causes none of the 14 reds. The coder said about 11; the true count is 14, the other 11 being probabilistic_claims, requirement_records and operation_effects.

Mutation analysis was done by hand, because a cargo-mutants run would rebuild the 12 GB target for each mutant. These mutants are killed:

- dropping the lock check (SU-04);
- swapping the `(None, false)` arm (the missing-member test and the CB-01 `k1` test);
- `is_clause := true` (every compensation case);
- `row_source_owner := None` (the SU-01 admit);
- dropping the writer member (the round-trip test).

One mutant survives: `find` changed to `rev().find` / last row in `source_owner_for_clause` (FND-001).

## Verdict

**Changes needed.** The row check, its loci, its order and the identity comparison conform to FR-322 "Clause source unit", FR-322-AC-503 and FR-778-AC-3..11. No valid static fixture now refuses: the `qsl-2347-emitted` fixtures hold no clause and need no `source_unit`. Two code defects remain, plus one cross-repository sequencing obligation:

- FND-001: aggregate bounds are resolved in the first row's unit only.
- FND-002: an unmetered quadratic row scan.
- FND-003: the FSL pin break.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Aggregate bounds are resolved in the first row's `source_unit` only (`source_owner_for_clause` uses `bindings.iter().find(...)`). Take a clause node with two occurrences whose rows name different locked units, both declaring `ms`. Each row passes its own `timestamp_unit` check, and the bounds typed by the first unit's `ms` node are admitted, although the second occurrence's clock unit is a different node. FR-778's last bullet refuses a bound typed by any unit node other than the clock unit node. `aggregate_clock_units` already treats a mixed `timestamp_unit` as "no shared clock" (bounds refuse, test `duration_bounds_match_every_occurrence_clock_unit`), and a mixed `source_unit` gets no equal treatment. Fold `source_unit` into the per-clause shared key, so that mixed means None and the bounds refuse, or get an FSpec ruling first (spec-review FND-001) | crates/filament-contract-model/src/checked_package/v2/temporal.rs:1600, crates/filament-contract-model/src/checked_package/v2/temporal.rs:1933-1938, crates/filament-contract-model/src/checked_package/v2/temporal.rs:1838-1870 |
| FND-002 | medium | `source_owner_for_clause` runs for every clause, including clauses with no aggregate. It runs before the `for formula in aggregates` loop and does a linear `find` over all `temporal_bindings` with full `CheckedNodeId` comparisons, and charges no work. That is O(clauses × rows) unmetered work in the strict reader. The old code was one indexed `graph.node` lookup. Compute it only when `aggregates` is non-empty, and take it from the sorted per-clause table `aggregate_clock_units` already builds (binary search), or charge the visits | crates/filament-contract-model/src/checked_package/v2/temporal.rs:1600, crates/filament-contract-model/src/checked_package/v2/temporal.rs:1937 |
| FND-003 | medium | The breaking change has no consumer sequencing. FSL `origin/main` f7b1d41bc builds `filament_contract_model::TemporalBinding` struct literals (fsl-package/src/emit.rs:9110, 9178, 9243, 9312; emit/tests/plan_subject_selection.rs:204) and emits no `source_unit`. At the next IR pin bump FSL fails to compile, and once it compiles, every FSL-emitted package with a clock-configured clause refuses `invalid_package`/`timed-node-reference` at `/temporal_bindings/{i}`. FSpec FR-778 traces to FSL FR-965. An FSL ticket for emitting `source_unit` must exist before merge. This PR also has to merge after FSpec #777: against FSpec main, every published clause row lacks the member and refuses | crates/filament-contract-model/src/checked_package/v2/temporal_binding.rs:107, crates/filament-contract-model/src/checked_package/v2/temporal_binding.rs:691 |
| FND-004 | low | `lock.sources.contains(unit)` is an unmetered linear scan for each clause row. It follows the precedent at mod.rs:5111, but it is new unmetered work, O(rows × sources). Charge it with `binding_work`, or bisect if `lock.sources` is canonically sorted | crates/filament-contract-model/src/checked_package/v2/temporal_binding.rs:684 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | The same-unit check compares each row with the latest clause row only: `clause_unit` is reset whenever a row of another clause intervenes. So `[c1/A, c2/X, c1/B]` passes the unit check at row 2 and refuses later, at `/temporal_bindings/2/declaration` (the order check). FSpec #777 at c01d61ce (FR-322:4575-4581) says a clause row "whose `source_unit` differs from that of the clause node's first row" refuses at `/temporal_bindings/{i}`, and the row visit runs the `source_unit` step before the order step. This only affects which locus an out-of-order package reports; such a package refuses either way. The confidence is low because the spec's "first row in ascending order" is not well defined for unsorted rows | crates/filament-contract-model/src/checked_package/v2/temporal_binding.rs:708-719 |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | `clause_units` is scanned linearly for each clause row, and every step is charged one unit of `binding_work`. A valid package with N single-row clauses therefore charges about N²/2 work in the binding stage, so a caller's work budget has to grow quadratically with the clause count. The work is metered, not hidden. Insert into a vector sorted by position and bisect it, for O(log N) charged work per row | crates/filament-contract-model/src/checked_package/v2/temporal_binding.rs:712-726 |

## Dispositions

Round 1, at head 16bc5fe6f.

Tests run, under the flock with `-j 4` and `--features conformance-cli`:

- Against FSpec 74e91f2e:
  - `conformance_fspec`, the temporal_aggregate / temporal_clock / timed_clock_no_row / source_unit / timed_slots filters: 25 of 27 pass. The two reds are the pre-existing `duration_bounds_match_every_occurrence_clock_unit` and `published_temporal_aggregate_arguments_match_strict_reader`, which also fail on main.
  - `it`, the compensation/await, hyperproperty_row, `checked_package_v2_temporal`, whole_read_accounting and probabilistic::join filters: 128 of 128 pass.
- Against FSpec PR #777's current head c01d61ce: `source_unit_vectors_resolve_the_clock_unit_in_the_named_unit_only` fails (gap-analysis FND-006).

Mutation claims:

- The two equivalent mutants are both genuinely equivalent.
  - `ClauseClock.source` taken from the first row versus the last row: the binding stage (mod.rs:4114) refuses a mixed-unit clause before `validate_temporal` (mod.rs:4290). The deferral at mod.rs:4126-4138 only defers `/clock/kind` compensation refusals, and the read still refuses, so no admitted outcome can differ.
  - Dropping `.filter(|_| !aggregates.is_empty())`: with no aggregates the loop that uses the owner does not run.
- The new same-unit check is killed by `every_row_of_one_clause_names_one_source_unit` ([A,B,A] refuses at row 1, [A,A,B] at row 2).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | temporal_binding.rs:708-719: "if is_clause { match clause_unit { Some((first, unit)) if first == position => { if unit != &row.source_unit { return Err(invalid(meter, at(), &row.declaration)); } } _ => clause_unit = Some((position, &row.source_unit)), } }". FSpec #777 c01d61ce now states the rule (FR-322:4575-4581, FR-778-AC-12, SU-05). The test fixture's second locked unit declares no `ms`. That does not weaken the row rule, which reads no declarations |
| FND-002 | fixed | temporal.rs:1604-1609: "let source_owner = clock.source.filter(\|_\| !aggregates.is_empty()).map(\|unit\| { super::CheckedNodeOwner::Source { ... } });". The owner comes from the `ClauseClock` in the `aggregate_clock_units` table, found by the existing binary search. That table is charged through `meter.vec`, which reserves `size_of::<T>()` per row, so the wider tuple is charged. No per-clause linear scan remains. The exact-work boundary tests pass |
| FND-003 | deferred | Cross-repo sequencing, out of scope for this IR PR. An FSL ticket for emitting `source_unit` (FSL FR-965), covering the struct literals at fsl-package/src/emit.rs:9110, 9178, 9243 and 9312, must exist before merge, and this PR must merge after FSpec #777. The code lead has to name the FSL ticket |
| FND-004 | fixed | temporal_binding.rs:688-695: "for source in &lock.sources { binding_work(meter, at().key(\"source_unit\"))?; if source == unit { locked = true; break; } }" |

Round 2, at head f90109938, tested against FSpec #777 at c01d61ce (`/home/peter/dev/worktrees/FSPEC-QSL2338`). FND-001, FND-002 and FND-004 are already `fixed`, and FND-003 is already `deferred`; their latest rows stand. The arch decision to put the sequencing line in the PR body (FSL emission rides QSL-2338; FSpec #777 merges first) is consistent with FND-003's deferral.

| FND | outcome | reason |
| --- | --- | --- |
| FND-005 | fixed | temporal_binding.rs:712-726: "let mut first = None; for (clause, unit) in &clause_units { binding_work(meter, at())?; if *clause == position { first = Some(*unit); break; } } match first { Some(unit) if unit != &row.source_unit => { return Err(invalid(meter, at(), &row.declaration)); } Some(_) => {} None => clause_units.push((position, &row.source_unit)), }". `interleaved_clause_rows_keep_the_first_source_unit_per_clause`: [c1/A, c2/A, c1/B] refuses at `/temporal_bindings/2`, before the order check |
