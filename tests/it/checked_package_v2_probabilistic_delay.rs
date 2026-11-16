// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-476: FR-640 delay distributions over the published timed model. The
//! published fixture is read from `FILAMENT_SPECIFICATION_DIR` (CI sets it);
//! a run without it skips, and `FSPEC_CONFORMANCE=1` makes it fail.

use crate::checked_package_v2_whole_read_accounting::fspec_inputs::{
    evidence_for_published, read_json, DOMAIN_DOCUMENT,
};
use crate::support::checked_package::{canonical, rebuild_source_map, refresh_identity};
use filament_contract_model::{
    CheckedDelayFamily, CheckedNodeId, CheckedPackageReadLimits, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use ix_trace_rs::trace;
use serde_json::{json, Value};
use std::path::PathBuf;

const FIXTURE: &str = "positive-timed-model-nodes.json";

fn proposal() -> Option<PathBuf> {
    match std::env::var("FILAMENT_SPECIFICATION_DIR") {
        Ok(root) if !root.is_empty() => {
            Some(PathBuf::from(root).join("proposals/checked-package-v2"))
        }
        _ if std::env::var_os("FSPEC_CONFORMANCE").is_none() => None,
        _ => panic!("FILAMENT_SPECIFICATION_DIR is required for FSpec conformance"),
    }
}

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
/// parameters)` on `Order::reset`.
fn with_delays(proposal: &std::path::Path, delays: &[(&str, &str, Value)]) -> Value {
    let mut wire =
        read_json(&proposal.join("fixtures").join(FIXTURE)).expect("published timed model");
    let order = node_id(&wire, 53);
    let text = node_id(&wire, 23);
    let rational = node_id(&wire, 35);
    let unit = node_id(&wire, 43);
    // A second unit of the time dimension, for the delays that name `OTHER`.
    let other = json!({"package": "example", "path": ["Example", "ms"], "ordinal": 0});
    if delays
        .iter()
        .any(|(_, _, parameters)| parameters.to_string().contains("OTHER"))
    {
        let mut second = wire["semantic_graph"]["nodes"][43].clone();
        second["node_id"] = other.clone();
        second["declaration"]["qualified_name"] = json!(["Example", "ms"]);
        second["body"]["target_unit_node_id"] = unit.clone();
        let dimension = second["semantic_type"].clone();
        second["dependencies"] = json!([dimension, unit]);
        second["body"]["scale"] = json!({"numerator": "1", "denominator": "1000"});
        wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(second);
    }
    for (label, family, parameters) in delays {
        let id = json!({"package": "example",
            "path": ["probabilistic", "delay_distribution", label], "ordinal": 0});
        let parameters = parameters
            .to_string()
            .replace("\"UNIT\"", &unit.to_string())
            .replace("\"OTHER\"", &other.to_string())
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
    rebuild_source_map(&mut wire);
    refresh_identity(&mut wire);
    wire
}

fn read(proposal: &std::path::Path, wire: &Value) -> CheckedPackageV2ReadResult {
    let evidence = evidence_for_published(
        wire,
        "checked package",
        &proposal.join(DOMAIN_DOCUMENT),
        true,
        None,
        &proposal.join("fixtures"),
    )
    .expect("evidence");
    CheckedPackageV2::read(
        &canonical(wire),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    )
}

fn discrete() -> Value {
    json!({"term": "aggregate", "members": [
        {"term": "aggregate", "members": [
            binding("delay", quantity(&json!("UNIT"), "1")),
            binding("weight", json!({"term": "literal", "type": "RATIONAL",
                "value_kind": "rational", "value": "90"})),
        ]},
        {"term": "aggregate", "members": [
            binding("delay", quantity(&json!("UNIT"), "2")),
            binding("weight", json!({"term": "literal", "type": "RATIONAL",
                "value_kind": "rational", "value": "10"})),
        ]},
    ]})
}

fn bounds() -> Value {
    json!({"term": "aggregate", "members": [
        binding("lower", quantity(&json!("UNIT"), "1")),
        binding("upper", quantity(&json!("UNIT"), "3")),
    ]})
}

fn rate() -> Value {
    json!({"term": "aggregate", "members": [
        binding("rate", json!({"term": "literal", "type": "RATIONAL",
            "value_kind": "rational", "value": "3/2"})),
    ]})
}

/// FR-369-AC-10, FR-369-AC-12: delays on a `dense` model are admitted and read
/// back for each family, with exact numbers and their unit.
#[trace("TC-476", "FR-369-AC-10")]
#[trace("TC-476", "FR-369-AC-12")]
#[test]
fn delays_on_a_dense_model_are_admitted_and_read_for_each_family() {
    let Some(proposal) = proposal() else { return };
    let wire = with_delays(&proposal, &[("a", "discrete", discrete())]);
    let CheckedPackageV2ReadResult::Admitted(package) = read(&proposal, &wire) else {
        panic!("a delay on a dense model must admit");
    };
    let order: CheckedNodeId = serde_json::from_value(node_id(&wire, 53)).expect("id");
    let unit: CheckedNodeId = serde_json::from_value(node_id(&wire, 43)).expect("id");
    let delays = package.delay_distributions(&order);
    assert_eq!(delays.len(), 1);
    assert_eq!(delays[0].operation, "reset");
    let CheckedDelayFamily::Discrete(rows) = &delays[0].family else {
        panic!("discrete family");
    };
    let spelled: Vec<_> = rows
        .iter()
        .map(|(delay, weight)| {
            (
                &*delay.magnitude.numerator,
                delay.unit == &unit,
                &*weight.numerator,
            )
        })
        .collect();
    assert_eq!(spelled, [("1", true, "90"), ("2", true, "10")]);

    for (family, parameters) in [("uniform_interval", bounds()), ("exponential", rate())] {
        let wire = with_delays(&proposal, &[("a", family, parameters)]);
        let CheckedPackageV2ReadResult::Admitted(package) = read(&proposal, &wire) else {
            panic!("a {family} delay on a dense model must admit");
        };
        let delays = package.delay_distributions(&order);
        match (&delays[0].family, family) {
            (CheckedDelayFamily::UniformInterval { lower, upper }, "uniform_interval") => {
                assert_eq!(
                    (&*lower.magnitude.numerator, &*upper.magnitude.numerator),
                    ("1", "3")
                );
                assert!(lower.unit == &unit && upper.unit == &unit);
            }
            (CheckedDelayFamily::Exponential { rate }, "exponential") => {
                assert_eq!((&*rate.numerator, &*rate.denominator), ("3", "2"));
            }
            (other, _) => panic!("unexpected family {other:?}"),
        }
    }
}

/// FR-369-AC-11: a second delay of one operation refuses at the later node.
#[trace("TC-476", "FR-369-AC-11")]
#[test]
fn a_second_delay_of_one_operation_is_a_conflicting_definition() {
    let Some(proposal) = proposal() else { return };
    let empty = json!({"term": "aggregate", "members": []});
    let wire = with_delays(
        &proposal,
        &[("a", "uniform", empty.clone()), ("b", "uniform", empty)],
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&proposal, &wire) else {
        panic!("a second delay must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::ConflictingDefinition)
    );
}

/// The least work limit under which `wire` reads to a verdict.
fn least_work(proposal: &std::path::Path, wire: &Value) -> u64 {
    let evidence = evidence_for_published(
        wire,
        "checked package",
        &proposal.join(DOMAIN_DOCUMENT),
        true,
        None,
        &proposal.join("fixtures"),
    )
    .expect("evidence");
    let bytes = canonical(wire);
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

/// `wire` with `count` copies of a scalar type node before the nodes it already holds.
fn padded(wire: &Value, count: usize) -> Value {
    let mut wire = wire.clone();
    let nodes = wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let template = nodes[23].clone();
    for index in 0..count {
        let id = json!({"package": "example", "path": ["pad", index.to_string()], "ordinal": 0});
        let mut node = template.clone();
        node["node_id"] = id.clone();
        node["semantic_type"] = id;
        nodes.insert(0, node);
    }
    rebuild_source_map(&mut wire);
    refresh_identity(&mut wire);
    wire
}

/// FR-369-AC-17: the join charges one work unit for each node it scans for a
/// delay's `dense` root, so a longer scan costs more work. The cost of the
/// delay is the work the package needs with it less the work it needs without,
/// and it grows by at least the nodes the scan passes.
#[trace("TC-476", "FR-369-AC-17")]
#[test]
fn the_join_charges_the_dense_root_scan() {
    let Some(proposal) = proposal() else { return };
    let without = with_delays(&proposal, &[]);
    let with = with_delays(&proposal, &[("a", "discrete", discrete())]);
    let cost = |count: usize| {
        let (with, without) = if count == 0 {
            (with.clone(), without.clone())
        } else {
            (padded(&with, count), padded(&without, count))
        };
        assert!(
            matches!(
                read(&proposal, &with),
                CheckedPackageV2ReadResult::Admitted(_)
            ),
            "the padded package admits: {:?}",
            read(&proposal, &with)
        );
        least_work(&proposal, &with) - least_work(&proposal, &without)
    };
    let pad = 40;
    let (short, long) = (cost(0), cost(pad));
    assert!(
        long >= short + u64::try_from(pad).expect("pad"),
        "the scan is charged: {short} then {long}"
    );
}

/// The refusal of `wire`, which must be `invalid-value` at `suffix` of the
/// last node.
fn refuse_at(proposal: &std::path::Path, wire: &Value, suffix: &str) {
    let CheckedPackageV2ReadResult::Refused(refusal) = read(proposal, wire) else {
        panic!("a delay quantity in another unit must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::InvalidValue)
    );
    let path = refusal.loci[0].path.as_ref().expect("path");
    let last = wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    assert_eq!(
        path.as_str(),
        format!("/semantic_graph/nodes/{last}/{suffix}")
    );
}

/// FR-369-AC-21: a delay quantity in another unit than the `dense` time
/// source's refuses at the quantity, in each family that has quantities; the
/// source's own unit is admitted.
#[trace("TC-476", "FR-369-AC-21")]
#[test]
fn a_delay_quantity_in_another_unit_than_the_time_source_is_refused() {
    let Some(proposal) = proposal() else { return };
    let discrete = json!({"term": "aggregate", "members": [
        {"term": "aggregate", "members": [
            binding("delay", quantity(&json!("UNIT"), "1")),
            binding("weight", json!({"term": "literal", "type": "RATIONAL",
                "value_kind": "rational", "value": "1"})),
        ]},
        {"term": "aggregate", "members": [
            binding("delay", quantity(&json!("OTHER"), "2")),
            binding("weight", json!({"term": "literal", "type": "RATIONAL",
                "value_kind": "rational", "value": "1"})),
        ]},
    ]});
    refuse_at(
        &proposal,
        &with_delays(&proposal, &[("a", "discrete", discrete)]),
        "body/members/3/value/members/1/members/0/value",
    );
    let interval = json!({"term": "aggregate", "members": [
        binding("lower", quantity(&json!("UNIT"), "1")),
        binding("upper", quantity(&json!("OTHER"), "3")),
    ]});
    refuse_at(
        &proposal,
        &with_delays(&proposal, &[("a", "uniform_interval", interval)]),
        "body/members/3/value/members/1/value",
    );
}

/// FR-369-AC-22: the join charges one validation visit for each delay
/// quantity it compares: a longer discrete list costs exactly 16 units a row,
/// one of them the join's and the rest the earlier stages'.
#[trace("TC-476", "FR-369-AC-22")]
#[test]
fn the_join_charges_each_delay_quantity() {
    let Some(proposal) = proposal() else { return };
    let rows = |count: usize| {
        let members: Vec<Value> = (1..=count)
            .map(|delay| {
                json!({"term": "aggregate", "members": [
                    binding("delay", quantity(&json!("UNIT"), &delay.to_string())),
                    binding("weight", json!({"term": "literal", "type": "RATIONAL",
                        "value_kind": "rational", "value": "1"})),
                ]})
            })
            .collect();
        let wire = with_delays(
            &proposal,
            &[(
                "a",
                "discrete",
                json!({"term": "aggregate", "members": members}),
            )],
        );
        assert!(matches!(
            read(&proposal, &wire),
            CheckedPackageV2ReadResult::Admitted(_)
        ));
        least_work(&proposal, &wire)
    };
    let (short, long) = (rows(1), rows(21));
    assert_eq!(
        long - short,
        16 * 20,
        "work for 1 then 21 rows: {short}, {long}"
    );
}
