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
#[trace("TC-491", "FR-038-AC-597", "FR-322-AC-510")]
fn shared_record_parents_preserve_two_visits_of_one_application_occurrence() {
    let cases = vectors();
    let case = cases["positive"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "BL-06")
        .unwrap();
    let package = package_with(&cases, case);
    let result = read(&package);
    let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
        panic!("BL-06 must admit: {result:?}");
    };
    let rows = &admitted.checked_term_data().unwrap().body_locations;
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].application, rows[1].application);
    assert_eq!(rows[0].occurrence, rows[1].occurrence);
    assert_eq!(rows[0].occurrence.ordinal, 2);
    assert_eq!((rows[0].term_ordinal, rows[1].term_ordinal), (0, 1));
    let filament_contract_model::CheckedBodyLocationOrigin::FunctionBody { function, .. } =
        &rows[0].location.origin;
    let mut cursor = admitted.body_locations(function, 1_000).unwrap();
    let first = cursor.next_location().unwrap().unwrap();
    assert_eq!(
        first.path,
        [
            filament_contract_model::CheckedKeyOrdinal::Safe(0),
            filament_contract_model::CheckedKeyOrdinal::Safe(0)
        ]
    );
    let second = cursor.next_location().unwrap().unwrap();
    assert_eq!(
        second.path,
        [
            filament_contract_model::CheckedKeyOrdinal::Safe(1),
            filament_contract_model::CheckedKeyOrdinal::Safe(0)
        ]
    );
    assert!(cursor.next_location().unwrap().is_none());
    for (member, value, locus) in [
        (
            "up",
            serde_json::json!(1),
            "/checked_term_data/body_locations/1/location/up",
        ),
        (
            "up",
            serde_json::json!(3),
            "/checked_term_data/body_locations/1/location/up",
        ),
    ] {
        let mut changed = package.clone();
        changed["checked_term_data"]["body_locations"][1]["location"][member] = value;
        if changed["checked_term_data"]["body_locations"][1]["location"]["up"] == 1 {
            changed["checked_term_data"]["body_locations"][1]["location"]["down"] =
                serde_json::json!([0, 1, 0]);
        }
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&changed) else {
            panic!("bad compact path must refuse");
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
        assert_eq!(refusal.loci[0].path.as_ref().unwrap().as_str(), locus);
    }
    let mut changed = package.clone();
    changed["checked_term_data"]["body_locations"][1]["occurrence"]["ordinal"] =
        serde_json::json!(3);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&changed) else {
        panic!("absent occurrence must refuse");
    };
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().as_str(),
        "/checked_term_data/body_locations/1/occurrence"
    );
}

#[test]
#[trace("TC-2090", "FR-322-AC-490", "FR-038-AC-581", "FR-038-AC-597")]
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
            "BL-04",
            "BL-05",
            "BL-06",
            "BL-R1",
            "BL-R2",
            "BL-R3",
            "BL-R4",
            "BL-R5",
            "BL-R6",
            "BL-R7-no-occurrence-1",
            "BL-R8-bl03-without-occurrence-1-row",
            "BL-R9-up-beyond-previous-path",
            "BL-R10-up-not-least",
            "BL-R11-composite-up-not-least"
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

/// BL-S5 and BL-S6 (FR-322-AC-510): a row location in the earlier `{origin,
/// path}` form, and one with `up` and no `down`, are `malformed_wire`; so is
/// one with `down` and no `up` and an `up` past the safe integer range.
#[test]
#[trace("TC-491", "FR-038-AC-597", "FR-322-AC-510")]
fn the_earlier_whole_path_form_and_a_partial_relative_form_are_malformed_wire() {
    let cases = vectors();
    let mut seen = 0;
    let mut values: Vec<(String, Value)> = Vec::new();
    for case in cases["schema_invalid"].as_array().unwrap() {
        if case["target"] == "BodyLocation" {
            values.push((
                case["id"].as_str().unwrap().to_owned(),
                case["value"].clone(),
            ));
        }
    }
    let origin = values[0].1["origin"].clone();
    values.push((
        "down-without-up".to_owned(),
        serde_json::json!({"origin": origin, "down": []}),
    ));
    values.push((
        "up-beyond-safe".to_owned(),
        serde_json::json!({"origin": origin, "up": 9_007_199_254_740_992_u64, "down": []}),
    ));
    values.push((
        "path-beside-relative".to_owned(),
        serde_json::json!({"origin": origin, "up": 0, "down": [], "path": []}),
    ));
    for (id, value) in values {
        let mut package = package_with(&cases, &cases["positive"][0]);
        package["checked_term_data"]["body_locations"][0]["location"] = value;
        refresh_identity(&mut package);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
            panic!("{id} must refuse");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::MalformedWire,
            "{id}"
        );
        assert_eq!(
            refusal.loci[0].path.as_ref().unwrap().as_str(),
            "/checked_term_data/body_locations/0/location",
            "{id}: malformed relative locations refuse at the whole location"
        );
        seen += 1;
    }
    assert_eq!(seen, 5);
}
