---
id: SR-967175026302197768900474181941935084553
title: "QSL2271 PR 1121 spec-review/ears-conformance of FR-378"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/filament-contract-ir; QSL-2271; PR 1121 head 50ba1ace53b6388ff1e41b329b6c0bfb33e25dcb; spec/checked_package/functional/FR-378-read-protocol-correlation-provenance-of-a-checked-package.md"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-378", type: references }
---

## Summary

EARS conformance of the new FR-378 statement and its Behavior SHALLs. Ticket: QSL-2271.

## Verdict

**CONDITIONAL**: one low finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The FR-378 Description statement is compound. One event-driven statement carries three SHALL obligations for three actors: the reader admits or refuses, the writer writes rows as given, and the admitted package exposes one typed read. FSpec FR-831 splits the same content into single-obligation statements. | spec/checked_package/functional/FR-378-read-protocol-correlation-provenance-of-a-checked-package.md:20-27 |

## Coverage

- Behavior SHALLs were examined: "The reader SHALL take each row through …" (line 47) and "The reader SHALL read a dependency package from the dependency's own member" (line 68). Both are single-actor and ubiquitous, with no grammar defect.
