---
id: SR-967175026302197768900474181941935084555
title: "QSL2271 PR 1121 spec-review/dependency of FR-378 relationships"
type: SpecReview
analysis: dependency
scope: "agent-ix/filament-contract-ir; QSL-2271; PR 1121 head 50ba1ace53b6388ff1e41b329b6c0bfb33e25dcb; spec/checked_package/functional/FR-378-read-protocol-correlation-provenance-of-a-checked-package.md (relationships, Dependencies)"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-378", type: references }
---

## Summary

Dependency review of the new FR-378 relationship edges. Ticket: QSL-2271. The edges are traces_to StR-001, depends_on FR-038, and references to FSpec FR-831 and FR-322.

## Verdict

**CONDITIONAL**: one low finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-378 charges the one shared NativeJoinBudget and its work and read_storage counters. Those are owned by this repo's FR-358, and FSpec FR-831 names IR FR-358 as their owner. FR-378 has no depends_on or references edge to FR-358 and no Dependencies entry for it. | spec/checked_package/functional/FR-378-read-protocol-correlation-provenance-of-a-checked-package.md:5-13,108-114 |

## Coverage

- The FR-038 depends_on edge is correct: the V2 reader, writer and refusal machinery.
- The FR-831 and FR-322 references are correct.
- The StR-001 trace is consistent with sibling checked_package FRs.
