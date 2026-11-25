---
id: SR-1599
title: "spec-review/integrity of IR-1614 FR-038-AC-629 owner operand sentence"
type: SpecReview
analysis: integrity
scope: "agent-ix/filament-contract-ir; PR #1173 at 414e28ad; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (FR-038-AC-629, FR-038-AC-80), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md (owner rows), spec/model/functional/FR-019-rust-library-interface.md (Public items)"
review_set: subset
---

## Summary

Spec-review/integrity of the FR-038-AC-629 sentence and the TC-048 row this PR adds, checked against filament-specification origin/main FR-322 "Structural node identity" and FR-322-AC-538. Ticket: IR-1614. The review also records the two spec defects behind the four tests this PR leaves red, since they block a green `make test`; both predate this PR.

## Examined scope

- FR-038-AC-629 added sentence (examined): it matches the code for declared clauses and the condition in FR-322-AC-538 ("whose node id path contains the owner-model operand").
- TC-048 added positive row (examined): consistent with the AC and the new test.
- FR-038-AC-80 (examined, pre-existing): against both Cargo manifests and the AC-80 test.
- FR-019 "Public items" (examined, pre-existing): against the crate root exports and the inventory parser in tests/it/public_interface.rs.

## Verdict

Changes requested on the spec text (FND-001); FND-002 is low. FND-003 and FND-004 predate this PR, are not regressions from it, and need spec edits before the four red tests can be made green without contradicting the spec.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new FR-038-AC-629 sentence cites FSpec FR-322-AC-538 for admitting a declaration-only path, but FR-322 "Structural node identity" says the reader verifies the owner target against the owner-model operand in the path of every clause carrying `owner_model`. AC-538 only conditions its own case. The sentence should state the limitation plainly: on a declaration-only path the owner edge is not compared, so a retargeted `owner_model` admits until the upstream gap is decided | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6981 |
| FND-002 | low | The sentence defines the operand-bearing path relative to "the declaration path" and says nothing for an owned clause without a declaration, which the code now never compares (code-review FND-001) | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6981 |
| FND-003 | high | Pre-existing: FR-038-AC-80 requires `quire-canonical` to stay a `branch = "main"` git dependency, while both manifests pin a tag and the AC-80 test asserts a specific older tag literal. The AC should require a tagged git dependency (a `tag`, never `branch`, `rev` or `path`) without naming a version, and the test should drop its version literal | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6709, Cargo.toml:130, crates/filament-contract-model/Cargo.toml:58, tests/it/checked_package_v2_canonical_encoding.rs:1277 |
| FND-004 | high | Pre-existing: the FR-019 Public items table omits the crate-root export `read_checked_package_structural`, which no requirement in this repository owns, and its V2 schedulability row puts the field names `task_set`, `release_bindings`, `mode_switch_bindings` in backticks in the items cell, which the exact inventory reads as exported items | spec/model/functional/FR-019-rust-library-interface.md:314, crates/filament-contract-model/src/lib.rs:152 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 4993b028 replaces the citation with a "Known limitation" sentence: on a declaration-only path the edge is not compared and a retarget admits; FR-322 asks for the comparison but does not say what it compares there, and AC-538 covers only an operand-bearing path. A test pins the uncompared admission. |
| FND-002 | fixed | 4993b028 states that a clause without a declaration is refused as `malformed_wire` at its node whatever its path holds, which flat_wire.rs does and a new test pins. |
| FND-003 | fixed | 4993b028 rewrites FR-038-AC-80 as a tag-pinned git dependency with no `branch`, `rev` or `path`, no version named. The test now requires `tag = "` and still refuses `branch =`, `rev =` and `path =` on the declaring line. It also requires the lock source to be a `?tag=` source. No pin changed. |
| FND-004 | fixed | 4993b028 lists `read_checked_package_structural` in the checked_package functions row and writes the three schedulability fields as prose, not backticked names. That row cites FR-038 and FR-047, but neither specifies this function; that gap is separate and is deferred to the spec-gap ticket the lead is filing. |
