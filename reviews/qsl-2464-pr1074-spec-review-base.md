---
id: SR-17940486709564847048085
title: "spec-review of FR-038-AC-562/563 and TC-486 status for QSL-2464"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1074 at cf3c7807 (base origin/main c7b62038); spec/checked_package/functional/FR-038-consume-checked-package-v2.md (AC-562, AC-563), spec/checked_package/matrix/tests.md (FR-038 and TC-486 rows)"
review_set: subset
---

## Summary

Ticket: QSL-2464. The PR changes no requirement text. It marks FR-038-AC-562 and AC-563 `Implemented (QSL-2464)` and moves the TC-486 matrix rows from Planned to Implemented. The ACs were authored on main by #1053.

Upstream authority: a private FSpec worktree at origin/main 3b5fc9e0. The relevant text is FR-402 "Body wrapper", "Slot parameters" and "Reader order" step 3, FR-402-AC-9 and AC-11, FR-370's `over` argument and its temporal-step `over` check, FR-322 "Hyper clause rows", FR-415-AC-24 and TC-2034.

`quire validate` on FR-038 and tests.md reports no errors. It does report non-canonical-shape grammar warnings on three ACs this PR did not touch (FR-038 lines 6709, 6717 and 6718). No open PR in agent-ix/filament-contract-ir adds FR-038-AC-562, FR-038-AC-563 or TC-486. Five open PRs were checked, #1065 (AC-571 to AC-573) among them.

## Examined scope

- FR-038-AC-562 (examined). It restates FR-402-AC-11's positive: the `Whole` wrapper's `over` is the vault `model_root` among the wrapper's `dependencies`, the package is admitted at the temporal and hyperproperty steps, and the rows are keyed by the hyperproperty node (FR-322 "Hyper clause rows").
- FR-038-AC-563 (examined). It restates FR-402-AC-11's three refusals, each `invalid_model_binding`/`malformed-declaration` at `body/arguments/0`. The NonInterference wrapper and the single-trace clause are refused at the temporal step, matching FR-370's `over` step. The root that no `subjects` binding names is refused at the hyperproperty step, matching FR-402 step 3. The `missing-name` carve-out for an undeclared `over` matches FR-370 ("a declared ... dependency, else `missing_declaration`/`missing-name`") and AC-103.
- FR-402 "Slot parameters" (context_only). It defines slot parameters as the `value`/`parameter` nodes through which a behaviours body reads trace state. A consumer "SHALL decode a slot's role and fields from the name by the same encoding". The node identity is name, level and type.
- FR-402 "Reader order" step 3 (examined). It lists three checks: a non-`temporal_clause` body target, then a non-empty wrapper fairness, then a root `over` that reads a slot or is not the first subject's root.
- The TC-486 matrix rows (examined). The status text agrees with the AC text.

## Verdict

The contract-ir ACs restate the FSpec rule faithfully, and the status change is accurate if the code-review and gap-analysis findings are resolved. There are two upstream (FSpec) gaps. Neither is a contract-ir spec defect, but both affect how the implementation must be judged.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FSpec gives no structural rule for "the body reads a slot parameter". FR-402 "Slot parameters" identifies slot parameters only by decoding the name ("a consumer SHALL decode a slot's role and fields from the name"). Nothing reserves the `slot`/`op`/`rcv`/`arg` encoding from other `value`/`parameter` names a body may read. FR-415-AC-24 also calls a timed guard's quantified binder `e` a "slot parameter", so the term is overloaded. A wrapper formula that reads a bound or capture parameter whose source name happens to decode as, for example, `op_sx` is classified differently by a name reader and by an intent reader. FR-038-AC-563 inherits the gap. Ask arch to rule on either a structural identification (for example, parameters reached from the hyper body that no binder introduces) or a reserved namespace | FSpec spec/objects/interfaces/FR-402-hyperproperty-obligation-identity.md:95-99,122-150; FSpec spec/functional/temporal/FR-415-declare-clocks-and-timed-models.md:543; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6747 |
| FND-002 | low | FR-402 "Reader order" step 3 gives the order of steps 1 to 4 ("the first failing check is its defect"), but not the order of the three checks inside step 3. The prose lists the non-empty-fairness refusal (`arguments/4`) before the model-root refusal (`arguments/0`). A wrapper with both defects has no explicit outcome, and FR-038-AC-563 does not say which check wins either. See code-review FND-001 for the implementation's choice | FSpec spec/objects/interfaces/FR-402-hyperproperty-obligation-identity.md:217-228; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6747 |


## Dispositions

Round 1, head 8bca5ece.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | Upstream FSpec gap, not a contract-ir defect. The PR now decodes slot names exactly by FR-402's encoding, which is the only identification the spec gives. A structural rule or a reserved namespace needs an FSpec arch ruling, and contract-ir follows it when it lands |
| FND-002 | deferred | Upstream FSpec wording. Contract-ir now follows the prose order (fairness at `arguments/4` before the root at `arguments/0`) and pins it with `tc_486_the_wrapper_fairness_refusal_precedes_the_model_root_refusal`. An explicit in-step order sentence belongs to FSpec FR-402 |
