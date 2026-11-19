---
id: TC-487
title: "Terminal rows read from an admitted package"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-060
    type: verifies
---

# TC-487: Terminal rows read from an admitted package

## Description

Verify [FR-060](../functional/FR-060-admit-model-terminal-declarations.md)'s
public terminal rows: `CheckedPackageV2::terminal_rows()` returns one typed row
per terminal node, built from FSpec FR-476's body bindings in body order, in
ascending structural node-id order (FSpec FR-322's order of structural keys,
not graph order), with no row for a model that declares no terminal state.
Scope: FR-060-AC-6.

## Test Procedure

1. Build an IR-owned V2 package selecting models `M`, `N` and `K`, with FSpec
   TC-961 MT-01 (`when` reading `self`) on `M`, MT-02 (`any`) on `N`, and no
   terminal node on `K`. First establish the key order: rebuild both terminal
   nodes' structural keys and compare them in FSpec FR-322's ascending
   node-id order, and assert that `M`'s terminal key sorts before `N`'s. Then
   place `N`'s terminal node before `M`'s in `semantic_graph.nodes`, opposite
   to that key order, rebuild dependencies and identities, read the package
   through the public reader and call `terminal_rows()`. Write the admitted
   package, read the output and call it again.
2. Build the same package with `M`'s terminal node before `N`'s in graph
   order and call `terminal_rows()`.
3. Replace MT-01 with MT-03 (`when` not reading `self`) and read again.
4. Read a package with no terminal node.
5. Add a second terminal node on `M`, FSpec TC-961 MT-12, and read the
   package.

## Expected Results

- Step 1: exactly two rows, in key order and so opposite to graph order:
  `{node: MT-01's node, model: M, declaration: When { parameter: Some(the
  level-0 self parameter node), condition: the Boolean expression node }}`
  then `{node: MT-02's node, model: N, declaration: Any}`; no row for `K`;
  every id is in the resolved-graph form and compares by structural key; the
  rows after write and re-read are equal and in equal order.
- Step 2: rows equal to step 1's, in the same order.
- Step 3: `M`'s row is `When { parameter: None, condition: the forall
  expression node }`.
- Step 4: an empty slice.
- Step 5: the reader refuses `ambiguous_declaration`/`ambiguous-name` at the
  second node in ascending node-id order, naming both (MT-12), so no read
  returns two rows for one model.
