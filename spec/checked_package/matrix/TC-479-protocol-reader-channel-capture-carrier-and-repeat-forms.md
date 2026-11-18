---
id: TC-479
title: "Strict V2 reader admits bounded channels, stable FIFO keys, population captures, reference carriers, post-test repeats and back-edge reads"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-372
    type: verifies
---
# TC-479: Strict V2 reader admits bounded channels, stable FIFO keys, population captures, reference carriers, post-test repeats and back-edge reads

## Description

Verify every AC of FR-372 through the public strict V2 reader over protocol
packages built in the test.

## Test Procedure

1. Point a channel's `carries` at a `bounded_domain` node of each of the six
   forms; expect admission. Point it at an `expression` node; expect
   `invalid_model_binding`/`malformed-declaration` at the target (FR-372-AC-1).
2. Key a FIFO channel by each FIFO-key-stable type of FR-372-AC-2; expect
   admission (FR-372-AC-2).
3. Key a FIFO channel by each float or event-bearing type of FR-372-AC-3;
   expect `ill_typed`/`operator-ineligible` at the key retaining `=` and the
   type; declare the same channel `unordered`; expect admission
   (FR-372-AC-3).
4. Bind a protocol capture, a registration and an activation capture, a
   `send` and an `event` constraint and a `finish` constraint to
   `size(allInstances<M::Order>(M::orders))` in a clause owned by `M`; expect
   admission. Replace the capture initializer with an out-of-scope parameter;
   expect `missing_declaration`/`missing-name`. Put the read in an activation
   guard, a `choice` guard and a `check` condition; expect
   `ill_typed`/`operator-ineligible` (FR-372-AC-4).
5. Put a `protocol_reference` carrier in each slot of FR-372-AC-5; expect
   admission. Read the referenced clause's finish binder directly, and `over`
   beside the carrier in a barred slot; expect the stated refusals
   (FR-372-AC-5).
6. Add `test` `post` to a `repeat` whose guard reads its body's binder with
   no, 3 and 1 `maximum`; expect admission. Set `maximum` 0, `test` `pre` and
   `middle`, and drop `test`; expect the stated refusals. Compare node bytes
   and ids of the pre-test and post-test forms (FR-372-AC-6).
7. Read a post-test `repeat` whose `invariant` and `variant` read the body's
   every-path binder; expect admission. Make its `visible` entry read a binder
   one `choice` branch binds; expect `missing_declaration`/`missing-name`.
   Admit pre-test `repeat`s whose `visible` entry, and whose `invariant` and
   `variant`, read the body binder without `test` (FR-372-AC-7).

## Expected Results

Each admitted form reads; each ineligible or malformed variant refuses with
the stated code, cause and locus.

## Status

Partial: the admitted cases of AC-1, AC-2, AC-4 (protocol capture) and AC-5
(capture initializer), the out-of-scope capture refusal of AC-4, and the
`post` admission and `middle` refusal of AC-6 are tested; the remaining cases
are not yet tested.
