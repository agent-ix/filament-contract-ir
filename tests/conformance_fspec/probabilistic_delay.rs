// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-640 delay distributions over the published timed model: the join with a
//! `time dense` source and its unit (FR-369-AC-10, FR-369-AC-11, FR-369-AC-12).

use super::*;
use filament_contract_model::{CheckedDelayFamily, CheckedNodeId};

const FIXTURE: &str = "positive-timed-model-nodes.json";

fn node_id(wire: &Value, position: usize) -> Value {
    wire["semantic_graph"]["nodes"][position]["node_id"].clone()
}

fn quantity(unit: &Value, value: &str) -> Value {
    json!({"term": "literal", "type": unit, "value_kind": "quantity", "value": value})
}

fn binding(name: &str, value: Value) -> Value {
    json!({"term": "binding", "name": name, "value": value})
}

/// The published timed model with delay distributions `(label, family,
/// parameters)` on `Order::reset` (`unit` is the quantity type they use).
pub(crate) fn with_delays(delays: &[(&str, &str, Value)], unit: Option<Value>) -> Value {
    let mut wire = read_json(&fixtures_dir().join(FIXTURE)).expect("published timed model");
    let order = node_id(&wire, 53);
    let text = node_id(&wire, 23);
    let rational = node_id(&wire, 35);
    let unit = unit.unwrap_or_else(|| node_id(&wire, 43));
    for (label, family, parameters) in delays {
        let id = json!({"package": "example",
            "path": ["probabilistic", "delay_distribution", label], "ordinal": 0});
        let parameters = parameters
            .to_string()
            .replace("\"UNIT\"", &unit.to_string())
            .replace("\"RATIONAL\"", &rational.to_string());
        let parameters: Value = serde_json::from_str(&parameters).expect("parameters");
        wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(json!({
                "schema_version": "filament.checked-semantic-graph/v2",
                "node_id": id, "semantic_type": id,
                "node_tag": "probabilistic", "semantic_form": "delay_distribution",
                "dependencies": [order],
                "occurrences": [{"role": "declaration", "ordinal": 0}],
                "body": {"term": "aggregate", "members": [
                    binding("declaring", json!({"term": "reference", "target": order})),
                    binding("operation", json!({"term": "literal", "type": text,
                        "value_kind": "text", "value": "reset"})),
                    binding("family", json!({"term": "literal", "type": text,
                        "value_kind": "text", "value": family})),
                    binding("parameters", parameters),
                ]},
            }));
    }
    fixture_support::rebuild_source_map(&mut wire);
    fixture_support::refresh_identity(&mut wire);
    wire
}

fn discrete() -> Value {
    json!({"term": "aggregate", "members": [
        {"term": "aggregate", "members": [
            binding("delay", quantity(&json!("UNIT"), "1")),
            binding("weight", json!({"term": "literal", "type": "RATIONAL",
                "value_kind": "rational", "value": "90"})),
        ]},
    ]})
}

fn reading(wire: &Value) -> String {
    reading_of(wire).expect("read")
}

/// FR-369-AC-10, FR-369-AC-12: a delay on a model with a `dense` source is
/// admitted and read back by its node.
#[trace("TC-476", "FR-369-AC-10")]
#[trace("TC-476", "FR-369-AC-12")]
#[test]
fn a_delay_on_a_dense_model_is_admitted_and_read() {
    let wire = with_delays(&[("a", "discrete", discrete())], None);
    let CheckedPackageV2ReadResult::Admitted(package) = read_package(&wire).expect("read") else {
        panic!(
            "a delay on a dense model must admit: {:?}",
            read_package(&wire)
        );
    };
    let order: CheckedNodeId = serde_json::from_value(node_id(&wire, 53)).expect("id");
    let delays = package.delay_distributions(&order);
    assert_eq!(delays.len(), 1);
    assert_eq!(delays[0].operation, "reset");
    let CheckedDelayFamily::Discrete(rows) = &delays[0].family else {
        panic!("discrete family");
    };
    assert_eq!(rows.len(), 1);
    assert_eq!(&*rows[0].0.magnitude.numerator, "1");
    assert_eq!(&*rows[0].1.numerator, "90");
}

/// FR-369-AC-11: a second delay of one operation refuses at the later node.
#[trace("TC-476", "FR-369-AC-11")]
#[test]
fn a_second_delay_of_one_operation_is_a_conflicting_definition() {
    let uniform = json!({"term": "aggregate", "members": []});
    let wire = with_delays(
        &[("a", "uniform", uniform.clone()), ("b", "uniform", uniform)],
        None,
    );
    assert_eq!(
        reading(&wire),
        "refused:invalid_package/conflicting-definition"
    );
}
