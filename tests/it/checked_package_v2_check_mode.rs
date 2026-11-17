// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FSpec FR-770: the function node's `check_mode` member in the V2 reader and
//! writer (FR-038-AC-555 through FR-038-AC-557).

use crate::support::checked_package::{canonical, evidence_for, v2_all_families};
use filament_contract_model::{
    encode_checked_package_v2, CheckMode, CheckedPackageReadLimits, CheckedPackageRefusalCode,
    CheckedPackageSidecarsRef, CheckedPackageSourceCorrespondenceRef, CheckedPackageV2,
    CheckedPackageV2ReadResult,
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

#[trace("TC-048", "FR-038-AC-555")]
#[test]
fn check_mode_is_typed_identity_bearing_and_written() {
    let linked = v2_all_families();
    let position = function_position(&linked);
    assert_eq!(
        linked["semantic_graph"]["nodes"][position]["check_mode"],
        "linked"
    );
    let mut kernel = linked.clone();
    kernel["semantic_graph"]["nodes"][position]["check_mode"] = json!("kernel");

    let (linked_read, kernel_read) = (admitted(&linked), admitted(&kernel));
    assert_eq!(
        linked_read.graph().nodes[position].check_mode,
        Some(CheckMode::Linked)
    );
    assert_eq!(
        kernel_read.graph().nodes[position].check_mode,
        Some(CheckMode::Kernel)
    );
    assert_ne!(linked_read.graph(), kernel_read.graph());

    let (linked_out, kernel_out) = (written(&linked_read), written(&kernel_read));
    assert_ne!(linked_out, kernel_out);
    for (out, mode) in [(&linked_out, "linked"), (&kernel_out, "kernel")] {
        for node in out["semantic_graph"]["nodes"].as_array().expect("nodes") {
            if node["node_tag"] == "function" {
                assert!(node["check_mode"] == mode || node["check_mode"] == "linked");
            } else {
                assert!(node.get("check_mode").is_none());
            }
        }
    }
    assert_eq!(
        kernel_out["semantic_graph"]["nodes"][position]["check_mode"],
        "kernel"
    );
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
