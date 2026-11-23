// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-488 public-reader coverage for the requirement-record package member.

use crate::checked_package_v2_parameters::parameters_package;
use crate::fixture_support::{
    canonical, evidence_for, index_fixture_node_ids, node_id, rebuild_body_locations,
    rebuild_complete_source_map, rebuild_source_map, settle, structural_fixture_id,
};
use filament_contract_model::{
    encode_checked_package_v2_with_path_condition_frames, CheckedPackageReadLimits,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedPackageSidecarsRef,
    CheckedPackageSourceCorrespondenceRef, CheckedPackageV2, CheckedPackageV2BudgetedReadResult,
    CheckedPackageV2ReadResult, JsonPointer, NativeJoinBudget, RequirementItemRow,
    RequirementRecord, ValueValidityRecord,
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

fn read_large_with_work_bound(
    mut package: Value,
    work_limit: u64,
) -> (CheckedPackageV2BudgetedReadResult, u64, std::time::Duration) {
    index_fixture_node_ids(&mut package);
    let bytes = canonical(&package);
    let evidence = evidence_for(&package);
    let nodes = u64::try_from(
        package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .len(),
    )
    .expect("node count fits u64");
    drop(package);
    let limits = CheckedPackageReadLimits {
        bytes: u64::try_from(bytes.len()).expect("byte count fits u64") + 1024,
        nodes: nodes + 16,
        edges: nodes * 16,
        occurrences: nodes * 4,
        diagnostics: 1,
        work: work_limit,
    };
    let mut budget = NativeJoinBudget::new(work_limit, u64::MAX).with_read_storage_limit(u64::MAX);
    let started = std::time::Instant::now();
    let result = CheckedPackageV2::read_with_budget(&bytes, limits, &evidence, &mut budget);
    (result, budget.consumed().work, started.elapsed())
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

fn canonical_nonnegative_integer(value: &Value) -> Option<u64> {
    match value {
        Value::Number(number) => number.as_u64(),
        Value::String(text) => text
            .parse::<u64>()
            .ok()
            .filter(|value| value.to_string() == *text),
        _ => None,
    }
}

fn binding_value<'a>(node: &'a Value, name: &str) -> &'a Value {
    node["body"]["members"]
        .as_array()
        .expect("record body bindings")
        .iter()
        .find(|binding| binding["name"] == name)
        .unwrap_or_else(|| panic!("missing {name} binding"))
        .get("value")
        .expect("binding value")
}

fn binding_value_mut<'a>(node: &'a mut Value, name: &str) -> &'a mut Value {
    node["body"]["members"]
        .as_array_mut()
        .expect("record body bindings")
        .iter_mut()
        .find(|binding| binding["name"] == name)
        .unwrap_or_else(|| panic!("missing {name} binding"))
        .get_mut("value")
        .expect("binding value")
}

fn value_validity_record(row: &RequirementItemRow) -> &ValueValidityRecord {
    match row.record.as_ref() {
        Some(RequirementRecord::ValueValidity(record)) => record,
        Some(RequirementRecord::OperationContract(_)) => {
            panic!("expected a value-validity requirement record")
        }
        None => panic!("requirement row has no record"),
    }
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

fn sort_graph_ids(ids: &mut Vec<Value>) {
    ids.sort_by(|left, right| {
        let left_package = left["package"].as_str().expect("dependency package");
        let right_package = right["package"].as_str().expect("dependency package");
        let left_root = left["path"][0].as_str().expect("dependency path root");
        let right_root = right["path"][0].as_str().expect("dependency path root");
        left_package
            .cmp(right_package)
            .then_with(|| left_root.cmp(right_root))
            .then_with(|| structural_fixture_id(left).cmp(&structural_fixture_id(right)))
    });
    ids.dedup();
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
    let mut root_dependencies = vec![
        node_id("expression/integer_ge"),
        node_id(app_key),
        boolean.clone(),
        integer.clone(),
    ];
    root_dependencies.sort_by_key(structural_fixture_id);
    let root = json!({
        "node_id":node_id("expression/conditional"),
        "schema_version":"filament.checked-semantic-graph/v2",
        "node_tag":"expression", "semantic_form":"conditional", "semantic_type":integer,
        "dependencies":root_dependencies,
        "occurrences":[{"role":"expression","ordinal":0}],
        "body":{"term":"application","operator":"conditional",
            "operation":{"identity":"filament.op.control.if","laws":[],"mode":null,"member":null,"leaves":[]},
            "result_type":integer,
            "arguments":[{"term":"reference","target":node_id("expression/integer_ge")},
                {"term":"reference","target":node_id(app_key)},
                {"term":"reference","target":node_id(app_key)}]}
    });
    nodes.push(parameter);
    nodes.push(application);
    nodes.push(guard);
    nodes.push(root);
    let parameter_position = nodes.len() - 4;
    let application_position = nodes.len() - 3;
    let guard_position = nodes.len() - 2;
    nodes[owner_position]["semantic_type"] = integer.clone();
    let mut owner_dependencies = vec![node_id(parameter_key), node_id("expression/conditional")];
    owner_dependencies.sort_by_key(structural_fixture_id);
    nodes[owner_position]["dependencies"] = json!(owner_dependencies);
    nodes[owner_position]["body"] = json!({"term":"aggregate","members":[
        {"term":"binding","name":"parameters","value":{"term":"aggregate","members":[
            {"term":"reference","target":node_id(parameter_key)}]}},
        {"term":"binding","name":"body","value":{"term":"reference","target":node_id("expression/conditional")}}
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
        let mut path = json!([
            "claim", "value_validity", {"node":owner_position}, {"node":application_position},
            occurrence.to_string(), "bound", {"node":bound_position},
            "conversion"
        ]);
        path.as_array_mut()
            .expect("record path")
            .extend([json!("path_condition"), json!(occurrence.to_string())]);
        path.as_array_mut()
            .expect("record path")
            .push(json!("roots"));
        path.as_array_mut()
            .expect("record path")
            .extend([json!({"node":parameter_position}), json!("objects")]);
        let record_id = json!({"package":package_name,"path":path,"ordinal":0});
        let mut record_dependencies = dependencies.clone();
        record_dependencies.push(guard.clone());
        record_dependencies.sort_by_key(structural_fixture_id);
        record_dependencies.dedup();
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
                binding("path_condition", json!({"term":"aggregate","members":[
                    {"term":"literal","type":node_id("scalar_type/integer"),"value_kind":"integer","value":occurrence}
                ]})),
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
    package["path_condition_frames"] = json!([
        {"parent":null,"guard":{"node":guard,"occurrence":{"role":"expression","ordinal":0}},"holds":true},
        {"parent":null,"guard":{"node":guard,"occurrence":{"role":"expression","ordinal":0}},"holds":false}
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

fn boolean_value_guard_package(spec: &Value) -> Value {
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
                        binding_value(node, "occurrence")["value"]
                            .as_u64()
                            .expect("application occurrence"),
                    )
                })
            })
            .collect::<Vec<_>>();
        (boolean_parameter_position, record_rows)
    };
    for &(record_position, application_occurrence) in &record_rows {
        let record_path = spec["record_path_condition"]
            .as_array()
            .expect("published record path conditions")
            .iter()
            .find(|row| row["application_occurrence"].as_u64() == Some(application_occurrence))
            .expect("record path for application occurrence");
        let frame_id = record_path["terminal_frame_id"]
            .as_u64()
            .expect("terminal frame id");
        let frame =
            &spec["path_condition_frames"][usize::try_from(frame_id).expect("frame id fits")];
        let guard_read_occurrence = frame["guard"]["occurrence"]["ordinal"]
            .as_u64()
            .expect("guard read occurrence");
        rekey_requirement_record(
            &mut package,
            record_position,
            boolean_parameter_position,
            guard_read_occurrence,
            frame["holds"].as_bool().expect("required guard outcome"),
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
        binding_value_mut(record, "extent_roots")["members"] = json!([
            {"term":"reference","target":node_id("value/parameter/n/0")},
            {"term":"reference","target":boolean_parameter_id.clone()}
        ]);
        binding_value_mut(record, "extent_roots")["members"]
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

fn large_requirement_record_package(record_count: usize) -> Value {
    assert!(
        record_count >= 2,
        "the chain fixture needs both branch records"
    );
    let generated_chain_count = record_count
        .checked_sub(1)
        .expect("the chain fixture includes one false-branch record");
    let proposal = super::proposal_dir_of(&super::fixtures_dir());
    let vectors = super::read_json(&proposal.join("requirement-record-vectors.json"))
        .expect("published FR-825 vectors");
    let spec = &vectors["positive"][0];
    let mut package = boolean_value_guard_package(spec);
    let record_template = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .find(|node| {
            node["semantic_form"] == "value_validity"
                && binding_value(node, "occurrence")["value"] == 0
        })
        .expect("true-branch record template")
        .clone();
    let original_application_id = binding_value(&record_template, "application")["target"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes")
        .retain(|node| node["semantic_form"] != "value_validity");
    package["requirement_items"] = json!([]);
    let package_name = package["name"].clone();
    let false_frame_id = package["path_condition_frames"]
        .as_array()
        .expect("path-condition frames")
        .iter()
        .position(|frame| frame["holds"] == false)
        .and_then(|position| u64::try_from(position).ok())
        .expect("false-branch frame id");

    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes");
    let owner_position = nodes
        .iter()
        .position(|node| node["node_tag"] == "function")
        .expect("function owner");
    let conditional_id = binding_value(&nodes[owner_position], "body")["target"].clone();
    let conditional_position = nodes
        .iter()
        .position(|node| node["node_id"] == conditional_id)
        .expect("conditional body root");
    let previous_position = nodes
        .iter()
        .position(|node| node["node_id"] == original_application_id)
        .expect("initial application");
    let mut previous_id = original_application_id.clone();
    assert_eq!(
        original_application_id["path"],
        json!(["expression/integer_add"])
    );
    assert!("expression" < "expression/integer_add");
    let bound_id = binding_value(&record_template, "result_bound")["target"].clone();
    let bound_position = nodes
        .iter()
        .position(|node| node["node_id"] == bound_id)
        .expect("record result bound");
    let extent_root_positions = binding_value(&record_template, "extent_roots")["members"]
        .as_array()
        .expect("extent roots")
        .iter()
        .map(|root| {
            nodes
                .iter()
                .position(|node| node["node_id"] == root["target"])
                .expect("extent root node")
        })
        .collect::<Vec<_>>();
    let application_template = nodes[previous_position].clone();
    nodes[previous_position]["occurrences"] = json!([
        {"role":"expression","ordinal":0},
        {"role":"expression","ordinal":1}
    ]);

    let mut application_positions = Vec::with_capacity(generated_chain_count);
    let mut application_ids = Vec::with_capacity(generated_chain_count);
    for generated_index in 0..generated_chain_count {
        let application_position = nodes.len();
        let application_id = json!({
            "package":package_name,
            "path":["expression","binary",format!("{generated_index:020}")],
            "ordinal":0
        });
        let mut application = application_template.clone();
        application["node_id"] = application_id.clone();
        application["occurrences"] = json!([{"role":"expression","ordinal":0}]);
        application["body"]["arguments"][0] = json!({"term":"reference","target":previous_id});
        let mut dependencies = vec![previous_id];
        sort_graph_ids(&mut dependencies);
        application["dependencies"] = Value::Array(dependencies);
        nodes.push(application);
        application_positions.push(application_position);
        application_ids.push(application_id.clone());
        previous_id = application_id;
    }

    let final_application_id = application_ids
        .last()
        .expect("at least one generated application")
        .clone();
    {
        let conditional = &mut nodes[conditional_position];
        conditional["body"]["arguments"][1] =
            json!({"term":"reference","target":final_application_id});
        conditional["body"]["arguments"][2] =
            json!({"term":"reference","target":original_application_id.clone()});
        let dependencies = conditional["dependencies"]
            .as_array_mut()
            .expect("conditional dependencies");
        if !dependencies.contains(&final_application_id) {
            dependencies.push(final_application_id.clone());
        }
        sort_graph_ids(
            conditional["dependencies"]
                .as_array_mut()
                .expect("conditional dependencies"),
        );
    }

    let terminal_frame_value =
        &binding_value(&record_template, "path_condition")["members"][0]["value"];
    let terminal_frame_id = canonical_nonnegative_integer(terminal_frame_value)
        .expect("terminal frame id is a canonical nonnegative integer");
    let mut requirement_rows = Vec::with_capacity(record_count);
    for (generated_index, (&application_position, application_id)) in application_positions
        .iter()
        .zip(&application_ids)
        .enumerate()
    {
        let record_id = json!({
            "package":package_name,
            "path":[
                "claim", "value_validity", {"node":owner_position}, {"node":application_position}, "0",
                "bound", {"node":bound_position}, "conversion", "path_condition",
                terminal_frame_id.to_string(),
                "roots",
                {"node":extent_root_positions[0]}, {"node":extent_root_positions[1]}, "objects"
            ],
            "ordinal":0
        });
        let mut record = record_template.clone();
        record["node_id"] = record_id.clone();
        record["occurrences"] = json!([{"role":"generated","ordinal":generated_index}]);
        binding_value_mut(&mut record, "application")["target"] = application_id.clone();
        for dependency in record["dependencies"]
            .as_array_mut()
            .expect("record dependencies")
        {
            if *dependency == original_application_id {
                *dependency = application_id.clone();
            }
        }
        sort_graph_ids(
            record["dependencies"]
                .as_array_mut()
                .expect("record dependencies"),
        );
        nodes.push(record);
        requirement_rows.push(json!({
            "claim":{"node":application_id,"occurrence":{"role":"expression","ordinal":0}},
            "kind":"value-validity",
            "extent":{"classification":"bounded"},
            "record":record_id
        }));
    }
    // The package carries both branch frames. Keep a distinct second
    // occurrence on the false branch so its record uses the false frame and
    // the admitted frame table has no orphan entry.
    let false_record_id = {
        let mut record = record_template.clone();
        let record_id = json!({
            "package":package_name,
            "path":[
                "claim", "value_validity", {"node":owner_position}, {"node":previous_position}, "1",
                "bound", {"node":bound_position}, "conversion", "path_condition",
                false_frame_id.to_string(), "roots",
                {"node":extent_root_positions[0]}, {"node":extent_root_positions[1]}, "objects"
            ],
            "ordinal":0
        });
        record["node_id"] = record_id.clone();
        record["occurrences"] = json!([{"role":"generated","ordinal":generated_chain_count}]);
        binding_value_mut(&mut record, "application")["target"] = original_application_id.clone();
        binding_value_mut(&mut record, "occurrence")["value"] = json!(1);
        binding_value_mut(&mut record, "path_condition")["members"][0]["value"] =
            json!(false_frame_id);
        nodes.push(record);
        record_id
    };
    requirement_rows.push(json!({
        "claim":{"node":original_application_id,"occurrence":{"role":"expression","ordinal":1}},
        "kind":"value-validity",
        "extent":{"classification":"bounded"},
        "record":false_record_id
    }));
    // Fixed-width numeric path segments make generated claim keys increase in
    // structural order. The generated path's first segment, "expression",
    // sorts before the seed's single "expression/integer_add" segment, so
    // appending rows in generation order is canonical.
    package["requirement_items"] = Value::Array(requirement_rows);
    super::fixture_support::rebuild_complete_source_map(&mut package);
    assert!(
        package["source_map"]
            .as_array()
            .expect("complete source map")
            .iter()
            .any(|entry| {
                entry["node_id"] == original_application_id
                    && entry["role"] == "expression"
                    && entry["ordinal"] == 1
                    && entry["regions"]
                        .as_array()
                        .is_some_and(|regions| !regions.is_empty())
            }),
        "the false-branch claim occurrence has a source region"
    );
    package
}

#[trace("TC-488", "FR-376-AC-3")]
#[ignore = "large reader performance lane; run explicitly on Titan"]
#[test]
fn tc_488_a_hundred_thousand_requirement_records_admit_within_fifteen_minutes() {
    const RECORDS: usize = 100_000;
    const WORK_PER_RECORD: u64 = 256;
    const READ_TIME_LIMIT: std::time::Duration = std::time::Duration::from_secs(15 * 60);

    let package = large_requirement_record_package(RECORDS);
    let work_limit = u64::try_from(RECORDS).expect("record count fits u64") * WORK_PER_RECORD;
    let (result, work, elapsed) = read_large_with_work_bound(package, work_limit);
    let CheckedPackageV2BudgetedReadResult::Admitted(admitted) = result else {
        panic!("100,000 reached requirement records admit within the bounded read: {result:?}");
    };
    assert_eq!(admitted.requirement_items().len(), RECORDS);
    assert!(
        work <= work_limit,
        "reader work {work} exceeded {work_limit}"
    );
    assert!(
        elapsed < READ_TIME_LIMIT,
        "100,000 requirement records took {elapsed:?}, limit {READ_TIME_LIMIT:?}"
    );
}

fn boolean_literal_guard_package(spec: &Value) -> Value {
    let mut package = boolean_value_guard_package(spec);
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
                && binding_value(node, "occurrence")["value"] == 1
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
    package["path_condition_frames"]
        .as_array_mut()
        .expect("path-condition frames")
        .truncate(1);

    let record_path = spec["record_path_condition"]
        .as_array()
        .expect("published record path conditions")
        .iter()
        .find(|row| row["application_occurrence"].as_u64() == Some(0))
        .expect("true-branch record path");
    let frame_id = record_path["terminal_frame_id"]
        .as_u64()
        .expect("true-branch terminal frame id");
    let frame = &spec["path_condition_frames"][usize::try_from(frame_id).expect("frame id")];
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
    let guard_read_occurrence = frame["guard"]["occurrence"]["ordinal"]
        .as_u64()
        .expect("guard read occurrence");
    let holds = frame["holds"].as_bool().expect("required guard outcome");
    rekey_requirement_record(
        &mut package,
        record_position,
        literal_position,
        guard_read_occurrence,
        holds,
        Some((parameter_position, parameter_id)),
    );
    settle(&mut package);
    let final_literal_id = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("settled graph nodes")
        .iter()
        .find(|node| {
            node["semantic_form"] == "literal"
                && node["node_tag"] == "value"
                && node["semantic_type"] == node_id("scalar_type/boolean")
        })
        .expect("settled Boolean literal")["node_id"]
        .clone();
    package["path_condition_frames"][0]["guard"]["node"] = final_literal_id;
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
    let guard = package["semantic_graph"]["nodes"][guard_position]["node_id"].clone();
    let (old_id, new_id) = {
        let record = &mut package["semantic_graph"]["nodes"][record_position];
        let old_id = record["node_id"].clone();
        {
            let path = record["node_id"]["path"]
                .as_array_mut()
                .expect("record structural path");
            let conversion = path
                .iter()
                .position(|segment| segment == "conversion")
                .expect("conversion path segment");
            let roots = path
                .iter()
                .enumerate()
                .skip(conversion + 1)
                .find_map(|(index, segment)| (segment == "roots").then_some(index))
                .expect("roots path segment");
            path.splice(
                conversion + 1..roots,
                [
                    json!("path_condition"),
                    json!(if holds { "0" } else { "1" }),
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
            if let Some(roots) = binding_value_mut(record, "extent_roots")["members"].as_array_mut()
            {
                roots.retain(|root| root["target"] != *root_id);
            }
            sync_body_dependencies(record);
        }
        let frame_id = if holds { 0 } else { 1 };
        binding_value_mut(record, "path_condition")["members"] = json!([
            {"term":"literal","type":node_id("scalar_type/integer"),"value_kind":"integer","value":frame_id}
        ]);
        (old_id, record["node_id"].clone())
    };
    let frame_id = if holds { 0 } else { 1 };
    package["path_condition_frames"][frame_id] = json!({
        "parent":null,
        "guard":{"node":guard,"occurrence":{"role":"expression","ordinal":guard_read_occurrence}},
        "holds":holds
    });
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
    package["requirement_items"] =
        json!([row(&claim, "finite-replay"), row(&claim, "monitorability"),]);

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
        filament_contract_model::CapabilityKind::Monitorability
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
    let record = value_validity_record(&rows[0]);
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
    assert_eq!(record.path_condition_frame.map(|frame| frame.0), Some(0));
    assert_eq!(record.extent_roots.len(), 1);
    assert_eq!(&record.extent_roots[0], resolved_parameter);
    assert!(record.objects.is_empty());
    assert_eq!(value_validity_record(&rows[1]).occurrence, 1);
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
        let reordered_record = value_validity_record(reordered_row);
        let record = value_validity_record(row);
        assert_eq!(reordered_record.owner, record.owner);
        assert_eq!(reordered_record.application, record.application);
        assert_eq!(reordered_record.occurrence, record.occurrence);
        assert_eq!(reordered_record.result_bound, record.result_bound);
        assert_eq!(reordered_record.conversion, record.conversion);
        assert_eq!(
            reordered_record.path_condition_frame,
            record.path_condition_frame
        );
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
    let bytes = encode_checked_package_v2_with_path_condition_frames(
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
        None,
        Some(admitted.path_condition_frames()),
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
    package["requirement_items"] =
        json!([row(&claim, "monitorability"), row(&claim, "finite-replay"),]);

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

/// FR-376/FR-322 strict-wire and owner checks precede the FR-825 record rule.
/// The extra `declaration` member is refused as `malformed_wire` at the node,
/// with no cause, so FR-825 does not run for this input.
#[trace("TC-488", "FR-376-AC-2")]
#[test]
fn public_reader_refuses_a_record_node_with_an_extra_declaration_member_as_malformed_wire() {
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
        CheckedPackageRefusalCode::MalformedWire,
        None,
        &format!("/semantic_graph/nodes/{record_position}"),
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
    package["requirement_items"] =
        json!([row(&claim, "monitorability"), row(&claim, "finite-replay"),]);
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
fn published_requirement_record_vectors_admit_shared_frames_and_refuse_non_boolean_guards() {
    let proposal = super::proposal_dir_of(&super::fixtures_dir());
    let vectors = super::read_json(&proposal.join("requirement-record-vectors.json"))
        .expect("published FR-825 vectors");
    let positive = vectors["positive"]
        .as_array()
        .expect("positive requirement-record vectors")
        .iter()
        .find(|vector| vector["id"] == "RRV-01-bare-boolean-parameter-guard")
        .expect("RRV-01");
    assert_eq!(positive["expected"]["record_count"], 2);
    assert_eq!(
        positive["path_condition_frames"].as_array().unwrap().len(),
        2
    );
    let package = boolean_value_guard_package(positive);
    let CheckedPackageV2ReadResult::Admitted(admitted) = read(&package) else {
        panic!(
            "RRV-01 admits both terminal frame ids: {:?}",
            read(&package)
        );
    };
    let records = admitted
        .requirement_items()
        .iter()
        .map(value_validity_record)
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 2);
    assert_eq!(
        records[0].path_condition_frame.map(|frame| frame.0),
        Some(0)
    );
    assert_eq!(
        records[1].path_condition_frame.map(|frame| frame.0),
        Some(1)
    );
    for (record, expected_holds) in records.iter().zip([true, false]) {
        let chain = admitted.path_condition(record).iter().collect::<Vec<_>>();
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0].holds, expected_holds);
        assert_eq!(chain[0].guard, &admitted.path_condition_frames()[0].guard);
    }

    let mut adverse = package;
    adverse["path_condition_frames"][0]["guard"]["node"] = node_id("value/parameter/n/0");
    assert_refusal(
        &adverse,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        "/path_condition_frames/0/guard",
    );
}

#[trace("TC-2053", "FR-825-AC-6")]
#[test]
fn public_reader_admits_a_reachable_boolean_literal_value_guard() {
    let proposal = super::proposal_dir_of(&super::fixtures_dir());
    let vectors = super::read_json(&proposal.join("requirement-record-vectors.json"))
        .expect("published FR-825 vectors");
    let spec = &vectors["positive"][0];
    let package = boolean_literal_guard_package(spec);
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
        .map(value_validity_record)
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 1);
    let guards = admitted
        .path_condition(records[0])
        .iter()
        .collect::<Vec<_>>();
    let guard_id = guards[0].guard;
    assert!(records.iter().all(|record| admitted
        .path_condition(record)
        .iter()
        .all(|guard| guard.guard == guard_id)));
    let guard = admitted
        .resolved_graph()
        .nodes
        .iter()
        .find(|node| &node.node_id == guard_id)
        .expect("retained Boolean literal guard node");
    assert_eq!(guard.node_tag.as_ref(), "value");
    assert_eq!(guard.semantic_form.as_ref(), "literal");
    assert_eq!(guards[0].occurrence, 0);
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

#[trace("TC-2053", "FR-825-AC-8")]
#[test]
fn path_frame_wire_rejects_non_boolean_holds() {
    let mut package = value_validity_package(false);
    package["path_condition_frames"][0]["holds"] = json!("true");
    assert_refusal(
        &package,
        CheckedPackageRefusalCode::MalformedWire,
        None,
        "/path_condition_frames/0/holds",
    );
}

#[trace("TC-2053", "FR-825-AC-8", "FR-825-AC-9")]
#[test]
fn path_frame_reader_rejects_bad_links_or_record_bindings() {
    let mut self_parent = value_validity_package(false);
    self_parent["path_condition_frames"][0]["parent"] = json!(0);
    assert_refusal(
        &self_parent,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        "/path_condition_frames/0/parent",
    );

    let mut duplicate = value_validity_package(false);
    duplicate["path_condition_frames"][1] = duplicate["path_condition_frames"][0].clone();
    assert_refusal(
        &duplicate,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::ConflictingDefinition),
        "/path_condition_frames/1",
    );

    let mut orphan = value_validity_package(false);
    let mut orphan_frame = orphan["path_condition_frames"][0].clone();
    orphan_frame["parent"] = json!(0);
    orphan["path_condition_frames"]
        .as_array_mut()
        .expect("frame table")
        .push(orphan_frame);
    assert_refusal(
        &orphan,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        "/path_condition_frames/2",
    );

    let mut body_disagrees = value_validity_package(false);
    let nodes = body_disagrees["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes");
    let record_position = nodes
        .iter()
        .position(|node| {
            node["semantic_form"] == "value_validity"
                && node["body"]["members"][2]["value"]["value"] == 0
        })
        .expect("first value-validity record");
    nodes[record_position]["body"]["members"][5]["value"]["members"][0]["value"] = json!(1);
    let body_pointer = format!("/semantic_graph/nodes/{record_position}/body/members/5");
    assert_refusal(
        &body_disagrees,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        &body_pointer,
    );

    let mut identity_disagrees = value_validity_package(false);
    let nodes = identity_disagrees["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes");
    let record_position = nodes
        .iter()
        .position(|node| {
            node["semantic_form"] == "value_validity"
                && node["body"]["members"][2]["value"]["value"] == 0
        })
        .expect("first value-validity record");
    let original_record_id = nodes[record_position]["node_id"].clone();
    let path = nodes[record_position]["node_id"]["path"]
        .as_array_mut()
        .expect("record identity path");
    let frame_marker = path
        .iter()
        .position(|segment| segment == "path_condition")
        .expect("path-condition identity marker");
    path[frame_marker + 1] = json!("9");
    let mismatched_record_id = nodes[record_position]["node_id"].clone();
    for row in identity_disagrees["source_map"]
        .as_array_mut()
        .expect("source map")
    {
        if row["node_id"] == original_record_id {
            row["node_id"] = mismatched_record_id.clone();
        }
    }
    for item in identity_disagrees["requirement_items"]
        .as_array_mut()
        .expect("requirement items")
    {
        if item["record"] == original_record_id {
            item["record"] = mismatched_record_id.clone();
        }
    }
    let identity_pointer = format!("/semantic_graph/nodes/{record_position}/node_id");
    assert_refusal(
        &identity_disagrees,
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        None,
        &identity_pointer,
    );
}

/// `value_validity_package` with its else branch changed to a bare literal, so
/// that branch holds no scalar operation: the emitter prunes its frame
/// (FR-825), leaving the then frame as the table's only row and one record.
fn else_without_scalar_operation_package() -> Value {
    let mut package = value_validity_package(false);
    let integer = node_id("scalar_type/integer");
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes");
    let conditional = nodes
        .iter()
        .position(|node| node["node_id"] == node_id("expression/conditional"))
        .expect("conditional body root");
    nodes[conditional]["body"]["arguments"][2] =
        json!({"term":"literal","type":integer,"value_kind":"integer","value":"0"});
    let application = nodes
        .iter()
        .position(|node| node["node_id"] == node_id("expression/integer_add"))
        .expect("application");
    nodes[application]["occurrences"] = json!([{"role":"expression","ordinal":0}]);
    let discarded = nodes
        .iter()
        .position(|node| {
            node["semantic_form"] == "value_validity"
                && binding_value(node, "occurrence")["value"] == 1
        })
        .expect("else-branch record");
    let discarded_id = nodes.remove(discarded)["node_id"].clone();
    package["requirement_items"]
        .as_array_mut()
        .expect("requirement rows")
        .retain(|row| row["record"] != discarded_id);
    package["path_condition_frames"]
        .as_array_mut()
        .expect("frame table")
        .truncate(1);
    settle(&mut package);
    rebuild_complete_source_map(&mut package);
    package
}

/// One value-validity record of `owner` at `application`'s `occurrence`, cited
/// at `frame`, the `generated_ordinal`th record of its owner.
fn push_record(
    package: &mut Value,
    owner: &Value,
    application: &Value,
    occurrence: u64,
    frame: Option<u64>,
    generated_ordinal: u64,
) -> Value {
    let package_name = package["name"].clone();
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes");
    let position = |id: &Value| {
        nodes
            .iter()
            .position(|node| node["node_id"] == *id)
            .expect("fixture node")
    };
    let owner_position = position(owner);
    let application_position = position(application);
    let bound = node_id("scalar_type/integer");
    let bound_position = position(&bound);
    let parameter = node_id("value/parameter/n/0");
    let parameter_position = position(&parameter);
    let mut path = json!([
        "claim", "value_validity", {"node":owner_position}, {"node":application_position},
        occurrence.to_string(), "bound", {"node":bound_position}, "conversion"
    ]);
    let path_condition = match frame {
        Some(frame) => {
            path.as_array_mut()
                .expect("record path")
                .extend([json!("path_condition"), json!(frame.to_string())]);
            json!([{"term":"literal","type":bound,"value_kind":"integer","value":frame}])
        }
        None => json!([]),
    };
    path.as_array_mut()
        .expect("record path")
        .extend([json!("roots"), json!({"node":parameter_position}), json!("objects")]);
    let record_id = json!({"package":package_name,"path":path,"ordinal":0});
    let mut dependencies = vec![
        owner.clone(),
        parameter.clone(),
        application.clone(),
        bound.clone(),
    ];
    if frame.is_some() {
        dependencies.push(node_id("expression/integer_ge"));
    }
    dependencies.sort_by_key(structural_fixture_id);
    dependencies.dedup();
    let binding = |name: &str, value: Value| json!({"term":"binding","name":name,"value":value});
    nodes.push(json!({
        "node_id":record_id,
        "schema_version":"filament.checked-semantic-graph/v2",
        "node_tag":"claim", "semantic_form":"value_validity", "semantic_type":node_id("scalar_type/boolean"),
        "dependencies":dependencies,
        "occurrences":[{"role":"generated","ordinal":generated_ordinal}],
        "body":{"term":"aggregate","members":[
            binding("owner",json!({"term":"reference","target":owner})),
            binding("application",json!({"term":"reference","target":application})),
            binding("occurrence",json!({"term":"literal","type":bound,"value_kind":"integer","value":occurrence})),
            binding("result_bound",json!({"term":"reference","target":bound})),
            binding("conversion",json!({"term":"aggregate","members":[]})),
            binding("path_condition",json!({"term":"aggregate","members":path_condition})),
            binding("extent_roots",json!({"term":"aggregate","members":[{"term":"reference","target":parameter}]})),
            binding("objects",json!({"term":"aggregate","members":[]})),
        ]}
    }));
    record_id
}

/// `value_validity_package` plus a second function `h` declared after it whose
/// body is the node `shared`, so the nodes `h` reads take expression
/// occurrences after the ones `both`'s body took (FR-825 package-wide
/// numbering). Returns the package, `h`'s node id and the shared node's count
/// of occurrences before `h`.
fn second_owner_package(shared: &str, extra_occurrences: u64) -> (Value, Value) {
    let mut package = value_validity_package(false);
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes");
    let root = node_id(shared);
    let mut owner = nodes
        .iter()
        .find(|node| node["node_tag"] == "function")
        .expect("function owner")
        .clone();
    owner["node_id"] = node_id("qualified:h");
    owner["declaration"] = json!({"qualified_name":["h"]});
    owner["dependencies"] = json!([root]);
    owner["body"] = json!({"term":"aggregate","members":[
        {"term":"binding","name":"parameters","value":{"term":"aggregate","members":[]}},
        {"term":"binding","name":"body","value":{"term":"reference","target":root}}
    ]});
    let owner_id = owner["node_id"].clone();
    nodes.push(owner);
    for key in ["expression/conditional", "expression/integer_ge", "expression/integer_add"] {
        let node = nodes
            .iter_mut()
            .find(|node| node["node_id"] == node_id(key))
            .expect("shared node");
        let occurrences = node["occurrences"].as_array_mut().expect("occurrences");
        let used = u64::try_from(occurrences.len()).expect("count");
        let added = match key {
            "expression/integer_add" => extra_occurrences,
            _ if shared == "expression/conditional" => 1,
            _ => 0,
        };
        for ordinal in used..used + added {
            occurrences.push(json!({"role":"expression","ordinal":ordinal}));
        }
    }
    (package, owner_id)
}

fn finish_second_owner(package: &mut Value, new_records: &[(Value, u64)], frames: &[Value]) {
    settle(package);
    rebuild_complete_source_map(package);
    let application = node_id("expression/integer_add");
    let items = package["requirement_items"]
        .as_array_mut()
        .expect("requirement rows");
    for (record, occurrence) in new_records {
        items.push(json!({"claim":{"node":application,"occurrence":{"role":"expression","ordinal":occurrence}},
            "kind":"value-validity","extent":{"classification":"bounded"},"record":record}));
    }
    package["path_condition_frames"]
        .as_array_mut()
        .expect("frame table")
        .extend(frames.iter().cloned());
}

#[trace("TC-2053", "FR-825-AC-6")]
#[test]
fn reader_admits_an_if_branch_that_holds_no_scalar_operation() {
    let package = else_without_scalar_operation_package();
    let CheckedPackageV2ReadResult::Admitted(admitted) = read(&package) else {
        panic!("an else branch with no scalar operation: {:?}", read(&package));
    };
    let records = admitted
        .requirement_items()
        .iter()
        .map(value_validity_record)
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].occurrence, 0);
    assert_eq!(
        records[0].path_condition_frame.map(|frame| frame.0),
        Some(0)
    );
    let chain = admitted.path_condition(records[0]).iter().collect::<Vec<_>>();
    assert_eq!(chain.len(), 1);
    assert!(chain[0].holds);
    assert_eq!(admitted.path_condition_frames().len(), 1);
}

#[trace("TC-2053", "FR-825-AC-8", "FR-825-AC-9")]
#[test]
fn reader_refuses_a_record_in_an_if_branch_whose_frame_is_absent() {
    // The else branch still applies `add`, so a record at occurrence 1 needs
    // the else frame; without the row a record that names none disagrees with
    // the derived chain at its own `path_condition` binding.
    let mut package = value_validity_package(false);
    package["path_condition_frames"]
        .as_array_mut()
        .expect("frame table")
        .truncate(1);
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes");
    let record = nodes
        .iter()
        .position(|node| {
            node["semantic_form"] == "value_validity"
                && binding_value(node, "occurrence")["value"] == 1
        })
        .expect("else-branch record");
    binding_value_mut(&mut nodes[record], "path_condition")["members"] = json!([]);
    let pointer = format!("/semantic_graph/nodes/{record}/body/members/5");
    assert_refusal(
        &package,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        &pointer,
    );
}

#[trace("TC-2053", "FR-825-AC-6", "FR-825-AC-9")]
#[test]
fn record_occurrence_of_a_node_shared_by_two_owners_is_numbered_package_wide() {
    // `both` reads `add` twice (occurrences 0, 1); `h` is declared after it
    // and reads the same node, which is occurrence 2.
    let (mut package, owner) = second_owner_package("expression/integer_add", 1);
    let application = node_id("expression/integer_add");
    let second = push_record(&mut package, &owner, &application, 2, None, 0);
    finish_second_owner(&mut package, &[(second, 2)], &[]);
    let CheckedPackageV2ReadResult::Admitted(admitted) = read(&package) else {
        panic!("a node shared by two owners: {:?}", read(&package));
    };
    let records = admitted
        .requirement_items()
        .iter()
        .map(value_validity_record)
        .collect::<Vec<_>>();
    assert_eq!(
        records
            .iter()
            .map(|record| (record.occurrence, record.path_condition_frame.map(|f| f.0)))
            .collect::<Vec<_>>(),
        vec![(0, Some(0)), (1, Some(1)), (2, None)]
    );

    // The second owner's read is occurrence 2, never 0: a record that names
    // occurrence 0 for `h` is a record `both` already owns.
    let mut local_numbering = package;
    let nodes = local_numbering["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes");
    let record = nodes
        .iter()
        .position(|node| {
            node["semantic_form"] == "value_validity"
                && node["body"]["members"][0]["value"]["target"] == owner
        })
        .expect("record of the second owner");
    binding_value_mut(&mut nodes[record], "occurrence")["value"] = json!(1);
    let pointer = format!("/semantic_graph/nodes/{record}/body/members/5");
    assert_refusal(
        &local_numbering,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        &pointer,
    );
}

#[trace("TC-2053", "FR-825-AC-6", "FR-825-AC-9")]
#[test]
fn guard_occurrence_of_a_node_shared_by_two_owners_is_numbered_package_wide() {
    // `h`'s body is the conditional `both` already reads: its guard is read a
    // second time (occurrence 1) and `add` twice more (occurrences 2, 3).
    let (mut package, owner) = second_owner_package("expression/conditional", 2);
    let application = node_id("expression/integer_add");
    let then_record = push_record(&mut package, &owner, &application, 2, Some(2), 0);
    let else_record = push_record(&mut package, &owner, &application, 3, Some(3), 1);
    let guard = node_id("expression/integer_ge");
    let frame = |holds: bool| {
        json!({"parent":null,"guard":{"node":guard,"occurrence":{"role":"expression","ordinal":1}},
            "holds":holds})
    };
    finish_second_owner(
        &mut package,
        &[(then_record, 2), (else_record, 3)],
        &[frame(true), frame(false)],
    );
    let CheckedPackageV2ReadResult::Admitted(admitted) = read(&package) else {
        panic!("a guard shared by two owners: {:?}", read(&package));
    };
    let frames = admitted.path_condition_frames();
    assert_eq!(frames.len(), 4);
    assert_eq!(
        frames
            .iter()
            .map(|frame| (frame.occurrence, frame.holds))
            .collect::<Vec<_>>(),
        vec![(0, true), (0, false), (1, true), (1, false)]
    );

    // A record of `h` that cites the other branch's frame disagrees with the
    // chain derived for it.
    let mut swapped = package;
    let nodes = swapped["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes");
    let record = nodes
        .iter()
        .position(|node| {
            node["semantic_form"] == "value_validity"
                && binding_value(node, "occurrence")["value"] == 2
        })
        .expect("then-branch record of the second owner");
    binding_value_mut(&mut nodes[record], "path_condition")["members"][0]["value"] = json!(3);
    let pointer = format!("/semantic_graph/nodes/{record}/body/members/5");
    assert_refusal(
        &swapped,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        &pointer,
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

#[cfg(test)]
mod order;
