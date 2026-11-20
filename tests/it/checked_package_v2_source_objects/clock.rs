// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-481 and TC-482: the source timed-model clock-place join (FR-374) and
//! the operation-body coverage (FR-375) over the package `fsl compile` emits
//! for FSpec FR-789-AC-1's unit (`record W { c: Clock; }`,
//! `object O { w: W; d: Clock; }`, `time O dense unit ms;`), committed as
//! `tests/fixtures/qsl-2347-emitted/clock-place.json`.

use super::*;
use filament_contract_model::{
    SelectedClockCollectionLayer, SelectedSourceClockRows, SelectedSourceObjectError,
    SelectedSourceRecordMemberRow, SelectedSourceRecordRow,
};
use filament_exact::{NodeKey, PathSegment, Presence};

type EntryLayerEdit = fn(&mut Vec<SelectedClockCollectionLayer>);
type EntryEdit = fn(&mut SelectedSourceObjectRow, CheckedNodeId);

const CLOCK: &str = "ix://filament/native/Clock";
const I: &str = "test:spine-model";

fn emitted() -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/qsl-2347-emitted/clock-place.json");
    serde_json::from_slice(&std::fs::read(path).expect("fixture")).expect("fixture json")
}

fn node_of(package: &Value, name: &str) -> CheckedNodeId {
    let index = nodes(package)
        .iter()
        .position(|node| node["declaration"]["qualified_name"][0] == name)
        .expect("declared node");
    serde_json::from_value(nodes(package)[index]["node_id"].clone()).expect("node id")
}

fn object_row(package: &Value) -> SelectedSourceObjectRow {
    SelectedSourceObjectRow {
        structural_node: node_of(package, "O"),
        source_authority: "agent-ix".into(),
        source_identity: "O".into(),
        is_abstract: false,
        supertypes: vec![],
        identity_fields: vec![],
        members: vec![
            SelectedSourceObjectMember::data("w", node_of(package, "W"), false),
            SelectedSourceObjectMember::clock("d", CLOCK),
        ],
        populations: vec![],
        operations: vec![SelectedSourceObjectOperation {
            identity: "O::step".into(),
            query: false,
            clock_parameters: vec![],
            parameters: vec![],
        }],
    }
}

fn record_row(package: &Value) -> SelectedSourceRecordRow {
    SelectedSourceRecordRow {
        structural_node: node_of(package, "W"),
        source_authority: "agent-ix".into(),
        source_identity: I.into(),
        qualified_declaration: vec!["W".into()],
        record_key: NodeKey::new("fsl-record", vec![PathSegment::Text("W".into())], 0)
            .expect("key"),
        members: vec![SelectedSourceRecordMemberRow {
            original_member_identity: "c".into(),
            admitted_name: "c".into(),
            presence: Presence::Required,
            type_node: None,
            value_type: None,
            member_type: Some(CLOCK.into()),
            collections: vec![],
        }],
    }
}

fn joined(
    package: &Value,
    object: SelectedSourceObjectRow,
    record: Option<SelectedSourceRecordRow>,
) -> Result<(), SelectedSourceObjectError> {
    let admitted = admitted(package);
    let records: Vec<_> = record.into_iter().collect();
    SelectedSourceObjects::from_stage_rows(
        &admitted,
        vec![object],
        SelectedSourceClockRows {
            records: &records,
            tuples: &[],
            unions: &[],
        },
    )
    .map(|_| ())
}

/// The `resolved_path` rows of the place at node `index`.
fn rows_mut(package: &mut Value, index: usize) -> &mut Vec<Value> {
    package["semantic_graph"]["nodes"][index]["body"]["members"][2]["value"]["members"]
        .as_array_mut()
        .expect("rows")
}

fn set_row(package: &mut Value, index: usize, row: usize, name: &str, value: &str) {
    let members = rows_mut(package, index)[row]["members"]
        .as_array_mut()
        .expect("row members");
    let binding = members
        .iter_mut()
        .find(|member| member["name"] == name)
        .expect("binding");
    binding["value"]["value"] = json!(value);
}

fn place(package: &Value, last: &str) -> usize {
    nodes(package)
        .iter()
        .position(|node| {
            node["semantic_form"] == "clock_place"
                && node["body"]["members"][2]["value"]["members"]
                    .as_array()
                    .and_then(|rows| rows.last())
                    .is_some_and(|row| {
                        row["members"].as_array().is_some_and(|members| {
                            members.iter().any(|member| {
                                member["name"] == "member_identity"
                                    && member["value"]["value"] == format!("ix://{I}/{last}")
                            })
                        })
                    })
        })
        .expect("place")
}

#[trace("TC-481", "FR-374-AC-1")]
#[test]
fn the_emitted_package_is_admitted_and_its_rows_join() {
    let package = emitted();
    joined(&package, object_row(&package), Some(record_row(&package))).expect("joins");
}

#[trace("TC-481", "FR-374-AC-2")]
#[test]
fn the_strict_read_refuses_each_edited_place_row() {
    let package = emitted();
    let (d, c) = (place(&package, "O/d"), place(&package, "W/c"));
    let cases: [(&str, usize, usize, &str, &str); 5] = [
        (
            "member identity",
            d,
            0,
            "member_identity",
            "ix://test:spine-model/O/e",
        ),
        (
            "foreign declaring",
            d,
            0,
            "declaring_type",
            "ix://other:id/O",
        ),
        ("not a clock", d, 0, "member_type", "Integer"),
        (
            "binding mismatch",
            c,
            0,
            "member_type",
            "ix://test:spine-model/V",
        ),
        (
            "selected declaring",
            c,
            1,
            "declaring_type",
            "ix://acme/orders/Order",
        ),
    ];
    for (case, node, row, name, value) in cases {
        let mut broken = package.clone();
        set_row(&mut broken, node, row, name, value);
        let path = format!("/semantic_graph/nodes/{node}/body/members/2/value/members/{row}");
        let refusal = refused(&broken);
        assert_eq!(refusal.code, Code::InvalidPackage, "{case}: {refusal:?}");
        assert_eq!(refusal.cause, Some(Cause::MalformedClockPlace), "{case}");
        assert_eq!(
            refusal.loci[0].path.as_ref().map(|at| at.as_str()),
            Some(path.as_str()),
            "{case}: {refusal:?}"
        );
    }
}

#[trace("TC-481", "FR-374-AC-3")]
#[test]
fn the_identity_row_join_refuses_a_place_the_rows_do_not_show() {
    let package = emitted();
    let w_clock = |row: &mut SelectedSourceRecordRow| row.members.clear();
    let mut missing_member = record_row(&package);
    w_clock(&mut missing_member);
    let error =
        joined(&package, object_row(&package), Some(missing_member)).expect_err("no `c` entry");
    assert_eq!(error.code, RowError::MalformedClockPlace);
    let mut no_clock = object_row(&package);
    no_clock.members.retain(|member| &*member.name != "d");
    let error = joined(&package, no_clock, Some(record_row(&package))).expect_err("no `d` entry");
    assert_eq!(error.code, RowError::MalformedClockPlace);
    let mut optional = object_row(&package);
    optional.members[1].member_type = Some(format!("Option<{CLOCK}>").into());
    let error = joined(&package, optional, Some(record_row(&package))).expect_err("optional");
    assert_eq!(error.code, RowError::MalformedClockPlace);
    let error = joined(&package, object_row(&package), None).expect_err("no W row");
    assert_eq!(error.code, RowError::MalformedClockPlace);
}

#[trace("TC-481", "FR-367-AC-7")]
#[test]
fn a_clock_entry_is_a_member_type_and_a_data_entry_a_type_node() {
    let package = emitted();
    let cases: [(&str, EntryEdit); 4] = [
        ("clock entry with a type node", |row, node| {
            row.members[1].type_node = Some(node)
        }),
        ("entry with neither", |row, _| {
            row.members[1].member_type = None
        }),
        ("clock entry named w", |row, _| {
            row.members[1].name = "w".into()
        }),
        ("clock entry typed Integer", |row, _| {
            row.members[1].member_type = Some("Integer".into())
        }),
    ];
    for (case, edit) in cases {
        let mut row = object_row(&package);
        edit(&mut row, node_of(&package, "W"));
        let error = joined(&package, row, Some(record_row(&package))).expect_err(case);
        assert_eq!(
            error.code,
            RowError::UnknownIdentityWitness,
            "{case}: {error:?}"
        );
    }
}

#[trace("TC-482", "FR-375-AC-1")]
#[test]
fn operation_bodies_cover_exactly_the_declared_operations() {
    let package = emitted();
    let mut absent = object_row(&package);
    absent.operations.clear();
    let error = joined(&package, absent, Some(record_row(&package))).expect_err("ghost row");
    assert_eq!(error.code, RowError::InvalidValue);
    assert_eq!(locus_path(&error), "/operation_bodies/0/operation");
    let mut extra = object_row(&package);
    let mut second = extra.operations[0].clone();
    second.identity = "O::bump".into();
    extra.operations.push(second);
    let error = joined(&package, extra, Some(record_row(&package))).expect_err("missing body");
    assert_eq!(error.code, RowError::InvalidValue);
    assert_eq!(locus_path(&error), "/operation_bodies");
}

fn locus_path(error: &SelectedSourceObjectError) -> &str {
    error
        .locus
        .as_ref()
        .and_then(|locus| locus.path.as_ref())
        .map(|path| path.as_str())
        .expect("located")
}

fn collections() -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/qsl-2347-emitted/clock-collections.json");
    serde_json::from_slice(&std::fs::read(path).expect("fixture")).expect("fixture json")
}

fn layer(kind: &str, minimum: &str, maximum: &str) -> SelectedClockCollectionLayer {
    SelectedClockCollectionLayer {
        kind: kind.into(),
        minimum: minimum.into(),
        maximum: maximum.into(),
    }
}

fn collections_object(package: &Value) -> SelectedSourceObjectRow {
    let bounded = nodes(package)
        .iter()
        .find(|node| node["node_tag"] == "bounded_domain")
        .expect("the collection bound node");
    let mut q = SelectedSourceObjectMember::clock("q", format!("Sequence<{CLOCK}>"));
    q.collections = vec![layer("sequence", "0", "3")];
    let mut row = object_row(package);
    row.members = vec![
        SelectedSourceObjectMember::data(
            "r",
            serde_json::from_value(bounded["node_id"].clone()).expect("bound id"),
            false,
        ),
        SelectedSourceObjectMember::clock("d", CLOCK),
        q,
    ];
    row.operations[0].identity = "O::arm".into();
    row.operations[0].clock_parameters = vec!["x".into()];
    row.operations[0].parameters = vec![SelectedSourceObjectMember::clock("x", CLOCK)];
    row
}

#[trace("TC-481", "FR-374-AC-3")]
#[test]
fn a_collection_place_joins_its_layers_and_refuses_a_mismatch() {
    let package = collections();
    joined(
        &package,
        collections_object(&package),
        Some(record_row(&package)),
    )
    .expect("the emitted collection places join");
    let edits: [(&str, EntryLayerEdit); 4] = [
        ("maximum", |layers| layers[0].maximum = "4".into()),
        ("kind", |layers| layers[0].kind = "set".into()),
        ("minimum", |layers| layers[0].minimum = "1".into()),
        ("no layer", |layers| layers.clear()),
    ];
    for (case, edit) in edits {
        let mut row = collections_object(&package);
        edit(&mut row.members[2].collections);
        let error = joined(&package, row, Some(record_row(&package))).expect_err(case);
        assert_eq!(
            error.code,
            RowError::MalformedClockPlace,
            "{case}: {error:?}"
        );
    }
}

#[trace("TC-481", "FR-374-AC-2")]
#[test]
fn the_strict_read_refuses_a_collection_place_that_disagrees_with_the_binding() {
    let package = collections();
    let r = place(&package, "W/c");
    let cases: [(&str, usize, &str, &str); 5] = [
        ("slot maximum", 0, "maximum", "4"),
        ("slot kind", 0, "collection_kind", "set"),
        ("slot minimum", 0, "minimum", "1"),
        (
            "unknown declaring type",
            0,
            "declaring_type",
            "ix://test:spine-model/Nope",
        ),
        ("malformed member type", 0, "member_type", "Sequence<"),
    ];
    for (case, row, name, value) in cases {
        let mut broken = package.clone();
        set_row(&mut broken, r, row, name, value);
        let refusal = refused(&broken);
        assert_eq!(
            refusal.cause,
            Some(Cause::MalformedClockPlace),
            "{case}: {refusal:?}"
        );
    }
}
