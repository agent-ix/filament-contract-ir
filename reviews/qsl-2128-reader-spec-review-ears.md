---
id: SR-967175026302197768900474181941935084582
title: "spec-review/ears of QSL-2128 FR-351 declaration-order paragraph and AC-6, AC-7"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/filament-contract-ir; PR #1122 at 5e39d58b88a836483732a351b1ecee0bb97ab03c; spec/checked_package/functional/FR-351-admit-step-relation-claim-body.md (Form and body paragraph lines 71-87, FR-351-AC-6, FR-351-AC-7)"
review_set: subset
---

## Summary

EARS review of the FR-351 statements and ACs added by this PR. Ticket: QSL-2128. `quire validate --scope .` on FR-351 gives RC=0 with no errors. Only workspace module warnings, which are not about this file, were printed.

## Examined scope

- FR-351 body sentence "The body shall be an aggregate of four ordered bindings ... optionally followed by one final binding `subject_declaration_order`" (examined). It is a ubiquitous shall statement and conforms.
- FR-351 declaration-order paragraph (examined). "It shall be an exact permutation" and "The reader shall retain the supplied sequence" conform. Three other statements do not; see FND-001.
- FR-351-AC-6 and FR-351-AC-7 (examined). Each is a testable criterion with a concrete stimulus and an observable result.

## Verdict

One low wording finding. No statement conflicts with FSL FR-940.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Three new normative statements are declaratives rather than shall statements: "The optional `subject_declaration_order` is a Group ...", "A body without it remains admitted, and the reader exposes the sequence ...", and "Its refusals are `invalid_semantic_graph` ...". The rest of FR-351, and FSL FR-940's paragraph, use SHALL. Fix: use "shall be", "shall remain admitted ... shall expose" and "shall refuse ... with". | spec/checked_package/functional/FR-351-admit-step-relation-claim-body.md:71-80 |
