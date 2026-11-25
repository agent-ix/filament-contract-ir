// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Strict-reader and lazy-cursor probes for FSpec FR-322's function body walk.

use crate::support::checked_package::{
    application_key_in_group, application_node_key, canonical, evidence_for, family_key, node_id,
    reassign_node_id, rebuild_body_locations, refresh_identity, settle, v2_all_families,
};
use filament_contract_model::{
    CheckedBodyLocationOrigin, CheckedNodeId, CheckedPackageReadLimits, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
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
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("body-walk package must admit: {other:?}"),
    }
}

fn reference(key: &str) -> Value {
    json!({"term":"reference","target":node_id(key)})
}

fn expression_application(
    package: &mut Value,
    form: &str,
    operator: &str,
    identity: &str,
    result: &str,
    arguments: Vec<Value>,
) -> Value {
    let body = json!({
        "term":"application", "operator":operator,
        "operation":{"identity":identity,"laws":[],"mode":null,"member":null,"leaves":[]},
        "result_type":node_id(result), "arguments":arguments,
    });
    let key = application_node_key("expression", form, result, &body);
    let node = json!({
        "node_id":node_id(&key), "schema_version":"filament.checked-semantic-graph/v2",
        "node_tag":"expression", "semantic_form":form,
        "semantic_type":node_id(result), "dependencies":[],
        "occurrences":[{"role":"expression","ordinal":0}], "body":body,
    });
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(node.clone());
    node["node_id"].clone()
}

fn function(
    package: &mut Value,
    name: &str,
    result: &str,
    body: Value,
    parameters: Vec<Value>,
    form: &str,
    recursion_group: Option<&str>,
) -> Value {
    let qualified_name = ["Example", name];
    let key = format!("qualified:Example/{name}");
    let mut node = json!({
        "node_id":node_id(&key), "schema_version":"filament.checked-semantic-graph/v2",
        "node_tag":"function", "semantic_form":form, "check_mode":"linked",
        "semantic_type":node_id(result), "dependencies":[],
        "occurrences":[{"role":"declaration","ordinal":0}],
        "declaration":{"qualified_name":qualified_name},
        "owner":{"kind":"source","authority":"agent-ix","identity":"filament.fixture.source/v1"},
        "body":{"term":"aggregate","members":[
            {"term":"binding","name":"parameters","value":{"term":"aggregate","members":parameters}},
            {"term":"binding","name":"body","value":body},
        ]},
    });
    if let Some(group) = recursion_group {
        node["recursion_group"] = json!(group);
    }
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(node.clone());
    node["node_id"].clone()
}

fn package_with_rows(mut package: Value) -> Value {
    settle(&mut package);
    rebuild_body_locations(&mut package);
    refresh_identity(&mut package);
    package
}

fn assign_recursion_group(package: &mut Value, name: &str, members: &[Value]) {
    for node in package["semantic_graph"]["nodes"].as_array_mut().unwrap() {
        if members.contains(&node["node_id"]) {
            node["recursion_group"] = json!(name);
        }
    }
    let nodes = package["semantic_graph"]["nodes"].as_array().unwrap();
    let group: Vec<Value> = nodes
        .iter()
        .filter(|node| node["recursion_group"] == name)
        .map(|node| node["node_id"].clone())
        .collect();
    let replacements: Vec<(Value, Value)> = nodes
        .iter()
        .filter(|node| node["recursion_group"] == name && node["body"]["term"] == "application")
        .map(|node| {
            let key = application_key_in_group(node, &group);
            (node["node_id"].clone(), node_id(&key))
        })
        .collect();
    for (old, fresh) in replacements {
        reassign_node_id(package, &old, &fresh);
    }
}

fn function_id(package: &Value, name: &str) -> CheckedNodeId {
    let node = package["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| {
            node["node_tag"] == "function" && node["declaration"]["qualified_name"][1] == name
        })
        .unwrap();
    serde_json::from_value(node["node_id"].clone()).unwrap()
}

fn cursor_rows(package: &CheckedPackageV2, function: &CheckedNodeId) -> Vec<(CheckedNodeId, u64)> {
    let mut cursor = package.body_locations(function, 100_000).unwrap();
    let mut result = Vec::new();
    while let Some(location) = cursor.next_location().unwrap() {
        let CheckedBodyLocationOrigin::FunctionBody {
            function: owner, ..
        } = &location.row.location.origin;
        assert_eq!(owner, function);
        result.push((location.row.application.clone(), location.row.term_ordinal));
    }
    result
}

fn sidecar_rows(package: &Value, function: &CheckedNodeId) -> Vec<(CheckedNodeId, u64)> {
    package["checked_term_data"]["body_locations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["location"]["origin"]["function"] == json!(function))
        .map(|row| {
            (
                serde_json::from_value(row["application"].clone()).unwrap(),
                row["term_ordinal"].as_u64().unwrap(),
            )
        })
        .collect()
}

fn call(package: &mut Value, target: &Value, result: &str) -> Value {
    expression_application(
        package,
        "call",
        "call",
        "filament.op.function.call",
        result,
        vec![json!({"term":"reference","target":target})],
    )
}

#[trace("TC-491", "FR-038-AC-581", "FR-322-AC-490")]
#[test]
fn caller_before_guarded_callee_keeps_each_walk_separate() {
    let boolean = family_key("aaaa");
    let mut package = v2_all_families();
    let guarded_term = expression_application(
        &mut package,
        "binary",
        "binary",
        "filament.op.boolean.and",
        &boolean,
        vec![
            json!({"term":"literal","type":node_id(&boolean),"value_kind":"boolean","value":true}),
            json!({"term":"literal","type":node_id(&boolean),"value_kind":"boolean","value":false}),
        ],
    );
    let guarded_body = expression_application(
        &mut package,
        "conditional",
        "conditional",
        "filament.op.control.if",
        &boolean,
        vec![
            json!({"term":"literal","type":node_id(&boolean),"value_kind":"boolean","value":true}),
            json!({"term":"reference","target":guarded_term}),
            json!({"term":"literal","type":node_id(&boolean),"value_kind":"boolean","value":false}),
        ],
    );
    let guarded = function(
        &mut package,
        "guarded",
        &boolean,
        json!({"term":"reference","target":guarded_body}),
        vec![],
        "pure_function",
        None,
    );
    let caller_call = call(&mut package, &guarded, &boolean);
    function(
        &mut package,
        "caller",
        &boolean,
        json!({"term":"reference","target":caller_call}),
        vec![],
        "pure_function",
        None,
    );
    let package = package_with_rows(package);
    let admitted = admitted(&package);
    let caller = function_id(&package, "caller");
    let guarded = function_id(&package, "guarded");
    let caller_call_id: CheckedNodeId = serde_json::from_value(caller_call.clone()).unwrap();
    assert_eq!(
        cursor_rows(&admitted, &caller),
        vec![(caller_call_id.clone(), 0)]
    );
    let guarded_rows = cursor_rows(&admitted, &guarded);
    assert_eq!(guarded_rows.len(), 1);
    let guarded_term_id: CheckedNodeId = serde_json::from_value(guarded_term).unwrap();
    assert!(guarded_rows
        .iter()
        .all(|(application, _)| application == &guarded_term_id));
}

#[trace("TC-491", "FR-038-AC-581", "FR-322-AC-490")]
#[test]
fn shared_k_call_plus_one_is_walked_once_per_function() {
    let integer = family_key("a3a3");
    let mut package = v2_all_families();
    let k = function(
        &mut package,
        "k",
        &integer,
        json!({"term":"literal","type":node_id(&integer),"value_kind":"integer","value":"2"}),
        vec![],
        "pure_function",
        None,
    );
    let k_call = call(&mut package, &k, &integer);
    let shared_add = expression_application(
        &mut package,
        "binary",
        "binary",
        "filament.op.integer.add",
        &integer,
        vec![
            json!({"term":"reference","target":k_call}),
            json!({"term":"literal","type":node_id(&integer),"value_kind":"integer","value":"1"}),
        ],
    );
    function(
        &mut package,
        "first",
        &integer,
        json!({"term":"reference","target":shared_add}),
        vec![],
        "pure_function",
        None,
    );
    function(
        &mut package,
        "second",
        &integer,
        json!({"term":"reference","target":shared_add}),
        vec![],
        "pure_function",
        None,
    );
    let package = package_with_rows(package);
    let admitted = admitted(&package);
    let first_node = package["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| {
            node["node_tag"] == "function" && node["declaration"]["qualified_name"][1] == "first"
        })
        .unwrap();
    let shared_add_value = first_node["body"]["members"]
        .as_array()
        .unwrap()
        .iter()
        .find(|member| member["name"] == "body")
        .unwrap()["value"]["target"]
        .clone();
    let shared_add_id: CheckedNodeId = serde_json::from_value(shared_add_value.clone()).unwrap();
    let shared_add_node = package["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["node_id"] == shared_add_value)
        .unwrap();
    let k_call_id: CheckedNodeId =
        serde_json::from_value(shared_add_node["body"]["arguments"][0]["target"].clone()).unwrap();
    let expected = vec![(shared_add_id, 0), (k_call_id, 0)];
    for name in ["first", "second"] {
        let function = function_id(&package, name);
        let rows = cursor_rows(&admitted, &function);
        assert_eq!(rows, expected);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows.iter().map(|(_, ordinal)| *ordinal).collect::<Vec<_>>(),
            [0, 0]
        );
        if name == "first" {
            let second = cursor_rows(&admitted, &function_id(&package, "second"));
            assert_eq!(rows, second);
        }
    }
    assert!(cursor_rows(&admitted, &function_id(&package, "k")).is_empty());
}

#[trace("TC-491", "FR-038-AC-581", "FR-322-AC-490")]
#[test]
fn mutual_even_odd_calls_do_not_cross_function_boundaries() {
    let boolean = family_key("aaaa");
    let mut package = v2_all_families();
    let even_id = node_id("qualified:Example/even");
    let odd_id = node_id("qualified:Example/odd");
    let even_call = call(&mut package, &odd_id, &boolean);
    let odd_call = call(&mut package, &even_id, &boolean);
    let even = function(
        &mut package,
        "even",
        &boolean,
        json!({"term":"reference","target":even_call}),
        vec![],
        "recursive_function",
        Some("parity"),
    );
    let odd = function(
        &mut package,
        "odd",
        &boolean,
        json!({"term":"reference","target":odd_call}),
        vec![],
        "recursive_function",
        Some("parity"),
    );
    assign_recursion_group(
        &mut package,
        "parity",
        &[even, even_call.clone(), odd, odd_call.clone()],
    );
    let package = package_with_rows(package);
    let admitted = admitted(&package);
    for name in ["even", "odd"] {
        let function = function_id(&package, name);
        let rows = cursor_rows(&admitted, &function);
        assert_eq!(rows, sidecar_rows(&package, &function));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].1, 0);
    }
}

#[trace("TC-491", "FR-038-AC-581", "FR-322-AC-490")]
#[test]
fn caller_before_unguarded_inc_does_not_enter_inc_body() {
    let integer = family_key("a3a3");
    let mut package = v2_all_families();
    let inc_parameter = node_id("value/parameter/inc-x");
    package["semantic_graph"]["nodes"].as_array_mut().unwrap().push(json!({
        "node_id":inc_parameter,"schema_version":"filament.checked-semantic-graph/v2",
        "node_tag":"value","semantic_form":"parameter","semantic_type":node_id(&integer),
        "dependencies":[],"occurrences":[{"role":"expression","ordinal":0}],
        "body":{"term":"aggregate","members":[
            {"term":"binding","name":"name","value":{"term":"literal","type":node_id(&family_key("a1a1")),"value_kind":"text","value":"x"}},
            {"term":"binding","name":"level","value":{"term":"literal","type":node_id(&family_key("a3a3")),"value_kind":"integer","value":"0"}}
        ]}
    }));
    let plus = expression_application(
        &mut package,
        "binary",
        "binary",
        "filament.op.integer.add",
        &integer,
        vec![
            json!({"term":"reference","target":inc_parameter.clone()}),
            json!({"term":"literal","type":node_id(&integer),"value_kind":"integer","value":"1"}),
        ],
    );
    let inc = function(
        &mut package,
        "inc",
        &integer,
        json!({"term":"reference","target":plus}),
        vec![reference("value/parameter/inc-x")],
        "pure_function",
        None,
    );
    let caller_call = expression_application(
        &mut package,
        "call",
        "call",
        "filament.op.function.call",
        &integer,
        vec![
            json!({"term":"reference","target":inc}),
            json!({"term":"literal","type":node_id(&integer),"value_kind":"integer","value":"4"}),
        ],
    );
    function(
        &mut package,
        "caller",
        &integer,
        json!({"term":"reference","target":caller_call}),
        vec![],
        "pure_function",
        None,
    );
    let package = package_with_rows(package);
    let admitted = admitted(&package);
    let caller_rows = cursor_rows(&admitted, &function_id(&package, "caller"));
    let inc_rows = cursor_rows(&admitted, &function_id(&package, "inc"));
    assert_eq!(
        caller_rows,
        sidecar_rows(&package, &function_id(&package, "caller"))
    );
    assert_eq!(
        inc_rows,
        sidecar_rows(&package, &function_id(&package, "inc"))
    );
    assert_eq!(caller_rows.len(), 1);
    assert_eq!(inc_rows.len(), 1);
    assert_eq!((caller_rows[0].1, inc_rows[0].1), (0, 0));
}

#[trace("TC-491", "FR-038-AC-581", "FR-322-AC-490")]
#[test]
fn a_function_node_at_the_body_root_is_rejected_before_body_walk() {
    let boolean = family_key("aaaa");
    let mut package = v2_all_families();
    let callee_term = expression_application(
        &mut package,
        "binary",
        "binary",
        "filament.op.boolean.and",
        &boolean,
        vec![
            json!({"term":"literal","type":node_id(&boolean),"value_kind":"boolean","value":true}),
            json!({"term":"literal","type":node_id(&boolean),"value_kind":"boolean","value":false}),
        ],
    );
    let callee = function(
        &mut package,
        "root_callee",
        &boolean,
        json!({"term":"reference","target":callee_term}),
        vec![],
        "pure_function",
        None,
    );
    function(
        &mut package,
        "root_caller",
        &boolean,
        json!({"term":"reference","target":callee}),
        vec![],
        "pure_function",
        None,
    );
    settle(&mut package);
    rebuild_body_locations(&mut package);
    let caller = package["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["declaration"]["qualified_name"][1] == "root_caller")
        .unwrap()["node_id"]
        .clone();
    package["checked_term_data"]["body_locations"]
        .as_array_mut()
        .unwrap()
        .retain(|row| row["location"]["origin"]["function"] != caller);
    refresh_identity(&mut package);

    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("a function node is not a valid expression result");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::IllTyped);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::TypeMismatch)
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().as_str(),
        "/semantic_graph/nodes/28"
    );
}
