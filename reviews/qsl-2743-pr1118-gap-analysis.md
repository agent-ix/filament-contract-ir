---
id: SR-967175026302197768900474181941935084519
title: "QSL-2743 PR 1118 gap-analysis of the changed obligations"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; QSL-2743 (QSL-2711 rides); PR 1118 head b7dcf6666bb715253e4c8af6f7920b7cd1784c7c; spec/model/functional/FR-053, spec/checked_package/functional/FR-353, FR-038-AC-237; crates/filament-contract-model/src/checked_package/{selected_population.rs,v2/equality_projection.rs}; crates/filament-contract-model/tests/selected_population_public.rs; tests/it/checked_package_v2_model_members.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-053", type: references }
  - { target: "ix://agent-ix/filament-contract-ir/FR-353", type: references }
  - { target: "ix://agent-ix/filament-contract-ir/FR-038", type: references }
---

## Summary

This is a planless gap analysis of the obligations PR 1118 exercises, checked against the computed Test Matrix (`quire matrix --scope . --format json`, quire 0.36.2, rc=0), the tagged tests and the source. Ticket: QSL-2743. The PR changes no spec. Every criterion it touches is tagged. One tagged binder's oracle no longer checks a clause of FR-053-AC-1.

## Verdict

**CONDITIONAL**: one medium finding. No criterion the PR touches is untagged.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Part of the evidence for FR-053-AC-1 ("returns the same package name, node ID, occurrence") is now hollow. The original binder selected_population_is_bound_to_the_emitted_node_and_source_occurrence feeds a `graph()`-spelled selector. Its `selected.binding == node.node_id` assert was replaced with a comparison against the document-held ID. The new binder selected_population_reads_source_for_node_held_operand_spelling asserts that a graph-spelled selector returns a different binding. No test now checks the AC's same-node-ID clause for the selector spelling `graph()` exposes. That clause is only met for document-held (`reaches_population`) selectors. | spec/model/functional/FR-053-selected-population-binding.md:163; crates/filament-contract-model/tests/selected_population_public.rs:202-212,547-550 |

## Coverage

- **Matrix, repo-wide:** 1203 tagged, 76 untagged, 26 method-without-symbol, 1 tagged-by-ignored-test. These counts are pre-existing. The PR adds and edits no criterion.
- **Touched criteria, all tagged:** FR-353-AC-1, FR-353-AC-6 and FR-038-AC-237 are bound by held_model_enum_equality_facts_are_local_for_eq_and_ne. FR-353-AC-7 is bound by unheld_or_wrong_owner_model_enum_fact_refuses_with_node. FR-053-AC-1 to AC-5 are bound by the two original selected_population_public tests. AC-1 and AC-2 are also bound by the new regression. The repo's doc-comment `Trace:` lines bind in the computed matrix.
- **Reverse gap:** none. The equality refusal-code precedence change (unheld node gives Foreign) is owned by FR-353-AC-7, and the population lookup is owned by FR-053. The unreachable StaleBinding arm is recorded in the code-review SR (FND-002), not here.
- **Stubs and coverage inflation:** no stubs. The fixture is real producer output, byte-equal to the arch evidence in /home/peter/pop2711.
- **Semantic review:** not run as a fanned-out pass. The checks for intent, test and code were done inline for the touched criteria only.
- Plan completion: not assessed

## Dispositions

Round 1 was checked at frozen head 688d827b9e5c795d917d634cdbfb861fbb63280f (fix 688d827b9 on top of merge d08d6af30 with origin/main e979980ad). Reviewer rerun: selected_population_public 3 passed, rc=0.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fixed at 688d827b9. The FR-053-AC-1 binder asserts `selected.binding == node.node_id` again for the graph-spelled selector (selected_population_public.rs:212), and its provenance oracle still finds the one source entry under the document-held ID. The regression asserts that the result equals the input binding for both spellings (547-550). Remaining limit: the unbounded branch is covered only with a document-held selector, by tc_048_population_maximum_is_canonical_or_explicitly_unbounded. That branch is the same `selector.binding.clone()` code, so I record no finding. |
