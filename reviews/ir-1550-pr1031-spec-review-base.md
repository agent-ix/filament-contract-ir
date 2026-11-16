---
id: SR-20261116122946
title: "spec-review base of IR-1550 remaining FR-369 rows (PR #1031)"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1031 at 91caae44 (base origin/main 8eb5201f); spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md, spec/checked_package/matrix/TC-476-probabilistic-node-reader.md, spec/checked_package/matrix/tests.md"
review_set: subset
---

## Summary

This is the base spec review of the FR-369, TC-476 and tests.md changes in filament-contract-ir PR #1031. Ticket: IR-1550. The PR is the follow-on to PR #1029. It replaces the FR-369 Planned section with three Behavior paragraphs and adds FR-369-AC-18 to AC-22. Committed copy: reviews/ir-1550-pr1031-spec-review-base.md. That name does not collide with the PR #1029 files reviews/ir-1550-{code-review,gap-analysis,spec-review-base}.md.

The rules were compared with FSpec at filament-specification `origin/main` 959a1982:

- FR-640: Properties, Reader rules 2, 4 and 5, and the visit charge.
- FR-322: the Rational value type row, "Text literal membership", the rational and quantity literal spelling rule, and "Results".
- FR-141: text profiles and length.

`quire validate` exits 0 on the three changed files.

## Examined scope

- FR-369 support-range paragraph (examined). The integer and rational rules match FR-640 Reader rule 2, "a value outside the parameter's type: `invalid-value`", and FR-322's type row, "the rationals in lowest terms whose numerator is in `[n1, n2]` and whose positive denominator is in `[d1, d2]`". Both bounds are inclusive. Lowest terms with a positive denominator is right for negative values and for zero, which reduces to 0/1. The text rule is wrong (FND-001).
- FR-369 delay-unit paragraph (examined). FR-640 Properties states "Time quantities are in the timed model's time unit (FR-415), converted exactly by FR-142". Reader rule 5's "parameters that do not match the family" gives the `invalid-value` cause. FR-640 types `exponential`'s rate as "an exact rational per time unit", a rational with no unit, and `uniform` has no parameters. So comparing the `uniform_interval` bounds and the `discrete` delays is the whole rule. Clean.
- FR-369 reward-body paragraph (examined). See FND-003.
- The sentence on support values of record, tuple, variant, option or collection type (examined). FR-640 makes a support value "a literal of the parameter's declared type". A type with no literal form therefore has no admissible support value. Clean.
- FR-369-AC-18 (examined). See FND-002.
- FR-369-AC-19 (examined). See FND-003.
- FR-369-AC-20 and FR-369-AC-22 (examined). See FND-004.
- FR-369-AC-21 (examined). It is testable and matches the delay-unit paragraph.
- TC-476 steps 18 to 22 and its Status (examined). They agree with the ACs. "Implemented" depends on FND-001 and FND-002.
- The tests.md FR-369 and TC-476 rows (examined). They agree with TC-476. FND-001 and FND-002 make "Implemented" overstated.
- The removal of the Planned section (examined). With FND-001 to FND-003 open, FR-369 still has open items.

## Verdict

Changes requested. The integer, rational-bound and delay-unit rules are right and complete against FR-640 and FR-322.

The text rule follows FR-141's runtime normalization. The checked-package reader is governed by FR-322 "Text literal membership", which normalizes nothing, requires the value to be in normal form already, and measures `binary-utf8` length in bytes (FND-001).

AC-18 tells the reader to admit two spellings that FR-322 refuses: a rational not in lowest terms (`2/4`) and a text that is not normalized (FND-002).

The reward-body paragraph presents IR's Reading A as settled fact. FR-640 calls the reward function a clause function, and FR-322 leaves bodyless clause functions out of "Results". So no reward that FR-640 describes ever has its expression's type compared (FND-003).

The AC-20 and AC-22 slopes fix an implementation total, not the requirement (FND-004).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-369's text rule measures "scalar count after its profile's normalization (FSpec FR-141)". FR-141 describes runtime evaluation. For a literal in a checked package, FR-322 "Text literal membership" rules instead. There, "the reader applies no normalization and never rewrites it". Under `nfc`, `nfd`, `nfkc` and `nfkd` the value "SHALL already equal its own normalization", and its length is the scalar count of the value as written, or the UTF-8 payload byte count under `binary-utf8`. Both FR-141 and FR-322 count bytes for `binary-utf8`, and the paragraph does not say so. | spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md:141-147 |
| FND-002 | high | AC-18 requires the reader to admit "a rational not in lowest terms that reduces into range and a text whose normalized length is in range". FR-322 refuses both. A `rational` literal has "one spelling: `2/2`, `1/1`, `02`, `+1`, `2/4` and `1/0` refuse `invalid_package`/`invalid-value`". A text not in its profile's normal form is not a member of `Text[min, max; profile]`. Admitting both also lets two support values that are equal, such as `1/2` with `2/4`, or `éb` with `e\u0301b` under `nfc`, pass FR-640 rule 2's "repeats a value". | spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md:188 |
| FND-003 | medium | FR-640 says the reward's function "is a clause function ... whose result type is `value_type`". FR-322 "Results" says "A function node with no `body` binding (a clause function) is outside this comparison". So for every FR-640-shaped reward, the type of the expression is never compared. Only the function's own `semantic_type` is checked, under AC-9. FR-369 states this reading, Reading A, as fact and flips the row to Implemented. It does not mark the reading as IR's reading of FR-322, as it does elsewhere in the same FR. It does not record that FR-640 is silent on whether a reward function has a `body`. AC-19 reads as if reward bodies are checked, but it only covers a shape FR-640 does not describe: a clause function that also has a `body` binding. | spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md:155-163,189 |
| FND-004 | low | AC-20 and AC-22 fix "16 work units a value/row, one of them the join's". The other 15 units are charges by earlier stages that FR-369 states only as "one visit for each ... member". So the ACs bind a total the requirement does not define. A change to an earlier stage's accounting breaks these ACs, though the join rule they verify has not changed. The requirement is one join visit per support value and per delay member, made before inspecting it. The AC should state that, for example as a budget one unit short that stops at the value's pointer. | spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md:190,192 |
