---
id: SR-2026112112042500000003
title: "spec-review base of QSL-2718 FR-060 model_root binding edit"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1115 at f4c8c2b5; spec/checked_package/functional/FR-060-admit-model-terminal-declarations.md"
review_set: subset
---

## Summary

Ticket: QSL-2718. PR #1115 edits FR-060's Behavior refusal sentence and FR-060-AC-2. The base spec review checks both edits against FSpec FR-476 at filament-specification origin/main: Reader order step 4, "Several terminal nodes on one model node" and AC-4. The requirements already own the behavior, so this review proposes no redesign.

## Examined scope

- FR-060 Behavior, the edited sentence (examined): "A `model` target that is not a `model`/`model_root` node, a `model`/`object_type` node included, refuses `invalid_model_binding`/`malformed-declaration` at the target". Its meaning agrees with FR-476 step 4 and AC-4, and it adds the locus, which matches FR-476's "The locus is the target".
- FR-060 Behavior, the binding sentence (examined): "`model` references the selected model node." See FND-001.
- FR-060-AC-2, the edited cell (examined). It agrees with FR-476-AC-4 and adds the `with and without self` and root read-back clauses. Tests can falsify it, and the code meets it.
- The edits add no requirement, ticket, pin or hash (examined).

## Verdict

The edits are correct against the oracle and the PR's code meets them. FND-001 is a low wording finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-060 Behavior still says "`model` references the selected model node", which is less exact than the edited refusal sentence and than FSpec FR-476 ("the selection's `model`/`model_root` node"). The appositive "a `model`/`object_type` node included" also reads awkwardly. The binding sentence could name the `model_root` node directly, so the refusal clause would need no appositive. | spec/checked_package/functional/FR-060-admit-model-terminal-declarations.md:45-46, spec/checked_package/functional/FR-060-admit-model-terminal-declarations.md:72-74 |
