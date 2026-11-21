---
id: SR-17940486709564847048130
title: "gap-analysis of QSL-2338 clause source_unit against FSpec FR-778 and FR-322-AC-503"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; branch code/qsl-2338-clock-unit-owner at dff7dfd7b (git diff origin/main...HEAD); FSpec PR #777 head 74e91f2e FR-322 'Clause source unit', FR-322-AC-359, FR-322-AC-503, FR-778-AC-1..AC-11, temporal-clock-binding-vectors.json source_unit_cases SU-01..SU-04, timed-clock-no-row-vectors.json; IR FR-038-AC-590; tests/conformance_fspec/temporal_aggregate.rs, temporal_clock_bindings.rs, timed_clock_no_row.rs, tests/it/checked_package_v2_protocol_body.rs"
review_set: subset
---

## Summary

Ticket: QSL-2338. This is a planless gap analysis. Plan completion was not assessed.

`quire matrix` (quire 0.36.2; 0.52 is not installed) shows FR-038-AC-590 tagged by `clause_row_requires_a_locked_source_unit` (temporal_aggregate.rs:793). The FSpec criteria were read from the `#[trace]` tags directly.

Examined, one line per criterion:

- FR-322-AC-359: closed row with `source_unit`. Tagged on `temporal_clock_binding_vectors_read_at_their_exact_loci`. The schema refusals in that test still hold.
- FR-322-AC-503. Missing member: asserted at `/temporal_bindings/0`. CB-01 `k1` with the member: asserted at its row. SU-04: asserted at `/source_unit`.
- FR-778-AC-1. The admit and "no `["scalar_type","unit","ms"]` node" are asserted. The `s` bound refusal asserts the code and cause but not the locus.
- FR-778-AC-2. Tag added. The first half (FR-370-AC-20) is asserted, the second half is not (FND-002).
- FR-778-AC-3, AC-4 and AC-5: each asserted with code, cause, the `timestamp_unit` locus and the clause locus.
- FR-778-AC-6: "only the row refusal" asserted on the code; "retains the clock name and `T`" asserted only as the clause locus.
- FR-778-AC-7: TCB-03 in the clock-binding vectors test.
- FR-778-AC-8..11: `source_unit_vectors_resolve_the_clock_unit_in_the_named_unit_only` runs all four SU cases, and the case count is asserted as 4. No case is skipped.
- FR-778-AC-9 also has its own record-subject test.

## Verdict

**Gaps.** Every SU vector runs and every refusal locus matches. What is missing:

- The SU harness drops half of each vector, so SU-03 does not test the subject-unit inference this PR removes.
- The FR-778-AC-2 tag overclaims.
- The await and hyperproperty arms of the new row rule are untested.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The SU harness reads only `time_units` (as "has `example`?") and `row.source_unit`. It ignores `subject`, `bound_unit` and `lock_sources`. For SU-03 it puts `ms`/`s`/`Time` under an `agent-ix/other` source owner that it also adds to `lock.sources` (`add_other_source_units`). The vector instead has the units declared by `acme/orders`, the unit that declares the clause's subject, and `lock_sources` = [`example`]. The clause subject is never declared by the unit that holds the time units, so AC-10's "never in the subject's unit" is not exercised: the removed subject-inference code also refuses this SU-03 shape. Build SU-03 with the subject's declaring unit owning the units, and assert `lock_sources`/`bound_unit` from the vector | tests/conformance_fspec/temporal_aggregate.rs:686-703, tests/conformance_fspec/temporal_aggregate.rs:738-771, tests/conformance_fspec/temporal_aggregate.rs:165-189 |
| FND-002 | medium | `duration_bounds_are_typed_by_the_clock_unit_in_its_owning_package` gains the tag `FR-778-AC-2`, but it asserts only the FR-370-AC-20 half, with a bound typed by the model `s` node. The AC's second clause is not asserted: "with its unit of compilation also declaring a source `ms`, a bound typed by that source node refuses `ill_typed`/`unit-mismatch` at the bound's value". The matrix shows AC-2 covered | tests/conformance_fspec/temporal_aggregate.rs:1081, tests/conformance_fspec/temporal_aggregate.rs:1110-1135 |
| FND-003 | low | Three assertions are weaker than their criteria. FR-778-AC-1's `s`-typed bound and FR-778-AC-6's "bound typed by `s` beside the bad row" check the code and cause but not the locus (`upper`'s value; the row's `timestamp_unit`). The "AC-6: the clock name and T are retained" comment is backed only by `refusal.locus == clause` | tests/conformance_fspec/temporal_aggregate.rs:878-880, tests/conformance_fspec/temporal_aggregate.rs:896-899, tests/conformance_fspec/temporal_aggregate.rs:931-936 |
| FND-004 | low | The new rule refuses a `source_unit` on a compensation, an await or a `hyperproperty` row (FR-322 "Clause source unit"; FR-038-AC-590 names all three). Only the compensation case (CB-01 `k1`) is tested. The await (AW-01) and hyper row cases have no assertion | tests/it/checked_package_v2_protocol_body.rs:8299-8317 |
| FND-005 | low | Two harnesses stamp `source_unit` instead of reading it. `compensation_clock_case` replaces the published CB-01 clause row's `source_unit` with `lock.sources[0]` of its own base package, and `timed_clock_no_row` stamps `lock.sources[0]` onto the TN rows, which in FSpec #777 carry no member (spec-review FND-002). A vector `source_unit` that disagreed with the published lock would therefore never be seen. Compare the published value against the base package's lock, or comment the FSpec gap at the stamp | tests/it/checked_package_v2_protocol_body.rs:8210-8214, tests/conformance_fspec/timed_clock_no_row.rs:196 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | medium | FSpec #777 has moved past the reviewed head 74e91f2e to c01d61ce. That head publishes `SU-05` (FR-778-AC-12, two rows of one clause naming `example` and `acme/orders`, refused at `/temporal_bindings/1`, with a `rows` member and not `row`). It also adds `source_unit` to the TN rows. Run against c01d61ce, `source_unit_vectors_resolve_the_clock_unit_in_the_named_unit_only` fails at `assert_eq!(cases.len(), 4)` (temporal_aggregate.rs:791). SU-05 is covered only by the in-house `every_row_of_one_clause_names_one_source_unit`, not read from the vector. The TN harness still overwrites the published `source_unit` with `lock.sources[0]`. Consume SU-05 (with `rows`), and read the TN rows' published `source_unit` before merge | tests/conformance_fspec/temporal_aggregate.rs:791, tests/conformance_fspec/timed_clock_no_row.rs:196-199 |

## Dispositions

Round 1, at head 16bc5fe6f.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | The vector fields are now consumed: subject, `bound_unit`, `lock_sources`, row clock and `source_unit`. But SU-03 still does not fail under the removed inference. On main, `source_owner_for_clause` took the owner of the clause node's `semantic_type`. In the published fixture that is `Example.Flag`, owned by `Source{agent-ix, example}`, so main inferred `example`. SU-03's units are in `acme/orders`, so main also refuses `missing-name` and passes. SU-01 and SU-02 also admit under main, because their units are in `example`. No case puts the time units in the unit that the old inference returns while `source_unit` names a different unit. The departure for SU-03 also turns `acme/orders` from a selected model into a locked source unit. That does not hide a mismatch for its expected outcome, but it is why the case cannot discriminate. To kill that mutant, add one discriminating case: units in `example` (the clause's `semantic_type` owner), with the row's `source_unit` naming a second locked unit that declares nothing. That refuses `missing-name`, and main's inference would admit |
| FND-002 | fixed | temporal_aggregate.rs (end of `duration_bounds_are_typed_by_the_clock_unit_in_its_owning_package`): "FR-778-AC-2, second half: with the unit of compilation also declaring a source `ms`, a bound typed by that source node refuses at the bound." It asserts `ill_typed`/`unit-mismatch` at `.../members/{upper}/value` |
| FND-003 | fixed | "for (name, package) in [(\"us\", &us), (\"m\", &length), (\"affine\", &affine)] { let mut with_bound = package.clone(); type_upper(&mut with_bound, id(&[\"s\"])); assert_eq!(unit_refusal(&with_bound), unit_refusal(package), \"{name}\"); }" and "assert_eq!((code, cause, at), (\"ill_typed\", Some(\"unit-mismatch\"), Some(upper)));" |
| FND-004 | still-open | The compensation case (every row without `source_unit` in each admitted CB case) and the hyperproperty case (`a_hyperproperty_row_naming_a_source_unit_is_refused`) are now covered. The await case is not: the published `compensation_cases` CB-01..CB-08 hold no await row (only `k1`/`k2`/`c`/`sd`), so the loop's comment "a compensation or await row" names a case it never visits. Add an await-row assertion |
| FND-005 | fixed | CB-01 keeps the published `source_unit`, and the harness locks it: "if let Some(unit) = row.get(\"source_unit\").cloned() { ... sources.push(unit); ... }". The TN stamp is documented at timed_clock_no_row.rs:196-198. The TN rows' published values now exist upstream and are tracked in FND-006 |

Round 2, at head f90109938, tested against FSpec #777 at c01d61ce. `conformance_fspec`, filtered: `source_unit_vectors_resolve_the_clock_unit_in_the_named_unit_only` (SU-01 to SU-05), `timed_clock_no_row_vectors_join_model_and_refuse_extra_row`, `a_named_unit_that_declares_nothing_refuses_the_clock_unit`, `every_row_of_one_clause_names_one_source_unit` and `interleaved_clause_rows_keep_the_first_source_unit_per_clause` all pass. Overall 27 of 29 pass; the two reds are the pre-existing main reds.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `a_named_unit_that_declares_nothing_refuses_the_clock_unit`: "let mut package = su_package(\"example\", EXAMPLE_UNIT()); ... package[\"temporal_bindings\"][0][\"source_unit\"] = other;" refuses `missing_declaration`/`missing-name` at `/temporal_bindings/0/clock/timestamp_unit`. The units sit in `example`, the owner of the clause's `semantic_type` (`Example.Flag`) that main's removed inference returned, so main would admit this package and the case kills that mutant |
| FND-004 | fixed | `await_clock_row_is_admitted_and_round_trips`: "sourced[\"temporal_bindings\"][row_index][\"source_unit\"] = sourced[\"lock\"][\"sources\"][0].clone();" refuses `invalid_package`/`timed-node-reference` at `/temporal_bindings/{row_index}` with the await declaration as locus |
| FND-006 | fixed | The SU test now reads `row` or `rows`, asserts that SU-01 to SU-05 are present and that `decided == cases.len()`, and maps the `timed-node-reference` cause. SU-05 is decided from its published `rows` and refuses at `/temporal_bindings/1`; the fixture's `acme/orders` declares no units, but SU-05 refuses before any unit resolution. TN-04 and TN-06 to TN-09 use the published `source_unit` values. Passes against c01d61ce |
