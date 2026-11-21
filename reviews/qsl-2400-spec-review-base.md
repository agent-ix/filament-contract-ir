---
id: SR-2026112109235103
title: "PR 1111 specification base review"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; QSL-2400; FR-038-AC-598/599 and changed FR-019 public API row"
review_set: subset
relationships:
  - {target: "ix://agent-ix/filament-contract-ir/FR-038", type: references}
  - {target: "ix://agent-ix/filament-contract-ir/FR-019", type: references}
---

## Summary

Base specification review of the two new reader criteria and public ClaimNameRow API documentation against merged FSpec PR 778. The source-unit carrier agrees with the architecture ruling: claim_names owns it; hyper temporal_bindings rows do not.

## Verdict

PASS for the changed specification text. Implementation defects and missing behavioral evidence are recorded separately in SR-2026112109235101 and SR-2026112109235102.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope

- FR-038-AC-598, examined: "A body wrapper's atom slots are the `slot` parameters its formula reaches, one node per name, in ascending byte order of `name`, at levels 0 to `n - 1`; its `over` is the first."
- FR-038-AC-599, examined: "`ClaimNameRow` gains `source_unit`, an optional `CheckedSourceRef` carried by the strict reader and canonical writer; a `claim`/`hyperproperty` node's row must name one of the lock's sources and no other row may."
- FR-019, examined: "`ClaimNameRow` gains the optional field source_unit (FR-322 Hyper clause source unit), a breaking change that rides the next contract-ir tag (FR-371 Compatibility)."
- FR-402-AC-14/15, FR-322-AC-508/511 and FR-778-AC-13, context_only: upstream requirements establish body-derived slot identity, leaf binder base and occurrence-owned hyper source identity.

## Coverage

Checked identifier uniqueness, existing structured upstream relationships, exact refusal codes/paths, source validation order, byte-name scope order, semantic carrier ownership, breakage notice and computed new-criterion tags. The change consumes the existing upstream value-object contract rather than creating a new domain object, endpoint, dependency relationship, requirement statement or system boundary; separate object, dependency, EARS and scope-boundary artifacts are not applicable. No active AssuranceProfile applies. Criterion-strength/Jev was not selected or run. No new file-tracking, digest ledger, duplicated normative vocabulary, or tool pin is introduced.

quire validate of both changed specification files exits 0. It emits four existing non-canonical-shape warnings elsewhere in FR-038 (lines 6722, 6864, 6875 and 6876), outside this diff. These were not edited or hidden. Matrix and source evidence limitations are recorded in the gap artifact.

## Round 1 verification

PASS at f23a39ec8693f1b0d32b68794c18db1262252b20. Original review and placeholder are unchanged; this method had no defect requiring disposition. The additive rebase preserves FR-038-AC-595 together with AC-598/599, and FR-019 retains the ClaimNameRow source_unit API notice. No new specification finding was identified. Code and gap dispositions are recorded in their own unchanged-id artifacts. Final compact-body API integration remains with the coordinator.
