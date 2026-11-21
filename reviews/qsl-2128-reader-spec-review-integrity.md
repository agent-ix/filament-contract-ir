---
id: SR-967175026302197768900474181941935084583
title: "spec-review/integrity of QSL-2128 FR-351 declaration-order extension"
type: SpecReview
analysis: integrity
scope: "agent-ix/filament-contract-ir; PR #1122 at 5e39d58b88a836483732a351b1ecee0bb97ab03c; spec/checked_package/functional/FR-351-admit-step-relation-claim-body.md (Form and body, Identity and order, FR-351-AC-6, FR-351-AC-7)"
review_set: subset
---

## Summary

Integrity review of the FR-351 edits, checked against FSL FR-940 "Declaration-order member contract" at f0a8e7f6 (identical at b891628f) and against the implementation. Ticket: QSL-2128. The scope stays narrow: the legacy four bindings plus an optional fifth. It is not a seven-binding migration. The FSL-side form-selection refusal (`ModelRelationFormRefusal::MissingSubjectDeclarationOrder`) correctly stays out of IR scope.

## Examined scope

- Declaration-order paragraph (examined). Its shape, exact permutation, shared root, kept order, legacy admission and pointer list agree with FR-940 and with the code.
- Identity and order, "Within one node the reader shall check shape, aliases, targets in body order, premises, then Boolean body" (examined). See FND-001.
- FR-351-AC-6 (examined). See FND-002.
- FR-351-AC-7 (examined). It is consistent with the pointer list.
- Frontmatter relationships (examined). See FND-003.

## Verdict

One medium integrity gap and two low findings.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-351's intra-node order ("check shape, aliases, targets in body order, premises, then Boolean body ... reporting the first failure") does not say where the new member's checks run. The code runs the entry shape, alias and join checks (step_relation.rs:727-787) after the subject model-root targets and before the execution operation targets. Two consequences follow. A shape-malformed declaration entry is reported after an alias fault. A join fault in member 4 is reported before an operation-target fault in member 1. Neither order can be derived from the text, so two implementers would disagree. Fix: add one sentence placing the declaration-order checks, for example "after the subject targets and before the operation targets", or reorder the code to match "body order", and add a two-defect test. | spec/checked_package/functional/FR-351-admit-step-relation-claim-body.md:116-119; crates/filament-contract-model/src/checked_package/v2/step_relation.rs:727-787 |
| FND-002 | low | FR-351-AC-6 combines five obligations in one criterion: admission and getter order, the write and read roundtrip, legacy admission with `None`, and `step_relation_body` invariance. A failure does not identify which one broke. Fix: split it into AC-6 (order and roundtrip) and a separate legacy and invariance AC, or accept it in the style of the existing FR-351 ACs. | spec/checked_package/functional/FR-351-admit-step-relation-claim-body.md:137 |
| FND-003 | low | FR-351 adds the contract that FSL FR-940 "Declaration-order member contract" names as "the optional fifth member of the current four-binding filament-contract-ir FR-351 body", but FR-351 has no `references` relationship to `ix://agent-ix/filament-spec-language/FR-940`. Its prose does not cite FR-940 either, so the upstream source cannot be traced from IR. Fix: add the `references` edge. | spec/checked_package/functional/FR-351-admit-step-relation-claim-body.md:1-23 |
