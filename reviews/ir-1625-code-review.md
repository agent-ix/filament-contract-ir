---
id: SR-7191
title: "code-review and rust-review of the unused-frame row refusal and the dimension-equivalence cycle locus"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1191 at 0ec6686e (git diff origin/main...HEAD); crates/filament-contract-model/src/checked_package/v2/requirement_records.rs, crates/filament-contract-model/src/checked_package/v2/identity.rs, tests/conformance_fspec/requirement_records.rs, tests/it/checked_package_v2_dimension_equivalence.rs"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in, for filament-contract-ir PR #1191. Tickets: IR-1625 (unused path-condition frame refused at the table, not the row) and IR-1624 (dimension-equivalence cycle refused at the wrong equivalence node). The diff has two commits: the fix and a tag-only commit.

The spec was checked at FSpec a6968fb8 (the CI ref): FR-825 "Shared path-condition frames", "Emission" and "Reader", FR-825-AC-9 and AC-12, the published requirement-record vectors RRV-01 to RRV-06, FR-322-AC-268, and the local FR-063-AC-9.

No cargo command was run for this review. The coder's gate logs at head 0ec6686e show: the two named tests pass, the `conformance_fspec` requirement_records and dimension_equivalence modules pass (38 passed, 1 ignored), the `it` modules pass (305), the model lib filters pass (24), clippy and check are clean. The baseline logs show the named tests and the new tests failing on the unmodified source.

## Examined scope

- FSpec FR-825-AC-9 (examined): "a frame chain with the wrong parent, guard order or `holds`, a missing table, a terminal frame id outside the table, a record id whose chain differs from the owner-body-derived chain, or an unreferenced frame row refuses `invalid_package`/`invalid-value` at the corresponding parent, `/path_condition_frames`, record `body/members/5`, or table row".
- FSpec FR-825-AC-12 (examined): "The same package with an extra row for `p` with `holds` `false` refuses `invalid_package`/`invalid-value` at that row (used by no record's chain). ... Removing the `p` frame refuses with the existing parent refusal."
- FSpec FR-322-AC-268 (examined): "a cycle whose root-scale product is not one ... each refuse `invalid_semantic_graph` at the later equivalence node in node-id order".
- FR-063-AC-9 (examined): A–B `1/1`, B–C `1/1`, A–C `999/1` refuses at E-AC, the later equivalence node in node-id order.
- FSpec FR-825 Reader, frame checks (context_only): "a frame row not used by any record's referenced chain: `invalid_package`/`invalid-value` at `/path_condition_frames/{f}`", and "A non-empty frame table when all record chains are empty, or a missing table when any derived chain is non-empty, refuses `invalid_package`/`invalid-value` at `/path_condition_frames`."
- FSpec vector RRV-05 (context_only): record occurrence 0 cites frame 1; expected `body/members/5`.

### Unused-frame refusal (IR-1625)

- The original failing arrangement (an orphan row the walk does not reach, while other rows are cited) now refuses at the row: in `derive_package_path_conditions`, an out-of-order or unreached table computes the rows cited by any value-validity record's chain and refuses the first uncited row (requirement_records.rs:2239-2258). Correct per FR-825-AC-9 and AC-12.
- `mark_chain` stops at an already-marked frame. This is sound: a frame is marked only together with its whole ancestor chain. It terminates because `validate_frames` has already required every parent to be smaller than its row. The per-record chain walk was uncharged O(R × depth) on main; it is now O(F + R) in total.
- The no-records exit. A package with `path_condition_frames` and no `claim`/`value_validity` node was admitted on main; it is now refused at row 0. The refusal is what the spec requires: the member "SHALL be absent exactly when every `value-validity` record has an empty path condition", which holds trivially with no records. No conforming producer emits this package, because the producer "SHALL write `path_condition_frames` exactly when at least one value-validity record has a non-empty path condition". The row pointer agrees with the frame-check listing and with the final check at requirement_records.rs:1291, which already refused an operation-contract-only package with a table at row 0 on main. It does not agree with this PR's own derive branch; see FND-001.
- Open item 1 (the unused-row check runs after the record checks when the walk reaches every row). Not a defect of this PR. The published vector RRV-05 leaves frame 0 cited by no record and requires `body/members/5`, so the record rule must come before the unused-row check. The code follows the vector. FR-825's Reader text lists the unused-row check among the frame checks it runs "first", which contradicts RRV-05. That is an FSpec defect: the FSpec text should place the unused-row check after the record rules.
- Open item 2 (a row the walk does not reach but a record cites refuses the whole table). The table pointer is right for a wrong guard order, a wrong `holds` or a wrong guard occurrence: FR-825-AC-9 maps those to `/path_condition_frames` (for example, the test at tests/conformance_fspec/requirement_records.rs:2384). It is wrong for a wrong parent, which AC-9 maps to "the corresponding parent". This predates the PR; see FND-002.
- Open item 3 (two overlapping FR-825 sentences). When every record chain is empty, every row is uncited, so the row rule (`/path_condition_frames/{f}`) and the table sentence (`/path_condition_frames`) both apply. Under the Reader's listed order the row rule fires first, so the table half of that sentence can never fire. That is an FSpec defect, not a defect of this PR. Concrete input: this PR's no-records package (`reader_refuses_a_frame_table_in_a_package_without_records`), which the row rule refuses at `/path_condition_frames/0` and the table sentence at `/path_condition_frames`. The code applies the row rule there, which is consistent with the listed order.

### Dimension-equivalence cycle (IR-1624)

- Edges are joined in node-id order: `validate_nominal_nodes` pushes equivalences while iterating the id-keyed index (`index.values()`, identity.rs:1942 and 1992). The invariant is `r(first) = root_scale × r(second)` within a component. A same-component edge requires `first_ratio == second_ratio × ratio`. A merge rewrites the smaller component by `first_ratio / required`, or the larger by `required / first_ratio`. Both directions preserve the invariant and keep the kept component's named dimension at scale one.
- The first refused edge is the smallest k for which edges 0..=k are inconsistent. That edge is the latest edge, in node-id order, of an inconsistent cycle it closes. Among several inconsistent cycles it is the one completed first. This satisfies FR-322-AC-268 and FR-063-AC-9 for any graph shape. Union-find admits exactly the graphs the old walk admitted; only the refusal locus changes.
- Exactness: every potential comes from `ExactRatio::one()` or from `multiply`, which reduces by the gcd. Root scales are positive, so `==` on reduced pairs is exact rational equality. A parsed `root_scale` is not reduced, but it is only ever multiplied, never compared.
- Self-loops and parallel edges: a self-loop reaches the same-component branch and requires `ratio == 1`. A parallel edge is checked against the existing class. Both pairs are refused earlier anyway, by the equal-pair and repeated-pair checks.
- Resources: `next` and `size` are charged `meter.vec`s of the id count, which replace the 2E adjacency vector. Every multiply and divide charges the meter. Union by size rewrites each dimension O(log n) times, so the work is O(E + n log n) charged operations, against O(E) before. Potential magnitudes are still path products within one component, so integer sizes have the same bound. There is no recursion and no quadratic path.
- The component name now depends on merge sizes instead of being the first structural id. Only `normalized_dimension` consumes it, and it compares two maps built from the same potentials, so the left/right scale ratio does not depend on which dimension names the component. Nothing is emitted from it.
- Hand-derived test values were checked. 1/1000, 1/1000, 1 admits and 2 refuses at node 2. A = 2B, C = 3D, B = 5D gives A = 10/3 C: 10/3 admits and 11/3 refuses at node 3. B = 7C, B = 2D, A = 5B gives A = 35C, A = 10D and C = 2/7 D: 11 and 7/2 refuse at node 4, and this case covers the smaller-first-side branch. In the two-cycle case, edge 2 closes A=B, A=C, B=2C before edge 4 closes A=D, C=7D. All values are correct.

### Tags

- The `FSpec-FR-322-AC-268` `production_orphan_tag` citations at identity.rs:2744 (the doc comment) and identity.rs:2746 (the attribute) come from the mention scanner. It matches the bare id inside "FSpec FR-322-AC-268" and inside the prefixed attribute string, and labels the match an orphan because the repository has no local FR-322. The `implements` claim itself resolves and joins the four new tests. `equivalence_potentials` really decides AC-268's cycle-product clause. Not a mis-tag.
- The new cycle tests also carry FR-063-AC-9, whose AC fixes a single A–B–C input that the existing test at line 478 asserts. The new tests assert the same rule on other graphs. Acceptable.
- The FR-825 tag spelling split is recorded in SR-7192.

## Verdict

Approve after FND-001 and SR-7192 FND-001 are fixed in this PR. Both are low-severity, one-line changes. The union-find replacement is correct for any graph, with exact arithmetic, charged work and bounds that stay within the old walk's order up to a log factor. The row refusal is right in every arrangement that has at least one cited row, and in the new no-records case. FND-002 predates this PR and should be a separate ticket. Open item 4 (FR-063-AC-9 still PLANNED/UNRUN) is not a finding: every FR-063 AC (AC-1 to AC-9) and the form note at line 120 carry the same stale marker, and the AC-9 test already existed on main. Changing AC-9 alone would leave the table inconsistent, so it belongs in one FR-063 status refresh.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The derive branch refuses the whole table when value-validity records exist but none cites a row and the walk leaves a row unreached (`if used.contains(&true)`). For the same condition with no records, this PR refuses the first uncited row (row 0), and the final check does the same. The comment ("refused as a whole ... when no record cites any") and the PR text describe a rule that the new no-records test contradicts. Input: the `a_guard_read_behind_a_literal_advances_the_later_guard_occurrence` package with rows 1 and 2 naming occurrence 0, and both records' `path_condition` (body and identity path) emptied. Read from code, this gives `/path_condition_frames`, where the row rule gives `/path_condition_frames/0`. Fix: call `refuse_unused_frame(&used, meter)?` unconditionally, keep the table refusal only for a table whose rows are all used, update the comment and the PR text, and add this input as a test (trace FR-825-AC-9). | crates/filament-contract-model/src/checked_package/v2/requirement_records.rs:2239-2258 |
| FND-002 | medium | Predates this PR (out of scope). A row whose parent is wrong, so that the walk does not reach it although a record cites it, refuses the whole table. FR-825-AC-9 maps a wrong parent to "the corresponding parent", and AC-12 says removing the `p` frame "refuses with the existing parent refusal". Failing input: the `missing_parent` package in `nested_pruned_branch_keeps_parent_frame_and_refuses_when_parent_is_removed`, a single row `(parent null, q, holds true)` cited by the record. It returns `/path_condition_frames`; AC-9 and AC-12 require `/path_condition_frames/0/parent`. Changing that test's expected pointer gives the failing test for a follow-up ticket. | tests/conformance_fspec/requirement_records.rs:3033-3070; crates/filament-contract-model/src/checked_package/v2/requirement_records.rs:2239-2258 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, head 0fc0e0cc. In `derive_package_path_conditions` the `if used.contains(&true)` guard is gone, so `refuse_unused_frame(&used, meter)?` runs unconditionally, and the comment now reads "The table is refused as a whole only when every row is used." The new test `reader_refuses_the_first_row_when_no_record_cites_any_frame` (trace FR-825-AC-9) is the review's input and expects `/path_condition_frames/0`. It fails at 0ec6686e and passes at 0fc0e0cc. The PR text now says "When no record cites any row, row 0 is refused." |
| FND-002 | deferred | IR-1637, which carries the failing test (`nested_pruned_branch_keeps_parent_frame_and_refuses_when_parent_is_removed` expecting `/path_condition_frames/0/parent`). Too big for this PR, and the reason holds. Today a row missing from the table collapses the walk's chain to `PRUNED_FRAME`, so the reader cannot tell a wrong parent from a pruned branch. Giving the parent pointer needs each record's derived chain kept independently of the table, plus a classifier that keeps the table pointer for a swapped guard order (AC-9). That is a walk redesign, not a local edit. The defect predates #1191, and the package is refused either way; only the pointer differs. |
