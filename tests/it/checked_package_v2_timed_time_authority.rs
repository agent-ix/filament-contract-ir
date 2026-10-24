// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Isolated nominal Time authority checks over the owner-published structural fixture.
//! The fixture's tick source with Clock places is not evidence of full timed validity.

use crate::support::checked_package::{canonical, rename_node};
use ix_trace_rs::trace;
use quire_contract_model::{
    derive_structural_candidates, CheckedArtifactRef, CheckedExtensionDefinition,
    CheckedPackageEvidence, CheckedPackageIdentityPreimageRefV2, CheckedPackageIdentityPreimageV2,
    CheckedPackageReadLimits, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, CheckedPackageV2, CheckedPackageV2ReadResult,
    CheckedSemanticNodeV2, NominalIdentityPreimage,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const BYTES: u64 = 1 << 24;

fn proposal() -> Option<PathBuf> {
    match std::env::var("QUIRE_SPECIFICATION_DIR") {
        Ok(root) if !root.is_empty() => Some(Path::new(&root).join("proposals/checked-package-v2")),
        _ if std::env::var_os("QSPEC_CONFORMANCE").is_none() => None,
        _ => panic!("QUIRE_SPECIFICATION_DIR is required for QSpec conformance"),
    }
}

fn document(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).expect("owner document must exist"))
        .expect("owner document must be JSON")
}

fn read(wire: &Value, proposal: &Path) -> CheckedPackageV2ReadResult {
    let mut evidence = CheckedPackageEvidence::new();
    for feature in wire["lock"]["required_features"].as_array().unwrap() {
        evidence.support_feature(feature.as_str().unwrap());
    }
    let definitions = document(&proposal.join("extension-definitions.json"));
    for selected in wire["lock"]["definition_selections"].as_array().unwrap() {
        let published = definitions["definitions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["reference"] == *selected)
            .expect("selected owner definition");
        let reference: CheckedArtifactRef = serde_json::from_value(selected.clone()).unwrap();
        let definition: CheckedExtensionDefinition =
            serde_json::from_value(published["definition"].clone()).unwrap();
        evidence
            .insert_extension_definition(reference, definition)
            .unwrap();
    }
    for selected in wire["lock"]["model_selections"].as_array().unwrap() {
        assert_eq!(selected["identity"], "acme/orders");
        evidence
            .insert_domain_package_document(
                selected["digest"].as_str().unwrap(),
                std::fs::read(proposal.join("domain-package-acme-orders.json")).unwrap(),
            )
            .unwrap();
    }
    CheckedPackageV2::read(
        &canonical(wire),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    )
}

fn nominal_key(node: &Value) -> String {
    let preimage: NominalIdentityPreimage =
        serde_json::from_value(node["nominal_identity_preimage"].clone()).unwrap();
    preimage
        .digest(BYTES)
        .expect("owner nominal identity encoding")
}

fn remint(wire: &mut Value) {
    let count = wire["semantic_graph"]["nodes"].as_array().unwrap().len();
    for _ in 0..=count {
        let mut changed = false;
        for index in 0..count {
            let node = &wire["semantic_graph"]["nodes"][index];
            if node.get("nominal_identity_preimage").is_some() {
                let old = node["node_id"]["digest"].as_str().unwrap().to_owned();
                let new = nominal_key(node);
                if old != new {
                    rename_node(wire, &old, &new);
                    changed = true;
                }
            }
        }
        let nodes: Vec<CheckedSemanticNodeV2> =
            serde_json::from_value(wire["semantic_graph"]["nodes"].clone()).unwrap();
        let candidates = derive_structural_candidates(&nodes, BYTES, u64::MAX).unwrap();
        for candidate in candidates.nodes() {
            let index = usize::try_from(candidate.position()).unwrap();
            let old = nodes[index].node_id.digest.as_ref();
            let new = candidate.candidate_node_id().digest.as_ref();
            if old != new {
                rename_node(wire, old, new);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    for node in wire["semantic_graph"]["nodes"].as_array_mut().unwrap() {
        node["dependencies"]
            .as_array_mut()
            .unwrap()
            .sort_by(|a, b| {
                (a["domain"].as_str(), a["digest"].as_str())
                    .cmp(&(b["domain"].as_str(), b["digest"].as_str()))
            });
    }
    let nodes: Vec<CheckedSemanticNodeV2> =
        serde_json::from_value(wire["semantic_graph"]["nodes"].clone()).unwrap();
    let candidates = derive_structural_candidates(&nodes, BYTES, u64::MAX).unwrap();
    for candidate in candidates.nodes() {
        assert_eq!(
            candidate.candidate_node_id(),
            &nodes[usize::try_from(candidate.position()).unwrap()].node_id
        );
    }
    for node in &nodes {
        if let Some(preimage) = &node.nominal_identity_preimage {
            assert_eq!(
                preimage.digest(BYTES).unwrap(),
                node.node_id.digest.as_ref()
            );
        }
    }
    let selection: CheckedPackageIdentityPreimageV2 =
        serde_json::from_value(wire["identity_preimage"].clone()).unwrap();
    let preimage = CheckedPackageIdentityPreimageRefV2 {
        version: &selection.version,
        edition: &selection.edition,
        profile_selections: &selection.profile_selections,
        definition_selections: &selection.definition_selections,
        model_selections: &selection.model_selections,
        required_features: &selection.required_features,
        dependency_selections: &selection.dependency_selections,
        identity_projection: &nodes,
    };
    wire["package_id"]["digest"] = json!(quire_canonical::sha256(
        &preimage,
        quire_canonical::Limits::new(BYTES)
    )
    .unwrap()
    .to_string());
}

fn rejects_non_time_dimension(derived: bool) {
    let Some(proposal) = proposal() else {
        return;
    };
    let mut wire = document(&proposal.join("fixtures/positive-timed-model-nodes.json"));
    assert!(
        matches!(
            read(&wire, &proposal),
            CheckedPackageV2ReadResult::Admitted(_)
        ),
        "unchanged owner structural baseline must admit through this reader"
    );
    let original_id = wire["package_id"].clone();
    if derived {
        let mut base = wire["semantic_graph"]["nodes"][42].clone();
        base["nominal_identity_preimage"]["qualified_declaration"] =
            json!(["Example", "DurationBase"]);
        base["declaration"]["qualified_name"] = json!(["Example", "DurationBase"]);
        let key = nominal_key(&base);
        base["node_id"]["digest"] = json!(key);
        base["semantic_type"] = base["node_id"].clone();
        let base_id = base["node_id"].clone();
        let mut origin = wire["source_map"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["node_id"] == wire["semantic_graph"]["nodes"][42]["node_id"])
            .unwrap()
            .clone();
        origin["node_id"] = base_id.clone();
        wire["source_map"].as_array_mut().unwrap().push(origin);
        wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(base);
        wire["semantic_graph"]["nodes"][42]["nominal_identity_preimage"]["terms"] =
            json!([{"dimension_node_id": base_id, "exponent": "1"}]);
        wire["semantic_graph"]["nodes"][42]["dependencies"] = json!([base_id]);
    } else {
        wire["semantic_graph"]["nodes"][42]["nominal_identity_preimage"]["qualified_declaration"] =
            json!(["Example", "Length"]);
        wire["semantic_graph"]["nodes"][42]["declaration"]["qualified_name"] =
            json!(["Example", "Length"]);
    }
    remint(&mut wire);
    assert_ne!(wire["package_id"], original_id);
    assert_eq!(
        wire["semantic_graph"]["nodes"][43]["nominal_identity_preimage"]["dimension_node_id"],
        wire["semantic_graph"]["nodes"][52]["body"]["members"][1]["value"]["target"]
    );
    let refusal = match read(&wire, &proposal) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        CheckedPackageV2ReadResult::Admitted(_) => panic!(
            "timed model admitted a unit whose exact nominal dimension is {}",
            if derived {
                "derived Time"
            } else {
                "base Length"
            }
        ),
        other => panic!("expected typed time-quantity refusal, got {other:?}"),
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::IllTyped, Some(Cause::InvalidTimeQuantity)),
        "{refusal:?}"
    );
    assert!(
        refusal.locus.is_some(),
        "time authority refusal must identify the failing node"
    );
    assert!(
        refusal.path.is_some(),
        "time authority refusal must identify the failing member"
    );
}

#[trace("TC-048", "FR-038-AC-320")]
#[test]
fn timed_source_rejects_a_unit_of_a_non_time_base_dimension() {
    rejects_non_time_dimension(false);
}

#[trace("TC-048", "FR-038-AC-320")]
#[test]
fn timed_source_rejects_a_unit_of_a_derived_dimension_named_time() {
    rejects_non_time_dimension(true);
}
