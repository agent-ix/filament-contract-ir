// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Catalogue-wide admission differential for the IR-225 member-result rule.

use crate::support::checked_package::{
    add_family_reference_target, canonical, evidence_for, family_key, index_fixture_node_ids,
    node_id, nominal_fixture_members, nominal_package, refresh_identity, settle,
    structural_fixture_id, v2_all_families,
};
use filament_contract_model::{
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult, CheckedTerm,
    OperationModeKind,
};
use ix_trace_rs::trace;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

fn node(package: &mut Value, key: &str, tag: &str, form: &str, ty: &str, body: Value) {
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push({ let mut node = json!({
            "node_id": node_id(key), "schema_version": "filament.checked-semantic-graph/v2",
            "node_tag": tag, "semantic_form": form, "semantic_type": node_id(ty),
            "dependencies": [], "occurrences": [{"role": "expression", "ordinal": 0}], "body": body
        }); crate::support::checked_package::carry_check_mode(&mut node); node });
}

struct Fixture {
    package: Value,
    unit: String,
    boolean: String,
    integer: String,
    rational: String,
    text: String,
    left: String,
}

fn fixture() -> Fixture {
    let members = nominal_fixture_members();
    let unit = members
        .iter()
        .find(|(p, _)| p["version"] == "filament.unit-node/v1")
        .expect("unit")
        .1
        .clone();
    let mut package = nominal_package(&members);
    let empty = json!({"term": "aggregate", "members": []});
    let scalar = |form: &str| format!("scalar_type/{form}");
    let boolean = scalar("boolean");
    let integer = scalar("integer");
    let rational = scalar("rational");
    let text = scalar("text");
    for (form, key) in [
        ("boolean", &boolean),
        ("integer", &integer),
        ("rational", &rational),
        ("text", &text),
    ] {
        node(&mut package, key, "scalar_type", form, key, empty.clone());
    }
    let body = json!({"term": "aggregate", "members": [
        {"term": "binding", "name": "name", "value":
            {"term": "literal", "type": node_id(&text), "value_kind": "text", "value": "left"}},
        {"term": "binding", "name": "level", "value":
            {"term": "literal", "type": node_id(&integer), "value_kind": "integer", "value": "0"}}
    ]});
    let left = "value/parameter/left".to_owned();
    node(&mut package, &left, "value", "parameter", &unit, body);
    settle(&mut package);
    Fixture {
        package,
        unit,
        boolean,
        integer,
        rational,
        text,
        left,
    }
}

fn reference(key: &str) -> Value {
    json!({"term": "reference", "target": node_id(key)})
}
fn integer_literal(key: &str) -> Value {
    json!({"term": "literal", "type": node_id(key), "value_kind": "integer", "value": "1"})
}
fn binder(value: Value) -> Value {
    json!({"term": "binding", "name": "item", "value": value})
}
fn composite(package: &mut Value, form: &str, member: &str) -> String {
    let body = json!({"term": "aggregate", "members": [reference(member)]});
    let key = format!("composite/{form}/{member}");
    node(package, &key, "composite_type", form, &key, body);
    key
}
fn population(f: &mut Fixture, reference_type: &str) -> String {
    let set_body = json!({"term": "aggregate", "members": [reference(reference_type)]});
    let set = format!("composite/set/{reference_type}");
    if !f.package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .any(|node| node["node_id"] == node_id(&set))
    {
        node(
            &mut f.package,
            &set,
            "composite_type",
            "set",
            &set,
            set_body,
        );
    }
    let body = json!({"term": "aggregate", "members": []});
    let ty = format!("bounded/model_population/{reference_type}");
    node(
        &mut f.package,
        &ty,
        "bounded_domain",
        "model_population",
        &set,
        body,
    );
    parameter(f, &ty, "population_a")
}
fn expression_reference(package: &mut Value, target: &str, ty: &str) -> String {
    let body = reference(target);
    let key = format!("expression/reference/{target}");
    node(package, &key, "expression", "reference", ty, body);
    key
}
fn rational_parameter(f: &mut Fixture) -> String {
    let body = json!({"term": "aggregate", "members": [
        {"term": "binding", "name": "name", "value": {"term": "literal",
            "type": node_id(&f.text), "value_kind": "text", "value": "ratio"}},
        {"term": "binding", "name": "level", "value": integer_literal(&f.integer)}]});
    let key = "value/parameter/ratio".to_owned();
    node(
        &mut f.package,
        &key,
        "value",
        "parameter",
        &f.rational,
        body,
    );
    key
}
fn scalar(f: &mut Fixture, form: &str) -> String {
    let body = json!({"term": "aggregate", "members": []});
    let key = format!("scalar_type/{form}");
    node(&mut f.package, &key, "scalar_type", form, &key, body);
    key
}
fn parameter(f: &mut Fixture, ty: &str, name: &str) -> String {
    parameter_at_level(f, ty, name, 0)
}
fn parameter_at_level(f: &mut Fixture, ty: &str, name: &str, level: u64) -> String {
    let body = json!({"term":"aggregate", "members":[
        {"term":"binding", "name":"name", "value":
            {"term":"literal", "type":node_id(&f.text), "value_kind":"text", "value":name}},
        {"term":"binding", "name":"level", "value":integer_literal(&f.integer)}
    ]});
    let mut body = body;
    body["members"][1]["value"]["value"] = json!(level.to_string());
    let key = if level == 0 {
        format!("value/parameter/{name}")
    } else {
        format!("value/parameter/{name}/{level}/{ty}")
    };
    node(&mut f.package, &key, "value", "parameter", ty, body);
    key
}
fn ieee_law(f: &mut Fixture, position: usize) {
    let definition = json!({"authority":"agent-ix",
        "identity":"filament.value.ieee754-2019-default/v1"});
    f.package["lock"]["definition_selections"]
        .as_array_mut()
        .expect("selections")
        .push(definition.clone());
    f.package["semantic_graph"]["nodes"][position]["body"]["operation"]["laws"] =
        json!([{"role":"ieee_profile", "definition":definition}]);
    settle(&mut f.package);
}

fn selected_catalog_law(f: &mut Fixture, position: usize, role: &str) {
    let catalog: Value = serde_json::from_str(
        filament_verification_contracts::operation_catalog::CHECKED_OPERATION_CATALOG_V1,
    )
    .expect("reader catalog");
    let definition = catalog["law_roles"][role][0].clone();
    f.package["lock"]["definition_selections"]
        .as_array_mut()
        .expect("selections")
        .push(definition.clone());
    f.package["semantic_graph"]["nodes"][position]["body"]["operation"]["laws"]
        .as_array_mut()
        .expect("laws")
        .push(json!({"role": role, "definition": definition}));
    settle(&mut f.package);
}

fn text_or_string_conversion(identity: &str) -> Option<Value> {
    let (source, destination) = identity
        .strip_prefix("filament.op.text.to_")
        .map(|to| ("text", to))
        .or_else(|| {
            identity
                .strip_prefix("filament.op.string.to_")
                .map(|to| ("string", to))
        })?;
    if !matches!(
        destination,
        "integer" | "decimal" | "rational" | "float32" | "float64" | "quantity"
    ) {
        return None;
    }
    let mut f = fixture();
    let source_type = if source == "text" {
        f.text.clone()
    } else {
        scalar(&mut f, "string")
    };
    let destination_type = match destination {
        "integer" => f.integer.clone(),
        "rational" => f.rational.clone(),
        "quantity" => f.unit.clone(),
        other => scalar(&mut f, other),
    };
    let rounding = matches!(destination, "decimal" | "float32" | "float64" | "quantity");
    let mode = if rounding {
        json!({"kind": "rounding", "value": "exact"})
    } else {
        Value::Null
    };
    let at = application(
        &mut f,
        "convert",
        "conversion",
        identity,
        mode,
        &destination_type,
        &destination_type,
        vec![json!({"term": "literal", "type": node_id(&source_type),
            "value_kind": source, "value": "1"})],
    );
    if source == "text" {
        selected_catalog_law(&mut f, at, "text_profile");
    }
    if matches!(destination, "float32" | "float64") {
        selected_catalog_law(&mut f, at, "ieee_profile");
    }
    Some(f.package)
}

fn add_sequence_owner(package: &mut Value) {
    let function_id = node_id("function/sequence-owner");
    let application_id = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["body"]["operation"]["identity"] == "filament.op.collection.seq")
        .expect("sequence application")["node_id"]
        .clone();
    let sequence_result = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_id"] == application_id)
        .expect("sequence application")["body"]["result_type"]
        .clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(json!({
            "node_id":function_id, "schema_version":"filament.checked-semantic-graph/v2",
            "node_tag":"function", "semantic_form":"pure_function",
            "check_mode":"linked", "semantic_type":sequence_result,
            "dependencies":[application_id.clone()],
            "occurrences":[{"role":"expression", "ordinal":0}],
            "body":{"term":"aggregate", "members":[
                {"term":"binding", "name":"parameters", "value":{"term":"aggregate", "members":[]}},
                {"term":"binding", "name":"body", "value":{
                    "term":"reference", "target":application_id}}
            ]}
        }));
    settle(package);
}
fn numeric_literal(ty: &str, form: &str) -> Value {
    match form {
        "float32" => json!({"term":"literal", "type":node_id(ty),
            "value_kind":"float32_bits", "value":"3f800000"}),
        "float64" => json!({"term":"literal", "type":node_id(ty),
            "value_kind":"float64_bits", "value":"3ff0000000000000"}),
        _ => unreachable!(),
    }
}
fn application(
    f: &mut Fixture,
    operator: &str,
    form: &str,
    identity: &str,
    mode: Value,
    declaration: &str,
    result: &str,
    arguments: Vec<Value>,
) -> usize {
    let member = json!({"kind": "type_argument", "declaration": node_id(declaration)});
    application_with_member(f, operator, form, identity, mode, member, result, arguments)
}
fn application_with_member(
    f: &mut Fixture,
    operator: &str,
    form: &str,
    identity: &str,
    mode: Value,
    member: Value,
    result: &str,
    arguments: Vec<Value>,
) -> usize {
    let operation = json!({"identity": identity, "laws": [], "mode": mode,
        "member": member, "leaves": []});
    let body = json!({"term": "application", "operator": operator,
        "operation": operation, "result_type": node_id(result), "arguments": arguments});
    let key = format!("expression/application/{identity}");
    let position = f.package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    node(&mut f.package, &key, "expression", form, result, body);
    settle(&mut f.package);
    position
}
fn read(package: &Value) -> CheckedPackageV2ReadResult {
    let mut wire = package.clone();
    index_fixture_node_ids(&mut wire);
    crate::read_and_account(
        &canonical(&wire),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&wire),
    )
}
fn read_with_evidence(
    package: &Value,
    evidence: &CheckedPackageEvidence,
) -> CheckedPackageV2ReadResult {
    let mut wire = package.clone();
    index_fixture_node_ids(&mut wire);
    crate::read_and_account(
        &canonical(&wire),
        CheckedPackageReadLimits::bounded(),
        evidence,
    )
}

fn assert_catalogue_application_round_trip(package: &Value) {
    let CheckedPackageV2ReadResult::Admitted(admitted) = read(package) else {
        panic!("catalogue application fixture refused: {:?}", read(package));
    };
    let bytes = crate::checked_package_v2_refinement::writer_bytes(package);
    let mut wire = package.clone();
    index_fixture_node_ids(&mut wire);
    let CheckedPackageV2ReadResult::Admitted(reread) = crate::read_and_account(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&wire),
    ) else {
        panic!("catalogue application writer output refused");
    };
    assert_eq!(admitted.graph(), reread.graph());
}

pub(super) fn simple_fixture(entry: &Value) -> Option<Value> {
    let identity = entry["identity"].as_str()?;
    let family = if identity.starts_with("filament.op.boolean.") {
        "boolean"
    } else if identity.starts_with("filament.op.integer.") {
        "integer"
    } else if identity.starts_with("filament.op.rational.")
        && identity != "filament.op.rational.narrow"
    {
        "rational"
    } else {
        return None;
    };
    if !entry["laws"].as_array()?.is_empty() {
        return None;
    }
    let operator = entry["operator"].as_str()?;
    if operator != "unary" && operator != "binary" {
        return None;
    }
    let mut f = fixture();
    let ty = match family {
        "boolean" => &f.boolean,
        "integer" => &f.integer,
        "rational" => &f.rational,
        _ => unreachable!(),
    }
    .clone();
    let result = if entry["result"] == "boolean" {
        f.boolean.clone()
    } else {
        ty.clone()
    };
    let argument = match family {
        "boolean" => json!({"term":"literal", "type":node_id(&ty),
            "value_kind":"boolean", "value":true}),
        "integer" => integer_literal(&ty),
        "rational" => {
            let parameter = rational_parameter(&mut f);
            reference(&parameter)
        }
        _ => unreachable!(),
    };
    let args = if operator == "unary" {
        vec![argument]
    } else {
        vec![argument.clone(), argument]
    };
    let body = json!({"term":"application", "operator":operator,
        "operation":{"identity":identity, "laws":[], "mode":null,
            "member":null, "leaves":[]},
        "result_type":node_id(&result), "arguments":args});
    let key = format!("expression/application/{identity}");
    node(&mut f.package, &key, "expression", operator, &result, body);
    settle(&mut f.package);
    Some(f.package)
}

fn unavailable_reason(entry: &Value) -> Option<&'static str> {
    if entry["identity"] == "filament.op.option.slots" {
        return Some(
            "requires a selected timed-model root and a resolved clock_place whose terminal slot bounds are joined to a model declaration",
        );
    }
    if entry["identity"] == "filament.op.clock_reference.ref" {
        return Some(
            "requires a selected timed-model root and a resolved clock_place, joined to a reference owner whose type conforms to the place model; the place_identity_terms and place_has_id productions are read by the operations tests of FR-829",
        );
    }
    let operator = entry["operator"].as_str()?;
    let laws = entry["laws"].as_array()?;
    let operands = entry["operands"].as_array()?;
    if matches!(
        operator,
        "temporal" | "temporal_formula" | "temporal_fairness"
    ) {
        return Some("requires a reached temporal formula or clause context");
    }
    if matches!(
        operator,
        "state_clause" | "state_transition" | "pre" | "claim" | "protocol_control"
    ) {
        return Some("requires a state, claim or protocol context");
    }
    if matches!(operator, "case" | "conditional" | "let") {
        return Some("requires a control-flow binder or branch context");
    }
    if matches!(operator, "deref" | "reaches") {
        return Some("requires selected model declarations and reference targets");
    }
    if matches!(operator, "collection" | "quantify") {
        return Some("requires a typed collection and member or binder context");
    }
    for (role, reason) in [
        (
            "ieee_profile",
            "requires a selected IEEE profile and typed float operands",
        ),
        (
            "text_profile",
            "requires a selected text profile and typed text operands",
        ),
        (
            "integer_division",
            "requires a selected integer division law",
        ),
    ] {
        if laws.iter().any(|law| law == role) {
            return Some(reason);
        }
    }
    for operand in operands {
        let reason = match operand.as_str()? {
            "decimal" => "requires a typed decimal operand",
            "quantity" => "requires a typed quantity operand",
            "enum_kind" | "ordered_enum" => "requires a declared enum operand",
            "structural_kind" => "requires a structural operand",
            "string" | "uuid" | "timestamp" => "requires a typed native scalar operand",
            "bytes" => "requires a typed byte sequence operand",
            "json_object" => "requires a typed JSON object operand",
            "reference" => "requires a typed reference operand",
            "option" => "requires a typed option operand",
            "function" => "requires a typed function operand",
            _ => continue,
        };
        return Some(reason);
    }
    None
}

// A fresh package is built for each identity. These are the entries for which
// the existing nominal/structural fixture helpers produce a valid source.
pub(crate) fn admitted_fixture(identity: &str) -> Option<Value> {
    if let Some(package) = text_or_string_conversion(identity) {
        return Some(package);
    }
    if identity == "filament.op.numeric.narrow" {
        return Some(crate::admitted_result_context::fixture());
    }
    if identity == "filament.op.collection.sum.carried_quantity" {
        return Some(crate::checked_package_v2_operand_families::carried_sum_control().0);
    }
    if identity == "filament.op.collection.sum.quantity" {
        return Some(crate::checked_package_v2_operand_families::quantity_sum_control().0);
    }
    if matches!(
        identity,
        "filament.op.ieee.to_float32"
            | "filament.op.ieee.to_float64"
            | "filament.op.ieee.to_rational"
    ) {
        let (source, destination) = match identity {
            "filament.op.ieee.to_float32" => ("float64", "float32"),
            "filament.op.ieee.to_float64" => ("float32", "float64"),
            "filament.op.ieee.to_rational" => ("float32", "rational"),
            _ => unreachable!(),
        };
        return Some(
            crate::checked_package_v2_ieee_admission::with_ieee_conversion_source(
                identity,
                source,
                destination,
                destination,
            )
            .0,
        );
    }
    let mut f = fixture();
    let i = f.integer.clone();
    let r = f.rational.clone();
    let u = f.unit.clone();
    let b = f.boolean.clone();
    let mut set_i = || composite(&mut f.package, "set", &i);
    match identity {
        "filament.op.reference.eq" => {
            let target = add_family_reference_target(&mut f.package);
            let reference_type = composite(&mut f.package, "reference", &target);
            let operand = parameter(&mut f, &reference_type, "same_reference");
            application_with_member(
                &mut f,
                "binary",
                "binary",
                identity,
                Value::Null,
                Value::Null,
                &b,
                vec![reference(&operand), reference(&operand)],
            );
        }
        "filament.op.clock_reference.eq" | "filament.op.clock_reference.ne" => {
            let clock_reference = scalar(&mut f, "clock_reference");
            let operand = parameter(&mut f, &clock_reference, "same_clock_reference");
            application_with_member(
                &mut f,
                "binary",
                "binary",
                identity,
                Value::Null,
                Value::Null,
                &b,
                vec![reference(&operand), reference(&operand)],
            );
        }
        "filament.op.ieee.from_exact" | "filament.op.numeric.convert_rounding" => {
            let family = if identity == "filament.op.ieee.from_exact" {
                "float32"
            } else {
                "decimal"
            };
            let destination = scalar(&mut f, family);
            let at = application(
                &mut f,
                "convert",
                "conversion",
                identity,
                json!({"kind":"rounding", "value":"exact"}),
                &destination,
                &destination,
                vec![integer_literal(&i)],
            );
            if identity == "filament.op.ieee.from_exact" {
                ieee_law(&mut f, at);
            }
        }
        "filament.op.collection.sum.decimal"
        | "filament.op.collection.sum.float32"
        | "filament.op.collection.sum.float64" => {
            let family = identity.rsplit('.').next().expect("family");
            let destination = scalar(&mut f, family);
            let set = composite(&mut f.package, "set", &destination);
            let bound = if family == "decimal" {
                reference(&parameter(&mut f, &destination, "decimal"))
            } else {
                numeric_literal(&destination, family)
            };
            let at = application(
                &mut f,
                "collection",
                "collection",
                identity,
                json!({"kind":"rounding", "value":"exact"}),
                &destination,
                &destination,
                vec![reference(&set), binder(bound)],
            );
            if family.starts_with("float") {
                ieee_law(&mut f, at);
            }
        }
        "filament.op.collection.fold" => {
            let set = set_i();
            application(
                &mut f,
                "collection",
                "collection",
                identity,
                Value::Null,
                &i,
                &i,
                vec![
                    reference(&set),
                    binder(integer_literal(&i)),
                    integer_literal(&i),
                ],
            );
        }
        "filament.op.collection.only" => {
            let set = set_i();
            application(
                &mut f,
                "collection",
                "collection",
                identity,
                Value::Null,
                &i,
                &i,
                vec![reference(&set), binder(integer_literal(&i))],
            );
        }
        "filament.op.collection.seq" => {
            let range = "bounded/integer_range/sequence-control".to_owned();
            let bound = |name: &str, value: &str| {
                json!({"term":"binding", "name":name, "value": {
                    "term":"literal", "type":node_id(&i),
                    "value_kind":"integer", "value":value
                }})
            };
            node(
                &mut f.package,
                &range,
                "bounded_domain",
                "integer_range",
                &i,
                json!({"term":"aggregate", "members":[bound("min", "0"), bound("max", "0")]}),
            );
            let sequence = composite(&mut f.package, "sequence", &range);
            let result = "bounded/collection_bounds/sequence-control";
            node(
                &mut f.package,
                result,
                "bounded_domain",
                "collection_bounds",
                &sequence,
                json!({"term":"aggregate", "members":[bound("min", "0"), bound("max", "1")]}),
            );
            let index = parameter(&mut f, &range, "sequence_index");
            let lower = "value/literal/sequence-lower".to_owned();
            let upper = "value/literal/sequence-upper".to_owned();
            node(
                &mut f.package,
                &lower,
                "value",
                "literal",
                &range,
                json!({"term":"literal", "type":node_id(&range),
                    "value_kind":"integer", "value":"0"}),
            );
            node(
                &mut f.package,
                &upper,
                "value",
                "literal",
                &range,
                json!({"term":"literal", "type":node_id(&range),
                    "value_kind":"integer", "value":"0"}),
            );
            let member = json!({"kind":"type_argument", "declaration":node_id(&range)});
            application_with_member(
                &mut f,
                "collection",
                "collection",
                identity,
                Value::Null,
                member,
                result,
                vec![
                    reference(&lower),
                    reference(&upper),
                    json!({"term":"binding", "name":"sequence_index",
                        "value":reference(&index)}),
                ],
            );
        }
        "filament.op.refinement.history" => {
            let declaration = add_family_reference_target(&mut f.package);
            let reference_type = composite(&mut f.package, "reference", &declaration);
            let receiver = parameter(&mut f, &reference_type, "history_receiver");
            let result = f.text.clone();
            application_with_member(
                &mut f,
                "query",
                "query",
                identity,
                Value::Null,
                json!({"kind":"history", "declaration":node_id(&declaration), "name":"past"}),
                &result,
                vec![reference(&receiver)],
            );
        }
        "filament.op.collection.reduce" | "filament.op.collection.sum.integer" => {
            let set = set_i();
            application(
                &mut f,
                "collection",
                "collection",
                identity,
                Value::Null,
                &i,
                &i,
                vec![reference(&set), binder(integer_literal(&i))],
            );
        }
        "filament.op.collection.count" => {
            let set = set_i();
            application(
                &mut f,
                "collection",
                "collection",
                identity,
                Value::Null,
                &i,
                &i,
                vec![
                    reference(&set),
                    binder(json!({"term":"literal", "type":node_id(&b),
                    "value_kind":"boolean", "value":true})),
                ],
            );
        }
        "filament.op.collection.sum.rational" => {
            let set = composite(&mut f.package, "set", &r);
            let parameter = rational_parameter(&mut f);
            application(
                &mut f,
                "collection",
                "collection",
                identity,
                Value::Null,
                &r,
                &r,
                vec![reference(&set), binder(reference(&parameter))],
            );
        }
        "filament.op.collection.size" => {
            let set = set_i();
            application(
                &mut f,
                "collection",
                "collection",
                identity,
                Value::Null,
                &i,
                &i,
                vec![reference(&set)],
            );
        }
        "filament.op.collection.convert" => {
            let set = set_i();
            let sequence = composite(&mut f.package, "sequence", &i);
            application(
                &mut f,
                "convert",
                "conversion",
                identity,
                Value::Null,
                &sequence,
                &sequence,
                vec![reference(&set)],
            );
        }
        "filament.op.numeric.convert" => {
            application(
                &mut f,
                "convert",
                "conversion",
                identity,
                Value::Null,
                &r,
                &r,
                vec![integer_literal(&i)],
            );
        }
        "filament.op.quantity.convert" => {
            let left = f.left.clone();
            application(
                &mut f,
                "convert",
                "conversion",
                identity,
                json!({"kind":"rounding", "value":"exact"}),
                &u,
                &u,
                vec![reference(&left)],
            );
        }
        "filament.op.rational.narrow" => {
            let parameter = rational_parameter(&mut f);
            application(
                &mut f,
                "convert",
                "conversion",
                identity,
                Value::Null,
                &r,
                &r,
                vec![reference(&parameter)],
            );
        }
        "filament.op.reference.upcast" => {
            let target = add_family_reference_target(&mut f.package);
            let reference_type = composite(&mut f.package, "reference", &target);
            let expression = expression_reference(&mut f.package, &f.left.clone(), &reference_type);
            application(
                &mut f,
                "convert",
                "conversion",
                identity,
                Value::Null,
                &reference_type,
                &reference_type,
                vec![reference(&expression)],
            );
        }
        "filament.op.model.all_instances" => {
            let target = add_family_reference_target(&mut f.package);
            let reference_type = composite(&mut f.package, "reference", &target);
            let set = composite(&mut f.package, "set", &reference_type);
            let pop = population(&mut f, &reference_type);
            application(
                &mut f,
                "query",
                "query",
                identity,
                Value::Null,
                &u,
                &set,
                vec![reference(&pop)],
            );
        }
        "filament.op.model.lookup" => {
            let target = add_family_reference_target(&mut f.package);
            let reference_type = composite(&mut f.package, "reference", &target);
            let option = composite(&mut f.package, "option", &u);
            let pop = population(&mut f, &reference_type);
            let expression = expression_reference(&mut f.package, &f.left.clone(), &reference_type);
            application(
                &mut f,
                "query",
                "query",
                identity,
                json!({"kind":"absence", "value":"option"}),
                &u,
                &option,
                vec![reference(&pop), reference(&expression)],
            );
        }
        "filament.op.tuple.project" => {
            let tuple = composite(&mut f.package, "tuple", &i);
            application_with_member(
                &mut f,
                "query",
                "query",
                identity,
                Value::Null,
                json!({"kind":"position", "declaration":node_id(&tuple), "position":0}),
                &i,
                vec![reference(&tuple)],
            );
        }
        "filament.op.collection.index" => {
            let set = set_i();
            application_with_member(
                &mut f,
                "query",
                "query",
                identity,
                Value::Null,
                json!({"kind":"element", "declaration":node_id(&set)}),
                &i,
                vec![
                    reference(&set),
                    json!({"term":"literal", "type":node_id(&i),
                    "value_kind":"integer", "value":"0"}),
                ],
            );
        }
        _ => return None,
    }
    if identity == "filament.op.collection.seq" {
        add_sequence_owner(&mut f.package);
    }
    Some(f.package)
}

pub(crate) fn admitted_catalogue_control(identity: &str) -> CheckedPackageV2 {
    let wire = admitted_fixture(identity).expect("constructible catalog control");
    match read(&wire) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("{identity}: catalog control refused: {other:?}"),
    }
}

#[trace(
    "TC-048",
    "FR-038-AC-331",
    "FR-038-AC-332",
    "FR-038-AC-333",
    "FR-038-AC-334",
    "FR-038-AC-341"
)]
#[test]
fn tc_048_public_operation_views_follow_admitted_values() {
    let (wire, evidence) = crate::checked_package_v2_model_fields::record_project_control();
    let admitted = match read_with_evidence(&wire, &evidence) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("field fixture refused: {other:?}"),
    };
    let operation = admitted
        .graph()
        .nodes()
        .find_map(|node| match &node.body {
            CheckedTerm::Application { operation, .. }
                if operation.identity() == Some("filament.op.record.project") =>
            {
                Some(operation)
            }
            _ => None,
        })
        .expect("field projection application");
    let member = &wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["body"]["operation"]["identity"] == "filament.op.record.project")
        .expect("field projection source")["body"]["operation"]["member"];
    let declaration = serde_json::from_value(member["declaration"].clone()).expect("node id");
    assert_eq!(
        operation.projection_field().expect("admitted views"),
        Some((&declaration, "string"))
    );
    assert_eq!(
        operation.member_declaration().expect("admitted views"),
        Some(&declaration)
    );
    assert_eq!(operation.mode().expect("admitted views"), None);
    assert!(operation
        .law_definitions()
        .expect("admitted views")
        .is_empty());

    for (identity, kind, value, laws) in [
        (
            "filament.op.numeric.convert_rounding",
            OperationModeKind::Rounding,
            "exact",
            0,
        ),
        (
            "filament.op.model.lookup",
            OperationModeKind::Absence,
            "option",
            0,
        ),
        (
            "filament.op.ieee.from_exact",
            OperationModeKind::Rounding,
            "exact",
            1,
        ),
    ] {
        let wire = admitted_fixture(identity).expect("admitted fixture");
        let admitted = match read(&wire) {
            CheckedPackageV2ReadResult::Admitted(package) => package,
            other => panic!("{identity} fixture refused: {other:?}"),
        };
        let operation = admitted
            .graph()
            .nodes()
            .find_map(|node| match &node.body {
                CheckedTerm::Application { operation, .. }
                    if operation.identity() == Some(identity) =>
                {
                    Some(operation)
                }
                _ => None,
            })
            .expect("admitted operation");
        let mode = operation
            .mode()
            .expect("admitted views")
            .expect("admitted mode");
        assert_eq!((mode.kind, mode.value), (kind, value));
        assert_eq!(
            operation.law_definitions().expect("admitted views").len(),
            laws
        );
        if laws == 1 {
            assert_eq!(
                operation.law_definitions().expect("admitted views")[0]
                    .authority
                    .as_ref(),
                "agent-ix"
            );
            assert_eq!(
                operation.law_definitions().expect("admitted views")[0]
                    .identity
                    .as_ref(),
                "filament.value.ieee754-2019-default/v1"
            );
        }
        assert_eq!(operation.projection_field().expect("admitted views"), None);
    }

    let wire = admitted_fixture("filament.op.tuple.project").expect("position fixture");
    let admitted = match read(&wire) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("position fixture refused: {other:?}"),
    };
    let operation = admitted
        .graph()
        .nodes()
        .find_map(|node| match &node.body {
            CheckedTerm::Application { operation, .. }
                if operation.identity() == Some("filament.op.tuple.project") =>
            {
                Some(operation)
            }
            _ => None,
        })
        .expect("position application");
    assert_eq!(operation.projection_field().expect("admitted views"), None);
    assert!(operation
        .member_declaration()
        .expect("admitted views")
        .is_some());
    assert_eq!(operation.mode().expect("admitted views"), None);
    assert!(operation
        .law_definitions()
        .expect("admitted views")
        .is_empty());

    let wire = admitted_fixture("filament.op.quantity.convert").expect("quantity fixture");
    let admitted = match read(&wire) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("quantity fixture refused: {other:?}"),
    };
    let operation = admitted
        .graph()
        .nodes()
        .find_map(|node| match &node.body {
            CheckedTerm::Application { operation, .. }
                if operation.identity() == Some("filament.op.quantity.convert") =>
            {
                Some(operation)
            }
            _ => None,
        })
        .expect("quantity conversion application");
    let declared = &wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["body"]["operation"]["identity"] == "filament.op.quantity.convert")
        .expect("quantity source")["body"]["operation"]["member"]["declaration"];
    let declared = serde_json::from_value(declared.clone()).expect("quantity declaration");
    assert_eq!(
        operation.member_declaration().expect("admitted views"),
        Some(&declared)
    );
    assert_eq!(operation.projection_field().expect("admitted views"), None);

    let catalog: Value = serde_json::from_str(
        filament_verification_contracts::operation_catalog::CHECKED_OPERATION_CATALOG_V1,
    )
    .expect("catalogue JSON");
    let entry = catalog["operations"]
        .as_array()
        .expect("operations")
        .iter()
        .find(|entry| entry["identity"] == "filament.op.integer.add")
        .expect("integer add entry");
    let wire = simple_fixture(entry).expect("plain operation fixture");
    let admitted = match read(&wire) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("plain fixture refused: {other:?}"),
    };
    let operation = admitted
        .graph()
        .nodes()
        .find_map(|node| match &node.body {
            CheckedTerm::Application { operation, .. }
                if operation.identity() == Some("filament.op.integer.add") =>
            {
                Some(operation)
            }
            _ => None,
        })
        .expect("plain application");
    assert_eq!(
        operation.member_declaration().expect("admitted views"),
        None
    );
    assert_eq!(operation.projection_field().expect("admitted views"), None);
    assert_eq!(operation.mode().expect("admitted views"), None);
    assert!(operation
        .law_definitions()
        .expect("admitted views")
        .is_empty());
}

#[trace(
    "TC-048",
    "FR-038-AC-331",
    "FR-038-AC-332",
    "FR-038-AC-333",
    "FR-038-AC-334"
)]
#[test]
fn tc_048_malformed_operation_facts_have_no_admitted_view() {
    let (mut field, evidence) = crate::checked_package_v2_model_fields::record_project_control();
    let field_node = field["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .iter_mut()
        .find(|node| node["body"]["operation"]["identity"] == "filament.op.record.project")
        .expect("field projection");
    field_node["body"]["operation"]["member"]["declaration"] = json!(false);
    refresh_identity(&mut field);
    let result = read_with_evidence(&field, &evidence);
    assert!(
        matches!(&result, CheckedPackageV2ReadResult::Refused(_)),
        "{result:?}"
    );

    let (mut unselected, evidence) =
        crate::checked_package_v2_model_fields::record_project_control();
    let field_node = unselected["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .iter_mut()
        .find(|node| node["body"]["operation"]["identity"] == "filament.op.record.project")
        .expect("field projection");
    field_node["body"]["operation"]["member"]["declaration"] = node_id(&"f".repeat(64));
    settle(&mut unselected);
    assert!(matches!(
        read_with_evidence(&unselected, &evidence),
        CheckedPackageV2ReadResult::Refused(_)
    ));

    let mut mode = admitted_fixture("filament.op.numeric.convert_rounding").expect("mode fixture");
    let mode_node = mode["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .iter_mut()
        .find(|node| {
            node["body"]["operation"]["identity"] == "filament.op.numeric.convert_rounding"
        })
        .expect("rounding conversion");
    mode_node["body"]["operation"]["mode"]["kind"] = json!("unknown");
    settle(&mut mode);
    assert!(matches!(
        read(&mode),
        CheckedPackageV2ReadResult::Refused(_)
    ));

    let mut law = admitted_fixture("filament.op.ieee.from_exact").expect("law fixture");
    law["lock"]["definition_selections"] = json!([]);
    settle(&mut law);
    assert!(matches!(read(&law), CheckedPackageV2ReadResult::Refused(_)));
}

#[trace("TC-048", "FR-038-AC-251", "TC-1862")]
#[test]
fn tc_048_every_catalogue_entry_retains_its_constructible_admission() {
    let catalog: Value = serde_json::from_str(
        filament_verification_contracts::operation_catalog::CHECKED_OPERATION_CATALOG_V1,
    )
    .expect("reader's catalogue is JSON");
    let entries = catalog["operations"].as_array().expect("operations");
    let mut admitted = BTreeSet::new();
    let mut unavailable = BTreeMap::new();
    assert!(
        unavailable_reason(&json!({
            "operator": "binary", "operands": ["integer", "integer"], "laws": []
        }))
        .is_none(),
        "an arbitrary new operation needs an admitted control or a specific fixture gap"
    );
    for entry in entries {
        let identity = entry["identity"].as_str().expect("identity");
        let external = match identity {
            "filament.op.record.project" => {
                Some(crate::checked_package_v2_model_fields::record_project_control())
            }
            "filament.op.model.navigate" | "filament.op.model.dispatch_call" => {
                Some(crate::checked_package_v2_model_members::model_member_control(identity))
            }
            _ => None,
        };
        if let Some((package, evidence)) = external {
            let result = read_with_evidence(&package, &evidence);
            assert!(
                matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
                "{identity}: {result:?}"
            );
            admitted.insert(identity.to_owned());
        } else if identity == "filament.op.refinement.history" {
            let (package, evidence) =
                crate::checked_package_v2_refinement::orphan_history_control();
            let position = package["semantic_graph"]["nodes"]
                .as_array()
                .expect("nodes")
                .iter()
                .position(|node| node["body"]["operation"]["identity"] == identity)
                .expect("history application");
            let CheckedPackageV2ReadResult::Refused(refusal) =
                read_with_evidence(&package, &evidence)
            else {
                panic!("an unreachable history row must refuse");
            };
            assert_eq!(refusal.code, CheckedPackageRefusalCode::IllTyped);
            assert_eq!(
                refusal.cause,
                Some(CheckedPackageRefusalCause::OperatorIneligible)
            );
            assert_eq!(
                refusal.loci[0].path.as_ref().map(ToString::to_string),
                Some(format!("/semantic_graph/nodes/{position}"))
            );
            unavailable.insert(
                identity.to_owned(),
                "a history application without a reachable, joined refinement history row must refuse"
                    .to_owned(),
            );
        } else if let Some(package) = admitted_fixture(identity).or_else(|| simple_fixture(entry)) {
            let result = read(&package);
            let CheckedPackageV2ReadResult::Admitted(admitted_package) = result else {
                panic!("{identity}: {result:?}");
            };
            if identity.starts_with("filament.op.text.to_")
                || identity.starts_with("filament.op.string.to_")
            {
                let position = package["semantic_graph"]["nodes"]
                    .as_array()
                    .expect("nodes")
                    .iter()
                    .position(|node| node["body"]["operation"]["identity"] == identity)
                    .expect("conversion application");
                let body = serde_json::to_value(
                    &admitted_package
                        .graph()
                        .node(position)
                        .expect("graph position exists")
                        .to_owned_wire()
                        .body,
                )
                .expect("typed conversion body");
                let source = identity.split('.').nth(2).expect("source family");
                assert_eq!(body["operation"]["identity"], identity);
                assert_eq!(body["arguments"][0]["value_kind"], source);
                assert_eq!(body["arguments"][0]["value"], "1");
                assert_eq!(
                    body["operation"]["member"]["declaration"],
                    body["result_type"]
                );
                assert_eq!(
                    serde_json::to_value(
                        &admitted_package
                            .graph()
                            .node(position)
                            .expect("graph position exists")
                            .semantic_type
                    )
                    .expect("typed result"),
                    body["result_type"]
                );
            }
            admitted.insert(identity.to_owned());
        } else {
            assert!(
                entry["result"] != "member" && entry["member"] != "type_argument",
                "member-result or type-argument lookalike {identity} needs an admitted control"
            );
            let reason = unavailable_reason(entry).unwrap_or_else(|| {
                panic!(
                    "{identity} needs an admitted control or a catalog-derived fixture gap: {entry}"
                )
            });
            assert!(!reason.is_empty(), "{identity}: empty fixture-gap reason");
            unavailable.insert(identity.to_owned(), reason.to_owned());
        }
    }
    assert_eq!(
        admitted.len() + unavailable.len(),
        entries.len(),
        "each catalog entry has an admitted control or an explicit unavailable reason"
    );

    // Each selected entry has an admitted witness and a mismatched-result
    // refusal. This catches selection and family regressions across the catalogue.
    for entry in entries
        .iter()
        .filter(|entry| entry["result"] == "member" && entry["member"] == "type_argument")
    {
        let identity = entry["identity"].as_str().expect("identity");
        let mut package = admitted_fixture(identity).expect("admitted control");
        let position = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .position(|node| node["body"]["operation"]["identity"] == identity)
            .expect("catalogued application");
        let other = match identity {
            "filament.op.ieee.to_float32" => "float64",
            "filament.op.ieee.to_float64" | "filament.op.ieee.to_rational" => "float32",
            "filament.op.collection.fold"
            | "filament.op.collection.reduce"
            | "filament.op.collection.count"
            | "filament.op.collection.sum.integer"
            | "filament.op.collection.size" => "rational",
            "filament.op.collection.only" => "rational",
            "filament.op.text.to_integer" | "filament.op.string.to_integer" => "rational",
            _ => "integer",
        };
        let key = if identity == "filament.op.numeric.narrow" {
            family_key("a3a3")
        } else if matches!(
            identity,
            "filament.op.collection.sum.quantity" | "filament.op.collection.sum.carried_quantity"
        ) {
            // The control keys its own scalars; a dangling `scalar_type/integer`
            // would be refused by join stage 4 before the result check.
            "operand/scalar/integer".to_owned()
        } else if identity == "filament.op.reference.upcast" {
            let flow = family_key("1616");
            let selected = v2_all_families();
            let declaration = selected["semantic_graph"]["nodes"]
                .as_array()
                .expect("nodes")
                .iter()
                .find(|node| node["node_id"] == node_id(&flow))
                .expect("selected Flow")
                .clone();
            package["semantic_graph"]["nodes"]
                .as_array_mut()
                .expect("nodes")
                .push(declaration);
            composite(&mut package, "reference", &flow)
        } else {
            format!("scalar_type/{other}")
        };
        package["semantic_graph"]["nodes"][position]["body"]["result_type"] = node_id(&key);
        package["semantic_graph"]["nodes"][position]["semantic_type"] = node_id(&key);
        settle(&mut package);
        assert_result_refusal(&package, position, identity);
    }
    for source in ["float32", "float64"] {
        for (member, result) in [(source, "rational"), ("rational", source), (source, source)] {
            let (package, position) =
                crate::checked_package_v2_ieee_admission::with_ieee_conversion_source(
                    "filament.op.ieee.to_rational",
                    source,
                    member,
                    result,
                );
            assert_result_refusal(&package, position, "filament.op.ieee.to_rational");
        }
    }
}

#[trace("TC-048", "FR-038-AC-251")]
#[test]
fn collection_seq_requires_bounded_inputs_matching_binder_and_sequence() {
    let base = admitted_fixture("filament.op.collection.seq").expect("sequence control");
    assert_catalogue_application_round_trip(&base);
    let function_id = node_id("function/sequence-owner");
    let base_result = read(&base);
    assert!(
        matches!(base_result, CheckedPackageV2ReadResult::Admitted(_)),
        "function-scoped sequence fixture: {base_result:?}"
    );
    let application = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["body"]["operation"]["identity"] == "filament.op.collection.seq")
        .expect("sequence application");
    let function = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == function_id)
        .expect("sequence owner function");

    let text_type = node_id("scalar_type/text");
    let integer_type = node_id("scalar_type/integer");
    let parameter_node = |id: Value, name: &str, level: &str, ty: Value| {
        json!({
            "node_id":id, "schema_version":"filament.checked-semantic-graph/v2",
            "node_tag":"value", "semantic_form":"parameter", "semantic_type":ty,
            "dependencies":[], "occurrences":[{"role":"expression", "ordinal":0}],
            "body":{"term":"aggregate", "members":[
                {"term":"binding", "name":"name", "value":{
                    "term":"literal", "type":text_type, "value_kind":"text", "value":name}},
                {"term":"binding", "name":"level", "value":{
                    "term":"literal", "type":integer_type, "value_kind":"integer", "value":level}}
            ]}
        })
    };
    let mut shadowed = base.clone();
    shadowed["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(parameter_node(
            node_id("value/parameter/sequence-index-shadow"),
            "sequence_index",
            "2",
            integer_type.clone(),
        ));
    settle(&mut shadowed);
    let shadowed_result = read(&shadowed);
    assert!(
        matches!(shadowed_result, CheckedPackageV2ReadResult::Admitted(_)),
        "an unrelated same-name parameter at another level does not shadow the binder: {shadowed_result:?}"
    );

    let mut unused_index = base.clone();
    let original_index = base["semantic_graph"]["nodes"][application]["body"]["arguments"][2]
        ["value"]["target"]
        .clone();
    let range_type = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_id"] == original_index)
        .expect("index parameter")["semantic_type"]
        .clone();
    let outer_x = node_id("value/parameter/outer-x");
    unused_index["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend([
            parameter_node(outer_x.clone(), "x", "0", range_type.clone()),
            parameter_node(
                node_id("value/parameter/sequence-index-level-one"),
                "sequence_index",
                "1",
                range_type,
            ),
            parameter_node(
                node_id("value/parameter/sequence-index-shadow-level-two"),
                "sequence_index",
                "2",
                integer_type.clone(),
            ),
        ]);
    unused_index["semantic_graph"]["nodes"][function]["body"]["members"][0]["value"] =
        json!({"term":"aggregate", "members":[{"term":"reference", "target":outer_x}]});
    let dependencies = unused_index["semantic_graph"]["nodes"][function]["dependencies"]
        .as_array_mut()
        .expect("function dependencies");
    dependencies.push(outer_x.clone());
    dependencies.sort_by_key(structural_fixture_id);
    unused_index["semantic_graph"]["nodes"][application]["body"]["arguments"][2]["value"]
        ["target"] = outer_x;
    settle(&mut unused_index);
    assert!(
        matches!(read(&unused_index), CheckedPackageV2ReadResult::Admitted(_)),
        "an unused i binder is resolved at level one from the outer x scope"
    );

    let mut boolean_element = base.clone();
    let boolean_type = node_id("scalar_type/boolean");
    let outer_boolean = node_id("value/parameter/outer-boolean");
    boolean_element["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(parameter_node(
            outer_boolean.clone(),
            "x",
            "0",
            boolean_type.clone(),
        ));
    let sequence_type =
        boolean_element["semantic_graph"]["nodes"][application]["body"]["result_type"].clone();
    let bounds_position = boolean_element["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == sequence_type)
        .expect("collection bounds");
    let sequence_id =
        boolean_element["semantic_graph"]["nodes"][bounds_position]["semantic_type"].clone();
    let sequence_position = boolean_element["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == sequence_id)
        .expect("sequence type");
    boolean_element["semantic_graph"]["nodes"][sequence_position]["body"]["members"][0]["target"] =
        boolean_type.clone();
    boolean_element["semantic_graph"]["nodes"][application]["body"]["operation"]["member"]
        ["declaration"] = boolean_type.clone();
    boolean_element["semantic_graph"]["nodes"][application]["body"]["arguments"][2]["value"]
        ["target"] = outer_boolean;
    boolean_element["semantic_graph"]["nodes"][application]["body"]["result_type"] =
        sequence_type.clone();
    boolean_element["semantic_graph"]["nodes"][application]["semantic_type"] =
        sequence_type.clone();
    boolean_element["semantic_graph"]["nodes"][function]["semantic_type"] = sequence_type;
    settle(&mut boolean_element);
    let boolean_result = read(&boolean_element);
    assert!(
        matches!(boolean_result, CheckedPackageV2ReadResult::Admitted(_)),
        "the seq index has integer_range type while its Boolean element is independent: {boolean_result:?}"
    );

    let mut unbounded_input = base.clone();
    let lower_literal = unbounded_input["semantic_graph"]["nodes"][application]["body"]
        ["arguments"][0]["target"]
        .clone();
    let lower_position = unbounded_input["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == lower_literal)
        .expect("lower literal");
    unbounded_input["semantic_graph"]["nodes"][lower_position]["semantic_type"] =
        node_id("scalar_type/integer");
    unbounded_input["semantic_graph"]["nodes"][lower_position]["body"]["type"] =
        node_id("scalar_type/integer");
    settle(&mut unbounded_input);
    assert_result_refusal(&unbounded_input, application, "filament.op.collection.seq");

    let mut wrong_index_range = base.clone();
    let parameter = wrong_index_range["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["semantic_form"] == "parameter"
                && node["body"]["members"][0]["value"]["value"] == "sequence_index"
        })
        .expect("index parameter");
    wrong_index_range["semantic_graph"]["nodes"][parameter]["semantic_type"] =
        node_id("scalar_type/integer");
    settle(&mut wrong_index_range);
    assert_result_refusal(
        &wrong_index_range,
        application,
        "filament.op.collection.seq",
    );

    let mut wrong_element = base.clone();
    let sequence_type =
        wrong_element["semantic_graph"]["nodes"][application]["body"]["result_type"].clone();
    let bounds = wrong_element["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == sequence_type)
        .expect("collection bounds");
    let sequence_id = wrong_element["semantic_graph"]["nodes"][bounds]["semantic_type"].clone();
    let sequence = wrong_element["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == sequence_id)
        .expect("sequence type");
    wrong_element["semantic_graph"]["nodes"][sequence]["body"]["members"][0]["target"] =
        node_id("scalar_type/rational");
    settle(&mut wrong_element);
    assert_result_refusal(&wrong_element, application, "filament.op.collection.seq");

    let mut wrong_cardinality = base.clone();
    wrong_cardinality["semantic_graph"]["nodes"][bounds]["body"]["members"][1]["value"]["value"] =
        json!("2");
    settle(&mut wrong_cardinality);
    assert_result_refusal(
        &wrong_cardinality,
        application,
        "filament.op.collection.seq",
    );

    let mut wrong_binder_shape = base.clone();
    wrong_binder_shape["semantic_graph"]["nodes"][application]["body"]["arguments"][2] =
        integer_literal("scalar_type/integer");
    settle(&mut wrong_binder_shape);
    assert_operand_refusal(
        &wrong_binder_shape,
        application,
        2,
        "filament.op.collection.seq",
    );
}

#[trace("TC-048", "FR-038-AC-251")]
#[test]
fn collection_only_round_trips_and_refuses_a_non_binder_operand() {
    let mut package = admitted_fixture("filament.op.collection.only").expect("only control");
    assert_catalogue_application_round_trip(&package);
    let application = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["body"]["operation"]["identity"] == "filament.op.collection.only")
        .expect("collection.only application");

    package["semantic_graph"]["nodes"][application]["body"]["arguments"][1] =
        integer_literal("scalar_type/integer");
    settle(&mut package);
    assert_operand_refusal(&package, application, 1, "filament.op.collection.only");
}

fn assert_result_refusal(package: &Value, position: usize, identity: &str) {
    let CheckedPackageV2ReadResult::Refused(refusal) = read(package) else {
        panic!(
            "{identity}: expected result refusal, got {:?}",
            read(package)
        );
    };
    assert_eq!(
        (
            refusal.code,
            refusal.cause,
            refusal.loci[0].path.as_ref().map(ToString::to_string),
            serde_json::to_value(refusal.locus).expect("locus")
        ),
        (
            CheckedPackageRefusalCode::IllTyped,
            Some(CheckedPackageRefusalCause::OperatorIneligible),
            // FSpec FR-322 locates a carried-sum result mismatch at the
            // application node itself, not at its `result_type`.
            Some(
                if matches!(
                    identity,
                    "filament.op.collection.sum.carried_quantity"
                        | "filament.op.collection.only"
                        | "filament.op.collection.seq"
                ) {
                    format!("/semantic_graph/nodes/{position}")
                } else {
                    format!("/semantic_graph/nodes/{position}/body/result_type")
                }
            ),
            package["semantic_graph"]["nodes"][position]["node_id"].clone()
        ),
        "{identity}"
    );
}

fn assert_operand_refusal(package: &Value, position: usize, operand: usize, identity: &str) {
    let CheckedPackageV2ReadResult::Refused(refusal) = read(package) else {
        panic!(
            "{identity}: expected operand refusal, got {:?}",
            read(package)
        );
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::IllTyped,
        "{identity}"
    );
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::OperatorIneligible),
        "{identity}"
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!(
            "/semantic_graph/nodes/{position}/body/arguments/{operand}"
        )),
        "{identity}"
    );
    assert_eq!(
        serde_json::to_value(refusal.locus).expect("locus"),
        package["semantic_graph"]["nodes"][position]["node_id"],
        "{identity}"
    );
}

#[trace("TC-048", "FR-038-AC-69", "FR-038-AC-251")]
#[test]
fn tc_048_member_kind_mismatch_precedes_member_result_check() {
    let mut package = admitted_fixture("filament.op.collection.fold").expect("fold control");
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    package["semantic_graph"]["nodes"][position]["body"]["operation"]["member"] = json!({"kind":"position", "declaration":node_id("scalar_type/integer"),
            "position":0});
    let wrong = "scalar_type/rational";
    package["semantic_graph"]["nodes"][position]["body"]["result_type"] = node_id(wrong);
    package["semantic_graph"]["nodes"][position]["semantic_type"] = node_id(wrong);
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("wrong member kind must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::OperationMemberMismatch)
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!(
            "/semantic_graph/nodes/{position}/body/operation/member"
        ))
    );
    assert_eq!(
        serde_json::to_value(refusal.locus).expect("locus"),
        package["semantic_graph"]["nodes"][position]["node_id"]
    );
}
