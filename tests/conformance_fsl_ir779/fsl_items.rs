// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

use std::{env, fs};

use fsl_foundation::{digest::WireNodeId, source::SourceIdentity};
use fsl_route::{request::items_from_requirements, Registry};
use fsl_semantics::check::PackageDeclarations;
use fsl_semantics::model::intake::SelectedModels;
use filament_semantic_value::checking::CheckingLimits;
use serde_json::{json, Value};

fn id(node: WireNodeId) -> String {
    node.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect()
}

fn main() {
    let source_path = env::args().nth(1).expect("source path");
    let source = fs::read(&source_path).expect("read source");
    let identity = SourceIdentity::new(
        "agent-ix",
        "test:ir779-rr13",
        "fixture",
        "fixture:1",
    );
    let parsed = fsl_cst::parse(
        identity,
        "ir779-rr13.native",
        &source,
        fsl_cst::Limits::default(),
    )
    .expect("FSL parses FR-062 RR-13 source");
    assert!(parsed.is_admissible(), "{:?}", parsed.diagnostics());
    let unit = fsl_forms::build_unit(&parsed).expect("FSL builds unit");
    let checked = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        unit,
        SelectedModels::default(),
        Vec::new(),
    )
    .expect("FSL assembles RR-13 source")
    .check(CheckingLimits::default())
    .expect("FSL checks RR-13 source");
    let registry = Registry::new();
    let items = items_from_requirements(checked.requirements(), &registry, None);
    let items = items
        .iter()
        .map(|item| {
            let origin = item.occurrence().origin();
            json!({
                "application": {
                    "node": id(item.occurrence().node()),
                    "role": origin.role().as_str(),
                    "ordinal": origin.ordinal(),
                },
                "guards": item.path_condition().iter().map(|guard| {
                    let origin = guard.occurrence().origin();
                    json!({
                        "occurrence": {
                            "node": id(guard.occurrence().node()),
                            "role": origin.role().as_str(),
                            "ordinal": origin.ordinal(),
                        },
                        "holds": guard.holds(),
                    })
                }).collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<Value>>();
    println!("{}", serde_json::to_string(&items).expect("serialize FSL items"));
}
