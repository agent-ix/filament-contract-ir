---
id: SR-612135
title: "Code review — align contract IR clock-reference pins"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; Cargo.toml, crates/filament-contract-model/Cargo.toml, tests/conformance_fsl_ir1384/Cargo.toml, Cargo.lock at af2852b4daad83638a6d1972b3730159f994dcbb and 636c2f9381ef69b7c8ffb783692dcedfe0de2f26"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-contract-ir/AD-007
    type: references
---

# SR-612135: Code review — align contract IR clock-reference pins

## Summary

Ticket: QSL-3312. Static independent review of PR #1184 at head `af2852b4daad83638a6d1972b3730159f994dcbb`. The PR changes only the root development dependency, the contract-model dependency declarations, and the lockfile.

## Verdict

**CLEAN** — both manifest dependency sites select `filament-exact` revision `743c42e4e458782bf7a8befc3b76cc8a190ed727`; the model selects `filament-semantic-value` revision `95a9034d5ac8abe856f1d67428dd60c9bbc85fc5`; and the root workspace lockfile contains one `filament-exact` package source at the requested exact revision. The pin-only change does not conflict with the repository's accepted dependency and shared-concept ownership architecture (AD-007).

The standalone `tests/conformance_fsl_ir1384` package was inspected as a separate Cargo workspace. It retains its own direct `filament-exact` pin and is excluded from the root workspace; its standalone resolution is not established by this root-workspace review. No Cargo command was run per dispatch instructions. The PR reports successful locked workspace checks and the filtered root test; those results were not independently rerun here.

## Scope examined

| Unit | Result |
| --- | --- |
| Root `Cargo.toml` and `crates/filament-contract-model/Cargo.toml` dependency revisions | Examined; both exact dependency sites match the intended revisions. |
| `Cargo.lock` package sources | Examined; one locked `filament-exact` source is present at 743c42e4 and semantic-value is at 95a9034d. |
| AD-007 cross-repo dependency graph and shared concept owners | Context only; the accepted architecture assigns shared exact and semantic-value concepts to their standalone crates and describes repository-local lockfile ownership. No contradiction found. |
| Separate `tests/conformance_fsl_ir1384` manifest | Examined as a separate workspace; it has an independent direct pin and is outside the root workspace lock identity assertion. |

## Review gates

No Cargo commands were run, per the dispatch. Static review covered the complete PR diff, the root lockfile package sources, repository dependency architecture, and the separate conformance manifest. Final controls remain with the code lead.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Re-review at head 636c2f9381ef69b7c8ffb783692dcedfe0de2f26

**Disposition: CLEAN for this upstream pin step; the standalone conformance lane failure is an expected dependency-chain interval, not a blocking finding for this PR.** The additional manifest changes the nested package's direct `filament-exact` pin to 743c42e4 and its direct observation pin to a571be6a. It does not commit the ignored nested lockfile.

The nested filtered command is a real failure: it exits 101 during compilation before tests run. I independently confirmed the FSL `main` dependency declarations at 9d8cc3c8 still select `filament-exact` 08a5a176, `filament-semantic-value` 689f259e, `filament-checked` / `filament-eval` 7fc24dc, and observation a36e5ed2. The nested package now consumes the deliberately newer IR-side identities, so that separate integration build sees the stale FSL graph and typed-identity mismatches. This is the downstream consumer pin that follows the upstream pin sequence; its current failure is disclosed in the PR body, is outside the root workspace CI, and must be rerun after the FSL consumer pin is merged. It is not represented as a passing test. No Cargo commands were run by this reviewer.

| Incremental unit | Result |
| --- | --- |
| `tests/conformance_fsl_ir1384/Cargo.toml` direct exact and observation revisions | Examined; pins match 743c42e4 and a571be6a. |
| Current FSL `main` shared dependency graph at 9d8cc3c8 | Context only; it retains old exact, semantic-value, eval/checked, and observation identities that explain the nested compile failure. |
| PR body and nested test result | Examined; failure and affected old transitive pins are stated accurately; command exited before tests. |

No new defect was found in the upstream pin changes. The aggregate sequence still requires the FSL consumer pin and a successful rerun of this nested lane afterward.
