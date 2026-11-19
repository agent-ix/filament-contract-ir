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
    BoundDirection, CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
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
        distributions: empty(),
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

fn reward() -> Value {
    aggregate(
        [
            operation("deposit"),
            vec![
                binding("name", text("duration")),
                binding("value_type", reference(id(&["builtin/integer"]))),
                binding("expression", reference(id(&["reward", "function"]))),
            ],
        ]
        .concat(),
    )
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
    cost.reward = vec![reference(duration())];
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
    p95.reward = vec![reference(duration())];
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
    document["types"][0]["operations"][0]["params"] = json!([{
        "name": "n", "typeRef": "ix://filament/native/Integer", "multiplicity": one,
    }]);
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
            binding("clause", text("body")),
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
        probabilistic("duration", "reward", reward()),
        "declaration",
    );
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
            _ => None,
        }
    };
    let (mut low, mut high) = (0_u64, 1_u64 << 40);
    while low < high {
        let middle = low + (high - low) / 2;
        match stops(middle) {
            Some(_) => low = middle + 1,
            None => high = middle,
        }
    }
    let p95 = claim_at(&value, "P95");
    let indifference = format!("/semantic_graph/nodes/{p95}/body/members/8/value/members/2");
    let stopped: Vec<String> = (low.saturating_sub(400)..low)
        .filter_map(|work| stops(work).flatten())
        .collect();
    assert!(
        stopped.contains(&indifference),
        "no budget stops at {indifference}"
    );
    // Charges run in body order: `form` before `confidence`.
    let form = format!("/semantic_graph/nodes/{p95}/body/members/5");
    let first_form = stopped.iter().position(|path| *path == form).expect("form");
    let last = stopped
        .iter()
        .position(|path| *path == indifference)
        .expect("last");
    assert!(first_form < last);
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
