// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-489: FR-640 rule 7's time check over the published timed models. A
//! `timed_model` root of either source gives its model a model-time binding:
//! `elapsed` and `expected_elapsed` read, their `within` and threshold are
//! quantities at the time unit, and only a `dense` root admits a `delay`
//! weighting (FR-377-AC-4, FR-640-AC-11).

use super::*;
use filament_contract_model::{
    BoundDirection, CheckedNodeId, ProbabilisticClaimForm, ProbabilisticExact,
    ProbabilisticMeasure, ProbabilisticWeighting,
};

const DENSE: &str = "positive-timed-model-nodes.json";
const TICK: &str = "positive-tick-timed-model-nodes.json";

/// Positions of the fixture nodes a claim names: `(order, root)` in the dense
/// and tick packages. The text, rational, unit, `Flag` expression and `Flag`
/// literal nodes stand at the same positions in both.
fn positions(fixture: &str) -> (usize, usize) {
    if fixture == DENSE {
        (53, 59)
    } else {
        (47, 52)
    }
}
const TEXT: usize = 23;
const INTEGER: usize = 24;
const RATIONAL: usize = 35;
const UNIT: usize = 43;
const PREDICATE_P: usize = 4;
const PREDICATE_Q: usize = 3;
const FORMULA: usize = 29;

fn node_id(wire: &Value, position: usize) -> Value {
    wire["semantic_graph"]["nodes"][position]["node_id"].clone()
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
fn literal(wire: &Value, kind: &str, value: &str) -> Value {
    let at = match kind {
        "text" => TEXT,
        "rational" => RATIONAL,
        "integer" => INTEGER,
        _ => UNIT,
    };
    json!({"term": "literal", "type": node_id(wire, at), "value_kind": kind, "value": value})
}

struct Wire {
    value: Value,
}

impl Wire {
    fn text(&self, value: &str) -> Value {
        literal(&self.value, "text", value)
    }
    fn rational(&self, value: &str) -> Value {
        literal(&self.value, "rational", value)
    }
    fn integer(&self, value: &str) -> Value {
        literal(&self.value, "integer", value)
    }
    fn quantity(&self, value: &str) -> Value {
        literal(&self.value, "quantity", value)
    }
    fn at(&self, position: usize) -> Value {
        reference(node_id(&self.value, position))
    }
    fn kinded(&self, kind: &str, rest: Vec<(&str, Value)>) -> Value {
        let mut members = vec![binding("kind", self.text(kind))];
        members.extend(rest.into_iter().map(|(name, value)| binding(name, value)));
        aggregate(members)
    }
    fn bound(&self, direction: &str, value: Value) -> Value {
        aggregate(vec![
            binding("direction", self.text(direction)),
            binding("value", value),
        ])
    }
}

/// The published timed model of `fixture` with one workload `Steady` on
/// `Order` and one claim `Probe` whose `form` is `form(wire)`.
fn with_claim(fixture: &str, form: impl Fn(&Wire) -> Value) -> Value {
    let mut value = read_json(&fixtures_dir().join(fixture)).expect("published timed model");
    let (order, root) = positions(fixture);
    let wire = Wire {
        value: value.clone(),
    };
    let package = value["semantic_graph"]["nodes"][0]["node_id"]["package"].clone();
    let order_ref = || reference(node_id(&wire.value, order));
    let steady_id = json!({"package": package,
        "path": ["probabilistic", "workload", "Steady"], "ordinal": 0});
    let row = |name: &str| {
        aggregate(vec![
            binding("declaring", order_ref()),
            binding("operation", wire.text(name)),
            binding("weight", wire.rational("1")),
        ])
    };
    let workload = aggregate(vec![
        binding("model", order_ref()),
        binding("name", wire.text("Steady")),
        binding(
            "weights",
            aggregate(vec![row("reset"), row("scaled"), row("size")]),
        ),
    ]);
    let claim_id = json!({"package": package,
        "path": ["claim", "probabilistic_claim", "Probe"], "ordinal": 0});
    let claim = aggregate(vec![
        binding("subject", wire.at(root)),
        binding("scheduler", reference(steady_id.clone())),
        binding("reward", aggregate(vec![])),
        binding("distributions", aggregate(vec![])),
        binding("delays", aggregate(vec![])),
        binding("form", form(&wire)),
        binding("over", aggregate(vec![])),
        binding("activation", wire.kinded("origin", vec![])),
        binding("confidence", aggregate(vec![])),
    ]);
    let mut targets = vec![node_id(&wire.value, order), node_id(&wire.value, root)];
    targets.push(steady_id.clone());
    for text in claim.to_string().split("\"target\":").skip(1) {
        let end = text.find("}}").map_or(text.len(), |end| end + 1);
        if let Ok(id) = serde_json::from_str::<Value>(&text[..end]) {
            targets.push(id);
        }
    }
    targets.retain(|id| *id != node_id(&wire.value, order));
    targets.sort_by_key(Value::to_string);
    targets.dedup();
    let nodes = value["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let node = |id: &Value, tag: &str, form: &str, dependencies: Vec<Value>, role: &str, body| {
        let mut node = json!({
            "schema_version": "filament.checked-semantic-graph/v2",
            "node_id": id, "semantic_type": id, "node_tag": tag, "semantic_form": form,
            "dependencies": dependencies,
            "occurrences": [{"role": role, "ordinal": 0}], "body": body,
        });
        fixture_support::carry_check_mode(&mut node);
        node
    };
    nodes.push(node(
        &steady_id,
        "probabilistic",
        "workload",
        vec![node_id(&wire.value, order)],
        "declaration",
        workload,
    ));
    nodes.push(node(
        &claim_id,
        "claim",
        "probabilistic_claim",
        targets,
        "claim",
        claim,
    ));
    value["checked_term_data"] = json!({
        "equality_decisions": [], "protocol_locations": [], "dispatch_tables": [],
        "probabilistic_claims": [{"declaration": claim_id,
            "occurrence": {"role": "claim", "ordinal": 0}}],
    });
    fixture_support::rebuild_source_map(&mut value);
    fixture_support::refresh_identity(&mut value);
    value
}

fn elapsed(wire: &Wire, within: Value) -> Value {
    wire.kinded(
        "quantile",
        vec![
            ("level", wire.rational("1/2")),
            (
                "measure",
                wire.kinded(
                    "elapsed",
                    vec![
                        ("from", wire.at(PREDICATE_P)),
                        ("until", wire.at(PREDICATE_Q)),
                        ("within", within),
                    ],
                ),
            ),
            ("bound", wire.bound("at_most", wire.quantity("3"))),
        ],
    )
}

fn expected_elapsed(wire: &Wire) -> Value {
    wire.kinded(
        "expected_elapsed",
        vec![
            ("until", wire.at(PREDICATE_P)),
            ("bound", wire.bound("at_most", wire.quantity("3"))),
        ],
    )
}

fn delay_fraction(wire: &Wire) -> Value {
    wire.kinded(
        "long_run_fraction",
        vec![
            ("predicate", wire.at(PREDICATE_P)),
            ("weighting", wire.kinded("delay", vec![])),
            ("bound", wire.bound("at_least", wire.rational("1/2"))),
        ],
    )
}

fn refusal_of(wire: &Value) -> (String, String) {
    let CheckedPackageV2ReadResult::Refused(refusal) = read_package(wire).expect("read") else {
        panic!("expected a refusal: {:?}", read_package(wire));
    };
    (
        reading_of(wire).expect("reading"),
        refusal.loci[0]
            .path
            .as_ref()
            .map(|path| path.to_string())
            .unwrap_or_default(),
    )
}

fn claim_position(wire: &Value) -> usize {
    wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1
}

/// The claim node's member `index` of its body, as a pointer tail.
fn member_at(wire: &Value, index: usize, rest: &str) -> String {
    format!(
        "/semantic_graph/nodes/{}/body/members/{index}{rest}",
        claim_position(wire)
    )
}

fn admitted(wire: &Value) -> filament_contract_model::CheckedPackageV2 {
    match read_package(wire).expect("read") {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("expected admission, read {other:?}"),
    }
}

/// FR-640-AC-11, FR-377-AC-4: over the dense and the tick timed model an
/// `elapsed` quantile with a quantity `within` and an `expected_elapsed`
/// claim are admitted and read back with quantities at the unit node.
#[trace("TC-489", "FR-377-AC-4")]
#[test]
fn elapsed_claims_over_a_timed_model_read_with_time_quantities() {
    for fixture in [DENSE, TICK] {
        let unit: CheckedNodeId = serde_json::from_value(node_id(
            &read_json(&fixtures_dir().join(fixture)).unwrap(),
            UNIT,
        ))
        .expect("unit id");
        let wire = with_claim(fixture, |wire| elapsed(wire, wire.quantity("5")));
        let package = admitted(&wire);
        let [claim] = package.probabilistic_claims() else {
            panic!("one claim in {fixture}");
        };
        let ProbabilisticClaimForm::Quantile { measure, bound, .. } = &claim.form else {
            panic!("quantile in {fixture}");
        };
        let ProbabilisticMeasure::Elapsed { within, .. } = measure else {
            panic!("elapsed in {fixture}: {measure:?}");
        };
        assert!(
            matches!(within, ProbabilisticExact::Quantity { unit: at, .. } if *at == unit),
            "{within:?}"
        );
        assert_eq!(bound.direction, BoundDirection::AtMost);
        assert!(
            matches!(&bound.value, ProbabilisticExact::Quantity { unit: at, .. } if *at == unit)
        );
        let wire = with_claim(fixture, expected_elapsed);
        let package = admitted(&wire);
        assert!(matches!(
            package.probabilistic_claims()[0].form,
            ProbabilisticClaimForm::ExpectedElapsed { .. }
        ));
        // `within` an integer, over a timed model, refuses at the literal.
        let wire = with_claim(fixture, |wire| elapsed(wire, wire.integer("5")));
        assert_eq!(
            refusal_of(&wire),
            (
                "refused:invalid_package/invalid-value".to_owned(),
                member_at(&wire, 5, "/value/members/2/value/members/3/value")
            )
        );
    }
}

/// FR-640-AC-11: a `delay` weighting needs a `dense` root: the dense model
/// admits it and reads it, the tick model refuses at the weighting's `kind`.
#[trace("TC-489", "FR-377-AC-4")]
#[test]
fn a_delay_weighting_needs_a_dense_root() {
    let wire = with_claim(DENSE, delay_fraction);
    let package = admitted(&wire);
    assert!(matches!(
        package.probabilistic_claims()[0].form,
        ProbabilisticClaimForm::LongRunFraction {
            weighting: ProbabilisticWeighting::Delay,
            ..
        }
    ));
    let wire = with_claim(TICK, delay_fraction);
    assert_eq!(
        refusal_of(&wire),
        (
            "refused:invalid_package/invalid-value".to_owned(),
            member_at(&wire, 5, "/value/members/2/value/members/0/value")
        )
    );
}
