---
id: SR-20261115234918
title: "gap-analysis of IR-1549 probabilistic node tag and four forms (PR #1027)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1027 at ed31ba20 (base origin/main 80e3f67f); FR-369, TC-476, spec/checked_package/matrix/tests.md; crates/filament-contract-model/src/checked_package/v2/probabilistic.rs"
review_set: subset
---

## Summary

This is a planless gap analysis of filament-contract-ir PR #1027. Ticket: IR-1549. Plan completion was not assessed.

Each AC of FR-369 was checked against its tagged test and against the code. FR-369 was then checked against FSpec FR-640 at filament-specification `origin/main` c143e2b5: its Properties, its five Reader rules, its budget paragraph and its Typed read. IR-1549's "Done when" and ACs were checked as well.

`quire coverage --scope . --strict` reports FR-369 4/4 backed. It also reports 61 unbacked rows, the same 61 as on `origin/main` 80e3f67f, all in other tests. So `make spec` exits 1 identically on the baseline.

A scratch probe, not committed, read packages through the public `CheckedPackageV2::read`.

## Examined scope

- FR-369-AC-1 (examined). `each_probabilistic_shape_decodes_and_round_trips` decodes the tag and the four forms and checks `all()`. It then reads and re-encodes seven shapes: the four forms, with all four delay families. This is backed.
- FR-369-AC-2 (examined). `a_missing_or_extra_member_is_refused` covers a missing, extra and swapped member and a `declaration` for each shape, plus a short support row and an extended discrete row. It is backed, apart from FND-003.
- FR-369-AC-3 (examined). `a_value_outside_the_form_rules_is_refused` covers an empty support, a zero support weight, an unknown family, a zero discrete weight and a zero rate, and admits a zero workload weight. This is backed.
- FR-369-AC-4 (examined). `an_unknown_tag_still_fails_decode_with_the_existing_error` checks `from_wire`, foreign forms and `unsupported_node_tag` at `/node_tag`. This is backed.
- IR-1549 "Done when" and ACs 1-3 (examined). The tag and four forms are carried and the strict reader decodes them. ACs 1-3 map to FR-369-AC-1, AC-2 and AC-4. The ticket's "five new node shapes" is loose wording: FR-640 defines one tag and four forms, and the PR implements exactly those.
- IR-1549 AC 4, FSL bumps its lock (examined). It can only be verified after merge. The probe shows the public reader admits a well-formed `random_parameter` today. FSL's test needs only `Read::Verified` and the node body, not a typed read.
- FR-640 Reader rules 1-5, `conflicting-definition` and the Typed read (examined against FR-369's exclusion paragraph). These are FND-001 and FND-002.

## Verdict

Changes requested. IR-1549's "Done when" asks only for the tag, the forms and strict decode, so deferring FR-640's join checks and typed read is a legitimate follow-up. But the deferral is not recorded as planned work with an owner. FR-369 says the checks are "not part of this requirement", and the matrix row reads ✅ Implemented. Meanwhile the strict reader admits packages that FR-640 requires it to refuse (FND-001). Two FR-640 rules need no other node, yet they are neither implemented nor deferred (FND-002).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-640's node-joining Reader rules are absent from this repo's spec, with no planned record, no owner and no ticket. FR-369 only says they "are not part of this requirement", and the matrix row is ✅ Implemented. The missing rules are: rule 1 operation references; rule 2 parameter name and type; rule 3 workload omit, repeat, misname and member order; rule 4 reward value_type, clause function and state reads; rule 5 delay on a non-`time dense` model; `conflicting-definition` across nodes; and the Typed read. The public strict reader therefore admits an out-of-order workload and operation references to a non-`model` node (probe). Record them as 🚧 Planned (FR-369 PARTIAL, or a planned FR plus matrix row) with owner "planner to assign". | spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md:55-57 |
| FND-002 | medium | Two FR-640 rules need no other node, yet they are neither implemented nor listed among FR-369's exclusions. An exact weight "in lowest terms" (FR-640 Properties) is not checked: `2/2` is admitted. A support that "repeats a value" (Reader rule 2) is not checked: a support of `1, 1` is admitted. FR-369's claim to cover "the rules that need no other node" is therefore incomplete. | crates/filament-contract-model/src/checked_package/v2/probabilistic.rs:158-168 |
| FND-003 | low | FR-369-AC-2 names "a short or extended nested support, weight or delay row", but no test damages a workload `weights` row. Only a support row (short) and a discrete row (extended) are exercised. | crates/filament-contract-model/src/checked_package/v2/probabilistic.rs:595-617 |

## Dispositions

Round 1 checks the fix commits 6d0eb621 and d0c796b0 at head d0c796b0. `quire coverage --scope . --strict` reports FR-369 5/5 backed. It still reports the 61 unbacked rows that `origin/main` has.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | FR-369 now has a `### Planned` section listing rule 1 to rule 5 joins, `conflicting-definition` and the Typed read. It marks the requirement PARTIAL with "Owner: planner to assign". The tests.md FR-369 and TC-476 rows and the TC-476 Status now read Partial / Planned. |
| FND-002 | fixed | `weight()` checks lowest terms with `natural::coprime`, and the support check refuses a repeated value at the later row. Tests cover 2/2, 0/5 and a support of 1, 3, 1, and the public reader refuses 2/2. Mutations M18 and M19 are killed. |
| FND-003 | fixed | `a_missing_or_extra_member_is_refused` now pops and extends a workload weights row, and refuses `malformed_wire` at `/body/members/2/value/members/0/members`. |
