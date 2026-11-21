---
id: SR-967175026302197768900474181941935084530
title: "QSL-2743 PR 1119 code-review (Rust lane) of the accepted-remainder test change"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; QSL-2743; PR 1119 head 5e7f682c7321a400fe693d649cd555095af19b5a base e979980ad; tests/it/checked_package_v2_model_fields.rs; read-only context crates/filament-contract-model/src/checked_package/v2/exact_projection.rs, exact_projection/model_union.rs, model_members.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-352", type: references }
---

## Summary

Code review, with the rust-review lane and the rust-style baseline folded in, of PR 1119. Ticket: QSL-2743. The PR changes no production code: one test file, two spec files, and a parent commit (7008a6132) that only copies the four PR 1117 SpecReview files. The changed test extends `every_strict_admitted_non_scalar_payload_form_joins_and_each_wrong_witness_is_refused`. It adds Float32, Float64, Bytes, JsonObject, carried Rational and Duration payload forms, each read through the real strict reader. Each form gets its own independently built Exact oracle. The test adds a real `DeclaredUnit` witness row on the package's unit node, a record-view projection through `project_exact_type`, a per-form wrong original type, and a missing-unit-row refusal.

## Verdict

**PASS** — one low finding (a stale test doc comment).

Held clean (examined, no finding):
- The oracles are independent. Each expected Exact type is built by the test (`FloatType::exact(IeeeWidth::…)`, `RationalDomain::new(interval(-3, 7), interval(1, 12))`, `CarriedQuantityType::new(unit_exact("second"), QuantityCarrier::Rational(domain(7)))`, `Quantity(unit_exact("s"))`), never read back from the projector under test.
- The positive record-field check goes through the record entry, `project_exact_type(record) -> record_view()`, not through the union join.
- The union join compares the supplied `original_type` with the shared projector's output by equality (model_union.rs:1408-1424). So a successful positive join shows that `project_wrapped_record_field_type` produced the oracle exactly.
- The Duration form uses the model's own `s` unit. This follows the reader's own-seconds rule (model_members.rs:2395-2428). There is no Clock or ReferenceClock substitution.
- No production code changed. The shared projector is unchanged from #1117.
- No hash, digest or pin was introduced.

Gates: the reviewer ran one test by name at frozen head 5e7f682c7, on this worktree's own target, `/tmp/ir-dynamic-build cargo test --features conformance-cli,test-support --test it -- --exact checked_package_v2_model_fields::every_strict_admitted_non_scalar_payload_form_joins_and_each_wrong_witness_is_refused`: 1 passed, rc=0 (log /tmp/ir-qsl2743-remainder-reviews/reviewer-strict-head.log). This run was needed because the coder's strict3 receipt (10:16:35 local) and mutant receipts (10:17) came before the test file's last write (10:19:49). No aggregate, workspace or clippy gate was run.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The test's doc comment is stale. It still describes only the bounded integer, profiled text, rational range and option payload forms. The test now also covers Float32, Float64, Bytes, JsonObject, carried Rational and Duration payloads, the record-view projection and the missing-unit refusal, and it is now the sole binder of FR-352-AC-38..44. | tests/it/checked_package_v2_model_fields.rs:9185-9190 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Disposition pass 1, head c9b9a2415c0c78e2b42a16757e1869ebdf850e96 (fix commit c9b9a2415 over merge 2c228fa22 of main a03c86b3b). The doc comment at tests/it/checked_package_v2_model_fields.rs:9214-9222 now lists all ten payload forms, the wrong-witness kinds and what is outside the test. The verbatim after_excerpt is in the round-1 dispositions comment. |

## Disposition round 1 verdict

**PASS.** FND-001 is fixed and the pass adds no new finding. The fix commit changes no production code. The new test `a_carried_quantity_record_field_needs_the_declared_unit_identity_row` follows the repo's test conventions: a `#[trace]` binding, test-built oracles, and no `unwrap` outside setup, with an exact code and locus asserted.
