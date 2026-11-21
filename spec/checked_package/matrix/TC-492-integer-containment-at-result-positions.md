---
id: TC-492
title: "Integer containment at result positions at the V2 reader"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: verifies
  - target: ix://agent-ix/filament-contract-ir/FR-347
    type: verifies
  - target: ix://agent-ix/filament-specification/FR-322-AC-495
    type: references
  - target: ix://agent-ix/filament-specification/TC-2099
    type: references
---
# TC-492: Integer containment at result positions at the V2 reader

## Description

This local test case implements FSpec TC-2099 ("Integer containment at result
positions") at this repository's reader; it carries its own local id.

Verify FR-038-AC-586 to FR-038-AC-589 and FR-038-AC-595 at the public V2 reader: a term whose
integer type an expected type contains is admitted with no convert node at the
result positions of FSpec FR-322 "Integer containment", and nothing else is
widened. The packages are built node by node in
`tests/it/checked_package_v2_operation_arguments.rs`, one per FSpec vector
`IC-01` to `IC-21`. FSpec b90d84ec does not yet publish
`integer-containment-vectors.json`, so this repository builds the packages from
its own vocabulary; the reader should switch to the published bytes when FSpec
ships them.

## Test Procedure

1. Read the function-result packages `IC-01` to `IC-04` and `IC-08`, and an
   `expression` root over a contained and an uncontained `result_type`.
2. Read the call packages `IC-05` to `IC-07`.
3. Read the `if`, `case` and `let` packages `IC-09` to `IC-13` and `IC-16` to
   `IC-18`.
4. Read the annotated `let` packages `IC-14`, `IC-15`, `IC-19`, `IC-20` and `IC-21`.
5. Read `IC-14` with each binder-link mutation of FSpec FR-322-AC-512.

## Expected Results

- Step 1: `IC-01` and `IC-02` admitted; `IC-03`, `IC-04` and `IC-08`
  `ill_typed`/`type-mismatch` at the function node.
- Step 2: `IC-05` admitted; `IC-06` and `IC-07` `ill_typed`/`operator-ineligible`
  at the call's second argument.
- Step 3: `IC-09`, `IC-10`, `IC-11`, `IC-16` and the hull-typed `IC-17`
  admitted; `IC-12` and `IC-18` `ill_typed`/`operator-ineligible` at the `if`
  application, `IC-13` at the `case` node.
- Step 4: `IC-14` and `IC-20` admitted; `IC-19` refused at the call;
  `IC-15` and `IC-21` (an unread narrow binder) `ill_typed`/`operator-ineligible`
  at the `let` application.
- Step 5: a `binder` removed, one naming a node of another name, a node that
  is no `value`/`parameter`, no node or a node outside the `dependencies`,
  a `binder` on a function's body binding, and a read of a second
  `value`/`parameter` named `y` at the binder's level each refuse
  `invalid_package`/`invalid-value` at the stated place.
