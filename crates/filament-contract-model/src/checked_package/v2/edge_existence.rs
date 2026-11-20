// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Join stage 4: the existence of the `semantic_type` and `dependencies`
//! edges of a node (FSpec FR-322 "One total order of the join stages",
//! FR-322-AC-275; Contract IR FR-038).
//!
//! A node of tag `scalar_type`, `composite_type`, `bounded_domain`, `value`,
//! `expression`, `function` or `model` whose `semantic_type` or `dependencies`
//! entry names no node refuses `invalid_semantic_graph`, with no cause, at the
//! edge member. The stage runs over every node, in graph order and the
//! `semantic_type` before the `dependencies` of each, after the owner join and
//! before the model declaration occurrence check. A node of any other tag is
//! outside the stage: its own artifact defines the refusal of a dangling edge.

use super::native_join_budget::PackageReadStage;
use super::stage_storage::{self, Path, Refusal, Site};
use super::{
    CheckedNodeKind, CheckedNodeTag, GraphIndex, RawSemanticNodeV2, ScalarTypeForm, WorkMeter,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::CheckedPackageRefusalCode;

/// Whether stage 4 reads the edges of a node of `kind`.
fn in_stage(kind: CheckedNodeKind) -> bool {
    matches!(
        kind.tag(),
        CheckedNodeTag::ScalarType
            | CheckedNodeTag::CompositeType
            | CheckedNodeTag::BoundedDomain
            | CheckedNodeTag::Value
            | CheckedNodeTag::Expression
            | CheckedNodeTag::Function
            | CheckedNodeTag::Model
    )
}

/// Whether the `dependencies` of a node of `kind` are read by the nominal
/// join, not here: a dimension, unit or dimension equivalence may depend on a
/// nominal node of a selected dependency package, and its absent operand is
/// refused at the operand of its body.
fn nominal_dependencies(kind: CheckedNodeKind) -> bool {
    matches!(
        kind,
        CheckedNodeKind::ScalarType(
            ScalarTypeForm::Dimension | ScalarTypeForm::Unit | ScalarTypeForm::DimensionEquivalence
        )
    )
}

/// Refuses the first edge of a stage 4 node that names no node.
pub(super) fn validate(
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &GraphIndex<'_>,
    meter: &mut WorkMeter<'_>,
) -> Result<(), ValidationFailure> {
    let root = Path::root().key("semantic_graph").key("nodes");
    let site = |position: usize, at: Path<'static>| {
        Site::new(PackageReadStage::Graph, position, "edge_existence", at)
    };
    let code = CheckedPackageRefusalCode::InvalidSemanticGraph;
    for (position, node) in nodes.iter().enumerate() {
        let Some(&kind) = kinds.get(position).filter(|kind| in_stage(**kind)) else {
            continue;
        };
        let at = root.index(position);
        stage_storage::work(
            meter,
            &site(position, at),
            1 + u64::try_from(node.dependencies.len()).unwrap_or(u64::MAX),
        )?;
        if !index.contains_key(&node.semantic_type) {
            let edge = at.key("semantic_type");
            return Err(stage_storage::refuse(
                meter,
                &site(position, edge),
                Refusal::new(code, edge),
            ));
        }
        if nominal_dependencies(kind) {
            continue;
        }
        for (edge_index, dependency) in node.dependencies.iter().enumerate() {
            if index.contains_key(dependency) {
                continue;
            }
            let edge = at.key("dependencies").index(edge_index);
            return Err(stage_storage::refuse(
                meter,
                &site(position, edge),
                Refusal::new(code, edge),
            ));
        }
    }
    Ok(())
}
