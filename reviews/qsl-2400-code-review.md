---
id: SR-2026112109235101
title: "PR 1111 strict hyper wrapper reader code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR 1111; QSL-2400; four changed model reader files and three changed test/support files"
review_set: subset
relationships:
  - {target: "ix://agent-ix/filament-contract-ir/FR-038", type: references}
  - {target: "ix://agent-ix/filament-contract-ir/FR-019", type: references}
---

## Summary

Independent review of frozen head 4f67f07a681a6efd99fb2c03a40322437cd000d4 against base 0e089f6ea5dea8d799cb0234ec9623e0073ffad4. Code-review includes the Rust lane, wire strictness, refusal order, scope identity, resource use, API compatibility and test intent. Ticket QSL-2400 is selected from the PR title and verified GitHub attachment; QSL-2631 is related emitter context, and QSL-2691/QSL-2338 are not closed by this review.

## Verdict

FAIL. Two paths admit wrappers that violate their owning clause's scope. Findings are established by control-flow inspection; no reproducer was executed because the coordinator owns focused execution.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | A non-slot parameter bypasses the requirement that over reference the first scoped slot. | crates/filament-contract-model/src/checked_package/v2/temporal.rs:3115 |
| FND-002 | high | A shared wrapper's duration bounds are checked only in the first referencing hyper clause's source and clock context. | crates/filament-contract-model/src/checked_package/v2/temporal.rs:729 |

## Finding details

FND-001: Start with the admitted two-slot wrapper fixture, leave both slots reachable by its formula, and retarget over to an ordinary value/parameter dependency. The temporal over check accepts that kind. The earlier hyper check only handles model roots. Here over_slot is false, so the first-slot comparison is skipped and the invalid wrapper is admitted. Compare every over target with scope[0] when the scope is nonempty; preserve the existing first-subject model-root rule for an empty scope. Add a public-reader negative at the wrapper's body/arguments/0 with invalid_model_binding/malformed-declaration.

FND-002: Two distinct hyperproperty nodes may share one wrapper while their claim_names rows name different locked source units. Let both source units declare valid linear Time units named ms, both clauses' trace clocks name ms, and the wrapper duration bounds be typed by the first unit's ms node. Both per-hyper timestamp-unit checks pass, but iter().find chooses only the first wrapper context for check_clause. The second clause's foreign bounds are never checked. Different timestamp clocks can similarly be hidden by the first context. Validate the wrapper's duration bounds under every referencing clause's context, without changing the normative model-owned clock-unit support. Test both hyper-node orders and exact unit-mismatch refusal.

## Scope

- FR-038-AC-598, examined: "A body wrapper's atom slots are the `slot` parameters its formula reaches, one node per name, in ascending byte order of `name`, at levels 0 to `n - 1`; its `over` is the first."
- FR-038-AC-599, examined: "The clause's `temporal_bindings` rows carry none, and its `timestamp_unit` and its wrapper's duration bounds resolve only in that unit, after these checks, never inferred."
- FR-019, examined: "`ClaimNameRow` gains the optional field source_unit (FR-322 Hyper clause source unit), a breaking change that rides the next contract-ir tag (FR-371 Compatibility)."
- FSpec FR-402-AC-14/15, FR-322-AC-508/511, FR-778-AC-13, context_only: authoritative upstream revision f73bd9f3fe5091ab8b459126808d7149cdb02b08 was read from the local FSpec git object.

## Coverage

All nine changed files were read. ClaimNameRow's closed serde schema, optional canonical member emission, locked-source equality, required/forbidden source presence, same-node consistency, and claim_names-before-temporal ordering were checked. Wrapper dependency reach, byte-name ordering, duplicate-name ordering before wrong levels, first-slot ordering and seq leaf base were checked. No unsafe code, vendoring, duplicated model vocabulary, new dependency, or public API beyond the documented breaking field was introduced. Scope/clock scans remain bounded by input sizes; shared-context selection is the correctness issue above.

No applicable AssuranceProfile exists under spec/. Existing evidence logs report 8 focused tests passing, both focused Clippy invocations passing and formatter passing; they were inspected, not rerun. Initial no-feature compile failure is a resolved invocation error, not a baseline source finding. No Cargo, broad gates, Python, helper runner, source edit, worktree, push or merge was performed. Legacy timed/profile behavior and model-owned unit identity remain in scope for preservation. Integration after preceding PRs remains with the coordinator.
