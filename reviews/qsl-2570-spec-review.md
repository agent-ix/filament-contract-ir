---
id: SR-2026111918035700000001
title: "spec-review of QSL-2570 FR-038 family guard parts"
type: SpecReview
analysis: integrity
scope: "agent-ix/filament-contract-ir; PR #1079 at 5eb7243b; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (Family guard parts, FR-038-AC-576..580), spec/checked_package/matrix/TC-490-family-guard-part-quantifier.md, spec/checked_package/matrix/tests.md, tests/fixtures/family-guard-part/family-part-vectors.json"
review_set: subset
---

## Summary

Spec review of the FR-038 "Family guard parts" section, FR-038-AC-576 to FR-038-AC-580, TC-490 and the FP-01 to FP-06 fixture. The review checked them against filament-specification main at ccac0403 (FR-322 "Quantified guards and invariants", FR-322-AC-479, the V2 `schema.json` `TimedGuardPart`/`TimedQuantifierBinding`, and the `timed-clause-node-vectors.json` cases CL-57, CL-58, CL-63 and CL-64). Ticket: QSL-2570. `quire validate --scope . 'spec/**/*.md'` fails only on the known `spec/tests.md:8`, which also fails on main.

## Examined scope

- Three part layouts: these match the closed `TimedGuardPart` oneOf (predicate; predicate+binders; predicate+binders+quantifier), and `each`/`some` matches the `TimedQuantifierBinding` enum. (examined)
- Refusals and pointers: CL-63 refuses `malformed_wire` at the part, CL-64 refuses `invalid_package`/`timed-node-reference` at `.../members/2/value`, and a family part without `quantifier` refuses at the part. All three agree with FR-322 and the vectors. (examined)
- Family definition: the PR uses FR-322's wire definition, "has binders and a constraint place with a slot step whose maximum is unbounded". (examined)
- Check order: the PR orders the joins as reference joins, then the binder join, then the family rule. FSpec says nothing about this order, so nothing contradicts it, and FP-06 can fail. (examined)
- Today's behavior: in `timed_model.rs`, `reference()` on members[2] means FP-01, FP-02 and CL-64 currently refuse `malformed_wire` at the quantifier member. The reader already supports a slot maximum of `unbounded`. The FP-03 edits replace every `"3"` copy of the `ts` place key; a jq search of the fixture found 10 copies, all of them patched, and no digest. FP-03 therefore admits today. This was checked by reading the code, not by running it. (examined)
- Ids: FR-038-AC-576..580 and TC-490 are not on main and not in open PRs #1065, #1072, #1074, #1076, #1077 or #1078. (examined)

## Verdict

Not mergeable yet. One high finding: FP-01 and FP-02 carry the same guard node id, and AC-576 asserts that the two packages differ only in the quantifier text. That is a layout assumption the PR cannot know until FSpec fixes it, and it contradicts FR-322-AC-479 and this PR's own prose. The fix is a small rewording. Everything else is consistent with FSpec.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FP-01/FP-02 keep CL-57's guard node id unchanged, and AC-576 plus TC-490 step 2 assert the two packages differ only in the quantifier text. This conflicts with FR-322-AC-479 ("the two guard nodes have different ids") and with FR-038:1945 ("read back with whatever distinct ids they carry"), and it assumes the quantifier is not in the id path | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6310 |
| FND-002 | low | The document `remove` of `/types/3/fields/2/multiplicity/upper` has no preceding `test` op, although AC-580 and the README say every edited member is checked first | tests/fixtures/family-guard-part/family-part-vectors.json:15 |
| FND-003 | low | No vector covers FR-322-AC-479's Map family (`um: Map<Text, Timer>`). The rule does not depend on the collection kind, so this is coverage only | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6310 |
