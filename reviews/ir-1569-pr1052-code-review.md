---
id: SR-20261118225202
title: "code-review and rust-review of IR-1569 Boolean and enum probabilistic join support values (PR #1052)"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1052 at 39d888c3 (base origin/main 836ae95c); crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs, tests/it/checked_package_v2_probabilistic.rs"
review_set: subset
---

## Summary

This is the code review of filament-contract-ir PR #1052, with the rust-review lane folded in. Ticket: IR-1569. The reference text is FSpec FR-640, FR-405 and FR-420 at filament-specification `origin/main` f92839de. The diff scope is `git diff origin/main...HEAD`. The branch is level with `origin/main` 836ae95c.

The work ran in a scratch worktree, `CIR-1052-review`, at the PR head. These gates passed:

- `cargo fmt --all -- --check`
- `make string-edge`
- `cargo clippy --locked --workspace --all-targets --features conformance-cli -- -D warnings`
- `cargo clippy --locked -p filament-contract-model -- -D warnings`

The full suite, `cargo test --locked --workspace --all-targets --features conformance-cli --no-fail-fast`, was run twice: once on the PR head and once on a clean `origin/main` 836ae95c worktree with its own target dir. Both runs had `FILAMENT_SPECIFICATION_DIR` set to FSpec f92839de, `QUIRE_LOCK_CHECKER` set to quire-0.52 and `fsl-string-edge` on PATH. The PR gave 2186 passed and 33 failed; main gave 2184 passed and 33 failed. The two failing sets are identical: 8 in `it`, 24 in the model lib and 1 in `temporal_binding_public`. The sampled failures refuse the `claim_names` member that FSpec main fixtures now carry, which is QSL-2410 / open PR #1046 and not this PR. The two new tests pass.

The reviewer applied eight mutations of their own to `probabilistic_join.rs` and ran the two new tests against each:

- M1: drop the enum `type_id` check.
- M2: drop the enum case check.
- M3: Boolean arm accepts any value.
- M4: remove the Boolean arm.
- M5: remove `Variant -> Enum`.
- M6: remove the Variant arm.
- M7: invert the `type_id` comparison.
- M8: skip `not_member`.

All eight were killed. A scratch probe read packages through the public `CheckedPackageV2::read`, and its results are recorded below. Neither the mutations nor the probe were committed.

## Examined scope

- `member_literal_kind` (examined). `MemberType::Variant(_)` now maps to `LiteralKind::Enum`. `slot_type` gives `Variant` for every selected variant declaration, payload cases included. So the expected kind for a payload variant is also `enum`. The probe shows that such a literal is refused earlier, by the generic literal check: `ill_typed`/`operator-ineligible` at the literal's `type`. This happens for both a payload case and a no-payload case of a mixed variant, so a payload variant is never admitted.
- `not_member`, Boolean arm (examined). `value.as_bool().is_none()` refuses a non-boolean value (M3 killed). A Boolean literal typed at another type (Tier, or the integer builtin) is refused `ill_typed` by the generic literal check (probe).
- `not_member`, Variant arm (examined). The value must be a string that names a case of the declaration (`selected_variant_case`), and `type_id` must equal the declaration. An unknown case, a case in the wrong letter case, a literal typed at another enumeration and a non-string value (probe: `1`) each refuse `invalid-value` at the value. M1, M2, M6 and M7 were all killed.
- `not_member` signature and call site (examined). The `&RetainedModels` threading is the minimal change. The doc comment is updated.
- The repeated-value check in `probabilistic.rs` `SupportKey`/`literal` (examined, outside the diff; reached newly by this change). See FND-001.
- The IR-1549 FND-001 claim (examined). `the_public_reader_refuses_a_value_outside_the_form_rules` asserts that a workload weight of `-1` refuses `invalid_package`/`invalid-value`. The claim holds.
- Rust idioms (examined). No panics, no `unsafe`, no new integer conversions and no unbounded work were added. The enum check is one table lookup, inside the per-value visit that is already charged.

## Verdict

The Variant/Boolean admission and its refusals are correct, and the new tests kill every mutation the reviewer tried. One real defect: admitting Boolean values exposes a hole in the earlier repeated-value check. A support of `true, true` is admitted, which breaks FR-640 Reader rule 2 and FR-369-AC-3. Not mergeable until FND-001 is fixed. FND-002 is a test-strength nit.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | A repeated Boolean support value is admitted. `SupportKey` keys a literal by its string or u64 value only. A JSON boolean falls to `Distinct(index)`, "never equal to another", so the probe admits `random_parameter` support `[true, true]` through the public reader. FR-640 Reader rule 2 and FR-369 ("a support value that repeats an earlier one", FR-369-AC-3) require `invalid_package`/`invalid-value` at the later value. Before this PR a Boolean support was unreachable, because it was refused as the wrong kind. Fix: add a Boolean key (for example `SupportKey::Boolean(kind, bool)` from `value.as_bool()`) and test `[true, true]` and `[false, true, false]` through the public reader. Repeated enum cases are already refused (probe). | crates/filament-contract-model/src/checked_package/v2/probabilistic.rs:351-372 |
| FND-002 | low | `the_public_reader_admits_boolean_and_enum_support_values` asserts only `support.len() == 2`. FR-369-AC-23 says the values "are read back with their supports", but the test never checks the read-back values (`true`/`false`, `Gold`/`Silver`) or the probabilities (1/2). Its doc comment also says "a mistyped or malformed one refuses", though the test refuses nothing. | tests/it/checked_package_v2_probabilistic.rs:1667-1682 |
