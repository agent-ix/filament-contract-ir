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
FSpec FR-476's terminal row order (by the model node's structural key, then
the terminal node's, FSpec FR-322's order of structural keys, not graph
order), each with its per-model `ordinal`, with no row for a model that
declares no terminal state, and several rows for a model node that several
terminal nodes reference. Scope: FR-060-AC-6, FR-060-AC-8.

## Test Procedure

1. Build an IR-owned V2 package selecting models `M`, `N` and `K`, with FSpec
   TC-961 MT-01 (`when` reading `self`) on `M`, MT-02 (`any`) on `N`, and no
   terminal node on `K`. First establish the key order: rebuild the
   `model_root` nodes' structural keys of `M` and `N` and compare them in FSpec
   FR-322's ascending node-id order, and assert that `M`'s sorts before `N`'s. Then
   place `N`'s terminal node before `M`'s in `semantic_graph.nodes`, opposite
   to that key order, rebuild dependencies and identities, read the package
   through the public reader and call `terminal_rows()`. Write the admitted
   package, read the output and call it again.
2. Build the same package with `M`'s terminal node before `N`'s in graph
   order and call `terminal_rows()`.
3. Replace MT-01 with MT-03 (`when` not reading `self`) and read again.
4. Read a package with no terminal node.
5. Add a second terminal node on `M`, FSpec TC-961 MT-12, and read the
   package; then build FSpec TC-961 MT-18 (MT-01 and MT-02 on one
   `model_root` node) in both graph orders, establishing the two terminal
   nodes' key order by comparing their rebuilt keys; then a third terminal
   node on that model.
6. Read FSpec TC-961 MT-16 (one node, two `claim` occurrences), write it
   and read the output.
7. Read FSpec TC-961 MT-19 (`V` selecting `test/vault`, `N` selecting
   `test/vault-other`, a terminal on each).

## Expected Results

- Step 1: exactly two rows, in key order and so opposite to graph order:
  `{node: MT-01's node, model: M, ordinal: 0, declaration: When { parameter: Some(the
  level-0 self parameter node), condition: the Boolean expression node }}`
  then `{node: MT-02's node, model: N, ordinal: 0, declaration: Any}`; no row for `K`;
  every id is in the resolved-graph form and compares by structural key; the
  rows after write and re-read are equal and in equal order.
- Step 2: rows equal to step 1's, in the same order.
- Step 3: `M`'s row is `When { parameter: None, condition: the forall
  expression node }`.
- Step 4: an empty slice.
- Step 5: each package is admitted with no `ambiguous_declaration`
  refusal; the model's rows are in terminal node key order with `ordinal` 0
  and 1 (0, 1 and 2 with the third node), equal under both graph orders.
- Step 6: admitted; the written package retains both occurrences and their
  `source_map` entries; one row with `ordinal` 0.
- Step 7: the `test/vault` row precedes the `test/vault-other` row, each
  with `ordinal` 0.
