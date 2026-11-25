// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-474: the V2 reader admits FSpec FR-490's `correspondence`/
//! `implementation_guarantee` body and FR-491's `correspondence`/
//! `component_composition` body, and refuses every other shape with the code,
//! cause and locus FR-038 "Implementation guarantee and component composition
//! bodies" names, in reader order.
//!
//! Every package is built in-repo from the temporal fixture of
//! `checked_package_v2_temporal` (a clause selecting the infinite-trace
//! profile, the `Order` and `Sub` object types and their domain package) plus
//! an empty abstraction relation node, a `predicate` node and the node under
//! test. No FSL value is used: the node shapes are FSpec's.

use crate::checked_package_v2_temporal::{
    clause, model_key, node_id, node_with_key, orders_document, push, read, select_profile,
    with_model, INFINITE, LEFT_NODE, ORDERS, ORDER_NODE, SUB_NODE,
};
use crate::support::checked_package::{
    canonical, evidence_with_document, fixture_source, index_fixture_node_ids, reassign_node_id,
    settle, source_owner, v2_all_families, wire_form,
};
use filament_contract_model::{
    encode_checked_package_v2, CheckedDependencyOperand, CheckedNodeId, CheckedNodeKind,
    CheckedNodePathSegment, CheckedOperandEntry, CheckedPackageReadLimits,
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
    CheckedPackageSourceCorrespondenceRef, CheckedPackageV2, CheckedPackageV2ReadResult,
    CheckedTerm, CorrespondenceForm,
};
use ix_trace_rs::trace;
use serde_json::{json, Value};

const GUARANTEE: &str = "guarantee/G";
/// The declared name of the composition node (`compose Sys`): its structural
/// path is this name, not a derived correspondence path.
const COMPOSITION: &str = "Sys";
/// The body arrays whose entries are the composition's members, in the
/// canonical concatenation order of FSpec FR-491.
const MEMBER_ARRAYS: [&str; 6] = [
    "components",
    "guarantees",
    "relies",
    "liveness_dependencies",
    "supplies",
    "environment",
];
const PREDICATE: &str = "guarantee/predicate";
/// The package the fixture's nodes are in, and the component package a
/// composition imports.
const DEPENDENCY: &str = "test/families";
const IMPORTER: &str = "test/importer";

/// A node of the importing package.
fn importer_id(label: &str) -> Value {
    let mut id = node_id(label);
    id["package"] = json!(IMPORTER);
    id
}

/// Moves every structural node id of `value` from the dependency's package
/// to the importer's.
fn rekey(value: &mut Value) {
    match value {
        Value::Object(object) => {
            if object.get("package") == Some(&json!(DEPENDENCY))
                && object.contains_key("path")
                && object.contains_key("ordinal")
            {
                object.insert("package".to_owned(), json!(IMPORTER));
            }
            object.values_mut().for_each(rekey);
        }
        Value::Array(items) => items.iter_mut().for_each(rekey),
        _ => {}
    }
}

/// The fixture as a package of its own that imports the same fixture, under
/// another name, as the one component package: the importer's nodes are in
/// `IMPORTER`, a composition's subjects and clauses in `DEPENDENCY`.
fn importer() -> Value {
    let mut package = base();
    rekey(&mut package);
    package["name"] = json!(IMPORTER);
    package["lock"]["dependency_selections"] = json!([{"identity": DEPENDENCY}]);
    settle(&mut package);
    package
}

/// Reads `package`, supplying the component package when it holds a
/// composition.
fn read_any(package: &Value) -> CheckedPackageV2ReadResult {
    let composes = nodes(package)
        .iter()
        .any(|node| node["semantic_form"] == "component_composition");
    if composes {
        read_composition_with_dependency(package, &base())
    } else {
        read(package)
    }
}
fn read_composition_with_dependency(
    package: &Value,
    component: &Value,
) -> CheckedPackageV2ReadResult {
    let dependency = match read(component) {
        CheckedPackageV2ReadResult::Admitted(dependency) => *dependency,
        other => panic!("the component package admits, read {other:?}"),
    };
    let mut evidence = evidence_with_document(package, ORDERS, &orders_document());
    evidence
        .insert_dependency_package(DEPENDENCY, dependency)
        .expect("dependency evidence agrees on admitted content");
    crate::read_and_account(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    )
}

fn order() -> Value {
    node_id(&model_key(ORDER_NODE))
}

fn sub() -> Value {
    node_id(&model_key(SUB_NODE))
}

fn relation() -> Value {
    node_id("guarantee/relation")
}

fn predicate() -> Value {
    node_id("guarantee/predicate")
}

fn nodes(package: &Value) -> &Vec<Value> {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
}

/// The clause node's id in `package`.
fn clause_id(package: &Value) -> Value {
    nodes(package)[clause(package)]["node_id"].clone()
}

fn typed(id: &Value) -> CheckedNodeId {
    serde_json::from_value(id.clone()).expect("structural node id")
}

/// The fixture every case starts from: the clause under the infinite-trace
/// profile, the model nodes, an empty abstraction relation and a predicate.
fn base() -> Value {
    let mut package = v2_all_families();
    select_profile(&mut package, INFINITE);
    with_model(&mut package);
    push(
        &mut package,
        node_with_key(
            "guarantee/relation",
            "correspondence",
            "abstraction_relation",
            json!({"term": "abstraction_relation", "objects": [], "populations": [],
                "frames": []}),
        ),
    );
    // A predicate is a function node; an application body leaves it unowned.
    let mut predicate = nodes(&package)
        .iter()
        .find(|node| node["node_tag"] == "function" && node["body"]["term"] == "application")
        .expect("the fixture carries a function application")
        .clone();
    predicate["node_id"] = node_id("guarantee/predicate");
    predicate["semantic_form"] = json!("predicate");
    push(&mut package, predicate);
    // Give the clause an actual model subject rather than the general fixture's
    // Boolean parameter. Other families retain their original parameter.
    let clause_at = clause(&package);
    let old_over = nodes(&package)[clause_at]["body"]["arguments"][0]["target"].clone();
    let mut over = nodes(&package)
        .iter()
        .find(|node| node["node_id"] == old_over)
        .expect("over parameter")
        .clone();
    over["node_id"] = node_id("guarantee/model-over");
    let reference = json!({"package": DEPENDENCY, "path": ["composite_type", "reference", order()], "ordinal": 0});
    push(
        &mut package,
        json!({
            "node_id": reference, "schema_version": "filament.checked-semantic-graph/v2",
            "node_tag": "composite_type", "semantic_form": "reference",
            "semantic_type": reference, "dependencies": [order()],
            "occurrences": [{"role": "type", "ordinal": 0}],
            "body": {"term": "aggregate", "members": [{"term": "reference", "target": order()}]},
        }),
    );
    over["semantic_type"] = reference;
    let over_id = over["node_id"].clone();
    push(&mut package, over);
    package["semantic_graph"]["nodes"][clause_at]["body"]["arguments"][0]["target"] =
        over_id.clone();
    let deps = &mut package["semantic_graph"]["nodes"][clause_at]["dependencies"];
    for id in deps.as_array_mut().expect("dependencies") {
        if *id == old_over {
            *id = over_id.clone();
        }
    }
    // A second clause has an unbounded eventually and is liveness; the
    // original bounded eventually is safety under the infinite-trace profile.
    let formula_id = nodes(&package)[clause_at]["body"]["arguments"][5]["target"].clone();
    let mut formula = nodes(&package)
        .iter()
        .find(|node| node["node_id"] == formula_id)
        .expect("formula")
        .clone();
    formula["node_id"] = node_id("guarantee/live-formula");
    formula["body"]["operation"]["member"]["interval"] = Value::Null;
    push(&mut package, formula);
    let mut live = nodes(&package)[clause_at].clone();
    live["node_id"] = node_id("guarantee/live-clause");
    live["body"]["arguments"][1]["value"] = json!("live_clause");
    live["body"]["arguments"][5]["target"] = node_id("guarantee/live-formula");
    live["dependencies"] = json!([over_id, node_id("guarantee/live-formula")]);
    push(&mut package, live);
    let relation_at = nodes(&package)
        .iter()
        .position(|node| node["node_id"] == relation())
        .expect("relation");
    let node = &mut package["semantic_graph"]["nodes"][relation_at];
    node["semantic_type"] = node["node_id"].clone();
    node["occurrences"] = json!([{"role": "declaration", "ordinal": 0}]);
    index_fixture_node_ids(&mut package);
    settle(&mut package);
    package
}

fn fairness(kind: &str, granularity: &str, context: &Value, operation: &str) -> Value {
    json!({"kind": kind, "granularity": granularity, "context": context,
        "operation": operation})
}

fn environment_fairness(name: &str, context: &Value, operation: &str) -> Value {
    json!({"name": name, "supplier": "scheduler",
        "supplies": {"fairness": fairness("weak", "whole", context, operation)}})
}

fn environment_assumption(name: &str, predicate: &Value) -> Value {
    json!({"name": name, "supplier": "operator", "supplies": {"assumption": predicate}})
}

fn guarantee_body(package: &Value) -> Value {
    json!({
        "term": "implementation_guarantee",
        "clause": clause_id(package),
        "relation": relation(),
        "initial": ["implementation", "initial"],
        "progress": [
            {"context": order(), "operation": "reset",
             "measure": {"decreases": ["implementation", "measure"]}},
            {"context": order(), "operation": "scaled", "measure": {"bound": 3}},
        ],
        "environment": [
            environment_assumption("env_ok", &predicate()),
            environment_fairness("fair_reset", &order(), "reset"),
        ],
    })
}

fn clause_entry(component: &str, clause: &Value) -> Value {
    json!({"component": component, "clause": clause})
}

fn premise(guarantee: (&str, &Value), on: (&str, &Value)) -> Value {
    json!({"guarantee": clause_entry(guarantee.0, guarantee.1),
        "on": clause_entry(on.0, on.1)})
}

fn composition_body(package: &Value) -> Value {
    let clause = clause_id(package);
    let live = node_id("guarantee/live-clause");
    json!({
        "term": "component_composition",
        "rule": "assume-guarantee",
        "components": [
            {"name": "Consumer", "package": DEPENDENCY, "subject": sub()},
            {"name": "Producer", "package": DEPENDENCY, "subject": order()},
        ],
        "guarantees": [
            clause_entry("Consumer", &clause),
            clause_entry("Producer", &live),
        ],
        "relies": [premise(("Consumer", &clause), ("Producer", &clause))],
        "liveness_dependencies": [premise(("Producer", &live), ("Consumer", &live))],
        "supplies": [
            fairness("strong", "each", &order(), "reset"),
            fairness("weak", "whole", &order(), "reset"),
        ],
        "environment": [environment_assumption("env_ok", &importer_id(PREDICATE))],
    })
}

/// Every target of `body`, unique and in structural order.
fn targets(body: &Value) -> Vec<Value> {
    fn collect(value: &Value, ids: &mut Vec<CheckedNodeId>) {
        match value {
            Value::Object(object) => {
                for (name, member) in object {
                    let node = matches!(
                        name.as_str(),
                        "clause" | "relation" | "context" | "assumption" | "subject"
                    );
                    if node && member.get("package").is_some() {
                        ids.push(typed(member));
                    } else {
                        collect(member, ids);
                    }
                }
            }
            Value::Array(items) => items.iter().for_each(|item| collect(item, ids)),
            _ => {}
        }
    }
    let mut ids = Vec::new();
    collect(body, &mut ids);
    ids.sort();
    ids.dedup();
    ids.iter()
        .map(|id| serde_json::to_value(id).expect("node id"))
        .collect()
}

fn node_of(label: &str, form: &str, body: Value) -> Value {
    let mut node = node_with_key(label, "correspondence", form, body.clone());
    node["semantic_type"] = node["node_id"].clone();
    node["occurrences"] = json!([{"role": "declaration", "ordinal": 0}]);
    node["dependencies"] = json!(targets(&body));
    node
}

fn guarantee_node(body: Value) -> Value {
    node_of(GUARANTEE, "implementation_guarantee", body)
}

fn composition_node(body: Value) -> Value {
    let members: usize = MEMBER_ARRAYS
        .iter()
        .map(|array| body[array].as_array().map_or(0, Vec::len))
        .sum();
    let mut node = node_of(COMPOSITION, "component_composition", body);
    node["declaration"] = json!({"qualified_name": [COMPOSITION]});
    node["owner"] = source_owner(&fixture_source());
    let mut occurrences = vec![json!({"role": "declaration", "ordinal": 0})];
    occurrences
        .extend((0..members).map(|ordinal| json!({"role": "generated", "ordinal": ordinal})));
    node["occurrences"] = Value::Array(occurrences);
    node
}

/// `package` holding `node`, settled.
fn with(mut package: Value, node: Value) -> (Value, usize) {
    let position = push(&mut package, node);
    settle(&mut package);
    (package, position)
}

fn guaranteed(edit: impl FnOnce(&mut Value)) -> (Value, usize) {
    let package = base();
    let mut body = guarantee_body(&package);
    edit(&mut body);
    with(package, guarantee_node(body))
}

/// The importer holding the composition node `edit` changes after it is
/// built from `body`.
fn composed_with(body: Value, edit: impl FnOnce(&mut Value)) -> (Value, usize) {
    let mut node = composition_node(body);
    node["node_id"]["package"] = json!(IMPORTER);
    node["semantic_type"] = node["node_id"].clone();
    edit(&mut node);
    with(importer(), node)
}

fn composed(edit: impl FnOnce(&mut Value)) -> (Value, usize) {
    let mut body = composition_body(&base());
    edit(&mut body);
    composed_with(body, |_| {})
}

fn admitted(case: &str, package: &Value) -> CheckedPackageV2 {
    match read_any(package) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("{case}: expected admission, read {other:?}"),
    }
}

/// Asserts the refusal's code, cause and pointer.
fn expect(case: &str, package: &Value, code: Code, cause: Option<Cause>, path: &str) {
    let refusal = match read_any(package) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("{case}: expected a refusal, read {other:?}"),
    };
    assert_eq!(refusal.code, code, "{case}: {refusal:?}");
    assert_eq!(refusal.cause, cause, "{case}: {refusal:?}");
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|path| path.as_str()),
        Some(path),
        "{case}: {refusal:?}"
    );
}

fn at(position: usize, tail: &str) -> String {
    format!("/semantic_graph/nodes/{position}{tail}")
}

const GRAPH: Code = Code::InvalidSemanticGraph;
const MALFORMED: (Code, Option<Cause>) =
    (Code::InvalidModelBinding, Some(Cause::MalformedDeclaration));
const CONFLICT: (Code, Option<Cause>) =
    (Code::InvalidModelBinding, Some(Cause::ConflictingBinding));
const MISSING: (Code, Option<Cause>) = (Code::MissingDeclaration, Some(Cause::MissingName));

/// One refused case: how the body is edited, the code and cause, and the
/// pointer tail within the node.
type Case<'a> = (&'a str, fn(&mut Value), (Code, Option<Cause>), &'a str);

fn run_guarantee(cases: &[Case<'_>]) {
    for (name, edit, (code, cause), tail) in cases {
        let (package, position) = guaranteed(edit);
        expect(name, &package, *code, *cause, &at(position, tail));
    }
}

fn run_composition(cases: &[Case<'_>]) {
    for (name, edit, (code, cause), tail) in cases {
        let (package, position) = composed(edit);
        expect(name, &package, *code, *cause, &at(position, tail));
    }
}

#[trace("TC-474", "FR-038-AC-531")]
#[test]
fn tc_474_an_implementation_guarantee_node_admits_with_its_typed_term() {
    let (package, position) = guaranteed(|_| {});
    let admitted = admitted("guarantee", &package);
    assert_eq!(
        admitted.node_kinds()[position],
        CheckedNodeKind::Correspondence(CorrespondenceForm::ImplementationGuarantee)
    );
    let node = &admitted.graph().nodes[position];
    let Some(CheckedTerm::ImplementationGuarantee {
        initial,
        progress,
        environment,
        ..
    }) = admitted.term(&node.node_id)
    else {
        panic!("an admitted guarantee has a typed guarantee body");
    };
    assert_eq!(initial.len(), 2);
    assert_eq!(progress.len(), 2);
    assert_eq!(environment.len(), 2);
    assert_eq!(
        serde_json::to_value(&node.body).expect("typed body"),
        guarantee_body(&package)
    );
    // A safety guarantee needs no progress member and no environment member.
    let (bare, _) = guaranteed(|body| {
        body["progress"] = json!([]);
        body["environment"] = json!([]);
    });
    admitted_ok("a guarantee with empty lists", &bare);
    // A fairness context no progress member names is a dependency too.
    let (left, _) = guaranteed(|body| {
        body["environment"][1] = environment_fairness("fair_reset", &left_object(), "act");
    });
    admitted_ok("a fairness context of its own", &left);
}

fn left_object() -> Value {
    node_id(&model_key(LEFT_NODE))
}

fn admitted_ok(case: &str, package: &Value) {
    drop(admitted(case, package));
}

#[trace("TC-474", "FR-038-AC-532")]
#[test]
fn tc_474_a_component_composition_node_admits_with_its_typed_term() {
    let (package, position) = composed(|_| {});
    let admitted = admitted("composition", &package);
    assert_eq!(
        admitted.node_kinds()[position],
        CheckedNodeKind::Correspondence(CorrespondenceForm::ComponentComposition)
    );
    let node = &admitted.graph().nodes[position];
    let Some(CheckedTerm::ComponentComposition {
        rule,
        components,
        guarantees,
        relies,
        liveness_dependencies,
        supplies,
        environment,
    }) = admitted.term(&node.node_id)
    else {
        panic!("an admitted composition has a typed composition body");
    };
    assert_eq!(&**rule, "assume-guarantee");
    assert_eq!(components.len(), 2);
    assert_eq!(guarantees.len(), 2);
    // The premise lists stay separate.
    assert_eq!((relies.len(), liveness_dependencies.len()), (1, 1));
    assert_eq!((supplies.len(), environment.len()), (2, 1));
    assert_eq!(
        serde_json::to_value(&node.body).expect("typed body"),
        composition_body(&base())
    );
    for rule in ["protocol-refinement", "refinement-transfer", "interface"] {
        let (other, _) = composed(|body| body["rule"] = json!(rule));
        admitted_ok(rule, &other);
    }
}

#[trace("TC-474", "FR-038-AC-533")]
#[test]
fn tc_474_a_body_of_the_wrong_shape_refuses_at_the_offending_entry() {
    run_guarantee(&[
        (
            "a missing member",
            |body| {
                body.as_object_mut().expect("body").remove("initial");
            },
            (GRAPH, None),
            "/body",
        ),
        (
            "an extra member",
            |body| body["extra"] = json!(true),
            (GRAPH, None),
            "/body",
        ),
        (
            "a term of the other form",
            |body| body["term"] = json!("component_composition"),
            (GRAPH, None),
            "/body",
        ),
        (
            "an empty initial path",
            |body| body["initial"] = json!([]),
            (GRAPH, None),
            "/body",
        ),
        (
            "a measure of two members",
            |body| body["progress"][0]["measure"] = json!({"bound": 1, "decreases": ["m"]}),
            (GRAPH, None),
            "/body/progress/0/measure",
        ),
        (
            "an unknown fairness kind",
            |body| {
                body["environment"][1]["supplies"]["fairness"]["kind"] = json!("sometimes");
            },
            (GRAPH, None),
            "/body/environment/1/supplies/fairness",
        ),
        (
            "a supplies of neither form",
            |body| body["environment"][0]["supplies"] = json!({"other": predicate()}),
            (GRAPH, None),
            "/body/environment/0/supplies",
        ),
    ]);
    run_composition(&[
        (
            "an unknown rule",
            |body| body["rule"] = json!("magic"),
            (GRAPH, None),
            "/body",
        ),
        (
            "a component without its package",
            |body| {
                body["components"][0]
                    .as_object_mut()
                    .expect("component")
                    .remove("package");
            },
            (GRAPH, None),
            "/body/components/0",
        ),
        (
            "a premise with an extra side",
            |body| body["relies"][0]["extra"] = json!(1),
            (GRAPH, None),
            "/body/relies/0",
        ),
        (
            "a term of the other form",
            |body| body["term"] = json!("implementation_guarantee"),
            (GRAPH, None),
            "/body",
        ),
    ]);
    // An application at the body root is the closed term grammar's.
    let mut package = base();
    let applied = nodes(&package)
        .iter()
        .find(|node| node["semantic_form"] == "predicate")
        .expect("the predicate")
        .clone();
    let mut node = guarantee_node(guarantee_body(&package));
    node["body"] = applied["body"].clone();
    node["dependencies"] = applied["dependencies"].clone();
    let position = push(&mut package, node);
    settle(&mut package);
    expect(
        "an application root",
        &package,
        Code::IllTyped,
        Some(Cause::OperatorIneligible),
        &at(position, ""),
    );
}

#[trace("TC-474", "FR-038-AC-534")]
#[test]
fn tc_474_semantic_type_and_dependencies_must_be_the_nodes_own_and_its_targets() {
    for form in ["guarantee", "composition"] {
        let build = |edit: &dyn Fn(&mut Value)| {
            if form == "guarantee" {
                let package = base();
                let mut node = guarantee_node(guarantee_body(&package));
                edit(&mut node);
                with(package, node)
            } else {
                composed_with(composition_body(&base()), edit)
            }
        };
        let own = if form == "guarantee" {
            node_id(PREDICATE)
        } else {
            importer_id(PREDICATE)
        };
        let (package, position) = build(&|node| node["semantic_type"] = own.clone());
        expect(form, &package, GRAPH, None, &at(position, ""));
        let (package, position) = build(&|node| {
            node["dependencies"]
                .as_array_mut()
                .expect("dependencies")
                .pop();
        });
        expect(form, &package, GRAPH, None, &at(position, ""));
        let (package, position) = build(&|node| {
            let extra = typed(&relation());
            let mut ids: Vec<CheckedNodeId> =
                serde_json::from_value(node["dependencies"].clone()).expect("ids");
            if !ids.contains(&extra) {
                ids.push(extra);
            }
            ids.sort();
            ids.reverse();
            node["dependencies"] = serde_json::to_value(ids).expect("ids");
        });
        expect(form, &package, GRAPH, None, &at(position, ""));
    }
}

/// FSpec FR-490: `dependencies` are `clause`, `relation` and every `context`
/// and `assumption` target, so a scheduling environment member's `context`
/// is one too.
#[trace("TC-474", "FR-038-AC-534")]
#[test]
fn tc_474_a_fairness_context_belongs_to_the_dependencies() {
    let context = left_object();
    let (package, position) = guaranteed(|body| {
        body["environment"][1] = environment_fairness("fair_reset", &context, "act");
    });
    let mut package = package;
    let node = &mut package["semantic_graph"]["nodes"][position];
    let kept: Vec<Value> = node["dependencies"]
        .as_array()
        .expect("dependencies")
        .iter()
        .filter(|id| **id != context)
        .cloned()
        .collect();
    assert_eq!(
        kept.len() + 1,
        node["dependencies"].as_array().expect("deps").len()
    );
    node["dependencies"] = Value::Array(kept);
    settle(&mut package);
    expect(
        "an omitted fairness context",
        &package,
        GRAPH,
        None,
        &at(position, ""),
    );
}

#[trace("TC-474", "FR-038-AC-535")]
#[test]
fn tc_474_an_array_out_of_canonical_order_refuses_at_the_array() {
    run_guarantee(&[
        (
            "progress",
            |body| body["progress"].as_array_mut().expect("progress").reverse(),
            (GRAPH, None),
            "/body/progress",
        ),
        (
            "environment",
            |body| {
                body["environment"]
                    .as_array_mut()
                    .expect("environment")
                    .reverse()
            },
            (GRAPH, None),
            "/body/environment",
        ),
    ]);
    run_composition(&[
        (
            "components",
            |body| {
                body["components"]
                    .as_array_mut()
                    .expect("components")
                    .reverse()
            },
            (GRAPH, None),
            "/body/components",
        ),
        (
            "guarantees",
            |body| {
                body["guarantees"]
                    .as_array_mut()
                    .expect("guarantees")
                    .reverse()
            },
            (GRAPH, None),
            "/body/guarantees",
        ),
        (
            "relies",
            |body| {
                let clause = body["guarantees"][0]["clause"].clone();
                body["relies"] = json!([
                    premise(("Producer", &clause), ("Consumer", &clause)),
                    premise(("Consumer", &clause), ("Producer", &clause)),
                ]);
            },
            (GRAPH, None),
            "/body/relies",
        ),
        (
            "liveness_dependencies",
            |body| {
                let clause = body["guarantees"][0]["clause"].clone();
                body["liveness_dependencies"] = json!([
                    premise(("Producer", &clause), ("Consumer", &clause)),
                    premise(("Consumer", &clause), ("Producer", &clause)),
                ]);
            },
            (GRAPH, None),
            "/body/liveness_dependencies",
        ),
        // The spellings order: `strong` before `weak`, `each` before `whole`.
        (
            "supplies by kind spelling",
            |body| body["supplies"].as_array_mut().expect("supplies").reverse(),
            (GRAPH, None),
            "/body/supplies",
        ),
        (
            "supplies by granularity spelling",
            |body| {
                body["supplies"] = json!([
                    fairness("weak", "whole", &order(), "reset"),
                    fairness("weak", "each", &order(), "reset"),
                ]);
            },
            (GRAPH, None),
            "/body/supplies",
        ),
        (
            "environment",
            |body| {
                let predicate = importer_id(PREDICATE);
                body["environment"] = json!([
                    environment_assumption("b", &predicate),
                    environment_assumption("a", &predicate),
                ]);
            },
            (GRAPH, None),
            "/body/environment",
        ),
    ]);
}

#[trace("TC-474", "FR-038-AC-536")]
#[test]
fn tc_474_a_guarantee_target_must_be_a_declared_node_of_its_kind() {
    run_guarantee(&[
        (
            "a clause that is no temporal clause",
            |body| body["clause"] = relation(),
            MALFORMED,
            "/body/clause",
        ),
        (
            "a relation that is no abstraction relation",
            |body| body["relation"] = predicate(),
            MALFORMED,
            "/body/relation",
        ),
        (
            "a progress context that is no object type",
            |body| {
                body["progress"][0]["context"] = predicate();
                body["progress"][1]["context"] = predicate();
            },
            MALFORMED,
            "/body/progress/0",
        ),
        (
            "a fairness context that is no object type",
            |body| {
                body["environment"][1]["supplies"]["fairness"]["context"] = predicate();
            },
            MALFORMED,
            "/body/environment/1",
        ),
        (
            "an assumption that is no predicate",
            |body| body["environment"][0]["supplies"] = json!({"assumption": relation()}),
            MALFORMED,
            "/body/environment/0",
        ),
    ]);
    // A clause that selects another temporal profile.
    let mut package = base();
    select_profile(
        &mut package,
        "filament.temporal.event-position.false-extension/v1",
    );
    let live_formula = nodes(&package)
        .iter()
        .position(|node| node["node_id"] == node_id("guarantee/live-formula"))
        .expect("live formula");
    package["semantic_graph"]["nodes"][live_formula]["body"]["operation"]["member"]["interval"] =
        json!({"lower": "0", "upper": "3"});
    let body = guarantee_body(&package);
    let (package, position) = with(package, guarantee_node(body));
    expect(
        "a bounded-profile clause",
        &package,
        MALFORMED.0,
        MALFORMED.1,
        &at(position, "/body/clause"),
    );
}

#[trace("TC-474", "FR-038-AC-537")]
#[test]
fn tc_474_a_guarantee_member_must_resolve_spell_and_be_unique() {
    run_guarantee(&[
        (
            "a progress operation the subject does not declare",
            |body| body["progress"][0]["operation"] = json!("missing"),
            MISSING,
            "/body/progress/0",
        ),
        (
            "a scheduling operation the subject does not declare",
            |body| {
                body["environment"][1]["supplies"]["fairness"]["operation"] = json!("missing");
            },
            MISSING,
            "/body/environment/1",
        ),
        (
            "an operation the context only inherits",
            |body| {
                body["progress"][0]["context"] = sub();
                body["progress"][1]["context"] = sub();
            },
            MALFORMED,
            "/body/progress/0",
        ),
        (
            "an initial path that is not a Rust path",
            |body| body["initial"] = json!(["9lives"]),
            MALFORMED,
            "/body/initial",
        ),
        (
            "a measure path that is not a Rust path",
            |body| body["progress"][0]["measure"] = json!({"decreases": ["crate"]}),
            MALFORMED,
            "/body/progress/0",
        ),
        (
            "a zero bound",
            |body| body["progress"][1]["measure"] = json!({"bound": 0}),
            MALFORMED,
            "/body/progress/1",
        ),
        (
            "two progress members for one operation",
            |body| body["progress"][1]["operation"] = json!("reset"),
            CONFLICT,
            "/body/progress/1",
        ),
        (
            "two environment members of one name",
            |body| body["environment"][1]["name"] = json!("env_ok"),
            CONFLICT,
            "/body/environment/1",
        ),
    ]);
}

#[trace("TC-474", "FR-038-AC-538")]
#[test]
fn tc_474_a_composition_member_must_name_its_components_and_guarantees() {
    run_composition(&[
        (
            "two components of one name",
            |body| body["components"][1]["name"] = json!("Consumer"),
            CONFLICT,
            "/body/components/1",
        ),
        (
            "two components over one subject",
            |body| body["components"][1]["subject"] = sub(),
            CONFLICT,
            "/body/components/1",
        ),
        (
            "a guarantee of an undeclared component",
            |body| body["guarantees"][1]["component"] = json!("Stranger"),
            MISSING,
            "/body/guarantees/1",
        ),
        (
            "a relies premise on an undeclared component",
            |body| body["relies"][0]["on"]["component"] = json!("Stranger"),
            MISSING,
            "/body/relies/0",
        ),
        (
            "a relies premise whose guarantee is not declared",
            |body| {
                body["guarantees"]
                    .as_array_mut()
                    .expect("guarantees")
                    .remove(0);
            },
            MALFORMED,
            "/body/relies/0",
        ),
        (
            "a liveness dependency of a component on itself",
            |body| body["liveness_dependencies"][0]["on"]["component"] = json!("Producer"),
            MALFORMED,
            "/body/liveness_dependencies/0",
        ),
        (
            "a component of a package the unit does not import",
            |body| body["components"][0]["package"] = json!("acme/stranger"),
            MISSING,
            "/body/components/0/package",
        ),
        (
            "a supplies operation no component subject declares",
            |body| body["supplies"][0]["operation"] = json!("missing"),
            MISSING,
            "/body/supplies/0",
        ),
        (
            "a scheduling operation no component subject declares",
            |body| {
                body["environment"] = json!([
                    environment_assumption("env_ok", &importer_id(PREDICATE)),
                    environment_fairness("fair", &order(), "missing"),
                ]);
            },
            MISSING,
            "/body/environment/1",
        ),
        (
            "two environment members of one name",
            |body| {
                let predicate = importer_id(PREDICATE);
                body["environment"] = json!([
                    environment_assumption("env_ok", &predicate),
                    environment_assumption("env_ok", &predicate),
                ]);
            },
            CONFLICT,
            "/body/environment/1",
        ),
    ]);
}

#[trace("TC-474", "FR-038-AC-538")]
#[test]
fn tc_474_a_composition_target_resolves_in_its_component_package() {
    run_composition(&[
        (
            "a subject that is no object type",
            |body| body["components"][0]["subject"] = relation(),
            MALFORMED,
            "/body/components/0",
        ),
        (
            "a guarantee that is no temporal clause",
            |body| body["guarantees"][0]["clause"] = order(),
            MALFORMED,
            "/body/guarantees/0",
        ),
        (
            "a supplies context that is no object type",
            |body| body["supplies"] = json!([fairness("strong", "each", &relation(), "reset")]),
            MALFORMED,
            "/body/supplies/0",
        ),
        (
            "a supplies context in no component package",
            |body| {
                body["supplies"] =
                    json!([fairness("strong", "each", &importer_id(PREDICATE), "reset")]);
            },
            MISSING,
            "/body/supplies/0",
        ),
        (
            "an assumption that is no predicate",
            |body| {
                body["environment"][0]["supplies"] =
                    json!({"assumption": importer_id("guarantee/relation")});
            },
            MALFORMED,
            "/body/environment/0",
        ),
    ]);
}

#[trace("TC-474", "FR-038-AC-538")]
#[test]
fn tc_474_composition_premises_keep_safety_and_liveness_separate() {
    run_composition(&[
        (
            "rely on liveness",
            |body| {
                body["relies"][0]["on"]["clause"] = node_id("guarantee/live-clause");
            },
            MALFORMED,
            "/body/relies/0",
        ),
        (
            "depends on safety",
            |body| {
                body["liveness_dependencies"][0]["on"]["clause"] = clause_id(&base());
            },
            MALFORMED,
            "/body/liveness_dependencies/0",
        ),
        (
            "depends from safety",
            |body| {
                body["liveness_dependencies"][0]["guarantee"] = body["guarantees"][0].clone();
                body["liveness_dependencies"][0]["on"]["component"] = json!("Producer");
            },
            MALFORMED,
            "/body/liveness_dependencies/0",
        ),
    ]);
}

#[trace("TC-474", "FR-038-AC-538")]
#[test]
fn tc_474_a_component_clause_must_be_over_its_model_subject() {
    let (package, position) = composed(|_| {});
    let mut component = base();
    let parameter = nodes(&component)
        .iter()
        .position(|node| node["node_id"] == node_id("guarantee/model-over"))
        .expect("parameter");
    component["semantic_graph"]["nodes"][parameter]["semantic_type"] = nodes(&component)
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "boolean")
        .expect("Boolean type")["node_id"]
        .clone();
    settle(&mut component);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_composition_with_dependency(&package, &component)
    else {
        panic!("a clause over a different subject must refuse");
    };
    assert_eq!((refusal.code, refusal.cause), MISSING);
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|p| p.as_str()),
        Some(at(position, "/body/guarantees/0").as_str())
    );
}

#[trace("TC-474", "FR-038-AC-533")]
#[test]
fn tc_474_a_malformed_target_key_refuses_as_wire() {
    let (mut package, position) = guaranteed(|_| {});
    package["semantic_graph"]["nodes"][position]["body"]["relation"]["ordinal"] = json!("wrong");
    // Keep the malformed key out of dependencies so only the body key is invalid.
    expect(
        "a nonnumeric target ordinal",
        &package,
        Code::MalformedWire,
        None,
        &at(position, "/body/relation"),
    );
}

#[trace("TC-474", "FR-038-AC-538")]
#[test]
fn tc_474_premise_classification_uses_negation_normal_form() {
    for (operator, negated, liveness) in [
        ("eventually", false, true),
        ("eventually", true, false),
        ("always", false, false),
        ("always", true, true),
        ("until", false, true),
        ("until", true, false),
        ("release", false, false),
        ("release", true, true),
    ] {
        let (package, position) = composed(|_| {});
        let mut component = base();
        let root = nodes(&component)
            .iter()
            .position(|node| node["node_id"] == node_id("guarantee/live-formula"))
            .expect("live formula");
        let formula = &mut component["semantic_graph"]["nodes"][root];
        formula["body"]["operation"]["identity"] =
            json!(format!("filament.op.temporal.{operator}"));
        if matches!(operator, "until" | "release") {
            let argument = formula["body"]["arguments"][0].clone();
            formula["body"]["arguments"]
                .as_array_mut()
                .expect("arguments")
                .push(argument);
        }
        if negated {
            let mut inner = formula.clone();
            inner["node_id"] = node_id("guarantee/negated-formula");
            formula["body"]["operation"]["identity"] = json!("filament.op.temporal.not");
            formula["body"]["operation"]["member"] = Value::Null;
            formula["body"]["arguments"] =
                json!([{"term": "reference", "target": inner["node_id"]}]);
            push(&mut component, inner);
        }
        settle(&mut component);
        let outcome = read_composition_with_dependency(&package, &component);
        if liveness {
            assert!(
                matches!(outcome, CheckedPackageV2ReadResult::Admitted(_)),
                "{operator} negated={negated}: {outcome:?}"
            );
        } else {
            let CheckedPackageV2ReadResult::Refused(refusal) = outcome else {
                panic!("{operator} negated={negated} must refuse a depends premise")
            };
            assert_eq!(
                (refusal.code, refusal.cause),
                MALFORMED,
                "{operator} negated={negated}"
            );
            assert_eq!(
                refusal.loci[0].path.as_ref().map(|p| p.as_str()),
                Some(at(position, "/body/liveness_dependencies/0").as_str())
            );
        }
    }
}

#[trace("TC-474", "FR-038-AC-537")]
#[test]
fn tc_474_guarantee_operations_must_belong_to_the_clause_model_subject() {
    let mut package = base();
    let parameter = nodes(&package)
        .iter()
        .position(|node| node["node_id"] == node_id("guarantee/model-over"))
        .expect("parameter");
    package["semantic_graph"]["nodes"][parameter]["semantic_type"] = nodes(&package)
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "boolean")
        .expect("Boolean type")["node_id"]
        .clone();
    let body = guarantee_body(&package);
    let (package, position) = with(package, guarantee_node(body));
    expect(
        "operations of a different subject",
        &package,
        MISSING.0,
        MISSING.1,
        &at(position, "/body/progress/0"),
    );
}

#[trace("TC-474", "FR-038-AC-531")]
#[test]
fn guarantee_without_progress_or_environment_retains_its_complete_typed_body() {
    let (package, position) = guaranteed(|body| {
        body["progress"] = json!([]);
        body["environment"] = json!([]);
    });
    let expected = nodes(&package)[position]["body"].clone();
    let admitted = admitted("empty guarantee lists", &package);
    let node = &admitted.graph().nodes[position];
    let term = admitted.term(&node.node_id).expect("retained guarantee");
    let CheckedTerm::ImplementationGuarantee {
        progress,
        environment,
        ..
    } = term
    else {
        panic!("the guarantee must retain its typed body");
    };
    assert!(progress.is_empty());
    assert!(environment.is_empty());
    assert_eq!(
        serde_json::to_value(term).expect("typed guarantee"),
        expected
    );
}

#[trace("TC-474", "FR-038-AC-532")]
#[test]
fn every_composition_rule_retains_its_rule_and_distinct_premise_lists() {
    for expected_rule in [
        "assume-guarantee",
        "protocol-refinement",
        "refinement-transfer",
        "interface",
    ] {
        let (package, position) = composed(|body| body["rule"] = json!(expected_rule));
        let expected = nodes(&package)[position]["body"].clone();
        let admitted = admitted(expected_rule, &package);
        let node = &admitted.graph().nodes[position];
        let term = admitted.term(&node.node_id).expect("retained composition");
        let CheckedTerm::ComponentComposition {
            rule,
            relies,
            liveness_dependencies,
            ..
        } = term
        else {
            panic!("the composition must retain its typed body");
        };
        assert_eq!(rule.as_ref(), expected_rule);
        assert_eq!(
            serde_json::to_value(relies).expect("typed relies"),
            expected["relies"]
        );
        assert_eq!(
            serde_json::to_value(liveness_dependencies).expect("typed liveness dependencies"),
            expected["liveness_dependencies"]
        );
        assert_eq!(
            serde_json::to_value(term).expect("typed composition"),
            expected
        );
    }
}

const ANCHOR: &str = "guarantee/anchor";
const LIVE_CLAUSE: &str = "guarantee/live-clause";

/// The component package with its `live` clause named by an operand: the
/// clause's id is derived from an anchor node only this package holds, placed
/// last, so the anchor's position is past the end of the importer's graph.
fn derived_dependency() -> (Value, u64) {
    let mut dependency = base();
    let mut anchor = nodes(&dependency)
        .iter()
        .find(|node| node["node_id"] == predicate())
        .expect("predicate")
        .clone();
    // Two further nodes before it keep its position past the importer's end.
    for label in ["guarantee/filler-0", "guarantee/filler-1"] {
        let mut filler = anchor.clone();
        filler["node_id"] = node_id(label);
        push(&mut dependency, filler);
    }
    anchor["node_id"] = node_id(ANCHOR);
    let position = push(&mut dependency, anchor);
    let derived = json!({"package": DEPENDENCY,
        "path": ["temporal", "temporal_clause", node_id(ANCHOR)], "ordinal": 0});
    reassign_node_id(&mut dependency, &node_id(LIVE_CLAUSE), &derived);
    index_fixture_node_ids(&mut dependency);
    settle(&mut dependency);
    (dependency, u64::try_from(position).expect("position"))
}

/// A clause of the derived dependency as the importer names it: the
/// derived path holds the anchor's position in the dependency's graph.
fn foreign_clause(package: &str, anchor: u64) -> Value {
    json!({"package": DEPENDENCY,
        "path": ["temporal", "temporal_clause", {"package": package, "node": anchor}],
        "ordinal": 0})
}

/// The importer holding a composition whose live clause is the foreign
/// clause `clause`, and the composition's position.
fn foreign_composition(clause: &Value) -> (Value, usize) {
    let mut body = composition_body(&base());
    reassign_node_id(&mut body, &node_id(LIVE_CLAUSE), clause);
    composed_with(body, |_| {})
}

fn read_with(package: &Value, dependency: &Value) -> CheckedPackageV2ReadResult {
    read_composition_with_dependency(package, dependency)
}

#[trace("TC-474", "FR-038-AC-539")]
#[test]
fn tc_474_a_composition_admits_foreign_clauses_named_by_dependency_positions() {
    let (dependency, anchor) = derived_dependency();
    let (package, position) = foreign_composition(&foreign_clause(DEPENDENCY, anchor));
    let CheckedPackageV2ReadResult::Admitted(admitted) = read_with(&package, &dependency) else {
        panic!("a clause of the dependency named by its own position admits");
    };
    // The dependency's nodes are not copied into the importer: its graph holds
    // none, and the anchor's position is not one of the importer's.
    let graph = admitted.graph();
    assert!(graph.nodes.iter().all(|node| node
        .node_id
        .path
        .iter()
        .all(|segment| *segment != CheckedNodePathSegment::Name(ANCHOR.into()))));
    assert!(anchor >= u64::try_from(graph.nodes.len()).expect("count"));
    // The row names the dependency position, and the typed body keeps it.
    let wanted = CheckedOperandEntry::Dependency(CheckedDependencyOperand {
        package: DEPENDENCY.into(),
        node: anchor,
    });
    assert!(graph.operand_rows[position].contains(&wanted));
    let node = &admitted.resolved_graph().nodes[position];
    let Some(CheckedTerm::ComponentComposition { guarantees, .. }) = admitted.term(&node.node_id)
    else {
        panic!("an admitted composition has a typed body");
    };
    let held = guarantees
        .iter()
        .flat_map(|entry| &entry.clause.path)
        .any(|segment| {
            *segment
                == CheckedNodePathSegment::Dependency {
                    package: DEPENDENCY.into(),
                    node: anchor,
                }
        });
    assert!(held, "the foreign clause keeps its dependency position");
    // The graph as shown is the wire form the reader admits again.
    let canonical_bytes = canonical(&wire_form(&package));
    let reread = crate::read_and_account(&canonical_bytes, CheckedPackageReadLimits::bounded(), &{
        let mut evidence = evidence_with_document(&package, ORDERS, &orders_document());
        let dependency = match read(&dependency) {
            CheckedPackageV2ReadResult::Admitted(dependency) => *dependency,
            other => panic!("the dependency admits, read {other:?}"),
        };
        evidence
            .insert_dependency_package(DEPENDENCY, dependency)
            .expect("dependency evidence");
        evidence
    });
    assert!(matches!(reread, CheckedPackageV2ReadResult::Admitted(_)));
}

#[trace("TC-474", "FR-038-AC-540")]
#[test]
fn tc_474_a_dependency_position_naming_no_dependency_node_is_malformed() {
    let (dependency, anchor) = derived_dependency();
    let count = u64::try_from(nodes(&dependency).len()).expect("count");
    let clause = foreign_clause(DEPENDENCY, anchor);
    let (_, position) = foreign_composition(&clause);
    let row_entry = format!("/semantic_graph/operand_rows/{position}/0");
    // An unknown package.
    let unknown = foreign_clause("no/such-package", anchor);
    let (package, _) = foreign_composition(&unknown);
    assert_eq!(
        match read_with(&package, &dependency) {
            CheckedPackageV2ReadResult::Refused(refusal) => (
                refusal.code,
                refusal.loci[0]
                    .path
                    .as_ref()
                    .map(|path| path.as_str().to_owned()),
            ),
            other => panic!("an unknown dependency package must refuse, read {other:?}"),
        },
        (Code::MalformedWire, Some(row_entry.clone()))
    );
    // A position past the dependency's last node.
    let past = foreign_clause(DEPENDENCY, count);
    let (package, _) = foreign_composition(&past);
    assert_eq!(
        match read_with(&package, &dependency) {
            CheckedPackageV2ReadResult::Refused(refusal) => (
                refusal.code,
                refusal.loci[0]
                    .path
                    .as_ref()
                    .map(|path| path.as_str().to_owned()),
            ),
            other => panic!("a position past the dependency must refuse, read {other:?}"),
        },
        (Code::MalformedWire, Some(row_entry.clone()))
    );
    // A local index in a foreign id names no node of the dependency.
    let local = json!({"package": DEPENDENCY,
        "path": ["temporal", "temporal_clause", {"node": 0}], "ordinal": 0});
    let (package, position) = foreign_composition(&local);
    match read_with(&package, &dependency) {
        CheckedPackageV2ReadResult::Refused(refusal) => {
            assert_eq!(refusal.code, Code::MissingDeclaration, "{refusal:?}");
            assert_eq!(refusal.cause, Some(Cause::MissingName), "{refusal:?}");
            assert!(
                refusal.loci[0]
                    .path
                    .as_ref()
                    .is_some_and(|path| path.as_str().starts_with(&at(position, "/body"))),
                "{refusal:?}"
            );
        }
        other => panic!("a local index in a foreign id must refuse, read {other:?}"),
    }
    // A `{package, node}` segment is a row entry only: written in an id's
    // path on the wire it is refused as malformed.
    let (package, position) = foreign_composition(&clause);
    let mut wire = wire_form(&package);
    wire["semantic_graph"]["nodes"][position]["dependencies"][0]["path"] =
        json!([{"package": DEPENDENCY, "node": anchor}]);
    match read_with(&wire, &dependency) {
        CheckedPackageV2ReadResult::Refused(refusal) => {
            assert_eq!(refusal.code, Code::MalformedWire, "{refusal:?}");
            assert_eq!(
                refusal.loci[0]
                    .path
                    .as_ref()
                    .map(|path| path.as_str().to_owned()),
                Some(format!(
                    "/semantic_graph/nodes/{position}/dependencies/0/path/0"
                )),
                "{refusal:?}"
            );
        }
        other => panic!("a dependency position in a path must refuse, read {other:?}"),
    }
}

#[trace("TC-474", "FR-038-AC-540")]
#[test]
fn tc_474_a_node_own_id_refuses_a_slot_naming_a_dependency_position() {
    let (dependency, anchor) = derived_dependency();
    let (package, position) = foreign_composition(&foreign_clause(DEPENDENCY, anchor));
    let mut wire = wire_form(&package);
    // Row entry 0 of this node is the dependency position; its own id now
    // holds slot 0, which can name only a node of its own graph.
    assert_eq!(
        wire["semantic_graph"]["operand_rows"][position][0],
        json!({"node": anchor, "package": DEPENDENCY})
    );
    wire["semantic_graph"]["nodes"][position]["node_id"]["path"] =
        json!(["composition", {"operand": 0}]);
    match read_with(&wire, &dependency) {
        CheckedPackageV2ReadResult::Refused(refusal) => {
            assert_eq!(refusal.code, Code::MalformedWire, "{refusal:?}");
            assert_eq!(
                refusal.loci[0]
                    .path
                    .as_ref()
                    .map(|path| path.as_str().to_owned()),
                Some(format!(
                    "/semantic_graph/nodes/{position}/node_id/path/1/operand"
                )),
                "{refusal:?}"
            );
        }
        other => panic!("a node's own id naming a dependency position must refuse, read {other:?}"),
    }
}

// FSpec FR-322 "Composition declaration names" and FR-491 "Composition member
// occurrences": a composition declares its name, carries one name row and one
// `generated` occurrence per body member with the member's authored region.

/// The canonical member count of the composition `composed` builds.
const MEMBERS: usize = 9;
/// The whole declaration, and the literal authored region of each member by
/// its canonical ordinal. The two guarantees (ordinals 2 and 3) are authored
/// in the reverse of their canonical order.
const DECLARATION: (u64, u64) = (1_000, 2_000);
const AUTHORED: [(u64, u64); MEMBERS] = [
    (1_100, 1_110),
    (1_120, 1_130),
    (1_500, 1_510),
    (1_200, 1_210),
    (1_300, 1_310),
    (1_400, 1_410),
    (1_600, 1_610),
    (1_700, 1_710),
    (1_800, 1_810),
];

fn graph_nodes(package: &Value) -> &Vec<Value> {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
}

fn region_value(package: &Value, source: usize, (start, end): (u64, u64)) -> Value {
    json!({"source": package["lock"]["sources"][source], "start": start, "end": end})
}

/// Writes literal authored regions over the composition at `position`: its
/// declaration entry, each member entry by ordinal, and its name row.
fn author(package: &mut Value, position: usize, declaration: (u64, u64), members: &[(u64, u64)]) {
    let id = graph_nodes(package)[position]["node_id"].clone();
    let whole = region_value(package, 0, declaration);
    let regions: Vec<Value> = members
        .iter()
        .map(|region| region_value(package, 0, *region))
        .collect();
    for entry in package["source_map"].as_array_mut().expect("map") {
        if entry["node_id"] != id {
            continue;
        }
        entry["regions"] = if entry["role"] == "declaration" {
            json!([whole])
        } else {
            let ordinal = entry["ordinal"].as_u64().expect("ordinal");
            json!([regions[usize::try_from(ordinal).expect("ordinal")]])
        };
    }
    for row in package["claim_names"].as_array_mut().expect("rows") {
        if row["declaration"] == id {
            row["declaration_region"] = whole.clone();
        }
    }
}

/// A valid authored composition and its node position.
fn authored() -> (Value, usize) {
    let (mut package, position) = composed(|_| {});
    author(&mut package, position, DECLARATION, &AUTHORED);
    (package, position)
}

/// The entry index of the composition's `role`/`ordinal` entry.
fn entry_index(package: &Value, position: usize, role: &str, ordinal: u64) -> usize {
    let id = &graph_nodes(package)[position]["node_id"];
    package["source_map"]
        .as_array()
        .expect("map")
        .iter()
        .position(|entry| {
            &entry["node_id"] == id && entry["role"] == role && entry["ordinal"] == ordinal
        })
        .expect("composition entry")
}

fn member_entry(package: &Value, position: usize, ordinal: u64) -> usize {
    entry_index(package, position, "generated", ordinal)
}

fn claim_row(package: &Value, position: usize) -> usize {
    let id = &graph_nodes(package)[position]["node_id"];
    package["claim_names"]
        .as_array()
        .expect("rows")
        .iter()
        .position(|row| &row["declaration"] == id)
        .expect("composition name row")
}

/// Asserts the refusal's code, cause and full primary pointer, and that it names no
/// node: the package-wide checks locate no node.
fn refused_unlocated(case: &str, package: &Value, code: Code, cause: Option<Cause>, path: &str) {
    let refusal = match read_any(package) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("{case}: expected a refusal, read {other:?}"),
    };
    assert_eq!(refusal.code, code, "{case}: {refusal:?}");
    assert_eq!(refusal.cause, cause, "{case}: {refusal:?}");
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|path| path.as_str()),
        Some(path),
        "{case}: {refusal:?}"
    );
    assert_eq!(refusal.locus, None, "{case}: {refusal:?}");
}

/// Asserts the refusal's code, cause, full primary pointer and node locus.
fn refused_at(
    case: &str,
    package: &Value,
    position: usize,
    code: Code,
    cause: Option<Cause>,
    path: &str,
) {
    let refusal = match read_any(package) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("{case}: expected a refusal, read {other:?}"),
    };
    assert_eq!(refusal.code, code, "{case}: {refusal:?}");
    assert_eq!(refusal.cause, cause, "{case}: {refusal:?}");
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|path| path.as_str()),
        Some(path),
        "{case}: {refusal:?}"
    );
    assert_eq!(
        refusal.locus,
        Some(typed(&graph_nodes(package)[position]["node_id"])),
        "{case}: {refusal:?}"
    );
}

const INVALID_VALUE: (Code, Option<Cause>) = (Code::InvalidPackage, Some(Cause::InvalidValue));

/// A mutation of the authored package, the entry index it refuses at (`None`
/// for the whole `/source_map`).
type Coordinate = (&'static str, fn(&mut Value, usize), Option<u64>);

fn rewrite_entry(
    package: &mut Value,
    position: usize,
    ordinal: u64,
    edit: impl FnOnce(&mut Value),
) {
    let at = member_entry(package, position, ordinal);
    edit(&mut package["source_map"][at]);
}

#[trace("TC-474", "FR-038-AC-602")]
#[test]
fn a_composition_names_itself_and_keeps_authored_member_regions() {
    let (package, position) = authored();
    let admitted = admitted("authored composition", &package);
    let node = &admitted.graph().nodes[position];
    // The declared name is the node's path and its one name row.
    assert_eq!(
        node.declaration
            .as_ref()
            .map(|name| name.qualified_name.clone()),
        Some(vec!["Sys".into()])
    );
    assert_eq!(
        node.node_id.path,
        vec![CheckedNodePathSegment::Name("Sys".into())]
    );
    let row = admitted
        .claim_name_for_path(&["Sys"])
        .expect("the composition's name row");
    assert_eq!(&*row.name, "Sys");
    assert_eq!(row.declaration, node.node_id);
    assert_eq!(row.occurrence.ordinal, 0);
    assert_eq!(
        (row.declaration_region.start, row.declaration_region.end),
        DECLARATION
    );
    assert_eq!(admitted.claim_names().expect("rows").len(), 1);
    // The member sequence is the canonical concatenation: the two guarantees
    // are ordinals 2 and 3, and each reads back with its own literal region.
    let held = |ordinal: u64| {
        admitted
            .source_map()
            .iter()
            .find(|entry| {
                entry.node_id == node.node_id
                    && format!("{:?}", entry.role) == "Generated"
                    && entry.ordinal == ordinal
            })
            .map(|entry| {
                entry
                    .regions
                    .iter()
                    .map(|region| (region.start, region.end))
                    .collect::<Vec<_>>()
            })
    };
    for (ordinal, region) in AUTHORED.iter().enumerate() {
        assert_eq!(
            held(ordinal as u64),
            Some(vec![*region]),
            "member {ordinal}"
        );
    }
    // Source order is the region order, not canonical order and not array order.
    assert!(AUTHORED[3].0 < AUTHORED[2].0);
    // Reordering the entries changes no attribution.
    let mut shuffled = package.clone();
    shuffled["source_map"]
        .as_array_mut()
        .expect("map")
        .reverse();
    let reordered = admitted_ok_package("reversed source map", &shuffled);
    assert_eq!(reordered.claim_names(), admitted.claim_names());
    let attribution = |package: &CheckedPackageV2| {
        let mut entries: Vec<_> = package
            .source_map()
            .iter()
            .map(|entry| format!("{entry:?}"))
            .collect();
        entries.sort();
        entries
    };
    assert_eq!(attribution(&reordered), attribution(&admitted));
}

fn admitted_ok_package(case: &str, package: &Value) -> CheckedPackageV2 {
    admitted(case, package)
}

#[trace("TC-474", "FR-038-AC-602")]
#[test]
fn a_written_composition_reads_back_with_its_name_and_coordinates() {
    let (package, _) = authored();
    let admitted = admitted("authored composition", &package);
    let written = encode_checked_package_v2(
        admitted.name(),
        admitted.lock(),
        admitted.graph(),
        CheckedPackageSourceCorrespondenceRef {
            source_map: admitted.source_map(),
            refinement_step_rows: admitted.refinement_step_rows(),
        },
        filament_contract_model::CheckedPackageSidecarsRef {
            temporal_clause_names: admitted.temporal_clause_names(),
            claim_names: admitted.claim_names(),
            operation_effects: None,
            operation_bodies: None,
            checked_term_data: admitted.checked_term_data(),
            temporal_bindings: admitted.temporal_bindings(),
            schedulability_claims: None,
        },
        admitted.capability_report(),
        admitted.diagnostics(),
        quire_canonical::Limits::new(u64::MAX),
    )
    .expect("writer");
    let dependency = match read(&base()) {
        CheckedPackageV2ReadResult::Admitted(dependency) => *dependency,
        other => panic!("the component package admits, read {other:?}"),
    };
    let mut evidence = evidence_with_document(&package, ORDERS, &orders_document());
    evidence
        .insert_dependency_package(DEPENDENCY, dependency)
        .expect("dependency evidence");
    let reread =
        match crate::read_and_account(&written, CheckedPackageReadLimits::bounded(), &evidence) {
            CheckedPackageV2ReadResult::Admitted(package) => *package,
            other => panic!("the written package reads back, read {other:?}"),
        };
    assert_eq!(reread.graph(), admitted.graph());
    assert_eq!(reread.claim_names(), admitted.claim_names());
    assert_eq!(reread.source_map(), admitted.source_map());
}

#[trace("TC-474", "FR-038-AC-603")]
#[test]
fn an_implementation_guarantee_declaration_stays_refused() {
    // A guarantee stays declaration-free.
    let (mut package, position) = guaranteed(|_| {});
    package["semantic_graph"]["nodes"][position]["declaration"] = json!({"qualified_name": ["G"]});
    package["semantic_graph"]["nodes"][position]["owner"] = source_owner(&fixture_source());
    expect(
        "a guarantee with a declaration",
        &package,
        GRAPH,
        None,
        &at(position, "/declaration"),
    );
}

#[trace("TC-474", "FR-038-AC-606")]
#[test]
fn a_composition_with_a_declaration_occurrence_requires_its_declaration() {
    let (package, position) = authored();
    admitted("positive control", &package);
    let mut package = package;
    let node = package["semantic_graph"]["nodes"][position]
        .as_object_mut()
        .expect("node");
    node.remove("declaration");
    node.remove("owner");
    expect(
        "a composition without its declaration",
        &package,
        GRAPH,
        None,
        &at(position, ""),
    );
}

#[trace("TC-474", "FR-038-AC-607")]
#[test]
fn a_composition_declaration_other_than_its_node_path_refuses_as_nominal() {
    let (package, position) = authored();
    let mut package = package;
    package["semantic_graph"]["nodes"][position]["declaration"] =
        json!({"qualified_name": ["Other"]});
    let row = claim_row(&package, position);
    package["claim_names"][row]["name"] = json!("Other");
    package["claim_names"][row]["qualified_name"] = json!(["Other"]);
    expect(
        "a declaration other than the node path",
        &package,
        Code::InvalidPackage,
        Some(Cause::DeclarationNominalMismatch),
        &at(position, "/node_id/path"),
    );
}

#[trace("TC-477", "FR-371-AC-12")]
#[test]
fn a_composition_name_row_is_exact_and_unique() {
    let (package, position) = authored();
    admitted("positive control", &package);
    let row = claim_row(&package, position);
    let row_at = |tail: &str| format!("/claim_names/{row}{tail}");
    // No row at all, and no occurrence left to name.
    let mut missing = package.clone();
    missing["claim_names"]
        .as_array_mut()
        .expect("rows")
        .remove(row);
    missing
        .as_object_mut()
        .expect("package")
        .remove("claim_names");
    refused_unlocated(
        "a missing name row",
        &missing,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/claim_names",
    );
    // A path other than the declared one whose last segment matches.
    let mut wrong = package.clone();
    wrong["claim_names"][row]["qualified_name"] = json!(["Scope", "Sys"]);
    refused_at(
        "a wrong qualified name",
        &wrong,
        position,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        &row_at("/qualified_name"),
    );
    // A region that contains the declaration, or merely lies inside it, is
    // not the declaration's region.
    for (case, region) in [
        ("a larger containing region", (900, 2_100)),
        ("a smaller contained region", (1_000, 1_999)),
    ] {
        let mut other = package.clone();
        other["claim_names"][row]["declaration_region"] = region_value(&package, 0, region);
        refused_at(
            case,
            &other,
            position,
            Code::InvalidPackage,
            Some(Cause::InvalidValue),
            &row_at("/declaration_region"),
        );
    }
    // A row for any occurrence other than the declaration at ordinal 0.
    let mut claim = package.clone();
    claim["claim_names"][row]["occurrence"] = json!({"role": "claim", "ordinal": 0});
    refused_at(
        "a claim-role row",
        &claim,
        position,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        &row_at("/occurrence"),
    );
}

#[trace("TC-477", "FR-371-AC-12")]
#[test]
fn two_distinct_compositions_cannot_share_a_qualified_name() {
    let (mut package, position) = authored();
    let mut twin = graph_nodes(&package)[position].clone();
    twin["node_id"]["ordinal"] = json!(1);
    twin["semantic_type"] = twin["node_id"].clone();
    push(&mut package, twin);
    settle(&mut package);
    let second = graph_nodes(&package).len() - 1;
    author(&mut package, position, DECLARATION, &AUTHORED);
    author(
        &mut package,
        second,
        (3_000, 4_000),
        &AUTHORED.map(|(s, e)| (s + 2_100, e + 2_100)),
    );
    refused_as_ambiguous("two compositions of one declared name", &package, second);
    // The same two compositions under distinct names admit.
    let mut distinct = package.clone();
    distinct["semantic_graph"]["nodes"][second]["node_id"]["path"] = json!(["Alpha"]);
    distinct["semantic_graph"]["nodes"][second]["semantic_type"] =
        distinct["semantic_graph"]["nodes"][second]["node_id"].clone();
    distinct["semantic_graph"]["nodes"][second]["declaration"] =
        json!({"qualified_name": ["Alpha"]});
    settle(&mut distinct);
    let alpha = graph_nodes(&distinct)
        .iter()
        .position(|node| node["node_id"]["path"] == json!(["Alpha"]))
        .expect("renamed composition");
    author(
        &mut distinct,
        alpha,
        (3_000, 4_000),
        &AUTHORED.map(|(s, e)| (s + 2_100, e + 2_100)),
    );
    let admitted = admitted("two compositions of distinct names", &distinct);
    assert_eq!(admitted.claim_names().expect("rows").len(), 2);
}

/// Asserts the later of two compositions' name rows refuses as ambiguous.
fn refused_as_ambiguous(case: &str, package: &Value, second: usize) {
    let later = claim_row(package, second);
    refused_at(
        case,
        package,
        second,
        Code::AmbiguousDeclaration,
        Some(Cause::AmbiguousName),
        &format!("/claim_names/{later}/qualified_name"),
    );
}

#[trace("TC-474", "FR-038-AC-604")]
#[test]
fn a_member_coordinate_that_is_not_the_one_authored_region_refuses_at_its_entry() {
    let (package, position) = authored();
    admitted("positive control", &package);
    let cases: [Coordinate; 7] = [
        (
            "an ordinal equal to the member count",
            |package, position| {
                let last = MEMBERS as u64 - 1;
                let id = graph_nodes(package)[position]["node_id"].clone();
                for occurrence in package["semantic_graph"]["nodes"][position]["occurrences"]
                    .as_array_mut()
                    .expect("occurrences")
                {
                    if occurrence["ordinal"] == last && occurrence["role"] == "generated" {
                        occurrence["ordinal"] = json!(MEMBERS);
                    }
                }
                rewrite_entry(package, position, last, |entry| {
                    assert_eq!(entry["node_id"], id);
                    entry["ordinal"] = json!(MEMBERS);
                });
            },
            Some(MEMBERS as u64 - 1),
        ),
        (
            "another locked source",
            |package, position| {
                let secondary = crate::support::checked_package::fixture_secondary_source();
                package["lock"]["sources"]
                    .as_array_mut()
                    .expect("sources")
                    .push(secondary.clone());
                rewrite_entry(package, position, 4, |entry| {
                    entry["regions"][0]["source"] = secondary;
                });
            },
            Some(4),
        ),
        (
            "outside the declaration",
            |package, position| {
                rewrite_entry(package, position, 4, |entry| {
                    entry["regions"][0]["start"] = json!(2_100);
                    entry["regions"][0]["end"] = json!(2_110);
                });
            },
            Some(4),
        ),
        // The first member entry has no earlier member to overlap, so only the
        // declaration interval itself refuses this one.
        (
            "equal to the declaration",
            |package, position| {
                rewrite_entry(package, position, 0, |entry| {
                    entry["regions"][0]["start"] = json!(DECLARATION.0);
                    entry["regions"][0]["end"] = json!(DECLARATION.1);
                });
            },
            Some(0),
        ),
        (
            "overlapping an earlier member",
            |package, position| {
                rewrite_entry(package, position, 5, |entry| {
                    entry["regions"][0]["start"] = json!(1_305);
                    entry["regions"][0]["end"] = json!(1_315);
                });
            },
            Some(5),
        ),
        (
            "two disjoint regions",
            |package, position| {
                rewrite_entry(package, position, 5, |entry| {
                    let second = json!({
                        "source": entry["regions"][0]["source"],
                        "start": 1_900, "end": 1_910,
                    });
                    entry["regions"]
                        .as_array_mut()
                        .expect("regions")
                        .push(second);
                });
            },
            Some(5),
        ),
        (
            "an expression role",
            |package, position| {
                package["semantic_graph"]["nodes"][position]["occurrences"][6]["role"] =
                    json!("expression");
                rewrite_entry(package, position, 5, |entry| {
                    entry["role"] = json!("expression")
                });
            },
            Some(5),
        ),
    ];
    for (case, mutate, ordinal) in cases {
        let mut mutated = package.clone();
        mutate(&mut mutated, position);
        if case == "another locked source" {
            crate::support::checked_package::refresh_identity(&mut mutated);
        }
        let path = match ordinal {
            Some(_) if case == "an ordinal equal to the member count" => {
                format!(
                    "/source_map/{}",
                    member_entry(&package, position, MEMBERS as u64 - 1)
                )
            }
            Some(ordinal) => format!("/source_map/{}", member_entry(&package, position, ordinal)),
            None => format!(
                "/source_map/{}",
                entry_index(&package, position, "declaration", 0)
            ),
        };
        refused_at(
            case,
            &mutated,
            position,
            INVALID_VALUE.0,
            INVALID_VALUE.1,
            &path,
        );
    }
}

#[trace("TC-474", "FR-038-AC-613")]
#[test]
fn a_missing_member_occurrence_pair_refuses_only_after_every_entry_passes() {
    let (package, position) = authored();
    let drop_member = |package: &mut Value, ordinal: u64| {
        let at = member_entry(package, position, ordinal);
        package["source_map"]
            .as_array_mut()
            .expect("map")
            .remove(at);
        let occurrences = package["semantic_graph"]["nodes"][position]["occurrences"]
            .as_array_mut()
            .expect("occurrences");
        let held = occurrences
            .iter()
            .position(|occurrence| {
                occurrence["role"] == "generated" && occurrence["ordinal"] == ordinal
            })
            .expect("member occurrence");
        occurrences.remove(held);
    };
    let mut missing = package.clone();
    drop_member(&mut missing, 4);
    refused_at(
        "a removed occurrence and entry",
        &missing,
        position,
        INVALID_VALUE.0,
        INVALID_VALUE.1,
        "/source_map",
    );
    // An invalid entry is the first defect, wherever the missing pair is.
    let mut both = missing.clone();
    rewrite_entry(&mut both, position, 5, |entry| {
        entry["regions"][0]["start"] = json!(2_100);
        entry["regions"][0]["end"] = json!(2_110);
    });
    refused_at(
        "an invalid entry before the missing pair",
        &both,
        position,
        INVALID_VALUE.0,
        INVALID_VALUE.1,
        &format!("/source_map/{}", member_entry(&both, position, 5)),
    );
    // The first invalid entry in `source_map` order, not in ordinal order.
    let mut ordered = package.clone();
    for ordinal in [3, 6] {
        rewrite_entry(&mut ordered, position, ordinal, |entry| {
            entry["regions"][0]["start"] = json!(2_100);
            entry["regions"][0]["end"] = json!(2_110);
        });
    }
    let early = member_entry(&ordered, position, 6);
    let late = member_entry(&ordered, position, 3);
    ordered["source_map"]
        .as_array_mut()
        .expect("map")
        .swap(early, late);
    refused_at(
        "the first invalid entry in source-map order",
        &ordered,
        position,
        INVALID_VALUE.0,
        INVALID_VALUE.1,
        &format!(
            "/source_map/{}",
            member_entry(&ordered, position, 3).min(member_entry(&ordered, position, 6))
        ),
    );
}

#[trace("TC-474", "FR-038-AC-605")]
#[test]
fn generic_source_map_checks_precede_and_body_checks_follow_composition_coordinates() {
    let (package, position) = authored();
    // An empty interval is the generic `invalid_source_map`, ahead of any
    // composition coordinate check, even when a coordinate defect comes
    // earlier in the array.
    let mut empty = package.clone();
    rewrite_entry(&mut empty, position, 6, |entry| {
        entry["regions"][0]["start"] = json!(1_700);
        entry["regions"][0]["end"] = json!(1_700);
    });
    let at = member_entry(&empty, position, 6);
    refused_unlocated(
        "an empty interval",
        &empty,
        Code::InvalidSourceMap,
        None,
        &format!("/source_map/{at}/regions/0"),
    );
    let mut combined = empty.clone();
    rewrite_entry(&mut combined, position, 2, |entry| {
        entry["regions"][0]["start"] = json!(2_100);
        entry["regions"][0]["end"] = json!(2_110);
    });
    let (early, late) = (
        member_entry(&combined, position, 2),
        member_entry(&combined, position, 6),
    );
    assert!(
        early < late,
        "the coordinate defect comes first in the array"
    );
    refused_unlocated(
        "generic before coordinate",
        &combined,
        Code::InvalidSourceMap,
        None,
        &format!("/source_map/{late}/regions/0"),
    );
    // A repeated key is the generic refusal as well.
    let mut repeated = package.clone();
    let copy = repeated["source_map"][member_entry(&package, position, 4)].clone();
    repeated["source_map"]
        .as_array_mut()
        .expect("map")
        .push(copy);
    rewrite_entry(&mut repeated, position, 2, |entry| {
        entry["regions"][0]["start"] = json!(2_100);
        entry["regions"][0]["end"] = json!(2_110);
    });
    let last = repeated["source_map"].as_array().expect("map").len() - 1;
    refused_unlocated(
        "a repeated key",
        &repeated,
        Code::InvalidSourceMap,
        None,
        &format!("/source_map/{last}"),
    );
    // Totality defects are generic as well, and each beats a coordinate defect
    // that comes earlier in the array: an entry naming an unknown node, an
    // entry naming a node that holds no such occurrence, and an occurrence
    // left without an entry.
    let defect = |package: &mut Value| {
        rewrite_entry(package, position, 2, |entry| {
            entry["regions"][0]["start"] = json!(2_100);
            entry["regions"][0]["end"] = json!(2_110);
        });
    };
    let other = (0..graph_nodes(&package).len())
        .find(|other| *other != position)
        .expect("another node");
    let mut unknown = package.clone();
    defect(&mut unknown);
    let late = member_entry(&unknown, position, 7);
    assert!(member_entry(&unknown, position, 2) < late);
    unknown["source_map"][late]["node_id"]["path"] = json!(["Nowhere"]);
    refused_unlocated(
        "an entry naming an unknown node",
        &unknown,
        Code::InvalidSourceMap,
        None,
        &format!("/source_map/{late}"),
    );
    let mut foreign = package.clone();
    defect(&mut foreign);
    foreign["source_map"][late]["node_id"] = graph_nodes(&package)[other]["node_id"].clone();
    refused_unlocated(
        "an entry naming another node's missing occurrence",
        &foreign,
        Code::InvalidSourceMap,
        None,
        &format!("/source_map/{late}"),
    );
    let mut total = package.clone();
    defect(&mut total);
    total["source_map"]
        .as_array_mut()
        .expect("map")
        .remove(late);
    refused_unlocated(
        "an occurrence without an entry",
        &total,
        Code::InvalidSourceMap,
        None,
        "/source_map",
    );
}

#[trace("TC-474", "FR-038-AC-610")]
#[test]
fn identity_dependency_and_order_checks_precede_coordinates_and_member_checks_follow() {
    let (package, position) = authored();
    // Identity, dependency and canonical-order defects precede coordinates.
    let mut order = package.clone();
    order["semantic_graph"]["nodes"][position]["body"]["guarantees"]
        .as_array_mut()
        .expect("guarantees")
        .reverse();
    rewrite_entry(&mut order, position, 4, |entry| {
        entry["regions"][0]["start"] = json!(2_100);
        entry["regions"][0]["end"] = json!(2_110);
    });
    refused_at(
        "an out-of-order array before coordinates",
        &order,
        position,
        Code::InvalidSemanticGraph,
        None,
        &at_node(position, "/body/guarantees"),
    );
    let mut identity = package.clone();
    identity["semantic_graph"]["nodes"][position]["dependencies"] = json!([]);
    rewrite_entry(&mut identity, position, 4, |entry| {
        entry["regions"][0]["start"] = json!(2_100);
        entry["regions"][0]["end"] = json!(2_110);
    });
    refused_at(
        "a dependency defect before coordinates",
        &identity,
        position,
        Code::InvalidSemanticGraph,
        None,
        &at_node(position, ""),
    );
    // Coordinates precede the member static checks.
    let mut static_check = package.clone();
    static_check["semantic_graph"]["nodes"][position]["body"]["components"][1]["name"] =
        json!("Consumer");
    rewrite_entry(&mut static_check, position, 4, |entry| {
        entry["regions"][0]["start"] = json!(2_100);
        entry["regions"][0]["end"] = json!(2_110);
    });
    refused_at(
        "a coordinate defect before a member static check",
        &static_check,
        position,
        INVALID_VALUE.0,
        INVALID_VALUE.1,
        &format!("/source_map/{}", member_entry(&static_check, position, 4)),
    );
}

fn at_node(position: usize, tail: &str) -> String {
    at(position, tail)
}

#[trace("TC-477", "FR-371-AC-15")]
#[test]
fn the_reader_returns_each_compositions_literal_region_whatever_the_entry_order() {
    let (mut package, sys) = authored();
    let mut twin = graph_nodes(&package)[sys].clone();
    twin["node_id"]["path"] = json!(["Alpha"]);
    twin["semantic_type"] = twin["node_id"].clone();
    twin["declaration"] = json!({"qualified_name": ["Alpha"]});
    let alpha = push(&mut package, twin);
    settle(&mut package);
    author(&mut package, sys, DECLARATION, &AUTHORED);
    author(
        &mut package,
        alpha,
        (5_000, 6_000),
        &AUTHORED.map(|(s, e)| (s + 4_000, e + 4_000)),
    );
    // The reader returns each name's declaration region as authored. It does
    // not select or order source units: a consumer owns that, from these
    // regions, so the oracle is the literal region of each canonical path.
    let regions = |package: &CheckedPackageV2| {
        ["Sys", "Alpha"].map(|name| {
            let row = package.claim_name_for_path(&[name]).expect("name row");
            (row.declaration_region.start, row.declaration_region.end)
        })
    };
    let forward = admitted("two compositions", &package);
    assert_eq!(regions(&forward), [(1_000, 2_000), (5_000, 6_000)]);
    let mut reversed = package.clone();
    reversed["source_map"]
        .as_array_mut()
        .expect("map")
        .reverse();
    let backward = admitted("reversed entries", &reversed);
    assert_eq!(regions(&backward), [(1_000, 2_000), (5_000, 6_000)]);
    // Rows keep canonical node-id order, not the order of the source regions.
    let names: Vec<&str> = forward
        .claim_names()
        .expect("rows")
        .iter()
        .map(|row| &*row.name)
        .collect();
    assert_eq!(names, ["Alpha", "Sys"]);
}

/// The composition's `claim` occurrence and entry at `region`, and a name row
/// for it that repeats the row of the `declaration` occurrence.
fn with_claim_row(package: &mut Value, position: usize, region: (u64, u64)) -> usize {
    let id = graph_nodes(package)[position]["node_id"].clone();
    package["semantic_graph"]["nodes"][position]["occurrences"]
        .as_array_mut()
        .expect("occurrences")
        .push(json!({"role": "claim", "ordinal": 0}));
    let entry = json!({"node_id": id, "role": "claim", "ordinal": 0,
        "regions": [region_value(package, 0, region)]});
    package["source_map"]
        .as_array_mut()
        .expect("map")
        .push(entry);
    let mut row = package["claim_names"][claim_row(package, position)].clone();
    row["occurrence"] = json!({"role": "claim", "ordinal": 0});
    row["declaration_region"] = region_value(package, 0, region);
    let rows = package["claim_names"].as_array_mut().expect("rows");
    rows.push(row);
    rows.len() - 1
}

#[trace("TC-477", "FR-371-AC-13")]
#[test]
fn a_name_row_for_a_composition_claim_occurrence_is_ineligible() {
    let (package, position) = authored();
    // A row naming the held `claim` occurrence, with a region equal to its
    // entry and the declared name, is refused for the occurrence alone.
    let mut named = package.clone();
    let row = with_claim_row(&mut named, position, (1_900, 1_950));
    refused_at(
        "a row for the composition's claim occurrence",
        &named,
        position,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        &format!("/claim_names/{row}/occurrence"),
    );
}

#[trace("TC-477", "FR-371-AC-14")]
#[test]
fn a_name_row_for_another_correspondence_form_keeps_its_ineligibility() {
    let (package, guarantee) = guaranteed(|_| {});
    // Without a row for the guarantee the package admits.
    admitted("no row for the guarantee", &package);
    let mut named = package.clone();
    let row = json!({
        "name": "G",
        "qualified_name": ["G"],
        "declaration": graph_nodes(&named)[guarantee]["node_id"].clone(),
        "occurrence": {"role": "declaration", "ordinal": 0},
        "declaration_region": region_value(&named, 0, (1_000, 2_000)),
    });
    named["claim_names"] = json!([row]);
    refused_at(
        "a row for an implementation guarantee",
        &named,
        guarantee,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/claim_names/0/declaration",
    );
}

/// Refuses member 0 rewritten to `region`, at its own entry.
fn shared_boundary(case: &str, package: &Value, position: usize, region: (u64, u64)) {
    let mut mutated = package.clone();
    rewrite_entry(&mut mutated, position, 0, |entry| {
        entry["regions"][0]["start"] = json!(region.0);
        entry["regions"][0]["end"] = json!(region.1);
    });
    refused_at(
        case,
        &mutated,
        position,
        INVALID_VALUE.0,
        INVALID_VALUE.1,
        &format!("/source_map/{}", member_entry(package, position, 0)),
    );
}

#[trace("TC-474", "FR-038-AC-608")]
#[test]
fn a_member_starting_at_the_declaration_start_refuses_at_its_entry() {
    let (package, position) = authored();
    let mut inside = package.clone();
    rewrite_entry(&mut inside, position, 0, |entry| {
        entry["regions"][0]["start"] = json!(DECLARATION.0 + 1);
        entry["regions"][0]["end"] = json!(DECLARATION.0 + 11);
    });
    admitted("one unit inside the start", &inside);
    shared_boundary(
        "starting at the declaration start",
        &package,
        position,
        (DECLARATION.0, DECLARATION.0 + 10),
    );
}

#[trace("TC-474", "FR-038-AC-609")]
#[test]
fn a_member_ending_at_the_declaration_end_refuses_at_its_entry() {
    let (package, position) = authored();
    let mut inside = package.clone();
    rewrite_entry(&mut inside, position, 0, |entry| {
        entry["regions"][0]["start"] = json!(DECLARATION.1 - 11);
        entry["regions"][0]["end"] = json!(DECLARATION.1 - 1);
    });
    admitted("one unit inside the end", &inside);
    shared_boundary(
        "ending at the declaration end",
        &package,
        position,
        (DECLARATION.1 - 10, DECLARATION.1),
    );
}

/// Moves every structural node id of the importer to package `name`.
#[cfg(feature = "test-support")]
fn rehome(value: &mut Value, name: &str) {
    match value {
        Value::Object(object) => {
            if object.get("package") == Some(&json!(IMPORTER))
                && object.contains_key("path")
                && object.contains_key("ordinal")
            {
                object.insert("package".to_owned(), json!(name));
            }
            object.values_mut().for_each(|member| rehome(member, name));
        }
        Value::Array(items) => items.iter_mut().for_each(|item| rehome(item, name)),
        _ => {}
    }
}

/// An authored composition without an environment member beside an empty
/// refinement, in the refinement fixture's package: the phases that need the
/// package-wide source-map check meet in one read. Returns the package, its
/// evidence and the number of `source_map` entries.
#[cfg(feature = "test-support")]
fn hosted() -> (Value, filament_contract_model::CheckedPackageEvidence) {
    let (mut host, mut evidence) = crate::checked_package_v2_refinement::machine_package();
    crate::checked_package_v2_refinement::row_empty_refinement(&mut host);
    let name = host["name"].as_str().expect("name").to_owned();
    let (mut guest, position) = composed(|body| body["environment"] = json!([]));
    author(&mut guest, position, DECLARATION, &AUTHORED);
    rehome(&mut guest, &name);
    let composition = nodes(&guest)[position].clone();
    let id = composition["node_id"].clone();
    host["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(composition);
    let source = host["lock"]["sources"][0].clone();
    for entry in guest["source_map"].as_array().expect("map") {
        if entry["node_id"] == id {
            let mut entry = entry.clone();
            for region in entry["regions"].as_array_mut().expect("regions") {
                region["source"] = source.clone();
            }
            host["source_map"].as_array_mut().expect("map").push(entry);
        }
    }
    host["lock"]["dependency_selections"] = json!([{"identity": DEPENDENCY}]);
    let dependency = match read(&base()) {
        CheckedPackageV2ReadResult::Admitted(dependency) => *dependency,
        other => panic!("the component package admits, read {other:?}"),
    };
    evidence
        .insert_dependency_package(DEPENDENCY, dependency)
        .expect("dependency evidence");
    crate::support::checked_package::refresh_claim_names(&mut host);
    crate::support::checked_package::refresh_identity(&mut host);
    (host, evidence)
}

#[cfg(feature = "test-support")]
mod charge_events {
    use super::*;
    use filament_contract_model::{
        with_native_budget_observer, NativeBudgetEvent, NativeBudgetResource,
        NativeJoinLocusSource, PackageReadStage,
    };
    use std::cell::RefCell;

    thread_local! {
        static AMOUNTS: RefCell<Vec<u128>> = const { RefCell::new(Vec::new()) };
    }

    /// Records the read-storage requests of the package-wide source-map check.
    /// The buffer is reserved before the read: the observer must not allocate.
    fn record(event: NativeBudgetEvent) {
        let source_map = event.resource == NativeBudgetResource::ReadStorage
            && event.admitted
            && event.amount > 0
            && event.locus.is_some_and(|locus| {
                locus.form == "source_map"
                    && matches!(
                        locus.source,
                        NativeJoinLocusSource::PackageRead(PackageReadStage::SourceMap, _)
                    )
            });
        if source_map {
            AMOUNTS.with(|amounts| amounts.borrow_mut().push(event.amount));
        }
    }

    /// The sizes, ascending, of the tables one run of the check requests,
    /// from the package alone: the graph's occurrence keys (a node, a role
    /// and an ordinal: three words), the entries' keys with positions (four
    /// words), the locked sources (one word) and, per entry, its regions with
    /// their source, bounds and position (four words each).
    fn one_pass(package: &Value) -> Vec<u128> {
        let word = size_of::<usize>() as u128;
        let occurrences: usize = nodes(package)
            .iter()
            .map(|node| node["occurrences"].as_array().map_or(0, Vec::len))
            .sum();
        let entries = package["source_map"].as_array().expect("entries");
        let mut sizes = vec![
            occurrences as u128 * 3 * word,
            entries.len() as u128 * 4 * word,
            package["lock"]["sources"]
                .as_array()
                .expect("sources")
                .len() as u128
                * word,
        ];
        sizes.extend(
            entries.iter().map(|entry| {
                entry["regions"].as_array().expect("regions").len() as u128 * 4 * word
            }),
        );
        sizes.sort_unstable();
        sizes
    }

    /// The source-map check's requests during one admitting read of `bytes`.
    fn requests(
        bytes: &[u8],
        evidence: &filament_contract_model::CheckedPackageEvidence,
    ) -> Vec<u128> {
        AMOUNTS.with(|amounts| {
            let mut amounts = amounts.borrow_mut();
            amounts.clear();
            amounts.reserve(1 << 16);
        });
        let result = with_native_budget_observer(record, || {
            CheckedPackageV2::read(bytes, CheckedPackageReadLimits::bounded(), evidence)
        });
        assert!(
            matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
            "the package admits, read {result:?}"
        );
        let mut amounts = AMOUNTS.with(|amounts| amounts.take());
        amounts.sort_unstable();
        amounts
    }

    #[trace("TC-474", "FR-038-AC-612")]
    #[test]
    fn a_composition_read_requests_the_source_map_check_tables_once() {
        // A composition alone: the check runs for the coordinates and the
        // package stage then reads the recorded success.
        let (package, _) = authored();
        let dependency = match read(&base()) {
            CheckedPackageV2ReadResult::Admitted(dependency) => *dependency,
            other => panic!("the component package admits, read {other:?}"),
        };
        let mut evidence = evidence_with_document(&package, ORDERS, &orders_document());
        evidence
            .insert_dependency_package(DEPENDENCY, dependency)
            .expect("dependency evidence");
        assert_eq!(
            requests(&canonical(&package), &evidence),
            one_pass(&package)
        );
    }

    #[trace("TC-474", "FR-038-AC-612")]
    #[test]
    fn a_composition_beside_a_refinement_requests_the_source_map_check_tables_once() {
        // The refinement step and the composition both need the check, and the
        // package stage follows them: three phases, one run.
        let (host, evidence) = hosted();
        let indexed = crate::checked_package_v2_refinement::indexed_package(&host);
        assert_eq!(
            requests(&canonical(&indexed), &evidence),
            one_pass(&indexed)
        );
    }
}
