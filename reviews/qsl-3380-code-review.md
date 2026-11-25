---
id: SR-202611253380
title: "code-review of QSL-3380 PR #1179 (CertificateMember zero_weight)"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR #1179 at 1a4f0498c4; crates/filament-contract-model/src/certificate_member.rs"
review_set: subset
---

## Summary

Ticket: QSL-3380. One file changes. The `certificate_members!` table gains `ZeroWeight => "zero_weight"` between `Value` and `Zone`, and the FR-070-AC-1 unit test is renamed to `horizon_part_and_zero_weight_have_schema_spellings` with a third `(ZeroWeight, "zero_weight")` pair. The rust-review lane is folded in here. No cargo was run by the reviewer; gate evidence is the author's logs in gates-lead-conf/QSL-3380-*.log.

## Examined scope

- `certificate_members!` table (examined). The 65 head spellings, in order, equal the `CertificateRejected` malformed `member` enum at FSpec a6968fb8 and at FSpec origin/main 53fe180b element by element (scripted diff, no difference). FSpec FR-331's Certificate member row at a6968fb8 lists `zero_weight` between `value` and `zone`, and FR-412-AC-33 (TC-2115) is the requirement that introduces the member (FSpec commit c7340991). The CI ref 01f8dd31 has 63 members and lacks both `candidate_blocks` and `zero_weight`, so base main already diverged from it.
- Macro (context_only). One table generates the enum, `ALL` and the exhaustive `as_str` match, so no second list can drift.
- Other enumerators (examined). In this repo only `tests/conformance_fspec/certificate_member.rs` (schema comparison) and `tests/it/public_interface.rs` (owner, `ALL`, `as_str` and per-row names, derived from the macro) touch the type; neither needs an edit. Across sibling repos only FSL uses it (`fsl_replay` re-exports it); no exhaustive `match` on `CertificateMember` exists there. filament-contract-codegen and filament-driver do not reference it.
- Renamed test (examined). Still asserts the literal spellings `horizon`, `part`, `zero_weight` and membership in `ALL`; the FR-070-AC-1 trace tag stays meaningful (enumerate `ALL`, exact spelling via `as_str`).
- Gates (examined, author logs at head 1a4f0498): check all-targets all-features exit 0; model clippy -D warnings exit 0; fmt exit 0; model lib `certificate_member::` 3 passed; `it public_interface::` 25 passed; `conformance_fspec certificate_member::` 3 passed at head and red at base (the log does not record the FSpec revision, but its schema side contains `candidate_blocks`, so it is post-01f8dd31).

## Verdict

PASS. The member and its position match the owning FSpec schema and FR-331 list. Merging cannot turn contract-ir CI red: ci.yml runs fmt, `make lint` and `make test` (`--all-targets`), and `conformance_fspec` is `test = false`, so the 01f8dd31 schema comparison never runs in CI; the unit and public-interface tests that do run passed at head.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
