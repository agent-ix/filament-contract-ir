// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Genuine FSL emission evidence for direct selected model enum projection.

use super::*;
use crate::checked_package::v2::{
    CheckedPackageReadLimits, CheckedPackageV2ReadResult, SelectedModelEnumDeclarationRow,
    SelectedModelEnumMemberRow, SelectedModelEnumShapeCaseRow, SelectedModelEnumShapeRow,
    SelectedModelEnums,
};
use crate::CheckedPackageEvidence;
use filament_exact::{PathSegment, VariantId};
use ix_trace_rs::trace;
use serde_json::Value;

const CHECKED: &[u8] =
    include_bytes!("../../../../../../tests/fixtures/model-enum-projection/checked.json");
const DOMAIN: &[u8] =
    include_bytes!("../../../../../../tests/fixtures/model-enum-projection/domain.json");
const ROWS: &str =
    include_str!("../../../../../../tests/fixtures/model-enum-projection/producer-rows.json");

fn text<'a>(value: &'a Value, name: &str) -> &'a str {
    value[name].as_str().expect("exported producer text")
}

fn node(value: &Value) -> CheckedNodeId {
    serde_json::from_value(value.clone()).expect("exported structural node")
}

// Decode the producer's original NodeKey serialization, including nested keys;
// this does not convert an IR CheckedNodeId into an opaque identity.
fn key(value: &Value) -> NodeKey {
    let path = value["path"]
        .as_array()
        .expect("exported key path")
        .iter()
        .map(|segment| match segment.as_str() {
            Some(name) => PathSegment::Text(name.into()),
            None => PathSegment::Node(key(segment)),
        })
        .collect();
    NodeKey::new(
        text(value, "package"),
        path,
        value["ordinal"].as_u64().expect("exported ordinal"),
    )
    .expect("original producer key")
}

fn variant(value: &Value) -> VariantId {
    VariantId::new(key(&value["owner"]), text(value, "case")).expect("original producer variant")
}

fn fixture() -> (
    Box<CheckedPackageV2>,
    SelectedModelEnumShapeRow,
    Vec<SelectedModelEnumDeclarationRow>,
    Vec<SelectedModelEnumMemberRow>,
) {
    fixture_with_checked(CHECKED)
}

fn fixture_with_checked(
    checked: &[u8],
) -> (
    Box<CheckedPackageV2>,
    SelectedModelEnumShapeRow,
    Vec<SelectedModelEnumDeclarationRow>,
    Vec<SelectedModelEnumMemberRow>,
) {
    let mut evidence = CheckedPackageEvidence::new();
    evidence
        .insert_domain_package_document("example/config-version", DOMAIN)
        .unwrap();
    evidence.support_feature("filament.value.complete/v1");
    let package = match CheckedPackageV2::read(
        &crate::checked_package::v2::monitor_plan::genuine_bytes_under_current_rules(checked),
        CheckedPackageReadLimits {
            bytes: 67_108_864,
            nodes: 1_000_000,
            edges: 10_000_000,
            occurrences: 10_000_000,
            diagnostics: 10_000,
            work: 1_000_000_000,
        },
        &evidence,
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("genuine final-emission Tier package must admit: {other:?}"),
    };
    let rows: Value = serde_json::from_str(ROWS).unwrap();
    let declarations = rows["declarations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            SelectedModelEnumDeclarationRow::new(
                node(&row["structural_node"]),
                text(row, "selected_document"),
                text(row, "original_identity"),
                key(&row["node_key"]),
            )
        })
        .collect();
    let members = rows["members"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            SelectedModelEnumMemberRow::new(
                node(&row["structural_node"]),
                node(&row["declaration_node"]),
                text(row, "selected_document"),
                text(row, "enum_original_identity"),
                text(row, "member_original_identity"),
                text(row, "case"),
                key(&row["member_key"]),
            )
        })
        .collect();
    assert_eq!(rows["shapes"].as_array().unwrap().len(), 1);
    let row = &rows["shapes"][0];
    let shape = SelectedModelEnumShapeRow {
        structural_node: node(&row["structural_node"]),
        node_key: key(&row["node_key"]),
        selected_document: text(row, "selected_document").into(),
        original_identity: text(row, "original_identity").into(),
        shape: EnumShape::new(
            row["shape"]["ordered"].as_bool().unwrap(),
            row["shape"]["variants"]
                .as_array()
                .unwrap()
                .iter()
                .map(variant),
        ),
        cases: row["cases"]
            .as_array()
            .unwrap()
            .iter()
            .map(|case| SelectedModelEnumShapeCaseRow {
                structural_node: node(&case["structural_node"]),
                case: text(case, "case").into(),
                member_key: key(&case["member_key"]),
                member_original_identity: text(case, "member_original_identity").into(),
                variant: variant(&case["variant"]),
            })
            .collect(),
    };
    (package, shape, declarations, members)
}

#[trace("TC-470", "FR-352-AC-30")]
#[test]
fn genuine_tier_projects_the_authenticated_installed_producer_shape() {
    let (package, row, declarations, members) = fixture();
    let project = |budget: &mut NativeJoinBudget| {
        let enums = SelectedModelEnums::from_stage_rows_runtime(
            &package,
            declarations.iter().cloned(),
            members.iter().cloned(),
            budget,
        )
        .expect("original declaration and case rows join admitted model");
        enums
            .read_shape_runtime(&package, &row, budget)
            .expect("complete original producer shape authenticates");
        let identities = SelectedExactIdentities::from_stage_rows(&package, [], [], budget)
            .expect("identity selection binds to this admission")
            .with_model_enum_shapes(std::slice::from_ref(&row), budget)
            .expect("install the checked producer shape");
        package.project_exact_type(&row.structural_node, &identities, budget)
    };
    let mut budget = NativeJoinBudget::new(u64::MAX, u64::MAX);
    let projected = project(&mut budget).expect("genuine Tier direct projection");
    assert_eq!(projected.value_type(), &ValueType::Enum(row.shape.clone()));
    assert_eq!(projected.type_node(), &row.structural_node);
    let ValueType::Enum(shape) = projected.value_type() else {
        panic!("Tier must project to Enum")
    };
    assert!(!shape.is_ordered());
    assert_eq!(
        shape
            .variants()
            .map(|variant| variant.case())
            .collect::<Vec<_>>(),
        ["Gold", "Standard"]
    );
    assert_eq!(
        shape
            .variants()
            .map(|variant| variant.owner())
            .collect::<Vec<_>>(),
        [&row.node_key, &row.node_key]
    );
    let used = budget.consumed();
    let mut exact = NativeJoinBudget::new(used.work, used.storage);
    assert_eq!(project(&mut exact), Ok(projected));
    assert_eq!(exact.consumed(), used);
    for (work, storage, kind) in [
        (used.work - 1, u64::MAX, NativeJoinChargeKind::Work),
        (u64::MAX, used.storage - 1, NativeJoinChargeKind::Storage),
    ] {
        let mut short = NativeJoinBudget::new(work, storage);
        let error = project(&mut short).expect_err("one-short budget denies the final projection");
        assert_eq!(error.denied, short.first_unpaid());
        assert_eq!(error.denied.unwrap().kind, kind);
        assert!(short.first_unpaid_locus().is_some());
        assert_eq!(short.last_success_delta(), None);
    }
}

fn runtime_fixture() -> (
    Box<CheckedPackageV2>,
    SelectedModelEnumShapeRow,
    Vec<SelectedModelEnumDeclarationRow>,
    Vec<SelectedModelEnumMemberRow>,
) {
    // Supply the separate operation-effect channel required by the full view.
    let mut checked: Value = serde_json::from_slice(CHECKED).unwrap();
    let object = checked["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["semantic_form"] == "object_type")
        .unwrap()["node_id"]
        .clone();
    checked["operation_effects"] = serde_json::json!([{
        "document": "example/config-version", "declaring": object,
        "operation": "attemptUpdate",
        "effect": {"modifies": [], "creates": [], "deletes": [], "deletes_scoped": []},
        "provenance": {"origin": "model_document_operation_frame",
            "member": "ix://example/config-version/ConfigVersion/attemptUpdate"}
    }]);
    fixture_with_checked(&serde_json::to_vec(&checked).unwrap())
}

fn runtime_identities(
    package: &CheckedPackageV2,
    budget: &mut NativeJoinBudget,
) -> SelectedExactIdentities {
    use filament_exact::OriginalDeclarationKey;
    let object = package
        .resolved_graph()
        .nodes
        .iter()
        .find(|node| node.semantic_form.as_ref() == "object_type")
        .unwrap();
    let original = "ix://example/config-version/ConfigVersion";
    SelectedExactIdentities::from_stage_rows(
        package,
        [SelectedExactDeclarationRow::new(
            object.node_id.clone(),
            "example/config-version",
            SelectedExactDeclarationForm::ObjectType,
            original,
            SelectedExactIdentity::Object(EffectiveId::new(
                None,
                OriginalDeclarationKey::new("example/config-version", original),
            )),
        )],
        [],
        budget,
    )
    .unwrap()
}

#[trace("FR-354-AC-1", "FR-354-AC-6", "FR-354-AC-7", "FR-352-AC-30")]
#[test]
fn model_enum_runtime_full_view_matches_direct_field_and_literal_types_with_cumulative_budget() {
    use crate::SelectedModelEnumRows;
    let (package, shape, declarations, members) = runtime_fixture();
    let read = |package: &CheckedPackageV2, budget: &mut NativeJoinBudget| {
        let identities = runtime_identities(package, budget).with_admitted_model_enum_rows(
            package,
            SelectedModelEnumRows {
                declarations: &declarations,
                members: &members,
                shapes: std::slice::from_ref(&shape),
            },
            budget,
        )?;
        let types = package
            .project_selected_type_declarations(&identities, &[], &[], budget)
            .map_err(|error| ExactTypeProjectionError {
                code: error.code,
                requested: error.at.clone(),
                at: error.at,
                form: error.form,
                work: error.work,
                denied: error.denied,
                configured_bound: None,
            })?;
        let literal =
            package.project_exact_type(&members[0].structural_node, &identities, budget)?;
        Ok::<_, ExactTypeProjectionError>((types, literal))
    };
    let mut ample = NativeJoinBudget::new(u64::MAX, u64::MAX);
    let (types, literal) = read(&package, &mut ample).expect("complete enum environment");
    assert_eq!(types.enums(), std::slice::from_ref(&shape));
    assert_eq!(literal.value_type(), &ValueType::Enum(shape.shape.clone()));
    let field = types.objects()[0]
        .fields()
        .iter()
        .find(|field| field.name() == "tier")
        .unwrap();
    assert_eq!(field.value_type(), literal.value_type());
    let used = ample.consumed();
    let mut exact = NativeJoinBudget::new(used.work, used.storage);
    assert_eq!(
        read(&package, &mut exact).unwrap(),
        (types.clone(), literal.clone())
    );
    assert_eq!(
        read(
            &package.clone(),
            &mut NativeJoinBudget::new(used.work, used.storage)
        )
        .unwrap(),
        (types, literal)
    );
    for (work, storage, kind) in [
        (used.work - 1, u64::MAX, NativeJoinChargeKind::Work),
        (u64::MAX, used.storage - 1, NativeJoinChargeKind::Storage),
    ] {
        let mut short = NativeJoinBudget::new(work, storage);
        let error = read(&package, &mut short).expect_err("one-short cumulative read is atomic");
        assert_eq!(error.denied.unwrap().kind, kind);
        assert!(short.first_unpaid_locus().is_some());
        assert_eq!(short.last_success_delta(), None);
    }
}

#[trace("FR-354-AC-2", "FR-354-AC-6", "FR-352-AC-30")]
#[test]
fn model_enum_runtime_rejects_missing_conflicting_foreign_and_incomplete_witnesses() {
    use crate::SelectedModelEnumRows;
    let (package, shape, declarations, members) = runtime_fixture();
    let mut missing_case = shape.clone();
    missing_case.cases.pop();
    missing_case.shape = EnumShape::new(
        false,
        missing_case.cases.iter().map(|case| case.variant.clone()),
    );
    let mut foreign = shape.clone();
    foreign.selected_document = "foreign/document".into();
    for (rows, expected) in [
        (
            vec![],
            ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
        ),
        (
            vec![shape.clone(), shape.clone()],
            ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
        ),
        (
            vec![foreign],
            ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
        ),
        (
            vec![missing_case],
            ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
        ),
    ] {
        let mut budget = NativeJoinBudget::new(u64::MAX, u64::MAX);
        let identities = runtime_identities(&package, &mut budget);
        let error = identities
            .with_admitted_model_enum_rows(
                &package,
                SelectedModelEnumRows {
                    declarations: &declarations,
                    members: &members,
                    shapes: &rows,
                },
                &mut budget,
            )
            .expect_err("incomplete or foreign evidence cannot expose any table");
        assert_eq!(error.code, expected);
        assert_eq!(budget.last_success_delta(), None);
    }
    let mut budget = NativeJoinBudget::new(u64::MAX, u64::MAX);
    let error = package
        .project_selected_type_declarations(
            &runtime_identities(&package, &mut budget),
            &[],
            &[],
            &mut budget,
        )
        .expect_err("a selected enum cannot be silently omitted from a full view");
    assert_eq!(
        error.code,
        ExactTypeProjectionErrorCode::UnavailableIdentityWitness
    );
    assert_eq!(error.at.as_deref(), Some(&shape.structural_node));
}
