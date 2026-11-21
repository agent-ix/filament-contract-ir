---
id: SR-17952567272132
title: "QSL-2717 PR 1107 disposition round 1 — gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "QSL-2717; protocol_body.rs; v2/mod.rs; tests/it/checked_package_v2_protocol_body.rs; FR-038-AC-596; TC-048"
review_set: subset
---

## Summary

Disposition of original SR-17940486709564847048139 (gap-analysis), cached comment 8e73bea9-ef95-48bb-b553-2f4c4cf98ef4. All 3 original findings are fixed.

## Verdict

**PASS** — CLEAR for this bounded disposition; original findings remain immutable historical records, and every current outcome is fixed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-596 test passes on the old wire-id reader (flat ids); an orders-differ fixture is feasible and required | Original SR-17940486709564847048139, comment 8e73bea9-ef95-48bb-b553-2f4c4cf98ef4 |
| FND-002 | medium | No reversed-graph (FR-322-AC-95) test; array-position mutant passes the AC-596 test | Original SR-17940486709564847048139, comment 8e73bea9-ef95-48bb-b553-2f4c4cf98ef4 |
| FND-003 | low | choice/repeat visible set order untested; mutant skipping it survives | Original SR-17940486709564847048139, comment 8e73bea9-ef95-48bb-b553-2f4c4cf98ef4 |

## Dispositions

Round 1.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The public-reader choice/repeat fixture authors operand-bearing g(high,low) and g(high,high) structural IDs. Admitted wire targets descend while held resolved keys ascend. The old wire comparator mutant refuses the primary choice at node 29, so this test distinguishes the fix from the old implementation. |
| FND-002 | fixed | The dedicated three-contract reversed-graph admission and operand-bearing reversed choice/repeat admissions exercise graph-position independence. Reversal operates on resolved-index fixture form, remaps local node segments and preserves foreign package references; canonical() rebuilds per-node operand rows. The array-position mutant refuses the reversed attempt. |
| FND-003 | fixed | Both choice and repeat exercise swapped visible members and duplicate targets with exact InvalidSemanticGraph node loci. The skipped-choice/repeat-check mutant admits the swapped choice and fails the assertion. |

```yaml
{
  "original_artifact_id": "SR-17940486709564847048139",
  "original_comment_id": "8e73bea9-ef95-48bb-b553-2f4c4cf98ef4",
  "original_review_head": "c68a838ec18a91f7d5bc84667928ecbb4f565a2d",
  "reviewed_head": "d695425f1ee05826ff9c930a989e24594df3c3da",
  "method": "gap-analysis",
  "dispositions": [
    {
      "fnd": "FND-001",
      "outcome": "fixed",
      "after_excerpt": "assert!(\n            first > second,\n            \"wire operand slots must disagree with structural order: {first:?}, {second:?}\"\n        );",
      "reason": "The public-reader choice/repeat fixture authors operand-bearing g(high,low) and g(high,high) structural IDs. Admitted wire targets descend while held resolved keys ascend. The old wire comparator mutant refuses the primary choice at node 29, so this test distinguishes the fix from the old implementation."
    },
    {
      "fnd": "FND-002",
      "outcome": "fixed",
      "after_excerpt": "fn reference_sets_admit_a_reversed_graph() {\n    let (mut package, _, _) = attempt_with_three_contracts([0, 1, 2]);\n    reverse_reference_set_graph(&mut package);\n    assert!(\n        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),\n        \"{:?}\",\n        read(&package)\n    );\n}",
      "reason": "The dedicated three-contract reversed-graph admission and operand-bearing reversed choice/repeat admissions exercise graph-position independence. Reversal operates on resolved-index fixture form, remaps local node segments and preserves foreign package references; canonical() rebuilds per-node operand rows. The array-position mutant refuses the reversed attempt."
    },
    {
      "fnd": "FND-003",
      "outcome": "fixed",
      "after_excerpt": "for operator in [\"choice\", \"repeat\"] {\n        let (package, holder) = operand_bearing_visible_set(operator);\n        for duplicate in [false, true] {\n            let mut wrong = package.clone();\n            let members = wrong[\"semantic_graph\"][\"nodes\"][holder][\"body\"][\"arguments\"][1][\"value\"]\n                [\"members\"]\n                .as_array_mut()\n                .expect(\"visible\");\n            if duplicate {\n                members[1] = members[0].clone();\n            } else {\n                members.swap(0, 1);\n            }",
      "reason": "Both choice and repeat exercise swapped visible members and duplicate targets with exact InvalidSemanticGraph node loci. The skipped-choice/repeat-check mutant admits the swapped choice and fails the assertion."
    }
  ]
}
```

## Coverage

Plan completion: not assessed

Disposition scope only: the three existing gap findings and their FR-038-AC-596 trace-tagged public tests were examined. No new repository-wide matrix audit or baseline campaign is claimed. The read wrapper calls canonical -> read_and_account with real evidence and bounded limits. Fixture canonical writes operand rows; reversal never treats node-local wire slots as graph indexes.

Verified frozen head d695425f1ee05826ff9c930a989e24594df3c3da against base e48c3dfc3f8c8806e570e082aa35d474fcdc833e. Original review head c68a838ec18a91f7d5bc84667928ecbb4f565a2d. Round 1 only; all seven original findings checked, no new findings. Original SR files were unavailable: immutable rows and full original records below are recovered from /private/tmp/ir-1107-ticket.json .comments.nodes. Fresh artifact IDs preserve the original-ID mapping.

Inspected existing actual results, without reviewer Cargo or broad-gate reruns: /private/tmp/ir1107-integrated-reference-sets-green.log (5 passed), /private/tmp/ir1107-integrated-model-unit-green.log (1 passed), /private/tmp/ir1107-clippy-model.log and ir1107-clippy-it.log (scoped clean), ir1107-fmt-check.log, ir1107-integrated-format.log, ir1107-quire-validation.log. Three red logs ir1107-mutant-old-wire.log, ir1107-mutant-array-position.log, ir1107-mutant-skip-choice-repeat.log show intended assertion failures, exit 101, not infrastructure failures. Production source restored. Direct git diff against de58348f shows identical protocol/test bytes; cmp of ir1107-entry-old.patch and ir1107-entry-new.patch succeeds. No source edits, worktree creation, push, merge or target deletion.

Full exact command/environment records and integration equivalence: /private/tmp/ir1107-fixround1-handoff.md and /private/tmp/ir1107-final-integration-handoff.md. Root focused reader proofs only; this is not the aggregate workspace gate. FSL live SB producer replay remains outside this IR round and is not claimed as verified. No applicable AssuranceProfile was identified in this bounded disposition context; no enforced spec-review run is claimed.

## Original review record (verbatim)

The following immutable source preserves all original finding rows, full finding text, excerpts, method, marker, and comment metadata.

````markdown
<!-- reviewer repo=agent-ix/filament-contract-ir visibility=public quoin=0.29.1 module=spec-artifacts-process@b82e42e893612b4b4ee94127714b4577ee893502 id=SR-17940486709564847048139 method=gap-analysis lang=rust pr=filament-contract-ir#1107 model=claude-opus-5-5 run=6626ffe7-f5dd-4f43-b60d-e649e8e24fff -->

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | medium | test-intent | AC-596 test passes on the old wire-id reader (flat ids); an orders-differ fixture is feasible and required |
| FND-002 | medium | coverage | No reversed-graph (FR-322-AC-95) test; array-position mutant passes the AC-596 test |
| FND-003 | low | coverage | choice/repeat visible set order untested; mutant skipping it survives |

+++ [reviewer data]

```yaml
clean: false
scope:
- id: FR-038-AC-596
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Implemented (QSL-2717). Consumes FSpec FR-464 "Canonical order" for the reference sets of the protocol node family (a clause''s `requirements`, `roles`, `channels`, `queues` and the other sets, a `choice` or `repeat` `children`, an `attempt` or `recommend` `contracts`). The strict reader orders a reference set by the resolved graph rank of each target, as it orders a node''s `dependencies` and a `related` record''s `relationship`, not by the wire id the set holds: the wire ids of a graph hold graph indices and a node''s operand slots are numbered by first appearance in its own bytes, so a producer'
- id: checked_package_v2_protocol_body::reference_sets_compare_by_resolved_graph_rank
  path: tests/it/checked_package_v2_protocol_body.rs
  role: examined
  excerpt: "fn reference_sets_compare_by_resolved_graph_rank() {\n    let (package, root, _) = attempt_with_three_contracts([0, 1, 2]);\n    assert!(\n        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),\n    ...\n    for (what, order) in [\n        (\"a set out of rank order\", [0, 2, 1]),\n        (\"a reversed set\", [2, 1, 0]),\n        (\"a repeated target\", [0, 1, 1]),\n        (\"a repeated first target\", [0, 0, 1]),\n    ] {"
findings:
- fnd: FND-001
  severity: medium
  confidence: high
  method: gap-analysis
  check: test-intent
  artifact_id: FR-038-AC-596
  related_ids:
  - checked_package_v2_protocol_body::reference_sets_compare_by_resolved_graph_rank
  path: tests/it/checked_package_v2_protocol_body.rs
  lines: 15738-15808
  excerpt: 'Implemented (QSL-2717). Consumes FSpec FR-464 "Canonical order" for the reference sets of the protocol node family (a clause''s `requirements`, `roles`, `channels`, `queues` and the other sets, a `choice` or `repeat` `children`, an `attempt` or `recommend` `contracts`). The strict reader orders a reference set by the resolved graph rank of each target, as it orders a node''s `dependencies` and a `related` record''s `relationship`, not by the wire id the set holds: the wire ids of a graph hold graph indices and a node''s operand slots are numbered by first appearance in its own bytes, so a producer'
  finding: 'No test fails on the old reader. Mutant ''old'' (reader reverted to wire-id reference_set_is_ordered) passes all 225 checked_package_v2_protocol_body:: tests including the AC-596 test, because the three contract ids are flat (no operand segment, empty operand row). A fixture where the orders differ is feasible: an in-crate unit test of reference_sets_are_ranked over a GraphIndex whose structural_keys order differs from the wire ids, or an it-harness package with {"node": i} id segments (as in checked_package_v2_guarantee_composition.rs:1406) set after an earlier operand that takes slot 0 (operand-keyed requirements after a captures initializer naming the higher-ranked operand, or visible members g(a,b), g(a,a) with key(b) < key(a)). Add at least one key-order-admitted test that fails on the old reader.'
- fnd: FND-002
  severity: medium
  confidence: high
  method: gap-analysis
  check: coverage
  artifact_id: FR-038-AC-596
  related_ids:
  - FR-322-AC-95
  path: tests/it/checked_package_v2_protocol_body.rs
  lines: 15738-15808
  excerpt: 'Implemented (QSL-2717). Consumes FSpec FR-464 "Canonical order" for the reference sets of the protocol node family (a clause''s `requirements`, `roles`, `channels`, `queues` and the other sets, a `choice` or `repeat` `children`, an `attempt` or `recommend` `contracts`). The strict reader orders a reference set by the resolved graph rank of each target, as it orders a node''s `dependencies` and a `related` record''s `relationship`, not by the wire id the set holds: the wire ids of a graph hold graph indices and a node''s operand slots are numbered by first appearance in its own bytes, so a producer'
  finding: No AC-596 test shows the order is the structural key and not graph array position. Mutant 'pos' (array position) passes the AC-596 test and is caught only incidentally by four older tests. FR-322-AC-95 admits a reversed graph; add the reversed-graph variant of the three-contract attempt (reverse nodes, remap index segments, settle, admitted). A reviewer probe of about 25 lines passes on this head.
- fnd: FND-003
  severity: low
  confidence: high
  method: gap-analysis
  check: coverage
  artifact_id: FR-038-AC-596
  related_ids:
  - FSpec FR-466 choice/repeat visible
  path: crates/filament-contract-model/src/checked_package/v2/protocol_body.rs
  lines: '1138'
  excerpt: 'Implemented (QSL-2717). Consumes FSpec FR-464 "Canonical order" for the reference sets of the protocol node family (a clause''s `requirements`, `roles`, `channels`, `queues` and the other sets, a `choice` or `repeat` `children`, an `attempt` or `recommend` `contracts`). The strict reader orders a reference set by the resolved graph rank of each target, as it orders a node''s `dependencies` and a `related` record''s `relationship`, not by the wire id the set holds: the wire ids of a graph hold graph indices and a node''s operand slots are numbered by first appearance in its own bytes, so a producer'
  finding: 'AC-596 lists the choice/repeat set (visible, position 1) as rank-ordered, but no checked_package_v2_protocol_body:: test checks its order: mutant ''nochoice'' (that set skipped) passes all 225 tests. Add an out-of-order and a repeated visible case.'
bindings:
- test_id: checked_package_v2_protocol_body::reference_sets_compare_by_resolved_graph_rank
  ac_id: FR-038-AC-596
  trace: correct
```

+++
````
