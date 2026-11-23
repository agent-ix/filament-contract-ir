---
id: TC-494
title: "Clock-reference members project to Exact ClockReference"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-352
    type: verifies
---

# TC-494: Clock-reference members project to Exact ClockReference

## Description

Verify [FR-352](../functional/FR-352-project-admitted-exact-value-types.md)'s
clock-reference projection: every data member typed by the package's one
`scalar_type`/`clock_reference` node projects to Exact `ClockReference`
(Exact FR-415) with its wrappers intact and no identity witness, a plain
`Clock` member keeps its existing outcomes, and the read accepts or refuses no
value. Scope: FR-352-AC-45 and FR-352-AC-46.

## Test Procedure

Fixture: FSpec FR-829-AC-1's `Call` model under `time dense unit ms`, with
the data members `r: Reference<Clock>`, `ra: Reference<A>` (`type A = Clock`),
optional `maybe: Reference<Clock>`, `refs: Sequence<Reference<Clock>>[0, 3]`,
`peer: Reference<Call>`, a tuple type `(Boolean, Reference<Clock>)` held by a
field `pair`, a tuple type `(Clock, Reference<Clock>)` held by a field
`mixed`, a model union `Tag` with cases `Named(Reference<Clock>)` and
`Timed(Clock)`, and the clock field `x: Clock`, compiled by FSL and admitted
through the strict reader with its selected identity rows.

1. Project `r`, `ra`, `maybe`, `refs`, `peer`, position 1 of `pair` and the
   `Named` payload through the public `ExactTypeProjection` and the shared
   record-field projection. Record whether any identity row was consulted for
   the clock-reference members.
2. Repeat step 1 on a package clone, then at the measured exact work
   allowance and at one unit less.
3. Project `x` and read the `Timed` payload.
4. Rewrite the `Named` payload's original type to `Reference<Call>`, and in a
   second fixture rewrite the original type of an object-reference payload
   node `Linked(Reference<Call>)` to the clock-reference type, and join each.
5. Project the step 1 members of a second package whose `Call` model declares
   only `y: Clock` in place of `x`, so that a snapshot reference to `c1.x`
   names no place of it.

## Expected Results

- Step 1: `r`, `ra`, position 1 of `pair` and the `Named` payload return
  Exact `ClockReference`; `maybe` returns `ClockReference` with
  `Presence::Optional`; `refs` returns `Sequence<ClockReference>[0, 3]`;
  `peer` returns `Reference(EffectiveId)` of `Call`. No clock-reference member
  returns `Reference(EffectiveId)`, `UnavailableIdentityWitness` or
  `UnsupportedExactType`, and none consults an identity row.
- Step 2: the clone gives equal views; the exact allowance succeeds and one
  unit less refuses with typed work exhaustion, charged work and the first
  uncharged member locus.
- Step 3: `x` returns `UnsupportedExactType` at the record node (form
  `model/record_value_type`); the tuple `mixed` refuses `UnsupportedExactType`
  at the tuple node, form `tuple_position`, because a tuple projects whole or
  not at all; `Timed` is
  retained as a Clock payload (FR-352-AC-32).
- Step 4: each join refuses `MismatchedIdentityWitness` at the payload node,
  form `/model_unions/i/cases/j/payload/original_type`.
- Step 5: the members project to the same types as in step 1; the read makes
  no place check, which belongs to FSL value admission.
