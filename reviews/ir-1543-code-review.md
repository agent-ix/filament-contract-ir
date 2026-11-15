---
id: SR-2026111422264100000004
title: IR-1543 FSL rename code-review
type: SpecReview
analysis: code-review
scope: agent-ix/filament-contract-ir; IR-1543; Cargo.toml, Cargo.lock, Makefile, README.md,
  crates/filament-contract-model, tests/, tools/read_path_proof/
review_set: subset
---

## Summary

Ticket: IR-1543. Reviewed all 90 changed paths against origin/main using the Python code-review lane and the Rust review lane. No introduced defect was found. The complete diff is a consistent spelling substitution except for the root serde-family lock resolution required by filament-observation. No applicable AssuranceProfile exists under spec/.

## Verdict

PASS for the scoped rename and dependency repair, subject to the explicit gate evidence below. This is not a full repository assurance claim.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed

Reconciliation: quoin matrix 0.32.0 and quire matrix 0.45.0 (engine 0.60.0). Both tools resolve from /home/peter/.npm-global/bin. The active process module is spec-artifacts-process@v0.31.0. Static counts are 425 tagged, 716 untagged, 26 method-without-symbol, 4 retired and 1 tagged-by-ignored-test (1172 total). The baseline archive of origin/main yields identical status for every criterion. The Quoin evidence store reports no run evidence. The coverage report contains 123 untracked symbols on both candidate and baseline; normalizing only the requested spellings yields identical symbol/tag sets, with no additions or removals. These are traceability results, not test-pass counts. Quire emits existing symbol-collision, comment-tag and missing-heading diagnostics; no diagnostic is represented as successful coverage. Optional full semantic audit was not requested and was skipped.

The source inventory contains 147 Rust source/test modules under src/ and crates/. In the reviewed delta there are zero new runtime behaviors, new public production APIs, new source stubs, new test stubs, new unsafe blocks or weakened CI lanes. All source edits are inside test code/feature selection. No new duplicate implementation or fixture content is introduced; git rename detection retains the fixture/helper bodies. Existing reference-based fixture access remains unchanged. The remaining executable/module Quire references name the current quire CLI, quire-rs or quire-canonical dependencies. Historical review excerpts retain old spellings as immutable evidence, not active targets. A tracked-file search found no qspec reference outside spec/, plan/ and reviews/.

The receipt oracle was inspected: real FSL and observation admission produce expected bytes independently of the IR receipt; tests assert byte equality, pointer borrowing, selected occurrence/clock, clone accounting, exact and one-below budgets, and typed ForeignEmission/ClauseMismatch failures. The prefix rename preserves length. The same real production path executes after the crate import rename. The model crate still has no FSL or observation dependency.

The root lock changes serde/serde_core/serde_derive to 1.0.228 and serde_json to 1.0.151, matching filament-observation's exact requirements. No downstream codegen/driver/integration lockfile was changed.

Execution evidence to date: make fmt-check passed; Python unittest discovery passed all 38 tests; make conformance-fspec-compile passed; make deny passed; the supplied read-only fsl-string-edge 0.1.0 source was built through locked-build.sh and make string-edge passed. Exact FILAMENT_SPEC_LANGUAGE_DIR=/tmp/ir1543-filament-spec-language-main make conformance-fsl-ir1384 passed all 3 receipt tests. The selected FSL checkout is clean. The complete make lint target passed after the scanner was available on PATH. The initial workspace run lacked FILAMENT_SPECIFICATION_DIR and the scanner on PATH, producing four accounting and five scanner test failures. Focused reruns supplied those prerequisites: all 10 whole-read accounting tests and all 6 scanner integration tests passed. The same fixture requirement exists on origin/main; these failures were setup failures, not introduced defects. The final lane results are recorded below.

## Execution Results

Focused validation PASS: make fmt-check; 38 Python tests; make conformance-fspec-compile; exact make conformance-fsl-ir1384 (3 passed); make lint; make deny; whole-read accounting rerun with FILAMENT_SPECIFICATION_DIR (10 passed); scanner integration rerun with the supplied scanner on PATH (6 passed). The initial broad workspace run lacked those two prerequisites, causing four accounting and five scanner failures; focused reruns cleared all nine.

Broad runs INCOMPLETE, stopped on the lead's explicit instruction to preserve focused review scope: make test was stopped after approximately 28 minutes overall, with 1454 individual passed results and nine setup failures reported across its started binaries, and two large integration tests still running. The independently executed FSpec binary was stopped after approximately 22 minutes at 442/443 passed, zero failures, with node_index::long_index_chain_admits_on_a_small_stack_with_shared_keys still running. Both stopped process groups returned exit 143 (SIGTERM). Neither is claimed as a passing full-suite gate; later make test stages were not reached. These are execution limits, not hidden passes. No aggregate merge gate was run.

No production behavior or oracle weakening was found in this rename delta. Historical matrix gaps remain outside this PR's change: repository-wide assurance is still FAIL even though the reviewed delta is clean. No fix round or dispositions were needed. No spec-review ran because spec/ and plan/ are unchanged; optional whole-repository semantic review was not requested. The frozen branch was not edited.

## Examined Scope

```yaml
scope:
- id: IR-1543-AC-1
  path: IR-1543
  role: examined
  excerpt: No file, directory, module or Makefile reference under filament-contract-ir carries a `qspec`/`quire`
    name except where the upstream name is still the real one.
- id: IR-1543-AC-2
  path: IR-1543
  role: examined
  excerpt: '`tests/conformance_fsl_ir1384/` resolves and builds against the renamed crates.'
- id: IR-1543-AC-3
  path: IR-1543
  role: examined
  excerpt: '`cargo fmt --check` passes and `cargo test` passes for the affected targets.'
- id: Cargo.lock
  path: Cargo.lock
  role: examined
  excerpt: "version = \"1.0.228\"\nchecksum = \"9a8e94ea7f378bd32cbbd37198a4a91436180c5bb472411e48b5ec2e2124ae9e\"\
    \nversion = \"1.0.228\"\nchecksum = \"41d385c7d4ca58e59fc732af25c3983b67ac852c1a25000afe1175de458b67ad\"\
    \nversion = \"1.0.228\"\nchecksum = \"d540f220d3187173da220f885ab66608367b6574e925011a9353e4badda91d79\"\
    \n \"syn 2.0.119\",\nversion = \"1.0.151\"\nchecksum = \"c841b55ecdae098c80dcae9cf767f6f8a0c2cdb3416bbef72181df4d0fe73f14\""
- id: Cargo.toml
  path: Cargo.toml
  role: examined
  excerpt: '# only `make conformance-fspec` runs its fixture tests.

    name = "conformance_fspec"

    path = "tests/conformance_fspec/main.rs"'
- id: Makefile
  path: Makefile
  role: examined
  excerpt: "\t@echo \"  make conformance-fspec-compile - Compile the FSpec conformance target without\
    \ fixtures (CI gate)\"\n\t@echo \"  make conformance-fspec - FSpec's positive fixtures admit, adverse\
    \ mutations refuse as recorded, selection identity derives (needs FILAMENT_SPECIFICATION_DIR; not\
    \ in ci)\"\n.PHONY: conformance-fspec-compile\nconformance-fspec-compile:\n\t$(CARGO) test $(LOCKED)\
    \ --workspace --features conformance-cli --no-run --test conformance_fspec --test monitor_plan_public\
    \ --test temporal_binding_public --test it\n.PHONY: conformance-fspec\nconformance-fspec:\n\t$(CARGO)\
    \ test $(LOCKED) -p filament-con"
- id: README.md
  path: README.md
  role: examined
  excerpt: Some conformance lanes (`make conformance-fspec`, `conformance-fsl-ir769` and
- id: crates/filament-contract-model/Cargo.toml
  path: crates/filament-contract-model/Cargo.toml
  role: examined
  excerpt: '# runs explicitly with conformance-fspec, outside ordinary workspace tests.

    # the explicit conformance-fspec lane, not the default workspace test gate.

    fspec-conformance = []'
- id: crates/filament-contract-model/src/checked_package/v2/identity.rs
  path: crates/filament-contract-model/src/checked_package/v2/identity.rs
  role: examined
  excerpt: "            Err(_) if std::env::var_os(\"FSPEC_CONFORMANCE\").is_none() => return,\n    fn\
    \ merged_fspec_structural_nominal_fixture_reads() {"
- id: crates/filament-contract-model/src/checked_package/v2/monitor_plan/selected_closure_walk.rs
  path: crates/filament-contract-model/src/checked_package/v2/monitor_plan/selected_closure_walk.rs
  role: examined
  excerpt: "#[cfg(all(test, feature = \"fspec-conformance\"))]\n#[cfg(all(test, feature = \"fspec-conformance\"\
    ))]\n#[cfg(all(test, feature = \"fspec-conformance\"))]\n#[cfg(all(test, feature = \"fspec-conformance\"\
    ))]\n#[cfg(all(test, feature = \"fspec-conformance\"))]\n#[cfg(all(test, feature = \"fspec-conformance\"\
    ))]\n    #[cfg(feature = \"fspec-conformance\")]\n    #[cfg(feature = \"fspec-conformance\")]\n  \
    \  #[cfg(feature = \"fspec-conformance\")]\n    #[cfg(feature = \"fspec-conformance\")]\n    #[cfg(feature\
    \ = \"fspec-conformance\")]\n    #[cfg(feature = \"fspec-conformance\")]\n    #[cfg(feature = \"fspec-conformance\"\
    )]"
- id: crates/filament-contract-model/src/checked_package/v2/protocol_body.rs
  path: crates/filament-contract-model/src/checked_package/v2/protocol_body.rs
  role: examined
  excerpt: "    type FSpecFrame = (ProtocolForm, &'static str, &'static [(&'static str, Shape)]);\n  \
    \  fn protocol_outer_layout_accepts_independent_fspec_frames() {\n        let cases: &[FSpecFrame]\
    \ = &["
- id: crates/filament-contract-model/src/checked_package/v2/vocabulary.rs
  path: crates/filament-contract-model/src/checked_package/v2/vocabulary.rs
  role: examined
  excerpt: '    fn timed_model_vocabulary_matches_fspec_eight_forms() {'
- id: crates/filament-contract-model/tests/temporal_binding_public.rs
  path: crates/filament-contract-model/tests/temporal_binding_public.rs
  role: examined
  excerpt: '        None if std::env::var_os("FSPEC_CONFORMANCE").is_none() => return,'
- id: tests/conformance_fsl_exact_producer/derived_shapes.rs
  path: tests/conformance_fsl_exact_producer/derived_shapes.rs
  role: examined
  excerpt: "    let fspec =\n    let proposal = Path::new(&fspec).join(\"proposals/checked-package-v2\"\
    );"
- id: tests/conformance_fsl_exact_producer/main.rs
  path: tests/conformance_fsl_exact_producer/main.rs
  role: examined
  excerpt: "    let fspec = env::var_os(\"FILAMENT_SPECIFICATION_DIR\")\n        &fs::read(fspec.join(\"\
    proposals/checked-package-v2/schema.json\"))"
- id: tests/conformance_fsl_ir1384/Cargo.toml
  path: tests/conformance_fsl_ir1384/Cargo.toml
  role: examined
  excerpt: 'filament-observation = { git = "https://github.com/agent-ix/filament-observation", branch
    = "main" }

    fsl-analyze = { git = "https://github.com/agent-ix/filament-spec-language", branch = "main" }

    fsl-foundation = { git = "https://github.com/agent-ix/filament-spec-language", branch = "main" }

    fsl-replay = { git = "https://github.com/agent-ix/filament-spec-language", branch = "main" }

    fsl-semantics = { git = "https://github.com/agent-ix/filament-spec-language", branch = "main" }'
- id: tests/conformance_fsl_ir1384/tests/receipt.rs
  path: tests/conformance_fsl_ir1384/tests/receipt.rs
  role: examined
  excerpt: "use fsl_analyze::monitor_root::{\nuse fsl_foundation::SourceIdentity;\nuse fsl_replay::spine::SpineLimits;\n\
    use fsl_replay::{MonitorTarget, TimingObligationSettings};\nuse fsl_semantics::check::TimeSourceKind;\n\
    use fsl_semantics::library::LibraryName;\nuse filament_observation::authority::{self, clock, progress,\
    \ SubjectSelection};\nfn domain_document(fsl: &std::path::Path) -> Vec<u8> {\n        &fs::read(fsl.join(\"\
    examples/config-version/model.semantic-ir.json\"))\nfn root_of(fsl: &std::path::Path, time_source:\
    \ &str) -> PlanRoot {\n    let document = domain_document(fsl);\n    let packages = fsl_semantics"
- id: tests/conformance_fsl_ir1384/tests/support/observation.rs
  path: tests/conformance_fsl_ir1384/tests/support/observation.rs
  role: examined
  excerpt: 'use filament_observation::authority::{

    use filament_observation::{'
- id: tests/conformance_fsl_ir760/main.rs
  path: tests/conformance_fsl_ir760/main.rs
  role: examined
  excerpt: '#[path = "../it/support/fspec_read_proof.rs"]

    mod fspec_read_proof;

    use fspec_read_proof::read_and_account;

    fn measure_fspec_read_requests(read: impl FnOnce()) -> u64 {

    fn without_fspec_instrumentation<T>(operation: impl FnOnce() -> T) -> T {'
- id: tests/conformance_fspec/carried_quantity.rs
  path: tests/conformance_fspec/carried_quantity.rs
  role: examined
  excerpt: use super::{proposal_dir_of, read_json};
- id: tests/conformance_fspec/certificate_member.rs
  path: tests/conformance_fspec/certificate_member.rs
  role: examined
  excerpt: fn model_certificate_members_match_fspec_schema() {
- id: tests/conformance_fspec/dependency_call.rs
  path: tests/conformance_fspec/dependency_call.rs
  role: examined
  excerpt: "use super::fspec_inputs::apply_dependency_call_patch as apply_patch;\n    let evidence = super::fspec_inputs::dependency_call_evidence_without_dependency(package);"
- id: tests/conformance_fspec/dimension_equivalence.rs
  path: tests/conformance_fspec/dimension_equivalence.rs
  role: examined
  excerpt: use super::{fixtures_dir, proposal_dir_of, read_json};
- id: tests/conformance_fspec/entry_keys.rs
  path: tests/conformance_fspec/entry_keys.rs
  role: examined
  excerpt: use super::monitor_plan_fixtures::*;
- id: tests/conformance_fspec/extension.rs
  path: tests/conformance_fspec/extension.rs
  role: examined
  excerpt: use super::*;
- id: tests/conformance_fspec/fairness_targets.rs
  path: tests/conformance_fspec/fairness_targets.rs
  role: examined
  excerpt: use filament_contract_model::CheckedFairnessTarget;
- id: tests/conformance_fspec/fma_application.rs
  path: tests/conformance_fspec/fma_application.rs
  role: examined
  excerpt: '#[path = "../it/support/fspec_fma_inputs.rs"]

    fn tc_048_fma_applications_follow_published_fspec_cases() {'
- id: tests/conformance_fspec/grouped_ordinal.rs
  path: tests/conformance_fspec/grouped_ordinal.rs
  role: examined
  excerpt: fn corrected_fspec_grouped_fixtures_admit_with_declared_paths() {
- id: tests/conformance_fspec/hybrid.rs
  path: tests/conformance_fspec/hybrid.rs
  role: examined
  excerpt: 'fn fspec_hybrid_family_in_published_package_admits() {

    fn fspec_hybrid_schema_defects_refuse_before_identity() {

    fn fspec_hybrid_body_edit_preserves_structural_identity_and_checks_body() {

    fn fspec_hybrid_dependency_shape_precedes_body_checks() {

    fn fspec_hybrid_capture_initializer_must_match_capture_type() {

    fn fspec_hybrid_capture_ignores_unrelated_same_name_and_level() {

    fn fspec_hybrid_braking_model_and_claim_admit_from_published_nodes() {

    fn fspec_hybrid_restored_model_and_claims_admit_from_published_nodes() {

    fn fspec_admitted_hybrid_fairness_maps_whole_nested_member_id() {

    fn fspec_hybr'
- id: tests/conformance_fspec/main.rs
  path: tests/conformance_fspec/main.rs
  role: examined
  excerpt: "//! compiles this target without running it. `make conformance-fspec` runs its\n#[path = \"\
    ../it/support/fspec_inputs.rs\"]\nmod fspec_inputs;\nuse fspec_inputs::{\nfn tc_048_positive_document_selection_follows_fspec_map()\
    \ {\n    let evidence = fspec_inputs::evidence_for_published(\nuse fspec_inputs::replace_at;\n   \
    \             if !fspec_inputs::flat_body_control_passes(&read_package(&control)?) {\nfn tc_048_fspec_positive_fixtures_admit()\
    \ {\nfn fspec_positive_fixtures_expose_complete_public_typed_reads() {\n            assert_fspec_term_record(&node.body,\
    \ &wire_node[\"body\"], &catalog);\nfn assert_fspec_t"
- id: tests/conformance_fspec/map_type_vectors.rs
  path: tests/conformance_fspec/map_type_vectors.rs
  role: examined
  excerpt: use super::{
- id: tests/conformance_fspec/model_option_types.rs
  path: tests/conformance_fspec/model_option_types.rs
  role: examined
  excerpt: '#[path = "../it/support/fspec_option_inputs.rs"]'
- id: tests/conformance_fspec/model_root.rs
  path: tests/conformance_fspec/model_root.rs
  role: examined
  excerpt: use super::fspec_inputs::apply_vector_patch;
- id: tests/conformance_fspec/model_tuple_union.rs
  path: tests/conformance_fspec/model_tuple_union.rs
  role: examined
  excerpt: '#[path = "../it/support/fspec_tuple_inputs.rs"]'
- id: tests/conformance_fspec/monitor_plan.rs
  path: tests/conformance_fspec/monitor_plan.rs
  role: examined
  excerpt: use super::monitor_plan_fixtures::*;
- id: tests/conformance_fspec/node_index.rs
  path: tests/conformance_fspec/node_index.rs
  role: examined
  excerpt: "use super::fspec_inputs::apply_node_index_patch as apply_patch;\n                super::fspec_inputs::operand_checks_pass(&actual),"
- id: tests/conformance_fspec/node_ordinal.rs
  path: tests/conformance_fspec/node_ordinal.rs
  role: examined
  excerpt: 'use super::fspec_inputs::ordinal_case_package as case_package;

    fn fspec_node_ordinal_uniqueness_cases_read_exactly() {'
- id: tests/conformance_fspec/operation_bodies.rs
  path: tests/conformance_fspec/operation_bodies.rs
  role: examined
  excerpt: use super::operation_effects::{expand_document, normalize_relationship, patch};
- id: tests/conformance_fspec/operation_effects.rs
  path: tests/conformance_fspec/operation_effects.rs
  role: examined
  excerpt: use super::{
- id: tests/conformance_fspec/probabilistic_claims.rs
  path: tests/conformance_fspec/probabilistic_claims.rs
  role: examined
  excerpt: use crate::fixture_support::{canonical, evidence_for, refresh_identity, settle, v2_all_families};
- id: tests/conformance_fspec/projection_read_form.rs
  path: tests/conformance_fspec/projection_read_form.rs
  role: examined
  excerpt: use super::{
- id: tests/conformance_fspec/protocol_locations.rs
  path: tests/conformance_fspec/protocol_locations.rs
  role: examined
  excerpt: '#[path = "../it/support/fspec_protocol_location_inputs.rs"]'
- id: tests/conformance_fspec/protocol_mutation_manifest.rs
  path: tests/conformance_fspec/protocol_mutation_manifest.rs
  role: examined
  excerpt: "fn fspec_protocol_mutation_v13_refuses_wrong_result_type() {\nfn fspec_protocol_mutation_v14_refuses_state_clause_protocol_member()\
    \ {\nfn current_fspec_protocol_reader_refusals_preserve_published_provenance() {\nfn current_fspec_protocol_layout_rows_refuse_at_exact_node_pointer()\
    \ {\nfn current_fspec_protocol_positive_packages_remain_admitted() {\nfn current_fspec_protocol_enumerations_remain_admitted()\
    \ {\nfn current_fspec_schema_valid_delivery_bounds_refuse_at_channel_node() {\n        assert_current_fspec_protocol_bodies_valid(&package,\
    \ \"channel\");\n    assert_current_fspec_protocol_bodies_valid(&pac"
- id: tests/conformance_fspec/protocol_vectors.rs
  path: tests/conformance_fspec/protocol_vectors.rs
  role: examined
  excerpt: 'fn fspec_k2_role_rename_changes_declared_and_operand_ids() {

    fn fspec_k4_poll_maximum_changes_body_without_changing_declared_id() {

    fn fspec_c4_removing_scheduling_retains_declared_clause_id() {

    fn fspec_k10_acyclic_protocol_member_admits_a_lone_group_label() {

    fn fspec_role_port_member_cases_use_the_selected_declaration_kind() {'
- id: tests/conformance_fspec/reaches_population.rs
  path: tests/conformance_fspec/reaches_population.rs
  role: examined
  excerpt: 'fn tc_048_fspec_reaches_population_cases_match_the_reader() {

    fn tc_233_fspec_unbounded_population_rows_read_with_exact_sources() {'
- id: tests/conformance_fspec/schedulability.rs
  path: tests/conformance_fspec/schedulability.rs
  role: examined
  excerpt: use super::*;
- id: tests/conformance_fspec/selection_vectors.rs
  path: tests/conformance_fspec/selection_vectors.rs
  role: examined
  excerpt: use crate::fixture_support as checked_package;
- id: tests/conformance_fspec/single_body.rs
  path: tests/conformance_fspec/single_body.rs
  role: examined
  excerpt: use super::{
- id: tests/conformance_fspec/temporal_aggregate.rs
  path: tests/conformance_fspec/temporal_aggregate.rs
  role: examined
  excerpt: use super::{
- id: tests/conformance_fspec/temporal_clock_bindings.rs
  path: tests/conformance_fspec/temporal_clock_bindings.rs
  role: examined
  excerpt: use super::*;
- id: tests/conformance_fspec/text_literals.rs
  path: tests/conformance_fspec/text_literals.rs
  role: examined
  excerpt: '#[path = "../it/support/fspec_text_inputs.rs"]

    fn fspec_text_bounds_shape_vectors_match_reader() {

    fn fspec_text_literal_vectors_match_reader() {'
- id: tests/conformance_fspec/timed_clock_no_row.rs
  path: tests/conformance_fspec/timed_clock_no_row.rs
  role: examined
  excerpt: use super::*;
- id: tests/conformance_fspec/timed_model.rs
  path: tests/conformance_fspec/timed_model.rs
  role: examined
  excerpt: use super::*;
- id: tests/conformance_fspec/timed_slots.rs
  path: tests/conformance_fspec/timed_slots.rs
  role: examined
  excerpt: use super::*;
- id: tests/conformance_fspec/transition.rs
  path: tests/conformance_fspec/transition.rs
  role: examined
  excerpt: use super::*;
- id: tests/fixtures/fold-binders/config-version-domain.json
  path: tests/fixtures/fold-binders/config-version-domain.json
  role: examined
  excerpt: '{"canonicalization":{"algorithm":"rfc8785-v1"},"constructs":[{"construct":{"identity":"none","meaning":"filament.meaning.model.object-type/v1","members":{},"shape":"record"},"kind":{"module":"example/config-version","name":"object_type"},"moduleVersion":"1.0.0"},{"construct":{"identity":"none","meaning":"filament.meaning.model.population/v1","members":{},"shape":"record"},"kind":{"module":"example/config-version","name":"population"},"moduleVersion":"1.0.0"},{"construct":{"identity":"none","meaning":"filament.meaning.model.value-type/v1","members":{},"shape":"record"},"kind":{"module":"example'
- id: tests/fixtures/inherited-dispatch/config-version-model.json
  path: tests/fixtures/inherited-dispatch/config-version-model.json
  role: examined
  excerpt: '              "path": "spec.fspec",'
- id: tests/fixtures/model-enum-projection/domain.json
  path: tests/fixtures/model-enum-projection/domain.json
  role: examined
  excerpt: '{"canonicalization":{"algorithm":"rfc8785-v1"},"constructs":[{"construct":{"identity":"none","meaning":"filament.meaning.model.object-type/v1","members":{},"shape":"record"},"kind":{"module":"example/config-version","name":"object_type"},"moduleVersion":"1.0.0"},{"construct":{"identity":"none","meaning":"filament.meaning.model.population/v1","members":{},"shape":"record"},"kind":{"module":"example/config-version","name":"population"},"moduleVersion":"1.0.0"},{"construct":{"identity":"none","meaning":"filament.meaning.model.value-type/v1","members":{},"shape":"record"},"kind":{"module":"example'
- id: tests/ir575_reader_memory.rs
  path: tests/ir575_reader_memory.rs
  role: examined
  excerpt: '#[path = "it/support/fspec_read_proof.rs"]

    mod fspec_read_proof;

    use fspec_read_proof::read_and_account;

    fn measure_fspec_read_requests(read: impl FnOnce()) -> u64 {

    fn without_fspec_instrumentation<T>(operation: impl FnOnce() -> T) -> T {'
- id: tests/it/checked_package_v2_budgeted_equivalence.rs
  path: tests/it/checked_package_v2_budgeted_equivalence.rs
  role: examined
  excerpt: "use crate::fspec_read_proof::assert_admitted_views_equal;\n    use crate::fspec_read_proof::equivalence::{take_oracle_visits,\
    \ OracleVisits, ReadViewScope};\n    use crate::fspec_read_proof::{equivalence::take_oracle_visits,\
    \ Counter, ReadViewScope};\n            crate::measure_fspec_read_requests(read)\n    let (core, core_events)\
    \ = crate::fspec_read_proof::observe_native_with_scope::<ScopeCounter>(\n    let (full, full_events)\
    \ = crate::fspec_read_proof::observe_native_with_scope::<ScopeCounter>(\n    let (legacy, legacy_events)\
    \ = crate::fspec_read_proof::observe_native::<ScopeCounter>(\n    use crate"
- id: tests/it/checked_package_v2_dimension_equivalence.rs
  path: tests/it/checked_package_v2_dimension_equivalence.rs
  role: examined
  excerpt: "        crate::fspec_read_proof::read_budget_and_account(\n        crate::fspec_read_proof::read_budget_and_account("
- id: tests/it/checked_package_v2_flat_wire.rs
  path: tests/it/checked_package_v2_flat_wire.rs
  role: examined
  excerpt: '//! FR-038-AC-118 is `tests/conformance_fspec`, which runs FSpec''s own

    //! mutations through `make conformance-fspec`.'
- id: tests/it/checked_package_v2_frame_entries.rs
  path: tests/it/checked_package_v2_frame_entries.rs
  role: examined
  excerpt: "    use crate::fspec_read_proof::observation::{measure_at, unpaid, Counter, UNPAID_BOUND};\n\
    \        crate::fspec_read_proof::read_budget_and_account("
- id: tests/it/checked_package_v2_graph_read_storage.rs
  path: tests/it/checked_package_v2_graph_read_storage.rs
  role: examined
  excerpt: '    crate::fspec_read_proof::read_budget_and_account('
- id: tests/it/checked_package_v2_identity_digests.rs
  path: tests/it/checked_package_v2_identity_digests.rs
  role: examined
  excerpt: //! single-node fixture. The published FSpec readback runs in conformance_fspec.
- id: tests/it/checked_package_v2_map_type.rs
  path: tests/it/checked_package_v2_map_type.rs
  role: examined
  excerpt: "        crate::fspec_read_proof::with_designated_read_provider(\n            || crate::fspec_read_proof::read_budget_and_account("
- id: tests/it/checked_package_v2_model_members/inherited_dispatch.rs
  path: tests/it/checked_package_v2_model_members/inherited_dispatch.rs
  role: examined
  excerpt: '            "origin": {"source": {"sourceIdentity": format!("ix://{PACKAGE}/spec"), "path":
    "spec.fspec", "startLine": 1, "startColumn": 1}},'
- id: tests/it/checked_package_v2_owners.rs
  path: tests/it/checked_package_v2_owners.rs
  role: examined
  excerpt: "fn retarget_fspec_application_argument(package: &mut Value, operation: &str, target: &Value)\
    \ {\nfn tc_229_fspec_published_population_binding_fixture_admits() {\n        _ if std::env::var_os(\"\
    FSPEC_CONFORMANCE\").is_none() => return,\n        retarget_fspec_application_argument(&mut package,\
    \ &operation, &population);\n        retarget_fspec_application_argument(&mut package, &also, &population);\n\
    fn tc_229_fspec_population_reference_cases_match_reader() {\n        _ if std::env::var_os(\"FSPEC_CONFORMANCE\"\
    ).is_none() => return,"
- id: tests/it/checked_package_v2_protocol_body.rs
  path: tests/it/checked_package_v2_protocol_body.rs
  role: examined
  excerpt: "        None if std::env::var_os(\"FSPEC_CONFORMANCE\").is_none() => return,\nfn fspec_typed_fairness_reader_vectors_match_owner_refusals()\
    \ {\n        None if std::env::var_os(\"FSPEC_CONFORMANCE\").is_none() => return,\n        crate::fspec_read_proof::read_budget_and_account("
- id: tests/it/checked_package_v2_protocol_reference.rs
  path: tests/it/checked_package_v2_protocol_reference.rs
  role: examined
  excerpt: 'fn acyclic_protocol_reference_with_group_matches_fspec_k10() {

    fn fspec_k14_plain_protocol_stays_outside_the_cyclic_group() {'
- id: tests/it/checked_package_v2_refinement.rs
  path: tests/it/checked_package_v2_refinement.rs
  role: examined
  excerpt: "        crate::fspec_read_proof::read_budget_and_account(\n            crate::fspec_read_proof::read_budget_and_account(\n\
    \    use crate::fspec_read_proof::observation::{measure_at, unpaid, Counter, UNPAID_BOUND};"
- id: tests/it/checked_package_v2_schedulability.rs
  path: tests/it/checked_package_v2_schedulability.rs
  role: examined
  excerpt: '            crate::fspec_read_proof::read_budget_and_account('
- id: tests/it/checked_package_v2_structural_keys.rs
  path: tests/it/checked_package_v2_structural_keys.rs
  role: examined
  excerpt: //! The published multi-package and graph-order vectors run in conformance_fspec.
- id: tests/it/checked_package_v2_temporal.rs
  path: tests/it/checked_package_v2_temporal.rs
  role: examined
  excerpt: //! fixtures are read by `make conformance-fspec`, FR-038-AC-107).
- id: tests/it/checked_package_v2_typed_terms.rs
  path: tests/it/checked_package_v2_typed_terms.rs
  role: examined
  excerpt: "    let fspec_positive_kinds = [\n    let covered: BTreeSet<&str> = fspec_positive_kinds"
- id: tests/it/checked_package_v2_union.rs
  path: tests/it/checked_package_v2_union.rs
  role: examined
  excerpt: //! (they are read by `make conformance-fspec`, FR-038-AC-107).
- id: tests/it/checked_package_v2_whole_read_accounting.rs
  path: tests/it/checked_package_v2_whole_read_accounting.rs
  role: examined
  excerpt: "#[path = \"support/fspec_fma_inputs.rs\"]\nmod fspec_fma_inputs;\n#[path = \"support/fspec_inputs.rs\"\
    ]\nmod fspec_inputs;\n#[path = \"support/fspec_option_inputs.rs\"]\nmod fspec_option_inputs;\n#[path\
    \ = \"support/fspec_protocol_location_inputs.rs\"]\nmod fspec_protocol_location_inputs;\n#[path =\
    \ \"support/fspec_text_inputs.rs\"]\nmod fspec_text_inputs;\n#[path = \"support/fspec_tuple_inputs.rs\"\
    ]\nmod fspec_tuple_inputs;\n    let (reference, events) = crate::fspec_read_proof::observe_native::<IntegrationCounter>(\n\
    \        let (exact, exact_events) = crate::fspec_read_proof::observe_native::<IntegrationCounter>(\n\
    \    "
- id: tests/it/main.rs
  path: tests/it/main.rs
  role: examined
  excerpt: 'mod source_obligation_filament;

    #[path = "support/fspec_read_proof.rs"]

    mod fspec_read_proof;

    use fspec_read_proof::read_and_account;

    fn measure_fspec_read_requests(read: impl FnOnce()) -> u64 {

    fn without_fspec_instrumentation<T>(operation: impl FnOnce() -> T) -> T {'
- id: tests/it/source_obligation_filament.rs
  path: tests/it/source_obligation_filament.rs
  role: examined
  excerpt: use std::{fs, path::Path};
- id: tests/it/support/fspec_fma_inputs.rs
  path: tests/it/support/fspec_fma_inputs.rs
  role: examined
  excerpt: use crate::support::checked_package as fixture_support;
- id: tests/it/support/fspec_inputs.rs
  path: tests/it/support/fspec_inputs.rs
  role: examined
  excerpt: use crate::support::checked_package as fixture_support;
- id: tests/it/support/fspec_option_inputs.rs
  path: tests/it/support/fspec_option_inputs.rs
  role: examined
  excerpt: use crate::support::checked_package as fixture;
- id: tests/it/support/fspec_protocol_location_inputs.rs
  path: tests/it/support/fspec_protocol_location_inputs.rs
  role: examined
  excerpt: use crate::support::checked_package::{
- id: tests/it/support/fspec_read_proof.rs
  path: tests/it/support/fspec_read_proof.rs
  role: examined
  excerpt: "        crate::measure_fspec_read_requests(read)\n    let (mode, test_name) = crate::without_fspec_instrumentation(||\
    \ {\n    let reference = crate::without_fspec_instrumentation(|| {\n    crate::without_fspec_instrumentation(||\
    \ {\n        let caller_requested = crate::measure_fspec_read_requests(|| {\n            worker_requested\
    \ = crate::without_fspec_instrumentation(|| {\n    let mode = crate::without_fspec_instrumentation(||\
    \ {\n    let reference = crate::without_fspec_instrumentation(|| {\n        crate::without_fspec_instrumentation(||\
    \ {\n    crate::without_fspec_instrumentation(|| {"
- id: tests/it/support/fspec_text_inputs.rs
  path: tests/it/support/fspec_text_inputs.rs
  role: examined
  excerpt: use crate::support::checked_package as fixture_support;
- id: tests/it/support/fspec_tuple_inputs.rs
  path: tests/it/support/fspec_tuple_inputs.rs
  role: examined
  excerpt: use crate::support::checked_package as fixture_support;
- id: tests/it/support/read_allocation_proof.rs
  path: tests/it/support/read_allocation_proof.rs
  role: examined
  excerpt: "use crate::fspec_read_proof::observation;\npub(crate) use crate::fspec_read_proof::ReadViewScope;\n\
    \        crate::fspec_read_proof::observe_native::<C>(bytes, evidence, limits, u64::MAX, u64::MAX);\n\
    \    crate::fspec_read_proof::assert_budgeted_results_equal("
- id: tests/read_entry_census_controls.rs
  path: tests/read_entry_census_controls.rs
  role: examined
  excerpt: '#[path = "it/support/fspec_read_proof.rs"]

    mod fspec_read_proof;

    use fspec_read_proof::read_and_account;

    fn measure_fspec_read_requests(read: impl FnOnce()) -> u64 {

    fn without_fspec_instrumentation<T>(operation: impl FnOnce() -> T) -> T {'
- id: tests/read_storage_alloc.rs
  path: tests/read_storage_alloc.rs
  role: examined
  excerpt: "        fspec_read_proof::assert_budgeted_results_equal(\n        fspec_read_proof::measure_seeded(&bytes,\
    \ &evidence, limits, seed.read_proof_snapshot());\n    let (_, results, complete) = fspec_read_proof::full_native_boundaries(\n\
    #[path = \"it/support/fspec_read_proof.rs\"]\nmod fspec_read_proof;\nuse fspec_read_proof::read_and_account;\n\
    use fspec_read_proof::read_and_account as fixture_read;\nfn measure_fspec_read_requests(read: impl\
    \ FnOnce()) -> u64 {\nfn without_fspec_instrumentation<T>(operation: impl FnOnce() -> T) -> T {"
- id: tools/read_path_proof/coverage_census.py
  path: tools/read_path_proof/coverage_census.py
  role: examined
  excerpt: "    (\"tests/it/support/fspec_read_proof.rs\", \"read_and_account\"): \"plain\",\n    (\"\
    tests/it/support/fspec_read_proof.rs\", \"measure_seeded\"): \"budgeted\",\n    (\"tests/it/support/fspec_read_proof.rs\"\
    , \"read_budget_and_account\"): \"budgeted\",\n    (\"tests/it/support/fspec_read_proof.rs\",\n  \
    \  wrapper = \"tests/it/support/fspec_read_proof.rs\""
- id: tools/read_path_proof/run_coverage_census.py
  path: tools/read_path_proof/run_coverage_census.py
  role: examined
  excerpt: "    \"fspec_read_proof::provider_scope_tests::provider_designated_read_preserves_the_original_scalar_refusal\"\
    : True,\n    \"fspec_read_proof::caller_budget_tests::caller_budget_observation_preserves_prior_charges_and_outer_capture\"\
    : True,\n    fspec = Path(config[\"fspec\"][\"path\"])\n    if git(fspec, \"status\", \"--porcelain\"\
    ):\n            if name == \"fspec_read_proof::provider_scope_tests::provider_designated_read_preserves_the_original_scalar_refusal\"\
    :\n    parser.add_argument(\"--corpus\", choices=[\"fspec\", \"tc048\"], default=\"fspec\")\n    parser.add_argument(\"\
    --fspec-dir\", type=Path, required=True)\n   "
- id: tools/read_path_proof/test_coverage_census.py
  path: tools/read_path_proof/test_coverage_census.py
  role: examined
  excerpt: "        (\"tests/it/support/fspec_read_proof.rs\", \"read_and_account\"): plain,\n       \
    \ (\"tests/it/support/fspec_read_proof.rs\", \"read_budget_and_account\"): caller,\n        (\"tests/it/support/fspec_read_proof.rs\"\
    , \"measure_seeded\"): 6*caller,\n        (\"tests/it/support/fspec_read_proof.rs\",\n    for file,\
    \ text in [(\"tests/conformance_fspec/main.rs\", \"crate::read_and_account(\"),\n                \
    \       (\"tests/it/checked_package_v2_graph_read_storage.rs\", \"crate::fspec_read_proof::read_budget_and_account(\"\
    )]:\n            row[\"filenames\"] = [name.replace(\"tests/it/support/\", \"tests/conformance_fspec/"
- id: tools/read_path_proof/test_run_coverage_census.py
  path: tools/read_path_proof/test_run_coverage_census.py
  role: examined
  excerpt: '            args = SimpleNamespace(out=root, mode="sample", repo=root, original_test=[], corpus="fspec")'
- id: FR-361-AC-1
  path: spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md
  role: context_only
  excerpt: An IR-owned strict admitted Timed monitor package with two claim occurrences of one clause
    and a row naming the second occurrence returns a closure whose receipt holds that occurrence, its
    own clock read and the row's exact byte slices; a row naming the first returns the first. Dense and
    tick sources with equal and different exact periods give the specified success or `TimeSourceMismatch`.
- id: FR-361-AC-2
  path: spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md
  role: context_only
  excerpt: Each of an independent strict read of equal package bytes, another package, a different clause,
    an absent ordinal, a configured clock, a dense row against a tick source and a different exact period
    returns its named `SelectedMonitorAuthorityErrorCode` at the stated locus, with charged work and no
    closure. Paired defects show the stated authority check order; a model-row defect beside an authority
    defect reports the model row, and an authority defect beside a source-row or equality defect reports
    the authority. `try_new` returns `SelectedMonitorAuthorityRowError` with component `Occurrence`, `O
- id: FR-361-AC-3
  path: spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md
  role: context_only
  excerpt: (Layout context implemented.) The root, entry and clause keys equal the plan's resolved structural
    keys. `package_lock()` and `occurrence()` are byte-equal to the corresponding member bytes of the
    admitted input. `profile()` and `clock_binding()` for dense and tick sources equal independently written
    expected RFC 8785 values with exactly the specified members; changing the model, tick operation, unit,
    period, source kind or profile each changes `clock_binding()`.
- id: FR-361-AC-4
  path: spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md
  role: context_only
  excerpt: 'The read succeeds at its exact measured work and storage allowance. One unit below the cumulative
    charge before each authority, layout-context and clock-binding charge refuses at that charge with
    its kind, amount and exact locus: `MonitorAuthority` for the admission, clause and occurrence inspections
    (including at an `OccurrenceMismatch`), and `MonitorSeed(k)` with the tabled form after resolution;
    no partial receipt escapes. The same read on a package clone yields equal values and charges.'
- id: FR-361-AC-5
  path: spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md
  role: context_only
  excerpt: API and dependency inspection shows `SelectedMonitorClosureRows::authority` is a required field,
    the row has private fields and one fallible constructor and borrows its byte values, the model crate
    gains no FSL, observation or runtime dependency, no byte value is decoded or re-encoded, no raw input
    buffer is retained, and no `PlanId`, digest or second identity appears.
- id: FR-361-AC-6
  path: spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md
  role: context_only
  excerpt: In `filament-integration`, a genuine `AdmittedPlanAuthority` row binds through the closure
    read on its own `admitted_package()`; the receipt bytes equal FSL's order value and the owner's association
    value. The same row against the package of a second admission of the same plan root's bytes refuses
    `ForeignEmission`. Depends on the FSL seam above.
bindings:
- test_id: genuine_fsl_authority_binds_dense_and_tick_emissions
  ac_id: FR-361-AC-6
  trace: correct
- test_id: mismatched_emissions_refuse_with_typed_authority_errors
  ac_id: FR-361-AC-6
  trace: correct
```
