// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The identity-row half of the source timed-model join (contract-ir FR-374)
//! and the operation-body coverage check (FR-375): every fact of a clock
//! place's row, a parameter constraint or reset, and the `operation_bodies`
//! carrier that only the producer's identity rows show.

use std::collections::BTreeMap;

use super::exact_projection::{
    SelectedSourceRecordRow, SelectedSourceTupleRow, SelectedSourceUnionRow,
};
use super::source_object::{
    is_source_declared_object_type, SelectedClockCollectionLayer, SelectedSourceObjectError,
    SelectedSourceObjectErrorCode, SelectedSourceObjectMember, SelectedSourceObjectRow,
    NATIVE_CLOCK,
};
use super::source_rendering::{self as rendering, Rendered, TypeGraph};
use super::{
    CheckedFrameModification, CheckedNodeId, CheckedNodeKind, CheckedPackageV2, CheckedTerm,
    TimedModelForm,
};
use crate::checked_package::shared::{CheckedPackageRefusalLocation, JsonPointer};

/// The identity rows of the native records, tuples and unions of one identity
/// file, borrowed (FR-374 "Inputs").
#[derive(Clone, Copy, Debug)]
pub struct SelectedSourceClockRows<'r> {
    /// The source record rows.
    pub records: &'r [SelectedSourceRecordRow],
    /// The source tuple rows.
    pub tuples: &'r [SelectedSourceTupleRow],
    /// The source union rows.
    pub unions: &'r [SelectedSourceUnionRow],
}

impl SelectedSourceClockRows<'static> {
    /// No record, tuple or union row: for a package whose identity file holds
    /// none.
    pub const EMPTY: Self = Self {
        records: &[],
        tuples: &[],
        unions: &[],
    };
}

impl TypeGraph for CheckedPackageV2 {
    fn kind(&self, position: usize) -> Option<CheckedNodeKind> {
        self.kinds.get(position).copied()
    }
    fn owner(&self, position: usize) -> Option<&super::CheckedNodeOwner> {
        self.resolved_graph().nodes.get(position)?.owner.as_ref()
    }
    fn declaration(&self, position: usize) -> Option<&super::CheckedDeclaration> {
        self.resolved_graph()
            .nodes
            .get(position)?
            .declaration
            .as_ref()
    }
    fn dependencies(&self, position: usize) -> &[CheckedNodeId] {
        self.resolved_graph()
            .nodes
            .get(position)
            .map_or(&[][..], |node| &node.dependencies)
    }
    fn body(&self, position: usize) -> Option<&CheckedTerm> {
        self.term_at(position)
    }
    fn locate(&self, id: &CheckedNodeId) -> Option<usize> {
        self.position_of(id)
    }
    fn size(&self) -> usize {
        self.resolved_graph().nodes.len()
    }
}

type Failure = SelectedSourceObjectError;

fn refuse(code: SelectedSourceObjectErrorCode, pointer: JsonPointer) -> Failure {
    SelectedSourceObjectError {
        code,
        locus: Some(CheckedPackageRefusalLocation {
            path: Some(pointer),
            source_region: None,
        }),
    }
}

fn node_pointer(position: usize) -> JsonPointer {
    JsonPointer::root()
        .key("semantic_graph")
        .key("nodes")
        .index(position)
}

fn members(term: &CheckedTerm) -> &[CheckedTerm] {
    match term {
        CheckedTerm::Aggregate { members } => members,
        CheckedTerm::OperationAnchorBody(body) => &body.members,
        _ => &[],
    }
}

fn bound(term: &CheckedTerm, index: usize) -> Option<&CheckedTerm> {
    match members(term).get(index)? {
        CheckedTerm::Binding { value, .. } => Some(value),
        _ => None,
    }
}

fn text(term: Option<&CheckedTerm>) -> Option<&str> {
    match term? {
        CheckedTerm::Literal { value, .. } => value.as_str(),
        _ => None,
    }
}

fn target(term: Option<&CheckedTerm>) -> Option<&CheckedNodeId> {
    match term? {
        CheckedTerm::Reference { target } => Some(target),
        _ => None,
    }
}

/// One position of a tuple or of a union payload: a data position (its type
/// node) or a clock position (its rendering and collection layers).
#[derive(Clone, Copy)]
struct Position<'r> {
    type_node: Option<&'r CheckedNodeId>,
    member_type: Option<&'r str>,
    collections: &'r [SelectedClockCollectionLayer],
}

/// Where the bounds of the `slot` steps after the current row come from.
#[derive(Clone, Copy)]
enum Slots<'r> {
    None,
    Layers(&'r [SelectedClockCollectionLayer]),
    Node(usize),
}

/// The identity-row join of every source timed model and operation body
/// coverage; the first failure is reported.
pub(super) fn join(
    package: &CheckedPackageV2,
    joined: &BTreeMap<usize, SelectedSourceObjectRow>,
    clock: SelectedSourceClockRows<'_>,
) -> Result<(), Failure> {
    join_places(package, joined, clock)?;
    join_parameters(package, joined)?;
    check_coverage(package, joined)
}

#[ix_trace_rs::string_edge]
fn join_places(
    package: &CheckedPackageV2,
    joined: &BTreeMap<usize, SelectedSourceObjectRow>,
    clock: SelectedSourceClockRows<'_>,
) -> Result<(), Failure> {
    let graph = package.resolved_graph();
    for (position, node) in graph.nodes.iter().enumerate() {
        if package.kinds.get(position) != Some(&CheckedNodeKind::TimedModel(TimedModelForm::Place))
        {
            continue;
        }
        let Some(body) = package.term_at(position) else {
            continue;
        };
        let Some(root) = target(bound(body, 0)).and_then(|id| package.position_of(id)) else {
            continue;
        };
        let (Some(root_node), Some(root_kind)) = (graph.nodes.get(root), package.kinds.get(root))
        else {
            continue;
        };
        if !is_source_declared_object_type(root_node, *root_kind) {
            continue;
        }
        let Some(super::CheckedNodeOwner::Source { identity, .. }) = root_node.owner.as_ref()
        else {
            continue;
        };
        let steps = bound(body, 1).map(members).unwrap_or_default();
        let rows = bound(body, 2).map(members).unwrap_or_default();
        let _ = node;
        let mut slots = Slots::None;
        let mut depth = 0_usize;
        let mut product: Option<Vec<Position<'_>>> = None;
        for (index, (step, row)) in steps.iter().zip(rows).enumerate() {
            let at = || {
                node_pointer(position)
                    .key("body")
                    .key("members")
                    .index(2)
                    .key("value")
                    .key("members")
                    .index(index)
            };
            let bad = || refuse(SelectedSourceObjectErrorCode::MalformedClockPlace, at());
            let declaring = text(bound(row, 1)).unwrap_or_default();
            let member_type = text(bound(row, 3)).unwrap_or_default();
            let kind = text(bound(step, 0)).unwrap_or_default();
            let previous_product = product.take();
            match kind {
                "slot" => {
                    depth += 1;
                }
                "field" | "tuple" | "union" => {
                    depth = 0;
                    slots = Slots::None;
                    let name = text(bound(step, 1)).unwrap_or_default();
                    let ordinal = match bound(step, 1) {
                        Some(CheckedTerm::Literal { value, .. }) => value.as_u64(),
                        _ => None,
                    };
                    match kind {
                        "field" => {
                            let Some(declared) = rendering::declaration_node(
                                package,
                                identity,
                                declaring,
                                rendering::DeclaringStep::Field,
                            ) else {
                                continue;
                            };
                            if let Some((_, target)) =
                                rendering::binding_of(package, declared, name)
                            {
                                // The strict read compared the row's type; the
                                // collections under it are read layer by layer.
                                slots =
                                    package.position_of(target).map_or(Slots::None, Slots::Node);
                            } else {
                                let Some(entry) =
                                    clock_entry(package, joined, clock, declared, name)
                                else {
                                    return Err(bad());
                                };
                                if entry.type_node.is_some()
                                    || entry.member_type.as_deref() != Some(member_type)
                                {
                                    return Err(bad());
                                }
                                slots = Slots::Layers(entry.collections);
                            }
                        }
                        "tuple" => {
                            let positions = match &previous_product {
                                Some(positions) if declaring.starts_with('(') => positions.clone(),
                                _ => {
                                    let Some(declared) = rendering::declaration_node(
                                        package,
                                        identity,
                                        declaring,
                                        rendering::DeclaringStep::Tuple,
                                    ) else {
                                        continue;
                                    };
                                    let Some(row) = tuple_row(package, clock, declared) else {
                                        return Err(bad());
                                    };
                                    row.positions
                                        .iter()
                                        .map(|position| Position {
                                            type_node: position.type_node.as_ref(),
                                            member_type: position.member_type.as_deref(),
                                            collections: &position.collections,
                                        })
                                        .collect()
                                }
                            };
                            let Some(found) = ordinal
                                .and_then(|ordinal| usize::try_from(ordinal).ok())
                                .and_then(|ordinal| positions.get(ordinal))
                            else {
                                return Err(bad());
                            };
                            if !position_matches(package, found, member_type) {
                                return Err(bad());
                            }
                            slots = slots_of(package, found);
                        }
                        _ => {
                            let Some(declared) = rendering::declaration_node(
                                package,
                                identity,
                                declaring,
                                rendering::DeclaringStep::Union,
                            ) else {
                                continue;
                            };
                            let Some(row) = union_row(package, clock, declared) else {
                                return Err(bad());
                            };
                            let Some(member) = row
                                .members
                                .iter()
                                .find(|member| member.member.identifier() == name)
                            else {
                                return Err(bad());
                            };
                            let positions = payload_positions(member);
                            if positions.is_empty() {
                                return Err(bad());
                            }
                            let parts: Vec<Rendered> = positions
                                .iter()
                                .map(|position| position_rendering(package, position))
                                .collect();
                            let agrees = if parts.iter().any(|part| part == &Rendered::Unknown) {
                                false
                            } else if parts.len() > 1
                                && parts.iter().any(|part| part == &Rendered::Interim)
                            {
                                // Only a product's whole text may skip a part
                                // FSpec FR-789 gives no rendering yet.
                                true
                            } else {
                                let texts: Vec<&str> = parts
                                    .iter()
                                    .filter_map(|part| match part {
                                        Rendered::Known(text) => Some(text.as_str()),
                                        _ => None,
                                    })
                                    .collect();
                                match (parts.len(), texts.as_slice()) {
                                    (1, [only]) => *only == member_type,
                                    (1, _) => false,
                                    (_, many) => format!("({})", many.join(", ")) == member_type,
                                }
                            };
                            if !agrees {
                                return Err(bad());
                            }
                            if let [only] = positions.as_slice() {
                                slots = slots_of(package, only);
                            }
                            product = Some(positions);
                        }
                    }
                }
                _ => continue,
            }
            let next_is_slot = steps
                .get(index + 1)
                .is_some_and(|next| text(bound(next, 0)) == Some("slot"));
            if next_is_slot {
                let collection = text(bound(row, 4)).unwrap_or_default();
                let minimum = text(bound(row, 5)).unwrap_or_default();
                let maximum = text(bound(row, 6)).unwrap_or_default();
                let fits = match slots {
                    // A slot needs a collection layer to check against.
                    Slots::None => false,
                    Slots::Layers(layers) => layers.get(depth).is_some_and(|layer| {
                        layer.kind.as_ref() == collection
                            && layer.minimum.as_ref() == minimum
                            && layer.maximum.as_ref() == maximum
                    }),
                    Slots::Node(node) => rendering::collection_layer(package, node, depth)
                        .is_some_and(|layer| layer == (collection, minimum, maximum)),
                };
                if !fits {
                    return Err(bad());
                }
            }
        }
    }
    Ok(())
}

/// The clock entry of the type at `declared`: an object type's row member or
/// a record row's member named `name`.
fn clock_entry<'r>(
    package: &CheckedPackageV2,
    joined: &'r BTreeMap<usize, SelectedSourceObjectRow>,
    clock: SelectedSourceClockRows<'r>,
    declared: usize,
    name: &str,
) -> Option<EntryRef<'r>> {
    if let Some(row) = joined.get(&declared) {
        return row
            .members
            .iter()
            .find(|member| member.name.as_ref() == name)
            .map(EntryRef::from_member);
    }
    let id = &package.resolved_graph().nodes.get(declared)?.node_id;
    clock
        .records
        .iter()
        .find(|row| &row.structural_node == id)?
        .members
        .iter()
        .find(|member| member.admitted_name.as_ref() == name)
        .map(|member| EntryRef {
            type_node: member.type_node.as_ref(),
            member_type: member.member_type.clone(),
            collections: &member.collections,
        })
}

struct EntryRef<'r> {
    type_node: Option<&'r CheckedNodeId>,
    member_type: Option<Box<str>>,
    collections: &'r [SelectedClockCollectionLayer],
}

impl<'r> EntryRef<'r> {
    fn from_member(member: &'r SelectedSourceObjectMember) -> Self {
        Self {
            type_node: member.type_node.as_ref(),
            member_type: member.member_type.clone(),
            collections: &member.collections,
        }
    }
}

fn tuple_row<'r>(
    package: &CheckedPackageV2,
    clock: SelectedSourceClockRows<'r>,
    declared: usize,
) -> Option<&'r SelectedSourceTupleRow> {
    let id = &package.resolved_graph().nodes.get(declared)?.node_id;
    clock.tuples.iter().find(|row| &row.structural_node == id)
}

fn union_row<'r>(
    package: &CheckedPackageV2,
    clock: SelectedSourceClockRows<'r>,
    declared: usize,
) -> Option<&'r SelectedSourceUnionRow> {
    let id = &package.resolved_graph().nodes.get(declared)?.node_id;
    clock.unions.iter().find(|row| &row.structural_node == id)
}

/// The payload positions of a union member by ordinal: its clock positions at
/// their ordinals and its data positions in the others, in order.
fn payload_positions(
    member: &super::exact_projection::SelectedUnionMemberRow,
) -> Vec<Position<'_>> {
    let total = member.payload_nodes.len() + member.clock_payload.len();
    let mut data = member.payload_nodes.iter();
    (0..total)
        .map(|ordinal| {
            match member
                .clock_payload
                .iter()
                .find(|entry| usize::try_from(entry.ordinal) == Ok(ordinal))
            {
                Some(entry) => Position {
                    type_node: None,
                    member_type: Some(&entry.member_type),
                    collections: &entry.collections,
                },
                None => Position {
                    type_node: data.next(),
                    member_type: None,
                    collections: &[],
                },
            }
        })
        .collect()
}

/// A position's rendering: a clock position's own, a data position's from its
/// type node.
fn position_rendering(package: &CheckedPackageV2, position: &Position<'_>) -> Rendered {
    match (position.member_type, position.type_node) {
        (Some(rendered), None) => Rendered::Known(rendered.to_owned()),
        (None, Some(node)) => package
            .position_of(node)
            .and_then(|at| rendering::render(package, at, &mut || Ok::<(), ()>(())).ok())
            .unwrap_or(Rendered::Unknown),
        _ => Rendered::Unknown,
    }
}

fn position_matches(
    package: &CheckedPackageV2,
    position: &Position<'_>,
    member_type: &str,
) -> bool {
    match position_rendering(package, position) {
        Rendered::Known(rendered) => rendered == member_type,
        // A range, a text bound or a quantity can never hold a Clock.
        Rendered::Interim | Rendered::Unknown => false,
    }
}

fn slots_of<'r>(package: &CheckedPackageV2, position: &Position<'r>) -> Slots<'r> {
    match (position.member_type, position.type_node) {
        (Some(_), None) => Slots::Layers(position.collections),
        (None, Some(node)) => package.position_of(node).map_or(Slots::None, Slots::Node),
        _ => Slots::None,
    }
}

/// The `Clock` parameters of an operation, and whether a row lists it: the
/// root row's entry, or the nearest supertype's in `supertypes` order.
#[ix_trace_rs::string_edge]
fn operation_entry<'r>(
    package: &CheckedPackageV2,
    joined: &'r BTreeMap<usize, SelectedSourceObjectRow>,
    root: usize,
    operation: &str,
) -> Option<&'r [Box<str>]> {
    let mut seen = std::collections::BTreeSet::new();
    let mut queue = std::collections::VecDeque::from([root]);
    while let Some(position) = queue.pop_front() {
        if !seen.insert(position) {
            continue;
        }
        let row = joined.get(&position)?;
        if let Some(entry) = row.operations.iter().find(|entry| {
            entry
                .identity
                .strip_prefix(row.source_identity.as_ref())
                .and_then(|rest| rest.strip_prefix("::"))
                == Some(operation)
        }) {
            return Some(&entry.clock_parameters);
        }
        queue.extend(
            row.supertypes
                .iter()
                .filter_map(|id| package.position_of(id)),
        );
    }
    None
}

fn join_parameters(
    package: &CheckedPackageV2,
    joined: &BTreeMap<usize, SelectedSourceObjectRow>,
) -> Result<(), Failure> {
    let graph = package.resolved_graph();
    for position in 0..graph.nodes.len() {
        let is_reset = match package.kinds.get(position) {
            Some(CheckedNodeKind::TimedModel(TimedModelForm::Reset)) => true,
            Some(CheckedNodeKind::TimedModel(TimedModelForm::Constraint)) => false,
            _ => continue,
        };
        let Some(body) = package.term_at(position) else {
            continue;
        };
        if text(bound(body, 0)) != Some("parameter") {
            continue;
        }
        let Some(anchor) = target(bound(body, 1)).and_then(|id| package.position_of(id)) else {
            continue;
        };
        let Some(anchor_body) = package.term_at(anchor) else {
            continue;
        };
        let Some(root) = target(bound(anchor_body, 0)).and_then(|id| package.position_of(id))
        else {
            continue;
        };
        if !joined.contains_key(&root) {
            continue;
        }
        let operation = text(bound(anchor_body, 1)).unwrap_or_default();
        let parameter = text(bound(body, 2)).unwrap_or_default();
        let at_parameter = || {
            refuse(
                SelectedSourceObjectErrorCode::TimedNodeReference,
                node_pointer(position).key("body").key("members").index(2),
            )
        };
        let Some(clocks) = operation_entry(package, joined, root, operation) else {
            return Err(at_parameter());
        };
        if !clocks.iter().any(|name| name.as_ref() == parameter) {
            return Err(at_parameter());
        }
        if !is_reset {
            continue;
        }
        let root_id = &graph.nodes[root].node_id;
        let writes_clock = target(bound(anchor_body, 2))
            .and_then(|id| package.position_of(id))
            .and_then(|frame| package.term_at(frame))
            .is_some_and(|frame| match frame {
                CheckedTerm::Frame { modifies, .. } => modifies.iter().any(|entry| {
                    matches!(entry, CheckedFrameModification::Field { declaration, name, .. }
                    if declaration == root_id
                        && joined.get(&root).is_some_and(|row| {
                            row.members.iter().any(|member| {
                                member.name.as_ref() == name.as_ref()
                                    && member.member_type.as_deref() == Some(NATIVE_CLOCK)
                            })
                        }))
                }),
                _ => false,
            });
        if !writes_clock {
            return Err(refuse(
                SelectedSourceObjectErrorCode::TimedNodeReference,
                node_pointer(position)
                    .key("body")
                    .key("members")
                    .index(1)
                    .key("value")
                    .key("target"),
            ));
        }
    }
    Ok(())
}

/// FR-375: the `operation_bodies` carrier holds exactly one row for each
/// operation each source type's row lists, and no other row of a source type.
#[ix_trace_rs::string_edge]
fn check_coverage(
    package: &CheckedPackageV2,
    joined: &BTreeMap<usize, SelectedSourceObjectRow>,
) -> Result<(), Failure> {
    let Some(rows) = package.operation_body_rows() else {
        return Ok(());
    };
    let carrier = || JsonPointer::root().key("operation_bodies");
    let graph = package.resolved_graph();
    for (&position, row) in joined {
        let id = &graph.nodes[position].node_id;
        for entry in &row.operations {
            let Some(operation) = entry
                .identity
                .strip_prefix(row.source_identity.as_ref())
                .and_then(|rest| rest.strip_prefix("::"))
            else {
                continue;
            };
            let covered = rows
                .iter()
                .any(|carried| &carried.declaring == id && carried.operation.as_ref() == operation);
            if !covered {
                return Err(refuse(
                    SelectedSourceObjectErrorCode::InvalidValue,
                    carrier(),
                ));
            }
        }
    }
    for (index, carried) in rows.iter().enumerate() {
        let Some(position) = package.position_of(&carried.declaring) else {
            continue;
        };
        let Some(row) = joined.get(&position) else {
            continue;
        };
        let listed = row.operations.iter().any(|entry| {
            entry
                .identity
                .strip_prefix(row.source_identity.as_ref())
                .and_then(|rest| rest.strip_prefix("::"))
                == Some(carried.operation.as_ref())
        });
        if !listed {
            return Err(refuse(
                SelectedSourceObjectErrorCode::InvalidValue,
                carrier().index(index).key("operation"),
            ));
        }
    }
    Ok(())
}
