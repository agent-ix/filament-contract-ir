---
id: SR-967175026302197768900474181941935084581
title: "gap-analysis of QSL-2128 subject_declaration_order reader half (FR-351-AC-6, AC-7)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1122 at 5e39d58b88a836483732a351b1ecee0bb97ab03c; FR-351-AC-6, FR-351-AC-7, crates/filament-contract-model/src/checked_package/v2/step_relation.rs, tests/it/checked_package_v2_frame_entries.rs"
review_set: subset
---

## Summary

Gap analysis scoped to the PR diff. Ticket: QSL-2128. Plan completion: not assessed. The matrix is the computed `quoin matrix --json` output (quoin 0.29.1, RC=0, /tmp/ir-qsl2128-reader-reviews/matrix.json), not a hand-made one. FR-351-AC-6 is `tagged` by 3 test binders and FR-351-AC-7 by 1. Both show `no run evidence`, as do all 1323 criteria in the repository, so that status is not specific to this PR. Upstream oracle: FSL FR-940-AC-27 at f0a8e7f6, identical at b891628f.

## Examined scope

- FR-351-AC-6 (examined). The tests `..._is_optional_and_legacy_four_binding_admits`, `..._preserves_supplied_order_over_one_root_and_roundtrips` and `..._first_use_subject_controls_survive...` cover: admission of `[W, V]` over one root, the getter returning `[W, V]` with the root id, the canonical write then read, the legacy 4-binding package and the no-member package giving `None`, `step_relation_body` being equal with and without the member, and first-use subjects `[V, W]` being unchanged. A mutant that rejects the fifth binding fails 3 tests, per the coder's mutant log.
- FR-351-AC-7 (examined). `step_relation_declaration_order_refuses_each_defect_at_its_pointer` asserts the code, cause `None` and the full pointer, including the node position, for: wrong name, non-Group value, malformed entry (Text), duplicate, unknown, non-reference value, a join to the operation anchor, an omitted alias, an empty group, a supplied offender before omission, the first of two offenders, and a sixth binding. The baseline is admitted first. The "reference to another subject's root" case is the gap in FND-001.
- Reverse gap (examined). The new production code (getter, `DeclarationOrder`, `ordered_prefix`, the view and validate blocks) all sits in the FR-351-owned module and is exercised by the tests above. No production code is unowned.
- Stubs and tautologies (examined). There are none. The `ExactSizeIterator` count is asserted against `size_hint` in `declared_order`.

## Verdict

One medium coverage gap: AC-7's cross-subject root join is untested. Everything else in AC-6 and AC-7 is bound to a failing-capable test.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-351-AC-7 "a reference to another subject's root" is not tested. `declaration_order_package` binds `V` and `W` to one model root (tests:5794), so no other subject's root exists. The "wrong root join" case (tests:6008) points at the operation anchor, which is not a model root. Two mutants pass every test: one that replaces the per-alias equality at step_relation.rs:766-769 with membership in the subjects' root set, and one that compares against `subjects[0]`. FSL FR-940-AC-27 also names "an alias joined to the other subject's root". Fix: build a fixture with two subjects over distinct model roots, swap their declaration-order targets, and assert `/body/members/4/value/members/0/value/target`. | tests/it/checked_package_v2_frame_entries.rs:5794,6008; crates/filament-contract-model/src/checked_package/v2/step_relation.rs:766-769 |
