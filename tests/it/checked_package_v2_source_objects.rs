// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-475: the checked-package reader over source-declared object types
//! (FR-362 through FR-368). Every package is built here node by node.

use crate::support::checked_package::{
    canonical, evidence_for, family_key, fixture_source, index_fixture_node_ids, node_id, settle,
    v2_all_families,
};
use filament_contract_model::{
    encode_checked_package_v2, CheckedNodeId, CheckedNodeOwner, CheckedOperationEffectOrigin,
    CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, CheckedPackageSidecarsRef,
    CheckedPackageSourceCorrespondenceRef, CheckedPackageV2, CheckedPackageV2ReadResult,
    SelectedSourceObjectErrorCode as RowError, SelectedSourceObjectMember, SelectedSourceObjectRow,
    SelectedSourceObjects, SelectedSourcePopulation,
};
use filament_contract_model::{
    CheckedCallableIdentity, CheckedCallableReadErrorCode, ModelForm, SelectedOperationIdentity,
    SelectedOperationMemberErrorCode as OperationError, SelectedOperationMemberRow,
    SelectedOperationMembers, SelectedSourceOperationRow, SelectedSourceOperations,
};
use ix_trace_rs::trace;
use quire_canonical::Limits;
use serde_json::{json, Value};

type EffectCase = (
    &'static str,
    Box<dyn Fn(&mut Value)>,
    (Code, Option<Cause>),
    String,
);
type DispatchCase = (&'static str, Box<dyn Fn(&mut Value)>, String);
type KeyEdit = (&'static str, Box<dyn Fn(&mut Value)>);
type RowEdit = (&'static str, Box<dyn Fn(&mut SelectedSourceObjectRow)>);
type OperationEdit = (
    &'static str,
    Box<dyn Fn(&mut SelectedSourceOperationRow)>,
    OperationError,
);

const SOURCE_FRAME_ROW: &str = "source_declaration_operation_frame";
const MODEL_FRAME_ROW: &str = "model_document_operation_frame";

/// The package with every nested structural id written as the graph-local
/// index the wire carries.
fn indexed(package: &Value) -> Value {
    let mut package = package.clone();
    index_fixture_node_ids(&mut package);
    package
}

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    let package = indexed(package);
    crate::read_and_account(
        &canonical(&package),
        CheckedPackageReadLimits::bounded(),
        &evidence_of(&package),
    )
}

/// The model operation `deposit` returns an `Integer`, so a call on it types.
fn dispatch_model_document() -> Value {
    let mut document = crate::support::checked_package::family_model_document();
    document["types"][0]["operations"][0]["returns"] = json!({
        "typeRef": "ix://filament/native/Integer",
        "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true},
        "nullable": false,
    });
    document
}

fn evidence_of(package: &Value) -> filament_contract_model::CheckedPackageEvidence {
    if nodes(package)
        .iter()
        .any(|node| node["node_id"] == node_id(MODEL_CALL))
    {
        crate::support::checked_package::evidence_with_document(
            package,
            "test/families",
            &dispatch_model_document(),
        )
    } else {
        evidence_for(package)
    }
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
    {
        let mut node = json!({
            "node_id": node_id(label),
            "schema_version": "filament.checked-semantic-graph/v2",
            "node_tag": tag,
            "semantic_form": form,
            "semantic_type": semantic_type,
            "dependencies": dependencies,
            "occurrences": [{"role": role, "ordinal": 0}],
            "body": body,
        });
        crate::support::checked_package::carry_check_mode(&mut node);
        node
    }
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

/// Trace: FR-362-AC-1. This test proves the package-level half: a package that
/// declares none of the source constructs holds no source-declared node and
/// re-encodes stably. The unchanged-behaviour half is TC-475 step 4: the
/// existing TC-048, TC-228, TC-229, TC-463 and TC-472 suites pass unedited.
#[trace("TC-475", "FR-362-AC-1")]
#[test]
fn a_package_without_a_source_object_reads_and_re_encodes_stably() {
    let package = v2_all_families();
    let admitted_package = admitted(&package);
    assert!(!admitted_package.graph().nodes.iter().any(is_source_node));
    let evidence = evidence_of(&indexed(&package));
    let name = package["name"].as_str().expect("name").to_owned();
    let encode = |package: &CheckedPackageV2| {
        encode_checked_package_v2(
            &name,
            package.lock(),
            package.graph(),
            CheckedPackageSourceCorrespondenceRef {
                source_map: package.source_map(),
                refinement_step_rows: package.refinement_step_rows(),
            },
            CheckedPackageSidecarsRef {
                temporal_clause_names: package.temporal_clause_names(),
                claim_names: package.claim_names(),
                schedulability_claims: None,
                operation_effects: package.operation_effect_rows(),
                operation_bodies: None,
                checked_term_data: package.checked_term_data(),
                temporal_bindings: package.temporal_bindings(),
            },
            package.capability_report(),
            package.diagnostics(),
            Limits::new(1 << 24),
        )
        .expect("typed package encodes")
    };
    let first = encode(&admitted_package);
    let CheckedPackageV2ReadResult::Admitted(reread) =
        CheckedPackageV2::read(&first, CheckedPackageReadLimits::bounded(), &evidence)
    else {
        panic!("the encoded package must admit");
    };
    assert_eq!(encode(&reread), first);
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

/// A declared source record `Line`, added to `package`.
fn add_source_record(package: &mut Value) {
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
    nodes_mut(package).push(record);
}

/// Trace: FR-363-AC-4.
#[trace("TC-475", "FR-363-AC-4")]
#[test]
fn a_clause_or_role_model_naming_a_source_node_that_is_no_object_type_refuses() {
    let expected = (Code::InvalidModelBinding, Some(Cause::MalformedDeclaration));
    for (case, target) in [
        ("source record", node_id("qualified:Line")),
        ("population reference", node_id("qualified:Pop")),
    ] {
        let mut package = source_package();
        add_source_record(&mut package);
        nodes_mut(&mut package).push(source_population("Pop"));
        let role = crate::support::checked_package::protocol_role_node();
        let role_id = role["node_id"].clone();
        nodes_mut(&mut package).push(role);
        let clause = position(&package, "protocol", "protocol_clause");
        nodes_mut(&mut package)[clause]["body"]["arguments"][3]["value"]["members"] =
            json!([reference(&role_id)]);
        let role = position(&package, "protocol", "role");
        nodes_mut(&mut package)[role]["body"]["arguments"][0]["value"] = reference(&target);
        settle(&mut package);
        let refusal = refused(&package);
        assert_eq!(
            (refusal.code, refusal.cause),
            expected,
            "role model: {case}"
        );
        let at = refusal.loci[0].path.as_ref().expect("path");
        assert!(
            at.as_str()
                .starts_with(&format!("/semantic_graph/nodes/{role}/")),
            "role model: {case}: {at:?}"
        );

        let mut package = source_package();
        add_source_record(&mut package);
        nodes_mut(&mut package).push(source_population("Pop"));
        let clause = position(&package, "protocol", "protocol_clause");
        nodes_mut(&mut package)[clause]["owner_model"] = target;
        settle(&mut package);
        expect(
            &format!("owner_model: {case}"),
            &package,
            expected.0,
            expected.1,
            &format!("/semantic_graph/nodes/{clause}/owner_model"),
        );
    }
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
    // An unselected target: the clause's model node names an owner identity the
    // lock does not select.
    let unselect = |package: &Value| {
        let mut package = package.clone();
        let object = position(&package, "model", "object_type");
        nodes_mut(&mut package)[object]["owner"]["identity"] = json!("unselected");
        settle(&mut package);
        refused(&package)
    };
    let expected = unselect(&base);
    let actual = unselect(&mixed);
    assert_eq!(actual.code, expected.code, "unselected target");
    assert_eq!(actual.cause, expected.cause, "unselected target");
    assert_eq!(actual.loci, expected.loci, "unselected target");
    assert_eq!(actual.locus, expected.locus, "unselected target");
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
            claim_names: admitted_package.claim_names(),
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
    let cases: Vec<EffectCase> = vec![
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
    let mut package = if source {
        mixed_package()
    } else {
        v2_all_families()
    };
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
    let package = &indexed(package);
    let evidence = crate::support::checked_package::evidence_with_document(
        package,
        "test/families",
        &population_document(),
    );
    match crate::read_and_account(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected a refusal, read {other:?}"),
    }
}

fn boolean() -> Value {
    node_id(&family_key("aaaa"))
}

fn application(
    label: &str,
    form: &str,
    operator: &str,
    identity: &str,
    member: Value,
    arguments: Vec<Value>,
) -> Value {
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
        assert_eq!(
            (source.code, source.cause),
            (model.code, model.cause),
            "{case}"
        );
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
        refusal.loci[0]
            .path
            .as_ref()
            .expect("path")
            .as_str()
            .rsplit('/')
            .next()
            .map(str::to_owned)
    };
    assert_eq!(tail(&refusal), tail(&oracle));
    assert_eq!(refusal.locus.is_some(), oracle.locus.is_some());
}

const FIXTURE_PACKAGE_NAME: &str = "test/families";
const MODEL_CALL: &str = "fixture/src/model-call";
const DISPATCH_CALL: &str = "fixture/src/call";

fn clause_function(owner: &Value, name: &[&str], result: &Value) -> Value {
    let label = format!("qualified:{}", name.join("/"));
    let mut node = plain(
        &label,
        "function",
        "pure_function",
        result,
        &[],
        "declaration",
        json!({"term": "aggregate", "members": []}),
    );
    node["declaration"] = json!({"qualified_name": name});
    node["owner"] = owner.clone();
    node
}

fn source_owner() -> Value {
    json!({"kind": "source", "authority": fixture_source()["authority"],
        "identity": fixture_source()["identity"]})
}

/// `Reference<target>`, keyed as the reader derives that anonymous type, and a
/// `self` parameter of that type.
fn receiver_nodes(target: &Value, label: &str) -> [Value; 2] {
    let reference_id = json!({"package": FIXTURE_PACKAGE_NAME,
        "path": ["composite_type", "reference", target], "ordinal": 0});
    let mut reference_type = plain(
        "unused",
        "composite_type",
        "reference",
        &reference_id,
        &[target],
        "type",
        json!({"term": "aggregate", "members": [reference(target)]}),
    );
    reference_type["node_id"] = reference_id.clone();
    [
        reference_type,
        plain(
            label,
            "value",
            "parameter",
            &reference_id,
            &[],
            "expression",
            json!({"term": "aggregate", "members": [
                binding("name", text("self")),
                binding("level", json!({"term": "literal", "type": integer(),
                    "value_kind": "integer", "value": "0"})),
            ]}),
        ),
    ]
}

fn dispatch_call(label: &str, receiver: &str, declaring: &Value) -> Value {
    let mut call = application(
        label,
        "call",
        "call",
        "filament.op.model.dispatch_call",
        json!({"kind": "operation", "declaration": declaring, "name": "deposit"}),
        vec![reference(&node_id(receiver))],
    );
    if label == MODEL_CALL {
        // A model call is typed by the member's result, the derived `Integer`.
        let derived = node_id("qualified:scalar_type/integer");
        call["body"]["result_type"] = derived.clone();
        call["semantic_type"] = derived;
    }
    call
}

/// The derived `Integer` node a model member result names.
fn derived_integer() -> Value {
    plain(
        "qualified:scalar_type/integer",
        "scalar_type",
        "integer",
        &node_id("qualified:scalar_type/integer"),
        &[],
        "type",
        json!({"term": "aggregate", "members": []}),
    )
}

fn candidate(declaring: &Value, key: &str, body: &Value, precondition: Value) -> Value {
    json!({
        "selected_operation": {"declaring": declaring, "operation": "deposit"},
        "selection_key": {"owner_effective_type": null,
            "original": {"package": source_identity(), "node": key}},
        "operation_body": body,
        "effective_precondition": precondition,
    })
}

fn table(application: &str, context: &Value, declaring: &Value, candidates: Vec<Value>) -> Value {
    json!({
        "application": node_id(application),
        "occurrence": {"role": "expression", "ordinal": 0},
        "operation": {"declaring": declaring, "operation": "deposit"},
        "context": context,
        "candidates": candidates,
    })
}

fn with_tables(mut package: Value, tables: Vec<Value>) -> Value {
    package["checked_term_data"] = json!({"equality_decisions": [], "protocol_locations": [],
        "dispatch_tables": tables, "probabilistic_claims": []});
    package
}

/// Adds the subtype `Sub`, the clause functions of `deposit`, a receiver and a
/// call on `Order`.
fn add_dispatch_nodes(package: &mut Value) {
    let owner = source_owner();
    let boolean = boolean();
    let sub = source_type("Sub", &["total"], &fixture_source());
    let [reference_type, receiver] = receiver_nodes(&order_id(), "fixture/src/receiver");
    nodes_mut(package).extend([
        sub,
        clause_function(&owner, &["Order", "deposit", "body"], &integer()),
        clause_function(
            &owner,
            &["Order", "deposit", "effective_precondition"],
            &boolean,
        ),
        clause_function(&owner, &["Sub", "deposit", "body"], &integer()),
        reference_type,
        receiver,
        dispatch_call(DISPATCH_CALL, "fixture/src/receiver", &order_id()),
    ]);
}

fn source_table(context: &Value, declaring: &Value) -> Value {
    let order_body = node_id("qualified:Order/deposit/body");
    let order_pre = node_id("qualified:Order/deposit/effective_precondition");
    let sub_body = node_id("qualified:Sub/deposit/body");
    let order_key = format!("ix://{}/Order", source_identity());
    let sub_key = format!("ix://{}/Sub", source_identity());
    table(
        DISPATCH_CALL,
        context,
        declaring,
        vec![
            candidate(&order_id(), &order_key, &order_body, order_pre),
            candidate(&order_id(), &sub_key, &sub_body, Value::Null),
        ],
    )
}

/// The source package with the subtype `Sub`, the clause functions of
/// `deposit`, a call on `Order` and its dispatch table over `Order` and `Sub`.
fn dispatch_package() -> Value {
    let mut package = source_package();
    add_dispatch_nodes(&mut package);
    let mut package = with_tables(package, vec![source_table(&order_id(), &order_id())]);
    settle(&mut package);
    package
}

fn dispatch_row(package: &Value) -> &Value {
    &package["checked_term_data"]["dispatch_tables"][0]
}

/// Trace: FR-366-AC-1.
#[trace("TC-475", "FR-366-AC-1")]
#[test]
fn source_dispatch_table_reads_context_candidate_order_and_clause_identities() {
    let package = dispatch_package();
    let read_package = admitted(&package);
    let occurrence =
        serde_json::from_value(dispatch_row(&package)["occurrence"].clone()).expect("occurrence");
    let view = read_package
        .dispatch_candidates(&id(DISPATCH_CALL), &occurrence, 16)
        .expect("the source table");
    assert_eq!(view.table.context, id("qualified:Order"));
    assert_eq!(view.table.operation.declaring, id("qualified:Order"));
    let keys: Vec<_> = view
        .candidates()
        .iter()
        .map(|candidate| candidate.selection_key.original.node.to_string())
        .collect();
    assert_eq!(
        keys,
        [
            format!("ix://{}/Order", source_identity()),
            format!("ix://{}/Sub", source_identity()),
        ]
    );
    assert_eq!(
        view.candidates()[0].operation_body,
        id("qualified:Order/deposit/body")
    );
    assert_eq!(
        view.candidates()[0].effective_precondition,
        Some(id("qualified:Order/deposit/effective_precondition"))
    );
    assert_eq!(
        view.candidates()[1].operation_body,
        id("qualified:Sub/deposit/body")
    );
    assert_eq!(view.candidates()[1].effective_precondition, None);
}

/// Trace: FR-366-AC-2.
#[trace("TC-475", "FR-366-AC-2")]
#[test]
fn the_three_clause_function_forms_admit_together_and_each_segment_is_an_identifier() {
    let mut package = dispatch_package();
    nodes_mut(&mut package).push(clause_function(
        &source_owner(),
        &["Order", "deposit", "precondition", "positive"],
        &boolean(),
    ));
    settle(&mut package);
    admitted(&package);
    let mut bad = package.clone();
    let at = nodes(&bad)
        .iter()
        .position(|node| node["declaration"]["qualified_name"][3] == "positive")
        .expect("authored precondition");
    nodes_mut(&mut bad)[at]["declaration"]["qualified_name"][3] = json!("not an identifier");
    settle(&mut bad);
    let refusal = refused(&bad);
    assert_eq!(refusal.code, Code::InvalidSemanticGraph);
    assert!(refusal.loci[0]
        .path
        .as_ref()
        .expect("path")
        .as_str()
        .ends_with(&format!("/{at}/declaration/qualified_name/3")));
}

/// Trace: FR-366-AC-3.
#[trace("TC-475", "FR-366-AC-3")]
#[test]
fn source_clause_functions_of_another_form_operation_or_owner_refuse() {
    let invalid = (Code::InvalidPackage, Some(Cause::InvalidValue));
    let base = "/checked_term_data/dispatch_tables/0/candidates/0";
    let cases: Vec<DispatchCase> = vec![
        (
            "body naming another declaration form",
            Box::new(|row| {
                row["candidates"][0]["operation_body"] =
                    node_id("qualified:Order/deposit/effective_precondition");
            }),
            format!("{base}/operation_body"),
        ),
        (
            "precondition naming a body",
            Box::new(|row| {
                row["candidates"][0]["effective_precondition"] =
                    node_id("qualified:Order/deposit/body");
            }),
            format!("{base}/effective_precondition"),
        ),
        (
            "body naming another operation",
            Box::new(|row| {
                row["candidates"][0]["operation_body"] = node_id("qualified:Order/withdraw/body");
            }),
            format!("{base}/operation_body"),
        ),
        (
            "body naming a non-source function",
            Box::new(|row| {
                row["candidates"][0]["operation_body"] = node_id("qualified:Example/Function");
            }),
            format!("{base}/operation_body"),
        ),
    ];
    for (case, edit, path) in cases {
        let mut package = dispatch_package();
        nodes_mut(&mut package).push(clause_function(
            &source_owner(),
            &["Order", "withdraw", "body"],
            &integer(),
        ));
        edit(&mut package["checked_term_data"]["dispatch_tables"][0]);
        settle(&mut package);
        expect(case, &package, invalid.0, invalid.1, &path);
    }
    // A clause node of another `SourceOwner`.
    let mut package = dispatch_package();
    let other = json!({"kind": "source", "authority": fixture_source()["authority"],
        "identity": "filament.fixture.other-source/v1"});
    package["lock"]["sources"]
        .as_array_mut()
        .expect("sources")
        .push(json!({"authority": other["authority"], "identity": other["identity"]}));
    let at = nodes(&package)
        .iter()
        .position(|node| node["node_id"] == node_id("qualified:Order/deposit/body"))
        .expect("body");
    nodes_mut(&mut package)[at]["owner"] = other.clone();
    settle(&mut package);
    for entry in package["source_map"].as_array_mut().expect("source map") {
        if entry["node_id"] == node_id("qualified:Order/deposit/body") {
            entry["regions"][0]["source"] =
                json!({"authority": other["authority"], "identity": other["identity"]});
        }
    }
    expect(
        "clause under another owner",
        &package,
        invalid.0,
        invalid.1,
        &format!("{base}/operation_body"),
    );
}

/// Trace: FR-366-AC-3.
#[trace("TC-475", "FR-366-AC-3")]
#[test]
fn source_candidate_selection_keys_refuse_unless_they_name_a_source_type_of_the_owner() {
    let at = "/checked_term_data/dispatch_tables/0/candidates/0/selection_key";
    let cases: Vec<KeyEdit> = vec![
        (
            "package of another identity",
            Box::new(|key| key["original"]["package"] = json!("test/families")),
        ),
        (
            "node naming no source type",
            Box::new(|key| {
                key["original"]["node"] = json!(format!("ix://{}/Missing", source_identity()))
            }),
        ),
        (
            "node naming a source node that is no type",
            Box::new(|key| {
                key["original"]["node"] = json!(format!("ix://{}/Line", source_identity()))
            }),
        ),
        (
            "bare type name instead of effective URI",
            Box::new(|key| key["original"]["node"] = json!("Order")),
        ),
        (
            "node naming a model type",
            Box::new(|key| key["original"]["node"] = json!("ix://test/families/Account")),
        ),
    ];
    for (case, edit) in cases {
        let mut package = dispatch_package();
        add_source_record(&mut package);
        edit(
            &mut package["checked_term_data"]["dispatch_tables"][0]["candidates"][0]
                ["selection_key"],
        );
        settle(&mut package);
        expect(
            case,
            &package,
            Code::InvalidPackage,
            Some(Cause::InvalidValue),
            at,
        );
    }
}

/// Trace: FR-366-AC-6.
#[trace("TC-475", "FR-366-AC-6")]
#[test]
fn the_source_object_read_refuses_a_candidate_outside_the_context_family() {
    // `Sub` redefines `deposit` for `Order`: the base table admits.
    let base = admitted(&dispatch_package());
    SelectedSourceObjects::from_stage_rows(&base, complete_rows()).expect("Sub is a subtype");

    // `Other` is a third source type with its own `deposit` body.
    let mut package = dispatch_package();
    nodes_mut(&mut package).extend([
        source_type("Other", &["total"], &fixture_source()),
        clause_function(&source_owner(), &["Other", "deposit", "body"], &integer()),
    ]);
    let other_key = format!("ix://{}/Other", source_identity());
    package["checked_term_data"]["dispatch_tables"][0]["candidates"]
        .as_array_mut()
        .expect("candidates")
        .push(candidate(
            &order_id(),
            &other_key,
            &node_id("qualified:Other/deposit/body"),
            Value::Null,
        ));
    settle(&mut package);
    let package = admitted(&package);
    let rows_with = |other_supertypes: Vec<CheckedNodeId>, sub_supertypes: Vec<CheckedNodeId>| {
        vec![
            row("Order", vec![], &["Orders"]),
            row("Sub", sub_supertypes, &[]),
            row("Other", other_supertypes, &[]),
        ]
    };
    let order = || vec![id("qualified:Order")];
    assert_eq!(
        code_of(&package, rows_with(vec![], order())),
        RowError::MismatchedIdentityWitness,
        "a type outside the family"
    );
    assert_eq!(
        code_of(&package, rows_with(order(), vec![])),
        RowError::MismatchedIdentityWitness,
        "a subtype row that does not reach the context"
    );
    // Reaching the context through a transitive supertype admits.
    SelectedSourceObjects::from_stage_rows(&package, rows_with(vec![id("qualified:Sub")], order()))
        .expect("Other reaches Order through Sub");
    // A clause function is declared under the candidate's type or a supertype.
    let with_body = |candidate: usize, body: &str| {
        let mut package = dispatch_package();
        package["checked_term_data"]["dispatch_tables"][0]["candidates"][candidate]
            ["operation_body"] = node_id(body);
        settle(&mut package);
        admitted(&package)
    };
    let inherited = with_body(1, "qualified:Order/deposit/body");
    SelectedSourceObjects::from_stage_rows(&inherited, complete_rows())
        .expect("Sub inherits Order's deposit");
    let upside_down = with_body(0, "qualified:Sub/deposit/body");
    assert_eq!(
        code_of(&upside_down, complete_rows()),
        RowError::MismatchedIdentityWitness,
        "a clause declared under a subtype of the candidate"
    );
    // The family check needs the complete row set, so missing ranks first.
    let mut rows = rows_with(vec![], order());
    rows.pop();
    assert_eq!(
        code_of(&package, rows),
        RowError::MissingIdentityWitness,
        "missing before mismatched"
    );
}

fn body_position(package: &Value) -> usize {
    nodes(package)
        .iter()
        .position(|node| node["node_id"] == node_id("qualified:Order/deposit/body"))
        .expect("Order's deposit body")
}

/// The two types of one unit share one `SourceOwner`; the anchor reads only the
/// `[<type>, <operation>, "body"]` function of its own type.
///
/// Trace: FR-368-AC-1.
#[trace("TC-475", "FR-368-AC-1")]
#[test]
fn an_anchor_reads_only_the_body_function_of_its_own_type() {
    let mut package = callable_package();
    // `Sub` also carries a `deposit` body, ahead of `Order`'s in graph order and
    // without a body expression, so reading it would refuse.
    let at = body_position(&package);
    let sub_body = clause_function(&source_owner(), &["Sub", "deposit", "body"], &boolean());
    let sub = source_type("Sub", &["total"], &fixture_source());
    nodes_mut(&mut package).splice(at..at, [sub, sub_body]);
    settle(&mut package);
    let package = admitted(&package);
    let anchor = anchor_of(&package);
    let rows = SelectedSourceOperations::from_stage_rows(
        &package,
        [operation_row(&package, "Order::deposit")],
        64,
    )
    .expect("the valid row");
    let read = package
        .source_operation_callable(&rows, &anchor, 256)
        .expect("Order's own body");
    assert_eq!(
        read.callable().expect("callable").body,
        id(&family_key("dddd"))
    );

    // With only `Sub`'s body present the `Order` anchor has none, and the
    // refusal reports every unit charged.
    let mut only_sub = callable_package();
    let at = body_position(&only_sub);
    nodes_mut(&mut only_sub)[at] =
        clause_function(&source_owner(), &["Sub", "deposit", "body"], &boolean());
    nodes_mut(&mut only_sub).push(source_type("Sub", &["total"], &fixture_source()));
    settle(&mut only_sub);
    let only_sub = admitted(&only_sub);
    let rows = SelectedSourceOperations::from_stage_rows(
        &only_sub,
        [operation_row(&only_sub, "Order::deposit")],
        64,
    )
    .expect("the valid row");
    let error = only_sub
        .source_operation_callable(&rows, &anchor_of(&only_sub), 4096)
        .expect_err("no Order body");
    assert_eq!(
        error.code,
        CheckedCallableReadErrorCode::MissingSelectedJoin
    );
    assert_eq!(
        error.work,
        1 + only_sub.graph().nodes.len() as u64,
        "the anchor plus one unit per scanned node"
    );
}

/// The least `work` limit under which the package reads.
fn least_work(package: &Value) -> u64 {
    let package = indexed(package);
    let bytes = canonical(&package);
    let evidence = evidence_of(&package);
    let admits = |work: u64| {
        let limits = CheckedPackageReadLimits {
            work,
            ..CheckedPackageReadLimits::bounded()
        };
        matches!(
            CheckedPackageV2::read(&bytes, limits, &evidence),
            CheckedPackageV2ReadResult::Admitted(_)
        )
    };
    let (mut low, mut high) = (0, 1 << 20);
    assert!(admits(high), "the package must admit");
    while low < high {
        let mid = (low + high) / 2;
        if admits(mid) {
            high = mid;
        } else {
            low = mid + 1;
        }
    }
    low
}

/// Each source candidate adds the same work whatever the graph size: the
/// `selection_key` lookup is indexed, not a scan of the graph.
///
/// Trace: FR-366-AC-3.
#[trace("TC-475", "FR-366-AC-3")]
#[test]
fn a_source_candidate_adds_work_independent_of_the_graph_size() {
    let candidate_cost = |padding: usize| {
        let mut one = dispatch_package();
        for at in 0..padding {
            nodes_mut(&mut one).push(source_type(
                &format!("Pad{at}"),
                &["total"],
                &fixture_source(),
            ));
        }
        let mut two = one.clone();
        one["checked_term_data"]["dispatch_tables"][0]["candidates"]
            .as_array_mut()
            .expect("candidates")
            .pop();
        settle(&mut one);
        settle(&mut two);
        least_work(&two) - least_work(&one)
    };
    assert_eq!(candidate_cost(0), candidate_cost(60));
}

/// Trace: FR-366-AC-4.
#[trace("TC-475", "FR-366-AC-4")]
#[test]
fn dispatch_rows_mixing_a_source_and_a_model_family_refuse() {
    let account = node_id("fixture/1515");
    let operation = "/checked_term_data/dispatch_tables/0/operation";
    // A source context with a model-owned declaring node.
    let mut source_over_model = mixed_package();
    add_dispatch_nodes(&mut source_over_model);
    let mut package = with_tables(source_over_model, vec![source_table(&order_id(), &account)]);
    settle(&mut package);
    expect(
        "source context, model declaring",
        &package,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        operation,
    );
    // A model context with a source declaring node.
    let mut model_over_source = mixed_package();
    add_dispatch_nodes(&mut model_over_source);
    let [reference_type, receiver] = receiver_nodes(&account, "fixture/src/model-receiver");
    nodes_mut(&mut model_over_source).extend([
        reference_type,
        receiver,
        dispatch_call(MODEL_CALL, "fixture/src/model-receiver", &account),
        derived_integer(),
    ]);
    let mut row = source_table(&account, &order_id());
    row["application"] = node_id(MODEL_CALL);
    let mut package = with_tables(model_over_source, vec![row]);
    settle(&mut package);
    expect(
        "model context, source declaring",
        &package,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        operation,
    );
}

/// Trace: FR-366-AC-5.
#[trace("TC-475", "FR-366-AC-5")]
#[test]
fn a_model_dispatch_table_reads_unchanged_beside_a_source_table() {
    let account = node_id("fixture/1515");
    let mut package = mixed_package();
    add_dispatch_nodes(&mut package);
    let [reference_type, receiver] = receiver_nodes(&account, "fixture/src/model-receiver");
    nodes_mut(&mut package).extend([
        reference_type,
        receiver,
        dispatch_call(MODEL_CALL, "fixture/src/model-receiver", &account),
        derived_integer(),
    ]);
    let model_candidate = json!({
        "selected_operation": {"declaring": account, "operation": "deposit"},
        "selection_key": {"owner_effective_type": null,
            "original": {"package": "test/families", "node": "ix://test/families/Account"}},
        "operation_body": node_id("qualified:Example/Function"),
        "effective_precondition": null,
    });
    let model_table = table(MODEL_CALL, &account, &account, vec![model_candidate]);
    let source_only = with_tables(
        package.clone(),
        vec![source_table(&order_id(), &order_id())],
    );
    let both = with_tables(
        package.clone(),
        vec![model_table.clone(), source_table(&order_id(), &order_id())],
    );
    let model_only = with_tables(package, vec![model_table]);
    let read_model = |package: &Value| {
        let mut package = package.clone();
        settle(&mut package);
        let admitted_package = admitted(&package);
        let occurrence = serde_json::from_value(json!({"role": "expression", "ordinal": 0}))
            .expect("occurrence");
        let table = admitted_package
            .dispatch_candidates(&id(MODEL_CALL), &occurrence, 16)
            .expect("the model table")
            .table
            .clone();
        serde_json::to_value(table).expect("table")
    };
    let mut sorted = both.clone();
    sorted["checked_term_data"]["dispatch_tables"]
        .as_array_mut()
        .expect("tables")
        .sort_by_key(|row| serde_json::to_string(&row["application"]).expect("id"));
    assert_eq!(read_model(&sorted), read_model(&model_only));
    admitted_settled(&source_only);
}

fn admitted_settled(package: &Value) -> CheckedPackageV2 {
    let mut package = package.clone();
    settle(&mut package);
    admitted(&package)
}

/// A `self.<field>` read of `declaring`'s field through a `deref` of a
/// reference-typed value, typed `result`.
fn field_read(
    label: &str,
    reference_type: &Value,
    declaring: &Value,
    field: &str,
    result: &Value,
) -> [Value; 2] {
    let deref_label = format!("{label}-deref");
    let deref = plain(
        &deref_label,
        "expression",
        "deref",
        declaring,
        &[reference_type],
        "expression",
        json!({"term": "application", "operator": "deref",
            "operation": {"identity": "filament.op.model.deref", "laws": [],
                "mode": null, "member": null, "leaves": []},
            "result_type": declaring,
            "arguments": [reference(reference_type)]}),
    );
    let read = plain(
        label,
        "expression",
        "query",
        result,
        &[declaring, &node_id(&deref_label)],
        "expression",
        json!({"term": "application", "operator": "query",
            "operation": {"identity": "filament.op.record.project", "laws": [],
                "mode": null,
                "member": {"kind": "field", "declaration": declaring, "name": field,
                    "read_form": "bare"},
                "leaves": []},
            "result_type": result,
            "arguments": [reference(&node_id(&deref_label))]}),
    );
    [deref, read]
}

fn reference_type_of(target: &Value) -> Value {
    json!({"package": FIXTURE_PACKAGE_NAME,
        "path": ["composite_type", "reference", target], "ordinal": 0})
}

/// The source package with the field-less source type `Empty` and a read of
/// `field` on `declaring`, typed `result`.
fn read_package(declaring: &str, field: &str, result: &Value) -> Value {
    let mut package = source_package();
    let sub = source_type("Sub", &["total"], &fixture_source());
    let empty = source_type("Empty", &[], &fixture_source());
    let declaring = node_id(&format!("qualified:{declaring}"));
    let [reference_type, _receiver] = receiver_nodes(&declaring, "fixture/src/read-receiver");
    let reference_id = reference_type_of(&declaring);
    nodes_mut(&mut package).extend([sub, empty, reference_type]);
    let [deref, read] = field_read("fixture/src/read", &reference_id, &declaring, field, result);
    nodes_mut(&mut package).extend([deref, read]);
    settle(&mut package);
    package
}

/// Trace: FR-367-AC-1.
#[trace("TC-475", "FR-367-AC-1")]
#[test]
fn a_field_read_resolves_to_the_type_of_its_binding_inherited_fields_included() {
    for declaring in ["Order", "Sub"] {
        admitted(&read_package(declaring, "total", &integer()));
        // The read's type is the binding's type.
        let refusal = refused(&read_package(declaring, "total", &boolean()));
        assert_eq!(refusal.code, Code::IllTyped, "{declaring}");
        assert_eq!(
            refusal.cause,
            Some(Cause::OperatorIneligible),
            "{declaring}"
        );
        assert!(refusal.loci[0]
            .path
            .as_ref()
            .expect("path")
            .as_str()
            .ends_with("/body/result_type"));
    }
}

/// Trace: FR-367-AC-2.
#[trace("TC-475", "FR-367-AC-2")]
#[test]
fn an_unbound_field_and_any_read_on_an_empty_source_type_refuse() {
    for (declaring, field) in [("Order", "absent"), ("Empty", "total")] {
        let package = read_package(declaring, field, &integer());
        let read = position(&package, "expression", "query");
        expect(
            declaring,
            &package,
            Code::IllTyped,
            Some(Cause::OperatorIneligible),
            &format!("/semantic_graph/nodes/{read}/body/operation/member/name"),
        );
    }
}

/// Trace: FR-367-AC-6.
#[trace("TC-475", "FR-367-AC-6")]
#[test]
fn a_model_field_read_is_unchanged_beside_a_source_type() {
    let account = node_id("fixture/1515");
    let build = |mixed: bool, field: &str, result: &Value| {
        let mut package = if mixed {
            mixed_package()
        } else {
            v2_all_families()
        };
        let [reference_type, _receiver] =
            receiver_nodes(&account, "fixture/src/model-read-receiver");
        let reference_id = reference_type_of(&account);
        nodes_mut(&mut package).extend([reference_type, derived_integer()]);
        let [deref, read] = field_read(
            "fixture/src/model-read",
            &reference_id,
            &account,
            field,
            result,
        );
        nodes_mut(&mut package).extend([deref, read]);
        settle(&mut package);
        package
    };
    let derived = node_id("qualified:scalar_type/integer");
    for mixed in [false, true] {
        admitted(&build(mixed, "balance", &derived));
    }
    let outcome = |mixed: bool, field: &str, result: &Value| {
        let refusal = refused(&build(mixed, field, result));
        let tail = refusal.loci[0]
            .path
            .as_ref()
            .expect("path")
            .as_str()
            .to_owned();
        let tail = tail
            .split_once("/nodes/")
            .map(|(_, rest)| rest.split_once('/').map(|(_, t)| t.to_owned()));
        (refusal.code, refusal.cause, tail)
    };
    for (field, result) in [("absent", derived.clone()), ("balance", boolean())] {
        assert_eq!(
            outcome(true, field, &result),
            outcome(false, field, &result),
            "{field}"
        );
    }
}

/// An admitted package holding the source types `Order` and `Sub`.
fn two_types() -> CheckedPackageV2 {
    let mut package = source_package();
    nodes_mut(&mut package).push(source_type("Sub", &["total"], &fixture_source()));
    settle(&mut package);
    admitted(&package)
}

fn row(
    name: &str,
    supertypes: Vec<CheckedNodeId>,
    populations: &[&str],
) -> SelectedSourceObjectRow {
    SelectedSourceObjectRow {
        structural_node: id(&format!("qualified:{name}")),
        source_authority: fixture_source()["authority"]
            .as_str()
            .expect("authority")
            .into(),
        source_identity: name.into(),
        is_abstract: false,
        supertypes,
        identity_fields: vec!["total".into()],
        members: vec![SelectedSourceObjectMember {
            name: "total".into(),
            type_node: id(&family_key("a3a3")),
            optional: false,
        }],
        populations: populations
            .iter()
            .map(|name| SelectedSourcePopulation {
                name: (*name).into(),
                symmetric: false,
            })
            .collect(),
    }
}

fn complete_rows() -> Vec<SelectedSourceObjectRow> {
    vec![
        row("Order", vec![], &["Orders"]),
        row("Sub", vec![id("qualified:Order")], &[]),
    ]
}

fn code_of(package: &CheckedPackageV2, rows: Vec<SelectedSourceObjectRow>) -> RowError {
    SelectedSourceObjects::from_stage_rows(package, rows)
        .expect_err("the supply must be refused")
        .code
}

/// Trace: FR-367-AC-5.
#[trace("TC-475", "FR-367-AC-5")]
#[test]
fn a_complete_row_set_exposes_every_source_type_through_a_typed_read() {
    let package = two_types();
    let rows = SelectedSourceObjects::from_stage_rows(&package, complete_rows())
        .expect("the complete supply");
    assert_eq!(rows.rows().len(), 2);
    let order = rows
        .read(&package, &id("qualified:Order"))
        .expect("Order row");
    assert!(!order.is_abstract);
    assert!(order.supertypes.is_empty());
    assert_eq!(order.identity_fields, ["total".into()] as [Box<str>; 1]);
    assert_eq!(order.members.len(), 1);
    assert_eq!(order.populations[0].name.as_ref(), "Orders");
    let sub = rows.read(&package, &id("qualified:Sub")).expect("Sub row");
    assert_eq!(sub.supertypes, [id("qualified:Order")]);
    assert!(rows.read(&package, &id(&family_key("a3a3"))).is_none());
}

/// Two `SourceOwner`s of one authority cannot each declare a type of one name,
/// so the (authority, `source_identity`) row key never collides across owners.
///
/// Trace: FR-367-AC-3.
#[trace("TC-475", "FR-367-AC-3")]
#[test]
fn a_same_named_type_under_a_second_owner_of_one_authority_refuses() {
    let mut package = source_package();
    let other = json!({"authority": fixture_source()["authority"],
        "identity": "filament.fixture.other-source/v1"});
    package["lock"]["sources"]
        .as_array_mut()
        .expect("sources")
        .push(other.clone());
    let mut second = source_type("Order", &["total"], &fixture_source());
    second["owner"] = json!({"kind": "source", "authority": other["authority"],
        "identity": other["identity"]});
    nodes_mut(&mut package).push(second);
    settle(&mut package);
    let at = nodes(&package).len() - 1;
    expect(
        "second owner, same type name",
        &package,
        Code::InvalidSemanticGraph,
        None,
        &format!("/semantic_graph/nodes/{at}/node_id"),
    );
}

/// Trace: FR-367-AC-3.
#[trace("TC-475", "FR-367-AC-3")]
#[test]
fn a_supply_reports_the_first_reason_in_foreign_unknown_conflicting_missing_order() {
    let package = two_types();
    let foreign = || {
        let mut row = row("Order", vec![], &[]);
        row.source_authority = "foreign".into();
        row
    };
    let unknown = || {
        let mut row = row("Sub", vec![id("qualified:Order")], &[]);
        row.structural_node = id(&family_key("a3a3"));
        row
    };
    let [order, sub] = <[_; 2]>::try_from(complete_rows()).expect("two rows");
    // Foreign comes first, whatever the order of the rows.
    for rows in [vec![foreign(), unknown()], vec![unknown(), foreign()]] {
        assert_eq!(code_of(&package, rows), RowError::ForeignIdentityWitness);
    }
    // Unknown outranks a duplicate-key pair.
    assert_eq!(
        code_of(&package, vec![unknown(), order.clone(), order.clone()]),
        RowError::UnknownIdentityWitness
    );
    // A duplicate-key pair outranks an omitted type.
    assert_eq!(
        code_of(&package, vec![order.clone(), order.clone()]),
        RowError::ConflictingIdentityWitness
    );
    // An omitted type is missing.
    assert_eq!(
        code_of(&package, vec![order.clone()]),
        RowError::MissingIdentityWitness
    );
    assert_eq!(code_of(&package, vec![]), RowError::MissingIdentityWitness);
    let _ = sub;
}

/// Trace: FR-367-AC-4.
#[trace("TC-475", "FR-367-AC-4")]
#[test]
fn rows_whose_members_or_supertypes_the_package_does_not_support_are_unknown() {
    let package = two_types();
    let [order, sub] = <[_; 2]>::try_from(complete_rows()).expect("two rows");
    let edits: Vec<RowEdit> = vec![
        ("name", Box::new(|row| row.members[0].name = "other".into())),
        (
            "type node",
            Box::new(|row| row.members[0].type_node = id(&family_key("aaaa"))),
        ),
        ("optional", Box::new(|row| row.members[0].optional = true)),
        ("missing member", Box::new(|row| row.members.clear())),
        (
            "extra member",
            Box::new(|row| {
                let member = row.members[0].clone();
                row.members.push(member);
            }),
        ),
        (
            "non-source supertype",
            Box::new(|row| row.supertypes = vec![id(&family_key("a3a3"))]),
        ),
        (
            "own supertype",
            Box::new(|row| row.supertypes = vec![row.structural_node.clone()]),
        ),
        (
            "identity",
            Box::new(|row| row.source_identity = "Other".into()),
        ),
    ];
    for (case, edit) in &edits {
        let mut edited = order.clone();
        edit(&mut edited);
        assert_eq!(
            code_of(&package, vec![edited, sub.clone()]),
            RowError::UnknownIdentityWitness,
            "{case}"
        );
    }
    // Members in another order.
    let mut package_two_fields = source_package();
    let mut pair = source_type("Order", &["total", "count"], &fixture_source());
    pair["declaration"] = json!({"qualified_name": ["Pair"]});
    pair["node_id"] = node_id("qualified:Pair");
    pair["semantic_type"] = node_id("qualified:Pair");
    nodes_mut(&mut package_two_fields).push(pair);
    settle(&mut package_two_fields);
    let two_fields = admitted(&package_two_fields);
    let member = |name: &str| SelectedSourceObjectMember {
        name: name.into(),
        type_node: id(&family_key("a3a3")),
        optional: false,
    };
    let mut pair_row = row("Pair", vec![], &[]);
    pair_row.members = vec![member("total"), member("count")];
    let order_row = row("Order", vec![], &[]);
    SelectedSourceObjects::from_stage_rows(&two_fields, [order_row.clone(), pair_row.clone()])
        .expect("members in body order");
    pair_row.members = vec![member("count"), member("total")];
    assert_eq!(
        code_of(&two_fields, vec![order_row, pair_row]),
        RowError::UnknownIdentityWitness,
        "order"
    );
    // A cycle.
    let mut cyclic_order = order;
    cyclic_order.supertypes = vec![id("qualified:Sub")];
    assert_eq!(
        code_of(&package, vec![cyclic_order, sub]),
        RowError::UnknownIdentityWitness,
        "cycle"
    );
}

/// The source package with the `[Order, deposit, body]` clause function a
/// callable read joins: its `self` parameter and a body expression.
fn callable_package() -> Value {
    let mut package = source_package();
    let [reference_type, receiver] = receiver_nodes(&order_id(), "fixture/src/body-receiver");
    let mut function = clause_function(&source_owner(), &["Order", "deposit", "body"], &boolean());
    function["body"] = json!({"term": "aggregate", "members": [
        binding("parameters", json!({"term": "aggregate",
            "members": [reference(&node_id("fixture/src/body-receiver"))]})),
        binding("body", reference(&node_id(&family_key("dddd")))),
    ]});
    function["dependencies"] = json!([
        node_id("fixture/src/body-receiver"),
        node_id(&family_key("dddd"))
    ]);
    nodes_mut(&mut package).extend([reference_type, receiver, function]);
    settle(&mut package);
    package
}

fn anchor_of(package: &CheckedPackageV2) -> CheckedNodeId {
    package
        .graph()
        .nodes
        .iter()
        .find(|node| node.semantic_form.as_ref() == "operation_anchor")
        .expect("an operation anchor")
        .node_id
        .clone()
}

fn operation_row(package: &CheckedPackageV2, identity: &str) -> SelectedSourceOperationRow {
    let anchor = anchor_of(package);
    SelectedSourceOperationRow {
        source_authority: fixture_source()["authority"]
            .as_str()
            .expect("authority")
            .into(),
        source_identity: identity.into(),
        anchor_id: anchor.clone(),
        structural_node: anchor,
    }
}

fn operation_code(
    package: &CheckedPackageV2,
    rows: Vec<SelectedSourceOperationRow>,
) -> OperationError {
    SelectedSourceOperations::from_stage_rows(package, rows, 64)
        .expect_err("the rows must be refused")
        .code
}

/// Trace: FR-368-AC-1, FR-368-AC-2.
#[trace("TC-475", "FR-368-AC-1", "FR-368-AC-2")]
#[test]
fn a_source_anchor_reads_its_joined_row_and_is_unavailable_without_one() {
    let package = admitted(&callable_package());
    let anchor = anchor_of(&package);
    let rows = SelectedSourceOperations::from_stage_rows(
        &package,
        [operation_row(&package, "Order::deposit")],
        64,
    )
    .expect("the valid row");
    let read = package
        .source_operation_callable(&rows, &anchor, 256)
        .expect("the joined callable");
    let callable = read.callable().expect("a callable");
    assert_eq!(
        callable.identity,
        CheckedCallableIdentity::SourceOperation {
            anchor: anchor.clone(),
            source_authority: fixture_source()["authority"]
                .as_str()
                .expect("authority")
                .into(),
            source_identity: "Order::deposit".into(),
        }
    );
    assert_eq!(callable.context, Some(id("qualified:Order")));
    assert_eq!(callable.frame, Some(id("fixture/src/frame")));
    assert_eq!(callable.parameters, [id("fixture/src/body-receiver")]);
    assert_eq!(callable.body, id(&family_key("dddd")));

    // Without a row the source anchor stays unavailable.
    let empty = SelectedSourceOperations::from_stage_rows(&package, [], 0).expect("empty");
    assert_eq!(
        package
            .source_operation_callable(&empty, &anchor, 256)
            .expect_err("no row")
            .code,
        CheckedCallableReadErrorCode::UnavailableWitness
    );
    let none = SelectedOperationMembers::from_stage_rows(&package, [], 0).expect("empty");
    assert_eq!(
        package
            .callable_table(&none, 256)
            .expect_err("no model row")
            .code,
        CheckedCallableReadErrorCode::UnavailableWitness
    );
}

/// Trace: FR-368-AC-3.
#[trace("TC-475", "FR-368-AC-3")]
#[test]
fn a_row_that_does_not_name_the_anchor_its_type_its_operation_or_authority_is_refused() {
    let package = admitted(&callable_package());
    let valid = operation_row(&package, "Order::deposit");
    let edits: Vec<OperationEdit> = vec![
        (
            "foreign anchor",
            Box::new(|row| {
                let mut foreign = row.anchor_id.clone();
                foreign.package = "foreign/package".into();
                row.anchor_id = foreign.clone();
                row.structural_node = foreign;
            }),
            OperationError::UnknownAnchor,
        ),
        (
            "dangling anchor",
            Box::new(|row| {
                row.anchor_id = id("qualified:Absent");
                row.structural_node = id("qualified:Absent");
            }),
            OperationError::UnknownAnchor,
        ),
        (
            "other type",
            Box::new(|row| row.source_identity = "Sub::deposit".into()),
            OperationError::MismatchedWitness,
        ),
        (
            "other operation",
            Box::new(|row| row.source_identity = "Order::withdraw".into()),
            OperationError::MismatchedWitness,
        ),
        (
            "wrong authority",
            Box::new(|row| row.source_authority = "elsewhere".into()),
            OperationError::MismatchedWitness,
        ),
        (
            "structural node other than the anchor",
            Box::new(|row| row.structural_node = id("qualified:Order")),
            OperationError::MismatchedWitness,
        ),
        (
            "a node that is no anchor",
            Box::new(|row| {
                row.anchor_id = id("qualified:Order");
                row.structural_node = id("qualified:Order");
            }),
            OperationError::WrongAnchorForm,
        ),
    ];
    for (case, edit, expected) in &edits {
        let mut row = valid.clone();
        edit(&mut row);
        assert_eq!(operation_code(&package, vec![row]), *expected, "{case}");
    }
    assert_eq!(
        operation_code(&package, vec![valid.clone(), valid]),
        OperationError::ConflictingWitness
    );
}

/// The mixed package with a model anchor over the selected `Account`.
fn model_anchor_package() -> Value {
    let mut package = mixed_package();
    let frame = nodes(&package)[position(&package, "state", "frame")]["node_id"].clone();
    let account = node_id("fixture/1515");
    nodes_mut(&mut package).push(plain(
        "fixture/src/model-anchor",
        "state",
        "operation_anchor",
        &account,
        &[&account, &frame],
        "anchor",
        json!({"term": "aggregate", "members": [
            binding("context", reference(&account)),
            binding("operation", text("deposit")),
            binding("frame", reference(&frame)),
        ]}),
    ));
    settle(&mut package);
    package
}

fn node_anchor(package: &CheckedPackageV2, label: &str) -> CheckedNodeId {
    let _ = package;
    id(label)
}

fn model_row(anchor: CheckedNodeId) -> SelectedOperationMemberRow {
    use filament_exact::{EffectiveId, OriginalDeclarationKey};
    SelectedOperationMemberRow {
        anchor,
        selected_document: "test/families".into(),
        declaring_form: ModelForm::ObjectType,
        original_model_identity: "ix://test/families/Account".into(),
        original_operation_identity: "ix://test/families/Account/deposit".into(),
        selected_operation_identity: SelectedOperationIdentity::new(
            EffectiveId::new(
                None,
                OriginalDeclarationKey::new("test/families", "ix://test/families/Account"),
            ),
            "deposit",
        ),
    }
}

/// Trace: FR-368-AC-4, FR-368-AC-5.
#[trace("TC-475", "FR-368-AC-4", "FR-368-AC-5")]
#[test]
fn source_and_model_rows_do_not_cross_and_model_rows_read_unchanged() {
    let package = admitted(&model_anchor_package());
    let model_anchor = node_anchor(&package, "fixture/src/model-anchor");
    // A source row for a model anchor.
    let mut row = operation_row(&package, "Account::deposit");
    row.anchor_id = model_anchor.clone();
    row.structural_node = model_anchor.clone();
    assert_eq!(
        operation_code(&package, vec![row]),
        OperationError::UnavailableWitness
    );
    // A model row for a source anchor.
    let source = admitted(&callable_package());
    let refused_row =
        SelectedOperationMembers::from_stage_rows(&source, [model_row(anchor_of(&source))], 16)
            .expect_err("a source anchor has no model member");
    assert_eq!(refused_row.code, OperationError::UnavailableWitness);
    // The model row for the model anchor still reads beside the source type.
    let rows =
        SelectedOperationMembers::from_stage_rows(&package, [model_row(model_anchor.clone())], 16)
            .expect("the model row joins");
    let read = rows.read(&package, &model_anchor).expect("the model row");
    assert_eq!(
        read.original_operation_identity.as_ref(),
        "ix://test/families/Account/deposit"
    );
}

/// Trace: FR-368-AC-6.
#[trace("TC-475", "FR-368-AC-6")]
#[test]
fn a_row_needs_an_effects_row_only_when_the_carrier_is_present() {
    // Present, with a row for `quote` only: `deposit` has none.
    let mut with_carrier = callable_package();
    with_carrier["operation_effects"] = json!([effect_row(
        &order_id(),
        &source_identity(),
        "quote",
        SOURCE_FRAME_ROW
    )]);
    let package = admitted(&with_carrier);
    assert_eq!(
        operation_code(&package, vec![operation_row(&package, "Order::deposit")]),
        OperationError::MismatchedWitness
    );
    // Present, with the row for `deposit`: it joins.
    let mut named = callable_package();
    named["operation_effects"] = source_rows();
    let package = admitted(&named);
    SelectedSourceOperations::from_stage_rows(
        &package,
        [operation_row(&package, "Order::deposit")],
        64,
    )
    .expect("the carrier names the operation");
    // Absent: the row joins and no effect is read.
    let package = admitted(&callable_package());
    assert!(package.operation_effects().is_none());
    SelectedSourceOperations::from_stage_rows(
        &package,
        [operation_row(&package, "Order::deposit")],
        64,
    )
    .expect("no carrier, no comparison");
}
