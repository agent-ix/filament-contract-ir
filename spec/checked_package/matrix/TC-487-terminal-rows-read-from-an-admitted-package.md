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
ascending structural node id, with no row for a model that declares no
terminal state. Scope: FR-060-AC-6.

## Test Procedure

1. Build an IR-owned V2 package selecting models `M`, `N` and `K`, with FSpec
   TC-961 MT-01 (`when` reading `self`) on `M`, MT-02 (`any`) on `N`, and no
   terminal node on `K`. Read it through the public reader and call
   `terminal_rows()`; write the admitted package, read the output and call
   it again.
2. Replace MT-01 with MT-03 (`when` not reading `self`) and read again.
3. Read a package with no terminal node.
4. Add a second terminal node on `M` and read the package.

## Expected Results

- Step 1: exactly two rows in ascending structural node id:
  `{node: MT-01's node, model: M, declaration: When { parameter: Some(the
  level-0 self parameter node), condition: the Boolean expression node }}`
  and `{node: MT-02's node, model: N, declaration: Any}`; no row for `K`; the
  rows after write and re-read are equal and in equal order.
- Step 2: `M`'s row is `When { parameter: None, condition: the forall
  expression node }`.
- Step 3: an empty slice.
- Step 4: the reader refuses `ambiguous_declaration`/`ambiguous-name` at the
  second node, so no read returns two rows for one model.
