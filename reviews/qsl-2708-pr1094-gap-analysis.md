---
id: SR-2026112011360400000002
title: "gap-analysis of QSL-2708/2707/2705 PR #1094 criteria, tests and the #ieee_operand claim"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1094 at d1668db7; FR-038-AC-583, FR-038-AC-584, FR-369-AC-26, FR-369-AC-27, FR-369-AC-28, FSpec FR-322-AC-490/491/492/493, FSpec FR-640 Reader rule 3; tests/conformance_fspec/type_argument_alias.rs, tests/conformance_fspec/timed_slots.rs, tests/it/checked_package_v2_source_objects/probabilistic.rs, tests/it/checked_package_v2_source_objects/probabilistic/join.rs"
review_set: subset
---

## Summary

This gap analysis is scoped to the criteria the PR's code and tests touch. Ticket: QSL-2708; the PR also carries QSL-2707 and QSL-2705. Plan completion: not assessed.

The FSpec checkout was filament-specification a90e26aa. The PR's claim for QSL-2705 was checked independently, in two ways. First, by reading the reader's parameter binder check against FSpec FR-322 line 408 ("the schema admits it only as the `name` literal of a `value`/`parameter` node's body"). Second, by running FSL main c39c4c75's ignored `the_checker_ieee_operand_binder_never_prints` with filament-contract-model patched to this PR's head, in a private FSL worktree.

The QSL-2705 claim holds: the strict reader is not the wrong side. Four results support this:

1. Reading the code. `is_parameter_name` (structural.rs:1695) admits `#ieee_operand` for every `value`/`parameter` node, whatever clause encloses it. This matches FR-322 line 408. No other parameter-binder site applies `is_identifier`.
2. The ignored FSL test, un-ignored locally. It fails at `compile`, before any package exists. The cause is the FSL checker's `CheckRefusal { origin: TypeDeclaration "temporal 0 holds 1", cause: IeeeProfileNotAdmitted }` at the float literal (region 359..392). The contract-ir reader is never reached.
3. FSL-emitted packages, read directly. The same temporal clause was compiled with a float type declared in the unit, for both the one-operand form `always [0, 3] holds(-A < Z)` and the two-operand form `holds(-A < -Z)`. The emitted packages have the `#ieee_operand` parameter at node 12 (the node the ticket names) and the `let` nodes in the temporal clause body. `CheckedPackageV2::read` (bounded limits, FSL's evidence) admits both, at this PR's head and at FSL's current pin 4c91eff6. The ticket's `/semantic_graph/nodes/12/body` refusal does not reproduce on either.
4. The package route, run with those packages. It then stops at FSL's "tl_lowering over a selected checked package is not implemented yet".

The open work is therefore FSL's: the checker's IEEE-profile admission for a float literal in a temporal clause, and package-route `tl_lowering`. QSL-2628 cannot close on contract-ir. FND-002 and FND-003 below concern only how the PR pins the claim.

## Examined scope

- FR-038-AC-583 / FSpec FR-322-AC-492. `every_published_type_argument_alias_vector_reads_as_published` reads all nine TA vectors. It checks the schema verdict, the code, the cause and the first locus, plus both loci for TA-09. `an_empty_alias_text_is_malformed_wire_at_that_member` covers the empty-text half of rule 1. `a_model_alias_resolving_to_another_type_than_the_declaration_is_a_type_mismatch` covers the rule 4 mismatch, using an `Integer` target in place of the AC's `Int[0, 4]` against `Int[0, 9]`. The trace tags are correct.
- FR-038-AC-584 / FSpec FR-322-AC-493. `an_alias_label_changes_no_admission_decision` admits the base package, TA-02's package and TA-02 with its keys exchanged.
- FR-369-AC-26 and FR-369-AC-27. The diamond test and the changed buffer-charge test were reviewed. Their defects are in the code-review SR (FND-005, FND-006), and the identity-row join coverage defect that QSL-2707 leaves open is that SR's FND-001.
- QSL-2705: `the_reserved_ieee_operand_name_is_admitted_on_every_parameter_node_a_temporal_clause_reads`.

## Verdict

Changes required. The TA vectors are covered as published. Three gaps remain:

- The FR-322-AC-493 coverage is partial.
- The QSL-2705 regression test does not build the case the ticket reports, although the claim it stands for was confirmed.
- The QSL-2705 regression test is traced to body_locations criteria it does not verify.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FSpec FR-322-AC-493 is only partly covered. The test checks admission of three packages. It does not check the same value and the same charges per application, or that removing `alias` or replacing a key leaves the refusal of an otherwise defective variant unchanged (operator-ineligible `result_type`, `bound_is_member` mismatch). A rule 2-4 check that ran before a constraint and changed refusal precedence, or that charged differently, would pass | tests/conformance_fspec/type_argument_alias.rs:98-121 |
| FND-002 | medium | The QSL-2705 regression test does not build the case the ticket reports. It renames the `name` of each existing `parameter` node of the published timed-slot package, all of them clause binders. It never puts in a temporal clause body the checker's `#ieee_operand` `let` (a `filament.op.control.let` whose binder parameter is introduced for an IEEE ordering's operands) over float operands. That is the reported shape, refused at `/semantic_graph/nodes/12/body`, and the ticket's done-when asks for "a fixture whose temporal clause body contains it" | tests/conformance_fspec/timed_slots.rs:637-693 |
| FND-003 | medium | Trace mismatch. The QSL-2705 test is tagged `#[trace("TC-048", "FR-322-AC-490", "FR-038-AC-581")]`, but both criteria are the `checked_term_data.body_locations` vectors (BL-01..BL-R6). The test reads no body_locations row. What it verifies is the reserved binder-name admission of FR-322 "Parameter" (line 408), which no IR criterion states | tests/conformance_fspec/timed_slots.rs:643 |

## Dispositions

Round 1, at head 9dda97186 (fix commit 9dda97186 over d1668db7). Focused runs in the private review worktree with a private target dir, via cargo-slot, one job at a time. Passing: conformance_fspec `type_argument_alias` (9) and `timed_slots` (8); `it` `checked_package_v2_source_objects::probabilistic` (36) and `checked_package_v2_owners` (34); lib `checked_package::v2::operations` (104), `checked_package::v2::probabilistic` (10) and `checked_package::shared` (21). The slow 300-row `checked_package_v2_probabilistic` filter was stopped unfinished, so its named-buffer test was not rerun here. The review-pass mutants M1 to M4 are now killed. M1 (key segment grammar), M2 (dependency owner join) and M3 (rule 4 always refuses) are each killed by one conformance test. M4 (rule 5 latest conflict) is killed by the lib test `the_earliest_alias_conflict_is_reported`. One new mutant survives: M5 drops `&& resolved.is_some()`. It is equivalent, because a `type_argument` member with no `declaration` is refused earlier by `member_is_well_formed_node`.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `an_alias_label_changes_no_refusal_and_no_declaration` refuses an operator-ineligible `result_type` variant and a `declaration` mismatch variant with the same code, cause and locus with no alias, one alias and another alias. It also asserts the same typed `declaration` for every application. FR-038-AC-584 states why that gives the same value and the same charges |
| FND-002 | fixed | `the_ieee_operand_let_of_a_temporal_clause_body_is_admitted` reads the committed FSL-emitted `tests/fixtures/qsl-2705-ieee-operand/temporal-holds-let.json` (26 nodes): node 12 is the `#ieee_operand` parameter, node 20 the `filament.op.control.let`, and node 21 `holds` reads the let. The package is admitted, and a `#other` rename is refused. This matches the package this review emitted independently from FSL |
| FND-003 | fixed | The test is now traced `#[trace("TC-048", "FR-038-AC-585")]`. FR-038-AC-585 states the reserved-binder admission, and TC-048 verifies it |
