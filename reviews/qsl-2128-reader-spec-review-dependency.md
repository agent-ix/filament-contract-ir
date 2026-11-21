---
id: SR-967175026302197768900474181941935084584
title: "spec-review/dependency of QSL-2128 FR-351 references edge to FSL FR-940"
type: SpecReview
analysis: dependency
scope: "agent-ix/filament-contract-ir; PR #1122 at c4dd58a5170bbb2fd40eb6166d71a0b7c202fc20; spec/checked_package/functional/FR-351-admit-step-relation-claim-body.md (relationships)"
review_set: subset
---

## Summary

Dependency analysis of the one `relationships` edge that fix round 1 adds to FR-351. Ticket: QSL-2128. This SR is new in disposition pass 1 because the edge did not exist at the first review. Model claude-opus-5-5, run 05d86976-7b49-4d52-aca2-f22ca0ff74ad.

## Examined scope

- Edge `references` → `ix://agent-ix/filament-spec-language/FR-940` (examined). The target exists on FSL origin/main 19264bdd0a4d39d700de08dc0602df600343c155 (FR-940, "analyze through one sealed package read view"). The edge type is `references`, the same type FR-351 already uses for the external FSpec FR-477, FR-322, FR-341 and FR-402. This is correct: FR-940 owns the package form and consumes the IR body, and the IR reader is not an enabler of FR-940. The edge adds no `depends_on` and no `implements`.
- Cycles (examined). FR-940's frontmatter has no edge to IR FR-351 at origin/main, f0a8e7f6 or b891628f. It names FR-351 only in prose, so no relationship cycle exists.
- Enablement versus feature (examined). The IR reader half is enablement for the FSL producer and form-selection work, which Terra owns. The IR spec does not claim that work.
- Content currency (context_only). The "Declaration-order member contract" paragraph is not yet on FSL origin/main (0 matches). It exists only at the reviewed FSL PR heads f0a8e7f6 and b891628f, where it is identical. Until that FSL PR merges, readers who follow the edge from IR main will see FR-940 without the paragraph. This sequencing was accepted by the architecture ruling. It is not a defect in the edge.

## Verdict

Clean. `quire validate` RC=0.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
