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
const PAIR: &str = "ix://acme/orders/Pair";
const WRAP: &str = "ix://acme/orders/Wrap";
const HELD: &str = "ix://acme/orders/Held";
const TIMER: &str = "ix://acme/orders/Timer";
const PLAIN_REC: &str = "ix://acme/orders/PlainRec";
const OPT_SLOT: &str = "ix://acme/orders/OptSlot";
const OPT_TIMER: &str = "ix://acme/orders/OptTimer";
const OPT_INT: &str = "ix://acme/orders/OptInt";
const OPT_POS: &str = "ix://acme/orders/OptPos";
const REC_POS: &str = "ix://acme/orders/RecPos";
const OPT_REC_POS: &str = "ix://acme/orders/OptRecPos";
const PLAIN_OPT_POS: &str = "ix://acme/orders/PlainOptPos";
const PLAIN_REC_POS: &str = "ix://acme/orders/PlainRecPos";
const REC_HELD: &str = "ix://acme/orders/RecHeld";
const PLAIN_HELD: &str = "ix://acme/orders/PlainHeld";
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
    let tuple_type = |name: &str, identity: &str, positions: [&str; 2]| {
        let rows: Vec<Value> = positions
            .iter()
            .enumerate()
            .map(|(index, type_ref)| {
                json!({"identity": format!("{identity}/p{index}"),
                    "typeRef": type_ref, "origin": origin})
            })
            .collect();
        json!({"constraints": [], "displayName": name, "extensions": [],
            "identity": identity, "kind": module("tuple"), "origin": origin, "roles": [],
            "unknownPolicy": "reject", "positions": rows})
    };
    types.push(tuple_type(
        "Pair",
        PAIR,
        [
            "ix://filament/native/Integer",
            "ix://filament/native/Integer",
        ],
    ));
    // A tuple whose clock is one level down, inside the `Slot` position.
    types.push(tuple_type(
        "Wrap",
        WRAP,
        [SLOT, "ix://filament/native/Integer"],
    ));
    // A union whose clock is one level down, inside the `Slot` payload.
    types.push(
        json!({"constraints": [], "displayName": "Held", "extensions": [],
        "identity": HELD, "kind": module("variant"), "origin": origin, "roles": [],
        "unknownPolicy": "reject",
        "variants": [
            {"identity": "ix://acme/orders/variant/Held-Has", "name": "Has",
                "origin": origin, "payloadType": SLOT},
            {"identity": "ix://acme/orders/variant/Held-None", "name": "Nothing",
                "origin": origin}]}),
    );
    // Options and records that do or do not carry a clock, each used as a
    // tuple position or a union payload.
    let option = |name: &str, identity: &str, inner: &str| {
        json!({"constraints": [], "displayName": name, "extensions": [],
            "identity": identity, "kind": "option", "origin": origin, "roles": [],
            "unknownPolicy": "reject", "inner": inner})
    };
    types.push(option("OptSlot", OPT_SLOT, SLOT));
    types.push(option("OptTimer", OPT_TIMER, TIMER));
    types.push(option("OptInt", OPT_INT, "ix://filament/native/Integer"));
    types.push(
        json!({"constraints": [], "displayName": "PlainRec", "extensions": [],
        "identity": PLAIN_REC, "kind": module("record"), "origin": origin, "roles": [],
        "unknownPolicy": "reject",
        "fields": [{"defaultKind": "none", "identity": format!("{PLAIN_REC}/n"),
            "multiplicity": {"lower": 1, "ordered": false, "unique": true, "upper": 1},
            "name": "n", "nullable": false, "origin": origin, "presence": "required",
            "typeRef": "ix://filament/native/Integer"}]}),
    );
    for (name, identity, first) in [
        ("OptPos", OPT_POS, OPT_SLOT),
        ("RecPos", REC_POS, TIMER),
        ("OptRecPos", OPT_REC_POS, OPT_TIMER),
        ("PlainOptPos", PLAIN_OPT_POS, OPT_INT),
        ("PlainRecPos", PLAIN_REC_POS, PLAIN_REC),
    ] {
        types.push(tuple_type(
            name,
            identity,
            [first, "ix://filament/native/Integer"],
        ));
    }
    for (name, identity, payload) in [
        ("RecHeld", REC_HELD, TIMER),
        ("PlainHeld", PLAIN_HELD, PLAIN_REC),
    ] {
        types.push(
            json!({"constraints": [], "displayName": name, "extensions": [],
            "identity": identity, "kind": module("variant"), "origin": origin, "roles": [],
            "unknownPolicy": "reject",
            "variants": [
                {"identity": format!("{identity}/Has"), "name": "Has",
                    "origin": origin, "payloadType": payload},
                {"identity": format!("{identity}/None"), "name": "Nothing",
                    "origin": origin}]}),
        );
    }
    let order = types
        .iter_mut()
        .find(|ty| ty["identity"] == ORDER)
        .expect("Order");
    for (name, type_ref, lower, upper, ordered) in [
        ("sl", SLOT, 1, 1, false),
        ("ph", PHASE, 1, 1, false),
        ("sls", SLOT, 0, 2, false),
        ("w", WRAP, 1, 1, false),
        ("h", HELD, 1, 1, false),
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
    push_node(wire, declaration_node("tuple_value_type", PAIR), 9102);
    push_node(wire, declaration_node("tuple_value_type", WRAP), 9103);
    push_node(wire, declaration_node("variant_type", HELD), 9104);
    for (offset, (form, identity)) in [
        ("tuple_value_type", OPT_POS),
        ("tuple_value_type", REC_POS),
        ("tuple_value_type", OPT_REC_POS),
        ("tuple_value_type", PLAIN_OPT_POS),
        ("tuple_value_type", PLAIN_REC_POS),
        ("variant_type", REC_HELD),
        ("variant_type", PLAIN_HELD),
        ("record_value_type", TIMER),
        ("record_value_type", PLAIN_REC),
    ]
    .into_iter()
    .enumerate()
    {
        push_node(wire, declaration_node(form, identity), 9110 + offset as u64);
    }
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

/// `left <operation> right` over two parameters, each typed by its `sides`
/// declaration (form, identity); returns the two parameters and the comparison, in that order.
fn equality(operation: &str, sides: [(&str, &str); 2], names: [&str; 2]) -> Vec<Value> {
    let parameter = |name: &str, (form, identity): (&str, &str)| {
        json!({
            "body": aggregate(vec![bind("name", text(name)), bind("level", json!({"term": "literal", "type": integer_type(), "value": "0", "value_kind": "integer"}))]),
            "dependencies": [],
            "node_id": {"ordinal": 0, "package": "example", "path": ["value", "parameter", name]},
            "node_tag": "value",
            "occurrences": [{"ordinal": 0, "role": "expression"}],
            "schema_version": "filament.checked-semantic-graph/v2",
            "semantic_form": "parameter",
            "semantic_type": declaration_id(form, identity),
        })
    };
    let left = parameter(names[0], sides[0]);
    let right = parameter(names[1], sides[1]);
    let mut dependencies = vec![left["node_id"].clone(), right["node_id"].clone()];
    sort_ids(&mut dependencies);
    let application = json!({
        "body": {
            "arguments": [
                {"term": "reference", "target": left["node_id"]},
                {"term": "reference", "target": right["node_id"]}],
            "operation": {"identity": operation, "laws": [], "leaves": [],
                "member": null, "mode": null},
            "operator": "binary",
            "result_type": integer_type(),
            "term": "application"},
        "dependencies": dependencies,
        "node_id": {"ordinal": 0, "package": "example",
            "path": ["expression", "binary", names[0], names[1]]},
        "node_tag": "expression",
        "occurrences": [{"ordinal": 0, "role": "expression"}],
        "schema_version": "filament.checked-semantic-graph/v2",
        "semantic_form": "binary",
        "semantic_type": integer_type(),
    });
    vec![left, right, application]
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

#[trace("TC-048", "FR-038-AC-554", "FR-069-AC-10")]
#[test]
fn equality_over_a_clock_bearing_tuple_or_union_refuses_clock_read_outside_constraint() {
    let tuple = "tuple_value_type";
    let variant = "variant_type";
    let record = "record_value_type";
    let structural_eq = "filament.op.structural.eq";
    let structural_ne = "filament.op.structural.ne";
    // (left side, right side, operation, clock-bearing)
    let cases = [
        ((tuple, PAIR), (tuple, PAIR), structural_eq, false),
        ((tuple, PAIR), (tuple, PAIR), structural_ne, false),
        ((tuple, SLOT), (tuple, SLOT), structural_eq, true),
        ((tuple, SLOT), (tuple, SLOT), structural_ne, true),
        ((variant, PHASE), (variant, PHASE), structural_eq, true),
        ((variant, PHASE), (variant, PHASE), structural_ne, true),
        // The clock sits one level down, in a nested position or payload.
        ((tuple, WRAP), (tuple, WRAP), structural_eq, true),
        ((variant, HELD), (variant, HELD), structural_ne, true),
        // A bare record read whole (FSpec FR-415-AC-6), with a clock-free control.
        ((record, TIMER), (record, TIMER), structural_eq, true),
        ((record, TIMER), (record, TIMER), structural_ne, true),
        (
            (record, PLAIN_REC),
            (record, PLAIN_REC),
            structural_eq,
            false,
        ),
        // The clock sits behind an option or a record; clock-free counterparts admit.
        ((tuple, OPT_POS), (tuple, OPT_POS), structural_eq, true),
        ((tuple, REC_POS), (tuple, REC_POS), structural_ne, true),
        (
            (tuple, OPT_REC_POS),
            (tuple, OPT_REC_POS),
            structural_eq,
            true,
        ),
        (
            (variant, REC_HELD),
            (variant, REC_HELD),
            structural_eq,
            true,
        ),
        (
            (tuple, PLAIN_OPT_POS),
            (tuple, PLAIN_OPT_POS),
            structural_eq,
            false,
        ),
        (
            (tuple, PLAIN_REC_POS),
            (tuple, PLAIN_REC_POS),
            structural_ne,
            false,
        ),
        (
            (variant, PLAIN_HELD),
            (variant, PLAIN_HELD),
            structural_ne,
            false,
        ),
        // Either operand alone carries the clock, in both operand orders.
        ((tuple, SLOT), (tuple, PAIR), structural_eq, true),
        ((tuple, PAIR), (tuple, SLOT), structural_eq, true),
        ((variant, PHASE), (variant, HELD), structural_ne, true),
        ((variant, HELD), (variant, PHASE), structural_ne, true),
    ];
    for (left, right, operation, clock_bearing) in cases {
        let (mut wire, document, proposal) = admitted_with(&[]);
        add_declarations(&mut wire);
        let mut at = 0;
        for (offset, node) in equality(operation, [left, right], ["a", "b"])
            .into_iter()
            .enumerate()
        {
            at = push_node(&mut wire, node, 9500 + offset as u64);
        }
        let label = format!("{operation} over {} and {}", left.1, right.1);
        let result = read(&wire, &document, &proposal, "equality");
        if !clock_bearing {
            assert_eq!(describe(&result), "admitted", "{label}");
            continue;
        }
        let refusal = refused(result, &label);
        assert_eq!(
            (refusal.code, refusal.cause),
            (
                Code::InvalidTimedModel,
                Some(Cause::ClockReadOutsideConstraint)
            ),
            "{label}"
        );
        assert_eq!(
            pointer_of(&refusal),
            format!("/semantic_graph/nodes/{at}"),
            "{label}"
        );
    }
}

fn wrap_place() -> Place {
    Place {
        name: "w-tuple-0-tuple-0",
        steps: vec![field("w"), tuple(0), tuple(0)],
        rows: vec![
            row(0, ORDER, "ix://acme/orders/Order/w", WRAP, "none", "", ""),
            row(1, WRAP, "ix://acme/orders/Wrap/p0", SLOT, "none", "", ""),
            row(2, SLOT, "ix://acme/orders/Slot/p0", CLOCK, "none", "", ""),
        ],
    }
}

fn held_place() -> Place {
    Place {
        name: "h-union-has-tuple-0",
        steps: vec![field("h"), union_case("Has"), tuple(0)],
        rows: vec![
            row(0, ORDER, "ix://acme/orders/Order/h", HELD, "none", "", ""),
            row(
                1,
                HELD,
                "ix://acme/orders/variant/Held-Has",
                SLOT,
                "none",
                "",
                "",
            ),
            row(2, SLOT, "ix://acme/orders/Slot/p0", CLOCK, "none", "", ""),
        ],
    }
}

#[trace("TC-048", "FR-038-AC-548")]
#[test]
fn non_terminal_tuple_and_union_rows_join_the_declared_member_type() {
    // The nested places admit when every row is the selected one.
    let (wire, document, proposal) = admitted_with(&[wrap_place(), held_place()]);
    assert_eq!(
        describe(&read(&wire, &document, &proposal, "nested")),
        "admitted"
    );
    // A non-terminal row typed other than the declared position or payload
    // refuses at that row, though its identity is the selected one and the
    // Clock terminal is intact.
    for (label, place, forged_type) in [
        ("tuple position typed as another tuple", wrap_place(), PAIR),
        (
            "tuple position typed as a data type",
            wrap_place(),
            "ix://filament/native/Integer",
        ),
        ("union payload typed as another tuple", held_place(), PAIR),
        (
            "union payload typed as a data type",
            held_place(),
            "ix://filament/native/Integer",
        ),
    ] {
        let mut place = place;
        let kept = place.rows[1].to_string();
        place.rows[1] =
            serde_json::from_str::<Value>(&kept.replace(SLOT, forged_type)).expect("forged row");
        // The next row's declaring type is left at the declared one.
        let name: &'static str = Box::leak(format!("forged-{label}").into_boxed_str());
        place.name = name;
        let (mut wire, document, proposal) = published();
        let id = add_place(&mut wire, &place, 9000);
        let position = wire["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .position(|node| node["node_id"] == id)
            .expect("added place");
        let refusal = refused(read(&wire, &document, &proposal, label), label);
        assert_eq!(
            (refusal.code, refusal.cause),
            (Code::InvalidPackage, Some(Cause::MalformedClockPlace)),
            "{label}"
        );
        assert_eq!(
            pointer_of(&refusal),
            format!("/semantic_graph/nodes/{position}/body/members/2/value/members/1"),
            "{label}"
        );
    }
}

#[trace("TC-048", "FR-038-AC-552")]
#[test]
fn set_of_clock_bearing_tuples_is_a_clock_collection_bound_to_the_field() {
    let (wire, document, proposal) = admitted_with(&[set_place()]);
    let CheckedPackageV2ReadResult::Admitted(package) = read(&wire, &document, &proposal, "set")
    else {
        panic!("the Set<Slot>[0, 2] clock place must admit");
    };
    let place = package
        .resolved_graph()
        .nodes
        .iter()
        .find(|node| {
            node.semantic_form.as_ref() == "clock_place"
                && serde_json::to_value(&node.body)
                    .expect("body")
                    .to_string()
                    .contains("sls")
        })
        .expect("the Set<Slot> place");
    let body = serde_json::to_value(&place.body).expect("body");
    let rows = body["members"][2]["value"]["members"]
        .as_array()
        .expect("rows");
    assert_eq!(
        rows.len(),
        3,
        "a row for the field, the slot and the position"
    );
    let row_text = |at: usize, name: &str| {
        rows[at]["members"]
            .as_array()
            .and_then(|members| members.iter().find(|member| member["name"] == name))
            .map(|member| member["value"]["value"].clone())
    };
    assert_eq!(row_text(0, "collection_kind"), Some(json!("set")));
    assert_eq!(row_text(0, "minimum"), Some(json!("0")));
    assert_eq!(row_text(0, "maximum"), Some(json!("2")));
    // A slot step whose maximum differs from the field's refuses at row 0.
    let differing = Place {
        name: "set-bound-3-dedicated",
        steps: vec![field("sls"), slot("set", 0, 3), tuple(0)],
        rows: set_place().rows,
    };
    let (mut wire, document, proposal) = published();
    let id = add_place(&mut wire, &differing, 9000);
    let position = wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == id)
        .expect("added place");
    let refusal = refused(read(&wire, &document, &proposal, "bound"), "bound");
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::MalformedClockPlace))
    );
    assert_eq!(
        pointer_of(&refusal),
        format!("/semantic_graph/nodes/{position}/body/members/2/value/members/0")
    );
}
