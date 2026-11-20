---
id: SR-17940486709564847048085
title: "Spec review of the QSL-2506 catalog-free T::op class rule (FR-061 vs FSpec FR-375)"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; spec/checked_package/functional/FR-061-admit-refinement-nodes.md (as merged by #1073, in PR #1072's merge base 8a24e2fe); FSpec f32bca6c spec/functional/analysis/FR-375-declare-model-refinement.md"
review_set: subset
---

## Summary

Ticket: QSL-2506. PR #1072 itself changes no spec file. This review checks the spec authority it implements: contract-ir FR-061 (as merged by #1073) against FSpec FR-375 "Step-row provenance" and FR-375-AC-31.

The question was whether a bare `op` must be refused. FSpec FR-375 supports refusal. It defines the operation class's alias-free `source_class` spelling as `T::op`. Without a catalog, "the reader derives `T` and `op` from `source_class`", and a bare `op` yields no `T`. FR-061 now states the same rule, with the refusal order not-`T::op`, then missing-name at `source_class`, then a `source_node` mismatch, citing FR-375-AC-31.

## Examined scope

- FSpec FR-375 Step-row provenance (examined): "Its alias-free `source_class` spelling is `T::op` ... without a catalog ... the reader derives `T` and `op` from `source_class`"; class resolved before `source_node`; an unresolvable class refuses at `source_class`.
- FSpec FR-375-AC-31 (examined): `source_node` refusals at `source_node`; a class naming no operation `T` itself declares refuses `missing_declaration`/`missing-name` at `source_class`, checked before `source_node`.
- FR-061 Catalog-free operation step join (examined): `T` is "the declaring type's full qualified name, its segments joined with `::`"; bare refuses `invalid_model_binding`/`malformed-declaration` at `source_class`; non-declaring `T` refuses missing-name at `source_class`; a resolved class with another anchor refuses at `source_node`.
- FR-061-AC-14, FR-061-AC-15 (examined): consistent with the body and with FR-375-AC-31.

## Verdict

PASS with one low clarity finding. FR-061 is consistent with FSpec FR-375 and with the ruling.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-061 defines `T` as a full qualified name of several `::` segments. The concrete model's declaring types are model-owned nodes `ix://<identity>/<name>` with one flat name, and FSL emits flat names (`Cell::increment`). So no multi-segment `T` can resolve, and the reader matches only the flat name. State that `T` is the model-owned type's name, or say how a namespaced model type is spelled. | spec/checked_package/functional/FR-061-admit-refinement-nodes.md |
