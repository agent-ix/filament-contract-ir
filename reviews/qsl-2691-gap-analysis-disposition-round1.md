---
id: SR-967175026302197768900474181941935084512
title: "QSL-2691 PR 1110 gap-analysis disposition round 1"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; QSL-2691; PR 1110; original findings and architecture additions"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-038", type: references }
---

## Summary

Fix-round 1 disposition of the existing Titan/Opus gap-analysis review, original head 2e68276519dbce5845d35dd76a06f12bb830d96e. Reviewed frozen head 8d4d8620a159fe2fa32693c3bb52978f2811ae05 on merged binder base dacfac6b371b89f1942849e3c502ad01fbb68d2c. Original SR SR-17940486709564847048142 is unavailable as a file; original private Linear comment [2628a1e0-c565-418e-9f5a-283d98442785](https://linear.app/agent-ix/issue/QSL-2691/body-location-rows-grow-quadratically-with-body-depth-give-them-a#comment-2628a1e0) is the authoritative retained record. This fresh-ID artifact preserves its findings and appends dispositions; it does not replace the original review or claim a new first review.

## Tracker disposition record

Private Linear disposition comment: fb846985-ca94-486e-a7b7-b2bab9f0e6a8. Read-back passed: exactly one marked comment; real YAML parser accepted the returned data; the disposition ids exactly match the original unresolved finding set; all fixed outcomes carry nonempty after_excerpt text of at most 600 characters. The has-review label was already present. This artifact and comment use fresh SR identity while retaining the original SR and comment provenance above.

## Verdict

**PASS — CLEAR for this bounded disposition.** Every original finding below is fixed. No new finding in the fix and requested architecture scope. Aggregate merge gates remain the lead's responsibility. FSL producer integration is separately owned and does not block this reader-first integration.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | AC-597's BL-D1/BL-D2 linear-work clause is untested at the reader. `deep_bodies_rebuild_in_linear_work` only exercises `step` on synthetic rows, and through the reader IR-1321 refuses both | tests/conformance_fspec/location_scaling.rs |
| FND-002 | medium | Vector evidence is at FSpec a34bb5cc, not #778's head 06e768a1, whose BL packages carry the `binder` member (#1112). Re-verify #1110 and #1112 together at 06e768a1 | tests/conformance_fspec/location_scaling.rs |
| FND-003 | medium | IR-1321 has no regression test (monitor-root shape, 100k protocol rows) | tests/conformance_fspec/location_scaling.rs |

## Original finding rows (verbatim)

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | high | test-intent | AC-597's BL-D1/BL-D2 linear-work clause is untested at the reader. `deep_bodies_rebuild_in_linear_work` only exercises `step` on synthetic rows, and through the reader IR-1321 refuses both |
| FND-002 | medium | coverage | Vector evidence is at FSpec a34bb5cc, not #778's head 06e768a1, whose BL packages carry the `binder` member (#1112). Re-verify #1110 and #1112 together at 06e768a1 |
| FND-003 | medium | coverage | IR-1321 has no regression test (monitor-root shape, 100k protocol rows) |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Public reader tests now admit both actual 100,000-depth body shapes within linear work allowances, then traverse the public lazy cursor with first/middle/last path oracles, exact/one-short, fused exhaustion, cancellation and measured linear peak storage. Reader n/2n exact-work tests fail with the original rescans restored. |
| FND-002 | fixed | Final vector run used merged FSpec 247a394157b58843643a61dfde16280c25d733c0 (#784), including binder links, against reader head based on merged binder reader dacfac6b. All four focused body-vector tests passed, exercising all positives and refusal vectors, BL-04/05/06 and BL-R11. No producer/consumer dependency bump is required prematurely. |
| FND-003 | fixed | The genuine monitor-root package regression preserves selected model evidence and admits at work=1,000,000. The restored-rescan countercheck fails at 1,000,001. Protocol scaling covers 100,000 rows, and body/protocol least-admitting limits both test exact and one-short. |

## Scope and evidence

This is the bounded disposition of the original gap findings, not a new repository-wide assurance verdict. New public cursor behavior maps to FR-038/AC-597 and FR-019; new indexing behavior maps to the indexed-join statement of FR-038. Changed tests exercise the real reader/cursor, with no replacement implementation or unconditional skip. No source/test stub was found in the changed scope. The optional whole-repository semantic audit was not requested and was not run. Plan completion: not assessed. Reconciliation: quire matrix (quire 0.52.0, engine 0.62.2), no run evidence read; both changed ACs tagged (597:14 binders; 019-29:1). Whole-repository static counts are tagged 1150, untagged 115, ignored-only 1, method-without-symbol 26, retired 8; --strict exits 1 also reporting 87 unrelated design gaps and symbol-collision diagnostics. These are a recorded global limitation, not a new gate or clean whole-repository claim. The already-started coverage report completed successfully; no further global scans or repairs were run.

Final supplied evidence was read, not re-executed: 20 focused tests passed; model-library Clippy, formatting, changed-spec validation and diff whitespace checks passed. No Cargo command was run by this reviewer. Final logs and exact commands: /private/tmp/ir1110-final-handoff.md and its /private/tmp/ir1110-final-*.log files. D1 lazy exact work 1,762,146, peak bytes 7,642,896; D2 3,231,080, peak bytes 7,145,824; both release cursor allocations on drop, admit 100,000 rows and pass kth-path, exact/one-short, zero-work, cancellation and fused exhaustion controls. Body n/2n work 4,618/9,218; protocol 4,007/8,007. Restored-rescan negative controls fail at monitor work 1,000,001 and body/protocol scaling 18,852/67,702 and 18,151/66,301. Sources: /private/tmp/ir1110-monitor-rescan-red.log and /private/tmp/ir1110-scaling-rescan-red.log. Final vectors are read in place from /private/tmp/ir1110-fspec-247a3941, merged FSpec #784. No worktree, source edits, push, merge, target removal or full gate was performed.

## Examined scope and machine-readable dispositions

```yaml
{
  "clean": true,
  "round": 1,
  "method": "gap-analysis",
  "original_review": {
    "id": "SR-17940486709564847048142",
    "comment_id": "2628a1e0-c565-418e-9f5a-283d98442785",
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
      "reason": "Public reader tests now admit both actual 100,000-depth body shapes within linear work allowances, then traverse the public lazy cursor with first/middle/last path oracles, exact/one-short, fused exhaustion, cancellation and measured linear peak storage. Reader n/2n exact-work tests fail with the original rescans restored.",
      "after_excerpt": "fn bl_d1_one_hundred_thousand_left_nested_applications_admit_at_the_reader() {\n    let result = read_owned(chain(100_000, true), 100_000_000);"
    },
    {
      "fnd": "FND-002",
      "outcome": "fixed",
      "reason": "Final vector run used merged FSpec 247a394157b58843643a61dfde16280c25d733c0 (#784), including binder links, against reader head based on merged binder reader dacfac6b. All four focused body-vector tests passed, exercising all positives and refusal vectors, BL-04/05/06 and BL-R11. No producer/consumer dependency bump is required prematurely.",
      "after_excerpt": "for case in cases[\"positive\"].as_array().unwrap() {\n        let id = case[\"id\"].as_str().unwrap();\n        let package = package_with(&cases, case);\n        let result = read(&package);"
    },
    {
      "fnd": "FND-003",
      "outcome": "fixed",
      "reason": "The genuine monitor-root package regression preserves selected model evidence and admits at work=1,000,000. The restored-rescan countercheck fails at 1,000,001. Protocol scaling covers 100,000 rows, and body/protocol least-admitting limits both test exact and one-short.",
      "after_excerpt": "fn monitor_root_location_join_admits_under_one_million_work() {"
    }
  ],
  "findings": []
}
```

## Original review record (verbatim)

<!-- reviewer repo=agent-ix/filament-contract-ir visibility=public quoin=0.29.1 module=spec-artifacts-process@b82e42e893612b4b4ee94127714b4577ee893502 id=SR-17940486709564847048142 method=gap-analysis lang=rust pr=filament-contract-ir#1110 model=claude-opus-5-5 run=qsl2691-pr1110-opus-review-2e682765 -->

Gap analysis of PR #1110 at 2e682765 (FR-038-AC-597 against FSpec #778 06e768a1 vectors). This was a manual AC-to-test check; plan completion was not assessed and no tests were run.

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | high | test-intent | AC-597's BL-D1/BL-D2 linear-work clause is untested at the reader. `deep_bodies_rebuild_in_linear_work` only exercises `step` on synthetic rows, and through the reader IR-1321 refuses both |
| FND-002 | medium | coverage | Vector evidence is at FSpec a34bb5cc, not #778's head 06e768a1, whose BL packages carry the `binder` member (#1112). Re-verify #1110 and #1112 together at 06e768a1 |
| FND-003 | medium | coverage | IR-1321 has no regression test (monitor-root shape, 100k protocol rows) |

+++ [reviewer data]

```yaml
clean: false
scope:
  - {id: FR-038-AC-597, path: "spec/checked_package/functional/FR-038-consume-checked-package-v2.md", role: examined, excerpt: "On FSpec's body_locations vectors BL-01 to BL-03 are admitted; BL-R9 (up beyond the previous path) and BL-R10 (up not least) are invalid_package/invalid-value at /checked_term_data/body_locations/8/location/up; BL-S5 ({origin, path}) and BL-S6 (up without down) are malformed_wire. A path of 100,000 left-nested terms (BL-D1) and 100,000 nested lets (BL-D2) rebuild in work linear in the body."}
  - {id: FR-019-AC-29, path: "spec/model/functional/FR-019-rust-library-interface.md", role: examined, excerpt: "A public consumer constructs a row from typed CheckedNodeId, CheckedOccurrence, term_ordinal, function CheckedNodeId, name, index, up count and down list without importing a private wire type, writes FSpec BL-01, and reads the same typed values back from the strictly admitted package"}
  - {id: TC-491, path: "spec/checked_package/matrix/TC-491-function-body-location-rows-read-at-the-v2-reader.md", role: context_only, excerpt: "Verify FR-038-AC-581, FR-038-AC-597, FR-019-AC-29 and FR-051-AC-5 at the public V2 reader and encoder over FSpec's checked-term-data-vectors.json body_locations cases"}
findings:
  - fnd: FND-001
    severity: high
    confidence: high
    method: gap-analysis
    check: test-intent
    artifact_id: FR-038-AC-597
    related_ids: [FR-322-AC-510, TC-491, IR-1321]
    path: crates/filament-contract-model/src/checked_package/v2/body_locations.rs
    lines: "412-447"
    excerpt: |-
      On FSpec's body_locations vectors BL-01 to BL-03 are admitted; BL-R9 (up beyond the previous path) and BL-R10 (up not least) are invalid_package/invalid-value at /checked_term_data/body_locations/8/location/up; BL-S5 ({origin, path}) and BL-S6 (up without down) are malformed_wire. A path of 100,000 left-nested terms (BL-D1) and 100,000 nested lets (BL-D2) rebuild in work linear in the body.
    finding: "No test exercises AC-597's BL-D1/BL-D2 clause at the reader. deep_bodies_rebuild_in_linear_work calls step() on synthetic rows: its BL-D1 half asserts charged == 0 (true by construction, all up = 0) and its BL-D2 half sums step's own up charge, so it cannot fail on a reader-level quadratic and never reads the deep_body vectors. Through the strict reader BL-D1/BL-D2 hit the IR-1321 rescan and are refused, while FSpec expects admitted. Required: reader-level BL-D1/BL-D2 tests per the vector rule, admitted under limits linear in the body, plus a scaling assertion that fails on the rescan."
  - fnd: FND-002
    severity: medium
    confidence: medium
    method: gap-analysis
    check: coverage
    artifact_id: FR-038-AC-597
    related_ids: [FR-322-AC-510, FR-322-AC-512]
    path: tests/conformance_fspec/body_locations.rs
    lines: "45-121"
    excerpt: |-
      On FSpec's body_locations vectors BL-01 to BL-03 are admitted; BL-R9 (up beyond the previous path) and BL-R10 (up not least) are invalid_package/invalid-value at /checked_term_data/body_locations/8/location/up
    finding: "Vector evidence is at a stale FSpec commit: PR body says tested against #778 a34bb5cc; #778 head is 06e768a1, whose BL packages' #ieee_operand let bindings carry a binder member (4 occurrences), which this head's reader does not take (binder #1112 does). The vector test is expected to fail on #1110 alone at #778 head (not run). Before merge, run conformance_fspec body_locations at FSpec 06e768a1 with #1110+#1112 together and record the commits and result."
  - fnd: FND-003
    severity: medium
    confidence: high
    method: gap-analysis
    check: coverage
    artifact_id: FR-038-AC-597
    related_ids: [IR-1321]
    path: crates/filament-contract-model/src/checked_package/v2/protocol_locations.rs
    lines: "173-256"
    excerpt: |-
      A path of 100,000 left-nested terms (BL-D1) and 100,000 nested lets (BL-D2) rebuild in work linear in the body.
    finding: "IR-1321 has no regression test: nothing shows a monitor-root-shaped package or a 100,000-row protocol_locations package that was over budget before and is admitted after. Code-review SR FND-001 R7 lists the required tests."
bindings:
  - {test_id: every_published_body_location_vector_runs_through_the_strict_reader, ac_id: FR-038-AC-597, trace: correct}
  - {test_id: the_earlier_whole_path_form_and_a_partial_relative_form_are_malformed_wire, ac_id: FR-038-AC-597, trace: correct}
  - {test_id: deep_bodies_rebuild_in_linear_work, ac_id: FR-038-AC-597, trace: correct}
```

+++

