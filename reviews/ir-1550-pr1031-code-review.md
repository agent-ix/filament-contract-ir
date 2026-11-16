---
id: SR-20261116122944
title: "code-review and rust-review of IR-1550 remaining FR-369 rows (PR #1031)"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1031 at 91caae44 (base origin/main 8eb5201f); crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs, tests/it/checked_package_v2_probabilistic.rs, tests/it/checked_package_v2_probabilistic_delay.rs"
review_set: subset
---

## Summary

This is the code review of filament-contract-ir PR #1031, with the rust-review lane folded in. Ticket: IR-1550. The scope is `git diff origin/main...HEAD` only. Committed copy: reviews/ir-1550-pr1031-code-review.md. That name does not collide with PR #1029's reviews/ir-1550-code-review.md. The reference text is FSpec at filament-specification `origin/main` 959a1982 (FR-640, FR-322, FR-141), read-only.

Gates were run at 91caae44 in a dedicated review worktree under `flock /tmp/cargo-slot.lock`, with `--locked` and the target directory inside that worktree:

- `cargo fmt --all -- --check`: pass.
- `cargo clippy --locked --workspace --all-targets --features conformance-cli -- -D warnings`: pass.
- `cargo deny --locked check`: advisories, bans, licenses and sources ok.
- `quire validate` over every tracked spec, plan and review file: exit 0.
- `quire coverage --scope . --strict`: exit 1 with 61 unbacked rows, the same 61 as before. FR-369 is 22/22.
- `make string-edge`: clean. The scanner was installed the way CI installs it (`cargo install --git` of filament-spec-language, resolved to d71c0a8d) into a scratch root on PATH.
- Without the new `#[ix_trace_rs::string_edge]` on `text_length`, the scanner reports 4 branch-gating occurrences, lines 185 to 188.
- `cargo test --locked --workspace --features conformance-cli --no-fail-fast`: `it` gave 1486 passed and 2 failed. The 2 are the environment-only `lock_coherence` tests: the installed quire 0.36.2 has no `check-lock`. Every other target passed.
- `cargo test --locked --features conformance-cli --test conformance_fspec -- --test-threads=1`, with `FILAMENT_SPECIFICATION_DIR` set: 443 passed and 2 failed.
  - `model_tuple_union` failed at TK-08, `unsupported_construct` at `/semantic_graph/nodes/7/owner`. That failure is older than this PR and belongs to QSL-2253.
  - `timed_clock_no_row` failed because FSpec publishes TN-01 to TN-09 and the test expects TN-01 to TN-05. Main lacks PR #1030.
  - Neither failure touches this PR's files. The `probabilistic_delay` and `probabilistic_claims` tests pass.
- Cargo.toml and Cargo.lock are unchanged. The PR adds no pin, hash, digest, compatibility layer or tool version. The new imports (`num_bigint`, `num_integer`, `num_traits`, `unicode_normalization`) are existing dependencies. The new tests carry `#[trace("TC-476", "FR-369-AC-n")]`.

Probes were run against the PR's own reader by a throwaway test, which was reverted and not committed. They used the test's `Text[2, 3; profile]` (`l`), `Rational[1, 3; 1, 4]` (`r`) and `Int[1, 12]` (`d`) parameters. Every case below was ADMITTED:

- Under `binary-utf8`: `éé` (2 scalars, 4 bytes) and `aéé` (5 bytes).
- Under `nfc`: `e\u0301b` (not NFC), and the support {`éb`, `e\u0301b`}.
- Rationals: `2/4`, `3/1`, `+1` and `01`, and the support {`1/2`, `2/4`}.
- Integers: `+5` and `05`, and the support {`5`, `05`}.

## Examined scope

- `outside_bounds`, the integer arm (examined). The bounds are inclusive, matching FR-322 `Int[lo, hi]`. The parse is lenient: see FND-004.
- `outside_bounds` and `exact_rational`, the rational arm (examined). Reduction to lowest terms with a positive denominator is right for negatives (`1/-2` becomes `-1/2`) and for zero (0/1). It also makes non-canonical spellings admissible: see FND-003. The `CarriedRational` arm compares the magnitude in the parameter's own unit, which the existing unit check enforces.
- `text_length` and the text arm (examined). See FND-001 and FND-002. The `string_edge` marker is legitimate: the match is on the declaration's wire profile text, which the parser has already restricted to the six FR-141 profiles. But the string match is where the `binary-utf8` case falls into `_` (FND-001). An enum profile would make that arm impossible to miss.
- `Join::dense_unit` and `Join::delay` (examined). The time source's `unit` binding is compared by node id with each quantity literal's `type`. `uniform_interval` members are bindings, and `discrete` members are rows whose delay is at member 0. The pointers are right, as the tests assert. A non-quantity delay literal is refused earlier by the local `quantity()` shape check, so the `_ => true` arm is safe. `exponential`'s rate is a rational with no unit, so nothing is compared for it, which is right.
- Join charges (examined). One visit is charged ahead of each support value and each delay parameter member. M6 and M7 confirm that each is load-bearing. See FND-005.
- The reward body (examined). The PR removes its own join check. The general FR-322 "Results" check refuses a reward function whose `body` root has another type with `ill_typed`/`type-mismatch` at the function. The AC-19 test reaches that check through the public reader. See spec-review FND-003 for the bodyless case.
- The delay test's cloned `Example::ms` unit (examined). Mutant M5 makes the package admitted, so the clone is a valid second unit. The refusal the test asserts comes from the unit check, not from a defect in the clone.
- Panic surface (examined). There is no new `unwrap`, `expect` or unchecked index. `u64::try_from(usize)` is handled.

## Verdict

Changes requested. The integer and rational bound logic, the delay-unit join and the charges are correct, and every one of the coder's seven mutants is killed.

Three defects remain in the bounds check. The text length is wrong for `binary-utf8`, which needs a byte count (FND-001). Text is normalized before its length is measured, which FR-322 forbids, so values that are not members of the type are admitted (FND-002). Non-canonical rational spellings are reduced and admitted, where FR-322 refuses them, which also defeats FR-640's repeated-value rule (FND-003).

The integer parse admits `+5` and `05` (FND-004). The slope tests are brittle (FND-005).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `text_length` sends `binary-utf8` to `_ => value.chars().count()`. FR-141 and FR-322 both make `binary-utf8` length "the UTF-8 payload byte count". So `Text[2, 3; binary-utf8]` admits `éé` (4 bytes) and `aéé` (5 bytes), which FSpec refuses. FR-322's own vector DL-10 refuses `éé` against `Text[1, 3; binary-utf8]`. | crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs:182-191 |
| FND-002 | medium | `text_length` normalizes the value (`value.nfc().count()` and the other forms) before counting. FR-322 "Text literal membership" says "the reader applies no normalization". A value under `nfc`, `nfd`, `nfkc` or `nfkd` "SHALL already equal its own normalization", or it is not a member. So `e\u0301b` is admitted under `Text[2, 3; nfc]`, and the AC-18 test asserts that it is. The support {`éb`, `e\u0301b`} is also admitted, which holds two values that are equal under the profile. The fix is to refuse a value that is not in normal form, and to count scalars as written. | crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs:184-191, tests/it/checked_package_v2_probabilistic.rs:1373 |
| FND-003 | high | `exact_rational` reduces any parseable `n/d` to lowest terms. FR-322 gives a rational, and a quantity magnitude, exactly one spelling: "`2/2`, `1/1`, `02`, `+1`, `2/4` and `1/0` refuse `invalid_package`/`invalid-value`". The join admits `2/4`, `3/1`, `+1` and `01` as `Rational[1, 3; 1, 4]` support values, and the AC-18 test asserts `2/4` is admitted. The local repeat check compares spellings, so {`1/2`, `2/4`} passes too. That is two equal values, against FR-640 rule 2's "repeats a value". The repo already has `canonical_rational` in graph_storage.rs:566, which refuses these spellings for other operands. | crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs:165-179,210-217, tests/it/checked_package_v2_probabilistic.rs:1361 |
| FND-004 | medium | The integer arm parses with `spelled.parse::<BigInt>()`, which accepts `+5` and `05`. Those are not the canonical decimal integer spelling FR-322 gives integer literals. `Int[1, 12]` admits them, and admits {`5`, `05`} as two distinct support values. The local spelling check is older than this PR and lets them through too. But the new range check reads the value without enforcing one spelling, so the range rule and the repeat rule disagree on what the value is. | crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs:205-207 |
| FND-005 | low | The AC-20 and AC-22 tests assert `long - short == 16 * 10` and `16 * 20`. Fifteen of each sixteen units are earlier-stage charges: the term, the member and the row. The join's single visit is isolated only by arithmetic. M6 and M7 do kill them, with a slope of 15. But any unrelated change to earlier-stage accounting breaks both tests, and the message "16 units a value" misstates what the join charges. A budget one unit short that returns `incomplete` at the join's value pointer would pin the join's visit alone. | tests/it/checked_package_v2_probabilistic.rs:1428-1449, tests/it/checked_package_v2_probabilistic_delay.rs:364-395 |

## Dispositions

Round 1, at head 0f400cd7. Each row was checked against the fix commit 0f400cd7 itself, using env-gated mutants (built once and reverted) and a throwaway probe test re-run at the new head.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | `text_length` now returns `value.len()` under `"binary-utf8"`. The probe refuses `éé` (4 bytes) and `aéé` (5 bytes) against `Text[2, 3; binary-utf8]` at the value, and admits `aé` (3 bytes). The bytes-as-chars mutant is killed by the AC-18 test. |
| FND-002 | fixed | `text_length` no longer normalizes. It checks `is_nfc`, `is_nfd`, `is_nfkc` or `is_nfkd` and returns `None` for a value not in normal form, then counts scalars as written. The probe refuses `e\u0301b` under `nfc`, and refuses {`éb`, `e\u0301b`} at row 1. The normalization-ignored mutant is killed. |
| FND-003 | fixed | The rational arm now uses `meter.canonical_rational`. The probe refuses `2/4`, `3/1`, `+1`, `01`, `0/5`, `1/-2` and {`1/2`, `2/4`}, and does the same for a quantity over `Rational[1, 3; 1, 4]`. The mutant that admits non-canonical rationals is killed. |
| FND-004 | fixed | The integer arm now uses `meter.exact_integer`. The probe refuses `+5`, `05`, `-0` and {`5`, `05`}, the last at row 1. The mutant that admits non-canonical integers is killed. |
| FND-005 | fixed | The AC-20 and AC-22 tests now scan the last 40 budgets and require the read to stop as incomplete at each row's value or quantity pointer. They no longer compare totals of other stages. The visit mutants M6 (support value) and M7 (delay quantity, now charged at `quantity_at`) are each killed. |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | A non-canonical spelling is refused at two different pointers depending on the literal kind. For a quantity (`c`), the earlier literal stage refuses `2/4`, `+1` or `1/0` at `.../members/0/value/value`. That is the literal's `value` member, the place FR-322 names ("refuse `invalid_package`/`invalid-value` at the literal's `value`"). For a rational (`r`) or an integer (`d`), the same spelling is refused by the join at `.../members/0/value`. `support_case` then accepts either pointer for every `c` case, including bound cases such as `4` and `1/5`, which the join refuses at `.../value`. The looseness is justified only for the spelling cases. Either pin the expected pointer per case, or refuse rational and integer spellings at the `value` member too. | tests/it/checked_package_v2_probabilistic.rs:1383-1406, crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs:200-213 |

## Dispositions (round 2)

Round 2, at head 27060075 (fix commit b78d7e9a).

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-006 | fixed | `not_member` now returns `NotMember::Literal` or `NotMember::Spelling`. A non-canonical rational or quantity magnitude (`2/4`, `3/1`, `+1`, `01`, `1/0`, `0/5`, `1/-2`, {`1/2`, `2/4`}) is refused at `value_at.key("value")`, the literal's `value` member, which is where FR-322 "Scalar literal values" places a spelling defect. A value outside the bounds stays at the literal. `support_case` pins one exact pointer per case (`Refused::Not`, `Literal(n)` or `Spelling(n)`) for both `r` and `c`. The probe at 27060075 confirms both pointers. Mutant P1 (spelling refused at the literal) is killed. Integer misspellings (`+5`, `05`, `-0`, {`5`, `05`}) stay at the literal. FR-322 "Integers" only says an integer literal value "is an `IntegerString`, the canonical decimal string", and gives no refusal or pointer. FR-369 records the choice as its own, and mutant P3 (integer at `value`) is killed. Placing the integer at the literal is consistent with FR-369's stated rule but not by analogy with FR-322's other spelling rules, which are all at `value`. That is a defensible reading, not a defect. |
