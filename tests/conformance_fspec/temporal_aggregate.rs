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

fn other_source_dimension(owner: &str) -> Value {
    let key = other_id(&["Time"]);
    json!({
        "node_id": key, "schema_version":"filament.checked-semantic-graph/v2",
        "node_tag":"scalar_type", "semantic_form":"dimension", "semantic_type":key,
        "declaration":{"qualified_name":["Time"]}, "dependencies":[],
        "occurrences":[{"role":"declaration","ordinal":0}],
        "owner":{"kind":"source","authority":"agent-ix","identity":owner},
        "body":{"term":"dimension","base":true,"terms":[]}
    })
}

fn other_source_unit(name: &str, owner: &str) -> Value {
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
        "owner":{"kind":"source","authority":"agent-ix","identity":owner},
        "body":{"term":"unit","dimension_node_id":dimension,
            "target_unit_node_id":target,"scale":scale,
            "offset":{"numerator":"0","denominator":"1"}}
    })
}

fn add_other_source_units(package: &mut Value) {
    add_source_units_of(package, "other");
}

/// Declares `Time`, `s`, `ms` in the locked source unit `owner`.
fn add_source_units_of(package: &mut Value, owner: &str) {
    package["lock"]["sources"]
        .as_array_mut()
        .expect("source lock")
        .push(json!({"authority":"agent-ix","identity":owner}));
    for node in [
        other_source_dimension(owner),
        other_source_unit("ms", owner),
        other_source_unit("s", owner),
    ] {
        let id = node["node_id"].clone();
        package["source_map"]
            .as_array_mut()
            .expect("source map")
            .push(json!({
                "node_id":id,"role":"declaration","ordinal":0,
                "regions":[{"source":{"authority":"agent-ix","identity":owner},
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
    source_duration_package_in(
        case,
        published,
        bound_unit,
        if include_clause_owner_units {
            "example"
        } else {
            "other"
        },
    )
}

/// The AG-07 duration package whose `Time`, `s` and `ms` are declared by the
/// locked source unit `units_owner`.
fn source_duration_package_in(
    mut case: Value,
    mut published: Value,
    bound_unit: Value,
    units_owner: &str,
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
    if units_owner == "example" {
        add_node(&mut package, source_dimension("Time"));
        add_node(&mut package, source_unit("ms", "Time"));
        add_node(&mut package, source_unit("s", "Time"));
    } else {
        add_source_units_of(&mut package, units_owner);
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

const EXAMPLE_UNIT: fn() -> Value = || json!({"authority": "agent-ix", "identity": "example"});

fn ms_case() -> (Value, Value) {
    let published = vectors();
    let case = published["positive"]
        .as_array()
        .expect("positive cases")
        .iter()
        .find(|case| case["id"] == "AG-07-duration-quantile")
        .expect("published duration case")
        .clone();
    (case, published)
}

/// A duration clause package whose `Time`, `s`, `ms` are declared by `owner`,
/// its row naming `source_unit`.
fn su_package(owner: &str, source_unit: Value) -> Value {
    let (case, published) = ms_case();
    let mut package = source_duration_package_in(case, published, id(&["ms"]), owner);
    package["temporal_bindings"][0]["source_unit"] = source_unit;
    package
}

fn su_refusal(package: &Value) -> (String, Option<&'static str>, String) {
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(package).expect("complete package read")
    else {
        panic!("package must refuse");
    };
    (
        code_word(refusal.code).to_owned(),
        refusal.cause.map(cause_word),
        refusal.loci[0]
            .path
            .as_ref()
            .map(ToString::to_string)
            .expect("locus"),
    )
}

fn su_admitted(package: &Value) -> bool {
    matches!(
        read_package(package).expect("complete package read"),
        CheckedPackageV2ReadResult::Admitted(_)
    )
}

/// Retargets the clause subject's reference type to the source model
/// `Example.Account`, the source-declared subject of SU-02 (`Pulse`).
fn source_record_subject(package: &mut Value) {
    let record = id(&["Example", "Account"]);
    for node in package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
    {
        if node["node_id"]["path"][0] == "composite_type"
            && node["node_id"]["path"][1] == "reference"
        {
            node["dependencies"] = json!([record]);
            node["body"]["members"][0]["target"] = record.clone();
        }
    }
}

/// FR-778-AC-8..11: SU-01..SU-04 built as published: the subject and the unit
/// declaring it, each unit's `Time` declarations, the bound's unit and the
/// row. The one addition is that a unit declaring the units is locked, as a
/// source-owned declaration requires.
#[trace(
    "TC-048",
    "FR-038-AC-590",
    "FR-778-AC-8",
    "FR-778-AC-9",
    "FR-778-AC-10",
    "FR-778-AC-11",
    "FR-778-AC-12"
)]
#[test]
fn source_unit_vectors_resolve_the_clock_unit_in_the_named_unit_only() {
    let published =
        read_json(&proposal_dir_of(&fixtures_dir()).join("temporal-clock-binding-vectors.json"))
            .expect("published vectors");
    let cases = published["source_unit_cases"]
        .as_array()
        .expect("source unit cases");
    let ids: Vec<&str> = cases
        .iter()
        .map(|case| case["id"].as_str().expect("case id"))
        .collect();
    for expected in ["SU-01", "SU-02", "SU-03", "SU-04", "SU-05"] {
        assert!(ids.contains(&expected), "published case {expected}");
    }
    let mut decided = 0;
    for case in cases {
        let name = case["id"].as_str().expect("case id");
        // A case holds one `row`, or `rows` (one clause read at several claim
        // occurrences): both are decided from the published members.
        let rows: Vec<Value> = match (case.get("row"), case.get("rows")) {
            (Some(row), None) => vec![row.clone()],
            (None, Some(rows)) => rows.as_array().expect("rows").clone(),
            _ => panic!("{name}: one of row and rows"),
        };
        let owners: Vec<&str> = case["time_units"]
            .as_object()
            .expect("time units")
            .keys()
            .map(String::as_str)
            .collect();
        // The fixture declares the units once, in the unit its first row names
        // when that unit declares them, else in the one that does.
        let first = rows[0]["source_unit"]["identity"].as_str().expect("unit");
        let units_owner = if owners.contains(&first) {
            first
        } else {
            assert_eq!(owners.len(), 1, "{name}: one declaring unit");
            owners[0]
        };
        assert!(
            owners.contains(&case["bound_unit"]["owner"].as_str().expect("owner")),
            "{name}: the bound is typed by a declaring unit's node"
        );
        let (ms_case, published_vectors) = ms_case();
        let mut package =
            source_duration_package_in(ms_case, published_vectors, id(&["ms"]), units_owner);
        assert_eq!(case["bound_unit"]["path"], json!(["ms"]));
        // The locked sources are the published ones.
        for source in case["lock_sources"].as_array().expect("lock sources") {
            let locked = package["lock"]["sources"].as_array_mut().expect("lock");
            if !locked.contains(source) {
                locked.push(source.clone());
            }
        }
        package["lock"]["sources"]
            .as_array_mut()
            .expect("lock")
            .sort_by_key(ToString::to_string);
        // The rows' `clock` and `source_unit`, as published; `clock_name` is
        // the vector's notation (`ts`) and stays the clause's own role name.
        if rows.len() > 1 {
            let units: Vec<Value> = rows.iter().map(|row| row["source_unit"].clone()).collect();
            package = with_occurrences(&package, &units);
        }
        for (at, row) in rows.iter().enumerate() {
            for member in ["clock", "source_unit"] {
                package["temporal_bindings"][at][member] = row[member].clone();
            }
        }
        // The subject: a `Reference<Order>` declared by the selected model
        // `acme/orders` is the fixture's own; a source `Pulse` is retargeted.
        match case["subject"]["declared_by"]
            .as_str()
            .expect("subject owner")
        {
            "acme/orders" => {}
            "example" => source_record_subject(&mut package),
            other => panic!("{name}: subject unit {other}"),
        }
        decided += 1;
        if case["expected"] == "admitted" {
            assert!(su_admitted(&package), "{name}");
        } else {
            assert_eq!(
                su_refusal(&package),
                (
                    case["code"].as_str().expect("code").to_owned(),
                    Some(match case["cause"].as_str().expect("cause") {
                        "missing-name" => "missing-name",
                        "invalid-value" => "invalid-value",
                        "timed-node-reference" => "timed-node-reference",
                        other => panic!("{other}"),
                    }),
                    case["locus"].as_str().expect("locus").to_owned()
                ),
                "{name}"
            );
        }
    }
    assert_eq!(decided, cases.len(), "every published case is decided");
}

/// The units are declared in `example`, the row names a second locked unit
/// that declares nothing: the name resolves in that unit only, so it is
/// refused even though another unit of the package declares it.
#[trace("TC-1890", "FR-778-AC-10")]
#[test]
fn a_named_unit_that_declares_nothing_refuses_the_clock_unit() {
    let other = json!({"authority": "agent-ix", "identity": "other"});
    let mut package = su_package("example", EXAMPLE_UNIT());
    package["lock"]["sources"]
        .as_array_mut()
        .unwrap()
        .push(other.clone());
    package["temporal_bindings"][0]["source_unit"] = other;
    assert_eq!(
        su_refusal(&package),
        (
            "missing_declaration".to_owned(),
            Some("missing-name"),
            "/temporal_bindings/0/clock/timestamp_unit".to_owned()
        )
    );
}

/// FR-778-AC-9: a clause over a source record in the unit that declares the
/// clock units is admitted; the subject is never read.
#[trace("TC-048", "FR-778-AC-9")]
#[test]
fn clause_over_a_source_record_subject_is_admitted() {
    let mut package = su_package("example", EXAMPLE_UNIT());
    source_record_subject(&mut package);
    assert!(su_admitted(&package));
}

/// FR-322-AC-503 / FR-778-AC-11: a missing member, and an unlocked unit.
#[trace("TC-233", "TC-1890", "FR-038-AC-590", "FR-322-AC-503", "FR-778-AC-11")]
#[test]
fn clause_row_requires_a_locked_source_unit() {
    let mut package = su_package("example", EXAMPLE_UNIT());
    package["temporal_bindings"][0]
        .as_object_mut()
        .expect("row")
        .remove("source_unit");
    assert_eq!(
        su_refusal(&package),
        (
            "invalid_package".to_owned(),
            Some("timed-node-reference"),
            "/temporal_bindings/0".to_owned()
        )
    );
    let unlocked = su_package(
        "example",
        json!({"authority": "agent-ix", "identity": "other"}),
    );
    assert_eq!(
        su_refusal(&unlocked),
        (
            "invalid_package".to_owned(),
            Some("invalid-value"),
            "/temporal_bindings/0/source_unit".to_owned()
        )
    );
    // The same authority under another identity, and another authority under
    // the same identity, are both unlocked.
    for unit in [
        json!({"authority": "other", "identity": "example"}),
        json!({"authority": "agent-ix", "identity": "Example"}),
    ] {
        assert_eq!(
            su_refusal(&su_package("example", unit)).1,
            Some("invalid-value")
        );
    }
}

/// Types the aggregate's `upper` bound by `unit`; returns the pointer of the
/// bound's value.
fn type_upper(package: &mut Value, unit: Value) -> String {
    for (position, node) in package["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .enumerate()
    {
        if let Some(members) = node
            .pointer_mut("/body/arguments/0/members")
            .and_then(Value::as_array_mut)
        {
            for (at, member) in members.iter_mut().enumerate() {
                if member["name"] == "upper" {
                    member["value"]["type"] = unit;
                    return format!(
                        "/semantic_graph/nodes/{position}/body/arguments/0/members/{at}/value"
                    );
                }
            }
        }
    }
    panic!("no upper bound");
}

/// `package` with the clause read at one claim occurrence per entry of
/// `units`, each row naming that `source_unit`.
fn with_occurrences(package: &Value, units: &[Value]) -> Value {
    let mut package = package.clone();
    let clause = package["temporal_bindings"][0]["declaration"].clone();
    let clause_at = package["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .position(|node| node["node_id"] == clause)
        .expect("clause node");
    let source_at = package["source_map"]
        .as_array()
        .unwrap()
        .iter()
        .position(|row| row["node_id"] == clause)
        .expect("clause source row");
    let row = package["temporal_bindings"][0].clone();
    let names = package["temporal_clause_names"][0].clone();
    let source = package["source_map"][source_at].clone();
    package["temporal_bindings"] = json!([]);
    package["temporal_clause_names"] = json!([]);
    for (ordinal, unit) in units.iter().enumerate() {
        let mut row = row.clone();
        row["occurrence"] = json!(ordinal);
        row["source_unit"] = unit.clone();
        package["temporal_bindings"]
            .as_array_mut()
            .unwrap()
            .push(row);
        let mut named = names.clone();
        named["occurrence"] = json!(ordinal);
        named["name"] = json!(format!("Clause{ordinal}"));
        named["declaration_region"]["end"] = json!(10 + 3 * ordinal);
        package["temporal_clause_names"]
            .as_array_mut()
            .unwrap()
            .push(named);
        if ordinal > 0 {
            package["semantic_graph"]["nodes"][clause_at]["occurrences"]
                .as_array_mut()
                .unwrap()
                .push(json!({"role": "claim", "ordinal": ordinal}));
            let mut region = source.clone();
            region["ordinal"] = json!(ordinal);
            region["regions"][0]["start"] = json!(8 + 3 * ordinal);
            region["regions"][0]["end"] = json!(9 + 3 * ordinal);
            package["source_map"]
                .as_array_mut()
                .unwrap()
                .insert(source_at + ordinal, region);
        }
    }
    package
}

/// FR-322 "Clause source unit" (arch ruling): a clause is declared in one
/// unit, so every row of one clause names the unit its first row names; the
/// first row that differs refuses `timed-node-reference` at that row.
#[trace("TC-048", "FR-038-AC-590", "FR-778-AC-12")]
#[test]
fn every_row_of_one_clause_names_one_source_unit() {
    let example = EXAMPLE_UNIT();
    let other = json!({"authority": "agent-ix", "identity": "other"});
    let mut base = su_package("example", example.clone());
    // `other` is a second locked unit; the row rule needs no declarations.
    base["lock"]["sources"]
        .as_array_mut()
        .unwrap()
        .push(other.clone());
    let three_same = with_occurrences(&base, &[example.clone(), example.clone(), example.clone()]);
    assert!(su_admitted(&three_same), "three rows naming one unit");
    for (name, units, at) in [
        (
            "second differs",
            [example.clone(), other.clone(), example.clone()],
            1,
        ),
        (
            "third differs",
            [example.clone(), example.clone(), other.clone()],
            2,
        ),
    ] {
        assert_eq!(
            su_refusal(&with_occurrences(&base, &units)),
            (
                "invalid_package".to_owned(),
                Some("timed-node-reference"),
                format!("/temporal_bindings/{at}")
            ),
            "{name}"
        );
    }
}

/// FR-778-AC-1, AC-3..AC-7 over the named unit.
#[trace(
    "TC-1890",
    "FR-778-AC-1",
    "FR-778-AC-3",
    "FR-778-AC-4",
    "FR-778-AC-5",
    "FR-778-AC-6",
    "FR-778-AC-7"
)]
#[test]
fn source_clock_unit_refusals_and_admissions() {
    let base = su_package("example", EXAMPLE_UNIT());
    assert!(su_admitted(&base));
    assert!(
        base["semantic_graph"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .all(|node| node["node_id"]["path"] != json!(["scalar_type", "unit", "ms"])),
        "AC-1: no model-owned ms node"
    );
    let locus = "/temporal_bindings/0/clock/timestamp_unit";
    let unit_refusal = |package: &Value| {
        let CheckedPackageV2ReadResult::Refused(refusal) = read_package(package).expect("read")
        else {
            panic!("must refuse");
        };
        // AC-6: the clock name and T are retained at the row.
        let retained = refusal.locus.clone();
        (
            code_word(refusal.code),
            refusal.cause.map(cause_word),
            refusal.loci[0].path.as_ref().map(ToString::to_string),
            retained,
        )
    };
    let clause = serde_json::from_value::<CheckedNodeId>(
        base["temporal_bindings"][0]["declaration"].clone(),
    )
    .unwrap();
    // AC-3: `us` is declared by no unit.
    let mut us = base.clone();
    us["temporal_bindings"][0]["clock"]["timestamp_unit"] = json!("us");
    assert_eq!(
        unit_refusal(&us),
        (
            "missing_declaration",
            Some("missing-name"),
            Some(locus.to_owned()),
            Some(clause.clone())
        )
    );
    // AC-4: a unit of another dimension.
    let mut length = base.clone();
    add_node(&mut length, source_dimension("Length"));
    add_node(&mut length, source_unit("m", "Length"));
    length["temporal_bindings"][0]["clock"]["timestamp_unit"] = json!("m");
    assert_eq!(
        unit_refusal(&length),
        (
            "ill_typed",
            Some("unit-mismatch"),
            Some(locus.to_owned()),
            Some(clause.clone())
        )
    );
    // AC-5: an affine unit of Time.
    let mut affine = base.clone();
    affine["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|node| node["node_id"]["path"] == json!(["ms"]))
        .unwrap()["body"]["offset"]["numerator"] = json!("1");
    assert_eq!(
        unit_refusal(&affine),
        (
            "ill_typed",
            Some("unit-mismatch"),
            Some(locus.to_owned()),
            Some(clause.clone())
        )
    );
    // AC-6: the same packages with `upper` also typed by the source `s` node
    // report only the row refusal, before any aggregate check.
    for (name, package) in [("us", &us), ("m", &length), ("affine", &affine)] {
        let mut with_bound = package.clone();
        type_upper(&mut with_bound, id(&["s"]));
        assert_eq!(unit_refusal(&with_bound), unit_refusal(package), "{name}");
    }
    // AC-1: `upper` typed by the source `s` node refuses at `upper`'s value.
    let mut bound_s = base.clone();
    let upper = type_upper(&mut bound_s, id(&["s"]));
    let (code, cause, at, _) = unit_refusal(&bound_s);
    assert_eq!(
        (code, cause, at),
        ("ill_typed", Some("unit-mismatch"), Some(upper))
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
#[trace("TC-048", "TC-1890", "FR-038-AC-518", "FR-778-AC-2")]
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
        CheckedPackageV2ReadResult::HostAllocationFailed(failure) => {
            panic!("the host refused an allocation: {failure:?}")
        }
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
    // FR-778-AC-2, second half: with the unit of compilation also declaring a
    // source `ms`, a bound typed by that source node refuses at the bound.
    let mut also_source = package.clone();
    add_node(&mut also_source, source_dimension("Time"));
    add_node(&mut also_source, source_unit("ms", "Time"));
    add_node(&mut also_source, source_unit("s", "Time"));
    also_source["semantic_graph"]["nodes"][formula]["body"]["arguments"][0]["members"][upper]
        ["value"]["type"] = id(&["ms"]);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package_with_document(&also_source, "upper in source ms", &document).expect("package")
    else {
        panic!("a bound typed by the source ms must refuse beside the model-owned clock unit");
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
