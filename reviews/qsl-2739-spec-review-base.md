---
id: SR-967175026302197768900474181941935084522
title: "spec-review (base checklist) of QSL-2739 AC10 composition reader requirements"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1120 at ddfb8854e; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (composition paragraph, FR-038-AC-602..605), spec/checked_package/functional/FR-371-read-claim-declaration-names.md (Behavior paragraph, FR-371-AC-12), spec/checked_package/matrix/TC-474-*.md, spec/checked_package/matrix/tests.md"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-038", type: references }
  - { target: "ix://agent-ix/filament-contract-ir/FR-371", type: references }
---

## Summary

Ticket: QSL-2739. This is the base checklist review of the spec text the PR adds, checked against the standard it consumes: FSpec FR-322 "Composition declaration names" (FR-322-AC-514/515) and FR-491 "Composition member occurrences" (FR-491-AC-7/8) at 9add8a0f, which includes the PR #798 empty-interval ruling. No `AssuranceProfile` document exists under `spec/`, so no review policy applies, and the review set was chosen as subset: base, integrity and ears-conformance. The ID formats are valid and unique against origin/main e979980ad: FR-038-AC-600/601 were the last used and FR-371-AC-11 the last before AC-12, and `git merge-tree` with main is clean. The matrix rows reference only existing ids. There are no hash, digest or pin uses. Two IR criteria narrow FSpec clauses without saying so, and one boundary is unstated.

## Examined scope

- FR-038 composition paragraph (examined). It adopts FR-491 and FR-322, and states the check order and the refusal codes and loci. It states the #798 precedence verbatim.
- FR-038-AC-602, AC-603, AC-604, AC-605 (examined).
- FR-371 Behavior composition paragraph and FR-371-AC-12 (examined).
- TC-474 Description, plus the tests.md FR-371, TC-477 and FR-038/TC-474 rows (examined). They are consistent with the ACs.
- FSpec FR-322-AC-514/515 and FR-491-AC-7/8 (context_only, as the consumed standard). FSpec #797 added AC text only and published no JSON vectors. The old-schema-red / new-schema-green control is not runnable in IR, because the V2 schema is not vendored here. That limitation is honest and is not a finding.

## Verdict

**CONDITIONAL**: two medium and one low finding, all fixable by amending the AC text in this PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-605 states only an empty interval and a repeated key as generic precedence. FSpec FR-491 also requires the unknown-node and total-occurrence/source-map refusals, and a generic defect of another node, to keep their precedence over composition coordinates. The AC narrows the standard to what the implementation tests. | FR-038-AC-605 |
| FND-002 | medium | FR-371-AC-12 consumes FR-322-AC-514 but omits two of its clauses without naming their owner. One is source-order default selection by lock-source position, then start, then end. The other is that another correspondence form stays ineligible for a claim-name row. | FR-371-AC-12 |
| FND-003 | low | FR-038-AC-604 lists "outside" and "equal to it" for the strict-inside boundary, but not a member region sharing only the declaration's start or only its end. Those are the boundaries that distinguish strict from non-strict containment. | FR-038-AC-604 |
