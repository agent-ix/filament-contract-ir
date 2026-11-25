---
id: SR-967175026302197768900474181941935084779
title: "Code review — QSL-3389 location scan"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; crates/filament-contract-model/src/checked_package/v2/{protocol_locations.rs,body_locations.rs}"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-038", type: references }
---

## Summary

Reviewed PR #1186 at `5128e8c1dcac06ead600dd063b7ce93ab32d4f99`, including the shared row validator, its body-location caller, the duplicate-key helper, and the FR-038-AC-597 acceptance text. The change replaces the earlier-row scan with the existing `stage_storage::first_repeat` helper; duplicate positions remain selected in ascending row order and refusals remain emitted during the ordered row-validation pass.

## Verdict

**PASS** — no correctness, Rust-style, or code-to-criterion defect found in the changed paths. The author reports focused 1k body and 100k protocol scaling tests and crate Clippy/format checks passed. The one BL-D2 run at this head was OOM-killed at 12 GiB without a test verdict; that result is retained in the PR body and is outside this review's code-correctness verdict.

## Findings

| ID      | Severity | Summary                  | Refs |
| ------- | -------- | ------------------------ | ---- |
| FND-001 | low      | No findings (placeholder) | - |

## Coverage

- Scope examined: FR-038-AC-597, which requires the body-location paths to rebuild with linear work; shared `validate_rows` and `body_locations::validate`; existing duplicate-row refusal assertion; scaling-test call sites; `stage_storage::first_repeat` implementation.
- The tuple key preserves the previous key fields: optional function, application, occurrence, and term ordinal. The helper returns the least repeated row index; the main loop still performs row intake and reports the same `InvalidPackage` / `ConflictingDefinition` refusal at that row.
- The changed production functions now carry the bare `#[implements("FR-038-AC-597")]` marker. Quire coverage reports that criterion as tagged, with existing test binders.
- `git diff --check` passed. No dependency or CI workflow files changed. Cargo checks were not rerun by this reviewer; the PR body records the author's focused tests, fmt, and Clippy results.
- Repository-wide reverse-gap inventory was not part of this code-review pass; see the separate gap-analysis status reported to the lead.
