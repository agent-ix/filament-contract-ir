// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-477: the strict reader admits `claim_names` and refuses every dangling,
//! duplicate and uncovered row (FSpec FR-322-AC-457 and FR-322-AC-458).

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
    encode_checked_package_v2, CheckedPackageEvidence, CheckedPackageLimit,
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
    assert_refusal(
        read_state(&package),
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/claim_names/0/qualified_name",
    );
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
    assert_eq!(name.as_deref(), Some(body_name.as_str()));
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

    // A body without a name has none for the row to equal.
    let mut unnamed = package;
    unnamed["semantic_graph"]["nodes"][position]["body"]
        .as_object_mut()
        .expect("body")
        .remove("name");
    assert_refusal(
        read_with(&unnamed, &evidence, CheckedPackageReadLimits::bounded()),
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        &format!("/claim_names/{index}/name"),
    );
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
    assert_refusal(
        read_state(&package),
        Code::AmbiguousDeclaration,
        Some(Cause::AmbiguousName),
        "/claim_names/1/qualified_name",
    );
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
    assert_refusal(
        read_state(&package),
        Code::InvalidPackage,
        Some(Cause::ConflictingDefinition),
        "/claim_names/1",
    );
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
    let evidence = step_relation_evidence(package);
    let incomplete = |work: u64| {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = work;
        match crate::read_and_account(&canonical(package), limits, &evidence) {
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
    // second pays for row 0's visit, its occurrence scan and its own visit.
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
    assert_eq!(second - first, 1 + scanned as u64);
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
        (1 + own + 1, before + own + 1)
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
