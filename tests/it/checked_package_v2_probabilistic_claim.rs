// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-489: the public strict reader admits the FSpec FR-640 nine-binding
//! `claim`/`probabilistic_claim` node, refuses each defect of rules 6 and 7 at
//! the member at fault, charges the reader budget per binding, and exposes each
//! claim through `probabilistic_claims()`.
//!
//! Every case edits the in-repo `v2_all_families()` package, whose `Account`
//! object type is the claims' model and whose `model_root` is their subject.

use crate::support::checked_package::{
    canonical, evidence_with_document, family_model_document, refresh_identity, v2_all_families,
};
use filament_contract_model::{
    encode_checked_package_v2, BoundDirection, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedPackageSidecarsRef,
    CheckedPackageSourceCorrespondenceRef, CheckedPackageV2, CheckedPackageV2ReadResult,
    ProbabilisticActivation, ProbabilisticClaimForm, ProbabilisticEvent, ProbabilisticExact,
    ProbabilisticMeasure, ProbabilisticScheduler,
};
use ix_trace_rs::trace;
use serde_json::{json, Value};

const PACKAGE: &str = "test/families";
const DEPOSIT: &str = "ix://test/families/Account/deposit";

fn id(path: &[&str]) -> Value {
    json!({"package": PACKAGE, "path": path, "ordinal": 0})
}
fn account() -> Value {
    id(&["model", "object_type", "ix://test/families/Account"])
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
fn literal(kind: &str, value: &str) -> Value {
    let type_id = match kind {
        "text" => id(&["fixture/a1a1"]),
        "rational" => id(&["builtin/rational"]),
        _ => id(&["builtin/integer"]),
    };
    json!({"term": "literal", "type": type_id, "value_kind": kind, "value": value})
}
fn text(value: &str) -> Value {
    literal("text", value)
}
fn rational(value: &str) -> Value {
    literal("rational", value)
}
fn integer(value: &str) -> Value {
    literal("integer", value)
}
fn kinded(kind: &str, rest: Vec<(&str, Value)>) -> Value {
    let mut members = vec![binding("kind", text(kind))];
    members.extend(rest.into_iter().map(|(name, value)| binding(name, value)));
    aggregate(members)
}
fn bound(direction: &str, value: Value) -> Value {
    aggregate(vec![
        binding("direction", text(direction)),
        binding("value", value),
    ])
}
fn empty() -> Value {
    aggregate(vec![])
}

/// The predicate, formula and parameter nodes of the family package.
fn predicate() -> Value {
    id(&["fixture/eeee"])
}
fn formula() -> Value {
    id(&["application/temporal/formula/filament.op.temporal.eventually"])
}
fn parameter() -> Value {
    id(&["fixture/a4a4"])
}
fn steady() -> Value {
    id(&["probabilistic", "workload", "steady"])
}
fn duration() -> Value {
    id(&["probabilistic", "reward", "duration"])
}

fn plain_node(label: &[&str], tag: &str, form: &str, semantic_type: Value, body: Value) -> Value {
    let node_id = id(label);
    let mut dependencies = Vec::new();
    collect_references(&body, &mut dependencies);
    dependencies.sort_by_key(Value::to_string);
    dependencies.dedup();
    let mut node = json!({
        "schema_version": "filament.checked-semantic-graph/v2",
        "node_id": node_id, "semantic_type": semantic_type,
        "node_tag": tag, "semantic_form": form, "dependencies": dependencies,
        "occurrences": [], "body": body,
    });
    crate::support::checked_package::carry_check_mode(&mut node);
    node
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

fn operation(operation: &str) -> Vec<Value> {
    vec![
        binding("declaring", reference(account())),
        binding("operation", text(operation)),
    ]
}

fn probabilistic(label: &str, form: &str, body: Value) -> Value {
    let node_id = id(&["probabilistic", form, label]);
    let mut node = plain_node(
        &["probabilistic", form, label],
        "probabilistic",
        form,
        node_id,
        body,
    );
    node["semantic_type"] = id(&["probabilistic", form, label]);
    node
}

fn weight_row(name: &str) -> Value {
    aggregate(vec![
        binding("declaring", reference(account())),
        binding("operation", text(name)),
        binding("weight", rational("1")),
    ])
}

/// The fairness member `weak whole Account::deposit`.
fn fairness(kind: &str, granularity: &str, name: &str) -> Value {
    aggregate(vec![
        binding("kind", text(kind)),
        binding("granularity", text(granularity)),
        binding("declaring", reference(account())),
        binding("operation", text(name)),
    ])
}

/// The nine bindings of a claim node.
struct Claim {
    scheduler: Value,
    reward: Vec<Value>,
    form: Value,
    over: Value,
    activation: Value,
    confidence: Value,
    distributions: Value,
}

fn claim(scheduler: Value, form: Value) -> Claim {
    Claim {
        scheduler,
        reward: vec![],
        form,
        over: empty(),
        activation: kinded("origin", vec![]),
        confidence: empty(),
        distributions: aggregate(vec![
            reference(parameter_deposit()),
            reference(parameter_withdraw()),
        ]),
    }
}

impl Claim {
    fn body(&self) -> Value {
        aggregate(vec![
            binding("subject", reference(id(&["model", "model_root"]))),
            binding("scheduler", self.scheduler.clone()),
            binding("reward", aggregate(self.reward.clone())),
            binding("distributions", self.distributions.clone()),
            binding("delays", empty()),
            binding("form", self.form.clone()),
            binding("over", self.over.clone()),
            binding("activation", self.activation.clone()),
            binding("confidence", self.confidence.clone()),
        ])
    }
}

fn workload() -> Value {
    aggregate(vec![
        binding("model", reference(account())),
        binding("name", text("Steady")),
        binding(
            "weights",
            aggregate(vec![weight_row("deposit"), weight_row("withdraw")]),
        ),
    ])
}

fn reward_of(operation_name: &str, name: &str, function: &str) -> Value {
    aggregate(
        [
            operation(operation_name),
            vec![
                binding("name", text(name)),
                binding("value_type", reference(id(&["builtin/integer"]))),
                binding("expression", reference(id(&["reward", function]))),
            ],
        ]
        .concat(),
    )
}

fn random_parameter(operation_name: &str) -> Value {
    aggregate(
        [
            operation(operation_name),
            vec![
                binding("parameter", text("n")),
                binding(
                    "support",
                    aggregate(vec![aggregate(vec![
                        binding("value", integer("1")),
                        binding("weight", rational("1")),
                    ])]),
                ),
            ],
        ]
        .concat(),
    )
}

/// The reward nodes in ascending node-id order: `cost` on `withdraw`, then the
/// two `duration` nodes, on `deposit` and on `withdraw`.
fn cost_reward() -> Value {
    id(&["probabilistic", "reward", "cost"])
}
fn duration_withdraw() -> Value {
    id(&["probabilistic", "reward", "duration_withdraw"])
}
/// The two random parameters in ascending node-id order.
fn parameter_deposit() -> Value {
    id(&["probabilistic", "random_parameter", "a_deposit"])
}
fn parameter_withdraw() -> Value {
    id(&["probabilistic", "random_parameter", "b_withdraw"])
}

fn avail() -> Claim {
    claim(
        reference(steady()),
        kinded(
            "probability",
            vec![
                (
                    "event",
                    kinded("formula", vec![("formula", reference(formula()))]),
                ),
                ("bound", bound("at_least", rational("1/2"))),
            ],
        ),
    )
}

fn cost() -> Claim {
    let mut cost = claim(
        reference(steady()),
        kinded(
            "expected_reward",
            vec![
                ("until", reference(predicate())),
                ("bound", bound("at_most", rational("3"))),
            ],
        ),
    );
    cost.reward = vec![reference(cost_reward())];
    cost
}

fn p95() -> Claim {
    let mut p95 = claim(
        reference(steady()),
        kinded(
            "quantile",
            vec![
                ("level", rational("19/20")),
                (
                    "measure",
                    kinded(
                        "accumulate",
                        vec![
                            ("from", reference(predicate())),
                            ("until", reference(predicate())),
                            ("within", integer("10")),
                        ],
                    ),
                ),
                ("bound", bound("at_most", rational("5"))),
            ],
        ),
    );
    p95.reward = vec![reference(duration()), reference(duration_withdraw())];
    p95.confidence = aggregate(vec![
        binding("alpha", rational("1/100")),
        binding("beta", rational("1/100")),
        binding("indifference", rational("1/100")),
    ]);
    p95
}

fn fair() -> Claim {
    let mut fair = avail();
    fair.scheduler = aggregate(vec![binding(
        "fairness",
        aggregate(vec![fairness("weak", "whole", "deposit")]),
    )]);
    fair
}

fn document() -> Value {
    let mut document = family_model_document();
    let one = json!({"lower": 1, "upper": 1, "ordered": false, "unique": true});
    for operation in 0..2 {
        document["types"][0]["operations"][operation]["params"] = json!([{
            "name": "n", "typeRef": "ix://filament/native/Integer", "multiplicity": one,
        }]);
    }
    document
}

/// The package with the claims `(name, body)` added over a workload `Steady`
/// and an integer reward `duration`.
fn package(claims: Vec<(&str, Value)>) -> Value {
    let mut value = v2_all_families();
    append(
        &mut value,
        plain_node(
            &["builtin/rational"],
            "scalar_type",
            "rational",
            id(&["builtin/rational"]),
            empty(),
        ),
        "generated",
    );
    let parameter = plain_node(
        &["reward", "parameter"],
        "value",
        "parameter",
        id(&["builtin/integer"]),
        aggregate(vec![
            binding("name", text("n")),
            binding("level", integer("0")),
        ]),
    );
    append(&mut value, parameter, "expression");
    let mut function = plain_node(
        &["reward", "function"],
        "function",
        "pure_function",
        id(&["builtin/integer"]),
        aggregate(vec![
            binding(
                "parameters",
                aggregate(vec![reference(id(&["reward", "parameter"]))]),
            ),
            binding("clause", text("reward")),
        ]),
    );
    function["owner"] = json!({"kind": "model", "identity": PACKAGE, "node": DEPOSIT});
    append(&mut value, function, "generated");
    append(
        &mut value,
        probabilistic("steady", "workload", workload()),
        "declaration",
    );
    append(
        &mut value,
        probabilistic(
            "duration",
            "reward",
            reward_of("deposit", "duration", "function"),
        ),
        "declaration",
    );
    for (label, name, function) in [
        ("duration_withdraw", "duration", "function_withdraw"),
        ("cost", "cost", "function_cost"),
    ] {
        let mut node = plain_node(
            &["reward", function],
            "function",
            "pure_function",
            id(&["builtin/integer"]),
            aggregate(vec![
                binding(
                    "parameters",
                    aggregate(vec![reference(id(&["reward", "parameter"]))]),
                ),
                binding("clause", text("reward")),
            ]),
        );
        node["owner"] = json!({"kind": "model", "identity": PACKAGE,
            "node": "ix://test/families/Account/withdraw"});
        append(&mut value, node, "generated");
        append(
            &mut value,
            probabilistic(label, "reward", reward_of("withdraw", name, function)),
            "declaration",
        );
    }
    for (label, operation_name) in [("a_deposit", "deposit"), ("b_withdraw", "withdraw")] {
        append(
            &mut value,
            probabilistic(label, "random_parameter", random_parameter(operation_name)),
            "declaration",
        );
    }
    let mut rows = Vec::new();
    for (name, body) in claims {
        let label = ["claim", "probabilistic_claim", name];
        let node = plain_node(&label, "claim", "probabilistic_claim", id(&label), body);
        rows.push(
            json!({"declaration": id(&label), "occurrence": {"role": "claim", "ordinal": 0}}),
        );
        append(&mut value, node, "claim");
    }
    rows.sort_by_key(Value::to_string);
    value["checked_term_data"] = json!({
        "equality_decisions": [], "protocol_locations": [], "dispatch_tables": [],
        "probabilistic_claims": rows,
    });
    refresh_identity(&mut value);
    value
}

fn all() -> Value {
    package(vec![
        ("Avail", avail().body()),
        ("Cost", cost().body()),
        ("Fair", fair().body()),
        ("P95", p95().body()),
    ])
}

fn read(value: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        &evidence_with_document(value, PACKAGE, &document()),
    )
}

fn admitted(value: &Value) -> CheckedPackageV2 {
    match read(value) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("expected admission, read {other:?}"),
    }
}

fn refused(value: &Value) -> CheckedPackageRefusal {
    match read(value) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected a refusal, read {other:?}"),
    }
}

/// The index of the claim node `name`.
fn claim_at(value: &Value, name: &str) -> usize {
    value["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"]["path"][2] == name && node["node_tag"] == "claim")
        .expect("claim node")
}

fn assert_refused(
    value: &Value,
    name: &str,
    code: CheckedPackageRefusalCode,
    cause: Option<CheckedPackageRefusalCause>,
    member: &str,
) {
    let refusal = refused(value);
    assert_eq!(refusal.code, code, "{refusal:?}");
    assert_eq!(refusal.cause, cause, "{refusal:?}");
    let node = claim_at(value, name);
    let expected = format!("/semantic_graph/nodes/{node}{member}");
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|path| path.as_str()),
        Some(expected.as_str()),
        "{refusal:?}"
    );
}

const INVALID: CheckedPackageRefusalCode = CheckedPackageRefusalCode::InvalidPackage;

fn invalid_value() -> Option<CheckedPackageRefusalCause> {
    Some(CheckedPackageRefusalCause::InvalidValue)
}

/// The package of `all()` with the body of claim `name` edited.
fn edited(name: &str, edit: impl FnOnce(&mut Value)) -> Value {
    let mut value = all();
    let node = claim_at(&value, name);
    edit(&mut value["semantic_graph"]["nodes"][node]["body"]);
    // The dependencies stay those of the unedited body; they are rewritten to
    // the edited body's targets.
    let mut dependencies = Vec::new();
    collect_references(
        &value["semantic_graph"]["nodes"][node]["body"],
        &mut dependencies,
    );
    dependencies.sort_by_key(Value::to_string);
    dependencies.dedup();
    value["semantic_graph"]["nodes"][node]["dependencies"] = json!(dependencies);
    refresh_identity(&mut value);
    value
}

fn member(body: &mut Value, at: usize) -> &mut Value {
    &mut body["members"][at]["value"]
}

/// FR-377-AC-1: the reader admits the four claims and `probabilistic_claims()`
/// gives each, in node-id order, with its scheduler, form and confidence.
#[trace("TC-489", "FR-377-AC-1")]
#[test]
fn the_public_read_gives_each_admitted_claim() {
    let package = admitted(&all());
    let claims = package.probabilistic_claims();
    assert_eq!(claims.len(), 4);
    let names: Vec<_> = claims
        .iter()
        .map(|claim| claim.node.path[2].clone())
        .collect();
    let rendered: Vec<String> = names.iter().map(|name| format!("{name:?}")).collect();
    assert!(
        rendered[0].contains("Avail") && rendered[3].contains("P95"),
        "{rendered:?}"
    );
    let [avail, cost, fair, p95] = claims else {
        panic!("four claims")
    };
    assert!(matches!(
        avail.scheduler,
        ProbabilisticScheduler::Workload(_)
    ));
    let ProbabilisticClaimForm::Probability { event, bound } = &avail.form else {
        panic!("probability")
    };
    assert!(matches!(event, ProbabilisticEvent::Formula(_)));
    assert_eq!(bound.direction, BoundDirection::AtLeast);
    assert!(matches!(&bound.value, ProbabilisticExact::Rational(r)
        if &*r.numerator == "1" && &*r.denominator == "2"));
    assert!(avail.rewards.is_empty() && avail.confidence.is_none());
    assert_eq!(avail.activation, ProbabilisticActivation::Origin);
    assert!(matches!(
        cost.form,
        ProbabilisticClaimForm::ExpectedReward { .. }
    ));
    assert_eq!(cost.rewards.len(), 1);
    let ProbabilisticScheduler::Every(members) = &fair.scheduler else {
        panic!("every")
    };
    assert_eq!(members.len(), 1);
    assert_eq!(&*members[0].operation, "deposit");
    let ProbabilisticClaimForm::Quantile {
        level,
        measure,
        bound,
    } = &p95.form
    else {
        panic!("quantile")
    };
    assert_eq!((&*level.numerator, &*level.denominator), ("19", "20"));
    assert!(
        matches!(measure, ProbabilisticMeasure::Accumulate { within, .. }
        if matches!(within, ProbabilisticExact::Integer(ten) if *ten == 10.into()))
    );
    assert_eq!(bound.direction, BoundDirection::AtMost);
    let confidence = p95.confidence.as_ref().expect("confidence");
    assert_eq!(&*confidence.indifference.denominator, "100");
    // A package with no claim has the empty read.
    assert!(admitted(&v2_all_families_with_sidecar())
        .probabilistic_claims()
        .is_empty());
}

fn v2_all_families_with_sidecar() -> Value {
    package(vec![])
}

/// FR-377-AC-2: each refusing mutation refuses with FR-640's code, cause and
/// pointer, and a body with a rule 6 and a rule 7 defect refuses at rule 6's.
#[trace("TC-489", "FR-377-AC-2")]
#[test]
fn each_defect_refuses_at_its_member() {
    let body = "/body";
    let form = |at: &str| format!("{body}/members/5/value{at}");
    // `P95`'s form `kind` `median`.
    let value = edited("P95", |b| {
        member(b, 5)["members"][0]["value"] = text("median");
    });
    assert_refused(
        &value,
        "P95",
        INVALID,
        invalid_value(),
        &form("/members/0/value"),
    );
    // `level` and `measure` swapped.
    let value = edited("P95", |b| {
        member(b, 5)["members"].as_array_mut().unwrap().swap(1, 2);
    });
    assert_refused(&value, "P95", INVALID, invalid_value(), &form(""));
    // A quantile whose measure is a fraction.
    let value = edited("P95", |b| {
        member(b, 5)["members"][2]["value"] = kinded(
            "fraction",
            vec![
                ("predicate", reference(predicate())),
                ("window", integer("10")),
                ("weighting", kinded("none", vec![])),
            ],
        );
    });
    assert_refused(
        &value,
        "P95",
        INVALID,
        invalid_value(),
        &form("/members/2/value"),
    );
    // `direction` `below`.
    let value = edited("P95", |b| {
        member(b, 5)["members"][3]["value"]["members"][0]["value"] = text("below");
    });
    assert_refused(
        &value,
        "P95",
        INVALID,
        invalid_value(),
        &form("/members/3/value/members/0/value"),
    );
    // A `bound` `value` of the wrong kind, and a `level` of 1.
    let value = edited("P95", |b| {
        member(b, 5)["members"][3]["value"]["members"][1]["value"] = integer("5");
    });
    assert_refused(
        &value,
        "P95",
        INVALID,
        invalid_value(),
        &form("/members/3/value/members/1/value"),
    );
    let value = edited("P95", |b| {
        member(b, 5)["members"][1]["value"] = rational("1");
    });
    assert_refused(
        &value,
        "P95",
        INVALID,
        invalid_value(),
        &form("/members/1/value"),
    );
    // A `within` of -1.
    let value = edited("P95", |b| {
        member(b, 5)["members"][2]["value"]["members"][3]["value"] = integer("-1");
    });
    assert_refused(
        &value,
        "P95",
        INVALID,
        invalid_value(),
        &form("/members/2/value/members/3/value"),
    );
    // `indifference` 1/20 with the level 19/20.
    let value = edited("P95", |b| {
        member(b, 8)["members"][2]["value"] = rational("1/20");
    });
    assert_refused(
        &value,
        "P95",
        INVALID,
        invalid_value(),
        "/body/members/8/value/members/2/value",
    );
    // `level` equal to `indifference` (θ − ι = 0) refuses; one step above it
    // (θ − ι > 0) and one step below 1 − ι (θ + ι < 1) are admitted.
    let at_level = |level: &str, indifference: &str| {
        edited("P95", |b| {
            member(b, 5)["members"][1]["value"] = rational(level);
            member(b, 8)["members"][2]["value"] = rational(indifference);
        })
    };
    assert_refused(
        &at_level("1/100", "1/100"),
        "P95",
        INVALID,
        invalid_value(),
        "/body/members/8/value/members/2/value",
    );
    assert_refused(
        &at_level("1/200", "1/100"),
        "P95",
        INVALID,
        invalid_value(),
        "/body/members/8/value/members/2/value",
    );
    admitted(&at_level("1/50", "1/100"));
    admitted(&at_level("49/50", "1/100"));
    assert_refused(
        &at_level("99/100", "1/100"),
        "P95",
        INVALID,
        invalid_value(),
        "/body/members/8/value/members/2/value",
    );
    // `indifference` zero is not above zero.
    let value = edited("P95", |b| {
        member(b, 8)["members"][2]["value"] = rational("0");
    });
    assert_refused(
        &value,
        "P95",
        INVALID,
        invalid_value(),
        "/body/members/8/value/members/2/value",
    );
    // `confidence` without `indifference`.
    let value = edited("P95", |b| {
        member(b, 8)["members"].as_array_mut().unwrap().pop();
    });
    assert_refused(
        &value,
        "P95",
        INVALID,
        invalid_value(),
        "/body/members/8/value",
    );
    // `Avail`'s `event` `formula` referencing a predicate node.
    let value = edited("Avail", |b| {
        member(b, 5)["members"][1]["value"]["members"][1]["value"] = reference(predicate());
    });
    assert_refused(
        &value,
        "Avail",
        CheckedPackageRefusalCode::IllTyped,
        Some(CheckedPackageRefusalCause::OperatorIneligible),
        &form("/members/1/value/members/1/value/target"),
    );
    // `Cost`'s empty `reward`, and `Avail`'s non-empty one.
    let value = edited("Cost", |b| {
        member(b, 2)["members"] = json!([]);
    });
    assert_refused(
        &value,
        "Cost",
        INVALID,
        invalid_value(),
        "/body/members/2/value",
    );
    let value = edited("Avail", |b| {
        member(b, 2)["members"] = json!([reference(duration())]);
    });
    assert_refused(
        &value,
        "Avail",
        INVALID,
        invalid_value(),
        "/body/members/2/value",
    );
    // An `activation` `each` whose `trigger` is no parameter node.
    let value = edited("Avail", |b| {
        *member(b, 7) = kinded(
            "each",
            vec![
                ("trigger", reference(predicate())),
                ("guard", reference(predicate())),
            ],
        );
    });
    assert_refused(
        &value,
        "Avail",
        CheckedPackageRefusalCode::InvalidModelBinding,
        Some(CheckedPackageRefusalCause::MalformedDeclaration),
        "/body/members/7/value/members/1/value/target",
    );
    // `over` of a parameter node is admitted; a text `over` is not.
    admitted(&edited("Avail", |b| {
        *member(b, 6) = reference(parameter());
    }));
    // `scheduler` a reward node, a fairness member twice and out of order, and
    // a fairness member naming no operation.
    let value = edited("Avail", |b| {
        *member(b, 1) = reference(duration());
    });
    assert_refused(
        &value,
        "Avail",
        INVALID,
        invalid_value(),
        "/body/members/1/value/target",
    );
    let twice = |b: &mut Value, members: Vec<Value>| {
        *member(b, 1) = aggregate(vec![binding("fairness", aggregate(members))]);
    };
    let at = "/body/members/1/value/members/0/value/members";
    let value = edited("Fair", |b| {
        twice(
            b,
            vec![
                fairness("weak", "whole", "deposit"),
                fairness("weak", "whole", "deposit"),
            ],
        )
    });
    assert_refused(&value, "Fair", INVALID, invalid_value(), &format!("{at}/1"));
    let value = edited("Fair", |b| {
        twice(
            b,
            vec![
                fairness("weak", "whole", "withdraw"),
                fairness("weak", "whole", "deposit"),
            ],
        )
    });
    assert_refused(&value, "Fair", INVALID, invalid_value(), &format!("{at}/1"));
    let value = edited("Fair", |b| {
        twice(b, vec![fairness("medium", "whole", "deposit")])
    });
    assert_refused(&value, "Fair", INVALID, invalid_value(), &format!("{at}/0"));
    let value = edited("Fair", |b| {
        twice(b, vec![fairness("weak", "whole", "nope")])
    });
    assert_refused(&value, "Fair", INVALID, invalid_value(), &format!("{at}/0"));
    // Two fairness members differing only in granularity are admitted, `each`
    // first.
    admitted(&edited("Fair", |b| {
        twice(
            b,
            vec![
                fairness("weak", "each", "deposit"),
                fairness("weak", "whole", "deposit"),
            ],
        )
    }));
    // A body missing `delays` and a `level` of 1: rule 6 refuses at `body`.
    let value = edited("P95", |b| {
        b["members"].as_array_mut().unwrap().remove(4);
        let form = b["members"][4]["value"].as_object_mut().unwrap();
        let _ = form;
        b["members"][4]["value"]["members"][1]["value"] = rational("1");
    });
    assert_refused(&value, "P95", INVALID, invalid_value(), "/body");
}

/// FR-377-AC-3: the claim checks charge one visit per member and binding, in
/// body order, so some budget stops the read at `P95`'s `indifference`.
#[trace("TC-489", "FR-377-AC-3")]
#[test]
fn the_claim_checks_charge_each_binding_in_body_order() {
    let value = all();
    let bytes = canonical(&value);
    let evidence = evidence_with_document(&value, PACKAGE, &document());
    let stops = |work: u64| {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = work;
        match CheckedPackageV2::read(&bytes, limits, &evidence) {
            CheckedPackageV2ReadResult::Incomplete(stop) => {
                Some(stop.path.map(|path| path.as_str().to_owned()))
            }
            CheckedPackageV2ReadResult::Admitted(_) => None,
            other => panic!("budget {work}: {other:?}"),
        }
    };
    // The least budget that admits is the exact full count; one visit short of
    // it is incomplete.
    let (mut low, mut high) = (0_u64, 1_u64 << 40);
    while low < high {
        let middle = low + (high - low) / 2;
        match stops(middle) {
            Some(_) => low = middle + 1,
            None => high = middle,
        }
    }
    let admitting = low;
    assert!(stops(admitting).is_none() && stops(admitting - 1).is_some());
    let p95 = claim_at(&value, "P95");
    let claim_path = |at: &str| format!("/semantic_graph/nodes/{p95}/body{at}");
    let indifference = claim_path("/members/8/value/members/2");
    // The visits charged up to the end of FR-640's reader: the budgets below it
    // stop at or before `P95`'s `indifference`, the last member that reader
    // charges. The budget one visit short of that count stops there, and the
    // next budget stops after it (the later steps charge the same budget).
    let last = (0..admitting)
        .rev()
        .find(|&work| stops(work).flatten().as_deref() == Some(indifference.as_str()))
        .expect("some budget stops at P95's indifference");
    assert_eq!(
        stops(last).flatten().as_deref(),
        Some(indifference.as_str()),
        "one visit short of the reader's count"
    );
    let after = stops(last + 1).flatten();
    assert_ne!(
        after.as_deref(),
        Some(indifference.as_str()),
        "the visit after the count is a later step's"
    );
    assert!(
        after
            .as_deref()
            .is_none_or(|path| !path.contains(&format!("/nodes/{p95}/body"))),
        "{after:?}"
    );
    // Body order: every earlier member of `P95` has a budget that stops at it
    // before the one that stops at `indifference`, `form` before `confidence`.
    let stopped: Vec<(u64, String)> = (last.saturating_sub(120)..=last)
        .filter_map(|work| stops(work).flatten().map(|path| (work, path)))
        .collect();
    let first_at = |path: &str| {
        stopped
            .iter()
            .find(|(_, at)| at == path)
            .map(|(work, _)| *work)
            .unwrap_or_else(|| panic!("no budget stops at {path}"))
    };
    let order = [
        format!("/semantic_graph/nodes/{p95}"),
        claim_path("/members/5"),
        claim_path("/members/6"),
        claim_path("/members/7"),
        claim_path("/members/8"),
        claim_path("/members/8/value/members/0"),
        claim_path("/members/8/value/members/1"),
        indifference.clone(),
    ];
    let works: Vec<u64> = order.iter().map(|path| first_at(path)).collect();
    assert!(works.windows(2).all(|pair| pair[0] < pair[1]), "{works:?}");
}

/// FR-377-AC-4: the placement pass admits a formula reference at the `event`
/// `formula` binding and refuses it anywhere else in the claim, at the claim
/// node, before FR-640's rules.
#[trace("TC-489", "FR-377-AC-4")]
#[test]
fn a_formula_reference_stands_only_at_the_event_formula() {
    let value = edited("P95", |b| {
        member(b, 5)["members"][2]["value"]["members"][1]["value"] = reference(formula());
    });
    assert_refused(
        &value,
        "P95",
        CheckedPackageRefusalCode::IllTyped,
        Some(CheckedPackageRefusalCause::OperatorIneligible),
        "",
    );
    // The same misplacement with a rule-6 defect still refuses at placement.
    let value = edited("P95", |b| {
        member(b, 5)["members"][2]["value"]["members"][1]["value"] = reference(formula());
        member(b, 2)["members"] = json!([]);
    });
    assert_refused(
        &value,
        "P95",
        CheckedPackageRefusalCode::IllTyped,
        Some(CheckedPackageRefusalCause::OperatorIneligible),
        "",
    );
}

/// The writer's bytes for an admitted package.
fn written(package: &CheckedPackageV2) -> Vec<u8> {
    encode_checked_package_v2(
        package.name(),
        package.lock(),
        package.graph(),
        CheckedPackageSourceCorrespondenceRef {
            source_map: package.source_map(),
            refinement_step_rows: package.refinement_step_rows(),
        },
        CheckedPackageSidecarsRef {
            temporal_clause_names: package.temporal_clause_names(),
            claim_names: package.claim_names(),
            operation_effects: None,
            operation_bodies: None,
            checked_term_data: package.checked_term_data(),
            temporal_bindings: package.temporal_bindings(),
            schedulability_claims: package.schedulability_claim_rows(),
        },
        package.capability_report(),
        package.diagnostics(),
        quire_canonical::Limits::new(1 << 24),
    )
    .expect("an admitted claim package writes")
}

/// FR-377-AC-1: the writer's output read back, and the same package with its
/// claim nodes in another graph order, return equal entries.
#[trace("TC-489", "FR-377-AC-1")]
#[test]
fn the_read_survives_a_write_and_a_graph_reorder() {
    let value = all();
    let package = admitted(&value);
    let bytes = written(&package);
    assert_eq!(bytes, canonical(&value));
    let evidence = evidence_with_document(&value, PACKAGE, &document());
    let CheckedPackageV2ReadResult::Admitted(again) =
        CheckedPackageV2::read(&bytes, CheckedPackageReadLimits::bounded(), &evidence)
    else {
        panic!("the written package must read")
    };
    assert_eq!(again.probabilistic_claims(), package.probabilistic_claims());
    // The four claim nodes stand last in the graph; reverse them.
    let mut reordered = value.clone();
    let nodes = reordered["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let at = nodes.len() - 4;
    nodes[at..].reverse();
    refresh_identity(&mut reordered);
    assert_eq!(
        admitted(&reordered).probabilistic_claims(),
        package.probabilistic_claims()
    );
}

/// The claim packages of `claims`, each node's path ending in its body, so a
/// claim has another node exactly when its body differs (FR-407 "Obligation
/// identity"), and the claims read back keyed by name.
fn by_body(
    claims: Vec<(&str, Value)>,
) -> Vec<(
    String,
    filament_contract_model::CheckedProbabilisticClaimNode,
)> {
    let labelled: Vec<(String, Value)> = claims
        .iter()
        .map(|(name, body)| (format!("{name}@{body}"), body.clone()))
        .collect();
    let value = package(
        labelled
            .iter()
            .map(|(n, b)| (n.as_str(), b.clone()))
            .collect(),
    );
    admitted(&value)
        .probabilistic_claims()
        .iter()
        .map(|claim| {
            let name = format!("{:?}", claim.node.path[2]);
            let name = name
                .split('@')
                .next()
                .unwrap_or_default()
                .trim_matches(|c| c == '"' || c == '(')
                .to_owned();
            (name, claim.clone())
        })
        .collect()
}

/// FR-377-AC-1 (FR-640-AC-10): each identity change of one claim changes that
/// claim's entry and `node` and no other claim's.
#[trace("TC-489", "FR-377-AC-1")]
#[test]
fn each_identity_change_gives_that_claim_another_entry() {
    let base = by_body(vec![
        ("Avail", avail().body()),
        ("Cost", cost().body()),
        ("P95", p95().body()),
    ]);
    let entry = |set: &[(
        String,
        filament_contract_model::CheckedProbabilisticClaimNode,
    )],
                 name: &str| {
        set.iter()
            .find(|(n, _)| n.contains(name))
            .expect("claim")
            .1
            .clone()
    };
    type Edit = Box<dyn Fn(&mut Value)>;
    let cases: Vec<(&str, Vec<Edit>)> = vec![
        (
            "P95",
            vec![
                Box::new(|b| {
                    member(b, 5)["members"][3]["value"]["members"][1]["value"] = rational("6")
                }),
                Box::new(|b| {
                    member(b, 5)["members"][3]["value"]["members"][0]["value"] = text("at_least")
                }),
                Box::new(|b| member(b, 5)["members"][1]["value"] = rational("9/10")),
                Box::new(|b| {
                    member(b, 5)["members"][2]["value"]["members"][3]["value"] = integer("11")
                }),
                Box::new(|b| member(b, 8)["members"][0]["value"] = rational("1/50")),
                Box::new(|b| *member(b, 8) = empty()),
            ],
        ),
        (
            "Avail",
            vec![
                Box::new(|b| {
                    member(b, 5)["members"][2]["value"]["members"][1]["value"] = rational("3/5")
                }),
                Box::new(|b| {
                    member(b, 5)["members"][2]["value"]["members"][0]["value"] = text("at_most")
                }),
            ],
        ),
        (
            "Cost",
            vec![Box::new(|b| {
                member(b, 5)["members"][2]["value"]["members"][1]["value"] = rational("4")
            })],
        ),
    ];
    for (name, edits) in cases {
        for edit in edits {
            let mut claims = vec![
                ("Avail", avail().body()),
                ("Cost", cost().body()),
                ("P95", p95().body()),
            ];
            let at = claims.iter().position(|(n, _)| *n == name).expect("claim");
            edit(&mut claims[at].1);
            let changed = by_body(claims);
            for other in ["Avail", "Cost", "P95"] {
                if other == name {
                    assert_ne!(entry(&changed, other).node, entry(&base, other).node);
                    assert_ne!(entry(&changed, other), entry(&base, other));
                } else {
                    assert_eq!(entry(&changed, other), entry(&base, other), "{other}");
                }
            }
        }
    }
}

/// FR-370-AC-22, FR-377-AC-4: a `temporal`/`fairness` node referenced as
/// `Avail`'s `event` `formula` refuses `ill_typed`/`operator-ineligible` at the
/// claim node, before FR-640's rules (also over a rule-6 defect).
#[trace("TC-489", "FR-377-AC-4")]
#[test]
fn a_fairness_node_at_the_event_formula_refuses_at_the_claim() {
    let mut value = all();
    let mut node = value["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_id"] == formula())
        .expect("formula node")
        .clone();
    let fairness_id = id(&["application/temporal/fairness/filament.op.temporal.fair"]);
    node["node_id"] = fairness_id.clone();
    node["semantic_form"] = json!("fairness");
    node["body"]["operator"] = json!("temporal_fairness");
    node["body"]["operation"]["identity"] = json!("filament.op.temporal.fair");
    node["body"]["operation"]["member"] = json!({
        "kind": "fairness", "fairness_kind": "weak", "granularity": "whole",
        "target": {"kind": "operation", "declaration": account(), "name": "deposit"},
    });
    node["body"]["arguments"] = json!([]);
    node["dependencies"] = json!([account()]);
    node["occurrences"] = json!([{"role": "generated", "ordinal": 0}]);
    append(&mut value, node, "generated");
    let claim = claim_at(&value, "Avail");
    let body = &mut value["semantic_graph"]["nodes"][claim]["body"];
    member(body, 5)["members"][1]["value"]["members"][1]["value"] = reference(fairness_id);
    let mut dependencies = Vec::new();
    collect_references(
        &value["semantic_graph"]["nodes"][claim]["body"],
        &mut dependencies,
    );
    dependencies.sort_by_key(Value::to_string);
    dependencies.dedup();
    value["semantic_graph"]["nodes"][claim]["dependencies"] = json!(dependencies);
    refresh_identity(&mut value);
    assert_refused(
        &value,
        "Avail",
        CheckedPackageRefusalCode::IllTyped,
        Some(CheckedPackageRefusalCause::OperatorIneligible),
        "",
    );
}

/// FR-377-AC-2 (FR-640-AC-9): each rule-6 set mutation refuses at its binding:
/// `reward`, `distributions` and `delays` are the exact sets, in order, and the
/// `subject` is a `model_root`.
#[trace("TC-489", "FR-377-AC-2")]
#[test]
fn the_read_sets_are_exact_and_the_subject_is_a_model_root() {
    let at = |index: usize| format!("/body/members/{index}/value");
    let set = |name: &str, index: usize, edit: &dyn Fn(&mut Vec<Value>)| {
        let value = edited(name, |b| {
            let members = member(b, index)["members"].as_array_mut().expect("set");
            edit(members);
        });
        assert_refused(&value, name, INVALID, invalid_value(), &at(index));
    };
    // `P95`'s `reward` without the `withdraw` duration, with the `cost` node
    // added, and with its two members swapped.
    set("P95", 2, &|members| {
        members.pop();
    });
    set("P95", 2, &|members| {
        members.insert(0, reference(cost_reward()));
    });
    set("P95", 2, &|members| members.reverse());
    // `Avail`'s `distributions` without one node, and with its two swapped.
    set("Avail", 3, &|members| {
        members.pop();
    });
    set("Avail", 3, &|members| {
        members.remove(0);
    });
    set("Avail", 3, &|members| members.reverse());
    // `Avail`'s `delays` holding a `random_parameter` node.
    set("Avail", 4, &|members| {
        members.push(reference(parameter_deposit()));
    });
    // `subject` referencing an `object_type` node.
    let value = edited("P95", |b| *member(b, 0) = reference(account()));
    assert_refused(
        &value,
        "P95",
        INVALID,
        invalid_value(),
        "/body/members/0/value/target",
    );
    // The reward set of a form that reads a reward name is complete.
    admitted(&all());
}

/// FR-377-AC-2 (FR-640-AC-11): a `within` typed at an `integer_range` node, and
/// the time forms over an untimed model, refuse at the literal and the `kind`;
/// a path rewritten against its body is admitted.
#[trace("TC-489", "FR-377-AC-2")]
#[test]
fn typed_literals_untimed_time_forms_and_path_rewrites() {
    use crate::support::checked_package::{bounds_body, integer_range_key, node_id};
    let key = integer_range_key("0", "10");
    let mut value = all();
    let range = plain_node(
        &[&key],
        "bounded_domain",
        "integer_range",
        id(&["builtin/integer"]),
        bounds_body("0", "10"),
    );
    let mut range = range;
    range["node_id"] = node_id(&key);
    range["semantic_type"] = id(&["builtin/integer"]);
    append(&mut value, range, "generated");
    let claim = claim_at(&value, "P95");
    {
        let body = &mut value["semantic_graph"]["nodes"][claim]["body"];
        member(body, 5)["members"][2]["value"]["members"][3]["value"]["type"] = node_id(&key);
    }
    refresh_identity(&mut value);
    assert_refused(
        &value,
        "P95",
        INVALID,
        invalid_value(),
        "/body/members/5/value/members/2/value/members/3/value",
    );
    // `elapsed` over the untimed family model refuses at the measure's `kind`.
    let value = edited("P95", |b| {
        member(b, 5)["members"][2]["value"]["members"][0]["value"] = text("elapsed");
    });
    assert_refused(
        &value,
        "P95",
        INVALID,
        invalid_value(),
        "/body/members/5/value/members/2/value/members/0/value",
    );
    // `expected_elapsed` over the untimed model refuses at the form's `kind`.
    let value = edited("Cost", |b| {
        member(b, 5)["members"][0]["value"] = text("expected_elapsed");
    });
    assert_refused(
        &value,
        "Cost",
        INVALID,
        invalid_value(),
        "/body/members/5/value/members/0/value",
    );
    // A path rewritten against an unchanged body is admitted: the reader does
    // not compare a claim's path with its body.
    let mut value = all();
    let claim = claim_at(&value, "P95");
    let old = value["semantic_graph"]["nodes"][claim]["node_id"].clone();
    let rewritten = id(&["claim", "probabilistic_claim", "P95", "6"]);
    crate::support::checked_package::reassign_node_id(&mut value, &old, &rewritten);
    let rows = value["checked_term_data"]["probabilistic_claims"]
        .as_array_mut()
        .expect("rows");
    rows.sort_by_key(Value::to_string);
    refresh_identity(&mut value);
    let package = admitted(&value);
    assert!(package
        .probabilistic_claims()
        .iter()
        .any(|claim| format!("{:?}", claim.node.path).contains('6')));
}
