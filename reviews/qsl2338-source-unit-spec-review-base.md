---
id: SR-17940486709564847048131
title: "spec-review of FR-322 'Clause source unit' and IR FR-038-AC-590 as QSL-2338 consumes them"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; branch code/qsl-2338-clock-unit-owner at dff7dfd7b; IR spec/checked_package/functional/FR-038-consume-checked-package-v2.md (AC-590), spec/model/functional/FR-019-rust-library-interface.md (temporal clock bindings row); FSpec PR #777 head 74e91f2e: FR-322 lines 4417-4430, 4518-4538, 4615-4630, FR-322-AC-359, FR-322-AC-503, FR-778, FR-370 lines 213-225, schema.json TemporalBinding.source_unit, temporal-clock-binding-vectors.json, timed-clock-no-row-vectors.json"
review_set: subset
---

## Summary

Ticket: QSL-2338. This review covers the IR spec edits and the FSpec PR #777 text the code consumes.

Examined:

- IR FR-038-AC-590. It is new; 589 is the highest number on main. No fetched IR ref carries an AC-590 (259 refs searched). FR-038 is this repository's requirement, so AC-590 belongs here and not in FSpec. The FSpec side is FR-322-AC-503 and FR-778-AC-8..11, which AC-590 cites.
- IR FR-019, the temporal clock bindings row: the new member is recorded.
- FSpec FR-322 "Clause source unit": the rule, the row-visit order, and the refusal loci.
- FR-322-AC-359 and AC-503.
- FR-778 Description and Behavior, and FR-370's clock-unit paragraph.
- `schema.json` TemporalBinding: `source_unit` is a `DefinitionRef`, optional at the schema level and required by a reader rule.
- The published vector files' rows, counted per file:
  - in `temporal-clock-binding-vectors.json`, every clause row except the case-level TCB positives carries `source_unit`;
  - `temporal-clock-occurrence-vectors.json` and `temporal-clause-name-vectors.json` carry it on every row;
  - every positive fixture carries it on its clause row;
  - `timed-clock-no-row-vectors.json` carries none.

`quire validate` of FR-038 and FR-019 reports only pre-existing grammar warnings.

## Verdict

**Changes needed, two of them FSpec-side.** AC-590 is numbered correctly, belongs in this repository, and states the breaking change. The FSpec text leaves rows of one clause naming different units undefined. The TN vectors also miss the new member. On the IR side, AC-590 cites a TC this repository does not declare.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FSpec gap. FR-322 says each clause row names "the source unit that declares the clause", and a reader resolves "every other unit name the clause owns, in that unit". Structurally equal clauses share one node, and each occurrence has its own row, so two rows of one node can name two units. FR-322 and FR-778 do not say whether those rows must agree, nor in which unit the node's aggregate bounds (one per node) resolve. The IR picks the first row (code-review FND-001), which is not a rule any criterion states. FSpec should rule, either "rows of one clause node name one `source_unit`; otherwise X at row i", or "bounds must match every row's clock unit", with a vector | FSpec spec/objects/interfaces/FR-322-checked-package-artifact.md:4518-4538, FSpec spec/functional/temporal/FR-778-admit-a-source-declared-time-unit-as-a-timestamped-clause-s-clock-unit.md |
| FND-002 | medium | FSpec PR #777 vector gap. The rows of `timed-clock-no-row-vectors.json` for TN-04 and TN-06 to TN-09 are rows keyed by a `temporal_clause`, and none carries `source_unit`. That contradicts FR-322-AC-503 and the README line "carried by every clause row of every published fixture and vector". Read as published, TN-06 (expected `admitted`) refuses `invalid_package`/`timed-node-reference` at `/temporal_bindings/0`. TN-08 and TN-09 now expect that same code at that same locus, so a reader that only checks the missing member passes them. The IR harness hides the gap by stamping the member (gap-analysis FND-005) | FSpec proposals/checked-package-v2/timed-clock-no-row-vectors.json |
| FND-003 | low | FR-038-AC-590's Verification is "Test (TC-1890; ...)", but TC-1890 is an FSpec test case that this repository does not declare. Its strict-reader criteria use TC-048 (368 ACs), and SR-17940486709564847048081 FND-183 already flagged TC-1890 as an untracked id here. Cite TC-048, with the FSpec TC-1890 as a reference in the text | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6373 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | FR-038-AC-590 now states the one-unit rule, but it still cites "FR-778-AC-1..AC-11". FSpec #777 at c01d61ce numbers that rule FR-778-AC-12 (SU-05). Cite AC-12 once the pinned FSpec head includes it | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6373 |

## Dispositions

Round 1, at head 16bc5fe6f, checked against FSpec #777 at c01d61ce.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FSpec FR-322:4575-4581 (c01d61ce): "every `temporal_bindings` row of one clause node SHALL name the same `source_unit`. In the same row visit, after its lock join, a clause row whose `source_unit` differs from that of the clause node's first row ... refuses `invalid_package`/`timed-node-reference` at that row". Also FR-778-AC-12 and SU-05 |
| FND-002 | fixed | At c01d61ce, `timed-clock-no-row-vectors.json` TN-04 and TN-06..TN-09 rows carry `"source_unit":{"authority":"agent-ix","identity":"example"}` |
| FND-003 | fixed | FR-038-AC-590 Verification: "Test (TC-048; FSpec FR-322-AC-359, AC-503, FR-778-AC-1..AC-11)" |

Round 2, at head f90109938.

| FND | outcome | reason |
| --- | --- | --- |
| FND-004 | fixed | FR-038-AC-590: "Consumes FSpec FR-322 \"Clause source unit\" (FR-322-AC-359, AC-503) and FR-778-AC-1..AC-12" and Verification "Test (TC-048; FSpec FR-322-AC-359, AC-503, FR-778-AC-1..AC-12)" |
