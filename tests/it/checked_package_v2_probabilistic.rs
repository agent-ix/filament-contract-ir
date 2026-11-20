// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-476: the public strict reader admits the FSpec FR-640 probabilistic
//! nodes and refuses their defects at the value at fault.
//!
//! Every case adds one `probabilistic` node to the in-repo `v2_all_families()`
//! package, whose `Account` object type is the declaring model.

use crate::support::checked_package::{
    canonical, evidence_with_document, family_key, family_model_document, model_owner, node_id,
    nominal_fixture_members, nominal_package, refresh_identity, v2_all_families,
};
use filament_contract_model::{
    CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use ix_trace_rs::trace;
use serde_json::{json, Value};

const PACKAGE: &str = "test/families";
const DEPOSIT: &str = "ix://test/families/Account/deposit";
const TIER: &str = "ix://test/families/Tier";
const OTHER: &str = "ix://test/families/Other";
const MIXED: &str = "ix://test/families/Mixed";
const WITHDRAW: &str = "ix://test/families/Account/withdraw";
/// The parameters of `withdraw`, in declared order.
const PARAMETERS: [&str; 11] = ["d", "t", "r", "r2", "l", "lb", "ls", "ld", "lk", "lq", "c"];

/// The family model document with the parameter `n: Integer` on `deposit`,
/// the parameters `d: Millis` (a value type `Int[1, 12]`) and
/// `t: Quantity<ms>` on `withdraw`.
fn document() -> Value {
    document_with(0)
}

/// [`document`] with `chain` supertypes without operations above `Base`.
fn document_with(chain: usize) -> Value {
    let mut document = family_model_document();
    let one = json!({"lower": 1, "upper": 1, "ordered": false, "unique": true});
    document["types"][0]["operations"][0]["params"] = json!([{
        "name": "n", "typeRef": "ix://filament/native/Integer", "multiplicity": one,
    }]);
    document["types"][0]["operations"][1]["params"] = json!([
        {"name": "d", "typeRef": "ix://test/families/Millis", "multiplicity": one},
        {"name": "t", "typeRef": "ix://filament/unit/ms", "multiplicity": one},
        {"name": "r", "typeRef": "ix://test/families/Ratio", "multiplicity": one},
        {"name": "r2", "typeRef": "ix://test/families/RatioFrom2", "multiplicity": one},
        {"name": "l", "typeRef": "ix://test/families/Label", "multiplicity": one},
        {"name": "lb", "typeRef": "ix://test/families/LabelBytes", "multiplicity": one},
        {"name": "ls", "typeRef": "ix://test/families/LabelScalars", "multiplicity": one},
        {"name": "ld", "typeRef": "ix://test/families/LabelNfd", "multiplicity": one},
        {"name": "lk", "typeRef": "ix://test/families/LabelNfkc", "multiplicity": one},
        {"name": "lq", "typeRef": "ix://test/families/LabelNfkd", "multiplicity": one},
        {"name": "c", "typeRef": "ix://test/families/Money", "multiplicity": one},
    ]);
    let above = |index: usize| format!("ix://{PACKAGE}/Up{index}");
    let base_supers = if chain == 0 {
        json!([])
    } else {
        json!([above(0)])
    };
    let mut declared = vec![
        ("Base".to_owned(), base_supers, vec!["ping"]),
        (
            "Mid".to_owned(),
            json!([format!("ix://{PACKAGE}/Base")]),
            vec!["ping", "pong"],
        ),
        (
            "Leaf".to_owned(),
            json!([format!("ix://{PACKAGE}/Mid")]),
            vec!["run"],
        ),
    ];
    for index in 0..chain {
        let supers = if index + 1 < chain {
            json!([above(index + 1)])
        } else {
            json!([])
        };
        declared.push((format!("Up{index}"), supers, vec![]));
    }
    for (name, supertypes, operations) in declared {
        let operations: Vec<Value> = operations
            .into_iter()
            .map(|operation| {
                let params = if (name.as_str(), operation) == ("Leaf", "run") {
                    json!([
                        {"name": "flag", "typeRef": "ix://filament/native/Boolean",
                         "multiplicity": one},
                        {"name": "tier", "typeRef": TIER, "multiplicity": one},
                        {"name": "mixed", "typeRef": MIXED, "multiplicity": one},
                    ])
                } else {
                    json!([])
                };
                json!({"identity": format!("ix://{PACKAGE}/{name}/{operation}"), "params": params})
            })
            .collect();
        document["types"]
            .as_array_mut()
            .expect("types")
            .push(json!({
                "identity": format!("ix://{PACKAGE}/{name}"), "displayName": name,
                "kind": {"module": PACKAGE, "name": "entity"},
                "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
                "supertypes": supertypes, "fields": [], "relationships": [],
                "operations": operations,
            }));
    }
    document["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind": {"module": PACKAGE, "name": "variant"},
            "construct": {"meaning": "filament.meaning.model.variant-type/v1"},
        }));
    // `Tier` and `Other`: enumerations of the cases `Gold` and `Silver`.
    for identity in [TIER, OTHER] {
        document["types"]
            .as_array_mut()
            .expect("types")
            .push(json!({
                "identity": identity, "displayName": identity,
                "kind": {"module": PACKAGE, "name": "variant"},
                "variants": [
                    {"identity": format!("{identity}/Gold"), "name": "Gold"},
                    {"identity": format!("{identity}/Silver"), "name": "Silver"},
                ],
            }));
    }
    // `Mixed`: a nullary case and a case with a payload, so a union.
    document["types"]
        .as_array_mut()
        .expect("types")
        .push(json!({
            "identity": MIXED, "displayName": "Mixed",
            "kind": {"module": PACKAGE, "name": "variant"},
            "variants": [
                {"identity": format!("{MIXED}/A"), "name": "A"},
                {"identity": format!("{MIXED}/B"), "name": "B",
                 "payloadType": "ix://filament/native/Integer"},
            ],
        }));
    document["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind": {"module": PACKAGE, "name": "millis"},
            "construct": {"meaning": "filament.meaning.model.value-type/v1"},
        }));
    document["types"]
        .as_array_mut()
        .expect("types")
        .push(json!({
            "identity": "ix://test/families/Millis", "displayName": "Millis",
            "kind": {"module": PACKAGE, "name": "millis"},
            "roles": [], "extensions": [], "unknownPolicy": "reject", "scalar": "integer",
            "constraints": [
                {"keyword": "min", "operands": {"value": 1}},
                {"keyword": "max", "operands": {"value": 12}},
            ],
        }));
    // `Rational[1, 3; 1, 4]`, `Rational[1, 3; 2, 4]` and `Text[2, 3; profile]`.
    for (name, min_denominator) in [("Ratio", 1), ("RatioFrom2", 2)] {
        let ratio = format!("ix://test/families/{name}");
        let bounds = [
            ("minNumerator", 1),
            ("maxNumerator", 3),
            ("minDenominator", min_denominator),
            ("maxDenominator", 4),
        ]
        .map(|(keyword, value)| {
            json!({"keyword": keyword, "appliesTo": ratio, "operands": {"value": value}})
        });
        document["types"]
            .as_array_mut()
            .expect("types")
            .push(json!({
                "identity": ratio, "displayName": name,
                "kind": {"module": PACKAGE, "name": "millis"},
                "roles": [], "extensions": [], "unknownPolicy": "reject", "scalar": "rational",
                "constraints": bounds,
            }));
    }
    for (name, profile) in [
        ("Label", "nfc"),
        ("LabelBytes", "binary-utf8"),
        ("LabelScalars", "unicode-scalars"),
        ("LabelNfd", "nfd"),
        ("LabelNfkc", "nfkc"),
        ("LabelNfkd", "nfkd"),
    ] {
        document["types"]
            .as_array_mut()
            .expect("types")
            .push(json!({
                "identity": format!("ix://test/families/{name}"), "displayName": name,
                "kind": {"module": PACKAGE, "name": "millis"},
                "roles": [], "extensions": [], "unknownPolicy": "reject", "scalar": "string",
                "constraints": [
                    {"keyword": "minLength", "operands": {"value": 2}},
                    {"keyword": "maxLength", "operands": {"value": 3}},
                    {"keyword": "format", "operands": {"name": format!("text-profile:{profile}")}},
                ],
            }));
    }
    // `Quantity<ms, Rational[1, 3; 1, 4]>`.
    let money = "ix://test/families/Money";
    document["types"]
        .as_array_mut()
        .expect("types")
        .push(json!({
            "identity": money, "displayName": "Money",
            "kind": {"module": PACKAGE, "name": "millis"},
            "roles": [], "extensions": [], "unknownPolicy": "reject", "scalar": "rational",
            "unit": "ix://filament/unit/ms",
            "constraints": [
                {"keyword": "minNumerator", "appliesTo": money, "operands": {"value": 1}},
                {"keyword": "maxNumerator", "appliesTo": money, "operands": {"value": 3}},
                {"keyword": "minDenominator", "appliesTo": money, "operands": {"value": 1}},
                {"keyword": "maxDenominator", "appliesTo": money, "operands": {"value": 4}},
            ],
        }));
    document
}

fn tier_id() -> Value {
    id(&["model", "variant_type", TIER])
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
    {
        let mut node = json!({
            "schema_version": "filament.checked-semantic-graph/v2",
            "node_id": node_id, "semantic_type": semantic_type,
            "node_tag": tag, "semantic_form": form, "dependencies": [],
            "occurrences": [], "body": body,
        });
        crate::support::checked_package::carry_check_mode(&mut node);
        node
    }
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

/// The unit `ms` and the nominal nodes beside it, added to `value`; the id of
/// the unit node.
fn add_units(value: &mut Value) -> Value {
    let mut members = nominal_fixture_members();
    let unit = members
        .iter_mut()
        .find(|(preimage, _)| preimage["version"] == "filament.unit-node/v1")
        .expect("unit");
    unit.0["qualified_declaration"] = json!(["ms"]);
    unit.1 = "ms".to_owned();
    let mut second = unit.0.clone();
    second["qualified_declaration"] = json!(["s"]);
    second["target_unit_node_id"] = node_id("ms");
    second["scale"]["numerator"] = json!("1000");
    members.push((second, "s".to_owned()));
    let nominal = nominal_package(&members);
    value["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend(
            nominal["semantic_graph"]["nodes"]
                .as_array()
                .expect("nodes")
                .iter()
                .cloned(),
        );
    value["source_map"]
        .as_array_mut()
        .expect("source map")
        .extend(
            nominal["source_map"]
                .as_array()
                .expect("map")
                .iter()
                .cloned(),
        );
    node_id("ms")
}

/// The family package with the nodes a reward joins and `nodes` added.
fn assemble(nodes: Vec<(Value, &str)>) -> Value {
    assemble_with(0, nodes)
}

fn assemble_with(chain: usize, nodes: Vec<(Value, &str)>) -> Value {
    assemble_models(chain, &[], nodes)
}

/// [`assemble_with`] with the model nodes `extra` added to the family's.
fn assemble_models(chain: usize, extra: &[&str], nodes: Vec<(Value, &str)>) -> Value {
    let mut value = v2_all_families();
    add_units(&mut value);
    let mut names = vec!["Base".to_owned(), "Mid".to_owned(), "Leaf".to_owned()];
    names.extend((0..chain).map(|index| format!("Up{index}")));
    names.extend(extra.iter().map(|name| (*name).to_owned()));
    for name in names {
        let identity = format!("ix://{PACKAGE}/{name}");
        let mut model = plain_node(
            &["model", "object_type", &identity],
            "model",
            "object_type",
            id(&["model", "object_type", &identity]),
            aggregate(vec![]),
        );
        model["dependencies"] = json!([id(&[&family_key("aaaa")])]);
        model["owner"] = model_owner(PACKAGE, &identity);
        append(&mut value, model, "generated");
    }
    for identity in [TIER, OTHER, MIXED] {
        let mut declaration = plain_node(
            &["model", "variant_type", identity],
            "model",
            "variant_type",
            id(&["model", "variant_type", identity]),
            aggregate(vec![]),
        );
        declaration["owner"] = model_owner(PACKAGE, identity);
        append(&mut value, declaration, "generated");
    }
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
            binding("clause", text("reward")),
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

/// The name of the `index`th operation of the `Wide` model.
fn wide_operation(index: usize) -> String {
    format!("op{index:04}")
}

/// [`document`] with the model `Wide`, which declares `rows` operations.
fn wide_document(rows: usize) -> Value {
    let mut document = document();
    let operations: Vec<Value> = (0..rows)
        .map(|index| {
            json!({"identity": format!("ix://{PACKAGE}/Wide/{}", wide_operation(index)),
                "params": []})
        })
        .collect();
    document["types"]
        .as_array_mut()
        .expect("types")
        .push(json!({
            "identity": format!("ix://{PACKAGE}/Wide"), "displayName": "Wide",
            "kind": {"module": PACKAGE, "name": "entity"},
            "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
            "supertypes": [], "fields": [], "relationships": [],
            "operations": operations,
        }));
    document
}

/// A package whose workload covers the `rows` operations of the model `Wide`.
fn wide_workload(rows: usize) -> Value {
    let wide = id(&["model", "object_type", &format!("ix://{PACKAGE}/Wide")]);
    let weights: Vec<Value> = (0..rows)
        .map(|index| {
            aggregate(vec![
                binding("declaring", reference(wide.clone())),
                binding("operation", text(&wide_operation(index))),
                binding("weight", rational("1")),
            ])
        })
        .collect();
    let body = with(
        vec![
            binding("model", reference(wide)),
            binding("name", text("Steady")),
        ],
        vec![binding("weights", aggregate(weights))],
    );
    assemble_models(
        0,
        &["Wide"],
        vec![(probabilistic("only", "workload", body), "declaration")],
    )
}

/// A workload of 300 rows over a model-owner node, for the whole-read
/// accounting registry: its `named` buffer alone is over the 4096-byte unpaid
/// allowance if it is not charged.
pub(crate) fn accounting_fixture() -> (Value, &'static str, Value) {
    accounting_fixture_of(300)
}

/// [`accounting_fixture`] with a workload of `rows` rows.
pub(crate) fn accounting_fixture_of(rows: usize) -> (Value, &'static str, Value) {
    (wide_workload(rows), PACKAGE, wide_document(rows))
}

/// FR-038-AC-511: a workload over 300 operations of a model-owner node is
/// admitted, and the buffer its read keeps for the named operations is
/// charged to the read-storage budget before it is made, one request of
/// 16 bytes a row (`&str`), at a `probabilistic` node.
#[trace("TC-472", "FR-038-AC-511", "FR-358-AC-12", "FR-369-AC-27")]
#[test]
fn a_wide_model_owner_workload_is_admitted_and_its_named_buffer_charged() {
    let (value, package, document) = accounting_fixture();
    let evidence = evidence_with_document(&value, package, &document);
    assert!(matches!(
        CheckedPackageV2::read(
            &canonical(&value),
            CheckedPackageReadLimits::bounded(),
            &evidence
        ),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let evidence = evidence_with_document(&value, package, &document);
    let requests = crate::checked_package_v2_source_objects::probabilistic::graph_requests_of(
        &canonical(&value),
        &evidence,
    );
    let name = u128::try_from(std::mem::size_of::<&str>()).expect("bytes") * 300;
    let named = requests
        .iter()
        .filter(|(form, amount)| *form == "probabilistic" && *amount == name)
        .count();
    assert_eq!(named, 1, "{requests:?}");
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
    let probabilities: Vec<(&str, &str)> = parameters[0]
        .support
        .iter()
        .map(|row| (&*row.probability.numerator, &*row.probability.denominator))
        .collect();
    assert_eq!(probabilities, [("90", "97"), ("7", "97")]);
    assert_eq!(parameters[0].parameter, "n");
    let rewards = package.rewards(&model, "deposit");
    assert_eq!(rewards.len(), 1);
    assert_eq!(rewards[0].name, "duration");
    let typed: filament_contract_model::CheckedNodeId =
        serde_json::from_value(integer()).expect("type id");
    let function: filament_contract_model::CheckedNodeId =
        serde_json::from_value(id(&["reward", "function"])).expect("function id");
    assert_eq!(rewards[0].value_type, &typed);
    assert_eq!(rewards[0].function, &function);
    let workloads = package.workloads(&model);
    assert_eq!(workloads[0].name, "Steady");
    let weights: Vec<(&str, &str, &str)> = workloads[0]
        .weights
        .iter()
        .map(|row| {
            (
                row.operation,
                &*row.weight.numerator,
                &*row.weight.denominator,
            )
        })
        .collect();
    assert_eq!(weights, [("deposit", "0", "1"), ("withdraw", "1", "1")]);
    assert!(workloads[0]
        .weights
        .iter()
        .all(|row| row.declaring == &model));
    assert!(package.random_parameters(&model, "withdraw").is_empty());
}

fn typed_literal(type_id: Value, kind: &str, value: &str) -> Value {
    json!({"term": "literal", "type": type_id, "value_kind": kind, "value": value})
}

/// A `random_parameter` of `parameter` on `operation` of the account whose
/// support values are `values`, each at weight 1.
fn parameter_on(operation: &str, parameter: &str, values: Vec<Value>) -> Value {
    with(
        vec![
            binding("declaring", reference(account())),
            binding("operation", text(operation)),
        ],
        vec![
            binding("parameter", text(parameter)),
            binding(
                "support",
                aggregate(
                    values
                        .into_iter()
                        .map(|value| {
                            aggregate(vec![
                                binding("value", value),
                                binding("weight", rational("1")),
                            ])
                        })
                        .collect(),
                ),
            ),
        ],
    )
}

fn millis() -> Vec<Value> {
    ["1", "3", "12"]
        .into_iter()
        .map(|value| literal("integer", json!(value)))
        .collect()
}

/// A reward over `type_id`, its function typed the same way.
fn reward_over(type_id: Value) -> Value {
    let body = with(
        operation(),
        vec![
            binding("name", text("duration")),
            binding("value_type", reference(type_id.clone())),
            binding("expression", reference(id(&["reward", "function"]))),
        ],
    );
    let mut value = package("reward", body, |_| {});
    node_by_label(&mut value, &["reward", "function"])["semantic_type"] = type_id;
    refresh_identity(&mut value);
    value
}

fn admitted(value: &Value) -> CheckedPackageV2 {
    match read(value) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("expected admission, read {other:?}"),
    }
}

/// FR-369-AC-13: a random parameter of a declared value type or a quantity
/// type, and a reward of a quantity or integer-range type, are admitted.
#[trace("TC-476", "FR-369-AC-13")]
#[test]
fn the_public_reader_admits_declared_and_quantity_typed_parameters_and_rewards() {
    let model: filament_contract_model::CheckedNodeId =
        serde_json::from_value(account()).expect("model id");
    // `d: Millis`, a declared `Int[1, 12]`.
    let value = package(
        "random_parameter",
        parameter_on("withdraw", "d", millis()),
        |_| {},
    );
    let package = admitted(&value);
    assert_eq!(
        package.random_parameters(&model, "withdraw")[0]
            .support
            .len(),
        3
    );
    // `t: Quantity<ms>`.
    let milliseconds = vec![typed_literal(node_id("ms"), "quantity", "5")];
    admitted(&package_of("withdraw", "t", milliseconds));
    // A reward whose type is a unit, and one whose type is an integer range.
    for type_id in [node_id("ms"), node_id(&family_key("cccc"))] {
        let value = reward_over(type_id.clone());
        let package = admitted(&value);
        assert_eq!(
            serde_json::to_value(package.rewards(&model, "deposit")[0].value_type).expect("id"),
            type_id
        );
    }
}

fn package_of(operation: &str, parameter: &str, values: Vec<Value>) -> Value {
    package(
        "random_parameter",
        parameter_on(operation, parameter, values),
        |_| {},
    )
}

/// FR-369-AC-14: a support value of another literal kind than the declared
/// type, or a quantity in another unit than the parameter's, refuses.
#[trace("TC-476", "FR-369-AC-14")]
#[test]
fn the_public_reader_refuses_a_support_value_of_another_kind_or_unit() {
    let at = "body/members/3/value/members/0/members/0/value";
    refuse_invalid(&package_of("withdraw", "d", vec![text("one")]), at);
    refuse_invalid(
        &package_of(
            "withdraw",
            "t",
            vec![typed_literal(node_id("s"), "quantity", "5")],
        ),
        at,
    );
    refuse_invalid(
        &package_of("withdraw", "t", vec![literal("integer", json!("5"))]),
        at,
    );
}

fn workload_of(model: &str, rows: &[(&str, &str)]) -> Value {
    let declaring = |name: &str| id(&["model", "object_type", &format!("ix://{PACKAGE}/{name}")]);
    with(
        vec![
            binding("model", reference(declaring(model))),
            binding("name", text("Steady")),
        ],
        vec![binding(
            "weights",
            aggregate(
                rows.iter()
                    .map(|(owner, operation)| {
                        aggregate(vec![
                            binding("declaring", reference(declaring(owner))),
                            binding("operation", text(operation)),
                            binding("weight", rational("1")),
                        ])
                    })
                    .collect(),
            ),
        )],
    )
}

/// FR-369-AC-15: a workload covers the model's own and inherited operations,
/// through supertypes of supertypes; an omitted own operation, an omitted
/// inherited one and an operation of an unrelated model each refuse.
#[trace("TC-476", "FR-369-AC-15")]
#[test]
fn the_public_reader_checks_own_and_transitively_inherited_operations() {
    let whole = [("Base", "ping"), ("Leaf", "run"), ("Mid", "pong")];
    admitted(&package("workload", workload_of("Leaf", &whole), |_| {}));
    let rows = "body/members/2/value/members";
    // The model's own operation is omitted although it has supertypes.
    refuse_invalid(
        &package(
            "workload",
            workload_of("Leaf", &[("Base", "ping"), ("Mid", "pong")]),
            |_| {},
        ),
        rows,
    );
    // The grandparent's operation is omitted.
    refuse_invalid(
        &package(
            "workload",
            workload_of("Leaf", &[("Leaf", "run"), ("Mid", "pong")]),
            |_| {},
        ),
        rows,
    );
    // A model that is no ancestor of Leaf.
    let mut foreign = workload_of("Leaf", &whole);
    foreign["members"][2]["value"]["members"][1]["members"][0]["value"] = reference(account());
    refuse_invalid(
        &package("workload", foreign, |_| {}),
        &format!("{rows}/1/members/0/value/target"),
    );
    // Mid does not inherit from Leaf.
    refuse_invalid(
        &package(
            "workload",
            workload_of("Mid", &[("Base", "ping"), ("Leaf", "run"), ("Mid", "pong")]),
            |_| {},
        ),
        &format!("{rows}/1/members/0/value/target"),
    );
}

/// FR-369-AC-16: FR-405's rule that a reward reads only the operation's
/// parameters and constants, positively: every other kind a function
/// reaches refuses, among them the seven kinds that read state.
#[trace("TC-476", "FR-369-AC-16")]
#[test]
fn the_public_reader_admits_only_the_reads_fr_405_allows() {
    for (tag, form) in [
        ("expression", "pre_read"),
        ("expression", "presence_read"),
        ("expression", "value_read"),
        ("expression", "deref"),
        ("expression", "query"),
        ("expression", "reachability"),
        ("state", "snapshot"),
    ] {
        let value = reward_package(|value| {
            let read = plain_node(&["reward", "read"], tag, form, integer(), aggregate(vec![]));
            append(value, read, "generated");
            node_by_label(value, &["reward", "function"])["dependencies"]
                .as_array_mut()
                .expect("dependencies")
                .push(id(&["reward", "read"]));
        });
        refuse_invalid(&value, &bad_target(4));
    }
    // A read through a model member: the function depends on the model.
    refuse_invalid(
        &reward_package(|value| {
            node_by_label(value, &["reward", "function"])["dependencies"]
                .as_array_mut()
                .expect("dependencies")
                .push(account());
        }),
        &bad_target(4),
    );
    // A parameter that is not the operation's.
    refuse_invalid(
        &reward_package(|value| {
            let other = plain_node(
                &["reward", "other"],
                "value",
                "parameter",
                integer(),
                aggregate(vec![
                    binding("name", text("other")),
                    binding("level", literal("integer", json!("0"))),
                ]),
            );
            append(value, other, "expression");
            node_by_label(value, &["reward", "function"])["dependencies"]
                .as_array_mut()
                .expect("dependencies")
                .push(id(&["reward", "other"]));
        }),
        &bad_target(4),
    );
    // A constant is allowed.
    let value = reward_package(|value| {
        let constant = plain_node(
            &["reward", "constant"],
            "value",
            "literal",
            integer(),
            literal("integer", json!("1")),
        );
        append(value, constant, "generated");
        node_by_label(value, &["reward", "function"])["dependencies"]
            .as_array_mut()
            .expect("dependencies")
            .push(id(&["reward", "constant"]));
    });
    admitted(&value);
}

/// FR-369-AC-13: a reward over a compound-unit quantity type is admitted.
#[trace("TC-476", "FR-369-AC-13")]
#[test]
fn the_public_reader_admits_a_reward_over_a_compound_unit() {
    let compound = id(&["reward", "compound"]);
    let mut node = plain_node(
        &["reward", "compound"],
        "scalar_type",
        "compound_unit",
        compound.clone(),
        aggregate(vec![aggregate(vec![
            binding("unit", reference(node_id("ms"))),
            binding("exponent", literal("integer", json!("2"))),
        ])]),
    );
    node["dependencies"] = json!([node_id("ms")]);
    let body = with(
        operation(),
        vec![
            binding("name", text("area")),
            binding("value_type", reference(compound.clone())),
            binding("expression", reference(id(&["reward", "function"]))),
        ],
    );
    // The compound unit precedes the reward that depends on it.
    let mut value = assemble(vec![
        (node, "generated"),
        (probabilistic("only", "reward", body), "declaration"),
    ]);
    node_by_label(&mut value, &["reward", "function"])["semantic_type"] = compound;
    refresh_identity(&mut value);
    admitted(&value);
}

/// A reward on `withdraw` whose function lists `listed` of the parameters
/// `d`, `t`, `r` and `l` (every reference legal), owned by `withdraw`.
fn withdraw_reward(listed: &[&str]) -> Value {
    let body = with(
        vec![
            binding("declaring", reference(account())),
            binding("operation", text("withdraw")),
        ],
        vec![
            binding("name", text("duration")),
            binding("value_type", reference(integer())),
            binding("expression", reference(id(&["reward", "function"]))),
        ],
    );
    let mut value = package("reward", body, |_| {});
    for name in PARAMETERS {
        let parameter = plain_node(
            &["reward", name],
            "value",
            "parameter",
            integer(),
            aggregate(vec![
                binding("name", text(name)),
                binding("level", literal("integer", json!("0"))),
            ]),
        );
        append(&mut value, parameter, "expression");
    }
    let function = node_by_label(&mut value, &["reward", "function"]);
    function["owner"]["node"] = json!(WITHDRAW);
    function["body"]["members"][0]["value"]["members"] = json!(listed
        .iter()
        .map(|name| reference(id(&["reward", name])))
        .collect::<Vec<_>>());
    function["dependencies"] = json!(PARAMETERS
        .into_iter()
        .filter(|name| listed.contains(name))
        .map(|name| id(&["reward", name]))
        .collect::<Vec<_>>());
    refresh_identity(&mut value);
    value
}

/// FR-369-AC-9: a function that lists the operation's parameters in another
/// count or order refuses although it reads only parameters it lists.
#[trace("TC-476", "FR-369-AC-9")]
#[test]
fn the_public_reader_refuses_a_reward_function_that_lists_the_wrong_parameters() {
    admitted(&withdraw_reward(&PARAMETERS));
    let mut swapped = PARAMETERS;
    swapped.swap(0, 1);
    refuse_invalid(&withdraw_reward(&swapped), &bad_target(4));
    refuse_invalid(&withdraw_reward(&["d"]), &bad_target(4));
    let mut repeated = PARAMETERS;
    repeated[1] = "d";
    refuse_invalid(&withdraw_reward(&repeated), &bad_target(4));
}

/// FR-369-AC-8: two rows that name one operation name through different
/// declaring models refuse, each row being valid alone.
#[trace("TC-476", "FR-369-AC-8")]
#[test]
fn the_public_reader_refuses_one_operation_name_declared_by_two_models() {
    let rows = "body/members/2/value/members";
    admitted(&package(
        "workload",
        workload_of(
            "Leaf",
            &[("Base", "ping"), ("Leaf", "run"), ("Mid", "pong")],
        ),
        |_| {},
    ));
    refuse_invalid(
        &package(
            "workload",
            workload_of(
                "Leaf",
                &[
                    ("Base", "ping"),
                    ("Leaf", "run"),
                    ("Mid", "ping"),
                    ("Mid", "pong"),
                ],
            ),
            |_| {},
        ),
        &format!("{rows}/2"),
    );
}

/// The least work limit under which `value` reads to a verdict.
fn least_work(value: &Value, chain: usize) -> u64 {
    let bytes = canonical(value);
    let evidence = evidence_with_document(value, PACKAGE, &document_with(chain));
    let (mut low, mut high) = (0_u64, 1_u64 << 40);
    while low < high {
        let middle = low + (high - low) / 2;
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = middle;
        match CheckedPackageV2::read(&bytes, limits, &evidence) {
            CheckedPackageV2ReadResult::Incomplete(_) => low = middle + 1,
            _ => high = middle,
        }
    }
    low
}

/// FR-369-AC-17: the join charges one validation visit for each supertype
/// edge it follows, so a workload costs more over a longer supertype chain.
#[trace("TC-476", "FR-369-AC-17")]
#[test]
fn the_join_charges_each_supertype_edge() {
    let whole = [("Base", "ping"), ("Leaf", "run"), ("Mid", "pong")];
    let cost = |chain: usize| {
        let node = probabilistic("only", "workload", workload_of("Leaf", &whole));
        let with = assemble_with(chain, vec![(node, "declaration")]);
        let without = assemble_with(chain, vec![]);
        match CheckedPackageV2::read(
            &canonical(&with),
            CheckedPackageReadLimits::bounded(),
            &evidence_with_document(&with, PACKAGE, &document_with(chain)),
        ) {
            CheckedPackageV2ReadResult::Admitted(_) => {}
            other => panic!("expected admission, read {other:?}"),
        }
        least_work(&with, chain) - least_work(&without, chain)
    };
    let extra = 20;
    let (short, long) = (cost(0), cost(extra));
    assert!(
        long >= short + u64::try_from(extra).expect("count"),
        "the edges are charged: {short} then {long}"
    );
}

/// The pointer of the value of support row `row`.
fn value_at(row: usize) -> String {
    format!("body/members/3/value/members/{row}/members/0/value")
}

/// Where a support value is refused.
#[derive(Clone, Copy)]
enum Refused {
    /// Admitted.
    Not,
    /// `invalid-value` at the literal of row `n`: outside its type, a
    /// non-canonical integer, or a text that is not a member.
    Literal(usize),
    /// `invalid-value` at the `value` member of the literal of row `n`: a
    /// rational or quantity not spelled canonically (FSpec FR-322 "Scalar
    /// literal values": a rational has one spelling, and a bad one refuses at
    /// the literal's `value`).
    Spelling(usize),
}

/// Reads `parameter` of `withdraw` with the support `values`, and expects
/// `outcome`.
fn support_case(parameter: &str, values: Vec<Value>, outcome: Refused) {
    let value = package_of("withdraw", parameter, values);
    match outcome {
        Refused::Not => {
            admitted(&value);
        }
        Refused::Literal(row) => refuse_invalid(&value, &value_at(row)),
        Refused::Spelling(row) => {
            refuse_invalid(&value, &format!("{}/value", value_at(row)));
        }
    }
}

/// FR-369-AC-18: a support value that is no member of its parameter's declared
/// type refuses at the value: outside the bounds, not the one canonical
/// spelling of an integer or rational, or a text that is not in its profile's
/// normal form or has the wrong length (a byte count under `binary-utf8`). The
/// bounds themselves are admitted.
#[trace("TC-476", "FR-369-AC-18")]
#[test]
fn the_public_reader_refuses_a_support_value_outside_its_declared_type() {
    let integers = |values: &[&str]| -> Vec<Value> {
        values
            .iter()
            .map(|value| literal("integer", json!(value)))
            .collect()
    };
    // `Int[1, 12]`. FR-322 places no pointer for a misspelt integer literal,
    // so FR-369's own is the literal.
    support_case("d", integers(&["1", "12"]), Refused::Not);
    for outside in ["0", "13", "-1", "+5", "05"] {
        support_case("d", integers(&[outside]), Refused::Literal(0));
    }
    support_case("d", integers(&["5", "05"]), Refused::Literal(1));
    // `Rational[1, 3; 1, 4]`, and the same bounds over the unit `ms`.
    let rationals =
        |values: &[&str]| -> Vec<Value> { values.iter().map(|value| rational(value)).collect() };
    let quantities = |values: &[&str]| -> Vec<Value> {
        values
            .iter()
            .map(|value| typed_literal(node_id("ms"), "quantity", value))
            .collect()
    };
    for (parameter, make) in [
        ("r", &rationals as &dyn Fn(&[&str]) -> Vec<Value>),
        ("c", &quantities),
    ] {
        support_case(parameter, make(&["1", "3", "1/2", "3/4"]), Refused::Not);
        // Outside the bounds: refused at the literal.
        for outside in ["0", "4", "1/5", "5/4", "-1"] {
            support_case(parameter, make(&[outside]), Refused::Literal(0));
        }
        // Not the one spelling of a rational: refused at the literal's `value`.
        for misspelt in ["2/4", "3/1", "+1", "01", "1/0"] {
            support_case(parameter, make(&[misspelt]), Refused::Spelling(0));
        }
        support_case(parameter, make(&["1/2", "2/4"]), Refused::Spelling(1));
    }
    // `Rational[1, 3; 2, 4]`: the denominator minimum is 2, so `1` (denominator
    // 1) is outside and `1/2` (denominator 2) is the lowest member.
    support_case("r2", rationals(&["1/2", "3/2", "1/4"]), Refused::Not);
    for outside in ["1", "3", "1/5"] {
        support_case("r2", rationals(&[outside]), Refused::Literal(0));
    }
    // `Text[2, 3; profile]`: `é` is U+00E9 and `é` below is `e` U+0301.
    let texts =
        |values: &[&str]| -> Vec<Value> { values.iter().map(|value| text(value)).collect() };
    let composed = "\u{e9}b";
    let decomposed = "e\u{301}b";
    let ligature = "\u{fb01}b";
    // (parameter, admitted, refused)
    let cases: [(&str, &[&str], &[&str]); 6] = [
        (
            "l",
            &["ab", "abc", composed, ligature],
            &["a", "abcd", decomposed],
        ),
        ("ld", &["ab", decomposed], &["a", "abcd", composed]),
        (
            "lk",
            &["ab", composed],
            &["a", "abcd", decomposed, ligature],
        ),
        (
            "lq",
            &["ab", decomposed],
            &["a", "abcd", composed, ligature],
        ),
        // Scalars as written, with no normal form: `e` U+0301 is two scalars.
        (
            "ls",
            &["ab", "e\u{301}", composed, decomposed],
            &["a", "abcd"],
        ),
        // UTF-8 bytes: `é` is two bytes.
        (
            "lb",
            &["ab", "\u{e9}", "a\u{e9}", composed],
            &["a", "\u{e9}\u{e9}", "a\u{e9}\u{e9}", "abcd"],
        ),
    ];
    for (parameter, admitted, refused) in cases {
        support_case(parameter, texts(admitted), Refused::Not);
        for value in refused {
            support_case(parameter, texts(&[value]), Refused::Literal(0));
        }
    }
    // Two spellings of one text under the profile: the second is no member.
    support_case("l", texts(&[composed, decomposed]), Refused::Literal(1));
}

/// A reward whose function has the root node `root` of type `root_type` as
/// its body.
fn reward_with_body(root_type: Value) -> Value {
    reward_package(|value| {
        let root = plain_node(
            &["reward", "root"],
            "value",
            "literal",
            root_type,
            literal("integer", json!("1")),
        );
        append(value, root, "generated");
        let function = node_by_label(value, &["reward", "function"]);
        function["body"]["members"]
            .as_array_mut()
            .expect("members")
            .insert(1, binding("body", reference(id(&["reward", "root"]))));
        function["dependencies"]
            .as_array_mut()
            .expect("dependencies")
            .push(id(&["reward", "root"]));
    })
}

/// FR-369-AC-19: a reward function whose body computes another type than the
/// reward's `value_type` is refused `ill_typed`/`type-mismatch` at the function
/// (FSpec FR-322 "Results"); a body of that type is admitted.
#[trace("TC-476", "FR-369-AC-19")]
#[test]
fn the_public_reader_refuses_a_reward_body_of_another_type() {
    admitted(&reward_with_body(integer()));
    let refusal = refused(&reward_with_body(id(&["builtin/rational"])));
    assert_eq!(refusal.code, CheckedPackageRefusalCode::IllTyped);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::TypeMismatch)
    );
    let function: filament_contract_model::CheckedNodeId =
        serde_json::from_value(id(&["reward", "function"])).expect("function id");
    assert_eq!(refusal.locus.as_ref(), Some(&function));
}

/// FR-369-AC-20: the join charges one validation visit for each support value,
/// ahead of inspecting it, at the value's pointer: some budget stops the read
/// as incomplete at exactly the value of each row. No earlier stage charges at
/// that pointer, so the totals of the other stages are not part of the check.
#[trace("TC-476", "FR-369-AC-20")]
#[test]
fn the_join_charges_each_support_value() {
    let rows: Vec<Value> = (1..=3)
        .map(|value| {
            aggregate(vec![
                binding("value", literal("integer", json!(value.to_string()))),
                binding("weight", rational("1")),
            ])
        })
        .collect();
    let mut body = parameter_on("withdraw", "d", vec![]);
    body["members"][3]["value"] = aggregate(rows);
    let with = package("random_parameter", body, |_| {});
    let bytes = canonical(&with);
    let evidence = evidence_with_document(&with, PACKAGE, &document_with(0));
    let top = least_work(&with, 0);
    let stopped: Vec<String> = (top - 40..top)
        .filter_map(|work| {
            let mut limits = CheckedPackageReadLimits::bounded();
            limits.work = work;
            match CheckedPackageV2::read(&bytes, limits, &evidence) {
                CheckedPackageV2ReadResult::Incomplete(stop) => {
                    stop.path.map(|path| path.as_str().to_owned())
                }
                _ => None,
            }
        })
        .collect();
    let node = node_index(&with);
    for row in 0..3 {
        let pointer = format!("/semantic_graph/nodes/{node}/{}", value_at(row));
        assert!(
            stopped.contains(&pointer),
            "no budget stops at {pointer}: {stopped:?}"
        );
    }
}

/// A `random_parameter` of `parameter` on `Leaf::run`, one support value each
/// at weight 1.
fn on_run(parameter: &str, values: Vec<Value>) -> Value {
    let mut body = parameter_on("run", parameter, values);
    body["members"][0] = binding(
        "declaring",
        reference(id(&["model", "object_type", "ix://test/families/Leaf"])),
    );
    package("random_parameter", body, |_| {})
}

fn case(name: &str) -> Value {
    typed_literal(tier_id(), "enum", name)
}

fn flag(value: Value) -> Value {
    json!({"term": "literal", "type": id(&["builtin/boolean"]),
        "value_kind": "boolean", "value": value})
}

/// FR-369-AC-23: a Boolean and an enumeration parameter admit their support
/// values, and the typed read gives back each value with its probability.
#[trace("TC-476", "FR-369-AC-23")]
#[test]
fn the_public_reader_admits_boolean_and_enum_support_values() {
    let model: filament_contract_model::CheckedNodeId =
        serde_json::from_value(id(&["model", "object_type", "ix://test/families/Leaf"]))
            .expect("model id");
    let read_back = |parameter: &str, values: Vec<Value>| {
        let package = admitted(&on_run(parameter, values));
        package.random_parameters(&model, "run")[0]
            .support
            .iter()
            .map(|row| {
                (
                    serde_json::to_value(row.value).expect("value"),
                    (
                        row.probability.numerator.to_string(),
                        row.probability.denominator.to_string(),
                    ),
                )
            })
            .collect::<Vec<_>>()
    };
    let half = ("1".to_owned(), "2".to_owned());
    let flags = vec![flag(json!(true)), flag(json!(false))];
    let read = read_back("flag", flags.clone());
    assert_eq!(
        read,
        flags
            .iter()
            .map(|value| (value.clone(), half.clone()))
            .collect::<Vec<_>>()
    );
    let tiers = vec![case("Gold"), case("Silver")];
    let read = read_back("tier", tiers.clone());
    assert_eq!(
        read,
        tiers
            .iter()
            .map(|value| (value.clone(), half.clone()))
            .collect::<Vec<_>>()
    );
}

/// FR-369-AC-3: a repeated Boolean support value refuses `invalid-value` at
/// the later value.
#[trace("TC-476", "FR-369-AC-3")]
#[test]
fn the_public_reader_refuses_a_repeated_boolean_support_value() {
    let at = |row: usize| format!("body/members/3/value/members/{row}/members/0/value");
    refuse_invalid(
        &on_run("flag", vec![flag(json!(true)), flag(json!(true))]),
        &at(1),
    );
    refuse_invalid(
        &on_run(
            "flag",
            vec![flag(json!(false)), flag(json!(true)), flag(json!(false))],
        ),
        &at(2),
    );
    admitted(&on_run("flag", vec![flag(json!(true)), flag(json!(false))]));
}

/// FR-369-AC-24: a support value of a variant type with a payload case is
/// refused by the generic literal check, `ill_typed`/`operator-ineligible` at
/// the literal's `type`, for the payload case and the nullary case alike.
#[trace("TC-476", "FR-369-AC-24")]
#[test]
fn the_public_reader_refuses_a_support_value_of_a_payload_variant() {
    for name in ["A", "B"] {
        let value = typed_literal(id(&["model", "variant_type", MIXED]), "enum", name);
        assert_refusal(
            &on_run("mixed", vec![value]),
            CheckedPackageRefusalCode::IllTyped,
            Some(CheckedPackageRefusalCause::OperatorIneligible),
            "body/members/3/value/members/0/members/0/value/type",
        );
    }
}

/// FR-369-AC-24: a Boolean or enumeration support value of another kind, an
/// unknown case, and an enumeration literal typed at
/// another declaration refuse `invalid-value` at the value.
#[trace("TC-476", "FR-369-AC-24")]
#[test]
fn the_public_reader_refuses_malformed_boolean_and_enum_support_values() {
    let at = "body/members/3/value/members/0/members/0/value";
    for value in [
        flag(json!("true")),
        flag(json!(1)),
        literal("integer", json!("1")),
        case("Gold"),
    ] {
        refuse_invalid(&on_run("flag", vec![value]), at);
    }
    let other = typed_literal(id(&["model", "variant_type", OTHER]), "enum", "Gold");
    for value in [
        case("Bronze"),
        case("gold"),
        other,
        flag(json!(true)),
        literal("text", json!("Gold")),
    ] {
        refuse_invalid(&on_run("tier", vec![value]), at);
    }
}

/// The `operation_bodies` rows of the family model: every operation member
/// of every object type, none with a body.
fn no_body_rows() -> Vec<Value> {
    let mut rows: Vec<(String, &str)> = [
        ("Account", "deposit"),
        ("Account", "withdraw"),
        ("Base", "ping"),
        ("Leaf", "run"),
        ("Mid", "ping"),
        ("Mid", "pong"),
    ]
    .into_iter()
    .map(|(declaring, operation)| (format!("ix://{PACKAGE}/{declaring}"), operation))
    .collect();
    rows.sort();
    rows.into_iter()
        .map(|(declaring, operation)| {
            json!({
                "document": PACKAGE,
                "declaring": id(&["model", "object_type", &declaring]),
                "operation": operation,
                "body": {"kind": "no_body"},
            })
        })
        .collect()
}

/// [`reward_package`] with the `no_body` rows an operation without a body
/// carries, `edit` applied after them.
fn rewarded_with_rows(edit: impl FnOnce(&mut Value)) -> Value {
    reward_package(|value| {
        value["operation_bodies"] = Value::Array(no_body_rows());
        edit(value);
    })
}

fn function_position(value: &Value) -> usize {
    let wanted = id(&["reward", "function"]);
    value["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == wanted)
        .expect("reward function")
}

fn refuse_at(
    value: &Value,
    code: CheckedPackageRefusalCode,
    cause: Option<CheckedPackageRefusalCause>,
    path: &str,
) {
    let refusal = refused(value);
    assert_eq!(refusal.code, code, "{refusal:?}");
    assert_eq!(refusal.cause, cause, "{refusal:?}");
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|path| path.as_str()),
        Some(path),
        "{refusal:?}"
    );
}

/// FSpec FR-322 "Owner joins" (W76), FR-640 rule 4, FR-322-AC-471: the clause
/// kind `reward` is owned by an operation member, the reader admits it, and
/// the operation it rewards keeps its `no_body` row: only a `body` clause
/// function gives an operation a body.
#[trace("TC-048", "FR-038-AC-571")]
#[test]
fn the_reader_admits_the_reward_clause_kind_and_demands_no_operation_body() {
    let value = rewarded_with_rows(|_| {});
    assert_eq!(
        node_by_label_ref(&value, &["reward", "function"])["body"]["members"][1]["value"]["value"],
        "reward"
    );
    let package = admitted(&value);
    let model: filament_contract_model::CheckedNodeId =
        serde_json::from_value(account()).expect("model id");
    let rewards = package.rewards(&model, "deposit");
    assert_eq!(rewards.len(), 1);
    assert_eq!(
        serde_json::to_value(rewards[0].function).expect("expression id"),
        id(&["reward", "function"])
    );
    let rows = package.operation_body_rows().expect("rows");
    let deposit = rows
        .iter()
        .find(|row| row.operation.as_ref() == "deposit")
        .expect("deposit row");
    assert_eq!(
        deposit.body,
        filament_contract_model::OperationBody::NoBody,
        "a reward clause function gives its operation no body"
    );
}

fn node_by_label_ref<'a>(value: &'a Value, label: &[&str]) -> &'a Value {
    let wanted = id(label);
    value["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_id"] == wanted)
        .expect("node")
}

/// FSpec FR-322 "Owner joins" (W76), FR-322-AC-471, FR-770-AC-2: a reward
/// clause function owned by an object type, by a field member or by an
/// undeclared member is not owned by a declared operation member, and a
/// declared clause is always `linked`.
#[trace("TC-048", "FR-038-AC-571")]
#[test]
fn a_reward_clause_function_joins_only_its_operation_member_and_is_linked() {
    for owner in [
        "ix://test/families/Account",
        "ix://test/families/Account/balance",
        "ix://test/families/Account/missing",
    ] {
        let value = rewarded_with_rows(|value| {
            node_by_label(value, &["reward", "function"])["owner"]["node"] = json!(owner);
        });
        let position = function_position(&value);
        refuse_at(
            &value,
            CheckedPackageRefusalCode::MissingDeclaration,
            Some(CheckedPackageRefusalCause::MissingSelection),
            &format!("/semantic_graph/nodes/{position}/node_id"),
        );
    }
    let value = rewarded_with_rows(|value| {
        node_by_label(value, &["reward", "function"])["check_mode"] = json!("kernel");
    });
    let position = function_position(&value);
    refuse_at(
        &value,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        &format!("/semantic_graph/nodes/{position}/check_mode"),
    );
}

/// FSpec FR-640 rule 4 and FR-322-AC-471: a reward's `expression` is a
/// `reward` clause function. A `precondition` one is admitted by the owner
/// join and refused at the reward node's `expression`; a `body` one gives the
/// operation a body, so the `operation_bodies` row refuses first when it says
/// `no_body`, and the reward's rule 4 refuses when the member is absent; an
/// `invariant` or unknown clause kind joins no operation member owner.
#[trace("TC-048", "FR-038-AC-572")]
#[test]
fn a_reward_expression_of_another_clause_kind_refuses() {
    let clause = |value: &mut Value, kind: &str| {
        node_by_label(value, &["reward", "function"])["body"]["members"][1]["value"]["value"] =
            json!(kind);
    };
    let precondition = rewarded_with_rows(|value| clause(value, "precondition"));
    refuse_invalid(&precondition, &bad_target(4));
    // The same function is not refused by the owner join: only rule 4 reads
    // the clause kind of an admitted precondition-owned function.
    let absent = reward_package(|value| clause(value, "body"));
    assert!(absent.get("operation_bodies").is_none());
    refuse_invalid(&absent, &bad_target(4));
    let body = rewarded_with_rows(|value| clause(value, "body"));
    let rows = body["operation_bodies"].as_array().expect("rows");
    let deposit = rows
        .iter()
        .position(|row| row["operation"] == "deposit")
        .expect("deposit row");
    refuse_at(
        &body,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        &format!("/operation_bodies/{deposit}/body"),
    );
    for kind in ["invariant", "postcondition", "Reward", ""] {
        let value = rewarded_with_rows(|value| clause(value, kind));
        let position = function_position(&value);
        refuse_at(
            &value,
            CheckedPackageRefusalCode::MissingDeclaration,
            Some(CheckedPackageRefusalCause::MissingSelection),
            &format!("/semantic_graph/nodes/{position}/node_id"),
        );
    }
    // A reward function owned by another operation member refuses rule 4.
    refuse_invalid(
        &rewarded_with_rows(|value| {
            node_by_label(value, &["reward", "function"])["owner"]["node"] = json!(WITHDRAW);
        }),
        &bad_target(4),
    );
}
