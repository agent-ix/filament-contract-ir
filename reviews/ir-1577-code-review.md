---
id: SR-967175026302197768900474181941935084586
title: "Code review — IR-1577 CertificateMember"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR #1128 head 5a6922bc3dc8a058705c9056adb73acc33b082d3; crates/filament-contract-model/src/certificate_member.rs; tests/conformance_fspec/certificate_member.rs; FR-070-AC-1..4; FSpec FR-331-AC-46 at 6a7611d8"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-070
    type: references
---

## Summary

Reviewed PR #1128 at `5a6922bc3dc8a058705c9056adb73acc33b082d3` against FR-070 and FSpec FR-331-AC-46. The variants are in the published sequence and their spellings are covered; one new schema-walker test does not prove collection through every conditional branch it claims to exercise.

## Verdict

**CONDITIONAL** — the `then`/`not` property-discovery path lacks an assertion that would fail if its recursion were removed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The conditional-schema test never puts a property under `then`/`not`, so it still passes if the walker stops collecting properties from those branches. | tests/conformance_fspec/certificate_member.rs:195-211 |

## Review scope

- FR-070, statement — examined: the model owns a target-neutral `CertificateMember` spelling vocabulary; FSL owns locus serialization.
- FR-070-AC-1 — examined: direct public API, `ALL`, `as_str()`, derives, and no FSL/backend dependency.
- FR-070-AC-2 — examined: schema-derived property/member/model set equality and malformed-source failure.
- FR-070-AC-3 — examined: ordered schema sequence equality.
- FR-070-AC-4 — examined: exact spelling per variant; no model serializer or locus shape.
- FSpec FR-331-AC-46 — context only: malformed member enum equals the schema property-name closure; order is not normative.

## Review methods

- Code review and Rust review: static diff, public API, trace tags, and test-oracle strength. No local build/tests were run; Titan executed the exact-head commands below.
- Duplication, vendoring, and integrity: no copied schema, new dependency, pin, or unrelated production behavior was added.
- Titan evidence at this head: both `--no-run` targets passed; `model_certificate_members_match_fspec_schema`, `certificate_property_walks_conditional_schemas`, `horizon_and_part_have_schema_spellings`, and `members_appear_once_with_distinct_plain_spellings` each ran once and passed; `cargo check --locked --workspace` passed. Logs: `/tmp/titan-ir1577-5a6922-logs/` on Titan.
- No applicable AssuranceProfile was found under `spec/`.
