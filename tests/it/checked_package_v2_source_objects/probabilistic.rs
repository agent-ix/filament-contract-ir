// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-484: probabilistic nodes whose `declaring` node is a source-declared
//! object type, the strict-read halves FR-369-AC-27 and FR-369-AC-28 of the
//! join ACs the matrix holds Planned (FSpec FR-640-AC-6, FR-640-AC-7). The
//! identity-row join half has no entry point yet (QSL-2645): the packages read
//! here are admitted with their operations, parameters and supports unjoined.

use super::*;

const POKE_REWARD: [&str; 4] = ["Other", "poke", "reward", "cost"];

fn id_value(label: &str) -> Value {
    node_id(label)
}

fn aggregate(members: Vec<Value>) -> Value {
    json!({"term": "aggregate", "members": members})
}

fn int_literal(type_id: &Value, value: &str) -> Value {
    json!({"term": "literal", "type": type_id, "value_kind": "integer", "value": value})
}

fn rational(value: &str) -> Value {
    json!({"term": "literal", "type": id_value("builtin/rational"),
        "value_kind": "rational", "value": value})
}

fn declared(mut node: Value, name: &[&str]) -> Value {
    node["declaration"] = json!({"qualified_name": name});
    node["owner"] = source_owner();
    node
}

pub(super) fn range() -> Value {
    node_id(&crate::support::checked_package::integer_range_key(
        "1", "3",
    ))
}

fn probabilistic(label: &str, form: &str, body: Value) -> Value {
    let id = json!({"package": node_id("x")["package"],
        "path": ["probabilistic", form, label], "ordinal": 0});
    let mut references = Vec::new();
    collect(&body, &mut references);
    // Dependencies sit in node-id order: the path segments, element-wise.
    references.sort_by_key(segments);
    references.dedup();
    json!({
        "schema_version": "filament.checked-semantic-graph/v2",
        "node_id": id, "semantic_type": id,
        "node_tag": "probabilistic", "semantic_form": form,
        "dependencies": references,
        "occurrences": [{"role": "declaration", "ordinal": 0}],
        "body": body,
    })
}

/// The path segments of a node id, the order dependencies sit in.
fn segments(id: &Value) -> Vec<String> {
    id["path"]
        .as_array()
        .expect("path")
        .iter()
        .map(Value::to_string)
        .collect()
}

/// The dependencies of `node` re-derived from the references of its body.
fn redepend(node: &mut Value) {
    let mut references = Vec::new();
    collect(&node["body"], &mut references);
    references.sort_by_key(segments);
    references.dedup();
    node["dependencies"] = json!(references);
}

fn collect(term: &Value, found: &mut Vec<Value>) {
    match term {
        Value::Object(map) if map.get("term") == Some(&json!("reference")) => {
            found.push(map["target"].clone());
        }
        Value::Object(map) => map.values().for_each(|inner| collect(inner, found)),
        Value::Array(items) => items.iter().for_each(|inner| collect(inner, found)),
        _ => {}
    }
}

fn reference_to(label: &str) -> Value {
    reference(&id_value(label))
}

fn operation(declaring: &str, name: &str) -> Vec<Value> {
    vec![
        binding("declaring", reference_to(declaring)),
        binding("operation", text(name)),
    ]
}

fn with(mut head: Vec<Value>, tail: Vec<Value>) -> Value {
    head.extend(tail);
    aggregate(head)
}

fn ms() -> Value {
    id_value("qualified:ms")
}

fn quantity(value: &str) -> Value {
    json!({"term": "literal", "type": ms(), "value_kind": "quantity", "value": value})
}

/// The reward function `cost`: the unit's `SourceOwner`, the declaration of
/// `POKE_REWARD`, clause `reward`, the parameters `listed`, typed at `Int[1, 3]`.
fn reward_function(listed: &[&str]) -> Value {
    let mut function = plain(
        "qualified:Other/poke/reward/cost",
        "function",
        "pure_function",
        &range(),
        &[],
        "declaration",
        aggregate(vec![
            binding(
                "parameters",
                aggregate(listed.iter().map(|name| reference_to(name)).collect()),
            ),
            binding("clause", text("reward")),
        ]),
    );
    function["dependencies"] = json!(listed.iter().map(|name| id_value(name)).collect::<Vec<_>>());
    declared(function, &POKE_REWARD)
}

fn parameter(label: &str, name: &str) -> Value {
    plain(
        label,
        "value",
        "parameter",
        &range(),
        &[],
        "expression",
        aggregate(vec![
            binding("name", text(name)),
            binding("level", int_literal(&integer(), "0")),
        ]),
    )
}

/// The AC-25 package: source types `O` and `Other`, the dense source time
/// `time O dense unit ms`, and one node of each probabilistic form.
pub(super) fn package() -> Value {
    let mut package = source_package();
    let source = fixture_source();
    let dimension = declared(
        plain(
            "qualified:Time",
            "scalar_type",
            "dimension",
            &id_value("qualified:Time"),
            &[],
            "declaration",
            json!({"term": "dimension", "base": true, "terms": []}),
        ),
        &["Time"],
    );
    let unit = declared(
        plain(
            "qualified:ms",
            "scalar_type",
            "unit",
            &id_value("qualified:Time"),
            &[&id_value("qualified:Time")],
            "declaration",
            json!({"term": "unit", "dimension_node_id": id_value("qualified:Time"),
                "target_unit_node_id": null,
                "scale": {"numerator": "1", "denominator": "1"},
                "offset": {"numerator": "0", "denominator": "1"}}),
        ),
        &["ms"],
    );
    let dense = plain(
        "fixture/src/dense",
        "timed_model",
        "time_source",
        &id_value("fixture/src/dense"),
        &[&ms()],
        "expression",
        aggregate(vec![
            binding("kind", text("dense")),
            binding("unit", reference(&ms())),
        ]),
    );
    let timed = declared(
        plain(
            "qualified:Timed",
            "timed_model",
            "timed_model",
            &id_value("qualified:Timed"),
            &[
                &id_value("qualified:O"),
                &id_value("qualified:Time"),
                &id_value("fixture/src/dense"),
            ],
            "declaration",
            aggregate(vec![
                binding("model", reference_to("qualified:O")),
                binding("time_dimension", reference_to("qualified:Time")),
                binding("source", reference_to("fixture/src/dense")),
                binding("clock_places", aggregate(vec![])),
                binding("resets", aggregate(vec![])),
                binding("guards", aggregate(vec![])),
                binding("invariants", aggregate(vec![])),
                binding("urgencies", aggregate(vec![])),
                binding("constraints", aggregate(vec![])),
            ]),
        ),
        &["Timed"],
    );
    let range_node = plain(
        &crate::support::checked_package::integer_range_key("1", "3"),
        "bounded_domain",
        "integer_range",
        &integer(),
        &[&integer()],
        "type",
        crate::support::checked_package::bounds_body("1", "3"),
    );
    let rational_type = plain(
        "builtin/rational",
        "scalar_type",
        "rational",
        &id_value("builtin/rational"),
        &[],
        "generated",
        aggregate(vec![]),
    );
    let support = aggregate(vec![
        aggregate(vec![
            binding("value", int_literal(&integer(), "1")),
            binding("weight", rational("1")),
        ]),
        aggregate(vec![
            binding("value", int_literal(&integer(), "3")),
            binding("weight", rational("3")),
        ]),
    ]);
    let discrete = aggregate(vec![
        aggregate(vec![
            binding("delay", quantity("1")),
            binding("weight", rational("9")),
        ]),
        aggregate(vec![
            binding("delay", quantity("3")),
            binding("weight", rational("1")),
        ]),
    ]);
    let weight_row = aggregate(vec![
        binding("declaring", reference_to("qualified:O")),
        binding("operation", text("step")),
        binding("weight", rational("1")),
    ]);
    let nodes = vec![
        source_type("O", &[], &source),
        source_type("Other", &["n"], &source),
        dimension,
        unit,
        dense,
        timed,
        range_node,
        rational_type,
        parameter("fixture/src/k", "k"),
        reward_function(&["fixture/src/k"]),
        probabilistic(
            "delay_step",
            "delay_distribution",
            with(
                operation("qualified:O", "step"),
                vec![
                    binding("family", text("uniform")),
                    binding("parameters", aggregate(vec![])),
                ],
            ),
        ),
        probabilistic(
            "delay_poke",
            "delay_distribution",
            with(
                operation("qualified:Other", "poke"),
                vec![
                    binding("family", text("discrete")),
                    binding("parameters", discrete),
                ],
            ),
        ),
        probabilistic(
            "random_k",
            "random_parameter",
            with(
                operation("qualified:Other", "poke"),
                vec![binding("parameter", text("k")), binding("support", support)],
            ),
        ),
        probabilistic(
            "workload_w",
            "workload",
            aggregate(vec![
                binding("model", reference_to("qualified:O")),
                binding("name", text("W")),
                binding("weights", aggregate(vec![weight_row])),
            ]),
        ),
        probabilistic(
            "reward_cost",
            "reward",
            with(
                operation("qualified:Other", "poke"),
                vec![
                    binding("name", text("cost")),
                    binding("value_type", reference(&range())),
                    binding(
                        "expression",
                        reference_to("qualified:Other/poke/reward/cost"),
                    ),
                ],
            ),
        ),
    ];
    let present: Vec<Value> = nodes_mut(&mut package)
        .iter()
        .map(|node| node["node_id"].clone())
        .collect();
    nodes_mut(&mut package).extend(
        nodes
            .into_iter()
            .filter(|node| !present.contains(&node["node_id"])),
    );
    settle(&mut package);
    package
}

fn at(package: &Value, form: &str) -> usize {
    position(package, "probabilistic", form)
}

fn edit(package: &mut Value, form: &str, change: impl FnOnce(&mut Value)) {
    let at = at(package, form);
    change(&mut nodes_mut(package)[at]);
    settle(package);
}

/// The AC-25 package with a reward listing `parameters` parameters and a
/// workload of `rows` rows. The registry holds 520 parameters and 257 rows, so
/// the 8-byte buffer the strict read keeps for the listed parameters and the
/// 16-byte `named` buffer of the workload would each exceed the 4096-byte
/// unpaid allowance were they not charged.
fn wide(parameters: usize, rows: usize) -> Value {
    let mut package = package();
    let names: Vec<String> = (0..parameters)
        .map(|index| format!("fixture/src/p{index:04}"))
        .collect();
    let mut added: Vec<Value> = names
        .iter()
        .enumerate()
        .map(|(index, label)| parameter(label, &format!("p{index:04}")))
        .collect();
    nodes_mut(&mut package).append(&mut added);
    let function = nodes(&package)
        .iter()
        .position(|node| node["node_id"] == id_value("qualified:Other/poke/reward/cost"))
        .expect("function");
    let listed: Vec<Value> = names.iter().map(|label| reference_to(label)).collect();
    nodes_mut(&mut package)[function]["body"]["members"][0]["value"] = aggregate(listed);
    redepend(&mut nodes_mut(&mut package)[function]);
    edit(&mut package, "workload", |node| {
        node["body"]["members"][2]["value"] = aggregate(
            (0..rows)
                .map(|row| {
                    aggregate(vec![
                        binding("declaring", reference_to("qualified:O")),
                        binding("operation", text(&format!("op{row:04}"))),
                        binding("weight", rational("1")),
                    ])
                })
                .collect(),
        );
        redepend(node);
    });
    package
}

/// The registry fixture: the admitted AC-25 package and its evidence.
pub(crate) fn accounting_fixture() -> (Value, filament_contract_model::CheckedPackageEvidence) {
    accounting_fixture_of(257)
}

/// The registry fixture with a workload of `rows` rows. It keeps the
/// package's `discrete` delay and random parameter, so the rational spellings
/// of both are read too.
pub(crate) fn accounting_fixture_of(
    rows: usize,
) -> (Value, filament_contract_model::CheckedPackageEvidence) {
    let package = wide(520, rows);
    let package = indexed(&package);
    let evidence = evidence_of(&package);
    (package, evidence)
}

/// A reward listing many parameters and a workload of many rows read through
/// the strict read are admitted: the wide package is a valid unit.
#[trace("TC-484", "FR-369-AC-27")]
#[test]
fn a_wide_source_reward_and_workload_are_admitted() {
    admitted(&wide(40, 20));
}

/// Rewrites every occurrence of the node id `from` to `to`.
fn rename(value: &mut Value, from: &Value, to: &Value) {
    if value == from {
        *value = to.clone();
        return;
    }
    match value {
        Value::Object(map) => map.values_mut().for_each(|inner| rename(inner, from, to)),
        Value::Array(items) => items.iter_mut().for_each(|inner| rename(inner, from, to)),
        _ => {}
    }
}

/// The reward function moved to the declaration `name`, its `node_id` path and
/// the reward's `expression` target rewritten to match, so FR-038-AC-21's
/// `declaration-nominal-mismatch` does not refuse first.
fn redeclare(package: &mut Value, name: &[&str]) {
    let from = id_value("qualified:Other/poke/reward/cost");
    let to = id_value(&format!("qualified:{}", name.join("/")));
    let function = nodes(package)
        .iter()
        .position(|node| node["node_id"] == from)
        .expect("function");
    nodes_mut(package)[function]["declaration"] = json!({"qualified_name": name});
    rename(package, &from, &to);
    let reward = at(package, "reward");
    let dependencies = nodes_mut(package)[reward]["dependencies"]
        .as_array_mut()
        .expect("dependencies");
    dependencies.sort_by_key(segments);
    settle(package);
}

fn target_at(package: &Value, form: &str, member: usize) -> String {
    format!(
        "/semantic_graph/nodes/{}/body/members/{member}/value/target",
        at(package, form)
    )
}

fn invalid() -> (Code, Option<Cause>) {
    (Code::InvalidPackage, Some(Cause::InvalidValue))
}

fn fail(case: &str, package: &Value, path: &str) {
    expect(case, package, invalid().0, invalid().1, path);
}

/// FR-369-AC-27 (strict-read half): the unit's delays, random parameter,
/// workload and reward on source-declared types are admitted, and the typed
/// read gives each keyed by its node, the reward function being the declared
/// node `{package, ["Other", "poke", "reward", "cost"], 0}` built here
/// independently of the reader.
#[trace("TC-484", "FR-369-AC-27")]
#[test]
fn source_declared_probabilistic_nodes_are_admitted_and_read() {
    let package = package();
    let CheckedPackageV2ReadResult::Admitted(read) = read(&package) else {
        panic!("expected admission");
    };
    let owner = |name: &str| id(&format!("qualified:{name}"));
    let (o, other) = (owner("O"), owner("Other"));
    let delays = read.delay_distributions(&o);
    assert_eq!(delays.len(), 1);
    assert_eq!(delays[0].operation, "step");
    assert_eq!(
        delays[0].family,
        filament_contract_model::CheckedDelayFamily::Uniform
    );
    let poke = read.delay_distributions(&other);
    assert_eq!(poke.len(), 1);
    assert_eq!(poke[0].operation, "poke");
    let filament_contract_model::CheckedDelayFamily::Discrete(rows) = &poke[0].family else {
        panic!("a discrete delay");
    };
    let unit = id("qualified:ms");
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|(quantity, _)| quantity.unit == &unit));
    let parameters = read.random_parameters(&other, "poke");
    assert_eq!(parameters.len(), 1);
    assert_eq!(parameters[0].parameter, "k");
    let probabilities: Vec<(&str, &str)> = parameters[0]
        .support
        .iter()
        .map(|row| (&*row.probability.numerator, &*row.probability.denominator))
        .collect();
    assert_eq!(probabilities, [("1", "4"), ("3", "4")]);
    let rewards = read.rewards(&other, "poke");
    assert_eq!(rewards.len(), 1);
    assert_eq!(rewards[0].name, "cost");
    let independent: CheckedNodeId = serde_json::from_value(json!({
        "package": node_id("x")["package"],
        "path": POKE_REWARD, "ordinal": 0}))
    .expect("declared node id");
    assert_eq!(rewards[0].function, &independent);
    let range: CheckedNodeId = serde_json::from_value(range()).expect("range id");
    assert_eq!(rewards[0].value_type, &range);
    let workloads = read.workloads(&o);
    assert_eq!(workloads.len(), 1);
    assert_eq!(workloads[0].name, "W");
    assert_eq!(workloads[0].weights.len(), 1);
    assert_eq!(workloads[0].weights[0].operation, "step");
    assert_eq!(workloads[0].weights[0].declaring, &o);
    assert!(read.rewards(&o, "step").is_empty());
}

/// FR-369-AC-28 (strict-read half): each refuses `invalid_package`/
/// `invalid-value` at the member.
#[trace("TC-484", "FR-369-AC-28")]
#[test]
fn source_declared_declaring_nodes_refuse_at_the_member() {
    // A `declaring` that is no source object type: a source dimension and a
    // source population, in each form.
    for (case, label) in [
        ("a source dimension", "qualified:Time"),
        ("a source population", "qualified:orders"),
    ] {
        for (form, member) in [
            ("delay_distribution", 0),
            ("random_parameter", 0),
            ("reward", 0),
            ("workload", 0),
        ] {
            let mut package = package();
            nodes_mut(&mut package).push(source_population("orders"));
            edit(&mut package, form, |node| {
                node["body"]["members"][member]["value"]["target"] = id_value(label);
                redepend(node);
            });
            let path = target_at(&package, form, member);
            fail(&format!("{form} on {case}"), &package, &path);
        }
    }
    // A delay on a source model of a unit with no `dense` time member.
    let mut package = package();
    nodes_mut(&mut package).retain(|node| {
        ![id_value("qualified:Timed"), id_value("fixture/src/dense")].contains(&node["node_id"])
    });
    settle(&mut package);
    // The first delay in node-id order is `delay_poke`.
    let first = nodes(&package)
        .iter()
        .position(|node| node["node_id"]["path"][2] == "delay_poke")
        .expect("delay_poke");
    fail(
        "no dense time member",
        &package,
        &format!("/semantic_graph/nodes/{first}/body/members/0/value/target"),
    );
}

/// FR-369-AC-28 (strict-read half): the reward `cost`'s function is refused at
/// the reward's `expression` when it is not the reward's own source clause
/// function.
#[trace("TC-484", "FR-369-AC-28")]
#[test]
fn a_source_reward_function_that_is_not_the_rewards_own_refuses_at_the_expression() {
    for (case, name) in [
        ("another reward name", ["Other", "poke", "reward", "load"]),
        ("another operation", ["Other", "step", "reward", "cost"]),
        ("another type", ["O", "poke", "reward", "cost"]),
    ] {
        let mut package = package();
        redeclare(&mut package, &name);
        let path = target_at(&package, "reward", 4);
        fail(case, &package, &path);
    }
    let mut package = package();
    let function = id_value("qualified:Other/poke/reward/cost");
    let at_function = |package: &Value| {
        nodes(package)
            .iter()
            .position(|node| node["node_id"] == function)
            .expect("function")
    };
    let function_at = at_function(&package);
    nodes_mut(&mut package)[function_at]["body"]["members"][1]["value"]["value"] =
        json!("precondition");
    settle(&mut package);
    let path = target_at(&package, "reward", 4);
    fail("clause precondition", &package, &path);
    // The owner of a second lock-selected source, the declaration's region
    // naming it: the owner join admits it, the probabilistic read refuses.
    let mut package = self::package();
    let other = json!({"kind": "source", "authority": fixture_source()["authority"],
        "identity": "filament.fixture.other-source/v1"});
    package["lock"]["sources"]
        .as_array_mut()
        .expect("sources")
        .push(json!({"authority": other["authority"], "identity": other["identity"]}));
    let function_at = at_function(&package);
    nodes_mut(&mut package)[function_at]["owner"] = other.clone();
    settle(&mut package);
    for entry in package["source_map"].as_array_mut().expect("source map") {
        if entry["node_id"] == function {
            entry["regions"][0]["source"] =
                json!({"authority": other["authority"], "identity": other["identity"]});
        }
    }
    let path = target_at(&package, "reward", 4);
    fail("another source owner", &package, &path);
    // A function typed other than the reward's `value_type`.
    let mut package = self::package();
    let function_at = at_function(&package);
    nodes_mut(&mut package)[function_at]["semantic_type"] = integer();
    settle(&mut package);
    let path = target_at(&package, "reward", 4);
    fail("another own type", &package, &path);
    // A listed node that is no parameter.
    let mut package = self::package();
    let function_at = at_function(&package);
    nodes_mut(&mut package)[function_at]["body"]["members"][0]["value"]["members"][0] =
        reference_to("qualified:Other");
    nodes_mut(&mut package)[function_at]["dependencies"] = json!([id_value("qualified:Other")]);
    settle(&mut package);
    let path = target_at(&package, "reward", 4);
    fail("a listed node that is no parameter", &package, &path);
    // A function that reads a node beyond its parameters, and one that reads
    // a parameter it does not list (the walk of FSpec FR-405 "Rewards").
    for (case, listed, reads) in [
        ("a read beyond the parameters", true, "qualified:Order"),
        ("a parameter it does not list", false, "fixture/src/k"),
    ] {
        let mut package = self::package();
        let function_at = at_function(&package);
        let node = &mut nodes_mut(&mut package)[function_at];
        if !listed {
            node["body"]["members"][0]["value"] = aggregate(vec![]);
        }
        let mut dependencies = node["dependencies"]
            .as_array()
            .expect("dependencies")
            .clone();
        dependencies.push(id_value(reads));
        dependencies.sort_by_key(segments);
        dependencies.dedup();
        node["dependencies"] = json!(dependencies);
        settle(&mut package);
        let path = target_at(&package, "reward", 4);
        fail(case, &package, &path);
    }
}

/// FR-640 Reader rule 3 over a source workload: weights out of order or
/// repeating an operation name refuse at the row; a row whose `declaring` node
/// is not a source object type refuses at its `declaring`.
#[trace("TC-484", "FR-369-AC-28")]
#[test]
fn a_source_workload_refuses_unordered_repeated_and_foreign_rows() {
    let row = |declaring: &str, operation: &str| {
        aggregate(vec![
            binding("declaring", reference_to(declaring)),
            binding("operation", text(operation)),
            binding("weight", rational("1")),
        ])
    };
    let workload = |rows: Vec<Value>| {
        let mut package = package();
        edit(&mut package, "workload", |node| {
            node["body"]["members"][2]["value"] = aggregate(rows);
            redepend(node);
        });
        package
    };
    let weights = |package: &Value, row: usize, tail: &str| {
        format!(
            "/semantic_graph/nodes/{}/body/members/2/value/members/{row}{tail}",
            at(package, "workload")
        )
    };
    let package = workload(vec![
        row("qualified:Other", "poke"),
        row("qualified:O", "step"),
    ]);
    fail("out of order", &package, &weights(&package, 1, ""));
    let package = workload(vec![
        row("qualified:O", "step"),
        row("qualified:Other", "step"),
    ]);
    fail(
        "a repeated operation name",
        &package,
        &weights(&package, 1, ""),
    );
    let mut package = workload(vec![
        row("qualified:O", "step"),
        row("qualified:Other", "zz"),
    ]);
    // The nodes on `Other` go first: they would refuse before the workload.
    nodes_mut(&mut package).retain(|node| {
        !["delay_poke", "random_k", "reward_cost"]
            .contains(&node["node_id"]["path"][2].as_str().unwrap_or(""))
    });
    other_owned(&mut package);
    fail(
        "a declaring node of another source owner",
        &package,
        &weights(&package, 1, "/members/0/value/target"),
    );
    let package = workload(vec![row("qualified:Time", "step")]);
    fail(
        "a foreign declaring node",
        &package,
        &weights(&package, 0, "/members/0/value/target"),
    );
}

/// FR-369-AC-11 over source-declared nodes: a second delay, reward or random
/// parameter of one operation (and parameter, reward name) refuses
/// `conflicting-definition` at the later node.
#[trace("TC-476", "FR-369-AC-11")]
#[test]
fn a_second_source_declared_definition_is_a_conflicting_definition() {
    for form in [
        "delay_distribution",
        "reward",
        "random_parameter",
        "workload",
    ] {
        let mut package = package();
        let node = nodes(&package)[at(&package, form)].clone();
        let mut second = node;
        let id = json!({"package": node_id("x")["package"],
            "path": ["probabilistic", form, "z_second"], "ordinal": 0});
        second["node_id"] = id.clone();
        second["semantic_type"] = id;
        nodes_mut(&mut package).push(second);
        settle(&mut package);
        let later = nodes(&package).len() - 1;
        expect(
            form,
            &package,
            Code::InvalidPackage,
            Some(Cause::ConflictingDefinition),
            &format!("/semantic_graph/nodes/{later}"),
        );
    }
}

/// A second timed root over `Order` whose source is a `tick` one, placed
/// before the `dense` root over `O`: the reader looks through it for a `dense`
/// root of the same `SourceOwner` (FSpec FR-640 Reader rule 5).
fn with_tick_root_first(package: &mut Value) {
    let tick = plain(
        "fixture/src/tick",
        "timed_model",
        "time_source",
        &id_value("fixture/src/tick"),
        &[&order_id(), &ms(), &id_value("fixture/src/anchor")],
        "expression",
        aggregate(vec![
            binding("kind", text("tick")),
            binding("unit", reference(&ms())),
            binding("model", reference(&order_id())),
            binding("operation", reference_to("fixture/src/anchor")),
            binding("period", rational("1")),
        ]),
    );
    let timed = declared(
        plain(
            "qualified:TickTimed",
            "timed_model",
            "timed_model",
            &id_value("qualified:TickTimed"),
            &[
                &order_id(),
                &id_value("qualified:Time"),
                &id_value("fixture/src/tick"),
            ],
            "declaration",
            aggregate(vec![
                binding("model", reference(&order_id())),
                binding("time_dimension", reference_to("qualified:Time")),
                binding("source", reference_to("fixture/src/tick")),
                binding("clock_places", aggregate(vec![])),
                binding("resets", aggregate(vec![])),
                binding("guards", aggregate(vec![])),
                binding("invariants", aggregate(vec![])),
                binding("urgencies", aggregate(vec![])),
                binding("constraints", aggregate(vec![])),
            ]),
        ),
        &["TickTimed"],
    );
    let (mut tick, mut timed) = (tick, timed);
    redepend(&mut tick);
    redepend(&mut timed);
    let at = nodes(package)
        .iter()
        .position(|node| node["node_id"] == id_value("fixture/src/dense"))
        .expect("dense source");
    nodes_mut(package).splice(at..at, [tick, timed]);
    settle(package);
}

/// FR-369-AC-27, FSpec FR-640 Reader rule 5: the delay of a source node finds
/// the `dense` root of its unit though a `tick` root over another source type
/// of the unit comes first.
#[trace("TC-484", "FR-369-AC-27")]
#[test]
fn a_source_delay_finds_the_dense_root_past_a_tick_root_of_its_unit() {
    let mut package = package();
    with_tick_root_first(&mut package);
    let CheckedPackageV2ReadResult::Admitted(read) = read(&package) else {
        panic!("expected admission");
    };
    assert_eq!(read.delay_distributions(&id("qualified:Other")).len(), 1);
}

/// `Other` moved to a second lock-selected source, its region naming it.
fn other_owned(package: &mut Value) {
    let other = json!({"kind": "source", "authority": fixture_source()["authority"],
        "identity": "filament.fixture.other-source/v1"});
    package["lock"]["sources"]
        .as_array_mut()
        .expect("sources")
        .push(json!({"authority": other["authority"], "identity": other["identity"]}));
    let owned = nodes(package)
        .iter()
        .position(|node| node["node_id"] == id_value("qualified:Other"))
        .expect("Other");
    nodes_mut(package)[owned]["owner"] = other.clone();
    settle(package);
    for entry in package["source_map"].as_array_mut().expect("source map") {
        if entry["node_id"] == id_value("qualified:Other") {
            entry["regions"][0]["source"] =
                json!({"authority": other["authority"], "identity": other["identity"]});
        }
    }
}

/// FR-369-AC-28: a delay on a source type of another `SourceOwner` than the
/// owner of every `dense` root refuses at `declaring`.
#[trace("TC-484", "FR-369-AC-28")]
#[test]
fn a_source_delay_needs_a_dense_root_of_its_own_source_owner() {
    let mut package = package();
    other_owned(&mut package);
    let first = nodes(&package)
        .iter()
        .position(|node| node["node_id"]["path"][2] == "delay_poke")
        .expect("delay_poke");
    fail(
        "a delay of another source owner",
        &package,
        &format!("/semantic_graph/nodes/{first}/body/members/0/value/target"),
    );
}

/// The least work limit that admits `package`, by bisection.
fn least_work(package: &Value) -> u64 {
    let package = indexed(package);
    let bytes = canonical(&package);
    let evidence = evidence_of(&package);
    let admits = |work: u64| {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = work;
        match CheckedPackageV2::read(&bytes, limits, &evidence) {
            CheckedPackageV2ReadResult::Admitted(_) => true,
            CheckedPackageV2ReadResult::Incomplete(_) => false,
            other => panic!("work {work}: {other:?}"),
        }
    };
    let (mut low, mut high) = (0, 1_u64 << 24);
    assert!(admits(high));
    while low < high {
        let middle = (low + high) / 2;
        if admits(middle) {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    low
}

/// The budgets below `package`'s least admitting one that stop the read at
/// the pointer `at`: one for each unit charged there.
fn units_charged_at(package: &Value, at: &str) -> usize {
    let least = least_work(package);
    let package = indexed(package);
    let bytes = canonical(&package);
    let evidence = evidence_of(&package);
    (0..least)
        .filter(|&work| {
            let mut limits = CheckedPackageReadLimits::bounded();
            limits.work = work;
            matches!(
                CheckedPackageV2::read(&bytes, limits, &evidence),
                CheckedPackageV2ReadResult::Incomplete(stop)
                    if stop.path.as_ref().map(|path| path.as_str()) == Some(at)
            )
        })
        .count()
}

/// FR-369-AC-5: the strict read charges one visit for each row of a source
/// workload beside the wire read's one, so two units are charged at each row.
#[trace("TC-476", "FR-369-AC-5")]
#[test]
fn a_source_workload_charges_one_visit_for_each_row() {
    let mut package = package();
    edit(&mut package, "workload", |node| {
        node["body"]["members"][2]["value"] = aggregate(
            (0..3)
                .map(|row| {
                    aggregate(vec![
                        binding("declaring", reference_to("qualified:O")),
                        binding("operation", text(&format!("op{row}"))),
                        binding("weight", rational("1")),
                    ])
                })
                .collect(),
        );
        redepend(node);
    });
    let row = format!(
        "/semantic_graph/nodes/{}/body/members/2/value/members/1",
        at(&package, "workload")
    );
    assert_eq!(units_charged_at(&package, &row), 2);
}

/// FR-369-AC-17: each graph node scanned for a delay's `dense` root is
/// charged at the delay, on a source node as on a selected one: the two nodes
/// added before the root add exactly two units at the delay's pointer.
#[trace("TC-476", "FR-369-AC-17")]
#[test]
fn a_source_delay_charges_each_node_scanned_for_the_root() {
    let units = |package: &Value| {
        let at = format!(
            "/semantic_graph/nodes/{}",
            at(package, "delay_distribution")
        );
        units_charged_at(package, &at)
    };
    let plain_units = units(&package());
    let mut padded = package();
    with_tick_root_first(&mut padded);
    assert_eq!(units(&padded), plain_units + 2);
}

/// The reward function declared `["Other", "poke", <kind>, "cost"]` of another
/// kind than `reward` belongs to no reward (FSpec FR-322 item 5).
#[trace("TC-484", "FR-369-AC-28")]
#[test]
fn a_source_function_of_another_declaration_kind_is_no_reward() {
    let mut package = package();
    redeclare(&mut package, &["Other", "poke", "body", "cost"]);
    let path = target_at(&package, "reward", 4);
    fail("a body function", &package, &path);
}

/// A listed entry the walk admits but that is no parameter (a type node)
/// refuses at the expression.
#[trace("TC-484", "FR-369-AC-28")]
#[test]
fn a_source_reward_listing_a_type_node_as_a_parameter_refuses() {
    let mut package = package();
    let function = nodes(&package)
        .iter()
        .position(|node| node["node_id"] == id_value("qualified:Other/poke/reward/cost"))
        .expect("function");
    nodes_mut(&mut package)[function]["body"]["members"][0]["value"]["members"][0] =
        reference(&range());
    redepend(&mut nodes_mut(&mut package)[function]);
    settle(&mut package);
    let path = target_at(&package, "reward", 4);
    fail("a type node listed", &package, &path);
}

/// Two delays of one source type on different operations are no conflict.
#[trace("TC-476", "FR-369-AC-11")]
#[test]
fn source_delays_of_different_operations_are_no_conflict() {
    let mut package = package();
    let mut second = nodes(&package)[at(&package, "delay_distribution")].clone();
    let id = json!({"package": node_id("x")["package"],
        "path": ["probabilistic", "delay_distribution", "z_second"], "ordinal": 0});
    second["node_id"] = id.clone();
    second["semantic_type"] = id;
    second["body"]["members"][1]["value"] = text("other_operation");
    nodes_mut(&mut package).push(second);
    settle(&mut package);
    admitted(&package);
}

/// A source-declared operation reference whose `operation` is not a text
/// literal is refused at the `operation` member, whatever its term. The wire
/// stage decodes the operation as a text literal first, so the refusal is
/// `malformed_wire` there and the join's own `invalid-value` branch for a
/// missing text is a second line of defence.
#[trace("TC-484", "FR-369-AC-28")]
#[test]
fn a_source_operation_reference_without_a_text_operation_refuses_at_the_operation() {
    for (case, form) in [
        ("delay", "delay_distribution"),
        ("random parameter", "random_parameter"),
        ("reward", "reward"),
    ] {
        for replacement in [
            int_literal(&integer(), "1"),
            reference_to("qualified:O"),
            aggregate(vec![]),
        ] {
            let mut package = package();
            edit(&mut package, form, |node| {
                node["body"]["members"][1]["value"] = replacement.clone();
                redepend(node);
            });
            let refusal = refused(&package);
            assert_eq!(refusal.code, Code::MalformedWire, "{case}: {refusal:?}");
            assert_eq!(
                refusal.loci[0].path.as_ref().map(|at| at.as_str()),
                Some(
                    format!(
                        "/semantic_graph/nodes/{}/body/members/1/value",
                        at(&package, form)
                    )
                    .as_str()
                ),
                "{case}"
            );
        }
    }
}

/// Every request the read makes at the graph stage, as `(form, amount)`, found
/// by raising the read-storage limit to admit each in turn.
fn graph_requests(package: &Value) -> Vec<(&'static str, u128)> {
    let package = indexed(package);
    graph_requests_of(&canonical(&package), &evidence_of(&package))
}

/// [`graph_requests`] over the bytes and evidence of a package.
pub(crate) fn graph_requests_of(
    bytes: &[u8],
    evidence: &filament_contract_model::CheckedPackageEvidence,
) -> Vec<(&'static str, u128)> {
    use filament_contract_model::{
        CheckedPackageV2BudgetedReadResult as Result, NativeJoinBudget, NativeJoinLocusSource,
        PackageReadStage,
    };
    let read = |limit: u64| {
        let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
        let result = CheckedPackageV2::read_with_budget(
            bytes,
            CheckedPackageReadLimits::bounded(),
            evidence,
            &mut budget,
        );
        (result, budget)
    };
    let total = read(u64::MAX).1.read_storage_consumed();
    let (mut limit, mut found) = (0, Vec::new());
    loop {
        let (result, budget) = read(limit);
        if matches!(result, Result::Admitted(_)) {
            break;
        }
        let locus = budget.first_unpaid_locus().expect("a denial has a locus");
        let amount = budget.first_unpaid_read_storage().expect("storage denial");
        if matches!(
            locus.source,
            NativeJoinLocusSource::PackageRead(PackageReadStage::Graph, _)
        ) {
            found.push((locus.form, amount));
        }
        limit += u64::try_from(amount).expect("amount");
        assert!(limit <= total);
    }
    found
}

/// FR-369-AC-27: the two buffers the strict read keeps for a source reward's
/// listed parameters (`usize`, one each) and a source workload's named
/// operations (`&str`, one each row) are charged to the read-storage budget
/// before they are made. The package whose reward lists 22 parameters and whose
/// workload has 13 rows makes exactly one request of each buffer's size
/// (8 x 22 and 16 x 13 bytes) at a probabilistic node.
#[trace("TC-484", "FR-369-AC-27")]
#[test]
fn the_source_buffers_are_charged_before_they_are_made() {
    let word = u128::try_from(std::mem::size_of::<usize>()).expect("bytes");
    let name = u128::try_from(std::mem::size_of::<&str>()).expect("bytes");
    let (listed, named) = (word * 22, name * 13);
    let requests = graph_requests(&wide(22, 13));
    let amounts: Vec<u128> = requests
        .iter()
        .filter(|(form, _)| *form == "probabilistic")
        .map(|(_, amount)| *amount)
        .collect();
    let count = |amount: u128| amounts.iter().filter(|&&found| found == amount).count();
    assert_eq!(count(listed), 1, "{amounts:?}");
    assert_eq!(count(named), 1, "{amounts:?}");
}
