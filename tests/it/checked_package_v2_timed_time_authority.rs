// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Nominal Time authority checks from independent owner-published dense and tick controls.

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

fn fixture(proposal: &Path, tick: bool) -> Value {
    document(&proposal.join(if tick {
        "fixtures/positive-tick-timed-model-nodes.json"
    } else {
        "fixtures/positive-timed-model-nodes.json"
    }))
}

fn admitted(wire: &Value, proposal: &Path) -> CheckedPackageV2 {
    match read(wire, proposal) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("unchanged owner positive must admit: {other:?}"),
    }
}

fn rejects_non_time_dimension(derived: bool, tick: bool) {
    let Some(proposal) = proposal() else {
        return;
    };
    let mut wire = fixture(&proposal, tick);
    let _ = admitted(&wire, &proposal);
    let root = if tick { 45 } else { 52 };
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
        wire["semantic_graph"]["nodes"][root]["body"]["members"][1]["value"]["target"]
    );
    let refusal = match read(&wire, &proposal) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        CheckedPackageV2ReadResult::Admitted(_) => panic!(
            "{} timed model admitted a unit whose exact nominal dimension is {}",
            if tick { "tick" } else { "dense" },
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
    assert_eq!(
        serde_json::to_value(refusal.locus).unwrap(),
        wire["semantic_graph"]["nodes"][44]["node_id"],
        "time authority refusal must identify the source node"
    );
    assert_eq!(
        refusal.path.unwrap().to_string(),
        "/semantic_graph/nodes/44/body/members/1/value",
        "FR-322 AC120 locates a non-time unit at the source unit binding"
    );
}

#[trace("TC-048", "FR-038-AC-320")]
#[test]
fn timed_source_rejects_a_unit_of_a_non_time_base_dimension() {
    rejects_non_time_dimension(false, false);
}

#[trace("TC-048", "FR-038-AC-320")]
#[test]
fn timed_source_rejects_a_unit_of_a_derived_dimension_named_time() {
    rejects_non_time_dimension(true, false);
}

#[trace("TC-048", "FR-038-AC-320")]
#[test]
fn tick_source_rejects_a_unit_of_a_non_time_base_dimension() {
    rejects_non_time_dimension(false, true);
}

#[trace("TC-048", "FR-038-AC-320")]
#[test]
fn tick_source_rejects_a_unit_of_a_derived_dimension_named_time() {
    rejects_non_time_dimension(true, true);
}

#[trace("TC-048", "FR-038-AC-320")]
#[test]
fn owner_dense_and_tick_positive_controls_admit_complete_graphs() {
    let Some(proposal) = proposal() else {
        return;
    };
    for tick in [false, true] {
        let wire = fixture(&proposal, tick);
        let package = admitted(&wire, &proposal);
        assert_eq!(
            serde_json::to_value(&package.graph().nodes).unwrap(),
            wire["semantic_graph"]["nodes"]
        );
        assert_eq!(
            serde_json::to_value(package.package_id()).unwrap(),
            wire["package_id"]
        );
        let root = if tick { 45 } else { 52 };
        let source = &wire["semantic_graph"]["nodes"][44]["body"]["members"];
        assert_eq!(
            source[0]["value"]["value"],
            if tick { "tick" } else { "dense" }
        );
        for member in wire["semantic_graph"]["nodes"][root]["body"]["members"]
            .as_array()
            .unwrap()
            .iter()
            .skip(3)
        {
            assert_eq!(
                member["value"]["members"].as_array().unwrap().is_empty(),
                tick
            );
        }
        println!(
            "owner {} positive admitted complete graph",
            if tick { "tick" } else { "dense" }
        );
    }
}

// FR-415 AC2/AC8 specifies invalid_timed_model/clock-without-dense-time.
// The IR refusal enum currently has neither member; this checks refusal only,
// without assigning the owner diagnostic an invented IR alias or locus.
#[trace("TC-048", "FR-415-AC-2", "FR-415-AC-8")]
#[test]
fn tick_source_with_clock_places_requires_refusal() {
    let Some(proposal) = proposal() else {
        return;
    };
    let mut wire = fixture(&proposal, false);
    let tick = fixture(&proposal, true);
    let _ = admitted(&wire, &proposal);
    let _ = admitted(&tick, &proposal);
    wire["semantic_graph"]["nodes"][44]["body"] =
        tick["semantic_graph"]["nodes"][44]["body"].clone();
    wire["semantic_graph"]["nodes"][44]["dependencies"] =
        tick["semantic_graph"]["nodes"][44]["dependencies"].clone();
    for index in [46, 48] {
        let node = tick["semantic_graph"]["nodes"][index].clone();
        let origin = tick["source_map"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["node_id"] == node["node_id"])
            .unwrap()
            .clone();
        wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(node);
        wire["source_map"].as_array_mut().unwrap().push(origin);
    }
    remint(&mut wire);
    match read(&wire, &proposal) {
        CheckedPackageV2ReadResult::Refused(refusal) => println!("unmapped owner tick+Clock refusal: {refusal:?}"),
        CheckedPackageV2ReadResult::Admitted(_) => panic!("tick source admitted the dense control's Clock places; owner refusal mapping remains absent"),
        other => panic!("expected typed owner refusal, got {other:?}"),
    }
}
