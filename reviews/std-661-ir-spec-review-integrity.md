---
id: SR-2026111422264100000003
title: "spec-review/integrity of STD-661 FR-038 operation-body carrier wording"
type: SpecReview
analysis: integrity
scope: "agent-ix/filament-contract-ir; PR #1024 at 5c735908; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (Operation-body carrier, FR-038-AC-541), spec/checked_package/matrix/tests.md (FR-038-AC-541 row)"
review_set: subset
---

## Summary

Spec-review/integrity of the FR-038 edits, checked against filament-specification FR-322 "Check order" and "Candidate join" and TC-987 at origin/main 17d49a98. Ticket: STD-661. `quire validate --scope .` passes on both files with 0 errors. AC-541 has no grammar warning.

## Examined scope

- FR-038 "Operation-body carrier" paragraph (examined). The joined candidate's receiver must be in the family and effectively expose the row's operation from the row's declaring node. A candidate outside the family, or one whose effective operation has no row, is not joined and goes to dispatch validation (OBD-01). This agrees with FR-322 "Candidate join". The has_body clause ("ModelOwner is the row's document and operation member") is equivalent to FR-322's node-only compare, by FR-322's own identity argument.
- FR-038-AC-541 (examined). "The reader reads FSpec TC-987's positive cases OBC-01 to OBC-07, including one dispatch table whose three candidates share a selected operation and join three different rows, and refuses its 13 schema cases OBS-01 to OBS-13 as `malformed_wire`, its 21 reader cases OBR-01 to OBR-20 and OBR-22 with the published code, cause and locus, and its dispatch-join case OBD-01 at the dispatch join". The ids and counts match the vectors (7 / 13 / 21 / 1). The OBR-21 and OBC-04 exceptions were removed correctly, since OBR-21 no longer exists and OBC-04's node is now owned by its own document.
- Matrix row FR-038-AC-541 (examined): it agrees with the AC and the code.
- Stale references (examined): outside `reviews/ir-1528-*.md`, nothing references OBR-21, the old OBR-20 caveat or "STD-661 tracks". Those review files are history and were left untouched, as required.

## Verdict

Clean. The FR-038 wording, AC-541, the matrix row and the code comments agree with FR-322 and TC-987.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
