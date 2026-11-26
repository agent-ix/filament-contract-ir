// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Structural dimension-equivalence admission and refusal controls. FSpec's
//! three-package XP-01 graph runs in the conformance target.

use crate::support::checked_package::{
    admitted_dependency, canonical, evidence_for, fixture_source, index_fixture_node_ids, pointer,
    read_with_dependencies, refresh_identity, v2_all_families,
};
use filament_contract_model::{
    encode_checked_package_v2, CheckedDimensionOperand, CheckedNodeId, CheckedNodeKind,
    CheckedPackageReadLimits, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, CheckedPackageSourceCorrespondenceRef, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedRational, CheckedTerm, ScalarTypeForm,
};
use ix_trace_rs::trace;
use serde_json::{json, Value};

fn dimension(name: &str, source: &Value) -> Value {
    let id = json!({"package":"test/families","path":["Measure",name],"ordinal":0});
    json!({
        "node_id":id,
        "schema_version":"filament.checked-semantic-graph/v2",
        "node_tag":"scalar_type", "semantic_form":"dimension",
        "semantic_type":id, "dependencies":[],
        "declaration":{"qualified_name":["Measure",name]},
        "occurrences":[{"role":"declaration","ordinal":0}],
        "owner":{"kind":"source","authority":source["authority"],"identity":source["identity"]},
        "body":{"term":"dimension","base":true,"terms":[]}
    })
}

fn equivalence(left: &Value, right: &Value, source: &Value) -> Value {
    let id = json!({
        "package":"test/families",
        "path":["scalar_type","dimension_equivalence",left,right],
        "ordinal":0
    });
    json!({
        "node_id":id,
        "schema_version":"filament.checked-semantic-graph/v2",
        "node_tag":"scalar_type", "semantic_form":"dimension_equivalence",
        "semantic_type":id, "dependencies":[left,right],
        "occurrences":[{"role":"anchor","ordinal":0}],
        "owner":{"kind":"source","authority":source["authority"],"identity":source["identity"]},
        "body":{"term":"dimension_equivalence","dimension_node_ids":[left,right],
            "root_scale":{"numerator":"1","denominator":"1"}}
    })
}

fn named_equivalence(
    edge_name: &str,
    left: &Value,
    right: &Value,
    source: &Value,
    numerator: &str,
    denominator: &str,
) -> Value {
    let id = json!({
        "package":"test/families",
        "path":["Equivalence",edge_name],
        "ordinal":0
    });
    json!({
        "node_id":id,
        "schema_version":"filament.checked-semantic-graph/v2",
        "node_tag":"scalar_type", "semantic_form":"dimension_equivalence",
        "semantic_type":id, "dependencies":[left,right],
        "occurrences":[{"role":"anchor","ordinal":0}],
        "owner":{"kind":"source","authority":source["authority"],"identity":source["identity"]},
        "body":{"term":"dimension_equivalence","dimension_node_ids":[left,right],
            "root_scale":{"numerator":numerator,"denominator":denominator}}
    })
}

pub(super) fn package() -> Value {
    let mut package = v2_all_families();
    let source = fixture_source();
    let scalar = package["semantic_graph"]["nodes"][0].clone();
    let alpha = dimension("Alpha", &source);
    let beta = dimension("Beta", &source);
    let pair = equivalence(&alpha["node_id"], &beta["node_id"], &source);
    let nodes = vec![scalar, alpha, beta, pair];
    let source_map = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            json!({
                "node_id":node["node_id"],
                "role":node["occurrences"][0]["role"],
                "ordinal":0,
                "regions":[{"source":source,"start":index,"end":index+1}]
            })
        })
        .collect::<Vec<_>>();
    package["semantic_graph"]["nodes"] = json!(nodes);
    package["source_map"] = json!(source_map);
    index_fixture_node_ids(&mut package);
    crate::support::checked_package::refresh_temporal_bindings(&mut package);
    package
}

fn read(value: &Value) -> CheckedPackageV2ReadResult {
    crate::read_and_account(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(value),
    )
}

fn expect_refusal(value: &Value, code: Code, cause: Option<Cause>, at: &str) {
    let CheckedPackageV2ReadResult::Refused(refusal) = read(value) else {
        panic!("adverse equivalence package must refuse");
    };
    assert_eq!(refusal.code, code, "{refusal:?}");
    assert_eq!(refusal.cause, cause, "{refusal:?}");
    assert_eq!(refusal.loci[0].path, Some(pointer(at)), "{refusal:?}");
}

fn rename_package(value: &mut Value, old: &str, new: &str) {
    match value {
        Value::String(text) if text == old => *text = new.to_owned(),
        Value::Array(items) => {
            for item in items {
                rename_package(item, old, new);
            }
        }
        Value::Object(members) => {
            for item in members.values_mut() {
                rename_package(item, old, new);
            }
        }
        _ => {}
    }
}

fn mixed_package() -> (Value, String, CheckedPackageV2) {
    let (dependency_name, dependency) = admitted_dependency(&package());
    let mut importer = package();
    rename_package(&mut importer, "test/families", "test/importer");
    let local = importer["semantic_graph"]["nodes"][1]["node_id"].clone();
    let foreign = dependency.document_held_id(2).expect("graph node").clone();
    let foreign = serde_json::to_value(foreign).expect("foreign id");
    let equivalence = &mut importer["semantic_graph"]["nodes"][3];
    let pair = json!([{"package":dependency_name,"node":foreign}, local]);
    let id = json!({
        "package":"test/importer",
        "path":["scalar_type","dimension_equivalence","acme/clock","Beta","test/importer","Alpha"],
        "ordinal":0
    });
    equivalence["node_id"] = id.clone();
    equivalence["semantic_type"] = id.clone();
    equivalence["dependencies"] = json!([local]);
    equivalence["body"]["dimension_node_ids"] = pair;
    importer["source_map"][3]["node_id"] = id;
    importer["lock"]["dependency_selections"] = json!([{"identity":dependency_name}]);
    refresh_identity(&mut importer);
    (importer, dependency_name, dependency)
}

#[trace("TC-280", "FR-038-AC-387", "FR-322-AC-299")]
#[test]
fn selected_dimension_operand_admits_and_retains_package_form() {
    let (wire, name, dependency) = mixed_package();
    let CheckedPackageV2ReadResult::Admitted(admitted) =
        read_with_dependencies(&wire, &[(name.as_str(), &dependency)])
    else {
        panic!("directly selected foreign dimension must admit");
    };
    let retained = serde_json::to_value(&admitted.graph().nodes[3]).expect("retained node");
    assert_eq!(
        retained["body"]["dimension_node_ids"],
        wire["semantic_graph"]["nodes"][3]["body"]["dimension_node_ids"]
    );
    assert_eq!(
        retained["dependencies"],
        wire["semantic_graph"]["nodes"][3]["dependencies"]
    );
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
    .expect("canonical writer admits the mixed dimension operands");
    let encoded: Value = serde_json::from_slice(&encoded).expect("canonical package JSON");
    assert_eq!(
        encoded["semantic_graph"]["nodes"][3]["body"]["dimension_node_ids"],
        wire["semantic_graph"]["nodes"][3]["body"]["dimension_node_ids"],
        "canonical writer preserves both authored operand forms"
    );
    assert_eq!(
        encoded["semantic_graph"]["nodes"][3]["dependencies"],
        wire["semantic_graph"]["nodes"][3]["dependencies"],
        "canonical writer lists only the local operand as a graph dependency"
    );
}

#[trace("TC-280", "FR-038-AC-388")]
#[test]
fn package_operand_requires_direct_selection_and_declared_node() {
    let (wire, name, dependency) = mixed_package();
    let mut unselected = wire.clone();
    unselected["lock"]["dependency_selections"] = json!([]);
    let CheckedPackageV2ReadResult::Refused(refusal) = read_with_dependencies(&unselected, &[])
    else {
        panic!("unselected package operand must refuse");
    };
    assert_eq!(refusal.code, Code::MissingDeclaration, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::MissingSelection), "{refusal:?}");
    assert_eq!(
        refusal.loci[0].path,
        Some(pointer(
            "/semantic_graph/nodes/3/body/dimension_node_ids/0/package"
        )),
        "{refusal:?}"
    );

    let mut missing = wire.clone();
    missing["semantic_graph"]["nodes"][3]["body"]["dimension_node_ids"][0]["node"]["path"] =
        json!(["Missing"]);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with_dependencies(&missing, &[(name.as_str(), &dependency)])
    else {
        panic!("absent selected node must refuse");
    };
    assert_eq!(refusal.code, Code::MissingDeclaration, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::MissingName), "{refusal:?}");
    assert_eq!(
        refusal.loci[0].path,
        Some(pointer(
            "/semantic_graph/nodes/3/body/dimension_node_ids/0/node"
        )),
        "{refusal:?}"
    );

    let mut wrong_package = wire;
    wrong_package["semantic_graph"]["nodes"][3]["body"]["dimension_node_ids"][0]["node"]
        ["package"] = json!("another/package");
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with_dependencies(&wrong_package, &[(name.as_str(), &dependency)])
    else {
        panic!("a node outside the selected package must refuse");
    };
    assert_eq!(refusal.code, Code::MissingDeclaration, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::MissingName), "{refusal:?}");
    assert_eq!(
        refusal.loci[0].path,
        Some(pointer(
            "/semantic_graph/nodes/3/body/dimension_node_ids/0/node"
        )),
        "{refusal:?}"
    );
}

#[trace("TC-280", "FR-038-AC-389")]
#[test]
fn foreign_operand_wrong_kind_and_self_minted_local_refuse_at_operand() {
    let (wire, name, dependency) = mixed_package();
    let mut wrong_kind = wire.clone();
    wrong_kind["semantic_graph"]["nodes"][3]["body"]["dimension_node_ids"][0]["node"] =
        serde_json::to_value(dependency.document_held_id(0).expect("graph node").clone())
            .expect("non-dimension id");
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with_dependencies(&wrong_kind, &[(name.as_str(), &dependency)])
    else {
        panic!("selected non-dimension operand must refuse");
    };
    assert_eq!(refusal.code, Code::InvalidSemanticGraph, "{refusal:?}");
    assert_eq!(refusal.cause, None, "{refusal:?}");
    assert_eq!(
        refusal.loci[0].path,
        Some(pointer("/semantic_graph/nodes/3/body/dimension_node_ids/0")),
        "{refusal:?}"
    );

    let mut self_minted = wire;
    let foreign =
        self_minted["semantic_graph"]["nodes"][3]["body"]["dimension_node_ids"][0]["node"].clone();
    self_minted["semantic_graph"]["nodes"][3]["body"]["dimension_node_ids"][0] = foreign;
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with_dependencies(&self_minted, &[(name.as_str(), &dependency)])
    else {
        panic!("foreign node written as a local ref must refuse");
    };
    assert_eq!(refusal.code, Code::InvalidSemanticGraph, "{refusal:?}");
    assert_eq!(refusal.cause, None, "{refusal:?}");
    assert_eq!(
        refusal.loci[0].path,
        Some(pointer("/semantic_graph/nodes/3/body/dimension_node_ids/0")),
        "{refusal:?}"
    );
}

/// A typed equivalence retains its exact structural id, pair, reduced scale,
/// two dependencies, source owner and anchor occurrence through the reader.
#[trace("TC-048", "FR-063-AC-1", "FR-063-AC-6")]
#[test]
fn typed_equivalence_retains_complete_structural_node() {
    let wire = package();
    let CheckedPackageV2ReadResult::Admitted(admitted) = read(&wire) else {
        panic!("well-formed equivalence package must admit");
    };
    let retained = serde_json::to_value(&admitted.graph().nodes[3]).expect("retained node");
    let expected = &crate::support::checked_package::wire_form(&wire)["semantic_graph"]["nodes"][3];
    assert_eq!(&retained, expected);
    assert_eq!(
        retained["dependencies"],
        retained["body"]["dimension_node_ids"]
    );
    assert_eq!(retained["semantic_type"], retained["node_id"]);
    assert!(retained.get("declaration").is_none());
    assert_eq!(
        retained["occurrences"],
        json!([{"role":"anchor","ordinal":0}])
    );
}

/// The scalar body is closed, and malformed body/owner shape wins over graph
/// consistency checks at the node where it occurs.
#[trace("TC-048", "FR-063-AC-7")]
#[test]
fn missing_equivalence_root_scale_refuses_at_its_node() {
    let mut wire = package();
    wire["semantic_graph"]["nodes"][3]["body"]
        .as_object_mut()
        .expect("equivalence body is an object")
        .remove("root_scale");
    expect_refusal(&wire, Code::MalformedWire, None, "/semantic_graph/nodes/3");
}

#[trace("TC-048", "FR-063-AC-7")]
#[test]
fn malformed_equivalence_root_scale_container_refuses_at_its_node() {
    let base = package();

    let mut non_object = base.clone();
    non_object["semantic_graph"]["nodes"][3]["body"]["root_scale"] = json!("1/1");
    expect_refusal(
        &non_object,
        Code::MalformedWire,
        None,
        "/semantic_graph/nodes/3",
    );

    let mut extra_member = base;
    extra_member["semantic_graph"]["nodes"][3]["body"]["root_scale"]["extra"] = json!("x");
    expect_refusal(
        &extra_member,
        Code::MalformedWire,
        None,
        "/semantic_graph/nodes/3",
    );
}

#[trace("TC-048", "FR-063-AC-5", "FR-063-AC-7")]
#[test]
fn malformed_equivalence_shape_refuses_at_its_node() {
    let base = package();
    type Case = (fn(&mut Value), &'static str);
    let cases: [Case; 7] = [
        (
            |v: &mut Value| {
                v["semantic_graph"]["nodes"][3]["body"]["root_scale"]["numerator"] = json!("0");
            },
            "/semantic_graph/nodes/3",
        ),
        (
            |v: &mut Value| {
                v["semantic_graph"]["nodes"][3]["body"]["root_scale"]["numerator"] = json!("-1");
            },
            "/semantic_graph/nodes/3",
        ),
        (
            |v: &mut Value| {
                v["semantic_graph"]["nodes"][3]["body"]["root_scale"]["denominator"] = json!("0");
            },
            "/semantic_graph/nodes/3",
        ),
        (
            |v: &mut Value| {
                v["semantic_graph"]["nodes"][3]["body"]["root_scale"]["denominator"] = json!("-1");
            },
            "/semantic_graph/nodes/3",
        ),
        (
            |v: &mut Value| {
                v["semantic_graph"]["nodes"][3]["body"]["dimension_node_ids"] =
                    json!([v["semantic_graph"]["nodes"][1]["node_id"]]);
            },
            "/semantic_graph/nodes/3/body/dimension_node_ids",
        ),
        (
            |v: &mut Value| {
                v["semantic_graph"]["nodes"][3]
                    .as_object_mut()
                    .unwrap()
                    .remove("owner");
            },
            "/semantic_graph/nodes/3",
        ),
        (
            |v: &mut Value| {
                v["semantic_graph"]["nodes"][3]["declaration"] =
                    json!({"qualified_name":["Measure","Pair"]});
            },
            "/semantic_graph/nodes/3",
        ),
    ];
    for (mutate, at) in cases {
        let mut adverse = base.clone();
        mutate(&mut adverse);
        expect_refusal(&adverse, Code::MalformedWire, None, at);
    }
}

/// Equal or reversed pair ids and an unreduced scale are graph failures,
/// rather than stale-key refusals from the retired content identity scheme.
#[trace("TC-048", "FR-063-AC-4", "FR-063-AC-5", "FR-063-AC-8")]
#[test]
fn invalid_pair_and_scale_refuse_at_equivalence_node() {
    let base = package();
    let mut equal = base.clone();
    equal["semantic_graph"]["nodes"][3]["body"]["dimension_node_ids"] = json!([
        base["semantic_graph"]["nodes"][1]["node_id"],
        base["semantic_graph"]["nodes"][1]["node_id"]
    ]);
    expect_refusal(
        &equal,
        Code::InvalidSemanticGraph,
        None,
        "/semantic_graph/nodes/3",
    );

    let mut reversed = base.clone();
    reversed["semantic_graph"]["nodes"][3]["body"]["dimension_node_ids"] = json!([
        base["semantic_graph"]["nodes"][2]["node_id"],
        base["semantic_graph"]["nodes"][1]["node_id"]
    ]);
    expect_refusal(
        &reversed,
        Code::InvalidSemanticGraph,
        None,
        "/semantic_graph/nodes/3",
    );

    let mut unreduced = base;
    unreduced["semantic_graph"]["nodes"][3]["body"]["root_scale"] =
        json!({"numerator":"2000","denominator":"2"});
    expect_refusal(
        &unreduced,
        Code::InvalidSemanticGraph,
        None,
        "/semantic_graph/nodes/3",
    );
}

/// An inconsistent ratio discovered only after traversing A-B and B-C is
/// attributed to the closing E-AC edge, even though that edge is visited last.
#[trace("TC-048", "FR-063-AC-9")]
#[test]
fn inconsistent_three_edge_ratio_cycle_refuses_at_closing_edge() {
    let mut wire = package();
    let source = fixture_source();
    let alpha = dimension("A", &source);
    let beta = dimension("B", &source);
    let gamma = dimension("C", &source);
    let alpha_id = alpha["node_id"].clone();
    let beta_id = beta["node_id"].clone();
    let gamma_id = gamma["node_id"].clone();
    let mut nodes = vec![
        named_equivalence("01-ab", &alpha_id, &beta_id, &source, "1", "1"),
        named_equivalence("02-bc", &beta_id, &gamma_id, &source, "1", "1"),
        named_equivalence("03-ac", &alpha_id, &gamma_id, &source, "999", "1"),
        alpha,
        beta,
        gamma,
    ];
    for (ordinal, node) in nodes.iter_mut().take(3).enumerate() {
        node["occurrences"][0]["ordinal"] = json!(ordinal);
    }
    wire["semantic_graph"]["nodes"] = json!(nodes);
    wire["source_map"] = json!(nodes
        .iter()
        .enumerate()
        .map(|(index, node)| json!({
            "node_id":node["node_id"],
            "role":node["occurrences"][0]["role"],
            "ordinal":node["occurrences"][0]["ordinal"],
            "regions":[{"source":source,"start":index,"end":index+1}]
        }))
        .collect::<Vec<_>>());
    index_fixture_node_ids(&mut wire);
    refresh_identity(&mut wire);

    expect_refusal(
        &wire,
        Code::InvalidSemanticGraph,
        None,
        "/semantic_graph/nodes/2",
    );
}

/// One equivalence of a cycle graph: the two base dimensions by position, in
/// ascending order, and `root_scale`.
type Edge = (usize, usize, &'static str, &'static str);

/// A package of the base dimensions A to D joined by `edges`, which are the
/// graph's first nodes and sort by node id in the order given. Equivalence `n`
/// is `/semantic_graph/nodes/n`.
fn cycle_package(edges: &[Edge]) -> Value {
    const LABELS: [&str; 6] = ["01", "02", "03", "04", "05", "06"];
    let mut wire = package();
    let source = fixture_source();
    let dimensions = ["A", "B", "C", "D"].map(|name| dimension(name, &source));
    let mut nodes = edges
        .iter()
        .enumerate()
        .map(|(ordinal, &(left, right, numerator, denominator))| {
            let mut node = named_equivalence(
                LABELS[ordinal],
                &dimensions[left]["node_id"],
                &dimensions[right]["node_id"],
                &source,
                numerator,
                denominator,
            );
            node["occurrences"][0]["ordinal"] = json!(ordinal);
            node
        })
        .collect::<Vec<_>>();
    nodes.extend(dimensions);
    wire["semantic_graph"]["nodes"] = json!(nodes);
    wire["source_map"] = json!(nodes
        .iter()
        .enumerate()
        .map(|(index, node)| json!({
            "node_id":node["node_id"],
            "role":node["occurrences"][0]["role"],
            "ordinal":node["occurrences"][0]["ordinal"],
            "regions":[{"source":source,"start":index,"end":index+1}]
        }))
        .collect::<Vec<_>>());
    index_fixture_node_ids(&mut wire);
    refresh_identity(&mut wire);
    wire
}

fn expect_admitted(edges: &[Edge]) {
    let result = read(&cycle_package(edges));
    assert!(
        matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
        "{edges:?}: {result:?}"
    );
}

fn expect_cycle_refusal(edges: &[Edge], node: usize) {
    expect_refusal(
        &cycle_package(edges),
        Code::InvalidSemanticGraph,
        None,
        &format!("/semantic_graph/nodes/{node}"),
    );
}

/// A = B / 1000, A = C / 1000 and so B = C: the third equivalence closes the
/// cycle, although a depth-first walk from A meets the disagreement on the
/// second.
#[trace("TC-048", "FR-063-AC-9", "FSpec-FR-322-AC-268")]
#[test]
fn cycle_refusal_is_not_where_a_depth_first_walk_meets_it() {
    expect_admitted(&[(0, 1, "1", "1000"), (0, 2, "1", "1000"), (1, 2, "1", "1")]);
    expect_cycle_refusal(
        &[(0, 1, "1", "1000"), (0, 2, "1", "1000"), (1, 2, "2", "1")],
        2,
    );
}

/// A = 2 B, C = 3 D and B = 5 D, so A = 10 D = 10/3 C. The third equivalence
/// joins two classes of two dimensions each.
#[trace("TC-048", "FR-063-AC-9", "FSpec-FR-322-AC-268")]
#[test]
fn cycle_through_two_joined_classes_refuses_at_its_latest_equivalence() {
    let joined = [(0, 1, "2", "1"), (2, 3, "3", "1"), (1, 3, "5", "1")];
    let closed = |numerator| {
        let mut edges = joined.to_vec();
        edges.push((0, 2, numerator, "3"));
        edges
    };
    expect_admitted(&closed("10"));
    expect_cycle_refusal(&closed("11"), 3);
}

/// B = 7 C, B = 2 D and then A = 5 B, so the single dimension A joins a class
/// of three: A = 35 C = 10 D and C = 2/7 D.
#[trace("TC-048", "FR-063-AC-9", "FSpec-FR-322-AC-268")]
#[test]
fn cycle_through_a_class_joined_from_its_smaller_side_keeps_every_ratio() {
    let tree = [(1, 2, "7", "1"), (1, 3, "2", "1"), (0, 1, "5", "1")];
    let with = |rest: &[Edge]| [&tree[..], rest].concat();
    expect_admitted(&with(&[
        (0, 2, "35", "1"),
        (0, 3, "10", "1"),
        (2, 3, "2", "7"),
    ]));
    expect_cycle_refusal(&with(&[(0, 2, "35", "1"), (0, 3, "11", "1")]), 4);
    expect_cycle_refusal(&with(&[(0, 3, "10", "1"), (2, 3, "7", "2")]), 4);
}

/// Two cycles disagree. The one the third equivalence closes is complete
/// first in node-id order, before the one the fifth closes.
#[trace("TC-048", "FR-063-AC-9", "FSpec-FR-322-AC-268")]
#[test]
fn first_disagreeing_cycle_in_node_id_order_is_refused() {
    expect_cycle_refusal(
        &[
            (0, 1, "1", "1"),
            (0, 2, "1", "1"),
            (1, 2, "2", "1"),
            (0, 3, "1", "1"),
            (2, 3, "7", "1"),
        ],
        2,
    );
}

/// The graph step requires the exact pair as dependencies and a self type;
/// owner resolution precedes those graph checks.
#[trace("TC-048", "FR-063-AC-5")]
#[test]
fn dependency_type_and_owner_joins_refuse_at_their_own_seam() {
    let base = package();
    let mut missing_dependency = base.clone();
    missing_dependency["semantic_graph"]["nodes"][3]["dependencies"] =
        json!([base["semantic_graph"]["nodes"][1]["node_id"]]);
    expect_refusal(
        &missing_dependency,
        Code::InvalidSemanticGraph,
        None,
        "/semantic_graph/nodes/3",
    );

    let mut foreign_type = base.clone();
    foreign_type["semantic_graph"]["nodes"][3]["semantic_type"] =
        base["semantic_graph"]["nodes"][1]["node_id"].clone();
    expect_refusal(
        &foreign_type,
        Code::InvalidSemanticGraph,
        None,
        "/semantic_graph/nodes/3",
    );

    let mut unselected_owner = base;
    unselected_owner["semantic_graph"]["nodes"][3]["owner"]["identity"] =
        json!("test/missing-source");
    expect_refusal(
        &unselected_owner,
        Code::MissingDeclaration,
        Some(Cause::MissingSelection),
        "/semantic_graph/nodes/3/node_id",
    );
}

/// A root scale beyond 128 bits is parsed, inverted and multiplied by the
/// exact-number library on the caller's budget, each limb block charged at
/// the equivalence node before it is requested (FR-038-AC-504).
#[trace("TC-472", "FR-038-AC-504")]
#[test]
fn wide_root_scale_arithmetic_is_charged_to_the_read_budget() {
    use crate::checked_package_v2_graph_read_storage::{
        assert_exact_and_one_short, stage_requests,
    };
    use filament_contract_model::{NativeJoinBudget, PackageReadStage};
    let mut wide = package();
    wide["semantic_graph"]["nodes"][3]["body"]["root_scale"] =
        json!({"numerator":"100000000000000000000000000000000000000000","denominator":"1"});
    let bytes = canonical(&wide);
    let evidence = evidence_for(&wide);
    let read = |budget: &mut NativeJoinBudget| {
        crate::fspec_read_proof::read_budget_and_account(
            &bytes,
            CheckedPackageReadLimits::bounded(),
            &evidence,
            budget,
        )
    };
    let requests = stage_requests(&read, PackageReadStage::Graph);
    let exact: Vec<_> = requests
        .iter()
        .filter(|request| request.index == 3 && request.form == "exact_number")
        .collect();
    assert!(
        exact.len() >= 3,
        "parse, inverse and product are charged at the node: {exact:?}"
    );
    assert_exact_and_one_short(&read, &requests);
    // Each potentials pass charges its id table, entries, adjacency and
    // queue at the equivalence node, ahead of the exact-number requests.
    let tables = requests
        .iter()
        .filter(|request| request.index == 3 && request.form == "scalar_type")
        .count();
    assert!(
        tables >= 10,
        "the equivalence tables are charged at the node: {tables}"
    );
}

/// A selected dependency's nominal nodes are read into a charged table: each
/// record's id, names and declaration are requested at the imported node
/// (`imported_nominals`, by its position in its own package) only after the
/// caller budget admits them (FR-038-AC-504).
#[trace("TC-472", "FR-038-AC-504")]
#[test]
fn imported_nominal_records_are_charged_at_the_imported_node() {
    use crate::checked_package_v2_graph_read_storage::{
        assert_exact_and_one_short, stage_requests,
    };
    use filament_contract_model::{NativeJoinBudget, PackageReadStage};
    let (wire, name, dependency) = mixed_package();
    let bytes = canonical(&wire);
    let mut evidence = evidence_for(&wire);
    evidence
        .insert_dependency_package(name.as_str(), dependency.clone())
        .expect("dependency evidence agrees on admitted content");
    let read = |budget: &mut NativeJoinBudget| {
        crate::fspec_read_proof::read_budget_and_account(
            &bytes,
            CheckedPackageReadLimits::bounded(),
            &evidence,
            budget,
        )
    };
    let requests = stage_requests(&read, PackageReadStage::Graph);
    let imported = requests
        .iter()
        .filter(|request| request.form == "imported_nominals")
        .count();
    assert!(
        imported > 0,
        "the imported records are charged at their nodes: {requests:?}"
    );
    assert_exact_and_one_short(&read, &requests);
}

fn node_id_of(value: &Value) -> CheckedNodeId {
    serde_json::from_value(value.clone()).expect("wire node id")
}

fn is_dimension(package: &CheckedPackageV2, id: &CheckedNodeId) -> bool {
    matches!(
        package.node_kind(id),
        Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Dimension))
    ) && matches!(
        package.term(id),
        Some(CheckedTerm::NominalBody(body)) if body.dimension_base().is_some()
    )
}

/// Resolve an authored operand to a dimension through public package reads only: a
/// local operand in the importing package, a package operand in the exact selected
/// dependency named by the reference.
fn resolves_to_dimension(importer: &CheckedPackageV2, operand: &CheckedDimensionOperand) -> bool {
    match operand {
        CheckedDimensionOperand::Local(id) => is_dimension(importer, id),
        CheckedDimensionOperand::Package(reference) => importer
            .direct_dependencies()
            .filter(|dependency| dependency.name() == &*reference.package)
            .any(|dependency| is_dimension(dependency, reference.node.node_id())),
    }
}

fn equivalence_view(
    package: &CheckedPackageV2,
    index: usize,
) -> Option<(&[CheckedDimensionOperand; 2], &CheckedRational)> {
    let CheckedTerm::NominalBody(body) = &package.graph().nodes[index].body else {
        return None;
    };
    body.dimension_equivalence_details()
}

/// The borrowed view returns the exact two local operands and the exact ratio.
#[trace("TC-048", "TC-058", "FR-038-AC-601")]
#[test]
fn local_equivalence_view_borrows_exact_operands_and_ratio() {
    let mut wire = package();
    wire["semantic_graph"]["nodes"][3]["body"]["root_scale"] =
        json!({"numerator":"3","denominator":"2"});
    refresh_identity(&mut wire);
    let CheckedPackageV2ReadResult::Admitted(admitted) = read(&wire) else {
        panic!("local equivalence with a reduced ratio must admit");
    };
    let (operands, ratio) = equivalence_view(&admitted, 3).expect("equivalence view");
    let alpha = node_id_of(&wire["semantic_graph"]["nodes"][1]["node_id"]);
    let beta = node_id_of(&wire["semantic_graph"]["nodes"][2]["node_id"]);
    assert_eq!(
        operands,
        &[
            CheckedDimensionOperand::Local(alpha),
            CheckedDimensionOperand::Local(beta)
        ]
    );
    assert_eq!(&*ratio.numerator, "3");
    assert_eq!(&*ratio.denominator, "2");
    for operand in operands {
        assert!(resolves_to_dimension(&admitted, operand));
    }
    let reread = reread_local(&admitted, &[]);
    let (reread_operands, reread_ratio) = equivalence_view(&reread, 3).expect("reread view");
    assert_eq!((reread_operands, reread_ratio), (operands, ratio));
}

/// A package operand keeps its authored form and resolves in the selected
/// dependency, not in the importing graph.
#[trace("TC-048", "TC-058", "FR-038-AC-601")]
#[test]
fn package_equivalence_view_resolves_in_selected_dependency() {
    let (wire, name, dependency) = mixed_package();
    let CheckedPackageV2ReadResult::Admitted(admitted) =
        read_with_dependencies(&wire, &[(name.as_str(), &dependency)])
    else {
        panic!("directly selected foreign dimension must admit");
    };
    let (operands, ratio) = equivalence_view(&admitted, 3).expect("equivalence view");
    assert_eq!((&*ratio.numerator, &*ratio.denominator), ("1", "1"));
    let [CheckedDimensionOperand::Package(foreign), CheckedDimensionOperand::Local(local)] =
        operands
    else {
        panic!("authored order is package then local: {operands:?}");
    };
    assert_eq!(&*foreign.package, name.as_str());
    assert_eq!(
        foreign.node.node_id(),
        &node_id_of(&wire["semantic_graph"]["nodes"][3]["body"]["dimension_node_ids"][0]["node"])
    );
    assert_eq!(
        local,
        &node_id_of(&wire["semantic_graph"]["nodes"][1]["node_id"])
    );
    for operand in operands {
        assert!(resolves_to_dimension(&admitted, operand));
    }
    assert!(is_dimension(
        admitted.direct_dependencies().next().expect("dependency"),
        foreign.node.node_id()
    ));
    // The foreign node is not minted into the importing graph, and the
    // reference naming a different package resolves nowhere.
    assert!(admitted.term(foreign.node.node_id()).is_none());
    let mut other = foreign.clone();
    other.package = "another/package".into();
    assert!(!resolves_to_dimension(
        &admitted,
        &CheckedDimensionOperand::Package(other)
    ));

    // The canonical writer output, read back with the same dependency, yields the same view.
    let reread = reread_local(&admitted, &[(name.as_str(), &dependency)]);
    assert_eq!(equivalence_view(&reread, 3), Some((operands, ratio)));
}

/// A wrong-kind operand never reaches the view: the reader refuses an operand naming
/// the boolean empty-aggregate node, and the equivalence node is not itself a dimension.
#[trace("TC-048", "TC-058", "FR-038-AC-601")]
#[test]
fn wrong_kind_operand_refuses_and_is_not_a_dimension() {
    let mut wire = package();
    wire["semantic_graph"]["nodes"][3]["body"]["dimension_node_ids"][0] =
        wire["semantic_graph"]["nodes"][0]["node_id"].clone();
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&wire) else {
        panic!("a non-dimension operand must refuse");
    };
    assert_eq!(refusal.code, Code::InvalidSemanticGraph, "{refusal:?}");

    let CheckedPackageV2ReadResult::Admitted(admitted) = read(&package()) else {
        panic!("equivalence package must admit");
    };
    // The equivalence node itself is a scalar type but not a dimension.
    let own = node_id_of(&package()["semantic_graph"]["nodes"][3]["node_id"]);
    assert!(!is_dimension(&admitted, &own));
}

/// Write an admitted package through the canonical writer and read it back, with the
/// same selected dependencies.
fn reread_local(
    admitted: &CheckedPackageV2,
    dependencies: &[(&str, &CheckedPackageV2)],
) -> CheckedPackageV2 {
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
    .expect("canonical writer encodes the admitted package");
    let encoded: Value = serde_json::from_slice(&encoded).expect("canonical package JSON");
    match read_with_dependencies(&encoded, dependencies) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("written package must read back: {other:?}"),
    }
}

/// Dimension, unit and enum bodies of an admitted package each reach the accessor
/// and return no equivalence view.
#[trace("TC-048", "TC-058", "FR-038-AC-601")]
#[test]
fn other_nominal_forms_have_no_equivalence_view() {
    let wire = crate::support::checked_package::v2_nominal();
    let CheckedPackageV2ReadResult::Admitted(admitted) = read(&wire) else {
        panic!("nominal fixture must admit");
    };
    let mut seen = std::collections::BTreeSet::new();
    for (authored, node) in wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("authored nodes")
        .iter()
        .zip(&admitted.graph().nodes)
    {
        if let CheckedTerm::NominalBody(body) = &node.body {
            assert!(body.dimension_equivalence_details().is_none());
            seen.insert(authored["body"]["term"].as_str().expect("term").to_owned());
        }
    }
    assert_eq!(
        seen,
        ["dimension", "enum", "unit"]
            .map(str::to_owned)
            .into_iter()
            .collect()
    );
    // The accessor does answer for an equivalence body, so the none above is not vacuous.
    let CheckedPackageV2ReadResult::Admitted(equivalence) = read(&package()) else {
        panic!("equivalence package must admit");
    };
    assert!(equivalence_view(&equivalence, 3).is_some());
}
