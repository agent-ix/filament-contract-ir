---
id: SR-17940486709564847048072
title: "gap-analysis of QSL-2347 source dispatch keys, operation bodies and identity-row operations"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1049 at 3d9417a7 (base origin/main 71914a07); FR-365, FR-366, FR-367, FR-373, TC-480 against crates/filament-contract-model/src/checked_package/v2/{dispatch_projection.rs,operation_bodies.rs,source_object.rs} and tests/it/checked_package_v2_source_objects.rs"
review_set: subset
---

## Summary

Gap analysis for PR #1049. Ticket: QSL-2347. Planless: plan completion not assessed. The ticket's three seams were compared with FSpec FR-788/789/790 (filament-specification origin/main 118febea). The IR ACs were compared with tagged tests and code.

`quire coverage --strict` (make spec): 79 unbacked rows at head and at clean main, 0 contradicted. FR-365/366/367 are 6/6 and FR-373 is 5/5. cargo-mutants 27.1.0 `--in-diff` lists the same 26 mutants the coder reports. The hand-mutant results are in the code-review SR.

## Examined scope

- FR-366-AC-3 / FR-788-AC-1..2 (examined): `source_candidate_selection_keys_refuse_unless_they_name_a_source_type_of_the_owner` gains the non-null `owner_effective_type` case. The FR-788-AC-2 "checked package's name" pair is still absent (code-review FND-004).
- FR-367-AC-5 (examined): `a_complete_row_set_exposes_every_source_type_through_a_typed_read` asserts `operations`.
- FR-373-AC-1 (examined): `source_operation_bodies_admit_and_join_each_candidate_to_its_own_row`. It asserts admission, each typed row by key, and serialization as written.
- FR-373-AC-2..4 (examined): `malformed_source_operation_body_rows_refuse`, 8 cases, with exact code, cause and locus.
- FR-373-AC-5 (examined): `an_inheriting_candidate_joins_the_row_of_the_declaring_type`.
- Seam 2, FSpec FR-789 (examined): not implemented. The coder's gap claims were checked against FSpec:
  - (a) FR-322's member-type table `Clock` row says "none: no node stands for a clock". It also says a record value type with a clock field has the `record_value_type` node of its data fields only. The IR identity-row member `SelectedSourceObjectMember{name, type_node: CheckedNodeId, optional}` is checked in order against the node body's bindings (FR-367), so neither `O.d: Clock` nor `W.c: Clock` of FR-789-AC-1 has an entry or binding to join. FSpec's own TM-S1 expectation needs `member_type` `ix://filament/native/Clock` from somewhere.
  - (b) The identity rows are supplied beside the package to `SelectedSourceObjects::from_stage_rows`, after `CheckedPackageV2::read`. Yet FR-789 states its refusal as an `invalid_package`/`malformed-clock-place` package-read locus.
  - (c) Source `operation_bodies` coverage of operations no candidate joins needs that same input (spec-review FND-004).

  All three claims are correct.
- FSpec conformance vectors (context_only): `tests/checked_package_v2.rs` in FSpec carries FR-788/789/790 vector tests (TC-1915 to TC-1920), all `#[ignore = "QSL-2347: ... not published yet"]`.
- Clean-main failing set (examined): see code-review SR.

## Verdict

Seams 1, 3 and 4 are delivered and traced. Every in-diff AC has a passing tagged test. Seams 1, 3 and 4 do not depend on seam 2:

- FR-788's last bullet (rendering `ix://I/<type>` in clock-place rows) only matters to FR-789.
- FR-373's coverage relaxation is self-contained for the package read.

The coder's seam-2 gap analysis is correct. draco:QSL:arch has ruled option (c): seam 2 is deferred to the post-read identity-row join, and W51 specs `member_type`, tuple/union/slot, the timed-model Clock-param check and FR-790 coverage. So QSL-2347 stays open for FR-789 and for the FSL-side acceptance checks.

Compatibility note, not a blocker: `SelectedSourceObjectMember.type_node` is a required `CheckedNodeId`, so a later Clock member (no node) needs a shape change. That is another breaking change to the same public row, so it is best batched with the `clock_parameters` addition the coordinator reports W51 asks for.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Seam 2 (FSpec FR-789, source timed `clock_place` join) is not delivered. QSL-2347 lists it in "What to build" and in its acceptance ("admits each of the three shapes"). Per the arch ruling it is deferred to the post-read join under W51, so the ticket must stay open, or the seam must be split to a W51 ticket before QSL-2347 closes | crates/filament-contract-model/src/checked_package/v2/source_object.rs:150-240 |
| FND-002 | medium | The ticket acceptance asks for "a conformance test for each against an emitted source-owned package". The PR's tests use hand-built in-repo fixtures, and FSpec's FR-788/790 vector tests (TC-1915, TC-1916, TC-1919, TC-1920) are still ignored as unpublished. No test reads an FSL-emitted or FSpec-published source package. This must stay open on QSL-2347 until the FSpec vectors publish (then wire them into tests/conformance_fspec) or the FSL-side QSL-1880 AC6 check passes | tests/it/checked_package_v2_source_objects.rs:2644-2840 |
| FND-003 | low | The PR title reads "admit source-owned dispatch keys, clock places and operation bodies", but clock places are not in the PR (the body says so). A squash merge would put a false seam-2 claim in main's history. Retitle to "dispatch keys, operation bodies and identity-row operations" | PR #1049 title |

## Dispositions

Round 1, checked at ab9dc00d.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | draco:QSL:arch ruled option (c): seam 2 (FR-789) moves to the identity-row join, specified by FSpec W51 and contract-ir #1050 (FR-374/FR-375, open). The PR body lists it under "Open on QSL-2347 (not closed by this PR)", so QSL-2347 stays open for it |
| FND-002 | deferred | The FSpec vectors TC-1915..1920 are still `#[ignore]`d upstream as unpublished, and the PR body records this. It stays open on QSL-2347 until the vectors publish or the FSL QSL-1880 AC6 check passes |
| FND-003 | fixed | after: PR title "QSL-2347: admit source-owned dispatch keys, operation bodies and identity operations (seams 1/3/4; refs QSL-2347)" |

Round 2, checked at 51edebff: no finding has a still-open latest outcome, so this round adds no rows. FND-001 and FND-002 stay deferred on QSL-2347, unchanged (seam 2 under W51/#1050, and the unpublished FSpec vectors).
