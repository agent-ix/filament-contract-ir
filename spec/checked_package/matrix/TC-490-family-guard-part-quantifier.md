---
id: TC-490
title: "Family guard part quantifier read and refused at the V2 reader"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: verifies
  - target: ix://agent-ix/filament-specification/FR-322-AC-479
    type: references
---
# TC-490: Family guard part quantifier read and refused at the V2 reader

## Description

Verify FR-038-AC-576 through FR-038-AC-580 at the public V2 reader. The
inputs are FSpec's published `fixtures/positive-timed-clause-nodes.json`, its
selected document `domain-package-acme-timed-clauses.json` and its
`timed-clause-node-vectors.json` cases `CL-57`, `CL-58`, `CL-63` and `CL-64`,
read from the `FILAMENT_SPECIFICATION_DIR` checkout by reference, and this
repository's `tests/fixtures/family-guard-part/family-part-vectors.json`
cases `FP-01` through `FP-06`. Each `FP` case names an FSpec base case and
adds only its own patch; its `unbounded_ts` patches remove the declared
maximum of `Order.ts` in the selected document and write `unbounded` in the
place's slot step, `resolved_path` row and every inline copy of the place
key, so the base guard part ranges over a collection with no declared
maximum. No FSpec file is copied into this repository.

## Test Procedure

1. Replay `FP-01` and `FP-02`; encode each admitted package and compare the
   canonical bytes with the candidate's wire form; read the part's third
   member and compare its name and text.
2. Compare the two admitted packages: the bytes differ only at the
   quantifier value, and each guard node keeps the node id its candidate
   wrote.
3. Replay the published unquantified controls (the clause fixture as
   published, `CL-57`, and guard part 1 of node 47, whose constraint selects
   the unbounded `clock` map by the literal key `audit` and has no binders):
   each admits with its body and node id unchanged.
4. Replay `CL-64`, `FP-03` and `FP-06`; compare code, cause, RFC 6901 path
   and the guard node locus.
5. Replay `CL-63`, `FP-04` and `FP-05`; each refuses `malformed_wire` with no
   cause at the part path the vector names, before any timed join.

## Expected Results

`FP-01` and `FP-02` admit and round-trip byte-identically, the part's third
member is the `quantifier` binding with text `each` or `some`, and the two
packages differ only in that text. The controls of step 3 admit unchanged.
`CL-64` refuses `invalid_package`/`timed-node-reference` at the quantifier
binding's value, `FP-03` at the part, and `FP-06` at its first bad binder's
`target`, each with node 47 as locus. `CL-63`, `FP-04` and `FP-05` refuse
`malformed_wire` at `/semantic_graph/nodes/47/body/members/1/value/members/0`.
Against the reader before this rule, `FP-01`, `FP-02` and `CL-64` refuse
`malformed_wire` at the quantifier member and `FP-03` admits, so the test
fails. Every test binds its criterion with `#[trace]`.

## Status

Planned (QSL-2570).
