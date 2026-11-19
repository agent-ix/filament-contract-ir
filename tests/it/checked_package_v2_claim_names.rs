// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-477: the strict reader admits `claim_names` and refuses every dangling,
//! duplicate and uncovered row (FSpec FR-322-AC-457 and FR-322-AC-458).

use crate::checked_package_v2_claim_terminal::claim_node;
use crate::checked_package_v2_frame_entries::{
    read as read_state, step_relation_evidence, step_relation_package, StatePackage,
};
use crate::checked_package_v2_refinement::{
    indexed_package, machine_package, row_empty_refinement,
};
use crate::support::checked_package::{
    canonical, evidence_for, refresh_identity, settle, v2_nominal,
};
use filament_contract_model::{
    encode_checked_package_v2, CheckedNodeId, CheckedPackageEvidence, CheckedPackageLimit,
    CheckedPackageReadLimits, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, CheckedPackageSourceCorrespondenceRef, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedTerm, ClaimNameRow,
};
use ix_trace_rs::trace;
use serde_json::{json, Value};

fn rows(package: &Value) -> &Vec<Value> {
    package["claim_names"].as_array().expect("claim_names rows")
}

fn nodes(package: &Value) -> &Vec<Value> {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
}

fn claim_bearing(node: &Value) -> bool {
    node["node_tag"] == "claim"
        || (node["node_tag"] == "state" && node["semantic_form"] == "state_clause")
        || (node["node_tag"] == "relation" && node["semantic_form"] == "refinement")
}

/// The state-clause fixture with the rows renamed after the FSpec example.
fn state_package() -> Value {
    let mut package = StatePackage::new().value;
    let invariant = nodes(&package)
        .iter()
        .position(|node| {
            node["semantic_form"] == "state_clause"
                && node["body"]["operation"]["member"]["clause"] == "invariant"
        })
        .expect("invariant clause");
    let invariant_id = nodes(&package)[invariant]["node_id"].clone();
    let precondition = nodes(&package)
        .iter()
        .position(|node| {
            node["semantic_form"] == "state_clause"
                && node["body"]["operation"]["member"]["clause"] == "precondition"
        })
        .expect("precondition clause");
    let claim_id = nodes(&package)[precondition]["node_id"].clone();
    for row in package["claim_names"].as_array_mut().expect("rows") {
        let name = if row["declaration"] == invariant_id {
            "NotAboveThree"
        } else if row["declaration"] == claim_id {
            "ReachThree"
        } else {
            continue;
        };
        row["name"] = json!(name);
        row["qualified_name"] = json!([name]);
    }
    package
}

fn refinement_package() -> (Value, CheckedPackageEvidence) {
    let (mut package, evidence) = machine_package();
    row_empty_refinement(&mut package);
    // Indexing re-derives the rows, so the tests edit the indexed form.
    (indexed_package(&package), evidence)
}

fn read_with(
    package: &Value,
    evidence: &CheckedPackageEvidence,
    limits: CheckedPackageReadLimits,
) -> CheckedPackageV2ReadResult {
    crate::read_and_account(&canonical(package), limits, evidence)
}

fn admitted(result: CheckedPackageV2ReadResult) -> CheckedPackageV2 {
    match result {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("expected admission, got {other:?}"),
    }
}

fn refusal_of(
    result: CheckedPackageV2ReadResult,
) -> filament_contract_model::CheckedPackageRefusal {
    match result {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected refusal, got {other:?}"),
    }
}

#[track_caller]
fn assert_refusal(
    result: CheckedPackageV2ReadResult,
    code: Code,
    cause: Option<Cause>,
    pointer: &str,
) -> filament_contract_model::CheckedPackageRefusal {
    let refusal = refusal_of(result);
    assert_eq!(refusal.code, code, "{refusal:?}");
    assert_eq!(refusal.cause, cause, "{refusal:?}");
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|path| path.as_str()),
        Some(pointer),
        "{refusal:?}"
    );
    refusal
}

/// The refusal is located at the node the row `index` of `package` declares.
#[track_caller]
fn assert_row_locus(
    refusal: &filament_contract_model::CheckedPackageRefusal,
    package: &Value,
    index: usize,
) {
    let declared: CheckedNodeId =
        serde_json::from_value(package["claim_names"][index]["declaration"].clone())
            .expect("row declaration");
    assert_eq!(refusal.locus.as_ref(), Some(&declared), "{refusal:?}");
}

#[track_caller]
fn assert_document_pointer(refusal: &filament_contract_model::CheckedPackageRefusal, want: &str) {
    assert_eq!(
        refusal.document_pointer.as_ref().map(|p| p.as_str()),
        Some(want),
        "{refusal:?}"
    );
}

fn write_again(package: &CheckedPackageV2) -> Vec<u8> {
    encode_checked_package_v2(
        package.name(),
        package.lock(),
        package.graph(),
        CheckedPackageSourceCorrespondenceRef {
            source_map: package.source_map(),
            refinement_step_rows: package.refinement_step_rows(),
        },
        filament_contract_model::CheckedPackageSidecarsRef {
            temporal_clause_names: package.temporal_clause_names(),
            claim_names: package.claim_names(),
            operation_effects: None,
            operation_bodies: None,
            checked_term_data: package.checked_term_data(),
            temporal_bindings: package.temporal_bindings(),
            schedulability_claims: None,
        },
        package.capability_report(),
        package.diagnostics(),
        quire_canonical::Limits::new(u64::MAX),
    )
    .expect("writer")
}

#[trace("TC-477", "FR-371-AC-1")]
#[test]
fn rows_name_each_claim_bearing_occurrence_and_survive_write_and_read() {
    let package = state_package();
    let admitted = admitted(read_state(&package));
    let read_rows = admitted.claim_names().expect("claim_names member");
    let expected: Vec<ClaimNameRow> =
        serde_json::from_value(package["claim_names"].clone()).expect("typed rows");
    assert_eq!(read_rows, expected.as_slice());
    let names: Vec<_> = read_rows
        .iter()
        .map(|row| {
            row.qualified_name
                .iter()
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>()
        })
        .collect();
    assert!(names.contains(&vec!["NotAboveThree"]), "{names:?}");
    assert!(names.contains(&vec!["ReachThree"]), "{names:?}");
    // Every claim-bearing node occurrence of the graph has exactly one row.
    let occurrences: usize = nodes(&package)
        .iter()
        .filter(|node| claim_bearing(node))
        .flat_map(|node| node["occurrences"].as_array().expect("occurrences"))
        .filter(|occurrence| occurrence["role"] == "declaration" || occurrence["role"] == "claim")
        .count();
    assert_eq!(read_rows.len(), occurrences);
    for row in read_rows {
        let path: Vec<&str> = row.qualified_name.iter().map(AsRef::as_ref).collect();
        assert_eq!(admitted.claim_name_for_path(&path), Some(row));
    }
    assert_eq!(admitted.claim_name_for_path(&["Missing"]), None);
    assert_eq!(admitted.claim_name_for_path::<&str>(&[]), None);
    // The writer emits the rows in the supplied order and a read preserves them.
    let written = write_again(&admitted);
    let reread = admitted_again(&written, &package);
    assert_eq!(reread.claim_names(), admitted.claim_names());
    assert_eq!(reread.graph(), admitted.graph());
}

fn admitted_again(bytes: &[u8], package: &Value) -> CheckedPackageV2 {
    admitted(crate::read_and_account(
        bytes,
        CheckedPackageReadLimits::bounded(),
        &step_relation_evidence(package),
    ))
}

#[trace("TC-477", "FR-371-AC-1")]
#[test]
fn renaming_a_claim_changes_no_node_and_one_node_carries_two_occurrence_rows() {
    let package = state_package();
    let mut renamed = package.clone();
    renamed["claim_names"][0]["name"] = json!("Renamed");
    renamed["claim_names"][0]["qualified_name"] = json!(["Renamed"]);
    assert_eq!(nodes(&package), nodes(&renamed));
    assert_eq!(
        admitted(read_state(&package)).graph(),
        admitted(read_state(&renamed)).graph()
    );

    // Two structurally equal clauses are one node with two occurrences.
    let mut shared = package;
    let invariant = nodes(&shared)
        .iter()
        .position(|node| {
            node["semantic_form"] == "state_clause"
                && node["body"]["operation"]["member"]["clause"] == "invariant"
        })
        .expect("invariant clause");
    shared["semantic_graph"]["nodes"][invariant]["occurrences"]
        .as_array_mut()
        .expect("occurrences")
        .push(json!({"role": "claim", "ordinal": 1}));
    settle(&mut shared);
    let id = nodes(&shared)[invariant]["node_id"].clone();
    let mut entry = shared["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .find(|entry| entry["node_id"] == id)
        .expect("source entry")
        .clone();
    entry["role"] = json!("claim");
    entry["ordinal"] = json!(1);
    // The second clause is its own declaration, so it has its own region.
    entry["regions"][0]["start"] = json!(5000);
    entry["regions"][0]["end"] = json!(5001);
    shared["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(entry);
    refresh_identity(&mut shared);
    let id = nodes(&shared)[invariant]["node_id"].clone();
    for (index, row) in rows(&shared).clone().iter().enumerate() {
        if row["declaration"] != id {
            continue;
        }
        let name = if row["occurrence"]["ordinal"] == 0 {
            "A"
        } else {
            "B"
        };
        shared["claim_names"][index]["name"] = json!(name);
        shared["claim_names"][index]["qualified_name"] = json!([name]);
    }
    let admitted = admitted(read_state(&shared));
    let twins: Vec<_> = admitted
        .claim_names()
        .expect("rows")
        .iter()
        .filter(|row| serde_json::to_value(&row.declaration).expect("id") == id)
        .collect();
    assert_eq!(twins.len(), 2);
    assert_ne!(twins[0].occurrence, twins[1].occurrence);
    assert_ne!(twins[0].qualified_name, twins[1].qualified_name);
    // Structurally equal declarations share the node, and each name resolves
    // to its own declaration's row, not to the first row of the node.
    for twin in &twins {
        let path: Vec<&str> = twin.qualified_name.iter().map(AsRef::as_ref).collect();
        assert_eq!(admitted.claim_name_for_path(&path), Some(*twin), "{path:?}");
    }
    assert_eq!(
        admitted
            .claim_name_for_path(&["B"])
            .map(|row| row.occurrence.ordinal),
        Some(1)
    );
}

#[trace("TC-477", "FR-371-AC-1")]
#[test]
fn a_package_without_a_claim_bearing_node_has_no_member() {
    let package = v2_nominal();
    assert!(nodes(&package).iter().all(|node| !claim_bearing(node)));
    assert!(package.get("claim_names").is_none());
    let admitted = admitted(crate::read_and_account(
        &canonical(&package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&package),
    ));
    assert_eq!(admitted.claim_names(), None);
    assert_eq!(admitted.claim_name_for_path(&["A"]), None);
}

#[trace("TC-477", "FR-371-AC-3", "FR-371-AC-5")]
#[test]
fn a_member_in_a_package_without_a_claim_bearing_node_is_dangling() {
    let mut package = v2_nominal();
    let node = nodes(&package)[0]["node_id"].clone();
    package["claim_names"] = json!([{
        "declaration": node, "occurrence": {"role": "claim", "ordinal": 0},
        "name": "A", "qualified_name": ["A"],
        "declaration_region": {"source": package["lock"]["sources"][0], "start": 0, "end": 1},
    }]);
    assert_refusal(
        crate::read_and_account(
            &canonical(&package),
            CheckedPackageReadLimits::bounded(),
            &evidence_for(&package),
        ),
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/claim_names/0/declaration",
    );
}

#[trace("TC-477", "FR-371-AC-2")]
#[test]
fn the_closed_row_shape_refuses_every_damaged_member_as_malformed_wire() {
    let package = state_package();
    type Edit = fn(&mut Value);
    let cases: [(&str, Edit, &str); 9] = [
        (
            "extra member",
            |row| row["extra"] = json!(1),
            "/claim_names/0",
        ),
        (
            "missing name",
            |row| {
                row.as_object_mut().unwrap().remove("name");
            },
            "/claim_names/0",
        ),
        (
            "missing qualified_name",
            |row| {
                row.as_object_mut().unwrap().remove("qualified_name");
            },
            "/claim_names/0",
        ),
        (
            "name not an identifier",
            |row| row["name"] = json!("1bad"),
            "/claim_names/0/name",
        ),
        (
            "empty path",
            |row| row["qualified_name"] = json!([]),
            "/claim_names/0/qualified_name",
        ),
        (
            "segment not an identifier",
            |row| row["qualified_name"] = json!(["a b", "x"]),
            "/claim_names/0/qualified_name",
        ),
        (
            "role outside declaration and claim",
            |row| row["occurrence"]["role"] = json!("type"),
            "/claim_names/0/occurrence/role",
        ),
        (
            "negative ordinal",
            |row| row["occurrence"]["ordinal"] = json!(-1),
            "/claim_names/0/occurrence/ordinal",
        ),
        (
            "occurrence extra member",
            |row| row["occurrence"]["extra"] = json!(1),
            "/claim_names/0",
        ),
    ];
    for (case, edit, pointer) in cases {
        let mut damaged = package.clone();
        edit(&mut damaged["claim_names"][0]);
        let refusal = refusal_of(read_state(&damaged));
        assert_eq!(refusal.code, Code::MalformedWire, "{case}: {refusal:?}");
        assert_eq!(
            refusal.loci[0].path.as_ref().map(|path| path.as_str()),
            Some(pointer),
            "{case}: {refusal:?}"
        );
    }
}

#[trace("TC-477", "FR-371-AC-3")]
#[test]
fn a_row_must_name_a_claim_bearing_node_and_one_of_its_occurrences() {
    let package = state_package();
    let missing = json!({"package": "absent", "path": ["nothing"], "ordinal": 0});
    let other_kind = nodes(&package)
        .iter()
        .find(|node| !claim_bearing(node))
        .expect("a node that bears no claim")["node_id"]
        .clone();
    for (case, declaration) in [("dangling", missing), ("wrong kind", other_kind)] {
        let mut damaged = package.clone();
        damaged["claim_names"][0]["declaration"] = declaration;
        assert_refusal(
            read_state(&damaged),
            Code::InvalidPackage,
            Some(Cause::InvalidValue),
            "/claim_names/0/declaration",
        );
        let _ = case;
    }
    for occurrence in [
        json!({"role": "claim", "ordinal": 9}),
        json!({"role": "declaration", "ordinal": 9}),
        json!({"role": "claim", "ordinal": 7}),
    ] {
        let mut damaged = package.clone();
        damaged["claim_names"][0]["occurrence"] = occurrence;
        assert_refusal(
            read_state(&damaged),
            Code::InvalidPackage,
            Some(Cause::InvalidValue),
            "/claim_names/0/occurrence",
        );
    }
    // A role the node does not carry is not one of its occurrences.
    let mut damaged = package.clone();
    let row = &mut damaged["claim_names"][0];
    let other = if row["occurrence"]["role"] == "claim" {
        "declaration"
    } else {
        "claim"
    };
    row["occurrence"]["role"] = json!(other);
    assert_refusal(
        read_state(&damaged),
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/claim_names/0/occurrence",
    );
}

#[trace("TC-477", "FR-371-AC-3")]
#[test]
fn the_last_path_segment_must_be_the_name() {
    let mut package = state_package();
    package["claim_names"][0]["qualified_name"] = json!(["Scope", "Other"]);
    let refusal = assert_refusal(
        read_state(&package),
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/claim_names/0/qualified_name",
    );
    assert_row_locus(&refusal, &package, 0);
}

#[trace("TC-477", "FR-371-AC-3", "FR-371-AC-1")]
#[test]
fn a_refinement_row_name_equals_the_body_name() {
    let (package, evidence) = refinement_package();
    let position = nodes(&package)
        .iter()
        .position(|node| node["semantic_form"] == "refinement")
        .expect("refinement");
    let body_name = nodes(&package)[position]["body"]["name"]
        .as_str()
        .expect("the fixture names the refinement body")
        .to_owned();
    let admitted = admitted(read_with(
        &package,
        &evidence,
        CheckedPackageReadLimits::bounded(),
    ));
    let CheckedTerm::Refinement { name, .. } = &admitted.graph().nodes[position].body else {
        panic!("a refinement body");
    };
    assert_eq!(name.as_ref(), body_name.as_str());
    let row = admitted
        .claim_names()
        .expect("rows")
        .iter()
        .find(|row| row.name.as_ref() == body_name)
        .expect("the refinement row");
    let path: Vec<&str> = row.qualified_name.iter().map(AsRef::as_ref).collect();
    assert_eq!(admitted.claim_name_for_path(&path), Some(row));
    assert_eq!(path.last(), Some(&body_name.as_str()));
    // The body name survives write and read.
    let written = write_again(&admitted);
    let reread = admitted_from(&written, &evidence);
    assert_eq!(reread.graph(), admitted.graph());
    assert_eq!(reread.claim_names(), admitted.claim_names());

    // A row repeating another identifier than the body's refuses at the name.
    let mut renamed = package.clone();
    let index = rows(&renamed)
        .iter()
        .position(|row| row["name"] == json!(body_name))
        .expect("row");
    renamed["claim_names"][index]["name"] = json!("Other");
    renamed["claim_names"][index]["qualified_name"] = json!(["Other"]);
    assert_refusal(
        read_with(&renamed, &evidence, CheckedPackageReadLimits::bounded()),
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        &format!("/claim_names/{index}/name"),
    );

    // A refinement body without a name is a body-schema error (FR-322-AC-200).
    let mut unnamed = package.clone();
    unnamed["semantic_graph"]["nodes"][position]["body"]
        .as_object_mut()
        .expect("body")
        .remove("name");
    assert_refusal(
        read_with(&unnamed, &evidence, CheckedPackageReadLimits::bounded()),
        Code::InvalidSemanticGraph,
        None,
        &format!("/semantic_graph/nodes/{position}/body"),
    );
    // A name that is not text is refused as well.
    let mut numbered = package;
    numbered["semantic_graph"]["nodes"][position]["body"]["name"] = json!(7);
    let refusal = refusal_of(read_with(
        &numbered,
        &evidence,
        CheckedPackageReadLimits::bounded(),
    ));
    assert_eq!(refusal.code, Code::InvalidSemanticGraph, "{refusal:?}");
}

fn admitted_from(bytes: &[u8], evidence: &CheckedPackageEvidence) -> CheckedPackageV2 {
    admitted(crate::read_and_account(
        bytes,
        CheckedPackageReadLimits::bounded(),
        evidence,
    ))
}

#[trace("TC-477", "FR-371-AC-4")]
#[test]
fn a_repeated_qualified_name_is_ambiguous_at_the_later_row() {
    let mut package = state_package();
    let first = package["claim_names"][0]["qualified_name"].clone();
    let name = package["claim_names"][0]["name"].clone();
    package["claim_names"][1]["qualified_name"] = first;
    package["claim_names"][1]["name"] = name;
    let refusal = assert_refusal(
        read_state(&package),
        Code::AmbiguousDeclaration,
        Some(Cause::AmbiguousName),
        "/claim_names/1/qualified_name",
    );
    // Both rows are named: the later is the locus, the earlier the document
    // position.
    assert_document_pointer(&refusal, "/claim_names/0/qualified_name");
    assert_row_locus(&refusal, &package, 1);
}

#[trace("TC-477", "FR-371-AC-4")]
#[test]
fn a_repeated_key_conflicts_and_wins_over_the_repeated_name() {
    let mut package = state_package();
    let row = package["claim_names"][0].clone();
    package["claim_names"]
        .as_array_mut()
        .expect("rows")
        .insert(1, row);
    let refusal = assert_refusal(
        read_state(&package),
        Code::InvalidPackage,
        Some(Cause::ConflictingDefinition),
        "/claim_names/1",
    );
    assert_row_locus(&refusal, &package, 1);
}

#[trace("TC-477", "FR-371-AC-4")]
#[test]
fn rows_out_of_canonical_order_refuse_at_the_later_row() {
    let mut package = state_package();
    package["claim_names"]
        .as_array_mut()
        .expect("rows")
        .swap(0, 1);
    assert_refusal(
        read_state(&package),
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/claim_names/1",
    );
}

#[trace("TC-477", "FR-371-AC-5")]
#[test]
fn an_uncovered_occurrence_or_a_missing_member_refuses_at_claim_names() {
    let package = state_package();
    for index in [0, rows(&package).len() - 1] {
        let mut removed = package.clone();
        removed["claim_names"]
            .as_array_mut()
            .expect("rows")
            .remove(index);
        assert_refusal(
            read_state(&removed),
            Code::InvalidPackage,
            Some(Cause::InvalidValue),
            "/claim_names",
        );
    }
    let mut absent = package.clone();
    absent
        .as_object_mut()
        .expect("package")
        .remove("claim_names");
    assert_refusal(
        read_state(&absent),
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/claim_names",
    );
    // A defect in a visited row wins over the coverage the later rows lack.
    let mut both = package;
    let last = both["claim_names"].as_array().expect("rows").len() - 1;
    both["claim_names"]
        .as_array_mut()
        .expect("rows")
        .remove(last);
    both["claim_names"][0]["qualified_name"] = json!(["x", "y"]);
    assert_refusal(
        read_state(&both),
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/claim_names/0/qualified_name",
    );
}

/// The smallest work limit at which `package` is decided, not incomplete.
fn smallest_deciding_work(package: &Value) -> u64 {
    smallest_deciding_work_with(package, &step_relation_evidence(package))
}

fn smallest_deciding_work_with(package: &Value, evidence: &CheckedPackageEvidence) -> u64 {
    let incomplete = |work: u64| {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = work;
        match crate::read_and_account(&canonical(package), limits, evidence) {
            CheckedPackageV2ReadResult::Incomplete(incomplete) => {
                assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
                true
            }
            _ => false,
        }
    };
    let (mut low, mut high) = (0_u64, CheckedPackageReadLimits::bounded().work);
    assert!(incomplete(low));
    assert!(!incomplete(high));
    while low + 1 < high {
        let mid = low + (high - low) / 2;
        if incomplete(mid) {
            low = mid;
        } else {
            high = mid;
        }
    }
    high
}

#[trace("TC-477", "FR-371-AC-6")]
#[test]
fn each_visited_row_and_each_occurrence_inspected_costs_one_work_visit() {
    let package = state_package();
    assert!(rows(&package).len() > 1);
    // Two packages refused at the declaration join of row 0 and of row 1: the
    // closed-schema check pays one visit per row for both, so the second pays
    // only for row 0's occurrence scan and region comparisons.
    let dangling = |index: usize| {
        let mut damaged = package.clone();
        damaged["claim_names"][index]["declaration"] =
            json!({"package": "absent", "path": ["nothing"], "ordinal": 0});
        damaged
    };
    let first = smallest_deciding_work(&dangling(0));
    let second = smallest_deciding_work(&dangling(1));
    let declaration = &rows(&package)[0]["declaration"];
    let node = nodes(&package)
        .iter()
        .find(|node| &node["node_id"] == declaration)
        .expect("row 0 node");
    let wanted = &rows(&package)[0]["occurrence"];
    let scanned = node["occurrences"]
        .as_array()
        .expect("occurrences")
        .iter()
        .position(|occurrence| occurrence == wanted)
        .expect("row 0 occurrence")
        + 1;
    // Row 0 is joined (the scan) and then has its region compared, once for
    // each region of its occurrence's source-map entry.
    let regions = package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .find(|entry| {
            entry["node_id"] == rows(&package)[0]["declaration"]
                && entry["role"] == wanted["role"]
                && entry["ordinal"] == wanted["ordinal"]
        })
        .expect("entry")["regions"]
        .as_array()
        .expect("regions")
        .len();
    assert_eq!(second - first, (scanned + regions) as u64);
}

#[trace("TC-477", "FR-371-AC-6")]
#[test]
fn the_coverage_walk_costs_one_work_visit_per_occurrence_it_inspects() {
    let package = state_package();
    let all = rows(&package).len();
    assert!(all > 1);
    // How many occurrences row `index` scans to find its own, and how many
    // the coverage walk inspects before it reaches that row's occurrence once
    // its row is missing.
    let costs = |index: usize| {
        let row = &rows(&package)[index];
        let position = nodes(&package)
            .iter()
            .position(|node| node["node_id"] == row["declaration"])
            .expect("row node");
        let own = nodes(&package)[position]["occurrences"]
            .as_array()
            .expect("occurrences")
            .iter()
            .position(|occurrence| occurrence == &row["occurrence"])
            .expect("row occurrence");
        let before: usize = nodes(&package)[..position]
            .iter()
            .filter(|node| claim_bearing(node))
            .map(|node| node["occurrences"].as_array().expect("occurrences").len())
            .sum();
        (own + 1, before + own + 1)
    };
    let without = |index: usize| {
        let mut damaged = package.clone();
        damaged["claim_names"]
            .as_array_mut()
            .expect("rows")
            .remove(index);
        damaged
    };
    let (last, first) = (all - 1, 0);
    let rows_paid = |skipped: usize| -> usize {
        (0..all)
            .filter(|index| *index != skipped)
            .map(|index| costs(index).0)
            .sum()
    };
    let (_, walk_last) = costs(last);
    let (_, walk_first) = costs(first);
    assert_ne!(walk_last, walk_first);
    let measured_last = smallest_deciding_work(&without(last));
    let measured_first = smallest_deciding_work(&without(first));
    let expected = (rows_paid(last) + walk_last) as i64 - (rows_paid(first) + walk_first) as i64;
    assert_eq!(measured_last as i64 - measured_first as i64, expected);
}

#[trace("TC-477", "FR-371-AC-1", "FR-371-AC-3")]
#[test]
fn a_claim_node_with_declaration_and_claim_occurrences_has_a_row_for_each() {
    let package = step_relation_package();
    let relation = nodes(&package)
        .iter()
        .find(|node| node["semantic_form"] == "step_relation")
        .expect("claim node")["node_id"]
        .clone();
    let own: Vec<_> = rows(&package)
        .iter()
        .filter(|row| row["declaration"] == relation)
        .collect();
    let roles: Vec<_> = own
        .iter()
        .map(|row| row["occurrence"]["role"].clone())
        .collect();
    assert_eq!(roles, [json!("declaration"), json!("claim")]);
    let admitted = admitted(read_state(&package));
    assert_eq!(
        admitted.claim_names().expect("rows").len(),
        rows(&package).len()
    );
    // Dropping the claim occurrence's row leaves that occurrence uncovered.
    let mut uncovered = package;
    let index = rows(&uncovered)
        .iter()
        .position(|row| row["declaration"] == relation && row["occurrence"]["role"] == "claim")
        .expect("claim row");
    uncovered["claim_names"]
        .as_array_mut()
        .expect("rows")
        .remove(index);
    assert_refusal(
        read_state(&uncovered),
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/claim_names",
    );
}

fn read_all(package: &Value) -> CheckedPackageV2ReadResult {
    read_with(
        package,
        &evidence_for(package),
        CheckedPackageReadLimits::bounded(),
    )
}

/// The state-clause fixture with a state-graph claim beside its temporal
/// clauses: the index of the claim's row and the name of the first temporal
/// clause.
fn namespace_package() -> (Value, usize, String) {
    let mut package = StatePackage::new().value;
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(claim_node());
    settle(&mut package);
    assert!(package.get("temporal_clause_names").is_some());
    let claim = nodes(&package)
        .iter()
        .position(|node| node["semantic_form"] == "state_graph")
        .expect("state-graph claim");
    let index = rows(&package)
        .iter()
        .position(|row| row["declaration"] == nodes(&package)[claim]["node_id"])
        .expect("claim row");
    let temporal = package["temporal_clause_names"][0]["name"]
        .as_str()
        .expect("clause name")
        .to_owned();
    (package, index, temporal)
}

fn rename_row(package: &mut Value, index: usize, name: &str, path: Value) {
    package["claim_names"][index]["name"] = json!(name);
    package["claim_names"][index]["qualified_name"] = path;
}

#[trace("TC-477", "FR-371-AC-7")]
#[test]
fn a_claim_name_equal_to_a_temporal_clause_name_is_ambiguous() {
    let (package, index, temporal) = namespace_package();
    for path in [json!([temporal]), json!(["Scope", temporal])] {
        let mut damaged = package.clone();
        rename_row(&mut damaged, index, &temporal, path);
        let refusal = assert_refusal(
            read_all(&damaged),
            Code::AmbiguousDeclaration,
            Some(Cause::AmbiguousName),
            &format!("/claim_names/{index}/name"),
        );
        // Both rows are named: the claim row is the locus and the temporal
        // clause row the document position.
        assert_document_pointer(&refusal, "/temporal_clause_names/0/name");
        assert_row_locus(&refusal, &damaged, index);
    }
}

/// Two clauses may bear one name (the temporal table does not forbid it), and
/// the collision names the first of them: the earliest row of the table, not
/// the last one scanned.
#[trace("TC-477", "FR-371-AC-7")]
#[test]
fn a_collision_with_a_name_shared_by_temporal_rows_names_the_first_row() {
    let (mut package, index, temporal) = namespace_package();
    // A second clause node of the same body, keyed after the first, so that
    // its row follows the first row and bears the same name.
    let mut twin = nodes(&package)
        .iter()
        .find(|node| node["node_tag"] == "temporal" && node["semantic_form"] == "temporal_clause")
        .expect("temporal clause")
        .clone();
    let key = twin["node_id"]["path"][0].as_str().expect("key").to_owned();
    twin["node_id"]["path"][0] = json!(format!("{key}-twin"));
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(twin);
    settle(&mut package);
    let names: Vec<_> = package["temporal_clause_names"]
        .as_array()
        .expect("clause rows")
        .iter()
        .map(|row| row["name"].as_str().expect("name").to_owned())
        .collect();
    assert_eq!(names.len(), 2, "two clause rows");
    assert_eq!(names[0], names[1], "one name on both");
    assert_eq!(names[0], temporal);
    // The shared name alone admits.
    admitted(read_all(&package));
    rename_row(&mut package, index, &temporal, json!([temporal]));
    let refusal = assert_refusal(
        read_all(&package),
        Code::AmbiguousDeclaration,
        Some(Cause::AmbiguousName),
        &format!("/claim_names/{index}/name"),
    );
    assert_document_pointer(&refusal, "/temporal_clause_names/0/name");
}

/// The `source_map` regions of one occurrence.
fn regions_of(package: &Value, node: &Value, role: &str, ordinal: u64) -> Value {
    package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .find(|entry| {
            &entry["node_id"] == node && entry["role"] == role && entry["ordinal"] == ordinal
        })
        .expect("source entry")["regions"]
        .clone()
}

/// Gives `node` a second `claim` occurrence (ordinal 1) whose name-token region
/// is `regions`, and re-derives the rows: the new occurrence's row carries the
/// derived name and the declaration region the occurrence's regions give.
fn add_claim_occurrence(package: &mut Value, node: &Value, regions: Value) {
    let at = nodes(package)
        .iter()
        .position(|candidate| &candidate["node_id"] == node)
        .expect("node");
    package["semantic_graph"]["nodes"][at]["occurrences"]
        .as_array_mut()
        .expect("occurrences")
        .push(json!({"role": "claim", "ordinal": 1}));
    settle(package);
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(json!({"node_id": node, "role": "claim", "ordinal": 1, "regions": regions}));
    refresh_identity(package);
}

/// A copy of the occurrence's regions moved to start at `at`, one byte wide.
fn moved(regions: &Value, at: u64) -> Value {
    let mut moved = regions.clone();
    moved[0]["start"] = json!(at);
    moved[0]["end"] = json!(at + 1);
    moved
}

/// The row index of the node's `claim` occurrence `ordinal`.
fn row_of(package: &Value, node: &Value, ordinal: u64) -> usize {
    rows(package)
        .iter()
        .position(|row| {
            &row["declaration"] == node
                && row["occurrence"]["role"] == "claim"
                && row["occurrence"]["ordinal"] == ordinal
        })
        .expect("row")
}

#[trace("TC-477", "FR-371-AC-10")]
#[test]
fn the_rows_of_one_declaration_share_its_name_and_are_no_duplicates() {
    let (mut package, index, _) = namespace_package();
    let node = rows(&package)[index]["declaration"].clone();
    // One declaration, two occurrences of one node: the second occurrence's
    // name token lies inside the first's region, so both rows carry one region.
    let regions = regions_of(&package, &node, "claim", 0);
    add_claim_occurrence(&mut package, &node, regions);
    let (first, second) = (row_of(&package, &node, 0), row_of(&package, &node, 1));
    assert_eq!(
        rows(&package)[first]["declaration_region"],
        rows(&package)[second]["declaration_region"]
    );
    assert_eq!(
        rows(&package)[first]["name"],
        rows(&package)[second]["name"]
    );
    assert_eq!(
        rows(&package)[first]["qualified_name"],
        rows(&package)[second]["qualified_name"]
    );
    let admitted = admitted(read_all(&package));
    let path: Vec<String> = serde_json::from_value(rows(&package)[first]["qualified_name"].clone())
        .expect("qualified name");
    // The lookup returns the least `claim` row.
    assert_eq!(
        admitted.claim_name_for_path(&path),
        Some(&admitted.claim_names().expect("rows")[first])
    );
    // One declaration has one name: the second row renamed refuses at the name.
    let mut renamed = package.clone();
    rename_row(&mut renamed, second, "Other", json!(["Other"]));
    assert_refusal(
        read_all(&renamed),
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        &format!("/claim_names/{second}/name"),
    );
    // The same name over another region is another declaration: a duplicate.
    let mut apart = package.clone();
    let key = &mut apart["source_map"]
        .as_array_mut()
        .expect("source map")
        .iter_mut()
        .find(|entry| entry["node_id"] == node && entry["ordinal"] == 1)
        .expect("second occurrence")["regions"];
    *key = moved(key, 7000);
    let name = rows(&apart)[first]["name"].clone();
    let path = rows(&apart)[first]["qualified_name"].clone();
    refresh_identity(&mut apart);
    apart["claim_names"][second]["name"] = name;
    apart["claim_names"][second]["qualified_name"] = path;
    let refusal = assert_refusal(
        read_all(&apart),
        Code::AmbiguousDeclaration,
        Some(Cause::AmbiguousName),
        &format!("/claim_names/{second}/qualified_name"),
    );
    assert_document_pointer(&refusal, &format!("/claim_names/{first}/qualified_name"));
}

/// A derived conjunction node, as FSpec's `CL-P7` and `CL-R17` shape it: a
/// second node carries a row for the declaration of another.
/// Two nodes carry one name: `other` conjoins the declaration of `own`, so its
/// second row repeats `own`'s name, path and region. Returns the package, the
/// two node ids and the conjoined row.
fn conjoined_package() -> (Value, Value, Value, usize) {
    let (mut package, index, _) = namespace_package();
    let own = rows(&package)[0]["declaration"].clone();
    let other = rows(&package)[index]["declaration"].clone();
    assert_ne!(own, other);
    let own_region = regions_of(
        &package,
        &own,
        rows(&package)[0]["occurrence"]["role"]
            .as_str()
            .expect("role"),
        rows(&package)[0]["occurrence"]["ordinal"]
            .as_u64()
            .expect("ordinal"),
    );
    add_claim_occurrence(&mut package, &other, own_region);
    let conjoined = row_of(&package, &other, 1);
    for member in ["name", "qualified_name", "declaration_region"] {
        let held = rows(&package)[0][member].clone();
        package["claim_names"][conjoined][member] = held;
    }
    (package, own, other, conjoined)
}

#[trace("TC-477", "FR-371-AC-6")]
#[test]
fn the_own_node_check_costs_the_declaration_set_elements_it_compares() {
    let (package, own, other, conjoined) = conjoined_package();
    // `own`'s set is its one declaration; `other`'s set holds its own
    // declaration and `own`'s.
    let set = |node: &Value| -> usize {
        let mut keys: Vec<_> = rows(&package)
            .iter()
            .filter(|row| &row["declaration"] == node)
            .map(|row| row["declaration_region"].to_string())
            .collect();
        keys.sort();
        keys.dedup();
        keys.len()
    };
    let (inner, outer) = (set(&own), set(&other));
    assert!(inner < outer);
    let evidence = evidence_for(&package);
    let shared = smallest_deciding_work_with(&package, &evidence);
    // The same package with the conjoined row a declaration and name of its
    // own has no name on two nodes and compares no sets.
    let mut apart = package.clone();
    let region = &mut apart["claim_names"][conjoined]["declaration_region"];
    region["end"] = json!(region["end"].as_u64().expect("end") + 1);
    rename_row(&mut apart, conjoined, "Alone", json!(["Alone"]));
    admitted(read_all(&apart));
    assert_eq!(
        shared - smallest_deciding_work_with(&apart, &evidence),
        (inner + outer) as u64
    );
}

#[trace("TC-477", "FR-371-AC-10")]
#[test]
fn a_name_on_two_nodes_resolves_to_the_node_whose_declaration_set_nests() {
    let (package, own, other, _) = conjoined_package();
    let name = rows(&package)[0]["qualified_name"].clone();
    let path: Vec<String> = serde_json::from_value(name).expect("path");
    let admitted = admitted(read_all(&package));
    // The declaration's own node answers, never the node that conjoins it.
    assert_eq!(
        admitted.claim_name_for_path(&path),
        Some(&admitted.claim_names().expect("rows")[0])
    );
    // Give `own` a second declaration: the two sets `{own, B}` and
    // `{other, own}` no longer nest.
    let mut apart = package;
    add_claim_occurrence_row(&mut apart, &own, 9000, "B");
    let conjoined = row_of(&apart, &other, 1);
    let refusal = assert_refusal(
        read_all(&apart),
        Code::AmbiguousDeclaration,
        Some(Cause::AmbiguousName),
        &format!("/claim_names/{conjoined}/qualified_name"),
    );
    assert_document_pointer(&refusal, "/claim_names/0/qualified_name");
}

/// A second `claim` occurrence (ordinal 1) of `node` for a declaration named
/// `name` of its own at byte `at`; the rows are re-derived and the new row named.
fn add_claim_occurrence_row(package: &mut Value, node: &Value, at: u64, name: &str) {
    let held = package["claim_names"].clone();
    let regions = moved(&regions_of(package, node, "claim", 0), at);
    let next = nodes(package)
        .iter()
        .find(|candidate| &candidate["node_id"] == node)
        .map(|candidate| {
            candidate["occurrences"]
                .as_array()
                .expect("occurrences")
                .len()
        })
        .expect("node") as u64;
    let position = nodes(package)
        .iter()
        .position(|candidate| &candidate["node_id"] == node)
        .expect("node");
    package["semantic_graph"]["nodes"][position]["occurrences"]
        .as_array_mut()
        .expect("occurrences")
        .push(json!({"role": "claim", "ordinal": next}));
    settle(package);
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(json!({"node_id": node, "role": "claim", "ordinal": next, "regions": regions}));
    refresh_identity(package);
    let added = row_of(package, node, next);
    package["claim_names"][added]["name"] = json!(name);
    package["claim_names"][added]["qualified_name"] = json!([name]);
    // The earlier rows keep the names the test gave them.
    for (at, row) in held.as_array().expect("rows").iter().enumerate() {
        let mut position = at;
        if let Some(found) = rows(package).iter().position(|candidate| {
            candidate["declaration"] == row["declaration"]
                && candidate["occurrence"] == row["occurrence"]
        }) {
            position = found;
        }
        for member in ["name", "qualified_name"] {
            package["claim_names"][position][member] = row[member].clone();
        }
    }
}

#[trace("TC-477", "FR-371-AC-7")]
#[test]
fn distinct_claim_and_temporal_clause_names_resolve_to_one_row_each() {
    let (mut package, index, temporal) = namespace_package();
    rename_row(&mut package, index, "ReachThree", json!(["ReachThree"]));
    let admitted = admitted(read_all(&package));
    let claim = admitted
        .claim_name_for_path(&["ReachThree"])
        .expect("the claim row");
    assert_eq!(claim.name.as_ref(), "ReachThree");
    let clause: Vec<_> = admitted
        .temporal_clause_names()
        .expect("clause rows")
        .iter()
        .filter(|row| row.name.as_ref() == temporal)
        .collect();
    assert_eq!(clause.len(), 1);
    assert_eq!(admitted.claim_name_for_path(&[temporal.as_str()]), None);
}

#[trace("TC-477", "FR-371-AC-7", "FR-371-AC-6")]
#[test]
fn the_one_namespace_check_costs_one_work_visit_per_row() {
    let (package, _, temporal) = namespace_package();
    let (first, last) = (0, rows(&package).len() - 1);
    assert!(last > first);
    let evidence = evidence_for(&package);
    let cost = |index: usize| {
        let mut damaged = package.clone();
        rename_row(&mut damaged, index, &temporal, json!([temporal]));
        smallest_deciding_work_with(&damaged, &evidence)
    };
    // Each row visited before the refusing one costs one validation visit.
    assert_eq!(cost(last) - cost(first), (last - first) as u64);
}

#[trace("TC-477", "FR-371-AC-3")]
#[test]
fn a_row_naming_a_temporal_clause_node_is_dangling() {
    let (mut package, index, _) = namespace_package();
    let clause = nodes(&package)
        .iter()
        .find(|node| node["node_tag"] == "temporal" && node["semantic_form"] == "temporal_clause")
        .expect("temporal clause")["node_id"]
        .clone();
    package["claim_names"][index]["declaration"] = clause;
    package["claim_names"][index]["occurrence"] = json!({"role": "claim", "ordinal": 0});
    let refusal = assert_refusal(
        read_all(&package),
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        &format!("/claim_names/{index}/declaration"),
    );
    assert_row_locus(&refusal, &package, index);
}

#[trace("TC-477", "FR-371-AC-2")]
#[test]
fn an_empty_or_null_member_refuses_as_malformed_wire() {
    let package = v2_nominal();
    for member in [json!([]), json!(null)] {
        let mut damaged = package.clone();
        damaged["claim_names"] = member;
        assert_refusal(
            crate::read_and_account(
                &canonical(&damaged),
                CheckedPackageReadLimits::bounded(),
                &evidence_for(&damaged),
            ),
            Code::MalformedWire,
            None,
            "/claim_names",
        );
    }
}

#[trace("TC-477", "FR-371-AC-2")]
#[test]
fn a_malformed_later_row_wins_over_a_dangling_earlier_row() {
    let mut package = state_package();
    package["claim_names"][0]["declaration"] =
        json!({"package": "absent", "path": ["nothing"], "ordinal": 0});
    package["claim_names"][1]["name"] = json!("not an identifier");
    assert_refusal(
        read_state(&package),
        Code::MalformedWire,
        None,
        "/claim_names/1/name",
    );
}

#[trace("TC-477", "FR-371-AC-1")]
#[test]
fn the_fspec_example_names_a_state_clause_and_a_state_graph_claim() {
    let (mut package, claim_row, _) = namespace_package();
    rename_row(&mut package, claim_row, "ReachThree", json!(["ReachThree"]));
    let clause_row = rows(&package)
        .iter()
        .position(|row| {
            let node = nodes(&package)
                .iter()
                .find(|node| node["node_id"] == row["declaration"])
                .expect("row node");
            node["semantic_form"] == "state_clause"
                && node["body"]["operation"]["member"]["clause"] == "invariant"
        })
        .expect("the invariant clause row");
    rename_row(
        &mut package,
        clause_row,
        "NotAboveThree",
        json!(["NotAboveThree"]),
    );
    let admitted = admitted(read_all(&package));
    let names: Vec<_> = admitted
        .claim_names()
        .expect("rows")
        .iter()
        .map(|row| {
            row.qualified_name
                .iter()
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>()
        })
        .collect();
    assert!(names.contains(&vec!["NotAboveThree"]));
    assert!(names.contains(&vec!["ReachThree"]));
    let invariant = admitted
        .claim_name_for_path(&["NotAboveThree"])
        .expect("clause");
    let reach = admitted
        .claim_name_for_path(&["ReachThree"])
        .expect("claim");
    assert_ne!(invariant.declaration, reach.declaration);
    // Renaming the clause changes no node identity.
    let before = nodes(&package).clone();
    let mut again = package.clone();
    rename_row(&mut again, clause_row, "Renamed", json!(["Renamed"]));
    assert_eq!(&before, nodes(&again));
    admitted_from(&canonical(&again), &evidence_for(&again));
}

#[trace("TC-477", "FR-371-AC-8")]
#[test]
fn two_refinements_with_one_name_are_ambiguous_at_the_later_body_name() {
    let (package, evidence) = refinement_package();
    let position = nodes(&package)
        .iter()
        .position(|node| node["semantic_form"] == "refinement")
        .expect("refinement");
    // A second refinement node, equal in every member but its `name`, then
    // renamed to the first one's name without re-deriving identity.
    let mut twin = package.clone();
    let mut node = nodes(&twin)[position].clone();
    let name = node["body"]["name"].as_str().expect("name").to_owned();
    node["body"]["name"] = json!("Twin");
    node["node_id"]["path"] = json!(["fixture", "twin"]);
    twin["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(node);
    let later = nodes(&twin).len() - 1;
    twin["semantic_graph"]["nodes"][later]["body"]["name"] = json!(name);
    let refusal = refusal_of(read_with(
        &twin,
        &evidence,
        CheckedPackageReadLimits::bounded(),
    ));
    assert_eq!(refusal.code, Code::AmbiguousDeclaration, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::AmbiguousName), "{refusal:?}");
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{later}/body/name").as_str()),
        "{refusal:?}"
    );
    assert_document_pointer(
        &refusal,
        &format!("/semantic_graph/nodes/{position}/body/name"),
    );
    let later_id: CheckedNodeId =
        serde_json::from_value(nodes(&twin)[later]["node_id"].clone()).expect("node id");
    assert_eq!(refusal.locus.as_ref(), Some(&later_id), "{refusal:?}");
}

/// The first `source_map` region of the occurrence the row `index` of `table`
/// names.
fn occurrence_region_of(package: &Value, table: &str, index: usize) -> Value {
    let row = &package[table][index];
    let (role, ordinal) = if table == "claim_names" {
        (
            row["occurrence"]["role"].clone(),
            row["occurrence"]["ordinal"].clone(),
        )
    } else {
        (json!("claim"), row["occurrence"].clone())
    };
    package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .find(|entry| {
            entry["node_id"] == row["declaration"]
                && entry["role"] == role
                && entry["ordinal"] == ordinal
        })
        .expect("the occurrence's source_map entry")["regions"][0]
        .clone()
}

const TABLES: [&str; 2] = ["claim_names", "temporal_clause_names"];

#[trace("TC-477", "FR-371-AC-9")]
#[test]
fn a_row_carries_the_declaration_region_and_the_reader_exposes_it() {
    let (package, _, _) = namespace_package();
    for table in TABLES {
        let region = package[table][0]["declaration_region"].clone();
        assert_eq!(region, occurrence_region_of(&package, table, 0));
    }
    // A declaration wider than the name token admits and moves no node.
    let narrow = admitted(read_all(&package));
    let mut wide = package.clone();
    for table in TABLES {
        let end = wide[table][0]["declaration_region"]["end"]
            .as_u64()
            .expect("end");
        wide[table][0]["declaration_region"]["end"] = json!(end + 40);
    }
    let widened = admitted(read_all(&wide));
    assert_eq!(widened.graph(), narrow.graph());
    let claim = &widened.claim_names().expect("rows")[0];
    assert_eq!(
        serde_json::to_value(&claim.declaration_region).expect("region"),
        wide["claim_names"][0]["declaration_region"]
    );
    let clause = &widened.temporal_clause_names().expect("rows")[0];
    assert_eq!(
        serde_json::to_value(&clause.declaration_region).expect("region"),
        wide["temporal_clause_names"][0]["declaration_region"]
    );
    // The regions survive write and read.
    let reread = admitted_from(&write_again(&widened), &evidence_for(&wide));
    assert_eq!(reread.claim_names(), widened.claim_names());
    assert_eq!(
        reread.temporal_clause_names(),
        widened.temporal_clause_names()
    );
}

#[trace("TC-477", "FR-371-AC-9")]
#[test]
fn a_missing_declaration_region_is_refused_at_the_row_and_a_malformed_one_at_the_member() {
    let (package, _, _) = namespace_package();
    for table in TABLES {
        let at = format!("/{table}/0/declaration_region");
        let mut missing = package.clone();
        missing[table][0]
            .as_object_mut()
            .expect("row")
            .remove("declaration_region");
        assert_refusal(
            read_all(&missing),
            Code::MalformedWire,
            None,
            &format!("/{table}/0"),
        );
        for damaged in [
            json!(7),
            json!(null),
            json!({}),
            json!({"start": 0, "end": 1}),
        ] {
            let mut bad = package.clone();
            bad[table][0]["declaration_region"] = damaged;
            assert_refusal(read_all(&bad), Code::MalformedWire, None, &at);
        }
        let mut extra = package.clone();
        extra[table][0]["declaration_region"]["extra"] = json!(1);
        assert_refusal(read_all(&extra), Code::MalformedWire, None, &at);
        let mut part = package.clone();
        part[table][0]["declaration_region"]["start"] = json!("zero");
        assert_refusal(read_all(&part), Code::MalformedWire, None, &at);
    }
}

#[trace("TC-477", "FR-371-AC-9")]
#[test]
fn a_region_that_is_not_the_occurrences_declaration_is_invalid() {
    let (package, _, _) = namespace_package();
    for table in TABLES {
        let at = format!("/{table}/0/declaration_region");
        let held = occurrence_region_of(&package, table, 0);
        let end = held["end"].as_u64().expect("end");
        let start = held["start"].as_u64().expect("start");
        // Shrunk to one byte that does not contain the name token.
        let mut shrunk = package.clone();
        shrunk[table][0]["declaration_region"]["start"] = json!(end);
        shrunk[table][0]["declaration_region"]["end"] = json!(end + 1);
        // Cut short at either side of the token.
        let mut early = package.clone();
        early[table][0]["declaration_region"]["end"] = json!(end - 1);
        // Another source.
        let mut foreign = package.clone();
        foreign[table][0]["declaration_region"]["source"] =
            json!({"authority": "other", "identity": "elsewhere"});
        let mut cases = vec![shrunk, early, foreign];
        if start > 0 {
            let mut late = package.clone();
            late[table][0]["declaration_region"]["start"] = json!(start + 1);
            cases.push(late);
        }
        for damaged in cases {
            let refusal = assert_refusal(
                read_all(&damaged),
                Code::InvalidPackage,
                Some(Cause::InvalidValue),
                &at,
            );
            let declared: CheckedNodeId =
                serde_json::from_value(damaged[table][0]["declaration"].clone()).expect("id");
            assert_eq!(refusal.locus.as_ref(), Some(&declared), "{refusal:?}");
        }
    }
}

#[trace("TC-477", "FR-371-AC-9")]
#[test]
fn the_region_check_follows_the_join_and_precedes_the_repeated_key() {
    let (package, _, _) = namespace_package();
    // After the occurrence join: a row with a missing occurrence and a bad
    // region refuses at the occurrence.
    let mut joined = package.clone();
    joined["claim_names"][0]["occurrence"]["ordinal"] = json!(99);
    joined["claim_names"][0]["declaration_region"]["end"] = json!(0);
    joined["claim_names"][0]["declaration_region"]["start"] = json!(0);
    assert_refusal(
        read_all(&joined),
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/claim_names/0/occurrence",
    );
    // Before the repeated-key check: the repeated row's bad region wins.
    let mut repeated = package.clone();
    let mut row = repeated["claim_names"][0].clone();
    row["declaration_region"]["end"] = json!(0);
    row["declaration_region"]["start"] = json!(0);
    repeated["claim_names"]
        .as_array_mut()
        .expect("rows")
        .insert(1, row);
    assert_refusal(
        read_all(&repeated),
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/claim_names/1/declaration_region",
    );
    let mut repeated_clause = package;
    let mut row = repeated_clause["temporal_clause_names"][0].clone();
    row["declaration_region"]["end"] = json!(0);
    row["declaration_region"]["start"] = json!(0);
    repeated_clause["temporal_clause_names"]
        .as_array_mut()
        .expect("rows")
        .insert(1, row);
    assert_refusal(
        read_all(&repeated_clause),
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/temporal_clause_names/1/declaration_region",
    );
}

#[trace("TC-477", "FR-371-AC-6", "FR-371-AC-9")]
#[test]
fn each_source_region_the_declaration_check_compares_costs_one_work_visit() {
    let (package, _, _) = namespace_package();
    assert!(rows(&package).len() > 1);
    let evidence = evidence_for(&package);
    let refused_at = |index: usize| {
        let mut damaged = package.clone();
        damaged["claim_names"][index]["declaration_region"]["source"] =
            json!({"authority": "other", "identity": "elsewhere"});
        smallest_deciding_work_with(&damaged, &evidence)
    };
    let (first, second) = (refused_at(0), refused_at(1));
    let row = &rows(&package)[0];
    let node = nodes(&package)
        .iter()
        .find(|node| node["node_id"] == row["declaration"])
        .expect("row node");
    let scanned = node["occurrences"]
        .as_array()
        .expect("occurrences")
        .iter()
        .position(|occurrence| occurrence == &row["occurrence"])
        .expect("occurrence")
        + 1;
    let regions = package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .find(|entry| {
            entry["node_id"] == row["declaration"]
                && entry["role"] == row["occurrence"]["role"]
                && entry["ordinal"] == row["occurrence"]["ordinal"]
        })
        .expect("entry")["regions"]
        .as_array()
        .expect("regions")
        .len();
    // The second pays for row 0's join scan and its region comparisons.
    assert_eq!(second - first, (scanned + regions) as u64);
}

#[trace("TC-477", "FR-371-AC-9")]
#[test]
fn an_occurrence_without_a_source_map_entry_is_left_to_the_source_map_check() {
    let (mut package, _, _) = namespace_package();
    let row = package["claim_names"][0].clone();
    let entries = package["source_map"].as_array_mut().expect("source map");
    let before = entries.len();
    entries.retain(|entry| {
        !(entry["node_id"] == row["declaration"]
            && entry["role"] == row["occurrence"]["role"]
            && entry["ordinal"] == row["occurrence"]["ordinal"])
    });
    assert!(entries.len() < before);
    let refusal = refusal_of(read_all(&package));
    assert_eq!(refusal.code, Code::InvalidSourceMap, "{refusal:?}");
}

#[trace("TC-477", "FR-371-AC-6")]
#[test]
fn the_closed_schema_check_costs_one_work_visit_per_row() {
    let package = state_package();
    let all = rows(&package).len();
    assert!(all > 1);
    // Row 0 dangles in both packages, so the second only holds one more row
    // for the closed-schema check to visit.
    let dangling = |mut damaged: Value| {
        damaged["claim_names"][0]["declaration"] =
            json!({"package": "absent", "path": ["nothing"], "ordinal": 0});
        smallest_deciding_work(&damaged)
    };
    let mut fewer = package.clone();
    fewer["claim_names"].as_array_mut().expect("rows").pop();
    assert_eq!(dangling(package) - dangling(fewer), 1);
}

fn refinement_body_name_vectors() -> Value {
    let root = std::env::var_os("FILAMENT_SPECIFICATION_DIR")
        .expect("FILAMENT_SPECIFICATION_DIR is required: it names the selected FSpec checkout");
    let path = std::path::Path::new(&root)
        .join("proposals/checked-package-v2/refinement-body-name-vectors.json");
    serde_json::from_slice(&std::fs::read(&path).expect("published refinement body-name vectors"))
        .expect("refinement body-name vectors are JSON")
}

/// Gives the refinement at `position` the body `name` and renames its
/// `claim_names` rows' last segment to it.
fn name_refinement(package: &mut Value, position: usize, name: Value) {
    let declaration = nodes(package)[position]["node_id"].clone();
    package["semantic_graph"]["nodes"][position]["body"]["name"] = name.clone();
    let count = rows(package).len();
    for index in 0..count {
        if package["claim_names"][index]["declaration"] == declaration {
            package["claim_names"][index]["name"] = name.clone();
            let path = package["claim_names"][index]["qualified_name"]
                .as_array_mut()
                .expect("qualified name");
            *path.last_mut().expect("last segment") = name.clone();
        }
    }
}

/// Replays FSpec's `refinement-body-name-vectors.json` (FR-322-AC-200, TC-310)
/// over the refinement package: the authored `body.name` is read, written and
/// named by its claim row, a body of another name is another node, and a
/// missing, empty, qualified or non-text name is a body-schema error.
#[trace("TC-477", "FR-371-AC-8")]
#[test]
fn published_refinement_body_name_vectors_replay() {
    let published = refinement_body_name_vectors();
    let (package, evidence) = refinement_package();
    let position = nodes(&package)
        .iter()
        .position(|node| node["semantic_form"] == "refinement")
        .expect("refinement");
    let mut admitted_names = Vec::new();
    let mut admitted_ids = Vec::new();
    for case in published["positive"].as_array().expect("positive cases") {
        let id = case["id"].as_str().expect("case id");
        let name = case["body"]["name"].clone();
        assert_eq!(case["claim_name"], name, "{id}");
        let mut named = package.clone();
        name_refinement(&mut named, position, name.clone());
        // The vector's node id is the producer-assigned id of FR-322-AC-200: the
        // name is the last identity segment, so one more name is one more node.
        // It is given to the refinement under the package's own name, then read.
        let held = nodes(&named)[position]["node_id"].clone();
        let mut expected = case["expected_node_id"].clone();
        expected["package"] = held["package"].clone();
        assert_eq!(
            expected["path"],
            json!(["relation", "refinement", name]),
            "{id}"
        );
        crate::support::checked_package::reassign_node_id(&mut named, &held, &expected);
        refresh_identity(&mut named);
        let read = admitted(read_with(
            &named,
            &evidence,
            CheckedPackageReadLimits::bounded(),
        ));
        let CheckedTerm::Refinement {
            name: read_name, ..
        } = &read.graph().nodes[position].body
        else {
            panic!("{id}: a refinement body");
        };
        assert_eq!(&json!(read_name.as_ref()), &name, "{id}");
        assert_eq!(
            serde_json::to_value(&read.graph().nodes[position].node_id).expect("node id"),
            expected,
            "{id}: the admitted node id is the vector's expected_node_id"
        );
        admitted_ids.push(expected);
        assert!(
            read.claim_names()
                .expect("rows")
                .iter()
                .any(|row| json!(row.name.as_ref()) == case["claim_name"]),
            "{id}: the claim row repeats the body name"
        );
        let reread = admitted_from(&write_again(&read), &evidence);
        assert_eq!(reread.graph(), read.graph(), "{id}");
        admitted_names.push(name);
    }
    assert_eq!(admitted_names.len(), 2);
    assert_ne!(
        admitted_names[0], admitted_names[1],
        "two names are two nodes"
    );
    assert_ne!(
        admitted_ids[0], admitted_ids[1],
        "two names are two node ids"
    );
    for case in published["schema_refusals"].as_array().expect("refusals") {
        let id = case["id"].as_str().expect("case id");
        assert_eq!(case["schema"], "invalid", "{id}");
        let from = published["positive"]
            .as_array()
            .expect("positive")
            .iter()
            .find(|row| row["id"] == case["from"])
            .unwrap_or_else(|| panic!("{id}: its published base case"));
        let mut damaged = package.clone();
        name_refinement(&mut damaged, position, from["body"]["name"].clone());
        for patch in case["patch"].as_array().expect("patches") {
            let body = damaged["semantic_graph"]["nodes"][position]["body"]
                .as_object_mut()
                .expect("body");
            match patch["op"].as_str().expect("operation") {
                "remove" => {
                    body.remove("name").expect("name member");
                }
                "replace" => {
                    body.insert("name".to_owned(), patch["value"].clone());
                }
                other => panic!("{id}: unsupported patch {other}"),
            }
        }
        // The claim row keeps its published name, so the body is judged alone.
        let refusal = refusal_of(read_with(
            &damaged,
            &evidence,
            CheckedPackageReadLimits::bounded(),
        ));
        assert_eq!(
            refusal.code,
            Code::InvalidSemanticGraph,
            "{id}: {refusal:?}"
        );
        // A missing name is refused at the body, a name that is no identifier
        // at the `name` member.
        let at = if id == "RB-S1-missing-name" {
            format!("/semantic_graph/nodes/{position}/body")
        } else {
            format!("/semantic_graph/nodes/{position}/body/name")
        };
        assert_eq!(
            refusal.loci[0].path.as_ref().map(|path| path.as_str()),
            Some(at.as_str()),
            "{id}: {refusal:?}"
        );
    }
}
