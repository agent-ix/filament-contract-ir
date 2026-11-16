---
id: SR-20261116122945
title: "gap-analysis of IR-1550 remaining FR-369 rows (PR #1031)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1031 at 91caae44 (base origin/main 8eb5201f); FR-369-AC-18 to FR-369-AC-22, TC-476 steps 18-22, spec/checked_package/matrix/tests.md; crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs, tests/it/checked_package_v2_probabilistic.rs, tests/it/checked_package_v2_probabilistic_delay.rs"
review_set: subset
---

## Summary

This is the gap analysis of filament-contract-ir PR #1031. Ticket: IR-1550. Committed copy: reviews/ir-1550-pr1031-gap-analysis.md. That name does not collide with PR #1029's reviews/ir-1550-gap-analysis.md. Plan completion: not assessed.

`quire coverage --scope . --strict` exits 1 with 61 unbacked rows. That is the same count as before, so the PR adds none, and FR-369 is 22/22.

Mutants were applied the way the PR #1029 review applied them. Each mutant is gated by an env var in `probabilistic_join.rs`. The tree was built once, and every mutant was run against the 29 `probabilistic` tests in `it`, with `FILAMENT_SPECIFICATION_DIR` set so the delay tests run. All seven mutants the coder listed are killed:

- M1, the integer-range arm returns false: killed by `the_public_reader_refuses_a_support_value_outside_its_declared_type`.
- M2, the rational arm returns false: killed by the same test.
- M3, the text arm returns false: killed by the same test.
- M4, length counted without normalization: killed by the same test.
- M5, the unit comparison always passes: killed by `a_delay_quantity_in_another_unit_than_the_time_source_is_refused`. It fails because the package is admitted, which proves that the cloned `Example::ms` unit node is valid. The test therefore does not pass for another reason.
- M6, no support-value visit: killed by `the_join_charges_each_support_value`, with a slope of 15, not 16.
- M7, no delay-member visit: killed by `the_join_charges_each_delay_quantity`, with a slope of 15, not 16.

The mutants were reverted. Nothing was committed.

## Examined scope

- FR-369-AC-18, test `the_public_reader_refuses_a_support_value_outside_its_declared_type` (examined). It covers the integer bounds and the values 0, 13 and -1. It covers `Rational[1, 3; 1, 4]` with its bounds, 3/4 and 2/4 admitted, and 0, 4, 1/5, 5/4 and -1 refused. It covers `Text[2, 3; nfc]`. See FND-001. The admitted `2/4` and `e\u0301b` are the spec defects described in spec-review FND-002 and code-review FND-002 and FND-003.
- FR-369-AC-19, test `the_public_reader_refuses_a_reward_body_of_another_type` (examined). It asserts `ill_typed`/`type-mismatch` with the locus at the reward function, so the FR-322 "Results" check is reached for a reward's function in the public read. The behaviour predates the PR: the test pins it and does not fail without a new check. See spec-review FND-003 for the shape the test uses.
- FR-369-AC-20 and AC-22, tests `the_join_charges_each_support_value` and `the_join_charges_each_delay_quantity` (examined). M6 and M7 kill them. See code-review FND-005.
- FR-369-AC-21, test `a_delay_quantity_in_another_unit_than_the_time_source_is_refused` (examined). It checks `discrete` (the second row) and `uniform_interval` (`upper`), each at the exact quantity pointer of the last node. M5 kills it.
- TC-476 steps 18 to 22 (examined). Each step names its AC, and each AC has a `#[trace("TC-476", "FR-369-AC-n")]` test.
- The tests.md FR-369 and TC-476 rows (examined). They match the traces.

## Verdict

Changes requested on coverage. Every flipped row has a test, and each of the coder's seven mutants is killed. But the AC-18 test covers only the `nfc` profile and only a rational with no unit. The `binary-utf8` defect (code-review FND-001) and the quantity-over-`Rational` arm therefore go unseen (FND-001).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The FR-369-AC-18 test declares only `Rational[1, 3; 1, 4]` with no unit, and `Text[2, 3; nfc]`. The FR-369 statement also covers "the magnitude of a quantity over a rational" (`MemberType::CarriedRational`) and every text profile. No test declares a `Quantity<u, Rational[..]>` parameter, or a `binary-utf8`, `unicode-scalars`, `nfd`, `nfkc` or `nfkd` text. So dropping `CarriedRational` from the rational pattern would survive, and the wrong `binary-utf8` length (code-review FND-001) passes the suite. | tests/it/checked_package_v2_probabilistic.rs:1343-1378 |

## Dispositions

Round 1, at head 0f400cd7.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The AC-18 test now declares `Text[2, 3; profile]` for all six profiles: `l` nfc, `ld` nfd, `lk` nfkc, `lq` nfkd, `ls` unicode-scalars and `lb` binary-utf8. It also declares `c: Quantity<ms, Rational[1, 3; 1, 4]>` (`Money`), and covers each one's admitted values, values outside the bounds, non-canonical spellings and duplicates. Dropping `CarriedRational` from the rational arm is now killed, as is bytes-as-chars. I also mutated each bound comparison on its own. Numerator min, numerator max, denominator max, integer lower, integer upper, text lower and text upper are all killed. Denominator min survives: see FND-002. |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | low | Removing `denominator < Exact::from(bounds.denominator_min)` alone survives all 29 `probabilistic` tests. Both rational types in the test, `Ratio` and `Money`, declare `minDenominator` 1, and every canonical rational has a positive denominator. So the lower denominator bound is never the rule a case breaks. A type such as `Rational[1, 3; 2, 4]` with the support value `1` (denominator 1) would cover it. | tests/it/checked_package_v2_probabilistic.rs:118-124,160-175, crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs:209 |

## Dispositions (round 2)

Round 2, at head 27060075 (fix commit b78d7e9a).

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-002 | fixed | The new parameter `r2: Rational[1, 3; 2, 4]` (`RatioFrom2`) admits `1/2`, `3/2` and `1/4`, and refuses `1`, `3` and `1/5` at the literal. The denominator-min mutant (N2), which survived round 1, is now killed. All 18 env-gated mutants are killed at 27060075: each bound comparison on its own (N1 to N8), `CarriedRational`, bytes-as-chars, normalization ignored, non-canonical rational and integer admitted, unit comparison, both visits, the spelling pointer (P1) and integer-at-`value` (P3). |
