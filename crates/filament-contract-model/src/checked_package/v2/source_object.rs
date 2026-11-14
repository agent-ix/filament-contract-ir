// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Source-declared object types (FR-362 through FR-368).

use super::{CheckedNodeKind, CheckedNodeOwner, CheckedSemanticNodeV2, ModelForm};

/// Whether `node` of `kind` is a source-declared object type: a
/// `model`/`object_type` node that carries a `declaration` and a `SourceOwner`
/// (FR-362). Only such a node is subject to the source-object rules.
pub(super) fn is_source_declared_object_type<Body>(
    node: &CheckedSemanticNodeV2<Body>,
    kind: CheckedNodeKind,
) -> bool {
    kind == CheckedNodeKind::Model(ModelForm::ObjectType)
        && node.declaration.is_some()
        && matches!(node.owner, Some(CheckedNodeOwner::Source { .. }))
}
