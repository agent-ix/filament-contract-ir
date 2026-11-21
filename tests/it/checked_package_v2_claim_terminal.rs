// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-060 and TC-061: public V2 read/write vectors for the two new body forms.

use crate::support::checked_package::{
    canonical, evidence_for, evidence_with_document, family_key, family_model_document,
    fixture_source, model_owner, node_id, refresh_identity, select_model, settle, sha256_hex,
    source_owner, v2_all_families,
};
use filament_contract_model::{
    encode_checked_package_v2, CheckedCapability, CheckedDiagnosticsV2, CheckedNodeId,
    CheckedOccurrence, CheckedPackageLimit, CheckedPackageLockV2, CheckedPackageReadLimits,
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedSemanticGraphV2, CheckedSourceMapEntry, TerminalDeclaration,
    TerminalRow,
};
use ix_trace_rs::trace;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::sync::OnceLock;

/// The shared V2 fixture grows as independent families become admissible.
fn base_node_index() -> usize {
    static COUNT: OnceLock<usize> = OnceLock::new();
    *COUNT.get_or_init(|| {
        v2_all_families()["semantic_graph"]["nodes"]
            .as_array()
            .expect("fixture nodes")
            .len()
    })
}

fn binding(name: &str, value: Value) -> Value {
    json!({"term": "binding", "name": name, "value": value})
}

fn empty() -> Value {
    json!({"term": "aggregate", "members": []})
}

fn reference(digest: &str) -> Value {
    json!({"term": "reference", "target": node_id(digest)})
}

fn word(value: &str) -> Value {
    json!({"term": "literal", "type": node_id(&family_key("a1a1")),
        "value_kind": "text", "value": value})
}

fn node(tag: &str, form: &str, body: Value, dependencies: &[String]) -> Value {
    let ty = family_key("aaaa");
    let named = match form {
        "state_graph" => body["members"][0]["value"]["value"].as_str(),
        "terminal" => body["members"][1]["value"]["value"].as_str(),
        _ => None,
    };
    let id = named.map_or_else(
        || format!("fixture/claim-terminal/{tag}/{form}"),
        |name| format!("fixture/claim-terminal/{tag}/{form}/{name}"),
    );
    let mut dependencies = dependencies
        .iter()
        .map(|key| node_id(key))
        .collect::<Vec<_>>();
    dependencies.sort_by_key(|id| {
        serde_json::from_value::<CheckedNodeId>(id.clone()).expect("dependency node ID")
    });
    dependencies.dedup();
    {
        let mut node = json!({
            "node_id": node_id(&id), "schema_version": "filament.checked-semantic-graph/v2",
            "node_tag": tag, "semantic_form": form, "semantic_type": node_id(&ty),
            "dependencies": dependencies,
            "occurrences": [{"role": "claim", "ordinal": 0}], "body": body,
        });
        crate::support::checked_package::carry_check_mode(&mut node);
        node
    }
}

fn state_graph(kind: &str, over: Value, from: Value, target: Value, deps: &[String]) -> Value {
    node(
        "claim",
        "state_graph",
        json!({"term": "aggregate", "members": [
            binding("kind", word(kind)), binding("over", over),
            binding("from", from), binding("target", target),
        ]}),
        deps,
    )
}

fn terminal(
    mode: &str,
    model: Value,
    parameters: Value,
    condition: Value,
    deps: &[String],
) -> Value {
    node(
        "temporal",
        "terminal",
        json!({"term": "aggregate", "members": [
            binding("model", model), binding("mode", word(mode)),
            binding("parameters", parameters), binding("condition", condition),
        ]}),
        deps,
    )
}

fn malformed_abstraction() -> Value {
    let body = json!({"term": "abstraction_relation", "objects": [{}],
        "populations": [], "frames": []});
    let key = sha256_hex(&canonical(&json!({
        "version": "filament.abstraction-relation-node/v1", "body": body
    })));
    json!({
        "node_id": node_id(&key), "schema_version": "filament.checked-semantic-graph/v2",
        "node_tag": "correspondence", "semantic_form": "abstraction_relation",
        "semantic_type": node_id(&key), "dependencies": [],
        "occurrences": [{"role": "declaration", "ordinal": 0}], "body": body,
    })
}

fn malformed_hybrid_model() -> Value {
    let owner = source_owner(&fixture_source());
    let declaration = json!({"qualified_name": ["Tank"]});
    let target = family_key("aaaa");
    let body = json!({"term": "aggregate", "members": [
        binding("variables", json!({"term": "aggregate", "members": [
            {"term": "aggregate", "members": [
                binding("parameter", reference(&target)), binding("class", word("clock")),
            ]},
        ]})),
        binding("modes", json!({"term": "aggregate", "members": [reference(&target)]})),
    ]});
    let key = sha256_hex(&canonical(&json!({
        "version": "filament.structural-node/v1", "node_tag": "hybrid",
        "semantic_form": "hybrid_model", "semantic_type": null,
        "declaration": declaration, "recursion": null, "owner": owner, "body": body,
    })));
    json!({
        "node_id": node_id(&key), "schema_version": "filament.checked-semantic-graph/v2",
        "node_tag": "hybrid", "semantic_form": "hybrid_model",
        "semantic_type": node_id(&key), "declaration": declaration, "owner": owner,
        "dependencies": [node_id(&target)],
        "occurrences": [{"role": "declaration", "ordinal": 0}], "body": body,
    })
}

/// A `possible` state-graph claim node (FR-059).
pub(super) fn claim_node() -> Value {
    let over = family_key("a4a4");
    let predicate = family_key("eeee");
    state_graph(
        "possible",
        reference(&over),
        empty(),
        reference(&predicate),
        &[over, predicate],
    )
}

fn package_with(node: Value) -> Value {
    package_with_nodes(vec![node])
}

fn package_with_nodes(nodes: Vec<Value>) -> Value {
    let mut package = v2_all_families();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend(nodes);
    settle(&mut package);
    package
}

fn terminal_with_self(target_model: &str, parameter_name: &str, level: &str) -> Value {
    let mut package = v2_all_families();
    let model = family_key("1515");
    let reference_body = json!({"term": "aggregate", "members": [reference(target_model)]});
    let reference_type = format!("fixture/claim-terminal/reference/{target_model}");
    let mut reference_node = node(
        "composite_type",
        "reference",
        reference_body,
        &[target_model.to_owned()],
    );
    reference_node["node_id"] = node_id(&reference_type);
    reference_node["semantic_type"] = node_id(&reference_type);
    reference_node["occurrences"] = json!([{"role": "generated", "ordinal": 0}]);
    let parameter_body = json!({"term": "aggregate", "members": [
        binding("name", word(parameter_name)),
        binding("level", json!({"term": "literal", "type": node_id(&family_key("a3a3")),
            "value_kind": "integer", "value": level})),
    ]});
    let parameter = format!("fixture/claim-terminal/parameter/{parameter_name}/{level}");
    let mut parameter_node = node("value", "parameter", parameter_body, &[]);
    parameter_node["node_id"] = node_id(&parameter);
    parameter_node["semantic_type"] = node_id(&reference_type);
    parameter_node["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
    let condition_body = reference(&parameter);
    let condition = format!("fixture/claim-terminal/condition/{parameter_name}/{level}");
    let condition_node = node(
        "expression",
        "reference",
        condition_body,
        std::slice::from_ref(&parameter),
    );
    let mut condition_node = condition_node;
    condition_node["node_id"] = node_id(&condition);
    let terminal_node = terminal(
        "when",
        reference(&model),
        json!({"term": "aggregate", "members": [reference(&parameter)]}),
        reference(&condition),
        &[model, parameter, condition],
    );
    if target_model == other_object_key() {
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(other_object_node());
    }
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend([
            reference_node,
            parameter_node,
            condition_node,
            terminal_node,
        ]);
    settle(&mut package);
    package
}

fn other_object_key() -> String {
    sha256_hex(&canonical(&json!({
        "version": "filament.structural-node/v1", "node_tag": "model",
        "semantic_form": "object_type", "semantic_type": null,
        "declaration": null, "recursion": null,
        "owner": model_owner("test/families", "ix://test/families/Other"),
        "body": empty(),
    })))
}

fn other_object_node() -> Value {
    let key = other_object_key();
    json!({
        "node_id": node_id(&key), "schema_version": "filament.checked-semantic-graph/v2",
        "node_tag": "model", "semantic_form": "object_type", "semantic_type": node_id(&key),
        "dependencies": [], "occurrences": [{"role": "generated", "ordinal": 0}],
        "owner": model_owner("test/families", "ix://test/families/Other"), "body": empty(),
    })
}

fn terminal_for_selected_model(model: &str) -> Value {
    let mut package = v2_all_families();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(other_object_node());
    let mut terminal_node = terminal(
        "any",
        reference(model),
        empty(),
        empty(),
        &[model.to_owned()],
    );
    terminal_node["node_id"] = node_id(&format!(
        "fixture/claim-terminal/temporal/terminal/any/{model}"
    ));
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(terminal_node);
    settle(&mut package);
    package
}

fn population_document() -> Value {
    let mut document = family_model_document();
    document["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind": {"module": "test/families", "name": "population"},
            "construct": {"meaning": "filament.meaning.model.population/v1"},
        }));
    document["populations"] = json!([{
        "identity": "ix://test/families/Accounts", "displayName": "Accounts",
        "kind": {"module": "test/families", "name": "population"},
        "members": ["ix://test/families/Account"], "extent": "closed",
        "origin": {"source": {"sourceIdentity": "ix://test/families/spec",
            "path": "models/account.md", "startLine": 1, "startColumn": 1}},
    }]);
    let mut other = document["types"][0].clone();
    other["identity"] = json!("ix://test/families/Other");
    other["displayName"] = json!("Other");
    other["fields"] = json!([]);
    other["operations"] = json!([]);
    other
        .as_object_mut()
        .expect("object type")
        .remove("relationships");
    document["types"].as_array_mut().expect("types").push(other);
    document
}

fn read_with_population(package: &Value) -> CheckedPackageV2ReadResult {
    read_with_population_limited(package, CheckedPackageReadLimits::bounded())
}

fn admitted_with_population(package: &Value) -> CheckedPackageV2 {
    match read_with_population(package) {
        CheckedPackageV2ReadResult::Admitted(value) => *value,
        other => panic!("expected population-backed admission, got {other:?}"),
    }
}

fn reread_written_with_population(package: &Value) -> CheckedPackageV2 {
    let written = encoded(package);
    assert_eq!(written, canonical(package));
    let evidence = evidence_with_document(package, "test/families", &population_document());
    match CheckedPackageV2::read(&written, CheckedPackageReadLimits::bounded(), &evidence) {
        CheckedPackageV2ReadResult::Admitted(value) => *value,
        other => panic!("expected population-backed written package to admit, got {other:?}"),
    }
}

fn read_with_population_limited(
    package: &Value,
    limits: CheckedPackageReadLimits,
) -> CheckedPackageV2ReadResult {
    let document = population_document();
    let evidence = evidence_with_document(package, "test/families", &document);
    CheckedPackageV2::read(&canonical(package), limits, &evidence)
}

fn second_claim_occurrence(package: &mut Value) {
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let last = nodes.last_mut().expect("claim node");
    let occurrences = last["occurrences"].as_array_mut().expect("occurrences");
    let ordinal = occurrences.len();
    occurrences.push(json!({"role": "claim", "ordinal": ordinal}));
    let first = package["source_map"]
        .as_array()
        .expect("source map")
        .last()
        .expect("entry")
        .clone();
    let mut second = first;
    second["ordinal"] = json!(ordinal);
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(second);
    refresh_identity(package);
}

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

fn admitted(package: &Value) -> CheckedPackageV2 {
    match read(package) {
        CheckedPackageV2ReadResult::Admitted(value) => *value,
        other => panic!("expected admission, got {other:?}"),
    }
}

fn reread_written(package: &Value) -> CheckedPackageV2 {
    let written = encoded(package);
    assert_eq!(written, canonical(package));
    match CheckedPackageV2::read(
        &written,
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    ) {
        CheckedPackageV2ReadResult::Admitted(value) => *value,
        other => panic!("expected written package to admit, got {other:?}"),
    }
}

fn refused(package: &Value, code: Code, cause: Option<Cause>, suffix: &str) {
    match read(package) {
        CheckedPackageV2ReadResult::Refused(value) => {
            assert_eq!(value.code, code, "{value:?}");
            assert_eq!(value.cause, cause);
            assert!(
                value.loci[0]
                    .path
                    .as_ref()
                    .is_some_and(|path| path.as_str().ends_with(suffix)),
                "{value:?}"
            );
        }
        other => panic!("expected refusal, got {other:?}"),
    }
}

fn typed<T: DeserializeOwned>(value: &Value) -> T {
    serde_json::from_value(value.clone()).expect("typed fixture")
}

fn encoded(package: &Value) -> Vec<u8> {
    let lock: CheckedPackageLockV2 = typed(&package["lock"]);
    let graph: CheckedSemanticGraphV2<Value> =
        typed(&crate::support::checked_package::wire_form(package)["semantic_graph"]);
    let source_map: Vec<CheckedSourceMapEntry> = typed(&package["source_map"]);
    let capabilities: Vec<CheckedCapability> = typed(&package["capability_report"]);
    let diagnostics: CheckedDiagnosticsV2 = typed(&package["diagnostics"]);
    let temporal_bindings: Option<Vec<filament_contract_model::TemporalBinding>> =
        package.get("temporal_bindings").map(typed);
    let temporal_clause_names: Option<Vec<filament_contract_model::TemporalClauseName>> =
        package.get("temporal_clause_names").map(typed);
    let claim_names: Option<Vec<filament_contract_model::ClaimNameRow>> =
        package.get("claim_names").map(typed);
    encode_checked_package_v2(
        package["name"].as_str().expect("package name"),
        &lock,
        &graph,
        filament_contract_model::CheckedPackageSourceCorrespondenceRef {
            source_map: &source_map,
            refinement_step_rows: None,
        },
        filament_contract_model::CheckedPackageSidecarsRef {
            temporal_clause_names: temporal_clause_names.as_deref(),
            claim_names: claim_names.as_deref(),
            operation_effects: None,
            operation_bodies: None,
            checked_term_data: None,
            temporal_bindings: temporal_bindings.as_deref(),
            schedulability_claims: None,
        },
        &capabilities,
        &diagnostics,
        quire_canonical::Limits::new(u64::MAX),
    )
    .expect("writer")
}

fn source_correspondence(package: &Value, node_position: usize, start: u64) -> Value {
    let mut changed = package.clone();
    let node_id = &package["semantic_graph"]["nodes"][node_position]["node_id"];
    let entry = changed["source_map"]
        .as_array_mut()
        .expect("source map")
        .iter_mut()
        .find(|entry| &entry["node_id"] == node_id)
        .expect("node source correspondence");
    entry["regions"][0]["start"] = json!(start);
    entry["regions"][0]["end"] = json!(start + 1);
    assert_ne!(changed["source_map"], package["source_map"]);
    crate::support::checked_package::recompute_declaration_regions(&mut changed);
    changed
}

#[trace("FR-059-AC-1", "FR-059-AC-5")]
#[test]
fn state_graph_forms_read_and_write() {
    let over = family_key("a4a4");
    let predicate = family_key("eeee");
    for (kind, from, deps) in [
        ("possible", empty(), vec![over.clone(), predicate.clone()]),
        (
            "always-possible",
            reference(&predicate),
            vec![over.clone(), predicate.clone()],
        ),
        (
            "unique-path",
            reference(&predicate),
            vec![over.clone(), predicate.clone()],
        ),
    ] {
        let graph_node = state_graph(kind, reference(&over), from, reference(&predicate), &deps);
        let package = package_with(graph_node.clone());
        let admitted = admitted(&package);
        assert_eq!(
            admitted
                .graph()
                .nodes
                .last()
                .expect("claim")
                .semantic_form
                .as_ref(),
            "state_graph"
        );
        let written = encoded(&package);
        assert_eq!(written, canonical(&package));
        let reread = match CheckedPackageV2::read(
            &written,
            CheckedPackageReadLimits::bounded(),
            &evidence_for(&package),
        ) {
            CheckedPackageV2ReadResult::Admitted(value) => value,
            other => panic!("writer output refused: {other:?}"),
        };
        assert_eq!(reread.graph(), admitted.graph());
        assert_eq!(reread.source_map(), admitted.source_map());
    }
}

#[trace("FR-060-AC-1", "FR-060-AC-5")]
#[test]
fn terminal_any_reads_and_writes() {
    let model = family_key("1515");
    let terminal_node = terminal("any", reference(&model), empty(), empty(), &[model]);
    let package = package_with(terminal_node);
    let admitted = admitted(&package);
    assert_eq!(
        admitted
            .graph()
            .nodes
            .last()
            .expect("terminal")
            .semantic_form
            .as_ref(),
        "terminal"
    );
    let written = encoded(&package);
    assert_eq!(written, canonical(&package));
    let reread = match CheckedPackageV2::read(
        &written,
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&package),
    ) {
        CheckedPackageV2ReadResult::Admitted(value) => value,
        other => panic!("writer output refused: {other:?}"),
    };
    assert_eq!(reread.graph(), admitted.graph());
    assert_eq!(reread.source_map(), admitted.source_map());
}

/// Two selected models `M` and `N` (and an unused `K`) with MT-01 on `M` and
/// MT-02 on `N`; the graph holds `N`'s terminal node before `M`'s.
fn two_model_package(model_n: &str, model_k: &str) -> (Value, Value, Value) {
    let model_m = family_key("1515");
    let model_n_id = json!({"package": model_n, "path": ["model", "model_root"], "ordinal": 0});
    let mut package = terminal_with_self(&model_m, "self", "0");
    select_model(&mut package, model_n);
    select_model(&mut package, model_k);
    refresh_identity(&mut package);
    let terminal_m = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| {
            node["node_tag"] == "temporal"
                && node["semantic_form"] == "terminal"
                && node["body"]["members"][1]["value"]["value"] == "when"
        })
        .expect("M terminal")
        .clone();
    let mut terminal_n = terminal("any", empty(), empty(), empty(), &[]);
    terminal_n["body"]["members"][0]["value"] = json!({"term": "reference", "target": model_n_id});
    terminal_n["dependencies"] = json!([model_n_id]);
    terminal_n["node_id"] = node_id("zzzz-terminal-N");
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(terminal_n.clone());
    settle(&mut package);
    let graph = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let n_position = graph
        .iter()
        .position(|node| node["node_id"] == terminal_n["node_id"])
        .expect("N terminal position");
    let m_position = graph
        .iter()
        .position(|node| node["node_id"] == terminal_m["node_id"])
        .expect("M terminal position");
    graph.swap(n_position, m_position);
    (package, terminal_m, terminal_n)
}

/// Trace: FR-060-AC-6
#[trace("TC-487", "FR-060-AC-6")]
#[test]
fn terminal_rows_retain_structural_order_and_round_trip() {
    let model_m = family_key("1515");
    let model_n = "zz/second-selected-model";
    let model_k = "zz/third-selected-model";
    let model_n_id = json!({"package": model_n, "path": ["model", "model_root"], "ordinal": 0});
    let mut no_terminals = v2_all_families();
    select_model(&mut no_terminals, model_n);
    select_model(&mut no_terminals, model_k);
    refresh_identity(&mut no_terminals);
    assert!(admitted(&no_terminals).terminal_rows().is_empty());
    let mut single = terminal_with_self(&model_m, "self", "0");
    select_model(&mut single, model_n);
    select_model(&mut single, model_k);
    refresh_identity(&mut single);
    assert_eq!(admitted_with_population(&single).terminal_rows().len(), 1);
    let (package, terminal_m, terminal_n) = two_model_package(model_n, model_k);
    let parameter_id = terminal_m["body"]["members"][2]["value"]["members"][0]["target"].clone();
    let condition_id = terminal_m["body"]["members"][3]["value"]["target"].clone();

    let m_node: CheckedNodeId =
        crate::support::checked_package::structural_fixture_id(&terminal_m["node_id"]);
    let n_node: CheckedNodeId =
        crate::support::checked_package::structural_fixture_id(&terminal_n["node_id"]);
    assert!(m_node < n_node, "the fixture keys put M before N");
    assert!(
        crate::support::checked_package::structural_fixture_id(&node_id(&model_m))
            < crate::support::checked_package::structural_fixture_id(&model_n_id),
        "M's model key sorts before N's"
    );
    let graph = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let n_position = graph
        .iter()
        .position(|node| node["node_id"] == terminal_n["node_id"])
        .expect("N terminal position");
    let m_position = graph
        .iter()
        .position(|node| node["node_id"] == terminal_m["node_id"])
        .expect("M terminal position");
    assert!(
        n_position < m_position,
        "wire order is opposite to key order"
    );

    let admitted_package = admitted_with_population(&package);
    let model_m_key = crate::support::checked_package::structural_fixture_id(&node_id(&model_m));
    let model_n_key = crate::support::checked_package::structural_fixture_id(&model_n_id);
    let model_k_key = crate::support::checked_package::structural_fixture_id(&json!({
        "package": model_k,
        "path": ["model", "model_root"],
        "ordinal": 0
    }));
    let parameter_key = crate::support::checked_package::structural_fixture_id(&parameter_id);
    let condition_key = crate::support::checked_package::structural_fixture_id(&condition_id);
    let expected = [
        TerminalRow {
            node: m_node,
            model: model_m_key,
            ordinal: 0,
            declaration: TerminalDeclaration::When {
                parameter: Some(parameter_key),
                condition: condition_key,
            },
        },
        TerminalRow {
            node: n_node,
            model: model_n_key,
            ordinal: 0,
            declaration: TerminalDeclaration::Any,
        },
    ];
    assert_eq!(admitted_package.terminal_rows(), expected);
    assert!(admitted_package
        .terminal_rows()
        .iter()
        .all(|row| row.model != model_k_key));

    let reread = reread_written_with_population(&package);
    assert_eq!(reread.terminal_rows(), expected);

    let mut key_ordered = package.clone();
    let last_terminal = key_ordered["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == terminal_m["node_id"])
        .expect("M terminal position");
    let first_terminal = key_ordered["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == terminal_n["node_id"])
        .expect("N terminal position");
    key_ordered["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .swap(first_terminal, last_terminal);
    settle(&mut key_ordered);
    assert_eq!(
        admitted_with_population(&key_ordered).terminal_rows(),
        expected
    );

    assert!(admitted(&v2_all_families()).terminal_rows().is_empty());

    let condition = family_key("eeee");
    let mut no_self = v2_all_families();
    let mut terminal_no_self = terminal(
        "when",
        reference(&model_m),
        empty(),
        reference(&condition),
        &[model_m.clone(), condition.clone()],
    );
    terminal_no_self["node_id"] = node_id("fixture/claim-terminal/rows/MT-03");
    no_self["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(terminal_no_self);
    settle(&mut no_self);
    let no_self_package = admitted_with_population(&no_self);
    let no_self_rows = no_self_package.terminal_rows();
    assert_eq!(no_self_rows.len(), 1);
    assert_eq!(
        no_self_rows[0].declaration,
        TerminalDeclaration::When {
            parameter: None,
            condition: crate::support::checked_package::structural_fixture_id(&node_id(&condition)),
        }
    );

    let mut multiply_populated_document = population_document();
    let mut second_population = multiply_populated_document["populations"][0].clone();
    second_population["identity"] = json!("ix://test/families/AccountsAgain");
    multiply_populated_document["populations"]
        .as_array_mut()
        .expect("populations")
        .push(second_population);
    let evidence = evidence_with_document(&package, "test/families", &multiply_populated_document);
    let ambiguous_self = CheckedPackageV2::read(
        &canonical(&package),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    );
    match ambiguous_self {
        CheckedPackageV2ReadResult::Refused(value) => {
            assert_eq!(value.code, Code::IllTyped);
            assert_eq!(value.cause, Some(Cause::OperatorIneligible));
            assert!(value.loci[0].path.as_ref().is_some_and(|path| path
                .as_str()
                .ends_with("/body/members/2/value/members/0/target")));
        }
        other => panic!("expected M's multiply populated self to refuse, got {other:?}"),
    }

    // A second terminal node on M (MT-12) is admitted: M has two rows.
    let mut second_terminal = terminal_m;
    second_terminal["node_id"] = node_id("fixture/claim-terminal/rows/M-second");
    let mut second = package;
    second["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(second_terminal.clone());
    settle(&mut second);
    let rows = admitted_with_population(&second);
    let on_m: Vec<_> = rows
        .terminal_rows()
        .iter()
        .filter(|row| row.model == model_m_key_for_test(&model_m))
        .collect();
    assert_eq!(on_m.len(), 2);
    assert_eq!(on_m[0].ordinal, 0);
    assert_eq!(on_m[1].ordinal, 1);
    assert!(on_m[0].node < on_m[1].node);
}

fn model_m_key_for_test(model_m: &str) -> CheckedNodeId {
    crate::support::checked_package::structural_fixture_id(&node_id(model_m))
}

/// Rows order by the model node's key first, then the terminal node's: here
/// the terminal key order disagrees with the model key order.
#[trace("TC-487", "FR-060-AC-8")]
#[test]
fn terminal_rows_order_by_model_key_before_terminal_key() {
    let model_m = family_key("1515");
    let model_n = "aa/before-selected-model";
    let (package, terminal_m, terminal_n) = two_model_package(model_n, "zz/third-selected-model");
    let m_node: CheckedNodeId =
        crate::support::checked_package::structural_fixture_id(&terminal_m["node_id"]);
    let n_node: CheckedNodeId =
        crate::support::checked_package::structural_fixture_id(&terminal_n["node_id"]);
    let model_n_key = crate::support::checked_package::structural_fixture_id(
        &json!({"package": model_n, "path": ["model", "model_root"], "ordinal": 0}),
    );
    let model_m_key = model_m_key_for_test(&model_m);
    assert!(m_node < n_node && model_n_key < model_m_key);
    let rows = admitted_with_population(&package);
    let order: Vec<_> = rows
        .terminal_rows()
        .iter()
        .map(|row| (row.node.clone(), row.model.clone(), row.ordinal))
        .collect();
    assert_eq!(
        order,
        vec![(n_node, model_n_key, 0), (m_node, model_m_key, 0)]
    );
}

#[trace("FR-059-AC-2", "FR-059-AC-5")]
#[test]
fn state_graph_refuses_kind_fit_and_target_meaning() {
    let over = family_key("a4a4");
    let predicate = family_key("eeee");
    let baseline = || (reference(&over), empty(), reference(&predicate));
    for (case, kind, over_term, from_term, target_term, deps, code, cause, suffix) in [
        {
            let (over_term, from_term, target_term) = baseline();
            (
                "SN-09",
                "eventually-possible",
                over_term,
                from_term,
                target_term,
                vec![over.clone(), predicate.clone()],
                Code::InvalidPackage,
                Some(Cause::InvalidValue),
                "/body/members/0",
            )
        },
        (
            "SN-10",
            "possible",
            reference(&over),
            reference(&predicate),
            reference(&predicate),
            vec![over.clone(), predicate.clone()],
            Code::InvalidSemanticGraph,
            None,
            "/body/members/2",
        ),
        (
            "SN-11",
            "possible",
            reference(&over),
            empty(),
            reference(&over),
            vec![over.clone()],
            Code::IllTyped,
            Some(Cause::OperatorIneligible),
            "/body/members/3/value/target",
        ),
        (
            "SN-14",
            "possible",
            word("over"),
            empty(),
            reference(&predicate),
            vec![predicate.clone()],
            Code::InvalidSemanticGraph,
            None,
            "/body/members/1",
        ),
    ] {
        let package = package_with(state_graph(kind, over_term, from_term, target_term, &deps));
        match read(&package) {
            CheckedPackageV2ReadResult::Refused(value) => {
                assert_eq!(value.code, code, "{case}");
                assert_eq!(value.cause, cause, "{case}");
                assert!(
                    value.loci[0]
                        .path
                        .as_ref()
                        .is_some_and(|path| path.as_str().ends_with(suffix)),
                    "{case}: {value:?}"
                );
            }
            other => panic!("{case}: expected refusal, got {other:?}"),
        }
    }
    let mut missing = package_with(state_graph(
        "possible",
        reference(&over),
        empty(),
        reference(&predicate),
        std::slice::from_ref(&over),
    ));
    // The body's target remains present, while the written dependency is absent.
    settle(&mut missing);
    refused(&missing, Code::InvalidSemanticGraph, None, "/dependencies");
    let mut reordered = state_graph(
        "always-possible",
        reference(&over),
        reference(&predicate),
        reference(&predicate),
        &[over.clone(), predicate.clone()],
    );
    reordered["body"]["members"]
        .as_array_mut()
        .expect("members")
        .swap(2, 3);
    refused(
        &package_with(reordered),
        Code::InvalidSemanticGraph,
        None,
        "/body",
    );
    let absent_from = package_with(state_graph(
        "unique-path",
        reference(&over),
        empty(),
        reference(&predicate),
        &[over.clone(), predicate.clone()],
    ));
    refused(
        &absent_from,
        Code::InvalidSemanticGraph,
        None,
        "/body/members/2",
    );
    let temporal_target = v2_all_families()["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "temporal" && node["semantic_form"] == "temporal_clause")
        .expect("temporal clause")["node_id"]["path"][0]
        .as_str()
        .expect("digest")
        .to_owned();
    let wrong_predicate = package_with(state_graph(
        "possible",
        reference(&over),
        empty(),
        reference(&temporal_target),
        &[over, temporal_target],
    ));
    refused(
        &wrong_predicate,
        Code::IllTyped,
        Some(Cause::OperatorIneligible),
        "/body/members/3/value/target",
    );
}

#[trace("FR-060-AC-1", "FR-060-AC-2", "FR-060-AC-5")]
#[test]
fn terminal_when_and_mode_refusals() {
    let model = family_key("1515");
    let predicate = family_key("eeee");
    let when = terminal(
        "when",
        reference(&model),
        empty(),
        reference(&predicate),
        &[model.clone(), predicate.clone()],
    );
    let package = package_with(when);
    let _ = admitted(&package);
    let bad_mode = package_with(terminal(
        "some",
        reference(&model),
        empty(),
        reference(&predicate),
        &[model.clone(), predicate.clone()],
    ));
    refused(
        &bad_mode,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/body/members/1",
    );
    let bad_fit = package_with(terminal(
        "any",
        reference(&model),
        empty(),
        reference(&predicate),
        &[model.clone(), predicate.clone()],
    ));
    refused(
        &bad_fit,
        Code::InvalidSemanticGraph,
        None,
        "/body/members/3",
    );
    let wrong_model = family_key("a4a4");
    let bad_model = package_with(terminal(
        "when",
        reference(&wrong_model),
        empty(),
        reference(&predicate),
        &[wrong_model, predicate],
    ));
    refused(
        &bad_model,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        "/body/members/0/value/target",
    );
    let mut reordered = terminal(
        "when",
        reference(&model),
        empty(),
        reference(&family_key("eeee")),
        &[model.clone(), family_key("eeee")],
    );
    reordered["body"]["members"]
        .as_array_mut()
        .expect("members")
        .swap(1, 3);
    refused(
        &package_with(reordered),
        Code::InvalidSemanticGraph,
        None,
        "/body",
    );
    let missing_condition = package_with(terminal(
        "when",
        reference(&model),
        empty(),
        empty(),
        std::slice::from_ref(&model),
    ));
    refused(
        &missing_condition,
        Code::InvalidSemanticGraph,
        None,
        "/body/members/3",
    );
    let extra_parameters = json!({"term": "aggregate", "members": [
        reference(&family_key("a4a4")), reference(&family_key("a4a4"))]});
    let two_parameters = package_with(terminal(
        "when",
        reference(&model),
        extra_parameters,
        reference(&family_key("eeee")),
        &[model.clone(), family_key("a4a4"), family_key("eeee")],
    ));
    refused(
        &two_parameters,
        Code::InvalidSemanticGraph,
        None,
        "/body/members/2",
    );
    let both = package_with(terminal(
        "any",
        reference(&model),
        json!({"term": "aggregate", "members": [reference(&family_key("a4a4"))]}),
        reference(&family_key("eeee")),
        &[model, family_key("a4a4"), family_key("eeee")],
    ));
    refused(&both, Code::InvalidSemanticGraph, None, "/body/members/2");
    let integer = family_key("a3a3");
    let integer_body = reference(&integer);
    let integer_condition = "fixture/claim-terminal/integer-condition".to_owned();
    let mut expression = node(
        "expression",
        "reference",
        integer_body,
        std::slice::from_ref(&integer),
    );
    expression["node_id"] = node_id(&integer_condition);
    expression["semantic_type"] = node_id(&integer);
    let mut wrong_condition = package_with(terminal(
        "when",
        reference(&family_key("1515")),
        empty(),
        reference(&integer_condition),
        &[family_key("1515"), integer_condition],
    ));
    wrong_condition["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .insert(base_node_index(), expression);
    settle(&mut wrong_condition);
    refused(
        &wrong_condition,
        Code::IllTyped,
        Some(Cause::NonBooleanRoot),
        "/body/members/3/value/target",
    );
}

#[trace("FR-059-AC-1", "FR-059-AC-4", "FR-059-AC-5")]
#[test]
fn state_graph_identity_and_shared_occurrences() {
    let over = family_key("a4a4");
    let predicate = family_key("eeee");
    let deps = [over.clone(), predicate.clone()];
    let possible = package_with(state_graph(
        "possible",
        reference(&over),
        empty(),
        reference(&predicate),
        &deps,
    ));
    let always = package_with(state_graph(
        "always-possible",
        reference(&over),
        reference(&predicate),
        reference(&predicate),
        &deps,
    ));
    assert_ne!(
        possible["semantic_graph"]["nodes"][base_node_index()]["node_id"],
        always["semantic_graph"]["nodes"][base_node_index()]["node_id"]
    );
    assert_ne!(encoded(&possible), encoded(&always));
    let moved = source_correspondence(&possible, base_node_index(), 100);
    assert_eq!(
        possible["semantic_graph"]["nodes"][base_node_index()],
        moved["semantic_graph"]["nodes"][base_node_index()]
    );
    assert_eq!(possible["name"], moved["name"]);
    assert_ne!(possible["source_map"], moved["source_map"]);
    let moved_read = admitted(&moved);
    let moved_reread = reread_written(&moved);
    let expected_source: Vec<CheckedSourceMapEntry> = typed(&moved["source_map"]);
    assert_eq!(moved_read.source_map(), expected_source);
    assert_eq!(moved_reread.source_map(), moved_read.source_map());
    assert_eq!(
        moved_reread.graph().nodes[base_node_index()],
        moved_read.graph().nodes[base_node_index()]
    );
    let mut instance_a = possible.clone();
    instance_a["source_map"][base_node_index()]["regions"][0]["start"] = json!(100);
    instance_a["source_map"][base_node_index()]["regions"][0]["end"] = json!(101);
    crate::support::checked_package::recompute_declaration_regions(&mut instance_a);
    let mut instance_b = possible.clone();
    instance_b["source_map"][base_node_index()]["regions"][0]["start"] = json!(200);
    instance_b["source_map"][base_node_index()]["regions"][0]["end"] = json!(201);
    crate::support::checked_package::recompute_declaration_regions(&mut instance_b);
    for instance in [&instance_a, &instance_b] {
        let read = admitted(instance);
        let written = reread_written(instance);
        assert_eq!(
            read.document_held_id(base_node_index())
                .expect("graph node")
                .clone(),
            admitted(&possible).graph().nodes[base_node_index()].node_id
        );
        assert_eq!(read.name(), admitted(&possible).name());
        assert_eq!(written.source_map(), read.source_map());
        assert_eq!(
            written.graph().nodes[base_node_index()],
            read.graph().nodes[base_node_index()]
        );
    }
    assert_ne!(instance_a["source_map"], instance_b["source_map"]);
    assert_eq!(
        instance_a["semantic_graph"]["nodes"][base_node_index()],
        instance_b["semantic_graph"]["nodes"][base_node_index()]
    );
    let mut shared = possible.clone();
    second_claim_occurrence(&mut shared);
    let admitted = admitted(&shared);
    let claim = admitted.graph().nodes.last().expect("claim");
    assert_eq!(claim.occurrences.len(), 2);
    assert_eq!(
        admitted
            .source_map()
            .iter()
            .filter(|entry| entry.node_id == claim.node_id)
            .count(),
        2
    );
    assert_eq!(possible["name"], shared["name"]);
    assert!(matches!(
        CheckedPackageV2::read(
            &encoded(&shared),
            CheckedPackageReadLimits::bounded(),
            &evidence_for(&shared)
        ),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

#[trace("FR-059-AC-1", "FR-059-AC-5")]
#[test]
fn state_graph_constant_predicate_and_absent_over() {
    let boolean = family_key("aaaa");
    let truth_body = json!({"term": "literal", "type": node_id(&boolean),
        "value_kind": "boolean", "value": true});
    let truth = "fixture/claim-terminal/truth".to_owned();
    let mut truth_node = node("value", "literal", truth_body, &[]);
    truth_node["node_id"] = node_id(&truth);
    truth_node["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
    let possible = package_with_nodes(vec![
        truth_node.clone(),
        state_graph(
            "possible",
            empty(),
            empty(),
            reference(&truth),
            std::slice::from_ref(&truth),
        ),
    ]);
    assert_eq!(
        admitted(&possible)
            .graph()
            .nodes
            .last()
            .expect("claim")
            .dependencies
            .len(),
        1
    );
    let predicate = family_key("eeee");
    let always = package_with_nodes(vec![
        truth_node,
        state_graph(
            "always-possible",
            empty(),
            reference(&truth),
            reference(&predicate),
            &[truth, predicate],
        ),
    ]);
    let _ = admitted(&always);
}

#[trace("FR-060-AC-2", "FR-060-AC-4", "FR-060-AC-5", "FR-060-AC-8")]
#[test]
fn terminal_identity_and_several_terminals_per_model() {
    let model = family_key("1515");
    let predicate = family_key("eeee");
    let any = package_with(terminal(
        "any",
        reference(&model),
        empty(),
        empty(),
        std::slice::from_ref(&model),
    ));
    let when = package_with(terminal(
        "when",
        reference(&model),
        empty(),
        reference(&predicate),
        &[model.clone(), predicate.clone()],
    ));
    assert_ne!(
        any["semantic_graph"]["nodes"][base_node_index()]["node_id"],
        when["semantic_graph"]["nodes"][base_node_index()]["node_id"],
        "forms: {:?} {:?}",
        any["semantic_graph"]["nodes"][base_node_index()]["semantic_form"],
        when["semantic_graph"]["nodes"][base_node_index()]["semantic_form"]
    );
    assert_ne!(encoded(&any), encoded(&when));
    let done = source_correspondence(&any, base_node_index(), 100);
    let stop = source_correspondence(&any, base_node_index(), 200);
    assert_eq!(
        done["semantic_graph"]["nodes"][base_node_index()],
        stop["semantic_graph"]["nodes"][base_node_index()]
    );
    assert_eq!(done["name"], stop["name"]);
    assert_ne!(done["source_map"], stop["source_map"]);
    for named in [&done, &stop] {
        let read = admitted(named);
        let written = reread_written(named);
        assert_eq!(
            read.document_held_id(base_node_index())
                .expect("graph node")
                .clone(),
            admitted(&any).graph().nodes[base_node_index()].node_id
        );
        assert_eq!(read.name(), admitted(&any).name());
        assert_eq!(written.source_map(), read.source_map());
        assert_eq!(
            written.graph().nodes[base_node_index()],
            read.graph().nodes[base_node_index()]
        );
    }
    assert_ne!(encoded(&any), encoded(&v2_all_families()));
    let changed_body = reference(&predicate);
    let changed_condition = "fixture/claim-terminal/changed-condition".to_owned();
    let mut changed_node = node(
        "expression",
        "reference",
        changed_body,
        std::slice::from_ref(&predicate),
    );
    changed_node["node_id"] = node_id(&changed_condition);
    changed_node["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
    let mut changed_terminal = terminal(
        "when",
        reference(&model),
        empty(),
        reference(&changed_condition),
        &[model.clone(), changed_condition],
    );
    changed_terminal["node_id"] =
        node_id("fixture/claim-terminal/temporal/terminal/when/changed-condition");
    let changed = package_with_nodes(vec![changed_node, changed_terminal]);
    let _ = admitted(&changed);
    assert_ne!(
        when["semantic_graph"]["nodes"][base_node_index()]["node_id"],
        changed["semantic_graph"]["nodes"][base_node_index() + 1]["node_id"]
    );
    assert_ne!(encoded(&when), encoded(&changed));
    let mut both = any.clone();
    both["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(when["semantic_graph"]["nodes"][base_node_index()].clone());
    settle(&mut both);
    let any_id: CheckedNodeId =
        typed(&any["semantic_graph"]["nodes"][base_node_index()]["node_id"]);
    let when_id: CheckedNodeId =
        typed(&when["semantic_graph"]["nodes"][base_node_index()]["node_id"]);
    let expected_order = if any_id < when_id {
        [any_id.clone(), when_id.clone()]
    } else {
        [when_id.clone(), any_id.clone()]
    };
    let rows = admitted(&both);
    let model_key = crate::support::checked_package::structural_fixture_id(&node_id(&model));
    assert_eq!(rows.terminal_rows().len(), 2);
    for (at, row) in rows.terminal_rows().iter().enumerate() {
        assert_eq!(row.node, expected_order[at]);
        assert_eq!(row.model, model_key);
        assert_eq!(usize::try_from(row.ordinal).expect("ordinal"), at);
    }
    let mut reversed = both.clone();
    reversed["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .swap(base_node_index(), base_node_index() + 1);
    settle(&mut reversed);
    assert_eq!(admitted(&reversed).terminal_rows(), rows.terminal_rows());
    assert_eq!(reread_written(&both).terminal_rows(), rows.terminal_rows());
    let mut bad_target = both.clone();
    let terminal = &mut bad_target["semantic_graph"]["nodes"][base_node_index() + 1];
    terminal["body"]["members"][3]["value"] = reference(&model);
    terminal["dependencies"] = json!([node_id(&model)]);
    settle(&mut bad_target);
    refused(
        &bad_target,
        Code::IllTyped,
        Some(Cause::OperatorIneligible),
        "/body/members/3/value/target",
    );
    // MT-16: one node with two, then three, `claim` occurrences is one row.
    let mut repeated = any;
    second_claim_occurrence(&mut repeated);
    let one_row = |package: &Value| {
        let read = admitted(package);
        let rows = read.terminal_rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].ordinal, 0);
        assert_eq!(rows[0].declaration, TerminalDeclaration::Any);
        let written = reread_written(package);
        assert_eq!(written.terminal_rows(), rows);
        assert_eq!(written.source_map(), read.source_map());
        read.graph().nodes[base_node_index()].occurrences.clone()
    };
    assert_eq!(
        one_row(&repeated),
        [0, 1].map(|ordinal| CheckedOccurrence {
            role: filament_contract_model::CheckedOccurrenceRole::Claim,
            ordinal
        })
    );
    second_claim_occurrence(&mut repeated);
    assert_eq!(one_row(&repeated).len(), 3);
    let first_model = terminal_for_selected_model(&model);
    let second_model = terminal_for_selected_model(&other_object_key());
    assert!(matches!(
        read_with_population(&first_model),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    assert!(matches!(
        read_with_population(&second_model),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    assert_ne!(
        first_model["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["semantic_form"] == "terminal")
            .expect("first terminal")["node_id"],
        second_model["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["semantic_form"] == "terminal")
            .expect("second terminal")["node_id"]
    );
    assert_ne!(encoded(&first_model), encoded(&second_model));
}

#[trace("FR-060-AC-1", "FR-060-AC-2", "FR-060-AC-5")]
#[test]
fn terminal_self_parameter_must_have_right_name_level_and_type() {
    let model = family_key("1515");
    let valid = terminal_with_self(&model, "self", "0");
    assert!(matches!(
        read_with_population(&valid),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let mut omitted = valid.clone();
    let terminal = omitted["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .iter_mut()
        .find(|node| node["semantic_form"] == "terminal")
        .expect("terminal");
    let parameter_id = terminal["body"]["members"][2]["value"]["members"][0]["target"].clone();
    terminal["body"]["members"][2]["value"] = empty();
    terminal["dependencies"]
        .as_array_mut()
        .expect("dependencies")
        .retain(|id| *id != parameter_id);
    settle(&mut omitted);
    match read_with_population(&omitted) {
        CheckedPackageV2ReadResult::Refused(value) => {
            assert_eq!(value.code, Code::InvalidSemanticGraph);
            assert!(
                value.loci[0]
                    .path
                    .as_ref()
                    .is_some_and(|path| path.as_str().ends_with("/body/members/2")),
                "{value:?}"
            );
        }
        other => panic!("expected omitted self refusal, got {other:?}"),
    }
    for (name, level) in [("result", "1"), ("self", "1")] {
        let invalid = terminal_with_self(&model, name, level);
        match read_with_population(&invalid) {
            CheckedPackageV2ReadResult::Refused(value) => {
                assert_eq!(value.code, Code::IllTyped);
                assert_eq!(value.cause, Some(Cause::OperatorIneligible));
                assert!(
                    value.loci[0].path.as_ref().is_some_and(|path| path
                        .as_str()
                        .ends_with("/body/members/2/value/members/0/target")),
                    "{value:?}"
                );
            }
            other => panic!("expected self refusal, got {other:?}"),
        }
    }
    let other_model = other_object_key();
    let mistyped = terminal_with_self(&other_model, "self", "0");
    match read_with_population(&mistyped) {
        CheckedPackageV2ReadResult::Refused(value) => {
            assert_eq!(value.code, Code::IllTyped);
            assert_eq!(value.cause, Some(Cause::OperatorIneligible));
        }
        other => panic!("expected mistyped self refusal, got {other:?}"),
    }
}

/// A `when { self... }` terminal whose `model` binds the selection's
/// `model_root` node, as FR-476 "Several terminal nodes on one model node"
/// fixes, with `self` typed `Reference` to the selection's one object type.
fn model_root_terminal_with_self() -> Value {
    model_root_terminal_with_self_typed(&family_key("1515"))
}

fn model_root_terminal_with_self_typed(self_object: &str) -> Value {
    let object = family_key("1515");
    let mut package = terminal_with_self(self_object, "self", "0");
    select_model(&mut package, "test/families");
    let root = json!({"package": "test/families", "path": ["model", "model_root"], "ordinal": 0});
    let terminal_node = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .iter_mut()
        .find(|node| node["semantic_form"] == "terminal")
        .expect("terminal");
    terminal_node["body"]["members"][0]["value"] = json!({"term": "reference", "target": root});
    let mut dependencies = terminal_node["dependencies"]
        .as_array()
        .expect("dependencies")
        .iter()
        .filter(|id| **id != node_id(&object))
        .cloned()
        .collect::<Vec<_>>();
    dependencies.push(root);
    dependencies
        .sort_by_key(|id| serde_json::from_value::<CheckedNodeId>(id.clone()).expect("node ID"));
    terminal_node["dependencies"] = json!(dependencies);
    settle(&mut package);
    package
}

fn read_with_document(package: &Value, document: &Value) -> CheckedPackageV2ReadResult {
    let evidence = evidence_with_document(package, "test/families", document);
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    )
}

fn assert_self_refused(result: CheckedPackageV2ReadResult) {
    match result {
        CheckedPackageV2ReadResult::Refused(value) => {
            assert_eq!(value.code, Code::IllTyped, "{value:?}");
            assert_eq!(value.cause, Some(Cause::OperatorIneligible), "{value:?}");
            assert!(
                value.loci[0].path.as_ref().is_some_and(|path| path
                    .as_str()
                    .ends_with("/body/members/2/value/members/0/target")),
                "{value:?}"
            );
        }
        other => panic!("expected self refusal, got {other:?}"),
    }
}

/// FR-476 "Several terminal nodes on one model node": a terminal bound to the
/// model root reads `self` from the selection's one population object.
#[trace("TC-961", "FR-060-AC-8")]
#[test]
fn model_root_terminal_with_self_takes_the_one_population_object() {
    let package = model_root_terminal_with_self();
    let admitted = admitted_with_population(&package);
    let rows = admitted.terminal_rows();
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].model,
        typed::<CheckedNodeId>(
            &json!({"package": "test/families", "path": ["model", "model_root"], "ordinal": 0})
        )
    );
    assert!(matches!(
        rows[0].declaration,
        TerminalDeclaration::When {
            parameter: Some(_),
            ..
        }
    ));
    // A `self` typed for another object type is still refused.
    assert_self_refused(read_with_population(&model_root_terminal_with_self_typed(
        &other_object_key(),
    )));
}

/// FR-366: one object covered by two populations leaves `self` ambiguous, so
/// the model-root arm counts (population, object) pairs, not distinct objects.
#[trace("TC-961", "FR-060-AC-2")]
#[test]
fn model_root_terminal_with_self_refuses_one_object_in_two_populations() {
    let package = model_root_terminal_with_self();
    let mut document = population_document();
    let mut again = document["populations"][0].clone();
    again["identity"] = json!("ix://test/families/AccountsAgain");
    again["displayName"] = json!("AccountsAgain");
    document["populations"]
        .as_array_mut()
        .expect("populations")
        .push(again);
    assert_self_refused(read_with_document(&package, &document));
}

/// FSpec TC-961 MT-18 / FR-476-AC-6: a `self`-reading `when` node and an `any`
/// node on one `model_root` are both admitted, as ordinals 0 and 1.
#[trace("TC-961", "FR-060-AC-8")]
#[test]
fn model_root_terminals_when_self_and_any_share_one_root() {
    let mut package = model_root_terminal_with_self();
    let root = json!({"package": "test/families", "path": ["model", "model_root"], "ordinal": 0});
    let mut any = terminal("any", empty(), empty(), empty(), &[]);
    any["body"]["members"][0]["value"] = json!({"term": "reference", "target": root});
    any["dependencies"] = json!([root]);
    any["node_id"] = node_id("fixture/claim-terminal/temporal/terminal/any/model-root");
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(any);
    settle(&mut package);
    let admitted = admitted_with_population(&package);
    let rows = admitted.terminal_rows();
    assert_eq!(rows.len(), 2);
    let model = typed::<CheckedNodeId>(&root);
    assert!(rows.iter().all(|row| row.model == model));
    assert_eq!(
        rows.iter().map(|row| row.ordinal).collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(
        rows.iter()
            .filter(|row| matches!(
                row.declaration,
                TerminalDeclaration::When {
                    parameter: Some(_),
                    ..
                }
            ))
            .count(),
        1
    );
    assert_eq!(
        rows.iter()
            .filter(|row| matches!(row.declaration, TerminalDeclaration::Any))
            .count(),
        1
    );
}

/// FR-476: a model root with no population object gives `self` no object.
#[trace("TC-961", "FR-060-AC-2")]
#[test]
fn model_root_terminal_with_self_refuses_without_a_population_object() {
    let package = model_root_terminal_with_self();
    let mut document = population_document();
    document["populations"] = json!([]);
    assert_self_refused(read_with_document(&package, &document));
    document
        .as_object_mut()
        .expect("document")
        .remove("populations");
    assert_self_refused(read_with_document(&package, &document));
}

/// FR-476: a model root with several population objects leaves `self`
/// ambiguous.
#[trace("TC-961", "FR-060-AC-2")]
#[test]
fn model_root_terminal_with_self_refuses_several_population_objects() {
    let package = model_root_terminal_with_self();
    let mut document = population_document();
    document["populations"][0]["members"] =
        json!(["ix://test/families/Account", "ix://test/families/Other"]);
    assert_self_refused(read_with_document(&package, &document));
    // Refused whichever of the several objects `self` is typed for.
    let for_other = model_root_terminal_with_self_typed(&other_object_key());
    assert_self_refused(read_with_document(&for_other, &document));
    let mut second_population = population_document();
    let mut second = second_population["populations"][0].clone();
    second["identity"] = json!("ix://test/families/Others");
    second["displayName"] = json!("Others");
    second["members"] = json!(["ix://test/families/Other"]);
    second_population["populations"]
        .as_array_mut()
        .expect("populations")
        .push(second);
    assert_self_refused(read_with_document(&package, &second_population));
    assert_self_refused(read_with_document(&for_other, &second_population));
}

#[trace("FR-060-AC-1", "FR-038-AC-121")]
#[test]
fn terminal_self_population_scan_obeys_work_limit() {
    let package = terminal_with_self(&family_key("1515"), "self", "0");
    let read_at = |work| {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = work;
        read_with_population_limited(&package, limits)
    };
    let (mut lower, mut upper) = (0, CheckedPackageReadLimits::bounded().work);
    assert!(matches!(
        read_at(lower),
        CheckedPackageV2ReadResult::Incomplete(_)
    ));
    assert!(matches!(
        read_at(upper),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    while lower + 1 < upper {
        let middle = lower + (upper - lower) / 2;
        if matches!(read_at(middle), CheckedPackageV2ReadResult::Incomplete(_)) {
            lower = middle;
        } else {
            upper = middle;
        }
    }
    assert!(matches!(
        read_at(upper),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let mut charged_population = false;
    for work in (upper.saturating_sub(2_048)..upper).rev() {
        let CheckedPackageV2ReadResult::Incomplete(incomplete) = read_at(work) else {
            panic!("work below exact decision boundary must be incomplete");
        };
        assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
        if incomplete
            .path
            .as_ref()
            .is_some_and(|path| path.as_str().ends_with("/body/members/2"))
        {
            assert_eq!(incomplete.consumed, work + 1);
            charged_population = true;
            break;
        }
    }
    assert!(
        charged_population,
        "self population traversal must have a charged boundary"
    );
}

#[trace("FR-059-AC-3", "FR-060-AC-3")]
#[test]
fn graph_shape_then_state_graph_then_terminal_order() {
    let over = family_key("a4a4");
    let predicate = family_key("eeee");
    let model = family_key("1515");
    let bad_claim = state_graph(
        "not-a-kind",
        reference(&over),
        empty(),
        reference(&predicate),
        &[over.clone(), predicate.clone()],
    );
    let bad_terminal = terminal(
        "not-a-mode",
        reference(&model),
        empty(),
        empty(),
        std::slice::from_ref(&model),
    );
    let both = package_with_nodes(vec![bad_terminal.clone(), bad_claim.clone()]);
    refused(
        &both,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/body/members/0",
    );
    let mixed = package_with(state_graph(
        "not-a-kind",
        reference(&over),
        reference(&predicate),
        reference(&predicate),
        &[over.clone(), predicate.clone()],
    ));
    refused(
        &mixed,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/body/members/0",
    );
    let second_bad_claim = state_graph(
        "another-invalid-kind",
        reference(&over),
        empty(),
        reference(&predicate),
        &[over.clone(), predicate.clone()],
    );
    let first_digest = bad_claim["node_id"]["path"][0]
        .as_str()
        .expect("digest")
        .to_owned();
    let second_digest = second_bad_claim["node_id"]["path"][0]
        .as_str()
        .expect("digest")
        .to_owned();
    let by_digest = package_with_nodes(vec![second_bad_claim, bad_claim.clone()]);
    refused(
        &by_digest,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        &format!(
            "/semantic_graph/nodes/{}/body/members/0",
            if second_digest < first_digest {
                base_node_index()
            } else {
                base_node_index() + 1
            }
        ),
    );
    let no_dependency = state_graph(
        "not-a-kind",
        reference(&over),
        empty(),
        reference(&predicate),
        &[over],
    );
    let graph_first = package_with_nodes(vec![bad_terminal.clone(), no_dependency]);
    refused(
        &graph_first,
        Code::InvalidSemanticGraph,
        None,
        "/dependencies",
    );
    let valid_terminal = terminal("any", reference(&model), empty(), empty(), &[model]);
    let terminal_first = package_with_nodes(vec![
        valid_terminal,
        terminal(
            "not-a-mode",
            reference(&family_key("1515")),
            empty(),
            empty(),
            &[family_key("1515")],
        ),
    ]);
    refused(
        &terminal_first,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/body/members/1",
    );
    let mut bad_temporal = package_with_nodes(vec![bad_claim.clone(), bad_terminal.clone()]);
    let temporal_position = bad_temporal["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["semantic_form"] == "temporal_clause")
        .expect("temporal clause");
    bad_temporal["semantic_graph"]["nodes"][temporal_position]["body"]["arguments"][0] =
        reference(&family_key("1515"));
    settle(&mut bad_temporal);
    refused(
        &bad_temporal,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/body/members/0",
    );
    let mut temporal_before_terminal = bad_temporal.clone();
    temporal_before_terminal["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .retain(|node| node["semantic_form"] != "state_graph");
    settle(&mut temporal_before_terminal);
    refused(
        &temporal_before_terminal,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        "/body/arguments/0",
    );
    let abstraction_before_terminal =
        package_with_nodes(vec![malformed_abstraction(), bad_terminal.clone()]);
    refused(
        &abstraction_before_terminal,
        Code::InvalidSemanticGraph,
        None,
        "/body/objects/0",
    );
    let claim_before_abstraction = package_with_nodes(vec![bad_claim, malformed_abstraction()]);
    refused(
        &claim_before_abstraction,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/body/members/0",
    );
    let wrong_protocol_target = family_key("aaaa");
    let mut bad_protocol_reference = node(
        "protocol",
        "protocol_reference",
        reference(&wrong_protocol_target),
        std::slice::from_ref(&wrong_protocol_target),
    );
    bad_protocol_reference["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
    let bad_protocol_only = package_with(bad_protocol_reference.clone());
    refused(
        &bad_protocol_only,
        Code::IllTyped,
        Some(Cause::OperatorIneligible),
        "/body/target",
    );
    let protocol_before_terminal =
        package_with_nodes(vec![bad_terminal.clone(), bad_protocol_reference]);
    refused(
        &protocol_before_terminal,
        Code::IllTyped,
        Some(Cause::OperatorIneligible),
        "/body/target",
    );
    let bad_hybrid_only = package_with(malformed_hybrid_model());
    refused(
        &bad_hybrid_only,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        "/body/members/0/value/members/0/members/0/value/target",
    );
    let terminal_before_hybrid =
        package_with_nodes(vec![bad_terminal.clone(), malformed_hybrid_model()]);
    refused(
        &terminal_before_hybrid,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/body/members/1",
    );
    let mut terminal_before_operations = package_with(bad_terminal);
    let function_position = terminal_before_operations["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "function")
        .expect("function node");
    terminal_before_operations["semantic_graph"]["nodes"][function_position]["body"]["operation"]
        ["identity"] = json!("filament.op.unknown");
    settle(&mut terminal_before_operations);
    refused(
        &terminal_before_operations,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/body/members/1",
    );
}

#[trace("FR-059-AC-2", "FR-060-AC-2")]
#[test]
fn unknown_forms_remain_closed() {
    for (tag, form) in [("claim", "other_claim"), ("temporal", "other_temporal")] {
        let package = package_with(node(tag, form, empty(), &[]));
        refused(&package, Code::InvalidSemanticGraph, None, "/semantic_form");
    }
}

fn sorted_deps(keys: &[&str]) -> Value {
    let mut ids = keys.iter().map(|key| node_id(key)).collect::<Vec<_>>();
    ids.sort_by_key(|id| serde_json::from_value::<CheckedNodeId>(id.clone()).expect("node ID"));
    json!(ids)
}

fn boolean_literal(name: &str, value: bool) -> Value {
    let body = json!({"term": "literal", "type": node_id(&family_key("aaaa")),
        "value_kind": "boolean", "value": value});
    let mut literal = node("value", "literal", body, &[]);
    literal["node_id"] = node_id(&format!("fixture/claim-terminal/literal/{name}"));
    literal["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
    literal
}

fn literal_terminal(name: &str, condition: &str, model: &str) -> Value {
    let mut terminal_node = terminal(
        "when",
        reference(model),
        empty(),
        reference(condition),
        &[model.to_owned(), condition.to_owned()],
    );
    terminal_node["node_id"] = node_id(&format!("fixture/claim-terminal/literal-terminal/{name}"));
    terminal_node
}

/// FSpec TC-961 MT-17.
#[trace("TC-061", "FR-060-AC-7")]
#[test]
fn terminal_admits_a_boolean_literal_condition() {
    let model = family_key("1515");
    for (name, value) in [("true", true), ("false", false)] {
        let literal = boolean_literal(name, value);
        let literal_id = format!("fixture/claim-terminal/literal/{name}");
        let package = package_with_nodes(vec![
            literal.clone(),
            literal_terminal(name, &literal_id, &model),
        ]);
        let read = admitted(&package);
        let rows = read.terminal_rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].declaration,
            TerminalDeclaration::When {
                parameter: None,
                condition: crate::support::checked_package::structural_fixture_id(&node_id(
                    &literal_id
                )),
            }
        );
        let reread = reread_written(&package);
        assert_eq!(reread.terminal_rows(), rows);
        assert_eq!(reread.graph(), read.graph());
        assert_eq!(reread.source_map(), read.source_map());
        let any_node = &admitted(&package_with(terminal(
            "any",
            reference(&model),
            empty(),
            empty(),
            std::slice::from_ref(&model),
        )))
        .graph()
        .nodes[base_node_index()]
        .node_id
        .clone();
        assert_ne!(&rows[0].node, any_node);
    }
    // A literal condition with a `self` parameter reads no `self`.
    let literal_id = "fixture/claim-terminal/literal/true".to_owned();
    let mut with_self = terminal_with_self(&model, "self", "0");
    let parameter = "fixture/claim-terminal/parameter/self/0".to_owned();
    with_self["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(boolean_literal("true", true));
    let last_terminal = with_self["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .iter_mut()
        .find(|node| node["semantic_form"] == "terminal")
        .expect("terminal");
    last_terminal["body"]["members"][3]["value"] = reference(&literal_id);
    last_terminal["dependencies"] = sorted_deps(&[&model, &parameter, &literal_id]);
    settle(&mut with_self);
    match read_with_population(&with_self) {
        CheckedPackageV2ReadResult::Refused(value) => {
            assert_eq!(value.code, Code::InvalidSemanticGraph, "{value:?}");
            assert!(value.loci[0]
                .path
                .as_ref()
                .is_some_and(|path| path.as_str().ends_with("/body/members/2")));
        }
        other => panic!("expected a self parameter the literal never reads to refuse, {other:?}"),
    }

    // The `self` parameter as the condition is not a predicate node.
    let base = terminal_with_self(&model, "self", "0");
    let mut parameter_condition = base.clone();
    let edited = parameter_condition["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .iter_mut()
        .find(|node| node["semantic_form"] == "terminal")
        .expect("terminal");
    edited["body"]["members"][2]["value"] = empty();
    edited["body"]["members"][3]["value"] = reference(&parameter);
    edited["dependencies"] = sorted_deps(&[&model, &parameter]);
    settle(&mut parameter_condition);
    match read_with_population(&parameter_condition) {
        CheckedPackageV2ReadResult::Refused(value) => {
            assert_eq!(value.code, Code::IllTyped, "{value:?}");
            assert_eq!(value.cause, Some(Cause::OperatorIneligible));
            assert!(value.loci[0]
                .path
                .as_ref()
                .is_some_and(|path| path.as_str().ends_with("/body/members/3/value/target")));
        }
        other => panic!("expected the parameter condition to refuse, {other:?}"),
    }

    // An integer literal condition is not Boolean.
    let integer_id = "fixture/claim-terminal/literal/three".to_owned();
    let mut integer = node(
        "value",
        "literal",
        json!({"term": "literal", "type": node_id(&family_key("a3a3")),
            "value_kind": "integer", "value": "3"}),
        &[],
    );
    integer["node_id"] = node_id(&integer_id);
    integer["semantic_type"] = node_id(&family_key("a3a3"));
    integer["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
    let package = package_with_nodes(vec![
        integer,
        literal_terminal("three", &integer_id, &model),
    ]);
    refused(
        &package,
        Code::IllTyped,
        Some(Cause::NonBooleanRoot),
        "/body/members/3/value/target",
    );
}

/// FSpec TC-961 MT-18: three terminal nodes on one model node.
#[trace("TC-487", "FR-060-AC-8")]
#[test]
fn three_terminal_nodes_on_one_model_get_ordinals_in_key_order() {
    let model = family_key("1515");
    let literals = [("one", true), ("two", false)];
    let mut nodes = vec![terminal(
        "any",
        reference(&model),
        empty(),
        empty(),
        std::slice::from_ref(&model),
    )];
    for (name, value) in literals {
        nodes.push(boolean_literal(name, value));
        nodes.push(literal_terminal(
            name,
            &format!("fixture/claim-terminal/literal/{name}"),
            &model,
        ));
    }
    let package = package_with_nodes(nodes);
    let read = admitted(&package);
    let rows = read.terminal_rows();
    assert_eq!(rows.len(), 3);
    for (at, row) in rows.iter().enumerate() {
        assert_eq!(usize::try_from(row.ordinal).expect("ordinal"), at);
        assert_eq!(row.model, rows[0].model);
    }
    assert!(rows.windows(2).all(|pair| pair[0].node < pair[1].node));
    assert_eq!(reread_written(&package).terminal_rows(), rows);
}

/// FSpec TC-961 MT-19: `test/vault` is a prefix of `test/vault-other`, so its
/// rows come first, under both graph orders and against the terminal keys.
#[trace("TC-487", "FR-060-AC-8")]
#[test]
fn terminal_rows_order_a_prefix_model_before_its_extension() {
    let vault = "test/vault";
    let other = "test/vault-other";
    let key = |identity: &str| {
        crate::support::checked_package::structural_fixture_id(
            &json!({"package": identity, "path": ["model", "model_root"], "ordinal": 0}),
        )
    };
    assert!(key(vault) < key(other), "the prefix key sorts first");
    let mut package = v2_all_families();
    select_model(&mut package, vault);
    select_model(&mut package, other);
    let mut ids = Vec::new();
    // The extension's terminal node id sorts before the prefix's terminal id.
    for (model, name) in [
        (other, "aaaa-terminal-other"),
        (vault, "zzzz-terminal-vault"),
    ] {
        let model_id = json!({"package": model, "path": ["model", "model_root"], "ordinal": 0});
        let mut terminal_node = terminal("any", empty(), empty(), empty(), &[]);
        terminal_node["body"]["members"][0]["value"] =
            json!({"term": "reference", "target": model_id});
        terminal_node["dependencies"] = json!([model_id]);
        terminal_node["node_id"] = node_id(name);
        ids.push(crate::support::checked_package::structural_fixture_id(
            &terminal_node["node_id"],
        ));
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(terminal_node);
    }
    assert!(
        ids[0] < ids[1],
        "terminal key order opposes model key order"
    );
    settle(&mut package);
    let expected = [(ids[1].clone(), key(vault)), (ids[0].clone(), key(other))];
    let mut swapped = package.clone();
    let nodes = swapped["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let last = nodes.len() - 1;
    nodes.swap(last, last - 1);
    settle(&mut swapped);
    for variant in [&package, &swapped] {
        let read = admitted(variant);
        let rows: Vec<_> = read
            .terminal_rows()
            .iter()
            .map(|row| (row.node.clone(), row.model.clone()))
            .collect();
        assert_eq!(rows, expected);
        assert!(read.terminal_rows().iter().all(|row| row.ordinal == 0));
    }
}
