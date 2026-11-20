// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FSpec's published temporal aggregate argument fragments in complete packages.

use super::{
    cause_word, code_word, fixture_support, fixtures_dir, proposal_dir_of, read_json, read_package,
    read_package_with_document,
};
use filament_contract_model::{CheckedNodeId, CheckedPackageV2ReadResult, CheckedTerm};
use ix_trace_rs::trace;
use serde_json::{json, Value};
use std::collections::BTreeSet;

fn vectors() -> Value {
    read_json(&proposal_dir_of(&fixtures_dir()).join("temporal-aggregate-vectors.json"))
        .expect("published FSpec temporal aggregate arguments")
}

fn id(path: &[&str]) -> Value {
    json!({"package": "example", "path": path, "ordinal": 0})
}

fn other_id(path: &[&str]) -> Value {
    // Source declarations share the checked package identity; this fixture
    // uses only one owner at a time so the one-segment name stays unambiguous.
    json!({"package": "example", "path": path, "ordinal": 0})
}

fn source_row(node: &Value) -> Value {
    let occurrence = &node["occurrences"][0];
    json!({
        "node_id": node["node_id"],
        "role": occurrence["role"],
        "ordinal": occurrence["ordinal"],
        "regions": [{
            "source": {"authority": "agent-ix", "identity": "example"},
            "start": 0, "end": 1,
        }],
    })
}

fn add_node(package: &mut Value, node: Value) {
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(source_row(&node));
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes")
        .push(node);
}

fn dimension(name: &str) -> Value {
    let key = id(&["scalar_type", "dimension", name]);
    json!({
        "node_id": key, "schema_version": "filament.checked-semantic-graph/v2",
        "node_tag": "scalar_type", "semantic_form": "dimension", "semantic_type": key,
        "declaration": {"qualified_name": ["scalar_type", "dimension", name]},
        "dependencies": [], "occurrences": [{"role": "declaration", "ordinal": 0}],
        "owner": {"kind": "source", "authority": "agent-ix", "identity": "example"},
        "body": {"term": "dimension", "base": true, "terms": []},
    })
}

fn unit(name: &str, dimension: &str) -> Value {
    let key = id(&["scalar_type", "unit", name]);
    let dimension = id(&["scalar_type", "dimension", dimension]);
    let target = (name == "s").then(|| id(&["scalar_type", "unit", "ms"]));
    let scale = if name == "s" { "1000" } else { "1" };
    let dependencies = if let Some(target) = &target {
        json!([dimension, target])
    } else {
        json!([dimension])
    };
    json!({
        "node_id": key, "schema_version": "filament.checked-semantic-graph/v2",
        "node_tag": "scalar_type", "semantic_form": "unit", "semantic_type": dimension,
        "declaration": {"qualified_name": ["scalar_type", "unit", name]},
        "dependencies": dependencies, "occurrences": [{"role": "declaration", "ordinal": 0}],
        "owner": {"kind": "source", "authority": "agent-ix", "identity": "example"},
        "body": {"term": "unit", "dimension_node_id": dimension,
                 "target_unit_node_id": target,
                 "scale": {"numerator": scale, "denominator": "1"},
                 "offset": {"numerator": "0", "denominator": "1"}},
    })
}

fn source_dimension(name: &str) -> Value {
    let key = id(&[name]);
    json!({
        "node_id": key, "schema_version": "filament.checked-semantic-graph/v2",
        "node_tag": "scalar_type", "semantic_form": "dimension", "semantic_type": key,
        "declaration": {"qualified_name": [name]},
        "dependencies": [], "occurrences": [{"role": "declaration", "ordinal": 0}],
        "owner": {"kind": "source", "authority": "agent-ix", "identity": "example"},
        "body": {"term": "dimension", "base": true, "terms": []},
    })
}

fn source_unit(name: &str, dimension: &str) -> Value {
    let key = id(&[name]);
    let dimension = id(&[dimension]);
    let target = (name == "ms").then(|| id(&["s"]));
    let scale = if name == "ms" {
        json!({"numerator": "1", "denominator": "1000"})
    } else {
        json!({"numerator": "1", "denominator": "1"})
    };
    let dependencies = if let Some(target) = &target {
        json!([dimension, target])
    } else {
        json!([dimension])
    };
    json!({
        "node_id": key, "schema_version": "filament.checked-semantic-graph/v2",
        "node_tag": "scalar_type", "semantic_form": "unit", "semantic_type": dimension,
        "declaration": {"qualified_name": [name]},
        "dependencies": dependencies,
        "occurrences": [{"role": "declaration", "ordinal": 0}],
        "owner": {"kind": "source", "authority": "agent-ix", "identity": "example"},
        "body": {"term": "unit", "dimension_node_id": dimension,
                 "target_unit_node_id": target,
                 "scale": scale,
                 "offset": {"numerator": "0", "denominator": "1"}},
    })
}

fn other_source_dimension() -> Value {
    let key = other_id(&["Time"]);
    json!({
        "node_id": key, "schema_version":"filament.checked-semantic-graph/v2",
        "node_tag":"scalar_type", "semantic_form":"dimension", "semantic_type":key,
        "declaration":{"qualified_name":["Time"]}, "dependencies":[],
        "occurrences":[{"role":"declaration","ordinal":0}],
        "owner":{"kind":"source","authority":"agent-ix","identity":"other"},
        "body":{"term":"dimension","base":true,"terms":[]}
    })
}

fn other_source_unit(name: &str) -> Value {
    let key = other_id(&[name]);
    let dimension = other_id(&["Time"]);
    let target = (name == "ms").then(|| other_id(&["s"]));
    let scale = if name == "ms" {
        json!({"numerator":"1","denominator":"1000"})
    } else {
        json!({"numerator":"1","denominator":"1"})
    };
    let dependencies = target
        .as_ref()
        .map_or_else(|| json!([dimension]), |target| json!([dimension, target]));
    json!({
        "node_id":key, "schema_version":"filament.checked-semantic-graph/v2",
        "node_tag":"scalar_type", "semantic_form":"unit", "semantic_type":dimension,
        "declaration":{"qualified_name":[name]}, "dependencies":dependencies,
        "occurrences":[{"role":"declaration","ordinal":0}],
        "owner":{"kind":"source","authority":"agent-ix","identity":"other"},
        "body":{"term":"unit","dimension_node_id":dimension,
            "target_unit_node_id":target,"scale":scale,
            "offset":{"numerator":"0","denominator":"1"}}
    })
}

fn add_other_source_units(package: &mut Value) {
    package["lock"]["sources"]
        .as_array_mut()
        .expect("source lock")
        .push(json!({"authority":"agent-ix","identity":"other"}));
    for node in [
        other_source_dimension(),
        other_source_unit("ms"),
        other_source_unit("s"),
    ] {
        let id = node["node_id"].clone();
        package["source_map"]
            .as_array_mut()
            .expect("source map")
            .push(json!({
                "node_id":id,"role":"declaration","ordinal":0,
                "regions":[{"source":{"authority":"agent-ix","identity":"other"},
                    "start":0,"end":1}]
            }));
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("semantic nodes")
            .push(node);
    }
}

fn replace_value(value: &mut Value, from: &Value, to: &Value) {
    if value == from {
        *value = to.clone();
        return;
    }
    match value {
        Value::Array(values) => {
            for value in values {
                replace_value(value, from, to);
            }
        }
        Value::Object(values) => {
            for value in values.values_mut() {
                replace_value(value, from, to);
            }
        }
        _ => {}
    }
}

fn source_duration_package(
    mut case: Value,
    mut published: Value,
    bound_unit: Value,
    include_clause_owner_units: bool,
) -> Value {
    published["clock_unit"] = bound_unit.clone();
    for member in case["argument"]["members"]
        .as_array_mut()
        .expect("aggregate members")
    {
        if member["value"]["value_kind"] == "quantity" {
            member["value"]["type"] = bound_unit.clone();
        }
    }
    let (mut package, _) = package_for(&case, &published);
    for node in package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
    {
        crate::support::checked_package::carry_check_mode(node);
    }
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .retain(|node| {
            !matches!(node["semantic_form"].as_str(), Some("dimension" | "unit"))
                || node["node_id"]["path"][0] != "scalar_type"
        });
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .retain(|row| {
            !matches!(row["node_id"]["path"][0].as_str(), Some("scalar_type"))
                || !matches!(
                    row["node_id"]["path"][1].as_str(),
                    Some("dimension" | "unit")
                )
        });
    if include_clause_owner_units {
        add_node(&mut package, source_dimension("Time"));
        add_node(&mut package, source_unit("ms", "Time"));
        add_node(&mut package, source_unit("s", "Time"));
    } else {
        add_other_source_units(&mut package);
    }
    package
}

fn expression(target: &Value, family: &str, value_type: &Value) -> (Value, Value) {
    let name = target["path"][1].as_str().expect("reference name");
    let value_id = id(&["value", "aggregate-input", name]);
    let (kind, value) = match family {
        "boolean" => ("boolean", json!(true)),
        "integer" => ("integer", json!("2")),
        "quantity" => ("quantity", json!("2")),
        _ => panic!("published reference family {family}"),
    };
    let value_node = json!({
        "node_id": value_id, "schema_version": "filament.checked-semantic-graph/v2",
        "node_tag": "value", "semantic_form": "literal", "semantic_type": value_type,
        "dependencies": [], "occurrences": [{"role": "expression", "ordinal": 0}],
        "body": {"term": "literal", "type": value_type, "value_kind": kind, "value": value},
    });
    let expression_node = json!({
        "node_id": target, "schema_version": "filament.checked-semantic-graph/v2",
        "node_tag": "expression", "semantic_form": "reference", "semantic_type": value_type,
        "dependencies": [value_id], "occurrences": [{"role": "expression", "ordinal": 0}],
        "body": {"term": "reference", "target": value_id},
    });
    (value_node, expression_node)
}

fn referenced_nodes(term: &Value, targets: &mut Vec<Value>) {
    match term {
        Value::Object(members) if members.get("term") == Some(&json!("reference")) => {
            targets.push(members["target"].clone());
        }
        Value::Object(members) => {
            for value in members.values() {
                referenced_nodes(value, targets);
            }
        }
        Value::Array(members) => {
            for value in members {
                referenced_nodes(value, targets);
            }
        }
        _ => {}
    }
}

fn replace_profile(value: &mut Value, selected: &str) {
    match value {
        Value::String(text) if text == "filament.temporal.event-position.false-extension/v1" => {
            *text = selected.to_owned();
        }
        Value::Array(items) => {
            for item in items {
                replace_profile(item, selected);
            }
        }
        Value::Object(members) => {
            for member in members.values_mut() {
                replace_profile(member, selected);
            }
        }
        _ => {}
    }
}

fn package_for(row: &Value, published: &Value) -> (Value, usize) {
    let mut package = read_json(&fixtures_dir().join("positive-all-families.json"))
        .expect("published positive package");
    let profile = row["profile"].as_str().expect("published profile");
    replace_profile(&mut package, profile);
    match profile {
        "filament.temporal.event-position.false-extension/v1" => {}
        "filament.temporal.timestamped-event.finite-window/v1" => {
            package["temporal_bindings"][0]["clock"] =
                json!({"kind": "timestamped_event", "timestamp_unit": "ms"});
        }
        "filament.temporal.infinite-trace/v1" => {
            package
                .as_object_mut()
                .expect("package")
                .remove("temporal_bindings");
        }
        other => panic!("published profile {other}"),
    }
    add_node(&mut package, dimension("Time"));
    add_node(&mut package, dimension("Length"));
    for (name, dimension) in [("ms", "Time"), ("s", "Time"), ("m", "Length")] {
        add_node(&mut package, unit(name, dimension));
    }
    for reference in published["references"].as_array().expect("references") {
        let family = reference["family"].as_str().expect("family");
        let value_type = match family {
            "boolean" => package["semantic_graph"]["nodes"][0]["node_id"].clone(),
            "integer" => id(&["scalar_type", "integer"]),
            "quantity" => published["clock_unit"].clone(),
            other => panic!("published family {other}"),
        };
        let (value, expression) = expression(&reference["target"], family, &value_type);
        add_node(&mut package, value);
        add_node(&mut package, expression);
    }
    let formula_index = 28;
    let formula = &mut package["semantic_graph"]["nodes"][formula_index];
    formula["body"]["operation"]["identity"] = json!("filament.op.temporal.aggregate");
    formula["body"]["arguments"] = json!([row["argument"]]);
    let mut targets = Vec::new();
    referenced_nodes(&row["argument"], &mut targets);
    targets.sort_by_key(fixture_support::structural_fixture_id);
    targets.dedup();
    formula["dependencies"] = json!(targets);
    (package, formula_index)
}

/// Trace: FR-038-AC-461, FR-038-AC-462; FSpec FR-370-AC-17/18, TC-303.
#[trace("TC-048", "FR-038-AC-461", "FR-038-AC-462")]
#[test]
fn published_temporal_aggregate_arguments_match_strict_reader() {
    let published = vectors();
    let positives = published["positive"].as_array().expect("positive cases");
    let readers = published["reader"].as_array().expect("reader cases");
    assert!(!positives.is_empty(), "published positives");
    assert!(!readers.is_empty(), "published reader mutations");
    let mut ids = BTreeSet::new();
    let mut mismatches = Vec::new();
    let mut matched = 0usize;
    for case in positives.iter().chain(readers) {
        let case_id = case["id"].as_str().expect("case id");
        assert!(ids.insert(case_id), "duplicate {case_id}");
        let (package, position) = package_for(case, &published);
        let result = read_package(&package).expect("complete package read");
        if case.get("expected").is_none() {
            let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
                mismatches.push(format!("{case_id}: expected admission, got refusal"));
                continue;
            };
            let CheckedTerm::Application {
                operation,
                arguments,
                ..
            } = &admitted.graph().nodes[position].body
            else {
                panic!("{case_id}: temporal formula body");
            };
            assert_eq!(
                operation.identity(),
                Some("filament.op.temporal.aggregate"),
                "{case_id}"
            );
            assert_eq!(arguments.len(), 1, "{case_id}: one aggregate argument");
            assert_eq!(
                serde_json::to_value(&arguments[0]).expect("typed argument"),
                case["argument"],
                "{case_id}: published aggregate bindings retained"
            );
            let expected_id: CheckedNodeId = serde_json::from_value(
                package["semantic_graph"]["nodes"][position]["node_id"].clone(),
            )
            .expect("published formula identity");
            assert_eq!(
                admitted
                    .document_held_id(position)
                    .expect("graph node")
                    .clone(),
                expected_id,
                "{case_id}: formula identity retained"
            );
            matched += 1;
        } else {
            let CheckedPackageV2ReadResult::Refused(refusal) = result else {
                mismatches.push(format!(
                    "{case_id}: expected refusal, got admission/incomplete"
                ));
                continue;
            };
            let suffix = case["expected"]["pointer"].as_str().expect("pointer");
            let actual = (
                code_word(refusal.code),
                refusal.cause.map(cause_word),
                refusal.loci[0].path.as_ref().map(ToString::to_string),
            );
            let expected = (
                case["expected"]["code"].as_str().expect("code"),
                case["expected"]["cause"].as_str(),
                Some(format!("/semantic_graph/nodes/{position}/body{suffix}")),
            );
            if actual != expected {
                mismatches.push(format!("{case_id}: {actual:?} != {expected:?}"));
                continue;
            }
            let expected_locus: CheckedNodeId = serde_json::from_value(
                package["semantic_graph"]["nodes"][position]["node_id"].clone(),
            )
            .expect("formula node id");
            assert_eq!(
                refusal.locus.as_ref(),
                Some(&expected_locus),
                "{case_id}: locus"
            );
            matched += 1;
        }
    }
    let total = positives.len() + readers.len();
    assert!(
        mismatches.is_empty(),
        "{matched}/{total} matched: {mismatches:#?}"
    );
}

/// The temporal aggregate stage precedes ordinary operation conformance.
#[trace("TC-048", "FR-038-AC-462")]
#[test]
fn aggregate_value_refusal_precedes_operation_result_refusal() {
    let published = vectors();
    let case = published["reader"]
        .as_array()
        .expect("reader cases")
        .iter()
        .find(|case| case["id"] == "AR-12-kind-median")
        .expect("published invalid kind");
    let (mut package, position) = package_for(case, &published);
    package["semantic_graph"]["nodes"][position]["body"]["result_type"] =
        id(&["scalar_type", "integer"]);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&package).expect("complete package")
    else {
        panic!("aggregate value defect must refuse");
    };
    assert_eq!(code_word(refusal.code), case["expected"]["code"]);
    assert_eq!(
        refusal.cause.map(cause_word),
        case["expected"]["cause"].as_str()
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!(
            "/semantic_graph/nodes/{position}/body{}",
            case["expected"]["pointer"].as_str().unwrap()
        ))
    );
}

/// Existing interval bounds precede a reached aggregate's argument checks.
#[trace("TC-048", "FR-038-AC-462")]
#[test]
fn interval_bounds_refusal_precedes_reached_aggregate_refusal() {
    let published = vectors();
    let case = published["reader"]
        .as_array()
        .expect("reader cases")
        .iter()
        .find(|case| case["id"] == "AR-12-kind-median")
        .expect("published invalid kind");
    let (mut package, _) = package_for(case, &published);
    package["semantic_graph"]["nodes"][29]["body"]["operation"]["member"]["interval"]["lower"] =
        json!("4");
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&package).expect("complete package")
    else {
        panic!("interval defect must refuse");
    };
    assert_eq!(code_word(refusal.code), "invalid_package");
    assert_eq!(refusal.cause.map(cause_word), Some("invalid-value"));
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some("/semantic_graph/nodes/29/body".to_owned())
    );
}

/// One structural clause can represent claims with distinct clock-unit bindings.
#[trace("TC-048", "FR-038-AC-462")]
#[test]
fn duration_bounds_match_every_occurrence_clock_unit() {
    let published = vectors();
    let case = published["positive"]
        .as_array()
        .expect("positive cases")
        .iter()
        .find(|case| case["id"] == "AG-07-duration-quantile")
        .expect("published duration case");
    let (mut package, formula) = package_for(case, &published);
    let clause = 8;
    let clause_id = package["semantic_graph"]["nodes"][clause]["node_id"].clone();
    package["semantic_graph"]["nodes"][clause]["occurrences"]
        .as_array_mut()
        .expect("clause occurrences")
        .push(json!({"role": "claim", "ordinal": 1}));
    let source_rows = package["source_map"].as_array_mut().expect("source map");
    let source_at = source_rows
        .iter()
        .position(|row| row["node_id"] == clause_id)
        .expect("clause source association");
    let mut second_source = source_rows[source_at].clone();
    second_source["ordinal"] = json!(1);
    second_source["regions"][0]["start"] = json!(11);
    second_source["regions"][0]["end"] = json!(12);
    let second_region = second_source["regions"][0].clone();
    source_rows.insert(source_at + 1, second_source);
    package["temporal_clause_names"]
        .as_array_mut()
        .expect("clause names")
        .insert(
            1,
            json!({"declaration": clause_id, "occurrence": 1, "name": "Clause1",
                "declaration_region": second_region}),
        );
    let bindings = package["temporal_bindings"]
        .as_array_mut()
        .expect("bindings");
    let mut second_binding = bindings[0].clone();
    second_binding["occurrence"] = json!(1);
    bindings.insert(1, second_binding);
    let CheckedPackageV2ReadResult::Admitted(admitted) =
        read_package(&package).expect("complete package")
    else {
        panic!("both claim occurrences using ms must admit");
    };
    assert_eq!(
        serde_json::to_value(&admitted.graph().nodes[formula].body).expect("typed formula"),
        package["semantic_graph"]["nodes"][formula]["body"]
    );

    package["temporal_bindings"][1]["clock"]["timestamp_unit"] = json!("s");
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&package).expect("complete package")
    else {
        panic!("second claim's s clock must refuse an ms bound");
    };
    assert_eq!(code_word(refusal.code), "ill_typed");
    assert_eq!(refusal.cause.map(cause_word), Some("unit-mismatch"));
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!(
            "/semantic_graph/nodes/{formula}/body/arguments/0/members/5/value"
        ))
    );
    let expected_id: CheckedNodeId =
        serde_json::from_value(package["semantic_graph"]["nodes"][formula]["node_id"].clone())
            .expect("formula identity");
    assert_eq!(refusal.locus.as_ref(), Some(&expected_id));
}

/// Trace: FR-778-AC-1/3, TC-1890.
/// Provenance: QSL-2338.
#[trace("TC-1890", "QSL-2338")]
#[test]
fn timestamped_duration_bounds_admit_only_linear_source_time_units() {
    let published = vectors();
    let case = published["positive"]
        .as_array()
        .expect("positive cases")
        .iter()
        .find(|case| case["id"] == "AG-07-duration-quantile")
        .expect("published duration case")
        .clone();
    let mut package = source_duration_package(case.clone(), published.clone(), id(&["ms"]), true);

    let admitted = read_package(&package).expect("complete package read");
    assert!(
        matches!(&admitted, CheckedPackageV2ReadResult::Admitted(_)),
        "a duration bound typed by the source-owned Time unit is admitted: {admitted:#?}"
    );

    let only_other_owner =
        source_duration_package(case.clone(), published.clone(), other_id(&["ms"]), false);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&only_other_owner).expect("cross-owner package read")
    else {
        panic!("a clause must not borrow another SourceOwner's same-name unit")
    };
    assert_eq!(
        code_word(refusal.code),
        "missing_declaration",
        "{refusal:#?}"
    );
    assert_eq!(refusal.cause.map(cause_word), Some("missing-name"));
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some("/temporal_bindings/0/clock/timestamp_unit".to_owned())
    );

    let mut nested_same_owner = source_duration_package(case, published, id(&["ms"]), true);
    let original_id = id(&["ms"]);
    let nested_id = id(&["other", "ms"]);
    replace_value(&mut nested_same_owner, &original_id, &nested_id);
    let nested_unit = nested_same_owner["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("semantic nodes")
        .iter_mut()
        .find(|node| node["node_id"] == nested_id)
        .expect("nested source unit");
    nested_unit["declaration"]["qualified_name"] = json!(["other", "ms"]);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&nested_same_owner).expect("nested source-unit package read")
    else {
        panic!("a nested same-owner source unit path must be refused")
    };
    assert_eq!(
        code_word(refusal.code),
        "missing_declaration",
        "{refusal:#?}"
    );
    assert_eq!(refusal.cause.map(cause_word), Some("missing-name"));
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some("/temporal_bindings/0/clock/timestamp_unit".to_owned())
    );

    let unit = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .iter_mut()
        .find(|node| node["node_id"]["path"] == json!(["ms"]))
        .expect("source unit");
    unit["body"]["offset"]["numerator"] = json!("1");
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&package).expect("complete package read")
    else {
        panic!("an affine source clock unit must refuse");
    };
    assert_eq!(code_word(refusal.code), "ill_typed", "{refusal:#?}");
    assert_eq!(refusal.cause.map(cause_word), Some("unit-mismatch"));
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some("/temporal_bindings/0/clock/timestamp_unit".to_owned())
    );
}

/// Operand order does not choose the first aggregate refusal.
#[trace("TC-048", "FR-038-AC-462")]
#[test]
fn reachable_aggregate_refusals_follow_formula_identity_order() {
    let published = vectors();
    let readers = published["reader"].as_array().expect("reader cases");
    let first = readers
        .iter()
        .find(|case| case["id"] == "AR-12-kind-median")
        .expect("first mutation");
    let second = readers
        .iter()
        .find(|case| case["id"] == "AR-13-kind-not-text")
        .expect("second mutation");
    let (mut package, first_position) = package_for(first, &published);
    let mut second_node = package["semantic_graph"]["nodes"][first_position].clone();
    let first_id = package["semantic_graph"]["nodes"][first_position]["node_id"].clone();
    let mut second_id = first_id.clone();
    second_id["path"]
        .as_array_mut()
        .expect("formula path")
        .push(json!("zz-aggregate"));
    second_node["node_id"] = second_id.clone();
    second_node["body"]["arguments"] = json!([second["argument"]]);
    add_node(&mut package, second_node);
    let second_position = package["semantic_graph"]["nodes"].as_array().unwrap().len() - 1;
    let parent = &mut package["semantic_graph"]["nodes"][29];
    parent["body"]["operation"]["identity"] = json!("filament.op.temporal.and");
    parent["body"]["operation"]["member"] = Value::Null;
    parent["body"]["arguments"] = json!([
        {"term": "reference", "target": second_id},
        {"term": "reference", "target": first_id},
    ]);
    let mut dependencies = vec![first_id, second_id];
    dependencies.sort_by_key(|value| {
        serde_json::from_value::<CheckedNodeId>(value.clone()).expect("formula identity")
    });
    parent["dependencies"] = json!(dependencies);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&package).expect("complete package")
    else {
        panic!("two defective aggregates must refuse");
    };
    assert_eq!(code_word(refusal.code), first["expected"]["code"]);
    assert_eq!(
        refusal.cause.map(cause_word),
        first["expected"]["cause"].as_str()
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!(
            "/semantic_graph/nodes/{first_position}/body{}",
            first["expected"]["pointer"].as_str().unwrap()
        ))
    );
    assert_ne!(first_position, second_position);
}

const OWNER: &str = "acme/orders";

/// The identity `ix://acme/orders/<kind>/<name>` of a time dimension or unit
/// the fixture declares in `example`, when `value` is its node identity.
fn owned_identity(value: &Value) -> Option<(String, String)> {
    let members = value.as_object()?;
    if members.get("package") != Some(&json!("example")) {
        return None;
    }
    let path = members.get("path")?.as_array()?;
    let [tag, kind, name] = path.as_slice() else {
        return None;
    };
    let (kind, name) = (kind.as_str()?, name.as_str()?);
    (tag == "scalar_type"
        && ((kind == "unit" && ["ms", "s"].contains(&name))
            || (kind == "dimension" && name == "Time")))
        .then(|| (kind.to_owned(), format!("ix://{OWNER}/{kind}/{name}")))
}

fn rename_time(value: &mut Value) {
    if let Some((kind, identity)) = owned_identity(value) {
        *value = json!({"package": OWNER, "path": ["scalar_type", kind, identity], "ordinal": 0});
        return;
    }
    match value {
        Value::Object(members) => members.values_mut().for_each(rename_time),
        Value::Array(items) => items.iter_mut().for_each(rename_time),
        _ => {}
    }
}

/// Moves the time dimension and units out of the clause's package into the
/// selected model package `OWNER`, as model-owned nodes named by identity.
fn move_time_to_owner(package: &mut Value) {
    rename_time(package);
    let mut owned = Vec::new();
    for node in package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes")
    {
        if node["node_id"]["package"] == OWNER && node["node_id"]["path"][0] == "scalar_type" {
            let identity = node["node_id"]["path"][2].clone();
            node.as_object_mut().expect("node").remove("declaration");
            node["owner"] = json!({"kind": "model", "identity": OWNER, "node": identity});
            node["body"] = json!({"term": "aggregate", "members": []});
            node["occurrences"] = json!([{"role": "type", "ordinal": 0}]);
            owned.push(node["node_id"].clone());
        }
    }
    for row in package["source_map"].as_array_mut().expect("source map") {
        if owned.contains(&row["node_id"]) {
            row["role"] = json!("type");
        }
    }
}

/// The selected domain document extended with the model's own time units.
fn document_with_time_units() -> std::path::PathBuf {
    let mut document =
        read_json(&proposal_dir_of(&fixtures_dir()).join("domain-package-acme-orders.json"))
            .expect("published selected document");
    document["dimensions"] = json!([{
        "identity": format!("ix://{OWNER}/dimension/Time"), "name": "Time", "base": true, "terms": [],
    }]);
    document["units"] = json!([
        {"identity": format!("ix://{OWNER}/unit/ms"), "name": "ms",
         "dimension": format!("ix://{OWNER}/dimension/Time"), "scale": "1", "offset": "0"},
        {"identity": format!("ix://{OWNER}/unit/s"), "name": "s",
         "dimension": format!("ix://{OWNER}/dimension/Time"),
         "target": format!("ix://{OWNER}/unit/ms"), "scale": "1000", "offset": "0"},
    ]);
    let directory = std::env::temp_dir().join(format!(
        "fsl-1662-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&directory).expect("temp dir");
    let path = directory.join("domain-with-time-units.json");
    std::fs::write(&path, serde_json::to_vec(&document).expect("document"))
        .expect("write document");
    path
}

/// The clock unit node is the unit's own node in its owning package (FSpec
/// FR-370-AC-20), not in the package of the clause that uses it.
#[trace("TC-048", "FR-038-AC-518")]
#[test]
fn duration_bounds_are_typed_by_the_clock_unit_in_its_owning_package() {
    let published = vectors();
    let case = published["positive"]
        .as_array()
        .expect("positive cases")
        .iter()
        .find(|case| case["id"] == "AG-07-duration-quantile")
        .expect("published duration case");
    let (mut package, formula) = package_for(case, &published);
    move_time_to_owner(&mut package);
    let identity = format!("ix://{OWNER}/unit/ms");
    package["temporal_bindings"][0]["clock"]["timestamp_unit"] = json!(identity);
    let document = document_with_time_units();
    match read_package_with_document(&package, "owned clock unit", &document).expect("package") {
        CheckedPackageV2ReadResult::Admitted(_) => {}
        CheckedPackageV2ReadResult::Refused(refusal) => panic!(
            "bounds typed by the owning package's unit node admit: {} {:?} {:?}",
            code_word(refusal.code),
            refusal.cause.map(cause_word),
            refusal.loci[0].path.as_ref().map(ToString::to_string)
        ),
        CheckedPackageV2ReadResult::Incomplete(_) => panic!("complete package"),
        CheckedPackageV2ReadResult::CatalogUnavailable(error) => {
            panic!("production operation catalog unavailable: {error:?}")
        }
    }

    // FSpec FR-370-AC-20: `upper` typed by the unit `s` node under a clock in
    // `ms` refuses at `upper`'s value.
    let s_unit = json!({
        "package": OWNER, "ordinal": 0,
        "path": ["scalar_type", "unit", format!("ix://{OWNER}/unit/s")],
    });
    let members = package["semantic_graph"]["nodes"][formula]["body"]["arguments"][0]["members"]
        .as_array_mut()
        .expect("aggregate members");
    let upper = members
        .iter()
        .position(|member| member["name"] == "upper")
        .expect("upper binding");
    members[upper]["value"]["type"] = s_unit;
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package_with_document(&package, "upper in s", &document).expect("package")
    else {
        panic!("an upper bound typed by the s node must refuse");
    };
    assert_eq!(code_word(refusal.code), "ill_typed");
    assert_eq!(refusal.cause.map(cause_word), Some("unit-mismatch"));
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!(
            "/semantic_graph/nodes/{formula}/body/arguments/0/members/{upper}/value"
        ))
    );
}
