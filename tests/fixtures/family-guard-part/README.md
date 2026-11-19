# Family guard part vectors

`family-part-vectors.json` holds the admitted and refused family guard part
cases of FR-038 "Family guard parts" (TC-490). It is not a package: each case
names a base case of FSpec's `timed-clause-node-vectors.json` and lists only
the edits this repository adds. The test reads FSpec's clause fixture, its
selected document and the base patch from the `FILAMENT_SPECIFICATION_DIR`
checkout, applies the `unbounded_ts` edits (the `test` operations check that
each edited member still holds the bounded value), then the case's own patch.
No FSpec file is copied here. The vectors are maintained with the
repository's AGPL-3.0-or-later tests.
