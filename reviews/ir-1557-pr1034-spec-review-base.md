---
id: SR-17940486709564847048040
title: "spec-review of IR-1557 FR-038 check_mode criteria"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1034 at 829e7c07; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (Function check mode, FR-038-AC-555 to AC-560), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md, spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

Spec review of the spec edits in PR #1034. Ticket: IR-1557. The new FR-038 prose and AC-555 to AC-560 were checked against FSpec FR-770 Behavior and AC-1/AC-2, CM-01 to CM-04, and the adverse.json check-mode entries (filament-specification origin/main 669da4fb). `quire validate` exits 0.

## Examined scope

- FR-038 "Function check mode" prose (examined): it matches FR-770 on presence, the closed values, non-function refusal, identity and the post-join `kernel` rule, and it names the pointer CM-03 and CM-04 record.
- FR-038-AC-555 to FR-038-AC-559 (examined): each is atomic and testable, with exact codes and loci. AC-556 adds `null`, which matches adverse `check-mode-null`. AC-557 adds `null` on a non-function node, which the schema's `not required` also refuses.
- FR-038-AC-560 (examined): a conformance criterion over the published vectors.
- TC-048 section and matrix row (examined): the procedure matches the ACs.
- FR-019 public-items table (examined): `CheckMode` is listed.

## Verdict

Clean. The criteria restate FR-770 faithfully and add only the reader-side refusals FSpec's vectors already imply. The V1 ContractPackage omission is in the gap-analysis SR (FND-002) as a planner question. It is not a defect in these criteria.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
