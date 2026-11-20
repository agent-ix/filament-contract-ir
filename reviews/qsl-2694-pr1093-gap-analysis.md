---
id: SR-2026112009294800000002
title: "gap-analysis of QSL-2694 PR #1093 stage-4 edge existence and changed tests"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1093 at 24ab77d9; FR-038-AC-582, FR-038-AC-272, FR-038-AC-275, FR-038-AC-280, FR-038-AC-229, FR-038-AC-230, FR-038-AC-511; tests/it/checked_package_v2_model_members/reaches_population.rs, tests/it/checked_package_v2_recursive_leaves.rs, tests/it/checked_package_v2_model_members.rs, tests/conformance_fspec/model_tuple_union.rs, tests/conformance_fspec/reaches_population.rs"
review_set: subset
---

## Summary

This gap analysis is scoped to the criteria that the PR's code and tests touch. Ticket: QSL-2694. Plan completion: not assessed.

`quire coverage --scope . --strict` reports 88 unbacked rows and 0 contradicted statuses. These are pre-existing and do not change with this PR. The criterion-to-test bindings below were checked by reading each test and by running it, both on the head and under local mutants.

Focused runs on the head:

| Suite | Result |
| --- | --- |
| `it` | 48/48 pass |
| lib (touched modules) | 166/166 pass |
| `conformance_fspec` model_tuple_union + reaches_population | 17 pass, 1 fail |

The failing test is `tc_048_fspec_reaches_population_cases_match_the_reader` at `reaches-population-names-absent-node`, which now gets `invalid_semantic_graph` at /semantic_graph/nodes/6/dependencies/2. This is a new red relative to the 16 known baseline reds on main.

## Examined scope

- FR-038-AC-582 (examined). Stage 4 before stage 5 with a dangling `semantic_type` is bound to `a_dangling_edge_is_refused_before_a_model_declaration_occurrence_defect`, which passes. The `dependencies/{k}` locus is pinned by the `dangling_dependency` helper in recursive_leaves. The stage-4 charge clause, the semantic_type-before-dependencies order within one node, and the out-of-stage tags have no test (FND-003).
- FR-038-AC-272, FR-038-AC-275, FR-038-AC-280 (examined). The tagged tests were changed to expect the stage-4 refusal, or (AC-280) to use a present population. Each AC still states the old outcome (FND-001).
- FR-038-AC-229, FR-038-AC-230 (examined; PLANNED/UNRUN). The tagged residual tests now assert stage 4. Mutant A shows that the "unresolved compared/inner type → ill_typed at operation.leaves" branch is reachable only when stage 4 is disabled (FND-002).
- FR-038-AC-511 relationship destination denial (examined). See code-review SR FND-001; this criterion has other bound tests.

## Verdict

Not complete. Three Implemented ACs now disagree with their tagged tests. One AC branch has lost its only exercising test. Part of AC-582 has no test. The published FSpec vector conflict is an upstream spec question.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Tests tagged FR-038-AC-272, AC-275 and AC-280 now assert invalid_semantic_graph, or substitute a present node, for a listed population that names no node. The ACs (Implemented) still state ill_typed/operator-ineligible at population, or for AC-280 the earlier operand refusal. The bindings are therefore wrong until the ACs are amended | tests/it/checked_package_v2_model_members/reaches_population.rs:463-497, 609-666, 1026-1080 |
| FND-002 | medium | FR-038-AC-229/230's "type remains unresolved after earlier checks → ill_typed/operator-ineligible at operation.leaves" branch has lost its only public-read tests. These now assert the stage-4 refusal, and mutant A shows the branch is reached only without stage 4. Either add a fixture that reaches it with all edges present, or record in the ACs that stage 4 makes it unreachable | tests/it/checked_package_v2_recursive_leaves.rs:735-818 |
| FND-003 | medium | FR-038-AC-582 stage-4 clauses with no test: the charge (one unit per in-stage node plus one per dependencies entry, no allocation), the semantic_type-before-dependencies order within a node, and the exclusion of state/temporal/protocol/relation/claim/correspondence/extension/hybrid/timed_model nodes. edge_existence.rs has no unit tests | crates/filament-contract-model/src/checked_package/v2/edge_existence.rs:57-103 |
| FND-004 | medium | Spec question (not a code defect): the published FSpec vector reaches-population-names-absent-node (FR-322-AC-73, TC-233) expects ill_typed/operator-ineligible, but FR-322-AC-275 stage 4 refuses the listed absent population first. conformance_fspec now fails on it; FSpec must decide | tests/conformance_fspec/reaches_population.rs:224 |

## Dispositions

Round 1, head 1f40c78e.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Verified in the fix commit. |
| FND-002 | fixed | Verified in the fix commit. |
| FND-003 | fixed | Verified in the fix commit. |
| FND-004 | deferred | Deviation recorded in the FR-038 population prose and FR-038-AC-272 (follows FR-322-AC-275; vector replay fails until FSpec decides); the FSpec decision is pending with draco:QSL:arch and is outside this PR. |
