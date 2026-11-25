---
id: SR-9663
title: "code-review and rust-review of the TC-048 model-member id test update"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1153 at b9d6ba98 (base origin/main b81baf14); crates/filament-contract-model/src/checked_package/v2/model_members/tests.rs"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in, for filament-contract-ir PR #1153. Ticket: IR-1614. The diff is one test file. It changes two tc_048 tests in `model_members/tests.rs` so they no longer expect a hyphenated object id to be refused. Reference: filament-specification origin/main, FR-154-AC-8.

Command run: `cargo test --locked -p filament-contract-model --lib -j 4 -- tc_048`, exit 0, 122 passed, 0 failed, 622 filtered out, on head b9d6ba98bd5df92b9ce5b4f2101a18b1bf7b0cba.

Gap analysis does not apply: the diff has no production code. Spec review does not apply: no spec or plan file changed.

## Examined scope

- FR-154-AC-8 (examined): "A type definition or population node whose artifact id begins with an ASCII letter and continues with ASCII letters, digits, underscores or hyphens is admitted ...; this includes `sys_pump`, `sys-pump` and `FR-001`. An artifact id that does not match this rule, such as `-sys_pump`, refuses `invalid_model_binding`/`malformed-declaration` ..., a reference to that node reports no `missing_declaration`/`missing-name` ..."
- `tc_048_semantic_ir_declaration_defects_refuse_with_their_fr_154_cause` (examined). `-sys_pump` is the spec's own example. `is_artifact_id` (model_members.rs:3788) refuses it because the first byte is not an ASCII letter, and `classify` (model_members.rs:4723) turns that into `ModelRefusal::malformed()`. The test passes for that reason. No other check in the path gives this code and cause for this node. The assertion still names the exact code and cause.
- `tc_048_a_reference_to_a_refused_node_leaves_that_nodes_own_refusal` (examined). `sys.pump` fails `is_artifact_id` only because of the `.`. `valid_semantic_identity` admits `.`, so the node is refused for its id and for nothing else. `s` (0x73) sorts after `G` and `W`, so the comment that each reference is read before the node it names is still true. The forward-reference case is still exercised. All three referencing shapes and the duplicate-identity case are unchanged.
- Hyphen admission (examined). The changed tests no longer assert that a hyphen is admitted. origin/main already asserts it in `intake_retention.rs` `selected_artifact_ids_admit_hyphens_and_reject_other_punctuation` (`Widget-type`, `widgets-second`). That test is outside the tc_048 filter, and CI runs it.
- Weakened assertions (examined). None. Each `assert_eq!` keeps its expected value. The two tests failed on origin/main because they expected `sys-pump` and `bad-id` to be refused, and FR-154-AC-8 now admits both.

## Verdict

Approve. Both changed tests now match FR-154-AC-8, and they pass for the stated reason. The tc_048 filter is fully green. One low wording nit is in the doc comment.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The new doc comment says "object id" where FR-154-AC-8 and `is_artifact_id` say "artifact id", and it says "is refused" twice in one clause | crates/filament-contract-model/src/checked_package/v2/model_members/tests.rs:1313-1314 |
