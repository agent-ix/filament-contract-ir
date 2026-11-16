// SPDX-License-Identifier: AGPL-3.0-or-later
//! FSpec FR-322-AC-399 and TC-1361: a `Clock` tuple position and a `Clock`
//! variant payload are clock members, reached by clock places whose paths
//! carry `tuple(index)` and `union(case)` steps.
//!
//! The published timed-slot package is the base. Each case extends its
//! selected document with the tuple `Slot` (`p0` a `Clock`, `p1` an `Integer`)
//! and the variant `Phase` (`Armed` carrying a `Clock`, `Idle` bare), adds the
//! fields `s: Slot`, `p: Phase` and `ss: Set<Slot>[0, 2]` to `Order`, and adds
//! the clock places over them to the timed root.

use super::*;

const ORDER: &str = "ix://acme/orders/Order";
const SLOT: &str = "ix://acme/orders/Slot";
const PHASE: &str = "ix://acme/orders/Phase";
const ARMED: &str = "ix://acme/orders/variant/Phase-Armed";
const CLOCK: &str = "ix://filament/native/Clock";
const SET_OF_SLOT: &str = "Set<ix://acme/orders/Slot>";

fn text_type() -> Value {
    json!({"ordinal": 0, "package": "example", "path": ["scalar_type", "text"]})
}

fn integer_type() -> Value {
    json!({"ordinal": 0, "package": "example", "path": ["scalar_type", "integer"]})
}

fn order_node() -> Value {
    json!({"ordinal": 0, "package": "acme/orders", "path": ["model", "object_type", ORDER]})
}

fn text(value: &str) -> Value {
    json!({"term": "literal", "type": text_type(), "value": value, "value_kind": "text"})
}

fn integer(value: u64) -> Value {
    json!({"term": "literal", "type": integer_type(), "value": value, "value_kind": "integer"})
}

fn bind(name: &str, value: Value) -> Value {
    json!({"name": name, "term": "binding", "value": value})
}

fn aggregate(members: Vec<Value>) -> Value {
    json!({"members": members, "term": "aggregate"})
}

fn field(name: &str) -> Value {
    aggregate(vec![bind("kind", text("field")), bind("name", text(name))])
}

fn tuple(index: u64) -> Value {
    aggregate(vec![
        bind("kind", text("tuple")),
        bind("index", integer(index)),
    ])
}

fn union_case(case: &str) -> Value {
    aggregate(vec![bind("kind", text("union")), bind("case", text(case))])
}

fn slot(collection: &str, minimum: u64, maximum: u64) -> Value {
    aggregate(vec![
        bind("kind", text("slot")),
        bind("collection", text(collection)),
        bind(
            "selector",
            json!({"term": "reference", "target": integer_type()}),
        ),
        bind("minimum", integer(minimum)),
        bind("maximum", integer(maximum)),
    ])
}

fn row(
    step: u64,
    declaring: &str,
    identity: &str,
    member_type: &str,
    collection: &str,
    minimum: &str,
    maximum: &str,
) -> Value {
    aggregate(vec![
        bind("path_step", integer(step)),
        bind("declaring_type", text(declaring)),
        bind("member_identity", text(identity)),
        bind("member_type", text(member_type)),
        bind("collection_kind", text(collection)),
        bind("minimum", text(minimum)),
        bind("maximum", text(maximum)),
    ])
}

/// One clock place: its path steps and its `resolved_path` rows.
struct Place {
    name: &'static str,
    steps: Vec<Value>,
    rows: Vec<Value>,
}

fn slot_place() -> Place {
    Place {
        name: "s-tuple-0",
        steps: vec![field("sl"), tuple(0)],
        rows: vec![
            row(0, ORDER, "ix://acme/orders/Order/sl", SLOT, "none", "", ""),
            row(1, SLOT, "ix://acme/orders/Slot/p0", CLOCK, "none", "", ""),
        ],
    }
}

fn phase_place() -> Place {
    Place {
        name: "p-union-armed",
        steps: vec![field("ph"), union_case("Armed")],
        rows: vec![
            row(0, ORDER, "ix://acme/orders/Order/ph", PHASE, "none", "", ""),
            row(1, PHASE, ARMED, CLOCK, "none", "", ""),
        ],
    }
}

fn set_place() -> Place {
    Place {
        name: "ss-set-tuple-0",
        steps: vec![field("sls"), slot("set", 0, 2), tuple(0)],
        rows: vec![
            row(
                0,
                ORDER,
                "ix://acme/orders/Order/sls",
                SET_OF_SLOT,
                "set",
                "0",
                "2",
            ),
            row(
                1,
                SLOT,
                "Set<ix://acme/orders/Slot>[slot]",
                SLOT,
                "none",
                "",
                "",
            ),
            row(2, SLOT, "ix://acme/orders/Slot/p0", CLOCK, "none", "", ""),
        ],
    }
}

fn id_of(name: &str) -> Value {
    super::fixture_support::structural_id(
        "example",
        vec![json!("timed_model"), json!("clock_place"), json!(name)],
        0,
    )
}

/// The selected document with `Slot`, `Phase` and the three `Order` fields.
fn selected_document(proposal: &Path) -> Value {
    let mut document = read_json(&proposal.join("domain-package-acme-timed-slots.json"))
        .expect("selected timed-slot document");
    let origin = json!({"source": {"path": "orders/Order.md",
        "sourceIdentity": "ix://acme/orders/spec", "startColumn": 1, "startLine": 1}});
    let module = |name: &str| json!({"module": "acme/orders", "name": name});
    for (name, meaning, shape, member) in [
        (
            "tuple",
            "filament.meaning.model.tuple-value-type/v1",
            "tuple",
            "positions",
        ),
        (
            "variant",
            "filament.meaning.model.variant-type/v1",
            "enumeration",
            "variants",
        ),
    ] {
        document["constructs"]
            .as_array_mut()
            .expect("constructs")
            .push(json!({"kind": module(name), "moduleVersion": "1.0.0",
                "construct": {"identity": "value", "meaning": meaning,
                    "members": {member: "required"}, "shape": shape}}));
    }
    let types = document["types"].as_array_mut().expect("types");
    types.push(
        json!({"constraints": [], "displayName": "Slot", "extensions": [],
        "identity": SLOT, "kind": module("tuple"), "origin": origin, "roles": [],
        "unknownPolicy": "reject",
        "positions": [
            {"identity": "ix://acme/orders/Slot/p0", "typeRef": CLOCK, "origin": origin},
            {"identity": "ix://acme/orders/Slot/p1",
                "typeRef": "ix://filament/native/Integer", "origin": origin}]}),
    );
    types.push(
        json!({"constraints": [], "displayName": "Phase", "extensions": [],
        "identity": PHASE, "kind": module("variant"), "origin": origin, "roles": [],
        "unknownPolicy": "reject",
        "variants": [
            {"identity": ARMED, "name": "Armed", "origin": origin, "payloadType": CLOCK},
            {"identity": "ix://acme/orders/variant/Phase-Idle", "name": "Idle",
                "origin": origin}]}),
    );
    let order = types
        .iter_mut()
        .find(|ty| ty["identity"] == ORDER)
        .expect("Order");
    for (name, type_ref, lower, upper, ordered) in [
        ("sl", SLOT, 1, 1, false),
        ("ph", PHASE, 1, 1, false),
        ("sls", SLOT, 0, 2, false),
    ] {
        order["fields"].as_array_mut().expect("fields").push(json!({
            "defaultKind": "none",
            "identity": format!("{ORDER}/{name}"),
            "multiplicity": {"lower": lower, "ordered": ordered, "unique": true, "upper": upper},
            "name": name,
            "nullable": false,
            "origin": origin,
            "presence": "required",
            "typeRef": type_ref,
        }));
    }
    document
}

fn published() -> (Value, Value, PathBuf) {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let wire = read_json(&fixtures.join("positive-timed-slot-nodes.json"))
        .expect("published timed-slot package");
    (wire, selected_document(&proposal), proposal)
}

/// Adds `place` to the timed root as a new `clock_place` node and returns its id.
fn add_place(wire: &mut Value, place: &Place, region: u64) -> Value {
    let id = id_of(place.name);
    let mut dependencies = vec![order_node()];
    if place
        .steps
        .iter()
        .any(|step| step.to_string().contains("slot"))
    {
        dependencies.push(integer_type());
    }
    let node = json!({
        "body": aggregate(vec![
            bind("model", json!({"term": "reference", "target": order_node()})),
            bind("path", aggregate(place.steps.clone())),
            bind("resolved_path", aggregate(place.rows.clone())),
        ]),
        "dependencies": dependencies,
        "node_id": id,
        "node_tag": "timed_model",
        "occurrences": [{"ordinal": 0, "role": "expression"}],
        "schema_version": "filament.checked-semantic-graph/v2",
        "semantic_form": "clock_place",
        "semantic_type": id,
    });
    let graph = wire["semantic_graph"].as_object_mut().expect("graph");
    graph["nodes"].as_array_mut().expect("nodes").push(node);
    wire["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(json!({
        "node_id": id, "ordinal": 0, "role": "expression",
        "regions": [{"end": region + 1, "start": region,
            "source": {"authority": "agent-ix", "identity": "example"}}]}));
    let root = wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .iter_mut()
        .find(|node| node["semantic_form"] == "timed_model")
        .expect("timed root");
    root["body"]["members"][3]["value"]["members"]
        .as_array_mut()
        .expect("clock places")
        .push(json!({"term": "reference", "target": id}));
    root["dependencies"]
        .as_array_mut()
        .expect("root dependencies")
        .push(id.clone());
    let dependencies = root["dependencies"].as_array_mut().expect("dependencies");
    dependencies.sort_by_cached_key(|dependency| {
        serde_json::from_value::<filament_contract_model::CheckedNodeId>(dependency.clone())
            .expect("typed node id")
    });
    id
}

fn read(wire: &Value, document: &Value, proposal: &Path, name: &str) -> CheckedPackageV2ReadResult {
    let bytes = serde_json::to_vec(document).expect("selected document bytes");
    read_package_with_document_override(
        wire,
        name,
        &proposal.join("domain-package-acme-timed-slots.json"),
        true,
        CheckedPackageReadLimits::bounded(),
        Some(&bytes),
    )
    .expect("selected timed evidence")
}

fn admitted_with(places: &[Place]) -> (Value, Value, PathBuf) {
    let (mut wire, document, proposal) = published();
    for (index, place) in places.iter().enumerate() {
        add_place(&mut wire, place, 9000 + index as u64);
    }
    (wire, document, proposal)
}

fn node_at<'a>(wire: &'a mut Value, form: &str) -> &'a mut Value {
    wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .iter_mut()
        .find(|node| node["semantic_form"] == form)
        .unwrap_or_else(|| panic!("published {form} node"))
}

/// Appends `node` and its source-map entry, returning its position.
fn push_node(wire: &mut Value, node: Value, region: u64) -> usize {
    let id = node["node_id"].clone();
    let role = node["occurrences"][0]["role"].clone();
    let nodes = wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    // The model root stays the last node of the graph.
    let position = nodes.len() - 1;
    nodes.insert(position, node);
    wire["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(json!({
        "node_id": id, "ordinal": 0, "role": role,
        "regions": [{"end": region + 1, "start": region,
            "source": {"authority": "agent-ix", "identity": "example"}}]}));
    position
}

fn declaration_node(form: &str, identity: &str) -> Value {
    let id = json!({"ordinal": 0, "package": "acme/orders", "path": ["model", form, identity]});
    json!({
        "body": {"members": [], "term": "aggregate"},
        "dependencies": [],
        "node_id": id,
        "node_tag": "model",
        "occurrences": [{"ordinal": 0, "role": "expression"}],
        "owner": {"identity": "acme/orders", "kind": "model", "node": identity},
        "schema_version": "filament.checked-semantic-graph/v2",
        "semantic_form": form,
        "semantic_type": id,
    })
}

fn declaration_id(form: &str, identity: &str) -> Value {
    declaration_node(form, identity)["node_id"].clone()
}

fn add_declarations(wire: &mut Value) {
    push_node(wire, declaration_node("tuple_value_type", SLOT), 9100);
    push_node(wire, declaration_node("variant_type", PHASE), 9101);
}

/// A `value`/`union_value` of `Phase` selecting `case` over `payload` terms.
fn phase_value(case: &str, payload: Vec<Value>, dependencies: Vec<Value>) -> Value {
    let id = json!({"ordinal": 0, "package": "example",
        "path": ["value", "union_value", case, "phase"]});
    json!({
        "body": aggregate(vec![bind(case, aggregate(payload))]),
        "dependencies": dependencies,
        "node_id": id,
        "node_tag": "value",
        "occurrences": [{"ordinal": 0, "role": "expression"}],
        "schema_version": "filament.checked-semantic-graph/v2",
        "semantic_form": "union_value",
        "semantic_type": declaration_id("variant_type", PHASE),
    })
}

fn describe(result: &CheckedPackageV2ReadResult) -> String {
    match result {
        CheckedPackageV2ReadResult::Admitted(_) => "admitted".into(),
        CheckedPackageV2ReadResult::Refused(refusal) => format!(
            "{:?} {} {:?} {:?}",
            refusal.document_pointer,
            code_word(refusal.code),
            refusal.cause.map(cause_word),
            refusal
                .loci
                .first()
                .and_then(|locus| locus.path.as_ref())
                .map(ToString::to_string),
        ),
        _ => "other".into(),
    }
}

fn map_type(identity: &str, keys: &str) -> Value {
    json!({"constraints": [], "displayName": "M", "extensions": [], "identity": identity,
        "kind": "map", "keys": keys, "values": "ix://filament/native/Integer",
        "bound": {"lower": 0}, "unknownPolicy": "reject", "roles": [],
        "origin": {"source": {"path": "orders/M.md",
            "sourceIdentity": "ix://acme/orders/spec", "startColumn": 1, "startLine": 1}}})
}

fn sort_ids(ids: &mut [Value]) {
    ids.sort_by_cached_key(|id| {
        serde_json::from_value::<filament_contract_model::CheckedNodeId>(id.clone())
            .expect("typed node id")
    });
}

/// The reset of `place` beside the frame that writes `Order.ph`: a copy of the
/// published reset with no slot binders, written by the published anchor whose
/// frame also modifies `written`.
fn add_reset(wire: &mut Value, place: &Value, written: &str) {
    let mut reset = node_at(wire, "clock_reset").clone();
    let id = json!({"ordinal": 0, "package": "example", "path": ["timed_model", "clock_reset", "phase"]});
    let old_place = reset["body"]["members"][2]["value"]["target"].clone();
    let slot = reset["body"]["members"][4]["value"]["members"][0]["target"].clone();
    reset["body"]["members"][2]["value"]["target"] = place.clone();
    reset["body"]["members"][4]["value"]["members"] = json!([]);
    let mut dependencies: Vec<Value> = reset["dependencies"]
        .as_array()
        .expect("reset dependencies")
        .iter()
        .filter(|dependency| **dependency != old_place && **dependency != slot)
        .cloned()
        .collect();
    dependencies.push(place.clone());
    sort_ids(&mut dependencies);
    reset["dependencies"] = json!(dependencies);
    reset["node_id"] = id.clone();
    reset["semantic_type"] = id.clone();
    push_node(wire, reset, 9300);
    let root = node_at(wire, "timed_model");
    root["body"]["members"][4]["value"]["members"]
        .as_array_mut()
        .expect("resets")
        .push(json!({"term": "reference", "target": id}));
    let mut root_dependencies = root["dependencies"].as_array().expect("deps").clone();
    root_dependencies.push(id);
    sort_ids(&mut root_dependencies);
    root["dependencies"] = json!(root_dependencies);
    let frame = wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .iter_mut()
        .find(|node| {
            node["semantic_form"] == "frame" && node["body"]["modifies"][0]["name"] == "clock"
        })
        .expect("anchor frame");
    frame["body"]["modifies"]
        .as_array_mut()
        .expect("modifies")
        .push(
            json!({"declaration": order_node(), "kind": "field", "name": written,
            "scope": "every-object"}),
        );
}

fn project(position: u64, result: Value) -> Vec<Value> {
    let parameter = json!({
        "body": aggregate(vec![bind("name", text("sl")), bind("level", json!({"term": "literal", "type": integer_type(), "value": "0", "value_kind": "integer"}))]),
        "dependencies": [],
        "node_id": {"ordinal": 0, "package": "example", "path": ["value", "parameter", "sl"]},
        "node_tag": "value",
        "occurrences": [{"ordinal": 0, "role": "expression"}],
        "schema_version": "filament.checked-semantic-graph/v2",
        "semantic_form": "parameter",
        "semantic_type": declaration_id("tuple_value_type", SLOT),
    });
    let reference = json!({"term": "reference", "target": parameter["node_id"]});
    let mut dependencies = vec![
        declaration_id("tuple_value_type", SLOT),
        parameter["node_id"].clone(),
    ];
    sort_ids(&mut dependencies);
    let application = json!({
        "body": {
            "arguments": [reference],
            "operation": {"identity": "filament.op.tuple.project", "laws": [], "leaves": [],
                "member": {"declaration": declaration_id("tuple_value_type", SLOT),
                    "kind": "position", "position": position},
                "mode": null},
            "operator": "query",
            "result_type": result,
            "term": "application"},
        "dependencies": dependencies,
        "node_id": {"ordinal": 0, "package": "example",
            "path": ["expression", "query", "project", position.to_string()]},
        "node_tag": "expression",
        "occurrences": [{"ordinal": 0, "role": "expression"}],
        "schema_version": "filament.checked-semantic-graph/v2",
        "semantic_form": "query",
        "semantic_type": result,
    });
    vec![parameter, application]
}

fn refused(
    result: CheckedPackageV2ReadResult,
    label: &str,
) -> filament_contract_model::CheckedPackageRefusal {
    match result {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("{label}: expected a refusal, got {}", describe(&other)),
    }
}

fn pointer_of(refusal: &filament_contract_model::CheckedPackageRefusal) -> String {
    refusal.loci[0]
        .path
        .as_ref()
        .expect("refusal pointer")
        .to_string()
}

fn assert_no_presence_member(term: &Value) {
    match term {
        Value::Object(members) => {
            assert_ne!(
                members.get("name").and_then(Value::as_str),
                Some("presence"),
                "a clock place carries no presence bit"
            );
            members.values().for_each(assert_no_presence_member);
        }
        Value::Array(values) => values.iter().for_each(assert_no_presence_member),
        _ => {}
    }
}

/// FR-322-AC-399: a `Clock` tuple position and a `Clock` variant payload are
/// admitted as clock members of the selected document (TK-08, TK-09), and the
/// clock places over them admit with a row per step ending at the `Clock`
/// terminal. No node stands for a clock, the place carries no presence bit,
/// and the `Slot` and `Phase` declaration nodes keep their empty bodies.
#[trace("TC-048", "FR-038-AC-548", "FR-069-AC-9")]
#[test]
fn clock_tuple_position_and_union_payload_places_admit_without_a_clock_node() {
    let (mut wire, document, proposal) = admitted_with(&[slot_place(), phase_place(), set_place()]);
    add_declarations(&mut wire);
    let CheckedPackageV2ReadResult::Admitted(package) = read(&wire, &document, &proposal, "places")
    else {
        panic!("the clock tuple position and union payload places must admit");
    };
    let nodes = &package.resolved_graph().nodes;
    assert!(
        !nodes
            .iter()
            .any(|node| node.semantic_form.as_ref() == "clock"),
        "no node stands for a clock"
    );
    let places: Vec<_> = nodes
        .iter()
        .filter(|node| node.semantic_form.as_ref() == "clock_place")
        .filter(|node| {
            serde_json::to_value(&node.body)
                .expect("place body")
                .to_string()
                .contains("Slot")
                || serde_json::to_value(&node.body)
                    .expect("place body")
                    .to_string()
                    .contains("Phase")
        })
        .collect();
    assert_eq!(
        places.len(),
        3,
        "the three published aggregate clock places"
    );
    for place in places {
        let body = serde_json::to_value(&place.body).expect("place body");
        assert_eq!(body["members"].as_array().map(Vec::len), Some(3));
        assert_no_presence_member(&body);
        let steps = body["members"][1]["value"]["members"]
            .as_array()
            .expect("path")
            .len();
        let rows = body["members"][2]["value"]["members"]
            .as_array()
            .expect("rows")
            .len();
        assert_eq!(steps, rows, "one resolved_path row for each path step");
    }
    let phase = nodes
        .iter()
        .find(|node| {
            node.semantic_form.as_ref() == "variant_type"
                && node.node_id.package.as_ref() == "acme/orders"
        })
        .expect("Phase declaration");
    let body = package
        .model_variant_body(&phase.node_id)
        .expect("selected Phase");
    assert_eq!(body.family(), "union");
    let arities: Vec<_> = body
        .cases()
        .map(|case| (case.name().to_owned(), case.arity()))
        .collect();
    assert_eq!(
        arities,
        vec![("Armed".to_owned(), 0), ("Idle".to_owned(), 0)],
        "a clock payload is payload arity 0 in every data read"
    );
    assert!(matches!(
        package.model_tuple_position_type(
            &nodes
                .iter()
                .find(|node| node.semantic_form.as_ref() == "tuple_value_type"
                    && node.node_id.package.as_ref() == "acme/orders")
                .expect("Slot declaration")
                .node_id,
            0
        ),
        Some(filament_contract_model::CheckedMemberType::Clock)
    ));
    for form in ["tuple_value_type", "variant_type"] {
        let declaration = nodes
            .iter()
            .find(|node| {
                node.semantic_form.as_ref() == form
                    && node.node_id.package.as_ref() == "acme/orders"
            })
            .expect("declaration node");
        assert_eq!(
            serde_json::to_value(&declaration.body).expect("body"),
            json!({"members": [], "term": "aggregate"}),
            "{form} is unchanged by its clock member"
        );
    }
}

/// Each defect of a tuple or union step refuses `malformed-clock-place` at the
/// first row that does not join the selected declaration.
#[trace("TC-048", "FR-038-AC-548")]
#[test]
fn forged_tuple_and_union_clock_place_rows_refuse_at_the_differing_row() {
    let forged =
        |name: &'static str, steps: Vec<Value>, rows: Vec<Value>| Place { name, steps, rows };
    let cases = [
        (
            "position past the tuple",
            forged(
                "tuple-2",
                vec![field("sl"), tuple(2)],
                vec![
                    row(0, ORDER, "ix://acme/orders/Order/sl", SLOT, "none", "", ""),
                    row(1, SLOT, "ix://acme/orders/Slot/p2", CLOCK, "none", "", ""),
                ],
            ),
            None,
        ),
        (
            "another position's identity",
            forged(
                "tuple-0-p1",
                vec![field("sl"), tuple(0)],
                vec![
                    row(0, ORDER, "ix://acme/orders/Order/sl", SLOT, "none", "", ""),
                    row(1, SLOT, "ix://acme/orders/Slot/p1", CLOCK, "none", "", ""),
                ],
            ),
            None,
        ),
        (
            "data position as the terminal",
            forged(
                "tuple-1",
                vec![field("sl"), tuple(1)],
                vec![
                    row(0, ORDER, "ix://acme/orders/Order/sl", SLOT, "none", "", ""),
                    row(
                        1,
                        SLOT,
                        "ix://acme/orders/Slot/p1",
                        "ix://filament/native/Integer",
                        "none",
                        "",
                        "",
                    ),
                ],
            ),
            None,
        ),
        (
            "nullary case",
            forged(
                "union-idle",
                vec![field("ph"), union_case("Idle")],
                vec![
                    row(0, ORDER, "ix://acme/orders/Order/ph", PHASE, "none", "", ""),
                    row(
                        1,
                        PHASE,
                        "ix://acme/orders/variant/Phase-Idle",
                        CLOCK,
                        "none",
                        "",
                        "",
                    ),
                ],
            ),
            None,
        ),
        (
            "undeclared case",
            forged(
                "union-gone",
                vec![field("ph"), union_case("Gone")],
                vec![
                    row(0, ORDER, "ix://acme/orders/Order/ph", PHASE, "none", "", ""),
                    row(
                        1,
                        PHASE,
                        "ix://acme/orders/variant/Phase-Gone",
                        CLOCK,
                        "none",
                        "",
                        "",
                    ),
                ],
            ),
            None,
        ),
        (
            "another case's identity",
            forged(
                "union-armed-idle-identity",
                vec![field("ph"), union_case("Armed")],
                vec![
                    row(0, ORDER, "ix://acme/orders/Order/ph", PHASE, "none", "", ""),
                    row(
                        1,
                        PHASE,
                        "ix://acme/orders/variant/Phase-Idle",
                        CLOCK,
                        "none",
                        "",
                        "",
                    ),
                ],
            ),
            None,
        ),
        (
            "position no longer a clock in the document",
            slot_place(),
            Some((
                "/types/4/positions/0/typeRef",
                "ix://filament/native/Integer",
            )),
        ),
        (
            "payload no longer a clock in the document",
            phase_place(),
            Some((
                "/types/5/variants/0/payloadType",
                "ix://filament/native/Integer",
            )),
        ),
        (
            "set bound different from the field's",
            forged(
                "set-bound-3",
                vec![field("sls"), slot("set", 0, 3), tuple(0)],
                set_place().rows,
            ),
            None,
        ),
    ];
    for (label, place, edit) in cases {
        let (mut wire, mut document, proposal) = published();
        let id = add_place(&mut wire, &place, 9000);
        let position = wire["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .position(|node| node["node_id"] == id)
            .expect("added place");
        if let Some((pointer, value)) = edit {
            *document.pointer_mut(pointer).expect("selected member") = json!(value);
        }
        let refusal = refused(read(&wire, &document, &proposal, label), label);
        assert_eq!(
            (refusal.code, refusal.cause),
            (Code::InvalidPackage, Some(Cause::MalformedClockPlace)),
            "{label}"
        );
        let row = if label == "set bound different from the field's" {
            0
        } else {
            1
        };
        assert_eq!(
            pointer_of(&refusal),
            format!("/semantic_graph/nodes/{position}/body/members/2/value/members/{row}"),
            "{label}"
        );
    }
}

/// `Phase::Armed` is a `union_value` with the empty payload `aggregate`; a
/// payload term on the clock case is no construction of the member.
#[trace("TC-048", "FR-038-AC-549")]
#[test]
fn union_value_of_a_clock_case_carries_the_empty_payload() {
    for (case, payload) in [("Armed", false), ("Idle", false), ("Armed", true)] {
        let (mut wire, document, proposal) = admitted_with(&[]);
        add_declarations(&mut wire);
        let literal = node_at(&mut wire, "literal")["node_id"].clone();
        let terms = if payload {
            vec![json!({"term": "reference", "target": literal})]
        } else {
            vec![]
        };
        let dependencies = if payload { vec![literal] } else { vec![] };
        let position = push_node(&mut wire, phase_value(case, terms, dependencies), 9200);
        let result = read(&wire, &document, &proposal, case);
        if !payload {
            assert_eq!(
                describe(&result),
                "admitted",
                "{case} with the empty payload"
            );
            continue;
        }
        let refusal = refused(result, "clock case with a payload");
        assert_eq!(
            (refusal.code, refusal.cause),
            (Code::IllTyped, Some(Cause::TypeMismatch))
        );
        assert_eq!(
            pointer_of(&refusal),
            format!("/semantic_graph/nodes/{position}")
        );
    }
}

/// The postcondition `self.p = Phase::Armed(0 ms)` is a `clock_reset` of the
/// `Armed` place beside the `union_value` with the empty payload; the reset
/// needs the operation's frame to write `p`.
#[trace("TC-048", "FR-038-AC-550")]
#[test]
fn writing_a_clock_union_case_is_a_reset_of_its_place() {
    let build = |written: &str| {
        let (mut wire, document, proposal) = admitted_with(&[phase_place()]);
        add_declarations(&mut wire);
        push_node(&mut wire, phase_value("Armed", vec![], vec![]), 9200);
        add_reset(&mut wire, &id_of("p-union-armed"), written);
        (wire, document, proposal)
    };
    let (wire, document, proposal) = build("ph");
    let CheckedPackageV2ReadResult::Admitted(package) = read(&wire, &document, &proposal, "reset")
    else {
        panic!("the reset of the Armed place must admit");
    };
    let nodes = &package.resolved_graph().nodes;
    assert!(nodes
        .iter()
        .any(|node| node.semantic_form.as_ref() == "clock_reset"
            && serde_json::to_value(&node.body)
                .expect("reset body")
                .to_string()
                .contains("p-union-armed")));
    assert!(!nodes
        .iter()
        .any(|node| node.semantic_form.as_ref() == "clock"));

    let (wire, document, proposal) = build("sl");
    let refusal = refused(read(&wire, &document, &proposal, "unwritten"), "unwritten");
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::TimedNodeReference))
    );
}

/// `filament.op.tuple.project` of a clock position in a value position refuses
/// `invalid_timed_model`/`clock-read-outside-constraint` at the application; a
/// data position of the same tuple projects.
#[trace("TC-048", "FR-038-AC-551")]
#[test]
fn projecting_a_clock_tuple_position_refuses_clock_read_outside_constraint() {
    for (position, clock) in [(1, false), (0, true)] {
        let (mut wire, document, proposal) = admitted_with(&[]);
        add_declarations(&mut wire);
        let nodes = project(position, integer_type());
        let mut at = 0;
        for (offset, node) in nodes.into_iter().enumerate() {
            at = push_node(&mut wire, node, 9400 + offset as u64);
        }
        let result = read(&wire, &document, &proposal, "project");
        if !clock {
            assert_eq!(describe(&result), "admitted", "position {position}");
            continue;
        }
        let refusal = refused(result, "clock position");
        assert_eq!(
            (refusal.code, refusal.cause),
            (
                Code::InvalidTimedModel,
                Some(Cause::ClockReadOutsideConstraint)
            )
        );
        assert_eq!(pointer_of(&refusal), format!("/semantic_graph/nodes/{at}"));
    }
}

/// A map keyed by the clock-bearing `Slot` or `Phase` refuses
/// `ill_typed`/`operator-ineligible` at its key type; a scalar key admits.
#[trace("TC-048", "FR-038-AC-553")]
#[test]
fn map_keyed_by_a_clock_bearing_type_refuses_at_the_key() {
    for (keys, admitted) in [
        (SLOT, false),
        (PHASE, false),
        ("ix://filament/native/Text", true),
    ] {
        let (wire, mut document, proposal) = admitted_with(&[]);
        let types = document["types"].as_array_mut().expect("types");
        let at = types.len();
        types.push(map_type("ix://acme/orders/KeyedMap", keys));
        let result = read(&wire, &document, &proposal, keys);
        if admitted {
            assert_eq!(describe(&result), "admitted", "{keys}");
            continue;
        }
        let refusal = refused(result, keys);
        assert_eq!(
            (refusal.code, refusal.cause),
            (Code::IllTyped, Some(Cause::OperatorIneligible)),
            "{keys}"
        );
        assert_eq!(
            refusal.document_pointer.as_ref().map(ToString::to_string),
            Some(format!("/types/{at}/keys")),
            "{keys}"
        );
    }
}
