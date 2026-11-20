---
id: SR-2026112011360400000003
title: "spec-review/base of QSL-2708/2707 PR #1094 IR spec edits"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1094 at d1668db7; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (FR-038-AC-583, FR-038-AC-584), spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md (workload rule, FR-369-AC-28 context), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/model/functional/FR-019-rust-library-interface.md; checked against filament-specification a90e26aa FR-322 'Named type arguments' (FR-322-AC-491..493) and FR-640 Reader rule 3"
review_set: subset
---

## Summary

This spec review covers the IR spec edits in PR #1094. Ticket: QSL-2708; the PR also carries QSL-2707. The edits were checked against FSpec a90e26aa. `quire validate --scope .` was run on the review artifacts. The PR's spec edits are four rows and one prose amendment:

- FR-038-AC-583 and FR-038-AC-584 (new).
- TC-048 relationships for the two new rows.
- An FR-019 inventory entry for `CheckedTypeArgumentAlias`.
- The FR-369 workload-rule amendment: rows are keyed by (`declaring` node id, `operation` name).

## Examined scope

- FR-038-AC-583 and the FSpec rules. For each of TA-01..TA-09 it gives the same code, cause and locus as FR-322-AC-492 and as the published vectors, and the same placement (wire validation for rule 1, after the operation stage for rule 5). Clean.
- TC-048 verifies FR-038-AC-583 and FR-038-AC-584. Clean.
- The FR-019 typed read views row now lists `CheckedTypeArgumentAlias`. Clean as an inventory entry.
- The FR-369 workload rule now matches FSpec FR-640 rule 3, which forbids repeating "an operation", not a name, and keys order by (declaring, name). Clean.

## Verdict

Changes required, all small. FR-369-AC-28 still states the old name-keyed repeat rule. FR-038-AC-584 consumes only part of FR-322-AC-493. The FR-019 row does not record that the change is breaking, which other FR-019 rows do.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-369-AC-28 still lists "a source workload with rows out of order, repeating an operation name or naming a `declaring` node that is no source object type of its owner" as a refusal. Under the amended rule (FR-369 workload bullet, QSL-2707), two rows of one name under two declaring nodes are admitted, so the AC now contradicts the requirement it verifies. It should say "repeating a (`declaring`, `operation`) pair" | spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md:350 |
| FND-002 | medium | FR-038-AC-584 claims to consume FSpec FR-322-AC-493 but narrows it to "three packages are admitted". FR-322-AC-493 also requires the same value and the same charges for every application, and requires that removing or replacing aliases leave every refusal of an otherwise defective variant unchanged (an operator-ineligible `result_type`, a `bound_is_member` mismatch). The IR AC states none of that, so a reader whose charges or refusal precedence changed with `alias` would still pass it | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6365 |
| FND-003 | low | The FR-019 typed read views row adds `CheckedTypeArgumentAlias`, but it does not say that `CheckedOperationMember::TypeArgument` gains the field `alias`. On an exhaustive pub enum that is a breaking change, and other FR-019 rows record such changes as "a breaking change that rides the next contract-ir tag (FR-371 Compatibility)" | spec/model/functional/FR-019-rust-library-interface.md:322 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | high | The FR-369-AC-26 amendment states that "where `Other` extends `O` and both declare `step`, a workload on `Other` that weighs `Other::poke` and `Other::step` and omits the inherited `O::step` refuses". FSL FR-185 "Workloads" says the effective view counts "an ancestor's operation that T or a nearer ancestor redefines" once, and counts an operation that several unrelated nearest ancestors declare once per declaration. `Other::step` redefines `O::step`, so that workload is complete and must be admitted. The diamond sentence is right; the redefinition sentence contradicts the producer's rule (see code-review FND-007). The selected-model rule kept in this PR (FR-369-AC-8, one operation per name) is consistent with FR-185 for redefinition | spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md:348 |

## Dispositions

Round 1, at head 9dda97186 (fix commit 9dda97186 over d1668db7). Focused runs in the private review worktree with a private target dir, via cargo-slot, one job at a time. Passing: conformance_fspec `type_argument_alias` (9) and `timed_slots` (8); `it` `checked_package_v2_source_objects::probabilistic` (36) and `checked_package_v2_owners` (34); lib `checked_package::v2::operations` (104), `checked_package::v2::probabilistic` (10) and `checked_package::shared` (21). The slow 300-row `checked_package_v2_probabilistic` filter was stopped unfinished, so its named-buffer test was not rerun here. The review-pass mutants M1 to M4 are now killed. M1 (key segment grammar), M2 (dependency owner join) and M3 (rule 4 always refuses) are each killed by one conformance test. M4 (rule 5 latest conflict) is killed by the lib test `the_earliest_alias_conflict_is_reported`. One new mutant survives: M5 drops `&& resolved.is_some()`. It is equivalent, because a `type_argument` member with no `declaration` is refused earlier by `member_is_well_formed_node`.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-369-AC-28 now reads "repeating a (`declaring`, `operation`) pair" |
| FND-002 | fixed | FR-038-AC-584 now also requires the same typed declaration for every application (hence the same value and charges), and the same refusal of two otherwise defective variants with no alias, one alias or another alias |
| FND-003 | fixed | The FR-019 row now ends: breaking changes that ride the next contract-ir tag (FR-371 Compatibility): `TypeArgument` gains `alias` and `CheckedPackageRefusal` gains `type_argument_detail` |

Round 2, at head 729efb3be (fix commit 729efb3be over 9dda9718). Focused run: `it` `checked_package_v2_source_objects::probabilistic`, 36 passed, which includes the join suite with its updated charge expectations. A local mutant that reverses the `reaches` direction in the effective-operation check fails `probabilistic::join` (20 passed, 1 failed), so it is killed. On the shadowed-row question: refusing a row that weighs the redeclared `O::step` at that row's operation name is right. Under FSL FR-185 that declaration is not an operation of `Other`'s effective view, so the row names an operation the model lacks. FSpec FR-640 rules 1 and 3 refuse such a misnamed operation `invalid-value` at that member, and an FSL producer never emits the row. Admitting it would give one operation two weights, which FR-185 refuses as `duplicate-weight`.

| FND | outcome | reason |
| --- | --- | --- |
| FND-004 | fixed | FR-369-AC-26 now says that coverage is keyed by the pair over the nearest declaration of each name (FSL FR-185): a redeclared ancestor operation is counted once and diamond declarations once each. `Other::poke` + `Other::step` is admitted, and omitting `Other::step` or weighing `O::step` refuses |
