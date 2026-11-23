---
id: SR-967175026302197768900474181941935084592
title: "code-review of QSL-2776 PR #1150 package-wide requirement occurrences"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR #1150 at 3ed709288ef742ed267c8727f205266f985f2341; crates/filament-contract-model/src/checked_package/v2/refinement.rs, crates/filament-contract-model/src/checked_package/v2/requirement_records.rs, tests/conformance_fspec/requirement_records.rs, tests/it/checked_package_v2_refinement.rs"
review_set: subset
---

## Summary

Reviewed the exact PR head against its changed behavior, local FR-376 acceptance criteria, FR-038 reader accounting, and the Rust conventions. The global occurrence numbering and new cases are coherent, but the new per-occurrence path maps are allocated without read-storage charges, so the reader's bounded-allocation contract is not upheld.

Ticket: QSL-2776. Frozen review worktree: `/home/peter/dev/worktrees/review-qsl2776` at `3ed709288ef742ed267c8727f205266f985f2341`. Cargo checks were not run by this reviewer; the coder is running focused checks on Titan.

## Verdict

FAIL: one high-severity resource-accounting defect remains.

## Examined scope

- `crates/filament-contract-model/src/checked_package/v2/requirement_records.rs` (examined): package-wide occurrence numbering and path derivation across owners, body-before-measure traversal, absent-frame sentinel handling, charging of owner-body work, and storage reservations.
- `crates/filament-contract-model/src/checked_package/v2/refinement.rs` (examined): history initial target validation now admits literal and option-value forms and rejects the unsupported record-value form.
- `tests/conformance_fspec/requirement_records.rs` (examined): new positive and refusal cases for empty branches, absent frames, shared nodes, and cross-owner occurrence numbering. Quire identifies their cited FSpec IDs as untracked by this repository.
- `tests/it/checked_package_v2_refinement.rs` (examined): public-reader coverage for the changed history target forms.
- FR-376-AC-1 (examined): positive requirement-record rows and frame views.
- FR-376-AC-2 (examined): exact refusal behavior for invalid records.
- FR-376-AC-3 (examined): validation order, one owner-body walk, and budget charging.
- FR-376-AC-4 (examined): 1,000 records and shared frames must retain linear storage and have linear measured reader-storage charge.
- FR-038-AC-511 (context_only): whole-read allocation requests may exceed the read-storage delta by at most a fixed 4,096 bytes, independent of input size.
- FR-061-AC-5 (examined): history initial reference targets accept a local literal and reject an unsupported target form.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Package-wide path maps allocate per-owner and per-occurrence entries without read-storage reservations, allowing unmetered allocation to grow with package size and violating the strict reader accounting bound. | crates/filament-contract-model/src/checked_package/v2/requirement_records.rs:2101-2143 |

The new traversal creates `visit_counts`, `paths`, `frame_orders`, a per-owner `OwnerPaths`, and per-owner `OwnerFrames` collections. It reserves storage for the `declared` and `owners` vectors, and charges traversal work, but does not reserve read storage for these maps/sets/vectors. Each scalar occurrence inserted into `derived` and each owner/frame entry adds allocation proportional to input size. FR-038-AC-511 requires allocation overhead to remain within a fixed 4,096-byte margin over the budget's read-storage delta; FR-376-AC-4 separately requires measured storage charge to scale with records and unique frames. Reserve the dynamically retained/transient map storage before growing it, or use a bounded representation whose allocation is already charged.



## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Disposition pass 1 at fe10f593 replaces the uncharged per-owner maps with input-sized vectors reserved through `stage_storage::vec`; the reusable walk stack charges each capacity before `try_reserve_exact`. Static inspection confirms all newly introduced traversal collections are reserved or bounded by the admitted frame/node/occurrence counts. Titan tests and workspace checks were reported by the lead/coder; this reviewer did not run Cargo. |

### FND-001 after_excerpt (round 1)

```rust
let mut paths =
    stage_storage::vec::<DerivedPath>(meter, &numbering_site, scalar_occurrence_count)?;
let mut visit_counts = stage_storage::vec::<usize>(meter, &numbering_site, nodes.len())?;
visit_counts.resize(nodes.len(), 0);
let mut charged_owner = stage_storage::vec::<usize>(meter, &numbering_site, nodes.len())?;
```

## Disposition verdict

PASS for the code-review finding at `fe10f593bcec4b4fdb46431aea0a3898fbf7b88c`. The final one-line refusal-locus correction preserves the original missing-path-condition locus. No new finding was identified in the fix delta.
