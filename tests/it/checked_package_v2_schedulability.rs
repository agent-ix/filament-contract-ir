// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Independent local task claims through the public authenticated reader.

use crate::support::checked_package as fixtures;
use filament_contract_model::{
    encode_checked_package_v2, CheckedLiteralRead, CheckedLiteralValue, CheckedNodeId,
    CheckedPackageReadLimits, CheckedPackageSidecarsRef, CheckedPackageSourceCorrespondenceRef,
    CheckedPackageV2, CheckedPackageV2ReadResult, CheckedSchedulabilityClaimRow,
    CheckedSchedulabilityKind, CheckedSchedulingPolicy, CheckedTaskArrival, CheckedTaskCriticality,
};
use ix_trace_rs::trace;
use serde_json::{json, Value};

fn aggregate(members: Vec<Value>) -> Value {
    json!({"term":"aggregate", "members":members})
}
fn binding(name: &str, value: Value) -> Value {
    json!({"term":"binding", "name":name, "value":value})
}
fn literal(type_id: &Value, kind: &str, value: &str) -> Value {
    json!({"term":"literal", "type":type_id, "value_kind":kind, "value":value})
}

pub(super) fn package(policy: &str, response: bool, high: bool, occurrences: u64) -> Value {
    let mut package = fixtures::v2_all_families();
    let find = |form: &str| {
        package["semantic_graph"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|node| node["semantic_form"] == form)
            .unwrap()["node_id"]
            .clone()
    };
    let boolean = find("boolean");
    let text = find("text");
    let integer = find("integer");
    let mut nominal = fixtures::v2_nominal();
    let old = fixtures::node_id("qualified:Example/Length");
    let time = fixtures::node_id("qualified:Example/Time");
    fixtures::reassign_node_id(&mut nominal, &old, &time);
    for node in nominal["semantic_graph"]["nodes"].as_array_mut().unwrap() {
        if node["semantic_form"] == "dimension" {
            node["declaration"]["qualified_name"] = json!(["Example", "Time"]);
        }
    }
    let unit = nominal["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["semantic_form"] == "unit")
        .unwrap()["node_id"]
        .clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .extend(
            nominal["semantic_graph"]["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|node| matches!(node["semantic_form"].as_str(), Some("unit" | "dimension")))
                .cloned(),
        );
    let word = |s: &str| literal(&text, "text", s);
    let quantity = |s: &str| literal(&unit, "quantity", s);
    let body = aggregate(vec![
        binding(
            "kind",
            word(if response { "response" } else { "schedulable" }),
        ),
        binding("policy", word(policy)),
        binding(
            "task",
            if response {
                word("worker")
            } else {
                aggregate(vec![])
            },
        ),
        binding(
            "bound",
            if response {
                quantity("17/2")
            } else {
                aggregate(vec![])
            },
        ),
        binding(
            "tasks",
            aggregate(vec![aggregate(vec![
                binding("name", word("worker")),
                binding("arrival", word("sporadic")),
                binding("period", quantity("19/2")),
                binding("deadline", quantity("17/2")),
                binding("bcet", quantity("3/2")),
                binding("wcet", quantity("5/2")),
                binding(
                    "wcet_hi",
                    if high {
                        quantity("7/2")
                    } else {
                        aggregate(vec![])
                    },
                ),
                binding("criticality", word(if high { "hi" } else { "lo" })),
                binding(
                    "priority",
                    if policy == "edf" {
                        aggregate(vec![])
                    } else {
                        literal(&integer, "integer", "7")
                    },
                ),
                binding("jitter", quantity("0/1")),
                binding("blocking", quantity("0/1")),
            ])]),
        ),
    ]);
    let id = fixtures::node_id("fixture/schedulability/worker");
    package["semantic_graph"]["nodes"].as_array_mut().unwrap().push(json!({
        "node_id":id, "schema_version":"filament.checked-semantic-graph/v2",
        "node_tag":"claim", "semantic_form":"schedulability", "semantic_type":boolean,
        "dependencies":[], "occurrences":(0..occurrences).map(|ordinal| json!({"role":"claim","ordinal":ordinal})).collect::<Vec<_>>(),
        "body":body
    }));
    package["schedulability_claims"] = json!((0..occurrences)
        .map(|occurrence| json!({"declaration":id,"occurrence":occurrence,"task_set":"Ctl","release_bindings":[],"mode_switch_bindings":[]}))
        .collect::<Vec<_>>());
    fixtures::rebuild_complete_source_map(&mut package);
    fixtures::index_fixture_node_ids(&mut package);
    package
}

fn package_with_release_binding(release_backed: bool) -> Value {
    let mut package: Value = serde_json::from_str(include_str!(
        "../fixtures/qsl-2347-emitted/clock-place.json"
    ))
    .expect("current timed-model fixture");
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let id = |path: &[&str]| json!({"package":"pkg","path":path,"ordinal":0});
    let claim_id = id(&["claim", "schedulability", "Ctl"]);
    let boolean = id(&["scalar_type", "boolean"]);
    let text = nodes
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "text")
        .expect("text type")["node_id"]
        .clone();
    let unit = nodes
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "unit")
        .expect("time unit")["node_id"]
        .clone();
    let integer = nodes
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "integer")
        .expect("integer type")["node_id"]
        .clone();
    let task = json!({"term":"aggregate","members":[
        binding("name", literal(&text,"text","worker")),
        binding("arrival", literal(&text,"text","periodic")),
        binding("period", literal(&unit,"quantity","4/1")),
        binding("deadline", literal(&unit,"quantity","4/1")),
        binding("bcet", literal(&unit,"quantity","0/1")),
        binding("wcet", literal(&unit,"quantity","1/1")),
        binding("wcet_hi", aggregate(vec![])),
        binding("criticality", literal(&text,"text","lo")),
        binding("priority", literal(&integer,"integer","1")),
        binding("jitter", literal(&unit,"quantity","0/1")),
        binding("blocking", literal(&unit,"quantity","0/1")),
    ]});
    let claim_body = aggregate(vec![
        binding("kind", literal(&text, "text", "schedulable")),
        binding("policy", literal(&text, "text", "fixed-priority")),
        binding("task", aggregate(vec![])),
        binding("bound", aggregate(vec![])),
        binding("tasks", aggregate(vec![task])),
    ]);
    let mut boolean_node = nodes
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "text")
        .expect("text node template")
        .clone();
    boolean_node["node_id"] = boolean.clone();
    boolean_node["semantic_type"] = boolean.clone();
    boolean_node["semantic_form"] = json!("boolean");
    let claim_node = json!({
        "node_id":claim_id,
        "schema_version":"filament.checked-semantic-graph/v2",
        "node_tag":"claim",
        "semantic_form":"schedulability",
        "semantic_type":boolean,
        "dependencies":[],
        "occurrences":[{"role":"claim","ordinal":0}],
        "body":claim_body,
    });
    nodes.push(boolean_node);
    nodes.push(claim_node);
    nodes.sort_by_key(|node| {
        serde_json::from_value::<CheckedNodeId>(node["node_id"].clone()).expect("node ID")
    });

    let source_rows = package["source_map"]
        .as_array()
        .expect("source map")
        .clone();
    let mut boolean_region = source_rows
        .iter()
        .find(|row| row["role"] == "generated")
        .expect("generated source region")
        .clone();
    boolean_region["node_id"] = boolean;
    let mut claim_region = source_rows
        .iter()
        .find(|row| row["role"] == "expression")
        .expect("source region template")
        .clone();
    claim_region["node_id"] = claim_id;
    claim_region["role"] = json!("claim");
    claim_region["ordinal"] = json!(0);
    package["source_map"]
        .as_array_mut()
        .unwrap()
        .extend([boolean_region, claim_region]);
    crate::support::checked_package::rebuild_complete_source_map(&mut package);
    crate::support::checked_package::index_fixture_node_ids(&mut package);
    crate::support::checked_package::refresh_identity(&mut package);
    let declaration = package["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["semantic_form"] == "schedulability")
        .expect("claim node")["node_id"]
        .clone();
    let object = package["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["declaration"]["qualified_name"] == json!(["O"]))
        .expect("timed model declaring object")["node_id"]
        .clone();
    package["schedulability_claims"] = json!([{
        "declaration": declaration,
        "occurrence": 0,
        "task_set": "Ctl",
        "release_bindings": if release_backed { json!([{
            "task_name":"worker",
            "selected_model_identity":"test:spine-model",
            "operation_declaring_identity":object,
            "operation_member":"step"
        }]) } else { json!([]) },
        "mode_switch_bindings": []
    }]);
    package
}

#[trace(
    "TC-451",
    "FR-246-AC-4",
    "FR-246-AC-5",
    "FR-038-AC-618",
    "FR-019-AC-30"
)]
#[test]
fn admitted_claim_views_preserve_task_set_and_release_bindings() {
    let without_release = package_with_release_binding(false);
    let unbacked = admitted(&without_release);
    let claims = unbacked.schedulability_claims().collect::<Vec<_>>();
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].task_set(), "Ctl");
    assert_eq!(claims[0].release_bindings().len(), 0);

    let with_release = package_with_release_binding(true);
    let backed = admitted(&with_release);
    let claims = backed.schedulability_claims().collect::<Vec<_>>();
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].task_set(), "Ctl");
    let releases = claims[0].release_bindings().collect::<Vec<_>>();
    assert_eq!(releases.len(), 1);
    assert_eq!(releases[0].task_name(), "worker");
    assert_eq!(releases[0].selected_model_identity(), "test:spine-model");
    let object: CheckedNodeId = serde_json::from_value(
        with_release["schedulability_claims"][0]["release_bindings"][0]
            ["operation_declaring_identity"]
            .clone(),
    )
    .expect("declaring object identity");
    assert_eq!(releases[0].operation_declaring_identity(), &object);
    assert_eq!(releases[0].operation_member(), "step");
}

fn as_mode_switch(wire: &mut Value) {
    let release = wire["schedulability_claims"][0]["release_bindings"][0].clone();
    wire["schedulability_claims"][0]["release_bindings"] = json!([]);
    wire["schedulability_claims"][0]["mode_switch_bindings"] = json!([{
        "selected_model_identity":release["selected_model_identity"],
        "operation_declaring_identity":release["operation_declaring_identity"],
        "operation_member":release["operation_member"]
    }]);
}

fn add_second_named_task(wire: &mut Value) {
    let claim = wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|node| node["semantic_form"] == "schedulability")
        .expect("claim node");
    let tasks = &mut claim["body"]["members"][4]["value"]["members"];
    let mut second = tasks[0].clone();
    second["members"][0]["value"]["value"] = json!("alpha");
    second["members"][8]["value"]["value"] = json!("2");
    let tasks = tasks.as_array_mut().expect("task list");
    tasks.push(second);
    tasks.sort_by(|left, right| {
        left["members"][0]["value"]["value"]
            .as_str()
            .cmp(&right["members"][0]["value"]["value"].as_str())
    });
}

fn add_second_source_operation(wire: &mut Value) {
    let mut second = wire["operation_bodies"][0].clone();
    second["operation"] = json!("zstep");
    wire["operation_bodies"]
        .as_array_mut()
        .unwrap()
        .push(second);
    wire["operation_bodies"]
        .as_array_mut()
        .unwrap()
        .sort_by(|left, right| left["operation"].as_str().cmp(&right["operation"].as_str()));
}

#[trace(
    "TC-048",
    "FR-038-AC-619",
    "FR-038-AC-620",
    "FR-038-AC-621",
    "FR-038-AC-622"
)]
#[test]
fn release_and_mode_switch_relations_join_exact_claim_model_and_member() {
    use filament_contract_model::{
        CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
    };

    let mut absent_task = package_with_release_binding(true);
    absent_task["schedulability_claims"][0]["release_bindings"][0]["task_name"] = json!("missing");
    refuses(
        &absent_task,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/schedulability_claims/0/release_bindings/0/task_name",
    );

    for mode_switch in [false, true] {
        let mut absent_model = package_with_release_binding(true);
        if mode_switch {
            as_mode_switch(&mut absent_model);
        }
        absent_model["schedulability_claims"][0][if mode_switch {
            "mode_switch_bindings"
        } else {
            "release_bindings"
        }][0]["selected_model_identity"] = json!("test:missing-model");
        refuses(
            &absent_model,
            Code::InvalidPackage,
            Some(Cause::InvalidValue),
            if mode_switch {
                "/schedulability_claims/0/mode_switch_bindings/0/selected_model_identity"
            } else {
                "/schedulability_claims/0/release_bindings/0/selected_model_identity"
            },
        );

        let mut wrong_declaration = package_with_release_binding(true);
        if mode_switch {
            as_mode_switch(&mut wrong_declaration);
        }
        wrong_declaration["schedulability_claims"][0][if mode_switch {
            "mode_switch_bindings"
        } else {
            "release_bindings"
        }][0]["operation_declaring_identity"] =
            json!({"package":"pkg","path":["Missing"],"ordinal":0});
        refuses(
            &wrong_declaration,
            Code::InvalidPackage,
            Some(Cause::InvalidValue),
            if mode_switch {
                "/schedulability_claims/0/mode_switch_bindings/0/operation_declaring_identity"
            } else {
                "/schedulability_claims/0/release_bindings/0/operation_declaring_identity"
            },
        );

        let mut wrong_member = package_with_release_binding(true);
        if mode_switch {
            as_mode_switch(&mut wrong_member);
        }
        wrong_member["schedulability_claims"][0][if mode_switch {
            "mode_switch_bindings"
        } else {
            "release_bindings"
        }][0]["operation_member"] = json!("missing");
        refuses(
            &wrong_member,
            Code::InvalidPackage,
            Some(Cause::InvalidValue),
            if mode_switch {
                "/schedulability_claims/0/mode_switch_bindings/0/operation_member"
            } else {
                "/schedulability_claims/0/release_bindings/0/operation_member"
            },
        );
    }
}

#[trace(
    "TC-048",
    "FR-038-AC-618",
    "FR-038-AC-623",
    "FR-038-AC-627",
    "FR-019-AC-30"
)]
#[test]
fn claim_occurrence_relations_round_trip_without_collapsing_shared_body() {
    let mut wire = package_with_release_binding(true);
    let node = wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|node| node["semantic_form"] == "schedulability")
        .expect("claim node");
    let declaration = node["node_id"].clone();
    node["occurrences"] = json!([
        {"role":"claim","ordinal":0},
        {"role":"claim","ordinal":1}
    ]);
    let mut second_row = wire["source_map"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["node_id"] == declaration && row["role"] == "claim")
        .expect("claim source row")
        .clone();
    second_row["ordinal"] = json!(1);
    wire["source_map"].as_array_mut().unwrap().push(second_row);
    let mut second_claim = wire["schedulability_claims"][0].clone();
    second_claim["occurrence"] = json!(1);
    second_claim["task_set"] = json!("Other");
    let release = second_claim["release_bindings"][0].clone();
    second_claim["release_bindings"] = json!([]);
    second_claim["mode_switch_bindings"] = json!([{
        "selected_model_identity":release["selected_model_identity"],
        "operation_declaring_identity":release["operation_declaring_identity"],
        "operation_member":release["operation_member"]
    }]);
    wire["schedulability_claims"]
        .as_array_mut()
        .unwrap()
        .push(second_claim);
    crate::support::checked_package::refresh_claim_names(&mut wire);
    wire["source_map"]
        .as_array_mut()
        .unwrap()
        .sort_by(|left, right| {
            let left_id: CheckedNodeId =
                serde_json::from_value(left["node_id"].clone()).expect("source node ID");
            let right_id: CheckedNodeId =
                serde_json::from_value(right["node_id"].clone()).expect("source node ID");
            left_id
                .cmp(&right_id)
                .then_with(|| left["role"].as_str().cmp(&right["role"].as_str()))
                .then_with(|| left["ordinal"].as_u64().cmp(&right["ordinal"].as_u64()))
        });

    let read = admitted(&wire);
    let claims = read.schedulability_claims().collect::<Vec<_>>();
    assert_eq!(claims.len(), 2);
    assert_eq!(claims[0].declaration(), claims[1].declaration());
    assert_eq!(claims[0].occurrence().ordinal, 0);
    assert_eq!(claims[1].occurrence().ordinal, 1);
    assert_eq!(claims[0].task_set(), "Ctl");
    assert_eq!(claims[1].task_set(), "Other");
    assert_eq!(claims[0].release_bindings().len(), 1);
    assert_eq!(claims[0].mode_switch_bindings().len(), 0);
    assert_eq!(claims[1].release_bindings().len(), 0);
    assert_eq!(claims[1].mode_switch_bindings().len(), 1);
    let mode_switch = claims[1]
        .mode_switch_bindings()
        .next()
        .expect("mode-switch relation");
    let source_binding: CheckedNodeId = serde_json::from_value(
        wire["schedulability_claims"][1]["mode_switch_bindings"][0]["operation_declaring_identity"]
            .clone(),
    )
    .expect("mode-switch declaring identity");
    assert_eq!(mode_switch.selected_model_identity(), "test:spine-model");
    assert_eq!(mode_switch.operation_declaring_identity(), &source_binding);
    assert_eq!(mode_switch.operation_member(), "step");

    let bytes = encode(&read);
    let encoded: Value = serde_json::from_slice(&bytes).expect("wire package");
    assert_eq!(
        encoded["schedulability_claims"],
        wire["schedulability_claims"]
    );
    let reread = admitted(&encoded);
    assert_eq!(
        reread.schedulability_claim_rows(),
        read.schedulability_claim_rows()
    );
    assert_eq!(reread.schedulability_claims().count(), 2);
}

#[trace("TC-048", "FR-038-AC-624")]
#[test]
fn release_bindings_use_canonical_task_order_without_repair() {
    use filament_contract_model::{
        CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
    };

    let mut descending = package_with_release_binding(true);
    add_second_named_task(&mut descending);
    let worker = descending["schedulability_claims"][0]["release_bindings"][0].clone();
    let mut alpha = worker.clone();
    alpha["task_name"] = json!("alpha");
    descending["schedulability_claims"][0]["release_bindings"] =
        json!([worker.clone(), alpha.clone()]);
    refuses(
        &descending,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/schedulability_claims/0/release_bindings/1",
    );

    let mut ascending = descending;
    ascending["schedulability_claims"][0]["release_bindings"] = json!([alpha, worker]);
    let read = admitted(&ascending);
    let releases = read
        .schedulability_claims()
        .next()
        .expect("claim")
        .release_bindings()
        .map(|release| release.task_name().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(releases, ["alpha", "worker"]);
    let mut raw_rows = read.schedulability_claim_rows().unwrap().to_vec();
    raw_rows[0].release_bindings.reverse();
    let bytes = encode_with_claim_rows(&read, Some(&raw_rows));
    let encoded: Value = serde_json::from_slice(&bytes).expect("encoded package");
    assert_eq!(
        encoded["schedulability_claims"][0]["release_bindings"][0]["task_name"],
        "worker"
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = crate::read_and_account(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &fixtures::evidence_for(&encoded),
    ) else {
        panic!("strict reader must refuse, not repair, descending release bindings")
    };
    assert_eq!(refusal.code, Code::InvalidPackage);
    assert_eq!(refusal.cause, Some(Cause::InvalidValue));
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().as_str(),
        "/schedulability_claims/0/release_bindings/1"
    );
}

#[trace("TC-048", "FR-038-AC-625")]
#[test]
fn mode_switch_bindings_use_canonical_operation_order_without_repair() {
    use filament_contract_model::{
        CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
    };

    let mut descending = package_with_release_binding(true);
    as_mode_switch(&mut descending);
    add_second_source_operation(&mut descending);
    let binding = descending["schedulability_claims"][0]["mode_switch_bindings"][0].clone();
    let mut zstep = binding.clone();
    zstep["operation_member"] = json!("zstep");
    descending["schedulability_claims"][0]["mode_switch_bindings"] =
        json!([zstep.clone(), binding.clone()]);
    refuses(
        &descending,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/schedulability_claims/0/mode_switch_bindings/1",
    );

    let mut ascending = descending;
    ascending["schedulability_claims"][0]["mode_switch_bindings"] = json!([binding, zstep]);
    let read = admitted(&ascending);
    let members = read
        .schedulability_claims()
        .next()
        .expect("claim")
        .mode_switch_bindings()
        .map(|binding| binding.operation_member().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(members, ["step", "zstep"]);
    let mut raw_rows = read.schedulability_claim_rows().unwrap().to_vec();
    raw_rows[0].mode_switch_bindings.reverse();
    let bytes = encode_with_claim_rows(&read, Some(&raw_rows));
    let encoded: Value = serde_json::from_slice(&bytes).expect("encoded package");
    assert_eq!(
        encoded["schedulability_claims"][0]["mode_switch_bindings"][0]["operation_member"],
        "zstep"
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = crate::read_and_account(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &fixtures::evidence_for(&encoded),
    ) else {
        panic!("strict reader must refuse, not repair, descending mode-switch bindings")
    };
    assert_eq!(refusal.code, Code::InvalidPackage);
    assert_eq!(refusal.cause, Some(Cause::InvalidValue));
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().as_str(),
        "/schedulability_claims/0/mode_switch_bindings/1"
    );
}

fn admitted(package: &Value) -> Box<CheckedPackageV2> {
    let result = crate::read_and_account(
        &fixtures::canonical(package),
        CheckedPackageReadLimits::bounded(),
        &fixtures::evidence_for(package),
    );
    match result {
        CheckedPackageV2ReadResult::Admitted(read) => read,
        other => panic!("local task claim must admit: {other:?}"),
    }
}

fn exact(read: CheckedLiteralRead<'_>, unit: &CheckedNodeId, numerator: i64, denominator: i64) {
    assert_eq!(read.type_id, unit);
    assert_eq!(
        read.value,
        CheckedLiteralValue::Rational {
            numerator: numerator.into(),
            denominator: denominator.into()
        }
    );
}

fn encode(read: &CheckedPackageV2) -> Vec<u8> {
    encode_with_claim_rows(read, read.schedulability_claim_rows())
}

fn encode_with_claim_rows(
    read: &CheckedPackageV2,
    schedulability_claims: Option<&[CheckedSchedulabilityClaimRow]>,
) -> Vec<u8> {
    encode_checked_package_v2(
        read.name(),
        read.lock(),
        read.graph(),
        CheckedPackageSourceCorrespondenceRef {
            source_map: read.source_map(),
            refinement_step_rows: read.refinement_step_rows(),
        },
        CheckedPackageSidecarsRef {
            temporal_clause_names: read.temporal_clause_names(),
            claim_names: read.claim_names(),
            operation_effects: None,
            operation_bodies: read.operation_body_rows(),
            checked_term_data: read.checked_term_data(),
            temporal_bindings: read.temporal_bindings(),
            schedulability_claims,
        },
        read.capability_report(),
        read.diagnostics(),
        quire_canonical::Limits::new(1 << 24),
    )
    .expect("complete encoding")
}

#[trace("TC-451", "FR-019-AC-19", "FR-038-AC-396", "FR-038-AC-398")]
#[test]
fn authenticated_views_preserve_exact_premises_and_shared_occurrences() {
    for (policy, expected_policy, response, high) in [
        (
            "fixed-priority",
            CheckedSchedulingPolicy::FixedPriority,
            true,
            false,
        ),
        ("edf", CheckedSchedulingPolicy::Edf, false, false),
        ("amc", CheckedSchedulingPolicy::Amc, false, true),
    ] {
        let wire = package(policy, response, high, 2);
        let read = admitted(&wire);
        let mut claims = read.schedulability_claims();
        assert_eq!(claims.len(), 2);
        let first = claims.next().expect("first occurrence");
        assert_eq!(claims.len(), 1);
        let second = claims.next().expect("second occurrence");
        assert_eq!(claims.len(), 0);
        assert!(claims.next().is_none());
        assert_eq!(first.declaration(), second.declaration());
        assert!(
            std::ptr::eq(first.body(), second.body()),
            "shared node has one borrowed body"
        );
        assert_eq!(first.occurrence().ordinal, 0);
        assert_eq!(second.occurrence().ordinal, 1);
        assert_eq!(first.policy(), expected_policy);
        assert_eq!(
            first.kind(),
            if response {
                CheckedSchedulabilityKind::Response
            } else {
                CheckedSchedulabilityKind::Schedulable
            }
        );
        assert_eq!(first.response_task(), response.then_some("worker"));
        assert_eq!(first.tasks().len(), 1);
        let task = first.tasks().next().expect("worker");
        assert_eq!(task.name(), "worker");
        assert_eq!(task.arrival(), CheckedTaskArrival::Sporadic);
        assert_eq!(
            task.criticality(),
            if high {
                CheckedTaskCriticality::Hi
            } else {
                CheckedTaskCriticality::Lo
            }
        );
        let unit = task.period().type_id;
        for (value, numerator, denominator) in [
            (task.period(), 19, 2),
            (task.deadline(), 17, 2),
            (task.bcet(), 3, 2),
            (task.wcet(), 5, 2),
            (task.jitter(), 0, 1),
            (task.blocking(), 0, 1),
        ] {
            exact(value, unit, numerator, denominator);
        }
        if high {
            exact(task.wcet_hi().expect("high work"), unit, 7, 2);
        } else {
            assert!(task.wcet_hi().is_none());
        }
        if policy == "edf" {
            assert!(task.priority().is_none());
        } else {
            assert_eq!(
                task.priority().expect("priority").value,
                CheckedLiteralValue::Integer(7_i64.into())
            );
        }
        if response {
            exact(first.response_bound().expect("response bound"), unit, 17, 2);
        } else {
            assert!(first.response_bound().is_none());
        }
        let bytes = encode(&read);
        let encoded: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            encoded["schedulability_claims"],
            wire["schedulability_claims"]
        );
        assert_eq!(
            encoded["semantic_graph"],
            crate::support::checked_package::wire_form(&wire)["semantic_graph"]
        );
        let again = admitted(&encoded);
        assert_eq!(encode(&again), bytes);
        assert_eq!(again.schedulability_claims().len(), 2);
    }
}

#[trace(
    "TC-048",
    "FR-038-AC-397",
    "FR-038-AC-399",
    "FR-038-AC-627",
    "FR-051-AC-7"
)]
#[test]
fn absent_index_stays_absent_and_raw_rows_do_not_authenticate_claims() {
    let wire = fixtures::v2_all_families();
    let read = admitted(&wire);
    assert!(read.schedulability_claim_rows().is_none());
    assert_eq!(read.schedulability_claims().len(), 0);
    let encoded: Value = serde_json::from_slice(&encode(&read)).unwrap();
    assert!(encoded.get("schedulability_claims").is_none());
    let explicit_empty: Value =
        serde_json::from_slice(&encode_with_claim_rows(&read, Some(&[]))).unwrap();
    assert_eq!(explicit_empty["schedulability_claims"], json!([]));
    assert_eq!(
        encoded,
        crate::support::checked_package::wire_form(&wire),
        "absent-index complete package is lossless"
    );
    let mut missing = package("fixed-priority", false, false, 1);
    let row: CheckedSchedulabilityClaimRow =
        serde_json::from_value(missing["schedulability_claims"][0].clone())
            .expect("untrusted row is parseable");
    assert_eq!(row.occurrence, 0);
    missing
        .as_object_mut()
        .unwrap()
        .remove("schedulability_claims");
    fixtures::refresh_identity(&mut missing);
    let result = crate::read_and_account(
        &fixtures::canonical(&missing),
        CheckedPackageReadLimits::bounded(),
        &fixtures::evidence_for(&missing),
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("raw row cannot replace the missing authenticated index")
    };
    assert_eq!(
        refusal.code,
        filament_contract_model::CheckedPackageRefusalCode::InvalidPackage
    );
    assert_eq!(
        refusal.cause,
        Some(filament_contract_model::CheckedPackageRefusalCause::InvalidValue)
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().as_str(),
        "/schedulability_claims"
    );
}

#[trace("TC-048", "FR-038-AC-399", "FR-051-AC-7")]
#[test]
fn complete_writer_preserves_supplied_index_order_without_repair() {
    let wire = package("fixed-priority", false, false, 2);
    let read = admitted(&wire);
    let mut rows = read
        .schedulability_claim_rows()
        .expect("identity rows")
        .to_vec();
    rows.reverse();
    let bytes = encode_checked_package_v2(
        read.name(),
        read.lock(),
        read.graph(),
        CheckedPackageSourceCorrespondenceRef {
            source_map: read.source_map(),
            refinement_step_rows: read.refinement_step_rows(),
        },
        CheckedPackageSidecarsRef {
            temporal_clause_names: read.temporal_clause_names(),
            claim_names: read.claim_names(),
            operation_effects: None,
            operation_bodies: None,
            checked_term_data: read.checked_term_data(),
            temporal_bindings: read.temporal_bindings(),
            schedulability_claims: Some(&rows),
        },
        read.capability_report(),
        read.diagnostics(),
        quire_canonical::Limits::new(1 << 24),
    )
    .expect("writer retains supplied order");
    let encoded: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(encoded["schedulability_claims"][0]["occurrence"], 1);
    assert_eq!(encoded["schedulability_claims"][1]["occurrence"], 0);
    let result = crate::read_and_account(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &fixtures::evidence_for(&encoded),
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("writer must not silently canonicalize identity rows")
    };
    assert_eq!(
        refusal.code,
        filament_contract_model::CheckedPackageRefusalCode::InvalidPackage
    );
    assert_eq!(
        refusal.cause,
        Some(filament_contract_model::CheckedPackageRefusalCause::InvalidValue)
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().as_str(),
        "/schedulability_claims/1"
    );
}

#[trace("TC-048", "FR-038-AC-397")]
#[test]
fn explicit_empty_identity_index_is_malformed_even_without_claim_nodes() {
    let mut wire = fixtures::v2_all_families();
    wire["schedulability_claims"] = json!([]);
    fixtures::refresh_identity(&mut wire);
    let result = crate::read_and_account(
        &fixtures::canonical(&wire),
        CheckedPackageReadLimits::bounded(),
        &fixtures::evidence_for(&wire),
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("an explicitly present empty carrier is malformed")
    };
    assert_eq!(
        refusal.code,
        filament_contract_model::CheckedPackageRefusalCode::MalformedWire
    );
    assert_eq!(refusal.cause, None);
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().as_str(),
        "/schedulability_claims"
    );
}

#[trace("TC-048", "FR-038-AC-397", "FR-038-AC-400")]
#[test]
fn carrier_work_is_charged_before_a_row_is_resolved() {
    let mut wire = package("fixed-priority", false, false, 1);
    wire["schedulability_claims"][0]["declaration"]["package"] = json!("foreign");
    fixtures::refresh_identity(&mut wire);
    let bytes = fixtures::canonical(&wire);
    let evidence = fixtures::evidence_for(&wire);
    let read = |work| {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = work;
        crate::read_and_account(&bytes, limits, &evidence)
    };
    let (mut low, mut high) = (0, CheckedPackageReadLimits::bounded().work);
    while low + 1 < high {
        let mid = low + (high - low) / 2;
        match read(mid) {
            CheckedPackageV2ReadResult::Incomplete(_) => low = mid,
            CheckedPackageV2ReadResult::Refused(refusal) => {
                assert_eq!(
                    refusal.code,
                    filament_contract_model::CheckedPackageRefusalCode::InvalidPackage
                );
                assert_eq!(
                    refusal.cause,
                    Some(filament_contract_model::CheckedPackageRefusalCause::InvalidValue)
                );
                assert_eq!(
                    refusal.loci[0].path.as_ref().unwrap().as_str(),
                    "/schedulability_claims/0/declaration"
                );
                high = mid;
            }
            other => panic!("invalid row must refuse once its charge succeeds: {other:?}"),
        }
    }
    let CheckedPackageV2ReadResult::Incomplete(report) = read(high - 1) else {
        panic!("row resolution needs its precharge")
    };
    assert_eq!(
        report.limit_kind,
        filament_contract_model::CheckedPackageLimit::Work
    );
    assert_eq!(
        report.path.as_ref().unwrap().as_str(),
        "/schedulability_claims/0"
    );
    assert_eq!(report.limit, high - 1);
}

fn refuses(
    wire: &Value,
    code: filament_contract_model::CheckedPackageRefusalCode,
    cause: Option<filament_contract_model::CheckedPackageRefusalCause>,
    path: &str,
) {
    let result = crate::read_and_account(
        &fixtures::canonical(wire),
        CheckedPackageReadLimits::bounded(),
        &fixtures::evidence_for(wire),
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("{path}: expected refusal, got {result:?}")
    };
    assert_eq!(refusal.code, code, "{refusal:?}");
    assert_eq!(refusal.cause, cause, "{refusal:?}");
    assert_eq!(refusal.loci.len(), 1);
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().as_str(),
        path,
        "{refusal:?}"
    );
}

#[trace("TC-048", "FR-038-AC-398", "FR-038-AC-399")]
#[test]
fn claim_body_refusals_follow_structural_order_among_nonclaim_nodes() {
    use filament_contract_model::{
        CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
    };

    let mut wire = package("fixed-priority", false, false, 1);
    let nodes = wire["semantic_graph"]["nodes"].as_array_mut().unwrap();
    let worker = nodes.len() - 1;
    let first = fixtures::node_id("fixture/schedulability/a");
    let second = nodes[worker]["node_id"].clone();
    let mut earlier_claim = nodes[worker].clone();
    earlier_claim["node_id"] = first.clone();
    nodes.push(earlier_claim);
    let earlier = nodes.len() - 1;
    for position in [worker, earlier] {
        nodes[position]["body"]["members"][1]["value"]["value"] = json!("unknown-policy");
    }
    wire["schedulability_claims"] = json!([
        {"declaration": first, "occurrence": 0, "task_set":"Ctl", "release_bindings":[], "mode_switch_bindings":[]},
        {"declaration": second, "occurrence": 0, "task_set":"Ctl", "release_bindings":[], "mode_switch_bindings":[]},
    ]);
    fixtures::rebuild_complete_source_map(&mut wire);
    fixtures::refresh_identity(&mut wire);

    refuses(
        &wire,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        &format!("/semantic_graph/nodes/{earlier}/body/members/1"),
    );
}

#[trace("TC-048", "FR-038-AC-397", "FR-038-AC-400")]
#[test]
fn carrier_join_checks_have_deterministic_precedence() {
    use filament_contract_model::{
        CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
    };
    for case in [
        "absent-local",
        "wrong-kind",
        "wrong-role",
        "wire-before-graph",
        "graph-before-carrier",
        "aggregate-before-carrier",
        "declaration-before-occurrence",
        "duplicate-before-order",
        "every-row-before-coverage",
        "carrier-before-body",
    ] {
        let mut wire = package("fixed-priority", false, false, 2);
        let n = wire["semantic_graph"]["nodes"].as_array().unwrap().len() - 1;
        let mut code = Code::InvalidPackage;
        let mut cause = Some(Cause::InvalidValue);
        let path = match case {
            "absent-local" => {
                wire["schedulability_claims"][0]["declaration"]["path"] = json!(["missing-local"]);
                "/schedulability_claims/0/declaration".to_owned()
            }
            "wrong-kind" => {
                wire["schedulability_claims"][0]["declaration"] =
                    wire["semantic_graph"]["nodes"][0]["node_id"].clone();
                "/schedulability_claims/0/declaration".to_owned()
            }
            "wrong-role" => {
                wire["semantic_graph"]["nodes"][n]["occurrences"][0]["role"] = json!("expression");
                "/schedulability_claims/0/occurrence".to_owned()
            }
            "wire-before-graph" => {
                wire["schedulability_claims"][0]["occurrence"] = json!(-1);
                wire["semantic_graph"]["nodes"][n]["dependencies"] =
                    json!([wire["semantic_graph"]["nodes"][0]["node_id"].clone()]);
                code = Code::MalformedWire;
                cause = None;
                "/schedulability_claims/0/occurrence".to_owned()
            }
            "graph-before-carrier" => {
                wire["schedulability_claims"][0]["occurrence"] = json!(99);
                wire["semantic_graph"]["nodes"][n]["dependencies"] =
                    json!([wire["semantic_graph"]["nodes"][0]["node_id"].clone()]);
                code = Code::InvalidSemanticGraph;
                cause = None;
                format!("/semantic_graph/nodes/{n}/dependencies")
            }
            "aggregate-before-carrier" => {
                wire["schedulability_claims"][0]["occurrence"] = json!(99);
                wire["semantic_graph"]["nodes"][n]["body"]["members"] = json!("not-an-array");
                code = Code::InvalidSemanticGraph;
                cause = None;
                format!("/semantic_graph/nodes/{n}/body/members")
            }
            "declaration-before-occurrence" => {
                wire["schedulability_claims"][0]["declaration"]["package"] = json!("foreign");
                wire["schedulability_claims"][0]["occurrence"] = json!(99);
                "/schedulability_claims/0/declaration".to_owned()
            }
            "duplicate-before-order" => {
                let first = wire["schedulability_claims"][1].clone();
                let second = wire["schedulability_claims"][0].clone();
                wire["schedulability_claims"] = json!([second.clone(), first, second]);
                cause = Some(Cause::ConflictingDefinition);
                "/schedulability_claims/2".to_owned()
            }
            "every-row-before-coverage" => {
                wire["schedulability_claims"][1]["occurrence"] = json!(99);
                "/schedulability_claims/1/occurrence".to_owned()
            }
            "carrier-before-body" => {
                wire["schedulability_claims"][0]["occurrence"] = json!(99);
                wire["semantic_graph"]["nodes"][n]["body"]["members"][1]["value"]["value"] =
                    json!("unknown-policy");
                "/schedulability_claims/0/occurrence".to_owned()
            }
            _ => unreachable!(),
        };
        fixtures::rebuild_complete_source_map(&mut wire);
        fixtures::refresh_identity(&mut wire);
        refuses(&wire, code, cause, &path);
    }
}

#[trace("TC-048", "FR-038-AC-400")]
#[test]
fn exact_measured_work_admits_and_one_less_has_no_admitted_view() {
    let wire = package("amc", false, true, 2);
    let bytes = fixtures::canonical(&wire);
    let evidence = fixtures::evidence_for(&wire);
    let read = |work| {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = work;
        crate::read_and_account(&bytes, limits, &evidence)
    };
    let (mut low, mut high) = (0, CheckedPackageReadLimits::bounded().work);
    while low + 1 < high {
        let middle = low + (high - low) / 2;
        match read(middle) {
            CheckedPackageV2ReadResult::Admitted(_) => high = middle,
            CheckedPackageV2ReadResult::Incomplete(_) => low = middle,
            other => panic!("valid claim changed outcome: {other:?}"),
        }
    }
    let CheckedPackageV2ReadResult::Admitted(exact) = read(high) else {
        panic!("exact measured work must admit")
    };
    assert_eq!(exact.schedulability_claims().len(), 2);
    let CheckedPackageV2ReadResult::Incomplete(report) = read(high - 1) else {
        panic!("one less work returns no authenticated view")
    };
    assert_eq!(
        report.limit_kind,
        filament_contract_model::CheckedPackageLimit::Work
    );
    assert_eq!(report.limit, high - 1);
}

#[trace("TC-451", "FR-038-AC-399", "FR-051-AC-7")]
#[test]
fn complete_claim_writer_has_exact_canonical_byte_boundary() {
    let wire = package("fixed-priority", true, false, 2);
    let read = admitted(&wire);
    let expected = fixtures::canonical(&wire);
    let length = u64::try_from(expected.len()).unwrap();
    let write = |limit| {
        encode_checked_package_v2(
            read.name(),
            read.lock(),
            read.graph(),
            CheckedPackageSourceCorrespondenceRef {
                source_map: read.source_map(),
                refinement_step_rows: read.refinement_step_rows(),
            },
            CheckedPackageSidecarsRef {
                temporal_clause_names: read.temporal_clause_names(),
                claim_names: read.claim_names(),
                operation_effects: None,
                operation_bodies: None,
                checked_term_data: read.checked_term_data(),
                temporal_bindings: read.temporal_bindings(),
                schedulability_claims: read.schedulability_claim_rows(),
            },
            read.capability_report(),
            read.diagnostics(),
            quire_canonical::Limits::new(limit),
        )
    };
    assert_eq!(write(length).expect("exact canonical limit"), expected);
    match write(length - 1) {
        Err(quire_canonical::Error::Limit(limit)) => {
            assert_eq!(limit.kind, quire_canonical::LimitKind::CanonicalBytes);
            assert_eq!(limit.bound, length - 1);
        }
        other => panic!("one less byte returns typed limit without output: {other:?}"),
    }
}

#[trace("TC-451", "FR-019-AC-19", "FR-038-AC-398")]
#[test]
fn sparse_high_occurrence_ordinals_resolve_independently_of_array_position() {
    let mut wire = package("fixed-priority", false, false, 2);
    let n = wire["semantic_graph"]["nodes"].as_array().unwrap().len() - 1;
    wire["semantic_graph"]["nodes"][n]["occurrences"] = json!([
        {"role":"claim","ordinal":9_000_001},
        {"role":"claim","ordinal":47},
    ]);
    wire["schedulability_claims"][0]["occurrence"] = json!(47);
    wire["schedulability_claims"][1]["occurrence"] = json!(9_000_001);
    fixtures::rebuild_complete_source_map(&mut wire);
    fixtures::refresh_identity(&mut wire);
    let read = admitted(&wire);
    let claims = read.schedulability_claims().collect::<Vec<_>>();
    assert_eq!(claims.len(), 2);
    assert_eq!(claims[0].occurrence().ordinal, 47);
    assert_eq!(claims[1].occurrence().ordinal, 9_000_001);
    assert!(std::ptr::eq(claims[0].body(), claims[1].body()));
    assert_eq!(
        serde_json::from_slice::<Value>(&encode(&read)).unwrap(),
        crate::support::checked_package::wire_form(&wire)
    );
}

#[trace("TC-048", "FR-038-AC-399")]
#[test]
fn malformed_exact_task_literals_refuse_at_payload_before_interpretation() {
    use filament_contract_model::{
        CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
    };
    for (member, spelling) in [
        (2, "1/0"),
        (2, "2/4"),
        (2, "-0/1"),
        (2, "01/1"),
        (2, "not-a-rational"),
        (8, "07"),
        (8, "1/1"),
    ] {
        let mut wire = package("fixed-priority", false, false, 1);
        let n = wire["semantic_graph"]["nodes"].as_array().unwrap().len() - 1;
        wire["semantic_graph"]["nodes"][n]["body"]["members"][4]["value"]["members"][0]
            ["members"][member]["value"]["value"] = json!(spelling);
        fixtures::refresh_identity(&mut wire);
        refuses(
            &wire,
            Code::InvalidPackage,
            Some(Cause::InvalidValue),
            &format!("/semantic_graph/nodes/{n}/body/members/4/value/members/0/members/{member}/value/value"),
        );
    }
    let mut negative = package("fixed-priority", false, false, 1);
    let n = negative["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .len()
        - 1;
    negative["semantic_graph"]["nodes"][n]["body"]["members"][4]["value"]["members"][0]
        ["members"][2]["value"]["value"] = json!("-1/2");
    fixtures::refresh_identity(&mut negative);
    refuses(
        &negative,
        Code::InvalidTimedModel,
        Some(Cause::InvalidTaskSet),
        &format!("/semantic_graph/nodes/{n}/body/members/4/value/members/0"),
    );
}

/// A read of a package with a local task claim reaches the claim carrier
/// join, the claim order, the priority set and the model table: every
/// graph-stage request, theirs among them, is denied one byte short and
/// admitted at its exact allowance, and the all-families base (which makes no
/// claim request) has fewer. Their allocated == requested equalities are the
/// lib tests `claim_carrier_join_allocates_exactly_what_it_charges` and
/// `model_table_allocates_exactly_what_it_charges`. The IDs a claim reads out
/// of Value members are charged only after they are parsed until the body
/// migration lands.
#[trace("TC-472", "FR-038-AC-504")]
#[test]
fn claim_carrier_tables_and_order_are_charged_exactly() {
    use crate::checked_package_v2_graph_read_storage::{
        assert_exact_and_one_short, stage_requests,
    };
    use filament_contract_model::{NativeJoinBudget, PackageReadStage};
    let graph_requests = |wire: &Value| {
        let bytes = fixtures::canonical(wire);
        let evidence = fixtures::evidence_for(wire);
        let read = move |budget: &mut NativeJoinBudget| {
            crate::fspec_read_proof::read_budget_and_account(
                &bytes,
                CheckedPackageReadLimits::bounded(),
                &evidence,
                budget,
            )
        };
        let requests = stage_requests(&read, PackageReadStage::Graph);
        assert_exact_and_one_short(&read, &requests);
        requests
    };
    let base = graph_requests(&fixtures::v2_all_families());
    let claimed = graph_requests(&package("fixed-priority", true, false, 2));
    assert!(
        claimed.len() > base.len(),
        "the claim package reaches the carrier, order and priority tables"
    );
    let amc = graph_requests(&package("amc", false, true, 2));
    assert!(amc.len() > base.len());
}
