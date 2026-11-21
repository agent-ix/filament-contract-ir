---
id: SR-17940486709564847048143
title: "IR-1112 spec-review-failure-domain"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/filament-contract-ir; IR-1112 changed files; QSL-2728; FR-038, FR-019, TC-492"
review_set: subset
---

## Summary

Ticket: QSL-2728. Independent static review of frozen head fc51b389a1669a38a51a07d20072ea21c4cf4e97 against base 0e089f6ea5dea8d799cb0234ec9623e0073ffad4 and supplied normative FR-322. No specification defect found in the selected changed units.

## Verdict

**PASS** — The selected changed obligations are consistent and verifiable.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Identity: the new obligation binds a let to exactly one parameter NodeRef and forbids a second same-name/same-level read. Type checking explicitly includes unread binders, closing the missing-use edge. Dependency inclusion makes the binder part of graph reachability; normative derived-path operands preserve its type/level identity at the producer boundary.

Topology and purity: existing work/read-storage budgets govern the new graph traversal; no new external effects or callbacks are introduced. Missing, unresolved, wrong-kind, unlisted and misnamed links have defined refusal classes and positions. No additional missing spec constraint was found. The implementation violates existing budget and traversal obligations, captured by the separate code review; the specification already states those obligations.

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

