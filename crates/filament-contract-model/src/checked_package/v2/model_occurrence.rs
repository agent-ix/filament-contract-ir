// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Join stage 5: the occurrence of every model declaration node in
//! `source_map` (FSpec FR-322 "Model declaration occurrences",
//! FR-322-AC-486; Contract IR FR-038).
//!
//! A model declaration node (a `model` or `relation` node carrying a
//! `ModelOwner`) has exactly one `source_map` entry: role `generated`, ordinal
//! 0, with exactly the regions of its identity's `model_root` entry.

use super::native_join_budget::PackageReadStage;
use super::stage_storage::{self, Path, Refusal, Site};
use super::{CheckedNodeKind, CheckedNodeOwner, GraphIndex, RawSemanticNodeV2, WorkMeter};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedOccurrenceRole, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    CheckedSourceMapEntry, CheckedSourceRegion,
};

/// Whether the node at `position` is a model declaration node.
fn is_declaration(node: &RawSemanticNodeV2, kind: Option<&CheckedNodeKind>) -> bool {
    matches!(
        kind,
        Some(CheckedNodeKind::Model(_) | CheckedNodeKind::Relation(_))
    ) && matches!(node.owner, Some(CheckedNodeOwner::Model { .. }))
}

/// Refuses the first model declaration node whose `source_map` entry is not
/// the one occurrence the specification fixes, else the first with none.
pub(super) fn validate(
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    source_map: &[CheckedSourceMapEntry],
    index: &GraphIndex<'_>,
    meter: &mut WorkMeter<'_>,
) -> Result<(), ValidationFailure> {
    let root = Path::root().key("source_map");
    let site = |at: Path<'static>| Site::new(PackageReadStage::Graph, 0, "model_occurrence", at);
    let declarations = nodes
        .iter()
        .enumerate()
        .filter(|(position, node)| is_declaration(node, kinds.get(*position)))
        .count();
    if declarations == 0 {
        return Ok(());
    }
    let model_roots = kinds
        .iter()
        .filter(|kind| **kind == CheckedNodeKind::Model(super::ModelForm::ModelRoot))
        .count();
    // The `generated` ordinal-zero entry of each model root, by identity.
    let mut roots: Vec<(&str, &[CheckedSourceRegion])> =
        stage_storage::vec(meter, &site(root), model_roots)?;
    for entry in source_map {
        if roots.len() == model_roots {
            break;
        }
        if entry.role != CheckedOccurrenceRole::Generated || entry.ordinal != 0 {
            continue;
        }
        let Some(&position) = index.get(&entry.node_id) else {
            continue;
        };
        if let (
            Some(CheckedNodeKind::Model(super::ModelForm::ModelRoot)),
            Some(CheckedNodeOwner::ModelRoot { identity }),
        ) = (kinds.get(position), nodes[position].owner.as_ref())
        {
            stage_storage::work(meter, &site(root), 1)?;
            roots.push((identity.as_ref(), entry.regions.as_slice()));
        }
    }
    // Stable, so the first entry of an identity stays the first found.
    roots.sort_by(|left, right| left.0.cmp(right.0));
    let mut seen = meter.filled(nodes.len(), false)?;
    let invalid = CheckedPackageRefusalCause::InvalidValue;
    let refuse = |meter: &mut WorkMeter<'_>, at: Path<'_>, id| {
        stage_storage::refuse(
            meter,
            &Site::new(PackageReadStage::Graph, 0, "model_occurrence", at),
            Refusal::new(CheckedPackageRefusalCode::InvalidPackage, at)
                .because(invalid)
                .at_node(id),
        )
    };
    for (entry_index, entry) in source_map.iter().enumerate() {
        let Some(&position) = index.get(&entry.node_id) else {
            continue;
        };
        let node = &nodes[position];
        if !is_declaration(node, kinds.get(position)) {
            continue;
        }
        let Some(CheckedNodeOwner::Model { identity, .. }) = node.owner.as_ref() else {
            continue;
        };
        stage_storage::work(meter, &site(root.index(entry_index)), 1)?;
        let start = roots.partition_point(|(held, _)| *held < identity.as_ref());
        let selecting = roots
            .get(start)
            .filter(|(held, _)| *held == identity.as_ref())
            .map(|(_, regions)| *regions);
        let repeated = seen[position];
        seen[position] = true;
        if entry.role != CheckedOccurrenceRole::Generated
            || entry.ordinal != 0
            || repeated
            || selecting != Some(entry.regions.as_slice())
        {
            return Err(refuse(meter, root.index(entry_index), &entry.node_id));
        }
    }
    for (position, node) in nodes.iter().enumerate() {
        if !is_declaration(node, kinds.get(position)) {
            continue;
        }
        stage_storage::work(meter, &site(root), 1)?;
        if !seen[position] {
            return Err(refuse(meter, root, &node.node_id));
        }
    }
    Ok(())
}
