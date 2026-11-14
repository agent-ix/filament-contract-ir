// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Fresh U12 square emission through the public FSL and strict IR readers.

use fsl_foundation::SourceIdentity;
use fsl_package::{emit_checked, CheckedPackage};
use fsl_semantics::{check::PackageDeclarations, model::intake::SelectedModels};
use filament_semantic_value::checking::CheckingLimits;
use std::io::Write;

fn main() {
    let source = "language \"ix:native\" edition \"1-draft\";\n\
        profile v = \"filament.value.complete/v1\";\n\
        dimension Distance;\n\
        unit m : Distance = rational(1, 1);\n\
        function square using v(): Boolean pure { 2 m ^ 2 = 2 m ^ 2 }\n";
    let parsed = fsl_cst::parse(
        SourceIdentity::new("agent-ix", "test:literal-body-admission"),
        "square.native",
        source.as_bytes(),
        fsl_cst::Limits::default(),
    )
    .expect("public source parser");
    assert_eq!(parsed.diagnostics(), []);
    let forms = fsl_forms::build_unit(&parsed).expect("public forms builder");
    let checked = CheckedPackage::link(
        PackageDeclarations::assemble(
            parsed.source().reference().clone(),
            "agent-ix/literal-body-admission".to_owned(),
            forms,
            SelectedModels::default(),
            Vec::new(),
        )
        .expect("public assembly")
        .check(CheckingLimits::default())
        .expect("public checking"),
    );
    let emission = emit_checked(&checked).expect("public checked V2 emission");
    assert_eq!(emission.omitted(), []);
    std::io::stdout()
        .write_all(emission.package().bytes())
        .expect("original canonical emission");
}
