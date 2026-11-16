---
id: SR-20261116190005
title: "spec-review of IR-1552 FR-038 and FR-069 Clock position and payload text"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1032 at 7852391e; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (clock place paragraph, FR-038-AC-548..553), spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md (Clock positions and payloads, FR-069-AC-9), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md"
review_set: subset
---

## Summary

Spec review of the FR-038 and FR-069 changes in PR #1032. Ticket: IR-1552 (bullets: Clock payload and tuple position admitted; clock places `field.tuple(0)` and `field.union(Armed)`; empty-payload `union_value`, no presence bit; `clock_reset` of `self.p = Phase::Armed(0 ms)`; `tuple.project` refusal; `Set<Slot>[0, 2]` and map-key refusal; FR-038/FR-069 text). Upstream oracle read: filament-specification origin/main 959a1982, FR-322 "Clock positions and payloads" (lines 2332-2357), the `resolved_path` definition, FR-322-AC-399, FR-322-AC-145, TC-1361 (CP-01..CP-07), FR-415 "Clock collections" and FR-415-AC-10. `quire validate` of the repo spec tree passes.

Completeness question resolved. FR-322-AC-399's own text names TK-08/TK-09, the two places, no clock node, the empty-payload `union_value`, liveness with no presence bit, the reset, the `tuple.project` refusal, `Set<Slot>[0, 2]` and the map-key refusal; it does not name `=`/`!=` (CP-07) or the `present(self.p.Armed)` guard (CP-03). But AC-399's verification is `Test (TC-1361)`, TC-1361 requires that "every vector gives exactly its expected outcome" and lists CP-03 and CP-07, and the FR-322 normative clause that AC-399 tests states the CP-07 rule as a reader refusal: "`=` and `!=` over a clock-bearing tuple or union would read its clocks, so they refuse `invalid_timed_model`/`clock-read-outside-constraint`". So CP-07 is part of the published vector set for AC-399 and of the FR-322 clause the PR mirrors (FND-001). CP-03's refusal half (`undefined_expression`/`unproved-presence`) is a producer/checker refusal with no reader cause in this repo; its admission half (a guard part carrying `present(self.p.Armed)`) is reader-visible but untested and unstated (FND-002).

## Examined scope

- FR-038 clock place paragraph (examined): the join of `tuple`/`union` rows to the selected declaration, the identity forms, the refusal list, `Set<Slot>[0, 2]`, no node, no presence member, liveness by active member, empty-payload `union_value` with `type-mismatch` on a payload term, arity zero, the reset frame rule, the `tuple.project` refusal and the map-key refusal all agree with FR-322 and the `resolved_path` definition.
- FR-038-AC-548..553 (examined): each is testable and matches the code; AC-552 has no tagged test (gap-analysis SR).
- FR-069 "Clock positions and payloads" and FR-069-AC-9 (examined): consistent with FR-322-AC-145 and AC-399 except the omitted equality sentence (FND-001).
- TC-048 section and tests.md rows (examined): see FND-003.
- `member_identity` form (examined): `<tuple identity>/p<i>` is defined by FR-154 line 168, FR-208 line 329 and FR-322 lines 2035/2231; no published vector carries a tuple or union `resolved_path` row, so no vector conflicts.

## Verdict

Not mergeable as is. The ticket bullets are delivered, but the PR records FSpec FR-322-AC-399 as IMPLEMENTED while the FR-322 clause's `=`/`!=` refusal (TC-1361 CP-07) is neither implemented nor recorded as Planned. Either implement it or state in FR-038/FR-069 that CP-07 (and the reader half of CP-03) remain Planned under a named ticket.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-322 "Clock positions and payloads" requires `=` and `!=` over a clock-bearing tuple or union to refuse `invalid_timed_model`/`clock-read-outside-constraint` (TC-1361 CP-07, part of AC-399's verification). The reader does not implement it (no equality check reads `MemberType::Clock`), and FR-038's new paragraph and FR-069's new section mirror the clause but drop that sentence, while FR-038-AC-548..553 and FR-069-AC-9 cite FSpec FR-322-AC-399 as IMPLEMENTED with no Planned remainder | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1759-1784; spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md:65-76 |
| FND-002 | low | TC-1361 CP-03's reader-visible half (a guard whose convex part carries the data predicate `present(self.p.Armed)` over the `Armed` place) is neither tested nor stated out of scope; the refusal half is producer-side (no `unproved-presence` cause in this repo), which the spec also does not say | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1773-1775 |
| FND-003 | low | FR-069-AC-9 is verified "Test (TC-940)" and tests.md adds an FR-069-AC-9/TC-940 row, but the TC-940 test-case row (tests.md line 74) still lists FR-069-AC-1, -2, -7 only, and both tests that carry FR-069-AC-9 are tagged `TC-048` | spec/checked_package/matrix/tests.md:44,74; tests/conformance_fspec/clock_payload_tuple.rs:545; tests/conformance_fspec/model_tuple_union.rs:786 |
