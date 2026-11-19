// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FSpec FR-770: the function node's `check_mode` member in the V2 reader and
//! writer (FR-038-AC-555 through FR-038-AC-557).

use crate::support::checked_package::{canonical, evidence_for, v2_all_families};
use filament_contract_model::{
    encode_checked_package_v2, CheckMode, CheckedNodeProjectionV2, CheckedPackageReadLimits,
    CheckedPackageRefusalCode, CheckedPackageSidecarsRef, CheckedPackageSourceCorrespondenceRef,
    CheckedPackageV2, CheckedPackageV2ReadResult,
};
use ix_trace_rs::trace;
use serde_json::{json, Value};

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    crate::read_and_account(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

fn admitted(package: &Value) -> Box<CheckedPackageV2> {
    match read(package) {
        CheckedPackageV2ReadResult::Admitted(read) => read,
        other => panic!("expected admission: {other:?}"),
    }
}

fn function_position(package: &Value) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "function" && node.get("declaration").is_some())
        .expect("a declared function node")
}

fn expect_refusal(package: &Value, code: CheckedPackageRefusalCode, path: &str, why: &str) {
    let CheckedPackageV2ReadResult::Refused(refusal) = read(package) else {
        panic!("{why}: expected a refusal");
    };
    assert_eq!(refusal.code, code, "{why}");
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|path| path.as_str()),
        Some(path),
        "{why}"
    );
}

fn written(package: &CheckedPackageV2) -> Value {
    let bytes = encode_checked_package_v2(
        package.name(),
        package.lock(),
        package.graph(),
        CheckedPackageSourceCorrespondenceRef {
            source_map: package.source_map(),
            refinement_step_rows: package.refinement_step_rows(),
        },
        CheckedPackageSidecarsRef {
            temporal_clause_names: package.temporal_clause_names(),
            claim_names: package.claim_names(),
            operation_effects: None,
            operation_bodies: None,
            schedulability_claims: None,
            checked_term_data: package.checked_term_data(),
            temporal_bindings: package.temporal_bindings(),
        },
        package.capability_report(),
        package.diagnostics(),
        quire_canonical::Limits::new(1 << 24),
    )
    .expect("encode");
    serde_json::from_slice(&bytes).expect("written JSON")
}

fn function_positions(package: &Value) -> Vec<usize> {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .enumerate()
        .filter(|(_, node)| node["node_tag"] == "function")
        .map(|(position, _)| position)
        .collect()
}

#[trace("TC-048", "FR-038-AC-555")]
#[test]
fn check_mode_is_typed_identity_bearing_and_written() {
    let linked = v2_all_families();
    let functions = function_positions(&linked);
    assert!(
        functions.len() >= 2,
        "a declared function and a second function node"
    );
    for position in &functions {
        assert_eq!(
            linked["semantic_graph"]["nodes"][*position]["check_mode"],
            "linked"
        );
    }
    let linked_read = admitted(&linked);
    let linked_out = written(&linked_read);
    // One function node `kernel`, then every function node `kernel`.
    for kernel_set in [&functions[..1], &functions[..]] {
        let mut kernel = linked.clone();
        for position in kernel_set {
            kernel["semantic_graph"]["nodes"][*position]["check_mode"] = json!("kernel");
        }
        let kernel_read = admitted(&kernel);
        assert_ne!(linked_read.graph(), kernel_read.graph());
        let kernel_out = written(&kernel_read);
        assert_ne!(linked_out, kernel_out);
        for (position, node) in kernel_out["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .enumerate()
        {
            if functions.contains(&position) {
                let expected = if kernel_set.contains(&position) {
                    ("kernel", CheckMode::Kernel)
                } else {
                    ("linked", CheckMode::Linked)
                };
                assert_eq!(node["check_mode"], expected.0);
                assert_eq!(
                    kernel_read.graph().nodes[position].check_mode,
                    Some(expected.1)
                );
            } else {
                assert!(node.get("check_mode").is_none());
                assert_eq!(kernel_read.graph().nodes[position].check_mode, None);
            }
        }
    }
}

fn projected(projection: &CheckedNodeProjectionV2) -> Value {
    let mut bytes = Vec::new();
    quire_canonical::encode(
        &mut bytes,
        projection,
        quire_canonical::Limits::new(1 << 24),
    )
    .expect("projection encodes");
    serde_json::from_slice(&bytes).expect("projection JSON")
}

#[trace("TC-048", "FR-038-AC-555")]
#[test]
fn the_node_projection_copies_and_writes_check_mode() {
    let linked = v2_all_families();
    let position = function_position(&linked);
    let mut kernel = linked.clone();
    kernel["semantic_graph"]["nodes"][position]["check_mode"] = json!("kernel");
    for (package, mode, name) in [
        (&linked, CheckMode::Linked, "linked"),
        (&kernel, CheckMode::Kernel, "kernel"),
    ] {
        let read = admitted(package);
        let nodes = &read.graph().nodes;
        let projection = CheckedNodeProjectionV2::from(&nodes[position]);
        assert_eq!(projection.check_mode, Some(mode));
        let emitted = projected(&projection);
        assert_eq!(emitted["check_mode"], name);
        let other = nodes
            .iter()
            .find(|node| node.node_tag.as_ref() != "function")
            .expect("a non-function node");
        let other = CheckedNodeProjectionV2::from(other);
        assert_eq!(other.check_mode, None);
        assert!(projected(&other).get("check_mode").is_none());
    }
}

#[trace("TC-048", "FR-038-AC-556")]
#[test]
fn a_function_node_without_a_valid_check_mode_refuses_malformed_wire() {
    let base = v2_all_families();
    let position = function_position(&base);
    let pointer = format!("/semantic_graph/nodes/{position}/check_mode");
    let mut absent = base.clone();
    absent["semantic_graph"]["nodes"][position]
        .as_object_mut()
        .expect("node")
        .remove("check_mode");
    expect_refusal(
        &absent,
        CheckedPackageRefusalCode::MalformedWire,
        &pointer,
        "absent",
    );
    for (name, value) in [
        ("strict", json!("strict")),
        ("empty", json!("")),
        ("boolean", json!(true)),
        ("null", Value::Null),
        ("capitalised", json!("Linked")),
    ] {
        let mut forged = base.clone();
        forged["semantic_graph"]["nodes"][position]["check_mode"] = value;
        expect_refusal(
            &forged,
            CheckedPackageRefusalCode::MalformedWire,
            &pointer,
            name,
        );
    }
}

#[trace("TC-048", "FR-038-AC-557")]
#[test]
fn check_mode_on_a_non_function_node_refuses_malformed_wire() {
    let base = v2_all_families();
    let nodes = base["semantic_graph"]["nodes"].as_array().expect("nodes");
    let expression = nodes
        .iter()
        .position(|node| node["node_tag"] == "expression")
        .expect("an expression node");
    for value in [json!("linked"), json!("kernel"), Value::Null] {
        let mut package = base.clone();
        package["semantic_graph"]["nodes"][expression]["check_mode"] = value;
        expect_refusal(
            &package,
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{expression}/check_mode"),
            "expression",
        );
    }
}
