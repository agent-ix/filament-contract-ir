// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-038-AC-96 through AC-105 and AC-108, at the package: the temporal step
//! of the reader order (FSpec FR-370) over temporal clauses, formulas and
//! fairness nodes.
//!
//! Every package is built here from this crate's own vocabulary, by editing
//! the in-repo all-families fixture (a clause over `eventually {0, 3}` over
//! `holds`) and re-deriving its application keys, dependency joins and
//! identity with `settle`; nothing of FSpec's fixtures is copied in (the
//! fixtures are read by `make conformance-fspec`, FR-038-AC-107).

use crate::support::checked_package::{
    canonical, evidence_with_document, family_key, fixture_source, model_owner,
    node_id as fixture_node_id, owned_structural_node, reassign_node_id,
    rebuild_complete_source_map, refresh_claim_names, select_model, settle, source_owner,
    v2_all_families,
};
use filament_contract_model::{
    CheckedNodeId, CheckedNodeTag, CheckedPackageLimit, CheckedPackageReadLimits,
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code, CheckedPackageV2,
    CheckedPackageV2ReadResult, CompleteLoweringProfileV2, CompleteLoweringRecordV2,
};
use ix_trace_rs::trace;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};

// ---------------------------------------------------------------------------
// Reading
// ---------------------------------------------------------------------------

pub(super) const ORDERS: &str = "acme/orders";
const ORDERS_VERSION: &str = "1.0.0";
/// A domain package identity the lock never selects.
const OTHER_PACKAGE: &str = "acme/other";
pub(super) const ORDER_NODE: &str = "ix://acme/orders/Order";
pub(super) const SUB_NODE: &str = "ix://acme/orders/Sub";
pub(super) const LEFT_NODE: &str = "ix://acme/orders/Left";
const RIGHT_NODE: &str = "ix://acme/orders/Right";
const BOTH_NODE: &str = "ix://acme/orders/Both";
const NATIVE_INTEGER: &str = "ix://filament/native/Integer";

fn slot(type_ref: &str) -> Value {
    json!({"typeRef": type_ref,
        "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true}})
}

fn domain_type(node: &str, supertypes: &[&str], operations: Value) -> Value {
    json!({
        "identity": node, "displayName": node,
        "kind": {"module": ORDERS, "name": "entity"},
        "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
        "supertypes": supertypes, "fields": [], "operations": operations,
    })
}

/// `Order { scaled(Integer): Integer; reset() }`, `Sub: Order`, and
/// `Both: Left, Right` where `Left` and `Right` each declare `act`.
pub(super) fn orders_document() -> Value {
    let act = |owner: &str| json!([{"identity": format!("{owner}/act"), "params": []}]);
    json!({
        "contractVersion": "2.0.0",
        "package": {"identity": ORDERS, "version": ORDERS_VERSION},
        "constructs": [{
            "kind": {"module": ORDERS, "name": "entity"},
            "construct": {"meaning": "filament.meaning.model.object-type/v1"},
        }],
        "types": [
            domain_type(ORDER_NODE, &[], json!([
                {"identity": format!("{ORDER_NODE}/scaled"), "params": [slot(NATIVE_INTEGER)],
                 "returns": slot(NATIVE_INTEGER)},
                {"identity": format!("{ORDER_NODE}/reset"), "params": []},
            ])),
            domain_type(SUB_NODE, &[ORDER_NODE], json!([])),
            domain_type(LEFT_NODE, &[], act(LEFT_NODE)),
            domain_type(RIGHT_NODE, &[], act(RIGHT_NODE)),
            domain_type(BOTH_NODE, &[LEFT_NODE, RIGHT_NODE], json!([])),
        ],
    })
}

/// The authored path label for a model declaration under `identity`.
fn model_key_in(identity: &str, node: &str) -> String {
    format!("{identity}/{node}")
}

pub(super) fn model_key(node: &str) -> String {
    model_key_in(ORDERS, node)
}

/// Model-owned object types use the selected package and FCD identity in their ID.
pub(super) fn node_id(key: &str) -> Value {
    if let Some((package, node)) = key.split_once("/ix://") {
        if package == ORDERS || package == OTHER_PACKAGE {
            return json!({
                "package": package,
                "path": ["model", "object_type", format!("ix://{node}")],
                "ordinal": 0,
            });
        }
    }
    fixture_node_id(key)
}

fn typed_node_id(key: &str) -> CheckedNodeId {
    serde_json::from_value(node_id(key)).expect("structural node ID")
}

/// Reads `package` with [`orders_document`] supplied under its identity.
pub(super) fn read(package: &Value) -> CheckedPackageV2ReadResult {
    read_limited(package, CheckedPackageReadLimits::bounded())
}

fn read_limited(package: &Value, limits: CheckedPackageReadLimits) -> CheckedPackageV2ReadResult {
    read_with_model(package, &orders_document(), limits)
}

fn read_with_model(
    package: &Value,
    document: &Value,
    limits: CheckedPackageReadLimits,
) -> CheckedPackageV2ReadResult {
    let evidence = evidence_with_document(package, ORDERS, document);
    crate::read_and_account(&canonical(package), limits, &evidence)
}

fn admitted(case: &str, package: &Value) -> CheckedPackageV2 {
    match read(package) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("{case}: expected admission, read {other:?}"),
    }
}

/// The refusal's code, cause and pointer text.
type Outcome = (Code, Option<Cause>, Option<String>);

fn refusal_of(case: &str, package: &Value) -> (Outcome, Option<CheckedNodeId>) {
    match read(package) {
        CheckedPackageV2ReadResult::Refused(refusal) => (
            (
                refusal.code,
                refusal.cause,
                refusal.loci[0]
                    .path
                    .clone()
                    .map(|path| path.as_str().to_owned()),
            ),
            refusal.locus,
        ),
        other => panic!("{case}: expected a refusal, read {other:?}"),
    }
}

/// Asserts the refusal's code, cause, pointer and locus, the locus being the key
/// of the node the pointer is on (the default of FR-038's "Path and locus"
/// table); a refusal with another locus is [`expect_located`].
fn expect(case: &str, package: &Value, code: Code, cause: Cause, path: &str) {
    let position: usize = path
        .strip_prefix("/semantic_graph/nodes/")
        .and_then(|rest| rest.split('/').next())
        .and_then(|node| node.parse().ok())
        .unwrap_or_else(|| panic!("{case}: {path} is on no node"));
    assert_eq!(
        refusal_of(case, package),
        (
            (code, Some(cause), Some(path.to_owned())),
            Some(
                serde_json::from_value(nodes(package)[position]["node_id"].clone())
                    .expect("node ID"),
            ),
        ),
        "{case}"
    );
}

/// Asserts the refusal's code, cause, pointer and locus (the node key).
fn expect_located(case: &str, package: &Value, outcome: (Code, Cause, &str), locus: &str) {
    let (code, cause, path) = outcome;
    assert_eq!(
        refusal_of(case, package),
        (
            (code, Some(cause), Some(path.to_owned())),
            Some(typed_node_id(locus))
        ),
        "{case}"
    );
}

// ---------------------------------------------------------------------------
// Building
// ---------------------------------------------------------------------------

fn nodes(package: &Value) -> &Vec<Value> {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
}

fn find(package: &Value, test: impl Fn(&Value) -> bool) -> usize {
    nodes(package)
        .iter()
        .position(test)
        .expect("the fixture carries the node")
}

fn find_form(package: &Value, tag: &str, form: &str) -> usize {
    find(package, |node| {
        node["node_tag"] == tag && node["semantic_form"] == form
    })
}

fn find_identity(package: &Value, identity: &str) -> usize {
    find(package, |node| {
        node["body"]["operation"]["identity"] == identity
    })
}

fn node_label(package: &Value, position: usize) -> String {
    let owner = &nodes(package)[position]["owner"];
    if owner["kind"] == "model" {
        return model_key_in(
            owner["identity"].as_str().expect("model package"),
            owner["node"].as_str().expect("model node"),
        );
    }
    nodes(package)[position]["node_id"]["path"][0]
        .as_str()
        .expect("key")
        .to_owned()
}

/// Assign distinct producer-owned structural IDs to exercise either node order.
fn order_pair(package: &mut Value, first: usize, second: usize) {
    let first_id = nodes(package)[first]["node_id"].clone();
    let second_id = nodes(package)[second]["node_id"].clone();
    reassign_node_id(package, &first_id, &node_id("a-temporal-order"));
    reassign_node_id(package, &second_id, &node_id("z-temporal-order"));
    settle(package);
}

fn refer(key: &str) -> Value {
    json!({"term": "reference", "target": node_id(key)})
}

fn aaaa() -> String {
    family_key("aaaa")
}

static NEXT: AtomicU64 = AtomicU64::new(1);

/// A distinct producer-assigned label for each synthetic temporal node.
fn fresh() -> String {
    format!("temporal-{}", NEXT.fetch_add(1, Ordering::Relaxed))
}

/// A node keyed `key` whose body is `body`.
pub(super) fn node_with_key(key: &str, tag: &str, form: &str, body: Value) -> Value {
    {
        let mut node = json!({
            "node_id": node_id(key),
            "schema_version": "filament.checked-semantic-graph/v2",
            "node_tag": tag, "semantic_form": form, "semantic_type": node_id(&aaaa()),
            "dependencies": [], "occurrences": [{"role": "generated", "ordinal": 0}],
            "body": body,
        });
        crate::support::checked_package::carry_check_mode(&mut node);
        node
    }
}

fn application(operator: &str, identity: &str, member: Value, arguments: Vec<Value>) -> Value {
    json!({
        "term": "application", "operator": operator,
        "operation": {"identity": identity, "laws": [], "mode": null,
                      "member": member, "leaves": []},
        "result_type": node_id(&aaaa()), "arguments": arguments,
    })
}

fn temporal_identity(short: &str) -> String {
    format!("filament.op.temporal.{short}")
}

/// A `temporal_formula` application of `filament.op.temporal.<short>`.
fn formula_body(short: &str, member: Value, arguments: Vec<Value>) -> Value {
    application(
        "temporal_formula",
        &temporal_identity(short),
        member,
        arguments,
    )
}

/// A `temporal`/`formula` node applying `filament.op.temporal.<short>`.
fn formula(short: &str, member: Value, arguments: Vec<Value>) -> Value {
    node_with_key(
        &fresh(),
        "temporal",
        "formula",
        formula_body(short, member, arguments),
    )
}

/// Appends `node`, returning its position.
pub(super) fn push(package: &mut Value, node: Value) -> usize {
    let list = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    list.push(node);
    list.len() - 1
}

fn interval(lower: Value, upper: Value) -> Value {
    json!({"kind": "temporal_interval", "interval": {"lower": lower, "upper": upper}})
}

fn closed(lower: &str, upper: &str) -> Value {
    interval(json!(lower), json!(upper))
}

fn unbounded() -> Value {
    json!({"kind": "temporal_interval", "interval": null})
}

fn lower_bounded(lower: &str) -> Value {
    interval(json!(lower), Value::Null)
}

const BOUNDED: &str = "filament.temporal.event-position.false-extension/v1";
pub(super) const INFINITE: &str = "filament.temporal.infinite-trace/v1";
const TIMED: &str = "filament.temporal.timed/v1";
const FIVE_PROFILES: [&str; 5] = [
    BOUNDED,
    "filament.temporal.fixed-sample.false-extension/v1",
    "filament.temporal.timestamped-event.finite-window/v1",
    INFINITE,
    TIMED,
];

/// Selects `identity` as the temporal profile, in the lock and in every law of
/// the `temporal_profile` role.
pub(super) fn select_profile(package: &mut Value, identity: &str) {
    for selection in package["lock"]["profile_selections"]
        .as_array_mut()
        .expect("selections")
    {
        if selection["role"] == "temporal_profile" {
            selection["definition"]["identity"] = json!(identity);
        }
    }
    for node in package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
    {
        if identity == TIMED
            && node["node_tag"] == "temporal"
            && node["semantic_form"] == "temporal_clause"
        {
            node["body"]["arguments"][1]["value"] = json!("fixture-clock");
        }
        // A finite-domain hyperproperty names its profile in its own body.
        if node["node_tag"] == "claim" && node["semantic_form"] == "hyperproperty" {
            if let Some(member) = node
                .get_mut("body")
                .and_then(|body| body.get_mut("temporal_profile"))
            {
                member["identity"] = json!(identity);
            }
        }
        // `get_mut`, not indexing: indexing a body without an `operation`
        // would insert one.
        let laws = node
            .get_mut("body")
            .and_then(|body| body.get_mut("operation"))
            .and_then(|operation| operation.get_mut("laws"))
            .and_then(Value::as_array_mut);
        if let Some(laws) = laws {
            for law in laws {
                if law["role"] == "temporal_profile" {
                    law["definition"]["identity"] = json!(identity);
                }
            }
        }
    }
}

/// The fixture, its clause's formula argument naming the application of
/// `filament.op.temporal.<short>` over `arguments` (default: the fixture's
/// `holds` node, `arity` times) with `member`, under `profile`. The position
/// of that root formula node.
fn with_root(
    profile: &str,
    short: &str,
    member: Value,
    arguments: Option<Vec<Value>>,
    arity: usize,
) -> (Value, usize) {
    let mut package = v2_all_families();
    select_profile(&mut package, profile);
    let root = find_identity(&package, &temporal_identity("eventually"));
    let holds = node_label(
        &package,
        find_identity(&package, &temporal_identity("holds")),
    );
    let arguments = arguments.unwrap_or_else(|| vec![refer(&holds); arity]);
    package["semantic_graph"]["nodes"][root]["body"] = formula_body(short, member, arguments);
    settle(&mut package);
    (package, root)
}

pub(super) fn clause(package: &Value) -> usize {
    find_form(package, "temporal", "temporal_clause")
}

fn at_node(position: usize, tail: &str) -> String {
    format!("/semantic_graph/nodes/{position}{tail}")
}

/// The model root selecting the domain package and its model declaration
/// nodes, added to `package`.
pub(super) fn with_model(package: &mut Value) {
    select_model(package, ORDERS);
    for declared in [ORDER_NODE, SUB_NODE, LEFT_NODE, RIGHT_NODE, BOTH_NODE] {
        let key = model_key(declared);
        let mut node = node_with_key(
            &key,
            "model",
            "object_type",
            json!({"term": "aggregate", "members": []}),
        );
        node["semantic_type"] = node_id(&key);
        node["occurrences"] = json!([{"role": "type", "ordinal": 0}]);
        push(
            package,
            owned_structural_node(node, model_owner(ORDERS, declared)),
        );
    }
}

fn fairness_member(declaration: &str, name: &str) -> Value {
    let mut closed_object = serde_json::Map::new();
    closed_object.insert("kind".to_owned(), json!("fairness"));
    closed_object.insert("fairness_kind".to_owned(), json!("weak"));
    closed_object.insert("granularity".to_owned(), json!("whole"));
    closed_object.insert(
        "target".to_owned(),
        json!({"kind":"operation", "declaration":node_id(declaration), "name":name}),
    );
    Value::Object(closed_object)
}

/// A `temporal`/`fairness` node applying `filament.op.temporal.fair` with `member`.
pub(super) fn fairness(member: Value) -> Value {
    node_with_key(
        &fresh(),
        "temporal",
        "fairness",
        application(
            "temporal_fairness",
            &temporal_identity("fair"),
            member,
            vec![],
        ),
    )
}

/// Makes the clause's fairness argument name `fairness_nodes` (positions), in
/// order.
fn with_fairness_argument(package: &mut Value, fairness_nodes: &[usize]) {
    let members: Vec<Value> = fairness_nodes
        .iter()
        .map(|position| refer(&node_label(package, *position)))
        .collect();
    let clause = clause(package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][4] =
        json!({"term": "aggregate", "members": members});
}

/// The model-enabled fixture with one fairness node over `member`, under
/// `profile`. The position of the fairness node.
fn with_fairness(profile: &str, member: Value) -> (Value, usize) {
    let mut package = v2_all_families();
    select_profile(&mut package, profile);
    with_model(&mut package);
    let fair = push(&mut package, fairness(member));
    with_fairness_argument(&mut package, &[fair]);
    settle(&mut package);
    (package, fair)
}

pub(crate) fn admitted_fairness_source_for_contract() -> CheckedPackageV2 {
    // Lowering chooses the first unsupported ID. Put the fairness child
    // before the clause root and formula children using an authored ID.
    let (mut package, fair) =
        with_fairness(INFINITE, fairness_member(&model_key(ORDER_NODE), "scaled"));
    let old = nodes(&package)[fair]["node_id"].clone();
    reassign_node_id(&mut package, &old, &node_id("a-fairness-source"));
    settle(&mut package);
    admitted("contract source with fairness", &package)
}

fn profile_supporting(tags: &[CheckedNodeTag]) -> CompleteLoweringProfileV2 {
    CompleteLoweringProfileV2 {
        reference: filament_contract_model::LoweringProfileRef {
            authority: "filament".into(),
            identity: "complete-v1".into(),
        },
        required_source_selections: Vec::new(),
        supported_tags: tags.iter().copied().collect(),
        require_bounds: false,
        work_limit: u64::MAX,
    }
}

// ---------------------------------------------------------------------------
// AC-96
// ---------------------------------------------------------------------------

#[trace("TC-048", "FR-038-AC-96")]
#[test]
fn tc_048_every_temporal_identity_admits_at_its_node_and_lowers_as_data() {
    let mut package = v2_all_families();
    select_profile(&mut package, BOUNDED);
    let holds = node_label(
        &package,
        find_identity(&package, &temporal_identity("holds")),
    );
    let eventually = find_identity(&package, &temporal_identity("eventually"));
    let mut tree = node_label(&package, eventually);
    let two = |short: &str, left: &str, right: &str| {
        formula(short, Value::Null, vec![refer(left), refer(right)])
    };
    let mut leaves = Vec::new();
    for short in ["true", "false"] {
        leaves.push(push(&mut package, formula(short, Value::Null, vec![])));
    }
    let truth = node_label(&package, leaves[0]);
    let falsehood = node_label(&package, leaves[1]);
    leaves.push(push(
        &mut package,
        formula("not", Value::Null, vec![refer(&holds)]),
    ));
    leaves.push(push(&mut package, two("or", &truth, &falsehood)));
    leaves.push(push(&mut package, two("implies", &truth, &falsehood)));
    for short in ["always", "once", "historically"] {
        leaves.push(push(
            &mut package,
            formula(short, closed("0", "3"), vec![refer(&holds)]),
        ));
    }
    for short in ["until", "release", "since", "triggered"] {
        leaves.push(push(
            &mut package,
            formula(short, closed("1", "2"), vec![refer(&holds), refer(&truth)]),
        ));
    }
    // `and` over the chain, so every formula node is an operand of one.
    for leaf in leaves {
        let operand = node_label(&package, leaf);
        let node = push(&mut package, two("and", &tree, &operand));
        tree = node_label(&package, node);
    }
    let clause_at = clause(&package);
    package["semantic_graph"]["nodes"][clause_at]["body"]["arguments"][5] = refer(&tree);
    settle(&mut package);

    let reader = admitted("fifteen identities", &package);
    let identities: std::collections::BTreeSet<&str> = nodes(&package)
        .iter()
        .filter(|node| node["semantic_form"] == "formula")
        .filter_map(|node| node["body"]["operation"]["identity"].as_str())
        .collect();
    assert_eq!(identities.len(), 15, "{identities:?}");

    // The clause lowers as data under a profile that supports `temporal`, and
    // returns `unsupported` naming it under one that does not.
    let clause_key = typed_node_id(&node_label(&package, clause_at));
    let temporal_only = profile_supporting(&[
        CheckedNodeTag::ScalarType,
        CheckedNodeTag::BoundedDomain,
        CheckedNodeTag::CompositeType,
        CheckedNodeTag::Value,
        CheckedNodeTag::Temporal,
    ]);
    let lowered = reader.lower(std::slice::from_ref(&clause_key), &temporal_only);
    assert!(
        matches!(lowered.records[0], CompleteLoweringRecordV2::Lowered { .. }),
        "{:?}",
        lowered.records[0]
    );
    let mut without = temporal_only.clone();
    without.supported_tags.remove(&CheckedNodeTag::Temporal);
    let refused = reader.lower(std::slice::from_ref(&clause_key), &without);
    assert!(
        matches!(
            refused.records[0],
            CompleteLoweringRecordV2::Unsupported {
                node_tag: CheckedNodeTag::Temporal,
                ..
            }
        ),
        "{:?}",
        refused.records[0]
    );
}

/// The fairness node admits as the body root of a `temporal`/`fairness` node
/// referenced from the fairness argument of a clause selecting the
/// infinite-trace profile; a bounded profile admits no fairness argument.
#[trace("TC-048", "FR-038-AC-96")]
#[test]
fn tc_048_a_fairness_node_admits_under_the_infinite_trace_profile() {
    let (package, fair) =
        with_fairness(INFINITE, fairness_member(&model_key(ORDER_NODE), "scaled"));
    admitted("infinite-trace", &package);
    let (bounded, _) = with_fairness(BOUNDED, fairness_member(&model_key(ORDER_NODE), "scaled"));
    let clause_at = clause(&bounded);
    expect(
        "bounded",
        &bounded,
        Code::InvalidPackage,
        Cause::OperationMemberMismatch,
        &at_node(clause_at, "/body"),
    );
    assert_eq!(nodes(&package)[fair]["node_tag"], "temporal");
}

#[trace("TC-048", "FR-038-AC-275")]
#[test]
fn tc_048_temporal_members_keep_their_closed_shape_for_population() {
    for (label, package, position) in [
        {
            let (package, position) = with_root(
                TIMED,
                "eventually",
                timed_closed(whole("0"), whole("3")),
                None,
                1,
            );
            ("temporal_interval", package, position)
        },
        {
            let (package, position) =
                with_fairness(INFINITE, fairness_member(&model_key(ORDER_NODE), "scaled"));
            ("fairness", package, position)
        },
    ] {
        admitted(label, &package);
        let mut with_population = package.clone();
        with_population["semantic_graph"]["nodes"][position]["body"]["operation"]["member"]
            ["population"] = node_id(&"a".repeat(64));
        settle(&mut with_population);
        let mut with_stray = package;
        with_stray["semantic_graph"]["nodes"][position]["body"]["operation"]["member"]["stray"] =
            node_id(&"a".repeat(64));
        settle(&mut with_stray);
        assert_eq!(
            refusal_of(label, &with_population).0,
            refusal_of(label, &with_stray).0,
            "{label}"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-97
// ---------------------------------------------------------------------------

const INTERVAL_OPERATORS: [(&str, usize); 8] = [
    ("eventually", 1),
    ("always", 1),
    ("once", 1),
    ("historically", 1),
    ("until", 2),
    ("release", 2),
    ("since", 2),
    ("triggered", 2),
];

#[trace("TC-048", "FR-038-AC-97")]
#[test]
fn tc_048_an_interval_member_admits_and_refuses_on_each_interval_operator() {
    let admits = [
        ("{0, 3}", closed("0", "3")),
        ("{9, 10}", closed("9", "10")),
        ("{2, null}", lower_bounded("2")),
        ("null interval", unbounded()),
    ];
    // Negative bounds are refused in strict wire validation
    // (`flat_wire::check_interval_bounds`), under every profile.
    let negative = [
        ("{-1, 3}", closed("-1", "3"), "lower"),
        ("{0, -2}", closed("0", "-2"), "upper"),
        ("{-5, -2}", closed("-5", "-2"), "lower"),
    ];
    let reversed = [
        ("{3, 0}", closed("3", "0")),
        ("{10, 9}", closed("10", "9")),
        (
            "{2^64+1, 2^64}",
            closed("18446744073709551617", "18446744073709551616"),
        ),
    ];
    let wrong_shape = vec![
        ("a fairness member", fairness_member(&aaaa(), "Step")),
        (
            "a member holding a third member",
            json!({"kind": "temporal_interval", "extra": 1,
                   "interval": {"lower": "0", "upper": "3"}}),
        ),
        (
            "an interval holding a third member",
            json!({"kind": "temporal_interval",
                   "interval": {"lower": "0", "upper": "3", "extra": 1}}),
        ),
    ];
    // A bound outside the non-negative integer pattern, malformed as well as
    // negative, is `invalid-value` at that bound in the early stage (merged
    // FSpec FR-370: strict wire validation, before the temporal step), first
    // in member order; a JSON integer is no string.
    let mut outside_pattern = vec![
        ("an integer lower", interval(json!(0), json!("3")), "lower"),
        ("an integer upper", interval(json!("0"), json!(3)), "upper"),
    ];
    for bad in ["1.5", "01", "+1", "", "3x"] {
        outside_pattern.push(("a lower outside the pattern", closed(bad, "9"), "lower"));
        outside_pattern.push(("an upper outside the pattern", closed("0", bad), "upper"));
    }
    for (short, arity) in INTERVAL_OPERATORS {
        for (case, member) in &admits {
            let (package, _) = with_root(INFINITE, short, member.clone(), None, arity);
            admitted(&format!("{short} {case}"), &package);
        }
        for profile in [BOUNDED, INFINITE] {
            for (case, member, bound) in &negative {
                let (package, root) = with_root(profile, short, member.clone(), None, arity);
                expect(
                    &format!("{short} {case} under {profile}"),
                    &package,
                    Code::InvalidPackage,
                    Cause::InvalidValue,
                    &at_node(root, &format!("/body/operation/member/interval/{bound}")),
                );
            }
        }
        for (case, member) in &reversed {
            let (package, root) = with_root(BOUNDED, short, member.clone(), None, arity);
            expect(
                &format!("{short} {case}"),
                &package,
                Code::InvalidPackage,
                Cause::InvalidValue,
                &at_node(root, "/body"),
            );
        }
        for (case, member) in &wrong_shape {
            let (package, root) = with_root(INFINITE, short, member.clone(), None, arity);
            expect(
                &format!("{short} {case} {member}"),
                &package,
                Code::InvalidPackage,
                Cause::OperationMemberMismatch,
                &at_node(root, "/body/operation/member"),
            );
        }
        for (case, member, bound) in &outside_pattern {
            let (package, root) = with_root(INFINITE, short, member.clone(), None, arity);
            expect(
                &format!("{short} {case} {member}"),
                &package,
                Code::InvalidPackage,
                Cause::InvalidValue,
                &at_node(root, &format!("/body/operation/member/interval/{bound}")),
            );
        }
        // A `null` member on an interval-capable operator is refused at the
        // application, not at `operation.member` (merged FSpec FR-370 text).
        let (package, root) = with_root(INFINITE, short, Value::Null, None, arity);
        expect(
            &format!("{short} a null member"),
            &package,
            Code::InvalidPackage,
            Cause::OperationMemberMismatch,
            &at_node(root, "/body"),
        );
    }
}

/// Trace: TC-048, FR-038-AC-378.
#[test]
fn tc_048_out_of_range_operand_precedes_interval_semantics() {
    let (mut package, root) = with_root(BOUNDED, "eventually", closed("-1", "3"), None, 1);
    let count = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .len();
    let path = package["semantic_graph"]["nodes"][root]["node_id"]["path"]
        .as_array_mut()
        .expect("node path");
    let slot = path.len();
    path.push(json!({"node": count}));
    // The out-of-range index is the first entry of the node's operand row.
    let _ = slot;
    let expected = format!("/semantic_graph/operand_rows/{root}/0");
    match read(&package) {
        CheckedPackageV2ReadResult::Refused(refusal) => {
            assert_eq!(refusal.code, Code::MalformedWire);
            assert_eq!(refusal.cause, None);
            assert_eq!(
                refusal.loci[0].path.as_ref().map(|path| path.as_str()),
                Some(expected.as_str())
            );
        }
        other => panic!("index refusal must precede interval defect: {other:?}"),
    }
}

#[trace("TC-048", "FR-038-AC-97")]
#[test]
fn tc_048_invalid_interval_precedes_a_missing_application_result_type() {
    let (mut package, root) = with_root(INFINITE, "always", closed("-1", "3"), None, 1);
    package["semantic_graph"]["nodes"][root]["body"]
        .as_object_mut()
        .expect("application body")
        .remove("result_type");
    expect(
        "invalid interval before incomplete application",
        &package,
        Code::InvalidPackage,
        Cause::InvalidValue,
        &at_node(root, "/body/operation/member/interval/lower"),
    );
}

/// A member on `holds`, `not` and the clause, none of which catalogues one.
#[trace("TC-048", "FR-038-AC-97")]
#[test]
fn tc_048_a_member_on_holds_not_and_the_clause_is_a_member_mismatch() {
    let member = closed("0", "3");
    let mut holds = v2_all_families();
    let position = find_identity(&holds, &temporal_identity("holds"));
    holds["semantic_graph"]["nodes"][position]["body"]["operation"]["member"] = member.clone();
    settle(&mut holds);
    expect(
        "holds",
        &holds,
        Code::InvalidPackage,
        Cause::OperationMemberMismatch,
        &at_node(position, "/body/operation/member"),
    );

    let (not, root) = with_root(BOUNDED, "not", member.clone(), None, 1);
    expect(
        "not",
        &not,
        Code::InvalidPackage,
        Cause::OperationMemberMismatch,
        &at_node(root, "/body/operation/member"),
    );

    let mut clause_member = v2_all_families();
    let clause_at = clause(&clause_member);
    clause_member["semantic_graph"]["nodes"][clause_at]["body"]["operation"]["member"] = member;
    settle(&mut clause_member);
    expect(
        "clause",
        &clause_member,
        Code::InvalidPackage,
        Cause::OperationMemberMismatch,
        &at_node(clause_at, "/body/operation/member"),
    );
}

// ---------------------------------------------------------------------------
// AC-98
// ---------------------------------------------------------------------------

#[trace("TC-048", "FR-038-AC-98")]
#[test]
fn tc_048_the_fairness_member_admits_when_closed_and_refuses_otherwise() {
    let order = model_key(ORDER_NODE);
    for (kind, granularity) in [("weak", "whole"), ("strong", "each")] {
        let mut member = fairness_member(&order, "scaled");
        member["fairness_kind"] = json!(kind);
        member["granularity"] = json!(granularity);
        let (package, _) = with_fairness(INFINITE, member);
        admitted(&format!("{kind}/{granularity}"), &package);
    }
    let mut medium = fairness_member(&order, "scaled");
    medium["fairness_kind"] = json!("medium");
    let mut part = fairness_member(&order, "scaled");
    part["granularity"] = json!("part");
    let mut nameless = fairness_member(&order, "scaled");
    nameless["target"]
        .as_object_mut()
        .expect("target")
        .remove("name");
    let mut extra = fairness_member(&order, "scaled");
    extra["extra"] = json!(1);
    let mut interval_kind = fairness_member(&order, "scaled");
    interval_kind["kind"] = json!("temporal_interval");
    for (case, member) in [
        ("medium", medium),
        ("part", part),
        ("no name", nameless),
        ("an extra member", extra),
        ("null", Value::Null),
        ("a temporal_interval kind", interval_kind),
        ("an interval member", closed("0", "3")),
    ] {
        let (package, fair) = with_fairness(INFINITE, member);
        expect(
            case,
            &package,
            Code::InvalidPackage,
            Cause::OperationMemberMismatch,
            &at_node(fair, "/body/operation/member"),
        );
    }
}

// ---------------------------------------------------------------------------
// AC-100
// ---------------------------------------------------------------------------

/// The refusal `ill_typed`/`operator-ineligible` at the node.
fn misplaced(case: &str, package: &Value, position: usize) {
    expect(
        case,
        package,
        Code::IllTyped,
        Cause::OperatorIneligible,
        &at_node(position, ""),
    );
}

#[trace("TC-048", "FR-038-AC-100")]
#[test]
fn tc_048_a_placed_application_in_another_form_refuses_at_its_node() {
    let holds_body = |package: &Value| {
        let holds = node_label(package, find_identity(package, &temporal_identity("holds")));
        formula_body("holds", Value::Null, vec![refer(&holds)])
    };
    let fair_body = application(
        "temporal_fairness",
        &temporal_identity("fair"),
        fairness_member(&aaaa(), "Step"),
        vec![],
    );
    let cases: [(&str, &str, &str, Option<Value>); 3] = [
        (
            "a formula application in a function node",
            "function",
            "pure_function",
            None,
        ),
        (
            "a fair application in a formula node",
            "temporal",
            "formula",
            Some(fair_body.clone()),
        ),
        (
            "a formula application in a fairness node",
            "temporal",
            "fairness",
            None,
        ),
    ];
    for (case, tag, form, body) in cases {
        let mut package = v2_all_families();
        let body = body.unwrap_or_else(|| holds_body(&package));
        let position = push(&mut package, node_with_key(&fresh(), tag, form, body));
        settle(&mut package);
        misplaced(case, &package, position);
    }
}

#[trace("TC-048", "FR-038-AC-100")]
#[test]
fn tc_048_a_formula_fairness_or_clause_node_needs_its_own_application() {
    let literal = json!({"term": "literal", "type": node_id(&aaaa()),
                         "value_kind": "boolean", "value": true});
    let other_class = application("call", "filament.op.function.call", Value::Null, vec![]);
    for (tag, form) in [
        ("temporal", "formula"),
        ("temporal", "fairness"),
        ("temporal", "temporal_clause"),
    ] {
        for (case, body) in [
            (
                "an empty aggregate",
                json!({"term": "aggregate", "members": []}),
            ),
            ("a literal", literal.clone()),
            ("an application of another class", other_class.clone()),
        ] {
            let mut package = v2_all_families();
            let position = push(&mut package, node_with_key(&fresh(), tag, form, body));
            settle(&mut package);
            misplaced(&format!("{form} with {case}"), &package, position);
        }
    }
}

#[trace("TC-048", "FR-038-AC-100")]
#[test]
fn tc_048_a_reference_to_a_formula_or_fairness_node_refuses_where_it_may_not_stand() {
    // A reference to the formula node from a function node's body.
    let mut package = v2_all_families();
    let formula_at = find_identity(&package, &temporal_identity("eventually"));
    let formula_key = node_label(&package, formula_at);
    let mut function = node_with_key(&fresh(), "function", "pure_function", refer(&formula_key));
    function["occurrences"] = json!([{"role": "declaration", "ordinal": 0}]);
    function["declaration"] = json!({"qualified_name": ["Example", "FormulaUser"]});
    let position = push(
        &mut package,
        owned_structural_node(function, source_owner(&fixture_source())),
    );
    settle(&mut package);
    misplaced("from a function body", &package, position);

    // From a clause's fairness argument.
    let mut package = v2_all_families();
    let clause_at = clause(&package);
    let formula_key = node_label(
        &package,
        find_identity(&package, &temporal_identity("eventually")),
    );
    package["semantic_graph"]["nodes"][clause_at]["body"]["arguments"][4] =
        json!({"term": "aggregate", "members": [refer(&formula_key)]});
    settle(&mut package);
    misplaced("from the fairness argument", &package, clause_at);

    // From a `case` application's argument.
    let mut package = v2_all_families();
    let formula_key = node_label(
        &package,
        find_identity(&package, &temporal_identity("eventually")),
    );
    let case = push(
        &mut package,
        node_with_key(
            &fresh(),
            "expression",
            "case",
            application(
                "case",
                "filament.op.control.case",
                Value::Null,
                vec![refer(&formula_key)],
            ),
        ),
    );
    settle(&mut package);
    misplaced("from a case argument", &package, case);

    // A reference to a fairness node from a clause's formula argument and
    // from a formula operand.
    let mut package = v2_all_families();
    let fair = push(&mut package, fairness(fairness_member(&aaaa(), "Step")));
    let fair_key = node_label(&package, fair);
    let clause_at = clause(&package);
    package["semantic_graph"]["nodes"][clause_at]["body"]["arguments"][5] = refer(&fair_key);
    settle(&mut package);
    misplaced("fairness from the formula argument", &package, clause_at);

    let mut package = v2_all_families();
    let fair = push(&mut package, fairness(fairness_member(&aaaa(), "Step")));
    let fair_key = node_label(&package, fair);
    let root = find_identity(&package, &temporal_identity("eventually"));
    package["semantic_graph"]["nodes"][root]["body"]["arguments"] = json!([refer(&fair_key)]);
    settle(&mut package);
    misplaced("fairness from a formula operand", &package, root);
}

/// A `case` application stands only at the body root of an `expression`/`case`
/// node, and an `expression`/`case` node has no other body.
#[trace("TC-048", "FR-038-AC-100")]
#[test]
fn tc_048_a_case_application_stands_only_at_the_root_of_a_case_node() {
    let case_body = || application("case", "filament.op.control.case", Value::Null, vec![]);
    let mut package = v2_all_families();
    let position = push(
        &mut package,
        node_with_key(&fresh(), "function", "pure_function", case_body()),
    );
    settle(&mut package);
    misplaced(
        "a case application as the root of a function node",
        &package,
        position,
    );

    let mut package = v2_all_families();
    let call = find(&package, |node| {
        node["node_tag"] == "function" && node["body"]["term"] == "application"
    });
    package["semantic_graph"]["nodes"][call]["body"]["arguments"]
        .as_array_mut()
        .expect("arguments")
        .push(case_body());
    settle(&mut package);
    // A nested `case` is refused in strict wire validation at its own
    // `operator`, with the holder as locus; the fixture's call holds one argument already.
    expect(
        "a nested case application",
        &package,
        Code::IllTyped,
        Cause::OperatorIneligible,
        &at_node(call, "/body/arguments/1/operator"),
    );

    // An `expression` node whose form contradicts its root's operator class is
    // `invalid_semantic_graph` at the node's body: a `case` node with another
    // body, and an `expression` node of another form whose root is a `case`
    // application.
    for (case, form, body) in [
        (
            "a case node with another body",
            "case",
            json!({"term": "aggregate", "members": []}),
        ),
        (
            "a call node with a case application as its root",
            "call",
            case_body(),
        ),
    ] {
        let mut package = v2_all_families();
        let position = push(
            &mut package,
            node_with_key(&fresh(), "expression", form, body),
        );
        settle(&mut package);
        let locus = node_label(&package, position);
        // `invalid_semantic_graph` carries no cause.
        assert_eq!(
            refusal_of(case, &package),
            (
                (
                    Code::InvalidSemanticGraph,
                    None,
                    Some(at_node(position, "/body"))
                ),
                Some(typed_node_id(&locus))
            ),
            "{case}"
        );
    }

    // `case` placement is the operation step's: a temporal-step defect is
    // reported ahead of it whatever the key order.
    for key in [LOWEST, HIGHEST] {
        let mut package = v2_all_families();
        let placement = push(&mut package, misplaced_formula(key));
        let case_position = push(
            &mut package,
            node_with_key(&fresh(), "function", "pure_function", case_body()),
        );
        settle(&mut package);
        assert_eq!(
            node_label(&package, placement) < node_label(&package, case_position),
            key == LOWEST,
            "the valid structural key must exercise this key order",
        );
        misplaced(
            &format!("a temporal placement defect beside a case placement defect, {key}"),
            &package,
            placement,
        );
    }
}

// ---------------------------------------------------------------------------
// AC-170
// ---------------------------------------------------------------------------

/// Exhaustive words of the reader's typed refusal variants.
fn code_word(code: Code) -> &'static str {
    match code {
        Code::UnknownContractVersion => "unknown_contract_version",
        Code::MalformedWire => "malformed_wire",
        Code::DuplicateMember => "duplicate_member",
        Code::UnknownMember => "unknown_member",
        Code::NoncanonicalWire => "noncanonical_wire",
        Code::UnknownRequiredCapability => "unknown_required_capability",
        Code::InvalidSemanticGraph => "invalid_semantic_graph",
        Code::InvalidSourceMap => "invalid_source_map",
        Code::UnsupportedNodeTag => "unsupported_node_tag",
        Code::MissingDeclaration => "missing_declaration",
        Code::InvalidModelBinding => "invalid_model_binding",
        Code::InvalidPackage => "invalid_package",
        Code::IllTyped => "ill_typed",
        Code::AmbiguousDeclaration => "ambiguous_declaration",
        Code::MissingImport => "missing_import",
        Code::UnknownProfile => "unknown_profile",
        Code::UnsupportedConstruct => "unsupported_construct",
        Code::WrongSnapshot => "wrong_snapshot",
        Code::InvalidTimedModel => "invalid_timed_model",
    }
}

fn cause_word(cause: Cause) -> &'static str {
    match cause {
        Cause::MalformedJson => "malformed-json",
        Cause::MissingMember => "missing-member",
        Cause::UnknownMember => "unknown-member",
        Cause::WrongValueKind => "wrong-value-kind",
        Cause::DefinitionCycle => "definition-cycle",
        Cause::IncompatibleDefinition => "incompatible-definition",
        Cause::FeatureSetMismatch => "feature-set-mismatch",
        Cause::InsufficientNextCharge => "insufficient-next-charge",
        Cause::MissingName => "missing-name",
        Cause::MalformedDeclaration => "malformed-declaration",
        Cause::InvalidDerivedShape => "invalid-derived-shape",
        Cause::UnknownOperation => "unknown-operation",
        Cause::OperationClassMismatch => "operation-class-mismatch",
        Cause::OperationLawMissing => "operation-law-missing",
        Cause::OperationLawMismatch => "operation-law-mismatch",
        Cause::OperationLawUnselected => "operation-law-unselected",
        Cause::OperationModeMismatch => "operation-mode-mismatch",
        Cause::OperationModeTypeMismatch => "operation-mode-type-mismatch",
        Cause::OperationMemberMismatch => "operation-member-mismatch",
        Cause::OperatorIneligible => "operator-ineligible",
        Cause::DeclarationNominalMismatch => "declaration-nominal-mismatch",
        Cause::AmbiguousName => "ambiguous-name",
        Cause::MissingSelection => "missing-selection",
        Cause::WrongModelSelection => "wrong-model-selection",
        Cause::ConflictingBinding => "conflicting-binding",
        Cause::UnpreservedModelMeaning => "unpreserved-model-meaning",
        Cause::ConflictingDefinition => "conflicting-definition",
        Cause::InvalidSemanticGraph => "invalid-semantic-graph",
        Cause::InvalidValue => "invalid-value",
        Cause::DuplicateMember => "duplicate-member",
        Cause::TypeMismatch => "type-mismatch",
        Cause::UnitMismatch => "unit-mismatch",
        Cause::NonBooleanRoot => "non-boolean-root",
        Cause::MissingTimedModel => "missing-timed-model",
        Cause::TimedNodeReference => "timed-node-reference",
        Cause::DuplicateClockPlace => "duplicate-clock-place",
        Cause::MalformedClockPlace => "malformed-clock-place",
        Cause::InvalidTimeQuantity => "invalid-time-quantity",
        Cause::ClockWithoutDenseTime => "clock-without-dense-time",
        Cause::UnsupportedSelection => "unsupported-selection",
        Cause::WrongSelectionRole => "wrong-selection-role",
        Cause::InexactInteger => "inexact-integer",
        Cause::InexactNumber => "inexact-number",
        Cause::ExpressionForm => "expression-form",
        Cause::DeclarationForm => "declaration-form",
        Cause::WrongExport => "wrong-export",
        Cause::WrongObservation => "wrong-observation",
        Cause::InvalidTaskSet => "invalid-task-set",
        Cause::StaleNodeKey => "stale-node-key",
        Cause::ClockReadOutsideConstraint => "clock-read-outside-constraint",
    }
}

#[trace("TC-367", "FR-038-AC-323")]
#[test]
fn clock_without_dense_time_has_its_paired_wire_cause() {
    assert_eq!(code_word(Code::InvalidPackage), "invalid_package");
    assert_eq!(
        cause_word(Cause::ClockWithoutDenseTime),
        "clock-without-dense-time"
    );
    assert_eq!(
        serde_json::to_value(Cause::ClockWithoutDenseTime).unwrap(),
        json!("clock-without-dense-time")
    );
}

#[trace("TC-048", "FR-038-AC-170")]
#[test]
fn tc_048_relationship_refusal_words_are_typed_and_diagnostics_stay_closed() {
    // The exhaustive mappings keep the public refusal variants explicit.
    assert_eq!(
        code_word(Code::UnsupportedConstruct),
        "unsupported_construct"
    );
    assert_eq!(cause_word(Cause::ExpressionForm), "expression-form");
    assert_eq!(code_word(Code::UnknownProfile), "unknown_profile");
    assert_eq!(cause_word(Cause::DuplicateMember), "duplicate-member");
    assert_eq!(cause_word(Cause::TypeMismatch), "type-mismatch");
    assert_eq!(
        cause_word(Cause::UnsupportedSelection),
        "unsupported-selection"
    );
    assert_eq!(
        cause_word(Cause::WrongSelectionRole),
        "wrong-selection-role"
    );
    // A diagnostics entry has its own closed cause vocabulary.
    let mut package = v2_all_families();
    package["diagnostics"]["entries"] = json!([{
        "stage": "type_checking", "code": "ill_typed", "cause_tag": "expression-form",
        "details": [], "loci": [],
    }]);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Refused(_)
    ));
}

/// A `details` term that references a `temporal`/`formula` or `temporal`/`fairness`
/// node is refused `ill_typed`/`operator-ineligible` at the entry
/// (`/diagnostics/entries/{e}/details/{d}`); the entry is no node, so the refusal
/// carries no node key.
#[trace("TC-048", "FR-038-AC-100")]
#[test]
fn tc_048_a_details_reference_to_a_formula_or_fairness_node_refuses_at_the_entry() {
    let formula_key = node_label(
        &v2_all_families(),
        find_identity(&v2_all_families(), &temporal_identity("eventually")),
    );
    let mut with_fairness_node = v2_all_families();
    let fair = push(
        &mut with_fairness_node,
        fairness(fairness_member(&aaaa(), "Step")),
    );
    settle(&mut with_fairness_node);
    let fair_key = node_label(&with_fairness_node, fair);
    for (case, mut package, target) in [
        ("a formula node", v2_all_families(), formula_key),
        ("a fairness node", with_fairness_node, fair_key),
    ] {
        package["diagnostics"]["entries"] = json!([{
            "stage": "type_checking", "code": "ill_typed", "cause_tag": "invalid-value",
            "details": [{"term": "aggregate", "members": [refer(&target)]}], "loci": [],
        }]);
        // The refusal is at the entry's detail, wherever the reference stands in it.
        assert_eq!(
            refusal_of(case, &package),
            (
                (
                    Code::IllTyped,
                    Some(Cause::OperatorIneligible),
                    Some("/diagnostics/entries/0/details/0".to_owned())
                ),
                None
            ),
            "{case}"
        );
    }
}

/// Forbidden diagnostic references precede the clause's unknown profile,
/// retaining the detail pointer and no node locus.
#[trace("TC-048", "FR-038-AC-100", "FR-038-AC-102")]
#[test]
fn forbidden_diagnostic_reference_precedes_unknown_temporal_profile() {
    let mut package = v2_all_families();
    select_profile(&mut package, "filament.fixture.temporal-profile/v1");
    settle(&mut package);
    let formula = node_label(
        &package,
        find_identity(&package, &temporal_identity("eventually")),
    );
    package["diagnostics"]["entries"] = json!([{
        "stage": "type_checking", "code": "ill_typed", "cause_tag": "invalid-value",
        "details": [refer(&formula)], "loci": [],
    }]);
    assert_eq!(
        refusal_of("a forbidden detail beside an unknown profile", &package),
        (
            (
                Code::IllTyped,
                Some(Cause::OperatorIneligible),
                Some("/diagnostics/entries/0/details/0".to_owned()),
            ),
            None,
        ),
    );
}

/// A diagnostics entry whose `code` is `unsupported_construct` reads, as FSpec's
/// schema allows: the diagnostics wire vocabulary keeps the word.
#[trace("TC-048", "FR-038-AC-101")]
#[test]
fn tc_048_a_diagnostics_entry_may_carry_the_unsupported_construct_code() {
    let mut package = v2_all_families();
    package["diagnostics"]["entries"] = json!([{
        "stage": "type_checking", "code": "unsupported_construct", "cause_tag": "invalid-value",
        "details": [], "loci": [],
    }]);
    settle(&mut package);
    let reader = admitted("unsupported_construct entry", &package);
    assert_eq!(
        serde_json::to_value(reader.diagnostics().entries[0].code).expect("code"),
        "unsupported_construct"
    );
}

// ---------------------------------------------------------------------------
// AC-102
// ---------------------------------------------------------------------------

/// A formula node with a placement defect and a valid low or high key.
fn misplaced_formula(key: &str) -> Value {
    assert!(key == LOWEST || key == HIGHEST, "placement fixture key");
    let body = json!({"term": "aggregate", "members": [{
        "term": "binding", "name": key,
        "value": {"term": "literal", "type": node_id(&aaaa()),
            "value_kind": "boolean", "value": true},
    }]});
    node_with_key(key, "temporal", "formula", body)
}

/// A function node whose application has an unknown identity: an operation
/// defect, keyed by its own authored structural ID.
fn unknown_identity_call() -> Value {
    node_with_key(
        &fresh(),
        "function",
        "pure_function",
        application(
            "call",
            "filament.op.function.not-catalogued",
            Value::Null,
            vec![],
        ),
    )
}

const LOWEST: &str = "a-temporal-placement";
const HIGHEST: &str = "z-temporal-placement";

#[trace("TC-048", "FR-038-AC-102")]
#[test]
fn tc_048_a_placement_defect_is_reported_ahead_of_every_other_defect() {
    for key in [LOWEST, HIGHEST] {
        // Beside an operation defect, whatever the key order.
        let mut package = v2_all_families();
        let placement = push(&mut package, misplaced_formula(key));
        let operation = push(&mut package, unknown_identity_call());
        settle(&mut package);
        assert_eq!(
            node_label(&package, placement) < node_label(&package, operation),
            key == LOWEST,
            "the valid structural key must exercise this key order",
        );
        misplaced(
            &format!("beside an operation defect, {key}"),
            &package,
            placement,
        );

        // Beside a clause defect (an unknown profile), whatever the key order.
        let mut package = v2_all_families();
        let placement = push(&mut package, misplaced_formula(key));
        select_profile(&mut package, "filament.fixture.temporal-profile/v1");
        settle(&mut package);
        misplaced(
            &format!("beside a clause defect, {key}"),
            &package,
            placement,
        );
    }
    // Graph placement also precedes a forbidden diagnostic reference.
    let mut package = v2_all_families();
    let placement = push(&mut package, misplaced_formula(LOWEST));
    settle(&mut package);
    let formula = node_label(
        &package,
        find_identity(&package, &temporal_identity("eventually")),
    );
    package["diagnostics"]["entries"] = json!([{
        "stage": "type_checking", "code": "ill_typed", "cause_tag": "invalid-value",
        "details": [refer(&formula)], "loci": [],
    }]);
    misplaced(
        "beside a forbidden diagnostic reference",
        &package,
        placement,
    );

    // Two placement defects: the lower key.
    let mut package = v2_all_families();
    let low = push(&mut package, misplaced_formula(LOWEST));
    let high = push(&mut package, misplaced_formula(HIGHEST));
    settle(&mut package);
    assert!(node_label(&package, low) < node_label(&package, high));
    misplaced("two placement defects", &package, low);
}

/// A `function` node whose application has an unknown identity, and one whose
/// `operator` differs from its entry's class (`filament.op.control.case` under
/// `unary`). Their IDs are assigned independently of their bodies.
fn operation_defects() -> [(&'static str, Value); 2] {
    let defect = |operator: &str, identity: &str| {
        node_with_key(
            &fresh(),
            "function",
            "pure_function",
            application(
                operator,
                identity,
                Value::Null,
                vec![json!({"term": "literal", "type": node_id(&aaaa()),
                            "value_kind": "text", "value": "s"})],
            ),
        )
    };
    [
        (
            "an unknown identity",
            defect("call", "filament.op.function.not-catalogued"),
        ),
        (
            "a class mismatch",
            defect("unary", "filament.op.control.case"),
        ),
    ]
}

/// The AC's own placement defect, a `temporal_formula` application as the body
/// root of a `function` node, beside each operation defect whose key is
/// lower and higher than its own, in both orders.
#[trace("TC-048", "FR-038-AC-102")]
#[test]
fn tc_048_a_formula_application_in_a_function_node_precedes_operation_defects() {
    for which in 0..2 {
        for placement_is_lower in [true, false] {
            let mut package = v2_all_families();
            let placement = push(
                &mut package,
                node_with_key(
                    &fresh(),
                    "function",
                    "pure_function",
                    formula_body(
                        "true",
                        Value::Null,
                        vec![json!({"term": "literal", "type": node_id(&aaaa()),
                                    "value_kind": "text", "value": "p"})],
                    ),
                ),
            );
            let [unknown, mismatch] = operation_defects();
            let (case, node) = if which == 0 { unknown } else { mismatch };
            let operation = push(&mut package, node);
            if placement_is_lower {
                order_pair(&mut package, placement, operation);
            } else {
                order_pair(&mut package, operation, placement);
            }
            misplaced(
                &format!("{case}, placement lower: {placement_is_lower}"),
                &package,
                placement,
            );
        }
    }
}

/// Two defects of one clause, the later stage's defect reported only once the
/// earlier one is cured: `over` before fairness resolution, and fairness
/// resolution before profile fit (the bounded profile also refuses the
/// non-empty fairness argument and the `null` interval, both fit defects).
#[trace("TC-048", "FR-038-AC-102")]
#[test]
fn tc_048_within_one_clause_the_stages_run_in_order() {
    let build = |profile: &str, over_defect: bool| {
        let (mut package, _) =
            with_fairness(profile, fairness_member(&model_key(ORDER_NODE), "missing"));
        let root = find_identity(&package, &temporal_identity("eventually"));
        package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"] = unbounded();
        let clause_at = clause(&package);
        if over_defect {
            package["semantic_graph"]["nodes"][clause_at]["body"]["arguments"][0] = refer(&aaaa());
        }
        settle(&mut package);
        (package, clause_at)
    };
    let (package, clause_at) = build(INFINITE, true);
    expect_located(
        "over before fairness resolution",
        &package,
        (
            Code::InvalidModelBinding,
            Cause::MalformedDeclaration,
            &at_node(clause_at, "/body/arguments/0"),
        ),
        &aaaa(),
    );
    let (package, _) = build(BOUNDED, false);
    let fair = find_form(&package, "temporal", "fairness");
    expect_located(
        "fairness resolution before profile fit",
        &package,
        (
            Code::MissingDeclaration,
            Cause::MissingName,
            &at_node(fair, "/body/operation/member/target/name"),
        ),
        &model_key(ORDER_NODE),
    );
}

/// The interval bounds are the last stage: `{3, 0}` under a bounded profile in
/// a clause whose profile fit also fails refuses the fit defect first, in both
/// operand orders of the two operators.
#[trace("TC-048", "FR-038-AC-102")]
#[test]
fn tc_048_profile_fit_precedes_bounds_whichever_operator_comes_first() {
    for fit_first in [true, false] {
        let mut package = v2_all_families();
        select_profile(&mut package, BOUNDED);
        let holds = node_label(
            &package,
            find_identity(&package, &temporal_identity("holds")),
        );
        let fit = push(
            &mut package,
            formula("always", unbounded(), vec![refer(&holds)]),
        );
        let bounds = push(
            &mut package,
            formula("once", closed("3", "0"), vec![refer(&holds)]),
        );
        let (first, second) = if fit_first {
            (fit, bounds)
        } else {
            (bounds, fit)
        };
        let operands = vec![
            refer(&node_label(&package, first)),
            refer(&node_label(&package, second)),
        ];
        let root = find_identity(&package, &temporal_identity("eventually"));
        package["semantic_graph"]["nodes"][root]["body"] =
            formula_body("and", Value::Null, operands);
        settle(&mut package);
        expect(
            &format!("fit first: {fit_first}"),
            &package,
            Code::InvalidPackage,
            Cause::OperationMemberMismatch,
            &at_node(fit, "/body"),
        );
    }
}

/// A formula node no clause reaches is checked for `lower > upper` too, in the
/// sweep after every clause (an IR reading, AC-97); `{0, 3}` there admits.
#[trace("TC-048", "FR-038-AC-97")]
#[test]
fn tc_048_an_unreached_formula_node_with_lower_above_upper_is_refused() {
    for (member, refused) in [(closed("0", "3"), false), (closed("3", "0"), true)] {
        let mut package = v2_all_families();
        let holds = node_label(
            &package,
            find_identity(&package, &temporal_identity("holds")),
        );
        let unreached = push(&mut package, formula("once", member, vec![refer(&holds)]));
        settle(&mut package);
        if refused {
            expect(
                "an unreached {3, 0}",
                &package,
                Code::InvalidPackage,
                Cause::InvalidValue,
                &at_node(unreached, "/body"),
            );
        } else {
            admitted("an unreached {0, 3}", &package);
        }
    }
}

/// A second clause named `name`, over its own formula tree (`eventually` with
/// `member` over a `holds` of its own) and the fixture's parameter, with the
/// `over` defect of naming a `scalar_type` when `over_defect`. Its position.
fn second_clause(package: &mut Value, name: &str, member: Value, over_defect: bool) -> usize {
    let first = clause(package);
    let mut holds_body =
        nodes(package)[find_identity(package, &temporal_identity("holds"))]["body"].clone();
    // A different operand, so the two `holds` nodes are not one node.
    holds_body["arguments"][0]["value"] = json!(false);
    let holds = push(
        package,
        node_with_key(&fresh(), "temporal", "formula", holds_body),
    );
    let operand = refer(&node_label(package, holds));
    let root = push(package, formula("eventually", member, vec![operand]));
    let mut body = nodes(package)[first]["body"].clone();
    body["arguments"][1]["value"] = json!(name);
    body["arguments"][5] = refer(&node_label(package, root));
    if over_defect {
        body["arguments"][0] = refer(&aaaa());
    }
    push(
        package,
        node_with_key(&fresh(), "temporal", "temporal_clause", body),
    )
}

/// Two clauses: the lower-key clause's later-stage defect (profile fit) is
/// reported ahead of the higher-key clause's earlier-stage defect (`over`),
/// in both producer-assigned ID orders.
#[trace("TC-048", "FR-038-AC-102")]
#[test]
fn tc_048_a_lower_key_clause_is_taken_through_every_stage_before_the_next() {
    for fit_clause_is_lower in [true, false] {
        // The fixture's own clause carries the fit defect (a null interval
        // under the bounded profile); the added one the `over` defect.
        let mut package = v2_all_families();
        select_profile(&mut package, BOUNDED);
        let root = find_identity(&package, &temporal_identity("eventually"));
        package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"] = unbounded();
        let added = second_clause(&mut package, "other", closed("0", "3"), true);
        let own = clause(&package);
        if fit_clause_is_lower {
            order_pair(&mut package, own, added);
            // The lower clause's later-stage defect comes first.
            expect(
                "the fit defect of the lower clause",
                &package,
                Code::InvalidPackage,
                Cause::OperationMemberMismatch,
                &at_node(root, "/body"),
            );
        } else {
            order_pair(&mut package, added, own);
            expect_located(
                "the over defect of the lower clause",
                &package,
                (
                    Code::InvalidModelBinding,
                    Cause::MalformedDeclaration,
                    &at_node(added, "/body/arguments/0"),
                ),
                &aaaa(),
            );
        }
    }
}

/// A member whose shape the operation step refuses is skipped by the temporal
/// step, whatever the profile: `{1.5, 0}` and a wrong-kind member refuse
/// `operation-member-mismatch`, never `invalid-value`.
#[trace("TC-048", "FR-038-AC-102")]
#[test]
fn tc_048_the_temporal_step_skips_a_member_the_operation_step_refuses() {
    for profile in [BOUNDED, INFINITE] {
        // A wrong-kind member is skipped by the temporal step and refused at
        // `operation.member`; a `null` member at the application; a bound outside
        // the pattern never reaches the step, being `invalid-value` at the bound
        // in strict wire validation (`flat_wire::check_interval_bounds`), never
        // `operation-member-mismatch`.
        for (case, member, cause, tail) in [
            (
                "a fairness member",
                fairness_member(&aaaa(), "Step"),
                Cause::OperationMemberMismatch,
                "/body/operation/member",
            ),
            ("null", Value::Null, Cause::OperationMemberMismatch, "/body"),
            (
                "{1.5, 0}",
                closed("1.5", "0"),
                Cause::InvalidValue,
                "/body/operation/member/interval/lower",
            ),
        ] {
            let (package, root) = with_root(profile, "eventually", member, None, 1);
            expect(
                &format!("{case} under {profile}"),
                &package,
                Code::InvalidPackage,
                cause,
                &at_node(root, tail),
            );
        }
    }
}

/// A negative bound is refused in strict wire validation
/// (`flat_wire::check_interval_bounds`): before placement
/// and every temporal step, so also beside a placement defect at a lower-key
/// node and under a profile whose fit it would also fail.
#[trace("TC-048", "FR-038-AC-102")]
#[test]
fn tc_048_a_negative_bound_is_refused_before_any_temporal_step() {
    for profile in [BOUNDED, INFINITE] {
        for (case, member, bound) in [
            ("{0, -2}", closed("0", "-2"), "upper"),
            ("{-1, -3}", closed("-1", "-3"), "lower"),
            ("{-1, null}", lower_bounded("-1"), "lower"),
        ] {
            let (mut package, root) = with_root(profile, "eventually", member, None, 1);
            expect(
                &format!("{case} under {profile}"),
                &package,
                Code::InvalidPackage,
                Cause::InvalidValue,
                &at_node(root, &format!("/body/operation/member/interval/{bound}")),
            );
            // Beside a placement defect at a lower-key node.
            push(&mut package, misplaced_formula(LOWEST));
            settle(&mut package);
            expect(
                &format!("{case} beside a placement defect"),
                &package,
                Code::InvalidPackage,
                Cause::InvalidValue,
                &at_node(root, &format!("/body/operation/member/interval/{bound}")),
            );
        }
    }
}

/// `lower > upper` is the bounds stage, after profile fit.
#[trace("TC-048", "FR-038-AC-102")]
#[test]
fn tc_048_lower_above_upper_is_refused_at_the_application_after_profile_fit() {
    let (package, root) = with_root(BOUNDED, "eventually", closed("3", "0"), None, 1);
    expect(
        "lower above upper",
        &package,
        Code::InvalidPackage,
        Cause::InvalidValue,
        &at_node(root, "/body"),
    );
    // With the profile fit also failing (a non-empty fairness argument under a
    // bounded profile), the fit defect is reported first.
    let (mut package, _) =
        with_fairness(BOUNDED, fairness_member(&model_key(ORDER_NODE), "scaled"));
    let root = find_identity(&package, &temporal_identity("eventually"));
    package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"] = closed("3", "0");
    settle(&mut package);
    let clause_at = clause(&package);
    expect(
        "fit before bounds",
        &package,
        Code::InvalidPackage,
        Cause::OperationMemberMismatch,
        &at_node(clause_at, "/body"),
    );
}

// ---------------------------------------------------------------------------
// AC-103
// ---------------------------------------------------------------------------

#[trace("TC-048", "FR-038-AC-103", "FR-038-AC-155")]
#[test]
fn tc_048_a_clause_over_must_be_a_declared_parameter() {
    admitted("a parameter dependency", &v2_all_families());
    // A declared dependency that is a `scalar_type` node.
    let mut package = v2_all_families();
    let clause_at = clause(&package);
    package["semantic_graph"]["nodes"][clause_at]["body"]["arguments"][0] = refer(&aaaa());
    settle(&mut package);
    // The path is on the argument and the locus is the target's key.
    expect_located(
        "a scalar_type over",
        &package,
        (
            Code::InvalidModelBinding,
            Cause::MalformedDeclaration,
            &at_node(clause_at, "/body/arguments/0"),
        ),
        &aaaa(),
    );
    // A target that is no node: no declared dependency, and no target key to
    // be the locus, so the clause's key is.
    let mut package = v2_all_families();
    package["semantic_graph"]["nodes"][clause_at]["body"]["arguments"][0] = refer(&"c1".repeat(32));
    settle(&mut package);
    expect_located(
        "an absent over",
        &package,
        (
            Code::MissingDeclaration,
            Cause::MissingName,
            &at_node(clause_at, "/body/arguments/0"),
        ),
        &node_label(&package, clause_at),
    );
}

#[trace("TC-048", "FR-038-AC-103")]
#[test]
fn tc_048_a_fairness_member_resolves_on_its_declaring_model_node() {
    let order = model_key(ORDER_NODE);
    // Own operation, and an operation the node only inherits.
    for (case, declaration, name) in [
        ("an own operation", model_key(ORDER_NODE), "scaled"),
        ("an inherited operation", model_key(SUB_NODE), "scaled"),
    ] {
        let (package, _) = with_fairness(INFINITE, fairness_member(&declaration, name));
        admitted(case, &package);
    }
    // Each row of FR-038's "Path and locus" table: a `name` refusal is on
    // `.../member/name` with the `declaration` target as locus; a refusal of the
    // `declaration` is on `.../member/declaration`, with the fairness node as
    // locus unless the target is what is malformed.
    struct Row {
        case: &'static str,
        declaration: String,
        name: &'static str,
        code: Code,
        cause: Cause,
        member: &'static str,
        locus_is_target: bool,
    }
    let rows = [
        Row {
            case: "a missing name",
            declaration: order.clone(),
            name: "nope",
            code: Code::MissingDeclaration,
            cause: Cause::MissingName,
            member: "name",
            locus_is_target: true,
        },
        Row {
            // A target that names no node is no `model`/`object_type` node
            // (merged FR-370 step 1), located at the key as named.
            case: "an absent declaration",
            declaration: "c2".repeat(32),
            name: "scaled",
            code: Code::InvalidModelBinding,
            cause: Cause::MalformedDeclaration,
            member: "declaration",
            locus_is_target: true,
        },
        Row {
            case: "two matches",
            declaration: model_key(BOTH_NODE),
            name: "act",
            code: Code::AmbiguousDeclaration,
            cause: Cause::AmbiguousName,
            member: "name",
            locus_is_target: true,
        },
    ];
    for row in rows {
        let (package, fair) = with_fairness(INFINITE, fairness_member(&row.declaration, row.name));
        let locus = if row.locus_is_target {
            row.declaration.clone()
        } else {
            node_label(&package, fair)
        };
        expect_located(
            row.case,
            &package,
            (
                row.code,
                row.cause,
                &at_node(
                    fair,
                    &format!("/body/operation/member/target/{}", row.member),
                ),
            ),
            &locus,
        );
    }
    // An unreachable node with an unselected owner is now refused before
    // the fairness member reaches its declaration target (FR-038-AC-155).
    let unselected = model_key_in(OTHER_PACKAGE, ORDER_NODE);
    let (mut package, _) = with_fairness(INFINITE, fairness_member(&unselected, "scaled"));
    let mut node = node_with_key(
        &unselected,
        "model",
        "object_type",
        json!({"term": "aggregate", "members": []}),
    );
    node["semantic_type"] = node_id(&unselected);
    node["occurrences"] = json!([{"role": "type", "ordinal": 0}]);
    let position = push(
        &mut package,
        owned_structural_node(node, model_owner(OTHER_PACKAGE, ORDER_NODE)),
    );
    settle(&mut package);
    expect_located(
        "an unrecovered owner",
        &package,
        (
            Code::MissingDeclaration,
            Cause::MissingSelection,
            &at_node(position, "/node_id"),
        ),
        &unselected,
    );
    // A declaration that is no `model`/`object_type` declaration node is
    // `malformed-declaration` at the member's `declaration` target: a
    // `scalar_type` and a `model`/`value_type` node.
    let value_type = fresh();
    for (case, declaration) in [
        ("a scalar_type declaration", aaaa()),
        ("a model/value_type declaration", value_type.clone()),
    ] {
        let (mut package, fair) = with_fairness(INFINITE, fairness_member(&declaration, "scaled"));
        let target = push(
            &mut package,
            node_with_key(
                &value_type,
                "model",
                "value_type",
                json!({"term": "aggregate", "members": []}),
            ),
        );
        settle(&mut package);
        let declaration = if case == "a model/value_type declaration" {
            node_label(&package, target)
        } else {
            declaration
        };
        // The path sits on the referencing member and the locus is the target
        // (FR-342 step 1): the declaration's own key.
        expect_located(
            case,
            &package,
            (
                Code::InvalidModelBinding,
                Cause::MalformedDeclaration,
                &at_node(fair, "/body/operation/member/target/declaration"),
            ),
            &declaration,
        );
    }
}

/// A selected systems interface is an authentic model declaration, but it
/// cannot declare the operation named by a temporal fairness member. A value
/// node cannot declare one either (FSpec FR-370-AC-11, TC-303 step 9).
#[trace("TC-048", "FR-038-AC-103")]
#[test]
fn tc_048_wrong_declaring_nodes_cannot_name_a_fairness_operation() {
    const INTERFACE: &str = "ix://acme/orders/Channel";
    let mut document = orders_document();
    document["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind": {"module": ORDERS, "name": "interface"},
            "construct": {"meaning": "filament.meaning.systems.interface/v1"},
        }));
    let mut declared = domain_type(INTERFACE, &[], json!([]));
    declared["kind"]["name"] = json!("interface");
    declared["interfaceFeatures"] = json!({});
    document["types"]
        .as_array_mut()
        .expect("types")
        .push(declared);

    let interface_key = format!("{ORDERS}/{INTERFACE}");
    let (mut package, fair) =
        with_fairness(INFINITE, fairness_member(&model_key(ORDER_NODE), "scaled"));
    let mut interface_node = node_with_key(
        &interface_key,
        "model",
        "systems_interface",
        json!({"term": "aggregate", "members": []}),
    );
    interface_node["semantic_type"] = node_id(&interface_key);
    interface_node["occurrences"] = json!([{"role": "type", "ordinal": 0}]);
    push(
        &mut package,
        owned_structural_node(interface_node, model_owner(ORDERS, INTERFACE)),
    );
    settle(&mut package);
    assert!(matches!(
        read_with_model(&package, &document, CheckedPackageReadLimits::bounded()),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    package["semantic_graph"]["nodes"][fair]["body"]["operation"]["member"]["target"]
        ["declaration"] = node_id(&interface_key);
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with_model(&package, &document, CheckedPackageReadLimits::bounded())
    else {
        panic!("a selected systems interface cannot declare a fairness operation");
    };
    assert_eq!(refusal.code, Code::InvalidModelBinding);
    assert_eq!(refusal.cause, Some(Cause::MalformedDeclaration));
    assert_eq!(
        refusal.loci[0].path.clone().expect("path").as_str(),
        at_node(fair, "/body/operation/member/target/declaration")
    );
    assert_eq!(
        serde_json::to_value(refusal.locus.expect("locus")).expect("locus"),
        node_id(&interface_key)
    );

    let base = v2_all_families();
    let value_key = node_label(&base, find(&base, |node| node["node_tag"] == "value"));
    let (package, fair) = with_fairness(INFINITE, fairness_member(&value_key, "scaled"));
    expect_located(
        "a value node cannot declare an operation",
        &package,
        (
            Code::InvalidModelBinding,
            Cause::MalformedDeclaration,
            &at_node(fair, "/body/operation/member/target/declaration"),
        ),
        &value_key,
    );
}

#[trace("TC-048", "FR-038-AC-103")]
#[test]
fn fairness_resolves_the_most_derived_operation_redefinition() {
    for redefined in [true, false] {
        let mut document = orders_document();
        for (position, owner) in [(1, SUB_NODE), (4, BOTH_NODE)] {
            let mut operation = json!({"identity": format!("{owner}/scaled"), "params": []});
            if redefined {
                operation["redefines"] = json!(format!("{ORDER_NODE}/scaled"));
            }
            document["types"][position]["operations"] = json!([operation]);
        }
        document["types"][4]["supertypes"] = json!([SUB_NODE]);
        let declaration = model_key(BOTH_NODE);
        let (package, _) = with_fairness(INFINITE, fairness_member(&declaration, "scaled"));
        let result = read_with_model(&package, &document, CheckedPackageReadLimits::bounded());
        if redefined {
            assert!(
                matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
                "most-derived redefinition must admit: {result:?}"
            );
        } else {
            let CheckedPackageV2ReadResult::Refused(refusal) = result else {
                panic!("unredefined operations must remain ambiguous: {result:?}");
            };
            assert_eq!(refusal.code, Code::AmbiguousDeclaration);
            assert_eq!(refusal.cause, Some(Cause::AmbiguousName));
            let fair = find_identity(&package, &temporal_identity("fair"));
            assert_eq!(
                refusal.loci[0].path.clone().map(|path| path.to_string()),
                Some(at_node(fair, "/body/operation/member/target/name"))
            );
            assert_eq!(
                refusal
                    .locus
                    .map(|locus| serde_json::to_value(locus).expect("locus")),
                Some(node_id(&declaration))
            );
        }
    }
}

// ---------------------------------------------------------------------------
// AC-104
// ---------------------------------------------------------------------------

#[trace("TC-048", "FR-038-AC-104")]
#[test]
fn tc_048_the_profile_fit_admits_each_interval_shape_per_profile() {
    let shapes = [
        ("closed", closed("0", "3")),
        ("lower-bounded", lower_bounded("2")),
        ("null", unbounded()),
    ];
    for (profile, bounded, finite_upper) in [
        (BOUNDED, true, false),
        (
            "filament.temporal.fixed-sample.false-extension/v1",
            true,
            false,
        ),
        (
            "filament.temporal.timestamped-event.finite-window/v1",
            true,
            false,
        ),
        (INFINITE, false, false),
        (TIMED, false, true),
    ] {
        for (case, member) in &shapes {
            let (package, root) = with_root(profile, "eventually", member.clone(), None, 1);
            let refuses = match *case {
                // Under `timed/v1` only `null` admits: an integer-form interval
                // is not the timed form (merged FR-370-AC-3).
                "closed" => finite_upper,
                "lower-bounded" => bounded || finite_upper,
                _ => bounded,
            };
            if refuses {
                expect(
                    &format!("{case} under {profile}"),
                    &package,
                    Code::InvalidPackage,
                    Cause::OperationMemberMismatch,
                    &at_node(root, "/body"),
                );
            } else {
                admitted(&format!("{case} under {profile}"), &package);
            }
        }
        // A non-empty fairness argument: refused only by the bounded profiles,
        // at the clause's application; an empty one admits.
        let (mut package, _) =
            with_fairness(profile, fairness_member(&model_key(ORDER_NODE), "scaled"));
        if finite_upper {
            // The timed profile admits a `null` interval, which the fairness
            // case then holds.
            let root = find_identity(&package, &temporal_identity("eventually"));
            package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"] = unbounded();
            settle(&mut package);
        }
        if bounded {
            expect(
                &format!("fairness under {profile}"),
                &package,
                Code::InvalidPackage,
                Cause::OperationMemberMismatch,
                &at_node(clause(&package), "/body"),
            );
        } else {
            admitted(&format!("fairness under {profile}"), &package);
        }
    }
}

#[trace("TC-048", "FR-038-AC-104")]
#[test]
fn tc_048_finite_window_refuses_timed_and_open_interval_members() {
    const FINITE_WINDOW: &str = "filament.temporal.timestamped-event.finite-window/v1";
    let (baseline, _) = with_root(FINITE_WINDOW, "eventually", closed("0", "3"), None, 1);
    admitted(
        "a closed interval under the finite-window profile",
        &baseline,
    );

    for (case, member, pointer) in [
        (
            "a timed_interval member",
            json!({"kind": "timed_interval", "interval": {"lower": "0", "upper": "3"}}),
            "/body/operation/member",
        ),
        (
            "an open temporal_interval member",
            json!({"kind": "temporal_interval", "interval": {"lower": "0", "upper": null}}),
            "/body",
        ),
    ] {
        let (package, root) = with_root(FINITE_WINDOW, "eventually", member, None, 1);
        expect(
            case,
            &package,
            Code::InvalidPackage,
            Cause::OperationMemberMismatch,
            &at_node(root, pointer),
        );
    }
}

/// The profile fit reads only the formula nodes reachable from the clause: a
/// second clause whose tree shares no node with the first is fitted against its
/// own profile law and tree, not the first's.
#[trace("TC-048", "FR-038-AC-104")]
#[test]
fn tc_048_the_profile_fit_reads_only_the_formula_nodes_reachable_from_the_clause() {
    let mut package = v2_all_families();
    select_profile(&mut package, BOUNDED);
    // An unreachable formula node with a null interval fits no clause.
    let holds = node_label(
        &package,
        find_identity(&package, &temporal_identity("holds")),
    );
    push(
        &mut package,
        formula("always", unbounded(), vec![refer(&holds)]),
    );
    second_clause(&mut package, "second", closed("1", "2"), false);
    settle(&mut package);
    admitted("an unreachable null interval and a second tree", &package);
}

// ---------------------------------------------------------------------------
// AC-105
// ---------------------------------------------------------------------------

#[trace("TC-048", "FR-038-AC-105")]
#[test]
fn tc_048_the_temporal_operands_are_checked_at_the_operation_step() {
    let boolean = family_key("a6a6");
    let holds =
        |package: &Value| node_label(package, find_identity(package, &temporal_identity("holds")));
    let ineligible = |case: &str, package: &Value, root: usize, tail: &str| {
        expect(
            case,
            package,
            Code::IllTyped,
            Cause::OperatorIneligible,
            &at_node(root, tail),
        );
    };
    // `holds` over a reference to a formula node refuses; over a Boolean-family
    // reference it admits.
    let base = v2_all_families();
    let formula_key = holds(&base);
    let (package, root) = with_root(
        BOUNDED,
        "holds",
        Value::Null,
        Some(vec![refer(&formula_key)]),
        0,
    );
    ineligible("holds over a formula", &package, root, "/body/arguments/0");
    let (package, _) = with_root(
        BOUNDED,
        "holds",
        Value::Null,
        Some(vec![refer(&boolean)]),
        0,
    );
    admitted("holds over a Boolean reference", &package);
    // Wrong operand counts.
    let (package, root) = with_root(
        BOUNDED,
        "until",
        closed("0", "3"),
        Some(vec![refer(&formula_key)]),
        0,
    );
    ineligible("until with one argument", &package, root, "/body/arguments");
    let (package, root) = with_root(
        BOUNDED,
        "not",
        Value::Null,
        Some(vec![refer(&formula_key), refer(&formula_key)]),
        0,
    );
    ineligible("not with two", &package, root, "/body/arguments");
    let (package, root) = with_root(
        BOUNDED,
        "true",
        Value::Null,
        Some(vec![refer(&formula_key)]),
        0,
    );
    ineligible("true with one", &package, root, "/body/arguments");
    // `and` over a Boolean-family reference refuses; `not` over a formula
    // reference admits.
    let (package, root) = with_root(
        BOUNDED,
        "and",
        Value::Null,
        Some(vec![refer(&boolean), refer(&formula_key)]),
        0,
    );
    ineligible("and over a Boolean", &package, root, "/body/arguments/0");
    let (package, _) = with_root(
        BOUNDED,
        "not",
        Value::Null,
        Some(vec![refer(&formula_key)]),
        0,
    );
    admitted("not over a formula", &package);
}

// ---------------------------------------------------------------------------
// AC-108
// ---------------------------------------------------------------------------

#[trace("TC-048", "FR-038-AC-108")]
#[test]
fn tc_048_each_of_the_five_fr_250_profiles_admits_as_the_clause_profile() {
    for profile in FIVE_PROFILES {
        let mut package = v2_all_families();
        select_profile(&mut package, profile);
        if profile == TIMED {
            // Under `timed/v1` only a `null` interval admits (merged FR-370-AC-3):
            // the fixture's integer-form `{0, 3}` does not (AC-104).
            let root = find_identity(&package, &temporal_identity("eventually"));
            package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"] = unbounded();
        }
        settle(&mut package);
        admitted(profile, &package);
    }
}

/// The fixture with the clause's own law naming `identity` (the lock still
/// selects the fixture's profile), and `lock_row` added to the lock's profile
/// selections when given. The package and the clause's position.
pub(super) fn clause_law(identity: &str, lock_row: Option<(&str, &str)>) -> (Value, usize) {
    let mut package = v2_all_families();
    let clause_at = clause(&package);
    package["semantic_graph"]["nodes"][clause_at]["body"]["operation"]["laws"][0]["definition"]
        ["identity"] = json!(identity);
    if let Some((role, definition)) = lock_row {
        package["lock"]["profile_selections"]
            .as_array_mut()
            .expect("selections")
            .push(json!({"role": role,
                         "definition": {"authority": "agent-ix", "identity": definition}}));
    }
    settle(&mut package);
    (package, clause_at)
}

#[trace("TC-048", "FR-038-AC-108")]
#[test]
fn tc_048_an_identity_outside_the_five_is_an_unknown_profile() {
    let unknown = |case: &str, identity: &str, row: Option<(&str, &str)>, cause: Cause| {
        let (package, clause_at) = clause_law(identity, row);
        expect(
            case,
            &package,
            Code::UnknownProfile,
            cause,
            &at_node(clause_at, "/body/operation/laws/0/definition"),
        );
    };
    unknown(
        "the fixture profile",
        "filament.fixture.temporal-profile/v1",
        None,
        Cause::UnsupportedSelection,
    );
    unknown("an empty identity", "", None, Cause::UnsupportedSelection);
    unknown(
        "a differently spelled identity",
        "filament.temporal.infinite-trace/v2",
        None,
        Cause::UnsupportedSelection,
    );
    // The same identities, selected by the package's own lock under another
    // role, are the wrong role; selected under none, unsupported.
    unknown(
        "composed, selected as a binding contract",
        "filament.package.composed/v1",
        Some(("binding_contract", "filament.package.composed/v1")),
        Cause::WrongSelectionRole,
    );
    unknown(
        "complete, selected as a protocol profile",
        "filament.protocol.complete/v1",
        Some(("protocol_profile", "filament.protocol.complete/v1")),
        Cause::WrongSelectionRole,
    );
    unknown(
        "composed, selected nowhere else",
        "filament.package.composed/v1",
        None,
        Cause::UnsupportedSelection,
    );
    unknown(
        "complete, selected nowhere else",
        "filament.protocol.complete/v1",
        None,
        Cause::UnsupportedSelection,
    );
}

/// The cause rule matches the same `{authority, identity}`: a law naming the
/// lock's own `edition` definition is the wrong role, and the same identity
/// under another authority, in the edition row or in a profile row, is not.
#[trace("TC-048", "FR-038-AC-108")]
#[test]
fn tc_048_the_cause_matches_the_edition_row_and_the_authority_of_a_definition() {
    let edition = |package: &Value| {
        package["lock"]["edition"]["definition"]["identity"]
            .as_str()
            .expect("the edition identity")
            .to_owned()
    };
    let at_law = |clause_at: usize| at_node(clause_at, "/body/operation/laws/0/definition");

    // The law names the edition definition: the wrong role.
    let (package, clause_at) = clause_law(&edition(&v2_all_families()), None);
    expect(
        "the edition definition",
        &package,
        Code::UnknownProfile,
        Cause::WrongSelectionRole,
        &at_law(clause_at),
    );

    // The edition row carries the identity under another authority: not the same
    // definition, so not the wrong role.
    let (mut package, clause_at) = clause_law(&edition(&v2_all_families()), None);
    package["lock"]["edition"]["definition"]["authority"] = json!("other");
    settle(&mut package);
    expect(
        "the edition identity under another authority",
        &package,
        Code::UnknownProfile,
        Cause::UnsupportedSelection,
        &at_law(clause_at),
    );

    // A profile row of another role carries the identity under another authority.
    let (mut package, clause_at) = clause_law("filament.package.composed/v1", None);
    package["lock"]["profile_selections"]
        .as_array_mut()
        .expect("selections")
        .push(json!({"role": "binding_contract",
                     "definition": {"authority": "other",
                                    "identity": "filament.package.composed/v1"}}));
    settle(&mut package);
    expect(
        "a profile row under another authority",
        &package,
        Code::UnknownProfile,
        Cause::UnsupportedSelection,
        &at_law(clause_at),
    );
}

/// An unknown profile is read first among a clause's checks and is never read
/// as bounded: a `null` interval under it refuses `unknown_profile`, ahead of
/// the clause's own `over` defect.
#[trace("TC-048", "FR-038-AC-108")]
#[test]
fn tc_048_an_unknown_profile_is_checked_first_and_is_not_read_as_bounded() {
    let mut package = v2_all_families();
    let root = find_identity(&package, &temporal_identity("eventually"));
    package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"] = unbounded();
    let clause_at = clause(&package);
    package["semantic_graph"]["nodes"][clause_at]["body"]["operation"]["laws"][0]["definition"]
        ["identity"] = json!("filament.fixture.temporal-profile/v1");
    package["semantic_graph"]["nodes"][clause_at]["body"]["arguments"][0] = refer(&aaaa());
    settle(&mut package);
    expect(
        "unknown profile with a null interval and an over defect",
        &package,
        Code::UnknownProfile,
        Cause::UnsupportedSelection,
        &at_node(clause_at, "/body/operation/laws/0/definition"),
    );
}

/// A typed clock row must not preempt an unknown temporal profile's definition
/// refusal. The producer captured the row under the original known profile,
/// so keep that row after changing the profile identity.
#[trace("TC-048", "FR-038-AC-108")]
#[test]
fn tc_048_unknown_profile_precedes_temporal_binding_profile_join() {
    let mut package = v2_all_families();
    let mut binding = package["temporal_bindings"][0].clone();
    let clause_at = clause(&package);
    package["semantic_graph"]["nodes"][clause_at]["body"]["operation"]["laws"][0]["definition"]
        ["identity"] = json!("filament.fixture.temporal-profile/v1");
    settle(&mut package);

    let current_clause = clause(&package);
    binding["declaration"] = package["semantic_graph"]["nodes"][current_clause]["node_id"].clone();
    package["temporal_bindings"] = json!([binding]);
    expect(
        "an unknown profile with its captured clock binding",
        &package,
        Code::UnknownProfile,
        Cause::UnsupportedSelection,
        &at_node(current_clause, "/body/operation/laws/0/definition"),
    );
}

/// A clause whose `laws` is not exactly one `temporal_profile` law skips the
/// profile check and its profile fit: its law defect refuses at the operation
/// step, at the first law past the catalogued one (`operation-law-mismatch`,
/// `laws/1`) or at the list (`operation-law-missing`).
#[trace("TC-048", "FR-038-AC-108")]
#[test]
fn tc_048_a_clause_without_exactly_one_profile_law_skips_the_profile_check() {
    let laws_at =
        |clause_at: usize, tail: &str| at_node(clause_at, &format!("/body/operation/laws{tail}"));

    // An unknown first law and the fixture's own law: not `unknown_profile`.
    let mut package = v2_all_families();
    let clause_at = clause(&package);
    let own = package["semantic_graph"]["nodes"][clause_at]["body"]["operation"]["laws"][0].clone();
    let mut unknown = own.clone();
    unknown["definition"]["identity"] = json!("filament.fixture.temporal-profile/v1");
    package["semantic_graph"]["nodes"][clause_at]["body"]["operation"]["laws"] =
        json!([unknown, own.clone()]);
    settle(&mut package);
    expect(
        "an unknown first law and a second law",
        &package,
        Code::InvalidPackage,
        Cause::OperationLawMismatch,
        &laws_at(clause_at, "/1"),
    );

    // A known profile twice and a `null` interval, which the bounded profile
    // would refuse as a profile fit: not `operation-member-mismatch`.
    let mut package = v2_all_families();
    let root = find_identity(&package, &temporal_identity("eventually"));
    package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"] = unbounded();
    let clause_at = clause(&package);
    package["semantic_graph"]["nodes"][clause_at]["body"]["operation"]["laws"] =
        json!([own.clone(), own]);
    settle(&mut package);
    expect(
        "the bounded profile twice and a null interval",
        &package,
        Code::InvalidPackage,
        Cause::OperationLawMismatch,
        &laws_at(clause_at, "/1"),
    );

    // No law at all, under a `null` interval the lock's bounded profile would
    // refuse as a profile fit: the lawless clause has no profile to fit.
    let mut package = v2_all_families();
    let root = find_identity(&package, &temporal_identity("eventually"));
    package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"] = unbounded();
    let clause_at = clause(&package);
    package["semantic_graph"]["nodes"][clause_at]["body"]["operation"]["laws"] = json!([]);
    settle(&mut package);
    expect(
        "no law",
        &package,
        Code::InvalidPackage,
        Cause::OperationLawMissing,
        &laws_at(clause_at, ""),
    );
}

/// Ahead of any defect of a higher-key clause: the lower-key clause is
/// reported, whether it holds the unknown profile or an `over` defect.
#[trace("TC-048", "FR-038-AC-108", "FR-038-AC-102")]
#[test]
fn tc_048_the_lower_key_clause_is_read_first_whatever_its_defect() {
    for own_is_unknown in [true, false] {
        for lower_is_own in [true, false] {
            let mut package = v2_all_families();
            // The added clause holds the `over` defect when the fixture's
            // own clause holds the unknown profile, and the reverse.
            let added = second_clause(&mut package, "other", closed("0", "3"), own_is_unknown);
            let own = clause(&package);
            if own_is_unknown {
                package["semantic_graph"]["nodes"][own]["body"]["operation"]["laws"][0]
                    ["definition"]["identity"] = json!("filament.fixture.temporal-profile/v1");
            } else {
                package["semantic_graph"]["nodes"][added]["body"]["operation"]["laws"][0]
                    ["definition"]["identity"] = json!("filament.fixture.temporal-profile/v1");
                package["semantic_graph"]["nodes"][own]["body"]["arguments"][0] = refer(&aaaa());
            }
            if lower_is_own {
                order_pair(&mut package, own, added);
            } else {
                order_pair(&mut package, added, own);
            }
            let (lower, lower_is_unknown) = if lower_is_own {
                (own, own_is_unknown)
            } else {
                (added, !own_is_unknown)
            };
            let expected = if lower_is_unknown {
                (
                    Code::UnknownProfile,
                    Cause::UnsupportedSelection,
                    "/body/operation/laws/0/definition",
                )
            } else {
                (
                    Code::InvalidModelBinding,
                    Cause::MalformedDeclaration,
                    "/body/arguments/0",
                )
            };
            // An unknown profile names the clause; a malformed `over` names
            // its scalar target.
            let locus = if lower_is_unknown {
                node_label(&package, lower)
            } else {
                aaaa()
            };
            expect_located(
                &format!("own unknown {own_is_unknown}, own lower {lower_is_own}"),
                &package,
                (expected.0, expected.1, &at_node(lower, expected.2)),
                &locus,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// AC-119 through AC-122: the timed interval form
// ---------------------------------------------------------------------------

fn rational(numerator: &str, denominator: &str) -> Value {
    json!({"numerator": numerator, "denominator": denominator})
}

/// The whole number `n` as a rational.
fn whole(n: &str) -> Value {
    rational(n, "1")
}

/// A timed-form `temporal_interval` member.
fn timed(lower: Value, upper: Value, lower_end: &str, upper_end: &str) -> Value {
    json!({"kind": "temporal_interval", "interval": {
        "lower": lower, "upper": upper, "lower_end": lower_end, "upper_end": upper_end}})
}

/// A timed-form member with both ends closed.
fn timed_closed(lower: Value, upper: Value) -> Value {
    timed(lower, upper, "closed", "closed")
}

/// Asserts an `invalid_semantic_graph` refusal (it names no cause) at `path`,
/// located at the key of the node the pointer is on.
fn expect_graph(case: &str, package: &Value, path: &str) {
    let position: usize = path
        .strip_prefix("/semantic_graph/nodes/")
        .and_then(|rest| rest.split('/').next())
        .and_then(|node| node.parse().ok())
        .unwrap_or_else(|| panic!("{case}: {path} is on no node"));
    assert_eq!(
        refusal_of(case, package),
        (
            (Code::InvalidSemanticGraph, None, Some(path.to_owned())),
            Some(
                serde_json::from_value(nodes(package)[position]["node_id"].clone())
                    .expect("node ID"),
            )
        ),
        "{case}"
    );
}

#[trace("TC-048", "FR-038-AC-119")]
#[test]
fn tc_048_the_timed_form_admits_under_the_timed_profile_on_each_interval_operator() {
    let end_pairs = [
        ("closed", "closed"),
        ("closed", "open"),
        ("open", "closed"),
        ("open", "open"),
    ];
    let mut admits = Vec::new();
    for (lower_end, upper_end) in end_pairs {
        admits.push((
            format!("[0, 3] with ends {lower_end}/{upper_end}"),
            timed(whole("0"), whole("3"), lower_end, upper_end),
        ));
    }
    admits.push((
        "lower 1/2".to_owned(),
        timed_closed(rational("1", "2"), whole("3")),
    ));
    admits.push((
        "a punctual [3, 3]".to_owned(),
        timed_closed(whole("3"), whole("3")),
    ));
    admits.push(("a null interval".to_owned(), unbounded()));
    for (short, arity) in INTERVAL_OPERATORS {
        for (case, member) in &admits {
            let (package, _) = with_root(TIMED, short, member.clone(), None, arity);
            admitted(&format!("{short} {case}"), &package);
        }
    }
}

#[trace("TC-048", "FR-038-AC-120")]
#[test]
fn tc_048_a_timed_bound_outside_the_rational_pattern_is_invalid_value_at_the_bound() {
    let cases = [
        (
            "lower numerator -1",
            timed_closed(rational("-1", "1"), whole("3")),
            "lower",
        ),
        (
            "lower numerator 01",
            timed_closed(rational("01", "1"), whole("3")),
            "lower",
        ),
        (
            "upper denominator 0",
            timed_closed(whole("0"), rational("3", "0")),
            "upper",
        ),
        (
            "upper denominator -2",
            timed_closed(whole("0"), rational("3", "-2")),
            "upper",
        ),
        (
            "lower a JSON integer",
            timed_closed(json!(0), whole("3")),
            "lower",
        ),
        (
            "lower the string 3",
            timed_closed(json!("3"), whole("3")),
            "lower",
        ),
        (
            "upper null with four members",
            timed_closed(whole("0"), Value::Null),
            "upper",
        ),
        (
            "lower numerator -1 and upper denominator 0",
            timed_closed(rational("-1", "1"), rational("3", "0")),
            "lower",
        ),
        (
            "a rational holding a third member",
            timed_closed(
                json!({"numerator": "0", "denominator": "1", "extra": 1}),
                whole("3"),
            ),
            "lower",
        ),
    ];
    for profile in FIVE_PROFILES {
        for (case, member, bound) in &cases {
            let (package, root) = with_root(profile, "eventually", member.clone(), None, 1);
            expect(
                &format!("{case} under {profile}"),
                &package,
                Code::InvalidPackage,
                Cause::InvalidValue,
                &at_node(root, &format!("/body/operation/member/interval/{bound}")),
            );
        }
    }
    // On every interval operator, not only `eventually`.
    for (short, arity) in INTERVAL_OPERATORS {
        let member = timed_closed(rational("-1", "1"), whole("3"));
        let (package, root) = with_root(TIMED, short, member, None, arity);
        expect(
            short,
            &package,
            Code::InvalidPackage,
            Cause::InvalidValue,
            &at_node(root, "/body/operation/member/interval/lower"),
        );
    }
}

/// A pattern failure in any node is refused ahead of a non-reduced bound in any
/// node, whichever comes first in node order.
#[trace("TC-048", "FR-038-AC-120")]
#[test]
fn tc_048_a_pattern_failure_in_a_later_node_precedes_a_non_reduced_bound() {
    let non_reduced = timed_closed(rational("2", "4"), whole("3"));
    let pattern_failure = timed_closed(whole("0"), rational("3", "0"));
    for reduced_node_first in [true, false] {
        let (first, second) = if reduced_node_first {
            (non_reduced.clone(), pattern_failure.clone())
        } else {
            (pattern_failure.clone(), non_reduced.clone())
        };
        let (mut package, _) = with_root(INFINITE, "eventually", first, None, 1);
        let holds = node_label(
            &package,
            find_identity(&package, &temporal_identity("holds")),
        );
        let added = push(&mut package, formula("once", second, vec![refer(&holds)]));
        settle(&mut package);
        // The arrangement is by node position, first and later node.
        let root = find_identity(&package, &temporal_identity("eventually"));
        assert!(root < added, "the pushed node is the later one");
        let (at, bound) = if reduced_node_first {
            (added, "upper")
        } else {
            (root, "upper")
        };
        expect(
            &format!("non-reduced node first: {reduced_node_first}"),
            &package,
            Code::InvalidPackage,
            Cause::InvalidValue,
            &at_node(at, &format!("/body/operation/member/interval/{bound}")),
        );
    }
}

/// The smallest work limit at which `package` is decided rather than
/// `incomplete`.
fn smallest_deciding_work(package: &Value) -> u64 {
    let incomplete = |work: u64| {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = work;
        matches!(
            read_limited(package, limits),
            CheckedPackageV2ReadResult::Incomplete(_)
        )
    };
    let (mut low, mut high) = (0_u64, CheckedPackageReadLimits::bounded().work);
    assert!(incomplete(low), "a zero work limit is incomplete");
    assert!(!incomplete(high), "the bounded limit decides the package");
    while low + 1 < high {
        let mid = low + (high - low) / 2;
        if incomplete(mid) {
            low = mid;
        } else {
            high = mid;
        }
    }
    high
}

/// `package` read with `work`, which must be `incomplete` naming the work limit.
fn incomplete_work_path(package: &Value, work: u64) -> (u64, Option<String>) {
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = work;
    match read_limited(package, limits) {
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
            assert_eq!(incomplete.limit, work);
            (
                incomplete.consumed,
                incomplete.path.map(|path| path.as_str().to_owned()),
            )
        }
        other => panic!("expected incomplete at work {work}, read {other:?}"),
    }
}

#[trace("TC-048", "FR-038-AC-121")]
#[test]
fn tc_048_a_bound_not_in_lowest_terms_is_refused_at_the_bound_before_the_temporal_step() {
    let (package, root) = with_root(
        TIMED,
        "eventually",
        timed_closed(rational("2", "4"), whole("3")),
        None,
        1,
    );
    expect_graph(
        "a non-reduced lower",
        &package,
        &at_node(root, "/body/operation/member/interval/lower"),
    );
    let (package, root) = with_root(
        TIMED,
        "eventually",
        timed_closed(whole("0"), rational("2", "4")),
        None,
        1,
    );
    expect_graph(
        "a non-reduced upper alone",
        &package,
        &at_node(root, "/body/operation/member/interval/upper"),
    );
    // A pattern failure at the other bound is refused first, `invalid-value`.
    let (package, root) = with_root(
        TIMED,
        "eventually",
        timed_closed(rational("2", "4"), rational("-1", "1")),
        None,
        1,
    );
    expect(
        "non-reduced lower, negative upper",
        &package,
        Code::InvalidPackage,
        Cause::InvalidValue,
        &at_node(root, "/body/operation/member/interval/upper"),
    );
    let (package, root) = with_root(
        TIMED,
        "eventually",
        timed_closed(rational("-1", "1"), rational("2", "4")),
        None,
        1,
    );
    expect(
        "negative lower, non-reduced upper",
        &package,
        Code::InvalidPackage,
        Cause::InvalidValue,
        &at_node(root, "/body/operation/member/interval/lower"),
    );

    // Beside a profile-fit defect (an integer-form interval under `timed/v1`)
    // in another node, the non-reduced bound is refused: the graph check is
    // ahead of the temporal step.
    let beside_fit_defect = |member: Value, fit_upper: u32| {
        let mut package = v2_all_families();
        select_profile(&mut package, TIMED);
        let holds = node_label(
            &package,
            find_identity(&package, &temporal_identity("holds")),
        );
        let fit = push(
            &mut package,
            formula(
                "always",
                closed("0", &fit_upper.to_string()),
                vec![refer(&holds)],
            ),
        );
        let root = find_identity(&package, &temporal_identity("eventually"));
        let operand = vec![refer(&node_label(&package, fit))];
        package["semantic_graph"]["nodes"][root]["body"] =
            formula_body("eventually", member, operand);
        settle(&mut package);
        (package, root, fit)
    };
    // Authored IDs put either defect first without changing the two bodies.
    for fit_is_lower in [true, false] {
        let (mut package, root, fit) =
            beside_fit_defect(timed_closed(rational("2", "4"), whole("3")), 1);
        if fit_is_lower {
            order_pair(&mut package, fit, root);
        } else {
            order_pair(&mut package, root, fit);
        }
        expect_graph(
            &format!("a non-reduced bound beside a profile-fit defect, fit lower: {fit_is_lower}"),
            &package,
            &at_node(root, "/body/operation/member/interval/lower"),
        );
    }
    let (package, _, fit) = beside_fit_defect(timed_closed(whole("1"), whole("3")), 3);
    expect(
        "the same fit defect with reduced bounds",
        &package,
        Code::InvalidPackage,
        Cause::OperationMemberMismatch,
        &at_node(fit, "/body"),
    );
}

#[trace("TC-048", "FR-038-AC-121")]
#[test]
fn tc_048_the_timed_bounds_compare_exactly_and_open_ends_refuse_equal_bounds() {
    let refuses = |case: &str, member: Value| {
        let (package, root) = with_root(TIMED, "eventually", member, None, 1);
        expect(
            case,
            &package,
            Code::InvalidPackage,
            Cause::InvalidValue,
            &at_node(root, "/body"),
        );
    };
    let admits = |case: &str, member: Value| {
        let (package, _) = with_root(TIMED, "eventually", member, None, 1);
        admitted(case, &package);
    };
    refuses("(3, 3]", timed(whole("3"), whole("3"), "open", "closed"));
    refuses("[3, 3)", timed(whole("3"), whole("3"), "closed", "open"));
    refuses("(3, 3)", timed(whole("3"), whole("3"), "open", "open"));
    refuses("5/2 over 2", timed_closed(rational("5", "2"), whole("2")));
    admits("[3, 3]", timed_closed(whole("3"), whole("3")));
    admits(
        "1/2 to 2/3",
        timed_closed(rational("1", "2"), rational("2", "3")),
    );
    refuses(
        "2/3 to 1/2",
        timed_closed(rational("2", "3"), rational("1", "2")),
    );
    // Open ends do not matter once the bounds differ.
    admits(
        "(1/2, 2/3)",
        timed(rational("1", "2"), rational("2", "3"), "open", "open"),
    );
    refuses(
        "(2/3, 1/2)",
        timed(rational("2", "3"), rational("1", "2"), "open", "open"),
    );
    // Beyond 2^64: a float comparison rounds both numerators to 2^64 and
    // admits the first; a checked parse into 64 bits refuses the second.
    let above = rational("18446744073709551617", "3");
    let below = rational("18446744073709551616", "3");
    refuses(
        "2^64+1 over 2^64, both over 3",
        timed_closed(above.clone(), below.clone()),
    );
    admits("2^64 over 2^64+1, both over 3", timed_closed(below, above));
    // Different denominators cross-multiply: 7/3 against 5/2.
    admits(
        "7/3 to 5/2",
        timed_closed(rational("7", "3"), rational("5", "2")),
    );
    refuses(
        "5/2 to 7/3",
        timed_closed(rational("5", "2"), rational("7", "3")),
    );
}

/// The work limit covers the lowest-terms check and the exact bounds check: a
/// limit the work goes past is `incomplete` naming `work` at the node's body,
/// and not a refusal of the interval.
#[trace("TC-048", "FR-038-AC-121")]
#[test]
fn tc_048_the_timed_bound_checks_are_charged_to_the_work_limit() {
    let (bounds, root) = with_root(
        TIMED,
        "eventually",
        timed_closed(
            rational("18446744073709551617", "3"),
            rational("18446744073709551616", "3"),
        ),
        None,
        1,
    );
    let body = at_node(root, "/body");
    let needed = smallest_deciding_work(&bounds);
    let mut exact = CheckedPackageReadLimits::bounded();
    exact.work = needed;
    expect(
        "the exact work decides the refusal",
        &bounds,
        Code::InvalidPackage,
        Cause::InvalidValue,
        &body,
    );
    assert!(matches!(
        read_limited(&bounds, exact),
        CheckedPackageV2ReadResult::Refused(_)
    ));
    // One under, the cross-multiplication of the bounds is what the limit stops.
    assert_eq!(
        incomplete_work_path(&bounds, needed - 1),
        (needed, Some(body))
    );
    // The bounds cost more than a pair of small ones of the same shape.
    let (small, _) = with_root(
        TIMED,
        "eventually",
        timed_closed(whole("0"), whole("1")),
        None,
        1,
    );
    assert!(needed > smallest_deciding_work(&small));

    // The GCD of a non-reduced bound is charged at the body too.
    let (non_reduced, root) = with_root(
        TIMED,
        "eventually",
        timed_closed(
            rational("55340232221128654851", "3"),
            whole("99999999999999999999999"),
        ),
        None,
        1,
    );
    let needed = smallest_deciding_work(&non_reduced);
    let mut exact = CheckedPackageReadLimits::bounded();
    exact.work = needed;
    let CheckedPackageV2ReadResult::Refused(refusal) = read_limited(&non_reduced, exact) else {
        panic!("the exact work decides the refusal");
    };
    assert_eq!(refusal.code, Code::InvalidSemanticGraph);
    assert_eq!(
        refusal.loci[0]
            .path
            .as_ref()
            .map(|path| path.as_str().to_owned()),
        Some(at_node(root, "/body/operation/member/interval/lower"))
    );
    assert_eq!(
        incomplete_work_path(&non_reduced, needed - 1),
        (needed, Some(at_node(root, "/body")))
    );
}

#[trace("TC-048", "FR-038-AC-122")]
#[test]
fn tc_048_the_timed_form_fits_only_the_timed_profile() {
    let member = || timed_closed(whole("0"), whole("3"));
    for profile in FIVE_PROFILES {
        let (package, root) = with_root(profile, "eventually", member(), None, 1);
        if profile == TIMED {
            admitted("the timed form under timed/v1", &package);
        } else {
            expect(
                &format!("the timed form under {profile}"),
                &package,
                Code::InvalidPackage,
                Cause::OperationMemberMismatch,
                &at_node(root, "/body"),
            );
        }
    }
}

#[trace("TC-048", "FR-038-AC-122")]
#[test]
fn tc_048_a_timed_form_member_set_defect_is_a_member_mismatch_at_the_member() {
    let interval = |members: Value| json!({"kind": "temporal_interval", "interval": members});
    let mismatches = [
        (
            "lower_end half",
            timed(whole("0"), whole("3"), "half", "closed"),
        ),
        (
            "upper_end half",
            timed(whole("0"), whole("3"), "closed", "half"),
        ),
        (
            "a missing end with integer-string bounds",
            interval(json!({"lower": "0", "upper": "3", "lower_end": "closed"})),
        ),
        (
            "a fifth member with integer-string bounds",
            interval(json!({"lower": "0", "upper": "3", "lower_end": "closed",
                            "upper_end": "closed", "extra": "x"})),
        ),
    ];
    for profile in [INFINITE, TIMED] {
        for (case, member) in &mismatches {
            let (package, root) = with_root(profile, "eventually", member.clone(), None, 1);
            expect(
                &format!("{case} under {profile}"),
                &package,
                Code::InvalidPackage,
                Cause::OperationMemberMismatch,
                &at_node(root, "/body/operation/member"),
            );
        }
        // The same member sets with the timed form's rational-object bounds have
        // no form, so the integer pattern refuses the object at `lower`.
        for (case, member) in [
            (
                "a missing end",
                interval(json!({"lower": whole("0"), "upper": whole("3"),
                                "lower_end": "closed"})),
            ),
            (
                "a fifth member",
                interval(json!({"lower": whole("0"), "upper": whole("3"),
                                "lower_end": "closed", "upper_end": "closed", "extra": "x"})),
            ),
        ] {
            let (package, root) = with_root(profile, "eventually", member, None, 1);
            expect(
                &format!("{case} with rational bounds under {profile}"),
                &package,
                Code::InvalidPackage,
                Cause::InvalidValue,
                &at_node(root, "/body/operation/member/interval/lower"),
            );
        }
    }
}

/// A member-set defect is the operation step's, after the whole temporal step:
/// a profile-fit defect in a higher-key clause is reported first.
#[trace("TC-048", "FR-038-AC-122")]
#[test]
fn tc_048_a_member_set_defect_is_reported_after_a_profile_fit_defect_of_a_later_clause() {
    let mut package = v2_all_families();
    select_profile(&mut package, TIMED);
    let root = find_identity(&package, &temporal_identity("eventually"));
    // The fixture's own clause holds the member-set defect, the added
    // clause the fit defect (an integer-form interval under `timed/v1`).
    package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"] =
        timed(whole("0"), whole("3"), "half", "closed");
    let added = second_clause(&mut package, "other", closed("0", "3"), false);
    let own = clause(&package);
    order_pair(&mut package, own, added);
    let added_root = nodes(&package)
        .iter()
        .rposition(|node| {
            node["body"]["operation"]["identity"] == "filament.op.temporal.eventually"
        })
        .expect("the added clause's formula");
    expect(
        "the fit defect of the higher clause",
        &package,
        Code::InvalidPackage,
        Cause::OperationMemberMismatch,
        &at_node(added_root, "/body"),
    );
}

/// The bound pattern is strict wire validation ahead of graph resolution.
/// An independently invalid semantic type is reported only after a valid
/// bound; an invalid bound always wins at its exact pointer.
#[trace("TC-048", "FR-038-AC-97", "FR-038-AC-120")]
#[test]
fn tc_048_a_bound_outside_the_pattern_is_refused_ahead_of_every_identity_check() {
    let invalid_graph_with = |member: Value| {
        let (mut package, root) = with_root(TIMED, "eventually", closed("0", "3"), None, 1);
        package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"] = member;
        package["semantic_graph"]["nodes"][root]["semantic_type"] =
            node_id("absent/temporal-semantic-type");
        (package, root)
    };
    let (good, root) = invalid_graph_with(timed_closed(whole("0"), whole("3")));
    let ((code, cause, path), _) = refusal_of("an invalid graph with a good bound", &good);
    assert_eq!(
        (code, cause, path),
        (
            Code::InvalidSemanticGraph,
            None,
            Some(at_node(root, "/semantic_type"))
        )
    );
    for (case, member, bound) in [
        (
            "a timed negative numerator",
            timed_closed(rational("-1", "1"), whole("3")),
            "lower",
        ),
        (
            "a timed zero denominator",
            timed_closed(whole("0"), rational("3", "0")),
            "upper",
        ),
        ("an integer bound", closed("1.5", "0"), "lower"),
        ("a negative integer bound", closed("0", "-2"), "upper"),
    ] {
        let (package, root) = invalid_graph_with(member);
        expect(
            case,
            &package,
            Code::InvalidPackage,
            Cause::InvalidValue,
            &at_node(root, &format!("/body/operation/member/interval/{bound}")),
        );
    }
}

/// The lowest-terms check reads nodes in ascending `node_id` key order, not
/// position order: of two non-reduced nodes, the refusal is at the one with the
/// lower key, whichever is first in the package.
#[trace("TC-048", "FR-038-AC-121")]
#[test]
fn tc_048_the_first_non_reduced_bound_is_the_lower_key_nodes() {
    for added_is_lower in [true, false] {
        let (mut package, root) = with_root(
            TIMED,
            "eventually",
            timed_closed(rational("2", "4"), whole("3")),
            None,
            1,
        );
        let holds = node_label(
            &package,
            find_identity(&package, &temporal_identity("holds")),
        );
        let second = timed_closed(whole("0"), rational("2", "4"));
        let added = push(&mut package, formula("once", second, vec![refer(&holds)]));
        assert!(root < added, "the pushed node is the later one");
        if added_is_lower {
            order_pair(&mut package, added, root);
        } else {
            order_pair(&mut package, root, added);
        }
        let (lower, bound) = if added_is_lower {
            (added, "upper")
        } else {
            (root, "lower")
        };
        expect_graph(
            &format!("added node has the lower ID: {added_is_lower}"),
            &package,
            &at_node(lower, &format!("/body/operation/member/interval/{bound}")),
        );
    }
}

// ---------------------------------------------------------------------------
// FR-038-AC-530 (FSpec FR-370-AC-21): fairness references of a hyperproperty
// ---------------------------------------------------------------------------

fn text(value: &str) -> Value {
    json!({"term": "literal", "type": node_id(&family_key("a1a1")),
           "value_kind": "text", "value": value})
}

fn named(name: &str, value: Value) -> Value {
    json!({"term": "binding", "name": name, "value": value})
}

fn group(members: Vec<Value>) -> Value {
    let mut all = vec![
        named("variable", text("a")),
        named("quantifier", text("forall")),
        named("alias", text("m")),
    ];
    all.extend(members);
    json!({"term": "aggregate", "members": all})
}

/// A `claim`/`hyperproperty` node whose seven bindings hold the given
/// `quantifiers` Tuple, `match` and `align_skip` members and `body` term.
fn hyperproperty(
    form: &str,
    quantifiers: Vec<Value>,
    matched: Vec<Value>,
    skip: Vec<Value>,
    body: Value,
) -> Value {
    let aggregate = |members: Vec<Value>| json!({"term": "aggregate", "members": members});
    let members = vec![
        named("domain", text("behaviours")),
        named(
            "subjects",
            aggregate(vec![named(
                "m",
                json!({"term": "reference", "target":
                    {"package": ORDERS, "path": ["model", "model_root"], "ordinal": 0}}),
            )]),
        ),
        named("parameters", aggregate(vec![])),
        named("quantifiers", aggregate(quantifiers)),
        named("match", aggregate(matched)),
        named("align_skip", aggregate(skip)),
        named("body", body),
    ];
    node_with_key(&fresh(), "claim", form, aggregate(members))
}

/// The model-enabled fixture with a fairness node, the clause's formula node
/// and the labels of both.
fn hyper_fixture() -> (Value, String, String) {
    hyper_fixture_with(INFINITE, fairness_member(&model_key(ORDER_NODE), "scaled"))
}

fn hyper_fixture_with(profile: &str, member: Value) -> (Value, String, String) {
    let (mut package, fair) = with_fairness(profile, member);
    let fair_key = node_label(&package, fair);
    let formula_key = node_label(
        &package,
        find_identity(&package, &temporal_identity("eventually")),
    );
    let clause_key = node_label(&package, clause(&package));
    // The clause no longer names the fairness node: only the claim does.
    let clause_at = clause(&package);
    package["semantic_graph"]["nodes"][clause_at]["body"]["arguments"][4] =
        json!({"term": "aggregate", "members": []});
    (package, fair_key, format!("{formula_key}\n{clause_key}"))
}

/// A quantifier Group binding trace variable `variable` over the alias `m`.
fn trace_group(variable: &str) -> Value {
    json!({"term": "aggregate", "members": [
        named("variable", text(variable)),
        named("quantifier", text("forall")),
        named("alias", text("m")),
    ]})
}

/// Gives the hyperproperty node at `at` its one `claim` occurrence.
fn claim_occurrence(package: &mut Value, at: usize) {
    package["semantic_graph"]["nodes"][at]["occurrences"] =
        json!([{"role": "claim", "ordinal": 0}]);
}

/// An event-position package with a hyperproperty quantifying the traces
/// `variables` over the clause as its body wrapper: the clause's and the
/// hyperproperty's graph positions. `settle` writes one row per trace, keyed on
/// the hyperproperty node.
fn hyper_clock_fixture(variables: &[&str]) -> (Value, usize, usize) {
    let (mut package, _, keys) =
        hyper_fixture_with(BOUNDED, fairness_member(&model_key(ORDER_NODE), "scaled"));
    let clause_key = keys.split('\n').nth(1).expect("clause").to_owned();
    let hyper = push(
        &mut package,
        hyperproperty(
            "hyperproperty",
            variables.iter().map(|name| trace_group(name)).collect(),
            vec![],
            vec![],
            refer(&clause_key),
        ),
    );
    claim_occurrence(&mut package, hyper);
    settle(&mut package);
    let wrapper = clause(&package);
    (package, wrapper, hyper)
}

const TIMED_NODE_REFERENCE: Cause = Cause::TimedNodeReference;

/// Asserts the refusal's code, cause and pointer, and that its locus is the node
/// the pointer is on: the row's declaration for a `/temporal_bindings/{i}`
/// pointer, the graph node for a `/semantic_graph/nodes/{N}` one.
fn refusal_at(package: &Value, code: Code, cause: Cause, path: &str) {
    let locus = if let Some(rest) = path.strip_prefix("/temporal_bindings/") {
        let row: usize = rest.split('/').next().unwrap().parse().expect("row");
        package["temporal_bindings"][row]["declaration"].clone()
    } else {
        let node: usize = path
            .strip_prefix("/semantic_graph/nodes/")
            .and_then(|rest| rest.split('/').next())
            .and_then(|node| node.parse().ok())
            .unwrap_or_else(|| panic!("{path} is on no row or node"));
        nodes(package)[node]["node_id"].clone()
    };
    let locus: CheckedNodeId = serde_json::from_value(locus).expect("node ID");
    assert_eq!(
        refusal_of("trace refusal", package),
        ((code, Some(cause), Some(path.to_owned())), Some(locus))
    );
}

fn refusal_at_trace(package: &Value, path: &str) {
    refusal_at(package, Code::InvalidPackage, TIMED_NODE_REFERENCE, path);
}

#[trace("TC-483", "FR-038-AC-561")]
#[test]
fn tc_483_a_hyperproperty_carries_one_row_per_quantified_trace_and_its_wrapper_none() {
    for variables in [["s", "n"], ["a", "b"]] {
        let (package, _, hyper) = hyper_clock_fixture(&variables);
        let admitted = admitted("one row per trace", &package);
        let rows = admitted.temporal_bindings().expect("rows");
        let traces: Vec<_> = rows.iter().map(|row| row.trace.as_deref()).collect();
        assert_eq!(traces, variables.map(Some), "rows ascend by trace position");
        let hyper_id = typed_node_id_of(&package, hyper);
        assert!(rows.iter().all(|row| row.declaration == hyper_id));
        assert_eq!(rows[0].occurrence, rows[1].occurrence);
        assert_eq!(rows[0].clock, rows[1].clock, "two traces of one model");
        assert_eq!(rows[0].clock_name, rows[1].clock_name);
    }
    let (single, _, _) = hyper_clock_fixture(&["s"]);
    admitted("a single trace", &single);
}

fn typed_node_id_of(package: &Value, position: usize) -> CheckedNodeId {
    serde_json::from_value(nodes(package)[position]["node_id"].clone()).expect("node ID")
}

/// FR-322-AC-503: a hyperproperty row carries no `source_unit`.
#[trace("TC-048", "FR-038-AC-590")]
#[test]
fn a_hyperproperty_row_naming_a_source_unit_is_refused() {
    let (mut package, _, _) = hyper_clock_fixture(&["s", "n"]);
    package["temporal_bindings"][0]["source_unit"] = package["lock"]["sources"][0].clone();
    refusal_at_trace(&package, "/temporal_bindings/0");
}

#[trace("TC-483", "FR-038-AC-561")]
#[test]
fn tc_483_each_trace_row_holds_its_own_configuration() {
    let (mut package, _, _) = hyper_clock_fixture(&["s", "n"]);
    package["temporal_bindings"][1]["clock"]["sequence_authority"] = json!("another-model");
    let admitted = admitted("two sequence authorities", &package);
    let rows = admitted.temporal_bindings().expect("rows");
    assert_ne!(rows[0].clock, rows[1].clock);
}

/// A finite-domain hyperproperty over traces `a` and `b` under the event-position
/// profile, with its graph position.
fn finite_clock_fixture() -> (Value, usize) {
    finite_clock_fixture_with(&["a", "b"])
}

fn finite_clock_fixture_with(variables: &[&str]) -> (Value, usize) {
    let (mut package, at) = finite_hyper(|body| {
        body["members"][3]["value"]["members"] = json!(variables
            .iter()
            .map(|variable| json!({"term": "aggregate", "members": [
                named("variable", text(variable)), named("quantifier", text("forall"))]}))
            .collect::<Vec<_>>());
    });
    select_profile(&mut package, BOUNDED);
    claim_occurrence(&mut package, at);
    settle(&mut package);
    (package, at)
}

fn hyper_rows(package: &Value) -> Vec<usize> {
    package["temporal_bindings"]
        .as_array()
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .enumerate()
        .filter(|(_, row)| row.get("trace").is_some())
        .map(|(at, _)| at)
        .collect()
}

#[trace("TC-483", "FR-038-AC-561")]
#[test]
fn tc_483_a_finite_domain_hyperproperty_keeps_its_per_trace_rows() {
    let (package, _) = finite_clock_fixture();
    let admitted = admitted("a finite domain", &package);
    let rows = admitted.temporal_bindings().expect("rows");
    let hyper: Vec<_> = rows.iter().filter(|row| row.trace.is_some()).collect();
    let traces: Vec<_> = hyper
        .iter()
        .filter_map(|row| row.trace.as_deref())
        .collect();
    assert_eq!(traces, ["a", "b"]);
    assert!(hyper.iter().all(|row| &*row.clock_name == "t"));
}

#[trace("TC-483", "FR-038-AC-561")]
#[test]
fn tc_483_a_finite_domain_row_is_refused_at_its_own_defect() {
    let (package, at) = finite_clock_fixture();
    let rows = hyper_rows(&package);
    let mut missing = package.clone();
    missing["temporal_bindings"]
        .as_array_mut()
        .expect("rows")
        .remove(rows[1]);
    refusal_at(
        &missing,
        Code::InvalidPackage,
        TIMED_NODE_REFERENCE,
        &at_node(at, "/body"),
    );
    let mut unknown = package.clone();
    unknown["temporal_bindings"][rows[0]]["trace"] = json!("c");
    refusal_at_trace(&unknown, &format!("/temporal_bindings/{}/trace", rows[0]));
    let mut role = package.clone();
    role["temporal_bindings"][rows[1]]["clock_name"] = json!("u");
    refusal_at_trace(&role, &format!("/temporal_bindings/{}/clock_name", rows[1]));
    let mut kind = package.clone();
    kind["temporal_bindings"][rows[0]]["clock"] = json!({
        "kind": "fixed_sample",
        "epoch": {"kind": "integer", "decimal": "0"},
        "period": {"kind": "integer", "decimal": "1"},
        "unit": "tick",
    });
    refusal_at_trace(&kind, &format!("/temporal_bindings/{}/clock/kind", rows[0]));
}

/// Replace the lock's temporal-role selections with `identities`, in order,
/// where the first stood.
fn select_temporal_profiles(package: &mut Value, identities: &[&str]) {
    let selections = package["lock"]["profile_selections"]
        .as_array_mut()
        .expect("selections");
    let first = selections
        .iter()
        .position(|selection| selection["role"] == "temporal_profile")
        .expect("a temporal selection");
    selections.retain(|selection| selection["role"] != "temporal_profile");
    for (offset, identity) in identities.iter().enumerate() {
        selections.insert(
            first + offset,
            json!({"role": "temporal_profile",
                   "definition": {"authority": "agent-ix", "identity": identity}}),
        );
    }
}

fn body_profile_path(at: usize) -> String {
    at_node(at, "/body/temporal_profile")
}

#[trace("TC-2028", "FR-322-AC-464")]
#[test]
fn tcb_14_a_finite_hyper_body_profile_is_the_one_its_rows_are_checked_against() {
    let (mut package, at) = finite_clock_fixture();
    select_temporal_profiles(&mut package, &[INFINITE, BOUNDED]);
    package["semantic_graph"]["nodes"][at]["body"]["temporal_profile"]["identity"] = json!(BOUNDED);
    settle(&mut package);
    let admitted = admitted("lock [T, D] with body D", &package);
    let rows: Vec<_> = admitted
        .temporal_bindings()
        .expect("rows")
        .iter()
        .filter(|row| row.trace.is_some())
        .collect();
    assert_eq!(rows.len(), 2);
    let hyper_id = typed_node_id_of(&package, at);
    for (row, trace) in rows.iter().zip(["a", "b"]) {
        assert_eq!(row.declaration, hyper_id);
        assert_eq!(row.occurrence, 0);
        assert_eq!(row.trace.as_deref(), Some(trace));
        assert_eq!(&*row.clock_name, "t");
    }
    let Some(filament_contract_model::CheckedTerm::Aggregate {
        temporal_profile: Some(profile),
        ..
    }) = admitted.term(&hyper_id)
    else {
        panic!("the admitted body carries its typed temporal_profile");
    };
    assert_eq!(&*profile.authority, "agent-ix");
    assert_eq!(&*profile.identity, BOUNDED);
    for row in &rows {
        assert_eq!(
            serde_json::to_value(&row.clock).expect("clock"),
            json!({"kind": "event_position", "sequence_authority": "fixture-events"})
        );
    }
}

#[trace("TC-2028", "FR-322-AC-464")]
#[test]
fn a_finite_hyper_body_profile_survives_write_then_read() {
    let (package, at) = finite_clock_fixture();
    let admitted = admitted("a finite hyperproperty", &package);
    let encoded = filament_contract_model::encode_checked_package_v2(
        admitted.name(),
        admitted.lock(),
        admitted.graph(),
        filament_contract_model::CheckedPackageSourceCorrespondenceRef {
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
    .expect("encode admitted package");
    let written: Value = serde_json::from_slice(&encoded).expect("written package");
    assert_eq!(
        written["semantic_graph"]["nodes"][at]["body"]["temporal_profile"],
        json!({"authority": "agent-ix", "identity": BOUNDED})
    );
    assert_eq!(
        encoded,
        canonical(&package),
        "the written bytes are the admitted canonical bytes"
    );
    self::admitted("the written package", &written);
}

#[trace("TC-2028", "FR-322-AC-464")]
#[test]
fn tcb_15_a_finite_hyper_body_profile_no_lock_row_selects_is_refused() {
    let (mut package, at) = finite_clock_fixture();
    // The body names the fixed-sample profile, which no lock row selects; the
    // fixture's clause keeps its own selected profile.
    select_temporal_profiles(&mut package, &[INFINITE, BOUNDED]);
    package["semantic_graph"]["nodes"][at]["body"]["temporal_profile"]["identity"] =
        json!("filament.temporal.fixed-sample.false-extension/v1");
    settle(&mut package);
    refusal_at(
        &package,
        Code::InvalidPackage,
        Cause::OperationLawUnselected,
        &body_profile_path(at),
    );
}

#[trace("TC-2028", "FR-322-AC-464")]
#[test]
fn tcb_16_a_finite_hyper_row_kind_is_refused_against_the_body_profile() {
    let (mut package, _) = finite_clock_fixture();
    select_temporal_profiles(&mut package, &[INFINITE, BOUNDED]);
    settle(&mut package);
    let row = hyper_rows(&package)[0];
    package["temporal_bindings"][row]["clock"] = json!({
        "kind": "fixed_sample",
        "epoch": {"kind": "integer", "decimal": "0"},
        "period": {"kind": "rational", "numerator": "1", "denominator": "10"},
        "unit": "milliseconds",
    });
    refusal_at_trace(&package, &format!("/temporal_bindings/{row}/clock/kind"));
}

#[trace("TC-2028", "FR-322-AC-464")]
#[test]
fn a_finite_hyper_body_profile_must_be_present_and_selected_exactly_once() {
    let (package, at) = finite_clock_fixture();
    let mut absent = package.clone();
    absent["semantic_graph"]["nodes"][at]["body"]
        .as_object_mut()
        .expect("body")
        .remove("temporal_profile");
    settle(&mut absent);
    refusal_at(
        &absent,
        Code::InvalidPackage,
        Cause::OperationLawUnselected,
        &body_profile_path(at),
    );
    let mut twice = package;
    select_temporal_profiles(&mut twice, &[BOUNDED, BOUNDED]);
    settle(&mut twice);
    refusal_at(
        &twice,
        Code::InvalidPackage,
        Cause::OperationLawUnselected,
        &body_profile_path(at),
    );
}

#[trace("TC-2028", "FR-322-AC-464")]
#[test]
fn a_behaviours_hyper_body_carries_no_profile_member_and_other_nodes_may_not() {
    let (package, _, hyper) = hyper_clock_fixture(&["s", "n"]);
    let member = json!({"authority": "agent-ix", "identity": BOUNDED});
    let mut behaviours = package.clone();
    behaviours["semantic_graph"]["nodes"][hyper]["body"]["temporal_profile"] = member.clone();
    refusal_at(
        &behaviours,
        Code::InvalidPackage,
        Cause::InvalidValue,
        &body_profile_path(hyper),
    );
    // The guard names the body, whatever shape the node's body has.
    let malformed_wire = |case: &str, package: &Value, at: usize| {
        assert_eq!(
            refusal_of(case, package).0,
            (Code::MalformedWire, None, Some(at_node(at, "/body")))
        );
    };
    let mut application = package.clone();
    let clause_at = clause(&application);
    application["semantic_graph"]["nodes"][clause_at]["body"]["temporal_profile"] = member.clone();
    malformed_wire("a profile member on a clause", &application, clause_at);
    let mut aggregate = package;
    let record = nodes(&aggregate)
        .iter()
        .position(|node| {
            node["semantic_form"] != "hyperproperty" && node["body"]["term"] == "aggregate"
        })
        .expect("an aggregate-bodied node");
    assert_eq!(
        aggregate["semantic_graph"]["nodes"][record]["body"]["term"],
        "aggregate"
    );
    aggregate["semantic_graph"]["nodes"][record]["body"]["temporal_profile"] = member;
    malformed_wire(
        "a profile member on an aggregate-bodied node",
        &aggregate,
        record,
    );
}

#[trace("TC-2028", "FR-322-AC-464")]
#[test]
fn a_malformed_finite_hyper_body_profile_is_malformed_wire_at_the_member() {
    let (package, at) = finite_clock_fixture();
    let member = body_profile_path(at);
    for (case, literal, path) in [
        (
            "a non-object",
            json!("filament.temporal.event-position.false-extension/v1"),
            member.clone(),
        ),
        (
            "an extra field",
            json!({"authority": "agent-ix", "identity": BOUNDED, "extra": "x"}),
            member.clone(),
        ),
        (
            "an empty authority",
            json!({"authority": "", "identity": BOUNDED}),
            format!("{member}/authority"),
        ),
        (
            "an empty identity",
            json!({"authority": "agent-ix", "identity": ""}),
            format!("{member}/identity"),
        ),
        (
            "a missing identity",
            json!({"authority": "agent-ix"}),
            member.clone(),
        ),
    ] {
        let mut malformed = package.clone();
        malformed["semantic_graph"]["nodes"][at]["body"]["temporal_profile"] = literal;
        assert_eq!(
            refusal_of(case, &malformed).0,
            (Code::MalformedWire, None, Some(path)),
            "{case}"
        );
    }
}

#[trace("TC-2028", "FR-322-AC-464")]
#[test]
fn finite_hyper_member_checks_report_shape_then_the_lock_join_then_rows() {
    let (package, at) = finite_clock_fixture();
    let row = hyper_rows(&package)[0];
    let bad_kind = json!({
        "kind": "fixed_sample",
        "epoch": {"kind": "integer", "decimal": "0"},
        "period": {"kind": "integer", "decimal": "1"},
        "unit": "tick",
    });
    // A malformed member is reported before a row defect.
    let mut malformed = package.clone();
    malformed["semantic_graph"]["nodes"][at]["body"]["temporal_profile"]["authority"] = json!("");
    malformed["temporal_bindings"][row]["clock"] = bad_kind.clone();
    assert_eq!(
        refusal_of("malformed member and bad row", &malformed).0,
        (
            Code::MalformedWire,
            None,
            Some(format!("{}/authority", body_profile_path(at)))
        )
    );
    // An unselected member is reported before a row defect.
    let mut unselected = package;
    unselected["semantic_graph"]["nodes"][at]["body"]["temporal_profile"]["identity"] =
        json!("filament.temporal.fixed-sample.false-extension/v1");
    unselected["temporal_bindings"][row]["clock"] = bad_kind;
    assert_eq!(
        refusal_of("unselected member and bad row", &unselected).0,
        (
            Code::InvalidPackage,
            Some(Cause::OperationLawUnselected),
            Some(body_profile_path(at))
        )
    );
}

#[trace("TC-483", "FR-038-AC-561")]
#[test]
fn tc_483_finite_domain_occurrences_may_carry_different_clock_roles() {
    let (mut package, at) = finite_clock_fixture();
    package["semantic_graph"]["nodes"][at]["occurrences"] =
        json!([{"role": "claim", "ordinal": 0}, {"role": "claim", "ordinal": 1}]);
    settle(&mut package);
    rebuild_complete_source_map(&mut package);
    refresh_claim_names(&mut package);
    let rows = hyper_rows(&package);
    assert_eq!(rows.len(), 4, "two Groups for each of two occurrences");
    for &row in &rows[2..] {
        package["temporal_bindings"][row]["clock_name"] = json!("u");
    }
    let admitted = admitted("two occurrences with roles t and u", &package);
    let names: Vec<_> = admitted
        .temporal_bindings()
        .expect("rows")
        .iter()
        .filter(|row| row.trace.is_some())
        .map(|row| &*row.clock_name)
        .collect();
    assert_eq!(names, ["t", "t", "u", "u"]);
    // Agreement is within one occurrence.
    let mut disagreeing = package.clone();
    disagreeing["temporal_bindings"][rows[3]]["clock_name"] = json!("v");
    refusal_at(
        &disagreeing,
        Code::InvalidPackage,
        TIMED_NODE_REFERENCE,
        &format!("/temporal_bindings/{}/clock_name", rows[3]),
    );
}

#[trace("TC-483", "FR-038-AC-561")]
#[test]
fn tc_483_a_behaviours_node_under_a_timed_wrapper_profile_has_no_rows() {
    let mut package = v2_all_families();
    select_profile(&mut package, TIMED);
    with_model(&mut package);
    let root = find_identity(&package, &temporal_identity("eventually"));
    package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"] = unbounded();
    settle(&mut package);
    let clause_key = node_label(&package, clause(&package));
    let hyper = push(
        &mut package,
        hyperproperty(
            "hyperproperty",
            vec![trace_group("s"), trace_group("n")],
            vec![],
            vec![],
            refer(&clause_key),
        ),
    );
    claim_occurrence(&mut package, hyper);
    settle(&mut package);
    assert!(hyper_rows(&package).is_empty());
    admitted("a timed wrapper profile", &package);
}

#[trace("TC-483", "FR-038-AC-561")]
#[test]
fn tc_483_a_finite_domain_node_under_a_profile_without_a_clock_refuses_at_its_body() {
    let (mut package, at) = finite_clock_fixture();
    admitted("a clock-configured profile", &package);
    select_profile(&mut package, INFINITE);
    settle(&mut package);
    assert!(
        hyper_rows(&package).is_empty(),
        "an infinite-trace lock configures no clock"
    );
    refusal_at(
        &package,
        Code::InvalidPackage,
        TIMED_NODE_REFERENCE,
        &at_node(at, "/body"),
    );
}

#[trace("TC-483", "FR-038-AC-561")]
#[test]
fn tc_483_a_row_keyed_on_the_body_wrapper_refuses_at_its_declaration() {
    let (mut package, wrapper, _) = hyper_clock_fixture(&["s", "n"]);
    package["temporal_bindings"][0]["declaration"] = nodes(&package)[wrapper]["node_id"].clone();
    refusal_at_trace(&package, "/temporal_bindings/0/declaration");
}

#[trace("TC-483", "FR-038-AC-561")]
#[test]
fn tc_483_hyperproperties_sharing_a_wrapper_each_own_their_rows() {
    let (mut package, wrapper, _) = hyper_clock_fixture(&["s", "n"]);
    let clause_key = node_label(&package, wrapper);
    let second = push(
        &mut package,
        hyperproperty(
            "hyperproperty",
            vec![trace_group("x"), trace_group("y"), trace_group("z")],
            vec![],
            vec![],
            refer(&clause_key),
        ),
    );
    claim_occurrence(&mut package, second);
    settle(&mut package);
    let admitted = admitted("two hyperproperties over one wrapper", &package);
    let rows = admitted.temporal_bindings().expect("rows");
    let mut by_node: Vec<(CheckedNodeId, Vec<&str>)> = Vec::new();
    for row in rows {
        let trace = row.trace.as_deref().expect("trace");
        match by_node.iter_mut().find(|(id, _)| *id == row.declaration) {
            Some((_, traces)) => traces.push(trace),
            None => by_node.push((row.declaration.clone(), vec![trace])),
        }
    }
    assert_eq!(by_node.len(), 2);
    assert!(by_node.iter().any(|(_, traces)| traces == &["s", "n"]));
    assert!(by_node.iter().any(|(_, traces)| traces == &["x", "y", "z"]));
}

#[trace("TC-483", "FR-038-AC-561")]
#[test]
fn tc_483_reversed_quantifiers_over_one_wrapper_keep_their_own_row_order() {
    let (mut package, wrapper, fwd) = hyper_clock_fixture(&["s", "n"]);
    let clause_key = node_label(&package, wrapper);
    let rev = push(
        &mut package,
        hyperproperty(
            "hyperproperty",
            vec![trace_group("n"), trace_group("s"), trace_group("m")],
            vec![],
            vec![],
            refer(&clause_key),
        ),
    );
    claim_occurrence(&mut package, rev);
    settle(&mut package);
    let admitted = admitted("Fwd and Rev", &package);
    let rows = admitted.temporal_bindings().expect("rows");
    let rev_id = typed_node_id_of(&package, rev);
    let fwd_id = typed_node_id_of(&package, fwd);
    let of = |id: &CheckedNodeId| -> Vec<&str> {
        rows.iter()
            .filter(|row| row.declaration == *id)
            .filter_map(|row| row.trace.as_deref())
            .collect()
    };
    assert_eq!(of(&fwd_id), ["s", "n"]);
    assert_eq!(of(&rev_id), ["n", "s", "m"]);
    // Rev's rows written in Fwd's order refuse at the later row's trace.
    let at: Vec<usize> = package["temporal_bindings"]
        .as_array()
        .expect("rows")
        .iter()
        .enumerate()
        .filter(|(_, row)| row["declaration"] == nodes(&package)[rev]["node_id"])
        .map(|(at, _)| at)
        .collect();
    let mut swapped = package.clone();
    swapped["temporal_bindings"]
        .as_array_mut()
        .expect("rows")
        .swap(at[0], at[1]);
    refusal_at_trace(&swapped, &format!("/temporal_bindings/{}/trace", at[1]));
    // The third quantifier's row is required.
    let mut missing = package.clone();
    missing["temporal_bindings"]
        .as_array_mut()
        .expect("rows")
        .remove(at[2]);
    refusal_at(
        &missing,
        Code::InvalidPackage,
        TIMED_NODE_REFERENCE,
        &at_node(rev, "/body"),
    );
}

/// FR-402-AC-10 refuses a repeated quantifier variable at the hyperproperty
/// step, so no checked package carries one. A reader that sees one anyway
/// matches coverage by key: the second row of two Groups' shared variable is a
/// repeated key, and one row for them is not a missing row.
#[trace("TC-483", "FR-038-AC-561")]
#[test]
fn tc_483_a_repeated_quantifier_variable_is_a_repeated_key_not_a_missing_row() {
    let (package, _, _) = hyper_clock_fixture(&["s", "s"]);
    assert_eq!(hyper_rows(&package).len(), 2, "one row per Group");
    refusal_at(
        &package,
        Code::InvalidPackage,
        Cause::ConflictingDefinition,
        "/temporal_bindings/1",
    );
    let (mut finite, _) = finite_clock_fixture_with(&["a", "a"]);
    refusal_at(
        &finite,
        Code::InvalidPackage,
        Cause::ConflictingDefinition,
        &format!("/temporal_bindings/{}", hyper_rows(&finite)[1]),
    );
    let at = hyper_rows(&finite)[1];
    finite["temporal_bindings"]
        .as_array_mut()
        .expect("rows")
        .remove(at);
    if let CheckedPackageV2ReadResult::Refused(refusal) = read(&finite) {
        let path = refusal.loci[0]
            .path
            .as_ref()
            .map(|path| path.as_str().to_owned());
        assert!(
            !path.as_deref().is_some_and(
                |path| path.ends_with("/body") && refusal.cause == Some(TIMED_NODE_REFERENCE)
            ),
            "one row for a repeated variable is not a missing row: {path:?}"
        );
    }
}

#[trace("TC-483", "FR-038-AC-561")]
#[test]
fn tc_483_a_behaviours_node_under_an_unconfigured_wrapper_profile_has_no_rows() {
    let (mut package, _, keys) = hyper_fixture();
    let clause_key = keys.split('\n').nth(1).expect("clause").to_owned();
    let hyper = push(
        &mut package,
        hyperproperty(
            "hyperproperty",
            vec![trace_group("s"), trace_group("n")],
            vec![],
            vec![],
            refer(&clause_key),
        ),
    );
    claim_occurrence(&mut package, hyper);
    settle(&mut package);
    assert!(package.get("temporal_bindings").is_none());
    admitted("an infinite-trace wrapper", &package);
}

#[trace("TC-483", "FR-038-AC-561")]
#[test]
fn tc_483_a_missing_trace_row_refuses_at_the_hyperproperty_body() {
    for dropped in [0, 1] {
        let (mut package, _, hyper) = hyper_clock_fixture(&["s", "n"]);
        package["temporal_bindings"]
            .as_array_mut()
            .expect("rows")
            .remove(dropped);
        expect(
            "a missing trace row",
            &package,
            Code::InvalidPackage,
            TIMED_NODE_REFERENCE,
            &at_node(hyper, "/body"),
        );
    }
}

#[trace("TC-483", "FR-038-AC-561")]
#[test]
fn tc_483_an_unknown_or_absent_trace_refuses_at_the_row_trace() {
    let (base, _, _) = hyper_clock_fixture(&["s", "n"]);
    let mut unknown = base.clone();
    unknown["temporal_bindings"][1]["trace"] = json!("x");
    refusal_at_trace(&unknown, "/temporal_bindings/1/trace");
    let mut absent = base;
    absent["temporal_bindings"][0]
        .as_object_mut()
        .expect("row")
        .remove("trace");
    refusal_at_trace(&absent, "/temporal_bindings/0/trace");
}

#[trace("TC-483", "FR-038-AC-561")]
#[test]
fn tc_483_a_repeated_trace_key_is_a_conflicting_definition_at_the_later_row() {
    let (mut package, _, _) = hyper_clock_fixture(&["s", "n"]);
    let rows = package["temporal_bindings"].as_array_mut().expect("rows");
    let again = rows[0].clone();
    rows.push(again);
    refusal_at(
        &package,
        Code::InvalidPackage,
        Cause::ConflictingDefinition,
        "/temporal_bindings/2",
    );
}

#[trace("TC-483", "FR-038-AC-561")]
#[test]
fn tc_483_rows_out_of_quantifier_order_refuse_at_the_later_trace() {
    let (mut package, _, _) = hyper_clock_fixture(&["s", "n"]);
    package["temporal_bindings"]
        .as_array_mut()
        .expect("rows")
        .swap(0, 1);
    refusal_at_trace(&package, "/temporal_bindings/1/trace");
}

#[trace("TC-483", "FR-038-AC-561")]
#[test]
fn tc_483_a_behaviours_row_carries_its_wrappers_clock_text() {
    let (mut package, _, _) = hyper_clock_fixture(&["s", "n"]);
    package["temporal_bindings"][1]["clock_name"] = json!("another-role");
    refusal_at_trace(&package, "/temporal_bindings/1/clock_name");
}

#[trace("TC-483", "FR-038-AC-561")]
#[test]
fn tc_483_a_trace_on_a_single_trace_row_refuses_at_its_trace() {
    let mut package = v2_all_families();
    select_profile(&mut package, BOUNDED);
    settle(&mut package);
    admitted("single-trace clause", &package);
    let mut missing = package.clone();
    missing
        .as_object_mut()
        .expect("package")
        .remove("temporal_bindings");
    expect(
        "a single-trace clause without its row",
        &missing,
        Code::InvalidPackage,
        TIMED_NODE_REFERENCE,
        &at_node(clause(&package), "/body"),
    );
    package["temporal_bindings"][0]["trace"] = json!("s");
    refusal_at_trace(&package, "/temporal_bindings/0/trace");
}

#[trace("TC-483", "FR-038-AC-561")]
#[test]
fn tc_483_an_empty_or_null_trace_is_malformed_wire() {
    for bad in [json!(""), Value::Null, json!(3)] {
        let (mut package, _, _) = hyper_clock_fixture(&["s", "n"]);
        package["temporal_bindings"][0]["trace"] = bad.clone();
        match read(&package) {
            CheckedPackageV2ReadResult::Refused(refusal) => {
                assert_eq!(refusal.code, Code::MalformedWire, "{bad}");
                let path = refusal.loci[0]
                    .path
                    .as_ref()
                    .map(|path| path.as_str().to_owned());
                assert_eq!(path.as_deref(), Some("/temporal_bindings/0/trace"), "{bad}");
            }
            other => panic!("{bad}: expected a refusal, read {other:?}"),
        }
    }
}

/// The shape check keys on the raw binding names: `parameters` (member 2) and
/// `body` (member 6) are contextual name kinds, not `Other`, and must still
/// match. Read without the allocation oracle, which is a separate gate.
#[trace("TC-346", "FR-038-AC-530")]
#[test]
fn tc_346_a_hyperproperty_with_its_parameters_and_body_bindings_is_admitted() {
    let (mut package, _, keys) = hyper_fixture();
    let clause_key = keys.split('\n').nth(1).expect("clause").to_owned();
    push(
        &mut package,
        hyperproperty(
            "hyperproperty",
            vec![group(vec![])],
            vec![],
            vec![],
            refer(&clause_key),
        ),
    );
    settle(&mut package);
    let evidence = evidence_with_document(&package, ORDERS, &orders_document());
    match CheckedPackageV2::read(
        &canonical(&package),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) {
        CheckedPackageV2ReadResult::Admitted(_) => {}
        other => panic!("expected admission, read {other:?}"),
    }
}

/// A hyper clause is named by the `claim_names` rows of its `hyperproperty`
/// node; its body wrapper has no `temporal_clause_names` row (FR-322-AC-466).
#[trace("TC-477", "FR-371-AC-11")]
#[test]
fn a_hyper_clauses_body_wrapper_has_no_temporal_name_row_and_its_node_is_named() {
    let (mut package, _, keys) = hyper_fixture();
    let clause_key = keys.split('\n').nth(1).expect("clause").to_owned();
    let mut node = hyperproperty(
        "hyperproperty",
        vec![group(vec![])],
        vec![],
        vec![],
        refer(&clause_key),
    );
    node["occurrences"] = json!([{"role": "claim", "ordinal": 0}]);
    let hyper = push(&mut package, node);
    settle(&mut package);
    let wrapper = nodes(&package)[clause(&package)]["node_id"].clone();
    let hyper_id = nodes(&package)[hyper]["node_id"].clone();
    // The wrapper carries a claim occurrence and no name row.
    assert!(!package["temporal_clause_names"]
        .as_array()
        .is_some_and(|rows| rows.iter().any(|row| row["declaration"] == wrapper)));
    let named: Vec<_> = package["claim_names"]
        .as_array()
        .expect("claim rows")
        .iter()
        .filter(|row| row["declaration"] == hyper_id)
        .collect();
    assert_eq!(named.len(), 1, "one row per claim occurrence of the node");
    admitted("a body wrapper with no name row", &package);
    // A row naming the wrapper refuses at its declaration.
    let mut wrapped = package.clone();
    let row = json!({
        "declaration": wrapper,
        "occurrence": 0,
        "name": "Wrapped",
        "declaration_region": crate::support::checked_package::occurrence_region(
            &wrapped, &wrapper, "claim", 0),
    });
    if wrapped.get("temporal_clause_names").is_none() {
        wrapped["temporal_clause_names"] = json!([]);
    }
    let rows = wrapped["temporal_clause_names"]
        .as_array_mut()
        .expect("clause rows");
    rows.push(row);
    let at = rows.len() - 1;
    assert_eq!(
        first_refusal("a row naming the body wrapper", &wrapped),
        outcome(
            Code::InvalidPackage,
            Some(Cause::InvalidValue),
            format!("/temporal_clause_names/{at}/declaration"),
        )
    );
}

#[trace("TC-346", "FR-038-AC-530")]
#[test]
fn tc_346_a_fairness_reference_in_a_quantifier_group_is_admitted() {
    let (mut package, fair, keys) = hyper_fixture();
    let clause_key = keys.split('\n').nth(1).expect("clause").to_owned();
    push(
        &mut package,
        hyperproperty(
            "hyperproperty",
            vec![group(vec![refer(&fair), refer(&fair)]), group(vec![])],
            vec![],
            vec![],
            refer(&clause_key),
        ),
    );
    settle(&mut package);
    admitted("fairness reference in a quantifier Group", &package);
}

#[trace("TC-346", "FR-038-AC-530")]
#[test]
fn tc_346_a_fairness_reference_elsewhere_in_a_hyperproperty_refuses_at_the_node() {
    let (base, fair, keys) = hyper_fixture();
    let clause_key = keys.split('\n').nth(1).expect("clause").to_owned();
    let body = || refer(&clause_key);
    let cases: Vec<(&str, Value)> = vec![
        (
            "in the match aggregate",
            hyperproperty(
                "hyperproperty",
                vec![group(vec![])],
                vec![refer(&fair)],
                vec![],
                body(),
            ),
        ),
        (
            "in the align_skip aggregate",
            hyperproperty(
                "hyperproperty",
                vec![group(vec![])],
                vec![],
                vec![refer(&fair)],
                body(),
            ),
        ),
        (
            "as the body target",
            hyperproperty(
                "hyperproperty",
                vec![group(vec![])],
                vec![],
                vec![],
                refer(&fair),
            ),
        ),
        (
            "as the value of a Group's binding",
            hyperproperty(
                "hyperproperty",
                vec![group(vec![named("extra", refer(&fair))])],
                vec![],
                vec![],
                body(),
            ),
        ),
        (
            "directly in the quantifiers Tuple",
            hyperproperty("hyperproperty", vec![refer(&fair)], vec![], vec![], body()),
        ),
        (
            "in the quantifier Group of another claim form",
            hyperproperty(
                "verification_claim",
                vec![group(vec![refer(&fair)])],
                vec![],
                vec![],
                body(),
            ),
        ),
    ];
    for (case, node) in cases {
        let mut package = base.clone();
        let position = push(&mut package, node);
        settle(&mut package);
        misplaced(case, &package, position);
    }
}

#[trace("TC-346", "FR-038-AC-530")]
#[test]
fn tc_346_a_formula_reference_anywhere_in_a_hyperproperty_refuses_at_the_node() {
    let (base, _, keys) = hyper_fixture();
    let (formula_key, clause_key) = keys.split_once('\n').expect("keys");
    let cases: Vec<(&str, Value)> = vec![
        (
            "in a quantifier Group",
            hyperproperty(
                "hyperproperty",
                vec![group(vec![refer(formula_key)])],
                vec![],
                vec![],
                refer(clause_key),
            ),
        ),
        (
            "in the match aggregate",
            hyperproperty(
                "hyperproperty",
                vec![group(vec![])],
                vec![refer(formula_key)],
                vec![],
                refer(clause_key),
            ),
        ),
        (
            "as the body target",
            hyperproperty(
                "hyperproperty",
                vec![group(vec![])],
                vec![],
                vec![],
                refer(formula_key),
            ),
        ),
        (
            "in the align_skip aggregate",
            hyperproperty(
                "hyperproperty",
                vec![group(vec![])],
                vec![],
                vec![refer(formula_key)],
                refer(clause_key),
            ),
        ),
        (
            "as the value of a Group's binding",
            hyperproperty(
                "hyperproperty",
                vec![group(vec![named("extra", refer(formula_key))])],
                vec![],
                vec![],
                refer(clause_key),
            ),
        ),
        (
            "directly in the quantifiers Tuple",
            hyperproperty(
                "hyperproperty",
                vec![refer(formula_key)],
                vec![],
                vec![],
                refer(clause_key),
            ),
        ),
    ];
    for (case, node) in cases {
        let mut package = base.clone();
        let position = push(&mut package, node);
        settle(&mut package);
        misplaced(case, &package, position);
    }
}

/// A package whose hyperproperty node has one Group, aliased `m` to the
/// `model_root` of `subject`, holding one reference to a fairness node over
/// `member`. The hyperproperty node's position.
fn hyper_resolution(profile: &str, member: Value, subject: &str) -> (Value, usize) {
    let (mut package, fair, keys) = hyper_fixture_with(profile, member);
    select_model(&mut package, subject);
    let clause_key = keys.split('\n').nth(1).expect("clause").to_owned();
    let mut node = hyperproperty(
        "hyperproperty",
        vec![group(vec![refer(&fair)])],
        vec![],
        vec![],
        refer(&clause_key),
    );
    let root = json!({"package": subject, "path": ["model", "model_root"], "ordinal": 0});
    node["body"]["members"][1]["value"]["members"] =
        json!([named("m", json!({"term": "reference", "target": root}))]);
    let position = push(&mut package, node);
    settle(&mut package);
    (package, position)
}

fn at_hyper_reference(position: usize) -> String {
    at_node(position, "/body/members/3/value/members/0/members/3")
}

#[trace("TC-346", "FR-038-AC-530")]
#[test]
fn tc_346_a_group_fairness_reference_resolves_as_a_clause_member_does() {
    let order = model_key(ORDER_NODE);
    let (package, _) = hyper_resolution(INFINITE, fairness_member(&order, "scaled"), ORDERS);
    admitted("a resolving reference under its alias's package", &package);

    let (package, at) = hyper_resolution(INFINITE, fairness_member(&order, "nope"), ORDERS);
    expect_located(
        "an operation the declaration lacks",
        &package,
        (
            Code::MissingDeclaration,
            Cause::MissingName,
            &at_hyper_reference(at),
        ),
        &order,
    );

    let mut member = fairness_member(&order, "scaled");
    member["target"] = json!({"kind": "protocol_root", "node": node_id(&aaaa())});
    let (package, at) = hyper_resolution(INFINITE, member, ORDERS);
    expect(
        "a target that is not an operation",
        &package,
        Code::InvalidModelBinding,
        Cause::MalformedDeclaration,
        &at_hyper_reference(at),
    );

    let (package, at) =
        hyper_resolution(INFINITE, fairness_member(&order, "scaled"), OTHER_PACKAGE);
    expect(
        "a declaring node outside the alias's package",
        &package,
        Code::InvalidModelBinding,
        Cause::MalformedDeclaration,
        &at_hyper_reference(at),
    );

    let (package, at) = hyper_resolution(BOUNDED, fairness_member(&order, "scaled"), ORDERS);
    expect(
        "a set under a profile that admits none",
        &package,
        Code::InvalidPackage,
        Cause::OperationMemberMismatch,
        &at_hyper_reference(at),
    );
}

/// The resolving hyperproperty package of [`hyper_resolution`], its node's body
/// edited by `edit`.
fn hyper_edited(edit: impl FnOnce(&mut Value)) -> (Value, usize) {
    let (mut package, at) = hyper_resolution(
        INFINITE,
        fairness_member(&model_key(ORDER_NODE), "scaled"),
        ORDERS,
    );
    edit(&mut package["semantic_graph"]["nodes"][at]["body"]);
    settle(&mut package);
    (package, at)
}

fn at_body(at: usize, tail: &str) -> String {
    at_node(at, &format!("/body{tail}"))
}

/// The first refusal of `package`: code, cause and pointer.
fn first_refusal(case: &str, package: &Value) -> Outcome {
    refusal_of(case, package).0
}

fn outcome(code: Code, cause: Option<Cause>, path: String) -> Outcome {
    (code, cause, Some(path))
}

#[trace("TC-346", "FR-038-AC-531")]
#[test]
fn tc_346_a_group_reference_to_a_non_fairness_node_is_refused_before_resolution() {
    let (package, _) = hyper_edited(|_| {});
    let wrapper = node_label(&package, clause(&package));
    let expression = node_label(&package, find_form(&package, "expression", "reference"));
    for (case, target) in [
        ("a body wrapper", wrapper),
        ("a Boolean expression", expression),
        ("an object type", model_key(ORDER_NODE)),
    ] {
        let (package, at) = hyper_edited(|body| {
            body["members"][3]["value"]["members"][0]["members"][3] = refer(&target);
        });
        expect(
            case,
            &package,
            Code::IllTyped,
            Cause::OperatorIneligible,
            &at_hyper_reference(at),
        );
    }
}

#[trace("TC-346", "FR-038-AC-530")]
#[test]
fn tc_346_the_hyperproperty_shape_is_refused_at_the_first_member_that_breaks_it() {
    let (package, _) = hyper_edited(|_| {});
    admitted("the unedited hyperproperty", &package);

    let (package, at) = hyper_edited(|body| {
        body["members"].as_array_mut().expect("members").pop();
    });
    assert_eq!(
        first_refusal("six bindings", &package),
        outcome(Code::InvalidSemanticGraph, None, at_body(at, ""))
    );
    let (package, at) = hyper_edited(|body| {
        body["members"].as_array_mut().expect("members").swap(4, 5);
    });
    assert_eq!(
        first_refusal("match and align_skip swapped", &package),
        outcome(Code::InvalidSemanticGraph, None, at_body(at, "/members/4"))
    );
    let (package, at) = hyper_edited(|body| {
        body["members"][3]["value"]["members"][0]["members"][1]["value"]["value"] =
            json!("sometimes");
    });
    assert_eq!(
        first_refusal("a quantifier word", &package),
        outcome(
            Code::InvalidPackage,
            Some(Cause::InvalidValue),
            at_body(at, "/members/3/value/members/0/members/1/value")
        )
    );
    let (package, at) = hyper_edited(|body| {
        body["members"][3]["value"]["members"][0]["members"][3] = text("x");
    });
    assert_eq!(
        first_refusal("a non-reference after the bindings", &package),
        outcome(
            Code::InvalidSemanticGraph,
            None,
            at_body(at, "/members/3/value/members/0/members/3")
        )
    );
}

#[trace("TC-346", "FR-038-AC-530")]
#[test]
fn tc_346_the_hyperproperty_subjects_are_checked_before_its_fairness() {
    let (package, at) = hyper_edited(|body| {
        body["members"][1]["value"]["members"][0]["value"]["target"] = node_id(&aaaa());
    });
    assert_eq!(
        first_refusal("a subject that is no model_root", &package),
        outcome(
            Code::InvalidModelBinding,
            Some(Cause::MalformedDeclaration),
            at_body(at, "/members/1/value/members/0/value/target")
        )
    );
    let (package, at) = hyper_edited(|body| {
        body["members"][3]["value"]["members"][0]["members"][2]["value"]["value"] = json!("other");
    });
    assert_eq!(
        first_refusal("an alias naming no subject", &package),
        outcome(
            Code::MissingDeclaration,
            Some(Cause::MissingName),
            at_body(at, "/members/3/value/members/0/members/2")
        )
    );
    let (package, at) = hyper_edited(|body| {
        body["members"][3]["value"]["members"] = json!([]);
    });
    assert_eq!(
        first_refusal("a subject no Group names", &package),
        outcome(
            Code::InvalidSemanticGraph,
            None,
            at_body(at, "/members/1/value/members/0")
        )
    );
    // A subjects defect and an unresolvable fairness reference together: the
    // earlier step decides.
    let (mut package, at) = hyper_resolution(
        INFINITE,
        fairness_member(&model_key(ORDER_NODE), "nope"),
        ORDERS,
    );
    package["semantic_graph"]["nodes"][at]["body"]["members"][1]["value"]["members"][0]["value"]
        ["target"] = node_id(&aaaa());
    settle(&mut package);
    assert_eq!(
        first_refusal("a subjects and a fairness defect", &package),
        outcome(
            Code::InvalidModelBinding,
            Some(Cause::MalformedDeclaration),
            at_body(at, "/members/1/value/members/0/value/target")
        )
    );
}

#[trace("TC-346", "FR-038-AC-530")]
#[test]
fn tc_346_a_temporal_step_defect_precedes_a_hyperproperty_fairness_defect() {
    let (mut package, _) = hyper_resolution(
        INFINITE,
        fairness_member(&model_key(ORDER_NODE), "nope"),
        ORDERS,
    );
    let holds = node_label(
        &package,
        find_identity(&package, &temporal_identity("holds")),
    );
    let bad = push(
        &mut package,
        formula("once", closed("3", "0"), vec![refer(&holds)]),
    );
    settle(&mut package);
    let (code, _, path) = first_refusal("two defects", &package);
    assert_eq!(code, Code::InvalidPackage);
    let path = path.expect("path");
    assert!(
        path.starts_with(&at_node(bad, "/")),
        "the temporal step's refusal comes first, got {path}"
    );
}

#[trace("TC-346", "FR-038-AC-530")]
#[test]
fn tc_346_the_hyperproperty_body_is_checked_after_its_subjects_and_before_its_fairness() {
    // A `behaviours` body that is no temporal clause.
    let (package, at) = hyper_edited(|body| {
        body["members"][6]["value"] = refer(&aaaa());
    });
    misplaced("a body target that is no temporal clause", &package, at);
    // A body wrapper with a fairness argument.
    let (mut package, at) = hyper_resolution(
        INFINITE,
        fairness_member(&model_key(ORDER_NODE), "scaled"),
        ORDERS,
    );
    let (fair_key, wrapper) = {
        let wrapper = clause(&package);
        let fair = package["semantic_graph"]["nodes"][at]["body"]["members"][3]["value"]["members"]
            [0]["members"][3]
            .clone();
        (fair, wrapper)
    };
    package["semantic_graph"]["nodes"][wrapper]["body"]["arguments"][4] =
        json!({"term": "aggregate", "members": [fair_key]});
    settle(&mut package);
    assert_eq!(
        first_refusal("a wrapper with fairness", &package),
        outcome(
            Code::InvalidSemanticGraph,
            None,
            at_node(wrapper, "/body/arguments/4")
        )
    );
    // A body defect precedes a fairness defect.
    let (mut package, at) = hyper_resolution(
        INFINITE,
        fairness_member(&model_key(ORDER_NODE), "nope"),
        ORDERS,
    );
    package["semantic_graph"]["nodes"][at]["body"]["members"][6]["value"] = refer(&aaaa());
    settle(&mut package);
    misplaced("a body and a fairness defect", &package, at);
}

/// A finite-domain hyperproperty over the fixture's Boolean `expression` node:
/// its domain is a reference and its Groups bind no alias.
fn finite_hyper(edit: impl FnOnce(&mut Value)) -> (Value, usize) {
    let mut package = v2_all_families();
    let record = fresh();
    let mut record_node = node_with_key(
        &record,
        "composite_type",
        "record",
        json!({"term":"aggregate", "members":[]}),
    );
    record_node["semantic_type"] = node_id(&record);
    push(&mut package, record_node);
    let integer = node_label(&package, find_form(&package, "scalar_type", "integer"));
    let finite_domain = fresh();
    let mut domain_node = node_with_key(
        &finite_domain,
        "bounded_domain",
        "finite_trace_population",
        json!({"term":"aggregate", "members":[named("bound", json!({
            "term":"literal", "type":node_id(&integer), "value_kind":"integer", "value":"3"
        }))]}),
    );
    domain_node["semantic_type"] = node_id(&record);
    push(&mut package, domain_node);
    let boolean_expression = find_form(&package, "expression", "reference");
    let body_target = node_label(&package, boolean_expression);
    let body = refer(&body_target);
    let mut node = hyperproperty(
        "hyperproperty",
        vec![json!({"term": "aggregate", "members": [
            named("variable", text("a")), named("quantifier", text("forall"))]})],
        vec![],
        vec![],
        body,
    );
    node["body"]["members"][0]["value"] = refer(&finite_domain);
    node["body"]["members"][1]["value"]["members"] = json!([]);
    node["body"]["temporal_profile"] = package["lock"]["profile_selections"]
        .as_array()
        .expect("selections")
        .iter()
        .find(|selection| selection["role"] == "temporal_profile")
        .map_or(Value::Null, |selection| selection["definition"].clone());
    let mut dependencies = [node_id(&finite_domain), node_id(&body_target)];
    dependencies.sort_by_key(|dependency| {
        serde_json::from_value::<CheckedNodeId>(dependency.clone()).expect("dependency ID")
    });
    node["dependencies"] = json!(dependencies);
    edit(&mut node["body"]);
    let at = push(&mut package, node);
    settle(&mut package);
    (package, at)
}

#[trace("TC-346", "FR-038-AC-530")]
#[test]
fn tc_346_a_finite_domain_hyperproperty_has_no_subjects_alias_or_fairness_and_a_boolean_body() {
    let (package, _) = finite_hyper(|_| {});
    let admitted_package = admitted("a finite-domain hyperproperty", &package);
    let domain = nodes(&package)
        .iter()
        .find(|node| node["semantic_form"] == "finite_trace_population")
        .expect("finite trace domain node");
    let claim = nodes(&package)
        .iter()
        .find(|node| node["semantic_form"] == "hyperproperty")
        .expect("hyperproperty node");
    assert!(claim["dependencies"]
        .as_array()
        .expect("dependency list")
        .contains(&domain["node_id"]));
    assert!(admitted_package.graph().nodes.iter().any(|node| {
        node.node_id
            == serde_json::from_value(domain["node_id"].clone()).expect("finite trace node ID")
    }));
    // A body that is a `value` node, not an `expression` node.
    let (package, at) = finite_hyper(|body| {
        body["members"][6]["value"] = refer(&aaaa());
    });
    assert_eq!(
        first_refusal("a body that is no expression node", &package),
        outcome(
            Code::IllTyped,
            Some(Cause::NonBooleanRoot),
            at_body(at, "/members/6/value/target")
        )
    );
    let (package, at) = finite_hyper(|body| {
        body["members"][1]["value"]["members"] = json!([named("m", text("x"))]);
    });
    assert_eq!(
        first_refusal("a subjects member", &package),
        outcome(
            Code::InvalidSemanticGraph,
            None,
            at_body(at, "/members/1/value/members/0")
        )
    );
    let (package, at) = finite_hyper(|body| {
        body["members"][3]["value"]["members"][0]["members"]
            .as_array_mut()
            .expect("members")
            .push(named("alias", text("m")));
    });
    assert_eq!(
        first_refusal("an alias binding", &package),
        outcome(
            Code::InvalidSemanticGraph,
            None,
            at_body(at, "/members/3/value/members/0/members/2")
        )
    );
    let (package, at) = finite_hyper(|body| {
        body["members"][3]["value"]["members"][0]["members"]
            .as_array_mut()
            .expect("members")
            .push(refer(&aaaa()));
    });
    assert_eq!(
        first_refusal("a fairness reference", &package),
        outcome(
            Code::InvalidSemanticGraph,
            None,
            at_body(at, "/members/3/value/members/0/members/2")
        )
    );
    let (package, at) = finite_hyper(|body| body["members"][0]["value"] = refer(&aaaa()));
    assert_eq!(
        first_refusal("a reference to a non-finite domain", &package),
        outcome(
            Code::InvalidSemanticGraph,
            None,
            at_body(at, "/members/0/value")
        )
    );

    for member_at in [2, 4, 5] {
        let (package, at) = finite_hyper(|body| {
            body["members"][member_at]["value"]["members"] = json!([text("unexpected")]);
        });
        assert_eq!(
            first_refusal("a nonempty finite-domain binding", &package),
            outcome(
                Code::InvalidSemanticGraph,
                None,
                at_body(at, &format!("/members/{member_at}/value/members/0"))
            )
        );
    }

    let (mut package, _) = finite_hyper(|_| {});
    let domain_at = find_form(&package, "bounded_domain", "finite_trace_population");
    package["semantic_graph"]["nodes"][domain_at]["body"]["members"][0]["value"]["value"] =
        json!("0");
    settle(&mut package);
    let refusal = refusal_of("a zero finite-trace bound", &package).0;
    assert_eq!(refusal.0, Code::InvalidPackage);
    assert_eq!(refusal.1, Some(Cause::InvalidDerivedShape));

    let (mut package, _) = finite_hyper(|_| {});
    let domain_at = find_form(&package, "bounded_domain", "finite_trace_population");
    let integer = node_label(&package, find_form(&package, "scalar_type", "integer"));
    package["semantic_graph"]["nodes"][domain_at]["semantic_type"] = node_id(&integer);
    settle(&mut package);
    let refusal = refusal_of("a non-record finite-trace base", &package).0;
    assert_eq!(refusal.0, Code::InvalidPackage);
    assert_eq!(refusal.1, Some(Cause::InvalidDerivedShape));
}

#[trace("TC-346", "FR-038-AC-530")]
#[test]
fn tc_346_a_subjects_or_finite_body_refusal_is_located_at_the_target_with_its_key() {
    let target: CheckedNodeId = serde_json::from_value(node_id(&aaaa())).expect("target ID");
    let (package, at) = hyper_edited(|body| {
        body["members"][1]["value"]["members"][0]["value"]["target"] = node_id(&aaaa());
    });
    assert_eq!(
        refusal_of("a subject that is no model_root", &package),
        (
            outcome(
                Code::InvalidModelBinding,
                Some(Cause::MalformedDeclaration),
                at_body(at, "/members/1/value/members/0/value/target")
            ),
            Some(target.clone())
        )
    );
    let (package, at) = finite_hyper(|body| {
        body["members"][6]["value"] = refer(&aaaa());
    });
    assert_eq!(
        refusal_of("a finite body that is no expression", &package),
        (
            outcome(
                Code::IllTyped,
                Some(Cause::NonBooleanRoot),
                at_body(at, "/members/6/value/target")
            ),
            Some(target)
        )
    );
}

// ---------------------------------------------------------------------------
// Window aggregate arguments (FR-038-AC-461, AC-462)
// ---------------------------------------------------------------------------

const POSITIONS: &str = "filament.temporal.event-position.false-extension/v1";
const DURATION: &str = "filament.temporal.timestamped-event.finite-window/v1";

fn typed(kind: &str, type_key: Value, value: &str) -> Value {
    json!({"term": "literal", "type": type_key, "value_kind": kind, "value": value})
}

fn builtin(name: &str) -> Value {
    json!({"package": "test/families", "path": [format!("builtin/{name}")], "ordinal": 0})
}

fn clock_unit(name: &str) -> Value {
    json!({
        "package": ORDERS,
        "path": ["scalar_type", "unit", format!("ix://{ORDERS}/unit/{name}")],
        "ordinal": 0
    })
}

fn scalar(kind: &str, value: &str) -> Value {
    typed(kind, builtin(kind), value)
}

fn word(value: &str) -> Value {
    typed("text", node_id(&family_key("a1a1")), value)
}

fn binding(name: &str, value: Value) -> Value {
    json!({"term": "binding", "name": name, "value": value})
}

/// The all-families package with its `holds` formula rewritten to apply
/// `filament.op.temporal.aggregate` to `members`, under `profile`. A duration
/// window also declares the fixture's `tick` clock unit the quantity bounds are typed by.
fn aggregate_package(profile: &str, members: Vec<Value>) -> (Value, usize) {
    let mut package = v2_all_families();
    select_profile(&mut package, profile);
    if profile == DURATION {
        select_model(&mut package, ORDERS);
        let dimension_identity = format!("ix://{ORDERS}/dimension/Time");
        let dimension = json!({"package": ORDERS,
                               "path": ["scalar_type", "dimension", dimension_identity], "ordinal": 0});
        let source_region = package["source_map"][0]["regions"][0].clone();
        for node in [
            json!({
                "node_id": dimension, "schema_version": "filament.checked-semantic-graph/v2",
                "node_tag": "scalar_type", "semantic_form": "dimension",
                "semantic_type": dimension,
                "dependencies": [],
                "occurrences": [{"role": "type", "ordinal": 0}],
                "owner": {"kind": "model", "identity": ORDERS, "node": dimension_identity},
                "body": {"term": "aggregate", "members": []},
            }),
            json!({
                "node_id": clock_unit("tick"), "schema_version": "filament.checked-semantic-graph/v2",
                "node_tag": "scalar_type", "semantic_form": "unit",
                "semantic_type": dimension,
                "dependencies": [dimension],
                "occurrences": [{"role": "type", "ordinal": 0}],
                "owner": {"kind": "model", "identity": ORDERS, "node": format!("ix://{ORDERS}/unit/tick")},
                "body": {"term": "aggregate", "members": []},
            }),
        ] {
            let row = json!({
                "node_id": node["node_id"], "role": "type", "ordinal": 0,
                "regions": [source_region.clone()],
            });
            package["source_map"]
                .as_array_mut()
                .expect("rows")
                .push(row);
            push(&mut package, node);
        }
    }
    let formula = find_identity(&package, &temporal_identity("holds"));
    let body = &mut package["semantic_graph"]["nodes"][formula]["body"];
    body["operation"]["identity"] = json!("filament.op.temporal.aggregate");
    body["arguments"] = json!([{"term": "aggregate", "members": members}]);
    package["semantic_graph"]["nodes"][formula]["dependencies"] =
        json!([fixture_node_id(&family_key("eeee"))]);
    settle(&mut package);
    if profile == DURATION {
        package["temporal_bindings"][0]["clock"]["timestamp_unit"] =
            json!(format!("ix://{ORDERS}/unit/tick"));
    }
    (package, formula)
}

fn count_members(window: &str, lower: Value, upper: Value, min_count: Value) -> Vec<Value> {
    vec![
        binding("kind", word("count")),
        binding("filter", refer(&family_key("eeee"))),
        binding("window_kind", word(window)),
        binding("lower", lower),
        binding("upper", upper),
        binding("min_count", min_count),
        binding("comparison", word("<=")),
        binding("threshold", scalar("integer", "2")),
    ]
}

/// Reads through the public reader without the whole-read allocation proof of
/// `read`: that proof is red on every package holding an aggregate term at
/// this base (IR-1404, the deferred AC-511 family), and these tests assert the
/// reader's literal-kind behavior, not its storage accounting.
fn read_aggregate(package: &Value) -> CheckedPackageV2ReadResult {
    let mut document = orders_document();
    document["dimensions"] = json!([{
        "identity": format!("ix://{ORDERS}/dimension/Time"), "name": "Time", "base": true, "terms": [],
    }]);
    document["units"] = json!([{
        "identity": format!("ix://{ORDERS}/unit/tick"), "name": "tick",
        "dimension": format!("ix://{ORDERS}/dimension/Time"), "scale": "1", "offset": "0",
    }]);
    let evidence = evidence_with_document(package, ORDERS, &document);
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    )
}

/// The aggregate argument a package retains for `formula`, as JSON.
fn retained_argument(package: &Value, formula: usize) -> Value {
    let result = read_aggregate(package);
    let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
        panic!("the aggregate package admits: {result:?}");
    };
    let filament_contract_model::CheckedTerm::Application { arguments, .. } =
        &admitted.graph().nodes[formula].body
    else {
        panic!("the formula body is an application");
    };
    serde_json::to_value(&arguments[0]).expect("typed aggregate argument")
}

/// A positions window reads its integer `lower`, `upper` and `min_count`
/// literals as typed integer literals and keeps their values.
#[trace("TC-048", "FR-038-AC-461")]
#[test]
fn tc_048_a_positions_aggregate_admits_integer_bounds_and_min_count() {
    let members = count_members(
        "positions",
        scalar("integer", "0"),
        scalar("integer", "9"),
        scalar("integer", "1"),
    );
    let (package, formula) = aggregate_package(POSITIONS, members.clone());
    let argument = retained_argument(&package, formula);
    assert_eq!(argument["members"], json!(members));
    assert_eq!(argument["members"][3]["value"]["value"], "0");
    assert_eq!(argument["members"][4]["value"]["value"], "9");
    assert_eq!(argument["members"][5]["value"]["value"], "1");
}

/// A duration window reads its quantity `lower` and `upper` typed by the
/// clock unit, with an integer `min_count`, and keeps their values.
#[trace("TC-048", "FR-038-AC-461")]
#[test]
fn tc_048_a_duration_aggregate_admits_quantity_bounds_and_integer_min_count() {
    let members = count_members(
        "duration",
        typed("quantity", clock_unit("tick"), "0"),
        typed("quantity", clock_unit("tick"), "300000"),
        scalar("integer", "1"),
    );
    let (package, formula) = aggregate_package(DURATION, members.clone());
    let argument = retained_argument(&package, formula);
    assert_eq!(argument["members"], json!(members));
    assert_eq!(argument["members"][3]["value"]["value_kind"], "quantity");
    assert_eq!(argument["members"][4]["value"]["value"], "300000");
}

/// A literal of another kind than the member needs refuses typed, at the
/// member's value: `lower` must match the window, `min_count` is an integer.
#[trace("TC-048", "FR-038-AC-462")]
#[test]
fn tc_048_an_aggregate_literal_of_the_wrong_kind_refuses_at_its_member() {
    let wrong_kind = |profile: &str,
                      window: &str,
                      lower: Value,
                      upper: Value,
                      min_count: Value,
                      member: usize| {
        let (package, formula) =
            aggregate_package(profile, count_members(window, lower, upper, min_count));
        let CheckedPackageV2ReadResult::Refused(refusal) = read_aggregate(&package) else {
            panic!("a literal of the wrong kind refuses");
        };
        assert_eq!(
            (refusal.code, refusal.cause),
            (Code::InvalidPackage, Some(Cause::WrongValueKind))
        );
        assert_eq!(
            refusal.loci[0].path.as_ref().map(ToString::to_string),
            Some(at_node(
                formula,
                &format!("/body/arguments/0/members/{member}/value")
            ))
        );
    };
    let tick = |value: &str| typed("quantity", clock_unit("tick"), value);
    let whole = |value: &str| scalar("integer", value);
    wrong_kind(POSITIONS, "positions", tick("0"), whole("9"), whole("1"), 3);
    wrong_kind(
        POSITIONS,
        "positions",
        whole("0"),
        scalar("rational", "9"),
        whole("1"),
        4,
    );
    wrong_kind(DURATION, "duration", whole("0"), tick("5"), whole("1"), 3);
    wrong_kind(DURATION, "duration", tick("0"), whole("5"), whole("1"), 4);
    wrong_kind(
        POSITIONS,
        "positions",
        whole("0"),
        whole("9"),
        scalar("rational", "1"),
        5,
    );
    wrong_kind(DURATION, "duration", tick("0"), tick("5"), word("1"), 5);
}

// ---------------------------------------------------------------------------
// FR-038-AC-562 / FR-038-AC-563 (FSpec FR-402-AC-11): a body wrapper over a
// model root
// ---------------------------------------------------------------------------

fn model_root_id(package: &str) -> Value {
    json!({"package": package, "path": ["model", "model_root"], "ordinal": 0})
}

/// The two-trace hyper clock fixture with its body wrapper's `over` edited to
/// the `model_root` of `root`: the wrapper's and the hyperproperty's positions.
pub(super) fn root_wrapper_fixture(root: &str) -> (Value, usize, usize) {
    let (mut package, wrapper, hyper) = hyper_clock_fixture(&["s", "n"]);
    select_model(&mut package, root);
    package["semantic_graph"]["nodes"][wrapper]["body"]["arguments"][0] =
        json!({"term": "reference", "target": model_root_id(root)});
    settle(&mut package);
    (package, wrapper, hyper)
}

fn at_over(wrapper: usize) -> String {
    at_node(wrapper, "/body/arguments/0")
}

/// Asserts `invalid_model_binding`/`malformed-declaration` at the wrapper's
/// `over`, located at the model root it names.
fn expect_root_refused(case: &str, package: &Value, wrapper: usize, root: &str) {
    let (outcome, locus) = refusal_of(case, package);
    assert_eq!(
        outcome,
        (
            Code::InvalidModelBinding,
            Some(Cause::MalformedDeclaration),
            Some(at_over(wrapper))
        ),
        "{case}"
    );
    let root: CheckedNodeId = serde_json::from_value(model_root_id(root)).expect("root ID");
    assert_eq!(locus, Some(root), "{case}");
}

#[trace("TC-486", "FR-038-AC-562")]
#[test]
fn tc_486_a_body_wrapper_over_its_first_subjects_model_root_reads_and_writes() {
    let (package, wrapper, hyper) = root_wrapper_fixture(ORDERS);
    let admitted = admitted("a wrapper over the first subject's root", &package);
    let hyper_id = typed_node_id_of(&package, hyper);
    let rows = admitted.temporal_bindings().expect("rows");
    assert_eq!(rows.len(), 2, "one row per quantified trace");
    assert!(rows.iter().all(|row| row.declaration == hyper_id));
    assert!(admitted.temporal_clause_names().is_none_or(|names| names
        .iter()
        .all(|row| row.declaration != typed_node_id_of(&package, wrapper))));
    let encoded = filament_contract_model::encode_checked_package_v2(
        admitted.name(),
        admitted.lock(),
        admitted.graph(),
        filament_contract_model::CheckedPackageSourceCorrespondenceRef {
            source_map: admitted.source_map(),
            refinement_step_rows: admitted.refinement_step_rows(),
        },
        filament_contract_model::CheckedPackageSidecarsRef {
            checked_term_data: admitted.checked_term_data(),
            temporal_bindings: admitted.temporal_bindings(),
            temporal_clause_names: admitted.temporal_clause_names(),
            claim_names: admitted.claim_names(),
            schedulability_claims: admitted.schedulability_claim_rows(),
            operation_effects: admitted.operation_effect_rows(),
            operation_bodies: admitted.operation_body_rows(),
        },
        admitted.capability_report(),
        admitted.diagnostics(),
        quire_canonical::Limits::new(1 << 24),
    )
    .expect("the typed writer encodes the package");
    let evidence = evidence_with_document(&package, ORDERS, &orders_document());
    let CheckedPackageV2ReadResult::Admitted(again) =
        crate::read_and_account(&encoded, CheckedPackageReadLimits::bounded(), &evidence)
    else {
        panic!("the typed re-encoding admits")
    };
    assert_eq!(admitted, *again);
}

#[trace("TC-486", "FR-038-AC-563")]
#[test]
fn tc_486_a_model_root_over_refuses_everywhere_the_spec_names() {
    // A wrapper over a selected package's root no `subjects` binding names:
    // admitted by the temporal step, refused by the hyperproperty step.
    let (package, wrapper, _) = root_wrapper_fixture(OTHER_PACKAGE);
    expect_root_refused("a root no subject names", &package, wrapper, OTHER_PACKAGE);

    // A single-trace clause: no body wrapper.
    let mut package = v2_all_families();
    select_model(&mut package, ORDERS);
    let single = clause(&package);
    package["semantic_graph"]["nodes"][single]["body"]["arguments"][0] =
        json!({"term": "reference", "target": model_root_id(ORDERS)});
    settle(&mut package);
    expect_root_refused("a single-trace clause", &package, single, ORDERS);

    // A wrapper whose formula reads a slot parameter, of any tag; a parameter
    // that is no slot parameter is read freely.
    for (name, slot) in [
        ("slot_sv_sa", true),
        ("op_sa", true),
        ("rcv_sa", true),
        ("arg_sa_sVault_sstep_si", true),
        ("slot_sa__b_sc", true),
        ("op_s_xC3_xA9", true),
        ("slotted", false),
        ("operand", false),
        // Prefix-alike names that are no encoding: the wrong field count, no
        // `_s` after the tag, a bad escape, escaped bytes that are no UTF-8.
        ("slot_sv", false),
        ("slot_sv_sa_sb", false),
        ("op_sa_sb", false),
        ("op_a", false),
        ("op_", false),
        ("rcv_s", false),
        ("arg_sa_sb_sc", false),
        ("op_sa_xzz", false),
        ("op_sa_x4", false),
        ("op_sa_x4a", false),
        ("op_s_xC3", false),
        ("op_sa_q", false),
    ] {
        let (mut package, wrapper, _) = root_wrapper_fixture(ORDERS);
        let parameter = find_form(&package, "value", "parameter");
        let holds = find_identity(&package, &temporal_identity("holds"));
        package["semantic_graph"]["nodes"][parameter]["body"]["members"][0]["value"]["value"] =
            json!(name);
        let parameter_id = nodes(&package)[parameter]["node_id"].clone();
        package["semantic_graph"]["nodes"][holds]["body"]["arguments"][0] =
            json!({"term": "reference", "target": parameter_id});
        settle(&mut package);
        if slot {
            expect_root_refused(name, &package, wrapper, ORDERS);
        } else {
            admitted(name, &package);
        }
    }

    // A root outside the wrapper's dependencies keeps `missing-name`.
    let (mut package, wrapper, _) = root_wrapper_fixture(ORDERS);
    package["semantic_graph"]["nodes"][wrapper]["body"]["arguments"][0] =
        json!({"term": "reference", "target": model_root_id("acme/absent")});
    settle(&mut package);
    let (outcome, _) = refusal_of("a root naming no node", &package);
    assert_eq!(
        outcome,
        (
            Code::MissingDeclaration,
            Some(Cause::MissingName),
            Some(at_over(wrapper))
        )
    );
}

/// The root-wrapper fixture with a second subject `k`, the `model_root` of
/// `OTHER_PACKAGE`, named by trace `n`'s Group; the wrapper's `over` is the root
/// of `over`.
fn two_subject_fixture(over: &str) -> (Value, usize) {
    let (mut package, wrapper, hyper) = root_wrapper_fixture(over);
    select_model(&mut package, OTHER_PACKAGE);
    let body = &mut package["semantic_graph"]["nodes"][hyper]["body"];
    body["members"][1]["value"]["members"]
        .as_array_mut()
        .expect("subjects")
        .push(named(
            "k",
            json!({"term": "reference", "target": model_root_id(OTHER_PACKAGE)}),
        ));
    body["members"][3]["value"]["members"][1]["members"][2]["value"]["value"] = json!("k");
    settle(&mut package);
    (package, wrapper)
}

#[trace("TC-486", "FR-038-AC-563")]
#[test]
fn tc_486_a_model_root_over_is_the_first_subjects_never_a_later_ones() {
    let (package, _) = two_subject_fixture(ORDERS);
    admitted("the first subject's root", &package);
    let (package, wrapper) = two_subject_fixture(OTHER_PACKAGE);
    expect_root_refused(
        "the second subject's root",
        &package,
        wrapper,
        OTHER_PACKAGE,
    );
}

#[trace("TC-486", "FR-038-AC-563")]
#[test]
fn tc_486_the_wrapper_fairness_refusal_precedes_the_model_root_refusal() {
    // FSpec FR-402 "Reader order" step 3: a non-empty `fairness` argument
    // refuses before a root that is not the first subject's.
    let (mut package, _) = hyper_resolution(
        INFINITE,
        fairness_member(&model_key(ORDER_NODE), "scaled"),
        ORDERS,
    );
    select_model(&mut package, OTHER_PACKAGE);
    let wrapper = clause(&package);
    let hyper = find_form(&package, "claim", "hyperproperty");
    let fair = package["semantic_graph"]["nodes"][hyper]["body"]["members"][3]["value"]["members"]
        [0]["members"][3]
        .clone();
    package["semantic_graph"]["nodes"][wrapper]["body"]["arguments"][4] =
        json!({"term": "aggregate", "members": [fair]});
    package["semantic_graph"]["nodes"][wrapper]["body"]["arguments"][0] =
        json!({"term": "reference", "target": model_root_id(OTHER_PACKAGE)});
    settle(&mut package);
    let (outcome, _) = refusal_of("fairness before the root", &package);
    assert_eq!(
        outcome,
        (
            Code::InvalidSemanticGraph,
            None,
            Some(at_node(wrapper, "/body/arguments/4"))
        )
    );
}

/// The all-families fixture, or `package`, with a clause leaf made
/// `let s = seq(0 .. 0: i) in <boolean>` (the `seq` binder `i` a
/// `value`/`parameter` node at the shape's level).
fn clause_leaf_seq(shape: Shape, mut package: Value) -> Value {
    let Shape {
        level,
        captures,
        trigger,
        place,
        wrap,
    } = shape;
    let nested = wrap == Wrap::Let;
    let in_guard = place == Place::Guard;
    let in_capture = match place {
        Place::Capture(index) => Some(index),
        _ => None,
    };
    let integer = family_key("a3a3");
    let boolean = family_key("aaaa");
    let text = family_key("a1a1");
    let literal = |ty: &str, kind: &str, value: Value| json!({"term": "literal", "type": node_id(ty), "value_kind": kind, "value": value});
    let add = |package: &mut Value, key: &str, tag: &str, form: &str, ty: &str, body: Value| {
        let mut node = node_with_key(key, tag, form, body);
        node["semantic_type"] = node_id(ty);
        node["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
        push(package, node);
    };
    let bound = |name: &str, value: &str| json!({"term": "binding", "name": name, "value": literal(&integer, "integer", json!(value))});
    let parameter = |name: &str, level: u64| {
        json!({"term": "aggregate", "members": [
            {"term": "binding", "name": "name", "value": literal(&text, "text", json!(name))},
            {"term": "binding", "name": "level",
             "value": literal(&integer, "integer", json!(level.to_string()))},
        ]})
    };
    let range = "bounded/integer_range/seq-range";
    let sequence = "composite/sequence/seq-range";
    let counted = "bounded/collection_bounds/seq-range";
    add(
        &mut package,
        range,
        "bounded_domain",
        "integer_range",
        &integer,
        json!({"term": "aggregate", "members": [bound("min", "0"), bound("max", "0")]}),
    );
    add(
        &mut package,
        sequence,
        "composite_type",
        "sequence",
        sequence,
        json!({"term": "aggregate", "members": [refer(range)]}),
    );
    add(
        &mut package,
        counted,
        "bounded_domain",
        "collection_bounds",
        sequence,
        json!({"term": "aggregate", "members": [bound("min", "0"), bound("max", "1")]}),
    );
    for key in ["seq-lower", "seq-upper"] {
        add(
            &mut package,
            key,
            "value",
            "literal",
            range,
            literal(range, "integer", json!("0")),
        );
    }
    add(
        &mut package,
        "seq-index",
        "value",
        "parameter",
        range,
        parameter("i", level),
    );
    add(
        &mut package,
        "seq-let-binder",
        "value",
        "parameter",
        counted,
        parameter("s", 1),
    );
    // The `fold` step form binds an integer value, not the sequence.
    add(
        &mut package,
        "seq-let-binder-int",
        "value",
        "parameter",
        &integer,
        parameter("s", 1),
    );
    add(
        &mut package,
        "seq-t-binder",
        "value",
        "parameter",
        &boolean,
        parameter("t", 1),
    );
    let seq_operation = json!({"identity": "filament.op.collection.seq", "laws": [], "mode": null,
        "member": {"kind": "type_argument", "declaration": node_id(range)}, "leaves": []});
    add(
        &mut package,
        "seq-application",
        "expression",
        "collection",
        counted,
        json!({
        "term": "application", "operator": "collection", "operation": seq_operation,
        "result_type": node_id(counted), "arguments": [
            refer("seq-lower"), refer("seq-upper"),
            {"term": "binding", "name": "i", "value": refer("seq-index")}]}),
    );
    let let_operation = json!({"identity": "filament.op.control.let", "laws": [], "mode": null,
        "member": null, "leaves": []});
    if nested {
        // `let t = <boolean> in seq(...)`: the `seq` sits in the body of a
        // `let`, one binder inside the bound value of the outer `let`.
        add(
            &mut package,
            "seq-inner",
            "expression",
            "let",
            counted,
            json!({
            "term": "application", "operator": "let", "operation": let_operation.clone(),
            "result_type": node_id(counted), "arguments": [
                {"term": "binding", "name": "t", "binder": node_id("seq-t-binder"),
                 "value": refer(&family_key("dddd"))},
                refer("seq-application")]}),
        );
    }
    add(
        &mut package,
        "seq-let",
        "expression",
        "let",
        &boolean,
        json!({
        "term": "application", "operator": "let", "operation": let_operation,
        "result_type": node_id(&boolean), "arguments": [
            {"term": "binding", "name": "s",
             "binder": node_id(if matches!(wrap, Wrap::Step(_)) { "seq-let-binder-int" } else { "seq-let-binder" }),
             "value": refer(if wrap == Wrap::Bare { "seq-application" } else { "seq-inner" })},
            refer(&family_key("dddd"))]}),
    );
    if let Wrap::Step(identity) = wrap {
        // `fold(acc, j in set: size(seq(...)), identity: 1)`: the step is
        // under the accumulator and the element binder.
        let member = json!({"kind": "type_argument", "declaration": node_id(&integer)});
        let operation = |identity: &str| {
            json!({"identity": identity, "laws": [], "mode": null,
                "member": member.clone(), "leaves": []})
        };
        add(
            &mut package,
            "seq-set",
            "composite_type",
            "set",
            "seq-set",
            json!({"term": "aggregate", "members": [refer(&integer)]}),
        );
        add(
            &mut package,
            "seq-size",
            "expression",
            "collection",
            &integer,
            json!({"term": "application", "operator": "collection",
                "operation": operation("filament.op.collection.size"),
                "result_type": node_id(&integer), "arguments": [refer("seq-application")]}),
        );
        let mut arguments = vec![
            refer("seq-set"),
            json!({"term": "binding", "name": "acc", "value": {
                "term": "aggregate", "members": [
                    {"term": "binding", "name": "j", "value": refer("seq-size")}]}}),
        ];
        if identity == "filament.op.collection.fold" {
            arguments.push(literal(&integer, "integer", json!("1")));
        }
        add(
            &mut package,
            "seq-inner",
            "expression",
            "collection",
            &integer,
            json!({"term": "application", "operator": "collection",
                "operation": operation(identity),
                "result_type": node_id(&integer), "arguments": arguments}),
        );
    }
    let holds = find_identity(&package, &temporal_identity("holds"));
    if place == Place::Formula {
        package["semantic_graph"]["nodes"][holds]["body"]["arguments"][0] = refer("seq-let");
    }
    if captures > 0 {
        let at = clause(&package);
        let members: Vec<Value> = (0..captures)
            .map(|index| {
                let value = if in_capture == Some(index) {
                    refer("seq-let")
                } else {
                    literal(&boolean, "boolean", json!(true))
                };
                json!({"term": "binding", "name": format!("c{index}"), "value": value})
            })
            .collect();
        package["semantic_graph"]["nodes"][at]["body"]["arguments"][3] =
            json!({"term": "aggregate", "members": members});
    }
    if trigger {
        add(
            &mut package,
            "seq-trigger",
            "value",
            "parameter",
            &boolean,
            parameter("t", 1),
        );
        let at = clause(&package);
        package["semantic_graph"]["nodes"][at]["body"]["arguments"][2] = json!({
            "term": "aggregate", "members": [
                {"term": "binding", "name": "kind", "value": literal(&text, "text", json!("each"))},
                {"term": "binding", "name": "trigger", "value": refer("seq-trigger")}]});
        if in_guard {
            package["semantic_graph"]["nodes"][at]["body"]["arguments"][2]["members"]
                .as_array_mut()
                .expect("members")
                .push(json!({"term": "binding", "name": "guard", "value": refer("seq-let")}));
        }
    }
    settle(&mut package);
    package
}

/// Where the `let` holding the `seq` stands in the clause.
#[derive(Clone, Copy, PartialEq)]
enum Place {
    /// The `holds` leaf of the clause's formula.
    Formula,
    /// The activation guard (needs a trigger).
    Guard,
    /// The initializer of this capture.
    Capture(usize),
}

/// What stands between the outer `let` and the `seq`.
#[derive(Clone, Copy, PartialEq)]
enum Wrap {
    /// Nothing: the `seq` is the bound value.
    Bare,
    /// The body of an inner `let`.
    Let,
    /// The step of a `fold` or `reduce`, as `size(seq(...))`.
    Step(&'static str),
}

/// A clause with `captures` captures, optionally an `each` trigger, and a
/// `seq` whose index binder carries `level`.
#[derive(Clone, Copy)]
struct Shape {
    level: u64,
    captures: usize,
    trigger: bool,
    place: Place,
    wrap: Wrap,
}

impl Shape {
    fn at(level: u64) -> Self {
        Self {
            level,
            captures: 0,
            trigger: false,
            place: Place::Formula,
            wrap: Wrap::Bare,
        }
    }
}

/// Asserts the shape admits at `level` and refuses one level either side.
fn admits_only_at(label: &str, level: u64, shape: impl Fn(u64) -> Shape, base: impl Fn() -> Value) {
    admitted(
        &format!("{label}: level {level}"),
        &clause_leaf_seq(shape(level), base()),
    );
    for wrong in [level.checked_sub(1), level.checked_add(1)]
        .into_iter()
        .flatten()
    {
        let package = clause_leaf_seq(shape(wrong), base());
        let seq = find_identity(&package, "filament.op.collection.seq");
        expect(
            &format!("{label}: level {wrong}"),
            &package,
            Code::IllTyped,
            Cause::OperatorIneligible,
            &at_node(seq, ""),
        );
    }
}

/// The binder rule of FSpec FR-322 "Parameter" for a `seq` binder at a
/// temporal clause leaf: its level is its position among the binders in
/// scope where it is bound (the clause's `over`, `each` trigger and captures,
/// then the binders of the leaf; the bound value of a `let` is outside its
/// binder and its body inside). FSL FR-337 gives the clause scopes: a capture
/// is not in the guard's scope, and capture `k` sees the captures before it.
#[test]
#[trace("TC-048", "FR-038-AC-598")]
fn tc_048_a_seq_binder_in_a_clause_leaf_resolves_its_level_from_the_binders_in_scope() {
    let base = v2_all_families;
    // A leaf under `over` alone: the bound value of a `let` is outside it.
    admits_only_at("formula leaf", 1, Shape::at, base);
    // In the body of a `let`, one binder further in.
    admits_only_at(
        "let body",
        2,
        |l| Shape {
            wrap: Wrap::Let,
            ..Shape::at(l)
        },
        base,
    );
    // A `fold` or `reduce` step is under its accumulator and element binders.
    for identity in [
        "filament.op.collection.fold",
        "filament.op.collection.reduce",
    ] {
        admits_only_at(
            identity,
            3,
            |l| Shape {
                wrap: Wrap::Step(identity),
                ..Shape::at(l)
            },
            base,
        );
    }
    // Each capture and the trigger are one more binder in the formula's scope.
    admits_only_at(
        "one capture",
        2,
        |l| Shape {
            captures: 1,
            ..Shape::at(l)
        },
        base,
    );
    admits_only_at(
        "trigger",
        2,
        |l| Shape {
            trigger: true,
            ..Shape::at(l)
        },
        base,
    );
    admits_only_at(
        "trigger and two captures",
        4,
        |l| Shape {
            trigger: true,
            captures: 2,
            ..Shape::at(l)
        },
        base,
    );
    // The guard is over `over` and the trigger only, whatever the captures.
    for captures in [0, 2] {
        admits_only_at(
            &format!("guard, {captures} captures"),
            2,
            |l| Shape {
                trigger: true,
                captures,
                place: Place::Guard,
                ..Shape::at(l)
            },
            base,
        );
    }
    // Capture k reads over, the trigger and the captures before k.
    for (capture, trigger, level) in [(0, false, 1), (1, false, 2), (0, true, 2), (1, true, 3)] {
        admits_only_at(
            &format!("capture {capture} initializer, trigger {trigger}"),
            level,
            |l| Shape {
                trigger,
                captures: 2,
                place: Place::Capture(capture),
                ..Shape::at(l)
            },
            base,
        );
    }
    // A function that also reads the leaf puts the binder at another level,
    // and one binder has one level.
    for level in [0, 1] {
        let mut package = clause_leaf_seq(Shape::at(level), v2_all_families());
        let mut owner = node_with_key(
            "seq-owner",
            "function",
            "pure_function",
            json!({"term": "aggregate", "members": [
                {"term": "binding", "name": "parameters",
                 "value": {"term": "aggregate", "members": []}},
                {"term": "binding", "name": "body", "value": refer("seq-let")}]}),
        );
        owner["semantic_type"] = node_id(&family_key("aaaa"));
        owner["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
        push(&mut package, owner);
        settle(&mut package);
        let seq = find_identity(&package, "filament.op.collection.seq");
        expect(
            &format!("two scopes, two levels, binder at {level}"),
            &package,
            Code::IllTyped,
            Cause::OperatorIneligible,
            &at_node(seq, ""),
        );
    }
    // A hyper body wrapper reads its leaves over its slot scope (FSpec FR-402
    // "Wrapper slot scope"): a `seq` binder there takes level `n` for the `n`
    // slots its formula reads, and is refused at every other level.
    let fixtures: [(u64, fn() -> Value); 3] = [
        (0, || hyper_clock_fixture(&["s", "n"]).0),
        (1, || slot_wrapper_fixture_with(&[(SLOT_A, "0")], true).0),
        (2, || {
            slot_wrapper_fixture_with(&[(SLOT_A, "0"), (SLOT_B, "1")], true).0
        }),
    ];
    for (slots, build) in fixtures {
        for level in 0..4 {
            let package = clause_leaf_seq(Shape::at(level), build());
            let case = format!("hyper wrapper, {slots} slots, binder at {level}");
            if level == slots {
                admitted(&case, &package);
            } else {
                let seq = find_identity(&package, "filament.op.collection.seq");
                expect(
                    &case,
                    &package,
                    Code::IllTyped,
                    Cause::OperatorIneligible,
                    &at_node(seq, ""),
                );
            }
        }
    }
}

/// FSpec FR-402-AC-12: the public `is_slot_parameter_name` recognizes exactly
/// the reserved slot form, as the reader does, non-canonical escapes included.
#[trace("TC-2082", "FR-402-AC-12")]
#[test]
fn is_slot_parameter_name_accepts_the_reserved_forms_and_refuses_near_misses() {
    use filament_contract_model::is_slot_parameter_name;
    for name in [
        "slot_sv_sa",
        "op_sa",
        "rcv_sa",
        "arg_sa_sVault_sstep_si",
        "arg_s1_s2_s3_s4",
        // `_` is `__`, any other byte `_x` and two uppercase hex digits.
        "slot_sa__b_sc",
        "slot_sa_sb__c",
        "op_s_xC3_xA9",
        "op_sa__",
        "op_s_x00",
        // Non-canonical escapes the decoder accepts are reserved too.
        "op_s_x41",
        "op_sx",
        // A field may itself begin with `s` or `x`.
        "op_ssx",
        "slot_sx_sx",
    ] {
        assert!(is_slot_parameter_name(name), "{name} is a slot parameter");
    }
    for name in [
        "",
        "slot",
        "slot_",
        "slotted",
        "operand",
        "Slot_sv_sa",
        "OP_sa",
        // The wrong field count for the tag.
        "slot_sv",
        "slot_sv_sa_sb",
        "op_sa_sb",
        "rcv_s",
        "arg_sa_sb_sc",
        "arg_sa_sb_sc_sd_se",
        // No `_s` after the tag, an empty field, a stray `_`.
        "op_a",
        "op_",
        "op__sa",
        "op_s",
        "op_sa_s",
        "op_sa_",
        // A bad or non-uppercase escape, a truncated escape, bad UTF-8.
        "op_sa_xzz",
        "op_sa_xGG",
        "op_sa_xZZ",
        "op_sa_xc3",
        "op_sa_x4",
        "op_sa_x4a",
        "op_s_xC3",
        "op_s_xFF",
        // A byte outside letters, digits and `_`.
        "op_sa-b",
        "op_s\u{e9}",
        "op_sa b",
        "op_sa_q",
        "op_sa_X41",
    ] {
        assert!(!is_slot_parameter_name(name), "{name} is no slot parameter");
    }
}

// ---------------------------------------------------------------------------
// FR-402-AC-14 / FR-402-AC-15 (TC-346 HI-19, HI-20): the wrapper slot scope
// ---------------------------------------------------------------------------

/// The two-trace hyper clock fixture whose body wrapper reads the slot
/// parameters `slots` (`name`, `level`), the first as its `over`: the
/// wrapper's position, the hyperproperty's and each slot's.
fn slot_wrapper_fixture(slots: &[(&str, &str)]) -> (Value, usize, usize, Vec<usize>) {
    slot_wrapper_fixture_with(slots, false)
}

/// As [`slot_wrapper_fixture`]; with `spare` the clause's own `holds` leaf
/// reads no slot and stays in the formula for a test to rewrite, each slot
/// read by a leaf of its own.
fn slot_wrapper_fixture_with(
    slots: &[(&str, &str)],
    spare: bool,
) -> (Value, usize, usize, Vec<usize>) {
    let (mut package, wrapper, hyper) = hyper_clock_fixture(&["a", "b"]);
    let template = find_form(&package, "value", "parameter");
    let holds = find_identity(&package, &temporal_identity("holds"));
    let eventually = find_identity(&package, &temporal_identity("eventually"));
    let mut positions = Vec::new();
    let mut leaves = Vec::new();
    for (at, (name, level)) in slots.iter().enumerate() {
        let mut slot = nodes(&package)[template].clone();
        slot["node_id"] = node_id(&fresh());
        slot["body"]["members"][0]["value"]["value"] = json!(name);
        slot["body"]["members"][1]["value"]["value"] = json!(level);
        let position = push(&mut package, slot);
        positions.push(position);
        let reading = if at == 0 && !spare {
            holds
        } else {
            let mut node = nodes(&package)[holds].clone();
            node["node_id"] = node_id(&fresh());
            push(&mut package, node)
        };
        package["semantic_graph"]["nodes"][reading]["body"]["arguments"][0] =
            refer(&node_label(&package, position));
        leaves.push(reading);
    }
    let mut tree = node_label(&package, if spare { holds } else { leaves[0] });
    for leaf in if spare { &leaves[..] } else { &leaves[1..] } {
        let leaf_key = node_label(&package, *leaf);
        let both = push(
            &mut package,
            formula("and", Value::Null, vec![refer(&tree), refer(&leaf_key)]),
        );
        tree = node_label(&package, both);
    }
    package["semantic_graph"]["nodes"][eventually]["body"]["arguments"][0] = refer(&tree);
    package["semantic_graph"]["nodes"][wrapper]["body"]["arguments"][0] =
        refer(&node_label(&package, positions[0]));
    settle(&mut package);
    (package, wrapper, hyper, positions)
}

const SLOT_A: &str = "slot_sv_sa";
const SLOT_B: &str = "slot_sv_sb";

/// A seq index witnesses the lexical depth inside each binder form: let and
/// forall add one level; fold/reduce reserve accumulator n and element n+1.
fn wrapper_binder_level_case(form: &str, level: u64) -> Value {
    let base = slot_wrapper_fixture_with(&[(SLOT_A, "0"), (SLOT_B, "1")], true).0;
    let wrap = match form {
        "let" => Wrap::Let,
        "fold" => Wrap::Step("filament.op.collection.fold"),
        "reduce" => Wrap::Step("filament.op.collection.reduce"),
        "forall" => Wrap::Bare,
        _ => panic!("unknown fixture binder"),
    };
    let mut package = clause_leaf_seq(
        Shape {
            wrap,
            ..Shape::at(level)
        },
        base,
    );
    // The let binder node records the same base whose nested seq witnesses.
    let binder = find(&package, |node| node["node_id"] == node_id("seq-t-binder"));
    package["semantic_graph"]["nodes"][binder]["body"]["members"][1]["value"]["value"] = json!("2");
    if form == "forall" {
        let integer = family_key("a3a3");
        let boolean = family_key("aaaa");
        let mut domain = node_with_key(
            "wrapper-quantifier-domain",
            "composite_type",
            "set",
            json!({"term":"aggregate", "members":[refer(&integer)]}),
        );
        domain["semantic_type"] = node_id("wrapper-quantifier-domain");
        push(&mut package, domain);
        let mut quantifier = node_with_key(
            "wrapper-quantifier",
            "expression",
            "quantify",
            json!({"term":"application", "operator":"quantify", "operation": {
                "identity":"filament.op.collection.forall", "laws":[], "mode":null, "member":null, "leaves":[]},
                "result_type":node_id(&boolean), "arguments":[refer("wrapper-quantifier-domain"),
                    {"term":"binding", "name":"q", "value":refer("seq-let")}]}),
        );
        quantifier["semantic_type"] = node_id(&boolean);
        push(&mut package, quantifier);
        let holds = find_identity(&package, &temporal_identity("holds"));
        package["semantic_graph"]["nodes"][holds]["body"]["arguments"][0] =
            refer("wrapper-quantifier");
    }
    settle(&mut package);
    package
}

#[trace("TC-048", "FR-038-AC-598")]
#[test]
fn hyper_wrapper_leaf_let_and_quantifier_start_after_the_slot_scope() {
    for form in ["let", "forall"] {
        for level in [2, 3, 4] {
            let package = wrapper_binder_level_case(form, level);
            if level == 3 {
                admitted(&format!("{form} binder 2, nested seq 3"), &package);
            } else {
                let seq = find_identity(&package, "filament.op.collection.seq");
                expect(
                    &format!("{form}: wrong nested seq level {level}"),
                    &package,
                    Code::IllTyped,
                    Cause::OperatorIneligible,
                    &at_node(seq, ""),
                );
            }
        }
    }
}

#[trace("TC-048", "FR-038-AC-598")]
#[test]
fn hyper_wrapper_leaf_fold_reserves_accumulator_and_element_levels() {
    for form in ["fold", "reduce"] {
        for level in [2, 3, 4, 5] {
            let package = wrapper_binder_level_case(form, level);
            if level == 4 {
                admitted(
                    &format!("{form}: accumulator 2, element 3, seq 4"),
                    &package,
                );
            } else {
                let seq = find_identity(&package, "filament.op.collection.seq");
                expect(
                    &format!("{form}: wrong step seq level {level}"),
                    &package,
                    Code::IllTyped,
                    Cause::OperatorIneligible,
                    &at_node(seq, ""),
                );
            }
        }
    }
}

#[trace("TC-048", "FR-038-AC-598")]
#[test]
fn hyper_wrapper_over_must_be_its_first_slot_even_for_an_ordinary_parameter() {
    let (mut package, wrapper, _, _) = slot_wrapper_fixture(&[(SLOT_A, "0"), (SLOT_B, "1")]);
    admitted("first scoped slot", &package);
    let ordinary = find_form(&package, "value", "parameter");
    assert!(
        !nodes(&package)[ordinary]["body"]["members"][0]["value"]["value"]
            .as_str()
            .expect("parameter name")
            .starts_with("slot_")
    );
    package["semantic_graph"]["nodes"][wrapper]["body"]["arguments"][0] =
        refer(&node_label(&package, ordinary));
    settle(&mut package);
    expect_at(
        "ordinary over with nonempty scope",
        &package,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        &at_node(wrapper, "/body/arguments/0"),
    );
}

#[trace("TC-048", "FR-038-AC-598")]
#[test]
fn hi_19_reversed_prefixes_share_one_wrapper_with_the_slot_scope() {
    let (mut package, wrapper, fwd, slots) = slot_wrapper_fixture(&[(SLOT_A, "0"), (SLOT_B, "1")]);
    let clause_key = node_label(&package, wrapper);
    let rev = push(
        &mut package,
        hyperproperty(
            "hyperproperty",
            vec![trace_group("b"), trace_group("a")],
            vec![],
            vec![],
            refer(&clause_key),
        ),
    );
    claim_occurrence(&mut package, rev);
    settle(&mut package);
    let admitted = admitted("Fwd and Rev over one wrapper", &package);
    let bodies = |position: usize| nodes(&package)[position]["body"]["members"][6]["value"].clone();
    assert_eq!(bodies(fwd)["target"], bodies(rev)["target"]);
    assert_eq!(slots.len(), 2);
    assert_eq!(admitted.temporal_bindings().expect("rows").len(), 4);
}

#[trace("TC-048", "FR-038-AC-598")]
#[test]
fn hi_20_a_wrapper_slot_off_its_scope_position_is_refused() {
    // (a) the two levels swapped, at the first slot in scope order.
    let (package, _, _, slots) = slot_wrapper_fixture(&[(SLOT_A, "1"), (SLOT_B, "0")]);
    expect_at(
        "levels swapped",
        &package,
        Code::InvalidSemanticGraph,
        None,
        &at_node(slots[0], "/body/members/1"),
    );
    // (c) a second node of one name.
    let (package, _, _, slots) =
        slot_wrapper_fixture(&[(SLOT_A, "0"), (SLOT_B, "1"), (SLOT_B, "2")]);
    expect_at(
        "a second slot_sv_sb",
        &package,
        Code::InvalidSemanticGraph,
        None,
        &at_node(slots[2], "/body/members/1"),
    );
    // The scope's levels right: admitted.
    let (package, ..) = slot_wrapper_fixture(&[(SLOT_A, "0"), (SLOT_B, "1")]);
    admitted("levels right", &package);
}

#[trace("TC-048", "FR-038-AC-598")]
#[test]
fn hi_20_over_on_a_later_slot_refuses_at_the_wrapper_argument() {
    let (mut package, wrapper, _, slots) = slot_wrapper_fixture(&[(SLOT_A, "0"), (SLOT_B, "1")]);
    package["semantic_graph"]["nodes"][wrapper]["body"]["arguments"][0] =
        refer(&node_label(&package, slots[1]));
    settle(&mut package);
    expect_at(
        "over on slot_sv_sb",
        &package,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        &at_node(wrapper, "/body/arguments/0"),
    );
}

// ---------------------------------------------------------------------------
// FR-322-AC-511: the hyper clause source unit on the claim_names row
// ---------------------------------------------------------------------------

fn expect_at(case: &str, package: &Value, code: Code, cause: Option<Cause>, path: &str) {
    let (outcome, _) = refusal_of(case, package);
    assert_eq!(outcome, (code, cause, Some(path.to_owned())), "{case}");
}

fn hyper_name_rows(package: &Value, hyper: usize) -> Vec<usize> {
    package["claim_names"]
        .as_array()
        .expect("claim_names")
        .iter()
        .enumerate()
        .filter(|(_, row)| row["declaration"] == nodes(package)[hyper]["node_id"])
        .map(|(at, _)| at)
        .collect()
}

#[trace("TC-048", "FR-038-AC-599")]
#[test]
fn a_hyper_claim_names_row_carries_its_locked_source_unit() {
    let (package, _, hyper) = hyper_clock_fixture(&["a", "b"]);
    let rows = hyper_name_rows(&package, hyper);
    assert_eq!(rows.len(), 1);
    assert_eq!(
        package["claim_names"][rows[0]]["source_unit"],
        package["lock"]["sources"][0]
    );
    admitted("a hyper row with its unit", &package);
    // The temporal_bindings rows carry none.
    assert!(package["temporal_bindings"]
        .as_array()
        .expect("rows")
        .iter()
        .all(|row| row.get("source_unit").is_none()));

    // Missing.
    let mut missing = package.clone();
    missing["claim_names"][rows[0]]
        .as_object_mut()
        .expect("row")
        .remove("source_unit");
    expect_at(
        "no source_unit",
        &missing,
        Code::InvalidPackage,
        Some(Cause::TimedNodeReference),
        &format!("/claim_names/{}", rows[0]),
    );
    // Foreign: not a lock source.
    let mut foreign = package.clone();
    foreign["claim_names"][rows[0]]["source_unit"] =
        json!({"authority": "agent-ix", "identity": "other"});
    expect_at(
        "a foreign unit",
        &foreign,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        &format!("/claim_names/{}/source_unit", rows[0]),
    );
    // On a row of another declaration.
    let other = package["claim_names"]
        .as_array()
        .expect("rows")
        .iter()
        .position(|row| row["declaration"] != nodes(&package)[hyper]["node_id"]);
    if let Some(other) = other {
        let mut stated = package.clone();
        stated["claim_names"][other]["source_unit"] = package["lock"]["sources"][0].clone();
        expect_at(
            "a unit on a state clause row",
            &stated,
            Code::InvalidPackage,
            Some(Cause::TimedNodeReference),
            &format!("/claim_names/{other}"),
        );
    }
    // On a temporal_bindings row.
    let mut on_trace = package.clone();
    on_trace["temporal_bindings"][0]["source_unit"] = package["lock"]["sources"][0].clone();
    refusal_at_trace(&on_trace, "/temporal_bindings/0");
}

#[trace("TC-048", "FR-038-AC-599")]
#[test]
fn every_claim_names_row_of_one_hyper_node_names_one_source_unit() {
    let (mut package, _, hyper) = hyper_clock_fixture(&["a", "b"]);
    package["semantic_graph"]["nodes"][hyper]["occurrences"] =
        json!([{"role": "claim", "ordinal": 0}, {"role": "claim", "ordinal": 1}]);
    settle(&mut package);
    rebuild_complete_source_map(&mut package);
    refresh_claim_names(&mut package);
    let rows = hyper_name_rows(&package, hyper);
    assert_eq!(rows.len(), 2);
    admitted("two rows, one unit", &package);
    let second = json!({"authority": "agent-ix", "identity": "second-unit"});
    package["lock"]["sources"]
        .as_array_mut()
        .expect("sources")
        .push(second.clone());
    package["claim_names"][rows[1]]["source_unit"] = second;
    expect_at(
        "a differing unit on the second row",
        &package,
        Code::InvalidPackage,
        Some(Cause::TimedNodeReference),
        &format!("/claim_names/{}", rows[1]),
    );
}

fn hyper_source_unit_id(name: &str, ordinal: u64) -> Value {
    let path = if ordinal == 0 {
        json!([name])
    } else {
        match name {
            "Time" => json!(["other", "Time"]),
            "ms" => json!(["us"]),
            "s" => json!(["other_s"]),
            _ => panic!("unknown fixture unit"),
        }
    };
    json!({"package": "test/families", "path": path, "ordinal": ordinal})
}

fn settle_hyper_sources(package: &mut Value) {
    settle(package);
    rebuild_complete_source_map(package);
    let sources: Vec<_> = nodes(package).iter().filter(|node| node["owner"]["kind"] == "source")
        .map(|node| (node["node_id"].clone(), json!({"authority":node["owner"]["authority"], "identity":node["owner"]["identity"]})))
        .collect();
    for row in package["source_map"].as_array_mut().expect("source rows") {
        if let Some((_, source)) = sources.iter().find(|(id, _)| row["node_id"] == *id) {
            for region in row["regions"].as_array_mut().expect("regions") {
                region["source"] = source.clone();
            }
        }
    }
}

/// Adds source declarations with distinct package-level qualified names.
fn add_hyper_source_units(package: &mut Value, source: &Value, ordinal: u64) {
    let dimension = hyper_source_unit_id("Time", ordinal);
    let owner = source_owner(source);
    let mut dimension_node = node_with_key(
        "hyper-Time",
        "scalar_type",
        "dimension",
        json!({"term": "dimension", "base": true, "terms": []}),
    );
    dimension_node["node_id"] = dimension.clone();
    dimension_node["semantic_type"] = dimension.clone();
    dimension_node["owner"] = owner.clone();
    dimension_node["declaration"] = json!({"qualified_name": dimension["path"]});
    dimension_node["occurrences"] = json!([{"role":"declaration","ordinal":0}]);
    push(package, dimension_node);
    for name in ["s", "ms"] {
        let target = (name == "ms").then(|| hyper_source_unit_id("s", ordinal));
        let mut unit = node_with_key(
            "hyper-unit",
            "scalar_type",
            "unit",
            json!({
            "term": "unit", "dimension_node_id": dimension,
            "target_unit_node_id": target,
            "scale": {"numerator":"1", "denominator": if name == "ms" {"1000"} else {"1"}},
            "offset": {"numerator":"0", "denominator":"1"}}),
        );
        unit["node_id"] = hyper_source_unit_id(name, ordinal);
        unit["semantic_type"] = dimension.clone();
        unit["dependencies"] = if let Some(target) = &target {
            json!([dimension, target])
        } else {
            json!([dimension])
        };
        unit["owner"] = owner.clone();
        unit["declaration"] = json!({"qualified_name": unit["node_id"]["path"]});
        unit["occurrences"] = json!([{"role":"declaration","ordinal":0}]);
        push(package, unit);
    }
}

/// A shared body wrapper with duration bounds typed by source 0's ms.
fn hyper_duration_fixture() -> (Value, usize, usize, usize) {
    let (mut package, wrapper, hyper) = hyper_clock_fixture(&["a", "b"]);
    select_profile(&mut package, DURATION);
    let source = package["lock"]["sources"][0].clone();
    add_hyper_source_units(&mut package, &source, 0);
    let formula = find_identity(&package, &temporal_identity("holds"));
    let bound = |value| typed("quantity", hyper_source_unit_id("ms", 0), value);
    package["semantic_graph"]["nodes"][formula]["body"]["operation"]["identity"] =
        json!("filament.op.temporal.aggregate");
    package["semantic_graph"]["nodes"][formula]["body"]["arguments"] = json!([
        {"term":"aggregate", "members": count_members("duration", bound("0"), bound("5"), scalar("integer", "0"))}]);
    settle_hyper_sources(&mut package);
    for row in package["temporal_bindings"]
        .as_array_mut()
        .expect("clock rows")
    {
        row["clock"]["timestamp_unit"] = json!("ms");
    }
    (package, wrapper, hyper, formula)
}

#[trace("TC-048", "FR-038-AC-599")]
#[test]
fn finite_hyper_timestamp_unit_resolves_in_its_claim_source() {
    let (mut package, hyper) = finite_clock_fixture();
    select_profile(&mut package, DURATION);
    let source = package["lock"]["sources"][0].clone();
    add_hyper_source_units(&mut package, &source, 0);
    settle_hyper_sources(&mut package);
    for row in package["temporal_bindings"].as_array_mut().expect("rows") {
        row["clock"]["timestamp_unit"] = json!("ms");
    }
    admitted("finite-domain hyper source ms", &package);
    let other = json!({"authority":"agent-ix", "identity":"hyper-other"});
    package["lock"]["sources"]
        .as_array_mut()
        .expect("sources")
        .push(other.clone());
    for at in hyper_name_rows(&package, hyper) {
        package["claim_names"][at]["source_unit"] = other.clone();
    }
    let row = hyper_rows(&package)[0];
    refusal_at(
        &package,
        Code::MissingDeclaration,
        Cause::MissingName,
        &format!("/temporal_bindings/{row}/clock/timestamp_unit"),
    );
}

#[trace("TC-048", "FR-038-AC-599")]
#[test]
fn hyper_wrapper_duration_resolves_only_in_its_claim_source_unit() {
    let (package, _, hyper, formula) = hyper_duration_fixture();
    admitted("source Time/s/ms and own duration bounds", &package);
    let bound_path = at_node(formula, "/body/arguments/0/members/3/value");
    let mut wrong = package.clone();
    wrong["semantic_graph"]["nodes"][formula]["body"]["arguments"][0]["members"][3]["value"]
        ["type"] = hyper_source_unit_id("s", 0);
    expect(
        "wrong unit in same source",
        &wrong,
        Code::IllTyped,
        Cause::UnitMismatch,
        &bound_path,
    );
    let mut foreign = package.clone();
    let other = json!({"authority":"agent-ix", "identity":"hyper-other"});
    foreign["lock"]["sources"]
        .as_array_mut()
        .expect("sources")
        .push(other.clone());
    for at in hyper_name_rows(&foreign, hyper) {
        foreign["claim_names"][at]["source_unit"] = other.clone();
    }
    let row = hyper_rows(&foreign)[0];
    refusal_at(
        &foreign,
        Code::MissingDeclaration,
        Cause::MissingName,
        &format!("/temporal_bindings/{row}/clock/timestamp_unit"),
    );
    let at = hyper_name_rows(&foreign, hyper)[0];
    foreign["claim_names"][at]
        .as_object_mut()
        .expect("name row")
        .remove("source_unit");
    expect_at(
        "missing source before unresolved timestamp unit",
        &foreign,
        Code::InvalidPackage,
        Some(Cause::TimedNodeReference),
        &format!("/claim_names/{at}"),
    );
}

#[trace("TC-048", "FR-038-AC-599")]
#[test]
fn shared_hyper_wrapper_checks_every_source_and_clock_in_both_orders() {
    for reverse in [false, true] {
        let (mut package, _, hyper, formula) = hyper_duration_fixture();
        let mut second = nodes(&package)[hyper].clone();
        second["node_id"] = node_id(&fresh());
        second["body"]["members"][3]["value"]["members"]
            .as_array_mut()
            .expect("quantifiers")
            .reverse();
        let second = push(&mut package, second);
        if reverse {
            order_pair(&mut package, second, hyper);
        } else {
            order_pair(&mut package, hyper, second);
        }
        // Both owners use the same source and clock, so the shared bounds fit both.
        settle(&mut package);
        rebuild_complete_source_map(&mut package);
        for row in package["temporal_bindings"].as_array_mut().expect("rows") {
            row["clock"]["timestamp_unit"] = json!("ms");
        }
        admitted("shared wrapper with compatible owners", &package);
        let mut other_clock = package.clone();
        let second_id = nodes(&package)[second]["node_id"].clone();
        for row in other_clock["temporal_bindings"]
            .as_array_mut()
            .expect("rows")
        {
            if row["declaration"] == second_id {
                row["clock"]["timestamp_unit"] = json!("s");
            }
        }
        expect(
            "shared wrapper's other clause has a different clock",
            &other_clock,
            Code::IllTyped,
            Cause::UnitMismatch,
            &at_node(formula, "/body/arguments/0/members/3/value"),
        );
        let other = json!({"authority":"agent-ix", "identity":"hyper-other"});
        package["lock"]["sources"]
            .as_array_mut()
            .expect("sources")
            .push(other.clone());
        add_hyper_source_units(&mut package, &other, 1);
        settle_hyper_sources(&mut package);
        for row in package["temporal_bindings"].as_array_mut().expect("rows") {
            row["clock"]["timestamp_unit"] = if row["declaration"] == second_id {
                json!("us")
            } else {
                json!("ms")
            };
        }
        for at in hyper_name_rows(&package, second) {
            package["claim_names"][at]["source_unit"] = other.clone();
        }
        expect(
            "second owner's us is a different unit",
            &package,
            Code::IllTyped,
            Cause::UnitMismatch,
            &at_node(formula, "/body/arguments/0/members/3/value"),
        );
        // Each clause alone admits with bounds typed by its own source's unit.
        for owner in [hyper, second] {
            let mut single = package.clone();
            let removed =
                nodes(&single)[if owner == hyper { second } else { hyper }]["node_id"].clone();
            single["semantic_graph"]["nodes"]
                .as_array_mut()
                .expect("nodes")
                .retain(|node| node["node_id"] != removed);
            let unit = if owner == hyper { 0 } else { 1 };
            for name in [3, 4] {
                single["semantic_graph"]["nodes"][formula]["body"]["arguments"][0]["members"]
                    [name]["value"]["type"] = hyper_source_unit_id("ms", unit);
            }
            settle_hyper_sources(&mut single);
            let own_id = nodes(&package)[owner]["node_id"].clone();
            for row in single["claim_names"].as_array_mut().expect("names") {
                if row["declaration"] == own_id && unit == 1 {
                    row["source_unit"] = other.clone();
                }
            }
            for row in single["temporal_bindings"].as_array_mut().expect("rows") {
                row["clock"]["timestamp_unit"] = if unit == 0 { json!("ms") } else { json!("us") };
            }
            admitted("each clause's own source unit fits", &single);
        }
    }
}
