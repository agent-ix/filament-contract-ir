// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! IR-1588: the requirement-record orderings select the same first refusal
//! through the public reader on a chain of scalar applications. Each
//! expectation is the refusal the reader returned before IR-1588 moved its
//! orderings onto structural-key ranks (this file passes on both).

use super::{assert_refusal, large_requirement_record_package, read};
use filament_contract_model::{
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedPackageV2ReadResult,
};
use ix_trace_rs::trace;
use serde_json::Value;

const RECORDS: usize = 48;

fn rows(package: &mut Value) -> &mut Vec<Value> {
    package["requirement_items"]
        .as_array_mut()
        .expect("requirement rows")
}

/// The graph position of the record node that row `row` names.
fn record_position(package: &Value, row: usize) -> usize {
    let record = &package["requirement_items"][row]["record"];
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .position(|node| node["node_id"] == *record)
        .expect("named record node")
}

#[trace("TC-488", "FR-376-AC-2")]
#[test]
fn chain_records_admit_and_their_order_defects_refuse_at_the_first_defect() {
    let package = large_requirement_record_package(RECORDS);
    let admitted = match read(&package) {
        CheckedPackageV2ReadResult::Admitted(admitted) => admitted,
        other => panic!("the chain package admits: {other:?}"),
    };
    assert_eq!(admitted.requirement_items().len(), RECORDS);

    // Two adjacent rows out of order: the later row refuses.
    for at in [0, RECORDS / 2, RECORDS - 2] {
        let mut swapped = package.clone();
        rows(&mut swapped).swap(at, at + 1);
        assert_refusal(
            &swapped,
            CheckedPackageRefusalCode::InvalidPackage,
            Some(CheckedPackageRefusalCause::InvalidSemanticGraph),
            &format!("/requirement_items/{}", at + 1),
        );
    }

    // A row repeating its predecessor's key: the repeat refuses.
    for at in [1, RECORDS / 2, RECORDS - 1] {
        let mut repeated = package.clone();
        let previous = rows(&mut repeated)[at - 1].clone();
        rows(&mut repeated)[at] = previous;
        assert_refusal(
            &repeated,
            CheckedPackageRefusalCode::InvalidPackage,
            Some(CheckedPackageRefusalCause::ConflictingDefinition),
            &format!("/requirement_items/{at}"),
        );
    }

    // Two records' generated ordinals exchanged: the first record (in owner,
    // then row, order) whose ordinal is not its dense position refuses.
    for at in [0, RECORDS / 2, RECORDS - 2] {
        let mut exchanged = package.clone();
        let (first, second) = (
            record_position(&exchanged, at),
            record_position(&exchanged, at + 1),
        );
        let nodes = exchanged["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("graph nodes");
        let first_ordinal = nodes[first]["occurrences"][0]["ordinal"].clone();
        nodes[first]["occurrences"][0]["ordinal"] =
            nodes[second]["occurrences"][0]["ordinal"].clone();
        nodes[second]["occurrences"][0]["ordinal"] = first_ordinal;
        crate::fixture_support::rebuild_source_map(&mut exchanged);
        assert_refusal(
            &exchanged,
            CheckedPackageRefusalCode::InvalidPackage,
            Some(CheckedPackageRefusalCause::InvalidValue),
            &format!("/semantic_graph/nodes/{first}/occurrences"),
        );
    }
}
