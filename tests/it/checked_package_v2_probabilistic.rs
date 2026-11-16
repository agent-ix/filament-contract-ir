// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-476: the public strict reader admits the FSpec FR-640 probabilistic
//! nodes and refuses their defects at the value at fault.
//!
//! Every case adds one `probabilistic` node to the in-repo `v2_all_families()`
//! package, whose `Account` object type is the declaring model.

use crate::support::checked_package::{
    canonical, evidence_with_document, family_model_document, refresh_identity, v2_all_families,
};
use filament_contract_model::{
    CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use ix_trace_rs::trace;
use serde_json::{json, Value};

const PACKAGE: &str = "test/families";
const DEPOSIT: &str = "ix://test/families/Account/deposit";
const WITHDRAW: &str = "ix://test/families/Account/withdraw";

/// The family model document with the parameter `n: Integer` on `deposit`.
fn document() -> Value {
    let mut document = family_model_document();
    document["types"][0]["operations"][0]["params"] = json!([{
        "name": "n", "typeRef": "ix://filament/native/Integer",
        "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true},
    }]);
    document
}

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
            aggregate(vec![
                weight_row("deposit", weight),
                weight_row("withdraw", "1"),
            ]),
        )],
    )
}
fn weight_row(operation: &str, weight: &str) -> Value {
    aggregate(vec![
        binding("declaring", reference(account())),
        binding("operation", text(operation)),
        binding("weight", rational(weight)),
    ])
}
fn reward() -> Value {
    with(
        operation(),
        vec![
            binding("name", text("duration")),
            binding("value_type", reference(integer())),
            binding("expression", reference(id(&["reward", "function"]))),
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

/// A node of the family package with `body`, added by [`append`].
fn plain_node(label: &[&str], tag: &str, form: &str, semantic_type: Value, body: Value) -> Value {
    let node_id = id(label);
    json!({
        "schema_version": "filament.checked-semantic-graph/v2",
        "node_id": node_id, "semantic_type": semantic_type,
        "node_tag": tag, "semantic_form": form, "dependencies": [],
        "occurrences": [], "body": body,
    })
}

/// Adds `node` with one occurrence of `role` and its source-map row.
fn append(value: &mut Value, mut node: Value, role: &str) {
    node["occurrences"] = json!([{"role": role, "ordinal": 0}]);
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

/// The probabilistic node `label` of `form`.
fn probabilistic(label: &str, form: &str, body: Value) -> Value {
    let node_id = id(&["probabilistic", form, label]);
    let mut dependencies = Vec::new();
    collect_references(&body, &mut dependencies);
    dependencies.sort_by_key(Value::to_string);
    dependencies.dedup();
    json!({
        "schema_version": "filament.checked-semantic-graph/v2",
        "node_id": node_id.clone(), "semantic_type": node_id,
        "node_tag": "probabilistic", "semantic_form": form,
        "dependencies": dependencies, "body": body,
    })
}

/// The family package with the nodes a reward joins and `nodes` added.
fn assemble(nodes: Vec<(Value, &str)>) -> Value {
    let mut value = v2_all_families();
    append(
        &mut value,
        plain_node(
            &["builtin/rational"],
            "scalar_type",
            "rational",
            id(&["builtin/rational"]),
            aggregate(vec![]),
        ),
        "generated",
    );
    let parameter = plain_node(
        &["reward", "parameter"],
        "value",
        "parameter",
        integer(),
        aggregate(vec![
            binding("name", text("n")),
            binding("level", literal("integer", json!("0"))),
        ]),
    );
    append(&mut value, parameter, "expression");
    let mut function = plain_node(
        &["reward", "function"],
        "function",
        "pure_function",
        integer(),
        aggregate(vec![
            binding(
                "parameters",
                aggregate(vec![reference(id(&["reward", "parameter"]))]),
            ),
            binding("clause", text("body")),
        ]),
    );
    function["dependencies"] = json!([id(&["reward", "parameter"])]);
    function["owner"] = json!({"kind": "model", "identity": PACKAGE, "node": DEPOSIT});
    append(&mut value, function, "generated");
    for (node, role) in nodes {
        append(&mut value, node, role);
    }
    refresh_identity(&mut value);
    value
}

/// The package with one probabilistic node of `form` and `body` added, its
/// source-map row and its identity.
fn package(form: &str, body: Value, edit: impl FnOnce(&mut Value)) -> Value {
    let mut node = probabilistic("only", form, body);
    edit(&mut node);
    assemble(vec![(node, "declaration")])
}

/// The package with the probabilistic nodes `(label, form, body)`.
fn packages(nodes: Vec<(&str, &str, Value)>) -> Value {
    assemble(
        nodes
            .into_iter()
            .map(|(label, form, body)| (probabilistic(label, form, body), "declaration"))
            .collect(),
    )
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
        &evidence_with_document(value, PACKAGE, &document()),
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
        .iter()
        .rposition(|node| {
            node["node_tag"] == "probabilistic" || node["node_tag"] == "probabilistic_model"
        })
        .expect("a probabilistic node")
}

fn location(value: &Value, member: &str) -> String {
    let node = format!("/semantic_graph/nodes/{}", node_index(value));
    if member.is_empty() {
        node
    } else {
        format!("{node}/{member}")
    }
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
        Some(location(value, member).as_str()),
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
    // A delay distribution joins a `time dense` model, which the family
    // package has none of; the conformance suite admits one.
    for (form, body) in shapes()
        .into_iter()
        .filter(|(form, _)| *form != "delay_distribution")
    {
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

fn bad_target(member: usize) -> String {
    format!("body/members/{member}/value/target")
}

fn refuse_invalid(value: &Value, member: &str) {
    assert_refusal(
        value,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        member,
    );
}

/// FR-369-AC-6: an operation reference whose `declaring` node is absent or not
/// a model node, or which names no operation of it, refuses at the member.
#[trace("TC-476", "FR-369-AC-6")]
#[test]
fn the_public_reader_refuses_a_bad_operation_reference() {
    for (form, body) in shapes().into_iter().filter(|(form, _)| *form != "workload") {
        let swap = |body: &Value, name: &str, value: Value| {
            let mut body = body.clone();
            let members = body["members"].as_array_mut().expect("members");
            let member = members
                .iter_mut()
                .find(|member| member["name"] == name)
                .expect("member");
            member["value"] = value;
            body
        };
        // A node that exists but is no model node.
        refuse_invalid(
            &package(form, swap(&body, "declaring", reference(integer())), |_| {}),
            &bad_target(0),
        );
        refuse_invalid(
            &package(form, swap(&body, "operation", text("missing")), |_| {}),
            "body/members/1/value/value",
        );
    }
    let valid = package("random_parameter", random_parameter(&[("1", "9")]), |_| {});
    assert!(matches!(
        read(&valid),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// FR-369-AC-7: a `parameter` that names no parameter of the operation, and a
/// support value of another literal kind than the parameter's type, refuse.
#[trace("TC-476", "FR-369-AC-7")]
#[test]
fn the_public_reader_refuses_a_parameter_the_operation_lacks_or_a_mistyped_value() {
    let mut misnamed = random_parameter(&[("1", "9")]);
    misnamed["members"][2]["value"] = text("m");
    refuse_invalid(
        &package("random_parameter", misnamed, |_| {}),
        "body/members/2/value/value",
    );
    let mut mistyped = random_parameter(&[("1", "9"), ("2", "1")]);
    mistyped["members"][3]["value"]["members"][1]["members"][0]["value"] = text("two");
    refuse_invalid(
        &package("random_parameter", mistyped, |_| {}),
        "body/members/3/value/members/1/members/0/value",
    );
}

/// FR-369-AC-8: a workload that omits, repeats or misnames an operation of
/// its model, or whose members are out of order, refuses at the member.
#[trace("TC-476", "FR-369-AC-8")]
#[test]
fn the_public_reader_refuses_a_workload_that_does_not_cover_its_model() {
    let weights = |rows: Vec<Value>| {
        with(
            vec![
                binding("model", reference(account())),
                binding("name", text("Steady")),
            ],
            vec![binding("weights", aggregate(rows))],
        )
    };
    let rows = "body/members/2/value/members";
    refuse_invalid(
        &package(
            "workload",
            weights(vec![weight_row("deposit", "1")]),
            |_| {},
        ),
        rows,
    );
    refuse_invalid(
        &package(
            "workload",
            weights(vec![
                weight_row("deposit", "1"),
                weight_row("deposit", "1"),
                weight_row("withdraw", "1"),
            ]),
            |_| {},
        ),
        &format!("{rows}/1"),
    );
    refuse_invalid(
        &package(
            "workload",
            weights(vec![
                weight_row("withdraw", "1"),
                weight_row("deposit", "1"),
            ]),
            |_| {},
        ),
        &format!("{rows}/1"),
    );
    refuse_invalid(
        &package(
            "workload",
            weights(vec![weight_row("deposit", "1"), weight_row("nothing", "1")]),
            |_| {},
        ),
        &format!("{rows}/1/members/1/value/value"),
    );
    // The model of the workload must be a model node.
    refuse_invalid(
        &package(
            "workload",
            with(
                vec![
                    binding("model", reference(integer())),
                    binding("name", text("Steady")),
                ],
                vec![binding("weights", aggregate(vec![]))],
            ),
            |_| {},
        ),
        &bad_target(0),
    );
}

fn reward_package(edit: impl FnOnce(&mut Value)) -> Value {
    let mut value = package("reward", reward(), |_| {});
    edit(&mut value);
    refresh_identity(&mut value);
    value
}

fn node_by_label<'a>(value: &'a mut Value, label: &[&str]) -> &'a mut Value {
    let wanted = id(label);
    value["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .iter_mut()
        .find(|node| node["node_id"] == wanted)
        .expect("node")
}

/// FR-369-AC-9: a reward whose `value_type` is no integer or quantity type, or
/// whose `expression` is no clause function of the operation with exactly its
/// parameters, or whose function reads state, refuses at the member.
#[trace("TC-476", "FR-369-AC-9")]
#[test]
fn the_public_reader_refuses_a_reward_outside_rule_four() {
    let boolean = id(&["builtin/boolean"]);
    let mut typed = reward();
    typed["members"][3]["value"] = reference(boolean);
    refuse_invalid(&package("reward", typed, |_| {}), &bad_target(3));
    // A function owned by another operation of the model.
    refuse_invalid(
        &reward_package(|value| {
            node_by_label(value, &["reward", "function"])["owner"]["node"] = json!(WITHDRAW);
        }),
        &bad_target(4),
    );
    // A function that does not list the operation's parameters.
    refuse_invalid(
        &reward_package(|value| {
            node_by_label(value, &["reward", "function"])["body"]["members"][0]["value"]
                ["members"] = json!([]);
        }),
        &bad_target(4),
    );
    // A function whose result type is not the reward's `value_type`.
    refuse_invalid(
        &reward_package(|value| {
            node_by_label(value, &["reward", "function"])["semantic_type"] =
                id(&["builtin/rational"]);
        }),
        &bad_target(4),
    );
    // A function that reaches a pre-state read.
    refuse_invalid(
        &reward_package(|value| {
            let read = plain_node(
                &["reward", "pre_read"],
                "expression",
                "pre_read",
                integer(),
                aggregate(vec![]),
            );
            append(value, read, "generated");
            node_by_label(value, &["reward", "function"])["dependencies"]
                .as_array_mut()
                .expect("dependencies")
                .push(id(&["reward", "pre_read"]));
        }),
        &bad_target(4),
    );
}

/// FR-369-AC-10: a delay distribution on a model with no `time dense` source
/// refuses at the model.
#[trace("TC-476", "FR-369-AC-10")]
#[test]
fn the_public_reader_refuses_a_delay_on_an_untimed_model() {
    refuse_invalid(
        &package(
            "delay_distribution",
            delay("uniform", aggregate(vec![])),
            |_| {},
        ),
        &bad_target(0),
    );
}

/// FR-369-AC-11: a second definition of one parameter, workload name, reward
/// name or delay refuses `conflicting-definition` at the later node.
#[trace("TC-476", "FR-369-AC-11")]
#[test]
fn the_public_reader_refuses_a_second_definition() {
    for (form, body) in [
        ("random_parameter", random_parameter(&[("1", "9")])),
        ("workload", workload("1")),
        ("reward", reward()),
    ] {
        let value = packages(vec![("a", form, body.clone()), ("b", form, body)]);
        assert_refusal(
            &value,
            CheckedPackageRefusalCode::InvalidPackage,
            Some(CheckedPackageRefusalCause::ConflictingDefinition),
            "",
        );
    }
}

/// FR-369-AC-12: the admitted package exposes each declaration by node.
#[trace("TC-476", "FR-369-AC-12")]
#[test]
fn the_public_reader_exposes_the_typed_read() {
    let value = packages(vec![
        (
            "a",
            "random_parameter",
            random_parameter(&[("1", "9"), ("3", "7/10")]),
        ),
        ("b", "workload", workload("0")),
        ("c", "reward", reward()),
    ]);
    let CheckedPackageV2ReadResult::Admitted(package) = read(&value) else {
        panic!("expected admission");
    };
    let model: filament_contract_model::CheckedNodeId =
        serde_json::from_value(account()).expect("model id");
    let parameters = package.random_parameters(&model, "deposit");
    assert_eq!(parameters.len(), 1);
    let probabilities: Vec<&str> = parameters[0]
        .support
        .iter()
        .map(|row| &*row.probability)
        .collect();
    assert_eq!(probabilities, ["90/97", "7/97"]);
    assert_eq!(parameters[0].parameter, "n");
    let rewards = package.rewards(&model, "deposit");
    assert_eq!(rewards.len(), 1);
    assert_eq!(rewards[0].name, "duration");
    let workloads = package.workloads(&model);
    assert_eq!(workloads[0].name, "Steady");
    assert_eq!(workloads[0].weights.len(), 2);
    assert_eq!(workloads[0].weights[0].weight, "0");
    assert!(package.random_parameters(&model, "withdraw").is_empty());
}
