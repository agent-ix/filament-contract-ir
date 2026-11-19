# Family guard part vectors

`family-part-vectors.json` holds the family guard part cases of FR-038
"Family guard parts" (TC-490) that FSpec does not publish. FSpec publishes
the admitted `Sequence` family parts as `CL-65` and `CL-66`. This file adds
the refusals (`FP-01`..`FP-04`) and the `Map` family (`FP-05`, `FP-06`).

It is not a package. Each case names a base case of FSpec's
`timed-clause-node-vectors.json` and lists only the edits this repository
adds. The test reads FSpec's clause fixture, its selected document and the
base patch from the `FILAMENT_SPECIFICATION_DIR` checkout. It then applies the
`unbounded_ts` edits when the case asks for them, then the case's own patch.
Each `test` operation checks that a member still holds the value an edit
expects before that edit is applied.

No FSpec file is copied here. Guard node ids are the producer's: the `Map`
case writes the FSpec FR-322-AC-480 suffix `quantifier`, `1`, `each` for
guard part 1. The vectors are maintained with the repository's
AGPL-3.0-or-later tests.
