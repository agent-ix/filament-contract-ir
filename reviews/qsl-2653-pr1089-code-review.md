---
id: SR-17940486709564847048104
title: "code-review of QSL-2653 lazy probabilistic wire failure (PR #1089)"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1089 at 1dc26097b (git diff origin/main...HEAD); crates/filament-contract-model/src/checked_package/v2/probabilistic.rs; tests/it/checked_package_v2_probabilistic.rs; tests/it/checked_package_v2_source_objects/probabilistic.rs; tests/it/checked_package_v2_whole_read_accounting.rs"
review_set: subset
---

## Summary

Ticket: QSL-2653. This is a code review with the rust-review lane folded in.

The diagnosis holds. `probabilistic::wire` is `Fail::Wire(Box::new(at))`, so `.ok_or(wire(..))` made one heap request on every successful `spelling` and `aggregate` call. The fix (`ok_or_else`) is the whole production change. A sweep of the v2 reader (`crates/filament-contract-model/src/checked_package/`) for eager error construction found no other allocating site. The searches covered `.ok_or(`, `.unwrap_or(`, `.map_or(Err(`, `.or(`, multi-line `ok_or(` and `let x = wire|invalid|refuse(..)`. The remaining eager arguments are fieldless or `const fn` values (`ModelRefusal::missing_name/malformed`, `Error::*`), stack `Path<'static>` values (`step_relation.rs`) or borrowed refs (`model_population_catalog.rs`). The `Box::new(at.clone())` sites (`selected_facts`, `equality_projection`, `monitor_plan`, `runtime_type_projection`, `exact_projection`) are inside error constructors reached only on failure. `probabilistic.rs` has no other production call to `wire(`/`invalid(` outside `Err(..)`/`ok_or_else`.

Tests run on one cargo slot with a private CARGO_TARGET_DIR, FSpec origin/main 73a169ff and QUIRE_LOCK_CHECKER=quire-0.52: `cargo test --locked --features conformance-cli --test it -- unpaid_heap_of_workload_rows a_wide_model_owner_workload the_source_buffers_are_charged every_registered_input_is_accounted_within_the_bound_at_core_bounds every_registered_positive_and_refusal_is_a_genuine`. All 5 passed.

A scratch probe, not pushed, printed outcome and unpaid bytes in the growth test. All six reads were Admitted. Unpaid heap was source 0/0/0 and model-owner 63/63/63 at 20/300/600 rows, so the margin under 4096 is large. With the eager `ok_or` restored in `spelling`, source unpaid was 14520/162360/320760 and model-owner 10887/158727/317127, which is 528 B more per row. The growth test failed on that mutant, so it is not vacuous.

## Verdict

**MERGEABLE.** The production fix is correct and complete for the reader. It changes no public API: `graph_requests_of` is `pub(crate)` inside the `it` test crate. The fixed 264 B `Box<Path>` on the failure path does not depend on input size, and FR-038-AC-511's fixed-size-diagnostic clause covers it. The registry fixtures exercise the bound at core bounds through `assert_read` with `Expected::Admitted`: `TC048/source_probabilistic` (520 params, 257 rows, with `delay_poke`/`random_k` kept) and `TC048/model_owner_workload` (300 rows). The coder reported 4 killed mutants. One of them (eager `spelling`) was reproduced here, and it is adequate. Two low test-quality findings follow; neither blocks merge.

Overlap: QSL-2645 (#1086, open) also edits `tests/it/checked_package_v2_source_objects/probabilistic.rs` and `spec/checked_package/matrix/tests.md`. The hunks are disjoint (module doc, `package_over`, `mod join`, and the FR-369/TC-484 rows, against `wide`/`accounting_fixture`/`graph_requests_of` and the FR-038-AC-511 row), so a textual conflict is not expected. #1086's join is a post-read API and is not measured by this registry. Its own unpaid join storage is SR-17940486709564847048101 FND-002.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | `unpaid_heap_of_workload_rows_and_rational_spellings_does_not_grow` never asserts that each of the six reads is `Admitted`. Only the 257/300-row registry cases and the 300-row charge test pin admission. If a 20- or 600-row fixture regressed to an early refusal, the equal-unpaid check could pass vacuously. A probe shows all six admit today | tests/it/checked_package_v2_whole_read_accounting.rs:324-334 |
| FND-002 | low | `a_wide_model_owner_workload_is_admitted_and_its_named_buffer_charged` pins a per-site charge amount (one `probabilistic` request of 16x300 B). That is the FR-358-AC-12 / FR-369-AC-27 shape its sibling `the_source_buffers_are_charged_before_they_are_made` is tagged with, but this test is tagged only FR-038-AC-511. Its doc also says "at the workload node", which the test does not check (form only) | tests/it/checked_package_v2_probabilistic.rs:609-637 |
