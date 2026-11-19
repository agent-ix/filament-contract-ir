---
id: SR-17940486709564847048089
title: "code-review and rust-review of QSL-2560 terminal literal condition and several terminals per model"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1076 at 8e69e047 (merge base c7b62038, origin/main 358e9e2d); crates/filament-contract-model/src/checked_package/{shared.rs,v2/claim_terminal.rs,v2/model_members/tests/budgeted_charge.rs}, crates/filament-contract-model/src/lib.rs, tests/it/checked_package_v2_claim_terminal.rs, tests/it/checked_package_v2_budgeted_read.rs, tests/it/checked_package_v2_model_members.rs, tests/read_storage_alloc.rs"
review_set: subset
---

## Summary

Code review, with the rust-review lane folded in, for filament-contract-ir PR #1076. Ticket: QSL-2560. Requirement source: FSpec origin/main (FR-476 "Several terminal nodes on one model node", "Reader order" rules 4 and 5, FR-476-AC-5/6) and contract-ir FR-060 (AC-6/7/8), FR-062 (withdrawn), FR-019.

Gates at 8e69e047, in the scratch worktree CIR-1076-review with its own target, FILAMENT_SPECIFICATION_DIR = FSpec origin/main:

- `cargo fmt --all -- --check`: clean.
- `make string-edge`: clean.
- `cargo clippy --workspace --all-targets --features conformance-cli -- -D warnings`: clean.
- Full workspace `cargo test --workspace --features conformance-cli --no-fail-fast`: 2418 passed, 18 failed. The 18 are exactly the known `monitor_plan::reached_model_read` lib reds.
- Focused: `--test it -- claim_terminal public_interface whole_read_accounting budgeted_read`: 64 passed, 0 failed. `--test read_storage_alloc`: 32 passed.
- Clean origin/main in its own worktree and target: the same 18 `reached_model_read` reds, plus the public_interface reds caused by the `CheckedTerminalDuplicate` drift. Those are green on the PR.

Hand mutants on `claim_terminal.rs`, each run against the `checked_package_v2_claim_terminal` tests. All 10 were killed:

1. Drop the model key from the sort.
2. Invert the ordinal reset (`==` to `!=`).
3. Ordinal step 2.
4. Remove literal admission.
5. Reverse the terminal-key tiebreak.
6. `at > 0` to `at > 1`.
7. Admit any `value` form as a condition (own mutant).
8. Sort models descending (own mutant).
9. Drop the terminal-node tiebreak (own mutant).
10. Start the ordinal at 1 (own mutant).

## Examined scope

- `validate_terminal` (examined). The condition admits `Expression(_) | Value(Literal)`. A `value`/`parameter` condition still refuses `ill_typed`/`operator-ineligible` at `body/members/3/value/target`. A non-Boolean literal refuses `ill_typed`/`non-boolean-root` at the target. A `self` parameter with a literal condition refuses `invalid_semantic_graph` at `body/members/2`, through the existing reads-equal-parameters check. Both duplicate refusals are deleted, and so is the metered `terminals` buffer. No refusal is left in the rows step, which matches FR-476 rule 5.
- `retain_rows` (examined). The sort key is (model key, terminal key). The ordinal is zero-based, resets at each model boundary and is assigned after sorting, so it does not depend on graph order. `u32` overflow is unreachable: the row count is bounded by the node limit, and `checked_add` with `expect` documents this. `TerminalRow` grows by 4 bytes, and that growth is charged through the existing `with_capacity` and `new_arc` at the `terminal_rows` site.
- Removal of `CheckedTerminalDuplicate` and `CheckedPackageRefusal::terminal_duplicate` (examined). The type, the field, the constructor default, the crate-root re-export and every test assertion are gone. A repo-wide grep finds no remaining code use. No dead helper remains.
- The modified AC-6 test (examined). N's model package was renamed from `aa/` to `zz/`. This is legitimate: FR-060-AC-6 now requires the test to establish that M's model key sorts before N's, and the test asserts that. The former `aa/` configuration became the new model-first ordering test, so coverage was strengthened, not weakened.

## Verdict

The change is correct and strongly tested. Every mutant was killed, the gates are clean, and the failing set is equal to main's known reds minus the fixed public_interface drift. Two low findings are below. Neither blocks merge.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | `tc_048_a_selected_fcd_relationship_navigates_through_its_named_end` still carries `#[trace(..., "FR-062-AC-3")]`. FR-062-AC-3 is now Withdrawn, and this PR removed the `terminal_duplicate == None` assertion that backed the tag, so `quire matrix` reports a withdrawn AC as `tagged`. Drop the tag | tests/it/checked_package_v2_model_members.rs:1963 |
| FND-002 | low | `retain_rows` sorts with the plain `CheckedNodeId::cmp`, which is uncharged and spills frames on two distinct nested operand chains. It now also compares model keys and tests `==` per row. The terminal-key comparison predates this PR, and model_root keys are flat, so the added exposure is nil in practice. But the read path's own contract (shared.rs `Ord` doc) says read-path comparisons are charged or rank-based. Consider sorting by graph rank or a prepaid `CheckedNodeIdOrder` | crates/filament-contract-model/src/checked_package/v2/claim_terminal.rs:629-643 |

## Dispositions

Round 1, at head 33d9ccab. That head is the fix commit plus a clean merge of origin/main 358e9e2d, and the merge-vs-main diff equals the original PR delta. Gates at the head:

- fmt, string-edge and clippy `-D warnings` are clean.
- `--test it` filtered to claim_terminal, public_interface, whole_read_accounting and budgeted_read: 65 passed, 0 failed.
- The `filament-contract-model` lib tests: 649 passed, 18 failed. The 18 are the known `reached_model_read` reds.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The `FR-062-AC-3` tag is removed from `tc_048_a_selected_fcd_relationship_navigates_through_its_named_end`, and `quire matrix` now shows FR-062-AC-3 as untagged |
| FND-002 | accepted-no-change | The plain uncharged comparison predates this PR. Model_root keys are flat, the sort is in place and allocates nothing, and the row capacity is already charged. The coder recorded the reason in the PR body. No added read-path exposure |
