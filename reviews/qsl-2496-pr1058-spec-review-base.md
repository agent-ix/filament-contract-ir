---
id: SR-2026111905212900000003
title: "spec-review of FR-371, TC-477 and FR-038-AC-286 changes for QSL-2496/QSL-2503"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1058 at 31e982b2; spec/checked_package/functional/FR-371-read-claim-declaration-names.md, spec/checked_package/matrix/TC-477-claim-declaration-names-reader.md, spec/checked_package/functional/FR-038-consume-checked-package-v2.md (AC-286)"
review_set: subset
---

## Summary

Ticket: QSL-2496. The spec changes are checked against FSpec origin/main 9a181ace, FR-322 "Rows of one declaration", "Claim declaration names", "Temporal clause names" and "One claim namespace", and against FR-465 and TC-947 (QU3 and QU7) for queue capacity.

## Examined scope

- FR-371 statement, steps 5 and 6, the own-node paragraph, the hyper body wrapper paragraph, and the lookup sentence (examined). They match FR-322 W58: the declaration key, one name per declaration, a duplicate only across declarations, the strict-containment own node, the refusal locus and document position, the claim row rule, and the wrapper refusal at `/temporal_clause_names/{i}/declaration`.
- The FR-371 charge sentence and FR-371-AC-6 (examined). FSpec gives no charge for the own-node pass, so this IR-local wording is allowed. It is stated precisely (two per row, one per set element compared) and it describes the code. The missing test is reported in the gap-analysis SR.
- FR-371-AC-10 and AC-11 (examined). Both are testable and match FR-322-AC-465 and AC-466.
- TC-477 steps 5, 11, 12 and 13 (examined). They list the published vectors and the new procedures.
- FR-038-AC-286 (examined). Capacity 0 admits and a negative capacity refuses `invalid_package`/`invalid-value` at the capacity value. This matches FR-465 "non-negative integer" and TC-947 QU3 and QU7.

## Verdict

The spec content is sound and has one wording defect.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-371's writer sentence repeats a phrase: "The canonical writer SHALL the canonical writer SHALL accept optional borrowed rows". | spec/checked_package/functional/FR-371-read-claim-declaration-names.md:174 |

## Dispositions

Round 1, head f3ab09af.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The duplicated phrase is gone: "The canonical writer SHALL accept optional borrowed rows". The lookup sentence now says that only rows carrying the path count. |
