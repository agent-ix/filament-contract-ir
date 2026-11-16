---
id: SR-17940486709564847048033
title: "gap-analysis of QSL-2254 timed ClockConfiguration kind"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1030 at 68d9732e; FR-038-AC-482, FR-038-AC-483, FR-038-AC-484, FR-038-AC-437, crates/filament-contract-model/src/checked_package/v2/{temporal_binding.rs,timed_model.rs,intake.rs,monitor_plan.rs}, tests/conformance_fspec/timed_clock_no_row.rs"
review_set: subset
---

## Summary

Gap analysis scoped to the PR diff. Ticket: QSL-2254 (AC 1: `ClockConfiguration::Timed` round-trips through reader and writer; AC 2: an unknown clock kind is still refused). Plan completion: not assessed. Vectors read by reference from filament-specification origin/main 959a1982 (`timed-clock-no-row-vectors.json` version 2).

Mutation checks, each run against `conformance_fspec timed_clock_no_row` (and `hybrid` for M5), source restored after each:

- M1 (trace timed clause not clock-configured, no row required): the stub fails; the row-less TN-02 package is admitted where `refusal` expects a refusal.
- M2 (whole-row refusal of a non-`timed` or `model-time` trace row removed): the stub fails; a TN-08/TN-09 package is admitted where a refusal is expected.
- M3 (non-empty `timed` unit check removed): the stub fails; the empty-unit package is admitted.
- M4 (intake `"timed" => "unit"` reverted): the stub passes and the whole workspace run shows only the environment failures (lock_coherence: `quire check-lock` absent; string_edge_gate: scanner off PATH in that run), so no test kills it (FND-002).
- M5 (hybrid not counted as a model subject): ten hybrid conformance tests fail (the hybrid timed clause refuses at its body for a missing row); the stub still passes.

## Examined scope

- FR-038-AC-482 (examined): TN-01, TN-02 (body refusal with the clause index), TN-03, TN-05, TN-06 (admission with the exact typed row and `model_time_for_clause` absence) are asserted at the published oracle. TN-02 changed from admission to refusal because the published vector changed (version 2: `expected` is `invalid_package`/`timed-node-reference` at `/semantic_graph/nodes/{clause-index}/body`); this follows the vector, it is not a weakening.
- FR-038-AC-483 (examined): TN-04, TN-07, TN-08 and TN-09 asserted at `/temporal_bindings/0` with code and cause from the vector.
- FR-038-AC-484 (examined): the stub checks `model_time_for_clause` absence for the trace clause; the monitor clock read of a timed trace clause returning `Configured(Timed)` has no test (FND-003); FND-001 of the code-review SR covers the hybrid case.
- FR-038-AC-437 (examined): the unit test `timed_clock_round_trips_and_refuses_violating_documents` is tagged to it; see FND-001.

## Verdict

Coverage of the TN vectors is complete and each new check is killed by a test. One medium gap: the ticket's round-trip and unknown-kind criteria, and the closed `timed` shape, are stated only in FR-038 prose with no acceptance criterion, so the tests that assert them are tagged to criteria that do not state them. A second medium gap: AC-484's new clause, the monitor read of a timed trace clause exposing its `timed` row, has no test. One low gap: the intake locus for a non-text unit has no test.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The `timed` closed shape, empty-unit malformed_wire, unknown-kind refusal and reader/writer round-trip (ticket AC 1 and AC 2) have no FR-038 AC; the new FR-038 prose states them, but the unit test is tagged FR-038-AC-437 (TemporalBinding member closure and occurrence) and AC-482, neither of which states them | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1940-1948; crates/filament-contract-model/src/checked_package/v2/temporal_binding.rs:1181-1210 |
| FND-002 | low | Reverting the intake mapping of a non-text `timed` unit (intake.rs `"fixed_sample" \| "timed" => "unit"`) fails no test; no reader-level case sends `{"kind":"timed","unit":3}` and asserts `/temporal_bindings/{i}/clock/unit` | crates/filament-contract-model/src/checked_package/v2/intake.rs:497; tests/conformance_fspec/timed_clock_no_row.rs:238-278 |
| FND-003 | medium | FR-038-AC-484 now states the monitor claim clock read exposes the configured `timed` row for a Timed trace clause (replacing the removed TraceTime), but no test calls clock_for_claim on a timed trace clause; tests/conformance_fspec/monitor_plan.rs covers only Configured rows of other profiles and the ModelTime case | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6556; tests/conformance_fspec/monitor_plan.rs:355-420; crates/filament-contract-model/src/checked_package/v2/monitor_plan.rs:466-488 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | New FR-038-AC-545 covers the closed `{kind: timed, unit}` shape, the reader/writer round-trip and the refusals of a missing, non-text or empty unit and of an extra member. New FR-038-AC-546 covers the unknown kind at `/clock/kind`, the non-text unit at `/clock/unit` and the empty unit. Both have a TC-048 section and a matrix row. `timed_clock_round_trips_and_refuses_violating_documents` is now tagged AC-545/AC-546, not AC-437/AC-482, and the stub carries AC-545/AC-546. |
| FND-002 | fixed | The new stub case `{"kind":"timed","unit":3}` asserts malformed_wire at `/temporal_bindings/0/clock/unit`. I checked the coder's equivalence claim myself: eight non-text units (3, null, true, [], {}, 1.5, ["ms"], {"a":"ms"}), a reordered object and an extra member each give the same code and pointer with and without the `\| "timed"` arm. `invalid_clock_field` has one caller, intake.rs:420, and acts only on a MalformedWire path that ends at `clock`. The decoder already locates a bad `timed` unit at `/clock/unit`, so for `timed` that path never ends at `clock`. The repo has no other wire reader for clocks (its only Python is under tools/read_path_proof and docs/images). Removing the dead arm is correct. |
| FND-003 | fixed | `monitor_plan_reads_the_timed_row_of_a_trace_clause` reads a timed trace clause through `clock_for_claim` and asserts `Configured(&row.clock)` with the `Timed{unit:"tick"}` row and clock name `plant`. Mutation H2 (refusing a timed clause that has no model time instead of reading its row) makes it fail. |

