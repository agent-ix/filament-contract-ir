---
id: SR-17952567272133
title: "QSL-2717 PR 1107 disposition round 1 — spec-review/base"
type: SpecReview
analysis: base
scope: "QSL-2717; protocol_body.rs; v2/mod.rs; tests/it/checked_package_v2_protocol_body.rs; FR-038-AC-596; TC-048"
review_set: subset
---

## Summary

Disposition of original SR-17940486709564847048140 (spec-review/base), cached comment dd0f38fd-50c2-480a-a864-30551b069e04. All 2 original findings are fixed.

## Verdict

**PASS** — CLEAR for this bounded disposition; original findings remain immutable historical records, and every current outcome is fixed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | 'resolved graph rank' undefined; reads as array position; restate as structural-key node-id order (reversed graph admitted) | Original SR-17940486709564847048140, comment dd0f38fd-50c2-480a-a864-30551b069e04 |
| FND-002 | low | AC-596 claims related is ordered by rank; code compares related wire ids via compare_ids | Original SR-17940486709564847048140, comment dd0f38fd-50c2-480a-a864-30551b069e04 |

## Dispositions

Round 1.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AC-596 and SetGraph now define strictly ascending node-id order of resolved structural keys, explicitly independent of graph array positions and node-local operand slots. Reversed-graph admission names FR-322-AC-95. The normative FR-464 archive already states the same structural-key rule; no FSpec amendment is required. |
| FND-002 | fixed | The unsupported related-record comparison was removed from AC-596. The current sentence states the reference-set rule directly and does not claim compare_ids orders related records by rank. TC-048 wording follows structural keys. |

```yaml
{
  "original_artifact_id": "SR-17940486709564847048140",
  "original_comment_id": "dd0f38fd-50c2-480a-a864-30551b069e04",
  "original_review_head": "c68a838ec18a91f7d5bc84667928ecbb4f565a2d",
  "reviewed_head": "d695425f1ee05826ff9c930a989e24594df3c3da",
  "method": "spec-review/base",
  "dispositions": [
    {
      "fnd": "FND-001",
      "outcome": "fixed",
      "after_excerpt": "Canonical order is strictly ascending node-id order of the resolved structural keys of the targets, independent of graph array position (including a reversed graph, FR-322-AC-95) and node-local operand slots numbered by first use.",
      "reason": "AC-596 and SetGraph now define strictly ascending node-id order of resolved structural keys, explicitly independent of graph array positions and node-local operand slots. Reversed-graph admission names FR-322-AC-95. The normative FR-464 archive already states the same structural-key rule; no FSpec amendment is required."
    },
    {
      "fnd": "FND-002",
      "outcome": "fixed",
      "after_excerpt": "Consumes FSpec FR-464 \"Canonical order\" for protocol reference sets: clause requirements, roles, channels, queues and other reference sets, choice/repeat visible, and attempt/recommend contracts.",
      "reason": "The unsupported related-record comparison was removed from AC-596. The current sentence states the reference-set rule directly and does not claim compare_ids orders related records by rank. TC-048 wording follows structural keys."
    }
  ]
}
```

## Coverage

Base disposition covers AC-596, SetGraph docs, and the affected TC-048 description. FSpec FR-464 Canonical order was read at /private/tmp/ir1110-fspec-247a3941/spec/objects/interfaces/FR-464-checked-package-protocol-node-bodies.md:199-209. It defines structural-key node-id order and per-node operand slots. Additional analysis lenses were not rerun: this is disposition round 1 of the existing review, not a new first review.

Verified frozen head d695425f1ee05826ff9c930a989e24594df3c3da against base e48c3dfc3f8c8806e570e082aa35d474fcdc833e. Original review head c68a838ec18a91f7d5bc84667928ecbb4f565a2d. Round 1 only; all seven original findings checked, no new findings. Original SR files were unavailable: immutable rows and full original records below are recovered from /private/tmp/ir-1107-ticket.json .comments.nodes. Fresh artifact IDs preserve the original-ID mapping.

Inspected existing actual results, without reviewer Cargo or broad-gate reruns: /private/tmp/ir1107-integrated-reference-sets-green.log (5 passed), /private/tmp/ir1107-integrated-model-unit-green.log (1 passed), /private/tmp/ir1107-clippy-model.log and ir1107-clippy-it.log (scoped clean), ir1107-fmt-check.log, ir1107-integrated-format.log, ir1107-quire-validation.log. Three red logs ir1107-mutant-old-wire.log, ir1107-mutant-array-position.log, ir1107-mutant-skip-choice-repeat.log show intended assertion failures, exit 101, not infrastructure failures. Production source restored. Direct git diff against de58348f shows identical protocol/test bytes; cmp of ir1107-entry-old.patch and ir1107-entry-new.patch succeeds. No source edits, worktree creation, push, merge or target deletion.

Full exact command/environment records and integration equivalence: /private/tmp/ir1107-fixround1-handoff.md and /private/tmp/ir1107-final-integration-handoff.md. Root focused reader proofs only; this is not the aggregate workspace gate. FSL live SB producer replay remains outside this IR round and is not claimed as verified. No applicable AssuranceProfile was identified in this bounded disposition context; no enforced spec-review run is claimed.

## Original review record (verbatim)

The following immutable source preserves all original finding rows, full finding text, excerpts, method, marker, and comment metadata.

````markdown
<!-- reviewer repo=agent-ix/filament-contract-ir visibility=public quoin=0.29.1 module=spec-artifacts-process@b82e42e893612b4b4ee94127714b4577ee893502 id=SR-17940486709564847048140 method=spec-review/base lang=spec pr=filament-contract-ir#1107 model=claude-opus-5-5 run=6626ffe7-f5dd-4f43-b60d-e649e8e24fff -->

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | medium | ambiguous | 'resolved graph rank' undefined; reads as array position; restate as structural-key node-id order (reversed graph admitted) |
| FND-002 | low | soundness | AC-596 claims related is ordered by rank; code compares related wire ids via compare_ids |

+++ [reviewer data]

```yaml
clean: false
scope:
- id: FR-038-AC-596
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Implemented (QSL-2717). Consumes FSpec FR-464 "Canonical order" for the reference sets of the protocol node family (a clause''s `requirements`, `roles`, `channels`, `queues` and the other sets, a `choice` or `repeat` `children`, an `attempt` or `recommend` `contracts`). The strict reader orders a reference set by the resolved graph rank of each target, as it orders a node''s `dependencies` and a `related` record''s `relationship`, not by the wire id the set holds: the wire ids of a graph hold graph indices and a node''s operand slots are numbered by first appearance in its own bytes, so a producer'
- id: TC-048 AC-596 bullet
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: 'Reference sets by graph rank (AC-596): the `contracts` set of an `attempt` with three entries in resolved-rank order admits; a set out of rank order, and a repeated target, refuse `invalid_semantic_graph` at the node.'
- id: FSpec FR-464 Canonical order
  path: filament-specification spec/objects/interfaces/FR-464-checked-package-protocol-node-bodies.md
  role: context_only
  excerpt: 'The canonical order of a reference set is the resolved graph rank of its targets, not the order of the wire ids the set holds: a wire id holds a graph index and a node''s operand slots are numbered by first appearance in its own bytes, so a producer cannot know the wire-id order when it builds the set, and the reader compares ranks, as it does for `related`.'
findings:
- fnd: FND-001
  severity: medium
  confidence: high
  method: spec-review/base
  check: ambiguous
  artifact_id: FR-038-AC-596
  related_ids:
  - FSpec FR-464
  - FSpec FR-322-AC-95
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  lines: '6374'
  excerpt: 'Implemented (QSL-2717). Consumes FSpec FR-464 "Canonical order" for the reference sets of the protocol node family (a clause''s `requirements`, `roles`, `channels`, `queues` and the other sets, a `choice` or `repeat` `children`, an `attempt` or `recommend` `contracts`). The strict reader orders a reference set by the resolved graph rank of each target, as it orders a node''s `dependencies` and a `related` record''s `relationship`, not by the wire id the set holds: the wire ids of a graph hold graph indices and a node''s operand slots are numbered by first appearance in its own bytes, so a producer'
  finding: 'AC-596 orders a set by ''the resolved graph rank of each target'' and never defines rank. It reads naturally as graph array position (FSpec''s own sentence adds ''the order the graph lists its nodes in''), but the rule is strictly ascending node-id order of the structural keys the targets resolve to (FR-322 ''Rebuilding the key''), so the set stays admitted under any admitted node order, including a reversed graph (FR-322-AC-95). Restate AC-596 that way, as FSpec #778 is being reworded; the SetGraph doc comment should follow.'
- fnd: FND-002
  severity: low
  confidence: medium
  method: spec-review/base
  check: soundness
  artifact_id: FR-038-AC-596
  related_ids:
  - protocol_body.rs::related_records_are_ordered
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  lines: '6374'
  excerpt: 'Implemented (QSL-2717). Consumes FSpec FR-464 "Canonical order" for the reference sets of the protocol node family (a clause''s `requirements`, `roles`, `channels`, `queues` and the other sets, a `choice` or `repeat` `children`, an `attempt` or `recommend` `contracts`). The strict reader orders a reference set by the resolved graph rank of each target, as it orders a node''s `dependencies` and a `related` record''s `relationship`, not by the wire id the set holds: the wire ids of a graph hold graph indices and a node''s operand slots are numbered by first appearance in its own bytes, so a producer'
  finding: AC-596 says sets are ordered 'as it orders a node's dependencies and a related record's relationship'. dependencies do use GraphIndex::resolved (mod.rs:4225-4231), but related_records_are_ordered compares the wire ids with meter.compare_ids (protocol_body.rs:691-692). The two agree only because a relationship target is a declared node with an operand-free id. Drop the related half of the claim or state why they agree.
bindings: []
```

+++
````
