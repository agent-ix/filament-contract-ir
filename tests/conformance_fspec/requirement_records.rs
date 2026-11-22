// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-488 public-reader coverage for the requirement-record package member.

use crate::checked_package_v2_parameters::parameters_package;
use crate::fixture_support::{
    canonical, evidence_for, index_fixture_node_ids, node_id, rebuild_body_locations,
    rebuild_complete_source_map, rebuild_source_map, settle, structural_fixture_id,
};
use filament_contract_model::{
    encode_checked_package_v2_with_requirement_items, CheckedPackageReadLimits,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedPackageSidecarsRef,
    CheckedPackageSourceCorrespondenceRef, CheckedPackageV2, CheckedPackageV2BudgetedReadResult,
    CheckedPackageV2ReadResult, JsonPointer, NativeJoinBudget,
};
use ix_trace_rs::trace;
use serde_json::{json, Value};

fn read(value: &Value) -> CheckedPackageV2ReadResult {
    let mut wire = value.clone();
    index_fixture_node_ids(&mut wire);
    CheckedPackageV2::read(
        &canonical(&wire),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&wire),
    )
}

fn function_claim(package: &Value) -> Value {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .find(|node| node["node_tag"] == "function")
        .map(|node| {
            json!({
                "node": node["node_id"],
                "occurrence": node["occurrences"][0],
            })
        })
        .expect("function node")
}

fn row(claim: &Value, kind: &str) -> Value {
    json!({
        "claim": claim,
        "kind": kind,
        "extent": {"classification": "bounded"},
    })
}

fn sync_body_dependencies(node: &mut Value) {
    fn references(value: &Value, found: &mut Vec<Value>) {
        if value["term"] == "reference" {
            found.push(value["target"].clone());
            return;
        }
        match value {
            Value::Array(items) => items.iter().for_each(|item| references(item, found)),
            Value::Object(members) => members
                .values()
                .for_each(|member| references(member, found)),
            _ => {}
        }
    }

    let mut dependencies = Vec::new();
    references(&node["body"], &mut dependencies);
    dependencies.sort_by_key(structural_fixture_id);
    dependencies.dedup();
    node["dependencies"] = json!(dependencies);
}

fn value_validity_package(reverse_graph: bool) -> Value {
    let mut package = parameters_package();
    if reverse_graph {
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("graph nodes")
            .reverse();
        rebuild_source_map(&mut package);
    }
    let package_name = package["name"].clone();
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes");
    let integer = node_id("scalar_type/integer");
    let text = node_id("scalar_type/text");
    let owner_position = nodes
        .iter()
        .position(|node| node["node_tag"] == "function")
        .expect("function owner");
    let parameter_key = "value/parameter/n/0";
    let app_key = "expression/integer_add";
    let parameter = json!({
        "node_id": node_id(parameter_key),
        "schema_version":"filament.checked-semantic-graph/v2",
        "node_tag":"value", "semantic_form":"parameter", "semantic_type":integer,
        "dependencies":[], "occurrences":[{"role":"expression","ordinal":0}],
        "body":{"term":"aggregate","members":[
            {"term":"binding","name":"name","value":{"term":"literal","type":text,"value_kind":"text","value":"n"}},
            {"term":"binding","name":"level","value":{"term":"literal","type":integer,"value_kind":"integer","value":"0"}}
        ]}
    });
    let mut application_dependencies = vec![node_id(parameter_key), integer.clone()];
    application_dependencies.sort_by_key(structural_fixture_id);
    let application = json!({
        "node_id": node_id(app_key),
        "schema_version":"filament.checked-semantic-graph/v2",
        "node_tag":"expression", "semantic_form":"binary", "semantic_type":integer,
        "dependencies":application_dependencies,
        "occurrences":[{"role":"expression","ordinal":0},{"role":"expression","ordinal":1}],
        "body":{"term":"application","operator":"binary",
            "operation":{"identity":"filament.op.integer.add","laws":[],"mode":null,"member":null,"leaves":[]},
            "result_type":integer,
            "arguments":[{"term":"reference","target":node_id(parameter_key)},
                {"term":"literal","type":integer,"value_kind":"integer","value":"1"}]}
    });
    let boolean = node_id("scalar_type/boolean");
    let mut guard_dependencies = vec![node_id(parameter_key), boolean.clone()];
    guard_dependencies.sort_by_key(structural_fixture_id);
    let guard = json!({
        "node_id":node_id("expression/integer_ge"),
        "schema_version":"filament.checked-semantic-graph/v2",
        "node_tag":"expression", "semantic_form":"binary", "semantic_type":boolean,
        "dependencies":guard_dependencies,
        "occurrences":[{"role":"expression","ordinal":0}],
        "body":{"term":"application","operator":"binary",
            "operation":{"identity":"filament.op.integer.ge","laws":[],"mode":null,"member":null,"leaves":[]},
            "result_type":boolean,
            "arguments":[{"term":"reference","target":node_id(parameter_key)},
                {"term":"literal","type":integer,"value_kind":"integer","value":"0"}]}
    });
    let mut equality_dependencies = vec![node_id(app_key), boolean.clone()];
    equality_dependencies.sort_by_key(structural_fixture_id);
    let equality = json!({
        "node_id":node_id("expression/integer_eq"),
        "schema_version":"filament.checked-semantic-graph/v2",
        "node_tag":"expression", "semantic_form":"binary", "semantic_type":boolean,
        "dependencies":equality_dependencies,
        "occurrences":[{"role":"expression","ordinal":0}],
        "body":{"term":"application","operator":"binary",
            "operation":{"identity":"filament.op.integer.eq","laws":[],"mode":null,"member":null,"leaves":[]},
            "result_type":boolean,
            "arguments":[{"term":"reference","target":node_id(app_key)},
                {"term":"literal","type":integer,"value_kind":"integer","value":"0"}]}
    });
    let mut root_dependencies = vec![
        node_id("expression/integer_ge"),
        node_id("expression/integer_eq"),
        boolean.clone(),
    ];
    root_dependencies.sort_by_key(structural_fixture_id);
    let root = json!({
        "node_id":node_id("expression/boolean_and"),
        "schema_version":"filament.checked-semantic-graph/v2",
        "node_tag":"expression", "semantic_form":"binary", "semantic_type":boolean,
        "dependencies":root_dependencies,
        "occurrences":[{"role":"expression","ordinal":0}],
        "body":{"term":"application","operator":"binary",
            "operation":{"identity":"filament.op.boolean.and","laws":[],"mode":null,"member":null,"leaves":[]},
            "result_type":boolean,
            "arguments":[{"term":"reference","target":node_id("expression/integer_ge")},
                {"term":"reference","target":node_id("expression/integer_eq")}]}
    });
    nodes.push(parameter);
    nodes.push(application);
    nodes.push(guard);
    nodes.push(equality);
    nodes.push(root);
    let parameter_position = nodes.len() - 5;
    let application_position = nodes.len() - 4;
    let guard_position = nodes.len() - 3;
    nodes[owner_position]["semantic_type"] = boolean.clone();
    let mut owner_dependencies = vec![node_id(parameter_key), node_id("expression/boolean_and")];
    owner_dependencies.sort_by_key(structural_fixture_id);
    nodes[owner_position]["dependencies"] = json!(owner_dependencies);
    nodes[owner_position]["body"] = json!({"term":"aggregate","members":[
        {"term":"binding","name":"parameters","value":{"term":"aggregate","members":[
            {"term":"reference","target":node_id(parameter_key)}]}},
        {"term":"binding","name":"body","value":{"term":"reference","target":node_id("expression/boolean_and")}}
    ]});
    settle(&mut package);

    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("settled nodes");
    let owner = nodes[owner_position]["node_id"].clone();
    let parameter = nodes[parameter_position]["node_id"].clone();
    let application = nodes[application_position]["node_id"].clone();
    let guard = nodes[guard_position]["node_id"].clone();
    let bound = nodes
        .iter()
        .find(|node| node["semantic_form"] == "integer")
        .expect("integer bound")["node_id"]
        .clone();
    let mut dependencies = vec![
        owner.clone(),
        parameter.clone(),
        application.clone(),
        bound.clone(),
    ];
    dependencies.sort_by_key(structural_fixture_id);
    let binding = |name: &str, value: Value| json!({"term":"binding","name":name,"value":value});
    let bound_position = nodes
        .iter()
        .position(|node| node["node_id"] == bound)
        .expect("bound index");
    let mut record_positions = Vec::new();
    for occurrence in 0..2 {
        let guard_path = if occurrence == 0 {
            Vec::new()
        } else {
            vec![json!({"node":guard_position}), json!("0"), json!("true")]
        };
        let mut path = json!([
            "claim", "value_validity", {"node":owner_position}, {"node":application_position},
            occurrence.to_string(), "bound", {"node":bound_position},
            "conversion", "guards"
        ]);
        path.as_array_mut().expect("record path").extend(guard_path);
        path.as_array_mut().expect("record path").extend([
            json!("roots"),
            json!({"node":parameter_position}),
            json!("objects"),
        ]);
        let record_id = json!({"package":package_name,"path":path,"ordinal":0});
        let mut record_dependencies = dependencies.clone();
        if occurrence == 1 {
            record_dependencies.push(guard.clone());
            record_dependencies.sort_by_key(structural_fixture_id);
            record_dependencies.dedup();
        }
        let record = json!({
            "node_id":record_id,
            "schema_version":"filament.checked-semantic-graph/v2",
            "node_tag":"claim", "semantic_form":"value_validity", "semantic_type":node_id("scalar_type/boolean"),
            "dependencies":record_dependencies,
            "occurrences":[{"role":"generated","ordinal":occurrence}],
            "body":{"term":"aggregate","members":[
                binding("owner",json!({"term":"reference","target":owner})),
                binding("application",json!({"term":"reference","target":application})),
                binding("occurrence",json!({"term":"literal","type":node_id("scalar_type/integer"),"value_kind":"integer","value":occurrence})),
                binding("result_bound",json!({"term":"reference","target":bound})),
                binding("conversion",json!({"term":"aggregate","members":[]})),
                binding("path_condition", if occurrence == 0 {
                    json!({"term":"aggregate","members":[]})
                } else {
                    json!({"term":"aggregate","members":[json!({"term":"aggregate","members":[
                        binding("guard",json!({"term":"reference","target":guard})),
                        binding("occurrence",json!({"term":"literal","type":node_id("scalar_type/integer"),"value_kind":"integer","value":0})),
                        binding("holds",json!({"term":"literal","type":node_id("scalar_type/boolean"),"value_kind":"boolean","value":true}))
                    ]})]})
                }),
                binding("extent_roots",json!({"term":"aggregate","members":[{"term":"reference","target":parameter}]})),
                binding("objects",json!({"term":"aggregate","members":[]})),
            ]}
        });
        nodes.push(record);
        record_positions.push(nodes.len() - 1);
    }
    settle(&mut package);
    rebuild_complete_source_map(&mut package);
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let application = nodes[application_position]["node_id"].clone();
    let record_ids = record_positions
        .iter()
        .map(|position| nodes[*position]["node_id"].clone())
        .collect::<Vec<_>>();
    package["requirement_items"] = json!([
        {"claim":{"node":application,"occurrence":{"role":"expression","ordinal":0}},
            "kind":"value-validity","extent":{"classification":"bounded"},"record":record_ids[0]},
        {"claim":{"node":application,"occurrence":{"role":"expression","ordinal":1}},
            "kind":"value-validity","extent":{"classification":"bounded"},"record":record_ids[1]}
    ]);
    let source_map = package["source_map"].as_array_mut().expect("source map");
    for (ordinal, record) in record_ids.iter().enumerate() {
        let application_region = source_map
            .iter()
            .find(|row| row["node_id"] == application && row["ordinal"] == ordinal)
            .expect("application occurrence region")["regions"][0]
            .clone();
        let record_source = source_map
            .iter_mut()
            .find(|row| row["node_id"] == *record)
            .expect("record source row");
        record_source["regions"][0] = application_region;
    }
    package
}

fn boolean_value_guard_package(path_condition: &Value) -> Value {
    let mut package = value_validity_package(false);
    let boolean = node_id("scalar_type/boolean");
    let (boolean_parameter_position, record_rows) = {
        let nodes = package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("graph nodes");
        let integer_position = nodes
            .iter()
            .position(|node| node["node_id"] == node_id("value/parameter/n/0"))
            .expect("integer parameter");
        nodes[integer_position]["body"]["members"][0]["value"]["value"] = json!("x");
        let integer_parameter = nodes[integer_position].clone();
        let boolean_parameter_id = node_id("value/parameter/p/0");
        let integer = integer_parameter["semantic_type"].clone();
        let mut boolean_parameter = integer_parameter;
        boolean_parameter["node_id"] = boolean_parameter_id.clone();
        boolean_parameter["semantic_type"] = boolean.clone();
        boolean_parameter["body"]["members"][0]["value"]["value"] = json!("p");
        nodes.push(boolean_parameter);

        let application_id = node_id("expression/integer_add");
        let conditional_id = node_id("expression/boolean_guard_if");
        let mut conditional_dependencies = vec![
            boolean_parameter_id.clone(),
            application_id.clone(),
            integer.clone(),
        ];
        conditional_dependencies.sort_by_key(structural_fixture_id);
        nodes.push(json!({
            "node_id":conditional_id,
            "schema_version":"filament.checked-semantic-graph/v2",
            "node_tag":"expression",
            "semantic_form":"conditional",
            "semantic_type":integer,
            "dependencies":conditional_dependencies,
            "occurrences":[{"role":"expression","ordinal":0}],
            "body":{"term":"application","operator":"conditional",
                "operation":{"identity":"filament.op.control.if","laws":[],"mode":null,"member":null,"leaves":[]},
                "result_type":integer,
                "arguments":[
                    {"term":"reference","target":boolean_parameter_id},
                    {"term":"reference","target":application_id},
                    {"term":"reference","target":application_id}
                ]}
        }));
        let owner = nodes
            .iter_mut()
            .find(|node| node["node_tag"] == "function")
            .expect("function owner");
        owner["semantic_type"] = integer;
        owner["dependencies"] = json!([
            node_id("value/parameter/n/0"),
            boolean_parameter_id.clone(),
            conditional_id
        ]);
        owner["body"]["members"][1]["value"]["target"] = conditional_id;
        owner["body"]["members"][0]["value"]["members"]
            .as_array_mut()
            .expect("owner parameters")
            .push(json!({"term":"reference","target":boolean_parameter_id.clone()}));

        let boolean_parameter_position = nodes
            .iter()
            .position(|node| node["node_id"] == boolean_parameter_id)
            .expect("Boolean parameter position");
        let record_rows = nodes
            .iter()
            .enumerate()
            .filter_map(|(position, node)| {
                (node["semantic_form"] == "value_validity").then(|| {
                    (
                        position,
                        node["body"]["members"][2]["value"]["value"]
                            .as_u64()
                            .expect("application occurrence"),
                    )
                })
            })
            .collect::<Vec<_>>();
        (boolean_parameter_position, record_rows)
    };
    for &(record_position, application_occurrence) in &record_rows {
        let vector = path_condition
            .as_array()
            .expect("published path conditions")
            .iter()
            .find(|row| row["application_occurrence"].as_u64() == Some(application_occurrence))
            .expect("path-condition vector for application occurrence");
        let guard_read_occurrence = vector["guard_read_occurrence"]
            .as_u64()
            .expect("guard read occurrence");
        rekey_requirement_record(
            &mut package,
            record_position,
            boolean_parameter_position,
            guard_read_occurrence,
            vector["holds"].as_bool().expect("required guard outcome"),
            None,
        );
    }
    let boolean_parameter_id = node_id("value/parameter/p/0");
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes");
    for record in nodes
        .iter_mut()
        .filter(|node| node["semantic_form"] == "value_validity")
    {
        let application_occurrence = record["body"]["members"][2]["value"]["value"]
            .as_u64()
            .expect("application occurrence");
        let vector = path_condition
            .as_array()
            .expect("published path conditions")
            .iter()
            .find(|row| row["application_occurrence"].as_u64() == Some(application_occurrence))
            .expect("path-condition vector for application occurrence");
        let guard_read_occurrence = vector["guard_read_occurrence"]
            .as_u64()
            .expect("guard read occurrence");
        let holds = vector["holds"].as_bool().expect("required guard outcome");
        record["body"]["members"][5]["value"]["members"] = json!([{
            "term":"aggregate","members":[
                {"term":"binding","name":"guard","value":{"term":"reference","target":boolean_parameter_id.clone()}},
                {"term":"binding","name":"occurrence","value":{"term":"literal","type":node_id("scalar_type/integer"),"value_kind":"integer","value":guard_read_occurrence}},
                {"term":"binding","name":"holds","value":{"term":"literal","type":boolean,"value_kind":"boolean","value":holds}}
            ]
        }]);
        record["body"]["members"][6]["value"]["members"] = json!([
            {"term":"reference","target":node_id("value/parameter/n/0")},
            {"term":"reference","target":boolean_parameter_id.clone()}
        ]);
        record["body"]["members"][6]["value"]["members"]
            .as_array_mut()
            .expect("extent roots")
            .sort_by_key(|root| structural_fixture_id(&root["target"]));
        sync_body_dependencies(record);
    }
    for node in nodes
        .iter_mut()
        .filter(|node| node["node_tag"] == "function")
    {
        sync_body_dependencies(node);
    }
    settle(&mut package);
    rebuild_body_locations(&mut package);
    rebuild_complete_source_map(&mut package);
    package
}

fn boolean_literal_guard_package(path_condition: &Value) -> Value {
    let mut package = boolean_value_guard_package(path_condition);
    let boolean = node_id("scalar_type/boolean");
    let literal_id = node_id("value/literal/boolean_guard");
    {
        let nodes = package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("graph nodes");
        let literal = json!({
            "node_id":literal_id,
            "schema_version":"filament.checked-semantic-graph/v2",
            "node_tag":"value",
            "semantic_form":"literal",
            "semantic_type":boolean,
            "dependencies":[],
            "occurrences":[{"role":"expression","ordinal":0}],
            "body":{"term":"literal","type":boolean,"value_kind":"boolean","value":true}
        });
        nodes.push(literal);
        let literal_position = nodes.len() - 1;
        let conditional_position = nodes
            .iter()
            .position(|node| node["node_id"] == node_id("expression/boolean_guard_if"))
            .expect("conditional body root");
        nodes[conditional_position]["body"]["arguments"][0] =
            json!({"term":"reference","target":literal_id});
        sync_body_dependencies(&mut nodes[conditional_position]);
        nodes[conditional_position]["dependencies"]
            .as_array_mut()
            .expect("conditional dependencies")
            .push(node_id("scalar_type/integer"));
        nodes[conditional_position]["dependencies"]
            .as_array_mut()
            .expect("conditional dependencies")
            .sort_by_key(structural_fixture_id);
    }

    // A true literal condition has no false branch, so the literal-form
    // control retains only the record for application occurrence zero.
    let discarded_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .position(|node| {
            node["semantic_form"] == "value_validity"
                && node["body"]["members"][2]["value"]["value"] == 1
        })
        .expect("false-branch value-validity record");
    let discarded_id = package["semantic_graph"]["nodes"][discarded_position]["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes")
        .remove(discarded_position);
    package["requirement_items"]
        .as_array_mut()
        .expect("requirement rows")
        .retain(|row| row["record"] != discarded_id);
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .retain(|row| row["node_id"] != discarded_id);

    let vector = path_condition
        .as_array()
        .expect("published path conditions")
        .iter()
        .find(|row| row["application_occurrence"].as_u64() == Some(0))
        .expect("true-branch path-condition vector");
    let parameter_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .position(|node| node["node_id"] == node_id("value/parameter/p/0"))
        .expect("Boolean parameter position");
    let parameter_id = package["semantic_graph"]["nodes"][parameter_position]["node_id"].clone();
    let owner_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .position(|node| node["node_tag"] == "function")
        .expect("function owner");
    package["semantic_graph"]["nodes"][owner_position]["body"]["members"][0]["value"]["members"]
        .as_array_mut()
        .expect("owner parameters")
        .retain(|parameter| parameter["target"] != parameter_id);
    sync_body_dependencies(&mut package["semantic_graph"]["nodes"][owner_position]);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes")
        .remove(parameter_position);
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .retain(|row| row["node_id"] != parameter_id);
    let literal_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .position(|node| node["node_id"] == literal_id)
        .expect("Boolean literal position after parameter removal");
    let record_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .position(|node| {
            node["semantic_form"] == "value_validity"
                && node["body"]["members"][2]["value"]["value"] == 0
        })
        .expect("true-branch value-validity record after parameter removal");
    {
        let record = &mut package["semantic_graph"]["nodes"][record_position];
        record["body"]["members"][5]["value"]["members"][0]["members"][0]["value"] =
            json!({"term":"reference","target":literal_id});
        sync_body_dependencies(record);
    }
    let guard_read_occurrence = vector["guard_read_occurrence"]
        .as_u64()
        .expect("guard read occurrence");
    let holds = vector["holds"].as_bool().expect("required guard outcome");
    rekey_requirement_record(
        &mut package,
        record_position,
        literal_position,
        guard_read_occurrence,
        holds,
        Some((parameter_position, parameter_id)),
    );
    settle(&mut package);
    rebuild_body_locations(&mut package);
    rebuild_complete_source_map(&mut package);
    package
}

fn rekey_requirement_record(
    package: &mut Value,
    record_position: usize,
    guard_position: usize,
    guard_read_occurrence: u64,
    holds: bool,
    extent_root_to_remove: Option<(usize, Value)>,
) {
    let (old_id, new_id) = {
        let record = &mut package["semantic_graph"]["nodes"][record_position];
        let old_id = record["node_id"].clone();
        {
            let path = record["node_id"]["path"]
                .as_array_mut()
                .expect("record structural path");
            let guards = path
                .iter()
                .position(|segment| segment == "guards")
                .expect("guard path segment");
            let roots = path
                .iter()
                .enumerate()
                .skip(guards + 1)
                .find_map(|(index, segment)| (segment == "roots").then_some(index))
                .expect("roots path segment");
            path.splice(
                guards + 1..roots,
                [
                    json!({"node":guard_position}),
                    json!(guard_read_occurrence.to_string()),
                    json!(holds.to_string()),
                ],
            );
            if let Some((root_position, root_id)) = &extent_root_to_remove {
                let index_root = json!({"node":root_position});
                if let Some(roots) = path.iter().position(|segment| segment == "roots") {
                    if let Some(root) =
                        path.iter()
                            .enumerate()
                            .skip(roots + 1)
                            .find_map(|(index, segment)| {
                                (segment == &index_root || segment == root_id).then_some(index)
                            })
                    {
                        path.remove(root);
                    }
                }
            }
        }
        if let Some((_, root_id)) = &extent_root_to_remove {
            if let Some(roots) = record["body"]["members"][6]["value"]["members"].as_array_mut() {
                roots.retain(|root| root["target"] != *root_id);
            }
            sync_body_dependencies(record);
        }
        (old_id, record["node_id"].clone())
    };
    for row in package["requirement_items"]
        .as_array_mut()
        .expect("requirement rows")
    {
        if row["record"] == old_id {
            row["record"] = new_id.clone();
        }
    }
}

#[trace("TC-488", "FR-376-AC-1")]
#[test]
fn public_reader_admits_typed_requirement_rows_in_package_order() {
    let mut package = parameters_package();
    let claim = function_claim(&package);
    package["requirement_items"] = json!([
        row(&claim, "finite-replay"),
        row(&claim, "operation-contract"),
    ]);

    let CheckedPackageV2ReadResult::Admitted(admitted) = read(&package) else {
        panic!("public V2 reader admits ordered requirement rows")
    };
    let rows = admitted.requirement_items();
    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows[0].kind,
        filament_contract_model::CapabilityKind::FiniteReplay
    );
    assert_eq!(
        rows[1].kind,
        filament_contract_model::CapabilityKind::OperationContract
    );
    assert_eq!(rows[0].claim, rows[1].claim);
    assert_eq!(rows[0].occurrence, rows[1].occurrence);
    assert_eq!(
        rows[0].extent,
        filament_contract_model::RequirementExtent::Bounded
    );
    assert!(rows.iter().all(|row| row.record.is_none()));
}

#[trace("TC-488", "FR-376-AC-1")]
#[test]
fn public_reader_returns_empty_requirement_reads_when_the_member_is_absent() {
    let package = parameters_package();
    let owner: filament_contract_model::CheckedNodeId = serde_json::from_value(
        package["semantic_graph"]["nodes"]
            .as_array()
            .expect("graph nodes")
            .iter()
            .find(|node| node["node_tag"] == "function")
            .expect("function node")["node_id"]
            .clone(),
    )
    .expect("owner ID");

    let CheckedPackageV2ReadResult::Admitted(admitted) = read(&package) else {
        panic!("public V2 reader admits package without requirement rows")
    };
    assert!(admitted.requirement_items().is_empty());
    assert!(admitted.value_validity_records_of(&owner).is_empty());
    let other: filament_contract_model::CheckedNodeId = serde_json::from_value(json!({
        "package":"test/other", "path":["function"], "ordinal":0
    }))
    .expect("non-owner ID");
    assert!(admitted.value_validity_records_of(&other).is_empty());
}

#[trace("TC-488", "FR-376-AC-1")]
#[test]
fn public_reader_retains_value_validity_record_facts_and_owner_index() {
    let mut package = value_validity_package(false);
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes");
    let parameter = nodes
        .iter()
        .find(|node| node["semantic_form"] == "value_validity")
        .expect("value-validity record")["body"]["members"][6]["value"]["members"][0]["target"]
        .clone();
    for row in package["requirement_items"]
        .as_array_mut()
        .expect("requirement rows")
    {
        row["extent"] = json!({"classification":"unbounded","domains":[{
            "domain":{"type":"node","node":parameter,"path":[]},"kind":"integer"
        }]});
    }
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes");
    let record = nodes
        .iter()
        .find(|node| node["semantic_form"] == "value_validity")
        .expect("value-validity record");
    let owner = &record["body"]["members"][0]["value"]["target"];
    assert_eq!(
        nodes
            .iter()
            .find(|node| node["node_id"] == *owner)
            .expect("record owner target")["node_tag"],
        "function"
    );
    let owner: filament_contract_model::CheckedNodeId = serde_json::from_value(
        package["semantic_graph"]["nodes"]
            .as_array()
            .expect("graph nodes")
            .iter()
            .find(|node| node["node_tag"] == "function")
            .expect("function owner")["node_id"]
            .clone(),
    )
    .expect("owner ID");
    let CheckedPackageV2ReadResult::Admitted(admitted) = read(&package) else {
        panic!(
            "public reader admits the value-validity fixture: {:?}",
            read(&package)
        )
    };
    let resolved_owner = admitted
        .resolved_graph()
        .nodes
        .iter()
        .find(|node| node.node_tag.as_ref() == "function")
        .map(|node| &node.node_id)
        .expect("resolved function owner");
    let rows = admitted.requirement_items();
    assert_eq!(rows.len(), 2);
    let wire_claim: filament_contract_model::CheckedNodeId =
        serde_json::from_value(package["requirement_items"][0]["claim"]["node"].clone())
            .expect("document-held claim ID");
    assert_eq!(rows[0].claim, wire_claim);
    assert!(admitted
        .resolved_graph()
        .nodes
        .iter()
        .any(|node| node.node_id == rows[0].claim));
    let filament_contract_model::RequirementExtent::Unbounded(domains) = &rows[0].extent else {
        panic!("unbounded extent is retained as typed domain rows")
    };
    assert_eq!(domains.len(), 1);
    let expected_domain: filament_contract_model::CheckedNodeId = serde_json::from_value(
        package["requirement_items"][0]["extent"]["domains"][0]["domain"]["node"].clone(),
    )
    .expect("document-held domain ID");
    let resolved_parameter = admitted
        .resolved_graph()
        .nodes
        .iter()
        .find(|node| node.node_id == expected_domain)
        .map(|node| &node.node_id)
        .expect("resolved domain target");
    assert_eq!(
        &domains[0].0,
        &filament_contract_model::DomainKey::Node {
            node: resolved_parameter.clone(),
            path: Vec::new(),
        }
    );
    let record = rows[0]
        .record
        .as_ref()
        .expect("typed row retains its value-validity record");
    let wire_record: filament_contract_model::CheckedNodeId =
        serde_json::from_value(package["requirement_items"][0]["record"].clone())
            .expect("document-held record ID");
    assert_eq!(record.node, wire_record);
    assert_eq!(&record.owner, resolved_owner);
    assert_eq!(record.owner, owner);
    assert_eq!(&record.application, &rows[0].claim);
    assert!(admitted
        .resolved_graph()
        .nodes
        .iter()
        .any(|node| node.node_id == record.result_bound));
    assert_eq!(record.occurrence, 0);
    assert!(record.conversion.is_none());
    assert!(record.path_condition.is_empty());
    assert_eq!(record.extent_roots.len(), 1);
    assert_eq!(&record.extent_roots[0], resolved_parameter);
    assert!(record.objects.is_empty());
    assert_eq!(
        rows[1].record.as_ref().expect("second record").occurrence,
        1
    );
    assert_eq!(admitted.value_validity_records_of(resolved_owner), &[0, 1]);

    let mut reordered_package = value_validity_package(true);
    let reordered_parameter = reordered_package["semantic_graph"]["nodes"]
        .as_array()
        .expect("reordered graph nodes")
        .iter()
        .find(|node| node["semantic_form"] == "value_validity")
        .expect("reordered value-validity record")["body"]["members"][6]["value"]["members"][0]
        ["target"]
        .clone();
    for row in reordered_package["requirement_items"]
        .as_array_mut()
        .expect("reordered requirement rows")
    {
        row["extent"] = json!({"classification":"unbounded","domains":[{
            "domain":{"type":"node","node":reordered_parameter,"path":[]},"kind":"integer"
        }]});
    }
    let CheckedPackageV2ReadResult::Admitted(reordered) = read(&reordered_package) else {
        panic!("public reader admits the same nodes in another graph order")
    };
    let reordered_rows = reordered.requirement_items();
    assert_eq!(reordered_rows.len(), rows.len());
    for (reordered_row, row) in reordered_rows.iter().zip(rows) {
        assert_eq!(reordered_row.claim, row.claim);
        assert_eq!(reordered_row.occurrence, row.occurrence);
        assert_eq!(reordered_row.kind, row.kind);
        assert_eq!(reordered_row.extent, row.extent);
        let reordered_record = reordered_row.record.as_ref().expect("reordered record");
        let record = row.record.as_ref().expect("record");
        assert_eq!(reordered_record.owner, record.owner);
        assert_eq!(reordered_record.application, record.application);
        assert_eq!(reordered_record.occurrence, record.occurrence);
        assert_eq!(reordered_record.result_bound, record.result_bound);
        assert_eq!(reordered_record.conversion, record.conversion);
        assert_eq!(reordered_record.path_condition, record.path_condition);
        assert_eq!(reordered_record.extent_roots, record.extent_roots);
        assert_eq!(reordered_record.objects, record.objects);
    }
    assert_eq!(reordered.value_validity_records_of(resolved_owner), &[0, 1]);
}

#[trace("TC-488", "FR-376-AC-1")]
#[test]
fn canonical_writer_readback_preserves_typed_requirement_rows() {
    let package = value_validity_package(false);
    let CheckedPackageV2ReadResult::Admitted(admitted) = read(&package) else {
        panic!(
            "public reader admits the bounded value-validity fixture: {:?}",
            read(&package)
        )
    };
    let bytes = encode_checked_package_v2_with_requirement_items(
        admitted.name(),
        admitted.lock(),
        admitted.graph(),
        CheckedPackageSourceCorrespondenceRef {
            source_map: admitted.source_map(),
            refinement_step_rows: admitted.refinement_step_rows(),
        },
        CheckedPackageSidecarsRef {
            checked_term_data: admitted.checked_term_data(),
            temporal_bindings: admitted.temporal_bindings(),
            temporal_clause_names: admitted.temporal_clause_names(),
            claim_names: admitted.claim_names(),
            schedulability_claims: None,
            operation_effects: None,
            operation_bodies: None,
        },
        Some(admitted.requirement_items()),
        admitted.capability_report(),
        admitted.diagnostics(),
        quire_canonical::Limits::new(1 << 24),
    )
    .expect("canonical writer retains requirement rows");
    let encoded: Value = serde_json::from_slice(&bytes).expect("canonical V2 JSON");
    let mut source = package.clone();
    index_fixture_node_ids(&mut source);
    source["semantic_graph"] = serde_json::to_value(admitted.graph())
        .expect("writer graph is the package's canonical slot graph");
    source
        .as_object_mut()
        .expect("package object")
        .remove("operation_effects");
    let differing_members = encoded
        .as_object()
        .expect("encoded package object")
        .iter()
        .filter_map(|(name, value)| (source.get(name) != Some(value)).then_some(name.as_str()))
        .collect::<Vec<_>>();
    assert!(
        differing_members.is_empty(),
        "typed re-encode changed members: {differing_members:?}"
    );
    // The writer's bytes are already canonical wire: read them as written.
    let CheckedPackageV2ReadResult::Admitted(round_trip) = CheckedPackageV2::read(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&encoded),
    ) else {
        panic!("writer output is admitted by the public reader")
    };
    assert_eq!(round_trip.requirement_items(), admitted.requirement_items());
}

#[trace("TC-488", "FR-376-AC-3")]
#[test]
fn public_reader_admits_at_the_exact_work_budget_and_reports_the_last_row_one_short() {
    let package = value_validity_package(false);
    let run = |package: &Value, limit: u64| {
        let mut wire = package.clone();
        index_fixture_node_ids(&mut wire);
        let bytes = canonical(&wire);
        let evidence = evidence_for(&wire);
        let mut budget = NativeJoinBudget::new(limit, u64::MAX).with_read_storage_limit(u64::MAX);
        let result = CheckedPackageV2::read_with_budget(
            &bytes,
            CheckedPackageReadLimits::bounded(),
            &evidence,
            &mut budget,
        );
        (result, budget.consumed().work)
    };

    // Measure the complete read's accepted work and the prefix before the
    // requirement rows. The no-row run identifies the boundary immediately
    // before those two row visits.
    let mut no_rows = package.clone();
    no_rows
        .as_object_mut()
        .expect("package")
        .remove("requirement_items");
    let (missing_rows, before_rows) = run(&no_rows, u64::MAX);
    let CheckedPackageV2BudgetedReadResult::Refused(refusal) = missing_rows else {
        panic!("missing rows are refused after the record-node and owner walk")
    };
    assert_eq!(
        refusal.loci[0].path,
        JsonPointer::parse("/requirement_items")
    );
    let (full, required) = run(&package, u64::MAX);
    assert!(matches!(
        full,
        CheckedPackageV2BudgetedReadResult::Admitted(_)
    ));
    assert!(required > before_rows);

    let (exact, exact_consumed) = run(&package, required);
    assert!(matches!(
        exact,
        CheckedPackageV2BudgetedReadResult::Admitted(_)
    ));
    assert_eq!(exact_consumed, required);

    let requirement_step = before_rows + 2;
    let (step_exact, step_consumed) = run(&package, requirement_step);
    let CheckedPackageV2BudgetedReadResult::BudgetIncomplete(step_incomplete) = step_exact else {
        panic!("the exact requirement-step budget stops in a later read stage")
    };
    assert_eq!(step_consumed, requirement_step);
    assert_ne!(
        step_incomplete.path,
        JsonPointer::parse("/requirement_items/1")
    );

    let (short, short_consumed) = run(&package, requirement_step - 1);
    let CheckedPackageV2BudgetedReadResult::BudgetIncomplete(incomplete) = short else {
        panic!("one work visit short of the requirement step returns incomplete")
    };
    assert!(short_consumed < requirement_step);
    assert_eq!(incomplete.path, JsonPointer::parse("/requirement_items/1"));
}

#[trace("TC-488", "FR-376-AC-2")]
#[test]
fn public_reader_refuses_out_of_order_rows_without_admitting_a_partial_package() {
    let mut package = parameters_package();
    let claim = function_claim(&package);
    package["requirement_items"] = json!([
        row(&claim, "operation-contract"),
        row(&claim, "finite-replay"),
    ]);

    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("public V2 reader refuses rows outside package order")
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::InvalidSemanticGraph)
    );
    assert_eq!(
        refusal.loci[0].path,
        JsonPointer::parse("/requirement_items/1")
    );
}

#[trace("TC-488", "FR-376-AC-2")]
#[test]
fn public_reader_refuses_a_malformed_requirement_row_at_its_wire_pointer() {
    let mut package = parameters_package();
    let claim = function_claim(&package);
    package["requirement_items"] = json!([{
        "claim": claim,
        "kind": "operation-contract",
        "extent": {"classification":"bounded"},
        "unexpected": true,
    }]);

    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("public V2 reader refuses an open requirement-row schema")
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::UnknownMember);
    assert_eq!(refusal.cause, None);
    assert_eq!(
        refusal.loci[0].path,
        JsonPointer::parse("/requirement_items/0/unexpected")
    );
}

#[trace("TC-488", "FR-376-AC-2")]
#[test]
fn public_reader_refuses_a_value_validity_row_without_its_record() {
    let mut package = parameters_package();
    let claim = function_claim(&package);
    package["requirement_items"] = json!([{
        "claim": claim,
        "kind": "value-validity",
        "extent": {"classification":"bounded"},
    }]);

    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("value-validity requirement has no complete record")
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::InvalidValue)
    );
    assert_eq!(
        refusal.loci[0].path,
        JsonPointer::parse("/requirement_items/0")
    );
}

#[trace("TC-488", "FR-376-AC-2")]
#[test]
fn public_reader_refuses_a_record_on_a_non_value_validity_row() {
    let mut package = parameters_package();
    let claim = function_claim(&package);
    let mut requirement = row(&claim, "operation-contract");
    requirement["record"] = json!({
        "package":"test/parameters", "path":["not-a-record"], "ordinal":0
    });
    package["requirement_items"] = json!([requirement]);

    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("non-value-validity requirement cannot name a value-validity record")
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::InvalidValue)
    );
    assert_eq!(
        refusal.loci[0].path,
        JsonPointer::parse("/requirement_items/0/record")
    );
}

#[trace("TC-488", "FR-376-AC-2")]
#[test]
fn public_reader_refuses_record_node_mutations_at_their_rule_pointers() {
    type Mutation = Box<dyn Fn(&mut Value)>;
    let mutations: Vec<(&str, Mutation, &str)> = vec![
        (
            "swapped result-bound and conversion bindings",
            Box::new(|record| {
                record["body"]["members"][3]["name"] = json!("conversion");
                record["body"]["members"][4]["name"] = json!("result_bound");
            }),
            "/body",
        ),
        (
            "wrong owner kind",
            Box::new(|record| {
                record["body"]["members"][0]["value"]["target"] =
                    record["body"]["members"][1]["value"]["target"].clone()
            }),
            "/body/members/0",
        ),
        (
            "wrong application kind",
            Box::new(|record| {
                record["body"]["members"][1]["value"]["target"] =
                    record["body"]["members"][0]["value"]["target"].clone()
            }),
            "/body/members/1",
        ),
        (
            "absent application occurrence",
            Box::new(|record| record["body"]["members"][2]["value"]["value"] = json!("9")),
            "/body/members/2",
        ),
        (
            "non-type result bound",
            Box::new(|record| {
                record["body"]["members"][3]["value"]["target"] =
                    record["body"]["members"][6]["value"]["members"][0]["target"].clone()
            }),
            "/body/members/3",
        ),
        (
            "malformed conversion",
            Box::new(
                |record| record["body"]["members"][4]["value"] = json!({"term":"aggregate","members":[{"term":"literal","type":node_id("scalar_type/integer"),"value_kind":"integer","value":"0"}]}),
            ),
            "/body/members/4",
        ),
        (
            "repeated extent root",
            Box::new(|record| {
                let root = record["body"]["members"][6]["value"]["members"][0].clone();
                record["body"]["members"][6]["value"]["members"] = json!([root.clone(), root]);
            }),
            "/body/members/6/value/members/1",
        ),
        (
            "wrong object type",
            Box::new(|record| {
                record["body"]["members"][7]["value"]["members"] = json!([
                    {"term":"reference","target":node_id("scalar_type/boolean")}
                ]);
            }),
            "/body/members/7/value/members/0",
        ),
    ];

    for (label, mutate, suffix) in mutations {
        let mut package = value_validity_package(false);
        let record_position = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .position(|node| {
                node["semantic_form"] == "value_validity" && node["occurrences"][0]["ordinal"] == 0
            })
            .expect("first record");
        mutate(&mut package["semantic_graph"]["nodes"][record_position]);
        sync_body_dependencies(&mut package["semantic_graph"]["nodes"][record_position]);

        let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
            panic!("{label} is refused without admitting a partial package")
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::InvalidPackage,
            "{label}: {refusal:?}"
        );
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::InvalidValue),
            "{label}: {refusal:?}"
        );
        assert_eq!(
            refusal.loci[0].path,
            JsonPointer::parse(&format!("/semantic_graph/nodes/{record_position}{suffix}")),
            "{label}: {refusal:?}"
        );
    }
}

/// FSpec FR-825-AC-4 rule 1 names a record node with a `declaration` member
/// as `invalid_package`/`invalid-value` at `occurrences`. The reader refuses
/// it at an earlier stage: the closed schema's `OwnerPresenceRule` (a node with
/// `declaration` needs an owner: `malformed_wire` at the node); with an owner,
/// `DeclarationOccurrenceRule` (`invalid_semantic_graph` at `declaration`);
/// with a `declaration` occurrence too, the `claim_names` stage
/// (`invalid_package` at `/claim_names/0/declaration`). No schema-valid record
/// node reaches rule 1, so this stays red pending an FSpec ruling.
#[trace("TC-488", "FR-376-AC-2")]
#[test]
fn public_reader_refuses_a_record_node_with_a_declaration_member_at_occurrences() {
    let mut package = value_validity_package(false);
    let record_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["semantic_form"] == "value_validity" && node["occurrences"][0]["ordinal"] == 0
        })
        .expect("first record");
    package["semantic_graph"]["nodes"][record_position]["declaration"] =
        json!({"qualified_name":["bad"]});
    assert_refusal(
        &package,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        &format!("/semantic_graph/nodes/{record_position}/occurrences"),
    );
}

#[trace("TC-488", "FR-376-AC-2")]
#[test]
fn public_reader_refuses_a_second_generated_occurrence_at_occurrences() {
    let mut package = value_validity_package(false);
    let record_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["semantic_form"] == "value_validity" && node["occurrences"][0]["ordinal"] == 0
        })
        .expect("first record");
    let record_id = package["semantic_graph"]["nodes"][record_position]["node_id"].clone();
    package["semantic_graph"]["nodes"][record_position]["occurrences"] = json!([
        {"role":"generated","ordinal":0},
        {"role":"generated","ordinal":1}
    ]);
    let mut source_row = package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .find(|row| row["node_id"] == record_id)
        .expect("record source row")
        .clone();
    source_row["ordinal"] = json!(1);
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(source_row);
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .sort_by_key(|row| {
            (
                row["node_id"].to_string(),
                row["role"].as_str().expect("role").to_owned(),
                row["ordinal"].as_u64().expect("ordinal"),
            )
        });

    assert_refusal(
        &package,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        &format!("/semantic_graph/nodes/{record_position}/occurrences"),
    );
}

#[trace("TC-488", "FR-376-AC-2")]
#[test]
fn public_reader_refuses_requirement_row_mutations_at_their_rule_pointers() {
    let mut absent = value_validity_package(false);
    absent
        .as_object_mut()
        .expect("package object")
        .remove("requirement_items");
    assert_refusal(
        &absent,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        "/requirement_items",
    );

    let mut empty = value_validity_package(false);
    empty["requirement_items"] = json!([]);
    assert_refusal(
        &empty,
        CheckedPackageRefusalCode::MalformedWire,
        None,
        "/requirement_items",
    );

    let mut missing_extent = value_validity_package(false);
    missing_extent["requirement_items"][0]
        .as_object_mut()
        .expect("row")
        .remove("extent");
    assert_refusal(
        &missing_extent,
        CheckedPackageRefusalCode::MalformedWire,
        None,
        "/requirement_items/0",
    );

    let mut unknown_kind = value_validity_package(false);
    unknown_kind["requirement_items"][0]["kind"] = json!("value-validty");
    assert_refusal(
        &unknown_kind,
        CheckedPackageRefusalCode::MalformedWire,
        None,
        "/requirement_items/0/kind",
    );

    let mut wrong_occurrence = value_validity_package(false);
    wrong_occurrence["requirement_items"][0]["claim"]["occurrence"]["ordinal"] = json!(7);
    assert_refusal(
        &wrong_occurrence,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        "/requirement_items/0/claim",
    );

    let mut duplicate_domain = value_validity_package(false);
    let parameter = node_id("value/parameter/n/0");
    duplicate_domain["requirement_items"][0]["extent"] = json!({
        "classification":"unbounded",
        "domains":[
            {"domain":{"type":"node","node":parameter,"path":[]},"kind":"integer"},
            {"domain":{"type":"node","node":parameter,"path":[]},"kind":"integer"}
        ]
    });
    assert_refusal(
        &duplicate_domain,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        "/requirement_items/0/extent/domains/1",
    );

    let mut repeated = value_validity_package(false);
    let first = repeated["requirement_items"][0].clone();
    repeated["requirement_items"][1] = first;
    assert_refusal(
        &repeated,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::ConflictingDefinition),
        "/requirement_items/1",
    );

    let mut missing_row = value_validity_package(false);
    missing_row["requirement_items"]
        .as_array_mut()
        .expect("rows")
        .pop();
    let record_position = missing_row["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["semantic_form"] == "value_validity" && node["occurrences"][0]["ordinal"] == 1
        })
        .expect("second record");
    assert_refusal(
        &missing_row,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        &format!("/semantic_graph/nodes/{record_position}/body"),
    );

    let mut wrong_owner_ordinals = value_validity_package(false);
    let record_positions = wrong_owner_ordinals["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .enumerate()
        .filter_map(|(position, node)| {
            (node["semantic_form"] == "value_validity").then_some(position)
        })
        .collect::<Vec<_>>();
    wrong_owner_ordinals["semantic_graph"]["nodes"][record_positions[0]]["occurrences"][0]
        ["ordinal"] = json!(1);
    wrong_owner_ordinals["semantic_graph"]["nodes"][record_positions[1]]["occurrences"][0]
        ["ordinal"] = json!(0);
    // The source map stage precedes the record step: keep each swapped
    // occurrence's source row at its new ordinal.
    for position in [record_positions[0], record_positions[1]] {
        let node = wrong_owner_ordinals["semantic_graph"]["nodes"][position].clone();
        for entry in wrong_owner_ordinals["source_map"]
            .as_array_mut()
            .expect("source map")
        {
            if entry["node_id"] == node["node_id"] {
                entry["ordinal"] = node["occurrences"][0]["ordinal"].clone();
            }
        }
    }
    let first = record_positions[0];
    assert_refusal(
        &wrong_owner_ordinals,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        &format!("/semantic_graph/nodes/{first}/occurrences"),
    );
}

#[trace("TC-488", "FR-376-AC-2")]
#[test]
fn public_reader_refuses_claim_names_that_name_a_valid_value_validity_node() {
    let mut package = value_validity_package(false);
    let record = package["requirement_items"][0]["record"].clone();
    let region = package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .find(|entry| entry["node_id"] == record)
        .expect("record source row")["regions"][0]
        .clone();
    package["claim_names"] = json!([{
        "declaration":record,
        "occurrence":{"role":"claim","ordinal":0},
        "name":"Generated",
        "qualified_name":["Generated"],
        "declaration_region":region,
    }]);

    // `read` re-derives `claim_names` from the graph (the fixture refresh),
    // which would drop the injected row; keep it and refresh identity only.
    let mut wire = package.clone();
    let injected = wire["claim_names"].clone();
    index_fixture_node_ids(&mut wire);
    wire["claim_names"] = injected;
    crate::fixture_support::refresh_identity_projection(&mut wire);
    let CheckedPackageV2ReadResult::Refused(refusal) = CheckedPackageV2::read(
        &canonical(&wire),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&wire),
    ) else {
        panic!("a claim_names row naming a value_validity node is refused")
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::InvalidValue)
    );
    assert_eq!(
        refusal.loci[0].path,
        JsonPointer::parse("/claim_names/0/declaration")
    );
}

fn add_out_of_order_requirement_rows(package: &mut Value) {
    let claim = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find_map(|node| {
            node["occurrences"]
                .as_array()?
                .first()
                .map(|occurrence| json!({"node":node["node_id"],"occurrence":occurrence}))
        })
        .expect("an occurrence key");
    package["requirement_items"] = json!([
        row(&claim, "operation-contract"),
        row(&claim, "finite-replay"),
    ]);
}

#[trace("TC-488", "FR-376-AC-3")]
#[test]
fn probabilistic_rule_six_refusal_precedes_requirement_record_refusal() {
    let mut package = crate::probabilistic_delay::with_delays(
        &[(
            "precedence",
            "uniform",
            json!({"term":"aggregate","members":[]}),
        )],
        None,
    );
    let delay_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "probabilistic")
        .expect("delay node");
    package["semantic_graph"]["nodes"][delay_position]["body"]["members"][2]["value"]["value"] =
        json!("unknown-family");
    crate::fixture_support::refresh_identity(&mut package);
    add_out_of_order_requirement_rows(&mut package);

    assert_refusal(
        &package,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        &format!("/semantic_graph/nodes/{delay_position}/body/members/2/value/value"),
    );
}

#[trace("TC-488", "FR-376-AC-3")]
#[test]
fn requirement_record_refusal_precedes_reversed_probabilistic_claim_rows() {
    let vectors = crate::probabilistic_claims::vectors();
    let package_name = "test/families";
    let rows = vectors["positive"]["probabilistic_claims"]
        .as_array()
        .expect("positive rows")
        .iter()
        .map(|value| crate::probabilistic_claims::row(value, package_name))
        .collect();
    let mut package = crate::probabilistic_claims::fixture(rows, &vectors);
    package["checked_term_data"]["probabilistic_claims"]
        .as_array_mut()
        .expect("probabilistic rows")
        .reverse();
    add_out_of_order_requirement_rows(&mut package);

    assert_refusal(
        &package,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidSemanticGraph),
        "/requirement_items/1",
    );
}

#[trace("TC-2053", "FR-825-AC-6", "FR-825-AC-7")]
#[test]
fn published_requirement_record_vectors_admit_boolean_value_guards_and_refuse_integer_guards() {
    let proposal = super::proposal_dir_of(&super::fixtures_dir());
    let vectors = super::read_json(&proposal.join("requirement-record-vectors.json"))
        .expect("published FR-825 vectors");
    let positive = vectors["positive"]
        .as_array()
        .expect("positive requirement-record vectors")
        .iter()
        .find(|vector| vector["id"] == "RRV-01-bare-boolean-parameter-guard")
        .expect("RRV-01");
    assert_eq!(positive["expected"]["admitted"], true);
    assert_eq!(positive["expected"]["record_count"], 2);
    let path_condition = positive["path_condition"]
        .as_array()
        .expect("RRV-01 path conditions");
    assert_eq!(path_condition.len(), 2);
    assert_eq!(path_condition[0]["application_occurrence"], 0);
    assert_eq!(path_condition[1]["application_occurrence"], 1);
    assert_eq!(path_condition[0]["guard_read_occurrence"], 0);
    assert_eq!(path_condition[1]["guard_read_occurrence"], 0);
    assert_eq!(path_condition[0]["holds"], true);
    assert_eq!(path_condition[1]["holds"], false);
    let refusal_vector = vectors["reader_refusals"]
        .as_array()
        .expect("requirement-record refusal vectors")
        .iter()
        .find(|vector| vector["id"] == "RRV-02-non-boolean-value-guard")
        .expect("RRV-02");
    assert_eq!(refusal_vector["expected"]["code"], "invalid_package");
    assert_eq!(refusal_vector["expected"]["cause"], "invalid-value");
    assert_eq!(
        refusal_vector["expected"]["pointer"],
        "body/members/5/value/members/0"
    );
    let mutation = &refusal_vector["mutation"];
    assert_eq!(mutation["application_occurrence"], 0);
    assert_eq!(mutation["replace_guard"], "p");
    assert_eq!(mutation["with"]["name"], "x");
    assert_eq!(mutation["with"]["role"], "value/parameter");
    assert_eq!(mutation["with"]["semantic_type"], "scalar_type/integer");
    assert_eq!(mutation["guard_read_occurrence"]["node"], "x");
    assert_eq!(
        mutation["guard_read_occurrence"]["source_occurrence"],
        "its read in x + 1"
    );

    let package = boolean_value_guard_package(&positive["path_condition"]);
    let admitted = match read(&package) {
        CheckedPackageV2ReadResult::Admitted(admitted) => admitted,
        result => {
            panic!("RRV-01: Boolean parameter is admitted as a guard and extent root: {result:?}")
        }
    };
    assert_eq!(admitted.requirement_items().len(), 2);
    let records = admitted
        .requirement_items()
        .iter()
        .map(|row| row.record.as_ref().expect("value-validity record"))
        .collect::<Vec<_>>();
    assert_eq!(records[0].path_condition.len(), 1);
    assert_eq!(records[1].path_condition.len(), 1);
    assert_eq!(
        records[0].path_condition[0].guard,
        records[1].path_condition[0].guard
    );
    assert!(records.iter().all(|record| record
        .extent_roots
        .contains(&record.path_condition[0].guard)));
    for (record, vector) in records.iter().zip(path_condition) {
        assert_eq!(
            record.occurrence,
            vector["application_occurrence"].as_u64().unwrap()
        );
        assert_eq!(
            record.path_condition[0].occurrence,
            vector["guard_read_occurrence"].as_u64().unwrap()
        );
        assert_eq!(
            record.path_condition[0].holds,
            vector["holds"].as_bool().unwrap()
        );
    }
    assert!(records.iter().all(|record| record.extent_roots.len() == 2));

    let mut adverse = package;
    let mutation = &refusal_vector["mutation"];
    let integer_parameter = match mutation["with"]["name"].as_str() {
        Some("x") => node_id("value/parameter/n/0"),
        other => panic!("RRV-02 integer parameter name is unsupported: {other:?}"),
    };
    let (record_position, integer_parameter_position, holds, guard_read_occurrence) = {
        let nodes = adverse["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes");
        let record_position = nodes
            .iter()
            .position(|node| {
                node["semantic_form"] == "value_validity"
                    && node["body"]["members"][2]["value"]["value"]
                        == mutation["application_occurrence"]
            })
            .expect("first occurrence record");
        let integer_parameter_position = nodes
            .iter()
            .position(|node| node["node_id"] == integer_parameter)
            .expect("integer parameter position");
        assert_eq!(
            nodes[integer_parameter_position]["semantic_form"],
            "parameter"
        );
        assert_eq!(
            nodes[integer_parameter_position]["semantic_type"],
            node_id("scalar_type/integer")
        );
        nodes[record_position]["body"]["members"][5]["value"]["members"][0]["members"][0]
            ["value"] = json!({"term":"reference","target":integer_parameter});
        sync_body_dependencies(&mut nodes[record_position]);
        let holds = path_condition[0]["holds"]
            .as_bool()
            .expect("RRV-01 outcome for RRV-02 base occurrence");
        (
            record_position,
            integer_parameter_position,
            holds,
            path_condition[0]["guard_read_occurrence"]
                .as_u64()
                .expect("RRV-02 guard read occurrence"),
        )
    };
    rekey_requirement_record(
        &mut adverse,
        record_position,
        integer_parameter_position,
        guard_read_occurrence,
        holds,
        None,
    );
    settle(&mut adverse);
    rebuild_complete_source_map(&mut adverse);
    let pointer = format!("/semantic_graph/nodes/{record_position}/body/members/5/value/members/0");
    assert_refusal(
        &adverse,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        &pointer,
    );
}

#[trace("TC-2053", "FR-825-AC-6")]
#[test]
fn public_reader_admits_a_reachable_boolean_literal_value_guard() {
    let proposal = super::proposal_dir_of(&super::fixtures_dir());
    let vectors = super::read_json(&proposal.join("requirement-record-vectors.json"))
        .expect("published FR-825 vectors");
    let path_condition = &vectors["positive"][0]["path_condition"];
    let package = boolean_literal_guard_package(path_condition);
    assert!(!package["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .any(|node| node["node_id"] == node_id("value/parameter/p/0")));
    let admitted = match read(&package) {
        CheckedPackageV2ReadResult::Admitted(admitted) => admitted,
        result => panic!("FR-825 item 6 admits a reachable Boolean literal guard: {result:?}"),
    };
    let records = admitted
        .requirement_items()
        .iter()
        .map(|row| row.record.as_ref().expect("value-validity record"))
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 1);
    let guard_id = &records[0].path_condition[0].guard;
    assert!(records
        .iter()
        .all(|record| &record.path_condition[0].guard == guard_id));
    let guard = admitted
        .resolved_graph()
        .nodes
        .iter()
        .find(|node| &node.node_id == guard_id)
        .expect("retained Boolean literal guard node");
    assert_eq!(guard.node_tag.as_ref(), "value");
    assert_eq!(guard.semantic_form.as_ref(), "literal");
    assert_eq!(records[0].path_condition[0].occurrence, 0);
    assert_eq!(records[0].extent_roots.len(), 1);
    let extent_root = admitted
        .resolved_graph()
        .nodes
        .iter()
        .find(|node| &node.node_id == &records[0].extent_roots[0])
        .expect("x parameter extent root");
    assert_eq!(extent_root.node_tag.as_ref(), "value");
    assert_eq!(extent_root.semantic_form.as_ref(), "parameter");
}

#[trace("TC-2053", "FR-825-AC-4")]
#[test]
fn guard_validation_precedes_malformed_holds_and_holds_remains_rejected_afterward() {
    let malformed_holds = |package: &mut Value, record_position: usize| {
        package["semantic_graph"]["nodes"][record_position]["body"]["members"][5]["value"]
            ["members"][0]["members"][2]["value"] = json!({
            "term":"literal",
            "type":node_id("scalar_type/text"),
            "value_kind":"text",
            "value":"not a boolean"
        });
        sync_body_dependencies(&mut package["semantic_graph"]["nodes"][record_position]);
    };
    let record_position = |package: &Value| {
        package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .position(|node| {
                node["semantic_form"] == "value_validity"
                    && node["body"]["members"][2]["value"]["value"] == 1
            })
            .expect("second value-validity record")
    };

    let mut wrong_type = value_validity_package(false);
    let wrong_record = record_position(&wrong_type);
    let integer_parameter = node_id("value/parameter/n/0");
    let integer_position = wrong_type["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == integer_parameter)
        .expect("integer parameter");
    wrong_type["semantic_graph"]["nodes"][wrong_record]["body"]["members"][5]["value"]["members"]
        [0]["members"][0]["value"] = json!({"term":"reference","target":integer_parameter});
    malformed_holds(&mut wrong_type, wrong_record);
    rekey_requirement_record(
        &mut wrong_type,
        wrong_record,
        integer_position,
        0,
        true,
        None,
    );
    settle(&mut wrong_type);
    rebuild_complete_source_map(&mut wrong_type);
    let guard_pointer =
        format!("/semantic_graph/nodes/{wrong_record}/body/members/5/value/members/0");
    assert_refusal(
        &wrong_type,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        &guard_pointer,
    );

    let mut bad_occurrence = value_validity_package(false);
    let bad_occurrence_record = record_position(&bad_occurrence);
    let expression_guard_position = bad_occurrence["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == node_id("expression/integer_ge"))
        .expect("expression guard");
    bad_occurrence["semantic_graph"]["nodes"][bad_occurrence_record]["body"]["members"][5]
        ["value"]["members"][0]["members"][1]["value"]["value"] = json!(99);
    malformed_holds(&mut bad_occurrence, bad_occurrence_record);
    rekey_requirement_record(
        &mut bad_occurrence,
        bad_occurrence_record,
        expression_guard_position,
        99,
        true,
        None,
    );
    settle(&mut bad_occurrence);
    rebuild_complete_source_map(&mut bad_occurrence);
    let occurrence_pointer =
        format!("/semantic_graph/nodes/{bad_occurrence_record}/body/members/5/value/members/0");
    assert_refusal(
        &bad_occurrence,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        &occurrence_pointer,
    );

    let mut malformed_holds_only = value_validity_package(false);
    let holds_record = record_position(&malformed_holds_only);
    malformed_holds(&mut malformed_holds_only, holds_record);
    settle(&mut malformed_holds_only);
    rebuild_complete_source_map(&mut malformed_holds_only);
    let holds_pointer =
        format!("/semantic_graph/nodes/{holds_record}/body/members/5/value/members/0");
    assert_refusal(
        &malformed_holds_only,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        &holds_pointer,
    );
}

fn assert_refusal(
    package: &Value,
    code: CheckedPackageRefusalCode,
    cause: Option<CheckedPackageRefusalCause>,
    pointer: &str,
) {
    let CheckedPackageV2ReadResult::Refused(refusal) = read(package) else {
        panic!("package mutation at {pointer} is refused without a partial package")
    };
    assert_eq!(refusal.code, code, "{pointer}: {refusal:?}");
    assert_eq!(refusal.cause, cause, "{pointer}: {refusal:?}");
    assert_eq!(
        refusal.loci[0].path,
        JsonPointer::parse(pointer),
        "{pointer}: {refusal:?}"
    );
}
