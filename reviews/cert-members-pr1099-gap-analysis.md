---
id: SR-17940486709564847048112
title: "gap-analysis of QSL-2726 PR #1099 (CertificateMember 61 names)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1099 at 764127e4; FR-070; crates/filament-contract-model/src/certificate_member.rs; tests/conformance_fspec/certificate_member.rs"
review_set: subset
---

## Summary

Ticket: QSL-2726. Planless audit of FR-070 against the changed code. Plan completion: not assessed. This is a manual AC-to-test check. The quoin matrix was not regenerated because the PR adds no criterion or tag namespace.

## Examined scope

- FR-070-AC-1 (examined): `model_certificate_members_match_fspec_schema` covers the export, the `ALL`/`as_str` surface and the derives. It passed.
- FR-070-AC-2 (examined): the same test computes the certificate property closure over the seven definitions FR-070 "Inputs" names and checks it against the malformed enum. The closure equals the schema enum, so the new names come from those definitions and no new certificate-owned definition is needed. It passed.
- FR-070-AC-3 (examined): `assert_eq!(model, published, "model member sequence differs")` checks the order. It passed with the 61-name sequence.
- FR-070-AC-4 (examined): `as_str` is generated from the table literal. The unit test and the conformance test both cover it.
- Gate placement (context_only): `conformance_fspec` is `test = false` in Cargo.toml, so the schema oracle runs only in the explicit conformance lane. This was already the case before this PR.

## Verdict

No gap. Every FR-070 AC has a passing test that can fail. The production change adds no behavior that lacks an owning requirement.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
