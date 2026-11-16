// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-476: the public strict reader admits the FSpec FR-640 probabilistic
//! nodes and refuses their defects at the value at fault.
//!
//! Every case adds one `probabilistic` node to the in-repo `v2_all_families()`
//! package, whose `Account` object type is the declaring model.

use crate::support::checked_package::{canonical, evidence_for, refresh_identity, v2_all_families};
use filament_contract_model::{
    CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use ix_trace_rs::trace;
use serde_json::{json, Value};

const PACKAGE: &str = "test/families";

fn id(path: &[&str]) -> Value {
    json!({"package": PACKAGE, "path": path, "ordinal": 0})
}
fn account() -> Value {
    id(&["model", "object_type", "ix://test/families/Account"])
}
fn integer() -> Value {
    id(&["builtin/integer"])
}
fn binding(name: &str, value: Value) -> Value {
    json!({"term": "binding", "name": name, "value": value})
}
fn aggregate(members: Vec<Value>) -> Value {
    json!({"term": "aggregate", "members": members})
}
fn reference(target: Value) -> Value {
    json!({"term": "reference", "target": target})
}
fn literal(kind: &str, value: Value) -> Value {
    let type_id = match kind {
        "text" => id(&["fixture/a1a1"]),
        "rational" => id(&["builtin/rational"]),
        _ => integer(),
    };
    json!({"term": "literal", "type": type_id, "value_kind": kind, "value": value})
}
fn text(value: &str) -> Value {
    literal("text", json!(value))
}
fn rational(value: &str) -> Value {
    literal("rational", json!(value))
}
fn operation() -> Vec<Value> {
    vec![
        binding("declaring", reference(account())),
        binding("operation", text("deposit")),
    ]
}
fn with(mut head: Vec<Value>, tail: Vec<Value>) -> Value {
    head.extend(tail);
    aggregate(head)
}
fn support(rows: &[(&str, &str)]) -> Value {
    aggregate(
        rows.iter()
            .map(|(value, weight)| {
                aggregate(vec![
                    binding("value", literal("integer", json!(value))),
                    binding("weight", rational(weight)),
                ])
            })
            .collect(),
    )
}
fn random_parameter(rows: &[(&str, &str)]) -> Value {
    with(
        operation(),
        vec![
            binding("parameter", text("n")),
            binding("support", support(rows)),
        ],
    )
}
fn workload(weight: &str) -> Value {
    with(
        vec![
            binding("model", reference(account())),
            binding("name", text("Steady")),
        ],
        vec![binding(
            "weights",
            aggregate(vec![aggregate(vec![
                binding("declaring", reference(account())),
                binding("operation", text("deposit")),
                binding("weight", rational(weight)),
            ])]),
        )],
    )
}
fn reward() -> Value {
    with(
        operation(),
        vec![
            binding("name", text("duration")),
            binding("value_type", reference(integer())),
            binding("expression", reference(id(&["Example", "Function"]))),
        ],
    )
}
fn delay(family: &str, parameters: Value) -> Value {
    with(
        operation(),
        vec![
            binding("family", text(family)),
            binding("parameters", parameters),
        ],
    )
}

/// The package with one probabilistic node of `form` and `body` added, its
/// source-map row and its identity.
fn package(form: &str, body: Value, edit: impl FnOnce(&mut Value)) -> Value {
    let mut value = v2_all_families();
    let node_id = id(&["probabilistic", form]);
    let mut dependencies = Vec::new();
    collect_references(&body, &mut dependencies);
    dependencies.sort_by_key(Value::to_string);
    dependencies.dedup();
    let mut node = json!({
        "schema_version": "filament.checked-semantic-graph/v2",
        "node_id": node_id.clone(), "semantic_type": node_id.clone(),
        "node_tag": "probabilistic", "semantic_form": form,
        "dependencies": dependencies,
        "occurrences": [{"role": "declaration", "ordinal": 0}],
        "body": body,
    });
    edit(&mut node);
    let rational_type = json!({
        "schema_version": "filament.checked-semantic-graph/v2",
        "node_id": id(&["builtin/rational"]), "semantic_type": id(&["builtin/rational"]),
        "node_tag": "scalar_type", "semantic_form": "rational", "dependencies": [],
        "occurrences": [{"role": "generated", "ordinal": 0}],
        "body": aggregate(vec![]),
    });
    for (node, role) in [(rational_type, "generated"), (node, "declaration")] {
        let node_id = node["node_id"].clone();
        value["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(node);
        let end = value["source_map"].as_array().expect("source map").len() as u64 + 1;
        value["source_map"]
            .as_array_mut()
            .expect("source map")
            .push(json!({
                "node_id": node_id, "ordinal": 0, "role": role,
                "regions": [{"end": end + 1, "start": end,
                    "source": {"authority": "agent-ix", "identity": "filament.fixture.source/v1"}}],
            }));
    }
    refresh_identity(&mut value);
    value
}

fn collect_references(term: &Value, found: &mut Vec<Value>) {
    match term {
        Value::Object(map) if map.get("term") == Some(&json!("reference")) => {
            found.push(map["target"].clone());
        }
        Value::Object(map) => map
            .values()
            .for_each(|inner| collect_references(inner, found)),
        Value::Array(items) => items
            .iter()
            .for_each(|inner| collect_references(inner, found)),
        _ => {}
    }
}

fn read(value: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(value),
    )
}

fn refused(value: &Value) -> CheckedPackageRefusal {
    match read(value) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected a refusal, read {other:?}"),
    }
}

fn node_index(value: &Value) -> usize {
    value["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1
}

fn assert_refusal(
    value: &Value,
    code: CheckedPackageRefusalCode,
    cause: Option<CheckedPackageRefusalCause>,
    member: &str,
) {
    let refusal = refused(value);
    assert_eq!(refusal.code, code, "{refusal:?}");
    assert_eq!(refusal.cause, cause, "{refusal:?}");
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{}/{member}", node_index(value)).as_str()),
        "{refusal:?}"
    );
}

fn shapes() -> Vec<(&'static str, Value)> {
    vec![
        (
            "random_parameter",
            random_parameter(&[("1", "9"), ("3", "7/10")]),
        ),
        ("workload", workload("1")),
        ("reward", reward()),
        ("delay_distribution", delay("uniform", aggregate(vec![]))),
        (
            "delay_distribution",
            delay(
                "exponential",
                aggregate(vec![binding("rate", rational("1/2"))]),
            ),
        ),
    ]
}

/// FR-369-AC-1: the strict reader admits a package holding each form.
#[trace("TC-476", "FR-369-AC-1")]
#[test]
fn the_public_reader_admits_each_probabilistic_form() {
    for (form, body) in shapes() {
        let value = package(form, body, |_| {});
        match read(&value) {
            CheckedPackageV2ReadResult::Admitted(_) => {}
            other => panic!("{form}: expected admission, read {other:?}"),
        }
    }
}

/// FR-369-AC-2: a missing or extra member refuses at the member.
#[trace("TC-476", "FR-369-AC-2")]
#[test]
fn the_public_reader_refuses_a_missing_or_extra_member() {
    for (form, body) in shapes() {
        let mut missing = body.clone();
        missing["members"].as_array_mut().expect("members").pop();
        assert_refusal(
            &package(form, missing, |_| {}),
            CheckedPackageRefusalCode::MalformedWire,
            None,
            "body/members",
        );
        let mut extra = body.clone();
        extra["members"]
            .as_array_mut()
            .expect("members")
            .push(binding("extra", text("x")));
        assert_refusal(
            &package(form, extra, |_| {}),
            CheckedPackageRefusalCode::MalformedWire,
            None,
            "body/members",
        );
    }
    let owned = package("workload", workload("1"), |node| {
        node["owner"] = json!({"kind": "model_root", "identity": PACKAGE});
    });
    assert_refusal(
        &owned,
        CheckedPackageRefusalCode::MalformedWire,
        None,
        "owner",
    );
}

/// FR-369-AC-3: a value outside FR-640's rules refuses `invalid-value`.
#[trace("TC-476", "FR-369-AC-3")]
#[test]
fn the_public_reader_refuses_a_value_outside_the_form_rules() {
    let invalid = CheckedPackageRefusalCause::InvalidValue;
    assert_refusal(
        &package("workload", workload("-1"), |_| {}),
        CheckedPackageRefusalCode::InvalidPackage,
        Some(invalid),
        "body/members/2/value/members/0/members/2/value/value",
    );
    assert_refusal(
        &package(
            "delay_distribution",
            delay("uniform", aggregate(vec![binding("rate", rational("1"))])),
            |_| {},
        ),
        CheckedPackageRefusalCode::InvalidPackage,
        Some(invalid),
        "body/members/3/value/members",
    );
    assert_refusal(
        &package(
            "random_parameter",
            random_parameter(&[("1", "2/2")]),
            |_| {},
        ),
        CheckedPackageRefusalCode::InvalidPackage,
        Some(invalid),
        "body/members/3/value/members/0/members/1/value/value",
    );
}

/// FR-369-AC-4: an unknown tag keeps the existing refusal.
#[trace("TC-476", "FR-369-AC-4")]
#[test]
fn the_public_reader_keeps_the_unknown_tag_refusal() {
    let value = package("workload", workload("1"), |node| {
        node["node_tag"] = json!("probabilistic_model");
    });
    assert_refusal(
        &value,
        CheckedPackageRefusalCode::UnsupportedNodeTag,
        None,
        "node_tag",
    );
}
