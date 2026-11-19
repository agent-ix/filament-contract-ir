---
id: SR-17940486709564847048071
title: "code-review and rust-review of QSL-2347 source dispatch keys, operation bodies and identity-row operations"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1049 at 3d9417a7 (base origin/main 71914a07); crates/filament-contract-model/src/checked_package/v2/{dispatch_projection.rs,operation_bodies.rs,source_object.rs,mod.rs}, crates/filament-contract-model/src/lib.rs, tests/it/checked_package_v2_source_objects.rs"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in, for filament-contract-ir PR #1049. Ticket: QSL-2347. Requirement source, read only: filament-specification origin/main 118febea (FR-788, FR-789, FR-790, FR-322 "Source-declared object types", FR-322-AC-290/397) and FSL FR-569 "Source-object identity rows".

Gates, rerun in scratch worktrees CIR-1049-head (3d9417a7) and CIR-1049-main (71914a07), with a shared scratch target dir, `--locked`, and `cargo-slot`:

- `cargo fmt --all -- --check`: clean.
- `cargo clippy --workspace --all-targets --features conformance-cli -- -D warnings`: clean. The consumer lane `-p filament-contract-model` was not run; it was cancelled in the slot queue.
- `make spec`: 79 unbacked rows at both head and main. New lint at head: a `tag-on-non-binding-symbol` note for the `// FR-788:` comment (FND-005), and three EARS warnings on FR-373 (spec-review SR).
- Clean main 71914a07, the whole workspace `cargo test --workspace --all-targets --features conformance-cli --no-fail-fast -- --include-ignored` with FILAMENT_SPECIFICATION_DIR (FSpec 118febea) and the installed `fsl-string-edge`: 2188 passed, 12 failed. The failures were 6 `timed_model::tests` in the model lib, 3 `whole_read_accounting`, 2 `lock_coherence`, and `default_feature_consumer_borrows_admitted_clock_rows`.
- Head, `--test it` (the target this PR touches): 1494 passed, 10 failed. These were the same 5 `it` failures as main, plus 5 `checked_package_v2_probabilistic_delay` tests. The shared FSpec checkout moved to 1ae83460 (#709, fixtures now carry `claim_names`) between the main and head runs, and those fixtures need #1046. Rerunning those 5 on clean main with FSpec 1ae83460 fails all 5 the same way, so the cause is environmental and not this PR. Net: the failing set at head equals clean main's. The whole-workspace head run was cancelled in the slot queue. Outside `it`, the PR changes no code path.
- Mutation: `cargo mutants --in-diff` lists the 26 mutants the coder reports. These were not re-executed. Instead, 6 hand mutants were run on the new branches that cargo-mutants does not generate, each against `checked_package_v2_source_objects`:
  - M2, the FR-788 source branch reverted to `unsupported_construct`: killed.
  - M10, the source repeated-key refusal removed: killed.
  - M1, M3, M6 and M8 survived (FND-002, FND-003, FND-006).

## Examined scope

- dispatch_projection.rs, the source branch for a non-null `owner_effective_type` (examined). It matches FR-788 and keeps the model rows' `unsupported_construct`. M2 kills it.
- operation_bodies.rs `source_types`, `source_bodies` and `source_candidates` (examined). Every Vec is allocated through `stage_storage::vec` and each candidate is charged one unit of work. Lookups are binary searches. No index from hostile input reaches `nodes[...]` unchecked: positions come from `by_id`.
- operation_bodies.rs, the source-row branch in `validate` and `check_source_row` (examined). Document, body and node arms, carrier order and the repeated key.
  - The `owned` scan (`join.nodes.iter().any`) is O(nodes) and unmetered, but it runs once, on the refusal path.
  - Dropping the candidate-body agreement check (commit 70a393aa) is sound. A candidate joins by its own body's declaration, and declarations are unique per package.
- `uncovered_source` coverage (examined): FR-373 item 3 and AC-4/AC-5.
- source_object.rs `SelectedSourceObjectOperation` and `SelectedSourceObjectRow.operations` (examined). They have no validation, which matches FR-367-AC-5 and FSL FR-569. The doc comment's "checks the entry set and types" paraphrases FR-569's JSON-reader rule.
- Downstream constructors (examined, read only): filament-spec-language origin/main fsl-package/src/bare.rs:750 and fsl-package/src/emit.rs:7630 build `SelectedSourceObjectRow {..}` literals. FSL already carries the same `{identity, query}` facts beside the row as `SourceObjectOperationFacts` (FND-001).
- tests/it/checked_package_v2_source_objects.rs, new TC-480 tests and the AC-3 case (examined).

## Verdict

The delivered seams are correct against FR-788, FR-790 and FSL FR-569:

- seam 1, the selection key;
- seam 3, source `operation_bodies`;
- seam 4, the `operations` member.

The model paths are unchanged and fmt and clippy are clean. Nothing blocks the code. The findings are test-coverage holes on secondary refusal arms (4 surviving hand mutants) and the downstream break the PR body already discloses.

The failing set matches clean main. Mergeable once two things are done:

- FND-002 and FND-006 get tests. They are cheap, and they belong in this PR per the one-AC-one-PR rule.
- The `operations` / `clock_parameters` break is batched into one tagged release with the FSL pin bump.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Breaking public API. `SelectedSourceObjectRow` is not `#[non_exhaustive]` and gains a required `operations` field. filament-spec-language origin/main builds the row by struct literal at fsl-package/src/bare.rs:750 and fsl-package/src/emit.rs:7630, so both fail to compile at the next contract-ir pin bump. FSL must move its side-carried `SourceObjectOperationFacts {identity, query}` into the row. The coordinator reports W51 will add `clock_parameters` to `SelectedSourceObjectOperation`, a second break to the same type. Batch both into one breaking tag and one FSL pin bump; the FR-019 row lists the type but records no break | crates/filament-contract-model/src/checked_package/v2/source_object.rs:75-112 |
| FND-002 | medium | Unkilled: FR-373 item 1's "another owner's identity refuses `invalid_package`/`invalid-value` at `document`". Hand mutant M1 (`if false && owned`, so it always refuses `missing_declaration`) survives `checked_package_v2_source_objects`. No case sets `document` to the `identity` of a second `SourceOwner` in the package | crates/filament-contract-model/src/checked_package/v2/operation_bodies.rs:546-562 |
| FND-003 | low | Unkilled: the source-row carrier order check. Hand mutant M6 removes `previous >= key` in the source branch and survives. No case puts two source rows out of `(document, declaring, operation)` order, and `operation_body()` binary-searches on that order, so an unchecked misorder would make typed reads miss rows | crates/filament-contract-model/src/checked_package/v2/operation_bodies.rs:431-434 |
| FND-004 | low | FR-788-AC-2's "checked package's name" key (`package` the checked package's name with `node` `ix://<that name>/Order`) is not among the selection-key cases, though amended FR-366-AC-3 lists it. The nearest cases change `package` alone (`test/families`) or `node` alone (`ix://test/families/Account`) | tests/it/checked_package_v2_source_objects.rs:1626-1678 |
| FND-005 | low | The new `// FR-788: ...` comment in `validate_checked_term_data` is reported by `quire coverage` as a `tag-on-non-binding-symbol` trace id (CR-061), which is new versus main. The function already carries the same note for FR-366. Prefer `Implements: FSpec FR-788` or reword so the id is not parsed as a trace tag | crates/filament-contract-model/src/checked_package/v2/dispatch_projection.rs:340 |
| FND-006 | low | Unkilled secondary arms. M8 removes the `ill_typed`/`operator-ineligible` check for a non-`function` `body.node` on a source row and survives. M3 removes the `name != operation` filter on source candidates and survives: a candidate whose body is another operation's clause function then refuses at `/operation_bodies` instead of being left to FR-366 dispatch validation, as FR-373 says. The test labelled "another operation's body" actually uses `Sub.deposit`'s body (the same operation, another type), so another operation's and another owner's function are untested | crates/filament-contract-model/src/checked_package/v2/operation_bodies.rs:336-338,592-603; tests/it/checked_package_v2_source_objects.rs:2779-2786 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | high | Commit 095459018 adds the cargo-mutants output directory `mutants.out/`: 60 files and about 19k lines of diffs, logs, `outcomes.json`, `mutants.json` and `debug.log`. Its logs carry local absolute paths (`/home/peter/...`). This is a public repository, and `.gitignore` has no `mutants.out` entry. Remove the directory from the branch and add `mutants.out/` to `.gitignore` | mutants.out/ (60 files); .gitignore:1-18 |
| FND-008 | medium | The string-edge fix hides the decision instead of remedying it. At 09d2f8561, `fsl-string-edge` reports `operation_bodies.rs:217 in source_body_declaration (branch-gating)` for `clause.as_ref() == "body"`. Commit 095459018 rewrites the same closed-vocabulary branch as `BODY_CLAUSE.eq(clause.as_ref())`, which the syntactic scan does not see. FR-056 says an exposed decision "shall be classified and remedied ... No baseline allowance, broad marker ... may turn an unresolved violation into a clean report". The comparison still gates a branch on a post-intake string. The same pattern predates this PR in dispatch_projection.rs:123 (the literal is passed as a parameter). Either classify the clause segment once into a typed kind, or record the decision with the FR-064/FR-056 owner; do not rely on a detector blind spot | crates/filament-contract-model/src/checked_package/v2/operation_bodies.rs:215-226 |

## Dispositions

Round 1, checked at ab9dc00d (merge of origin/main a15bf349). FSpec was pinned at b7c3a4b1 in a private worktree.

Gates:

- fmt: clean.
- clippy, both the workspace lane with `--features conformance-cli -D warnings` and the `-p filament-contract-model` lane: clean.
- `make string-edge`: clean (but see FND-008).
- `make spec`: 70 unbacked rows at both head and main, FR-373 6/6, and no FR-373 EARS warnings or FR-788 tag note.
- Whole workspace `--include-ignored`, each tree with its own target dir: head 2237 passed, 20 failed; clean main a15bf349 2232 passed, 20 failed. The failing set is identical: 18 `monitor_plan::reached_model_read` lib tests and 2 `lock_coherence`.

Hand mutants against `checked_package_v2_source_objects`: M1, M2, M3, M6, M8, M10 and M11 (`BODY_CLAUSE.eq` forced true) are all killed.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | accepted-no-change | The break is real and intended. The PR body now records it under "Breaking API (one coordinated contract-ir tag, one FSL pin bump)", including `clock_parameters` and the FSL `bare.rs`/`emit.rs` literals, batched with #1050 |
| FND-002 | fixed | after: case "another owner's identity as the document" sets `document` to `src/other-unit` and expects `invalid_package`/`invalid-value` at `/operation_bodies/0/document` (FR-373-AC-6). M1 is now killed |
| FND-003 | fixed | after: case "rows out of key order" swaps rows 0 and 1 and expects `invalid_package`/`invalid-value` at `/operation_bodies/1`. M6 is now killed |
| FND-004 | fixed | after: case "the checked package's name as package and node prefix" sets `package` to `FIXTURE_PACKAGE_NAME` and `node` to `ix://{FIXTURE_PACKAGE_NAME}/Order` |
| FND-005 | fixed | after: `// A source declaration key carries no owner effective type.` quire no longer reports the tag |
| FND-006 | fixed | after: cases "another operation's body function", "another owner's function" and "a node that is no function" (`ill_typed`/`operator-ineligible`), plus `a_candidate_body_of_another_operation_is_refused_by_dispatch_validation`. M3 and M8 are now killed |
| FND-007 | still-open | New in this round: `mutants.out/` is committed on the public branch |
| FND-008 | still-open | New in this round: the string-edge decision is hidden behind `.eq`, not remedied |

Round 2, checked at 51edebff: fix commit bdd595bb plus a clean merge of origin/main 6e3c0360. Against main, the PR diff is the 14 expected files. `git ls-files` lists no `mutants.out` and no log or outcome artefacts, and `.gitignore` has `mutants.out/`. fmt is clean and `make string-edge` is clean. The clippy and `checked_package_v2_source_objects` results are recorded in the Linear round-2 dispositions comment.

| FND | outcome | reason |
| --- | --- | --- |
| FND-007 | fixed | after: `mutants.out/` is untracked (`git ls-files` has no entry) and `.gitignore` gains `mutants.out/` |
| FND-008 | fixed | after: `#[ix_trace_rs::string_edge] fn source_body_declaration(...)` with plain `[type_name, operation, clause] if clause.as_ref() == "body"`. This is a narrow classifier of the declaration key's wire label into a typed `Option<(type, operation)>`, marked as `source_type_identity` and `bound_type` already are. That fits FR-056's "reviewed intake conversion ... or its typed refusal". It is not a broad validator exemption, so no typed kind is required |
