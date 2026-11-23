//! The admitted V2 body vocabulary is available as Rust variants.

use crate::support::checked_package::{
    canonical, evidence_for, positive_operation_identities, v2_all_families,
};
use filament_contract_model::{
    encode_checked_package_v2, CheckedApplicationOperation, CheckedArtifactRef, CheckedBindingName,
    CheckedBindingNameKind, CheckedFairnessGranularity, CheckedFairnessKind, CheckedLiteralValue,
    CheckedOperationMember, CheckedOperationViewError, CheckedPackageReadLimits,
    CheckedPackageSourceCorrespondenceRef, CheckedPackageV2ReadResult, CheckedTerm, ExecutionPoint,
    ExecutionPointName, OperationMemberKind, StateClauseKind,
};
use ix_trace_rs::trace;
use serde_json::json;
use std::collections::BTreeSet;

#[trace(
    "TC-048",
    "FR-038-AC-331",
    "FR-038-AC-332",
    "FR-038-AC-333",
    "FR-038-AC-334",
    "FR-038-AC-341",
    "FR-040-AC-37"
)]
#[test]
fn public_deserialization_does_not_admit_operation_views() {
    let forged = json!({
        "identity": "not-in-the-operation-catalog",
        "laws": [
            {"role": "ieee_profile", "definition": {"authority": "agent-ix", "identity": "unselected"}},
            {"role": "ieee_profile", "definition": false}
        ],
        "mode": {"kind": "rounding", "value": "arbitrary"},
        "member": {"kind": "field", "declaration": {
            "package": "forged", "path": ["field"], "ordinal": 0
        }, "name": "forged"},
        "leaves": []
    });
    let operation: CheckedApplicationOperation =
        serde_json::from_value(forged.clone()).expect("public decode retains the wire record");
    assert_eq!(operation.identity(), Some("not-in-the-operation-catalog"));
    assert_eq!(
        operation.state_clause_kind(),
        Err(CheckedOperationViewError::NotAdmitted)
    );
    assert_eq!(
        operation.projection_field(),
        Err(CheckedOperationViewError::NotAdmitted)
    );
    assert_eq!(
        operation.member_declaration(),
        Err(CheckedOperationViewError::NotAdmitted)
    );
    assert_eq!(
        operation.mode(),
        Err(CheckedOperationViewError::NotAdmitted)
    );
    assert_eq!(
        operation.law_definitions(),
        Err(CheckedOperationViewError::NotAdmitted)
    );
    assert_eq!(
        serde_json::to_value(&operation).expect("wire round trip"),
        forged
    );

    let clause = json!({
        "identity": "filament.op.state.clause", "laws": [], "mode": null,
        "member": {"kind": "state_clause", "clause": "invariant"}, "leaves": []
    });
    let decoded: CheckedApplicationOperation =
        serde_json::from_value(clause.clone()).expect("public clause decode");
    assert_eq!(
        decoded.state_clause_kind(),
        Err(CheckedOperationViewError::NotAdmitted)
    );
    assert_eq!(
        serde_json::to_value(&decoded).expect("clause wire round trip"),
        clause
    );
    assert_eq!(
        CheckedApplicationOperation::state_clause(StateClauseKind::Invariant, None)
            .state_clause_kind(),
        Ok(Some(StateClauseKind::Invariant)),
    );

    let absent: CheckedApplicationOperation = serde_json::from_value(json!({
        "identity": "filament.op.function.call", "laws": [], "mode": null,
        "member": null, "leaves": []
    }))
    .expect("closed operation record");
    assert_eq!(
        absent.state_clause_kind(),
        Err(CheckedOperationViewError::NotAdmitted)
    );
    assert_eq!(
        absent.projection_field(),
        Err(CheckedOperationViewError::NotAdmitted)
    );
    assert_eq!(
        absent.member_declaration(),
        Err(CheckedOperationViewError::NotAdmitted)
    );
    assert_eq!(absent.mode(), Err(CheckedOperationViewError::NotAdmitted));
    assert_eq!(
        absent.law_definitions(),
        Err(CheckedOperationViewError::NotAdmitted)
    );
}

#[trace("TC-047", "FR-035-AC-22", "FR-035-AC-25")]
#[test]
fn operation_equality_uses_record_and_execution_point_not_view_provenance() {
    let admitted = CheckedApplicationOperation::state_clause(
        StateClauseKind::Invariant,
        Some(ExecutionPoint::Pre {
            operation: ExecutionPointName::new("step").expect("point name"),
        }),
    );
    let decoded: CheckedApplicationOperation =
        serde_json::from_value(serde_json::to_value(&admitted).expect("operation wire"))
            .expect("operation decode");
    assert_eq!(admitted, decoded);
    assert_eq!(decoded.mode(), Err(CheckedOperationViewError::NotAdmitted));
    assert_eq!(admitted.mode(), Ok(None));
    let other_point = CheckedApplicationOperation::state_clause(
        StateClauseKind::Invariant,
        Some(ExecutionPoint::Post {
            operation: ExecutionPointName::new("step").expect("point name"),
        }),
    );
    assert_ne!(admitted, other_point);
}

#[trace("FR-349-AC-1", "FR-349-AC-3")]
#[test]
fn admitted_v2_bodies_are_closed_typed_terms() {
    let fixture = v2_all_families();
    let admitted = match crate::read_and_account(
        &canonical(&fixture),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&fixture),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("positive V2 fixture refused: {other:?}"),
    };

    let mut saw_aggregate = false;
    let mut saw_reference = false;
    let mut saw_application_argument = false;
    let mut saw_nested_binding = false;
    for node in &admitted.graph().nodes {
        let body: &CheckedTerm = &node.body;
        let term = admitted
            .term(&node.node_id)
            .expect("every admitted node has a typed body");
        assert_eq!(
            term, body,
            "typed body preserves the admitted wire value for {:?}",
            node.node_id
        );
        match term {
            CheckedTerm::Aggregate { members, .. } => {
                saw_aggregate = true;
                saw_nested_binding |= members.iter().any(|member| {
                    matches!(member, CheckedTerm::Binding { value, .. }
                        if matches!(value.as_ref(), CheckedTerm::Literal { .. }))
                });
            }
            CheckedTerm::Reference { .. } => saw_reference = true,
            CheckedTerm::Application { arguments, .. } => {
                saw_application_argument |= arguments
                    .iter()
                    .any(|argument| matches!(argument, CheckedTerm::Reference { .. }));
            }
            _ => {}
        }
    }
    assert!(saw_aggregate, "positive fixture includes an aggregate body");
    assert!(saw_reference, "positive fixture includes a reference root");
    assert!(
        saw_application_argument,
        "application has a typed reference argument"
    );
    assert!(
        saw_nested_binding,
        "aggregate has a typed binding and literal"
    );
    let literal_fixture = positive_operation_identities();
    let literal_package = match crate::read_and_account(
        &canonical(&literal_fixture),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&literal_fixture),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("literal V2 fixture refused: {other:?}"),
    };
    assert!(literal_package.graph().nodes.iter().any(|node| {
        matches!(
            literal_package.term(&node.node_id),
            Some(CheckedTerm::Literal { .. })
        )
    }));
    assert_eq!(
        admitted.name(),
        fixture["name"].as_str().expect("package name")
    );
}

#[trace("FR-349-AC-6", "FR-349-AC-7", "FR-349-AC-9")]
#[test]
fn public_literal_and_binding_reads_preserve_exact_values() {
    for (name, expected) in [
        ("parameters", CheckedBindingNameKind::Parameters),
        ("body", CheckedBindingNameKind::Body),
        ("optional", CheckedBindingNameKind::Optional),
        ("clause", CheckedBindingNameKind::Clause),
        ("other.name", CheckedBindingNameKind::Other("other.name")),
    ] {
        let binding = CheckedBindingName::from(name);
        assert_eq!(binding.kind(), expected);
        assert_eq!(binding.as_ref(), name);
    }

    let fixture = positive_operation_identities();
    let bytes = canonical(&fixture);
    let admitted = match crate::read_and_account(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&fixture),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("positive V2 fixture refused: {other:?}"),
    };
    let mut saw_boolean = false;
    for node in &admitted.graph().nodes {
        if let Some(literal) = node.body.literal_read() {
            assert_eq!(literal.type_id, &node.semantic_type);
            if let CheckedLiteralValue::Boolean(true) = literal.value {
                saw_boolean = true;
            }
        }
    }
    assert!(saw_boolean, "fixture contains an admitted Boolean literal");
    let encoded = encode_checked_package_v2(
        admitted.name(),
        admitted.lock(),
        admitted.graph(),
        CheckedPackageSourceCorrespondenceRef {
            source_map: admitted.source_map(),
            refinement_step_rows: admitted.refinement_step_rows(),
        },
        filament_contract_model::CheckedPackageSidecarsRef {
            temporal_clause_names: admitted.temporal_clause_names(),
            claim_names: admitted.claim_names(),
            operation_effects: None,
            operation_bodies: None,
            checked_term_data: admitted.checked_term_data(),
            temporal_bindings: admitted.temporal_bindings(),
            schedulability_claims: None,
        },
        admitted.capability_report(),
        admitted.diagnostics(),
        quire_canonical::Limits::new(u64::MAX),
    )
    .expect("encode admitted package");
    assert_eq!(encoded, bytes);
}

#[trace("FR-349-AC-6")]
#[test]
fn public_literal_read_covers_the_closed_payload_vocabulary() {
    let type_id = filament_contract_model::CheckedNodeId {
        package: "literal-test".into(),
        path: vec![filament_contract_model::CheckedNodePathSegment::Name(
            "type".into(),
        )],
        ordinal: 0,
    };
    let cases = [
        ("boolean", json!(true)),
        ("integer", json!("123456789012345678901234567890")),
        ("rational", json!("-5/7")),
        ("quantity", json!("12345678901234567890/7")),
        ("decimal", json!("1.00")),
        ("float32_bits", json!("80000000")),
        ("float64_bits", json!("7ff8000000000001")),
        ("text", json!("text")),
        ("string", json!("string")),
        ("uuid", json!("00000000-0000-0000-0000-000000000000")),
        (
            "timestamp",
            json!("170141183460469231731687303715884105727"),
        ),
        ("enum", json!("READY")),
        ("none", json!(null)),
        ("reference", json!("7331")),
    ];
    assert_eq!(cases.len(), filament_contract_model::LiteralKind::ALL.len());
    for (kind, value) in cases {
        let term = CheckedTerm::Literal {
            type_id: type_id.clone(),
            value_kind: filament_contract_model::LiteralKind::from_wire(kind)
                .expect("closed literal kind"),
            value: filament_contract_model::CheckedJson::from_serializable(&value)
                .expect("a literal"),
        };
        let read = term
            .literal_read()
            .expect("every closed kind has a typed value");
        assert_eq!(read.type_id, &type_id);
        assert_eq!(read.kind.as_wire(), kind);
        match (kind, read.value) {
            ("boolean", CheckedLiteralValue::Boolean(true)) => {}
            ("integer", CheckedLiteralValue::Integer(value)) => {
                assert_eq!(value.to_string(), "123456789012345678901234567890")
            }
            (
                "rational",
                CheckedLiteralValue::Rational {
                    numerator,
                    denominator,
                },
            ) => {
                assert_eq!(
                    (numerator.to_string(), denominator.to_string()),
                    ("-5".into(), "7".into())
                );
            }
            (
                "quantity",
                CheckedLiteralValue::Rational {
                    numerator,
                    denominator,
                },
            ) => {
                assert_eq!(
                    (numerator.to_string(), denominator.to_string()),
                    ("12345678901234567890".into(), "7".into())
                );
            }
            ("decimal", CheckedLiteralValue::Decimal { coefficient, scale }) => {
                assert_eq!(coefficient.to_string(), "100");
                assert_eq!(scale, 2);
            }
            ("float32_bits", CheckedLiteralValue::Float32Bits(0x80000000))
            | ("float64_bits", CheckedLiteralValue::Float64Bits(0x7ff8000000000001))
            | ("text", CheckedLiteralValue::Text("text"))
            | ("string", CheckedLiteralValue::String("string"))
            | ("uuid", CheckedLiteralValue::Uuid("00000000-0000-0000-0000-000000000000"))
            | ("timestamp", CheckedLiteralValue::Timestamp(i128::MAX))
            | ("enum", CheckedLiteralValue::Enum("READY"))
            | ("none", CheckedLiteralValue::None) => {}
            ("reference", CheckedLiteralValue::Reference("7331")) => {}
            (_, other) => panic!("{kind} mapped to {other:?}"),
        }
    }
}

/// Trace: FR-480-AC-3, TC-965-RL-06.
/// Provenance: IR-1559.
#[trace("TC-965", "FR-480-AC-3")]
#[test]
fn reference_literal_read_preserves_the_canonical_target_identity() {
    let type_id = filament_contract_model::CheckedNodeId {
        package: "literal-test".into(),
        path: vec![filament_contract_model::CheckedNodePathSegment::Name(
            "reference-type".into(),
        )],
        ordinal: 0,
    };
    let term = CheckedTerm::Literal {
        type_id: type_id.clone(),
        value_kind: filament_contract_model::LiteralKind::Reference,
        value: filament_contract_model::CheckedJson::from_serializable(&"7331")
            .expect("reference identity"),
    };

    let read = term.literal_read().expect("reference literal read");
    assert_eq!(read.type_id, &type_id);
    assert_eq!(read.kind, filament_contract_model::LiteralKind::Reference);
    assert_eq!(read.value, CheckedLiteralValue::Reference("7331"));
}

#[trace("FR-349-AC-8", "FR-349-AC-9")]
#[test]
fn public_operation_read_exposes_admitted_records_and_rejects_forgery() {
    let fixture = positive_operation_identities();
    let admitted = match crate::read_and_account(
        &canonical(&fixture),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&fixture),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("positive V2 fixture refused: {other:?}"),
    };
    let mut count = 0;
    for node in &admitted.graph().nodes {
        if let CheckedTerm::Application { operation, .. } = &node.body {
            let read = operation
                .read()
                .expect("reader installed complete admission view");
            assert_eq!(Some(read.identity), operation.identity());
            assert_eq!(
                read.laws
                    .iter()
                    .map(|law| &law.definition)
                    .collect::<Vec<_>>(),
                operation
                    .law_definitions()
                    .expect("admitted laws")
                    .iter()
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                read.mode.map(|mode| (mode.kind, mode.value)),
                operation
                    .mode()
                    .expect("admitted mode")
                    .map(|mode| (mode.kind, mode.value))
            );
            if let Some(CheckedOperationMember::Field {
                declaration, name, ..
            }) = read.member
            {
                assert_eq!(
                    operation.projection_field().expect("admitted field"),
                    Some((declaration, name.as_ref()))
                );
            }
            assert!(
                read.constraints.is_empty(),
                "function call has no catalog constraints"
            );
            count += 1;
        }
    }
    assert!(count > 0, "fixture contains applications");

    let forged: CheckedApplicationOperation = serde_json::from_value(json!({
        "identity": "filament.op.function.call", "laws": [], "mode": null,
        "member": null, "leaves": []
    }))
    .expect("direct decode");
    assert!(matches!(
        forged.read(),
        Err(CheckedOperationViewError::NotAdmitted)
    ));

    #[cfg(feature = "test-support")]
    {
        let fixture = filament_contract_model::CheckedApplicationOperationFixture::default();
        let operation =
            CheckedApplicationOperation::from_test_fixture(fixture).expect("valid test fixture");
        assert!(matches!(
            operation.read(),
            Err(CheckedOperationViewError::NotAdmitted)
        ));
    }
}

#[trace("FR-349-AC-8", "FR-349-AC-9", "TC-1862")]
#[test]
fn admitted_controls_expose_missing_catalog_constraint_productions() {
    let catalog: serde_json::Value = serde_json::from_str(
        filament_verification_contracts::operation_catalog::CHECKED_OPERATION_CATALOG_V1,
    )
    .expect("authoritative catalog");
    let controls = [
        ("filament.op.collection.fold", "bound_is_member"),
        ("filament.op.reference.eq", "conforming_reference"),
        ("filament.op.numeric.convert", "exact_conversion"),
        ("filament.op.numeric.narrow", "range_narrowing"),
        ("filament.op.rational.narrow", "rational_narrowing"),
        ("filament.op.quantity.convert", "same_dimension"),
        ("filament.op.numeric.convert_rounding", "scale_reduction"),
        ("filament.op.collection.sequence", "uniform_rest"),
        ("filament.op.model.reaches_field", "reference_edge"),
        (
            "filament.op.collection.sum.carried_quantity",
            "carried_member",
        ),
    ];
    for (identity, required_kind) in controls {
        let package = match identity {
            "filament.op.collection.sequence" => {
                crate::checked_package_v2_composite_operands::admitted_sequence_control()
            }
            "filament.op.model.reaches_field" => {
                crate::checked_package_v2_model_members::admitted_reaches_field_control()
            }
            _ => crate::checked_package_v2_catalogue_differential::admitted_catalogue_control(
                identity,
            ),
        };
        let operation = package
            .graph()
            .nodes
            .iter()
            .find_map(|node| match &node.body {
                CheckedTerm::Application { operation, .. }
                    if operation.identity() == Some(identity) =>
                {
                    Some(operation)
                }
                _ => None,
            })
            .expect("admitted target operation");
        let read = operation.read().expect("public admission record");
        assert_eq!(read.identity, identity);
        let entry = catalog["operations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["identity"] == identity)
            .expect("catalog entry");
        let constraints = entry["constraints"]
            .as_array()
            .expect("catalog constraints");
        assert_eq!(read.constraints.len(), constraints.len(), "{identity}");
        for (actual, expected) in read.constraints.iter().zip(constraints) {
            assert_eq!(actual.kind.as_wire(), expected["kind"].as_str().unwrap());
            let expected_operands = expected["operands"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_u64().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(actual.operands, expected_operands);
            assert_eq!(actual.family.as_deref(), expected["family"].as_str());
        }
        assert!(
            read.constraints
                .iter()
                .any(|constraint| constraint.kind.as_wire() == required_kind),
            "{identity}: required production"
        );
    }
    let catalog_kinds: BTreeSet<&str> = catalog["operations"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|entry| entry["constraints"].as_array().unwrap())
        .map(|constraint| constraint["kind"].as_str().unwrap())
        .collect();
    let fspec_positive_kinds = [
        "bound_family",
        "inner_type",
        "member_family",
        "member_of",
        "promotes_exact",
        "same_family",
        "same_type",
        "union_arms",
    ];
    // The separate FR-772 decoder and operation tests own these productions;
    // this census has no admitted full-package option.slots control because it
    // requires a selected timed-model root and a resolved clock place. The
    // same holds for `clock_reference.ref`, the only entry with the
    // place_identity_terms and place_has_id productions (FR-829).
    let reader_specific_kinds = [
        "place_slot_terms",
        "place_identity_terms",
        "place_has_id",
        "quantifier_domain_only",
    ];
    let covered: BTreeSet<&str> = fspec_positive_kinds
        .into_iter()
        .chain(controls.into_iter().map(|(_, kind)| kind))
        .chain(reader_specific_kinds)
        .collect();
    assert_eq!(
        catalog_kinds, covered,
        "all authoritative constraint productions have admitted controls"
    );
}

#[trace("FR-349-AC-8", "FR-349-AC-9")]
#[test]
fn admitted_controls_expose_element_and_fairness_members() {
    let package = crate::checked_package_v2_catalogue_differential::admitted_catalogue_control(
        "filament.op.collection.index",
    );
    let operation = package
        .graph()
        .nodes
        .iter()
        .find_map(|node| match &node.body {
            CheckedTerm::Application { operation, .. }
                if operation.identity() == Some("filament.op.collection.index") =>
            {
                Some(operation)
            }
            _ => None,
        })
        .expect("admitted index operation");
    let CheckedOperationMember::Element { declaration } = operation.read().unwrap().member.unwrap()
    else {
        panic!("admitted index member must be Element")
    };
    let wire = serde_json::to_value(operation).unwrap();
    assert_eq!(
        serde_json::to_value(declaration).unwrap(),
        wire["member"]["declaration"]
    );

    let package = crate::checked_package_v2_temporal::admitted_fairness_source_for_contract();
    let operation = package
        .graph()
        .nodes
        .iter()
        .find_map(|node| match &node.body {
            CheckedTerm::Application { operation, .. }
                if operation.identity() == Some("filament.op.temporal.fair") =>
            {
                Some(operation)
            }
            _ => None,
        })
        .expect("admitted fairness operation");
    let CheckedOperationMember::Fairness {
        fairness_kind,
        granularity,
        target,
    } = operation.read().unwrap().member.unwrap()
    else {
        panic!("admitted fair member must be Fairness")
    };
    let wire = serde_json::to_value(operation).unwrap();
    assert_eq!(*fairness_kind, CheckedFairnessKind::Weak);
    assert_eq!(*granularity, CheckedFairnessGranularity::Whole);
    assert_eq!(
        serde_json::to_value(target).unwrap(),
        wire["member"]["target"]
    );
    let filament_contract_model::CheckedFairnessTarget::Operation { declaration, .. } = target
    else {
        panic!("operation target")
    };
    assert_eq!(operation.member_declaration().unwrap(), Some(declaration));
    let member_kinds: BTreeSet<&str> = OperationMemberKind::ALL
        .iter()
        .map(|kind| kind.as_wire())
        .collect();
    let covered: BTreeSet<&str> = [
        "field",
        "operation",
        "position",
        "profile_operator",
        "relationship_end",
        "state_clause",
        "temporal_interval",
        "type_argument",
        "element",
        "fairness",
        "history",
    ]
    .into_iter()
    .collect();
    assert_eq!(
        member_kinds, covered,
        "all closed member productions have admitted controls"
    );
}

#[trace("TC-2028", "FR-322-AC-464")]
#[test]
fn an_aggregate_temporal_profile_round_trips_and_an_absent_one_stays_absent() {
    let profile = json!({
        "authority": "agent-ix",
        "identity": "filament.temporal.event-position.false-extension/v1"
    });
    let with = json!({"term": "aggregate", "members": [], "temporal_profile": profile});
    let term: CheckedTerm = serde_json::from_value(with.clone()).expect("aggregate with profile");
    assert_eq!(
        term,
        CheckedTerm::Aggregate {
            members: vec![],
            temporal_profile: Some(CheckedArtifactRef {
                authority: "agent-ix".into(),
                identity: "filament.temporal.event-position.false-extension/v1".into(),
            }),
        }
    );
    assert_eq!(serde_json::to_value(&term).expect("serialize"), with);

    let bare = json!({"term": "aggregate", "members": []});
    let term: CheckedTerm = serde_json::from_value(bare.clone()).expect("bare aggregate");
    assert_eq!(
        term,
        CheckedTerm::Aggregate {
            members: vec![],
            temporal_profile: None,
        }
    );
    assert_eq!(serde_json::to_value(&term).expect("serialize"), bare);

    for refused in [
        json!({"term": "aggregate", "members": [], "extra": 1}),
        json!({"term": "aggregate", "members": [], "temporal_profile": "text"}),
        json!({"term": "aggregate", "members": [], "temporal_profile": {"authority": "a"}}),
    ] {
        assert!(serde_json::from_value::<CheckedTerm>(refused).is_err());
    }
}
