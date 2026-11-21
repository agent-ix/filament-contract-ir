---
id: SR-112259010033744557987310838477872830073
duplicate_review_alias: "SR-17940486709564847048139"
title: "IR-1112 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; IR-1112 changed files; QSL-2728; FR-038, FR-019, TC-492"
review_set: subset
---

## Summary

Ticket: QSL-2728. Independent static review of frozen head fc51b389a1669a38a51a07d20072ea21c4cf4e97 against base 0e089f6ea5dea8d799cb0234ec9623e0073ffad4 and supplied normative FR-322. 1 concrete finding(s).

## Verdict

**CONDITIONAL** — Resolve the recorded findings in the original PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Binder evidence covers an expression-call reference but no wrong-binder reference through a value container or budgeted binder-validation allocation boundary. | tests/it/checked_package_v2_operation_arguments.rs:2449 |

## Coverage

Plan completion: not assessed

Reconciliation: quoin matrix (quoin 0.36.7; quire 0.52.0 engine 0.62.2). The whole-repository matrix contains 1149 tagged, 115 untagged, 1 tagged-by-ignored-test, 26 method-without-symbol and 8 retired criteria. These are repository-wide context, not a claim that this ticket repairs the wider backlog. All selected live FR-038-AC-586 through AC-589 and AC-595 are tagged. No passing run evidence is populated in the matrix store; the dispatch's concrete test log is reported separately.

quire coverage returned 292 untracked symbols and symbol-id-collision diagnostics in existing conditional/container declarations. The selected operation-argument test file has no untracked-symbol row. No new trace extraction failure was found for the changed binder AC. The hand-maintained tests.md status row was not used as coverage evidence.

Reverse inventory in the authorized changed surface: 7 behavior groups (public binding construction; decode/encode; ID visitation/replacement/mapping; dependency inclusion; binder placement/link validation; exact body-read identity; binder type compatibility). All 7 have owning requirements in FR-038/FR-349/FR-019 or normative FR-322. No new stub or no-op production behavior was found. The code-to-test check nevertheless found an uncovered value-container traversal branch and absent budget-boundary evidence for the new allocator paths (FND-001); see the companion code review for concrete defects. Existing tests exercise real public reads, assert exact code/cause/pointers for the mutation vectors, and include IC-15 and IC-21 refusal plus unread-compatible controls.

This is a ticket-scoped gap review under the dispatch's scope override, not a full-repository assurance PASS. Optional semantic fan-out was not run; focused behavior and negative-vector inspection was explicitly requested and performed inside code review.

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

Canonical identity: SR-112259010033744557987310838477872830073. The former IR-1112 alias SR-17940486709564847048139 collided with an earlier independently allocated review identity in the parallel review lanes. It is retained only as a noncanonical historical alias, not reused as this document's identity. Immutable pre-correction copies of this artifact and original private comment are retained at /private/tmp/ir-1112-review/original-duplicate-aliases/. IR-1107's original identities and records were not changed. Six fresh UUID-derived decimal identities were checked against this worktree's reviews/spec-reviews and the available QSL-2717/QSL-2728 comment records before use.

## Dispositions

Fix round 1; independently inspected head a9f0f2d4c9d62d0cce6be1ce60bac798d4496dbd. Original finding rows and excerpts above are unchanged.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | New tests cover each of five value-container wrong-read paths with correct-link admission controls, function/initializer boundaries, a shared body checked against distinct links, 100000 nested lets, and six budget accounting/exact/one-short cases. |

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
      "after_excerpt": "let_container_case(form, true).refuses(\n            form,\n            Code::InvalidPackage,\n            Cause::InvalidValue,\n            tail,\n        );\n    }\n}",
      "reason": "New tests cover each of five value-container wrong-read paths with correct-link admission controls, function/initializer boundaries, a shared body checked against distinct links, 100000 nested lets, and six budget accounting/exact/one-short cases."
    }
  ]
}
```

Execution evidence read directly: /private/tmp/ir-1112-fix1-tc492.log (15 passed including the public 100000-let test); /private/tmp/ir-1112-fix1-shared-body.log (1 passed); /private/tmp/ir-1112-fix1-budget.log (1 passed, six exact/one-short cases); /private/tmp/ir-1112-fix1-compile.log (four focused targets compiled); check and formatting logs successful. No reviewer Cargo command ran. Source and original findings remained frozen throughout this read-only pass.


