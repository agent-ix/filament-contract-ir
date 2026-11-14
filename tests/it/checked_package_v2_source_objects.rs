// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-475: the checked-package reader over source-declared object types
//! (FR-362 through FR-368). Every package is built here node by node.

use crate::support::checked_package::{
    canonical, evidence_for, family_key, fixture_source, node_id, settle, v2_all_families,
};
use filament_contract_model::{
    encode_checked_package_v2, CheckedNodeId, CheckedNodeOwner, CheckedOperationEffectOrigin,
    CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, CheckedPackageSidecarsRef,
    CheckedPackageSourceCorrespondenceRef, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use ix_trace_rs::trace;
use quire_canonical::Limits;
use serde_json::{json, Value};

const SOURCE_FRAME_ROW: &str = "source_declaration_operation_frame";
const MODEL_FRAME_ROW: &str = "model_document_operation_frame";

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    crate::read_and_account(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

fn admitted(package: &Value) -> CheckedPackageV2 {
    match read(package) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("expected admission, read {other:?}"),
    }
}

fn refused(package: &Value) -> CheckedPackageRefusal {
    match read(package) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected a refusal, read {other:?}"),
    }
}

/// Asserts the refusal's code, cause and first locus pointer.
fn expect(case: &str, package: &Value, code: Code, cause: Option<Cause>, path: &str) {
    let refusal = refused(package);
    assert_eq!(refusal.code, code, "{case}: {refusal:?}");
    assert_eq!(refusal.cause, cause, "{case}: {refusal:?}");
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|at| at.as_str()),
        Some(path),
        "{case}: {refusal:?}"
    );
}

fn nodes(package: &Value) -> &Vec<Value> {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
}

fn nodes_mut(package: &mut Value) -> &mut Vec<Value> {
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
}

fn position(package: &Value, tag: &str, form: &str) -> usize {
    nodes(package)
        .iter()
        .position(|node| node["node_tag"] == tag && node["semantic_form"] == form)
        .unwrap_or_else(|| panic!("a {tag}/{form} node"))
}

fn reference(target: &Value) -> Value {
    json!({"term": "reference", "target": target})
}

fn binding(name: &str, value: Value) -> Value {
    json!({"term": "binding", "name": name, "value": value})
}

fn integer() -> Value {
    node_id(&family_key("a3a3"))
}

fn text(value: &str) -> Value {
    json!({"term": "literal", "type": node_id(&family_key("a1a1")),
        "value_kind": "text", "value": value})
}

fn plain(
    label: &str,
    tag: &str,
    form: &str,
    semantic_type: &Value,
    dependencies: &[&Value],
    role: &str,
    body: Value,
) -> Value {
    json!({
        "node_id": node_id(label),
        "schema_version": "filament.checked-semantic-graph/v2",
        "node_tag": tag,
        "semantic_form": form,
        "semantic_type": semantic_type,
        "dependencies": dependencies,
        "occurrences": [{"role": role, "ordinal": 0}],
        "body": body,
    })
}

fn source_identity() -> String {
    fixture_source()["identity"]
        .as_str()
        .expect("identity")
        .to_owned()
}

fn id(label: &str) -> CheckedNodeId {
    serde_json::from_value(node_id(label)).expect("node id")
}

fn order_id() -> Value {
    node_id("qualified:Order")
}

/// A source-declared object type `name` with the `fields`, each an integer.
fn source_type(name: &str, fields: &[&str], source: &Value) -> Value {
    let mut node = plain(
        &format!("qualified:{name}"),
        "model",
        "object_type",
        &node_id(&format!("qualified:{name}")),
        &[&integer()],
        "declaration",
        json!({"term": "aggregate", "members":
            fields.iter().map(|field| binding(field, reference(&integer()))).collect::<Vec<_>>()}),
    );
    node["declaration"] = json!({"qualified_name": [name]});
    node["owner"] = json!({"kind": "source",
        "authority": source["authority"], "identity": source["identity"]});
    node
}

/// The source type `Order` with its field `total`, the modifying operation
/// `deposit` (a frame granting `total`, and its anchor) and the query `quote`
/// (no anchor), added to `package`.
fn add_source_order(package: &mut Value) {
    let order = order_id();
    let frame = node_id("fixture/src/frame");
    let added = [
        source_type("Order", &["total"], &fixture_source()),
        plain(
            "fixture/src/frame",
            "state",
            "frame",
            &order,
            &[&order],
            "generated",
            json!({"term": "frame", "modifies": [
                {"kind": "field", "declaration": order, "name": "total", "scope": "every-object"}],
                "creates": [], "deletes": [], "deletes_scoped": []}),
        ),
        plain(
            "fixture/src/anchor",
            "state",
            "operation_anchor",
            &order,
            &[&order, &frame],
            "anchor",
            json!({"term": "aggregate", "members": [
                binding("context", reference(&order)),
                binding("operation", text("deposit")),
                binding("frame", reference(&frame)),
            ]}),
        ),
    ];
    nodes_mut(package).extend(added);
}

/// The families package with its selected model removed, the protocol clause
/// moved onto the source type, and the source type added.
fn source_package() -> Value {
    let mut package = v2_all_families();
    let frame = position(&package, "state", "frame");
    nodes_mut(&mut package).retain(|node| {
        !matches!(node["owner"]["kind"].as_str(), Some("model" | "model_root"))
            && node["semantic_form"] != "frame"
    });
    let _ = frame;
    add_source_order(&mut package);
    let clause = position(&package, "protocol", "protocol_clause");
    nodes_mut(&mut package)[clause]["owner_model"] = order_id();
    settle(&mut package);
    package
}

/// The families package, which selects a model, with the source type added.
fn mixed_package() -> Value {
    let mut package = v2_all_families();
    add_source_order(&mut package);
    settle(&mut package);
    package
}

fn is_source_node(node: &filament_contract_model::CheckedSemanticNodeV2) -> bool {
    node.declaration.is_some()
        && node.semantic_form.as_ref() == "object_type"
        && matches!(node.owner, Some(CheckedNodeOwner::Source { .. }))
}

fn effect_row(declaring: &Value, document: &str, operation: &str, origin: &str) -> Value {
    let name = declaring["path"][0].as_str().expect("declared type name");
    let modifies = if operation == "deposit" {
        json!([{"kind": "field", "declaration": declaring, "name": "total",
            "scope": "every-object"}])
    } else {
        json!([])
    };
    json!({
        "document": document,
        "declaring": declaring,
        "operation": operation,
        "effect": {"modifies": modifies, "creates": [], "deletes": [], "deletes_scoped": []},
        "provenance": {"origin": origin, "member": format!("{name}::{operation}")},
    })
}

fn source_rows() -> Value {
    json!([
        effect_row(&order_id(), &source_identity(), "deposit", SOURCE_FRAME_ROW),
        effect_row(&order_id(), &source_identity(), "quote", SOURCE_FRAME_ROW),
    ])
}

fn with_effects(mut package: Value, rows: Value) -> Value {
    package["operation_effects"] = rows;
    package
}

/// Trace: FR-362-AC-2.
#[trace("TC-475", "FR-362-AC-2")]
#[test]
fn source_object_with_one_field_and_one_operation_admits_as_a_source_node() {
    let package = admitted(&source_package());
    let node = package
        .graph()
        .nodes
        .iter()
        .find(|node| is_source_node(node))
        .expect("the source-declared type");
    assert!(matches!(
        &node.owner,
        Some(CheckedNodeOwner::Source { identity, .. }) if identity.as_ref() == source_identity()
    ));
    assert_eq!(
        node.declaration.as_ref().map(|d| d.qualified_name.clone()),
        Some(vec!["Order".into()])
    );
}

/// Trace: FR-362-AC-3.
#[trace("TC-475", "FR-362-AC-3")]
#[test]
fn source_type_is_never_joined_to_a_selected_model() {
    // A lock selecting no domain document admits the source type.
    let alone = source_package();
    assert!(nodes(&alone)
        .iter()
        .all(|node| node["semantic_form"] != "model_root"));
    admitted(&alone);

    // A same-name pair keeps two distinct nodes with distinct owners.
    let mut pair = v2_all_families();
    let mut account = source_type("Account", &["total"], &fixture_source());
    account["dependencies"] = json!([integer()]);
    nodes_mut(&mut pair).push(account);
    settle(&mut pair);
    let package = admitted(&pair);
    let accounts: Vec<_> = package
        .graph()
        .nodes
        .iter()
        .filter(|node| {
            node.semantic_form.as_ref() == "object_type"
                && match &node.owner {
                    Some(CheckedNodeOwner::Source { .. }) => {
                        node.declaration
                            .as_ref()
                            .map(|d| d.qualified_name == vec!["Account".into()])
                            == Some(true)
                    }
                    Some(CheckedNodeOwner::Model { node, .. }) => node.ends_with("/Account"),
                    _ => false,
                }
        })
        .collect();
    assert_eq!(accounts.len(), 2);
    assert_ne!(accounts[0].node_id, accounts[1].node_id);
    assert_ne!(accounts[0].owner, accounts[1].owner);
}

/// Trace: FR-363-AC-1, FR-363-AC-2.
#[trace("TC-475", "FR-363-AC-1", "FR-363-AC-2")]
#[test]
fn protocol_clause_and_role_over_a_source_type_admit() {
    let package = source_package();
    let admitted_package = admitted(&package);
    let clause = admitted_package
        .graph()
        .nodes
        .iter()
        .find(|node| node.semantic_form.as_ref() == "protocol_clause")
        .expect("clause");
    assert_eq!(clause.owner_model.as_ref(), Some(&id("qualified:Order")));

    let mut roled = package.clone();
    let role = crate::support::checked_package::protocol_role_node();
    let role_id = role["node_id"].clone();
    nodes_mut(&mut roled).push(role);
    let clause = position(&roled, "protocol", "protocol_clause");
    nodes_mut(&mut roled)[clause]["body"]["arguments"][3]["value"]["members"] =
        json!([reference(&role_id)]);
    let role = position(&roled, "protocol", "role");
    nodes_mut(&mut roled)[role]["body"]["arguments"][0]["value"] = reference(&order_id());
    settle(&mut roled);
    admitted(&roled);
}

/// Trace: FR-363-AC-3.
#[trace("TC-475", "FR-363-AC-3")]
#[test]
fn missing_lock_entry_refuses_at_the_source_type_not_the_clause() {
    let mut package = source_package();
    let order = position(&package, "model", "object_type");
    nodes_mut(&mut package)[order]["owner"]["identity"] = json!("filament.fixture.unlocked/v1");
    settle(&mut package);
    let refusal = refused(&package);
    assert_eq!(refusal.code, Code::MissingDeclaration);
    assert_eq!(refusal.cause, Some(Cause::MissingSelection));
    assert_eq!(
        refusal
            .locus
            .as_ref()
            .map(|id| serde_json::to_value(id).expect("locus")),
        Some(order_id())
    );
    assert_ne!(
        refusal.loci[0].path.as_ref().map(|at| at.as_str()),
        Some(
            format!(
                "/semantic_graph/nodes/{}/owner_model",
                position(&package, "protocol", "protocol_clause")
            )
            .as_str()
        )
    );
}

/// Trace: FR-363-AC-4.
#[trace("TC-475", "FR-363-AC-4")]
#[test]
fn clause_edge_to_a_source_node_that_is_no_object_type_refuses() {
    let mut package = source_package();
    let clause = position(&package, "protocol", "protocol_clause");
    // A source record: a declared composite_type/record node.
    let mut record = plain(
        "qualified:Line",
        "composite_type",
        "record",
        &node_id("qualified:Line"),
        &[&integer()],
        "declaration",
        json!({"term": "aggregate", "members": [binding("n", reference(&integer()))]}),
    );
    record["declaration"] = json!({"qualified_name": ["Line"]});
    record["owner"] = json!({"kind": "source", "authority": fixture_source()["authority"],
        "identity": fixture_source()["identity"]});
    nodes_mut(&mut package).push(record);
    nodes_mut(&mut package)[clause]["owner_model"] = node_id("qualified:Line");
    settle(&mut package);
    expect(
        "record owner_model",
        &package,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        &format!("/semantic_graph/nodes/{clause}/owner_model"),
    );
}

/// Trace: FR-363-AC-5.
#[trace("TC-475", "FR-363-AC-5")]
#[test]
fn model_owned_clause_refusals_are_unchanged_beside_a_source_type() {
    let base = v2_all_families();
    let mixed = mixed_package();
    admitted(&mixed);
    let clause = position(&mixed, "protocol", "protocol_clause");
    let edits: [(&str, Value); 3] = [
        ("missing node", node_id("fixture/src/absent")),
        ("non-model node", integer()),
        (
            "model_root",
            nodes(&base)[position(&base, "model", "model_root")]["node_id"].clone(),
        ),
    ];
    for (case, target) in edits {
        let mut plain_package = base.clone();
        nodes_mut(&mut plain_package)[clause]["owner_model"] = target.clone();
        let mut mixed_package = mixed.clone();
        nodes_mut(&mut mixed_package)[clause]["owner_model"] = target;
        let expected = refused(&plain_package);
        let actual = refused(&mixed_package);
        assert_eq!(actual.code, expected.code, "{case}");
        assert_eq!(actual.cause, expected.cause, "{case}");
        assert_eq!(actual.loci, expected.loci, "{case}");
        assert_eq!(actual.locus, expected.locus, "{case}");
    }
}

/// Trace: FR-365-AC-1.
#[trace("TC-475", "FR-365-AC-1")]
#[test]
fn source_effect_rows_admit_read_in_order_and_re_encode_identically() {
    let package = with_effects(source_package(), source_rows());
    let admitted_package = admitted(&package);
    let rows = admitted_package
        .operation_effects()
        .expect("carrier")
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].operation(), "deposit");
    assert_eq!(rows[1].operation(), "quote");
    for row in &rows {
        assert_eq!(row.document(), source_identity());
        assert_eq!(row.declaring(), &id("qualified:Order"));
        assert_eq!(
            row.provenance().origin,
            CheckedOperationEffectOrigin::SourceDeclarationOperationFrame
        );
    }
    assert_eq!(rows[0].provenance().member.as_ref(), "Order::deposit");
    assert_eq!(rows[0].effect().modifies.len(), 1);
    assert!(rows[1].effect().modifies.is_empty());
    assert!(rows[1].effect().creates.is_empty());
    assert!(rows[1].effect().deletes.is_empty());
    assert!(rows[1].effect().deletes_scoped.is_empty());

    let evidence = evidence_for(&package);
    let encoded = encode_checked_package_v2(
        package["name"].as_str().expect("name"),
        admitted_package.lock(),
        admitted_package.graph(),
        CheckedPackageSourceCorrespondenceRef {
            source_map: admitted_package.source_map(),
            refinement_step_rows: admitted_package.refinement_step_rows(),
        },
        CheckedPackageSidecarsRef {
            temporal_clause_names: admitted_package.temporal_clause_names(),
            schedulability_claims: None,
            operation_effects: admitted_package.operation_effect_rows(),
            operation_bodies: None,
            checked_term_data: admitted_package.checked_term_data(),
            temporal_bindings: admitted_package.temporal_bindings(),
        },
        admitted_package.capability_report(),
        admitted_package.diagnostics(),
        Limits::new(1 << 24),
    )
    .expect("typed rows encode");
    let CheckedPackageV2ReadResult::Admitted(reread) =
        CheckedPackageV2::read(&encoded, CheckedPackageReadLimits::bounded(), &evidence)
    else {
        panic!("the encoded rows must admit");
    };
    assert_eq!(
        reread.operation_effect_rows(),
        admitted_package.operation_effect_rows()
    );
}

/// Trace: FR-365-AC-2, FR-365-AC-3, FR-365-AC-4.
#[trace("TC-475", "FR-365-AC-2", "FR-365-AC-3", "FR-365-AC-4")]
#[test]
fn edited_source_effect_rows_refuse_at_the_first_failing_check() {
    let invalid = (Code::InvalidPackage, Some(Cause::InvalidValue));
    let at = |member: &str| format!("/operation_effects/0/{member}");
    let cases: Vec<(&str, Box<dyn Fn(&mut Value)>, (Code, Option<Cause>), String)> = vec![
        (
            "document",
            Box::new(|row| row["document"] = json!("filament.fixture.other/v1")),
            invalid,
            at("document"),
        ),
        (
            "provenance.member",
            Box::new(|row| row["provenance"]["member"] = json!("Order::withdraw")),
            invalid,
            at("provenance/member"),
        ),
        (
            "origin over a source type",
            Box::new(|row| row["provenance"]["origin"] = json!(MODEL_FRAME_ROW)),
            invalid,
            at("provenance/origin"),
        ),
        (
            "effect.modifies",
            Box::new(|row| row["effect"]["modifies"] = json!([])),
            invalid,
            at("effect/modifies"),
        ),
        (
            "effect.creates",
            Box::new(|row| row["effect"]["creates"] = json!([order_id()])),
            invalid,
            at("effect/creates"),
        ),
        (
            "effect.deletes",
            Box::new(|row| row["effect"]["deletes"] = json!([order_id()])),
            invalid,
            at("effect/deletes"),
        ),
        (
            "effect.deletes_scoped",
            Box::new(|row| row["effect"]["deletes_scoped"] = json!([{"kind": "receiver"}])),
            invalid,
            at("effect/deletes_scoped"),
        ),
    ];
    for (case, edit, (code, cause), path) in cases {
        let mut rows = source_rows();
        edit(&mut rows[0]);
        expect(
            case,
            &with_effects(source_package(), rows),
            code,
            cause,
            &path,
        );
    }
    // A source origin over a model-owned declaring node.
    let mut rows = source_rows();
    rows[0]["declaring"] = node_id("fixture/1515");
    rows[0]["document"] = json!("test/families");
    expect(
        "origin over a model node",
        &with_effects(mixed_package(), rows),
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        &at("provenance/origin"),
    );
    // A declaring node no row can name.
    let mut rows = source_rows();
    rows[0]["declaring"] = node_id("qualified:Absent");
    expect(
        "absent declaring",
        &with_effects(source_package(), rows),
        Code::MissingDeclaration,
        Some(Cause::MissingName),
        &at("declaring"),
    );
}

/// Trace: FR-365-AC-5.
#[trace("TC-475", "FR-365-AC-5")]
#[test]
fn source_and_model_rows_with_equal_documents_coexist_and_repeats_refuse() {
    // A second locked source whose identity is the selected model's identity
    // gives a source row and a model row the same `document` string.
    let mut package = v2_all_families();
    let shared = json!({"authority": fixture_source()["authority"], "identity": "test/families"});
    package["lock"]["sources"]
        .as_array_mut()
        .expect("sources")
        .push(shared.clone());
    nodes_mut(&mut package).push(source_type("Ledger", &["total"], &shared));
    settle(&mut package);
    let ledger = node_id("qualified:Ledger");
    for entry in package["source_map"].as_array_mut().expect("source map") {
        if entry["node_id"] == ledger {
            entry["regions"][0]["source"] = shared.clone();
        }
    }
    let model_row = |op: &str| {
        let mut row = effect_row(
            &node_id("fixture/1515"),
            "test/families",
            op,
            MODEL_FRAME_ROW,
        );
        row["provenance"]["member"] = json!(format!("ix://test/families/Account/{op}"));
        row["effect"]["modifies"] = json!([]);
        row
    };
    let source_row = effect_row(&ledger, "test/families", "post", SOURCE_FRAME_ROW);
    let mut rows = vec![
        model_row("deposit"),
        model_row("withdraw"),
        source_row.clone(),
    ];
    rows.sort_by_key(|row| {
        let declaring: CheckedNodeId =
            serde_json::from_value(row["declaring"].clone()).expect("id");
        (
            row["document"].as_str().expect("document").to_owned(),
            declaring,
            row["operation"].as_str().expect("op").to_owned(),
        )
    });
    let package = with_effects(package, json!(rows));
    let read_package = admitted(&package);
    let documents: Vec<_> = read_package
        .operation_effects()
        .expect("carrier")
        .map(|row| (row.document().to_owned(), row.provenance().origin))
        .collect();
    assert_eq!(documents.len(), 3);
    assert!(documents
        .iter()
        .all(|(document, _)| document == "test/families"));
    assert_eq!(
        documents
            .iter()
            .filter(|(_, origin)| *origin
                == CheckedOperationEffectOrigin::SourceDeclarationOperationFrame)
            .count(),
        1
    );
    // A repeated key refuses as the model carrier does.
    let mut repeated = rows.clone();
    repeated.insert(repeated.len(), source_row);
    let refusal = refused(&with_effects(package.clone(), json!(repeated)));
    assert_eq!(refusal.code, Code::InvalidPackage);
    assert_eq!(refusal.cause, Some(Cause::ConflictingDefinition));
}

/// Trace: FR-365-AC-6.
#[trace("TC-475", "FR-365-AC-6")]
#[test]
fn model_rows_read_unchanged_beside_a_source_row_and_a_source_type_needs_no_row() {
    let mut mixed = mixed_package();
    let model_rows = |origin_member: &str| {
        let mut row = effect_row(
            &node_id("fixture/1515"),
            "test/families",
            origin_member,
            MODEL_FRAME_ROW,
        );
        row["provenance"]["member"] = json!(format!("ix://test/families/Account/{origin_member}"));
        row["effect"]["modifies"] = json!([]);
        row
    };
    let source_row = effect_row(&order_id(), &source_identity(), "quote", SOURCE_FRAME_ROW);
    let mut rows = vec![model_rows("deposit"), model_rows("withdraw"), source_row];
    rows.sort_by_key(|row| {
        let declaring: CheckedNodeId =
            serde_json::from_value(row["declaring"].clone()).expect("id");
        (
            row["document"].as_str().expect("document").to_owned(),
            declaring,
            row["operation"].as_str().expect("op").to_owned(),
        )
    });
    mixed["operation_effects"] = json!(rows);
    let package = admitted(&mixed);
    let read: Vec<_> = package
        .operation_effects()
        .expect("carrier")
        .map(|row| row.provenance().origin)
        .collect();
    assert_eq!(
        read.iter()
            .filter(|origin| **origin == CheckedOperationEffectOrigin::ModelDocumentOperationFrame)
            .count(),
        2
    );
    // A carrier holding only model rows beside a source type admits: the
    // source type owes no row.
    let mut only_model = mixed_package();
    only_model["operation_effects"] = json!([model_rows("deposit"), model_rows("withdraw")]);
    admitted(&only_model);
}

fn source_population(name: &str) -> Value {
    let mut node = plain(
        &format!("qualified:{name}"),
        "relation",
        "population",
        &node_id(&format!("qualified:{name}")),
        &[],
        "declaration",
        json!({"term": "aggregate", "members": []}),
    );
    node["declaration"] = json!({"qualified_name": [name]});
    node["owner"] = json!({"kind": "source", "authority": fixture_source()["authority"],
        "identity": fixture_source()["identity"]});
    node
}

/// A `Set<Reference<Order>>` result type.
fn add_set_of_orders(package: &mut Value) -> Value {
    let reference_type = plain(
        "query/type/reference",
        "composite_type",
        "reference",
        &node_id("query/type/reference"),
        &[],
        "type",
        json!({"term": "aggregate", "members": [reference(&order_id())]}),
    );
    let set_type = plain(
        "query/type/set",
        "composite_type",
        "set",
        &node_id("query/type/set"),
        &[],
        "type",
        json!({"term": "aggregate", "members": [reference(&node_id("query/type/reference"))]}),
    );
    nodes_mut(package).extend([reference_type, set_type]);
    node_id("query/type/set")
}

/// `all_instances` over `population`, typed `Set<Reference<Order>>`.
fn all_instances(population: &Value, set: &Value, label: &str) -> Value {
    plain(
        label,
        "expression",
        "query",
        set,
        &[],
        "expression",
        json!({"term": "application", "operator": "query",
            "operation": {"identity": "filament.op.model.all_instances", "laws": [],
                "mode": null,
                "member": {"kind": "type_argument", "declaration": order_id()}, "leaves": []},
            "result_type": set, "arguments": [reference(population)]}),
    )
}

fn node_id_at(package: &Value, label: &str) -> usize {
    nodes(package)
        .iter()
        .position(|node| node["node_id"] == node_id(label))
        .unwrap_or_else(|| panic!("node {label}"))
}

/// The source package with the source populations `names`.
fn population_package(names: &[&str]) -> Value {
    let mut package = source_package();
    for name in names {
        nodes_mut(&mut package).push(source_population(name));
    }
    settle(&mut package);
    package
}

/// Trace: FR-364-AC-1.
#[trace("TC-475", "FR-364-AC-1")]
#[test]
fn source_population_reference_admits_self_typed_and_distinct_per_name() {
    let package = population_package(&["Orders", "Archive"]);
    let read_package = admitted(&package);
    let populations: Vec<_> = read_package
        .graph()
        .nodes
        .iter()
        .filter(|node| node.semantic_form.as_ref() == "population")
        .collect();
    assert_eq!(populations.len(), 2);
    for node in &populations {
        assert_eq!(node.semantic_type, node.node_id, "self-typed");
        assert!(matches!(node.owner, Some(CheckedNodeOwner::Source { .. })));
    }
    assert_ne!(populations[0].node_id, populations[1].node_id);
}

fn add_lookup(package: &mut Value, population: &Value) {
    let set = add_set_of_orders(package);
    let reference_type = node_id("query/type/reference");
    let option = plain(
        "query/type/option",
        "composite_type",
        "option",
        &node_id("query/type/option"),
        &[],
        "type",
        json!({"term": "aggregate", "members": [reference(&reference_type)]}),
    );
    let identity = plain(
        "query/expression/object",
        "expression",
        "reference",
        &reference_type,
        &[],
        "expression",
        reference(&order_id()),
    );
    let lookup = plain(
        "query/app/lookup",
        "expression",
        "query",
        &node_id("query/type/option"),
        &[],
        "expression",
        json!({"term": "application", "operator": "query",
            "operation": {"identity": "filament.op.model.lookup", "laws": [],
                "mode": {"kind": "absence", "value": "option"},
                "member": {"kind": "type_argument", "declaration": order_id()}, "leaves": []},
            "result_type": node_id("query/type/option"),
            "arguments": [reference(population), reference(&node_id("query/expression/object"))]}),
    );
    nodes_mut(package).extend([option, identity, lookup]);
    nodes_mut(package).push(all_instances(population, &set, "query/app/all"));
}

/// Trace: FR-364-AC-2.
#[trace("TC-475", "FR-364-AC-2")]
#[test]
fn all_instances_and_lookup_over_a_source_population_admit() {
    let mut package = population_package(&["Orders"]);
    let population = node_id("qualified:Orders");
    add_lookup(&mut package, &population);
    settle(&mut package);
    let read_package = admitted(&package);
    for label in ["query/app/all", "query/app/lookup"] {
        let application = read_package
            .graph()
            .nodes
            .iter()
            .find(|node| node.node_id == id(label))
            .expect("application");
        assert!(
            application.dependencies.contains(&id("qualified:Orders")),
            "{label} depends on the population"
        );
    }
}

const MODEL_POPULATION: &str = "ix://test/families/Pop";

fn population_document() -> Value {
    let mut document = crate::support::checked_package::family_model_document();
    document["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind": {"module": "test/families", "name": "population"},
            "construct": {"meaning": "filament.meaning.model.population/v1"},
        }));
    document["populations"] = json!([{
        "identity": MODEL_POPULATION, "displayName": "Pop",
        "kind": {"module": "test/families", "name": "population"},
        "members": ["ix://test/families/Account"], "extent": "closed",
        "origin": {"source": {"sourceIdentity": "ix://test/families/spec",
            "path": "spec/populations.md", "startLine": 1, "startColumn": 1}},
    }]);
    document
}

/// The id of the population `Pop` and the package that declares it, owned
/// by the selected model or declared in source.
fn forbidden_position_package(source: bool, application: &dyn Fn(&Value) -> Value) -> Value {
    let mut package = if source { mixed_package() } else { v2_all_families() };
    let (population, node) = if source {
        (node_id("qualified:Pop"), source_population("Pop"))
    } else {
        let id = json!({"package": "test/families",
            "path": ["relation", "population", MODEL_POPULATION], "ordinal": 0});
        let mut node = plain(
            "fixture/model-pop",
            "relation",
            "population",
            &id,
            &[],
            "generated",
            json!({"term": "aggregate", "members": []}),
        );
        node["node_id"] = id.clone();
        node["owner"] = json!({"kind": "model", "identity": "test/families",
            "node": MODEL_POPULATION});
        (id, node)
    };
    // A `Reference<Account>` parameter, the receiver the member checks expect.
    let account = node_id("fixture/1515");
    let reference_type = plain(
        "fixture/src/ref-account",
        "composite_type",
        "reference",
        &node_id("fixture/src/ref-account"),
        &[&account],
        "type",
        json!({"term": "aggregate", "members": [reference(&account)]}),
    );
    let receiver = plain(
        "fixture/src/receiver",
        "value",
        "parameter",
        &node_id("fixture/src/ref-account"),
        &[],
        "expression",
        json!({"term": "aggregate", "members": [
            binding("name", text("self")),
            binding("level", json!({"term": "literal", "type": integer(),
                "value_kind": "integer", "value": "0"})),
        ]}),
    );
    nodes_mut(&mut package).extend([reference_type, receiver, node]);
    nodes_mut(&mut package).push(application(&population));
    settle(&mut package);
    package
}

fn read_forbidden(package: &Value) -> CheckedPackageRefusal {
    let evidence = crate::support::checked_package::evidence_with_document(
        package,
        "test/families",
        &population_document(),
    );
    match crate::read_and_account(&canonical(package), CheckedPackageReadLimits::bounded(), &evidence) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected a refusal, read {other:?}"),
    }
}

fn boolean() -> Value {
    node_id(&family_key("aaaa"))
}

fn application(label: &str, form: &str, operator: &str, identity: &str, member: Value, arguments: Vec<Value>) -> Value {
    plain(
        label,
        "expression",
        form,
        &boolean(),
        &[],
        "expression",
        json!({"term": "application", "operator": operator,
            "operation": {"identity": identity, "laws": [], "mode": null,
                "member": member, "leaves": []},
            "result_type": boolean(), "arguments": arguments}),
    )
}

/// Trace: FR-364-AC-3.
#[trace("TC-475", "FR-364-AC-3")]
#[test]
fn forbidden_population_positions_refuse_for_source_as_for_model_populations() {
    type Case = (&'static str, Box<dyn Fn(&Value) -> Value>, &'static str);
    let receiver = || reference(&node_id("fixture/src/receiver"));
    let cases: Vec<Case> = vec![
        (
            "relationship declaration",
            Box::new(move |pop| {
                application(
                    "q/app",
                    "reachability",
                    "reaches",
                    "filament.op.model.reaches",
                    json!({"kind": "relationship_end", "declaration": pop, "name": "order",
                        "destination_type": order_id()}),
                    vec![receiver(), receiver()],
                )
            }),
            "/body/operation/member/declaration",
        ),
        (
            "field declaration",
            Box::new(move |pop| {
                application(
                    "q/app",
                    "query",
                    "query",
                    "filament.op.record.project",
                    json!({"kind": "field", "declaration": pop, "name": "n"}),
                    vec![receiver()],
                )
            }),
            "/body/arguments/0",
        ),
        (
            "operation declaration",
            Box::new(move |pop| {
                application(
                    "q/app",
                    "call",
                    "call",
                    "filament.op.model.dispatch_call",
                    json!({"kind": "operation", "declaration": pop, "name": "run"}),
                    vec![receiver()],
                )
            }),
            "/body/operation/member/name",
        ),
        (
            "collection operand",
            Box::new(move |pop| {
                application(
                    "q/app",
                    "collection",
                    "collection",
                    "filament.op.collection.contains",
                    Value::Null,
                    vec![reference(pop), reference(&boolean())],
                )
            }),
            "/body/arguments/0",
        ),
    ];
    for (case, build, suffix) in &cases {
        let model = read_forbidden(&forbidden_position_package(false, build.as_ref()));
        let source = read_forbidden(&forbidden_position_package(true, build.as_ref()));
        assert_eq!(
            (model.code, model.cause),
            (Code::IllTyped, Some(Cause::OperatorIneligible)),
            "{case}: the model-owned oracle"
        );
        assert_eq!((source.code, source.cause), (model.code, model.cause), "{case}");
        let pointer = |refusal: &CheckedPackageRefusal| {
            refusal.loci[0]
                .path
                .as_ref()
                .expect("path")
                .as_str()
                .split_once("/nodes/")
                .and_then(|(_, rest)| rest.split_once('/'))
                .map(|(_, tail)| format!("/{tail}"))
                .expect("a node pointer")
        };
        assert_eq!(pointer(&model), *suffix, "{case}: the model locus");
        assert_eq!(pointer(&source), pointer(&model), "{case}");
        assert_eq!(source.locus, model.locus, "{case}");
    }
}

/// A source-declared record `name`, an existing declared construct whose
/// repeated declaration the reader already refuses.
fn source_record(name: &str) -> Value {
    let mut record = plain(
        &format!("qualified:{name}"),
        "composite_type",
        "record",
        &node_id(&format!("qualified:{name}")),
        &[&integer()],
        "declaration",
        json!({"term": "aggregate", "members": [binding("n", reference(&integer()))]}),
    );
    record["declaration"] = json!({"qualified_name": [name]});
    record["owner"] = json!({"kind": "source", "authority": fixture_source()["authority"],
        "identity": fixture_source()["identity"]});
    record
}

/// Trace: FR-364-AC-4.
#[trace("TC-475", "FR-364-AC-4")]
#[test]
fn population_owner_and_declaration_defects_refuse_as_the_owner_join_does() {
    let at = |package: &Value| position(package, "relation", "population");
    // A `declaration` beside a `ModelOwner` of no selected declaration.
    let mut package = population_package(&["Orders"]);
    let position = at(&package);
    nodes_mut(&mut package)[position]["owner"] = json!({"kind": "model",
        "identity": "test/families", "node": "ix://test/families/Orders"});
    settle(&mut package);
    expect(
        "model owner with a declaration",
        &package,
        Code::MalformedWire,
        None,
        &format!("/semantic_graph/nodes/{position}/owner"),
    );
    // A `SourceOwner` and no `declaration`.
    let mut package = population_package(&["Orders"]);
    let position = at(&package);
    let node = &mut nodes_mut(&mut package)[position];
    node.as_object_mut().expect("node").remove("declaration");
    node["occurrences"] = json!([{"role": "generated", "ordinal": 0}]);
    settle(&mut package);
    expect(
        "source owner without a declaration",
        &package,
        Code::MalformedWire,
        None,
        &format!("/semantic_graph/nodes/{position}/owner"),
    );
}

/// Trace: FR-364-AC-5.
#[trace("TC-475", "FR-364-AC-5")]
#[test]
fn source_populations_key_by_name_and_a_repeated_name_refuses_as_a_repeated_declaration() {
    let package = population_package(&["Orders", "Archive"]);
    let keys: Vec<_> = nodes(&package)
        .iter()
        .filter(|node| node["semantic_form"] == "population")
        .map(|node| node["node_id"].clone())
        .collect();
    assert_eq!(keys.len(), 2);
    assert_ne!(keys[0], keys[1]);
    admitted(&package);

    let mut repeated = population_package(&["Orders"]);
    nodes_mut(&mut repeated).push(source_population("Orders"));
    settle(&mut repeated);
    let refusal = refused(&repeated);
    // The oracle: a repeated declaration of any other declared construct.
    let mut record_pair = source_package();
    nodes_mut(&mut record_pair).extend([source_record("Line"), source_record("Line")]);
    settle(&mut record_pair);
    let oracle = refused(&record_pair);
    assert_eq!(refusal.code, oracle.code);
    assert_eq!(refusal.cause, oracle.cause);
    let tail = |refusal: &CheckedPackageRefusal| {
        refusal.loci[0].path.as_ref().expect("path").as_str().rsplit('/').next().map(str::to_owned)
    };
    assert_eq!(tail(&refusal), tail(&oracle));
    assert_eq!(refusal.locus.is_some(), oracle.locus.is_some());
}
