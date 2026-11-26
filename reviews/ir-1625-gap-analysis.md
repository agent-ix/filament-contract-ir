---
id: SR-7192
title: "gap-analysis of the unused-frame row refusal and the dimension-equivalence cycle locus"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1191 at 0ec6686e (git diff origin/main...HEAD); FR-825-AC-9, FR-825-AC-12, FR-322-AC-268 (FSpec), FR-063-AC-9 (local)"
review_set: subset
---

## Summary

A gap analysis of filament-contract-ir PR #1191, limited to the criteria the diff claims. Tickets: IR-1625 and IR-1624. The run was planless. The matrix comes from `quire matrix --scope . --format json`, and each criterion was also traced with `quire trace --id`.

FSpec criteria (FR-825, FR-322) have no row in the local matrix, because they are external. Their binding was checked with `quire trace` under both spellings, the bare id and the `FSpec-` prefixed id. The semantic review (intent, test and code) was run inline for these four criteria, as the review brief asked. Its rulings are in SR-7191.

## Coverage

- FR-063-AC-9 (local, examined): status `tagged`, five binders. One is the existing test at tests/it/checked_package_v2_dimension_equivalence.rs:478; four are new tests. The new `implements` on `equivalence_potentials` joins them. The AC text still opens with "PLANNED/UNRUN", the same as AC-1 to AC-8 of FR-063. This marker predates the PR and is not a finding here (SR-7191 Verdict).
- FSpec-FR-322-AC-268 (examined): five claims (four new tests and one `implements`). The `production_orphan_tag` citations on bare `FR-322-AC-268` are scanner mentions of the prefixed id (identity.rs:2744, 2746). They are not a mis-tag.
- FR-825-AC-9 (examined): the bare id has six test claims, including the two new tests. `FSpec-FR-825-AC-9` has only the new `implements`, with no test. See FND-001.
- FR-825-AC-12 (examined): the bare id has eight test claims. `FSpec-FR-825-AC-12` has only the `implements`. See FND-001.
- Reverse gap: the new helpers `mark_chain`, `path_condition_members` and `record_terminal_frame` serve the tagged `refuse_unused_frame` path and the record reader. No unowned behaviour was added.
- Stubs and inflation: none. Each new test asserts an exact code, cause and pointer, or an admission, on an input built to separate the old behaviour from the new. The baseline logs show the new tests failing on the unmodified source, except the two-joined-classes cycle test, which the PR declares passes there by walk order.
- Semantic review: run inline; see SR-7191.
- Plan completion: not assessed

## Verdict

CONDITIONAL. Every claimed criterion has tagged tests. The one gap is the FR-825 id spelling split, which leaves the new `implements` claims unjoined to the tests that verify the same criteria.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | `refuse_unused_frame` carries `#[implements("FSpec-FR-825-AC-9", "FSpec-FR-825-AC-12")]`, but the PR's own new tests, like the 28 existing `conformance_fspec` FR-825 tags, use the bare `FR-825-AC-9` and `FR-825-AC-12`. As a result `quire trace --id FSpec-FR-825-AC-9` shows an `implements` with no verifying test, and the bare id shows only a mention of the code. Fix: `#[implements("FR-825-AC-9", "FR-825-AC-12")]`, matching the tests. | crates/filament-contract-model/src/checked_package/v2/requirement_records.rs:1317 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, head 0fc0e0cc. `refuse_unused_frame` now carries `#[implements("FR-825-AC-9", "FR-825-AC-12")]`. `quire trace --id FR-825-AC-9` lists it as an `implements` claim beside the tests that verify the same id. |
