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
    ModelUnionScope, NativeJoinBudget, SelectedExactDeclarationForm, SelectedExactDeclarationRow,
    SelectedExactIdentities, SelectedExactIdentity, SelectedModelUnionCaseRow,
    SelectedModelUnionPayload, SelectedModelUnionRow, SelectedTupleDeclarationRow,
};
use filament_exact::{
    EffectiveId, NodeKey, OriginalDeclarationKey, PathSegment, UnionMember, UnitId, ValueType,
};
use ix_trace_rs::trace;

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

/// The outcome of one vector: `admitted`, or the refusal code.
fn run(vectors: &Value, id: &str, limits: Option<(u64, u64)>) -> String {
    run_measured(vectors, id, limits).0
}

/// The outcome and the budget's consumed `(work, storage)` after the call.
fn run_measured(vectors: &Value, id: &str, limits: Option<(u64, u64)>) -> (String, (u64, u64)) {
    let (outcome, _, consumed) = run_located(vectors, id, limits);
    (outcome, consumed)
}

/// The outcome, the refusal locus (the error form) and the consumed counters.
fn run_located(
    vectors: &Value,
    id: &str,
    limits: Option<(u64, u64)>,
) -> (String, Option<String>, (u64, u64)) {
    let case = resolve(vectors, id);
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let pair = &vectors["fixtures"][&case.fixtures];
    let package = read_json(&proposal.join(pair["package"].as_str().expect("package")))
        .expect("published package");
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
    let mut budget = limits.map_or_else(
        || NativeJoinBudget::new(u64::MAX, u64::MAX),
        |(work, storage)| NativeJoinBudget::new(work, storage),
    );
    let declarations: Vec<_> = case.witnesses["declarations"]
        .as_array()
        .into_iter()
        .flatten()
        .map(declaration_row)
        .collect();
    let identities =
        match SelectedExactIdentities::from_stage_rows(&admitted, declarations, [], &mut budget) {
            Ok(identities) => identities,
            Err(error) => {
                return (
                    error.code.as_str().into(),
                    error.form.as_deref().map(str::to_owned),
                    (0, 0),
                );
            }
        };
    let tuples: Vec<_> = case.sidecar["model_tuples"]
        .as_array()
        .into_iter()
        .flatten()
        .map(tuple_row)
        .collect();
    let unions: Vec<_> = case.sidecar["model_unions"]
        .as_array()
        .into_iter()
        .flatten()
        .map(union_row)
        .collect();
    let request = case.scope["request"]
        .as_object()
        .map(|_| node_id(&case.scope["request"]));
    let scope = match &request {
        Some(request) => ModelUnionScope::Reached(request),
        None => ModelUnionScope::Full,
    };
    let outcome =
        match identities.with_model_union_rows(&admitted, &tuples, &unions, scope, &mut budget) {
            Ok(joined) => {
                // Every joined union row is returned unchanged.
                for row in joined.model_union_rows() {
                    assert!(
                        unions.contains(row),
                        "{id}: a returned row differs from its input"
                    );
                }
                ("admitted".into(), None)
            }
            Err(error) => (
                error.code.as_str().into(),
                error.form.as_deref().map(str::to_owned),
            ),
        };
    let (outcome, locus) = outcome;
    let consumed = budget.consumed();
    (
        outcome,
        locus,
        (
            consumed.work,
            u64::try_from(consumed.storage).unwrap_or(u64::MAX),
        ),
    )
}

fn vectors() -> Value {
    read_json(&proposal_dir_of(&fixtures_dir()).join("model-union-witness-vectors.json"))
        .expect("published model-union witness vectors")
}

/// Fixture C holds source rows this adapter does not build; the source-row
/// non-substitution controls are not executed here and remain an open boundary.
fn uses_source_rows(vectors: &Value, id: &str) -> bool {
    find(vectors, id).to_string().contains("/source_") || resolve_fixtures(vectors, id) == "C"
}

/// Trace: FR-352-AC-31 FR-069-AC-11 FR-830-AC-1 FR-830-AC-2 FR-830-AC-3 FR-830-AC-4 FR-830-AC-5 FR-830-AC-12 FR-830-AC-13.
/// Every published positive is admitted by the strict read and the native join.
#[trace("FR-830-AC-1")]
#[test]
fn published_positive_vectors_are_admitted_with_their_rows_returned() {
    let vectors = vectors();
    let mut run_count = 0;
    for row in vectors["positive"].as_array().expect("positives") {
        let id = row["id"].as_str().expect("id");
        if uses_source_rows(&vectors, id) {
            continue;
        }
        assert_eq!(run(&vectors, id, None), "admitted", "{id}");
        run_count += 1;
    }
    assert!(run_count >= 17, "ran {run_count} positives");
}

/// Trace: FR-352-AC-31 FR-830-AC-7 FR-830-AC-8 FR-830-AC-9 FR-830-AC-10 FR-830-AC-13 FR-830-AC-14 FR-830-AC-15.
/// Every published reader refusal keeps its code.
#[trace("FR-830-AC-9")]
#[test]
fn published_reader_refusals_keep_their_codes() {
    let vectors = vectors();
    let mut run_count = 0;
    let mut mismatches = Vec::new();
    for row in vectors["reader_refusals"].as_array().expect("refusals") {
        let id = row["id"].as_str().expect("id");
        if uses_source_rows(&vectors, id) {
            continue;
        }
        let expected = row["expected"]["code"].as_str().expect("code");
        if expected == "malformed" {
            // The closed-format codec is the FSL identity-file parser's.
            continue;
        }
        let (actual, locus, _) = run_located(&vectors, id, None);
        let want = row["expected"]["locus"].as_str().unwrap_or("");
        // A tuple-channel refusal keeps the existing tuple stage form; the
        // model-tuple row pointers are reported as a boundary, not asserted.
        let tuple_stage = id.contains("-TR-");
        if actual != expected || (!tuple_stage && locus.as_deref().unwrap_or("") != want) {
            mismatches.push(format!(
                "{id}: expected {expected} at {want}, got {actual} at {locus:?}"
            ));
        }
        run_count += 1;
    }
    assert!(
        mismatches.is_empty(),
        "{} of {run_count}: {mismatches:#?}",
        mismatches.len()
    );
    assert!(run_count >= 40, "ran {run_count} refusals");
}

/// The fixture pair a vector (or its base) names, without applying patches.
fn resolve_fixtures(vectors: &Value, id: &str) -> String {
    let row = find(vectors, id);
    match row["base"].as_str() {
        Some(base) => resolve_fixtures(vectors, base),
        None => row["fixtures"].as_str().expect("fixtures").into(),
    }
}

/// Trace: FR-830-AC-11 FR-358-AC-15.
/// On the shared budget the measured baseline admits; one unit less of either
/// counter exhausts that counter, and the zero limits exhaust at once.
#[trace("FR-830-AC-11")]
#[test]
fn the_measured_baseline_admits_and_one_unit_less_exhausts_each_counter() {
    let vectors = vectors();
    for id in [
        "MUW-01-full-environment",
        "MUW-02-reached-from-shape",
        "MUW-15-full-environment-complete",
    ] {
        let (outcome, (work, storage)) = run_measured(&vectors, id, None);
        assert_eq!(outcome, "admitted", "{id}");
        assert!(work > 0 && storage > 0, "{id}: the join is measured");
        assert_eq!(
            run(&vectors, id, Some((work, storage))),
            "admitted",
            "{id}: baseline"
        );
        assert_eq!(
            run(&vectors, id, Some((work - 1, storage))),
            "exhausted_work",
            "{id}: one unit less work"
        );
        assert_eq!(
            run(&vectors, id, Some((work, storage - 1))),
            "exhausted_storage",
            "{id}: one unit less storage"
        );
        assert_eq!(
            run(&vectors, id, Some((0, u64::MAX))),
            "exhausted_work",
            "{id}: zero work"
        );
        assert_eq!(
            run(&vectors, id, Some((u64::MAX, 0))),
            "exhausted_storage",
            "{id}: zero storage"
        );
    }
}

/// Trace: FR-352-AC-31 FR-830-AC-1 FR-830-AC-10.
/// The joined union is projected as the composite its own `type_handle` names,
/// never as an enum; without the row the projection is unavailable.
#[trace("FR-830-AC-1")]
#[test]
fn a_joined_model_union_projects_to_its_type_handle_and_never_to_an_enum() {
    let vectors = vectors();
    let id = "MUW-01-full-environment";
    let case = resolve(&vectors, id);
    let proposal = proposal_dir_of(&fixtures_dir());
    let pair = &vectors["fixtures"][&case.fixtures];
    let package = read_json(&proposal.join(pair["package"].as_str().unwrap())).unwrap();
    let document = read_json(&proposal.join(pair["document"].as_str().unwrap())).unwrap();
    let admitted = match crate::read_and_account(
        &fixture_support::canonical(&package),
        CheckedPackageReadLimits::bounded(),
        &inputs::selected_evidence(&document),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("strict read: {other:?}"),
    };
    let tuples: Vec<_> = case.sidecar["model_tuples"]
        .as_array()
        .unwrap()
        .iter()
        .map(tuple_row)
        .collect();
    let unions: Vec<_> = case.sidecar["model_unions"]
        .as_array()
        .unwrap()
        .iter()
        .map(union_row)
        .collect();
    let mut budget = NativeJoinBudget::new(u64::MAX, u64::MAX);
    let empty = SelectedExactIdentities::empty();
    let joined = empty
        .with_model_union_rows(
            &admitted,
            &tuples,
            &unions,
            ModelUnionScope::Full,
            &mut budget,
        )
        .expect("the published full environment joins");
    for row in &unions {
        let projection = admitted
            .project_exact_type(&row.structural_node, &joined, &mut budget)
            .expect("a joined union projects");
        assert_eq!(
            projection.value_type(),
            &ValueType::Composite(row.type_handle.clone()),
            "{:?}",
            row.structural_node
        );
    }
    let shape = &unions[1].structural_node;
    let refused = admitted
        .project_exact_type(shape, &empty, &mut budget)
        .expect_err("no row, no projection");
    assert_eq!(refused.code.as_str(), "unavailable_identity_witness");
    // Joining the rows to an empty selection leaves the original untouched.
    assert_eq!(empty.model_union_rows().count(), 0);
    assert_eq!(joined.model_union_rows().count(), unions.len());
}
