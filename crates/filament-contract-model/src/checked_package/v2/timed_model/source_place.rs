// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The strict-read half of the source timed-model clock-place join (contract-ir
//! FR-374, FSpec FR-789 "What the strict read checks"): the facts of a
//! `resolved_path` row that the package alone shows. A clock member, a tuple
//! position and a union member are left to the identity-row join.

use super::super::source_rendering::{self as rendering, TypeGraph};
use super::{decimal_is, text_member, typed_members, work, Graph, PathStepKind, Word};
use super::{CheckedNodeKind, CheckedNodeOwner, CheckedTerm};
use super::{ValidationFailure, WorkMeter};

impl TypeGraph for Graph<'_> {
    fn kind(&self, position: usize) -> Option<CheckedNodeKind> {
        self.kinds.get(position).copied()
    }
    fn owner(&self, position: usize) -> Option<&CheckedNodeOwner> {
        self.nodes.get(position)?.owner.as_ref()
    }
    fn declaration(&self, position: usize) -> Option<&super::super::CheckedDeclaration> {
        self.nodes.get(position)?.declaration.as_ref()
    }
    fn dependencies(&self, position: usize) -> &[super::CheckedNodeId] {
        self.nodes
            .get(position)
            .map_or(&[][..], |node| &node.dependencies)
    }
    fn body(&self, position: usize) -> Option<&CheckedTerm> {
        self.terms.get(position).and_then(Option::as_ref)
    }
    fn locate(&self, id: &super::CheckedNodeId) -> Option<usize> {
        self.index.get(id).copied()
    }
    fn size(&self) -> usize {
        self.nodes.len()
    }
}

/// The identity of the `SourceOwner` of a source-declared object type root,
/// `None` for a selected model's root.
pub(super) fn source_root_identity(node: &super::CheckedSemanticNodeV2) -> Option<&str> {
    match (&node.owner, &node.declaration) {
        (Some(CheckedNodeOwner::Source { identity, .. }), Some(_)) => Some(identity),
        _ => None,
    }
}

/// The root's source declaration key spelling, `ix://I/<type>`.
pub(super) fn root_rendering(
    node: &super::CheckedSemanticNodeV2,
    identity: &str,
    meter: &mut WorkMeter,
) -> Result<Option<Word<'static>>, ValidationFailure> {
    let Some(declaration) = node.declaration.as_ref() else {
        return Ok(None);
    };
    let [name] = declaration.qualified_name.as_slice() else {
        return Ok(None);
    };
    Ok(Some(Word::Owned(
        meter.formatted(format_args!("ix://{identity}/{name}"))?,
    )))
}

/// Whether `text` is `prefix` followed by the decimal spelling of `number`.
pub(super) fn is_prefixed_decimal(text: &str, prefix: &[&str], number: u64) -> bool {
    let mut rest = text;
    for part in prefix {
        match rest.strip_prefix(part) {
            Some(after) => rest = after,
            None => return false,
        }
    }
    decimal_is(rest, number)
}

/// What the strict read checks of one `field`, `tuple` or `union` row that
/// has a declaring node: `Ok(true)` when the row passes.
///
/// `declaring` is the row's `declaring_type`; `after_union` is true when the
/// previous row was a `union` row, whose product rendering a `tuple` row may
/// then name. `next_slot` is the `slot` step that follows, with its row's
/// collection kind, minimum and maximum.
#[allow(clippy::too_many_arguments)]
pub(super) fn check_row(
    graph: &Graph<'_>,
    identity: &str,
    kind: PathStepKind,
    step: &CheckedTerm,
    declaring: &str,
    member_type: &str,
    after_union: bool,
    slot: Option<(&str, &str, &str)>,
    meter: &mut WorkMeter,
) -> Result<bool, ValidationFailure> {
    let product = after_union && kind == PathStepKind::Tuple && declaring.starts_with('(');
    let node = if product {
        None
    } else {
        work(
            meter,
            super::Path::root(),
            u64::try_from(graph.nodes.len()).unwrap_or(u64::MAX),
        )?;
        let step_kind = match kind {
            PathStepKind::Field => rendering::DeclaringStep::Field,
            PathStepKind::Tuple => rendering::DeclaringStep::Tuple,
            _ => rendering::DeclaringStep::Union,
        };
        match rendering::declaration_node(graph, identity, declaring, step_kind) {
            Some(node) => Some(node),
            None => return Ok(false),
        }
    };
    if !rendering::well_formed(member_type) {
        return Ok(false);
    }
    let Some(node) = node else {
        // A product's position is joined, with the slot bounds that follow,
        // at the identity-row join.
        return Ok(true);
    };
    if kind != PathStepKind::Field {
        return Ok(true);
    }
    let name = text_member(step, 1).unwrap_or_default();
    let Some((optional, target)) = rendering::binding_of(graph, node, name) else {
        // A clock member has no binding: joined at the identity-row join.
        return Ok(true);
    };
    let Some(target) = graph.index.get(target).copied() else {
        return Ok(false);
    };
    let mut tick = || work(meter, super::Path::root(), 1);
    match rendering::render(graph, target, &mut tick)? {
        rendering::Rendered::Known(rendered) => {
            let expected = if optional {
                meter.formatted(format_args!("Option<{rendered}>"))?
            } else {
                rendered.into_boxed_str()
            };
            if expected.as_ref() != member_type {
                return Ok(false);
            }
        }
        // A range, a text bound or a quantity can never hold a Clock, so a
        // clock place cannot pass through a field of that type, whatever
        // spelling FSpec FR-789 later gives it.
        rendering::Rendered::Interim | rendering::Rendered::Unknown => return Ok(false),
    }
    let Some((collection, minimum, maximum)) = slot else {
        return Ok(true);
    };
    let Some((kind, lower, upper)) = rendering::collection_layer(graph, target, 0) else {
        return Ok(false);
    };
    Ok(kind == collection && lower == minimum && upper == maximum)
}

/// Whether `name` is a field of the root node's body with no binding: a clock
/// member of the source object type (FSpec FR-789 "Clock parameters and reset
/// frames").
pub(super) fn names_clock_field(graph: &Graph<'_>, root: usize, name: &str) -> bool {
    graph
        .terms
        .get(root)
        .and_then(Option::as_ref)
        .is_some_and(|body| {
            !typed_members(body).iter().any(|member| {
                matches!(member, CheckedTerm::Binding { name: bound_name, .. }
                    if bound_name.as_ref() == name)
            })
        })
}
