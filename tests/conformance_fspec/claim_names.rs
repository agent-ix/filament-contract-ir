// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Published claim-name vectors (FSpec `claim-name-vectors.json`, FR-322-AC-457
//! to FR-322-AC-461, TC-2011, TC-2012, TC-2015) and the refinement body-name
//! vectors (FR-322-AC-200, TC-310), replayed against the production reader.

use super::*;
use fspec_inputs::read_json;
use temporal_clock_bindings::{apply_occurrence_patch, assert_vector_node_locus};

fn claim_vectors() -> Value {
    fspec_inputs::read_json_wire(&proposal_dir_of(&fixtures_dir()).join("claim-name-vectors.json"))
        .expect("published claim-name vectors")
}

/// The case `id`, the group it sits in, and the package it patches: the
/// published base with the vectors' own `base_patch` applied first.
fn patched(published: &Value, case: &Value) -> Value {
    let base = published["base"].as_str().expect("published base");
    let mut package =
        read_json(&proposal_dir_of(&fixtures_dir()).join(base)).expect("published claim-name base");
    let clause = package["semantic_graph"]["nodes"][3]["node_id"].clone();
    assert_eq!(
        package["semantic_graph"]["nodes"][3]["semantic_form"], "temporal_clause",
        "the vectors' <clause> is graph node 3"
    );
    for patch in published["base_patch"]
        .as_array()
        .expect("base patch")
        .iter()
        .chain(case["patch"].as_array().expect("case patch"))
    {
        apply_occurrence_patch(&mut package, patch, &clause);
    }
    package
}

/// The production read of `package` exactly as written, over the claim-name
/// vectors' own domain document.
fn read_as_published(package: &Value) -> CheckedPackageV2ReadResult {
    let document = proposal_dir_of(&fixtures_dir()).join(DOMAIN_DOCUMENT);
    let evidence = fspec_inputs::evidence_for_published(
        package,
        "claim-name vector",
        &document,
        true,
        None,
        &fixtures_dir(),
    )
    .expect("selected FSpec evidence");
    crate::read_and_account(
        &fixture_support::canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    )
}

fn replay(ids: &[&str]) {
    let published = claim_vectors();
    let mut seen = Vec::new();
    for group in ["positive", "schema_refusals", "semantic_refusals"] {
        for case in published[group].as_array().expect("cases") {
            let id = case["id"].as_str().expect("case id");
            if !ids.contains(&id) {
                continue;
            }
            seen.push(id.to_owned());
            let package = patched(&published, case);
            let outcome = read_as_published(&package);
            if group == "positive" {
                admits(id, case, &package, outcome);
            } else {
                refuses(id, group, case, &package, outcome);
            }
        }
    }
    assert_eq!(
        seen.len(),
        ids.len(),
        "every named vector case is published and replayed: {seen:?}"
    );
}

fn admits(id: &str, case: &Value, package: &Value, outcome: CheckedPackageV2ReadResult) {
    let CheckedPackageV2ReadResult::Admitted(admitted) = outcome else {
        panic!("{id}: expected admission, got {outcome:?}");
    };
    let rows = admitted.claim_names().expect("admitted claim names");
    let expected = case["names"].as_array().expect("expected names");
    let names: Vec<&str> = rows.iter().map(|row| row.name.as_ref()).collect();
    // A published name list holds each row's name, and `B` is the second
    // occurrence's own name where one node carries two rows.
    assert_eq!(names.len(), expected.len(), "{id}: {names:?}");
    for (row, name) in rows.iter().zip(expected) {
        assert_eq!(row.name.as_ref(), name.as_str().expect("name"), "{id}");
    }
    // The writer re-reads to the same rows, regions included.
    let encoded = encode_checked_package_v2(
        admitted.name(),
        admitted.lock(),
        admitted.graph(),
        CheckedPackageSourceCorrespondenceRef {
            source_map: admitted.source_map(),
            refinement_step_rows: admitted.refinement_step_rows(),
        },
        filament_contract_model::CheckedPackageSidecarsRef {
            operation_effects: None,
            operation_bodies: None,
            checked_term_data: admitted.checked_term_data(),
            temporal_bindings: admitted.temporal_bindings(),
            temporal_clause_names: admitted.temporal_clause_names(),
            claim_names: admitted.claim_names(),
            schedulability_claims: None,
        },
        admitted.capability_report(),
        admitted.diagnostics(),
        quire_canonical::Limits::new(1 << 24),
    )
    .expect("encoded named package");
    let output: Value = serde_json::from_slice(&encoded).expect("encoded package");
    let CheckedPackageV2ReadResult::Admitted(round_trip) = read_as_published(&output) else {
        panic!("{id}: writer read-back must admit");
    };
    assert_eq!(round_trip.claim_names(), admitted.claim_names(), "{id}");
    assert_eq!(
        round_trip.temporal_clause_names(),
        admitted.temporal_clause_names(),
        "{id}"
    );
    // One namespace: the temporal clause name and each claim name resolve to
    // their own row.
    let temporal = admitted.temporal_clause_names().expect("temporal names");
    assert_eq!(temporal.len(), 1, "{id}");
    if id == "CL-P5-one-namespace-lookup" {
        assert_eq!(temporal[0].name.as_ref(), "Bounded", "{id}");
        let claim = admitted
            .claim_name_for_path(&["ReachThree"])
            .expect("ReachThree resolves to the claim row");
        assert_eq!(claim.name.as_ref(), "ReachThree", "{id}");
        assert!(
            admitted.claim_name_for_path(&["Bounded"]).is_none(),
            "{id}: Bounded is the temporal row, not a claim row"
        );
    }
    // A published lookup names the row a path resolves to. A path that names a
    // temporal clause row is no claim name: the claim lookup finds nothing.
    if let Some(lookups) = case["lookups"].as_object() {
        for (name, pointer) in lookups {
            let pointer = pointer.as_str().expect("lookup pointer");
            let found = admitted.claim_name_for_path(&[name.as_str()]);
            if let Some(at) = pointer.strip_prefix("/temporal_clause_names/") {
                let at: usize = at.parse().expect("temporal row index");
                assert_eq!(
                    admitted.temporal_clause_names().expect("temporal rows")[at]
                        .name
                        .as_ref(),
                    name,
                    "{id}: {pointer} is the temporal row of {name}"
                );
                assert!(found.is_none(), "{id}: {name} is no claim name");
                continue;
            }
            let at: usize = pointer
                .strip_prefix("/claim_names/")
                .and_then(|index| index.parse().ok())
                .unwrap_or_else(|| panic!("{id}: lookup {name} names a row, got {pointer}"));
            assert_eq!(found, Some(&rows[at]), "{id}: {name} resolves to {pointer}");
        }
    }
    let _ = package;
}

fn refuses(
    id: &str,
    group: &str,
    case: &Value,
    package: &Value,
    outcome: CheckedPackageV2ReadResult,
) {
    let CheckedPackageV2ReadResult::Refused(refusal) = outcome else {
        panic!("{id}: expected refusal, got {outcome:?}");
    };
    let expected = &case["expected"];
    assert_eq!(code_word(refusal.code), expected["code"], "{id}: code");
    if let Some(cause) = expected["cause"].as_str() {
        assert_eq!(refusal.cause.map(cause_word), Some(cause), "{id}: cause");
    }
    let locus = expected["locus"].as_str().expect("expected pointer");
    assert_eq!(
        refusal.loci[0].path.as_ref().expect("pointer").to_string(),
        locus,
        "{id}: primary pointer"
    );
    let carrier = if locus.starts_with("/temporal_clause_names") {
        "/temporal_clause_names"
    } else {
        "/claim_names"
    };
    assert_vector_node_locus(&refusal, package, group, locus, carrier, id);
    // A refusal that names both rows carries the other row as its second
    // document position.
    if let Some(names) = expected["names"].as_array() {
        let named: Vec<&str> = names.iter().map(|row| row.as_str().expect("row")).collect();
        let held = locus.rsplit_once('/').map_or(locus, |(row, _)| row);
        let other = named
            .iter()
            .find(|row| **row != held)
            .expect("the other named row");
        let second = refusal
            .document_pointer
            .as_ref()
            .expect("second position")
            .to_string();
        assert!(
            second.starts_with(&format!("{other}/")),
            "{id}: names {named:?}, second position {second}"
        );
    }
}

/// Trace: TC-2011 step 4 (FSpec), FR-322-AC-457, FR-322-AC-458.
#[trace(
    "TC-477",
    "FR-371-AC-1",
    "FR-371-AC-2",
    "FR-371-AC-3",
    "FR-371-AC-4",
    "FR-371-AC-5"
)]
#[test]
fn published_claim_name_vectors_replay_rows_schema_and_join_refusals() {
    replay(&[
        "CL-P1-claim-rows",
        "CL-P2-shared-node-two-rows",
        "CL-P3-rename-keeps-identity",
        "CL-P4-scoped-qualified-name",
        "CL-S1-extra-member",
        "CL-S2-missing-qualified-name",
        "CL-S3-empty-qualified-name",
        "CL-R1-dangling-declaration",
        "CL-R2-temporal-clause-declaration",
        "CL-R3-unowned-occurrence",
        "CL-R4-qualified-name-other-segment",
        "CL-R5-duplicate-qualified-name",
        "CL-R6-repeated-key",
        "CL-R7-row-removed",
        "CL-R8-member-absent",
        "CL-R9-out-of-order",
    ]);
}

/// Trace: TC-2012 step 4 (FSpec), FR-322-AC-459, and TC-2036 step 3,
/// FR-322-AC-466 (`CL-R18`: the namespace is refused over equal regions).
#[trace("TC-477", "FR-371-AC-7", "FR-371-AC-11")]
#[test]
fn published_claim_name_vectors_replay_the_one_namespace() {
    replay(&[
        "CL-P5-one-namespace-lookup",
        "CL-R14-namespace-collision",
        "CL-R15-namespace-collision-scoped",
        "CL-R18-namespace-collision-one-region",
    ]);
}

/// Trace: TC-2035 step 5 (FSpec), FR-322-AC-465: the rows of one declaration
/// share its name, its own node answers a lookup, and declaration sets that do
/// not nest are ambiguous. `CL-R5` is the duplicate of two declarations.
#[trace("TC-477", "FR-371-AC-10")]
#[test]
fn published_claim_name_vectors_replay_the_rows_of_one_declaration() {
    replay(&[
        "CL-P7-one-declaration-two-nodes",
        "CL-P8-one-declaration-two-occurrences",
        "CL-R5-duplicate-qualified-name",
        "CL-R16-one-declaration-two-names",
        "CL-R17-own-node-not-nested",
    ]);
}

/// Trace: TC-2015 step 4 (FSpec), FR-322-AC-460, FR-322-AC-461.
#[trace("TC-477", "FR-371-AC-9")]
#[test]
fn published_claim_name_vectors_replay_the_declaration_regions() {
    replay(&[
        "CL-P1-claim-rows",
        "CL-P6-layout-moves-region-only",
        "CL-S4-temporal-region-removed",
        "CL-S5-claim-region-removed",
        "CL-S6-claim-region-not-source-region",
        "CL-R10-temporal-region-shrunk",
        "CL-R11-temporal-region-other-source",
        "CL-R12-claim-region-shrunk",
        "CL-R13-claim-region-other-source",
    ]);
}
