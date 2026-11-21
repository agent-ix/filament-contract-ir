---
id: SR-967175026302197768900474181941935084533
title: "QSL-2743 PR 1119 EARS conformance of the new FR-358 closure-locus statement"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/filament-contract-ir; QSL-2743; PR 1119 head c9b9a2415c0c78e2b42a16757e1869ebdf850e96; spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md lines 213-226 (new Behavior paragraph)"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-358", type: references }
---

## Summary

EARS requirement-grammar analysis of the one requirement statement PR 1119 adds in fix round 1: the FR-358 Behavior paragraph that locates reached-closure charges. Ticket: QSL-2743. The paragraph has one statement with a `SHALL`: "Because such a charge names no sidecar row, IR SHALL locate it at `NativeJoinLocusSource::GraphNode` of the admitted package node being expanded or whose edge is being examined." That statement is clean.

The engine check is `quire validate --scope .` on FR-358 (quire 0.36.2). It reports no `[ears:*]` warning for lines 213-226. Its only FR-358 grammar warnings are at line 246 (`ears:unclassifiable` and `ears:missing-subject`). That statement predates this PR; it was line 231 before the 15-line insertion.

## Verdict

**PASS** — the new statement conforms.
- It is a ubiquitous requirement, singular, with the named subject `IR`.
- Its response is concrete and verifiable: the locus source `NativeJoinLocusSource::GraphNode`, at a stated node, with forms named in the same paragraph. B02 asserts it.
- The leading "Because …" clause is a rationale, not a trigger. It does not turn the statement into an event or state requirement, so no `When` or `While` keyword is called for.
- The paragraph's remaining sentences carry no `shall`. They are context: the FR-830 relationship, and that the common-pass and row-join loci are unchanged.

The content defects in this paragraph (its form list, and how it paraphrases FR-830) belong to the base checklist, SR-967175026302197768900474181941935084532 FND-003 and FND-004. They are not EARS grammar defects.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
