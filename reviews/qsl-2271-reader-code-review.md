---
id: SR-967175026302197768900474181941935084550
title: "QSL2271 PR 1121 code-review with rust-review lane: protocol correlation provenance strict reader"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; QSL-2271; PR 1121 head 50ba1ace53b6388ff1e41b329b6c0bfb33e25dcb (base origin/main a03c86b3b, clean merge); crates/filament-contract-model/src/checked_package/v2/{protocol_correlation.rs,mod.rs,intake.rs,document_de.rs}, crates/filament-contract-model/src/checked_package/shared.rs, crates/filament-contract-model/src/lib.rs, tests/conformance_fspec/{protocol_correlation.rs,main.rs}"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-378", type: references }
---

## Summary

Code review of PR 1121, with the rust-review lane folded in. Ticket: QSL-2271. The authority is FSpec FR-831, TC-2113, schema.json and protocol-correlation-vectors.json at merged filament-specification f6b0480cd (PR 794). I compared all 95 files under proposals/checked-package-v2 in the coder's /tmp/ir-q2271/fspec archive byte for byte with `git show f6b0480cd:<path>`: 0 differ.

Reviewer runs at the frozen head, both rc=0, on CPUs 18-31 with -j14 and the worktree's own target:
- `FILAMENT_SPECIFICATION_DIR=/tmp/ir-q2271/fspec cargo test --features test-support --test conformance_fspec protocol_correlation::correlation_`: 8 passed, 542 filtered out.
- `cargo test -p filament-contract-model --lib writer_preserves_correlation_rows`: 1 passed.

GitHub CI "Rust Checks" (run 44935869806) fails at its first step, Format. Clippy and the tests never ran on CI.

## Examined scope

- Carrier (examined, clean). The member is an optional top-level field in BorrowedWire and the envelope, placed beside `claim_names`. Rows are `deny_unknown_fields` with exactly 5 members. `canonical_index` and `source_ordinal` must be safe integers, and `alias` must be an Identifier. A `null` member refuses as malformed (intake.rs check_lossless_source). `[]` stays distinct from absence (None vs Some). The writer and old default encoders pass `None`, so they are backward compatible.
- Row pass (protocol_correlation.rs:454-570; examined, clean). The 1+B+L+Bp charge comes before any inspection, with Bp taken from the previous row's control (0 on row 0). Joins run in FR-831 order: own-package node by the retained binary-search index, eligible operator, declaration occurrence plus the full-key source_map entry, then `canonical_index < m`. An equal key is checked before order. Then the zero-contiguous ordinal check.
- Coverage pass (625-696; examined, clean apart from FND-003). Each control node is charged 1+2Bn(+O+m) before inspection, the m flag bytes are reserved before allocation, and the first uncovered index is retained in CorrelationCoverage.
- Typed read (examined, clean). It borrows rows and the whole source_map entry (`&wire.source_map[i]`) with no clones. Groups are stored as positions. The dependency answers from its own member only.
- Writer unit test and the 8 conformance tests (examined; see FND-001 and the gap-analysis SR).
- reference_bytes treats a `Node(_)` segment as 0 (examined, no finding). Wire deserialization never produces Node segments (shared.rs:705-772), so the arm cannot be reached from a row.

## Verdict

**FAIL**: one high and two medium findings. The reader logic matches FR-831 on every path the vectors drive. The exact storage vector is reinterpreted, CI is red, and an index outside the step is unmetered.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The test calls CO-B11 a PASS when the public strict read returns BudgetIncomplete. CO-B11 publishes `decision: admit` at B_storage+G+7. At that limit the reader denies the Retained/"result" box charged after the step. The test then raises the limit by the denied amount and re-reads. That adds uncontracted slack to both the limit and the oracle. FR-831-AC-7 says "Over the measured baseline the exact offset admits". | tests/conformance_fspec/protocol_correlation.rs:687-727; crates/filament-contract-model/src/checked_package/v2/budgeted_read.rs:224-235 |
| FND-002 | medium | CI Rust Checks is red at its Format step, because of rustfmt drift on lines this PR added. The drift is in the intake.rs UnknownMember match, both mod.rs `pub use protocol_correlation` blocks, the lib.rs re-export block and the conformance_fspec/main.rs `mod protocol_correlation` order. Clippy and tests never ran on CI for this head. | crates/filament-contract-model/src/checked_package/v2/intake.rs:401-408; crates/filament-contract-model/src/checked_package/v2/mod.rs:252-255; crates/filament-contract-model/src/lib.rs:274-278; tests/conformance_fspec/main.rs:103-104 |
| FND-003 | medium | The member-gated `control_positions` table is built at retention, outside the FR-831 pass. Building it does an unmetered scan of every node's kind and an unmetered node-id sort, and only its storage is charged. FR-831 says "no scan, index or table is built outside it", and the walk's node-id comparisons are paid only by each node's 2·Bn. | crates/filament-contract-model/src/checked_package/v2/protocol_correlation.rs:408-427; crates/filament-contract-model/src/checked_package/v2/mod.rs:1856-1860 |
| FND-004 | low | The view loop repeats the full-key source_map lookup once per group, after the row pass already did it under each row's charge. That is an extra uncharged binary search over source_index. Its `ok_or_else` refusal at the view site cannot be reached. | crates/filament-contract-model/src/checked_package/v2/protocol_correlation.rs:587-603 |

## Coverage

- Public API checked: encode_checked_package_v2_with_protocol_correlation_occurrences, ProtocolCorrelationRow, ProtocolCorrelationProvenance (Complete/Unavailable), ProtocolCorrelationTable, ProtocolCorrelationGroup, ProtocolCorrelationReadError, UnavailableCorrelationProvenance, the CheckedPackageV2 methods protocol_correlation_provenance() and protocol_correlation_clauses(), and the additive CheckedProtocolRefusalDetail::CorrelationCoverage. Re-exported in lib.rs.
- Every other constructor change is a test-only `correlation: Default::default()` initializer: callable, exact_projection, parameter_declaration, scalar_operands, comparison_operands, ceiling_tests and model_fields/tests.
- No hashes, pins, new dependencies, shims or vendored FSpec files were added.
- Mutant proof: the coder's logs (/tmp/ir-q2271/mut-*.log) show oldreader 8 FAIL, badjoin 1 FAIL, lostrows 8 FAIL and prevtext 1 FAIL. The oldreader mutant fails at the carrier intake: schema test at :390, refusal test at :224, positives at :128. Not rerun by me.
- Remaining limits: producer/consumer parity with FSL is not claimed.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | medium | CI Rust Checks now fails at Lint (run 44939885887). fsl-string-edge reports 7 unmarked occurrences at this head and 6 at origin/main 7e4f54095. The added one is protocol_correlation.rs:72 in safe_integer (branch-gating), so the PR worsens the known string-edge baseline, and clippy and tests again did not run on CI. | crates/filament-contract-model/src/checked_package/v2/protocol_correlation.rs:70-76 |

## Dispositions

Round 1 is checked at frozen head 6df4aaf2c86160772eb39f8c9d736f2b8a3998b7: fix c091064ca, then merge of origin/main 7e4f54095. Arch ruling /tmp/ir-qsl2271-cob11-ruling.txt chose option A: the generic result box is reserved before the last carrier step. Reviewer rerun: name-filtered protocol_correlation::correlation_ conformance (see report).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fixed in c091064ca; after_excerpt in the round-1 dispositions comment. |
| FND-002 | fixed | Fixed in c091064ca; after_excerpt in the round-1 dispositions comment. |
| FND-003 | fixed | Fixed in c091064ca; after_excerpt in the round-1 dispositions comment. |
| FND-004 | rejected | FR-831 step 5 requests the view only after the last row and charges only its read_storage block G. The native group record must hold the borrowed whole source_map entry. So the entry position cannot be held before G is paid without an uncharged allocation, and the one contract-compatible construction is to look it up again per group against the same immutable retained source index. That lookup reads only the group's first-row control text, which FR-831 B already prices ('the source_map lookup read this row's control text'). The arm join 3 makes unreachable is a typed refusal rather than a panic, as the no-panic library style requires. It is not a defect. |
