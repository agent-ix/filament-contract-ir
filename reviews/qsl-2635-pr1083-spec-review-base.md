---
id: SR-17940486709564847048100
title: "spec-review of QSL-2635 FR-038-AC-582 and its TC-048 and matrix rows"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1083 at 3585ced2 (merge base 312f3752); spec/checked_package/functional/FR-038-consume-checked-package-v2.md (FR-038-AC-582), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md"
review_set: subset
---

## Summary

This is a spec review of the PR's three spec edits. Ticket: QSL-2635 (with QSL-2619). The edits were checked against FSpec origin/main 45b5e1b6, which descends from 04109664: FR-322 "Model declaration occurrences" and "One total order of the join stages" (FR-322-AC-275), FR-322-AC-371 (vector `stage`), FR-322-AC-485 and FR-322-AC-486.

`quire validate` on the three files reports no errors and no warning on the new row. The id FR-038-AC-582 is free on main. FR-038-AC-581 is reserved on the open branch `w82-QSL-2566`, so the skipped number does not collide.

## Examined scope

- FR-038-AC-582 (examined). The definition of a model declaration node, the entry pass in `source_map` order with its four defects, the missing pass in graph order, the code, cause and loci, the TW-29 to TW-34 outcomes, the extra cases and the charging sentence. These match FSpec FR-322 "Model declaration occurrences" except for the order clause (FND-001).
- TC-048 relationships and "Model declaration occurrences (FR-038-AC-582)" section (examined): accurate.
- matrix/tests.md FR-038-AC-582 row (examined): accurate.
- FSpec FR-322-AC-275 / "One total order of the join stages" (context_only).

## Verdict

The criterion is precise and testable. It restates FSpec's locus and precedence rules faithfully and adds the cases FSpec leaves to the reader: graph order for the first missing node, and a root entry of another role.

One defect. FSpec places stage 5 "after the edge existence check of stage 4". The AC says only "after the owner, derived-shape and selected-bound stages", and so silently drops stage 4. That clause is exactly the one the implementation cannot meet today (see code-review FND-003).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-582 orders stage 5 "after the owner, derived-shape and selected-bound stages", omitting FSpec FR-322's "after the edge existence check of stage 4". Because the reader has no stage-4 pass, the omission hides a real divergence: a package with a dangling stage-4 edge and a stage-5 defect reports stage 5. Either add the stage-4 pass and state the full order, or state the deviation in the AC and the matrix row ("stage 4 not yet a graph pass; follow-up <ticket>") so the row is not read as full conformance | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6323 |

## Dispositions

Round 1, at head de77dc0a (fix commit over 3585ced2), against FSpec 45b5e1b6. The focused runs are all green: `filament-contract-model --lib model_occurrence` 4/4, `conformance_fspec` (`model_tuple_union`, `clock_payload_tuple`, `extension`) 39/39, and `it --features conformance-cli` (the same filter as the review pass) 57/57. `rustfmt --check` is clean.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-038-AC-582 now says FSpec orders stage 5 after the stage-4 edge existence check, that the reader has no stage-4 graph pass yet so a package with both defects reports stage 5, and that QSL-2644 adds the pass. The TC-048 section and the matrix row repeat the known deviation (QSL-2644) |
