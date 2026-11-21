// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-448 (FR-347): a record value type that conforms to the record type an
//! operand position declares is admitted at an operation-call argument, a
//! field-read operand, a `case` arm body, a `union_value` payload, a function
//! or expression body root and a `function.call` argument, and every other
//! type stays exact.
//!
//! Every package is built here, node by node, with structural node paths and
//! a real Semantic IR 2.0.0 domain package document selected by the lock.

use crate::support::checked_package::{
    canonical, fixture_source, index_fixture_node_ids, node_id as fixture_node_id, nominal_package,
    pointer, select_model, selected_model_identities, settle, source_owner, structural_id,
    COMPLETE_VALUE_FEATURE,
};
use filament_contract_model::{
    CheckedPackageEvidence, CheckedPackageLimit, CheckedPackageReadLimits,
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code, CheckedPackageV2,
    CheckedPackageV2ReadResult,
};
use ix_trace_rs::trace;
use serde_json::{json, Value};
use std::collections::BTreeSet;

const LEDGER: &str = "acme/ledger";
const VENDOR: &str = "acme/vendor";
const NATIVE: &str = "ix://filament/native/";

fn iri(identity: &str, short: &str) -> String {
    format!("ix://{identity}/{short}")
}

fn node_id(key: &str) -> Value {
    if let Some(model) = key.strip_prefix("model:") {
        let mut parts = model.splitn(3, ':');
        let identity = parts.next().expect("model package");
        let form = parts.next().expect("model form");
        let declaration = parts.next().expect("model declaration");
        return structural_id(
            identity,
            vec![json!("model"), json!(form), json!(declaration)],
            0,
        );
    }
    if let Some(form) = key.strip_prefix("scalar_type/") {
        return structural_id("test/families", vec![json!("scalar_type"), json!(form)], 0);
    }
    if let Some(encoded_positions) = key.strip_prefix("composite_type/tuple/") {
        let positions: Vec<String> =
            serde_json::from_str(encoded_positions).expect("encoded tuple position keys");
        let mut path = vec![json!("composite_type"), json!("tuple")];
        path.extend(positions.iter().map(|position| node_id(position)));
        return structural_id("test/families", path, 0);
    }
    for form in ["reference", "option", "set", "sequence"] {
        if let Some(element) = key.strip_prefix(&format!("composite_type/{form}/")) {
            return structural_id(
                "test/families",
                vec![json!("composite_type"), json!(form), node_id(element)],
                0,
            );
        }
    }
    if let Some(bounds) = key.strip_prefix("bounded_domain/integer_range/") {
        let (lower, upper) = bounds.split_once('/').expect("integer bounds");
        return structural_id(
            "test/families",
            vec![
                json!("bounded_domain"),
                json!("integer_range"),
                json!("min"),
                node_id("scalar_type/integer"),
                json!(lower),
                json!("max"),
                json!(upper),
            ],
            0,
        );
    }
    if let Some(bounds) = key.strip_prefix("bounded_domain/collection_bounds/") {
        let (collection_and_lower, upper) = bounds.rsplit_once('/').expect("collection upper");
        let (collection, lower) = collection_and_lower
            .rsplit_once('/')
            .expect("collection lower");
        return structural_id(
            "test/families",
            vec![
                json!("bounded_domain"),
                json!("collection_bounds"),
                json!("min"),
                node_id("scalar_type/integer"),
                json!(lower),
                json!("max"),
                json!(upper),
                node_id(collection),
            ],
            0,
        );
    }
    fixture_node_id(key)
}

fn label_of(id: &Value) -> String {
    let path = id["path"].as_array().expect("structural path");
    if path.len() == 3 && path[0] == "model" {
        return format!(
            "model:{}:{}:{}",
            id["package"].as_str().expect("model package"),
            id["path"][1].as_str().expect("model form"),
            id["path"][2].as_str().expect("model declaration")
        );
    }
    if path.len() == 2 && path[0] == "scalar_type" {
        return format!("scalar_type/{}", path[1].as_str().expect("scalar form"));
    }
    if path.len() == 3 && path[0] == "composite_type" {
        if path[1] == "tuple" {
            let positions: Vec<String> = path[2..].iter().map(label_of).collect();
            return format!(
                "composite_type/tuple/{}",
                serde_json::to_string(&positions).expect("tuple position key JSON")
            );
        }
        return format!(
            "composite_type/{}/{}",
            path[1].as_str().expect("composite form"),
            label_of(&path[2]),
        );
    }
    if path.len() > 3 && path[0] == "composite_type" && path[1] == "tuple" {
        let positions: Vec<String> = path[2..].iter().map(label_of).collect();
        return format!(
            "composite_type/tuple/{}",
            serde_json::to_string(&positions).expect("tuple position key JSON")
        );
    }
    if path.len() == 7 && path[0] == "bounded_domain" && path[1] == "integer_range" {
        return format!(
            "bounded_domain/integer_range/{}/{}",
            path[4].as_str().expect("lower bound"),
            path[6].as_str().expect("upper bound"),
        );
    }
    if path.len() == 8 && path[0] == "bounded_domain" && path[1] == "collection_bounds" {
        return format!(
            "bounded_domain/collection_bounds/{}/{}/{}",
            label_of(&path[7]),
            path[4].as_str().expect("lower bound"),
            path[6].as_str().expect("upper bound"),
        );
    }
    id["path"][0].as_str().expect("structural path").to_owned()
}

fn empty() -> Value {
    json!({"term": "aggregate", "members": []})
}

fn aggregate(members: Vec<Value>) -> Value {
    json!({"term": "aggregate", "members": members})
}

fn binding(name: &str, value: Value) -> Value {
    json!({"term": "binding", "name": name, "value": value})
}

/// The binding term of a `let`, carrying the `NodeRef` of its binder.
fn let_binding(name: &str, binder: &str, value: Value) -> Value {
    json!({"term": "binding", "name": name, "binder": node_id(binder), "value": value})
}

fn reference(key: &str) -> Value {
    json!({"term": "reference", "target": node_id(key)})
}

fn literal(ty: &str, value_kind: &str, value: &str) -> Value {
    json!({"term": "literal", "type": node_id(ty), "value_kind": value_kind, "value": value})
}

fn wire_node(
    key: &str,
    tag: &str,
    form: &str,
    semantic_type: &str,
    dependencies: &[&str],
    role: &str,
    body: Value,
) -> Value {
    {
        let mut node = json!({
            "node_id": node_id(key),
            "schema_version": "filament.checked-semantic-graph/v2",
            "node_tag": tag,
            "semantic_form": form,
            "semantic_type": node_id(semantic_type),
            "dependencies": dependencies.iter().map(|key| node_id(key)).collect::<Vec<_>>(),
            "occurrences": [{"role": role, "ordinal": 0}],
            "body": body,
        });
        crate::support::checked_package::carry_check_mode(&mut node);
        node
    }
}

/// The targets FR-322's application-node dependency join names in `term`.
fn targets(term: &Value, out: &mut BTreeSet<String>) {
    match term["term"].as_str() {
        Some("reference") => {
            out.insert(label_of(&term["target"]));
        }
        Some("application") => {
            if let Some(declaration) = term["operation"]["member"]
                .get("declaration")
                .filter(|value| !value.is_null())
            {
                out.insert(label_of(declaration));
            }
            for argument in term["arguments"].as_array().into_iter().flatten() {
                targets(argument, out);
            }
        }
        Some("aggregate") => {
            for member in term["members"].as_array().into_iter().flatten() {
                targets(member, out);
            }
        }
        Some("binding") => {
            // FR-322 "The binder link": a binding's `binder` is a dependency.
            if let Some(binder) = term.get("binder") {
                out.insert(label_of(binder));
            }
            targets(&term["value"], out);
        }
        _ => {}
    }
}

fn dependencies_of(body: &Value) -> Vec<String> {
    let mut out = BTreeSet::new();
    targets(body, &mut out);
    out.into_iter().collect()
}

// --- the domain package documents ---------------------------------------------------

const ONE: (u64, Option<u64>, bool, bool) = (1, Some(1), false, true);

fn multiplicity((lower, upper, ordered, unique): (u64, Option<u64>, bool, bool)) -> Value {
    let mut multiplicity = json!({"lower": lower, "ordered": ordered, "unique": unique});
    if let Some(upper) = upper {
        multiplicity["upper"] = json!(upper);
    }
    multiplicity
}

fn slot(type_ref: &str, shape: (u64, Option<u64>, bool, bool)) -> Value {
    json!({"typeRef": type_ref, "multiplicity": multiplicity(shape)})
}

fn declared_type(
    identity: &str,
    short: &str,
    kind: &str,
    supertypes: &[&str],
    fields: &[(&str, String)],
    operations: Vec<Value>,
) -> Value {
    let node = iri(identity, short);
    let fields: Vec<Value> = fields
        .iter()
        .map(|(name, type_ref)| {
            json!({
                "identity": format!("{node}/{name}"), "name": name, "typeRef": type_ref,
                "presence": "required", "nullable": false, "defaultKind": "none",
                "multiplicity": multiplicity(ONE),
            })
        })
        .collect();
    json!({
        "identity": node, "displayName": short,
        "kind": {"module": identity, "name": kind},
        "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
        "supertypes": supertypes.iter().map(|short| iri(identity, short)).collect::<Vec<_>>(),
        "fields": fields, "operations": operations,
    })
}

fn document(identity: &str, types: Vec<Value>) -> Value {
    json!({
        "contractVersion": "2.0.0",
        "package": {"identity": identity, "version": "1.0.0"},
        "constructs": [
            {"kind": {"module": identity, "name": "entity"},
             "construct": {"meaning": "filament.meaning.model.object-type/v1"}},
            {"kind": {"module": identity, "name": "record"},
             "construct": {"meaning": "filament.meaning.model.record-value-type/v1"}},
            {"kind": {"module": identity, "name": "digit"},
             "construct": {"meaning": "filament.meaning.model.value-type/v1"}},
        ],
        "types": types,
    })
}

fn native(name: &str) -> String {
    format!("{NATIVE}{name}")
}

/// `Money`; `TaxedMoney` and `GiftTaxedMoney` below it; an unrelated
/// `Voucher`; the diamond `D` over `B` and `C` over `A`; `OrderPlaced` with
/// its child `ExpressOrderPlaced` and grandchild `RushOrderPlaced`; the
/// `Ledger` object (and its subtype) declaring every operation; and the
/// `Digit` value type `Int[0, 9]`.
fn ledger_document() -> Value {
    let amount = [("amount", native("Integer"))];
    let record = |short: &str, supertypes: &[&str], fields: &[(&str, String)]| {
        declared_type(LEDGER, short, "record", supertypes, fields, vec![])
    };
    let money = iri(LEDGER, "Money");
    let operation = |name: &str, params: Vec<Value>, returns: &str| {
        json!({
            "identity": iri(LEDGER, &format!("Ledger/{name}")),
            "params": params,
            "returns": slot(returns, ONE),
        })
    };
    let one = |type_ref: &str| slot(type_ref, ONE);
    let integer = native("Integer");
    let operations = vec![
        operation("charge", vec![one(&money)], &integer),
        operation("settle", vec![one(&iri(LEDGER, "TaxedMoney"))], &integer),
        operation(
            "grandsettle",
            vec![one(&iri(LEDGER, "GiftTaxedMoney"))],
            &integer,
        ),
        operation(
            "pair",
            vec![one(&money), one(&iri(LEDGER, "TaxedMoney"))],
            &integer,
        ),
        operation("take_int", vec![one(&integer)], &integer),
        operation("take_range", vec![one(&iri(LEDGER, "Digit"))], &integer),
        operation("take_bool", vec![one(&native("Boolean"))], &integer),
        operation("take_ref", vec![one(&iri(LEDGER, "Ledger"))], &integer),
        operation(
            "take_set",
            vec![slot(&money, (0, Some(1), false, true))],
            &integer,
        ),
        operation("over_a", vec![one(&iri(LEDGER, "A"))], &integer),
        operation("mint_money", vec![], &money),
        operation("mint_taxed", vec![], &iri(LEDGER, "TaxedMoney")),
        operation("mint_gift", vec![], &iri(LEDGER, "GiftTaxedMoney")),
        operation("mint_voucher", vec![], &iri(LEDGER, "Voucher")),
    ];
    let digit = json!({
        "identity": iri(LEDGER, "Digit"), "displayName": "Digit",
        "kind": {"module": LEDGER, "name": "digit"}, "roles": [], "extensions": [],
        "unknownPolicy": "reject", "scalar": "integer",
        "constraints": [
            {"keyword": "min", "operands": {"value": "0"}},
            {"keyword": "max", "operands": {"value": "9"}},
        ],
    });
    document(
        LEDGER,
        vec![
            record("Money", &[], &amount),
            record("TaxedMoney", &["Money"], &[]),
            record("GiftTaxedMoney", &["TaxedMoney"], &[]),
            record("Voucher", &[], &[]),
            record("A", &[], &[]),
            record("B", &["A"], &[]),
            record("C", &["A"], &[]),
            record("D", &["B", "C"], &[]),
            record(
                "OrderPlaced",
                &[],
                &[
                    ("order_id", native("Integer")),
                    ("occurred_at", native("Integer")),
                ],
            ),
            record(
                "ExpressOrderPlaced",
                &["OrderPlaced"],
                &[("promised_by", native("Integer"))],
            ),
            record("RushOrderPlaced", &["ExpressOrderPlaced"], &[]),
            declared_type(LEDGER, "Ledger", "entity", &[], &[], operations),
            declared_type(LEDGER, "SubLedger", "entity", &["Ledger"], &[], vec![]),
            digit,
        ],
    )
}

/// The ledger document with tuple declarations used by TC-448's tuple-value cases.
fn ledger_tuple_document() -> Value {
    let mut document = ledger_document();
    document["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind": {"module": LEDGER, "name": "tuple"},
            "construct": {"meaning": "filament.meaning.model.tuple-value-type/v1"}
        }));
    let source = |path: &str| {
        json!({
            "source": {
                "path": path,
                "sourceIdentity": iri(LEDGER, "spec"),
                "startColumn": 1,
                "startLine": 1
            }
        })
    };
    for (short, positions) in [
        ("Pair", vec![iri(LEDGER, "Money"), native("Integer")]),
        ("Clocked", vec![native("Clock"), native("Integer")]),
    ] {
        let identity = iri(LEDGER, short);
        document["types"]
            .as_array_mut()
            .expect("types")
            .push(json!({
                "identity": identity,
                "displayName": short,
                "kind": {"module": LEDGER, "name": "tuple"},
                "origin": source(&format!("{short}.md")),
                "positions": positions.into_iter().enumerate().map(|(ordinal, type_ref)| {
                    json!({
                        "identity": format!("{identity}/p{ordinal}"),
                        "typeRef": type_ref,
                        "origin": source(&format!("{short}.md"))
                    })
                }).collect::<Vec<_>>(),
                "constraints": [],
                "extensions": []
            }));
    }
    document
}

/// A second selected document declaring its own `Money` and `TaxedMoney`.
fn vendor_document() -> Value {
    document(
        VENDOR,
        vec![
            declared_type(VENDOR, "Money", "record", &[], &[], vec![]),
            declared_type(VENDOR, "TaxedMoney", "record", &["Money"], &[], vec![]),
        ],
    )
}

// --- the package ------------------------------------------------------------------

/// A type, as a test states it; the node key is derived from it.
#[derive(Clone, Debug)]
enum Ty {
    Boolean,
    Integer,
    Range(i128, i128),
    /// A record value type of the ledger document.
    Rec(&'static str),
    /// A record value type of the vendor document.
    Foreign(&'static str),
    /// An object type of the ledger document, as the type itself.
    Object(&'static str),
    /// `Reference<O>` for an object type of the ledger document.
    Ref(&'static str),
    Option(Box<Ty>),
    /// A `Set<E>[lower, upper]`.
    Set(Box<Ty>, u64, u64),
    /// A `Sequence<E>[lower, upper]`.
    Sequence(Box<Ty>, u64, u64),
    /// A source-declared tuple whose positions are these exact type nodes.
    Tuple(Vec<Ty>),
    /// A tuple value type selected from the ledger document.
    ModelTuple(&'static str),
    /// A source-declared `composite_type`/`record` node named `name`.
    SourceRecord(&'static str),
    /// A raw node key.
    Key(String),
}

struct Scene {
    documents: Vec<Value>,
    nodes: Vec<Value>,
    seen: BTreeSet<String>,
}

fn path_of(node: &Value) -> String {
    label_of(&node["node_id"])
}

fn bounds_body(integer: &str, lower: &str, upper: &str) -> Value {
    let bound = |name: &str, value: &str| binding(name, literal(integer, "integer", value));
    aggregate(vec![bound("min", lower), bound("max", upper)])
}

impl Scene {
    fn new(documents: Vec<Value>) -> Self {
        Self {
            documents,
            nodes: Vec::new(),
            seen: BTreeSet::new(),
        }
    }

    fn add(&mut self, node: Value) -> String {
        let key = path_of(&node);
        if self.seen.insert(key.clone()) {
            self.nodes.push(node);
        }
        key
    }

    fn scalar(&mut self, form: &str) -> String {
        let key = format!("scalar_type/{form}");
        self.add(wire_node(
            &key,
            "scalar_type",
            form,
            &key,
            &[],
            "type",
            empty(),
        ))
    }

    /// The model declaration node of `short` in the document `identity`.
    fn model(&mut self, identity: &str, short: &str, form: &str) -> String {
        let owner = json!({"kind": "model", "identity": identity, "node": iri(identity, short)});
        let key = format!("model:{identity}:{form}:{}", iri(identity, short));
        let mut node = wire_node(&key, "model", form, &key, &[], "type", empty());
        node["owner"] = owner;
        self.add(node)
    }

    fn ty(&mut self, ty: &Ty) -> String {
        match ty {
            Ty::Boolean => self.scalar("boolean"),
            Ty::Integer => self.scalar("integer"),
            Ty::Range(lower, upper) => {
                let integer = self.scalar("integer");
                let body = bounds_body(&integer, &lower.to_string(), &upper.to_string());
                let key = format!("bounded_domain/integer_range/{lower}/{upper}");
                self.add(wire_node(
                    &key,
                    "bounded_domain",
                    "integer_range",
                    &integer,
                    &[&integer],
                    "type",
                    body,
                ))
            }
            Ty::Rec(short) => self.model(LEDGER, short, "record_value_type"),
            Ty::Foreign(short) => self.model(VENDOR, short, "record_value_type"),
            Ty::Object(short) => self.model(LEDGER, short, "object_type"),
            Ty::Ref(short) => {
                let target = self.model(LEDGER, short, "object_type");
                self.composite("reference", &target)
            }
            Ty::Option(element) => {
                let element = self.ty(element);
                self.composite("option", &element)
            }
            Ty::Set(element, lower, upper) | Ty::Sequence(element, lower, upper) => {
                let form = if matches!(ty, Ty::Set(..)) {
                    "set"
                } else {
                    "sequence"
                };
                let element = self.ty(element);
                let collection = self.composite(form, &element);
                let integer = self.scalar("integer");
                let body = bounds_body(&integer, &lower.to_string(), &upper.to_string());
                let key = format!("bounded_domain/collection_bounds/{collection}/{lower}/{upper}");
                self.add(wire_node(
                    &key,
                    "bounded_domain",
                    "collection_bounds",
                    &collection,
                    &[&collection],
                    "type",
                    body,
                ))
            }
            Ty::Tuple(elements) => {
                let element_keys = elements
                    .iter()
                    .map(|element| self.ty(element))
                    .collect::<Vec<_>>();
                let key = format!(
                    "composite_type/tuple/{}",
                    serde_json::to_string(&element_keys).expect("tuple position key JSON")
                );
                let body = aggregate(
                    element_keys
                        .iter()
                        .map(|element| reference(element))
                        .collect(),
                );
                let dependencies: Vec<&str> = element_keys.iter().map(String::as_str).collect();
                self.add(wire_node(
                    &key,
                    "composite_type",
                    "tuple",
                    &key,
                    &dependencies,
                    "type",
                    body,
                ))
            }
            Ty::ModelTuple(short) => self.model(LEDGER, short, "tuple_value_type"),
            Ty::SourceRecord(name) => {
                let integer = self.scalar("integer");
                let body = aggregate(vec![binding("amount", reference(&integer))]);
                let owner = source_owner(&fixture_source());
                let key = format!("qualified:{name}");
                let mut node = wire_node(
                    &key,
                    "composite_type",
                    "record",
                    &key,
                    &[&integer],
                    "declaration",
                    body,
                );
                node["declaration"] = json!({"qualified_name": [name]});
                node["owner"] = owner;
                self.add(node)
            }
            Ty::Key(key) => key.clone(),
        }
    }

    fn composite(&mut self, form: &str, element: &str) -> String {
        let body = aggregate(vec![reference(element)]);
        let key = format!("composite_type/{form}/{element}");
        self.add(wire_node(
            &key,
            "composite_type",
            form,
            &key,
            &[element],
            "type",
            body,
        ))
    }

    /// A `value`/`parameter` node named `name` typed `ty`.
    fn param(&mut self, name: &str, ty: &Ty) -> String {
        let ty = self.ty(ty);
        let text = self.scalar("text");
        let integer = self.scalar("integer");
        let body = aggregate(vec![
            binding("name", literal(&text, "text", name)),
            binding("level", literal(&integer, "integer", "0")),
        ]);
        let key = format!("value/parameter/{name}/{ty}");
        self.add(wire_node(
            &key,
            "value",
            "parameter",
            &ty,
            &[],
            "expression",
            body,
        ))
    }

    /// An application node over `arguments`, typed `semantic` with the
    /// application's `result_type` `result`.
    fn application(&mut self, app: App<'_>) -> String {
        let body = json!({
            "term": "application",
            "operator": app.operator,
            "operation": {
                "identity": app.identity, "laws": [], "mode": null,
                "member": app.member, "leaves": [],
            },
            "result_type": node_id(app.result),
            "arguments": app.arguments,
        });
        let key = format!("application/{}/{}/{}", app.tag, app.form, self.nodes.len());
        let dependencies = dependencies_of(&body);
        let dependencies: Vec<&str> = dependencies.iter().map(String::as_str).collect();
        self.add(wire_node(
            &key,
            app.tag,
            app.form,
            app.semantic,
            &dependencies,
            "expression",
            body,
        ))
    }

    /// `receiver.name(arguments)`: a `dispatch_call` of the operation `name`
    /// declared by `declaring`, typed at `result`.
    fn dispatch(&mut self, declaring: &str, name: &str, params: &[String], result: &str) -> String {
        self.application(App {
            tag: "expression",
            form: "call",
            operator: "call",
            identity: "filament.op.model.dispatch_call",
            member: json!({"kind": "operation", "declaration": node_id(declaring), "name": name}),
            semantic: result,
            result,
            arguments: params.iter().map(|key| reference(key)).collect(),
        })
    }

    /// A source-declared `function`/`pure_function` of `semantic` whose body
    /// is the aggregate of `members`.
    fn function(&mut self, name: &str, semantic: &str, members: Vec<Value>) -> String {
        let body = aggregate(members);
        let owner = source_owner(&fixture_source());
        let key = format!("qualified:{name}");
        let dependencies: Vec<String> = dependencies_of(&body)
            .into_iter()
            .filter(|key| self.seen.contains(key))
            .collect();
        let dependencies: Vec<&str> = dependencies.iter().map(String::as_str).collect();
        let mut node = wire_node(
            &key,
            "function",
            "pure_function",
            semantic,
            &dependencies,
            "declaration",
            body,
        );
        node["declaration"] = json!({"qualified_name": [name]});
        node["owner"] = owner;
        self.add(node)
    }

    /// A source-declared union type of `members` (name, payload type keys).
    fn union(&mut self, name: &str, members: &[(&str, Vec<String>)]) -> String {
        let body = aggregate(
            members
                .iter()
                .map(|(member, payload)| {
                    binding(
                        member,
                        aggregate(payload.iter().map(|ty| reference(ty)).collect()),
                    )
                })
                .collect(),
        );
        let owner = source_owner(&fixture_source());
        let key = format!("qualified:{name}");
        let mut payload: Vec<&str> = members
            .iter()
            .flat_map(|(_, payload)| payload.iter().map(String::as_str))
            .collect();
        payload.sort_unstable();
        payload.dedup();
        let mut node = wire_node(
            &key,
            "composite_type",
            "union",
            &key,
            &payload,
            "declaration",
            body,
        );
        node["declaration"] = json!({"qualified_name": [name]});
        node["owner"] = owner;
        self.add(node)
    }

    /// `ty::member(payload...)`, the payload terms value nodes.
    fn union_value(&mut self, ty: &str, member: &str, payload: &[String]) -> String {
        let body = aggregate(vec![binding(
            member,
            aggregate(payload.iter().map(|key| reference(key)).collect()),
        )]);
        let key = format!(
            "application/value/union_value/{member}/{}",
            self.nodes.len()
        );
        let dependencies = dependencies_of(&body);
        let dependencies: Vec<&str> = dependencies.iter().map(String::as_str).collect();
        self.add(wire_node(
            &key,
            "value",
            "union_value",
            ty,
            &dependencies,
            "expression",
            body,
        ))
    }

    /// A tuple value whose ordered members are already admitted terms.
    fn tuple_value(&mut self, ty: &str, members: Vec<Value>) -> String {
        let body = aggregate(members);
        let key = format!("application/value/tuple_value/{}", self.nodes.len());
        let dependencies = dependencies_of(&body);
        let dependencies: Vec<&str> = dependencies.iter().map(String::as_str).collect();
        self.add(wire_node(
            &key,
            "value",
            "tuple_value",
            ty,
            &dependencies,
            "expression",
            body,
        ))
    }

    fn finish(self) -> Case {
        let mut documents = self.documents;
        documents.sort_by_key(|document| {
            document["package"]["identity"]
                .as_str()
                .expect("identity")
                .to_owned()
        });
        let mut package = nominal_package(&[]);
        for document in &documents {
            select_model(
                &mut package,
                document["package"]["identity"].as_str().expect("identity"),
            );
        }
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .extend(self.nodes);
        settle(&mut package);
        let mut evidence = CheckedPackageEvidence::new();
        evidence.support_feature(COMPLETE_VALUE_FEATURE);
        for document in &documents {
            evidence
                .insert_domain_package_document(
                    document["package"]["identity"].as_str().expect("identity"),
                    canonical(document),
                )
                .expect("distinct argument document key");
        }
        Case {
            package,
            evidence,
            node: 0,
        }
    }
}

struct App<'a> {
    tag: &'a str,
    form: &'a str,
    operator: &'a str,
    identity: &'a str,
    member: Value,
    semantic: &'a str,
    result: &'a str,
    arguments: Vec<Value>,
}

/// A package, its evidence and the position of the node under test.
struct Case {
    package: Value,
    evidence: CheckedPackageEvidence,
    node: usize,
}

impl Case {
    /// The case whose node under test is `key`.
    fn at(mut self, key: &str) -> Self {
        self.node = self.package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .position(|node| node["node_id"] == node_id(key))
            .expect("the node under test is in the graph");
        self
    }

    fn read_at(&self, work: Option<u64>) -> CheckedPackageV2ReadResult {
        let mut limits = CheckedPackageReadLimits::bounded();
        if let Some(work) = work {
            limits.work = work;
        }
        let mut wire = self.package.clone();
        index_fixture_node_ids(&mut wire);
        CheckedPackageV2::read(&canonical(&wire), limits, &self.evidence)
    }

    fn read(&self) -> CheckedPackageV2ReadResult {
        self.read_at(None)
    }

    fn admits(&self, label: &str) {
        match self.read() {
            CheckedPackageV2ReadResult::Admitted(_) => {}
            other => panic!("{label}: expected admission, read {other:?}"),
        }
    }

    /// The read refuses with `code`/`cause` at `tail` below the node under test.
    fn refuses(&self, label: &str, code: Code, cause: Cause, tail: &str) {
        let CheckedPackageV2ReadResult::Refused(refusal) = self.read() else {
            panic!("{label}: expected a refusal");
        };
        let expected = format!("/semantic_graph/nodes/{}{tail}", self.node);
        assert_eq!(
            (refusal.code, refusal.cause, refusal.loci[0].path.clone()),
            (code, Some(cause), Some(pointer(&expected))),
            "{label}: {refusal:?}"
        );
    }

    fn ineligible(&self, label: &str, tail: &str) {
        self.refuses(label, Code::IllTyped, Cause::OperatorIneligible, tail);
    }

    fn mismatch(&self, label: &str) {
        self.refuses(label, Code::IllTyped, Cause::TypeMismatch, "");
    }

    /// The least `work` limit under which the read admits.
    fn least_work(&self) -> u64 {
        let (mut low, mut high) = (0, CheckedPackageReadLimits::bounded().work);
        assert!(
            matches!(
                self.read_at(Some(high)),
                CheckedPackageV2ReadResult::Admitted(_)
            ),
            "the case admits under the default work limit"
        );
        while low < high {
            let middle = low + (high - low) / 2;
            match self.read_at(Some(middle)) {
                CheckedPackageV2ReadResult::Admitted(_) => high = middle,
                CheckedPackageV2ReadResult::Incomplete(_) => low = middle + 1,
                other => panic!("expected admission or the work limit, read {other:?}"),
            }
        }
        low
    }
}

fn ledger_scene() -> Scene {
    Scene::new(vec![ledger_document()])
}

fn ledger_tuple_scene() -> Scene {
    Scene::new(vec![ledger_tuple_document()])
}

/// The result type each operation of the ledger declares.
fn result_of(operation: &str) -> Ty {
    match operation {
        "mint_money" => Ty::Rec("Money"),
        "mint_taxed" => Ty::Rec("TaxedMoney"),
        "mint_gift" => Ty::Rec("GiftTaxedMoney"),
        "mint_voucher" => Ty::Rec("Voucher"),
        _ => Ty::Integer,
    }
}

/// `receiver.operation(arguments)`, the arguments parameters typed as given.
/// Every case holds the same parameter nodes, so every case costs the same
/// work apart from the conformance walk.
fn call_with(
    scene: &mut Scene,
    receiver: &Ty,
    operation: &str,
    arguments: &[Ty],
    result: &Ty,
) -> String {
    for (name, ty) in [
        ("pad_money", Ty::Rec("Money")),
        ("pad_taxed", Ty::Rec("TaxedMoney")),
        ("pad_gift", Ty::Rec("GiftTaxedMoney")),
        ("pad_d", Ty::Rec("D")),
        ("pad_b", Ty::Rec("B")),
        ("pad_c", Ty::Rec("C")),
        ("pad_a", Ty::Rec("A")),
    ] {
        scene.param(name, &ty);
    }
    let declaring = scene.ty(&Ty::Object("Ledger"));
    let mut params = vec![scene.param("receiver", receiver)];
    for (position, ty) in arguments.iter().enumerate() {
        params.push(scene.param(&format!("arg{position}"), ty));
    }
    let result = scene.ty(result);
    scene.dispatch(&declaring, operation, &params, &result)
}

fn call(operation: &str, arguments: &[Ty]) -> Case {
    call_over(&Ty::Ref("Ledger"), operation, arguments)
}

fn call_over(receiver: &Ty, operation: &str, arguments: &[Ty]) -> Case {
    let mut scene = ledger_scene();
    let key = call_with(
        &mut scene,
        receiver,
        operation,
        arguments,
        &result_of(operation),
    );
    scene.finish().at(&key)
}

const ARG_1: &str = "/body/arguments/1";
const ARG_2: &str = "/body/arguments/2";

// --- TC-448: operation-call arguments (FR-347-AC-1 to AC-7) ---------------------------

#[trace("TC-448", "FR-347-AC-1")]
#[test]
fn tc_448_a_child_and_a_grandchild_record_are_admitted_at_a_parent_parameter() {
    for argument in ["Money", "TaxedMoney", "GiftTaxedMoney"] {
        let case = call("charge", &[Ty::Rec(argument)]);
        case.admits(argument);
        // The conformance is the document's: the package carries neither a
        // `supertypes` member nor a conversion node.
        let text = String::from_utf8(canonical(&case.package)).expect("utf-8");
        assert!(!text.contains("supertypes"), "{argument}");
        assert!(!text.contains("conversion"), "{argument}");
    }
}

#[trace("TC-448", "FR-347-AC-2")]
#[test]
fn tc_448_an_unrelated_record_is_refused_at_its_argument() {
    call("charge", &[Ty::Rec("Voucher")]).ineligible("Voucher at Money", ARG_1);
}

#[trace("TC-448", "FR-347-AC-3")]
#[test]
fn tc_448_an_ancestor_never_conforms_to_a_descendant() {
    call("settle", &[Ty::Rec("Money")]).ineligible("Money at TaxedMoney", ARG_1);
    call("grandsettle", &[Ty::Rec("TaxedMoney")]).ineligible("TaxedMoney at Gift", ARG_1);
    call("grandsettle", &[Ty::Rec("Money")]).ineligible("Money at Gift", ARG_1);
    call("settle", &[Ty::Rec("GiftTaxedMoney")]).admits("Gift at TaxedMoney");
    call("settle", &[Ty::Rec("TaxedMoney")]).admits("TaxedMoney at TaxedMoney");
    call("grandsettle", &[Ty::Rec("GiftTaxedMoney")]).admits("Gift at Gift");
}

#[trace("TC-448", "FR-347-AC-4")]
#[test]
fn tc_448_every_other_parameter_type_is_compared_exactly() {
    let money_set = |element: &'static str| Ty::Set(Box::new(Ty::Rec(element)), 0, 1);
    let admitted = [
        ("take_int", Ty::Integer),
        ("take_range", Ty::Range(0, 9)),
        ("take_bool", Ty::Boolean),
        ("take_ref", Ty::Ref("Ledger")),
        ("take_set", money_set("Money")),
    ];
    for (operation, argument) in admitted {
        call(operation, &[argument]).admits(operation);
    }
    let refused = [
        ("take_int", Ty::Rec("TaxedMoney")),
        ("take_range", Ty::Range(0, 5)),
        ("take_bool", Ty::Integer),
        ("take_ref", Ty::Ref("SubLedger")),
        ("take_set", Ty::Rec("TaxedMoney")),
        ("take_set", money_set("TaxedMoney")),
        ("charge", Ty::Option(Box::new(Ty::Rec("Money")))),
    ];
    for (operation, argument) in refused {
        let label = format!("{operation} over {argument:?}");
        call(operation, std::slice::from_ref(&argument)).ineligible(&label, ARG_1);
    }
}

#[trace("TC-448", "FR-347-AC-5")]
#[test]
fn tc_448_ancestry_is_read_from_the_selected_document_only() {
    // A source-declared record with `TaxedMoney`'s name and fields.
    call("charge", &[Ty::SourceRecord("TaxedMoney")]).ineligible("source record at Money", ARG_1);
    // `TaxedMoney` of a second selected document does not conform to this
    // document's `Money`.
    let mut scene = Scene::new(vec![ledger_document(), vendor_document()]);
    let key = call_with(
        &mut scene,
        &Ty::Ref("Ledger"),
        "charge",
        &[Ty::Foreign("TaxedMoney")],
        &Ty::Integer,
    );
    scene
        .finish()
        .at(&key)
        .ineligible("foreign TaxedMoney", ARG_1);
}

#[trace("TC-448", "FR-347-AC-6")]
#[test]
fn tc_448_the_first_offending_argument_in_position_order_is_reported() {
    let (money, taxed, voucher) = (Ty::Rec("Money"), Ty::Rec("TaxedMoney"), Ty::Rec("Voucher"));
    call("pair", &[taxed.clone(), taxed.clone()]).admits("both conforming");
    call("pair", &[taxed.clone(), money.clone()]).ineligible("conforming then ancestor", ARG_2);
    call("pair", &[money.clone(), money.clone()]).ineligible("exact then ancestor", ARG_2);
    call("pair", &[voucher.clone(), taxed.clone()]).ineligible("unrelated then conforming", ARG_1);
    call("pair", &[voucher, money]).ineligible("both non-conforming", ARG_1);
    // The arity check precedes every argument check.
    let arguments = "/body/arguments";
    call("charge", &[Ty::Rec("Voucher"), Ty::Rec("Money")]).ineligible("one too many", arguments);
    call("charge", &[]).ineligible("one too few", arguments);
}

#[trace("TC-448", "FR-347-AC-7")]
#[test]
fn tc_448_the_receiver_and_the_result_type_stay_exact() {
    call_over(&Ty::Ref("SubLedger"), "charge", &[Ty::Rec("Money")])
        .ineligible("Reference of a subtype", "/body/arguments/0");
    let mut scene = ledger_scene();
    let key = call_with(
        &mut scene,
        &Ty::Ref("Ledger"),
        "charge",
        &[Ty::Rec("Money")],
        &Ty::Boolean,
    );
    scene
        .finish()
        .at(&key)
        .ineligible("result_type differs", "/body/result_type");
}

// --- TC-448: work accounting (FR-347-AC-8) -------------------------------------------

#[trace("TC-448", "FR-347-AC-8")]
#[test]
fn tc_448_conformance_charges_every_reachable_supertype_edge_once() {
    let charged =
        |operation: &str, exact: &'static str, conforming: &'static str, edges: u64, row: bool| {
            let control = call(operation, &[Ty::Rec(exact)]);
            let walked = call(operation, &[Ty::Rec(conforming)]);
            let (baseline, total) = (control.least_work(), walked.least_work());
            assert_eq!(
                total - baseline,
                edges,
                "{conforming} at {exact}: the walk charges {edges}"
            );
            // One unit below the measured total is the work limit; the total admits.
            match walked.read_at(Some(total - 1)) {
                CheckedPackageV2ReadResult::Incomplete(incomplete) => {
                    assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
                }
                other => panic!("{conforming} at {exact}: expected the work limit, read {other:?}"),
            }
            assert!(matches!(
                walked.read_at(Some(total)),
                CheckedPackageV2ReadResult::Admitted(_)
            ));
            // The walk is charged at the selection row of the document that
            // declares the expected record: that many more limits run out there.
            let at_row = |case: &Case, total: u64| {
                // The first selected document, by identity, owns the row.
                let roots = selected_model_identities(&case.package);
                let first = roots.iter().min().expect("a selected model");
                let position = case.package["semantic_graph"]["nodes"]
                    .as_array()
                    .expect("nodes")
                    .iter()
                    .position(|node| {
                        node["semantic_form"] == "model_root" && node["owner"]["identity"] == *first
                    })
                    .expect("model root");
                let row = format!("/semantic_graph/nodes/{position}/owner");
                (0..total)
                    .filter(|limit| match case.read_at(Some(*limit)) {
                        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
                            incomplete.path.map(|path| path.to_string()).as_deref()
                                == Some(row.as_str())
                        }
                        _ => false,
                    })
                    .count()
            };
            if !row {
                return;
            }
            let edges_at_row = at_row(&walked, total) - at_row(&control, baseline);
            assert_eq!(
                edges_at_row as u64, edges,
                "{conforming} at {exact}: the walk is charged at the selection row"
            );
        };
    // The walk does not stop at the parameter's record.
    charged("charge", "Money", "GiftTaxedMoney", 2, true);
    charged("settle", "TaxedMoney", "GiftTaxedMoney", 2, false);
    // A diamond costs its four distinct edges.
    charged("over_a", "A", "D", 4, false);
}

// --- TC-448: field reads (FR-347-AC-9) ------------------------------------------------

/// `operand.field` as a `filament.op.record.project` of `field` declared by
/// `declaring`, the operand a parameter typed `operand`.
fn read_of(declaring: &Ty, field: &str, operand: &Ty) -> Case {
    let mut scene = ledger_scene();
    let declaring = scene.ty(declaring);
    // An object operand is the declaration node itself: no parameter is typed
    // at an object type.
    let operand = match operand {
        Ty::Object(_) => scene.ty(operand),
        _ => scene.param("operand", operand),
    };
    let integer = scene.scalar("integer");
    let key = scene.application(App {
        tag: "expression",
        form: "query",
        operator: "query",
        identity: "filament.op.record.project",
        member: json!({"kind": "field", "declaration": node_id(&declaring), "name": field}),
        semantic: &integer,
        result: &integer,
        arguments: vec![reference(&operand)],
    });
    scene.finish().at(&key)
}

#[trace("TC-448", "FR-347-AC-9")]
#[test]
fn tc_448_a_record_field_is_read_over_a_conforming_operand() {
    let placed = Ty::Rec("OrderPlaced");
    let express = Ty::Rec("ExpressOrderPlaced");
    for operand in ["OrderPlaced", "ExpressOrderPlaced", "RushOrderPlaced"] {
        read_of(&placed, "occurred_at", &Ty::Rec(operand)).admits(operand);
    }
    // An inherited field, named on the child.
    read_of(&express, "order_id", &Ty::Rec("ExpressOrderPlaced")).admits("inherited");
    read_of(&express, "order_id", &Ty::Rec("RushOrderPlaced")).admits("inherited, grandchild");
    read_of(&placed, "occurred_at", &Ty::Rec("Voucher"))
        .ineligible("unrelated record", "/body/arguments/0");
    read_of(&placed, "occurred_at", &Ty::Object("Ledger"))
        .ineligible("object operand", "/body/arguments/0");
    // The child's operand does not give `OrderPlaced` the child's field.
    read_of(&placed, "promised_by", &Ty::Rec("ExpressOrderPlaced"))
        .ineligible("descendant-only field", "/body/operation/member/name");
    read_of(&express, "promised_by", &Ty::Rec("ExpressOrderPlaced")).admits("own field");
    // A record owns no operation.
    let mut scene = ledger_scene();
    let record = scene.ty(&placed);
    // Supply a valid reference receiver so the catalogued operand-family
    // check passes and the record's forbidden operation is the first defect.
    let operand = scene.param("operand", &Ty::Ref("Ledger"));
    let integer = scene.scalar("integer");
    let key = scene.dispatch(&record, "occurred_at", &[operand], &integer);
    scene
        .finish()
        .at(&key)
        .ineligible("operation naming a record", "/body/operation/member/name");
}

// --- TC-448: case arms and union payloads (FR-347-AC-10, AC-11) -------------------

/// A `case` over `Payment { Pay(Money), Alt(Money) }` whose arm bodies are
/// parameters typed `first` and `second`, whose binders are typed `binder`
/// (`Money` when it conforms) and whose result is `result`.
fn case_over(first: &Ty, second: &Ty, binder: &Ty, result: &Ty) -> Case {
    let mut scene = ledger_scene();
    let money = scene.ty(&Ty::Rec("Money"));
    let payment = scene.union(
        "Payment",
        &[("Pay", vec![money.clone()]), ("Alt", vec![money])],
    );
    let scrutinee = scene.param("scrutinee", &Ty::Key(payment));
    let arm = |scene: &mut Scene, member: &str, name: &str, binder_ty: &Ty, body: &Ty| {
        let binder = scene.param(&format!("binder_{name}"), binder_ty);
        let body = scene.param(&format!("body_{name}"), body);
        binding(
            member,
            aggregate(vec![aggregate(vec![reference(&binder)]), reference(&body)]),
        )
    };
    let pay = arm(&mut scene, "Pay", "pay", binder, first);
    let alt = arm(&mut scene, "Alt", "alt", &Ty::Rec("Money"), second);
    let result = scene.ty(result);
    let key = scene.application(App {
        tag: "expression",
        form: "case",
        operator: "case",
        identity: "filament.op.control.case",
        member: Value::Null,
        semantic: &result,
        result: &result,
        arguments: vec![reference(&scrutinee), pay, alt],
    });
    scene.finish().at(&key)
}

#[trace("TC-448", "FR-347-AC-10")]
#[test]
fn tc_448_a_case_arm_body_conforms_to_the_result_record() {
    let (money, taxed) = (Ty::Rec("Money"), Ty::Rec("TaxedMoney"));
    case_over(&taxed, &money, &money, &money).admits("TaxedMoney and Money at Money");
    case_over(&money, &money, &money, &money).admits("exact");
    case_over(&Ty::Rec("Voucher"), &money, &money, &taxed)
        .refuses_at_node("Voucher arm at TaxedMoney");
    case_over(&money, &taxed, &money, &taxed).refuses_at_node("Money arm at TaxedMoney");
    // A binder is bound to the payload type: it stays exact.
    case_over(&money, &money, &taxed, &money).refuses_at_node("child-typed binder");
    // A non-record result type stays exact.
    case_over(&Ty::Integer, &Ty::Integer, &money, &Ty::Integer).admits("Integer arms");
    case_over(&taxed, &taxed, &money, &Ty::Integer).refuses_at_node("record arm at Integer");
}

impl Case {
    /// `ill_typed`/`operator-ineligible` at the node itself.
    fn refuses_at_node(&self, label: &str) {
        self.ineligible(label, "");
    }
}

/// `Pay(payload)` of a union declaring a `declared` payload.
fn payload_of(declared: &'static str, payload: &Ty) -> Case {
    let mut scene = ledger_scene();
    let declared = scene.ty(&Ty::Rec(declared));
    let union = scene.union("Payment", &[("Pay", vec![declared])]);
    let term = scene.param("term", payload);
    let key = scene.union_value(&union, "Pay", &[term]);
    scene.finish().at(&key)
}

#[trace("TC-448", "FR-347-AC-11")]
#[test]
fn tc_448_a_union_payload_conforms_to_the_declared_record() {
    for payload in ["Money", "TaxedMoney", "GiftTaxedMoney"] {
        payload_of("Money", &Ty::Rec(payload)).admits(payload);
    }
    payload_of("TaxedMoney", &Ty::Rec("Voucher")).mismatch("Voucher at TaxedMoney");
    payload_of("TaxedMoney", &Ty::Rec("Money")).mismatch("Money at TaxedMoney");
    // A non-record payload stays exact.
    payload_of("Money", &Ty::Integer).mismatch("Integer at Money");
}

/// Trace: FR-347-AC-15, TC-448.
#[trace("TC-448", "FR-347-AC-15")]
#[test]
fn tc_448_tuple_value_positions_match_declared_types_exactly() {
    let mut scene = ledger_tuple_scene();
    let tuple = scene.ty(&Ty::ModelTuple("Pair"));
    let taxed = scene.param("taxed", &Ty::Rec("TaxedMoney"));
    let integer = scene.ty(&Ty::Integer);
    let key = scene.tuple_value(
        &tuple,
        vec![reference(&taxed), literal(&integer, "integer", "1")],
    );
    scene
        .finish()
        .at(&key)
        .admits("model tuple record position conforms");

    let mut scene = ledger_tuple_scene();
    let tuple = scene.ty(&Ty::ModelTuple("Pair"));
    let taxed = scene.param("taxed", &Ty::Rec("TaxedMoney"));
    let boolean = scene.ty(&Ty::Boolean);
    let key = scene.tuple_value(
        &tuple,
        vec![reference(&taxed), literal(&boolean, "boolean", "true")],
    );
    scene
        .finish()
        .at(&key)
        .mismatch("Boolean at the Integer position");

    let mut scene = ledger_tuple_scene();
    let tuple = scene.ty(&Ty::Tuple(vec![Ty::Rec("Money"), Ty::Integer]));
    let taxed = scene.param("taxed", &Ty::Rec("TaxedMoney"));
    let narrow = scene.param("narrow", &Ty::Range(0, 1000));
    let integer = Ty::Integer;
    let converted = numeric_to(
        &mut scene,
        "filament.op.numeric.convert",
        &integer,
        reference(&narrow),
    );
    let key = scene.tuple_value(&tuple, vec![reference(&taxed), reference(&converted)]);
    scene
        .finish()
        .at(&key)
        .admits("explicit numeric.convert at the Integer position");

    let mut scene = ledger_tuple_scene();
    let tuple = scene.ty(&Ty::Tuple(vec![Ty::Rec("Money"), Ty::Integer]));
    let taxed = scene.param("taxed", &Ty::Rec("TaxedMoney"));
    let narrow = scene.param("narrow", &Ty::Range(0, 1000));
    let key = scene.tuple_value(&tuple, vec![reference(&taxed), reference(&narrow)]);
    scene
        .finish()
        .at(&key)
        .mismatch("bare narrow integer at the Integer position");

    let mut scene = ledger_tuple_scene();
    let tuple = scene.ty(&Ty::Tuple(vec![Ty::Integer, Ty::Integer]));
    let integer = scene.ty(&Ty::Integer);
    let key = scene.tuple_value(&tuple, vec![literal(&integer, "integer", "1")]);
    scene
        .finish()
        .at(&key)
        .mismatch("short structural tuple value");

    let mut scene = ledger_tuple_scene();
    let tuple = scene.ty(&Ty::Tuple(vec![Ty::Integer, Ty::Integer]));
    let integer = scene.ty(&Ty::Integer);
    let key = scene.tuple_value(
        &tuple,
        vec![
            literal(&integer, "integer", "1"),
            literal(&integer, "integer", "2"),
            literal(&integer, "integer", "3"),
        ],
    );
    scene
        .finish()
        .at(&key)
        .mismatch("overlong structural tuple value");

    let mut scene = ledger_tuple_scene();
    let tuple = scene.ty(&Ty::ModelTuple("Pair"));
    let taxed = scene.param("taxed", &Ty::Rec("TaxedMoney"));
    let key = scene.tuple_value(&tuple, vec![reference(&taxed)]);
    scene
        .finish()
        .at(&key)
        .mismatch("short selected model tuple value");

    let mut scene = ledger_tuple_scene();
    let tuple = scene.ty(&Ty::ModelTuple("Pair"));
    let taxed = scene.param("taxed", &Ty::Rec("TaxedMoney"));
    let integer = scene.ty(&Ty::Integer);
    let key = scene.tuple_value(
        &tuple,
        vec![
            reference(&taxed),
            literal(&integer, "integer", "1"),
            literal(&integer, "integer", "2"),
            literal(&integer, "integer", "3"),
        ],
    );
    scene
        .finish()
        .at(&key)
        .mismatch("overlong selected model tuple value");

    let mut scene = ledger_tuple_scene();
    let tuple = scene.ty(&Ty::ModelTuple("Pair"));
    let voucher = scene.param("voucher", &Ty::Rec("Voucher"));
    let integer = scene.ty(&Ty::Integer);
    let key = scene.tuple_value(
        &tuple,
        vec![reference(&voucher), literal(&integer, "integer", "1")],
    );
    scene
        .finish()
        .at(&key)
        .mismatch("unrelated record at the Money position");

    let mut scene = ledger_tuple_scene();
    let tuple = scene.ty(&Ty::ModelTuple("Clocked"));
    let integer = scene.ty(&Ty::Integer);
    let key = scene.tuple_value(&tuple, vec![literal(&integer, "integer", "1")]);
    scene
        .finish()
        .at(&key)
        .admits("clock position is excluded from data terms");

    let mut scene = ledger_tuple_scene();
    let tuple = scene.ty(&Ty::ModelTuple("Clocked"));
    let key = scene.tuple_value(&tuple, vec![]);
    scene
        .finish()
        .at(&key)
        .mismatch("short clocked tuple data positions");

    let mut scene = ledger_tuple_scene();
    let tuple = scene.ty(&Ty::ModelTuple("Clocked"));
    let integer = scene.ty(&Ty::Integer);
    let key = scene.tuple_value(
        &tuple,
        vec![
            literal(&integer, "integer", "1"),
            literal(&integer, "integer", "2"),
        ],
    );
    scene
        .finish()
        .at(&key)
        .mismatch("extra clocked tuple data term");

    let mut scene = ledger_tuple_scene();
    let tuple = scene.ty(&Ty::ModelTuple("Clocked"));
    let narrow = scene.param("narrow", &Ty::Range(0, 9));
    let key = scene.tuple_value(&tuple, vec![reference(&narrow)]);
    scene
        .finish()
        .at(&key)
        .mismatch("narrow integer at a clock tuple data position");
}

// --- TC-448: body roots (FR-347-AC-12) ------------------------------------------------

/// A `function` of declared type `declared` whose `body` binding references
/// a root parameter typed `root`.
fn function_root(declared: &'static str, root: &Ty) -> Case {
    let mut scene = ledger_scene();
    let declared = scene.ty(&Ty::Rec(declared));
    let root = scene.param("root", root);
    let key = scene.function(
        "produce",
        &declared,
        vec![
            binding("parameters", aggregate(vec![reference(&root)])),
            binding("body", reference(&root)),
        ],
    );
    scene.finish().at(&key)
}

/// A node of declared type `declared` whose body is the application
/// `mint_<root>()`, tagged `expression` or `function`.
fn application_root(tag: &str, declared: &'static str, root: &str) -> Case {
    let mut scene = ledger_scene();
    let declared = scene.ty(&Ty::Rec(declared));
    let ledger = scene.ty(&Ty::Object("Ledger"));
    let receiver = scene.param("receiver", &Ty::Ref("Ledger"));
    let result = scene.ty(&result_of(root));
    let (form, role_tag) = if tag == "function" {
        ("pure_function", "function")
    } else {
        ("call", "expression")
    };
    let key = scene.application(App {
        tag: role_tag,
        form,
        operator: "call",
        identity: "filament.op.model.dispatch_call",
        member: json!({"kind": "operation", "declaration": node_id(&ledger), "name": root}),
        semantic: &declared,
        result: &result,
        arguments: vec![reference(&receiver)],
    });
    scene.finish().at(&key)
}

#[trace("TC-448", "FR-347-AC-12")]
#[test]
fn tc_448_a_body_root_conforms_to_the_declared_record() {
    for root in ["Money", "TaxedMoney", "GiftTaxedMoney"] {
        function_root("Money", &Ty::Rec(root)).admits(root);
    }
    function_root("Money", &Ty::Rec("Voucher")).mismatch("Voucher root");
    function_root("TaxedMoney", &Ty::Rec("Money")).mismatch("ancestor root");
    function_root("Money", &Ty::Integer).mismatch("non-record root at a record");
    for tag in ["expression", "function"] {
        for root in ["mint_money", "mint_taxed", "mint_gift"] {
            application_root(tag, "Money", root).admits(&format!("{tag} {root}"));
        }
        application_root(tag, "Money", "mint_voucher").mismatch(&format!("{tag} Voucher"));
        application_root(tag, "TaxedMoney", "mint_money").mismatch(&format!("{tag} ancestor"));
    }
    // A non-record declared type stays exact.
    let mut scene = ledger_scene();
    let integer = scene.ty(&Ty::Integer);
    let root = scene.param("root", &Ty::Rec("Money"));
    let key = scene.function(
        "produce",
        &integer,
        vec![
            binding("parameters", aggregate(vec![reference(&root)])),
            binding("body", reference(&root)),
        ],
    );
    scene.finish().at(&key).mismatch("record root at Integer");
}

// --- TC-448: function.call arguments (FR-347-AC-13, AC-14) ------------------------

/// A `function.call` of the callee `f` with `arguments` parameters typed as
/// given. `f` takes `signature` parameters and binds the `members` after the
/// `parameters` binding the `bindings` closure builds from their keys.
fn function_call(
    signature: &[Ty],
    arguments: &[Ty],
    bindings: impl FnOnce(&mut Scene, &[String]) -> Vec<Value>,
) -> Case {
    let mut scene = ledger_scene();
    let integer = scene.ty(&Ty::Integer);
    let declared: Vec<String> = signature
        .iter()
        .enumerate()
        .map(|(position, ty)| scene.param(&format!("p{position}"), ty))
        .collect();
    let members = bindings(&mut scene, &declared);
    let callee = scene.function("f", &integer, members);
    let mut terms = vec![reference(&callee)];
    for (position, ty) in arguments.iter().enumerate() {
        let argument = scene.param(&format!("a{position}"), ty);
        terms.push(reference(&argument));
    }
    let key = scene.application(App {
        tag: "expression",
        form: "call",
        operator: "call",
        identity: "filament.op.function.call",
        member: Value::Null,
        semantic: &integer,
        result: &integer,
        arguments: terms,
    });
    scene.finish().at(&key)
}

/// The well-formed members: `parameters` listing every declared parameter and
/// a `body` reading the last one.
fn well_formed(_: &mut Scene, declared: &[String]) -> Vec<Value> {
    vec![
        binding(
            "parameters",
            aggregate(declared.iter().map(|key| reference(key)).collect()),
        ),
        binding("body", reference(declared.last().expect("a parameter"))),
    ]
}

#[trace("TC-448", "FR-347-AC-13")]
#[test]
fn tc_448_function_call_arguments_follow_the_callee_parameters() {
    let signature = [Ty::Rec("Money"), Ty::Integer];
    for first in ["Money", "TaxedMoney", "GiftTaxedMoney"] {
        function_call(&signature, &[Ty::Rec(first), Ty::Integer], well_formed).admits(first);
    }
    let arguments = "/body/arguments";
    function_call(&signature, &[Ty::Rec("Money")], well_formed)
        .ineligible("one argument", arguments);
    function_call(
        &signature,
        &[Ty::Rec("Money"), Ty::Integer, Ty::Integer],
        well_formed,
    )
    .ineligible("three arguments", arguments);
    function_call(&signature, &[Ty::Rec("Voucher"), Ty::Integer], well_formed)
        .ineligible("Voucher", ARG_1);
    function_call(
        &signature,
        &[Ty::Rec("Money"), Ty::Range(0, 5)],
        well_formed,
    )
    .ineligible("Int[0, 5] at Integer", ARG_2);
    function_call(
        &[Ty::Rec("TaxedMoney"), Ty::Integer],
        &[Ty::Rec("Money"), Ty::Integer],
        well_formed,
    )
    .ineligible("ancestor", ARG_1);
    // A parameter the body never reads still counts.
    let unread = [Ty::Rec("Money"), Ty::Integer, Ty::Integer];
    let reads_one = |_: &mut Scene, declared: &[String]| {
        vec![
            binding(
                "parameters",
                aggregate(declared.iter().map(|key| reference(key)).collect()),
            ),
            binding("body", reference(&declared[1])),
        ]
    };
    function_call(&unread, &[Ty::Rec("Money"), Ty::Integer], reads_one)
        .ineligible("two arguments, three parameters", arguments);
    function_call(
        &unread,
        &[Ty::Rec("Money"), Ty::Integer, Ty::Integer],
        reads_one,
    )
    .admits("three arguments, three parameters");
    // A callee without the binding takes no argument check.
    let unbound = |_: &mut Scene, declared: &[String]| {
        vec![binding(
            "body",
            reference(declared.last().expect("a parameter")),
        )]
    };
    function_call(
        &signature,
        &[Ty::Rec("Voucher"), Ty::Integer, Ty::Integer],
        unbound,
    )
    .admits("no parameters binding");
}

fn parameters_binding(entries: Vec<Value>) -> Value {
    binding("parameters", aggregate(entries))
}

#[trace("TC-448", "FR-347-AC-14")]
#[test]
fn tc_448_a_malformed_parameters_binding_is_refused_at_the_callee() {
    let signature = [Ty::Rec("Money"), Ty::Integer];
    let arguments = [Ty::Rec("Money"), Ty::Integer];
    let callee = "/body/arguments/0";
    type Members = Box<dyn Fn(&mut Scene, &[String]) -> Vec<Value>>;
    let cases: Vec<(&str, Members)> = vec![
        (
            "second in the body",
            Box::new(|_, declared| {
                vec![
                    binding("body", reference(&declared[1])),
                    parameters_binding(declared.iter().map(|key| reference(key)).collect()),
                ]
            }),
        ),
        (
            "not an aggregate",
            Box::new(|_, declared| {
                vec![
                    binding("parameters", reference(&declared[0])),
                    binding("body", reference(&declared[1])),
                ]
            }),
        ),
        (
            "a literal entry",
            Box::new(|scene, declared| {
                let integer = scene.scalar("integer");
                vec![
                    parameters_binding(vec![
                        literal(&integer, "integer", "1"),
                        reference(&declared[1]),
                    ]),
                    binding("body", reference(&declared[1])),
                ]
            }),
        ),
        (
            "an absent target",
            Box::new(|_, declared| {
                let absent = "absent".to_owned();
                vec![
                    parameters_binding(vec![reference(&absent), reference(&declared[1])]),
                    binding("body", reference(&declared[1])),
                ]
            }),
        ),
        (
            "a non-parameter target",
            Box::new(|scene, declared| {
                let integer = scene.scalar("integer");
                vec![
                    parameters_binding(vec![reference(&integer), reference(&declared[1])]),
                    binding("body", reference(&declared[1])),
                ]
            }),
        ),
    ];
    for (label, members) in cases {
        function_call(&signature, &arguments, members).ineligible(label, callee);
    }
}

// --- the widening stops at record value types --------------------------------

#[trace("TC-448", "FR-347-AC-10", "FR-347-AC-13")]
#[test]
fn tc_448_a_parameter_typed_at_an_object_type_is_still_refused() {
    let mut scene = ledger_scene();
    let key = scene.param("object_typed", &Ty::Object("Ledger"));
    let case = scene.finish().at(&key);
    let CheckedPackageV2ReadResult::Refused(refusal) = case.read() else {
        panic!("a parameter typed at an object type must be refused");
    };
    assert_eq!(
        (refusal.code, refusal.cause, refusal.loci[0].path.clone()),
        (
            Code::InvalidSemanticGraph,
            None,
            Some(pointer(&format!(
                "/semantic_graph/nodes/{}/semantic_type",
                case.node
            )))
        ),
        "{refusal:?}"
    );
}

#[trace("TC-448", "FR-347-AC-11")]
#[test]
fn tc_448_a_union_payload_naming_an_object_type_is_still_refused() {
    let mut scene = ledger_scene();
    let object = scene.ty(&Ty::Object("Ledger"));
    let key = scene.union("Holder", &[("Hold", vec![object])]);
    scene.finish().at(&key).refuses(
        "object payload",
        Code::IllTyped,
        Cause::OperatorIneligible,
        "",
    );
}

/// Required operation members distinguish an absent field from explicit null.
#[trace("FR-038-AC-81")]
#[test]
fn operation_mode_and_member_are_required_even_when_null_is_admitted() {
    let mut scene = Scene::new(Vec::new());
    let integer = scene.ty(&Ty::Integer);
    let key = scene.application(App {
        tag: "expression",
        form: "binary",
        operator: "binary",
        identity: "filament.op.integer.add",
        member: Value::Null,
        semantic: &integer,
        result: &integer,
        arguments: vec![
            literal(&integer, "integer", "1"),
            literal(&integer, "integer", "2"),
        ],
    });
    let base = scene.finish().at(&key);
    base.admits("integer add with mode and member explicitly null");

    for field in ["mode", "member"] {
        let mut case = Case {
            package: base.package.clone(),
            evidence: base.evidence.clone(),
            node: base.node,
        };
        let operation = case.package["semantic_graph"]["nodes"][case.node]["body"]["operation"]
            .as_object_mut()
            .expect("operation object");
        assert_eq!(operation.remove(field), Some(Value::Null));
        settle(&mut case.package);
        assert_ne!(case.package, base.package);
        let CheckedPackageV2ReadResult::Refused(refusal) = case.read() else {
            panic!("omitting {field} must refuse");
        };
        assert_eq!(
            refusal.code,
            Code::InvalidSemanticGraph,
            "{field}: {refusal:?}"
        );
        assert_eq!(refusal.cause, None, "{field}: {refusal:?}");
        assert_eq!(
            refusal.loci[0].path.clone(),
            Some(pointer(&format!(
                "/semantic_graph/nodes/{}/body/operation",
                case.node
            ))),
            "{field}: {refusal:?}",
        );

        // A catalog error cannot displace the containing object's shape error.
        case.package["semantic_graph"]["nodes"][case.node]["body"]["operation"]["identity"] =
            json!("filament.op.unknown");
        settle(&mut case.package);
        let CheckedPackageV2ReadResult::Refused(before_catalog) = case.read() else {
            panic!("omitting {field} with an unknown identity must refuse");
        };
        assert_eq!(before_catalog.code, Code::InvalidSemanticGraph);
        assert_eq!(before_catalog.cause, None);
        assert_eq!(
            before_catalog.loci[0].path.clone(),
            refusal.loci[0].path.clone()
        );

        case.package["semantic_graph"]["nodes"][case.node]["body"]["operation"]["identity"] =
            json!("filament.op.integer.add");
        case.package["semantic_graph"]["nodes"][case.node]["body"]["operation"][field] =
            Value::Null;
        settle(&mut case.package);
        assert_eq!(
            case.package, base.package,
            "restoring {field} restores the original package"
        );
        case.admits(&format!("restored explicit null {field}"));
    }
}

// --- TC-492 (FSpec TC-2099): integer containment at result positions (FSpec FR-322-AC-495..502) ---

/// A `function` named `name` of declared type `declared` whose `body`
/// binding references the node `root`, over the parameters `parameters`.
fn function_over(
    scene: &mut Scene,
    name: &str,
    declared: &Ty,
    parameters: &[&String],
    root: &str,
) -> String {
    let declared = scene.ty(declared);
    scene.function(
        name,
        &declared,
        vec![
            binding(
                "parameters",
                aggregate(parameters.iter().map(|key| reference(key)).collect()),
            ),
            binding("body", reference(root)),
        ],
    )
}

/// IC-01 to IC-04 and IC-08: `function c1 using v(x: <root>): <declared> pure { x }`.
fn function_result(declared: &Ty, root: &Ty) -> Case {
    let mut scene = ledger_scene();
    let x = scene.param("x", root);
    let key = function_over(&mut scene, "c1", declared, &[&x], &x);
    scene.finish().at(&key)
}

/// A `filament.op.numeric.<name>` application of `argument` typed at `ty`.
fn numeric_to(scene: &mut Scene, identity: &str, ty: &Ty, argument: Value) -> String {
    let ty = scene.ty(ty);
    scene.application(App {
        tag: "expression",
        form: "conversion",
        operator: "convert",
        identity,
        member: json!({"kind": "type_argument", "declaration": node_id(&ty)}),
        semantic: &ty,
        result: &ty,
        arguments: vec![argument],
    })
}

/// A Boolean literal value node standing for `true`.
fn truth(scene: &mut Scene) -> String {
    let boolean = scene.ty(&Ty::Boolean);
    let key = "value/literal/true".to_owned();
    scene.add(wire_node(
        &key,
        "value",
        "literal",
        &boolean,
        &[],
        "expression",
        literal(&boolean, "boolean", "true"),
    ))
}

/// `function g using v(a: <parameter>): Boolean pure { true }` and the call
/// `g(<argument>)` over `x: <argument type>`. `converted` writes the one
/// `numeric.convert` of FR-149 to the parameter type over `x`.
fn call_of(parameter: &Ty, argument: &Ty, converted: bool) -> Case {
    let mut scene = ledger_scene();
    let boolean = scene.ty(&Ty::Boolean);
    let a = scene.param("a", parameter);
    let value = truth(&mut scene);
    let g = scene.function(
        "g",
        &boolean,
        vec![
            binding("parameters", aggregate(vec![reference(&a)])),
            binding("body", reference(&value)),
        ],
    );
    let x = scene.param("x", argument);
    let term = if converted {
        let convert = numeric_to(
            &mut scene,
            "filament.op.numeric.convert",
            parameter,
            reference(&x),
        );
        reference(&convert)
    } else {
        reference(&x)
    };
    let key = scene.application(App {
        tag: "expression",
        form: "call",
        operator: "call",
        identity: "filament.op.function.call",
        member: Value::Null,
        semantic: &boolean,
        result: &boolean,
        arguments: vec![reference(&g), term],
    });
    scene.finish().at(&key)
}

/// A node of the `if` the producer writes: `then` and `otherwise` are the
/// branch terms, `result` the application's `result_type`.
fn conditional(
    scene: &mut Scene,
    condition: &str,
    then: Value,
    otherwise: Value,
    result: &Ty,
) -> String {
    let result = scene.ty(result);
    scene.application(App {
        tag: "expression",
        form: "conditional",
        operator: "conditional",
        identity: "filament.op.control.if",
        member: Value::Null,
        semantic: &result,
        result: &result,
        arguments: vec![reference(condition), then, otherwise],
    })
}

/// The literal integer `value` narrowed to `ty` (FSL FR-093-AC-4).
fn narrowed(scene: &mut Scene, value: &str, ty: &Ty) -> String {
    let integer = scene.ty(&Ty::Integer);
    numeric_to(
        scene,
        "filament.op.numeric.narrow",
        ty,
        literal(&integer, "integer", value),
    )
}

/// How the two branches of the `if` of `if_case` are written.
enum Branch {
    /// The parameter `x: Int[0, 9]`.
    X,
    /// The parameter `y: Int[5, 20]`.
    Y,
    /// The parameter `w: Int[0, 1000]`.
    W,
    /// The literal narrowed to the range.
    Narrow(&'static str, i128, i128),
}

/// `function f using v(c: Boolean, x: Int[0, 9], y: Int[5, 20], w: Int[0, 1000])
/// : <result> pure { if c then <then> else <otherwise> }` with the `if`
/// typed `result`; the case's node under test is the `if` application when
/// `at_function` is false and the function otherwise.
fn if_case(then: Branch, otherwise: Branch, result: &Ty, at_function: bool) -> Case {
    let mut scene = ledger_scene();
    let c = scene.param("c", &Ty::Boolean);
    let x = scene.param("x", &Ty::Range(0, 9));
    let y = scene.param("y", &Ty::Range(5, 20));
    let w = scene.param("w", &Ty::Range(0, 1000));
    let term = |scene: &mut Scene, branch: Branch| match branch {
        Branch::X => reference(&x),
        Branch::Y => reference(&y),
        Branch::W => reference(&w),
        Branch::Narrow(value, lower, upper) => {
            reference(&narrowed(scene, value, &Ty::Range(lower, upper)))
        }
    };
    let then = term(&mut scene, then);
    let otherwise = term(&mut scene, otherwise);
    let key = conditional(&mut scene, &c, then, otherwise, result);
    let function = function_over(&mut scene, "f", result, &[&c, &x, &y, &w], &key);
    scene
        .finish()
        .at(if at_function { &function } else { &key })
}

#[trace("TC-492", "FR-038-AC-586")]
#[test]
fn tc_492_a_function_result_admits_a_contained_integer_root() {
    function_result(&Ty::Range(0, 10), &Ty::Range(0, 9)).admits("IC-01");
    function_result(&Ty::Integer, &Ty::Range(0, 9)).admits("IC-02");
    function_result(&Ty::Range(0, 9), &Ty::Range(0, 9)).admits("equal range");
    function_result(&Ty::Range(0, 8), &Ty::Range(0, 9)).mismatch("IC-03");
    function_result(&Ty::Range(0, 10), &Ty::Integer).mismatch("IC-04");
    function_result(&Ty::Range(0, 10), &Ty::Range(1, 10)).admits("an equal upper bound");
    function_result(&Ty::Range(-5, 10), &Ty::Range(0, 9)).admits("a lower negative bound");
    function_result(&Ty::Range(1, 9), &Ty::Range(0, 9)).mismatch("a raised lower bound");
    function_result(&Ty::Range(0, 9), &Ty::Range(0, 10)).mismatch("a raised upper bound");
    function_result(&Ty::Integer, &Ty::Boolean).mismatch("a Boolean root at Integer");
    function_result(&Ty::Range(0, 10), &Ty::Boolean).mismatch("a Boolean root");
    function_result(&Ty::Boolean, &Ty::Range(0, 9)).mismatch("a Boolean result");
    function_result(
        &Ty::Sequence(Box::new(Ty::Range(0, 10)), 0, 3),
        &Ty::Sequence(Box::new(Ty::Range(0, 9)), 0, 3),
    )
    .mismatch("IC-08: not through a constructor");
    function_result(
        &Ty::Set(Box::new(Ty::Range(0, 10)), 0, 3),
        &Ty::Set(Box::new(Ty::Range(0, 9)), 0, 3),
    )
    .mismatch("a Set is no more contained than a Sequence");
}

#[trace("TC-492", "FR-038-AC-586")]
#[test]
fn tc_492_an_expression_root_admits_a_contained_integer_result() {
    // An `expression` node root: the application's `result_type` against the node's.
    for (declared, root, admitted) in [
        (Ty::Range(0, 10), Ty::Range(0, 9), true),
        (Ty::Integer, Ty::Range(0, 9), true),
        (Ty::Range(0, 8), Ty::Range(0, 9), false),
        (Ty::Range(0, 10), Ty::Integer, false),
    ] {
        let mut scene = ledger_scene();
        let declared = scene.ty(&declared);
        let root = scene.ty(&root);
        let integer = scene.ty(&Ty::Integer);
        let key = scene.application(App {
            tag: "expression",
            form: "conversion",
            operator: "convert",
            identity: "filament.op.numeric.convert",
            member: json!({"kind": "type_argument", "declaration": node_id(&root)}),
            semantic: &declared,
            result: &root,
            arguments: vec![literal(&integer, "integer", "1")],
        });
        let case = scene.finish().at(&key);
        if admitted {
            case.admits(&format!("{declared} at {root}"));
        } else {
            case.mismatch(&format!("{declared} at {root}"));
        }
    }
}

#[trace("TC-492", "FR-038-AC-587")]
#[test]
fn tc_492_a_call_argument_keeps_the_exact_match() {
    call_of(&Ty::Range(0, 10), &Ty::Range(0, 9), true).admits("IC-05");
    call_of(&Ty::Range(0, 10), &Ty::Range(0, 10), false).admits("an equal bare argument");
    call_of(&Ty::Range(0, 10), &Ty::Range(0, 9), false).ineligible("IC-06", ARG_1);
    call_of(&Ty::Range(0, 8), &Ty::Range(0, 9), false).ineligible("IC-07", ARG_1);
    call_of(&Ty::Integer, &Ty::Range(0, 9), false).ineligible("a bare argument at Integer", ARG_1);
}

#[trace("TC-492", "FR-038-AC-588")]
#[test]
fn tc_492_both_branches_of_an_if_are_checked_against_its_result() {
    let narrow = |value, upper| Branch::Narrow(value, 0, upper);
    if_case(Branch::X, narrow("10", 10), &Ty::Range(0, 10), false).admits("IC-09");
    if_case(narrow("10", 10), Branch::X, &Ty::Range(0, 10), false).admits("IC-16");
    if_case(Branch::X, Branch::Y, &Ty::Range(0, 20), false).admits("IC-17: the hull result");
    if_case(Branch::X, Branch::Y, &Ty::Range(0, 20), true).admits("IC-17 at the function");
    if_case(Branch::X, narrow("8", 8), &Ty::Range(0, 8), false).ineligible("IC-12", "");
    if_case(narrow("10", 10), Branch::W, &Ty::Range(0, 10), false).ineligible("IC-18", "");
    if_case(Branch::W, narrow("10", 10), &Ty::Range(0, 10), false)
        .ineligible("a then-branch outside the result", "");
    if_case(Branch::X, Branch::Y, &Ty::Range(0, 19), false)
        .ineligible("a hull that misses an upper bound", "");
    if_case(Branch::X, Branch::Y, &Ty::Range(1, 20), false)
        .ineligible("a hull that misses a lower bound", "");
    if_case(Branch::X, Branch::Y, &Ty::Integer, false).admits("Integer takes every range");
}

/// IC-10 and IC-13: a `case` over `union U { A(Int[0, 9]), B(Int[0, 20]) }`
/// whose arm bodies are the two binder reads.
fn case_of_ranges(result: &Ty) -> Case {
    let mut scene = ledger_scene();
    let (low, high) = (scene.ty(&Ty::Range(0, 9)), scene.ty(&Ty::Range(0, 20)));
    let union = scene.union("U", &[("A", vec![low]), ("B", vec![high])]);
    let scrutinee = scene.param("scrutinee", &Ty::Key(union));
    let a = scene.param("a", &Ty::Range(0, 9));
    let b = scene.param("b", &Ty::Range(0, 20));
    let arm = |member: &str, binder: &String| {
        binding(
            member,
            aggregate(vec![aggregate(vec![reference(binder)]), reference(binder)]),
        )
    };
    let result = scene.ty(result);
    let key = scene.application(App {
        tag: "expression",
        form: "case",
        operator: "case",
        identity: "filament.op.control.case",
        member: Value::Null,
        semantic: &result,
        result: &result,
        arguments: vec![reference(&scrutinee), arm("A", &a), arm("B", &b)],
    });
    scene.finish().at(&key)
}

#[trace("TC-492", "FR-038-AC-588")]
#[test]
fn tc_492_case_arm_bodies_admit_a_contained_integer() {
    case_of_ranges(&Ty::Integer).admits("IC-10");
    case_of_ranges(&Ty::Range(0, 20)).admits("the widest arm");
    case_of_ranges(&Ty::Range(0, 9)).refuses_at_node("IC-13");
    case_of_ranges(&Ty::Range(1, 20)).refuses_at_node("a raised lower bound");
}

/// `let <name>: <binder> = <bound> in <body>` as the body of a function of
/// `result`; the node under test is the `let` application. The body reads
/// the binder, or calls `g(a: <parameter>)` over it when `parameter` is set.
fn let_case(bound: &Ty, binder: &Ty, result: &Ty, parameter: Option<&Ty>, at_call: bool) -> Case {
    let mut scene = ledger_scene();
    let x = scene.param("x", bound);
    let y = scene.param("y", binder);
    let mut call_key = None;
    let body = match parameter {
        None => reference(&y),
        Some(parameter) => {
            let boolean = scene.ty(&Ty::Boolean);
            let a = scene.param("a", parameter);
            let value = truth(&mut scene);
            let g = scene.function(
                "g",
                &boolean,
                vec![
                    binding("parameters", aggregate(vec![reference(&a)])),
                    binding("body", reference(&value)),
                ],
            );
            let call = scene.application(App {
                tag: "expression",
                form: "call",
                operator: "call",
                identity: "filament.op.function.call",
                member: Value::Null,
                semantic: &boolean,
                result: &boolean,
                arguments: vec![reference(&g), reference(&y)],
            });
            call_key = Some(call.clone());
            reference(&call)
        }
    };
    let result = scene.ty(result);
    let key = scene.application(App {
        tag: "expression",
        form: "let",
        operator: "let",
        identity: "filament.op.control.let",
        member: Value::Null,
        semantic: &result,
        result: &result,
        arguments: vec![let_binding("y", &y, reference(&x)), body],
    });
    let at = if at_call {
        call_key.unwrap_or(key)
    } else {
        key
    };
    scene.finish().at(&at)
}

#[trace("TC-492", "FR-038-AC-588")]
#[test]
fn tc_492_a_let_body_is_checked_against_its_result() {
    let (low, wide) = (Ty::Range(0, 9), Ty::Range(0, 10));
    let_case(&low, &low, &wide, None, false).admits("IC-11");
    let_case(&low, &low, &Ty::Range(0, 8), None, false)
        .ineligible("a let body outside its result", "");
    let thousand = Ty::Range(0, 1000);
    let_case(&thousand, &thousand, &Ty::Boolean, Some(&thousand), false)
        .admits("an unannotated let");
}

/// IC-20: `let r: Money = <bound>() in r`, the binder typed `binder`.
fn let_record_case(binder: &'static str, bound: &str) -> Case {
    let mut scene = ledger_scene();
    let ledger = scene.ty(&Ty::Object("Ledger"));
    let receiver = scene.param("receiver", &Ty::Ref("Ledger"));
    let result = scene.ty(&result_of(bound));
    let value = scene.application(App {
        tag: "expression",
        form: "call",
        operator: "call",
        identity: "filament.op.model.dispatch_call",
        member: json!({"kind": "operation", "declaration": node_id(&ledger), "name": bound}),
        semantic: &result,
        result: &result,
        arguments: vec![reference(&receiver)],
    });
    let r = scene.param("y", &Ty::Rec(binder));
    let money = scene.ty(&Ty::Rec("Money"));
    let key = scene.application(App {
        tag: "expression",
        form: "let",
        operator: "let",
        identity: "filament.op.control.let",
        member: Value::Null,
        semantic: &money,
        result: &money,
        arguments: vec![let_binding("y", &r, reference(&value)), reference(&r)],
    });
    scene.finish().at(&key)
}

/// What a vector's package reads as.
enum Expect {
    Admitted,
    /// `ill_typed`/`type-mismatch` at the node under test.
    Mismatch,
    /// `ill_typed`/`operator-ineligible` at `tail` below the node under test.
    Ineligible(&'static str),
}

/// The twenty-one packages of FSpec FR-322-AC-495 to FR-322-AC-502 and
/// FR-322-AC-512 (`IC-01` to `IC-21`), each built here and read below. A vector file can be serialized
/// from this list: `Case.package` is the wire package of each.
fn ic_vectors() -> Vec<(&'static str, Case, Expect)> {
    let (nine, ten, eight) = (Ty::Range(0, 9), Ty::Range(0, 10), Ty::Range(0, 8));
    let (thousand, almost) = (Ty::Range(0, 1000), Ty::Range(0, 999));
    let sequence = |upper| Ty::Sequence(Box::new(Ty::Range(0, upper)), 0, 3);
    let narrow = |value, upper| Branch::Narrow(value, 0, upper);
    vec![
        ("IC-01", function_result(&ten, &nine), Expect::Admitted),
        (
            "IC-02",
            function_result(&Ty::Integer, &nine),
            Expect::Admitted,
        ),
        ("IC-03", function_result(&eight, &nine), Expect::Mismatch),
        (
            "IC-04",
            function_result(&ten, &Ty::Integer),
            Expect::Mismatch,
        ),
        ("IC-05", call_of(&ten, &nine, true), Expect::Admitted),
        (
            "IC-06",
            call_of(&ten, &nine, false),
            Expect::Ineligible(ARG_1),
        ),
        (
            "IC-07",
            call_of(&eight, &nine, false),
            Expect::Ineligible(ARG_1),
        ),
        (
            "IC-08",
            function_result(&sequence(10), &sequence(9)),
            Expect::Mismatch,
        ),
        (
            "IC-09",
            if_case(Branch::X, narrow("10", 10), &ten, false),
            Expect::Admitted,
        ),
        ("IC-10", case_of_ranges(&Ty::Integer), Expect::Admitted),
        (
            "IC-11",
            let_case(&nine, &nine, &ten, None, false),
            Expect::Admitted,
        ),
        (
            "IC-12",
            if_case(Branch::X, narrow("8", 8), &eight, false),
            Expect::Ineligible(""),
        ),
        ("IC-13", case_of_ranges(&nine), Expect::Ineligible("")),
        (
            "IC-14",
            let_case(
                &thousand,
                &Ty::Integer,
                &Ty::Boolean,
                Some(&Ty::Integer),
                false,
            ),
            Expect::Admitted,
        ),
        (
            "IC-15",
            let_case(&thousand, &almost, &Ty::Boolean, Some(&almost), false),
            Expect::Ineligible(""),
        ),
        (
            "IC-16",
            if_case(narrow("10", 10), Branch::X, &ten, false),
            Expect::Admitted,
        ),
        (
            "IC-17",
            if_case(Branch::X, Branch::Y, &Ty::Range(0, 20), false),
            Expect::Admitted,
        ),
        (
            "IC-18",
            if_case(narrow("10", 10), Branch::W, &ten, false),
            Expect::Ineligible(""),
        ),
        (
            "IC-19",
            let_case(&thousand, &thousand, &Ty::Boolean, Some(&Ty::Integer), true),
            Expect::Ineligible(ARG_1),
        ),
        (
            "IC-20",
            let_record_case("Money", "mint_taxed"),
            Expect::Admitted,
        ),
        (
            "IC-21",
            let_unread_case(&thousand, &almost),
            Expect::Ineligible(""),
        ),
    ]
}

/// IC-21: `let y: <binder> = x in true` over `x: <bound>`; the body never
/// reads the binder, so only its link finds it.
fn let_unread_case(bound: &Ty, binder: &Ty) -> Case {
    let mut scene = ledger_scene();
    let x = scene.param("x", bound);
    let y = scene.param("y", binder);
    let body = truth(&mut scene);
    let boolean = scene.ty(&Ty::Boolean);
    let key = scene.application(App {
        tag: "expression",
        form: "let",
        operator: "let",
        identity: "filament.op.control.let",
        member: Value::Null,
        semantic: &boolean,
        result: &boolean,
        arguments: vec![let_binding("y", &y, reference(&x)), reference(&body)],
    });
    scene.finish().at(&key)
}

/// `Case::package`'s `filament.op.control.let` node, edited by `edit`.
fn edit_let(case: &mut Case, edit: impl FnOnce(&mut Value)) {
    let nodes = case.package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let node = nodes
        .iter_mut()
        .find(|node| node["body"]["operation"]["identity"] == "filament.op.control.let")
        .expect("a let node");
    edit(node);
}

/// IC-14 as built, whose `let` is the node under test.
fn ic_14_let() -> Case {
    let thousand = Ty::Range(0, 1000);
    let mut case = let_case(
        &thousand,
        &Ty::Integer,
        &Ty::Boolean,
        Some(&Ty::Integer),
        false,
    );
    let position = case.package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["body"]["operation"]["identity"] == "filament.op.control.let")
        .expect("a let node");
    case.node = position;
    case
}

#[trace("TC-492", "FR-038-AC-595")]
#[test]
fn tc_492_ic_21_an_unread_narrow_binder_is_refused() {
    let thousand = Ty::Range(0, 1000);
    let almost = Ty::Range(0, 999);
    let_unread_case(&thousand, &almost).ineligible("IC-21", "");
    let_unread_case(&thousand, &thousand).admits("an unread binder of its value's type");
    let_unread_case(&thousand, &Ty::Integer).admits("an unread Integer binder");
    let_unread_case(&almost, &thousand).admits("an unread wider binder");
}

#[trace("TC-492", "FR-038-AC-595")]
#[test]
fn tc_492_the_let_binder_link_is_checked() {
    ic_14_let().admits("IC-14 carries its binder link");
    let invalid = |case: &Case, label: &str, tail: &str| {
        case.refuses(label, Code::InvalidPackage, Cause::InvalidValue, tail);
    };

    let mut missing = ic_14_let();
    edit_let(&mut missing, |node| {
        let binder = node["body"]["arguments"][0]
            .as_object_mut()
            .expect("binding")
            .remove("binder");
        // The join then names no binder either.
        node["dependencies"]
            .as_array_mut()
            .expect("dependencies")
            .retain(|dependency| Some(dependency) != binder.as_ref());
    });
    invalid(&missing, "binder removed", "/body/arguments/0");

    let mut misnamed = ic_14_let();
    {
        let mut scene = ledger_scene();
        let z = scene.param("z", &Ty::Integer);
        let nodes = scene.finish().package["semantic_graph"]["nodes"].clone();
        let z = nodes
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["node_id"] == node_id(&z))
            .expect("z")
            .clone();
        let id = z["node_id"].clone();
        misnamed.package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(z);
        edit_let(&mut misnamed, |node| {
            let old = node["body"]["arguments"][0]["binder"].clone();
            node["body"]["arguments"][0]["binder"] = id.clone();
            let dependencies = node["dependencies"].as_array_mut().expect("dependencies");
            dependencies.retain(|dependency| *dependency != old);
            dependencies.push(id);
        });
    }
    invalid(&misnamed, "binder named z", "/body/arguments/0/binder");

    let mut unlisted = ic_14_let();
    edit_let(&mut unlisted, |node| {
        let binder = node["body"]["arguments"][0]["binder"].clone();
        node["dependencies"]
            .as_array_mut()
            .expect("dependencies")
            .retain(|dependency| *dependency != binder);
    });
    invalid(
        &unlisted,
        "binder outside dependencies",
        "/body/arguments/0/binder",
    );

    let mut not_a_parameter = ic_14_let();
    edit_let(&mut not_a_parameter, |node| {
        let target = node["body"]["arguments"][0]["value"]["target"].clone();
        let old = std::mem::replace(&mut node["body"]["arguments"][0]["binder"], target);
        node["dependencies"]
            .as_array_mut()
            .expect("dependencies")
            .retain(|dependency| *dependency != old);
    });
    invalid(
        &not_a_parameter,
        "binder names a non-binder",
        "/body/arguments/0/binder",
    );

    let mut dangling = ic_14_let();
    edit_let(&mut dangling, |node| {
        let old = std::mem::replace(
            &mut node["body"]["arguments"][0]["binder"],
            node_id("value/parameter/absent"),
        );
        node["dependencies"]
            .as_array_mut()
            .expect("dependencies")
            .retain(|dependency| *dependency != old);
    });
    invalid(
        &dangling,
        "binder names no node",
        "/body/arguments/0/binder",
    );
}

#[trace("TC-492", "FR-038-AC-595")]
#[test]
fn tc_492_a_read_of_another_same_named_binder_is_refused() {
    let mut case = ic_14_let();
    let wire = case.package.clone();
    let nodes = wire["semantic_graph"]["nodes"].as_array().expect("nodes");
    let call = nodes
        .iter()
        .position(|node| node["body"]["operation"]["identity"] == "filament.op.function.call")
        .expect("the call");
    let binder = nodes
        .iter()
        .find(|node| node["node_id"] == nodes[case.node]["body"]["arguments"][0]["binder"])
        .expect("the binder")
        .clone();
    // A second `y` at the binder's level. (Another type at the call's
    // `Integer` parameter would meet the call's own refusal first.)
    let mut other = binder.clone();
    other["node_id"] = node_id("value/parameter/y-other");
    case.package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(other);
    let call_id = nodes[call]["node_id"].clone();
    let old_binder = binder["node_id"]["path"][0]
        .as_str()
        .expect("key")
        .to_owned();
    let graph = case.package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let call_node = graph
        .iter_mut()
        .find(|node| node["node_id"] == call_id)
        .expect("the call");
    call_node["body"]["arguments"][1]["target"] = node_id("value/parameter/y-other");
    let dependencies = call_node["dependencies"]
        .as_array_mut()
        .expect("dependencies");
    dependencies.retain(|dependency| *dependency != node_id(&old_binder));
    dependencies.push(node_id("value/parameter/y-other"));
    let CheckedPackageV2ReadResult::Refused(refusal) = case.read() else {
        panic!("a read of another same-named binder must be refused");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::InvalidValue)),
        "{refusal:?}"
    );
    let expected = format!("/semantic_graph/nodes/{call}/body/arguments/1");
    assert_eq!(
        refusal.loci[0].path,
        Some(pointer(&expected)),
        "{refusal:?}"
    );
}

#[trace("TC-492", "FR-038-AC-595")]
#[test]
fn tc_492_a_binder_on_another_binding_is_refused() {
    let mut case = ic_14_let();
    let node = case.node;
    let binder =
        case.package["semantic_graph"]["nodes"][node]["body"]["arguments"][0]["binder"].clone();
    // The call's callee function node: its `parameters` binding.
    let nodes = case.package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let (position, function) = nodes
        .iter_mut()
        .enumerate()
        .find(|(_, node)| node["node_tag"] == "function")
        .expect("g");
    function["body"]["members"][0]["binder"] = binder;
    case.node = position;
    case.refuses(
        "binder on a function body binding",
        Code::InvalidPackage,
        Cause::InvalidValue,
        "/body/members/0/binder",
    );
}

/// A let returns a container with one integer parameter read. The counterfeit
/// parameter deliberately has the same name, level and type as the binder.
fn let_container_case(form: &str, wrong: bool) -> Case {
    let mut scene = ledger_scene();
    let integer = scene.ty(&Ty::Integer);
    let x = scene.param("x", &Ty::Integer);
    let y = scene.param("y", &Ty::Integer);
    let mut other = scene
        .nodes
        .iter()
        .find(|node| node["node_id"] == node_id(&y))
        .expect("y")
        .clone();
    let other_key = "value/parameter/y-other";
    other["node_id"] = node_id(other_key);
    scene.add(other);
    let read = if wrong { other_key } else { &y };
    let (ty, body) = match form {
        "tuple_value" => (
            scene.ty(&Ty::Tuple(vec![Ty::Integer])),
            aggregate(vec![reference(read)]),
        ),
        "record_value" => (
            scene.ty(&Ty::Rec("Money")),
            aggregate(vec![binding("amount", reference(read))]),
        ),
        "collection_value" => (
            scene.ty(&Ty::Sequence(Box::new(Ty::Integer), 0, 3)),
            aggregate(vec![reference(read)]),
        ),
        "union_value" => {
            let ty = scene.union("BinderContainer", &[("Some", vec![integer.clone()])]);
            (
                ty,
                aggregate(vec![binding("Some", aggregate(vec![reference(read)]))]),
            )
        }
        "option_value" => (
            scene.ty(&Ty::Option(Box::new(Ty::Integer))),
            aggregate(vec![binding("some", reference(read))]),
        ),
        _ => panic!("test container form"),
    };
    let container_key = format!("application/value/{form}/container");
    scene.add(wire_node(
        &container_key,
        "value",
        form,
        &ty,
        &[read],
        "expression",
        body,
    ));
    let key = scene.application(App {
        tag: "expression",
        form: "let",
        operator: "let",
        identity: "filament.op.control.let",
        member: Value::Null,
        semantic: &ty,
        result: &ty,
        arguments: vec![
            let_binding("y", &y, reference(&x)),
            reference(&container_key),
        ],
    });
    let mut case = scene.finish().at(&key);
    // The refusal belongs to the read in the container, not the let.
    if wrong {
        case = case.at(&container_key);
    }
    case
}

#[trace("TC-492", "FR-038-AC-595")]
#[test]
fn tc_492_binder_reads_follow_value_containers() {
    for (form, tail) in [
        ("tuple_value", "/body/members/0"),
        ("record_value", "/body/members/0/value"),
        ("collection_value", "/body/members/0"),
        ("union_value", "/body/members/0/value/members/0"),
        ("option_value", "/body/members/0/value"),
    ] {
        let_container_case(form, false).admits(&format!("linked binder in {form}"));
        let_container_case(form, true).refuses(
            form,
            Code::InvalidPackage,
            Cause::InvalidValue,
            tail,
        );
    }
}

#[trace("TC-492", "FR-038-AC-595")]
#[test]
fn tc_492_binder_scope_stops_at_functions_and_excludes_the_bound_value() {
    let mut scene = ledger_scene();
    let integer = scene.ty(&Ty::Integer);
    let y = scene.param("y", &Ty::Integer);
    let mut other = scene
        .nodes
        .iter()
        .find(|node| node["node_id"] == node_id(&y))
        .expect("y")
        .clone();
    let other_key = "value/parameter/y-other";
    other["node_id"] = node_id(other_key);
    scene.add(other);
    // g's own parameter also spells y at level zero, but is not captured by
    // the let surrounding a call to g.
    let g = scene.function(
        "binder_scope_g",
        &integer,
        vec![
            binding("parameters", aggregate(vec![reference(other_key)])),
            binding("body", reference(other_key)),
        ],
    );
    let call = scene.application(App {
        tag: "expression",
        form: "call",
        operator: "call",
        identity: "filament.op.function.call",
        member: Value::Null,
        semantic: &integer,
        result: &integer,
        arguments: vec![reference(&g), reference(&y)],
    });
    let key = scene.application(App {
        tag: "expression",
        form: "let",
        operator: "let",
        identity: "filament.op.control.let",
        member: Value::Null,
        semantic: &integer,
        result: &integer,
        // The outer y is also legitimate in the bound value, before this
        // let's binder enters scope.
        arguments: vec![let_binding("y", &y, reference(other_key)), reference(&call)],
    });
    scene
        .finish()
        .at(&key)
        .admits("function boundary and bound value scope");
}

#[trace("TC-492", "FR-038-AC-595")]
#[test]
fn tc_492_a_shared_body_is_checked_against_each_linked_binder() {
    let mut case = let_container_case("tuple_value", false);
    let nodes = case.package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let mut second = nodes[case.node].clone();
    second["node_id"] = node_id("application/expression/let/second");
    second["body"]["arguments"][0]["binder"] = node_id("value/parameter/y-other");
    nodes.push(second);
    case.node = nodes
        .iter()
        .position(|node| node["semantic_form"] == "tuple_value")
        .expect("shared tuple");
    // Both lets share precisely the same body node. The first link is valid;
    // memoizing that body must not hide its wrong identity for the second.
    settle(&mut case.package);
    case.refuses(
        "shared body with a different linked binder",
        Code::InvalidPackage,
        Cause::InvalidValue,
        "/body/members/0",
    );
}

fn nested_linked_lets(count: usize) -> Case {
    let mut scene = ledger_scene();
    let integer = scene.ty(&Ty::Integer);
    let mut binders = Vec::with_capacity(count + 1);
    for i in 0..=count {
        binders.push(scene.param(&format!("x{i:06}"), &Ty::Integer));
        scene.nodes.last_mut().expect("new parameter")["body"]["members"][1]["value"]["value"] =
            json!(i.to_string());
    }
    let mut body = binders[count].clone();
    for i in (1..=count).rev() {
        body = scene.application(App {
            tag: "expression",
            form: "let",
            operator: "let",
            identity: "filament.op.control.let",
            member: Value::Null,
            semantic: &integer,
            result: &integer,
            arguments: vec![
                let_binding(&format!("x{i:06}"), &binders[i], reference(&binders[i - 1])),
                reference(&body),
            ],
        });
    }
    scene.finish().at(&body)
}

#[trace("TC-492", "FR-038-AC-595", "FR-038-AC-504")]
#[test]
fn tc_492_one_hundred_thousand_nested_lets_share_the_binder_walk() {
    for count in [10, 100_000] {
        let case = nested_linked_lets(count);
        let mut wire = case.package;
        index_fixture_node_ids(&mut wire);
        let bytes = canonical(&wire);
        let limits = CheckedPackageReadLimits {
            bytes: bytes.len() as u64,
            nodes: 4 * count as u64 + 1000,
            edges: 10 * count as u64 + 1000,
            occurrences: 10 * count as u64 + 1000,
            diagnostics: 1000,
            work: 1000 * count as u64 + 10_000,
        };
        let result = CheckedPackageV2::read(&bytes, limits, &case.evidence);
        assert!(
            matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
            "{count} linked lets: {result:?}"
        );
    }
}

#[trace("TC-492", "FR-038-AC-595")]
#[test]
fn tc_492_linked_let_read_storage_is_accounted_at_exact_and_one_short() {
    use filament_contract_model::{
        initialize_operation_catalog, CheckedPackageV2BudgetedReadResult as Outcome,
        NativeJoinBudget,
    };
    initialize_operation_catalog().expect("operation catalog");
    for (label, case, admitted) in [
        ("linked", let_container_case("tuple_value", false), true),
        ("shared nested body", nested_linked_lets(128), true),
        (
            "wrong linked read",
            let_container_case("tuple_value", true),
            false,
        ),
        (
            "unread incompatible binder",
            let_unread_case(&Ty::Range(0, 1000), &Ty::Range(0, 999)),
            false,
        ),
    ] {
        let mut wire = case.package;
        index_fixture_node_ids(&mut wire);
        let bytes = canonical(&wire);
        let read = |budget: &mut NativeJoinBudget| {
            CheckedPackageV2::read_with_budget(
                &bytes,
                CheckedPackageReadLimits::bounded(),
                &case.evidence,
                budget,
            )
        };
        let mut measure = NativeJoinBudget::unlimited();
        let mut result = None;
        let allocated = allocation_counter::measure(|| {
            result = Some(read(&mut measure));
        })
        .bytes_total;
        let reference = result.expect("read ran");
        assert_eq!(
            matches!(reference, Outcome::Admitted(_)),
            admitted,
            "{label}: {reference:?}"
        );
        assert!(
            matches!(reference, Outcome::Admitted(_) | Outcome::Refused(_)),
            "{label}: {reference:?}"
        );
        let required = measure.read_storage_consumed();
        assert!(
            allocated <= required + 4096,
            "{label}: {allocated} requested, {required} charged"
        );
        let mut exact = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(required);
        let exact_result = read(&mut exact);
        assert_eq!(
            exact_result, reference,
            "{label}: exact storage changes outcome"
        );
        assert_eq!(exact.read_storage_consumed(), required);
        let mut short = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(required - 1);
        let mut denied = None;
        let allocated = allocation_counter::measure(|| {
            denied = Some(read(&mut short));
        })
        .bytes_total;
        assert!(
            matches!(denied, Some(Outcome::BudgetIncomplete(_))),
            "{label}: {denied:?}"
        );
        assert!(short.first_unpaid_read_storage().is_some(), "{label}");
        assert!(
            allocated <= short.read_storage_consumed() + 4096,
            "{label}: short read allocated {allocated} bytes"
        );
    }
}

#[trace(
    "TC-492",
    "FR-038-AC-586",
    "FR-038-AC-587",
    "FR-038-AC-588",
    "FR-038-AC-589"
)]
#[test]
fn tc_492_the_ic_vectors_read_as_fspec_specifies() {
    for (id, case, expect) in ic_vectors() {
        match expect {
            Expect::Admitted => case.admits(id),
            Expect::Mismatch => case.mismatch(id),
            Expect::Ineligible(tail) => case.ineligible(id, tail),
        }
    }
}
