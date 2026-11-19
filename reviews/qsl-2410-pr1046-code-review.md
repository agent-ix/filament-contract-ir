---
id: SR-17940486709564847048068
title: "code-review and rust-review of QSL-2410 claim_names in the checked-package schema"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1046 at 11029d36 (base origin/main b13123f0); crates/filament-contract-model/src/checked_package/v2/{claim_names.rs,mod.rs,intake.rs,document_de.rs,refinement.rs,term_view.rs,typed_terms.rs}, crates/filament-contract-model/src/lib.rs, tests/conformance_fspec/main.rs, tests/it/support/{checked_package.rs,fspec_inputs.rs}, tests/it/checked_package_v2_whole_read_accounting.rs"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in, for filament-contract-ir PR #1046. Ticket: QSL-2410. Requirement source (read only): a fresh export of filament-specification origin/main f16e1701, FR-322 "Claim declaration names", "One claim namespace", FR-322-AC-200, AC-457, AC-458 and AC-459, and `proposals/checked-package-v2/schema.json` (`ClaimNameRow`, `RefinementBody`).

Gates rerun at 11029d36 in the review worktree, with CARGO_TARGET_DIR in that worktree, `--locked`, and serialized on `/tmp/cargo-slot.lock`:

- `cargo fmt --all -- --check`: clean.
- `cargo clippy --workspace --all-targets --features conformance-cli -- -D warnings` and `-p filament-contract-model`: both clean.
- `make spec`: exit 2, with the same 68 unbacked rows as clean origin/main. The only differences are line shifts; FR-371 is 6/6.
- Whole workspace `cargo test --workspace --all-targets --features conformance-cli --no-fail-fast -- --include-ignored`, with FILAMENT_SPECIFICATION_DIR set: 13 failures in `it`, the same names as clean origin/main.
- `cargo test --test conformance_fspec --no-fail-fast`: 459 passed, 3 failed (`operation_effects::tc_977_published_operation_effect_vectors` and two `temporal_aggregate` tests). The names and panic sites match clean main.
- `FSPEC_CONFORMANCE=1 tc_048_fspec_all_positive_packages_and_owner_identities`: passes.
- `monitor_plan_public` and `temporal_binding_public`: pass.

Hostile-input probes, written as a temporary test that was removed afterwards:

- `claim_names: []` refuses `malformed_wire` at `/claim_names`.
- `claim_names: null`, with and without claim-bearing nodes, refuses `malformed_wire` at `/claim_names`.
- A duplicate name refuses with exactly one locus.
- Row 0 dangling plus row 1 with a non-identifier name refuses `invalid_package` at `/claim_names/0/declaration`.

Overlap merges were real local merge commits in scratch worktrees, never pushed. Each was run with `cargo test --test it checked_package_v2` and FILAMENT_SPECIFICATION_DIR set.

- #1046 + #1040: failure set identical to #1040 alone (7 failures, all of #1040's own).
- #1046 + #1045: #1045 alone passes all 1178 tests. The merge fails 3 `whole_read_accounting` tests on a #1046-caused defect (FND-001).

## Examined scope

- claim_names.rs `validate` (examined): row visit order, the per-row check order 1-5 against FR-322 "Claim declaration names", refusal code/cause/pointer per rule, the coverage walk, work charges (1 per row, 1 per occurrence scanned in the join and in the walk), storage charged through `stage_storage::vec`. No wildcard arm over `CheckedNodeKind` in the claim-bearing filter beyond `Claim(_)`, which FR-322 intends (every non-temporal claim form). Indexing `kinds[position]`, `nodes[position]`, `terms[position]` uses positions from the graph index; no panic path from hostile input found. The ordering key reuses `GraphIndex::resolved_at` plus the derived `CheckedOccurrence` order (role enum order declaration, type, expression, anchor, claim, generated, then ordinal), matching FR-322's occurrence-key order and `temporal_clause_names`.
- claim_names.rs refinement row rule (examined): when the body decodes to `CheckedTerm::Refinement`, `name.as_deref() != Some(row.name)` refuses at `/claim_names/{i}/name`. That includes a body with no `name`, so the PR's own disclosure ("compares only when the body carries one") understates the check. An unnamed refinement body can never be admitted: with a row it refuses at `/name`, and without one it refuses as uncovered. See FND-006 for what FR-322-AC-200 still lacks.
- mod.rs envelope, `CheckedPackageSidecarsRef.claim_names`, `claim_names()` and `claim_name_for_path()` (examined). The lookup is whole-path only, which matches the FR-322 reader exposure; bare-name resolution is the consumer's job.
- intake.rs `BorrowedWire.claim_names`, `classify_wire_fault` and the lossless compare (examined); document_de.rs FORMS (examined).
- refinement.rs, term_view.rs and typed_terms.rs `name` (examined): optional member, accepted in both closed-member lists, and encoded before `abstract`.
- tests/conformance_fspec/main.rs retry shim (examined). It retries only when the refusal's first locus is `/claim_names` and the package has no member, then derives rows with `refresh_claim_names`. The derived names are synthetic (`ClaimN`/`ScopeN`, `RefinementN`), not source names. That is honest for FR-322 conformance, because FR-322 itself makes the member mandatory and only FSpec's fixtures lag. 8 of the 20 published fixture packages carry claim-bearing nodes and no member. See the gap-analysis SR for the missing owner.
- tests/it/support/fspec_inputs.rs `read_json` shim (examined); `read_json_wire` has none (FND-001).
- Consumers at origin/main (examined, read only):
  - filament-contract-codegen (contract-ir v0.1.4): `CheckedPackageSidecarsRef {..}` literals at src/oracle/equality/mod.rs:789 and :4026, src/oracle/function/owner_path.rs:282, and tests/it/admitted_composite_equality.rs:613.
  - filament-spec-language: the same literal at fsl-package/src/emit.rs:6073 and :7177, fsl-package/src/emit/monitor_plan.rs:1348, src/population_fixtures.rs:447, tests/it/emit_checked_v2.rs:449 and fsl-package/src/emit/tests/operation_effects.rs:657.
  - The FSL emitter writes no `claim_names`.
  - filament-driver builds no sidecar literal.
  - `CheckedTerm::Refinement` is matched only with `{ .. }` in codegen.

## Verdict

The rules of FR-322-AC-458 are implemented with the right code, cause and pointer, in the stated order. Every rule check is mutation-killed (see the gap-analysis SR). The PR is not mergeable as it stands:

- FND-001: the "13 failures identical to main" claim is true by name but hides a new #1046 defect. Main's own earlier panic in `whole_read_accounting` masks it, and it turns main red as soon as #1045 lands.
- FND-002: FR-322-AC-459, from the same FSpec section and merged before this PR's commit, is not implemented.

The release coordination in FND-005 must be recorded on the next contract-ir tag.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The test-side claim_names derivation covers `fspec_inputs::read_json` but not `read_json_wire`. The whole-read-accounting registry builds the FSpec model-root vectors on `read_json_wire(positive-all-families.json)`, so the published MR-00 base (admitted) is now refused `invalid_package`/`invalid-value` at `/claim_names`. On origin/main and at this head, the 3 `whole_read_accounting` tests already panic earlier, at tests/it/checked_package_v2_protocol_body.rs:6424 ("no entry found for key"), so the regression is invisible. In the real merge with #1045 that early panic is gone and all 3 tests fail on "FSpec/model-root/MR-00: fixture outcome differs from its required class: Refused(... /claim_names ...)". #1045 alone passes all 1178 checked_package_v2 tests. MR-00 is the only defect listed among 332 registered inputs. Owner: this PR (derive the rows on the MR wire after its patches, as `read_json` does) | tests/it/checked_package_v2_whole_read_accounting.rs:725-760; tests/it/support/fspec_inputs.rs:22-40 |
| FND-002 | medium | FR-322-AC-459 "One claim namespace" (FSpec #698, merged 00:44 on 2026-11-18, before this PR's 01:27 commit, in the same FR-322 section) is not implemented: `claim_names::validate` never consults `temporal_clause_names`. A `claim_names` row whose `name` equals a `temporal_clause_names` row's `name` is admitted, where FR-322 requires `ambiguous_declaration`/`ambiguous-name` at `/claim_names/{i}/name`, naming both rows, with one work visit per visited row | crates/filament-contract-model/src/checked_package/v2/claim_names.rs:113-253 |
| FND-003 | medium | The duplicate-name refusal names only the later row: one locus `/claim_names/{i}/qualified_name` plus the row's node, confirmed by probe. FR-322 rule 5 says it refuses "naming both rows". `names` keeps only the paths, so the earlier row index is lost. `Refusal::in_document` exists for a second position | crates/filament-contract-model/src/checked_package/v2/claim_names.rs:216-235 |
| FND-004 | low | The closed-schema checks (identifier `name`, empty or non-identifier `qualified_name`, `occurrence.role`) run inside the per-row loop, after earlier rows' semantic checks. A package with row 0 dangling and row 1 holding a non-identifier `name` refuses `invalid_package` at `/claim_names/0/declaration` (probe), where FSpec's closed schema (`ClaimNameRow`, `Identifier`) refuses `malformed_wire` at row 1 before the reader visits rows. This matches the existing `temporal_clause_names` precedent but departs from FR-322's schema-then-reader order | crates/filament-contract-model/src/checked_package/v2/claim_names.rs:126-143 |
| FND-005 | medium | Public API flag day, not recorded in FR-019 or the PR as a breaking release. `CheckedPackageSidecarsRef` (not `#[non_exhaustive]`) gained `claim_names` and `CheckedTerm::Refinement` gained `name`. At their next bump, every `CheckedPackageSidecarsRef {..}` literal in filament-contract-codegen (4 sites) and filament-spec-language (6 sites) fails to compile. Semantically, the FSL emitter writes no `claim_names`, so every FSL package with a state clause, refinement or claim is refused by this strict reader. Codegen's re-encode must pass `original.claim_names()`, or its re-read is refused. The change must ride the next contract-ir tag as a breaking bump, landing with the FSL writer (QSL-2402 scope) | crates/filament-contract-model/src/checked_package/v2/mod.rs:968-990; crates/filament-contract-model/src/checked_package/v2/typed_terms.rs:2244-2248 |
| FND-006 | medium | FR-322-AC-200 is only partly met. FSpec `RefinementBody` requires `name` ("a body without `name` fails both schemas"), but `CheckedTerm::Refinement.name` is `Option` and the body decoders accept a missing name. A nameless body therefore refuses late, at `/claim_names/{i}/name` or `/claim_names`, not as a body-schema refusal at the node body. Two refinement nodes with an equal `body.name` are not refused `ambiguous_declaration`/`ambiguous-name` at the later node's `body/name` (two different scopes give distinct `qualified_name`s, so rule 5 does not catch it). The admission half of FR-322-AC-458 (row name equals body name) holds. Missing piece, owned by contract-ir: make `name` a required `Box<str>`, add it to the required-member lists in refinement.rs `check_body_header`, term_view.rs `RefinementDocument` and `refinement_fault`, and add the package-wide duplicate refinement-name refusal at `/semantic_graph/nodes/{n}/body/name` | crates/filament-contract-model/src/checked_package/v2/typed_terms.rs:2244-2248; crates/filament-contract-model/src/checked_package/v2/refinement.rs:1055-1059,1248-1252; crates/filament-contract-model/src/checked_package/v2/term_view.rs:979,994 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | low | The conformance lane overrides one published vector: CN-S9-missing-declaration-region expects `/temporal_clause_names/0`, and the test holds it to `/temporal_clause_names/0/declaration_region` instead. The test says "reported to FSpec", but no ticket is recorded on QSL-2410 or in Linear. The override is right on the merits: FR-322 "Clause declaration regions" says "as `malformed_wire` at that member", and FR-322-AC-461, CL-S4 and CL-S5 agree. The ask for the FSpec owner was to change CN-S9 `expected.locus` in `proposals/checked-package-v2/temporal-clause-name-vectors.json` to `/temporal_clause_names/0/declaration_region`. The coordinator reports FSpec #707 made exactly that fix, so the override must be removed in the next head | tests/conformance_fspec/temporal_clock_bindings.rs:356-367 |

## Dispositions

Round 1, checked at 60c1db52 (mutants and probes) and 23b41755. claim_names.rs, declaration_region.rs, temporal_clause_names.rs and intake.rs are byte-identical between those two heads.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The model-root registry now derives the rows on the MR wire after its patches (`refresh_claim_names` when the member is absent). Merged with #1045 in scratch at 60c1db52 and run against FSpec f4583226, `whole_read_accounting` passes 10/10, where it failed 3/3 on MR-00 before. At 23b41755, origin/main 71914a07 (with #1040) no longer has the protocol_body.rs:6424 panic, so the registry runs to completion without #1045. #1045 itself now conflicts with origin/main (equality_projection.rs, checked_package_v2_model_fields.rs) and must rebase; that is #1045's work, not this PR's. |
| FND-002 | fixed | `one_namespace` runs after both tables pass. It refuses `ambiguous_declaration`/`ambiguous-name` at `/claim_names/{i}/name`, names `/temporal_clause_names/{j}/name` as the document position, and charges one visit per row. Mutants N01 (call removed), N02 (second position dropped) and N03 (work charge dropped) are killed by FR-371-AC-7 tests. |
| FND-003 | fixed | The duplicate-name refusal names the earlier row's `/claim_names/{j}/qualified_name` through `in_document`. Mutant N05 is killed by `a_repeated_qualified_name_is_ambiguous_at_the_later_row`. |
| FND-004 | fixed | The closed-schema checks run over every row before the visit loop. `a_malformed_later_row_wins_over_a_dangling_earlier_row` pins the order, and mutant N18 (visit loop reversed) is killed. |
| FND-005 | fixed | The new FR-371 "Compatibility" section and the FR-019 row record the breaking change: the `CheckedPackageSidecarsRef.claim_names` field, the required `CheckedTerm::Refinement.name`, and the required `declaration_region` on `TemporalClauseName` and `ClaimNameRow`. They state that it rides the next contract-ir tag as a breaking bump, list the 10 consumer sites, and assign the FSL writer duty to QSL-2402. |
| FND-006 | fixed | `CheckedTerm::Refinement.name` is a required `Box<str>`. A body without `name` refuses as a body-schema error (`invalid_semantic_graph` at the node body, this reader's code for closed-body violations). `validate_refinement_names` refuses the later of two equal names at `/semantic_graph/nodes/{n}/body/name` and names the earlier one. At 23b41755 a non-identifier name is refused on both decode paths. Mutants N12, N13 and N19 are killed. |
| FND-007 | fixed | FSpec settled the locus. #707 first moved CN-S9 to the member; #708 reverted that, and FR-322-AC-461, CL-S4, CL-S5 and TC-2015 now all refuse a missing `declaration_region` at the row, while CL-S6 (not a source region) stays at the member. At b4311609 the reader follows: intake.rs's missing-member re-location is gone and the CN-S9 override is removed, so temporal_clock_bindings.rs now replays every published locus unchanged. Against an FSpec 1ae83460 snapshot the conformance lane fails only tc_977, the 2 temporal_aggregate tests and probabilistic_claims::claim_row_coverage_and_exact_work_boundary, all of which also fail on clean main. |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-008 | low | The model-root registry still derives `claim_names` when a patched MR wire lacks the member, under a comment that says the published base "predates the `claim_names` member". At FSpec 1ae83460 (#709) `positive-all-families.json` carries its row and no MR vector removes the member. The branch is therefore dead and the comment is false, and if a future FSpec fixture dropped the rows the branch would hide that regression instead of failing. This also contradicts the round's "harness derives nothing" | tests/it/checked_package_v2_whole_read_accounting.rs:796-800 |
