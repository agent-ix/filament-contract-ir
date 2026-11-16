// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Timed no-row qualification against FSpec's TN vectors by reference.

use super::*;
use filament_contract_model::CheckedModelTimeReadErrorCode;
use std::collections::BTreeSet;

fn vectors() -> Value {
    read_json(&proposal_dir_of(&fixtures_dir()).join("timed-clock-no-row-vectors.json"))
        .expect("published Timed no-row vectors")
}

fn timed_model_clause() -> Value {
    let mut wire = read_json(&fixtures_dir().join("positive-timed-model-nodes.json"))
        .expect("published timed model fixture");
    for selection in wire["lock"]["profile_selections"]
        .as_array_mut()
        .expect("profile selections")
    {
        if selection["role"] == "temporal_profile" {
            selection["definition"]["identity"] = json!("filament.temporal.timed/v1");
        }
    }
    for node in wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
    {
        if let Some(laws) = node
            .get_mut("body")
            .and_then(|body| body.get_mut("operation"))
            .and_then(|operation| operation.get_mut("laws"))
            .and_then(Value::as_array_mut)
        {
            for law in laws {
                if law["role"] == "temporal_profile" {
                    law["definition"]["identity"] = json!("filament.temporal.timed/v1");
                }
            }
        }
    }
    let clause = &mut wire["semantic_graph"]["nodes"][8];
    clause["body"]["arguments"][1]["value"] = json!("model-time");
    wire["semantic_graph"]["nodes"][29]["body"]["operation"]["member"] = json!({
        "kind":"temporal_interval", "interval": {
            "lower":{"numerator":"0","denominator":"1"},
            "upper":{"numerator":"3","denominator":"1"},
            "lower_end":"closed", "upper_end":"closed"
        }
    });
    fixture_support::settle(&mut wire);
    fixture_support::refresh_temporal_bindings(&mut wire);
    fixture_support::refresh_identity(&mut wire);
    wire
}

/// The timed fixture with its clause ranging over a trace parameter and
/// naming `clock` as its role, and no `temporal_bindings` row.
fn trace_clause(clock: &str) -> Value {
    let mut trace = timed_model_clause();
    trace["semantic_graph"]["nodes"][8]["body"]["arguments"][0]["target"] =
        trace["semantic_graph"]["nodes"][36]["node_id"].clone();
    trace["semantic_graph"]["nodes"][8]["body"]["arguments"][1]["value"] = json!(clock);
    fixture_support::settle(&mut trace);
    fixture_support::refresh_temporal_bindings(&mut trace);
    fixture_support::refresh_identity(&mut trace);
    trace
}

fn admitted(wire: &Value) -> Box<CheckedPackageV2> {
    match read_package(wire).expect("published selection evidence") {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("expected Timed admission: {other:?}"),
    }
}

fn refusal(wire: &Value) -> filament_contract_model::CheckedPackageRefusal {
    match read_package(wire).expect("published selection evidence") {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected Timed refusal: {other:?}"),
    }
}

#[trace("TC-048", "FR-038-AC-482", "FR-038-AC-483", "FR-038-AC-484")]
#[test]
fn timed_clock_no_row_vectors_join_model_and_refuse_extra_row() {
    let published = vectors();
    let cases = published["cases"].as_array().expect("TN cases");
    assert_eq!(
        cases
            .iter()
            .map(|case| case["id"].as_str().expect("case id"))
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "TN-01", "TN-02", "TN-03", "TN-04", "TN-05", "TN-06", "TN-07", "TN-08", "TN-09"
        ])
    );
    let case = |id: &str| cases.iter().find(|case| case["id"] == id).expect("TN case");
    assert_eq!(case("TN-01")["clock_name"], "model-time");
    assert!(case("TN-01")["temporal_bindings"].is_null());

    let wire = timed_model_clause();
    assert!(wire.get("temporal_bindings").is_none(), "TN-01 has no row");
    let package = admitted(&wire);
    let clause = &package.resolved_graph().nodes[8];
    let binding = package
        .model_time_for_clause(&clause.node_id, 100)
        .expect("bounded model-time read")
        .expect("model subject");
    assert_eq!(binding.clause(), &clause.node_id);
    assert_eq!(
        binding.timed_root(),
        &package.document_held_id(52).expect("graph node").clone()
    );
    assert_eq!(
        binding.model(),
        &package.document_held_id(53).expect("graph node").clone()
    );
    assert_eq!(
        binding.source(),
        &package.document_held_id(44).expect("graph node").clone()
    );
    assert_eq!(
        binding.unit(),
        &package.document_held_id(43).expect("graph node").clone()
    );
    assert_eq!(
        package
            .model_time_for_clause(&clause.node_id, binding.work())
            .unwrap(),
        Some(binding)
    );
    let denied = package
        .model_time_for_clause(&clause.node_id, binding.work() - 1)
        .unwrap_err();
    assert_eq!(
        (denied.code, denied.work, denied.attempted_work),
        (
            CheckedModelTimeReadErrorCode::WorkLimit,
            binding.work() - 1,
            1
        )
    );
    assert_eq!(*denied.at, *binding.unit());
    let wrong_clause = package
        .model_time_for_clause(binding.source(), 100)
        .unwrap_err();
    assert_eq!(
        wrong_clause.code,
        CheckedModelTimeReadErrorCode::WrongClause
    );
    assert_eq!(*wrong_clause.at, *binding.source());

    // TN-02, TN-06 to TN-09: a clause over a trace subject takes exactly one
    // closed `timed {unit}` row naming its clock role.
    let mut trace = trace_clause("plant");
    trace
        .as_object_mut()
        .expect("package")
        .remove("temporal_bindings");
    fixture_support::refresh_identity_projection(&mut trace);
    let absent_row = refusal(&trace);
    assert_eq!(
        (
            absent_row.code,
            absent_row.cause,
            absent_row.loci[0].path.as_ref().map(ToString::to_string)
        ),
        (
            Code::InvalidPackage,
            Some(Cause::TimedNodeReference),
            Some(
                case("TN-02")["expected"]["pointer"]
                    .as_str()
                    .unwrap()
                    .replace("{clause-index}", "8")
            )
        )
    );
    let timed_row = |name: &str, clock: Value| {
        let mut package = trace_clause(name);
        let declaration = package["semantic_graph"]["nodes"][8]["node_id"].clone();
        package["temporal_bindings"] = json!([{
            "declaration": declaration,
            "occurrence": 0,
            "clock_name": name,
            "clock": clock,
        }]);
        fixture_support::refresh_identity_projection(&mut package);
        package
    };
    let row_of = |id: &str| case(id)["row"].clone();
    for id in ["TN-06"] {
        let row = row_of(id);
        let trace_row = timed_row(row["clock_name"].as_str().unwrap(), row["clock"].clone());
        let trace_package = admitted(&trace_row);
        let trace_clause = trace_package
            .document_held_id(8)
            .expect("graph node")
            .clone();
        assert_eq!(
            trace_package.temporal_bindings().unwrap(),
            [filament_contract_model::TemporalBinding {
                declaration: trace_clause.clone(),
                occurrence: 0,
                clock_name: "plant".into(),
                clock: filament_contract_model::ClockConfiguration::Timed { unit: "ms".into() },
            }]
        );
        assert!(
            trace_package
                .model_time_for_clause(&trace_clause, 100)
                .unwrap()
                .is_none(),
            "{id}: a trace clause reads no checked model time"
        );
    }
    for id in ["TN-08", "TN-09"] {
        let row = row_of(id);
        let name = row["clock_name"].as_str().unwrap();
        let rejected = refusal(&timed_row(name, row["clock"].clone()));
        assert_eq!(
            (
                rejected.code,
                rejected.cause,
                rejected.loci[0].path.as_ref().map(ToString::to_string)
            ),
            (
                Code::InvalidPackage,
                Some(Cause::TimedNodeReference),
                case(id)["expected"]["pointer"]
                    .as_str()
                    .map(|pointer| pointer.to_owned())
            ),
            "{id}"
        );
    }

    // The `timed` alternative is closed: an empty unit is malformed, a missing
    // unit, an extra member and an unknown clock kind are each refused, and
    // none is defaulted or substituted.
    let refused: Vec<_> = [
        json!({"kind": "timed", "unit": ""}),
        json!({"kind": "timed"}),
        json!({"kind": "timed", "unit": "ms", "period": "1"}),
        json!({"kind": "wall_clock", "unit": "ms"}),
    ]
    .into_iter()
    .map(|clock| {
        let rejected = refusal(&timed_row("plant", clock));
        (
            rejected.code,
            rejected.loci[0].path.as_ref().map(ToString::to_string),
        )
    })
    .collect();
    assert_eq!(
        refused,
        vec![
            (
                Code::MalformedWire,
                Some("/temporal_bindings/0/clock/unit".to_owned())
            ),
            (
                Code::MalformedWire,
                Some("/temporal_bindings/0/clock".to_owned())
            ),
            (
                Code::UnknownMember,
                Some("/temporal_bindings/0/clock/period".to_owned())
            ),
            (
                Code::MalformedWire,
                Some("/temporal_bindings/0/clock/kind".to_owned())
            ),
        ]
    );

    // TN-03: the clause selects a second admitted model type while the only
    // checked timed-model root still belongs to the original model subject.
    let mut missing = wire.clone();
    let alternate_model = missing["semantic_graph"]["nodes"][13]["node_id"].clone();
    let mut alternate_type = missing["semantic_graph"]["nodes"][26].clone();
    alternate_type["node_id"]["path"][2] = json!({"node":13});
    alternate_type["semantic_type"] = alternate_type["node_id"].clone();
    alternate_type["body"]["members"][0]["target"] = alternate_model.clone();
    alternate_type["dependencies"] = json!([alternate_model]);
    let alternate_type_id = alternate_type["node_id"].clone();
    missing["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(alternate_type);
    let mut alternate = missing["semantic_graph"]["nodes"][27].clone();
    alternate["node_id"]["path"][4] = json!("alternate");
    alternate["node_id"]["path"][8] = json!({"node":60});
    alternate["semantic_type"] = alternate_type_id;
    alternate["body"]["members"][0]["value"]["value"] = json!("alternate");
    let alternate_id = alternate["node_id"].clone();
    missing["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(alternate);
    missing["semantic_graph"]["nodes"][8]["body"]["arguments"][0]["target"] = alternate_id;
    fixture_support::settle(&mut missing);
    fixture_support::refresh_temporal_bindings(&mut missing);
    fixture_support::refresh_identity(&mut missing);
    let absent = refusal(&missing);
    assert_eq!(
        (
            absent.code,
            absent.cause,
            absent.loci[0].path.as_ref().map(ToString::to_string)
        ),
        (
            Code::InvalidPackage,
            Some(Cause::TimedNodeReference),
            Some(
                case("TN-03")["expected"]["pointer"]
                    .as_str()
                    .unwrap()
                    .replace("{clause-index}", "8")
            )
        )
    );
    assert_eq!(
        absent.locus,
        Some(
            serde_json::from_value(missing["semantic_graph"]["nodes"][8]["node_id"].clone())
                .unwrap()
        )
    );

    let mut wrong_name = wire.clone();
    wrong_name["semantic_graph"]["nodes"][8]["body"]["arguments"][1]["value"] =
        json!("model-steps");
    fixture_support::settle(&mut wrong_name);
    fixture_support::refresh_identity(&mut wrong_name);
    let wrong = refusal(&wrong_name);
    assert_eq!(
        (
            wrong.code,
            wrong.cause,
            wrong.loci[0].path.as_ref().map(ToString::to_string)
        ),
        (
            Code::InvalidPackage,
            Some(Cause::TimedNodeReference),
            Some("/semantic_graph/nodes/8/body".to_owned())
        )
    );

    let mut extra = wire.clone();
    let mut original = read_json(&fixtures_dir().join("positive-timed-model-nodes.json"))
        .expect("configured control");
    fixture_support::refresh_temporal_bindings(&mut original);
    fixture_support::refresh_identity_projection(&mut original);
    let mut row = original["temporal_bindings"][0].clone();
    row["declaration"] = extra["semantic_graph"]["nodes"][8]["node_id"].clone();
    extra["temporal_bindings"] = json!([row]);
    fixture_support::refresh_identity_projection(&mut extra);
    let actual = refusal(&extra);
    assert_eq!(
        (
            actual.code,
            actual.cause,
            actual.loci[0].path.as_ref().map(ToString::to_string)
        ),
        (
            Code::InvalidPackage,
            Some(Cause::TimedNodeReference),
            Some(
                case("TN-04")["expected"]["pointer"]
                    .as_str()
                    .unwrap()
                    .to_owned()
            )
        )
    );
    assert_eq!(
        actual.locus,
        Some(
            serde_json::from_value(extra["semantic_graph"]["nodes"][8]["node_id"].clone()).unwrap()
        )
    );

    // TN-07: a `timed` row for a clause over a model subject refuses at the row.
    let mut model_row = wire.clone();
    let mut row = case("TN-07")["row"].clone();
    row["declaration"] = model_row["semantic_graph"]["nodes"][8]["node_id"].clone();
    model_row["temporal_bindings"] = json!([row]);
    fixture_support::refresh_identity_projection(&mut model_row);
    let model_timed = refusal(&model_row);
    assert_eq!(
        (
            model_timed.code,
            model_timed.cause,
            model_timed.loci[0].path.as_ref().map(ToString::to_string)
        ),
        (
            Code::InvalidPackage,
            Some(Cause::TimedNodeReference),
            Some(
                case("TN-07")["expected"]["pointer"]
                    .as_str()
                    .unwrap()
                    .to_owned()
            )
        )
    );

    // TN-05: the source fixture retains its configured row and still admits.
    assert_eq!(admitted(&original).temporal_bindings().unwrap().len(), 1);
}
