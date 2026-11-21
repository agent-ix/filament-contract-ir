---
id: SR-967175026302197768900474181941935084511
title: "QSL-2691 PR 1110 code-review disposition round 1"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; QSL-2691; PR 1110; original findings and architecture additions"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-038", type: references }
---

## Summary

Fix-round 1 disposition of the existing Titan/Opus code-review review, original head 2e68276519dbce5845d35dd76a06f12bb830d96e. Reviewed frozen head 8d4d8620a159fe2fa32693c3bb52978f2811ae05 on merged binder base dacfac6b371b89f1942849e3c502ad01fbb68d2c. Original SR SR-17940486709564847048141 is unavailable as a file; original private Linear comment [9e23741a-b66f-4b74-b483-72e640cf9309](https://linear.app/agent-ix/issue/QSL-2691/body-location-rows-grow-quadratically-with-body-depth-give-them-a#comment-9e23741a) is the authoritative retained record. This fresh-ID artifact preserves its findings and appends dispositions; it does not replace the original review or claim a new first review.

## Tracker disposition record

Private Linear disposition comment: 13a0013e-6745-4c36-bb0c-434346ec33c5. Read-back passed: exactly one marked comment; real YAML parser accepted the returned data; the disposition ids exactly match the original unresolved finding set; all fixed outcomes carry nonempty after_excerpt text of at most 600 characters. The has-review label was already present. This artifact and comment use fresh SR identity while retaining the original SR and comment provenance above.

## Verdict

**PASS — CLEAR for this bounded disposition.** Every original finding below is fixed. No new finding in the fix and requested architecture scope. Aggregate merge gates remain the lead's responsibility. FSL producer integration is separately owned and does not block this reader-first integration.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | IR-1321 not fixed: `validate_rows` rescans every node and the whole source_map for each row, so work is O(B·N+B·S). BL-D1/BL-D2 and FSL monitor_root exhaust the work bound | crates/filament-contract-model/src/checked_package/v2/protocol_locations.rs |
| FND-002 | medium | The per-row `node.occurrences` scan is k² for a node reached along k references | crates/filament-contract-model/src/checked_package/v2/protocol_locations.rs |
| FND-003 | medium | An `up` above the safe range refuses at `location/up`, while FR-038 and FR-322 say `location`; no malformed-location locus is asserted | crates/filament-contract-model/src/checked_package/v2/protocol_locations.rs |

## Original finding rows (verbatim)

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | high | code-bug | IR-1321 not fixed: `validate_rows` rescans every node and the whole source_map for each row, so work is O(B·N+B·S). BL-D1/BL-D2 and FSL monitor_root exhaust the work bound |
| FND-002 | medium | code-bug | The per-row `node.occurrences` scan is k² for a node reached along k references |
| FND-003 | medium | soundness | An `up` above the safe range refuses at `location/up`, while FR-038 and FR-322 say `location`; no malformed-location locus is asserted |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | R1–R9 inspected. Nonempty arrays build prepaid sorted node, source and occurrence indexes once; first equal node uses lowest position. Application and occurrence lookups each charge once per row. Body walk reuses the node index. Empty joins allocate and charge nothing. Refusal ordering is unchanged. Both scaling tests and genuine monitor regression distinguish the original rescan. |
| FND-002 | fixed | Each row now binary-searches the once-built (node position, occurrence) index. The 1,000-occurrence public-reader regression admits at 17,030 exact work and refuses one short. |
| FND-003 | fixed | Unsafe up returns malformed_wire at the whole location; intake normalizes missing up/down, legacy path and mixed path-relative shape faults there too. All five cases assert that exact locus. Canonical-delta semantic failures remain invalid_package/invalid-value at location/up. |

## Scope and evidence

Rust lane included. Reviewed new cursor construction and lending lifetime, exact captured-row reference, one-current-path storage, geometric prepaid task/path capacity, wide-ordinal bytes before clone, typed work failure, fused termination and cancellation by drop. Retained identity/source joins use admitted typed keys; no provenance is fabricated. The original accepted delta algorithm remains unchanged. New composite traversal follows stored record/tuple components and revisits shared nodes. R8 definitions are unchanged relative to merged binder base dacfac6b; no FSL budget, Cargo manifest/lock, CI lane or gate change is present. Existing full-body-walk allocation machinery was not re-audited as a new scope; new join allocations and new cursor-owned allocations were inspected before request.

Final supplied evidence was read, not re-executed: 20 focused tests passed; model-library Clippy, formatting, changed-spec validation and diff whitespace checks passed. No Cargo command was run by this reviewer. Final logs and exact commands: /private/tmp/ir1110-final-handoff.md and its /private/tmp/ir1110-final-*.log files. D1 lazy exact work 1,762,146, peak bytes 7,642,896; D2 3,231,080, peak bytes 7,145,824; both release cursor allocations on drop, admit 100,000 rows and pass kth-path, exact/one-short, zero-work, cancellation and fused exhaustion controls. Body n/2n work 4,618/9,218; protocol 4,007/8,007. Restored-rescan negative controls fail at monitor work 1,000,001 and body/protocol scaling 18,852/67,702 and 18,151/66,301. Sources: /private/tmp/ir1110-monitor-rescan-red.log and /private/tmp/ir1110-scaling-rescan-red.log. Final vectors are read in place from /private/tmp/ir1110-fspec-247a3941, merged FSpec #784. No worktree, source edits, push, merge, target removal or full gate was performed.

## Examined scope and machine-readable dispositions

```yaml
{
  "clean": true,
  "round": 1,
  "method": "code-review",
  "original_review": {
    "id": "SR-17940486709564847048141",
    "comment_id": "9e23741a-b66f-4b74-b483-72e640cf9309",
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
      "reason": "R1–R9 inspected. Nonempty arrays build prepaid sorted node, source and occurrence indexes once; first equal node uses lowest position. Application and occurrence lookups each charge once per row. Body walk reuses the node index. Empty joins allocate and charge nothing. Refusal ordering is unchanged. Both scaling tests and genuine monitor regression distinguish the original rescan.",
      "after_excerpt": "let mut node_index = stage_storage::vec(meter, &site, nodes.len())?;\n    node_index.extend(nodes.iter().enumerate().map(|(i, node)| (&node.node_id, i)));\n    node_index.sort_unstable();"
    },
    {
      "fnd": "FND-002",
      "outcome": "fixed",
      "reason": "Each row now binary-searches the once-built (node position, occurrence) index. The 1,000-occurrence public-reader regression admits at 17,030 exact work and refuses one short.",
      "after_excerpt": "if occurrences\n            .binary_search(&(position, row.occurrence()))\n            .is_err()\n        {\n            return Err(invalid(meter, index, here.key(\"occurrence\")));\n        }"
    },
    {
      "fnd": "FND-003",
      "outcome": "fixed",
      "reason": "Unsafe up returns malformed_wire at the whole location; intake normalizes missing up/down, legacy path and mixed path-relative shape faults there too. All five cases assert that exact locus. Canonical-delta semantic failures remain invalid_package/invalid-value at location/up.",
      "after_excerpt": "if self.location.up > MAX_SAFE {\n            return Err(malformed(meter, index, at(array, index).key(\"location\")));\n        }"
    }
  ],
  "findings": []
}
```

## Original review record (verbatim)

<!-- reviewer repo=agent-ix/filament-contract-ir visibility=public quoin=0.29.1 module=spec-artifacts-process@b82e42e893612b4b4ee94127714b4577ee893502 id=SR-17940486709564847048141 method=code-review lang=rust pr=filament-contract-ir#1110 model=claude-opus-5-5 run=qsl2691-pr1110-opus-review-2e682765 -->

Code review (with the rust-review lane) of PR #1110 at 2e68276519dbce5845d35dd76a06f12bb830d96e, against FSpec #778 at 06e768a1. Verdict: **NOT MERGEABLE**. The `{up, down}` rebuild (`step`, visit/row pairing, walk order) is correct and linear. But the shared row join is quadratic (IR-1321), so the reader cannot admit BL-D1/BL-D2 or FSL's monitor_root packages. The IR-1321 fix must land in this PR; requirements R1-R9 are below. Focused tests were not run: this session's permission layer refused the `flock /tmp/cargo-slot.lock` wrapper, and cargo was not run without it.

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | high | code-bug | IR-1321 not fixed: `validate_rows` rescans every node and the whole source_map for each row, so work is O(B·N+B·S). BL-D1/BL-D2 and FSL monitor_root exhaust the work bound |
| FND-002 | medium | code-bug | The per-row `node.occurrences` scan is k² for a node reached along k references |
| FND-003 | medium | soundness | An `up` above the safe range refuses at `location/up`, while FR-038 and FR-322 say `location`; no malformed-location locus is asserted |

**IR-1321 fix requirements (FND-001), for the coder in this PR:**

- R1. When `rows` is non-empty, build once per `validate_rows` call: a node-id → position index and a source-map (`node_id`, `role`, `ordinal`) index. Reuse `node_positions` or `retained_index::PositionIndex`, with first-of-equal tie-break on position. Charge the build once (one unit per node and one per entry, plus the allocation) at `/checked_term_data/<array>`.
- R2. An empty or absent array builds nothing, so its accounting is unchanged.
- R3. Each row's lookup costs one unit at `/application` and one at `/occurrence`. The occurrence check is also O(1) or O(log) per row (FND-002).
- R4. Every refusal keeps its code, cause, locus and precedence: duplicate, order, node absent, wrong form, occurrence absent, unmapped, intake, then missing-predecessor.
- R5. Build one node index and share it with `body_locations::validate`, which builds `node_positions` again.
- R6. The fix covers both `protocol_locations` (IR-1321's 100k protocol cycle) and `body_locations`.
- R7. Reader-level tests:
  - (a) A monitor-root-shaped package that is Incomplete/Work at 1,000,000 units before the fix and admitted after.
  - (b) A 2n-vs-n scaling test that fails on the rescan.
  - (c) The same scaling test for `protocol_locations`.
  - (d) BL-D1 and BL-D2 admitted under limits linear in the body.
  - (e) Exact/one-short accounting: the least admitting limit admits, one unit less is Incomplete.
- R8. These stay green unmodified: `genuine_monitor_base_fold_is_admitted_without_removing_semantics`, `body_location_rows_are_read_and_refused_at_their_own_pointers`, `a_body_location_origin_name_is_charged_by_its_length`, `bounded_read_returns_complete_borrowed_path_only_at_the_exact_work_limit`. No loosened tests; any re-derived exact-work constant is named in the commit.
- R9. No FSL budget change. Mark IR-1321 In Progress. Record the indexed, linear join in FR-038.

+++ [reviewer data]

```yaml
clean: false
scope:
  - {id: "body_locations.rs::step", path: "crates/filament-contract-model/src/checked_package/v2/body_locations.rs", role: examined, excerpt: "let Some(keep) = usize::try_from(up).ok().and_then(|up| previous.len().checked_sub(up)) else { return Ok(false); }; charge_up(up)?; if let (Some(first), Some(removed)) = (down.first(), previous.get(keep)) { if first == removed { return Ok(false); } }"}
  - {id: "body_locations.rs::validate rebuild", path: "crates/filament-contract-model/src/checked_package/v2/body_locations.rs", role: examined, excerpt: "for application in &walk.order { let rank = visited.entry(*application).or_default(); let row_index = slots.get(&(position, *application)).and_then(|rows| rows.get(*rank)).copied()"}
  - {id: "body_locations.rs::Graph::walk", path: "crates/filament-contract-model/src/checked_package/v2/body_locations.rs", role: examined, excerpt: "if is_term(*kind, term) { *walk.counts.entry(position).or_default() += 1; walk.order.push(position); }"}
  - {id: "protocol_locations.rs::validate_rows", path: "crates/filament-contract-model/src/checked_package/v2/protocol_locations.rs", role: examined, excerpt: "for (position, node) in nodes.iter().enumerate() { work(meter, index, here.key(\"application\"), 1)?; ... } ... for entry in source_map { work(meter, index, here.key(\"occurrence\"), 1)?; mapped |= ..."}
  - {id: "protocol_locations.rs::check_origin (body)", path: "crates/filament-contract-model/src/checked_package/v2/protocol_locations.rs", role: examined, excerpt: "let up = at(array, index).key(\"location\").key(\"up\"); work(meter, index, up, 1)?; if self.location.up > MAX_SAFE { return Err(malformed(meter, index, up)); }"}
  - {id: "checked_term_data.rs::CheckedBodyLocation", path: "crates/filament-contract-model/src/checked_package/v2/checked_term_data.rs", role: examined, excerpt: "pub origin: CheckedBodyLocationOrigin, pub up: u64, #[serde(deserialize_with = \"super::document_de::exact_vec\")] pub down: Vec<CheckedKeyOrdinal>,"}
  - {id: FR-322-AC-510, path: "filament-specification spec/objects/interfaces/FR-322-checked-package-artifact.md", role: context_only, excerpt: "Once every row-level check and the missing-row check below pass, the reader SHALL rebuild each row's path in walk order and SHALL refuse invalid_package/invalid-value at /checked_term_data/body_locations/{i}/location/up ... The typed read returns the rebuilt path."}
findings:
  - fnd: FND-001
    severity: high
    confidence: high
    method: code-review
    check: code-bug
    artifact_id: "protocol_locations.rs::validate_rows"
    related_ids: [FR-038-AC-597, FR-322-AC-510, IR-1321]
    path: crates/filament-contract-model/src/checked_package/v2/protocol_locations.rs
    lines: "217-252"
    excerpt: |-
      for (position, node) in nodes.iter().enumerate() {
          work(meter, index, here.key("application"), 1)?;
          if &node.node_id == row.application() { selected = Some((position, node)); break; }
      }
      ...
      for entry in source_map {
          work(meter, index, here.key("occurrence"), 1)?;
          mapped |= &entry.node_id == row.application() && entry.role == row.occurrence().role && entry.ordinal == row.occurrence().ordinal;
      }
    finding: "IR-1321 not fixed: validate_rows, shared by protocol_locations and body_locations, scans semantic_graph.nodes and the whole source_map (no break) per row, O(B*N+B*S). BL-D1/BL-D2 (100,000 rows, expected admitted, AC-597 'linear') need ~1e10 units; FSL monitor_root packages exhaust the default 1,000,000. Fix requirements R1-R9 in the comment and SR file."
  - fnd: FND-002
    severity: medium
    confidence: medium
    method: code-review
    check: code-bug
    artifact_id: "protocol_locations.rs::validate_rows"
    related_ids: [IR-1321]
    path: crates/filament-contract-model/src/checked_package/v2/protocol_locations.rs
    lines: "235-242"
    excerpt: |-
      let mut occurrence_found = false;
      for occurrence in &node.occurrences {
          work(meter, index, here.key("occurrence"), 1)?;
          occurrence_found |= occurrence == row.occurrence();
      }
    finding: "The per-row node.occurrences scan (no break) is linear in the application's occurrences for each row; a node reached along k references has k occurrences and k rows, k^2 work. The IR-1321 fix must make this join O(1)/O(log) per row too."
  - fnd: FND-003
    severity: medium
    confidence: high
    method: code-review
    check: soundness
    artifact_id: "protocol_locations.rs::check_origin (body)"
    related_ids: [FR-038-AC-597, FR-322-AC-510]
    path: crates/filament-contract-model/src/checked_package/v2/protocol_locations.rs
    lines: "97-101"
    excerpt: |-
      let up = at(array, index).key("location").key("up");
      work(meter, index, up, 1)?;
      if self.location.up > MAX_SAFE {
          return Err(malformed(meter, index, up));
      }
    finding: "FR-038 says an up above the safe integer range is malformed_wire 'at the row's location' (FSpec FR-322: BL-S5/S6 at that row's location); code refuses at location/up, and the malformed-wire test asserts only refusal.code for all five cases. Align code or FR-038 text and assert the locus."
bindings:
  - {test_id: a_row_rebuilds_its_path_from_the_previous_one, ac_id: FR-038-AC-597, trace: correct}
  - {test_id: an_up_beyond_the_previous_path_is_refused, ac_id: FR-038-AC-597, trace: correct}
  - {test_id: an_up_that_is_not_the_least_is_refused, ac_id: FR-038-AC-597, trace: correct}
```

+++

