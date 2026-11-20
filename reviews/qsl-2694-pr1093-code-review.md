---
id: SR-2026112009294800000001
title: "code-review (rust-review lane) of QSL-2694 PR #1093 fixtures and stage-4 edge existence"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR #1093 at 24ab77d9, git diff origin/main...HEAD; crates/filament-contract-model/src/checked_package/v2/{edge_existence.rs,mod.rs,monitor_plan.rs,monitor_plan/reached_model_read.rs,monitor_plan/layout_context.rs,exact_projection/model_enum_tests.rs,model_members/tests/map_types.rs,operations.rs,relationship_end.rs,timed_model.rs}; tests/it/*; tests/conformance_fspec/model_tuple_union.rs"
review_set: subset
---

## Summary

This is the code-review of PR #1093, with the rust-review lane folded into this file. It covers QSL-2694 (28 stale lib fixtures), QSL-2666 (4 `it` tests) and QSL-2644 (the stage-4 edge-existence pass of FSpec FR-322-AC-275). Ticket: QSL-2694. Evidence came from focused, name-filtered runs on the head, all done in a private worktree with a private target dir:

- 48 `it` tests (reaches_population, residual_, refinement, catalogue, typed_terms, dependency_reference, relationship destination): pass.
- Two local mutants, reverted afterwards:
  - Mutant A skips the dependencies loop.
  - Mutant B removes the nominal exemption.
- Instrumenting the binary-search window test.

## Examined scope

- `edge_existence::validate`: covered tags, edge order (semantic_type before dependencies), code `invalid_semantic_graph` with no cause, edge locus, placement between `validate_selected_type_bounds` and `model_occurrence::validate`. These agree with FR-322 stage 4 and with FR-038-AC-582.
- The surviving mutant "dependency loop skipped". It is not equivalent. With the loop disabled, 4 `it` tests fail:
  - `tc_048_a_population_that_is_not_a_listed_node_id_is_refused_ill_typed`
  - `tc_048_a_navigate_application_never_carries_a_population`
  - `residual_required_leaf_sources_refuse_missing_referenced_types_on_public_read`
  - `residual_set_result_refuses_missing_element_type_on_public_read`

  Each of them then reaches the operation stage and refuses ill_typed. The mutant only survived a lib-scoped mutation run.
- `ValidatedGraph<'w>` now carries the `GraphIndex` built once in `validate_graph` (with ranks), and `requirement_records::validate` reuses it. This removes a second `GraphIndex::build`, which charged a stale Graph-stage site after SourceMap. The lookups are by id and are unchanged by ranks. No meter release semantics depend on the earlier drop. Clean.
- Lib fixtures: the reader is not relaxed. `genuine_bytes_under_current_rules` adds body_locations rows and model-declaration source_map entries to the committed genuine packages at test time. The relationship_end fixture is corrected in place. The clock_reference catalog row is added. The 8 FSpec-checkout tests return early when FILAMENT_SPECIFICATION_DIR is unset. That matches the existing pattern in identity.rs, and CI sets the variable.
- `it` fixture edits (refinement transitive deps, dependency_reference package rename, catalogue sum key, clock_reference.ref unavailable reason): these are fixture corrections and do not weaken any assertion. The 7 tests that built deliberate dangling edges now assert the stage-4 refusal. The spec impact of that is recorded in the spec-review and gap-analysis SRs.
- Binary search in `a_denied_relationship_destination_read_is_refused_not_a_double_borrow` (FND-001).

## Verdict

Changes required. The stage-4 pass is correct against FR-322 for the tags it covers, and the GraphIndex reuse is sound. Two problems remain. The binary-searched window no longer reaches the flat-wire destination read the test exists for. The nominal exemption is a reader-local deviation, kept only to preserve one IR test's locus.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The binary-searched window opens at the first limit past Canonical, and every one of its 700 limits (735334..736033) stops at PackageRead(WireDecode, 0). Not one reaches WireChecks, where the flat-wire relationship destination read is charged. The test therefore no longer exercises the double-borrow denial it is named for. The search should target the first limit whose denial is the destination read | tests/it/checked_package_v2_model_members.rs:4372-4392 |
| FND-002 | medium | The `nominal_dependencies` exemption skips the dependencies of dimension/unit/dimension_equivalence nodes. FSpec FR-322 stage 4 has no such exemption: it covers every scalar_type node. Mutant B (exemption removed) breaks only `constructed_body_does_not_bypass_package_semantic_admission`, whose locus moves from body/terms/0/dimension_node_id to dependencies/0. The deviation preserves an IR-local locus (FR-038-AC-374, PLANNED/UNRUN) against FSpec order | crates/filament-contract-model/src/checked_package/v2/edge_existence.rs:44-55, 87-89 |
| FND-003 | low | Stage 4 charges 1 + dependencies.len() even for an exempt nominal node whose dependencies it does not read. It also computes `1 + u64::try_from(..).unwrap_or(u64::MAX)`, which would overflow if the fallback were ever taken; `saturating_add` states the intent | crates/filament-contract-model/src/checked_package/v2/edge_existence.rs:74-78 |
| FND-004 | low | The new stage-4 branches of the reaches/navigate population tests assert only (code, cause) = invalid_semantic_graph. The /semantic_graph/nodes/{n}/dependencies/{k} locus is not pinned, although the ill_typed branch pins POPULATION_AT | tests/it/checked_package_v2_model_members/reaches_population.rs:481-496, 656-666 |
| FND-005 | low | The committed genuine producer fixtures (monitor-base.json, the model_enum CHECKED bytes) stay non-conforming on disk. They are upgraded at read time by `genuine_bytes_under_current_rules`, so any later drift is hidden by the transform rather than surfaced. Regenerating the fixtures, or a follow-up ticket to do so, would keep them genuine | crates/filament-contract-model/src/checked_package/v2/monitor_plan/reached_model_read.rs:809-823 |

## Dispositions

Round 1, head 1f40c78e.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Verified in the fix commit. |
| FND-002 | deferred | The exemption is now recorded as an explicit DEVIATION in FR-038-AC-582 and TC-048, pending an arch/FSpec decision (draco:QSL:arch); the code is unchanged until that ruling, which is outside this PR. |
| FND-003 | fixed | Verified in the fix commit. |
| FND-004 | fixed | Verified in the fix commit. |
| FND-005 | accepted-no-change | Documented as a known limit on genuine_bytes_under_current_rules: the fixtures are regenerated by the producer fixture refresh, and this crate does not depend on the producer tool. Reviewer accepts this for this PR. |
