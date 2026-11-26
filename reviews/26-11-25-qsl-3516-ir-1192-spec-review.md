---
id: SR-5761734
title: "Dependency link review of QSL-3516 contract-ir PR 1192"
type: SpecReview
analysis: dependency
scope: "agent-ix/filament-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md frontmatter relationships; head 5d8460473419967543fabfe6114a82e295cadc02"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: references
---

## Summary

Ticket: QSL-3516. Reviewed PR #1192 at head 5d8460473419967543fabfe6114a82e295cadc02. The diff adds one entry to FR-038's `relationships`: `ix://agent-ix/filament-specification/FR-322-AC-291` with type `depends_on`. Arch agreed this link replaces the contract-ir FR-066 link the ticket names, because FR-066 covers only execution points.

Checked: (1) whether the link holds for the text of both requirements; (2) the link form; (3) `quire validate`; (4) overlap with open PR #1154. This was a spec-only review, so no builds or tests were run.

## Verdict

PASS, no findings.

- **The link holds.** FSpec FR-322-AC-291, on filament-specification origin/main f6500eef, says the producer contract for each `protocol_locations` row is: it joins to one local `expression`/`application` node and to one exact source occurrence; terms that share an application and occurrence have distinct `term_ordinal` values; and ordinals in each group are consecutive from zero, with PL-01-root admitted and PL-R2 refused at `/checked_term_data/protocol_locations`. FR-038's Description states the matching strict reader rules: the row join to an admitted local `expression`/`application` node, the duplicate-key and canonical-order checks, and the missing-predecessor (consecutive-ordinal) array check. Its `body_locations` rules reuse that "row shape, key, consecutive-ordinal, row-join" rule set. The reader enforces the producer's rule, so `depends_on` points the right way (reader depends on the producer contract). FR-066 has no protocol-location or `term_ordinal` text, so moving the link to FR-038 is correct.
- **The link form is correct.** The target uses the cross-repo `ix://agent-ix/filament-specification/<AC-id>` form that the other ~70 FSpec entries in FR-038 use. The AC id exists on FSpec main (FR-322 table, `spec/objects/interfaces/FR-322-checked-package-artifact.md`). `depends_on` is an accepted relationship type, and this repo already uses it for FSpec targets (FR-058 to FSpec FR-341/342, FR-353 to FSpec FR-322). The entry sits in numeric order between AC-288 and AC-297. It does not duplicate an existing `references` entry for AC-291, because there is none.
- **`quire validate` passes.** quire 0.52.1 exits 0 on the file. Its four `ac:non-canonical-shape` grammar warnings (lines 7168, 7313, 7324, 7325) also appear on the origin/main copy, so this PR did not introduce them.
- **No conflict with PR #1154.** PR #1154 (head 681ff5cb) changes README.md, `checked_json.rs` and two `reviews/` files. It does not touch FR-038. The ticket comment that says #1154 "touches FR-038 as well" is out of date. PR #1192 reports MERGEABLE.

Methods: spec-review dependency check only. code-review, gap-analysis and EARS were skipped because the diff contains no code and no requirement statement or criterion text.

Note, not a defect: FR-038 has its own local criterion FR-038-AC-291 (binder levels), which has nothing to do with FSpec FR-322-AC-291. A reader should not take the shared number to mean the two are linked.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
