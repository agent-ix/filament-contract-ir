---
id: SR-967175026302197768900474181941935084523
title: "spec-review integrity analysis of QSL-2739 AC10 composition reader requirements"
type: SpecReview
analysis: integrity
scope: "agent-ix/filament-contract-ir; PR #1120 at ddfb8854e; FR-038 composition paragraph and FR-038-AC-602..605, FR-371 Behavior paragraph and FR-371-AC-12, TC-474, spec/checked_package/matrix/tests.md"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-038", type: references }
  - { target: "ix://agent-ix/filament-contract-ir/FR-371", type: references }
---

## Summary

Ticket: QSL-2739. This is the integrity analysis (completeness, consistency, atomicity) of the requirement units the PR adds. The FR-038 paragraph, the FR-371 paragraph and the five ACs agree with each other and with FSpec FR-322 and FR-491 on the codes, loci and check order. The TC-474 and TC-477 matrix rows list the new ids consistently. Two low atomicity and ordering defects remain.

## Examined scope

- FR-038-AC-602 (examined): one positive scenario with its write and re-read. It is atomic.
- FR-038-AC-603 (examined): compound. See FND-001.
- FR-038-AC-604 and AC-605 (examined): enumerated mutation families of one rule, the same form as FSpec FR-491-AC-8. Acceptable.
- FR-371-AC-12 (examined): one positive case and enumerated refusals of one row rule, the same form as FR-322-AC-515.
- Consistency (examined). The FR-371 paragraph and the FR-038 paragraph agree on "equal, not contains", on the declaration at ordinal 0 and on the refusal loci. The FR-038 paragraph's "after the package-wide source-map checks" agrees with AC-605.
- tests.md rows (examined). FR-371 now reads "AC-1 through AC-9, AC-12", and the AC-10/11 omission was already present before this PR.

## Verdict

**CONDITIONAL**: low findings only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-038-AC-603 bundles three independent obligations: an `implementation_guarantee` still refuses a `declaration` member, a composition requires its `declaration`, and a declared path other than `node_id.path` refuses `declaration-nominal-mismatch`. One failing clause cannot be traced on its own. | FR-038-AC-603 |
| FND-002 | low | FR-038-AC-602..605 are inserted between AC-601 and AC-596, which extends the AC table's non-sequential order. A reader scanning for the highest id or the next free id can misread it. | FR-038-AC-602 |

## Dispositions

Disposition pass 1 covers fix commit 009c94728 at head 009c9472805e4a80d4851ccf0d0ff2fe57f061b8.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AC-603 is split into atomic AC-603 (guarantee declaration refused), AC-606 (composition declaration required, with a positive control) and AC-607 (path mismatch nominal), each with its own test. |
| FND-002 | fixed | Rows FR-038-AC-602..613 now follow the AC-596 row, so the 596, 602..613 block is in numeric order. |
