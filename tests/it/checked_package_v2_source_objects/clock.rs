// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-481 and TC-482: the source timed-model clock-place join (FR-374) and
//! the operation-body coverage (FR-375) over the package FSL's emit harness
//! writes for FSpec FR-789-AC-1's unit (`record W { c: Clock; }`,
//! `object O { w: W; d: Clock; }`, `time O dense unit ms;`), committed as
//! `tests/fixtures/qsl-2347-emitted/clock-place.json`. The fixtures are the
//! output of FSL's emit test harness; their sources and provenance are in the
//! README beside them.

use super::*;
use filament_contract_model::{
    SelectedClockCollectionLayer, SelectedSourceClockRows, SelectedSourceObjectError,
    SelectedSourceRecordMemberRow, SelectedSourceRecordRow, SelectedSourceTupleRow,
    SelectedSourceUnionRow, SelectedTuplePositionRow, SelectedUnionClockPayloadRow,
    SelectedUnionMemberRow,
};
use filament_exact::{NodeKey, PathSegment, Presence, UnionMember, ValueType};

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
    let (d, c) = (place(&package, "O/d"), place(&package, "W/c"));
    let at = |node: usize, row: usize| {
        format!("/semantic_graph/nodes/{node}/body/members/2/value/members/{row}")
    };
    let mut missing_member = record_row(&package);
    missing_member.members.clear();
    let error =
        joined(&package, object_row(&package), Some(missing_member)).expect_err("no `c` entry");
    assert_eq!(error.code, RowError::MalformedClockPlace);
    assert_eq!(locus_path(&error), at(c, 1));
    let mut no_clock = object_row(&package);
    no_clock.members.retain(|member| &*member.name != "d");
    let error = joined(&package, no_clock, Some(record_row(&package))).expect_err("no `d` entry");
    assert_eq!(error.code, RowError::MalformedClockPlace);
    assert_eq!(locus_path(&error), at(d, 0));
    let mut optional = object_row(&package);
    optional.members[1].member_type = Some(format!("Option<{CLOCK}>").into());
    let error = joined(&package, optional, Some(record_row(&package))).expect_err("optional");
    assert_eq!(error.code, RowError::MalformedClockPlace);
    assert_eq!(locus_path(&error), at(d, 0));
    let error = joined(&package, object_row(&package), None).expect_err("no W row");
    assert_eq!(error.code, RowError::MalformedClockPlace);
    assert_eq!(locus_path(&error), at(c, 1));
    // A place whose coherent rows name a clock `O` declares no entry for
    // (FSpec FR-789-AC-3's `field(e)`): both stages accept the rows, the
    // join refuses.
    let mut undeclared = package.clone();
    *binding_mut(step_mut(&mut undeclared, d, 0), "name") = json!("e");
    set_row(
        &mut undeclared,
        d,
        0,
        "member_identity",
        "ix://test:spine-model/O/e",
    );
    let error = joined(
        &undeclared,
        object_row(&undeclared),
        Some(record_row(&undeclared)),
    )
    .expect_err("undeclared clock");
    assert_eq!(error.code, RowError::MalformedClockPlace);
    assert_eq!(locus_path(&error), at(d, 0));
}

#[trace("TC-481", "FR-374-AC-3")]
#[test]
fn a_missing_identity_row_is_reported_before_a_clock_place_defect() {
    let package = tuple_union();
    let rows = tuple_union_rows(&package);
    let mut object = tuple_union_object(&package, "O");
    object.members.retain(|member| &*member.name != "d");
    let admitted = admitted(&package);
    let error = SelectedSourceObjects::from_stage_rows(
        &admitted,
        vec![object],
        SelectedSourceClockRows {
            records: &rows.records,
            tuples: &rows.tuples,
            unions: &rows.unions,
        },
    )
    .expect_err("Q has no row and O no `d` entry");
    assert_eq!(error.code, RowError::MissingIdentityWitness);
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

/// FSpec FR-790-AC-1's package: `Order` declares `deposit` and `quote`, `Sub`
/// redefines `deposit` and inherits `quote`.
fn coverage_rows() -> Vec<SelectedSourceObjectRow> {
    let mut order = row("Order", vec![], &["Orders"]);
    order.operations.push(SelectedSourceObjectOperation {
        identity: "Order::quote".into(),
        query: true,
        clock_parameters: vec![],
        parameters: vec![],
    });
    vec![order, row("Sub", vec![id("qualified:Order")], &[])]
}

fn cover(package: &Value) -> Result<(), SelectedSourceObjectError> {
    let admitted = admitted(package);
    SelectedSourceObjects::from_stage_rows(
        &admitted,
        coverage_rows(),
        SelectedSourceClockRows::EMPTY,
    )
    .map(|_| ())
}

#[trace("TC-482", "FR-375-AC-1")]
#[test]
fn operation_bodies_cover_exactly_the_declared_operations() {
    cover(&bodies_package())
        .expect("two rows for Order, one for Sub, none for the inherited quote");
    let mut uncovered = bodies_package();
    body_rows_mut(&mut uncovered).retain(|row| row["operation"] != "quote");
    settle(&mut uncovered);
    let error = cover(&uncovered).expect_err("Order.quote has no body row");
    assert_eq!(error.code, RowError::InvalidValue);
    assert_eq!(locus_path(&error), "/operation_bodies");
    let mut ghost = bodies_package();
    let mut extra = row_of(&mut ghost, "qualified:Order", "quote").clone();
    extra["operation"] = json!("ghost");
    let mut all = body_rows_mut(&mut ghost).clone();
    all.push(extra);
    ghost["operation_bodies"] = sorted_body_rows(all);
    settle(&mut ghost);
    let k = body_rows_mut(&mut ghost)
        .iter()
        .position(|row| row["operation"] == "ghost")
        .expect("row");
    let error = cover(&ghost).expect_err("no row lists Order.ghost");
    assert_eq!(error.code, RowError::InvalidValue);
    assert_eq!(
        locus_path(&error),
        format!("/operation_bodies/{k}/operation")
    );
    // `Sub` lists no `quote`: a row for `Sub.quote` is one no row lists.
    let mut inherited = bodies_package();
    let mut extra = row_of(&mut inherited, "qualified:Order", "quote").clone();
    extra["declaring"] = node_id("qualified:Sub");
    let mut all = body_rows_mut(&mut inherited).clone();
    all.push(extra);
    inherited["operation_bodies"] = sorted_body_rows(all);
    settle(&mut inherited);
    let error = cover(&inherited).expect_err("Sub lists no quote");
    assert_eq!(error.code, RowError::InvalidValue);
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

/// The package FSL emits for `tuple P (Clock, Integer)`,
/// `union U { armed(Clock), idle }` and an object `O` over them, with the
/// tuple and union member identities respelled as FSpec FR-789 rule 4
/// requires (`P/p0`, `U/armed`): FSL spells them `P/tuple/0` and
/// `U/union/armed`.
fn tuple_union() -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/qsl-2347-emitted/clock-tuple-union.json");
    let text = std::fs::read_to_string(path).expect("fixture");
    let respelled = text
        .replace("/P/tuple/", "/P/p")
        .replace("/U/union/", "/U/");
    serde_json::from_str(&respelled).expect("fixture json")
}

fn typed(package: &Value, declaration: &str, field: &str) -> CheckedNodeId {
    let node = &nodes(package)[nodes(package)
        .iter()
        .position(|node| node["declaration"]["qualified_name"][0] == declaration)
        .expect("declared node")];
    let binding = node["body"]["members"]
        .as_array()
        .expect("members")
        .iter()
        .find(|member| member["name"] == field)
        .expect("binding");
    serde_json::from_value(binding["value"]["target"].clone()).expect("type node")
}

fn integer_node(package: &Value) -> CheckedNodeId {
    typed(package, "Q", "x")
}

fn tuple_union_object(package: &Value, name: &str) -> SelectedSourceObjectRow {
    let mut row = object_row(package);
    row.structural_node = node_of(package, name);
    row.source_identity = name.into();
    row.operations.clear();
    row.members = match name {
        "Q" => vec![SelectedSourceObjectMember::data(
            "x",
            integer_node(package),
            false,
        )],
        _ => {
            let mut members: Vec<_> = ["w", "p", "u", "r", "qq", "s", "t"]
                .iter()
                .map(|field| {
                    SelectedSourceObjectMember::data(*field, typed(package, "O", field), false)
                })
                .collect();
            let mut q = SelectedSourceObjectMember::clock("q", format!("Sequence<{CLOCK}>"));
            q.collections = vec![layer("sequence", "0", "3")];
            members.push(SelectedSourceObjectMember::clock("d", CLOCK));
            members.push(q);
            row.operations = vec![SelectedSourceObjectOperation {
                identity: "O::arm".into(),
                query: false,
                clock_parameters: vec!["x".into()],
                parameters: vec![
                    SelectedSourceObjectMember::clock("x", CLOCK),
                    SelectedSourceObjectMember::data("k", integer_node(package), false),
                ],
            }];
            members
        }
    };
    row
}

fn key(name: &str) -> NodeKey {
    NodeKey::new("fsl-clock", vec![PathSegment::Text(name.into())], 0).expect("key")
}

fn clock_position(ordinal: u64, member_type: &str) -> SelectedTuplePositionRow {
    SelectedTuplePositionRow {
        ordinal,
        type_node: None,
        value_type: None,
        member_type: Some(member_type.into()),
        collections: vec![],
    }
}

struct Rows {
    records: Vec<SelectedSourceRecordRow>,
    tuples: Vec<SelectedSourceTupleRow>,
    unions: Vec<SelectedSourceUnionRow>,
}

fn tuple_union_rows(package: &Value) -> Rows {
    let mut w = record_row(package);
    w.structural_node = node_of(package, "W");
    w.members.push(SelectedSourceRecordMemberRow {
        original_member_identity: "n".into(),
        admitted_name: "n".into(),
        presence: Presence::Required,
        type_node: Some(typed(package, "W", "n")),
        value_type: Some(ValueType::Integer),
        member_type: None,
        collections: vec![],
    });
    let tuple = SelectedSourceTupleRow {
        structural_node: node_of(package, "P"),
        source_authority: "agent-ix".into(),
        source_identity: I.into(),
        qualified_declaration: vec!["P".into()],
        type_handle: key("p"),
        positions: vec![
            clock_position(0, CLOCK),
            SelectedTuplePositionRow {
                ordinal: 1,
                type_node: Some(integer_node(package)),
                value_type: Some(ValueType::Integer),
                member_type: None,
                collections: vec![],
            },
        ],
    };
    let settled = key("u-settled");
    let mut armed = SelectedUnionMemberRow::new(
        UnionMember::from_admitted(settled.clone(), "armed").expect("member"),
        vec![],
        vec![],
    );
    armed.clock_payload = vec![SelectedUnionClockPayloadRow {
        ordinal: 0,
        presence: Presence::Required,
        member_type: CLOCK.into(),
        collections: vec![],
    }];
    let idle = SelectedUnionMemberRow::new(
        UnionMember::from_admitted(settled.clone(), "idle").expect("member"),
        vec![],
        vec![],
    );
    let union = SelectedSourceUnionRow::new(
        node_of(package, "U"),
        "agent-ix",
        I,
        vec!["U".into()],
        key("u-handle"),
        settled,
        vec![armed, idle],
    );
    Rows {
        records: vec![w],
        tuples: vec![tuple],
        unions: vec![union],
    }
}

fn join_rows(package: &Value, rows: &Rows) -> Result<(), SelectedSourceObjectError> {
    let admitted = admitted(package);
    SelectedSourceObjects::from_stage_rows(
        &admitted,
        vec![
            tuple_union_object(package, "O"),
            tuple_union_object(package, "Q"),
        ],
        SelectedSourceClockRows {
            records: &rows.records,
            tuples: &rows.tuples,
            unions: &rows.unions,
        },
    )
    .map(|_| ())
}

#[trace("TC-481", "FR-374-AC-3")]
#[test]
fn a_tuple_and_a_union_place_join_their_clock_positions() {
    let package = tuple_union();
    join_rows(&package, &tuple_union_rows(&package)).expect("FR-789-AC-4 joins");
}

/// The value of the binding `name` of an aggregate.
fn binding_mut<'v>(aggregate: &'v mut Value, name: &str) -> &'v mut Value {
    let found = aggregate["members"]
        .as_array_mut()
        .expect("aggregate members")
        .iter_mut()
        .find(|member| member["name"] == name)
        .expect("binding");
    &mut found["value"]["value"]
}

fn step_mut(package: &mut Value, node: usize, step: usize) -> &mut Value {
    &mut package["semantic_graph"]["nodes"][node]["body"]["members"][1]["value"]["members"][step]
}

fn row_mut(package: &mut Value, node: usize, row: usize) -> &mut Value {
    &mut rows_mut(package, node)[row]
}

/// The place whose first path step is the field `name`.
fn place_of(package: &Value, name: &str) -> usize {
    nodes(package)
        .iter()
        .position(|node| {
            node["semantic_form"] == "clock_place"
                && node["body"]["members"][1]["value"]["members"][0]["members"][1]["value"]["value"]
                    == name
        })
        .expect("place")
}

fn refuses_at_row(case: &str, package: &Value, rows: &Rows, node: usize, row: usize) {
    let error = join_rows(package, rows).expect_err(case);
    assert_eq!(
        error.code,
        RowError::MalformedClockPlace,
        "{case}: {error:?}"
    );
    assert_eq!(
        locus_path(&error),
        format!("/semantic_graph/nodes/{node}/body/members/2/value/members/{row}"),
        "{case}"
    );
}

#[trace("TC-481", "FR-374-AC-3")]
#[test]
fn the_join_refuses_a_tuple_or_union_place_the_rows_do_not_show() {
    let package = tuple_union();
    let rows = tuple_union_rows(&package);
    let (p, u) = (place_of(&package, "p"), place_of(&package, "u"));
    // `tuple(1)` is a data position, not a Clock.
    let mut data_position = package.clone();
    *binding_mut(step_mut(&mut data_position, p, 1), "index") = json!(1);
    set_row(
        &mut data_position,
        p,
        1,
        "member_identity",
        "ix://test:spine-model/P/p1",
    );
    refuses_at_row("data position", &data_position, &rows, p, 1);
    // `tuple(2)` is no position of `P`.
    let mut absent = package.clone();
    *binding_mut(step_mut(&mut absent, p, 1), "index") = json!(2);
    set_row(
        &mut absent,
        p,
        1,
        "member_identity",
        "ix://test:spine-model/P/p2",
    );
    refuses_at_row("absent position", &absent, &rows, p, 1);
    // `idle` has no payload.
    let mut nullary = package.clone();
    *binding_mut(step_mut(&mut nullary, u, 1), "case") = json!("idle");
    set_row(
        &mut nullary,
        u,
        1,
        "member_identity",
        "ix://test:spine-model/U/idle",
    );
    refuses_at_row("nullary case", &nullary, &rows, u, 1);
    // `armed` has no such case in the row.
    let mut no_member = package.clone();
    *binding_mut(step_mut(&mut no_member, u, 1), "case") = json!("gone");
    set_row(
        &mut no_member,
        u,
        1,
        "member_identity",
        "ix://test:spine-model/U/gone",
    );
    refuses_at_row("absent case", &no_member, &rows, u, 1);
    // The identity rows list no tuple or union row.
    let mut bare = tuple_union_rows(&package);
    bare.tuples.clear();
    refuses_at_row("no tuple row", &package, &bare, p, 1);
    let mut bare = tuple_union_rows(&package);
    bare.unions.clear();
    refuses_at_row("no union row", &package, &bare, u, 1);
    // A position typed otherwise than the row says.
    let mut forged = tuple_union_rows(&package);
    forged.tuples[0].positions[0].member_type = Some(format!("Option<{CLOCK}>").into());
    refuses_at_row("forged presence", &package, &forged, p, 1);
    let mut forged = tuple_union_rows(&package);
    forged.unions[0].members[0].clock_payload[0].member_type =
        "ix://filament/native/Integer".into();
    refuses_at_row("forged payload", &package, &forged, u, 1);
}

/// A `p`, `tuple(0)`, `slot` place over `tuple P (Sequence<Clock>[0, 2],
/// Integer)`: the slot's bounds are the position's, checked at the join.
fn sequence_position(package: &Value) -> (Value, usize) {
    let mut package = package.clone();
    let (p, q) = (place_of(&package, "p"), place_of(&package, "q"));
    let slot_step = step_mut(&mut package, q, 1).clone();
    let slot_row = row_mut(&mut package, q, 1).clone();
    let sequence = format!("Sequence<{CLOCK}>");
    let tuple_row = row_mut(&mut package, p, 1);
    *binding_mut(tuple_row, "member_type") = json!(sequence);
    *binding_mut(tuple_row, "collection_kind") = json!("sequence");
    *binding_mut(tuple_row, "minimum") = json!("0");
    *binding_mut(tuple_row, "maximum") = json!("2");
    let mut step = slot_step;
    *binding_mut(&mut step, "maximum") = json!(2);
    let mut row = slot_row;
    *binding_mut(&mut row, "path_step") = json!(2);
    package["semantic_graph"]["nodes"][p]["body"]["members"][1]["value"]["members"]
        .as_array_mut()
        .expect("steps")
        .push(step);
    rows_mut(&mut package, p).push(row);
    // The slot step's selector is an `Integer` the place now depends on.
    let integer = package["semantic_graph"]["nodes"][q]["dependencies"][1].clone();
    package["semantic_graph"]["nodes"][p]["dependencies"]
        .as_array_mut()
        .expect("dependencies")
        .push(integer);
    (package, p)
}

#[trace("TC-481", "FR-374-AC-3")]
#[test]
fn a_slot_after_a_tuple_position_is_admitted_and_joined_against_the_position() {
    let package = tuple_union();
    let (package, p) = sequence_position(&package);
    let mut rows = tuple_union_rows(&package);
    rows.tuples[0].positions[0].member_type = Some(format!("Sequence<{CLOCK}>").into());
    rows.tuples[0].positions[0].collections = vec![layer("sequence", "0", "2")];
    join_rows(&package, &rows).expect("the strict read admits it and the join accepts");
    let mut wrong = rows;
    wrong.tuples[0].positions[0].collections = vec![layer("sequence", "0", "3")];
    refuses_at_row("slot bounds", &package, &wrong, p, 1);
}

/// `package` with the data field `field` of `owner` retyped to the node at
/// position `to`.
fn retyped(package: &Value, owner: &str, field: &str, to: usize) -> Value {
    let mut package = package.clone();
    let at = nodes(&package)
        .iter()
        .position(|node| node["declaration"]["qualified_name"][0] == owner)
        .expect("owner");
    let new = nodes(&package)[to]["node_id"].clone();
    let body = &mut package["semantic_graph"]["nodes"][at]["body"]["members"];
    let binding = body
        .as_array_mut()
        .expect("members")
        .iter_mut()
        .find(|member| member["name"] == field)
        .expect("binding");
    let old = std::mem::replace(&mut binding["value"]["target"], new.clone());
    for dependency in package["semantic_graph"]["nodes"][at]["dependencies"]
        .as_array_mut()
        .expect("dependencies")
    {
        if *dependency == old {
            *dependency = new.clone();
        }
    }
    package
}

fn position_where(package: &Value, wanted: impl Fn(&Value) -> bool) -> usize {
    nodes(package).iter().position(wanted).expect("node")
}

#[trace("TC-481", "FR-374-AC-2")]
#[test]
fn a_data_field_retyped_to_another_declaration_refuses_at_the_strict_read() {
    let package = emitted();
    let w = place(&package, "W/c");
    for (case, form) in [("source unit", "unit"), ("native integer", "integer")] {
        let to = position_where(&package, |node| node["semantic_form"] == form);
        let forged = retyped(&package, "O", "w", to);
        let refusal = refused(&forged);
        assert_eq!(refusal.code, Code::InvalidPackage, "{case}: {refusal:?}");
        assert_eq!(refusal.cause, Some(Cause::MalformedClockPlace), "{case}");
        assert_eq!(
            refusal.loci[0].path.as_ref().map(|at| at.as_str()),
            Some(format!("/semantic_graph/nodes/{w}/body/members/2/value/members/0").as_str()),
            "{case}"
        );
    }
}

#[trace("TC-481", "FR-374-AC-2")]
#[test]
fn a_data_field_over_a_range_stays_admitted_until_fspec_renders_it() {
    let package = tuple_union();
    let range = position_where(&package, |node| node["semantic_form"] == "integer_range");
    admitted(&retyped(&package, "O", "w", range));
}

fn id_of(path: &[&str]) -> Value {
    json!({"ordinal": 0, "package": "pkg", "path": path})
}

fn text_literal(package: &Value, value: &str) -> Value {
    let text = nodes(package)
        .iter()
        .find(|node| node["semantic_form"] == "text")
        .expect("text node")["node_id"]
        .clone();
    json!({"term": "literal", "type": text, "value": value, "value_kind": "text"})
}

fn binding(name: &str, value: Value) -> Value {
    json!({"term": "binding", "name": name, "value": value})
}

fn clause_node(path: &[&str], form: &str, body: Value, dependencies: Vec<Value>) -> Value {
    json!({
        "body": body, "dependencies": dependencies, "node_id": id_of(path),
        "node_tag": "timed_model", "occurrences": [{"ordinal": 0, "role": "expression"}],
        "schema_version": "filament.checked-semantic-graph/v2", "semantic_form": form,
        "semantic_type": id_of(path),
    })
}

/// The package FSL emits for `object O { d: Clock; q: Sequence<Clock>[0, 3];
/// ...; operation arm(x: Clock) modifies self.d; }`, with the two clauses FSL
/// does not emit written by hand: a `parameter` reset of `parameter` on the
/// anchor of `arm`, whose frame writes the field `written`. (A constraint is
/// only reachable through a guard, which needs an expression node; the join
/// reads a reset and a constraint alike.)
fn with_parameter_clauses(parameter: &str, written: &str) -> Value {
    let mut package = collections();
    let o = node_of(&package, "O");
    let o = serde_json::to_value(o).expect("O id");
    let ms = nodes(&package)
        .iter()
        .find(|node| node["semantic_form"] == "unit")
        .expect("unit")["node_id"]
        .clone();
    let frame = json!({
        "body": {"creates": [], "deletes": [], "deletes_scoped": [], "term": "frame",
            "modifies": [{"declaration": o, "kind": "field", "name": written, "scope": "receiver"}]},
        "dependencies": [o], "node_id": id_of(&["state", "frame", "arm"]),
        "node_tag": "state", "occurrences": [{"ordinal": 0, "role": "generated"}],
        "schema_version": "filament.checked-semantic-graph/v2", "semantic_form": "frame",
        "semantic_type": o,
    });
    let anchor_id = id_of(&["state", "operation_anchor", "arm"]);
    let anchor = json!({
        "body": {"term": "aggregate", "members": [
            binding("context", json!({"term": "reference", "target": o})),
            binding("operation", text_literal(&package, "arm")),
            binding("frame", json!({"term": "reference", "target": id_of(&["state", "frame", "arm"])})),
        ]},
        "dependencies": [o, id_of(&["state", "frame", "arm"])], "node_id": anchor_id,
        "node_tag": "state", "occurrences": [{"ordinal": 0, "role": "anchor"}],
        "schema_version": "filament.checked-semantic-graph/v2", "semantic_form": "operation_anchor",
        "semantic_type": o,
    });
    let quantity = |value: &str| json!({"term": "literal", "type": ms, "value": value, "value_kind": "quantity"});
    let operation = binding(
        "operation",
        json!({"term": "reference", "target": anchor_id}),
    );
    let reset_path = ["timed_model", "clock_reset", "arm"];
    let reset = clause_node(
        &reset_path,
        "clock_reset",
        json!({"term": "aggregate", "members": [
            binding("kind", text_literal(&package, "parameter")), operation,
            binding("parameter", text_literal(&package, parameter)), binding("value", quantity("0/1")),
        ]}),
        vec![anchor_id.clone()],
    );
    let graph = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let root = graph.pop().expect("root");
    graph.extend([frame, anchor, reset, root]);
    let root = graph.last_mut().expect("root");
    let reference = |path: &[&str]| json!({"term": "reference", "target": id_of(path)});
    let members = root["body"]["members"].as_array_mut().expect("members");
    let at = members
        .iter_mut()
        .find(|member| member["name"] == "resets")
        .expect("array");
    at["value"]["members"] = json!([reference(&reset_path)]);
    let region = json!([{"end": 332, "start": 311,
        "source": {"authority": "agent-ix", "identity": "test:spine-model"}}]);
    let entry = |path: &[&str], role: &str| json!({"node_id": id_of(path), "ordinal": 0, "regions": region, "role": role});
    let entries = [
        entry(&["state", "frame", "arm"], "generated"),
        entry(&["state", "operation_anchor", "arm"], "anchor"),
        entry(&reset_path, "expression"),
    ];
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .extend(entries);
    let root = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .last_mut()
        .expect("root");
    let dependencies = root["dependencies"].as_array_mut().expect("dependencies");
    // Dependencies are in node-id order: the reset precedes the time source.
    let last = dependencies.len() - 1;
    dependencies.insert(last, id_of(&reset_path));
    package
}

#[trace("TC-481", "FR-374-AC-4")]
#[test]
fn a_parameter_reset_joins_the_operations_clock_parameters() {
    let package = with_parameter_clauses("x", "d");
    let object = collections_object(&package);
    joined(&package, object.clone(), Some(record_row(&package))).expect("x is a Clock parameter");
    let reset = nodes(&package).len() - 2;
    let at = format!("/semantic_graph/nodes/{reset}/body/members/2");
    // `k` names no Clock parameter of `arm`.
    let other = with_parameter_clauses("k", "d");
    let error = joined(&other, collections_object(&other), Some(record_row(&other)))
        .expect_err("k is no clock parameter");
    assert_eq!(error.code, RowError::TimedNodeReference);
    assert_eq!(locus_path(&error), at);
    // `x` is a parameter whose declared type is no Clock.
    let integer = position_where(&package, |node| node["semantic_form"] == "integer");
    let integer = serde_json::from_value(nodes(&package)[integer]["node_id"].clone()).expect("id");
    let mut data = object;
    data.operations[0].clock_parameters.clear();
    data.operations[0].parameters = vec![SelectedSourceObjectMember::data("x", integer, false)];
    let error = joined(&package, data, Some(record_row(&package))).expect_err("a data x");
    assert_eq!(error.code, RowError::TimedNodeReference);
    assert_eq!(locus_path(&error), at);
}

#[trace("TC-481", "FR-374-AC-4")]
#[test]
fn a_parameter_reset_whose_frame_writes_no_clock_refuses_at_the_strict_read() {
    // `r` is a data field of `O`: the frame writes no clock.
    let data = with_parameter_clauses("x", "r");
    let reset = nodes(&data).len() - 2;
    let refusal = refused(&data);
    assert_eq!(refusal.code, Code::InvalidPackage);
    assert_eq!(refusal.cause, Some(Cause::TimedNodeReference));
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|at| at.as_str()),
        Some(format!("/semantic_graph/nodes/{reset}/body/members/1/value/target").as_str())
    );
    // `d` is a clock member of `O`: no binding names it.
    admitted(&with_parameter_clauses("x", "d"));
}

#[trace("TC-481", "FR-374-AC-2")]
#[test]
fn a_declaring_type_of_the_wrong_declaration_kind_refuses_at_the_strict_read() {
    let package = tuple_union();
    let (p, w) = (place_of(&package, "p"), place_of(&package, "w"));
    let cases: [(&str, usize, &str, &str); 3] = [
        (
            "a record under a tuple step",
            p,
            "declaring_type",
            "ix://test:spine-model/W",
        ),
        (
            "a tuple under a field step",
            w,
            "declaring_type",
            "ix://test:spine-model/P",
        ),
        (
            "an object type under a tuple step",
            p,
            "declaring_type",
            "ix://test:spine-model/O",
        ),
    ];
    for (case, node, name, value) in cases {
        let mut broken = package.clone();
        set_row(&mut broken, node, 1, name, value);
        let refusal = refused(&broken);
        assert_eq!(
            refusal.cause,
            Some(Cause::MalformedClockPlace),
            "{case}: {refusal:?}"
        );
    }
}
