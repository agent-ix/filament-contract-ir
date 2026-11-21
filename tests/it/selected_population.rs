// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

use crate::support::checked_package::refresh_identity_projection;
use filament_contract_model::{
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use ix_trace_rs::trace;
use serde_json::Value;
use std::{fs, path::PathBuf, process::Command};

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn fsl_fixture() -> Scratch {
    let fsl = std::env::var_os("FILAMENT_SPEC_LANGUAGE_DIR")
        .map(PathBuf::from)
        .expect("selected-population test requires FILAMENT_SPEC_LANGUAGE_DIR");
    let scratch_path =
        std::env::temp_dir().join(format!("ir760-duplicate-max-{}", std::process::id()));
    let _ = fs::remove_dir_all(&scratch_path);
    fs::create_dir(&scratch_path).expect("create FSL producer scratch");
    let output = scratch_path.join("emitted");
    let target = std::env::var_os("FILAMENT_SPEC_LANGUAGE_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/fsl-producer"));
    let result = Command::new("cargo")
        .arg("run")
        .arg("--locked")
        .arg("--manifest-path")
        .arg(fsl.join("Cargo.toml"))
        .arg("--example")
        .arg("population_v2_fixtures")
        .arg("--features")
        .arg("population-fixtures")
        .arg("--")
        .arg(&output)
        .env("CARGO_TARGET_DIR", target)
        .output()
        .expect("run authoritative FSL producer");
    assert!(
        result.status.success(),
        "FSL producer failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Scratch(scratch_path)
}

fn read_rekeyed(package: &Value, domain: &Value) -> CheckedPackageV2ReadResult {
    let domain_bytes = quire_canonical::to_vec(domain, quire_canonical::Limits::new(1 << 30))
        .expect("FSL domain package is canonicalizable");
    let mut evidence = CheckedPackageEvidence::new();
    for root in package["semantic_graph"]["nodes"]
        .as_array()
        .expect("FSL graph nodes")
        .iter()
        .filter(|node| node["semantic_form"] == "model_root")
    {
        let identity = root["owner"]["identity"].as_str().expect("model identity");
        evidence
            .insert_domain_package_document(identity, domain_bytes.clone())
            .expect("selected domain package evidence");
    }
    evidence.support_feature("filament.value.complete/v1");
    let bytes = quire_canonical::to_vec(package, quire_canonical::Limits::new(1 << 30))
        .expect("FSL package is canonicalizable");
    CheckedPackageV2::read(&bytes, CheckedPackageReadLimits::bounded(), &evidence)
}

#[trace("FR-053-AC-3", "TC-453")]
#[test]
fn duplicate_max_population_body_is_refused_by_the_reader() {
    let scratch = fsl_fixture();
    let domain: Value = serde_json::from_slice(
        &fs::read(scratch.0.join("emitted/spine-model.semantic-ir.json"))
            .expect("FSL domain package output"),
    )
    .expect("FSL domain package JSON");
    let mut package: Value = serde_json::from_slice(
        &fs::read(scratch.0.join("emitted/population-2-5.json"))
            .expect("FSL emitted population package"),
    )
    .expect("FSL package JSON");
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("semantic graph nodes");
    let position = nodes
        .iter()
        .position(|node| {
            node["node_tag"] == "bounded_domain" && node["semantic_form"] == "model_population"
        })
        .expect("FSL emitted population node");
    let population = &mut nodes[position];
    let maximum = population["body"]["members"]
        .as_array()
        .expect("population members")
        .iter()
        .find(|member| member["name"] == "max")
        .expect("original max binding")
        .clone();
    population["body"]["members"]
        .as_array_mut()
        .expect("population members")
        .push(maximum);

    refresh_identity_projection(&mut package);
    // The FR-322 derived-shape check (a population body is `max` alone, "member
    // beside max", FR-322-AC-74; structural.rs derived-shape arm) refuses a body
    // binding `max` twice at the node id, so the strict reader never admits it
    // and the projection has no admitted-but-not-population case to select.
    let CheckedPackageV2ReadResult::Refused(refusal) = read_rekeyed(&package, &domain) else {
        panic!("a population body binding max twice must be refused by the reader")
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::InvalidDerivedShape)
    );
    assert_eq!(
        refusal.loci[0]
            .path
            .as_ref()
            .expect("refusal path")
            .as_str(),
        format!("/semantic_graph/nodes/{position}/node_id"),
        "{refusal:?}"
    );
}
