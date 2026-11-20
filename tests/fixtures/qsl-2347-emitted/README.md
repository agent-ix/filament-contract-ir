# QSL-2347 emitted-package fixtures

Each file is the checked package that FSL's emit test harness (`emit_checked`
in `fsl-package`, source identity `test:spine-model`, package id `pkg`) writes
for the FSL source below. They are the harness's output, not `fsl compile`
output. The harness was run from a scratch test in an FSL checkout at FSL main
of 2026-11-20; the scratch test is not kept, so regenerate a fixture by emitting
the source below with the same harness. The files are re-serialised with sorted
keys and no other change.

All three sources start with
`profile T = "filament.temporal.complete/v1"; dimension Time; unit ms : Time = rational(1, 1);`.

- `clock-place.json`:
  `record W { c: Clock; }`
  `object O { w: W; d: Clock; operation step() modifies self.d; }`
  `time O dense unit ms;`
  (FSpec FR-789-AC-1.)
- `clock-collections.json`:
  `record W { c: Clock; }`
  `object O { d: Clock; q: Sequence<Clock>[0, 3]; r: Sequence<W>[0, 3]; operation arm(x: Clock) modifies self.d; }`
  `time O dense unit ms;`
- `clock-tuple-union.json`:
  `record W { c: Clock; n: Int[0, 9]; }` `tuple P (Clock, Integer);`
  `union U { armed(Clock), idle }` `object Q { x: Integer; }`
  `object O { w: W; d: Clock; q: Sequence<Clock>[0, 3]; p: P; u: U; r: Sequence<W>[0, 3]; qq: Q; s: Sequence<Integer>[1, 2]; t: Text[0, 4; nfc]; operation arm(x: Clock, k: Integer) modifies self.d; }`
  `time O dense unit ms;`
  FSL spells the tuple and union member identities `ix://I/P/tuple/0` and
  `ix://I/U/union/armed`; FSpec FR-789 rule 4 requires `ix://I/P/p0` and
  `ix://I/U/armed`. The reader follows FSpec, so the test respells them before
  reading.

FSL emits no `parameter` clock constraint or reset. The tests write a reset by
hand on top of `clock-collections.json` (see `with_parameter_clauses`).
