---
id: SR-967175026302197768900474181941935084587
title: "Gap analysis — filament-contract-ir PR #1128"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; repository matrix plus PR #1128 changed behavior; FR-070-AC-1..4; source crates/filament-contract-model/src; tests/conformance_fspec/certificate_member.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-contract-ir
    type: references
---

## Summary

This planless review checked the repository matrix and the PR’s changed behavior against FR-070 and FSpec FR-331-AC-46. All four FR-070 criteria remain statically tagged and the matrix summary is identical on origin/main; the gap in this diff is that FR-070-AC-1 and AC-4 declare a combined verification method the matrix cannot classify.

## Verdict

**CONDITIONAL** — the changed source behavior is owned and tested, but the two owning criteria use an unsupported combined verification value.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-070-AC-1 declares `Test, Inspection`, which the computed matrix reports as an unknown method, leaving its evidence unbound. | spec/model/functional/FR-070-certificate-member-vocabulary.md:82 |
| FND-002 | medium | FR-070-AC-4 declares `Test, Inspection`, which the computed matrix reports as an unknown method, leaving its evidence unbound. | spec/model/functional/FR-070-certificate-member-vocabulary.md:85 |

## Coverage

- Repository: `agent-ix/filament-contract-ir`; spec root `spec/`; source `crates/` and `src/`; tests `tests/`; planless.
- Reconciliation: `quoin matrix` 0.36.8, Quire 0.52.0 (engine 0.62.2), spec-artifacts-process v0.36.0. No active module declares a traceability model, so this is the matrix engine’s provisional repository scan.
- Plan completion: not assessed
- Criteria: 1,207 tagged; 115 untagged; 1 tagged only by ignored test; 26 method-without-symbol; 8 retired (1,357 total). All four FR-070 criteria are tagged. FR-070-AC-1 and AC-4 additionally report `unknown-method` for `Test, Inspection`.
- Evidence: no run evidence in the matrix. Separate Titan exact-head evidence confirms the schema/member tests ran and passed.
- Baseline: the same counts and FR-070 statuses appear at origin/main `990ee0c002e663c493d333d8c50adc38486ade0d`; no matrix status changed in this PR.
- Reverse gap: the two enum variants and schema walker are owned by FR-070; no new untraced behavior was added by the diff. No TODO/FIXME/XXX/todo!/unimplemented! marker occurs in changed files; no stub or skipped test was added.
- Semantic review: skipped (not requested).
