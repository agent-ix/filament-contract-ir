---
id: FR-060
title: "Admit checked-package model terminal declarations"
type: FR
relationships:
  - target: ix://agent-ix/filament-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/filament-specification/FR-476
    type: references
  - target: ix://agent-ix/filament-specification/FR-322
    type: references
---

# FR-060: Admit checked-package model terminal declarations

## Description

When a V2 checked package carries a `temporal`/`terminal` node, the Contract
IR reader SHALL admit exactly FSpec FR-476's declaration body and joins, or refuse the
whole package at the first defect in FR-476's reader order. The canonical
writer SHALL retain the admitted node's complete body and occurrences. The
reader obtains the model's terminal declaration from this node alone.

## Inputs

The untrusted `filament.checked-package/v2` bytes, evidence and limits of
[FR-038](./FR-038-consume-checked-package-v2.md). The form and body are
specified by `ix://agent-ix/filament-specification/FR-476` and the identity,
graph and refusal ordering by `ix://agent-ix/filament-specification/FR-322`.

## Outputs

An admitted package retaining the terminal node, its exact semantic body,
dependencies, Boolean type and claim occurrences; or one typed refusal with
code, cause when defined and RFC 6901 pointer. No partial package.

## Behavior

The reader SHALL add `terminal` to the closed `temporal` form vocabulary
while continuing to refuse every unknown `temporal` form as
`invalid_semantic_graph` at `/semantic_graph/nodes/{n}/semantic_form`. A
terminal body is an aggregate of four bindings in fixed order: `model`,
`mode`, `parameters`, `condition`. `model` references the selected model
node. `mode` is a text literal `when` or `any`. `parameters` is empty or holds
one reference to the level-0 `self` `value`/`parameter` node, typed
`Reference<M::T>` for the referenced model's FR-366 object type. It is empty
for `any`. `condition` is a reference to a Boolean `expression` for `when`
and the empty aggregate for `any`.

The node SHALL have Boolean `semantic_type`, no `declaration`, one `claim`
occurrence per source declaration, and exactly the unique structural-id-ascending
set of body reference targets as `dependencies`. The written terminal name
exists only in its source occurrence; renaming it changes neither the structural node id nor package name. A valid change to model, mode or condition changes the complete body; the reader validates its dependencies and meaning without re-deriving an id. The node, not a separate request field, is the
carrier used by the deadlock-freedom subject.

The reader SHALL run FR-322's earlier strict-wire, graph-shape and structural-id checks
before the terminal body step. The terminal step SHALL follow the state-graph,
temporal and abstraction steps and precede the step-relation, hybrid,
timed-model, protocol and schedulability steps. It SHALL visit terminal
nodes by ascending structural node id. For each node it SHALL check exact body
shape; mode; mode fit of `parameters` then `condition`; targets in body order;
and, after every node's own checks, uniqueness per model. Shape and mode-fit
defects refuse `invalid_semantic_graph` at the body or offending binding.
Unknown mode refuses `invalid_package`/`invalid-value` at `body/members/1`.
A non-model `model` target refuses `invalid_model_binding`/
`malformed-declaration`; a wrong `self` target or non-expression condition
refuses `ill_typed`/`operator-ineligible`; a non-Boolean expression condition
refuses `ill_typed`/`non-boolean-root`, each at the target. A second terminal
node for one model refuses `ambiguous_declaration`/`ambiguous-name` at the
second node in structural-id order, naming both; two `claim` occurrences on one
terminal node refuse the same at that node, naming the first two
occurrences. A dependency mismatch refuses at `/dependencies` in the
earlier graph-shape step.

### Public terminal rows

An admitted package SHALL expose its model terminal declarations as typed
rows through one public read, `CheckedPackageV2::terminal_rows(&self) ->
&[TerminalRow]`, so a consumer reads the deadlock-freedom subject's terminal
declaration from the admitted package and never re-walks the graph or
re-parses a body:

```rust
pub struct TerminalRow {
    pub node: CheckedNodeId,        // the temporal/terminal node
    pub model: CheckedNodeId,       // the model node the declaration is on
    pub declaration: TerminalDeclaration,
}
pub enum TerminalDeclaration {
    Any,
    When {
        parameter: Option<CheckedNodeId>, // the level-0 `self` parameter node
        condition: CheckedNodeId,         // the Boolean `expression` node
    },
}
```

- Each row SHALL be built from the admitted node's body, in FSpec FR-476's
  body order: `model` from binding 1, the model reference's target;
  `declaration` from binding 2, the mode text, `any` giving `Any` and `when`
  giving `When` (any other mode is refused at admission, above);
  `parameter` from binding 3, `None` when the `parameters` aggregate is
  empty and `Some` of its one `self` parameter reference's target otherwise,
  so it is present exactly when the condition reads `self`; and `condition`
  from binding 4, the reference's target. An `any` node's `parameters` and
  `condition` bindings are the empty aggregate (FSpec FR-476), so `Any`
  carries neither.
- The rows SHALL be in the admitted graph's node order (ascending
  structural node id, the order the terminal step visits), so two reads of
  one package, and a read of the writer's output, return equal rows in equal
  order.
- A model with no terminal node SHALL have no row: it declares no terminal
  state, and the read never returns an empty or placeholder row for it. A
  model has at most one row, because a second terminal node for one model is
  refused at admission (`ambiguous_declaration`/`ambiguous-name`, above),
  not reported by this read.
- The read SHALL allocate nothing per call and return the rows retained at
  admission; a package with no terminal node returns the empty slice.

Note: FSL's `fsl-package` reads the same rows today in
`checked_v2/terminal.rs` (`ReadTerminal`); it is the consumer this read
serves.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-060-AC-1 | The reader admits FSpec TC-961 MT-01 through MT-03 as `temporal`/`terminal` nodes with the exact four-binding body, `when` and `any` modes, level-0 `self` parameter only when read, Boolean type, absent `declaration`, exact dependency set and `claim` occurrence. The writer emits the complete admitted shape, and a read of its output returns the same nodes and occurrences. | Test |
| FR-060-AC-2 | FSpec TC-961 MT-07 through MT-16 refuse with FR-476's exact code, cause and pointer after identities are rebuilt: swapped bindings, invalid mode, wrong mode fit, wrong model, wrong or mistyped `self`, non-Boolean condition, and both kinds of duplicate declaration are separately observed. An unknown `temporal` form still refuses at `semantic_form`, and no malformed terminal node is omitted or reinterpreted. | Test |
| FR-060-AC-3 | The reader observes FR-322 and FR-476 precedence: graph-shape defects precede the terminal step; state-graph, temporal and abstraction body defects precede it; each terminal node's own checks precede one-per-model checks; nodes visit in structural node-id order; and a terminal defect precedes step-relation and later body defects. | Test |
| FR-060-AC-4 | Withdrawn by the IR-1141 structural-identity decision under FSpec #517: the reader does not recompute identity from body content or refuse a retained structural id solely because a valid body changed. This id is retained and never reused. | Withdrawn |
| FR-060-AC-5 | IR-owned positive and negative read/write vectors for FSpec TC-961 MT-01 through MT-16 assert the published wire shapes and refusals through the public V2 reader and writer, without copying FSpec fixture or schema files into IR. | Test |
| FR-060-AC-6 | Over one admitted package selecting models `M` and `N` with FSpec TC-961 MT-01 (`when` reading `self`) on `M` and MT-02 (`any`) on `N`, and a third selected model `K` with no terminal node, `terminal_rows()` returns exactly two rows in ascending structural node id: `{node: MT-01's node, model: M, declaration: When { parameter: Some(the level-0 self parameter node), condition: the Boolean expression node }}` and `{node: MT-02's node, model: N, declaration: Any}`, and no row for `K`; MT-03 (`when` not reading `self`) on `M` in place of MT-01 gives `When { parameter: None, condition: the forall expression node }`; the rows of the writer's output read back are equal and in equal order; a package with no terminal node returns an empty slice; and a second terminal node on `M` is refused at admission, so no read returns two rows for one model. | Test (TC-487) |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns the V2 reader, writer,
  identity and shared refusal machinery.
- `ix://agent-ix/filament-specification/FR-476` owns the exact terminal body,
  reader step and target rules; `ix://agent-ix/filament-specification/FR-322`
  owns the enclosing package and earlier checks.

## Status

Planned. The current reader rejects `temporal`/`terminal` at its closed form
gate; no reader or writer acceptance is claimed by this spec.
