---
id: SR-967175026302197768900474181941935084554
title: "QSL2271 PR 1121 spec-review/integrity of FR-378 and TC-493"
type: SpecReview
analysis: integrity
scope: "agent-ix/filament-contract-ir; QSL-2271; PR 1121 head 50ba1ace53b6388ff1e41b329b6c0bfb33e25dcb; spec/checked_package/functional/FR-378-read-protocol-correlation-provenance-of-a-checked-package.md, spec/checked_package/matrix/TC-493-protocol-correlation-provenance-read-from-an-admitted-package.md"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-378", type: references }
---

## Summary

Integrity review of FR-378 and TC-493 for completeness, consistency and atomicity. Ticket: QSL-2271. Both files validate with quire.

## Verdict

**CONDITIONAL**: one low finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-378 Behavior is incomplete against its own AC-4. It states only the row work charge. It omits the coverage-pass work charge 1+2·Bn(+O+m), the read_storage view block G and the per-node m flag bytes, all of which AC-4 verifies by name. It also does not say they are delegated to FR-831. | spec/checked_package/functional/FR-378-read-protocol-correlation-provenance-of-a-checked-package.md:46-66,105 |

## Coverage

- Atomicity of AC-1 to AC-5 was examined: each verifies one vector group.
- TC-493 scope and procedure agree with AC-1 to AC-5.
- The tests.md rows are consistent with the TC.
- The inconsistent coverage wording is recorded once, in spec-review/base FND-001.

## Dispositions

Round 1 is checked at frozen head 6df4aaf2c86160772eb39f8c9d736f2b8a3998b7: fix c091064ca, then merge of origin/main 7e4f54095. Arch ruling /tmp/ir-qsl2271-cob11-ruling.txt chose option A: the generic result box is reserved before the last carrier step. Reviewer rerun: name-filtered protocol_correlation::correlation_ conformance (see report).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fixed in c091064ca; after_excerpt in the round-1 dispositions comment. |
