---
id: SR-270178054278704885503246294768428943306
duplicate_review_alias: "SR-17940486709564847048138"
title: "IR-1112 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; IR-1112 changed files; QSL-2728; FR-038, FR-019, TC-492"
review_set: subset
---

## Summary

Ticket: QSL-2728. Independent static review of frozen head fc51b389a1669a38a51a07d20072ea21c4cf4e97 against base 0e089f6ea5dea8d799cb0234ec9623e0073ffad4 and supplied normative FR-322. 2 concrete finding(s).

## Verdict

**FAIL** — Resolve the recorded findings in the original PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | New binder validation allocates paths, owned node loci and filtered dependencies without charging read storage before allocation. | crates/filament-contract-model/src/checked_package/v2/operations.rs:4658 |
| FND-002 | medium | The let-body binder-identity walk skips referenced value containers, so a different same-name/same-level parameter inside a tuple or record is admitted. | crates/filament-contract-model/src/checked_package/v2/operations.rs:4677 |

## Coverage

The new public field and constructor are documented, and serialization, reference visiting, replacement and ID mapping carry the binder. All 48 changed files were inspected, including mechanical destructuring adaptations. The explicit missing, unresolved, wrong-kind, misnamed, unlisted, stray and unread-narrow-binder paths match the supplied normative contract on their direct paths.

FND-001: check_let_binder builds application.body and node_pointer strings, clones a full JsonPointer for every argument/member, and constructs ValidationFailure::refused_at with node.clone without a diagnostic reservation. meter.push only charges the Vec block, not the owned paths passed as arguments. stray_binder_defect repeats the eager-path/direct-refusal pattern at operations.rs:4524-4573. structural.rs:864-869 collects/clones a filtered dependency vector without meter allocation or clone_id charges. These requests happen even on valid linked lets and can exceed the caller's read-storage allowance; a denial in meter.push also occurs after its item's path has already allocated. Use metered borrowed paths/parent edges and stage_storage refusal helpers, and metered reference filtering or charged ID clones. Add allocator-accounted exact/one-short admission and refusal tests over a linked let.

FND-002: operations.rs:4686 follows only Expression kinds. A valid let whose body references a value/tuple_value containing reference(y_other), with y_other a separate parameter of the linked binder's name and level, is not traversed. tuple_value_defect checks the member's type, not this identity. The same problem affects record/collection/union/option value terms. Follow expression and value term containers with a visited set, while retaining function/scope boundaries. Add a tuple or record negative vector and a correct-binder control. This conclusion follows the production control flow; no new runtime reproduction was claimed.

No CI gate is changed. CI fetches filament-specification ref main, so no stale fixture pin exists in this PR. Derived-path construction belongs to the producer; the reader preserves declared structural NodeRefs. The external normative revision was inspected for MemberTerm binder shape and published fixture evolution; no requirement to invent a new reader-side identity derivation was inferred.

## Additional evidence

Additional FND-001 boundedness evidence: check_let_binder starts a fresh pending/visited walk for every let (operations.rs:4658-4670), then follows the whole body expression closure (4686-4702). For a chain of N nested let nodes, the same suffixes are revisited N + (N-1) + ... + 1 times, at least quadratic; visited.contains additionally linearly scans that per-let list. The standard BL-D2 100000-nested-let case therefore cannot retain a linear graph walk merely by charging the existing implementation. Use a shared, scope-aware binder/reference index or a single scoped traversal and constant/logarithmic visited lookup, with metered storage, rather than preserving a whole-body scan per let. Verify through the existing required BL-D2 named filter after the fix, without adding broad performance gates.

## Examined units

```yaml
{
  "scope": [
    {
      "id": "FR-038-AC-586",
      "path": "spec/checked_package/functional/FR-038-consume-checked-package-v2.md",
      "role": "examined",
      "excerpt": "Integer containment at a function or expression result (FSpec FR-322 \"Integer containment\", FR-322-AC-495 to FR-322-AC-497 and FR-322-AC-499, vectors IC-01 to IC-04 and IC-08). `function c1 using v(x: Int[0, 9]): Int[0, 10] pure { x }`, whose `body` binding references `x`'s parameter node with no `convert` node, is admitted, as is the same function of result `Integer` and an `expression` node whose root application's `result_type` is contained by its `semantic_type`. A result `Int[0, 8]`, a root `Integer` at an `Int[0, 10]` result and `Sequence<Int[0, 9]>[0, 3]` at `Sequence<Int[0, 10]>[0, 3]`"
    },
    {
      "id": "FR-038-AC-587",
      "path": "spec/checked_package/functional/FR-038-consume-checked-package-v2.md",
      "role": "examined",
      "excerpt": "A call argument keeps the exact match (FR-322-AC-498, IC-05 to IC-07). `g(x)` over `g(a: Int[0, 10])` and `x: Int[0, 9]` is admitted when its argument is one `filament.op.numeric.convert` to `Int[0, 10]`; the bare `reference` to `x`, at an `Int[0, 10]`, `Int[0, 8]` or `Integer` parameter, refuses `ill_typed`/`operator-ineligible` at `body/arguments/1` of the call."
    },
    {
      "id": "FR-038-AC-588",
      "path": "spec/checked_package/functional/FR-038-consume-checked-package-v2.md",
      "role": "examined",
      "excerpt": "Integer containment at control results (FR-322-AC-500 and FR-322-AC-502, IC-09 to IC-13 and IC-16 to IC-18). A `filament.op.control.if` admits each of its two branches whose type node equals its `result_type`, is a conforming record or is integer-contained by it, so the hull-typed `if c then x else y` over `x: Int[0, 9]` and `y: Int[5, 20]` with `result_type` `Int[0, 20]` is admitted, and it refuses `ill_typed`/`operator-ineligible` at the `if` application a then-branch or an else-branch the `result_type` does not contain (`Int[0, 1000]` at `Int[0, 10]`). A `case` over `U { A(Int[0, 9]), B(Int"
    },
    {
      "id": "FR-038-AC-589",
      "path": "spec/checked_package/functional/FR-038-consume-checked-package-v2.md",
      "role": "examined",
      "excerpt": "Annotated `let` (FR-322-AC-501, IC-14, IC-15, IC-19, IC-20). `let y: Integer = x in g(y)` over `x: Int[0, 1000]`, whose binder `y` is typed by the builtin `Integer` node, and `let r: Money = make_taxed() in r` are admitted. A bare `Int[0, 1000]` binder passed to an `Integer` parameter refuses `ill_typed`/`operator-ineligible` at the call, not the `let`. The reader checks a `let` binder's `semantic_type` against the bound value through the binder link of FR-038-AC-595, so IC-15 (an `Int[0, 999]` binder over an `Int[0, 1000]` value) refuses `ill_typed`/`operator-ineligible` at the `let` applicat"
    },
    {
      "id": "FR-038-AC-595",
      "path": "spec/checked_package/functional/FR-038-consume-checked-package-v2.md",
      "role": "examined",
      "excerpt": "Implemented (QSL-2728). Consumes FSpec FR-322 \"The binder link\" (FR-322-AC-512, IC-14, IC-15, IC-21). The binding term of a `filament.op.control.let` application carries `binder`, the `NodeRef` of its binder's `value`/`parameter` node, and that node is among the application's `dependencies`. The typed `CheckedTerm` binding gains the optional `binder` member and the constructor binding_with_binder; the strict reader and canonical writer carry it. The reader refuses `invalid_package`/`invalid-value` for a `binder` absent from a `let` binding term (at `arguments/0`), for a `binder` that names no "
    },
    {
      "id": "FR-038-AC-504",
      "path": "spec/checked_package/functional/FR-038-consume-checked-package-v2.md",
      "role": "examined",
      "excerpt": "Graph stage. For genuine packages that together reach every node family validator (frame, temporal, protocol, timed model, hybrid, refinement, operation, extension, state, claim and value bodies) and every sidecar row check, a read-storage limit one below the cumulative read storage through the last request the graph stage makes for node `i` refuses at it with locus `PackageRead(Graph, i)` and the node's `node_tag` as form (or the sidecar member and row index for a sidecar row); the typed `CheckedTerm` of node `i`, the declaration index, the lowering facts, one family validator's temporary tab"
    },
    {
      "id": "FR-019",
      "path": "spec/model/functional/FR-019-rust-library-interface.md",
      "role": "examined",
      "excerpt": "types `ApplicationOperator`, `CheckedAbstractionField`, `CheckedAbstractionFrame`, `CheckedAbstractionObject`, `CheckedAbstractionParameter`, `CheckedAbstractionPopulation`, `CheckedFieldScope`, `CheckedFrameModification`, `CheckedScopedDeletion`, `CheckedTerm`, `LiteralKind`; the `CheckedTerm` binding variant gains the optional binder member and the constructor binding_with_binder (FR-038-AC-595)"
    },
    {
      "id": "TC-492",
      "path": "spec/checked_package/matrix/TC-492-integer-containment-at-result-positions.md",
      "role": "examined",
      "excerpt": "Verify FR-038-AC-586 to FR-038-AC-589 and FR-038-AC-595 at the public V2 reader: a term whose"
    }
  ]
}
```

## Validation evidence and limits

Read-only review; no Cargo, make, new worktree, source edit, push or merge. The dispatch supplied /private/tmp/ir-1112-tc492.log: 11 selected tc_492 tests passed, including IC-01 through IC-21. /private/tmp/ir-1112-compile.log records four focused conformance targets compiling successfully with conformance-cli; formatting was reported successful. These are prior-run evidence, not newly executed reviewer gates. Broad crate/workspace gates were prohibited by the dispatch. Normative FSpec dependency f73bd9f3fe5091ab8b459126808d7149cdb02b08 is merged; schema and vectors were fetched at that exact revision into scratch. No repository fixture pin needs updating because CI checks out main.

## Review identity correction

Canonical identity: SR-270178054278704885503246294768428943306. The former IR-1112 alias SR-17940486709564847048138 collided with an earlier independently allocated review identity in the parallel review lanes. It is retained only as a noncanonical historical alias, not reused as this document's identity. Immutable pre-correction copies of this artifact and original private comment are retained at /private/tmp/ir-1112-review/original-duplicate-aliases/. IR-1107's original identities and records were not changed. Six fresh UUID-derived decimal identities were checked against this worktree's reviews/spec-reviews and the available QSL-2717/QSL-2728 comment records before use.

## Dispositions

Fix round 1; independently inspected head a9f0f2d4c9d62d0cce6be1ce60bac798d4496dbd. Original finding rows and excerpts above are unchanged.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Prepaid arena/worklist/dependency storage and lazily prepaid diagnostics replace eager owned paths. Shared persistent summaries compute each term once, sharing nested suffixes; trie recursion is bounded by usize bit width. The public 100000-let test and six storage exact/one-short cases passed. |
| FND-002 | fixed | All five value containers propagate their summaries; functions remain boundaries and each let checks only its body, excluding its own initializer. Negative and correct-link oracles and shared-body identity regression passed. |

## Current verdict

**PASS** — all original findings in this method are fixed; no new finding was identified in this disposition pass. No new review was started.

```yaml
{
  "round": 1,
  "reviewed_head": "a9f0f2d4c9d62d0cce6be1ce60bac798d4496dbd",
  "dispositions": [
    {
      "fnd": "FND-001",
      "outcome": "fixed",
      "after_excerpt": "let mut filtered = meter.vec(node.dependencies.len())?;\n                for id in &node.dependencies {\n                    if !binders.contains(id) || expected.contains(id) {\n                        filtered.push(meter.clone_id(id)?);\n                    }\n                }\n                unbound = filtered;\n                &unbound",
      "reason": "Prepaid arena/worklist/dependency storage and lazily prepaid diagnostics replace eager owned paths. Shared persistent summaries compute each term once, sharing nested suffixes; trie recursion is bounded by usize bit width. The public 100000-let test and six storage exact/one-short cases passed."
    },
    {
      "fnd": "FND-002",
      "outcome": "fixed",
      "after_excerpt": "}\n}\n\nfn follows(kind: CheckedNodeKind) -> bool {\n    matches!(\n        kind,\n        CheckedNodeKind::Expression(_)\n            | CheckedNodeKind::Value(\n                ValueForm::RecordValue\n                    | ValueForm::TupleValue\n                    | ValueForm::CollectionValue\n                    | ValueForm::UnionValue\n                    | ValueForm::OptionValue\n            )\n    )\n}",
      "reason": "All five value containers propagate their summaries; functions remain boundaries and each let checks only its body, excluding its own initializer. Negative and correct-link oracles and shared-body identity regression passed."
    }
  ]
}
```

Execution evidence read directly: /private/tmp/ir-1112-fix1-tc492.log (15 passed including the public 100000-let test); /private/tmp/ir-1112-fix1-shared-body.log (1 passed); /private/tmp/ir-1112-fix1-budget.log (1 passed, six exact/one-short cases); /private/tmp/ir-1112-fix1-compile.log (four focused targets compiled); check and formatting logs successful. No reviewer Cargo command ran. Source and original findings remained frozen throughout this read-only pass.


