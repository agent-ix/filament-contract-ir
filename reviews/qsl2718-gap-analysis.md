---
id: SR-2026112105511900000002
title: "gap-analysis of QSL-2718 model_root terminal self binding"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1109 at de9833fe; FR-060-AC-1, FR-060-AC-2, FR-060-AC-8, crates/filament-contract-model/src/checked_package/v2/claim_terminal.rs, tests/it/checked_package_v2_claim_terminal.rs"
review_set: subset
---

## Summary

Gap analysis scoped to the PR diff. Ticket: QSL-2718. Plan completion: not assessed. The analysis checks each branch of the new `model_root` arm against a test that fails when the branch is removed or changed, and checks the ticket's expected completion against FSpec FR-476 at main 6ad873d5. The full matrix was not run, because the brief allows a focused run only.

## Examined scope

- FR-060-AC-2 (examined): "FSpec TC-961 MT-07 through MT-11 and MT-13 through MT-15 refuse with FR-476's exact code, cause and pointer after identities are rebuilt: ... wrong model, wrong or mistyped `self` ...". The wrong or mistyped `self` on a `model_root` target is covered by the new tests. The `object_type` half of MT-10 ("wrong model") has no test, and the code admits it (FND-001).
- FR-060-AC-8 (examined): "Over FSpec TC-961 MT-18, one `model_root` node referenced by MT-01's `when` node and MT-02's `any` node: the reader admits the package ...". Before this PR, a `model_root`-bound MT-01 (which reads `self`) could not be admitted. The new positive test admits a single `model_root`-bound MT-01. No test combines it with a second node on the same root (FND-002).
- FR-060-AC-1 (context_only): the writer round trip of the admitted shape. Existing coverage applies to the `object_type` fixture only.
- Branches of the `model_root` arm (examined): a missing package owner, no pair, several pairs (in one population and across populations), a non-object member, the declaration-id lookup. All but "one object in two populations" (code-review FND-002) and "non-object member" (an unreachable or defensive path, not a defect) have a test that fails.

## Verdict

The `model_root` admission is covered and its tests are meaningful. Two gaps against the ticket's acceptance remain: the `object_type` refusal (FND-001, shared with code-review FND-001) and MT-18 with a `self`-reading node on a shared `model_root` (FND-002).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-060-AC-2 claims MT-10 is covered, but the `object_type` `model` target is admitted. No test asserts its `invalid_model_binding`/`malformed-declaration` refusal, and QSL-2718's expected completion requires it. | crates/filament-contract-model/src/checked_package/v2/claim_terminal.rs:470, tests/it/checked_package_v2_claim_terminal.rs:976-989 |
| FND-002 | medium | No test realizes FR-060-AC-8's MT-18 as FR-476-AC-6 states it: a `self`-reading `when` node and an `any` node on one `model_root`, ordinals 0 and 1. This is the case QSL-2718 unblocks. | tests/it/checked_package_v2_claim_terminal.rs:1506-1531 |

## Dispositions

Round 1, at b201ff1d. Focused run: 21 passed, 0 failed.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | deferred | arch ruling: #1109 admits both ModelRoot and ObjectType; the object_type refusal is follow-up PR scope remaining on QSL-2718, after the FSL emitter emits model_root |
| FND-002 | fixed | New test `model_root_terminals_when_self_and_any_share_one_root` admits a `self`-reading `when` node and an `any` node on one `model_root`. It asserts two rows on that root, ordinals [0, 1], and one When{parameter: Some} row and one Any row. Key-order assignment is already covered by the existing FR-060-AC-8 tests |
