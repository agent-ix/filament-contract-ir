---
id: SR-17940486709564847048099
title: "gap-analysis of QSL-2635 reader join stage 5 against FR-038-AC-582 and FSpec FR-322-AC-485/486"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1083 at 3585ced2 (merge base 312f3752); FR-038-AC-582, TC-048, spec/checked_package/matrix/tests.md, crates/filament-contract-model/src/checked_package/v2/model_occurrence.rs, tests/conformance_fspec/model_tuple_union.rs, tests/it/checked_package_v2_whole_read_accounting.rs"
review_set: subset
---

## Summary

This is a planless gap analysis of PR #1083 (QSL-2635, QSL-2619): spec, matrix, tags, tests and code. Plan completion: not assessed.

`quoin matrix --json` on the PR head lists FR-038-AC-582 as `static_status: tagged`. It is bound to `published_model_tuple_union_wire_cases_match_reader_refusals` and to `model_declaration_occurrence_stage_five_orders_and_locates_each_defect`. Its `evidence_status` is "no run evidence", which is the same status as its neighbours. TC-048 gains the `verifies` edge to AC-582, a section naming both tests, and a matrix row. FSpec FR-322-AC-485 and AC-486 are FSpec's TC-2085 and are replayed here by reference.

Tests run (focused, against FSpec 45b5e1b6): the 13 `model_tuple_union` tests pass. The 57 focused `it` tests pass, including `whole_read_accounting`, the TC-044 and TC-048 source-map cases and the owners suite. The 25 `clock_payload_tuple`/`extension` conformance tests also pass. Mutants were not re-run, so the coder's 4 survivors are judged by inspection.

## Examined scope

- FR-038-AC-582 (examined). Covered: TW-29 to TW-34 through the published replay, whose presence check now runs to TW-34 (TW-00 to TW-34, except TW-05). Also covered, in the new test: the repeated generated entry, a missing root entry, an entry defect beside a missing entry, a root entry of another role, and two missing entries. Not covered: a `relation` declaration node (FND-001) and the charging sentence (FND-002).
- FSpec FR-322-AC-486 (examined). Every TW row's code, cause, locus and `stage` is matched by the replay.
- FSpec FR-322-AC-485 (examined). The reader admits every published positive package. The two positives the AC names, `positive-model-tuple-union.json` and `positive-all-families.json`, are read by existing admitted tests.
- `whole_read_accounting` registry (examined). It registers every `tuple-union/wire` row, so TW-29 to TW-34 are accounted within the bound. This was the red the ticket names.
- matrix/tests.md FR-038-AC-582 row and TC-048 section (examined). The ids and test names are accurate.

## Verdict

The published vectors are fully replayed and the extra ordering cases are well chosen. The new test pins the precedence of the entry pass over the missing pass, the first-missing choice, and the root-of-another-role case.

Two parts of AC-582 have no test that could fail. The first is the `relation` half of "a `model` or `relation` node". A reviewer probe on `positive-all-families.json` `/source_map/6` shows the code is right today, so the missing case costs only a few lines. This matches the surviving mutant the coder reported for the relation-node case: yes, it should have a test. The second is the exact charging sentence. Neither is a correctness defect today. Both are medium coverage gaps.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-582 defines a model declaration node as "a `model` or `relation` node carrying a `ModelOwner`", but every stage-5 vector and case uses `model` nodes of `positive-model-tuple-union.json`. Dropping `CheckedNodeKind::Relation(_)` from `is_declaration` survives. Add cases over `positive-all-families.json`'s relationship `Order-order-Order` (entry `/source_map/6`): own region and role `expression` refuse at `/source_map/6`, and removing the entry refuses at `/source_map` (reviewer probe P4 confirms each outcome) | crates/filament-contract-model/src/checked_package/v2/model_occurrence.rs:22-27 |
| FND-002 | medium | FR-038-AC-582 states the pass charges one work unit per model root entry, per declaration entry and per declaration node checked, and holds exactly two exact-capacity tables. No test pins a stage-5 work count, or a work or storage denial inside stage 5. The accounting registry only checks that the TW reads stay within the bound, so changing or removing any of the three `stage_storage::work` charges would survive. Add an exact-work boundary case (least admitting budget, with a denial located in stage 5 one unit short), as other stages do, or drop the exact-charge wording from the AC | tests/conformance_fspec/model_tuple_union.rs:588-690 |
