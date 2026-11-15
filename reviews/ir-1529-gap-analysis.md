---
id: SR-20261115010005
title: "gap-analysis of IR-1529 source-declared object reader (PR #1025)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1025 at 3615bd2f (base origin/main ce1970fb); FR-362..FR-368, TC-475; crates/filament-contract-model/src/checked_package/v2/*, tests/it/checked_package_v2_source_objects.rs"
review_set: subset
---

## Summary

Planless gap analysis of filament-contract-ir PR #1025. Ticket: IR-1529. Plan completion was not assessed. Each acceptance criterion of FR-362 through FR-368 was checked against the tagged test in `tests/it/checked_package_v2_source_objects.rs` (31 tests, all `#[trace("TC-475", ...)]`, 31/31 passing) and against the code in the diff. The FR-366 candidate join was read against FSpec FR-322 "Candidate join" and the `EffectiveId` selection key, FSL FR-569 "Dispatch tables", and FSL ADR-013 QC-30 items 2d and 2f, all at their `origin/main`.

The coder's documented assumptions were checked against the spec:

- No work charge for the FR-367 join, and the reason order foreign, unknown, conflicting, missing: both match FR-367 Behavior word for word ("at the `Package` locus with no partial view and no meter charge"). They are not gaps.
- The anchor body function located as `[_, op, "body"]` under the same owner: this does not match the spec. FR-366 and QC-30 (2d) give the form `[<type>, <operation>, "body"]`, so the code should be fixed, not the spec (see code-review FND-001).
- The FR-366 candidate `selection_key.original` check and AC-6: a genuine spec gap (FND-001 below).

FR-362-AC-1 depends on the existing suites (TC-048, TC-228, TC-229, TC-463, TC-472, the FR-064 tests and the FSpec published fixtures) passing with no edited expectation. The diff edits no existing test file, and the `it` run over them is recorded in Verdict.

## Examined scope

- FR-362-AC-1 (examined): "The existing suites of TC-048, TC-228, TC-229, TC-463 and TC-472, FR-064's protocol-owner tests and the published FSpec checked-package fixtures pass with no edit to any expectation, including every exact work-charge assertion, and none of those packages declares a source object type."
- FR-362-AC-2, FR-362-AC-3 (examined): clean.
- FR-363-AC-1, FR-363-AC-2, FR-363-AC-3 (examined): clean.
- FR-363-AC-4 (examined): "An `owner_model` and a role `model` naming a source-local node that is not an object type (a source record, a population reference) refuse `invalid_model_binding`/`malformed-declaration` at the edge."
- FR-363-AC-5 (examined): "A model-owned clause over a selected model reads unchanged, and the FR-064 refusals for an unselected target, a missing node, a non-model node and the `model_root` keep their codes and loci in a package that also holds a source type."
- FR-364-AC-1, FR-364-AC-2, FR-364-AC-4, FR-364-AC-5 (examined): clean.
- FR-364-AC-3 (examined): "The forbidden positions of TC-229 (collection operand, reaches link, forbidden declaring member positions) refuse for a source population with the code, cause and locus they give for a model population."
- FR-365-AC-1 through FR-365-AC-6 (examined): clean.
- FR-366 (examined): "It SHALL require ... every candidate's `selection_key.original` to name a source-declared type node, each `operation_body` to name a clause function node with `declaration` `[<type>, <operation>, \"body\"]` ..."
- FR-366-AC-1, FR-366-AC-2, FR-366-AC-4, FR-366-AC-5 (examined): clean, apart from the AC-1 fixture point in FND-002.
- FR-366-AC-3 (examined): "A candidate `selection_key` naming a non-source node, an `operation_body` or `effective_precondition` naming a node of another declaration form or another operation, and a clause node with a different `SourceOwner` each refuse with the matching model-row check's code, cause and locus."
- FR-366-AC-6 (examined): "In the FR-367 read with its supertype rows, a candidate that is neither the context nor a transitive subtype of it refuses, and `Sub` for context `Order` admits."
- FR-367-AC-1 through FR-367-AC-6 (examined): clean.
- FR-368-AC-1 through FR-368-AC-6 (examined): clean as tagged. The AC-1 package holds only one body function, so the test cannot catch code-review FND-001.
- FSpec FR-322 "Candidate join" and the `EffectiveId` paragraph (context_only): "`EffectiveId.original` remains the original `{package, node}` declaration key ... the reader compares the captured identity componentwise with the candidate's admitted effective declaration identity. It does not derive the key from a name, graph node id or reverse lookup."
- FSL FR-569 "Dispatch tables" (context_only): "each candidate's `EffectiveId` the source declaration key the check selected it by".

## Verdict

Not complete as claimed. FR-362, FR-363, FR-365, FR-367 and FR-368 are implemented, and their tagged tests can fail. FR-363-AC-4, FR-363-AC-5 and FR-364-AC-3 each leave one or more named cases untested (FND-004 to FND-006). FR-366 is partial. The `selection_key.original` clause of FR-366 Behavior and AC-3 is neither checked nor tested, and AC-6 is not implemented.

This is a genuine spec ambiguity, not an implementer choice (FND-001). The spec never defines what `{package, node}` strings a source candidate carries, so "name a source-declared type node" has no decidable meaning. FR-322 forbids deriving the key from a name or node id. FR-569 says only "the source declaration key the check selected it by". The PR must not claim FR-366 complete. Its body says "FR-366: ... (AC-1..5; AC-6 not implemented)", but AC-3 is also only partly implemented.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-366 requires "every candidate's `selection_key.original` to name a source-declared type node" without defining the source declaration key. `CheckedOriginalDeclarationId` is two strings `{package, node}`, and FR-322 forbids deriving it from a name or node id. FR-366 also leaves out the AC-6 family check's input, outcome code and position in the reason order. Proposed amendment: (1) for a source context, every candidate's `selection_key` has `owner_effective_type` null, `original.package` equal to the `identity` of the context's `SourceOwner` (as FR-365 item 1 does for `document`), and `original.node` equal to the final `declaration` segment of a source-declared object type node under that same `SourceOwner`, its `source_identity`; any other key refuses `invalid_package`/`invalid-value` at `selection_key`, as the model check does. (2) `operation_body`/`effective_precondition` `<type>` is the declaring type of the candidate's effective operation, which the strict reader cannot know without supertype rows, so the strict reader checks only `<operation>` and the clause kind, and the FR-367 read checks `<type>`. (3) AC-6: the FR-367 read also takes the package's dispatch tables; it maps each candidate key to the row whose (`source_authority` of that owner, `source_identity` = `original.node`) matches; it requires that row to be the context's row or to reach it through `supertypes`; it refuses otherwise with a named `SelectedSourceObjectErrorCode` (`UnknownIdentityWitness`, or a new mismatched code); and it ranks that check in the reason order (after missing, since it needs the complete row set). | FR-366 |
| FND-002 | medium | The candidate `selection_key` clause of FR-366-AC-3 ("a candidate `selection_key` naming a non-source node ... refuse") has no check and no test. The `Route::Source` path skips every `selection_key.original` check, and the AC-1 fixture's source candidates carry `original.package` `"test/families"`, the selected model document's identity. Any reading of the amendment in FND-001 refuses that key, so the fixture would need to change. | FR-366-AC-3; crates/filament-contract-model/src/checked_package/v2/dispatch_projection.rs:317-334; tests/it/checked_package_v2_source_objects.rs:1331-1339 |
| FND-003 | medium | FR-366-AC-6 is not implemented and has no test: `SelectedSourceObjects::from_stage_rows` takes no dispatch table and runs no family check. It is blocked on the spec amendment in FND-001. | FR-366-AC-6; crates/filament-contract-model/src/checked_package/v2/source_object.rs:134 |
| FND-004 | medium | FR-364-AC-3 names three forbidden positions: collection operand, reaches link, and declaring member. The test covers the collection operand and the relationship, field and operation declaring members, but not the TC-229 reaches population link (the `population` link of a reaches application) for a source population. | FR-364-AC-3; tests/it/checked_package_v2_source_objects.rs:1057-1151 |
| FND-005 | medium | FR-363-AC-4 has only the clause `owner_model` → source record case. The role `model` cases and the population-reference target are untested. By reading, `check_target` refuses them with the right code, but nothing pins that. | FR-363-AC-4; tests/it/checked_package_v2_source_objects.rs:461-489 |
| FND-006 | low | The FR-363-AC-5 test compares three of the four named FR-064 refusals (missing node, non-model node, `model_root`); the "unselected target" case is missing. | FR-363-AC-5; tests/it/checked_package_v2_source_objects.rs:492-520 |
| FND-007 | low | The test tagged FR-362-AC-1 re-encodes one package and compares it with its own re-encode, so it cannot detect a change from base. The AC's real evidence is the unedited existing suites, and the tag claims more than the test shows. | FR-362-AC-1; tests/it/checked_package_v2_source_objects.rs:301-340 |
| FND-008 | low | FR-362 promises to "read ... and refuse a package that declares no source object type exactly as it did before". FR-365 and FR-364 change two such packages: a source-origin effect row was a schema-level refusal before and is now `invalid_package` at `provenance/origin`, and a `SourceOwner` population with a declaration was refused before and now admits. The spec should scope FR-362's promise to packages with no `SourceOwner`-owned population and no source-origin row. | FR-362 |
| FND-009 | low | FR-367 keys conflicting rows by (`source_authority`, `source_identity`). Two `SourceOwner`s of one authority (FR-322-AC-53 uses `agent-ix`/`example-a` and `agent-ix`/`example-b`) that each declare a same-named type would collide, and the row carries no owner `identity` to tell them apart. This applies only if one package can hold source types of two owners. | FR-367 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 1c71d496 amends FR-366 and FR-367 as proposed. The key is `{package: <owner identity>, node: <source_identity>}` with a null `owner_effective_type`. The strict reader checks the operation, the clause kind and the owner. The FR-367 read takes the dispatch tables and refuses `MismatchedIdentityWitness`, ranked after missing. The `<type>` choice is sound as a necessary condition: an effective operation's clause is declared by the candidate or by an ancestor it reaches, so the rule never refuses a valid emission. It cannot tell a redefining candidate from one that inherits, because the package has no operation list. FR-366 says so. |
| FND-002 | fixed | `Route::Source` now requires `original.package` to equal the identity of the context's `SourceOwner` and `original.node` to name a source-declared type under that owner. The test `source_candidate_selection_keys_refuse_unless_they_name_a_source_type_of_the_owner` covers four cases at `selection_key`. The AC-1 fixture now uses the source identity. |
| FND-003 | fixed | `dispatch_families_hold` and the `MismatchedIdentityWitness` code are implemented. The test `the_source_object_read_refuses_a_candidate_outside_the_context_family` covers: a type outside the family, a subtype row that does not reach the context, a transitive admit, an inherited body admit, a clause under a subtype refused, and missing ranked first. |
| FND-004 | fixed | `tc_475_a_source_population_as_a_reaches_link_refuses_at_the_population_member` refuses `ill_typed` at the member's `population`, the oracle that the model-population cases in the same module use. |
| FND-005 | fixed | `a_clause_or_role_model_naming_a_source_node_that_is_no_object_type_refuses` covers a source record and a population reference, as a role `model` and as an `owner_model`. |
| FND-006 | fixed | The FR-363-AC-5 test adds the unselected-target case and compares code, cause, loci and locus against the package without a source type. |
| FND-007 | fixed | The FR-362-AC-1 text now separates the two halves. The test is renamed to "re-encodes stably", and TC-475 step 4 names the unedited existing suites as the evidence for unchanged behaviour. |
| FND-008 | fixed | FR-362 is scoped to packages with none of the source constructs, and it states how the two edge packages now behave. |
| FND-009 | fixed | FR-367 now states that a type's structural identity follows its declaration whatever its owner, so a second same-named type under another `SourceOwner` refuses at admission. The test `a_same_named_type_under_a_second_owner_of_one_authority_refuses` covers it (`invalid_semantic_graph` at the node's `node_id`). |
