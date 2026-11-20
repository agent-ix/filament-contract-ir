---
id: TC-2099
title: "Integer containment at result positions at the V2 reader"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: verifies
  - target: ix://agent-ix/filament-contract-ir/FR-347
    type: verifies
  - target: ix://agent-ix/filament-specification/FR-322-AC-495
    type: references
---
# TC-2099: Integer containment at result positions at the V2 reader

## Description

Verify FR-038-AC-586 to FR-038-AC-589 at the public V2 reader: a term whose
integer type an expected type contains is admitted with no convert node at the
result positions of FSpec FR-322 "Integer containment", and nothing else is
widened. The packages are built node by node in
`tests/it/checked_package_v2_operation_arguments.rs`, one per FSpec vector
`IC-01` to `IC-20` (FSpec publishes the vector file with the checked-package
corpus; this repository builds the same packages from its own vocabulary).

## Test Procedure

1. Read the function-result packages `IC-01` to `IC-04` and `IC-08`, and an
   `expression` root over a contained and an uncontained `result_type`.
2. Read the call packages `IC-05` to `IC-07`.
3. Read the `if`, `case` and `let` packages `IC-09` to `IC-13` and `IC-16` to
   `IC-18`.
4. Read the annotated `let` packages `IC-14`, `IC-15`, `IC-19` and `IC-20`.

## Expected Results

- Step 1: `IC-01` and `IC-02` admitted; `IC-03`, `IC-04` and `IC-08`
  `ill_typed`/`type-mismatch` at the function node.
- Step 2: `IC-05` admitted; `IC-06` and `IC-07` `ill_typed`/`operator-ineligible`
  at the call's second argument.
- Step 3: `IC-09`, `IC-10`, `IC-11`, `IC-16` and the hull-typed `IC-17`
  admitted; `IC-12` and `IC-18` `ill_typed`/`operator-ineligible` at the `if`
  application, `IC-13` at the `case` node.
- Step 4: `IC-14` and `IC-20` admitted; `IC-15` refused at the `let`
  application; `IC-19` refused at the call.
