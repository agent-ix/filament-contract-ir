// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Genuine FSL emission evidence for direct selected model enum projection.

use super::*;
use crate::checked_package::v2::{
    CheckedPackageReadLimits, CheckedPackageV2ReadResult, SelectedModelEnumDeclarationRow,
    SelectedModelEnumMemberRow, SelectedModelEnumShapeCaseRow, SelectedModelEnumShapeRow,
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
    fixture_with_documents(checked, DOMAIN)
}

fn fixture_with_documents(
    checked: &[u8],
    domain: &[u8],
) -> (
    Box<CheckedPackageV2>,
    SelectedModelEnumShapeRow,
    Vec<SelectedModelEnumDeclarationRow>,
    Vec<SelectedModelEnumMemberRow>,
) {
    let mut evidence = CheckedPackageEvidence::new();
    evidence
        .insert_domain_package_document("example/config-version", domain)
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
        let identities = SelectedExactIdentities::from_stage_rows(&package, [], [], budget)
            .expect("identity selection binds to this admission")
            .with_admitted_model_enum_rows(
                &package,
                crate::SelectedModelEnumRows {
                    declarations: &declarations,
                    members: &members,
                    shapes: std::slice::from_ref(&row),
                },
                budget,
            )
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
    runtime_fixture_with(|_, _| {})
}

fn runtime_fixture_with(
    change: impl FnOnce(&mut Value, &mut Value),
) -> (
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
        "effect": {"modifies": [{"kind": "field", "declaration": object,
            "name": "versionNumber", "scope": "receiver"}],
            "creates": [], "deletes": [], "deletes_scoped": []},
        "provenance": {"origin": "model_document_operation_frame",
            "member": "ix://example/config-version/ConfigVersion/attemptUpdate"}
    }]);
    let mut domain: Value = serde_json::from_slice(DOMAIN).unwrap();
    change(&mut checked, &mut domain);
    fixture_with_documents(
        &serde_json::to_vec(&checked).unwrap(),
        &serde_json::to_vec(&domain).unwrap(),
    )
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
    let (package, shape, declarations, members) = runtime_fixture_with(|_, domain| {
        domain["types"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "identity": "ix://example/config-version/TierAlias", "kind": "alias",
                "target": "ix://example/config-version/Tier"
            }));
        let object = domain["types"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|ty| ty["identity"] == "ix://example/config-version/ConfigVersion")
            .unwrap();
        object["fields"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|field| field["name"] == "tier")
            .unwrap()["typeRef"] = "ix://example/config-version/TierAlias".into();
        let tier = object["fields"]
            .as_array()
            .unwrap()
            .iter()
            .find(|field| field["name"] == "tier")
            .unwrap()
            .clone();
        for name in ["previousTier", "nextTier"] {
            let mut field = tier.clone();
            field["identity"] = format!("ix://example/config-version/ConfigVersion/{name}").into();
            field["name"] = name.into();
            object["fields"].as_array_mut().unwrap().push(field);
        }
    });
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
        let units: Vec<_> = package
            .resolved_graph()
            .nodes
            .iter()
            .filter_map(|node| {
                let Some(CheckedNodeOwner::Source {
                    authority,
                    identity,
                }) = &node.owner
                else {
                    return None;
                };
                if node.semantic_form.as_ref() != "unit" {
                    return None;
                }
                let qualified = node.declaration.as_ref().unwrap().qualified_name.clone();
                Some(SelectedSourceUnitRow::new(
                    node.node_id.clone(),
                    authority.clone(),
                    identity.clone(),
                    qualified.clone(),
                    UnitId::declared(
                        NodeKey::new(
                            "reader-unit-fixture",
                            qualified
                                .iter()
                                .map(|part| PathSegment::Text(part.to_string()))
                                .collect(),
                            0,
                        )
                        .unwrap(),
                    ),
                ))
            })
            .collect();
        let types = package
            .project_selected_type_declarations(&identities, &[], &units, budget)
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
    for fields in [
        types.objects()[0].fields(),
        types.objects()[0].declared_fields(),
    ] {
        let enum_fields: Vec<_> = fields
            .iter()
            .filter(|field| ["tier", "previousTier", "nextTier"].contains(&field.name()))
            .collect();
        assert_eq!(enum_fields.len(), 3);
        for field in enum_fields {
            assert_eq!(field.value_type(), literal.value_type());
        }
    }
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
    let mut wrong_label = shape.clone();
    for (index, case) in wrong_label.cases.iter_mut().enumerate() {
        case.variant =
            VariantId::new(wrong_label.node_key.clone(), format!("Forged{index}")).unwrap();
    }
    wrong_label.shape = EnumShape::new(
        false,
        wrong_label.cases.iter().map(|case| case.variant.clone()),
    );
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
        (
            vec![wrong_label],
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
    for (declaration_rows, member_rows, expected) in [
        (
            &declarations[..0],
            &members[..],
            ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
        ),
        (
            &declarations[..],
            &members[..0],
            ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
        ),
    ] {
        let error = runtime_identities(&package, &mut budget)
            .with_admitted_model_enum_rows(
                &package,
                SelectedModelEnumRows {
                    declarations: declaration_rows,
                    members: member_rows,
                    shapes: std::slice::from_ref(&shape),
                },
                &mut budget,
            )
            .unwrap_err();
        assert_eq!(error.code, expected);
        assert_eq!(budget.last_success_delta(), None);
    }
    let rows = SelectedModelEnumRows {
        declarations: &declarations,
        members: &members,
        shapes: std::slice::from_ref(&shape),
    };
    let joined = runtime_identities(&package, &mut budget)
        .with_admitted_model_enum_rows(&package, rows, &mut budget)
        .unwrap();
    assert_eq!(
        joined
            .clone()
            .with_admitted_model_enum_rows(&package, rows, &mut budget)
            .unwrap(),
        joined
    );
    let mut changed_shape = shape.clone();
    changed_shape.shape = EnumShape::new(true, shape.shape.variants().cloned());
    let error = joined
        .with_admitted_model_enum_rows(
            &package,
            SelectedModelEnumRows {
                shapes: std::slice::from_ref(&changed_shape),
                ..rows
            },
            &mut budget,
        )
        .unwrap_err();
    assert_eq!(
        error.code,
        ExactTypeProjectionErrorCode::ConflictingIdentityWitness
    );
    assert_eq!(budget.last_success_delta(), None);
}

#[trace("FR-354-AC-1", "FR-354-AC-2", "FR-354-AC-6", "FR-352-AC-30")]
#[test]
fn model_enum_runtime_requires_unused_enum_and_keeps_same_case_declarations_distinct() {
    use crate::SelectedModelEnumRows;
    let other = "ix://example/config-version/OtherTier";
    let (package, shape, mut declarations, mut members) =
        runtime_fixture_with(|checked, domain| {
            let mut ty = domain["types"]
                .as_array()
                .unwrap()
                .iter()
                .find(|ty| ty["identity"] == "ix://example/config-version/Tier")
                .unwrap()
                .clone();
            ty["identity"] = other.into();
            for case in ty["variants"].as_array_mut().unwrap() {
                case["identity"] = format!("{other}/{}", case["name"].as_str().unwrap()).into();
            }
            domain["types"].as_array_mut().unwrap().push(ty);
            let original_nodes = checked["semantic_graph"]["nodes"]
                .as_array()
                .unwrap()
                .clone();
            let mut declaration = original_nodes
                .iter()
                .find(|node| node["semantic_form"] == "variant_type")
                .unwrap()
                .clone();
            declaration["node_id"]["path"][2] = other.into();
            let id = declaration["node_id"].clone();
            declaration["semantic_type"] = id.clone();
            declaration["owner"]["node"] = other.into();
            declaration["occurrences"] = serde_json::json!([{"role":"generated", "ordinal":0}]);
            let mut correspondence = checked["source_map"][0].clone();
            correspondence["node_id"] = id.clone();
            checked["source_map"]
                .as_array_mut()
                .unwrap()
                .push(correspondence);
            checked["semantic_graph"]["nodes"]
                .as_array_mut()
                .unwrap()
                .push(declaration);
            for node in original_nodes
                .iter()
                .filter(|node| node["semantic_form"] == "enum_value")
            {
                let mut node = node.clone();
                node["node_id"] = serde_json::json!({"package":"unused-fixture",
                    "path":["value", "enum_value", other, node["body"]["value"]], "ordinal":0});
                node["semantic_type"] = id.clone();
                node["body"]["type"] = id.clone();
                node["dependencies"] = serde_json::json!([id]);
                node["occurrences"] = serde_json::json!([{"role":"generated", "ordinal":0}]);
                let mut correspondence = checked["source_map"][0].clone();
                correspondence["node_id"] = node["node_id"].clone();
                checked["source_map"]
                    .as_array_mut()
                    .unwrap()
                    .push(correspondence);
                checked["semantic_graph"]["nodes"]
                    .as_array_mut()
                    .unwrap()
                    .push(node);
            }
        });
    let mut budget = NativeJoinBudget::new(u64::MAX, u64::MAX);
    let error = runtime_identities(&package, &mut budget)
        .with_admitted_model_enum_rows(
            &package,
            SelectedModelEnumRows {
                declarations: &declarations,
                members: &members,
                shapes: std::slice::from_ref(&shape),
            },
            &mut budget,
        )
        .unwrap_err();
    assert_eq!(
        error.code,
        ExactTypeProjectionErrorCode::UnavailableIdentityWitness
    );
    let other_node = package.resolved_graph().nodes.iter().find(|node|
        matches!(&node.owner, Some(CheckedNodeOwner::Model {node, ..}) if node.as_ref() == other)).unwrap().node_id.clone();
    let other_key = NodeKey::new(
        "reader-fixture",
        vec![PathSegment::Text("OtherTier".into())],
        0,
    )
    .unwrap();
    declarations.push(SelectedModelEnumDeclarationRow::new(
        other_node.clone(),
        "example/config-version",
        other,
        other_key.clone(),
    ));
    let mut other_shape = shape.clone();
    other_shape.structural_node = other_node.clone();
    other_shape.node_key = other_key.clone();
    other_shape.original_identity = other.into();
    for case in &mut other_shape.cases {
        case.structural_node = package.resolved_graph().nodes.iter().find(|node|
            node.semantic_type == other_node && matches!(package.term(&node.node_id), Some(CheckedTerm::Literal {value, ..}) if value.as_str() == Some(case.case.as_ref())))
            .unwrap().node_id.clone();
        case.member_original_identity = format!("{other}/{}", case.case).into();
        case.member_key = NodeKey::new(
            "reader-fixture",
            vec![PathSegment::Text(format!("OtherTier/{}", case.case))],
            0,
        )
        .unwrap();
        case.variant = VariantId::new(other_key.clone(), case.case.as_ref()).unwrap();
        members.push(SelectedModelEnumMemberRow::new(
            case.structural_node.clone(),
            other_node.clone(),
            "example/config-version",
            other,
            case.member_original_identity.clone(),
            case.case.clone(),
            case.member_key.clone(),
        ));
    }
    other_shape.shape = EnumShape::new(
        true,
        other_shape.cases.iter().map(|case| case.variant.clone()),
    );
    let shapes = [shape.clone(), other_shape.clone()];
    assert_eq!(
        other_shape
            .shape
            .variants()
            .map(|variant| variant.case())
            .collect::<Vec<_>>(),
        ["Gold", "Standard"]
    );
    assert!(other_shape.shape.is_ordered());
    let identities = runtime_identities(&package, &mut budget)
        .with_admitted_model_enum_rows(
            &package,
            SelectedModelEnumRows {
                declarations: &declarations,
                members: &members,
                shapes: &shapes,
            },
            &mut budget,
        )
        .unwrap();
    let first = package
        .project_exact_type(&shape.cases[0].structural_node, &identities, &mut budget)
        .unwrap();
    let second = package
        .project_exact_type(
            &other_shape.cases[0].structural_node,
            &identities,
            &mut budget,
        )
        .unwrap();
    assert_ne!(first.value_type(), second.value_type());
    assert_eq!(second.value_type(), &ValueType::Enum(other_shape.shape));
    assert_eq!(shape.cases[0].case, other_shape.cases[0].case);
}

#[trace("FR-354-AC-2", "FR-354-AC-6", "FR-354-AC-8")]
#[test]
fn model_enum_runtime_payload_variant_refuses_without_becoming_an_enum() {
    use crate::SelectedModelEnumRows;
    let union = "ix://example/config-version/Result";
    let (package, shape, declarations, members) = runtime_fixture_with(|checked, domain| {
        let mut ty = domain["types"]
            .as_array()
            .unwrap()
            .iter()
            .find(|ty| ty["identity"] == "ix://example/config-version/Tier")
            .unwrap()
            .clone();
        ty["identity"] = union.into();
        for case in ty["variants"].as_array_mut().unwrap() {
            case["identity"] = format!("{union}/{}", case["name"].as_str().unwrap()).into();
            case["payloadType"] = "ix://filament/native/Integer".into();
        }
        domain["types"].as_array_mut().unwrap().push(ty);
        let mut node = checked["semantic_graph"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|node| node["semantic_form"] == "variant_type")
            .unwrap()
            .clone();
        node["node_id"]["path"][2] = union.into();
        node["semantic_type"] = node["node_id"].clone();
        node["owner"]["node"] = union.into();
        node["occurrences"] = serde_json::json!([{"role":"generated", "ordinal":0}]);
        let mut correspondence = checked["source_map"][0].clone();
        correspondence["node_id"] = node["node_id"].clone();
        checked["source_map"]
            .as_array_mut()
            .unwrap()
            .push(correspondence);
        checked["semantic_graph"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(node);
    });
    let mut budget = NativeJoinBudget::new(u64::MAX, u64::MAX);
    let identities = runtime_identities(&package, &mut budget)
        .with_admitted_model_enum_rows(
            &package,
            SelectedModelEnumRows {
                declarations: &declarations,
                members: &members,
                shapes: std::slice::from_ref(&shape),
            },
            &mut budget,
        )
        .unwrap();
    let refusal = package
        .project_selected_type_declarations(&identities, &[], &[], &mut budget)
        .unwrap_err();
    // FR-830 supersedes the old unsupported gap: a payload union is witnessed by
    // its own model-union row, never by an enum row, so the absent row is the
    // named missing-witness refusal at the same locus.
    assert_eq!(
        refusal.code,
        ExactTypeProjectionErrorCode::UnavailableIdentityWitness
    );
    assert_eq!(refusal.form.as_deref(), Some("selected_union"));
    assert_eq!(budget.last_success_delta(), None);
    let node = package.resolved_graph().nodes.iter().find(|node|
        matches!(&node.owner, Some(CheckedNodeOwner::Model {node, ..}) if node.as_ref() == union)).unwrap().node_id.clone();
    let mut false_declarations = declarations;
    false_declarations.push(SelectedModelEnumDeclarationRow::new(
        node,
        "example/config-version",
        union,
        NodeKey::new(
            "reader-fixture",
            vec![PathSegment::Text("Result".into())],
            0,
        )
        .unwrap(),
    ));
    let error = runtime_identities(&package, &mut budget)
        .with_admitted_model_enum_rows(
            &package,
            SelectedModelEnumRows {
                declarations: &false_declarations,
                members: &members,
                shapes: std::slice::from_ref(&shape),
            },
            &mut budget,
        )
        .unwrap_err();
    assert_eq!(
        error.code,
        ExactTypeProjectionErrorCode::MismatchedIdentityWitness
    );
}

#[trace("FR-354-AC-6", "FR-354-AC-2", "FR-352-AC-30")]
#[test]
fn model_enum_runtime_authenticates_case_order_against_selected_declaration() {
    use crate::SelectedModelEnumRows;
    for ordered in [false, true] {
        let (package, mut row, declarations, members) = runtime_fixture();
        row.shape = EnumShape::new(ordered, row.cases.iter().map(|case| case.variant.clone()));
        let join = |row: &SelectedModelEnumShapeRow, budget: &mut NativeJoinBudget| {
            runtime_identities(&package, budget).with_admitted_model_enum_rows(
                &package,
                SelectedModelEnumRows {
                    declarations: &declarations,
                    members: &members,
                    shapes: std::slice::from_ref(row),
                },
                budget,
            )
        };
        let mut budget = NativeJoinBudget::new(u64::MAX, u64::MAX);
        join(&row, &mut budget)
            .expect("Gold precedes Standard in both canonical and authored order");
        row.cases.reverse();
        row.shape = EnumShape::new(ordered, row.cases.iter().map(|case| case.variant.clone()));
        let error = join(&row, &mut budget)
            .expect_err("permuting both channels cannot change admitted ranks");
        assert_eq!(
            error.code,
            ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
            "ordered={ordered}"
        );
        assert_eq!(budget.last_success_delta(), None);
    }

    let (package, mut row, declarations, members) = runtime_fixture_with(|_, domain| {
        let tier = domain["types"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|ty| ty["identity"] == "ix://example/config-version/Tier")
            .unwrap();
        tier["variants"].as_array_mut().unwrap().reverse();
    });
    let mut budget = NativeJoinBudget::new(u64::MAX, u64::MAX);
    let join = |row: &SelectedModelEnumShapeRow, budget: &mut NativeJoinBudget| {
        runtime_identities(&package, budget).with_admitted_model_enum_rows(
            &package,
            SelectedModelEnumRows {
                declarations: &declarations,
                members: &members,
                shapes: std::slice::from_ref(row),
            },
            budget,
        )
    };
    join(&row, &mut budget)
        .expect("unordered cases keep UTF-8 order even when authored in reverse");
    row.shape = EnumShape::new(true, row.cases.iter().map(|case| case.variant.clone()));
    assert_eq!(
        join(&row, &mut budget).unwrap_err().code,
        ExactTypeProjectionErrorCode::MismatchedIdentityWitness
    );
    row.cases.reverse();
    row.shape = EnumShape::new(true, row.cases.iter().map(|case| case.variant.clone()));
    let identities = join(&row, &mut budget)
        .expect("ordered cases preserve the selected document's authored positions");
    let projected = package
        .project_exact_type(&row.structural_node, &identities, &mut budget)
        .unwrap();
    assert_eq!(projected.value_type(), &ValueType::Enum(row.shape));
}

#[trace("FR-358-AC-3", "FR-354-AC-1", "FR-354-AC-7", "FR-352-AC-30")]
#[test]
fn model_enum_runtime_field_copies_pay_for_every_variant_and_case_byte() {
    use crate::SelectedModelEnumRows;
    let (package, row, declarations, members) = runtime_fixture();
    let mut setup = NativeJoinBudget::new(u64::MAX, u64::MAX);
    let identities = runtime_identities(&package, &mut setup)
        .with_admitted_model_enum_rows(
            &package,
            SelectedModelEnumRows {
                declarations: &declarations,
                members: &members,
                shapes: std::slice::from_ref(&row),
            },
            &mut setup,
        )
        .unwrap();
    let (position, _) = package.node_with_position(&row.structural_node).unwrap();
    let read = |budget: &mut NativeJoinBudget| {
        budget.set_active_locus(Some(crate::NativeJoinLocus {
            source: crate::NativeJoinLocusSource::GraphNode(position),
            form: "enum_field_copy",
        }));
        (0..3)
            .map(|_| {
                project_record_field_type(
                    &package,
                    &identities,
                    &MemberType::Variant(row.structural_node.clone()),
                    &row.structural_node,
                    budget,
                )
            })
            .collect::<Result<Vec<_>, _>>()
    };
    // Each copy inspects two variants and 12 case bytes: 16 work units.
    // Storage owns one ValueType, one vector, two variant slots, two strings
    // and 12 string bytes: 18 units. Three fields must pay this three times.
    let mut ample = NativeJoinBudget::new(u64::MAX, u64::MAX);
    let values = read(&mut ample).unwrap();
    assert_eq!(values, vec![ValueType::Enum(row.shape.clone()); 3]);
    assert_eq!(
        ample.consumed(),
        NativeJoinCharge {
            work: 48,
            storage: 54
        }
    );
    let mut exact = NativeJoinBudget::new(48, 54);
    assert_eq!(read(&mut exact).unwrap(), values);
    for (work, storage, kind) in [
        (47, 54, NativeJoinChargeKind::Work),
        (48, 53, NativeJoinChargeKind::Storage),
    ] {
        let mut short = NativeJoinBudget::new(work, storage);
        let error =
            read(&mut short).expect_err("a one-short budget cannot return all three field types");
        assert_eq!(error.denied.unwrap().kind, kind);
        assert_eq!(short.last_success_delta(), None);
        let locus = short.first_unpaid_locus().unwrap();
        assert_eq!(
            locus.source,
            crate::NativeJoinLocusSource::GraphNode(position)
        );
        assert_eq!(
            locus.form,
            match kind {
                NativeJoinChargeKind::Work => "native_enum_footprint",
                NativeJoinChargeKind::Storage => "record_field_variant",
            }
        );
    }
}
