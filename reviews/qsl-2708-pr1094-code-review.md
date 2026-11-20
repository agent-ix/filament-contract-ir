---
id: SR-2026112011360400000001
title: "code-review (rust-review lane) of QSL-2708/2707/2705 PR #1094 type_argument alias, pair-keyed workload rows, #ieee_operand pin"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR #1094 at d1668db7, git diff origin/main...HEAD; crates/filament-contract-model/src/checked_package/v2/{flat_wire.rs,model_members.rs,operations.rs,probabilistic_join.rs,source_join.rs (context),temporal.rs,typed_terms.rs}, crates/filament-contract-model/src/lib.rs; tests/conformance_fspec/{main.rs,timed_slots.rs,type_argument_alias.rs}; tests/it/checked_package_v2_node_map.rs, tests/it/checked_package_v2_source_objects/{probabilistic.rs,probabilistic/join.rs}"
review_set: subset
---

## Summary

This is the code-review of PR #1094, with the rust-review lane folded into this file. Ticket: QSL-2708; the PR also carries QSL-2707 and QSL-2705. It was checked against FSpec FR-322 "Named type arguments" and FR-640 Reader rule 3 at filament-specification a90e26aa. All runs used a private review worktree and a private target dir, with cargo-slot, one job at a time:

- lib, name-filtered: checked_package::v2::{typed_terms, operations, flat_wire, probabilistic, temporal, model_members, structural}. 216 pass.
- tests/it (`--features conformance-cli`), name-filtered: checked_package_v2_source_objects::probabilistic, node_map, operation, typed, ieee_admission, catalogue_differential, owners, operand_families, typed_terms, model_members::reaches_population, recursive_leaves, admitted_guard_context, admitted_result_context. 219 pass.
- conformance_fspec, full: 482 pass and 16 fail. That is the known baseline count on main, and none of the failures is in a touched module. type_argument_alias (4), timed_slots (8), carried_quantity (19) and clock_payload_tuple (11) all pass.
- A scratch `it` test (local only, reverted) reproduced FND-001.
- Four local mutants of operations.rs, each reverted after its run. All four survive the PR's tests (FND-004).

## Examined scope

- Rule 1 (shape). The check in flat_wire.rs runs in the wire-validation stage. It refuses malformed_wire at `.../member/alias` for a non-object, an inexact member set or a non-type_argument member, and at `.../alias/{package,node}` for an empty or non-string text. temporal.rs admits the extra `alias` key on the closed type_argument shape. This agrees with FR-322 rule 1, TA-03, TA-04 and the empty-text test.
- Rule 2 (owner join). A package naming no selected model, no lock.sources entry and no dependency_selections entry refuses missing_declaration/missing-selection at `.../alias/package` (TA-05).
- Rule 3 (key spelling). is_declaration_key_of applies to source and dependency packages only and refuses invalid_package/invalid-value at `.../alias/node` (TA-06, TA-07).
- Rule 4 (model resolution). It runs via alias_resolved_node. A name that is no alias refuses missing-name (TA-08). A resolved node that differs from `declaration` refuses ill_typed/type-mismatch at `.../alias`.
- Rule 5 (one key, one type). one_alias_one_type runs once, after first_refusal. Its group-sort and earliest-conflict selection agree with "a later member whose alias equals an earlier member's while its declaration differs". It reports the later alias pointer and then the earlier one (TA-09).
- typed_terms: both decode paths carry the alias. The raw member JSON is retained, so re-encoding preserves it, and every `TypeArgument { declaration, .. }` pattern was updated. The PR's breaking API is a new pub field on a variant of an exhaustive pub enum, plus the new pub CheckedTypeArgumentAlias, and it is listed in FR-019. FSL main does not destructure the contract-ir `CheckedOperationMember::TypeArgument` (grep), so FSL is not broken at compile.
- probabilistic_join: the selected-model workload now keys coverage by (node, name), and source_workload drops its name buffer. Strict ascent of (declaring, name) already refuses a repeated pair.
- QSL-2705: the PR makes no reader change. is_parameter_name already admits `#ieee_operand` (#1088), and no other `is_identifier` site checks a parameter binder name. The temporal-path findings are in the gap-analysis SR.

## Verdict

Changes required. Rules 1, 2, 3 and 5 match FR-322 and the published vectors. Rule 4 and the QSL-2707 half need work:

- The identity-row join still checks workload coverage by operation name alone. Now that the strict read admits same-named rows, a workload that omits an inherited operation is admitted. This was reproduced (FND-001).
- Rule 4 silently admits an alias whose resolution it cannot map to a node.
- Four hand mutants survive the new tests.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The source identity-row join's workload coverage is still keyed by operation name alone: `named: Vec<&str>` and `for &(_, name) in &declared { named_has(&named, name) }`. Take a lineage where Other extends O and both declare `step`. A workload on Other weighing Other::poke and Other::step, and omitting the inherited O::step, is admitted. A scratch test reproduced this: `join(..)` returned `Ok(())`. QSL-2707 lets same-named rows past the strict read, which makes this omission reachable. Coverage must be keyed by (declaring position, name), as the selected-model path in probabilistic_join.rs now is | crates/filament-contract-model/src/checked_package/v2/source_join.rs:644-686 |
| FND-002 | medium | Rule 4 refuses only when both sides are known. `if let (Some(resolved), Some(declaration)) = (resolved, member_declaration(operation))` admits silently when alias_resolved_node returns Some(None), meaning the chain resolves to a type this reader cannot map to a node. FR-322 rule 4 refuses whenever the resolved type's step-4 node is not the node `declaration` names, and an unmappable resolution is not that node | crates/filament-contract-model/src/checked_package/v2/operations.rs:1852-1869; crates/filament-contract-model/src/checked_package/v2/model_members.rs:1740-1755 |
| FND-003 | medium | The rule 4 type-mismatch refusal carries no `expected` (the resolved type's node id) or `actual` (the `declaration`), which FR-322 rule 4 requires. `application.refuse` records only the code, the cause and the `.../alias` locus, and no test checks either id | crates/filament-contract-model/src/checked_package/v2/operations.rs:1860-1867 |
| FND-004 | medium | Mutant adequacy. The coder did not rerun mutation after adding the last two tests, and four hand mutants of the new code all survive the 4 type_argument_alias tests: (M1) `is_declaration_key_of` reduced to the prefix check, dropping the identifier-segment grammar; (M2) the `dependency_selections` owner join disabled; (M3) rule 4 refusing every resolved model alias (`if true`), which survives because no test admits a correct model_root alias; (M4) rule 5 choosing the latest conflict instead of the earliest across groups. Each needs a case: a key with a non-identifier segment, an alias of a selected dependency, a model_root alias that resolves to its declaration and is admitted, and two conflicting groups | crates/filament-contract-model/src/checked_package/v2/operations.rs:540, 1799-1804, 1827-1834, 1860-1867; tests/conformance_fspec/type_argument_alias.rs |
| FND-005 | medium | The QSL-2707 test does not test the ticket's scenario. `a_workload_row_is_keyed_by_its_declaring_node_and_operation_name` says "two inherited operations of one name, declared by two supertypes (a diamond)", but it uses rows O::step and Other::step on model Other. Other declares no `step` and no supertype is set, and the test runs only the strict read (`admitted`), never the identity-row join. So it neither builds a diamond nor reaches the join where FND-001 lives | tests/it/checked_package_v2_source_objects/probabilistic/join.rs:575-615 |
| FND-006 | low | `the_source_buffers_are_charged_before_they_are_made` now asserts `count(16 x 13) == 0`. With the buffer removed, nothing can request that amount, so the assertion cannot fail and pins nothing. The selected-model workload buffer, whose element type grew to `(&str, &str)` (32 bytes per row), has no charge pin at all | tests/it/checked_package_v2_source_objects/probabilistic.rs:1097-1119; crates/filament-contract-model/src/checked_package/v2/probabilistic_join.rs:928-932 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | high | The pair-keyed identity-row coverage now also requires a weight for an ancestor operation that the model redefines. `a_diamond_workload_must_weigh_every_declaring_nodes_operation_of_one_name` asserts this: where Other extends O and both declare `step`, the workload {Other::poke, Other::step} is refused at `.../members`, and only {O::step, Other::poke, Other::step} is admitted. FSL FR-185 "Workloads" says the effective view counts "an ancestor's operation that T or a nearer ancestor redefines" once, and counts an operation that several unrelated nearest ancestors declare once per declaration. So the redefined O::step is not a separate operation, and the reader now refuses the valid workload an FSL S3 producer emits for a redefinition. The selected-model path in this same PR keeps one-operation-per-name for the same reason (FR-369-AC-8). Coverage should require a pair only for the nearest declarations of each name. That keeps the diamond rows (P::step and Q::step) and drops redefined ancestors. This chain case was also the example in the review pass's FND-001, so the reviewer's example was wrong and the fix followed it | crates/filament-contract-model/src/checked_package/v2/source_join.rs:631-689; tests/it/checked_package_v2_source_objects/probabilistic/join.rs:680-700 |
| FND-008 | low | The doc comment of `the_source_buffers_are_charged_before_they_are_made` says the selected-model workload buffer is "32 bytes a row". The buffer is back to `Vec<&str>`, and the pinning test `a_wide_model_owner_workload_is_admitted_and_its_named_buffer_charged` charges `size_of::<&str>()`, 16 bytes a row | tests/it/checked_package_v2_source_objects/probabilistic.rs:1103-1104 |

## Dispositions

Round 1, at head 9dda97186 (fix commit 9dda97186 over d1668db7). Focused runs in the private review worktree with a private target dir, via cargo-slot, one job at a time. Passing: conformance_fspec `type_argument_alias` (9) and `timed_slots` (8); `it` `checked_package_v2_source_objects::probabilistic` (36) and `checked_package_v2_owners` (34); lib `checked_package::v2::operations` (104), `checked_package::v2::probabilistic` (10) and `checked_package::shared` (21). The slow 300-row `checked_package_v2_probabilistic` filter was stopped unfinished, so its named-buffer test was not rerun here. The review-pass mutants M1 to M4 are now killed. M1 (key segment grammar), M2 (dependency owner join) and M3 (rule 4 always refuses) are each killed by one conformance test. M4 (rule 5 latest conflict) is killed by the lib test `the_earliest_alias_conflict_is_reported`. One new mutant survives: M5 drops `&& resolved.is_some()`. It is equivalent, because a `type_argument` member with no `declaration` is refused earlier by `member_is_well_formed_node`.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `named` is now `Vec<(usize, &str)>`, and `named_has(&named, declaring, name, ..)` matches the position as well as the name, so a diamond workload that omits P::step or Q::step is refused. The redefinition case the review used as its example is now wrongly refused; that is new finding FND-007 |
| FND-002 | fixed | `if resolved.as_ref() == declaration && resolved.is_some() { return Ok(None); }`: any other resolution, unmappable or absent included, refuses ill_typed/type-mismatch at `.../alias`. `a_model_alias_the_reader_cannot_map_to_a_node_is_a_type_mismatch` pins it |
| FND-003 | fixed | New `CheckedTypeArgumentRefusalDetail { expected, actual }` on `CheckedPackageRefusal.type_argument_detail`, set at the rule 4 refusal and asserted by the mismatch tests |
| FND-004 | fixed | Four new conformance tests and a lib unit test of `first_alias_conflict` kill M1 to M4 (rerun here). The only new survivor, M5, is equivalent |
| FND-005 | fixed | `a_diamond_workload_must_weigh_every_declaring_nodes_operation_of_one_name` builds D over P and Q, each declaring `step`, and runs `join()`: both rows are admitted, and either one omitted is refused at `.../members` |
| FND-006 | fixed | The vacuous `count(named) == 0` assertion is removed. The selected-model workload keeps `Vec<&str>` by design (FR-369-AC-8), and its charge is pinned by the existing `a_wide_model_owner_workload_is_admitted_and_its_named_buffer_charged` (`size_of::<&str>() * 300`). The stale doc comment is FND-008 |

Round 2, at head 729efb3be (fix commit 729efb3be over 9dda9718). Focused run: `it` `checked_package_v2_source_objects::probabilistic`, 36 passed, which includes the join suite with its updated charge expectations. A local mutant that reverses the `reaches` direction in the effective-operation check fails `probabilistic::join` (20 passed, 1 failed), so it is killed. On the shadowed-row question: refusing a row that weighs the redeclared `O::step` at that row's operation name is right. Under FSL FR-185 that declaration is not an operation of `Other`'s effective view, so the row names an operation the model lacks. FSpec FR-640 rules 1 and 3 refuse such a misnamed operation `invalid-value` at that member, and an FSL producer never emits the row. Admitting it would give one operation two weights, which FR-185 refuses as `duplicate-weight`.

| FND | outcome | reason |
| --- | --- | --- |
| FND-007 | fixed | `workload` builds `effective`, which drops each declaration whose name a descendant in the lineage redeclares (`reaches(descendant, ancestor)`), and uses it for both the row membership and the coverage. The chain case now admits {Other::poke, Other::step}, and refuses omitting Other::step and weighing the shadowed O::step. The diamond and the nested diamond-over-redeclaration keep P::step and Q::step |
| FND-008 | fixed | The doc comment now reads "16 bytes a row (`&str`)" |
