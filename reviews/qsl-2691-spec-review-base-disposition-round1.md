---
id: SR-967175026302197768900474181941935084513
title: "QSL-2691 PR 1110 spec-review-base disposition round 1"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; QSL-2691; PR 1110; original findings and architecture additions"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-038", type: references }
---

## Summary

Fix-round 1 disposition of the existing Titan/Opus spec-review-base review, original head 2e68276519dbce5845d35dd76a06f12bb830d96e. Reviewed frozen head 8d4d8620a159fe2fa32693c3bb52978f2811ae05 on merged binder base dacfac6b371b89f1942849e3c502ad01fbb68d2c. Original SR SR-17940486709564847048143 is unavailable as a file; original private Linear comment [dd9d2ec1-587f-46cf-b6f8-5f06fe26fad5](https://linear.app/agent-ix/issue/QSL-2691/body-location-rows-grow-quadratically-with-body-depth-give-them-a#comment-dd9d2ec1) is the authoritative retained record. This fresh-ID artifact preserves its findings and appends dispositions; it does not replace the original review or claim a new first review.

## Tracker disposition record

Private Linear disposition comment: 1dcc3a7b-bee1-4103-9309-d17f5bc423c8. Read-back passed: exactly one marked comment; real YAML parser accepted the returned data; the disposition ids exactly match the original unresolved finding set; all fixed outcomes carry nonempty after_excerpt text of at most 600 characters. The has-review label was already present. This artifact and comment use fresh SR identity while retaining the original SR and comment provenance above.

## Verdict

**PASS — CLEAR for this bounded disposition.** Every original finding below is fixed. No new finding in the fix and requested architecture scope. Aggregate merge gates remain the lead's responsibility. FSL producer integration is separately owned and does not block this reader-first integration.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FSpec says "The typed read returns the rebuilt path". FR-038 exposes only `up`/`down`, and the deviation is unrecorded. Record it pending an arch ruling; no accessor without one | spec/checked_package/functional/FR-038-consume-checked-package-v2.md |
| FND-002 | low | AC-597 omits the breaking change: `CheckedBodyLocation` loses `path` and the row bytes change | spec/checked_package/functional/FR-038-consume-checked-package-v2.md |
| FND-003 | low | TC-491's steps and expected results are misaligned. BL-S5/S6 have no procedure step, and BL-D1/D2 are absent | spec/checked_package/functional/FR-038-consume-checked-package-v2.md |
| FND-004 | low | The tests.md TC-491 row still says BL-01..R6 "are not implemented" before its new "implemented" sentence | spec/checked_package/functional/FR-038-consume-checked-package-v2.md |

## Original finding rows (verbatim)

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | medium | exceeds | FSpec says "The typed read returns the rebuilt path". FR-038 exposes only `up`/`down`, and the deviation is unrecorded. Record it pending an arch ruling; no accessor without one |
| FND-002 | low | other | AC-597 omits the breaking change: `CheckedBodyLocation` loses `path` and the row bytes change |
| FND-003 | low | other | TC-491's steps and expected results are misaligned. BL-S5/S6 have no procedure step, and BL-D1/D2 are absent |
| FND-004 | low | other | The tests.md TC-491 row still says BL-01..R6 "are not implemented" before its new "implemented" sentence |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The architecture scope requires the unchanged upstream rebuilt-path contract. FR-038 now explicitly specifies a lazy typed body_locations cursor and next_location path views; implementation returns a borrowed rebuilt path, with linear retained state and caller work limits. Compact rows remain available as storage, and no longer substitute for the required path API. |
| FND-002 | fixed | AC-597 explicitly records removal of CheckedBodyLocation.path and the canonical row byte change. FR-038 also names that breaking change. |
| FND-003 | fixed | TC-491 now separates ordinary refusals, relative/canonical and malformed cases, typed writer round-trip, and deep/scaling work procedure steps, each with matching expected results. BL-D1/D2 and exact/one-short are explicit. |
| FND-004 | fixed | The TC-491 status row now consistently says Implemented, including compact rows, all vectors, lazy typed paths and exact/one-short work. |

## Scope and evidence

Base checklist applied to changed FR-038/FR-019 statements, AC-597 and TC-491/status corrections. The added architecture requirements agree with merged FSpec composite order, shared occurrence/term rank and unchanged typed rebuilt-path semantics. No new identity scheme, vendored vector, digest ledger, exact prose tool pin, scope conflict or ambiguous refusal distinction is introduced. No applicable AssuranceProfile was found under spec/. Other spec lenses and an enforced human-acceptance workflow are outside this original-review disposition; none is represented as completed. Optional semantic review not run. Existing grammar warnings are outside the changed criteria.

Final supplied evidence was read, not re-executed: 20 focused tests passed; model-library Clippy, formatting, changed-spec validation and diff whitespace checks passed. No Cargo command was run by this reviewer. Final logs and exact commands: /private/tmp/ir1110-final-handoff.md and its /private/tmp/ir1110-final-*.log files. D1 lazy exact work 1,762,146, peak bytes 7,642,896; D2 3,231,080, peak bytes 7,145,824; both release cursor allocations on drop, admit 100,000 rows and pass kth-path, exact/one-short, zero-work, cancellation and fused exhaustion controls. Body n/2n work 4,618/9,218; protocol 4,007/8,007. Restored-rescan negative controls fail at monitor work 1,000,001 and body/protocol scaling 18,852/67,702 and 18,151/66,301. Sources: /private/tmp/ir1110-monitor-rescan-red.log and /private/tmp/ir1110-scaling-rescan-red.log. Final vectors are read in place from /private/tmp/ir1110-fspec-247a3941, merged FSpec #784. No worktree, source edits, push, merge, target removal or full gate was performed.

## Examined scope and machine-readable dispositions

```yaml
{
  "clean": true,
  "round": 1,
  "method": "spec-review/base",
  "original_review": {
    "id": "SR-17940486709564847048143",
    "comment_id": "dd9d2ec1-587f-46cf-b6f8-5f06fe26fad5",
    "head": "2e68276519dbce5845d35dd76a06f12bb830d96e",
    "model": "claude-opus-5-5"
  },
  "reviewed_head": "8d4d8620a159fe2fa32693c3bb52978f2811ae05",
  "base": "dacfac6b371b89f1942849e3c502ad01fbb68d2c",
  "scope": [
    {
      "id": "FR-038-AC-597",
      "path": "spec/checked_package/functional/FR-038-consume-checked-package-v2.md",
      "role": "examined",
      "excerpt": "The lazy typed body-location cursor returns rebuilt paths in walk order with linear state, caller-prepaid work, exact/one-short exhaustion and cancellation."
    },
    {
      "id": "FR-019-AC-29",
      "path": "spec/model/functional/FR-019-rust-library-interface.md",
      "role": "examined",
      "excerpt": "A public consumer constructs a row from typed `CheckedNodeId`, `CheckedOccurrence`, `term_ordinal`, function `CheckedNodeId`, `name`, `index`, up count and down list without importing a private wire type"
    },
    {
      "id": "FR-322-AC-513",
      "path": "filament-specification/spec/objects/interfaces/FR-322-checked-package-artifact.md",
      "role": "context_only",
      "excerpt": "Composite values are walked through and are not terms."
    }
  ],
  "dispositions": [
    {
      "fnd": "FND-001",
      "outcome": "fixed",
      "reason": "The architecture scope requires the unchanged upstream rebuilt-path contract. FR-038 now explicitly specifies a lazy typed body_locations cursor and next_location path views; implementation returns a borrowed rebuilt path, with linear retained state and caller work limits. Compact rows remain available as storage, and no longer substitute for the required path API.",
      "after_excerpt": "`CheckedPackageV2::body_locations(function, work_limit)` SHALL return a lazy\ntyped cursor in body-walk order. Each `next_location` view borrows the exact\ncaptured row and its rebuilt path, keeping only the current path in memory."
    },
    {
      "fnd": "FND-002",
      "outcome": "fixed",
      "reason": "AC-597 explicitly records removal of CheckedBodyLocation.path and the canonical row byte change. FR-038 also names that breaking change.",
      "after_excerpt": "The public `CheckedBodyLocation.path` field is removed and the canonical row bytes change."
    },
    {
      "fnd": "FND-003",
      "outcome": "fixed",
      "reason": "TC-491 now separates ordinary refusals, relative/canonical and malformed cases, typed writer round-trip, and deep/scaling work procedure steps, each with matching expected results. BL-D1/D2 and exact/one-short are explicit.",
      "after_excerpt": "4. Read BL-R9, BL-R10 and BL-R11, BL-S5 and BL-S6, and the additional missing-up, unsafe-up and mixed-format location cases."
    },
    {
      "fnd": "FND-004",
      "outcome": "fixed",
      "reason": "The TC-491 status row now consistently says Implemented, including compact rows, all vectors, lazy typed paths and exact/one-short work.",
      "after_excerpt": "Implemented: compact body-location rows, public encoder input, body-walk coverage and exact refusal loci; BL-01..BL-06, BL-R1..BL-R11, BL-S5/S6 and BL-D1/D2 (FR-038-AC-597). The lazy typed cursor returns rebuilt paths with linear state and exact/one-short work accounting."
    }
  ],
  "findings": []
}
```

## Original review record (verbatim)

<!-- reviewer repo=agent-ix/filament-contract-ir visibility=public quoin=0.29.1 module=spec-artifacts-process@b82e42e893612b4b4ee94127714b4577ee893502 id=SR-17940486709564847048143 method=spec-review/base lang=spec pr=filament-contract-ir#1110 model=claude-opus-5-5 run=qsl2691-pr1110-opus-review-2e682765 -->

Base spec review of PR #1110 at 2e682765. AC-597 numbering is clean, which clears #1108's AC-591 collision. The FR-019 changes are clean, with no stray inventory backticks.

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | medium | exceeds | FSpec says "The typed read returns the rebuilt path". FR-038 exposes only `up`/`down`, and the deviation is unrecorded. Record it pending an arch ruling; no accessor without one |
| FND-002 | low | other | AC-597 omits the breaking change: `CheckedBodyLocation` loses `path` and the row bytes change |
| FND-003 | low | other | TC-491's steps and expected results are misaligned. BL-S5/S6 have no procedure step, and BL-D1/D2 are absent |
| FND-004 | low | other | The tests.md TC-491 row still says BL-01..R6 "are not implemented" before its new "implemented" sentence |

+++ [reviewer data]

```yaml
clean: false
scope:
  - {id: FR-038, path: "spec/checked_package/functional/FR-038-consume-checked-package-v2.md", role: examined, excerpt: "The rebuild charges one unit per row and one per entry removed, so the reader's work is linear in the body walk and the entries the rows hold. The typed view exposes each admitted row with its function, name, index, up and down."}
  - {id: FR-038-AC-597, path: "spec/checked_package/functional/FR-038-consume-checked-package-v2.md", role: examined, excerpt: "Implemented (QSL-2691). Consumes FSpec FR-322-AC-510. CheckedBodyLocation is {origin, up, down}: up a safe non-negative integer, down a list of CheckedKeyOrdinal."}
  - {id: FR-019-AC-29, path: "spec/model/functional/FR-019-rust-library-interface.md", role: examined, excerpt: "A public consumer constructs a row from typed CheckedNodeId, CheckedOccurrence, term_ordinal, function CheckedNodeId, name, index, up count and down list without importing a private wire type"}
  - {id: TC-491, path: "spec/checked_package/matrix/TC-491-function-body-location-rows-read-at-the-v2-reader.md", role: examined, excerpt: "3. Read BL-R1 to BL-R6, BL-R9 and BL-R10. ... Step 4: BL-R9 and BL-R10 refused invalid_package/invalid-value at the row's location/up; BL-S5 and BL-S6 malformed_wire at the row's location; refused at /checked_term_data/body_locations; malformed_wire at that kind."}
  - {id: "tests.md TC-491 row", path: "spec/checked_package/matrix/tests.md", role: examined, excerpt: "🚧 Planned (QSL-2625): the fifth checked_term_data array body_locations, its typed row and encoder input, the wire body walk and its refusals over FSpec BL-01..BL-03 and BL-R1..BL-R6 are not implemented. The relative {origin, up, down} row, BL-R9, BL-R10, BL-S5 and BL-S6 are implemented (QSL-2691, FR-038-AC-597)."}
findings:
  - fnd: FND-001
    severity: medium
    confidence: high
    method: spec-review/base
    check: exceeds
    artifact_id: FR-038
    related_ids: [FR-038-AC-597, FR-322-AC-510]
    path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
    lines: "5599-5600"
    excerpt: |-
      The rebuild charges one unit per row and one per entry removed, so the reader's work is linear in the body walk and the entries the rows hold. The typed view exposes each admitted row with its function, name, index, up and down.
    finding: "FSpec FR-322 Reader: 'The typed read returns the rebuilt path.' FR-038 exposes only up/down; the rebuilt path is computed and dropped, and the deviation is not recorded as one. No accessor may be added absent an arch ruling: record the deviation in FR-038, AC-597 and the TC-491 matrix row citing the pending ruling; add a public borrowed accessor only if arch rules for it."
  - fnd: FND-002
    severity: low
    confidence: high
    method: spec-review/base
    check: other
    artifact_id: FR-038-AC-597
    related_ids: [FR-038-AC-590]
    path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
    lines: "6387"
    excerpt: |-
      Implemented (QSL-2691). Consumes FSpec FR-322-AC-510. CheckedBodyLocation is {origin, up, down}: up a safe non-negative integer, down a list of CheckedKeyOrdinal.
    finding: "AC-597 does not state the breaking change (public CheckedBodyLocation loses path, gains up and down; canonical bytes of body_locations rows change), as sibling AC-590 does for its field change."
  - fnd: FND-003
    severity: low
    confidence: high
    method: spec-review/base
    check: other
    artifact_id: TC-491
    related_ids: [FR-038-AC-597]
    path: spec/checked_package/matrix/TC-491-function-body-location-rows-read-at-the-v2-reader.md
    lines: "27-45"
    excerpt: |-
      3. Read BL-R1 to BL-R6, BL-R9 and BL-R10. ... Step 4: BL-R9 and BL-R10 refused invalid_package/invalid-value at the row's location/up; BL-S5 and BL-S6 malformed_wire at the row's location; refused at /checked_term_data/body_locations; malformed_wire at that kind.
    finding: "TC-491 procedure step 3 reads BL-R9/R10 but their outcomes sit under Expected Step 4 with the old carrier/kind text; BL-S5/S6 have an expected result but no procedure step; BL-D1/BL-D2, cited by AC-597 via TC-491, appear in neither; the fourth procedure item still begins with a bare '.'."
  - fnd: FND-004
    severity: low
    confidence: high
    method: spec-review/base
    check: other
    artifact_id: "tests.md TC-491 row"
    related_ids: [TC-491, FR-038-AC-597]
    path: spec/checked_package/matrix/tests.md
    lines: "24"
    excerpt: |-
      🚧 Planned (QSL-2625): the fifth checked_term_data array body_locations, its typed row and encoder input, the wire body walk and its refusals over FSpec BL-01..BL-03 and BL-R1..BL-R6 are not implemented. The relative {origin, up, down} row, BL-R9, BL-R10, BL-S5 and BL-S6 are implemented (QSL-2691, FR-038-AC-597).
    finding: "The tests.md TC-491 row still opens 'Planned ... are not implemented' for BL-01..BL-R6 (implemented on main) and then says the relative form is implemented; restate as one current status."
bindings: []
```

+++

