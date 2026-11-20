// SPDX-License-Identifier: AGPL-3.0-or-later
//! FSpec's `body_locations` vectors (FR-322-AC-490, TC-2090) against the
//! production reader. The vectors are read in place from the checkout named by
//! `FILAMENT_SPECIFICATION_DIR`; nothing of them is copied here. The reserved
//! `#ieee_operand` binder name is in the vectors' packages, so every positive
//! vector also proves the reader admits it.

use crate::fixture_support::{canonical, evidence_for, refresh_identity};
use filament_contract_model::{
    CheckedPackageReadLimits, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    CheckedPackageV2ReadResult,
};
use ix_trace_rs::trace;
use serde_json::Value;
use std::{fs, path::PathBuf};

fn vectors() -> Value {
    let root = PathBuf::from(
        std::env::var_os("FILAMENT_SPECIFICATION_DIR")
            .expect("FILAMENT_SPECIFICATION_DIR must select the owning FSpec vector checkout"),
    );
    let path = root.join("proposals/checked-package-v2/checked-term-data-vectors.json");
    let all: Value = serde_json::from_slice(
        &fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display())),
    )
    .expect("the vectors are JSON");
    all["body_locations"].clone()
}

fn package_with(cases: &Value, case: &Value) -> Value {
    let mut package = cases["packages"][case["package"].as_str().unwrap()].clone();
    package["checked_term_data"]["body_locations"] = case["rows"].clone();
    refresh_identity(&mut package);
    package
}

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    crate::read_and_account(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

#[test]
#[trace("TC-2090", "FR-322-AC-490", "FR-038-AC-581")]
fn every_published_body_location_vector_runs_through_the_strict_reader() {
    let cases = vectors();
    let mut seen = Vec::new();
    for case in cases["positive"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let package = package_with(&cases, case);
        let result = read(&package);
        let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
            panic!("{id} must admit: {result:?}");
        };
        // The typed read keeps the captured rows, paths included.
        let held = admitted.checked_term_data().expect("the sidecar is kept");
        let held = serde_json::to_value(&held.body_locations).unwrap();
        assert_eq!(held, case["rows"], "{id}");
        seen.push(id.to_owned());
    }
    for case in cases["reader_refusals"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let package = package_with(&cases, case);
        let result = read(&package);
        let CheckedPackageV2ReadResult::Refused(refusal) = result else {
            panic!("{id} must refuse: {result:?}");
        };
        assert_eq!(
            serde_json::to_value(refusal.code).unwrap(),
            case["expected_code"],
            "{id}"
        );
        assert_eq!(
            serde_json::to_value(refusal.cause).unwrap(),
            case["expected_cause"],
            "{id}"
        );
        assert_eq!(
            refusal.loci[0].path.as_ref().unwrap().as_str(),
            case["locus"].as_str().unwrap(),
            "{id}"
        );
        seen.push(id.to_owned());
    }
    let carrierless = &cases["carrierless"];
    let mut bare = cases["packages"][carrierless["package"].as_str().unwrap()].clone();
    bare.as_object_mut().unwrap().remove("checked_term_data");
    refresh_identity(&mut bare);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&bare) else {
        panic!("the carrierless package must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::InvalidValue)
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().as_str(),
        carrierless["locus"].as_str().unwrap()
    );
    assert_eq!(
        seen,
        [
            "BL-01",
            "BL-02",
            "BL-03",
            "BL-R1",
            "BL-R2",
            "BL-R3",
            "BL-R4",
            "BL-R5",
            "BL-R6",
            "BL-R7-no-occurrence-1",
            "BL-R8-bl03-without-occurrence-1-row"
        ]
    );
}

/// The vectors' schema-negative cases are `malformed_wire` at the carrier or
/// origin; the reader refuses the carrier without `body_locations` the same way.
#[test]
#[trace("TC-2090", "FR-322-AC-490")]
fn a_carrier_without_body_locations_is_malformed_wire() {
    let cases = vectors();
    let mut package = cases["packages"]["f"].clone();
    package["checked_term_data"]
        .as_object_mut()
        .unwrap()
        .remove("body_locations");
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("a carrier without `body_locations` must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);
}
