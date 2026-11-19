// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-488 public-reader coverage for the requirement-record package member.

use crate::checked_package_v2_parameters::parameters_package;
use crate::fixture_support::{
    canonical, evidence_for, index_fixture_node_ids, node_id, rebuild_complete_source_map,
    rebuild_source_map, settle, structural_fixture_id,
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
                    json!({"term":"aggregate","members":[binding("guard",json!({"term":"aggregate","members":[
                        binding("guard",json!({"term":"reference","target":guard})),
                        binding("occurrence",json!({"term":"literal","type":node_id("scalar_type/integer"),"value_kind":"integer","value":0})),
                        binding("holds",json!({"term":"literal","type":node_id("scalar_type/boolean"),"value_kind":"boolean","value":true}))
                    ]}))]})
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
    let package = value_validity_package(false);
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
            "public reader admits the bounded value-validity fixture: {:?}",
            read(&package)
        )
    };
    let rows = admitted.requirement_items();
    assert_eq!(rows.len(), 2);
    let record = rows[0]
        .record
        .as_ref()
        .expect("typed row retains its value-validity record");
    assert_eq!(record.owner, owner);
    assert_eq!(record.occurrence, 0);
    assert!(record.conversion.is_none());
    assert!(record.path_condition.is_empty());
    assert_eq!(record.extent_roots.len(), 1);
    assert!(record.objects.is_empty());
    assert_eq!(
        rows[1].record.as_ref().expect("second record").occurrence,
        1
    );
    assert_eq!(admitted.value_validity_records_of(&owner), &[0, 1]);

    let reordered = value_validity_package(true);
    let CheckedPackageV2ReadResult::Admitted(reordered) = read(&reordered) else {
        panic!("public reader admits the same nodes in another graph order")
    };
    assert_eq!(reordered.requirement_items(), admitted.requirement_items());
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
    let CheckedPackageV2ReadResult::Admitted(round_trip) = read(&encoded) else {
        panic!("writer output is admitted by the public reader")
    };
    assert_eq!(round_trip.requirement_items(), admitted.requirement_items());
}

#[trace("TC-488", "FR-376-AC-3")]
#[test]
fn public_reader_admits_at_the_exact_work_budget_and_reports_the_last_row_one_short() {
    let package = value_validity_package(false);
    let mut wire = package.clone();
    index_fixture_node_ids(&mut wire);
    let bytes = canonical(&wire);
    let evidence = evidence_for(&wire);
    let mut ample = NativeJoinBudget::new(u64::MAX, u64::MAX);
    let result = CheckedPackageV2::read_with_budget(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &evidence,
        &mut ample,
    );
    assert!(matches!(
        result,
        CheckedPackageV2BudgetedReadResult::Admitted(_)
    ));
    let required = ample.consumed().work;

    let mut exact = NativeJoinBudget::new(required, u64::MAX);
    assert!(matches!(
        CheckedPackageV2::read_with_budget(
            &bytes,
            CheckedPackageReadLimits::bounded(),
            &evidence,
            &mut exact,
        ),
        CheckedPackageV2BudgetedReadResult::Admitted(_)
    ));

    let mut short = NativeJoinBudget::new(required - 1, u64::MAX);
    let CheckedPackageV2BudgetedReadResult::BudgetIncomplete(incomplete) =
        CheckedPackageV2::read_with_budget(
            &bytes,
            CheckedPackageReadLimits::bounded(),
            &evidence,
            &mut short,
        )
    else {
        panic!("one work visit short returns a budget incomplete result")
    };
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
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::UnknownMember)
    );
    assert_eq!(
        refusal.loci[0].path,
        JsonPointer::parse("/requirement_items/0")
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
            "declaration member",
            Box::new(|record| record["declaration"] = json!({"qualified_name":["bad"]})),
            "/occurrences",
        ),
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

    let mut wrong_occurrence = value_validity_package(false);
    wrong_occurrence["requirement_items"][0]["claim"]["occurrence"]["ordinal"] = json!(7);
    assert_refusal(
        &wrong_occurrence,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        "/requirement_items/0/claim",
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

    assert_refusal(
        &package,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        "/claim_names/0/declaration",
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
