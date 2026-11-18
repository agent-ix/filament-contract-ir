// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-322 caller-selected dependency clock lock rows.

use crate::support::checked_package;
use crate::support::checked_package::{canonical, evidence_for, pointer};
use filament_contract_model::{
    CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, CheckedPackageV2ReadResult,
};
use ix_trace_rs::trace;
use serde_json::{json, Value};

#[path = "support/dependency_selection_inputs.rs"]
mod dependency_selection_inputs;

fn base() -> Value {
    dependency_selection_inputs::dependency_selection_package("test/clock-importer")
}

fn clock(name: &str, unit: &str) -> Value {
    json!({"clock_name": name, "clock": {"kind": "timed", "unit": unit}})
}

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    crate::read_and_account(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

fn refused(package: &Value) -> CheckedPackageRefusal {
    match read(package) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected refusal, got {other:?}"),
    }
}

fn expect_refusal(package: &Value, code: Code, path: &str, cause: Option<Cause>) {
    let refusal = refused(package);
    assert_eq!(refusal.code, code, "{refusal:?}");
    assert_eq!(refusal.cause, cause, "{refusal:?}");
    assert!(
        refusal
            .loci
            .iter()
            .any(|locus| locus.path == Some(pointer(path))),
        "{refusal:?}"
    );
}

#[trace("TC-048", "FR-322-AC-452")]
#[test]
fn absent_and_nonempty_sorted_clock_selections_admit_and_round_trip() {
    let absent = base();
    assert!(matches!(
        read(&absent),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    assert!(absent["lock"].get("dependency_clock_selections").is_none());

    let mut selected = base();
    let selections = json!([clock("caller-a", "s"), clock("caller-b", "ms")]);
    selected["lock"]["dependency_clock_selections"] = selections.clone();
    let CheckedPackageV2ReadResult::Admitted(admitted) = read(&selected) else {
        panic!("canonical nonempty dependency clock selections must admit");
    };
    assert_eq!(
        serde_json::to_value(admitted.lock().dependency_clock_selections.as_ref())
            .expect("retained clock selections"),
        json!(selections)
    );
    assert_eq!(
        serde_json::to_value(admitted.lock())
            .expect("lock wire")
            .get("dependency_clock_selections"),
        Some(&selections)
    );
}

#[trace("TC-048", "FR-322-AC-452")]
#[test]
fn dependency_clock_selection_shape_is_closed_and_nonempty() {
    let mut package = base();
    for (value, path, code) in [
        (
            json!([]),
            "/lock/dependency_clock_selections",
            Code::MalformedWire,
        ),
        (
            Value::Null,
            "/lock/dependency_clock_selections",
            Code::MalformedWire,
        ),
        (
            json!([{}]),
            "/lock/dependency_clock_selections/0",
            Code::MalformedWire,
        ),
        (
            json!([{"clock_name":"caller","clock":{"kind":"timed","unit":"ms","extra":true}}]),
            "/lock/dependency_clock_selections/0/clock/extra",
            Code::UnknownMember,
        ),
        (
            json!([{"clock_name":"caller","clock":{"kind":"timed","unit":""}}]),
            "/lock/dependency_clock_selections/0/clock/unit",
            Code::MalformedWire,
        ),
    ] {
        package["lock"]["dependency_clock_selections"] = value;
        let refusal = refused(&package);
        assert_eq!(refusal.code, code, "{refusal:?}");
        assert!(
            refusal
                .loci
                .iter()
                .any(|locus| locus.path == Some(pointer(path))),
            "{refusal:?}"
        );
    }
}

#[trace("TC-048", "FR-322-AC-452")]
#[test]
fn duplicate_misordered_and_reserved_clock_names_refuse() {
    let mut package = base();
    package["lock"]["dependency_clock_selections"] =
        json!([clock("caller-a", "s"), clock("caller-a", "ms")]);
    let duplicate = refused(&package);
    assert_eq!(duplicate.code, Code::InvalidPackage);
    assert_eq!(duplicate.cause, Some(Cause::ConflictingDefinition));
    assert!(duplicate.loci.iter().any(|locus| {
        locus.path == Some(pointer("/lock/dependency_clock_selections/1/clock_name"))
    }));

    package["lock"]["dependency_clock_selections"] =
        json!([clock("caller-b", "s"), clock("caller-a", "s")]);
    expect_refusal(
        &package,
        Code::InvalidPackage,
        "/lock/dependency_clock_selections/1/clock_name",
        Some(Cause::InvalidValue),
    );

    for reserved in ["model-steps", "model-time"] {
        package["lock"]["dependency_clock_selections"] = json!([clock(reserved, "s")]);
        expect_refusal(
            &package,
            Code::MalformedWire,
            "/lock/dependency_clock_selections/0/clock_name",
            None,
        );
    }

    package["lock"]["dependency_clock_selections"] = json!([
        clock("caller-\u{ff61}", "s"),
        clock("caller-\u{1f600}", "s")
    ]);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}
