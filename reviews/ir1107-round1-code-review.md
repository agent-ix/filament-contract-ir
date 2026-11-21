---
id: SR-17952567272131
title: "QSL-2717 PR 1107 disposition round 1 — code-review"
type: SpecReview
analysis: code-review
scope: "QSL-2717; protocol_body.rs; v2/mod.rs; tests/it/checked_package_v2_protocol_body.rs; FR-038-AC-596; TC-048"
review_set: subset
---

## Summary

Disposition of original SR-17940486709564847048138 (code-review), cached comment 15d0d6c7-bfe7-444a-af23-29a669b6d15d. All 2 original findings are fixed.

## Verdict

**PASS** — CLEAR for this bounded disposition; original findings remain immutable historical records, and every current outcome is fixed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Unit-only validate_protocol_body still orders sets by wire id; two definitions of a canonical set | Original SR-17940486709564847048138, comment 15d0d6c7-bfe7-444a-af23-29a669b6d15d |
| FND-002 | low | Fairness pre-pass change only reorders refusals (cannot admit), but is untested and unrecorded | Original SR-17940486709564847048138, comment 15d0d6c7-bfe7-444a-af23-29a669b6d15d |

## Dispositions

Round 1.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Removed reference_set_is_ordered and the graph-free set check. The view now explicitly requires the graph-aware reader for structural order; the renamed shape unit test accepts reversed references. Both production protocol branches still run the graph-aware check. |
| FND-002 | fixed | AC-596 precisely records envelope/layout-only temporal prevalidation and the later canonical-set refusal. The combined-defect vector checks clause InvalidSemanticGraph with a valid fairness target and earlier MissingDeclaration/MissingName at an absent fairness branch. Both refuse; no admission bypass. |

```yaml
{
  "original_artifact_id": "SR-17940486709564847048138",
  "original_comment_id": "15d0d6c7-bfe7-444a-af23-29a669b6d15d",
  "original_review_head": "c68a838ec18a91f7d5bc84667928ecbb4f565a2d",
  "reviewed_head": "d695425f1ee05826ff9c930a989e24594df3c3da",
  "method": "code-review",
  "dispositions": [
    {
      "fnd": "FND-001",
      "outcome": "fixed",
      "after_excerpt": "/// This has no graph context; target forms, binder scope and membership are\n/// checked by the protocol reader step. Reference-set order also requires\n/// resolved structural keys and is checked only at that graph-aware boundary.",
      "reason": "Removed reference_set_is_ordered and the graph-free set check. The view now explicitly requires the graph-aware reader for structural order; the renamed shape unit test accepts reversed references. Both production protocol branches still run the graph-aware check."
    },
    {
      "fnd": "FND-002",
      "outcome": "fixed",
      "after_excerpt": "assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);\n    assert_eq!(refusal.cause, Some(CheckedPackageRefusalCause::MissingName));\n    assert!(matches!(refusal.protocol_detail.as_deref(),\n        Some(CheckedProtocolRefusalDetail::BinderScope { stage, read, .. })\n        if stage.as_ref() == \"run\" && read.as_ref() == \"stage\"));",
      "reason": "AC-596 precisely records envelope/layout-only temporal prevalidation and the later canonical-set refusal. The combined-defect vector checks clause InvalidSemanticGraph with a valid fairness target and earlier MissingDeclaration/MissingName at an absent fairness branch. Both refuse; no admission bypass."
    }
  ]
}
```

## Coverage

Rust lane checked the bounded fix and callers: GraphIndex::resolved uses computed structural ranks; ResolvedKey::order compares local ranks without allocation and otherwise uses WorkMeter::compare_ids. The implementation propagates ValidationFailure with ?, keeps strict less-than duplicate refusal, and validates both control and clause sets. validate_temporal precedes validate_protocol_nodes; the latter remains mandatory. No wire/API, dependency or gate changes.

Verified frozen head d695425f1ee05826ff9c930a989e24594df3c3da against base e48c3dfc3f8c8806e570e082aa35d474fcdc833e. Original review head c68a838ec18a91f7d5bc84667928ecbb4f565a2d. Round 1 only; all seven original findings checked, no new findings. Original SR files were unavailable: immutable rows and full original records below are recovered from /private/tmp/ir-1107-ticket.json .comments.nodes. Fresh artifact IDs preserve the original-ID mapping.

Inspected existing actual results, without reviewer Cargo or broad-gate reruns: /private/tmp/ir1107-integrated-reference-sets-green.log (5 passed), /private/tmp/ir1107-integrated-model-unit-green.log (1 passed), /private/tmp/ir1107-clippy-model.log and ir1107-clippy-it.log (scoped clean), ir1107-fmt-check.log, ir1107-integrated-format.log, ir1107-quire-validation.log. Three red logs ir1107-mutant-old-wire.log, ir1107-mutant-array-position.log, ir1107-mutant-skip-choice-repeat.log show intended assertion failures, exit 101, not infrastructure failures. Production source restored. Direct git diff against de58348f shows identical protocol/test bytes; cmp of ir1107-entry-old.patch and ir1107-entry-new.patch succeeds. No source edits, worktree creation, push, merge or target deletion.

Full exact command/environment records and integration equivalence: /private/tmp/ir1107-fixround1-handoff.md and /private/tmp/ir1107-final-integration-handoff.md. Root focused reader proofs only; this is not the aggregate workspace gate. FSL live SB producer replay remains outside this IR round and is not claimed as verified. No applicable AssuranceProfile was identified in this bounded disposition context; no enforced spec-review run is claimed.

## Original review record (verbatim)

The following immutable source preserves all original finding rows, full finding text, excerpts, method, marker, and comment metadata.

````markdown
<!-- reviewer repo=agent-ix/filament-contract-ir visibility=public quoin=0.29.1 module=spec-artifacts-process@b82e42e893612b4b4ee94127714b4577ee893502 id=SR-17940486709564847048138 method=code-review lang=rust pr=filament-contract-ir#1107 model=claude-opus-5-5 run=6626ffe7-f5dd-4f43-b60d-e649e8e24fff -->

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | low | other | Unit-only validate_protocol_body still orders sets by wire id; two definitions of a canonical set |
| FND-002 | low | other | Fairness pre-pass change only reorders refusals (cannot admit), but is untested and unrecorded |

+++ [reviewer data]

```yaml
clean: false
scope:
- id: protocol_body.rs::reference_sets_are_ranked
  path: crates/filament-contract-model/src/checked_package/v2/protocol_body.rs
  role: examined
  excerpt: "fn reference_sets_are_ranked(\n    form: ProtocolForm,\n    operator: &str,\n    arguments: &[CheckedTerm],\n    graph: &SetGraph<'_, '_>,\n    meter: &mut WorkMeter,\n) -> Result<bool, ValidationFailure> {\n    for position in set_positions(form, operator) {\n        ...\n            let key = graph.index.resolved(target, graph.structural_keys);\n            if let Some(held) = &previous {\n                if held.order(&key, meter)? != std::cmp::Ordering::Less {\n                    return Ok(false);"
- id: protocol_body.rs::set_positions
  path: crates/filament-contract-model/src/checked_package/v2/protocol_body.rs
  role: examined
  excerpt: "fn set_positions(form: ProtocolForm, operator: &str) -> std::ops::Range<usize> {\n    match (form, operator) {\n        (ProtocolForm::ProtocolClause, _) => 3..9,\n        (ProtocolForm::Control, \"choice\" | \"repeat\") => 1..2,\n        (ProtocolForm::Control, \"attempt\") => 3..4,\n        (ProtocolForm::Control, \"recommend\") => 2..3,\n        _ => 0..0,\n    }\n}"
- id: protocol_body.rs::validate_protocol_body
  path: crates/filament-contract-model/src/checked_package/v2/protocol_body.rs
  role: examined
  excerpt: "    validate_layout(form, operator, arguments)?;\n    if set_positions(form, operator).any(|position| {\n        argument(arguments, position).is_none_or(|set| !reference_set_is_ordered(set))\n    }) {\n        return Err(ProtocolBodyDefect::NoncanonicalSet);\n    }"
- id: protocol_body.rs::validate_temporal_fairness
  path: crates/filament-contract-model/src/checked_package/v2/protocol_body.rs
  role: examined
  excerpt: "        if let Ok((operator, arguments)) = validate_envelope(\n            form,\n            term,\n            &graph.nodes[position].semantic_type,\n            selected_profile,\n        ) {\n            if validate_layout(form, operator, arguments).is_ok() {\n                validated[position] = Some((operator, arguments));\n            }\n        }"
- id: mod.rs::validate_graph dependencies order
  path: crates/filament-contract-model/src/checked_package/v2/mod.rs
  role: context_only
  excerpt: expected.extend(targets.map(|(target, _)| index.resolved(target, structural_keys))); expected.sort_unstable(); expected.dedup();
- id: FR-038-AC-596
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: 'Implemented (QSL-2717). Consumes FSpec FR-464 "Canonical order" for the reference sets of the protocol node family (a clause''s `requirements`, `roles`, `channels`, `queues` and the other sets, a `choice` or `repeat` `children`, an `attempt` or `recommend` `contracts`). The strict reader orders a reference set by the resolved graph rank of each target, as it orders a node''s `dependencies` and a `related` record''s `relationship`, not by the wire id the set holds: the wire ids of a graph hold graph indices and a node''s operand slots are numbered by first appearance in its own bytes, so a producer'
findings:
- fnd: FND-001
  severity: low
  confidence: high
  method: code-review
  check: other
  artifact_id: protocol_body.rs::validate_protocol_body
  related_ids:
  - FR-038-AC-596
  path: crates/filament-contract-model/src/checked_package/v2/protocol_body.rs
  lines: 1413-1417
  excerpt: "    validate_layout(form, operator, arguments)?;\n    if set_positions(form, operator).any(|position| {\n        argument(arguments, position).is_none_or(|set| !reference_set_is_ordered(set))\n    }) {\n        return Err(ProtocolBodyDefect::NoncanonicalSet);\n    }"
  finding: 'validate_protocol_body (the graph-free unit-test view, #[allow(dead_code)]) still checks set order with the wire-id reference_set_is_ordered, so the crate now has two definitions of a canonical set and the unit view keeps the order the reader dropped. Remove the view''s set check or document it as a wire-only approximation.'
- fnd: FND-002
  severity: low
  confidence: high
  method: code-review
  check: other
  artifact_id: protocol_body.rs::validate_temporal_fairness
  related_ids:
  - FR-038-AC-596
  path: crates/filament-contract-model/src/checked_package/v2/protocol_body.rs
  lines: '3755'
  excerpt: "        if let Ok((operator, arguments)) = validate_envelope(\n            form,\n            term,\n            &graph.nodes[position].semantic_type,\n            selected_profile,\n        ) {\n            if validate_layout(form, operator, arguments).is_ok() {\n                validated[position] = Some((operator, arguments));\n            }\n        }"
  finding: The validate_temporal_fairness pre-pass no longer applies the set check. It cannot admit anything (validate_protocol_nodes at mod.rs:4477 always runs after validate_temporal at mod.rs:4291 and refuses), but it changes the first refusal for a non-canonical clause set that a fairness target depends on, from a temporal fairness defect to invalid_semantic_graph at the clause. Nothing tests or records this; add an AC-596 note and a vector, or apply the ranked check in the pre-pass.
bindings: []
```

+++
````
