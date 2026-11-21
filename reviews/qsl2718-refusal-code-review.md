---
id: SR-2026112112042500000001
title: "code-review and rust-review of QSL-2718 terminal model_root binding and object_type refusal"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1115 at f4c8c2b5 (base origin/main 5964a8c0); crates/filament-contract-model/src/checked_package/v2/claim_terminal.rs, tests/it/checked_package_v2_claim_terminal.rs, tests/it/support/checked_package.rs"
review_set: subset
---

## Summary

This is the code review, with the rust-review lane folded in, for filament-contract-ir PR #1115. Ticket: QSL-2718. It is a new review of this PR and does not reuse the #1109 records that the PR carries under `reviews/qsl2718-*.md`. The oracle, read only, is filament-specification origin/main FR-476: "Several terminal nodes on one model node", Reader order step 4 ("a `model` target that is not a `model`/`model_root` node refuses `invalid_model_binding`/`malformed-declaration` ... The locus is the target") and FR-476-AC-4 ("a `model` target that is a `value`/`parameter` node, and one that is a `model`/`object_type` node, refuse `invalid_model_binding`/`malformed-declaration`").

## Examined scope

- `validate_terminal` model-target check, `claim_terminal.rs:434-441` (examined). It now admits only `Model(ModelRoot)`, and every other kind refuses with `InvalidModelBinding`/`MalformedDeclaration` at `target_path(position, 0)`, which is `body/members/0/value/target`. This matches FR-476 step 4 exactly. The check still runs after the mode-fit checks and before the `parameters` and `condition` target checks, so the body-order target precedence is unchanged.
- The `object_type` arm of `self_object_key` is deleted (examined). With the model check above, the arm is unreachable, so deleting it is correct and removes a dead metered loop. The `model_root` arm, its metering and its `None` paths are unchanged. `terminal_self_population_scan_obeys_work_limit` still passes on the migrated fixture.
- No other path admits an `object_type` model (examined). `self_object_key` is now reached only for a `ModelRoot` model.
- New test `terminal_model_must_be_the_model_root_not_an_object_type` (examined). The root, without and with `self`, admits one row, and its writer output (asserted `encoded == canonical`) reads back with equal rows. Both the population object type (`1515`) and the uncovered `Other` object type refuse, with no `self`. An `object_type` target with a correctly typed `self` and rebuilt dependencies refuses too. Each refusal asserts the code, the cause and the `/body/members/0/value/target` locus. Two mutants are killed: the old reader fails it (coder mutant log, `expected an object_type model binding to refuse, got Admitted`). A reorder that runs the `self` check before the model check would refuse `ill_typed` at the parameter instead, and the self-path case's code assertion catches that.
- Test migration (examined). Ten fixtures move from the `1515` object type to the `test/families` `model_root` through `families()` and the `fixture/model-root` label. The `self` Reference target stays the object type (`object_key()`). `model_root_terminal_with_self_typed` collapses to `terminal_with_self`, because that helper now binds the root. All the mistyped-`self`, several-objects, no-population and work-limit refusals keep their exact assertions. One assertion was removed: the `first_model`/`second_model` block admitted a terminal on an `object_type`, which is now invalid. Removing it is correct, but the TC-061 control it served was not migrated (gap-analysis SR-2026112112042500000002 FND-001).
- Support helper `node_id("fixture/model-root")` (examined). It returns the exact FR-476 key `{package: test/families, path: [model, model_root], ordinal: 0}`.
- Rust idioms (examined): no new `unwrap` or panics in production, no new integer conversions, no unsafe code.

## Verdict

The implementation meets FR-476 step 4 and AC-4 for both the no-`self` and the `self` paths. The `model_root` positive and round-trip are covered. Refusal precedence is preserved and pinned by the tests. Reviewer focused run at the frozen head: `cargo test -j 4 -p filament-contract-ir --features conformance-cli --test it checked_package_v2_claim_terminal` rebuilt filament-contract-model from the committed bytes with no warnings, 22 passed, 0 failed (`/tmp/ir-qsl2718-refusal-review-run.log`). That run was needed because the coder's last model build in the target dir was the old-code mutant. Clippy was not run, per the brief's ban on broad sweeps. The only finding is FND-001, a low naming nit. The PR is mergeable once that finding and gap-analysis FND-001 are handled in this PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | After the migration, the `terminal_with_self` parameter `target_model` names the `self` Reference target object type, not the terminal's model (the model is now always the `model_root`). The local `other_model` passed to it in the mistyped-`self` case is also an object type. The names now say the opposite of what the values are. | tests/it/checked_package_v2_claim_terminal.rs:192, tests/it/checked_package_v2_claim_terminal.rs:1486-1487 |

## Dispositions

Round 1, at d13bcfcb (fix commit d13bcfcbbf681af884a9efb0f01ebdd9c3fdda39, verified against the committed diff). Coder focused run `/tmp/qsl2718-fix1-run.log`: 22 passed, 0 failed, no warnings. Its `it` binary (04:07:41) postdates the last test edit (04:07:32), and the production code is unchanged since the reviewer's own run at f4c8c2b5.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | `terminal_with_self(object_type_key: &str, ...)`. The mistyped-`self` local is now `other_object`, so both names say what the values are |
