---
id: SR-17940486709564847048122
title: "Spec review — FR-726 FIFO-key-stable rule as implemented for QSL-2726"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; branch code/fifo-key-quantity at fbcddd32; reference spec agent-ix/filament-specification FR-726, FR-322 Equality-stable types, TC-1770, proposals/checked-package-v2/fixtures/protocol-node-vectors.json"
review_set: subset
---

## Summary

Ticket: QSL-2726. The PR has no spec diff. This base spec review checks that the FSpec rule the code implements is consistent and complete. It reads FR-726 Description/Behavior/AC-1..3, the FR-322 "Equality-stable types" paragraph and FIFO-key-stable sentence, TC-1770, and the FSpec checked-package-v2 protocol-node vectors.

## Examined scope

- FR-322 "Equality-stable types" (examined). It lists the families `boolean`, `integer`, `rational`, `decimal`, `text`, `string`, `uuid`, `timestamp`, `bytes`, `json_object`, `enum`, `ordered_enum`, `quantity`, `reference` and `step_identity`, plus structural kinds with no `event_type` leaf. `bytes` and `json_object` are named equality-stable here and in FR-322-AC-102, so admitting them is required, not optional.
- FR-726 Description (examined). It defines FIFO-key-stable as equality-stable with no float32/float64 leaf, and enumerates quantity, bytes, json_object, enum, reference and model value types. This is consistent with FR-322 and with the code.
- FR-726 Behavior (examined). The walk goes through option, collection, record, tuple and variant to every leaf. This matches `equality_stable`.
- FR-726-AC-2 and AC-3 (examined). Each is atomic enough and testable, and TC-1770 steps 2 and 3 match them.
- FSpec protocol-node vectors (examined). The FIFO cases (167, 170, 171) cover the binder and a missing key only. No vector exercises the key type (quantity, bytes or json_object admitted; float leaf refused).

## Verdict

**PASS with one low finding**, which lies on the FSpec side. The rule is consistent between FR-322, FR-726, FR-372 and the code.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FSpec ships no protocol-node conformance vector for FIFO key-type admission: no admitted quantity, bytes or json_object key and no refused float-leaf key. `make conformance-fspec` therefore cannot detect producer/reader drift on FR-726-AC-2/AC-3, and only the repo-local fixtures guard it. This is out of this PR's scope; it should be filed against filament-specification if wanted. | proposals/checked-package-v2/fixtures/protocol-node-vectors.json |
