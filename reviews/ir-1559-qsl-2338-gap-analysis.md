---
id: SR-202611081512300013
title: "PR 1035 acceptance coverage gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1035 at e038db0fe027d0491be7bb49d57754b1e82919ac vs bc8a7b96; disposition head 100aaa6b6ebf6b4e788d1b836f87d879dd040632; IR-1559/FR-480-AC-3; QSL-2338/FR-965-AC-1,AC-3; QSL-2347/FR-322-AC-290 dispatch selection_key seam 1; tests/conformance_fspec/temporal_aggregate.rs; tests/it/checked_package_v2_{typed_terms,source_objects}.rs"
review_set: subset
---
# SR-202611081512300013: PR 1035 acceptance coverage gap analysis

## Summary

Mapped the scoped requirements for reference literals, source clock units, and dispatch selection keys to their focused tests at frozen PR #1035 head. `quoin gap-analysis` is unavailable on this machine (`command gap-analysis not found`), so mapping was performed manually. FND-001's missing owner-collision case is fixed by the new A/B same-name `ms` test. The clock-unit coverage still lacks a same-owner nested-path refusal, and the collision fixture itself currently gives B the nested path `other/ms`; this can let the test pass while relaxing FR-965's exact `[T]` source path. W19 seams were excluded. No aggregate gate was run.

## Coverage

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-480-AC-3 | examined | A reference literal lowers to `value_kind` `reference`, with type the reference node over the model object and lowercase hexadecimal key bytes; aliases of the same package share the term. |
| FR-965-AC-1 | examined | A source-owned `ms` node is the clock unit selected by `timestamp_unit=ms`, and duration bounds use that exact node. |
| FR-965-AC-3 | examined | Source unit name resolves to the source-owned node; an FCD identity resolves to the selected model-owned unit. |
| FR-965 source-unit behavior | examined | The source clock node's path is its declaration path `[T]` and it carries the clause's source-unit `SourceOwner`. |
| FR-322-AC-290 | examined | Candidate `selection_key` is the exact structural EffectiveId matching the candidate's admitted effective declaration identity, not a NodeRef or inferred name. |
| `reference_literals_require_canonical_hex_and_an_object_reference_type` | examined | Unit test exercises canonical key form and typed object target. |
| `reference_literal_read_preserves_the_canonical_target_identity` | examined | Integration test exercises literal read-back. |
| `timestamped_duration_bounds_admit_only_linear_source_time_units` | examined | Conformance test now exercises A+B same-name source units, B-only refusal, and an affine-unit refusal. |
| `source_candidate_selection_keys_refuse_unless_they_name_a_source_type_of_the_owner` | examined | Integration test exercises valid owner-scoped source type keys and malformed/foreign alternatives. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The first version's source-clock test had no same-name second SourceOwner, so it could not reveal the missing clause-owner join. | `tests/conformance_fspec/temporal_aggregate.rs:463-534` |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | The new A/B test distinguishes owners, but B's fixture uses path `other/ms`; no test refuses a same-owner nested path. Since FR-965 requires exact path `[T]`, this test shape does not ensure only the canonical source path admits. Add a same-owner nested-path refusal, and represent B as a distinct package with path `[ms]` for the collision case. | `tests/conformance_fspec/temporal_aggregate.rs:123-156,573-610` |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The revised test admits A+B for A's clause and refuses when only B's same-named unit is present. |
| FND-002 | still-open | The fixture uses B's nested `other/ms` path and does not assert that a nested path for the clause owner refuses. |
| FND-002 | fixed | At head `84da1257efd53f0a43afd1ab17fffbe59295beed`, the collision case uses a separate owner with `[ms]`, and a same-owner `[other, ms]` path refuses. The focused source-clock test passes 1/1. |

## Verdict

**PASS.** All scoped acceptance coverage gaps are closed at `84da1257efd53f0a43afd1ab17fffbe59295beed`. The generated quoin gap-analysis command was unavailable, so the mapping remains manual; excluded W19 seams were not assessed.
