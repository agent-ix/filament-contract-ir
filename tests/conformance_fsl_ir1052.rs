// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Fresh FSL producer readback for FR-058 and FR-067. This is an explicit
//! external lane: build FSL's `emit_checked_v2` example from its checkout,
//! then set `FILAMENT_SPEC_LANGUAGE_DIR` and `FSL_EMIT_CHECKED_V2_BIN`.

use filament_contract_model::{
    ApplicationOperator, CheckedNodeId, CheckedOccurrenceRole, CheckedPackageEvidence,
    CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult, CheckedTerm,
    ExecutionPoint, ExecutionPointName,
};
use ix_trace_rs::trace;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_SCRATCH: AtomicUsize = AtomicUsize::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "ir-producer-readback-{}-{}",
            std::process::id(),
            NEXT_SCRATCH.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).expect("create producer scratch directory");
        Self(path)
    }

    fn write(&self, name: &str, bytes: impl AsRef<[u8]>) -> PathBuf {
        let path = self.0.join(name);
        std::fs::write(&path, bytes).expect("write generated FSL input");
        path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.0) {
            eprintln!("could not remove {}: {error}", self.0.display());
        }
    }
}

fn producer() -> (PathBuf, PathBuf) {
    let fsl = PathBuf::from(std::env::var_os("FILAMENT_SPEC_LANGUAGE_DIR").expect("FSL checkout"));
    let binary = PathBuf::from(std::env::var_os("FSL_EMIT_CHECKED_V2_BIN").expect("FSL binary"));
    assert!(fsl
        .join("examples/switch-transition/switch.native")
        .is_file());
    assert!(binary.is_file());
    (fsl, binary)
}

fn emit(fsl: &Path, binary: &Path, source: &Path, model: &Path, points: Option<&Path>) -> Vec<u8> {
    let mut command = Command::new(binary);
    command
        .current_dir(fsl)
        .arg("--model")
        .arg(model)
        .arg("--package-name")
        .arg("ir-1013-readback");
    if let Some(points) = points {
        command.arg("--execution-points").arg(points);
    }
    let output = command.arg(source).output().expect("run FSL emitter");
    assert!(
        output.status.success(),
        "FSL emitter refused: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!output.stdout.is_empty(), "FSL emitted no package bytes");
    output.stdout
}

fn admit(bytes: &[u8], model: &[u8]) -> CheckedPackageV2 {
    let wire: Value = serde_json::from_slice(bytes).expect("FSL checked-package JSON");
    let identity = serde_json::from_slice::<Value>(model).expect("model JSON")["package"]
        ["identity"]
        .as_str()
        .expect("model package identity")
        .to_owned();
    let mut evidence = CheckedPackageEvidence::new();
    evidence
        .insert_domain_package_document(identity, model.to_vec())
        .expect("selected model evidence");
    for feature in wire["lock"]["required_features"]
        .as_array()
        .expect("required features")
    {
        evidence.support_feature(feature.as_str().expect("feature name"));
    }
    match CheckedPackageV2::read(bytes, CheckedPackageReadLimits::bounded(), &evidence) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("fresh FSL package must admit: {other:?}"),
    }
}

fn target(term: &CheckedTerm) -> &CheckedNodeId {
    let CheckedTerm::Reference { target } = term else {
        panic!("expected a typed node reference: {term:?}");
    };
    target
}

fn node<'a>(
    package: &'a CheckedPackageV2,
    id: &CheckedNodeId,
) -> &'a filament_contract_model::CheckedSemanticNodeV2 {
    // A reference names its target in the form the reader checks (operands
    // as node indexes), which only the resolved graph spells the same way.
    package
        .resolved_graph()
        .nodes
        .iter()
        .find(|node| &node.node_id == id)
        .expect("admitted reference target")
}

/// A fresh producer result supplies all four point variants on distinct
/// accepted claims. Source bytes are dropped before the public accessor runs.
#[trace("TC-462", "FR-058-AC-1", "FR-058-AC-6")]
#[test]
fn fsl_four_execution_points_read_back_through_public_accessor() {
    let (fsl, binary) = producer();
    assert!(fsl
        .join("examples/switch-transition/adjust.native")
        .is_file());
    let scratch = Scratch::new();
    let original = std::fs::read_to_string(fsl.join("examples/switch-transition/adjust.native"))
        .expect("FSL adjust source");
    let original_model =
        std::fs::read(fsl.join("examples/switch-transition/switch.semantic-ir.json"))
            .expect("FSL Switch model");
    let source = format!(
        "{original}\ninvariant Init using v on M::Switch at current {{ true }}\n\
         constraint Handle using v on M::Switch at current {{ true }}\n"
    );
    let source_path = scratch.write("four-points.native", source);
    let points = scratch.write(
        "points.json",
        serde_json::to_vec(&json!({
            "Init": {"kind": "initialization", "name": "startup"},
            "Handle": {"kind": "handler", "name": "tick"},
            "P1": {"kind": "pre", "operation": "adjust"},
            "Q2": {"kind": "post", "operation": "adjust"},
        }))
        .expect("points JSON"),
    );
    let source_arg = source_path
        .strip_prefix(&scratch.0)
        .expect("scratch-relative source");
    let points_arg = points
        .strip_prefix(&scratch.0)
        .expect("scratch-relative points");
    let model_arg = fsl.join("examples/switch-transition/switch.semantic-ir.json");
    let bytes = emit(
        &scratch.0,
        &binary,
        source_arg,
        &model_arg,
        Some(points_arg),
    );
    let package = admit(&bytes, &original_model);
    drop(bytes);

    let expected = [
        ExecutionPoint::Initialization {
            name: ExecutionPointName::new("startup").expect("point name"),
        },
        ExecutionPoint::Handler {
            name: ExecutionPointName::new("tick").expect("point name"),
        },
        ExecutionPoint::Pre {
            operation: ExecutionPointName::new("adjust").expect("operation name"),
        },
        ExecutionPoint::Post {
            operation: ExecutionPointName::new("adjust").expect("operation name"),
        },
    ];
    let mut observed = Vec::new();
    for clause in package.graph().nodes.iter().filter(|node| {
        node.node_tag.as_ref() == "state" && node.semantic_form.as_ref() == "state_clause"
    }) {
        let claim = clause
            .occurrences
            .iter()
            .find(|claim| claim.role == CheckedOccurrenceRole::Claim)
            .expect("producer clause claim");
        observed.push(
            package
                .clause_execution_point(&clause.node_id, (&clause.node_id, claim))
                .expect("producer point through public accessor"),
        );
    }
    assert_eq!(
        observed.len(),
        expected.len(),
        "one claim per source clause"
    );
    for point in expected {
        assert_eq!(
            observed.iter().filter(|actual| **actual == point).count(),
            1,
            "each point kind and exact name occurs once: {point:?}"
        );
    }
}

/// FSL's Switch source contributes an actual transition claim. Its public
/// typed application exposes Boolean subject, ordered inputs and result.
#[trace("TC-064", "FR-067-AC-7")]
#[ix_trace_rs::string_edge]
#[test]
fn fsl_boolean_transition_readback_retains_typed_subject_inputs_and_result() {
    let (fsl, binary) = producer();
    let source = Path::new("examples/switch-transition/switch.native");
    let model = Path::new("examples/switch-transition/switch.semantic-ir.json");
    let model_bytes = std::fs::read(fsl.join(model)).expect("FSL Switch model");
    let bytes = emit(&fsl, &binary, source, model, None);
    let package = admit(&bytes, &model_bytes);
    let transitions: Vec<_> = package
        .resolved_graph()
        .nodes
        .iter()
        .filter(|node| {
            node.node_tag.as_ref() == "state" && node.semantic_form.as_ref() == "transition"
        })
        .collect();
    let [transition] = transitions.as_slice() else {
        panic!(
            "FSL must emit one Switch transition, got {}",
            transitions.len()
        );
    };
    assert!(
        transition
            .occurrences
            .iter()
            .any(|occurrence| { occurrence.role == CheckedOccurrenceRole::Claim }),
        "transition has an authentic claim"
    );
    let CheckedTerm::Application {
        operator: ApplicationOperator::StateTransition,
        arguments,
        result_type,
        ..
    } = &transition.body
    else {
        panic!("FSL transition is a typed state-transition application");
    };
    assert_eq!(&transition.semantic_type, result_type);
    let boolean = node(&package, result_type);
    assert_eq!(boolean.semantic_form.as_ref(), "boolean");
    let bindings: Vec<_> = arguments
        .iter()
        .map(|argument| match argument {
            CheckedTerm::Binding { name, value, .. } => (name.as_ref(), &**value),
            other => panic!("transition argument is a typed binding: {other:?}"),
        })
        .collect();
    assert_eq!(
        bindings.iter().map(|(name, _)| *name).collect::<Vec<_>>(),
        ["anchor", "subject", "inputs", "pre", "post"]
    );
    let subject = node(&package, target(bindings[1].1));
    assert_eq!(
        subject.semantic_type, *result_type,
        "actual subject is Boolean"
    );
    let CheckedTerm::Application { operation, .. } = &subject.body else {
        panic!("transition subject is a typed field projection");
    };
    let (_, field_name) = operation
        .projection_field()
        .expect("admitted operation view")
        .expect("subject field projection");
    assert_eq!(field_name, "on", "Switch transition subject");
    let CheckedTerm::Aggregate { members, .. } = bindings[2].1 else {
        panic!("ordered transition inputs are a typed aggregate");
    };
    let [input] = members.as_slice() else {
        panic!("Switch.set has one input, got {}", members.len());
    };
    let input = node(&package, target(input));
    assert_eq!(input.semantic_form.as_ref(), "parameter");
    assert_eq!(input.semantic_type, *result_type, "input flag is Boolean");
    let post = node(&package, target(bindings[4].1));
    assert_eq!(post.semantic_form.as_ref(), "state_clause");
    assert_eq!(post.semantic_type, *result_type, "post result is Boolean");
    let CheckedTerm::Application {
        operator: ApplicationOperator::StateClause,
        arguments: post_arguments,
        ..
    } = &post.body
    else {
        panic!("post is a typed state clause application");
    };
    let [_, _, post_result] = post_arguments.as_slice() else {
        panic!("post clause has parameters, anchor, and result");
    };
    let result = node(&package, target(post_result));
    assert_eq!(
        result.semantic_type, *result_type,
        "post expression is Boolean"
    );
    let CheckedTerm::Application { operation, .. } = &result.body else {
        panic!("post result is a typed field projection");
    };
    let (_, field_name) = operation
        .projection_field()
        .expect("admitted post result operation")
        .expect("post result field projection");
    assert_eq!(field_name, "on", "Switch post result field");
}
