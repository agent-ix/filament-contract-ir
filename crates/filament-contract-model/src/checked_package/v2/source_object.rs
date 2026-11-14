// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Source-declared object types (FR-362 through FR-368).

use super::{
    CheckedNodeId, CheckedNodeKind, CheckedNodeOwner, CheckedSemanticNodeV2, CheckedTerm,
    ModelForm, RelationForm,
};

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

/// Whether `node` of `kind` is a source-declared population reference: a
/// `relation`/`population` node that carries a `declaration` and a
/// `SourceOwner` (FR-364).
pub(super) fn is_source_population<Body>(
    node: &CheckedSemanticNodeV2<Body>,
    kind: CheckedNodeKind,
) -> bool {
    kind == CheckedNodeKind::Relation(RelationForm::Population)
        && node.declaration.is_some()
        && matches!(node.owner, Some(CheckedNodeOwner::Source { .. }))
}

/// The type node a record-style field binding names: the reference target of
/// `name -> reference` or of `name -> aggregate{optional -> reference}`.
pub(super) fn bound_type(value: &CheckedTerm) -> Option<&CheckedNodeId> {
    match value {
        CheckedTerm::Reference { target } => Some(target),
        CheckedTerm::Aggregate { members } => match members.as_slice() {
            [CheckedTerm::Binding { name, value }] if name.as_ref() == "optional" => {
                match value.as_ref() {
                    CheckedTerm::Reference { target } => Some(target),
                    _ => None,
                }
            }
            _ => None,
        },
        _ => None,
    }
}

/// One field of a source-declared object type in the producer's identity row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedSourceObjectMember {
    /// Field name.
    pub name: Box<str>,
    /// The type node the field's binding references.
    pub type_node: CheckedNodeId,
    /// Whether the binding carries the `optional` group.
    pub optional: bool,
}

/// One population a source-declared object type declares itself.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedSourcePopulation {
    /// Population name.
    pub name: Box<str>,
    /// Whether the population is symmetric.
    pub symmetric: bool,
}

/// One FSL-stage source-object identity row for an admitted source-declared
/// object type. The row is a producer fact; admission checks its join to the
/// package, and `identity_fields` and `populations` are read only from a
/// joined row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedSourceObjectRow {
    /// The admitted `model`/`object_type` node.
    pub structural_node: CheckedNodeId,
    /// Authority of the node's `SourceOwner`.
    pub source_authority: Box<str>,
    /// The type's qualified source name.
    pub source_identity: Box<str>,
    /// Whether the type is abstract.
    pub is_abstract: bool,
    /// The direct supertype nodes, in source order.
    pub supertypes: Vec<CheckedNodeId>,
    /// The effective identity field names.
    pub identity_fields: Vec<Box<str>>,
    /// One entry per field of the effective view, in body binding order.
    pub members: Vec<SelectedSourceObjectMember>,
    /// The populations the type itself declares, in source order.
    pub populations: Vec<SelectedSourcePopulation>,
}

/// Why a supply of source-object identity rows was refused, in the order the
/// refusals take precedence.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum SelectedSourceObjectErrorCode {
    /// A row's authority is the authority of no `SourceOwner` of the package.
    #[error("source-object row names an authority the package does not own")]
    ForeignIdentityWitness,
    /// A row names no source-declared object type, disagrees with its node, or
    /// carries members or supertypes the package does not support.
    #[error("source-object row does not match an admitted source-declared type")]
    UnknownIdentityWitness,
    /// Two rows carry one (authority, identity) key.
    #[error("competing source-object rows")]
    ConflictingIdentityWitness,
    /// A source-declared object type has no row.
    #[error("source-declared object type has no identity row")]
    MissingIdentityWitness,
}

/// A refused supply, at the `Package` locus, with no partial view.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("{code}")]
pub struct SelectedSourceObjectError {
    /// The first reason in precedence order.
    pub code: SelectedSourceObjectErrorCode,
}

/// The complete joined source-object rows of one admitted package instance.
#[derive(Clone, Debug)]
pub struct SelectedSourceObjects<'a> {
    package: &'a super::CheckedPackageV2,
    rows: std::collections::BTreeMap<usize, SelectedSourceObjectRow>,
}

impl<'a> SelectedSourceObjects<'a> {
    /// Classify every row, then join the complete supply to the package. Nothing
    /// is charged to a work or storage meter.
    pub fn from_stage_rows(
        package: &'a super::CheckedPackageV2,
        rows: impl IntoIterator<Item = SelectedSourceObjectRow>,
    ) -> Result<Self, SelectedSourceObjectError> {
        let rows: Vec<SelectedSourceObjectRow> = rows.into_iter().collect();
        let fail = |code| Err(SelectedSourceObjectError { code });
        let graph = package.resolved_graph();
        let is_type = |position: usize| {
            graph
                .nodes
                .get(position)
                .zip(package.kinds.get(position))
                .is_some_and(|(node, kind)| is_source_declared_object_type(node, *kind))
        };
        let owners_authority = |authority: &str| {
            graph.nodes.iter().any(|node| {
                matches!(&node.owner, Some(CheckedNodeOwner::Source { authority: held, .. })
                    if held.as_ref() == authority)
            })
        };
        if rows
            .iter()
            .any(|row| !owners_authority(&row.source_authority))
        {
            return fail(SelectedSourceObjectErrorCode::ForeignIdentityWitness);
        }
        let mut positions = Vec::with_capacity(rows.len());
        for row in &rows {
            let Some(position) = package
                .position_of(&row.structural_node)
                .filter(|&p| is_type(p))
            else {
                return fail(SelectedSourceObjectErrorCode::UnknownIdentityWitness);
            };
            if !row_matches_node(package, row, position, &is_type) {
                return fail(SelectedSourceObjectErrorCode::UnknownIdentityWitness);
            }
            positions.push(position);
        }
        if supertypes_cycle(package, &rows, &positions) {
            return fail(SelectedSourceObjectErrorCode::UnknownIdentityWitness);
        }
        let mut keys: Vec<(&str, &str)> = rows
            .iter()
            .map(|row| (row.source_authority.as_ref(), row.source_identity.as_ref()))
            .collect();
        keys.sort_unstable();
        if keys.windows(2).any(|pair| pair[0] == pair[1]) {
            return fail(SelectedSourceObjectErrorCode::ConflictingIdentityWitness);
        }
        let mut joined = std::collections::BTreeMap::new();
        for (row, position) in rows.into_iter().zip(positions) {
            joined.insert(position, row);
        }
        if (0..graph.nodes.len())
            .any(|position| is_type(position) && !joined.contains_key(&position))
        {
            return fail(SelectedSourceObjectErrorCode::MissingIdentityWitness);
        }
        Ok(Self {
            package,
            rows: joined,
        })
    }

    /// The joined row of a source-declared object type, `None` for another
    /// node, another package, or a type this read holds no row for.
    pub fn read(
        &self,
        package: &super::CheckedPackageV2,
        node: &CheckedNodeId,
    ) -> Option<&SelectedSourceObjectRow> {
        if !std::ptr::eq(self.package, package) {
            return None;
        }
        self.rows.get(&package.position_of(node)?)
    }

    /// Every joined row, in graph order.
    pub fn rows(&self) -> impl ExactSizeIterator<Item = &SelectedSourceObjectRow> {
        self.rows.values()
    }
}

/// The field bindings of a source-declared type's body, in order.
fn body_bindings<'p>(
    package: &'p super::CheckedPackageV2,
    position: usize,
) -> Option<Vec<(&'p str, &'p CheckedNodeId, bool)>> {
    let CheckedTerm::Aggregate { members } = package.term_at(position)? else {
        return None;
    };
    members
        .iter()
        .map(|member| match member {
            CheckedTerm::Binding { name, value } => Some((
                name.as_ref(),
                bound_type(value)?,
                matches!(value.as_ref(), CheckedTerm::Aggregate { .. }),
            )),
            _ => None,
        })
        .collect()
}

/// Whether `row` agrees with the source-declared type at `position`: its
/// authority, `source_identity`, members and supertypes.
fn row_matches_node(
    package: &super::CheckedPackageV2,
    row: &SelectedSourceObjectRow,
    position: usize,
    is_type: &dyn Fn(usize) -> bool,
) -> bool {
    let node = &package.resolved_graph().nodes[position];
    let Some(CheckedNodeOwner::Source { authority, .. }) = node.owner.as_ref() else {
        return false;
    };
    let named = node
        .declaration
        .as_ref()
        .and_then(|declaration| declaration.qualified_name.last());
    if *authority != row.source_authority || named != Some(&row.source_identity) {
        return false;
    }
    let Some(bindings) = body_bindings(package, position) else {
        return false;
    };
    if bindings.len() != row.members.len()
        || !bindings
            .iter()
            .zip(&row.members)
            .all(|((name, target, optional), member)| {
                *name == member.name.as_ref()
                    && *optional == member.optional
                    && package.position_of(target).is_some()
                    && package.position_of(target) == package.position_of(&member.type_node)
            })
    {
        return false;
    }
    row.supertypes.iter().all(|supertype| {
        package
            .position_of(supertype)
            .is_some_and(|at| at != position && is_type(at))
    })
}

/// Whether the supertype relation of the supplied rows has a cycle.
fn supertypes_cycle(
    package: &super::CheckedPackageV2,
    rows: &[SelectedSourceObjectRow],
    positions: &[usize],
) -> bool {
    let edges: std::collections::BTreeMap<usize, Vec<usize>> = rows
        .iter()
        .zip(positions)
        .map(|(row, &position)| {
            (
                position,
                row.supertypes
                    .iter()
                    .filter_map(|supertype| package.position_of(supertype))
                    .collect(),
            )
        })
        .collect();
    // 0: unvisited, 1: on the path, 2: done.
    let mut state: std::collections::BTreeMap<usize, u8> = std::collections::BTreeMap::new();
    for &start in edges.keys() {
        if state.contains_key(&start) {
            continue;
        }
        let mut path = vec![(start, 0_usize)];
        state.insert(start, 1);
        while let Some((node, next)) = path.last().copied() {
            let successors = edges.get(&node).map_or(&[][..], Vec::as_slice);
            if let Some(&successor) = successors.get(next) {
                if let Some(top) = path.last_mut() {
                    top.1 += 1;
                }
                match state.get(&successor).copied() {
                    Some(1) => return true,
                    Some(_) => {}
                    None => {
                        state.insert(successor, 1);
                        path.push((successor, 0));
                    }
                }
            } else {
                state.insert(node, 2);
                path.pop();
            }
        }
    }
    false
}
