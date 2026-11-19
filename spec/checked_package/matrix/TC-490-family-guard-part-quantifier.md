---
id: TC-490
title: "Family guard part quantifier read and refused at the V2 reader"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: verifies
  - target: ix://agent-ix/filament-specification/FR-322-AC-479
    type: references
  - target: ix://agent-ix/filament-specification/FR-322-AC-480
    type: references
---
# TC-490: Family guard part quantifier read and refused at the V2 reader

## Description

Verify FR-038-AC-576 through FR-038-AC-580 at the public V2 reader. The
inputs are read by reference from the `FILAMENT_SPECIFICATION_DIR` checkout:
FSpec's `fixtures/positive-timed-clause-nodes.json`, its selected document
`domain-package-acme-timed-clauses.json`, and the
`timed-clause-node-vectors.json` cases `CL-57`, `CL-58`, `CL-63`, `CL-64`,
`CL-65` and `CL-66` (the last two with their `document_patch`). The
repository's `tests/fixtures/family-guard-part/family-part-vectors.json`
adds cases `FP-01` through `FP-06`. Each `FP` case names an FSpec base case
and lists only its own edits: `FP-01` to `FP-04` apply the `unbounded_ts`
edits, which are the `Order.ts` edits that `CL-65` publishes. `FP-05` and
`FP-06` give guard part 1 a `Text` binder `k` over the unbounded `clock` map.
Every replaced or removed member is checked first by a `test` operation. No
FSpec file is copied into this repository.

## Test Procedure

1. Replay `CL-65`, `CL-66` and `FP-05`. Encode each admitted package and
   compare the canonical bytes with the candidate's wire form. Read the part
   member after `binders` and compare its name and text, and compare each
   guard's node id with the one its candidate wrote.
2. Replay the published clause fixture and `CL-57`. Each admits with its
   body and node id unchanged.
3. Replay `CL-64`, `FP-01`, `FP-04` and `FP-06`. Compare the code, cause,
   RFC 6901 path and guard node locus.
4. Replay `CL-63`, `FP-02` and `FP-03`. Each refuses `malformed_wire` with no
   cause at the part path the case names, before any timed join.
5. Change one `test` value of an `FP` case and check that the replay fails
   before the read.

## Expected Results

`CL-65`, `CL-66` and `FP-05` are admitted and each writes back
byte-identically. The quantifier binding holds `each`, `some` and `each`
respectively. `CL-65` and `CL-66` keep their different guard ids, which end
in `quantifier`, `0`, `each` and `quantifier`, `0`, `some`. The controls in
step 2 are admitted unchanged.

`CL-64` refuses `invalid_package`/`timed-node-reference` at the quantifier
binding's value, `FP-01` and `FP-06` at their parts, and `FP-04` at its
first bad binder's `target`. Each refusal has node 47 as its locus. `CL-63`,
`FP-02` and `FP-03` refuse `malformed_wire` at
`/semantic_graph/nodes/47/body/members/1/value/members/0`.

The test can fail. Against the reader without these rules, `CL-65`, `CL-66`,
`FP-05` and `CL-64` refuse `malformed_wire` at the quantifier member, and
`FP-01` and `FP-06` are admitted. Every test binds its criterion with
`#[trace]`.

## Status

Planned (QSL-2570).
