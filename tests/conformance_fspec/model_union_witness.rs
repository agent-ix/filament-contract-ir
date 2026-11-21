// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FSpec FR-830 / TC-2112: the published model-union and model-tuple witness
//! vectors, executed through the production strict read and the native join.
//!
//! The vectors and their complete input packages are read in place from the
//! checkout `FILAMENT_SPECIFICATION_DIR` names; nothing of them is copied.
//! The rows below are built from the vectors' JSON by a test-only adapter and
//! are consistency-only claims: they are not genuine producer emission, and
//! IR claims no provenance from them. The sidecar's closed-format, depth and
//! ordering vectors (`schema_refusals`) belong to the FSL identity-file parser
//! and are not executed here.

use super::*;
use filament_contract_model::{
    with_native_budget_observer, ExactTypeProjectionError, ModelUnionScope, NativeBudgetEvent,
    NativeBudgetResource, NativeJoinBudget, NativeJoinChargeKind, NativeJoinLocus,
    NativeJoinLocusSource, SelectedExactDeclarationForm, SelectedExactDeclarationRow,
    SelectedExactIdentities, SelectedExactIdentity, SelectedModelEnumDeclarationRow,
    SelectedModelEnumMemberRow, SelectedModelEnumRows, SelectedModelEnumShapeCaseRow,
    SelectedModelEnumShapeRow, SelectedModelUnionCaseRow, SelectedModelUnionPayload,
    SelectedModelUnionRow, SelectedSourceTupleRow, SelectedSourceUnionRow,
    SelectedTupleDeclarationRow, SelectedTuplePositionRow, SelectedUnionMemberRow,
};
use filament_exact::{
    EffectiveId, NodeKey, OriginalDeclarationKey, PathSegment, UnionMember, UnitId, ValueType,
};
use ix_trace_rs::trace;
use std::cell::RefCell;

#[path = "../it/support/fspec_tuple_inputs.rs"]
mod inputs;
use serde_json::Value;

fn key(value: &Value) -> NodeKey {
    let path = value["path"]
        .as_array()
        .unwrap_or_else(|| panic!("key path: {value}"))
        .iter()
        .map(|segment| match segment {
            Value::String(text) => PathSegment::Text(text.clone()),
            other => PathSegment::Node(key(other)),
        })
        .collect();
    NodeKey::new(
        value["package"].as_str().expect("key package"),
        path,
        value["ordinal"].as_u64().expect("key ordinal"),
    )
    .expect("a vector key")
}

fn node_id(value: &Value) -> CheckedNodeId {
    serde_json::from_value(value.clone()).expect("a vector node reference")
}

fn effective(value: &Value) -> EffectiveId {
    let mut id = None;
    for link in value["chain"].as_array().expect("effective chain") {
        id = Some(EffectiveId::new(
            id,
            OriginalDeclarationKey::new(
                link["package"].as_str().expect("link package"),
                link["node"].as_str().expect("link node"),
            ),
        ));
    }
    id.expect("a non-empty effective chain")
}

fn original_type(value: &Value) -> ValueType {
    match value["kind"].as_str().expect("original type kind") {
        "boolean" => ValueType::Boolean,
        "integer" => ValueType::Integer,
        "string" => ValueType::String,
        "uuid" => ValueType::Uuid,
        "timestamp" => ValueType::Timestamp,
        "bytes" => ValueType::Bytes,
        "json_object" => ValueType::JsonObject,
        "float" => ValueType::Float(filament_exact::FloatType::exact(
            match value["width"].as_u64() {
                Some(32) => filament_exact::IeeeWidth::Binary32,
                _ => filament_exact::IeeeWidth::Binary64,
            },
        )),
        "composite" => ValueType::Composite(key(&value["node"])),
        "quantity" => ValueType::Quantity(match value["unit"]["kind"].as_str() {
            Some("compound") => UnitId::Compound(
                value["unit"]["terms"]
                    .as_array()
                    .expect("compound terms")
                    .iter()
                    .map(|term| {
                        let exponent: i64 = term["exponent"]
                            .as_str()
                            .expect("exponent")
                            .parse()
                            .expect("integer");
                        (key(&term["node"]), filament_exact::Integer::from(exponent))
                    })
                    .collect(),
            ),
            _ => UnitId::Declared(key(&value["unit"]["node"])),
        }),
        "reference" => ValueType::Reference(effective(&value["effective"])),
        "option" => ValueType::Option(std::sync::Arc::new(original_type(&value["element"]))),
        other => panic!("{other}: outside this test adapter's original-type forms"),
    }
}

fn union_row(value: &Value) -> SelectedModelUnionRow {
    let settled = key(&value["settled_key"]);
    let document = value["selected_document"].as_str().expect("document");
    let cases = value["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .map(|case| {
            let name = case["name"].as_str().expect("case name");
            let payload = match case["payload"]["kind"].as_str().expect("payload kind") {
                "none" => SelectedModelUnionPayload::None,
                "clock" => SelectedModelUnionPayload::Clock {
                    payload_type: case["payload"]["payload_type"]
                        .as_str()
                        .expect("clock identity")
                        .into(),
                },
                _ => SelectedModelUnionPayload::Data {
                    node: node_id(&case["payload"]["node"]),
                    original_type: original_type(&case["payload"]["original_type"]),
                },
            };
            SelectedModelUnionCaseRow {
                rank: case["rank"].as_u64().expect("rank"),
                position: case["position"].as_u64().expect("position"),
                name: name.into(),
                original_case: OriginalDeclarationKey::new(
                    case["original_case"]["package"].as_str().expect("package"),
                    case["original_case"]["node"].as_str().expect("node"),
                ),
                member: UnionMember::from_admitted(
                    key(&case["member"]["declaration"]),
                    case["member"]["identifier"]
                        .as_str()
                        .expect("member identifier"),
                )
                .expect("a member"),
                payload,
            }
        })
        .collect();
    SelectedModelUnionRow {
        structural_node: node_id(&value["structural_node"]),
        selected_document: document.into(),
        type_handle: key(&value["type_handle"]),
        settled_key: settled,
        cases,
    }
}

fn tuple_row(value: &Value) -> SelectedTupleDeclarationRow {
    SelectedTupleDeclarationRow::new(
        node_id(&value["structural_node"]),
        value["selected_document"].as_str().expect("document"),
        value["original_identity"].as_str().expect("identity"),
        key(&value["node_key"]),
    )
}

fn declaration_row(value: &Value) -> SelectedExactDeclarationRow {
    let exact = &value["exact_identity"];
    let (form, identity) = match exact["kind"].as_str().expect("identity kind") {
        "record" => (
            SelectedExactDeclarationForm::RecordValueType,
            SelectedExactIdentity::Record(key(&exact["value"])),
        ),
        "object" => (
            SelectedExactDeclarationForm::ObjectType,
            SelectedExactIdentity::Object(effective(&exact["value"])),
        ),
        "declared_unit" => (
            SelectedExactDeclarationForm::DeclaredUnit,
            SelectedExactIdentity::DeclaredUnit(UnitId::Declared(key(&exact["value"]["node"]))),
        ),
        other => panic!("{other}: outside this test adapter's declaration forms"),
    };
    SelectedExactDeclarationRow::new(
        node_id(&value["structural_node"]),
        value["selected_document"].as_str().expect("document"),
        form,
        value["original_identity"].as_str().expect("identity"),
        identity,
    )
}

/// RFC 6902 `replace`, `add` and `remove` over the sidecar object.
fn patch(target: &mut Value, operations: &Value) {
    for operation in operations.as_array().expect("patch operations") {
        let path = operation["path"].as_str().expect("patch path");
        let (parent, member) = path.rsplit_once('/').expect("patch member");
        let parent = target
            .pointer_mut(parent)
            .unwrap_or_else(|| panic!("patch parent of {path}"));
        match (operation["op"].as_str().expect("op"), parent) {
            ("replace", Value::Object(map)) => {
                *map.get_mut(member).expect("replace target") = operation["value"].clone();
            }
            ("replace", Value::Array(items)) => {
                items[member.parse::<usize>().expect("index")] = operation["value"].clone();
            }
            ("add", Value::Object(map)) => {
                map.insert(member.into(), operation["value"].clone());
            }
            ("add", Value::Array(items)) if member == "-" => {
                items.push(operation["value"].clone());
            }
            ("add", Value::Array(items)) => {
                items.insert(
                    member.parse::<usize>().expect("index"),
                    operation["value"].clone(),
                );
            }
            ("remove", Value::Object(map)) => {
                map.remove(member);
            }
            ("remove", Value::Array(items)) => {
                items.remove(member.parse::<usize>().expect("index"));
            }
            (other, _) => panic!("{other}: unsupported patch operation"),
        }
    }
}

struct Case {
    sidecar: Value,
    witnesses: Value,
    scope: Value,
    fixtures: String,
    document_patch: Option<Value>,
}

fn find<'a>(vectors: &'a Value, id: &str) -> &'a Value {
    ["positive", "reader_refusals"]
        .iter()
        .flat_map(|section| vectors[*section].as_array().into_iter().flatten())
        .find(|case| case["id"] == id)
        .unwrap_or_else(|| panic!("{id}: no such vector"))
}

fn resolve(vectors: &Value, id: &str) -> Case {
    let row = find(vectors, id);
    let mut case = match row["base"].as_str() {
        Some(base) => resolve(vectors, base),
        None => Case {
            sidecar: serde_json::json!({
                "model_tuples": row["model_tuples"],
                "model_unions": row["model_unions"],
                "source_unions": row["source_unions"],
                "source_tuples": row["source_tuples"],
            }),
            witnesses: row["witnesses"].clone(),
            scope: row["scope"].clone(),
            fixtures: row["fixtures"].as_str().expect("fixtures").into(),
            document_patch: row.get("document_patch").cloned().filter(|p| !p.is_null()),
        },
    };
    if let Some(operations) = row.get("patch").filter(|p| !p.is_null()) {
        patch(&mut case.sidecar, operations);
    }
    if let Some(operations) = row.get("witnesses_patch").filter(|p| !p.is_null()) {
        if case.witnesses.is_null() {
            case.witnesses = serde_json::json!({"declarations": []});
        }
        patch(&mut case.witnesses, operations);
    }
    if let Some(scope) = row.get("scope_override").filter(|p| !p.is_null()) {
        case.scope = scope.clone();
    }
    case
}

// ---------------------------------------------------------------------------
// Source-row adapters (fixture C): typed `SelectedSourceUnionRow` and
// `SelectedSourceTupleRow` built from the vectors' `source_unions` and
// `source_tuples`, joined by the production source joins before the model join.
// ---------------------------------------------------------------------------

fn source_union_row(value: &Value) -> SelectedSourceUnionRow {
    let settled = key(&value["settled_key"]);
    let members = value["members"]
        .as_array()
        .expect("source members")
        .iter()
        .map(|member| {
            SelectedUnionMemberRow::new(
                UnionMember::from_admitted(
                    key(&member["declaration"]),
                    member["identifier"].as_str().expect("identifier"),
                )
                .expect("a sealed member"),
                member["payload_nodes"]
                    .as_array()
                    .expect("payload nodes")
                    .iter()
                    .map(node_id)
                    .collect(),
                member["payload"]
                    .as_array()
                    .expect("payload")
                    .iter()
                    .map(original_type)
                    .collect(),
            )
        })
        .collect();
    SelectedSourceUnionRow::new(
        node_id(&value["structural_node"]),
        value["source_authority"].as_str().expect("authority"),
        value["source_identity"].as_str().expect("identity"),
        value["qualified_declaration"]
            .as_array()
            .expect("qualified")
            .iter()
            .map(|segment| segment.as_str().expect("segment").into())
            .collect(),
        key(&value["type_handle"]),
        settled,
        members,
    )
}

fn source_tuple_row(value: &Value) -> SelectedSourceTupleRow {
    SelectedSourceTupleRow {
        structural_node: node_id(&value["structural_node"]),
        source_authority: value["source_authority"]
            .as_str()
            .expect("authority")
            .into(),
        source_identity: value["source_identity"].as_str().expect("identity").into(),
        qualified_declaration: value["qualified_declaration"]
            .as_array()
            .expect("qualified")
            .iter()
            .map(|segment| segment.as_str().expect("segment").into())
            .collect(),
        type_handle: key(&value["type_handle"]),
        positions: value["positions"]
            .as_array()
            .expect("positions")
            .iter()
            .map(|position| SelectedTuplePositionRow {
                ordinal: position["ordinal"].as_u64().expect("ordinal"),
                type_node: Some(node_id(&position["type_node"])),
                value_type: Some(original_type(&position["value_type"])),
                member_type: None,
                collections: Vec::new(),
            })
            .collect(),
    }
}

// ---------------------------------------------------------------------------
// The production join, observed.
// ---------------------------------------------------------------------------

thread_local! {
    static LOG: RefCell<Vec<NativeBudgetEvent>> = const { RefCell::new(Vec::new()) };
}

fn record(event: NativeBudgetEvent) {
    LOG.with(|log| log.borrow_mut().push(event));
}

/// Run `body` with the production charge observer and return its charge log.
fn observed<T>(body: impl FnOnce() -> T) -> (T, Vec<NativeBudgetEvent>) {
    LOG.with(|log| log.borrow_mut().clear());
    let result = with_native_budget_observer(record, body);
    (
        result,
        LOG.with(|log| std::mem::take(&mut *log.borrow_mut())),
    )
}

/// One vector's strict-read package, staged identities and sidecar rows.
struct Prepared {
    admitted: CheckedPackageV2,
    identities: SelectedExactIdentities,
    tuples: Vec<SelectedTupleDeclarationRow>,
    unions: Vec<SelectedModelUnionRow>,
    request: Option<CheckedNodeId>,
}

/// A refusal before the model join (the staged declaration rows).
type Staged = (String, Option<String>);

fn prepare(vectors: &Value, id: &str) -> Result<Prepared, Staged> {
    prepare_case(vectors, id, resolve(vectors, id), None)
}

/// `prepare` over an already resolved case, optionally with RFC 6902 operations
/// applied to the checked package before the strict read.
fn prepare_case(
    vectors: &Value,
    id: &str,
    case: Case,
    package_patch: Option<&Value>,
) -> Result<Prepared, Staged> {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let pair = &vectors["fixtures"][&case.fixtures];
    let mut package = read_json(&proposal.join(pair["package"].as_str().expect("package")))
        .expect("published package");
    if let Some(operations) = package_patch {
        patch(&mut package, operations);
    }
    let mut document_value =
        read_json(&proposal.join(pair["document"].as_str().expect("document")))
            .expect("published document");
    if let Some(operations) = &case.document_patch {
        patch(&mut document_value, operations);
    }
    let evidence = inputs::selected_evidence(&document_value);
    let admitted = match crate::read_and_account(
        &fixture_support::canonical(&package),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("{id}: the strict read must admit the input: {other:?}"),
    };
    let mut stage = NativeJoinBudget::new(u64::MAX, u64::MAX);
    let declarations: Vec<_> = case.witnesses["declarations"]
        .as_array()
        .into_iter()
        .flatten()
        .map(declaration_row)
        .collect();
    let mut identities =
        SelectedExactIdentities::from_stage_rows(&admitted, declarations, [], &mut stage).map_err(
            |error| {
                (
                    error.code.as_str().into(),
                    error.form.as_deref().map(str::to_owned),
                )
            },
        )?;
    // The production source joins carry the typed fixture-C source rows.
    let source_unions: Vec<_> = case.sidecar["source_unions"]
        .as_array()
        .into_iter()
        .flatten()
        .map(source_union_row)
        .collect();
    if !source_unions.is_empty() {
        identities = identities
            .with_source_union_rows(&admitted, source_unions, &mut stage)
            .unwrap_or_else(|error| panic!("{id}: the source union row joins: {error:?}"));
    }
    let source_tuples: Vec<_> = case.sidecar["source_tuples"]
        .as_array()
        .into_iter()
        .flatten()
        .map(source_tuple_row)
        .collect();
    if !source_tuples.is_empty() {
        identities = identities
            .with_source_tuple_rows(&admitted, source_tuples, &mut stage)
            .unwrap_or_else(|error| panic!("{id}: the source tuple row joins: {error:?}"));
    }
    let tuples = case.sidecar["model_tuples"]
        .as_array()
        .into_iter()
        .flatten()
        .map(tuple_row)
        .collect();
    let unions = case.sidecar["model_unions"]
        .as_array()
        .into_iter()
        .flatten()
        .map(union_row)
        .collect();
    let request = case.scope["request"]
        .as_object()
        .map(|_| node_id(&case.scope["request"]));
    Ok(Prepared {
        admitted: *admitted,
        identities,
        tuples,
        unions,
        request,
    })
}

type JoinResult = Result<SelectedExactIdentities, ExactTypeProjectionError>;

/// The production model join of one prepared vector on the caller's budget.
fn join(
    prepared: &Prepared,
    budget: &mut NativeJoinBudget,
) -> (JoinResult, Vec<NativeBudgetEvent>) {
    let scope = match &prepared.request {
        Some(request) => ModelUnionScope::Reached(request),
        None => ModelUnionScope::Full,
    };
    observed(|| {
        prepared.identities.with_model_union_rows(
            &prepared.admitted,
            &prepared.tuples,
            &prepared.unions,
            scope,
            budget,
        )
    })
}

fn vectors() -> Value {
    read_json(&proposal_dir_of(&fixtures_dir()).join("model-union-witness-vectors.json"))
        .expect("published model-union witness vectors")
}

fn ample() -> NativeJoinBudget {
    NativeJoinBudget::new(u64::MAX, u64::MAX)
}

fn consumed(budget: &NativeJoinBudget) -> (u64, u64) {
    let used = budget.consumed();
    (used.work, u64::try_from(used.storage).expect("storage"))
}

fn unwrap_join(id: &str, result: JoinResult) -> SelectedExactIdentities {
    result.unwrap_or_else(|error| panic!("{id}: expected admission, got {error:?}"))
}

fn admitted_events(
    events: &[NativeBudgetEvent],
    resource: NativeBudgetResource,
) -> Vec<&NativeBudgetEvent> {
    events
        .iter()
        .filter(|event| event.resource == resource && event.admitted)
        .collect()
}

fn total(events: &[NativeBudgetEvent], resource: NativeBudgetResource) -> u64 {
    admitted_events(events, resource)
        .iter()
        .map(|event| u64::try_from(event.amount).expect("amount"))
        .sum()
}

fn form_of(event: &NativeBudgetEvent) -> &'static str {
    event.locus.expect("a join charge has a locus").form
}

fn source_of(event: &NativeBudgetEvent) -> NativeJoinLocusSource {
    event.locus.expect("a join charge has a locus").source
}

fn retained_unions(joined: &SelectedExactIdentities) -> Vec<CheckedNodeId> {
    joined
        .model_union_rows()
        .map(|row| row.structural_node.clone())
        .collect()
}

fn retained_tuples(joined: &SelectedExactIdentities) -> Vec<CheckedNodeId> {
    joined
        .model_tuple_rows()
        .map(|row| row.structural_node.clone())
        .collect()
}

fn sorted(mut nodes: Vec<CheckedNodeId>) -> Vec<CheckedNodeId> {
    nodes.sort();
    nodes
}

/// The rows of a vector's sidecar that its reached closure leaves out: the
/// published exceptions are the out-of-closure List row of MUW-03 and the
/// out-of-closure Swap tuple of MUW-16; every other sidecar row is in scope.
fn out_of_closure(id: &str) -> (&'static [&'static str], &'static [&'static str]) {
    match id {
        "MUW-03-reached-does-not-join-unrelated-row" => (&["ix://acme/shapes/List"], &[]),
        "MUW-16-reached-does-not-join-unrelated-tuple-row" => (&[], &["ix://acme/shapes/Swap"]),
        _ => (&[], &[]),
    }
}

fn leaf(node: &CheckedNodeId) -> String {
    match node.path.get(2) {
        Some(filament_contract_model::CheckedNodePathSegment::Name(name)) => name.to_string(),
        other => panic!("a model node names its declaration: {other:?}"),
    }
}

/// Every published positive: admitted, the exact retained sets, the closure
/// edge count, and every retained row equal to its input.
#[trace("FR-352-AC-31", "FR-352-AC-35", "FR-358-AC-17", "FR-069-AC-11")]
#[test]
fn published_positive_vectors_admit_with_exact_retained_sets_and_closure_edges() {
    let vectors = vectors();
    let mut ran = 0;
    let mut edge_mismatches: Vec<String> = Vec::new();
    for row in vectors["positive"].as_array().expect("positives") {
        let id = row["id"].as_str().expect("id");
        let prepared = prepare(&vectors, id).unwrap_or_else(|staged| panic!("{id}: {staged:?}"));
        let mut budget = ample();
        let (result, events) = join(&prepared, &mut budget);
        let joined = unwrap_join(id, result);
        let (skip_unions, skip_tuples) = out_of_closure(id);
        let want_unions: Vec<_> = prepared
            .unions
            .iter()
            .filter(|union| !skip_unions.contains(&leaf(&union.structural_node).as_str()))
            .map(|union| union.structural_node.clone())
            .collect();
        let want_tuples: Vec<_> = prepared
            .tuples
            .iter()
            .filter(|tuple| !skip_tuples.contains(&leaf(&tuple.structural_node).as_str()))
            .map(|tuple| tuple.structural_node.clone())
            .collect();
        assert_eq!(
            sorted(retained_unions(&joined)),
            sorted(want_unions),
            "{id}: union set"
        );
        assert_eq!(
            sorted(retained_tuples(&joined)),
            sorted(want_tuples),
            "{id}: tuple set"
        );
        assert!(
            !retained_unions(&joined).is_empty(),
            "{id}: a positive retains union rows, never nothing"
        );
        for retained in joined.model_union_rows() {
            let input = prepared
                .unions
                .iter()
                .find(|union| union.structural_node == retained.structural_node)
                .expect("a retained row was supplied");
            assert_eq!(retained, input, "{id}: rows are returned unchanged");
        }
        for retained in joined.model_tuple_rows() {
            assert!(
                prepared.tuples.contains(retained),
                "{id}: tuple rows unchanged"
            );
        }
        if let Some(edges) = row["closure_edges"].as_u64() {
            let walked = events
                .iter()
                .filter(|event| {
                    event.admitted
                        && event.resource == NativeBudgetResource::Work
                        && form_of(event) == "model_union_closure_edge"
                })
                .count();
            if walked as u64 != edges {
                edge_mismatches.push(format!("{id}: walked {walked}, published {edges}"));
            }
            assert!(row["scope"]["mode"] == "reached", "{id}");
        } else {
            assert!(
                events
                    .iter()
                    .all(|event| form_of(event) != "model_union_closure_edge"),
                "{id}: a full scope walks no closure"
            );
        }
        // The charge log is the budget's consumption, charge for charge.
        assert_eq!(
            (
                total(&events, NativeBudgetResource::Work),
                total(&events, NativeBudgetResource::LogicalStorage)
            ),
            consumed(&budget),
            "{id}: the observed charges are the consumption"
        );
        ran += 1;
    }
    assert!(edge_mismatches.is_empty(), "{edge_mismatches:#?}");
    assert_eq!(ran, vectors["positive"].as_array().unwrap().len());
    assert_eq!(
        ran, 20,
        "MUW-01 to MUW-20, fixture-C source controls included"
    );
}

/// The source rows of fixture C stay the source join's: the model join returns
/// the same source rows untouched and retains no source row as a model row.
#[trace("FR-352-AC-36")]
#[test]
fn source_rows_stay_beside_model_rows_and_are_never_read_as_model_rows() {
    let vectors = vectors();
    let prepared = prepare(&vectors, "MUW-19-source-union-row-beside-model-rows").unwrap();
    let (result, _) = join(&prepared, &mut ample());
    let joined = unwrap_join("MUW-19", result);
    assert_eq!(
        joined.source_union_rows().count(),
        1,
        "the source union row stays"
    );
    assert_eq!(
        retained_unions(&joined),
        vec![prepared.unions[0].structural_node.clone()],
        "only the model row of Link is retained"
    );
    let prepared = prepare(&vectors, "MUW-20-source-tuple-row-beside-model-rows").unwrap();
    let (result, _) = join(&prepared, &mut ample());
    let joined = unwrap_join("MUW-20", result);
    assert_eq!(
        joined.source_tuple_rows().count(),
        1,
        "the source tuple row stays"
    );
    assert_eq!(
        retained_tuples(&joined).len(),
        1,
        "the model tuple Pair row only"
    );
    assert_eq!(leaf(&retained_tuples(&joined)[0]), "ix://acme/shapes/Pair");
}

/// Every published reader refusal that the production join reads (the closed
/// format, parser and depth cases belong to the FSL identity-file parser) keeps
/// its code AND its compact locus, and a coverage vector keeps its node.
#[trace("FR-352-AC-31", "FR-352-AC-34", "FR-358-AC-19")]
#[test]
fn published_reader_refusals_keep_their_codes_loci_and_nodes() {
    let vectors = vectors();
    let rows = vectors["reader_refusals"].as_array().expect("refusals");
    let mut ran = 0;
    let mut malformed = 0;
    let mut mismatches = Vec::new();
    for row in rows {
        let id = row["id"].as_str().expect("id");
        let expected = row["expected"]["code"].as_str().expect("code");
        if expected == "malformed" {
            malformed += 1;
            continue;
        }
        let want = row["expected"]["locus"].as_str().expect("locus");
        let (code, locus, at) = match prepare(&vectors, id) {
            Err((code, locus)) => (code, locus.unwrap_or_default(), None),
            Ok(prepared) => match join(&prepared, &mut ample()).0 {
                Ok(_) => ("admitted".to_owned(), String::new(), None),
                Err(error) => (
                    error.code.as_str().to_owned(),
                    error.form.as_deref().unwrap_or_default().to_owned(),
                    error.at.map(|node| *node),
                ),
            },
        };
        let want_node = row["expected"]
            .get("node")
            .filter(|node| !node.is_null())
            .map(node_id);
        if code != expected || locus != want || (want_node.is_some() && at != want_node) {
            mismatches.push(format!(
                "{id}: expected {expected} at {want} node {want_node:?}, got {code} at {locus} node {at:?}"
            ));
        }
        ran += 1;
    }
    assert!(
        mismatches.is_empty(),
        "{} of {ran}: {mismatches:#?}",
        mismatches.len()
    );
    assert_eq!(
        ran + malformed,
        rows.len(),
        "every refusal is executed or is a parser case"
    );
    assert_eq!(rows.len(), 75, "the published refusal count");
    assert!(
        malformed <= 4,
        "only the closed-format cases are excluded: {malformed}"
    );
    assert!(ran >= 71, "ran {ran} refusals with code, locus and node");
}

// ---------------------------------------------------------------------------
// FR-830 budget cases B-01 to B-08 on the production shared budgets.
// ---------------------------------------------------------------------------

/// A probe run on a fresh join budget with these limits.
struct Probe {
    result: JoinResult,
    events: Vec<NativeBudgetEvent>,
    consumed: (u64, u64),
    denied: Option<(NativeJoinChargeKind, u128)>,
    locus: Option<NativeJoinLocus>,
}

fn probe(prepared: &Prepared, work: u64, storage: u64) -> Probe {
    let mut budget = NativeJoinBudget::new(work, storage);
    let (result, events) = join(prepared, &mut budget);
    Probe {
        result,
        events,
        consumed: consumed(&budget),
        denied: budget.first_unpaid().map(|d| (d.kind, d.amount)),
        locus: budget.first_unpaid_locus(),
    }
}

fn admitted_work_and_storage(events: &[NativeBudgetEvent], upto: usize) -> (u64, u64) {
    (
        cumulative(events, upto, NativeBudgetResource::Work),
        cumulative(events, upto, NativeBudgetResource::LogicalStorage),
    )
}

fn cumulative(events: &[NativeBudgetEvent], upto: usize, resource: NativeBudgetResource) -> u64 {
    total(&events[..upto], resource)
}

/// The index of the last admitted charge of `resource` with a positive amount
/// that satisfies `pick`: only a positive charge can exhaust a limit.
fn last_positive(
    events: &[NativeBudgetEvent],
    resource: NativeBudgetResource,
    pick: impl Fn(&NativeBudgetEvent) -> bool,
) -> Option<usize> {
    events.iter().rposition(|event| {
        event.admitted && event.resource == resource && event.amount > 0 && pick(event)
    })
}

/// The index of the first admitted positive charge of `resource` at or after `from`.
fn first_positive(
    events: &[NativeBudgetEvent],
    resource: NativeBudgetResource,
    from: usize,
) -> Option<usize> {
    (from..events.len())
        .find(|&at| events[at].admitted && events[at].resource == resource && events[at].amount > 0)
}

fn kind_of(resource: NativeBudgetResource) -> NativeJoinChargeKind {
    match resource {
        NativeBudgetResource::Work => NativeJoinChargeKind::Work,
        _ => NativeJoinChargeKind::Storage,
    }
}

fn limits_for(resource: NativeBudgetResource, value: u64) -> (u64, u64) {
    match resource {
        NativeBudgetResource::Work => (value, u64::MAX),
        _ => (u64::MAX, value),
    }
}

/// Assert a probe exhausted at baseline charge `events[at]`: the full
/// attempted amount and the channel, row index and form of that charge are
/// reported, every earlier charge stays consumed, the denied one consumed
/// nothing, and no rows are returned.
fn assert_denied_at(probe: &Probe, base: &[NativeBudgetEvent], at: usize, what: &str) {
    let event = base[at];
    let error = probe.result.as_ref().expect_err(what);
    let kind = kind_of(event.resource);
    assert_eq!(
        error.code.as_str(),
        match kind {
            NativeJoinChargeKind::Work => "exhausted_work",
            NativeJoinChargeKind::Storage => "exhausted_storage",
        },
        "{what}"
    );
    assert_eq!(
        probe.denied,
        Some((kind, event.amount)),
        "{what}: the full attempted amount"
    );
    assert_eq!(
        probe.locus, event.locus,
        "{what}: channel, sidecar index and form"
    );
    // The charge log is the baseline's up to the denied charge, which is not
    // admitted and consumed nothing: the denied resource's consumption before
    // it is exactly the earlier charges (a refusal diagnostic is charged after).
    let before = admitted_work_and_storage(base, at);
    assert_eq!(
        probe.events[..at],
        base[..at],
        "{what}: the earlier charges are the baseline's"
    );
    let denied = probe.events[at];
    assert!(!denied.admitted, "{what}");
    assert_eq!(
        (denied.amount, denied.locus),
        (event.amount, event.locus),
        "{what}"
    );
    assert_eq!(
        denied.consumed_before,
        match event.resource {
            NativeBudgetResource::Work => before.0,
            _ => before.1,
        },
        "{what}: the denied charge consumed nothing"
    );
}

fn budget_case<'a>(vectors: &'a Value, id: &str) -> &'a Value {
    vectors["budget_cases"]
        .as_array()
        .expect("budget cases")
        .iter()
        .find(|case| case["id"] == id)
        .unwrap_or_else(|| panic!("{id}: no such budget case"))
}

#[trace("FR-358-AC-15", "FR-358-AC-18")]
#[test]
fn budget_case_b01_full_baseline_denies_the_last_charge_of_each_counter() {
    let vectors = vectors();
    let case = budget_case(&vectors, "MUW-B-01-full-baseline");
    let prepared = prepare(&vectors, case["base"].as_str().unwrap()).unwrap();
    let base = probe(&prepared, u64::MAX, u64::MAX);
    let baseline = unwrap_join("B-01", base.result.clone());
    let (w, s) = base.consumed;
    assert!(w > 0 && s > 0);
    let again = probe(&prepared, w, s);
    assert_eq!(
        again.consumed,
        (w, s),
        "the same delta at the baseline limits"
    );
    assert_eq!(
        retained_unions(&unwrap_join("B-01", again.result)),
        retained_unions(&baseline)
    );
    use NativeBudgetResource::{LogicalStorage, Work};
    let last_work = last_positive(&base.events, Work, |_| true).unwrap();
    let last_storage = last_positive(&base.events, LogicalStorage, |_| true).unwrap();
    assert_denied_at(&probe(&prepared, w - 1, s), &base.events, last_work, "W-1");
    assert_denied_at(
        &probe(&prepared, w, s - 1),
        &base.events,
        last_storage,
        "S-1",
    );
    for (resource, at) in [
        (Work, first_positive(&base.events, Work, 0).unwrap()),
        (
            LogicalStorage,
            first_positive(&base.events, LogicalStorage, 0).unwrap(),
        ),
    ] {
        let (lw, ls) = limits_for(resource, 0);
        let zero = probe(&prepared, lw, ls);
        assert_denied_at(&zero, &base.events, at, "zero limit");
        assert_eq!(
            zero.result.as_ref().unwrap_err().configured_bound,
            Some(0),
            "the configured bound reported is 0"
        );
    }
}

#[trace("FR-358-AC-17", "FR-358-AC-18")]
#[test]
fn budget_case_b02_reached_closure_edge_boundary_denies_edge_then_first_row_join() {
    use NativeBudgetResource::{LogicalStorage, Work};
    let vectors = vectors();
    let case = budget_case(&vectors, "MUW-B-02-reached-baseline");
    let prepared = prepare(&vectors, case["base"].as_str().unwrap()).unwrap();
    let base = probe(&prepared, u64::MAX, u64::MAX);
    unwrap_join("B-02", base.result.clone());
    let (w, _) = base.consumed;
    let common = |event: &NativeBudgetEvent| {
        matches!(form_of(event), "model_tuples_row" | "model_unions_row")
    };
    let edge = |event: &NativeBudgetEvent| form_of(event) == "model_union_closure_edge";
    let wp_at = last_positive(&base.events, Work, common).unwrap();
    let ec_at = last_positive(&base.events, Work, edge).unwrap();
    let wp = cumulative(&base.events, wp_at + 1, Work);
    let ec = cumulative(&base.events, ec_at + 1, Work);
    assert!(wp <= ec, "Wp {wp} <= Ec {ec}");
    let last_work = last_positive(&base.events, Work, |_| true).unwrap();
    assert_denied_at(
        &probe(&prepared, w - 1, u64::MAX),
        &base.events,
        last_work,
        "W-1",
    );
    // The published document fixes the edges: Shape has two typed cases (Flag:
    // Boolean, Both: Pair; Empty is nullary) and Pair two positions, 4 edges in
    // all. Each edge charge is located at the graph node it is examined from.
    let graph = prepared.admitted.resolved_graph();
    let origin = |event: &NativeBudgetEvent| match source_of(event) {
        NativeJoinLocusSource::GraphNode(position) => leaf(&graph.nodes[position].node_id),
        other => panic!("a closure edge names its graph node, not {other:?}"),
    };
    let mut origins: Vec<_> = base
        .events
        .iter()
        .filter(|event| edge(event) && event.admitted && event.amount > 0)
        .map(|event| (origin(event), event.amount))
        .collect();
    origins.sort();
    assert_eq!(
        origins,
        [
            ("ix://acme/shapes/Pair".to_owned(), 1),
            ("ix://acme/shapes/Pair".to_owned(), 1),
            ("ix://acme/shapes/Shape".to_owned(), 1),
            ("ix://acme/shapes/Shape".to_owned(), 1),
        ]
    );
    // Ec-1: the denied charge is the last closure edge, before any row joins.
    let denied = probe(&prepared, ec - 1, u64::MAX);
    assert_denied_at(&denied, &base.events, ec_at, "Ec-1");
    assert_eq!(
        denied.locus.map(|locus| (locus.form, locus.source)),
        Some(("model_union_closure_edge", source_of(&base.events[ec_at]))),
        "Ec-1: the denial names the edge origin node"
    );
    assert!(
        matches!(
            denied.locus.map(|locus| locus.source),
            Some(NativeJoinLocusSource::GraphNode(_))
        ),
        "Ec-1: not the package"
    );
    assert!(
        denied.events.iter().all(|event| !matches!(
            form_of(event),
            "model_tuple_row" | "model_union_row" | "model_union_retain"
        )),
        "Ec-1: no row has been joined"
    );
    // Ec: every edge is paid and stays consumed; the first row-join charge is denied.
    let next = first_positive(&base.events, Work, ec_at + 1).expect("a row-join charge follows");
    assert_eq!(form_of(&base.events[next]), "model_tuple_row");
    let denied = probe(&prepared, ec, u64::MAX);
    assert_denied_at(&denied, &base.events, next, "Ec");
    assert_eq!(denied.consumed.0, ec);
    let NativeJoinLocusSource::ModelTupleRow(row) = source_of(&base.events[next]) else {
        panic!("the first row-join charge names a model tuple row");
    };
    assert_eq!(
        leaf(&prepared.tuples[row].structural_node),
        "ix://acme/shapes/Pair",
        "the denied locus names a row of the closure"
    );
    let _ = LogicalStorage;
}

#[trace("FR-358-AC-16", "FR-358-AC-15")]
#[test]
fn budget_case_b03_cumulative_consumption_across_two_joins_on_one_budget() {
    use NativeBudgetResource::Work;
    let vectors = vectors();
    let case = budget_case(&vectors, "MUW-B-03-cumulative-shared-budget");
    let first = prepare(&vectors, case["base"].as_str().unwrap()).unwrap();
    let second = prepare(&vectors, case["then"].as_str().unwrap()).unwrap();
    let one = probe(&first, u64::MAX, u64::MAX).consumed;
    let two_run = probe(&second, u64::MAX, u64::MAX);
    let two = two_run.consumed;
    let mut budget = ample();
    unwrap_join("first", join(&first, &mut budget).0);
    assert_eq!(consumed(&budget), one);
    let (result, _) = join(&second, &mut budget);
    unwrap_join("second", result);
    assert_eq!(
        consumed(&budget),
        (one.0 + two.0, one.1 + two.1),
        "the second join's delta is its own baseline and nothing is restored"
    );
    // Prior consumption counts against the same limit: one unit short of the sum.
    let mut short = NativeJoinBudget::new(one.0 + two.0 - 1, u64::MAX);
    unwrap_join("first", join(&first, &mut short).0);
    let (result, events) = join(&second, &mut short);
    let error = result.expect_err("the second join exhausts the shared work limit");
    assert_eq!(error.code.as_str(), "exhausted_work");
    let last = last_positive(&two_run.events, Work, |_| true).unwrap();
    assert_eq!(
        short.first_unpaid().map(|d| d.amount),
        Some(two_run.events[last].amount)
    );
    assert_eq!(short.first_unpaid_locus(), two_run.events[last].locus);
    assert_eq!(
        consumed(&short).0,
        one.0 + admitted_work_and_storage(&two_run.events, last).0,
        "the first join's consumption and the second's admitted charges stay"
    );
    assert!(events.iter().any(|event| !event.admitted));
}

#[trace("FR-358-AC-18")]
#[test]
fn budget_case_b04_union_refusal_keeps_every_admitted_charge() {
    let vectors = vectors();
    refusal_keeps_charges(&vectors, "MUW-B-04-refusal-keeps-charges");
}

/// B-04 and B-07 share one patched sidecar: the rank of case 0 of union 1.
fn refusal_keeps_charges(vectors: &Value, id: &str) {
    let case = budget_case(vectors, id);
    let mut resolved = resolve(vectors, case["base"].as_str().unwrap());
    patch(&mut resolved.sidecar, &case["patch"]);
    let mut prepared = prepare(vectors, case["base"].as_str().unwrap()).unwrap();
    prepared.unions = resolved.sidecar["model_unions"]
        .as_array()
        .unwrap()
        .iter()
        .map(union_row)
        .collect();
    prepared.tuples = resolved.sidecar["model_tuples"]
        .as_array()
        .unwrap()
        .iter()
        .map(tuple_row)
        .collect();
    let baseline = probe(
        &prepare(vectors, case["base"].as_str().unwrap()).unwrap(),
        u64::MAX,
        u64::MAX,
    );
    let run = probe(&prepared, u64::MAX, u64::MAX);
    let error = run.result.as_ref().expect_err("the patched row refuses");
    assert_eq!(error.code.as_str(), "mismatched_identity_witness");
    assert_eq!(error.form.as_deref(), Some("/model_unions/1/cases/0/rank"));
    assert_eq!(
        run.consumed,
        (
            total(&run.events, NativeBudgetResource::Work),
            total(&run.events, NativeBudgetResource::LogicalStorage)
        ),
        "{id}: every admitted charge stays consumed"
    );
    assert!(
        run.consumed.0 > 0 && run.consumed.0 < baseline.consumed.0,
        "{id}: no success delta"
    );
    // The tuple rows were joined and charged before the union refusal.
    assert!(
        run.events
            .iter()
            .any(|event| form_of(event) == "model_tuple_row" && event.admitted),
        "{id}: the model_tuples charges are kept"
    );
    assert!(
        run.result.is_err(),
        "{id}: no rows of either channel are returned"
    );
}

#[trace("FR-358-AC-18", "FR-358-AC-19")]
#[test]
fn budget_case_b05_the_channel_split_of_work_and_storage_on_one_budget() {
    use NativeBudgetResource::{LogicalStorage, Work};
    let vectors = vectors();
    let case = budget_case(&vectors, "MUW-B-05-two-channels-one-budget");
    let prepared = prepare(&vectors, case["base"].as_str().unwrap()).unwrap();
    let base = probe(&prepared, u64::MAX, u64::MAX);
    unwrap_join("B-05", base.result.clone());
    let tuple_row = |event: &NativeBudgetEvent| {
        matches!(source_of(event), NativeJoinLocusSource::ModelTupleRow(_))
    };
    for resource in [Work, LogicalStorage] {
        let last_at = last_positive(&base.events, resource, tuple_row).expect("tuple-row charges");
        let amount_total = cumulative(&base.events, last_at + 1, resource);
        let (lw, ls) = limits_for(resource, amount_total - 1);
        let below = probe(&prepared, lw, ls);
        assert_denied_at(&below, &base.events, last_at, "Wt-1 / St-1");
        assert!(
            matches!(
                below.locus.expect("a locus").source,
                NativeJoinLocusSource::ModelTupleRow(_)
            ),
            "{resource:?}: the denied charge is inside /model_tuples: {:?}",
            below.locus
        );
        let next = first_positive(&base.events, resource, last_at + 1).expect("a union charge");
        let (lw, ls) = limits_for(resource, amount_total);
        let at = probe(&prepared, lw, ls);
        assert_denied_at(&at, &base.events, next, "Wt / St");
        assert!(
            matches!(
                at.locus.expect("a locus").source,
                NativeJoinLocusSource::ModelUnionRow(_)
            ),
            "{resource:?}: the denied charge names /model_unions: {:?}",
            at.locus
        );
        let used = match resource {
            Work => at.consumed.0,
            _ => at.consumed.1,
        };
        assert_eq!(used, amount_total, "every tuple charge stays consumed");
    }
}

#[trace("FR-358-AC-18", "FR-358-AC-19")]
#[test]
fn budget_cases_b06_and_b07_refusals_keep_the_charges_of_both_channels() {
    let vectors = vectors();
    let b06 = budget_case(&vectors, "MUW-B-06-tuple-refusal-keeps-charges");
    let base = b06["base"].as_str().unwrap();
    let mut resolved = resolve(&vectors, base);
    patch(&mut resolved.sidecar, &b06["patch"]);
    let mut prepared = prepare(&vectors, base).unwrap();
    prepared.tuples = resolved.sidecar["model_tuples"]
        .as_array()
        .unwrap()
        .iter()
        .map(tuple_row)
        .collect();
    let run = probe(&prepared, u64::MAX, u64::MAX);
    let error = run.result.as_ref().expect_err("tuple 3 refuses");
    assert_eq!(error.code.as_str(), "mismatched_identity_witness");
    assert_eq!(
        error.form.as_deref(),
        Some("/model_tuples/3/original_identity")
    );
    assert_eq!(
        run.consumed,
        (
            total(&run.events, NativeBudgetResource::Work),
            total(&run.events, NativeBudgetResource::LogicalStorage)
        )
    );
    let forms: Vec<_> = run.events.iter().map(form_of).collect();
    assert!(
        forms.contains(&"model_union_handle"),
        "the preflight is charged"
    );
    assert!(
        forms.contains(&"model_tuple_row"),
        "the tuple rows before the refusal"
    );
    for stage in [
        "model_union_retain",
        "model_union_row",
        "model_union_prior_copy",
        "model_unions_seen",
        "model_union_census",
        "model_union_case",
        "model_union_coverage",
    ] {
        assert!(
            !forms.contains(&stage),
            "no model_unions row is charged: {stage}"
        );
    }
    assert!(
        run.events
            .iter()
            .filter(|event| form_of(event) == "model_tuple_row")
            .all(|event| matches!(source_of(event), NativeJoinLocusSource::ModelTupleRow(k) if k <= 3)),
        "tuple rows stop at the refused index"
    );
    // B-07: a union refusal keeps the tuple charges and the union charges before it.
    refusal_keeps_charges(&vectors, "MUW-B-07-union-refusal-keeps-tuple-charges");
}

#[trace("FR-358-AC-16", "FR-358-AC-17")]
#[test]
fn budget_case_b08_unreached_row_costs_only_common_index_work() {
    let vectors = vectors();
    let case = budget_case(
        &vectors,
        "MUW-B-08-reached-charges-common-work-only-outside-the-closure",
    );
    let wide = prepare(&vectors, case["base"].as_str().unwrap()).unwrap();
    let narrow = prepare(&vectors, case["compare"].as_str().unwrap()).unwrap();
    let wide_run = probe(&wide, u64::MAX, u64::MAX);
    let narrow_run = probe(&narrow, u64::MAX, u64::MAX);
    let wide_joined = unwrap_join("MUW-03", wide_run.result.clone());
    let narrow_joined = unwrap_join("MUW-02", narrow_run.result.clone());
    assert_eq!(
        retained_unions(&wide_joined),
        retained_unions(&narrow_joined),
        "the same rows"
    );
    assert_eq!(
        retained_tuples(&wide_joined),
        retained_tuples(&narrow_joined)
    );
    // The out-of-closure List row is the first union row of MUW-03.
    let list = wide
        .unions
        .iter()
        .position(|union| leaf(&union.structural_node) == "ix://acme/shapes/List")
        .expect("List is supplied");
    let is_list =
        |event: &NativeBudgetEvent| source_of(event) == NativeJoinLocusSource::ModelUnionRow(list);
    let (extra, shared): (Vec<NativeBudgetEvent>, Vec<NativeBudgetEvent>) = wide_run
        .events
        .iter()
        .copied()
        .partition(|event| is_list(event));
    assert!(!extra.is_empty(), "the unreached row is not free");
    for event in &extra {
        assert!(
            matches!(form_of(event), "model_union_handle" | "model_unions_row"),
            "an unreached row carries only common forms, not {}",
            form_of(event)
        );
    }
    assert!(
        extra
            .iter()
            .any(|event| form_of(event) == "model_unions_row"),
        "the common pass reads and charges the row"
    );
    let key =
        |event: &NativeBudgetEvent| (event.resource, form_of(event), event.amount, event.admitted);
    let shared: Vec<_> = shared.iter().map(key).collect();
    let narrow_log: Vec<_> = narrow_run.events.iter().map(key).collect();
    assert_eq!(
        shared, narrow_log,
        "retention, closure and join charges are MUW-02's"
    );
    assert!(
        wide_run.consumed.0 > narrow_run.consumed.0 && wide_run.consumed.1 >= narrow_run.consumed.1
    );
}

// ---------------------------------------------------------------------------
// FR-830 payload forms over strict admitted inputs: any admitted non-Clock
// payload type, each backed by a real package node and document variant.
// ---------------------------------------------------------------------------

/// Add scalar node `form` to the strict-read package, with its source map entry.
fn scalar_node_ops(form: &str) -> Value {
    let node_id =
        serde_json::json!({"ordinal": 0, "package": "example", "path": ["scalar_type", form]});
    serde_json::json!([
        {"op": "add", "path": "/semantic_graph/nodes/0", "value": {
            "body": {"members": [], "term": "aggregate"},
            "dependencies": [],
            "node_id": node_id,
            "node_tag": "scalar_type",
            "occurrences": [{"ordinal": 0, "role": "type"}],
            "schema_version": "filament.checked-semantic-graph/v2",
            "semantic_form": form,
            "semantic_type": node_id,
        }},
        {"op": "add", "path": "/source_map/0", "value": {
            "node_id": node_id, "ordinal": 0, "role": "type",
            "regions": [{"end": 1092, "source": {"authority": "agent-ix", "identity": "example"}, "start": 1088}],
        }},
    ])
}

/// MUW-07's Link with a Tick case whose payload is native type `native`
/// (document spelling), node `form` and Exact original type `original`.
fn link_with_payload(
    vectors: &Value,
    native: &str,
    form: &str,
    original: Value,
    node_form: &str,
) -> Result<Prepared, Staged> {
    let mut case = resolve(vectors, "MUW-07-clock-payload");
    let mut document_patch = case
        .document_patch
        .take()
        .expect("MUW-07 patches the document");
    document_patch[0]["value"]["payloadType"] = format!("ix://filament/native/{native}").into();
    case.document_patch = Some(document_patch);
    let cases = case.sidecar["model_unions"][0]["cases"]
        .as_array_mut()
        .expect("cases");
    let mut tick = cases.pop().expect("a case");
    if tick["name"] != "Tick" {
        cases.push(tick.clone());
        tick = cases[0].clone();
    }
    let node =
        serde_json::json!({"package": "example", "path": ["scalar_type", form], "ordinal": 0});
    let mut tick = tick;
    tick["payload"] = serde_json::json!({"kind": "data", "node": node, "original_type": original});
    case.sidecar["model_unions"][0]["cases"]
        .as_array_mut()
        .expect("cases")
        .push(tick);
    let ops = scalar_node_ops(node_form);
    prepare_case(vectors, "link-with-payload", case, Some(&ops))
}

#[trace("FR-352-AC-32", "FR-352-AC-34")]
#[test]
fn any_admitted_non_clock_payload_is_joined_and_each_wrong_witness_is_refused() {
    let vectors = vectors();
    for (native, form, original) in [
        (
            "Float32",
            "float32",
            serde_json::json!({"kind": "float", "width": 32}),
        ),
        (
            "Float64",
            "float64",
            serde_json::json!({"kind": "float", "width": 64}),
        ),
        ("Bytes", "bytes", serde_json::json!({"kind": "bytes"})),
        (
            "JsonObject",
            "json_object",
            serde_json::json!({"kind": "json_object"}),
        ),
        ("String", "string", serde_json::json!({"kind": "string"})),
        ("UUID", "uuid", serde_json::json!({"kind": "uuid"})),
        (
            "Timestamp",
            "timestamp",
            serde_json::json!({"kind": "timestamp"}),
        ),
    ] {
        let prepared = link_with_payload(&vectors, native, form, original.clone(), form)
            .unwrap_or_else(|staged| panic!("{native}: {staged:?}"));
        let (result, _) = join(&prepared, &mut ample());
        let joined = unwrap_join(native, result);
        let row = joined
            .model_union_rows()
            .next()
            .expect("the Link row is retained");
        let SelectedModelUnionPayload::Data { original_type, .. } = &row.cases[2].payload else {
            panic!("{native}: the Tick payload is data");
        };
        assert_eq!(original_type, &self::original_type(&original), "{native}");
        // The same payload with another form's Exact type is refused at the payload.
        let wrong = if form == "bytes" {
            "json_object"
        } else {
            "bytes"
        };
        let wrong_original = serde_json::json!({"kind": wrong});
        let prepared = link_with_payload(&vectors, native, form, wrong_original, form).unwrap();
        let (result, _) = join(&prepared, &mut ample());
        let error = result.expect_err("a wrong original type refuses");
        assert_eq!(
            error.code.as_str(),
            "mismatched_identity_witness",
            "{native}"
        );
        assert_eq!(
            error.form.as_deref(),
            Some("/model_unions/0/cases/2/payload/original_type"),
            "{native}"
        );
        // A payload node the package does not hold is unknown at the payload node.
        let ops = serde_json::json!([]);
        let mut case = resolve(&vectors, "MUW-07-clock-payload");
        let mut document_patch = case.document_patch.take().unwrap();
        document_patch[0]["value"]["payloadType"] = format!("ix://filament/native/{native}").into();
        case.document_patch = Some(document_patch);
        let tick = &mut case.sidecar["model_unions"][0]["cases"][2];
        tick["payload"] = serde_json::json!({
            "kind": "data",
            "node": {"package": "example", "path": ["scalar_type", form], "ordinal": 0},
            "original_type": original,
        });
        let prepared = prepare_case(&vectors, "absent-payload-node", case, Some(&ops)).unwrap();
        {
            let (result, _) = join(&prepared, &mut ample());
            let error = result.expect_err("an absent payload node refuses");
            assert_eq!(error.code.as_str(), "unknown_node", "{native}");
            assert_eq!(
                error.form.as_deref(),
                Some("/model_unions/0/cases/2/payload/node"),
                "{native}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// The public runtime surface: `SelectedTypeDeclarations::unions()` and the
// reached monitor closure, over real model-union rows.
// ---------------------------------------------------------------------------

fn color_enum_rows(
    prepared: &Prepared,
) -> (
    Vec<SelectedModelEnumDeclarationRow>,
    Vec<SelectedModelEnumMemberRow>,
    Vec<SelectedModelEnumShapeRow>,
) {
    let color: CheckedNodeId = serde_json::from_value(serde_json::json!({
        "package": "acme/shapes",
        "path": ["model", "variant_type", "ix://acme/shapes/Color"],
        "ordinal": 0
    }))
    .unwrap();
    let red: CheckedNodeId = serde_json::from_value(serde_json::json!({
        "package": "example",
        "path": ["value", "enum_value", {"operand": 0}, "Red"],
        "ordinal": 0
    }))
    .unwrap();
    let color_key = NodeKey::new("fsl-enum", vec![PathSegment::Text("Color".into())], 0).unwrap();
    let red_key = NodeKey::new("fsl-enum", vec![PathSegment::Text("Color.Red".into())], 0).unwrap();
    let variant = filament_exact::VariantId::new(color_key.clone(), "Red").unwrap();
    let declaration = SelectedModelEnumDeclarationRow::new(
        color.clone(),
        "acme/shapes",
        "ix://acme/shapes/Color",
        color_key.clone(),
    );
    let member = SelectedModelEnumMemberRow::new(
        red.clone(),
        color.clone(),
        "acme/shapes",
        "ix://acme/shapes/Color",
        "ix://acme/shapes/variant/Color-Red",
        "Red",
        red_key.clone(),
    );
    let shape = SelectedModelEnumShapeRow {
        structural_node: color,
        node_key: color_key,
        selected_document: "acme/shapes".into(),
        original_identity: "ix://acme/shapes/Color".into(),
        shape: filament_exact::EnumShape::new(false, [variant.clone()]),
        cases: vec![SelectedModelEnumShapeCaseRow {
            case: "Red".into(),
            structural_node: red,
            member_key: red_key,
            member_original_identity: "ix://acme/shapes/variant/Color-Red".into(),
            variant,
        }],
    };
    (vec![declaration], vec![member], vec![shape])
}

/// The selected-type runtime reads the joined rows: every joined model union is
/// declared once with the type handle its row names, and the same selection
/// without the union rows is refused as an unavailable identity witness.
#[trace("FR-352-AC-35", "FR-358-AC-17")]
#[test]
fn runtime_selected_types_declare_each_joined_union_by_its_type_handle() {
    let vectors = vectors();
    let mut case = resolve(&vectors, "MUW-01-full-environment");
    case.document_patch = Some(serde_json::json!([
        {"op": "remove", "path": "/types/6/variants/1"},
        {"op": "remove", "path": "/types/0"}
    ]));
    let mut prepared = prepare_case(&vectors, "runtime", case, None).unwrap();
    let (declarations, members, shapes) = color_enum_rows(&prepared);
    prepared.identities = prepared
        .identities
        .clone()
        .with_admitted_model_enum_rows(
            &prepared.admitted,
            SelectedModelEnumRows {
                declarations: &declarations,
                members: &members,
                shapes: &shapes,
            },
            &mut ample(),
        )
        .expect("the enum rows admit");
    let joined = unwrap_join("MUW-01", join(&prepared, &mut ample()).0);
    let rows: Vec<_> = joined.model_union_rows().collect();
    let declared = prepared
        .admitted
        .project_selected_type_declarations(&joined, &prepared.tuples, &[], &mut ample())
        .expect("the joined rows project");
    assert_eq!(declared.unions().len(), rows.len());
    for row in &rows {
        assert!(
            declared
                .unions()
                .iter()
                .any(|union| union.type_handle == row.type_handle),
            "{:?} is declared by its own handle",
            row.type_handle
        );
    }
}

/// Intake: a union payload naming an object type that no selected-package
/// declaration provides is refused by the strict read; the same union with the
/// selected-package object is admitted by every published positive vector.
#[trace("FR-069-AC-12")]
#[test]
fn a_union_payload_naming_an_object_outside_the_selected_package_is_refused_at_intake() {
    let vectors = vectors();
    let case = resolve(&vectors, "MUW-06-reached-through-object-fields");
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let pair = &vectors["fixtures"][&case.fixtures];
    let package = read_json(&proposal.join(pair["package"].as_str().unwrap())).unwrap();
    let mut document = read_json(&proposal.join(pair["document"].as_str().unwrap())).unwrap();
    let held = document["types"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|ty| ty["variants"].as_array().into_iter().flatten())
        .filter(|variant| variant["payloadType"] == "ix://acme/shapes/Holder")
        .count();
    assert!(held >= 1, "the published union holds an object payload");
    let admit = |document: &Value| {
        crate::read_and_account(
            &fixture_support::canonical(&package),
            CheckedPackageReadLimits::bounded(),
            &inputs::selected_evidence(document),
        )
    };
    assert!(matches!(
        admit(&document),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    for ty in document["types"].as_array_mut().unwrap() {
        for variant in ty["variants"].as_array_mut().into_iter().flatten() {
            if variant["payloadType"] == "ix://acme/shapes/Holder" {
                variant["payloadType"] = "ix://elsewhere/shapes/Holder".into();
            }
        }
    }
    assert!(
        !matches!(admit(&document), CheckedPackageV2ReadResult::Admitted(_)),
        "an object outside the selected package is not an admitted payload"
    );
}

/// Two published defects stacked on one sidecar: the join refuses the one of
/// the earlier stage (identity, census, case, payload, then coverage), at its
/// published code and locus, whichever row order the patches land in.
#[trace("FR-352-AC-33")]
#[test]
fn stacked_defects_are_refused_at_the_earliest_failing_stage() {
    let vectors = vectors();
    // (earlier defect, later defect, applied later-first so row indices hold)
    let pairs = [
        (
            "MUW-R-04-document-unselected",
            "MUW-R-27-payload-node-wrong",
        ),
        ("MUW-R-11-extra-case", "MUW-R-27-payload-node-wrong"),
        ("MUW-R-18-member-identifier", "MUW-R-30-original-type-shape"),
        ("MUW-R-27-payload-node-wrong", "MUW-R-33-full-missing-row"),
    ];
    for (early, late) in pairs {
        let (base, extra) = if late.contains("R-33") {
            (early, late)
        } else {
            (late, early)
        };
        let mut case = resolve(&vectors, base);
        let operations = find(&vectors, extra)["patch"].clone();
        patch(&mut case.sidecar, &operations);
        let prepared = prepare_case(&vectors, "stacked", case, None).expect("prepares");
        let error = join(&prepared, &mut ample())
            .0
            .err()
            .unwrap_or_else(|| panic!("{early}+{late} must refuse"));
        let want = &find(&vectors, early)["expected"];
        let code = want["code"].as_str().unwrap();
        // The removal in the later defect shifts row 1 to row 0.
        let locus = want["locus"].as_str().unwrap();
        let shifted = locus.replacen("/model_unions/1/", "/model_unions/0/", 1);
        let locus = if late.contains("R-33") {
            shifted
        } else {
            locus.to_owned()
        };
        assert_eq!(
            (error.code.as_str(), error.form.as_deref()),
            (code, Some(locus.as_str())),
            "{early} before {late}"
        );
    }
}

/// A witness holding only the reached rows of a package whose model declares
/// more unions is not a complete witness: `validate_and_rebind` refuses it and
/// leaves the receiver untouched. (The published fixtures supply no complete
/// declaration environment, so the refusal here may precede the union
/// completeness stage, which no fixture reaches in isolation.)
#[trace("FR-352-AC-37")]
#[test]
fn rebinding_a_reached_only_witness_is_refused_and_never_mutates_the_receiver() {
    let vectors = vectors();
    let reached =
        prepare(&vectors, "MUW-03-reached-does-not-join-unrelated-row").expect("prepares");
    let partial = unwrap_join("MUW-03", join(&reached, &mut ample()).0);
    let before: Vec<_> = partial.model_union_rows().cloned().collect();
    assert!(
        !before.is_empty(),
        "the reached join retained its closure row"
    );
    partial
        .validate_and_rebind(&reached.admitted, &mut ample())
        .expect_err("a reached-only witness is not a complete witness");
    assert_eq!(
        partial.model_union_rows().cloned().collect::<Vec<_>>(),
        before,
        "the receiver is unchanged"
    );
}

/// A node of the selected package that no row names refuses `UnknownNode`
/// (published MUW-R-01); the same row naming a node of another package refuses
/// `ForeignNode`, at the same locus.
#[trace("FR-352-AC-34")]
#[test]
fn a_row_naming_another_packages_node_refuses_foreign_node() {
    let vectors = vectors();
    let mut case = resolve(&vectors, "MUW-01-full-environment");
    patch(
        &mut case.sidecar,
        &serde_json::json!([
            {"op": "replace", "path": "/model_unions/1/structural_node/package", "value": "elsewhere/pkg"}
        ]),
    );
    let prepared = prepare_case(&vectors, "foreign", case, None).expect("prepares");
    let error = join(&prepared, &mut ample())
        .0
        .expect_err("a foreign node refuses");
    assert_eq!(
        (error.code.as_str(), error.form.as_deref()),
        ("foreign_node", Some("/model_unions/1/structural_node"))
    );
}

// ---------------------------------------------------------------------------
// Quantity, object reference and composite payloads of the published mixed
// vector, and an enum-family payload, with their real witnesses.
// ---------------------------------------------------------------------------

fn case_payload<'a>(
    joined: &'a SelectedExactIdentities,
    union: &str,
    case: &str,
) -> &'a SelectedModelUnionPayload {
    let row = joined
        .model_union_rows()
        .find(|row| {
            matches!(row.structural_node.path.last(),
                Some(filament_contract_model::CheckedNodePathSegment::Name(name)) if name.ends_with(union))
        })
        .unwrap_or_else(|| panic!("{union}: a joined row"));
    &row.cases
        .iter()
        .find(|candidate| candidate.name.as_ref() == case)
        .unwrap_or_else(|| panic!("{case}: a case"))
        .payload
}

/// MUW-11: the unit-typed, object-typed and record, tuple and union payloads
/// each project the Exact type their witness names; a wrong unit, a wrong
/// object identity and a wrong composite key are each refused at the payload.
#[trace("FR-352-AC-32", "FR-352-AC-34")]
#[test]
fn quantity_object_and_composite_payloads_join_with_their_witnesses() {
    let vectors = vectors();
    let prepared = prepare(&vectors, "MUW-11-all-payload-forms").expect("prepares");
    let joined = unwrap_join("MUW-11", join(&prepared, &mut ample()).0);
    let key = |path: serde_json::Value| {
        key(&serde_json::json!({
            "package": "acme/shapes", "path": path, "ordinal": 0
        }))
    };
    let original = |union: &str, case: &str| match case_payload(&joined, union, case) {
        SelectedModelUnionPayload::Data { original_type, .. } => original_type.clone(),
        other => panic!("{case}: data expected, found {other:?}"),
    };
    assert_eq!(
        original("Mix", "Span"),
        ValueType::Quantity(UnitId::Declared(key(serde_json::json!([
            "scalar_type",
            "unit",
            "ix://acme/shapes/unit/ms"
        ]))))
    );
    assert_eq!(
        original("Mix", "Held"),
        ValueType::Reference(EffectiveId::new(
            None,
            OriginalDeclarationKey::new("acme/shapes", "ix://acme/shapes/Holder")
        ))
    );
    for (case, path) in [
        (
            "Wrap",
            ["model", "record_value_type", "ix://acme/shapes/Wrapper"],
        ),
        (
            "Pairing",
            ["model", "tuple_value_type", "ix://acme/shapes/Pair"],
        ),
        ("Next", ["model", "variant_type", "ix://acme/shapes/Link"]),
    ] {
        assert_eq!(
            original("Mix", case),
            ValueType::Composite(key(serde_json::json!(path))),
            "{case}"
        );
    }
    // Wrong witnesses: the Exact type changes, the node stays.
    let cases = |name: &str| -> usize {
        prepared.unions[1]
            .cases
            .iter()
            .position(|case| case.name.as_ref() == name)
            .expect("a Mix case")
    };
    for (case, wrong) in [
        (
            "Span",
            ValueType::Quantity(UnitId::Declared(key(serde_json::json!([
                "scalar_type",
                "unit",
                "ix://acme/shapes/unit/other"
            ])))),
        ),
        (
            "Held",
            ValueType::Reference(EffectiveId::new(
                None,
                OriginalDeclarationKey::new("acme/shapes", "ix://acme/shapes/Other"),
            )),
        ),
        (
            "Wrap",
            ValueType::Composite(key(serde_json::json!([
                "model",
                "record_value_type",
                "ix://acme/shapes/Node"
            ]))),
        ),
    ] {
        let mut tampered = prepare(&vectors, "MUW-11-all-payload-forms").expect("prepares");
        let index = cases(case);
        let model_unions_index = 1;
        let SelectedModelUnionPayload::Data { original_type, .. } =
            &mut tampered.unions[model_unions_index].cases[index].payload
        else {
            panic!("{case}: data");
        };
        *original_type = wrong;
        let error = join(&tampered, &mut ample())
            .0
            .expect_err("a wrong witness refuses");
        assert_eq!(
            (error.code.as_str(), error.form.as_deref()),
            (
                "mismatched_identity_witness",
                Some(format!("/model_unions/1/cases/{index}/payload/original_type").as_str())
            ),
            "{case}"
        );
    }
}

/// An enum-family payload (an all-nullary declaration named by a case) joins
/// with the real enum shape its enum rows carry; another Exact type is refused
/// at the payload, and without the enum rows the shape is unavailable.
#[trace("FR-352-AC-32", "FR-352-AC-34")]
#[test]
fn an_enum_family_payload_joins_with_its_enum_shape_witness() {
    let vectors = vectors();
    let color = serde_json::json!({
        "package": "acme/shapes",
        "path": ["model", "variant_type", "ix://acme/shapes/Color"],
        "ordinal": 0
    });
    // Mix.Bool names the enum-family declaration Color (Red only).
    let build = |with_enum_rows: bool, original: Option<ValueType>| {
        let mut case = resolve(&vectors, "MUW-11-all-payload-forms");
        case.document_patch = Some(serde_json::json!([
            {"op": "remove", "path": "/types/6/variants/1"},
            {"op": "replace", "path": "/types/12/variants/7/payloadType",
                "value": "ix://acme/shapes/Color"}
        ]));
        let mut prepared = prepare_case(&vectors, "enum-payload", case, None).expect("prepares");
        let (declarations, members, shapes) = color_enum_rows(&prepared);
        let shape = ValueType::Enum(shapes[0].shape.clone());
        if with_enum_rows {
            prepared.identities = prepared
                .identities
                .clone()
                .with_admitted_model_enum_rows(
                    &prepared.admitted,
                    SelectedModelEnumRows {
                        declarations: &declarations,
                        members: &members,
                        shapes: &shapes,
                    },
                    &mut ample(),
                )
                .expect("the enum rows admit");
        }
        let flag = prepared.unions[1]
            .cases
            .iter()
            .position(|candidate| candidate.name.as_ref() == "Bool")
            .expect("Mix.Bool");
        prepared.unions[1].cases[flag].payload = SelectedModelUnionPayload::Data {
            node: node_id(&color),
            original_type: original.unwrap_or_else(|| shape.clone()),
        };
        (prepared, flag, shape)
    };
    let (prepared, flag, expected) = build(true, None);
    let joined = unwrap_join("enum-payload", join(&prepared, &mut ample()).0);
    let SelectedModelUnionPayload::Data { original_type, .. } =
        case_payload(&joined, "Mix", "Bool")
    else {
        panic!("Bool carries data");
    };
    assert_eq!(original_type, &expected);
    // Another Exact type for the same enum node is refused at the payload.
    let (wrong, _, _) = build(true, Some(ValueType::Boolean));
    let error = join(&wrong, &mut ample())
        .0
        .expect_err("a wrong enum witness refuses");
    assert_eq!(
        (error.code.as_str(), error.form.as_deref()),
        (
            "mismatched_identity_witness",
            Some(format!("/model_unions/1/cases/{flag}/payload/original_type").as_str())
        )
    );
    // Without the enum rows the shape is not available.
    let (bare, _, _) = build(false, None);
    let error = join(&bare, &mut ample())
        .0
        .expect_err("the enum shape is unavailable without its rows");
    assert_eq!(error.code.as_str(), "unavailable_identity_witness");
}
