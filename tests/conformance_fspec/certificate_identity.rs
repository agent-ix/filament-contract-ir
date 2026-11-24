// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-412-AC-34: certificate identities use recursive structural NodeIds,
//! while checked-package path segments keep their separate wire grammar.

use super::*;
use jsonschema::{Draft, JSONSchema};

fn specification_root() -> PathBuf {
    let root = std::env::var_os(SPECIFICATION_DIR)
        .filter(|root| !root.is_empty())
        .unwrap_or_else(|| {
            panic!("{SPECIFICATION_DIR} must name a filament-specification checkout")
        });
    PathBuf::from(root)
}

fn local_definition_schema(schema: &Value, definition: &str, referenced: &[&str]) -> Value {
    let source = schema["$defs"]
        .as_object()
        .expect("published schema definitions");
    let mut definitions = serde_json::Map::new();
    for name in std::iter::once(&definition).chain(referenced.iter()) {
        let value = source
            .get(*name)
            .unwrap_or_else(|| panic!("schema is missing $defs.{name}"));
        definitions.insert((*name).to_owned(), value.clone());
    }
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$defs": definitions,
        "$ref": format!("#/$defs/{definition}")
    })
}

fn compile(schema: &Value) -> JSONSchema {
    JSONSchema::options()
        .with_draft(Draft::Draft202012)
        .compile(schema)
        .expect("published certificate schema compiles")
}

fn assert_valid(schema: &JSONSchema, value: &Value, label: &str) {
    if let Err(mut errors) = schema.validate(value) {
        panic!(
            "{label} should be admitted: {}",
            errors.next().expect("schema validation error")
        );
    }
}

fn assert_refused(schema: &JSONSchema, value: &Value, label: &str) {
    assert!(schema.validate(value).is_err(), "{label} should be refused");
}

fn structural_node() -> Value {
    json!({
        "package": "pkg",
        "path": [
            "claim",
            {"package": "pkg", "path": ["operand"], "ordinal": 2}
        ],
        "ordinal": 7
    })
}

#[trace("FR-412-AC-34")]
#[test]
fn certificate_identity_admits_recursive_structural_node_id_and_refuses_malformed_segments() {
    let root = specification_root();
    let backend = read_json(&root.join("proposals/backend-provider-v1/schema.json"))
        .expect("FSpec backend-provider-v1 schema");
    let certificate_identity = local_definition_schema(
        &backend,
        "CertificateIdentity",
        &[
            "CertificateNodeId",
            "CertificatePathSegment",
            "Nonempty",
            "Count",
        ],
    );
    let validator = compile(&certificate_identity);
    let identity = json!({
        "node_id": structural_node(),
        "model": "pkg",
        "ordinal": 0
    });
    assert_valid(
        &validator,
        &identity,
        "canonical nested certificate identity",
    );

    let mut malformed = Vec::new();
    for member in ["package", "path", "ordinal"] {
        let mut value = identity.clone();
        value["node_id"]["path"][1]
            .as_object_mut()
            .expect("nested NodeId object")
            .remove(member);
        malformed.push((format!("nested NodeId missing {member}"), value));
    }
    let mut extra = identity.clone();
    extra["node_id"]["path"][1]["extra"] = json!(true);
    malformed.push(("nested NodeId extra member".to_owned(), extra));

    let mut empty_package = identity.clone();
    empty_package["node_id"]["path"][1]["package"] = json!("");
    malformed.push(("empty nested package".to_owned(), empty_package));
    let mut empty_path = identity.clone();
    empty_path["node_id"]["path"][1]["path"] = json!([]);
    malformed.push(("empty nested path".to_owned(), empty_path));
    let mut mistyped = identity.clone();
    mistyped["node_id"]["path"][1]["ordinal"] = json!("2");
    malformed.push(("mistyped nested ordinal".to_owned(), mistyped));
    let mut malformed_nested = identity.clone();
    malformed_nested["node_id"]["path"][1]["path"] = json!([{"package":"pkg", "path":["operand"]}]);
    malformed.push((
        "malformed recursively nested NodeId".to_owned(),
        malformed_nested,
    ));
    let mut empty_root_package = identity.clone();
    empty_root_package["node_id"]["package"] = json!("");
    malformed.push(("empty root package".to_owned(), empty_root_package));
    let mut empty_root_path = identity.clone();
    empty_root_path["node_id"]["path"] = json!([]);
    malformed.push(("empty root path".to_owned(), empty_root_path));

    for (label, value) in malformed {
        assert_refused(&validator, &value, &label);
    }
}

#[trace("FR-412-AC-34")]
#[test]
fn checked_package_path_segment_still_rejects_certificate_node_id_objects() {
    let root = specification_root();
    let checked = read_json(&root.join("proposals/checked-package-v2/schema.json"))
        .expect("FSpec checked-package-v2 schema");
    let path_segment =
        local_definition_schema(&checked, "PathSegment", &["NodeIndex", "OperandSlot"]);
    let validator = compile(&path_segment);
    assert_valid(&validator, &json!("claim"), "checked-package name segment");
    assert_valid(
        &validator,
        &json!({"node": 2}),
        "checked-package node index",
    );
    assert_valid(
        &validator,
        &json!({"operand": 1}),
        "checked-package operand slot",
    );
    assert_refused(
        &validator,
        &json!({"package":"pkg", "path":["operand"], "ordinal":2}),
        "certificate structural NodeId used as checked-package path segment",
    );

    let checked_node: Result<CheckedNodeId, _> = serde_json::from_value(json!({
        "package": "pkg",
        "path": ["claim", {"package":"pkg", "path":["operand"], "ordinal":2}],
        "ordinal": 7
    }));
    assert!(
        checked_node.is_err(),
        "the checked-package model must not deserialize certificate path objects"
    );
}
