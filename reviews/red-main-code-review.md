---
id: SR-17940486709564847048135
title: "Code review — QSL-2711 red-main conformance baseline root causes"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR #1105 at 57f2659d (git diff origin/main...HEAD); Makefile; spec/checked_package/matrix/TC-464-model-operation-census.md; tests/conformance_fspec/{monitor_plan,operation_effects,probabilistic_claim_timed,probabilistic_claims,requirement_records,temporal_aggregate}.rs; tests/it/selected_population.rs; tests/it/support/checked_package.rs"
review_set: subset
---

## Summary

Ticket: QSL-2711. I reviewed `git diff origin/main...HEAD` for PR #1105 at 57f2659d (10 files, +210/-58, 3 commits). The reference spec is FSpec main at FSPEC-main-6ad873. The Rust review lane is folded in. The PR changes only tests, harness, a Makefile comment and one TC sentence. No reader or production code changed.

The review checked each touched test: what it asserted before and after, and whether the drift fixed is real against FSpec.

## Examined scope

- `body_locations` in hand-built `checked_term_data` (examined). This is in probabilistic_claim_timed.rs, probabilistic_claims.rs and support `rebuild_body_locations`. FSpec FR-322 (lines 1268-1273) and local FR-038-AC-390 require five arrays, `body_locations` included. Adding the missing array repairs fixture drift and changes no assertion. Renaming `four_array_...` to `five_array_schema_and_graph_admission_precede_sidecar_join` adds `body_locations` to the arrays removed one at a time, which adds one refusal case, so the test is stronger. It passes.
- monitor_plan hybrid subject (examined). FSpec FR-727 says a clause whose `over` type reaches no object type has no `owner_model` edge, and the reader refuses one that has it with `malformed_wire`. The fixture retargets `over` to a `hybrid_model`, so removing `owner_model` is the correct root cause. The assertion is still `Err(MissingClock)`, unchanged. The test passes. The loop removes `owner_model` from every protocol clause, not only the retargeted one. The recipe has a single clause and the hybrid vectors add none (0 of 30 nodes), so this is equivalent.
- operation_effects OEC-08 (examined). FSpec's `operation-effect-vectors.json` gained `OEC-08-source-declaration`, a `source: true` sketch for a FR-610 source-declared type. `add_source_declaration` builds only the input package. The expected rows are still the published `rows` (document, declaring, operation, effect, provenance), compared field by field, so the assertion strength is unchanged. `tc_977_published_operation_effect_vectors` passes.
- requirement_records (examined):
  - Writer readback now reads the writer's exact bytes instead of re-indexing and refreshing them through `read()`. This is stronger. It is still red, as listed.
  - The `declaration member` row moved out of the mutation table into its own test with the identical code, cause and pointer (`invalid_package`/`invalid-value` at `/occurrences`, FSpec FR-825-AC-4 rule 1). Nothing was loosened, and the other rows of the table now run (it passes). The new test stays red, as listed. Its doc comment matches FR-825-AC-4.
  - Ordinal swap: the source-map ordinals now follow the swapped occurrences, so the refusal reaches the record step. The pointer is unchanged and the test passes.
  - `claim_names` naming a `value_validity` node: `read()` → `refresh_identity` → `refresh_temporal_bindings` → `refresh_claim_names` dropped the injected row (verified at tests/it/support/checked_package.rs:2974). The new path keeps the row and asserts the same code, cause and `/claim_names/0/declaration`. Same strength, and it passes.
- temporal_aggregate (examined). Adding `declaration_region` to the inserted `temporal_clause_names` row matches FSpec FR-322-AC-460/461, which require it. The test is still red (duration_bounds), as listed.
- selected_population / ir760 (examined):
  - **Before:** the test asserted that the reader admits a population body with `max` twice, and that `select_population` returns `NotPopulation`.
  - **After:** it asserts the reader refuses `invalid_package`/`invalid-derived-shape` with a locus ending `/node_id`.
  - This is spec-correct. FSpec FR-322-AC-74 says "a second member beside `max` ... refuse[s] `invalid_package`/`invalid-derived-shape` at the population node ... at the population node's `node_id`". The FR-322 derived-shape table (line 3009) lists "a member beside `max`". Local FR-053-AC-3 says a second body member "fails V2 admission ... rather than producing a projection outcome", and TC-453 steps 7 and 8 agree.
  - The reader enforces this at structural.rs:1080-1089 (`_ => false`), refusing at `node_path.key("node_id")`.
  - The old assertion contradicted the spec, so this is a real drift, not a bent assertion. The new locus check is weaker than the spec allows, though (FND-001).
- Makefile and TC-464 doc (examined). See SR-17940486709564847048137. Both are accurate.
- Focused runs (CARGO_TARGET_DIR in my scratchpad, FILAMENT_SPECIFICATION_DIR=FSPEC-main-6ad873, `--features conformance-cli`, `-j 4`, flock):
  - `conformance_fspec`, name-filtered: 20 pass, 4 fail. The failures are exactly the listed known reds: `canonical_writer_readback...`, `...exact_work_budget...` and `...declaration_member_at_occurrences` in requirement_records, plus `temporal_aggregate::duration_bounds...`.
  - `it` (inherited_dispatch and operand_families, the `rebuild_body_locations` callers): 40/40 pass.
  - `conformance_fsl_ir760 duplicate_max`, run against a snapshot of FSL GitHub main 9ec62fb9: 1/1 pass.

## Verdict

**Mergeable with one medium test-strength finding.** No assertion was deleted or loosened to go green. Every touched fixture fixes a drift that FSpec text confirms. The ir760 refusal is the spec-correct FR-322-AC-74 derived-shape refusal. FND-001 asks for the exact node pointer. FND-002 is a misleading comment.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The ir760 duplicate-max test checks only that `refusal.loci[0].path` ends with `/node_id`. FSpec FR-322-AC-74 and FR-053-AC-3 place the refusal at the population node's `node_id`. A refusal at any other node's `node_id` (for example another derived-shape or owner-join refusal elsewhere in the FSL package) would pass. Record the population node's position when it is mutated and assert the exact pointer `/semantic_graph/nodes/{position}/node_id`. | tests/it/selected_population.rs:123-131 |
| FND-002 | low | The rationale comment says FR-322 "derives a population node's id from its body ... so a body binding `max` twice has no derivable identity". The refusal is actually the FR-322 stage-3 derived-shape check, "a member beside `max`" (FR-322-AC-74, derived-shape table, `RM-POP-MAX` family), made by structural.rs:1080-1089. It is not caused by an underivable id: the structural path is unchanged by the duplicate. Cite FR-322-AC-74 instead. | tests/it/selected_population.rs:112-115 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | edc9b65b records `position` of the population node and asserts `assert_eq!(refusal.loci[0].path...as_str(), format!("/semantic_graph/nodes/{position}/node_id"))` (tests/it/selected_population.rs:93-133). `refresh_identity_projection` only rewrites ids and does not reorder nodes, so the position stays valid. ir760 duplicate_max passes against FSL main 865737aa (merge head 23eea2d8). |
| FND-002 | fixed | edc9b65b rewrites the comment to cite the FR-322 derived-shape check ("member beside max", FR-322-AC-74; the structural.rs derived-shape arm) and no longer mentions an underivable id. |
