---
id: SR-967175026302197768900474181941935084585
title: "Code review of IR WorkScope owner pins"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; Cargo.toml, crates/filament-contract-model/Cargo.toml, Cargo.lock; QSL-2158"
review_set: subset
---

## Summary

Ticket QSL-2158, PR1124. Independent native Codex review of frozen head c1fd822190d7d362b781814195657d2a99fc8299 against 96554ea506a19b16ff9863ad3d30066578bc609c. The three changed files substitute only the accepted exact and semantic-value revisions in production/dev declarations and matching lock sources.

## Verdict

**PASS** for the mechanical dependency diff; no findings. This does not waive the default-feature gate failure, authorize its repair, or qualify the complete FSL cumulative implementation. The lead retains the final current-main exact-head gate and architecture disposition of the feature configuration before merge.

## Scope

Code-review and its Rust configuration lane apply. Loaded reviewer, code-review, rust-review and rust-style, including API, testing and detailed review checks. Read AGENTS.md, CLAUDE.md, CONTRIBUTING.md, deny.toml, clippy.toml, rustfmt.toml, package lints and the existing workflow as context. No applicable AssuranceProfile declaration was found under spec/. No Rust source, test, spec, feature declaration, public API, wire format, workflow, lint or gate configuration is changed. Explicit user authorization for this owner pin chain and focused service checks supersedes generic broad gates and pin-policy restrictions for this task. Both workspace packages are included in supplied compile qualification; no root-only acceptance is claimed.

```yaml
{
  "clean": true,
  "scope": [
    {
      "id": "Cargo.toml::filament-exact",
      "path": "Cargo.toml",
      "role": "examined",
      "excerpt": "filament-exact = { version = \"0.3\", git = \"https://github.com/agent-ix/filament-exact\", rev = \"a58474cc105a86f3307a11def3fc001e2459679f\" }"
    },
    {
      "id": "crates/filament-contract-model/Cargo.toml::filament-exact",
      "path": "crates/filament-contract-model/Cargo.toml",
      "role": "examined",
      "excerpt": "filament-exact = { version = \"0.3\", git = \"https://github.com/agent-ix/filament-exact\", rev = \"a58474cc105a86f3307a11def3fc001e2459679f\" }"
    },
    {
      "id": "crates/filament-contract-model/Cargo.toml::filament-semantic-value",
      "path": "crates/filament-contract-model/Cargo.toml",
      "role": "examined",
      "excerpt": "filament-semantic-value = { version = \"0.1\", git = \"https://github.com/agent-ix/filament-semantic-value\", rev = \"31d807ccb8ffd5a88beb0b1a570b80acb099f5e9\" }"
    },
    {
      "id": "Cargo.lock::filament-exact",
      "path": "Cargo.lock",
      "role": "examined",
      "excerpt": "name = \"filament-exact\"\nversion = \"0.3.0\"\nsource = \"git+https://github.com/agent-ix/filament-exact?rev=a58474cc105a86f3307a11def3fc001e2459679f#a58474cc105a86f3307a11def3fc001e2459679f\""
    },
    {
      "id": "Cargo.lock::filament-semantic-value",
      "path": "Cargo.lock",
      "role": "examined",
      "excerpt": "name = \"filament-semantic-value\"\nversion = \"0.1.5\"\nsource = \"git+https://github.com/agent-ix/filament-semantic-value?rev=31d807ccb8ffd5a88beb0b1a570b80acb099f5e9#31d807ccb8ffd5a88beb0b1a570b80acb099f5e9\""
    }
  ],
  "findings": [],
  "bindings": []
}
```

## Dependency coherence

Independent owner PR API reads confirm exact #191 merged as a58474cc105a86f3307a11def3fc001e2459679f and semantic-value #122 merged as 31d807ccb8ffd5a88beb0b1a570b80acb099f5e9. Local accepted owner manifests show exact remains 0.3.0 and semantic-value remains 0.1.5; semantic-value selects that same exact owner. Exact's accepted change has no manifest/lock dependency delta. Semantic-value's accepted manifest and lock change only the exact revision. The owner graph remains acyclic; model has no new FSL, observation, protocol, TL or IR dependency.

A Python 3.13 TOML parse plus complete byte comparison verifies all three files are exactly their base bytes after only the two authorized revision substitutions. There is one lock package record per owner, unchanged dependency lists, versions, features, unrelated pins and all other file bytes. The reviewed head descends from the accepted feature-wave base. The existing standalone tests/conformance_fsl_ir1384 manifest and lock are unchanged and explicitly outside this workspace pin change; its external FSL lane is not claimed qualified. git diff --check passes. No copied implementation, compatibility shim, duplicate work ledger, vendoring or local patch is introduced.

## Validation evidence and existing default-feature failure

Lead-supplied actual Titan results at the frozen head: model library checked_package::v2::exact_projection::tests:: selected 7, passed 7, failed 0, ignored 0. Root integration exact_projection_limits:: with existing --features conformance-cli selected 7, passed 7, failed 0, ignored 0. The --locked --all-targets check naming filament-contract-model and filament-contract-ir with that feature exited 0, with zero runtime tests. These are supplied service executions, not reviewer-local runs.

The ORIGINAL default-feature root integration compile and all-target check both failed E0432 at tests/it/source_obligation_filament.rs:22:21, importing crate::support::corpus. Inspection of unchanged base source confirms tests/it/main.rs unconditionally declares source_obligation_filament while tests/it/support/mod.rs gates corpus on conformance-cli. This is preserved as an existing red, not reported green or repaired by this PR. The successful feature-enabled filtered command does not run the external conformance corpus. The lead has requested arch disposition of using that existing feature configuration for the final gate; this artifact does not grant that disposition.

No local Cargo, broad runtime suite, clippy, cargo-deny, Kani, measurements, external conformance or FSL consumer gate was run. The mandatory final rebased head check remains the lead's responsibility after artifact custody. Model and root focused passes do not prove complete WorkScope forwarding in FSL.

## Method applicability

Gap-analysis N/A: this pin-only diff adds no production implementation, criteria or tests. Spec-review and sub-analyses N/A: spec/ and plan/ are unchanged. No full matrix sweep or unrelated source audit was performed. QSL-2158 remains incomplete.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Tracker publication

Private QSL-2158 marked review comment and publication manifest are prepared locally. No Linear or ix-board reads/writes, labels, metadata, dedup or status queries were attempted under root's tracker-budget hold. Publication requires explicit root release; readback remains pending. No findings or excerpts were posted to GitHub.
