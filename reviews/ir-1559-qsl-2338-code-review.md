---
id: SR-202611081512300012
title: "PR 1035 reference, source clock, and dispatch review"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR #1035 at e038db0fe027d0491be7bb49d57754b1e82919ac vs bc8a7b96; disposition head 100aaa6b6ebf6b4e788d1b836f87d879dd040632; IR-1559/FR-480-AC-3; QSL-2338/FR-965-AC-1,AC-3; QSL-2347/FR-322-AC-290 dispatch selection_key seam 1; crates/filament-contract-model/src/checked_package/v2/{identity.rs,operations.rs,temporal.rs,dispatch_projection.rs,typed_terms.rs,vocabulary.rs}; tests/conformance_fspec/temporal_aggregate.rs; tests/it/checked_package_v2_{typed_terms,source_objects}.rs"
review_set: subset
---
# SR-202611081512300012: PR 1035 reference, source clock, and dispatch review

## Summary

Reviewed frozen PR #1035 at `e038db0fe027d0491be7bb49d57754b1e82919ac` against `bc8a7b96`; re-reviewed the source-owner fix at `100aaa6b6ebf6b4e788d1b836f87d879dd040632`. Scope is IR-1559 reference literals, QSL-2338 source clock units, and QSL-2347 dispatch `selection_key` seam 1 only. The W19 `clock_place` and `operation_bodies` seams were excluded. FND-001 is fixed: source unit, dimension, conversion chain, and bound are now joined to the clause's source owner, and the A/B same-name test exercises the collision. FND-002 remains: the source matchers accept any path ending in `T`, despite FR-965 requiring exactly `[T]`. No aggregate gate was run. The coder reported the affected focused filters, fmt/diff and Clippy passing.

## Coverage

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-480-AC-3 | examined | `M::Server#"s1"` lowers to a `literal` term of `value_kind` `reference`, `type` the `composite_type`/`reference` node over `Server`, and `value` `7331`. Two occurrences of it, one written through a second alias of the same package, share the term. |
| FR-965-AC-1 | examined | A source unit declares Time units `s` and `ms`; under timestamp unit `ms` duration bounds are typed by the source-owned `ms` unit node, declaration `ms`, owner kind `source`, and no model-shaped `scalar_type/unit/ms` node is emitted. |
| FR-965-AC-3 | examined | A source unit may also select a model package: source `ms` selects the source-owned node, while an FCD identity selects the model-owned `scalar_type/unit/<identity>` node. |
| FR-965 source-unit behavior | examined | When `T` names a unit in the clause's own source unit, its clock node is that source-owned node: its `node_id.path` is its declaration path `[T]`, it carries that source unit's `SourceOwner`, and it is the node every other quantity of the unit names. |
| FR-322-AC-290 (dispatch candidate join) | examined | Each candidate selection key is the exact FR-150 structural EffectiveId supplied by the producer and matches the candidate's admitted effective declaration identity; it is not a NodeRef or inferred from a name. |
| `reference_literals_require_canonical_hex_and_an_object_reference_type` | examined | Canonical lowercase hexadecimal and reference-to-object typing are checked; invalid identity and wrong static type refuse. |
| `reference_literal_read_preserves_the_canonical_target_identity` | examined | Read-back preserves the reference literal's canonical target identity. |
| `timestamped_duration_bounds_admit_only_linear_source_time_units` | examined | Source owner's unit admits; foreign owner's same-name unit refuses; affine source unit refuses. |
| `source_candidate_selection_keys_refuse_unless_they_name_a_source_type_of_the_owner` | examined | Bare, non-type, model, missing and foreign-package candidate keys refuse. |

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Source clock lookup was keyed only by the bare unit name and could use any SourceOwner's same-named unit. | `crates/filament-contract-model/src/checked_package/v2/temporal.rs:1645-1668,1691-1729`; `crates/filament-contract-model/src/checked_package/v2/identity.rs:2263-2283` | missing-requirement |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-002 | medium | Source-unit lookup and bound matching compare only the final path segment, so a same-owner unit with path `other/ms` can satisfy `timestamp_unit=ms`. FR-965 requires the source node path to be exactly `[T]`. Require the exact one-segment path and matching declaration path; keep the model-owned path branch unchanged. The new second-owner fixture can use a distinct node-id package with path `[ms]`, and a nested-path refusal should pin the closed path. | `crates/filament-contract-model/src/checked_package/v2/temporal.rs:1674-1682,1746-1760`; `crates/filament-contract-model/src/checked_package/v2/identity.rs:2280-2288`; `tests/conformance_fspec/temporal_aggregate.rs:135-156` | missing-requirement |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | At head `100aaa6b6ebf6b4e788d1b836f87d879dd040632`, source unit lookup and bound matching use the clause's SourceOwner; the two-owner same-name test passes. |
| FND-002 | still-open | At frozen head `100aaa6b6ebf6b4e788d1b836f87d879dd040632`, matching still accepts any path whose last segment is `T`, contrary to FR-965's exact `[T]` path. |
| FND-002 | fixed | At head `84da1257efd53f0a43afd1ab17fffbe59295beed`, source unit and bound matchers require one-segment `[T]` paths. The same-owner nested-path case refuses, and the focused source-clock regression passes 1/1. |

## Verdict

**PASS.** FND-001 and FND-002 are fixed at `84da1257efd53f0a43afd1ab17fffbe59295beed`. The targeted source-clock regression passes; no new code findings were found. No aggregate gate was run.
