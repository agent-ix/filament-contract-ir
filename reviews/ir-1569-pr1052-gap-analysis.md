---
id: SR-20261118225203
title: "gap-analysis of IR-1569 Boolean and enum probabilistic join support values (PR #1052)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1052 at 39d888c3 (base origin/main 836ae95c); FR-369, TC-476, spec/checked_package/matrix/tests.md; crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs, probabilistic.rs; tests/it/checked_package_v2_probabilistic.rs"
review_set: subset
---

## Summary

This is a planless gap analysis of filament-contract-ir PR #1052. Ticket: IR-1569. Plan completion was not assessed. The reference text is FSpec FR-640, FR-405 and FR-420 at filament-specification `origin/main` f92839de.

The analysis checked the new criteria against their tests, the trace tags, and the FSpec rules for other positions.

## Examined scope

- FR-369-AC-23 (examined). Its test is `the_public_reader_admits_boolean_and_enum_support_values`, tagged TC-476/FR-369-AC-23, and it reads through the public reader. Its weak assertion is recorded in the code-review SR.
- FR-369-AC-24 (examined). Its test is `the_public_reader_refuses_malformed_boolean_and_enum_support_values`, tagged TC-476/FR-369-AC-24. It covers each listed case at the value pointer with `invalid-value`.
- TC-476 steps 23-24 and the `tests.md` row (examined). They list AC-23/24, and the status text agrees.
- Other positions (examined). FR-640 types weights and rates as exact rationals, delays as time quantities, workload weights as exact weights, and a reward `value_type` as integer or quantity. So Boolean and enum literals are not affected anywhere outside `random_parameter` support values. A reward typed Boolean stays refused (FR-640-AC-4).
- Numbering (examined). FR-369-AC-23/24 and TC-476 steps 23/24 are not used on `origin/main` or by the open PRs: #1046 (FR-371, TC-477), #1049 (FR-373, TC-475/TC-480), #1050 (FR-374/375, TC-481/482) and #1051 (TC-480). `git merge-tree` of this head with each of those heads is clean. Separately, #1049 and #1051 both add TC-480. That collision is between those two PRs, not this one.
- Negative delay quantity (examined, outside the diff, raised by the coder). See FND-002.

## Verdict

The two new ACs are each covered by a failing-capable, correctly tagged public-reader test. One gap sits in the diff: the payload-variant refusal claim is untested. One gap from before this PR is raised for the record: the negative-delay cause code. Neither of these blocks merge. The blocking item is code-review FND-001.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The new FR-369 prose says a support value "of a variant type with a payload case ... has no literal kind and is refused", but no test exercises a payload variant. The probe shows the refusal is `ill_typed`/`operator-ineligible` at the literal's `type`, from the generic literal check, not this join. It is refused for both the payload case and the no-payload case of a mixed variant. Add one public-reader case to the AC-24 test (or to FR-369-AC-5's mistyped-value test) that pins the code and pointer. | spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md:183-185 |
| FND-002 | medium | Pre-existing and outside the diff. A negative time quantity in `uniform_interval` or a `discrete` delay refuses `malformed_wire` (`quantity()`, tested by `a_negative_time_quantity_is_refused` under FR-369-AC-2, "a binding of another term kind"). It is a well-formed quantity literal of the right term kind, and FR-369 states no such refusal. FR-640 Reader rule 5 refuses "parameters that do not match the family" `invalid-value`, and FR-420 defines `discrete` delays as non-negative. So `invalid_package`/`invalid-value` is the FSpec reading. Fix here with `Fail::Value`, an FR-369 bullet and a test moved to AC-3, or file a separate IR ticket. This finding does not gate #1052. | crates/filament-contract-model/src/checked_package/v2/probabilistic.rs:418-424 |
