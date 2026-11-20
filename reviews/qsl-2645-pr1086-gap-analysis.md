---
id: SR-17940486709564847048102
title: "gap-analysis of QSL-2645 identity-row join against FR-369-AC-25/26 and TC-484"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1086 at 7b30d82da (git diff origin/main...HEAD); FR-369-AC-25, FR-369-AC-26, TC-484 steps 1, 3, 5, 6 and the golden pair, FR-358, spec/checked_package/matrix/tests.md, crates/filament-contract-model/src/checked_package/v2/source_join.rs, tests/it/checked_package_v2_source_objects/probabilistic/join.rs, tests/fixtures/qsl-2645-golden"
review_set: subset
---

## Summary

This is a planless gap analysis (Plan completion: not assessed). Ticket: QSL-2645. The authority is the arch ruling comment on QSL-2645 and the merged FR-369, FR-358, FR-374 and FR-375 (#1085).

Every new test carries `#[trace("TC-484", "FR-369-AC-25")]` or `"FR-369-AC-26"`.

AC-25 is covered by:

- the golden pair (`tests/fixtures/qsl-2645-golden/package.json` and `typed-read.json`, checked against the in-code builder and against the independently built declared-node id);
- the selected-model pass-through;
- the empty-package case.

AC-26's join half is covered by:

- `nudge` on the delay, the random parameter and the reward;
- `j`;
- support `4`;
- the omitted `O::step`;
- `j` in place of `k`, and no parameter;
- the clock variant (`x, k` admitted, `k` alone and `k, x` refused);
- the budget stop at every limit below the full count.

The strict-read half stays with AC-27/AC-28 in `probabilistic.rs`.

Tests run: 27 passed, 0 failed (`cargo test --locked --features conformance-cli --test it checked_package_v2_source_objects::probabilistic`, one cargo slot, a private CARGO_TARGET_DIR, FSpec origin/main 2b9645be); `rustfmt --check` is clean on the two new files.

## Verdict

**Partially covered; the matrix row over-claims.** The join behaviour AC-26 enumerates is tested, but three gaps remain:

- **FND-001.** TC-484 step 5's body variants ("`j` (read by the body)", "none with the constant body `1`") are not built. Every reward function in `join.rs` is the bodyless clause function of `reward_function`, so the strict read's walk never has a body to admit before the join refuses.
- **FND-002.** The join charges a fifth visit kind that the FR-369 enumeration and TC-484 step 6's independent count do not list: one per parameter entry examined in `random_parameter`. `AC25_VISITS` includes it ("one parameter entry"). One visit per workload row is FR-369's "operation reference joined", because a weights row is an operation reference, so that charge is in spec.
- **FND-003.** On the coder's question: the refusal of every support value for an entry typed by a source enumeration is a false refusal under FR-369 Behavior. Behavior requires membership "by the rules the strict read applies to a selected parameter", and those rules admit an `enum` literal naming a case (FR-369-AC-23). The join cannot apply that rule because `SelectedSourceObjects` does not carry the FR-352 `SelectedSourceEnumRow` cases, so this is an input gap the ruling did not cover. Arch must rule on it; it is not an implementation choice. The bounded-quantity and integer- or decimal-carried-quantity refusals match the strict read's selected rule: `member_literal_kind` maps only `Quantity` and `CarriedRational`. They are therefore no join-specific false refusal; FR-322/FR-369 leaves them open for both stages.

FND-001 in the code-review SR also makes the matrix claim "refusals `InvalidValue` at the strict read's member" untrue for `operation` and `parameter`.

The coder's two surviving mutants could not be re-identified from the PR or the ticket, because neither lists them. Mutants were not re-run (Titan is thermally limited). By inspection, the `declared.is_none() ||` guard in `reward` and the `continue` arms of `workload` are the likely equivalents: `from_stage_rows` guarantees a row for every source type (`MissingIdentityWitness`), so the `row(..) == None` arms are unreachable.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-369-AC-26 and TC-484 step 5 name the reward function "listing a parameter `j` (read by the body) in place of `k`, and listing none with a constant body `1`". `with_reward` builds the bodyless `reward_function(listed)` for both cases, so the step's premise is never exercised: that the strict read's walk admits the body (a listed `j`, a constant) and only the join refuses. The matrix marks FR-369-AC-25/26 ✅ Implemented. Add the two body variants: a `body` reading `j`, and a constant `1` | tests/it/checked_package_v2_source_objects/probabilistic/join.rs:127-137,276-292; tests/it/checked_package_v2_source_objects/probabilistic.rs:118-136 |
| FND-002 | medium | FR-369 states the join charges "one visit per operation reference joined, per operation entry and supertype edge read, per support value checked and per reward function parameter compared". `random_parameter` also charges one visit for each operation parameter entry examined, and `AC25_VISITS` counts it, so TC-484 step 6's count "independently of the reader" over the four spec kinds differs from the test's. The charge is FR-358-consistent (a sequence scan charges each examined element), so the fix is spec-side: add "per parameter entry examined" to FR-369's enumeration and TC-484 step 6. Otherwise, drop the charge | crates/filament-contract-model/src/checked_package/v2/source_join.rs:273-290; tests/it/checked_package_v2_source_objects/probabilistic/join.rs:396-401 |
| FND-003 | medium | An entry typed by a source enumeration refuses every support value: `slot()` returns `None`, so every value is refused at the value. FR-369 Behavior requires membership "by the rules the strict read applies to a selected parameter", and those rules admit an `enum` literal typed at the declaration and naming one of its cases (FR-369-AC-23). The result is a false refusal of a valid FR-640 package (FR-640 admits "a random parameter of any parameter type"). `join_probabilistic` has no input carrying the FR-352 `SelectedSourceEnumRow` cases, so arch must rule on the input (or on a narrowing of FR-369). The bounded-quantity and integer- or decimal-carried-quantity refusals mirror the strict read's selected rule and are no join-specific defect | crates/filament-contract-model/src/checked_package/v2/source_join.rs:137-205; spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md:283-292 |

## Dispositions

Round 1, at head 0c9d47aea.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `a_reward_function_with_a_body_is_joined_by_its_listed_parameters` builds real `body` bindings with `with_reward_body`. `k` read by the body joins. `j` read by the body in place of `k`, and no parameter with the constant body `fixture/src/one`, are each admitted by the strict read (`admitted(..)`) and then refused by the join at the reward's `expression`. `nudge` refuses at `operation` |
| FND-002 | fixed | FR-369 and TC-484 step 6 now list "per parameter entry of the operation read", and the per-node, per-byte and lookup charges as well. `ac25_work` counts exactly those kinds |
| FND-003 | deferred | QSL-2655 (Backlog) owns the source-enumeration input ruling. FR-369 Planned names it and the matrix rows are 🚧 Partial; the bounded and int/decimal carried quantities are stated as matching the strict read |
