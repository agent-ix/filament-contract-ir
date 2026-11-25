// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The requirement records carried by a checked-package V2 envelope.

use super::stage_storage::{self, Path, Refusal, Site};
use super::{
    CheckedNodeKind, CheckedSemanticNodeV2, CheckedTerm, GraphIndex, LiteralKind, ModelForm,
    PackageReadStage, RawSemanticNodeV2, ScalarTypeForm, ValueForm, WorkMeter,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{CheckedNodeId, CheckedOccurrence};
use crate::checked_package::shared::{
    CheckedNodePathSegment, CheckedOperandEntry, CheckedSourceMapEntry, CheckedSourceRef,
};
use crate::checked_package::shared::{
    CheckedOccurrenceRole, CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
};
use quire_canonical::{Encode, Error, FixedShape, Sink, Writer};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// One capability requested at a claim occurrence.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
pub enum CapabilityKind {
    /// Scalar-expression definedness or range validity.
    #[serde(rename = "value-validity")]
    ValueValidity,
    /// State-operation contract validity.
    #[serde(rename = "operation-contract")]
    OperationContract,
    /// Finite execution replay.
    #[serde(rename = "finite-replay")]
    FiniteReplay,
    /// Temporal requirement satisfaction.
    #[serde(rename = "temporal-satisfaction")]
    TemporalSatisfaction,
    /// Finite global protocol conformance.
    #[serde(rename = "global-conformance")]
    GlobalConformance,
    /// Protocol monitorability.
    #[serde(rename = "monitorability")]
    Monitorability,
    /// Local protocol projection.
    #[serde(rename = "local-projection")]
    LocalProjection,
    /// Protocol refinement.
    #[serde(rename = "refinement")]
    Refinement,
    /// Protocol realizability.
    #[serde(rename = "realizability")]
    Realizability,
    /// Protocol composition.
    #[serde(rename = "composition")]
    Composition,
    /// Probabilistic claim satisfaction.
    #[serde(rename = "probabilistic-satisfaction")]
    ProbabilisticSatisfaction,
    /// Real-time task schedulability.
    #[serde(rename = "schedulability")]
    Schedulability,
}

/// One domain kind used by an unbounded extent.
#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize, FixedShape,
)]
pub enum DomainKind {
    /// Unbounded collection cardinality.
    #[serde(rename = "collection")]
    Collection,
    /// Unbounded model population.
    #[serde(rename = "population")]
    Population,
    /// Unbounded integer range.
    #[serde(rename = "integer")]
    Integer,
    /// Unbounded recursive depth.
    #[serde(rename = "recursive")]
    Recursive,
    /// Unbounded quantity magnitude.
    #[serde(rename = "quantity")]
    Quantity,
    /// Unbounded loop count.
    #[serde(rename = "loop")]
    Loop,
    /// Infinite-trace behavior.
    #[serde(rename = "infinite-trace")]
    InfiniteTrace,
}

/// The typed identity of one domain contributing to an unbounded request.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum DomainKey {
    /// A type domain rooted at a graph node.
    #[serde(rename = "node")]
    Node {
        /// The graph node carrying the domain.
        node: CheckedNodeId,
        /// Numeric path into that node's type.
        path: Vec<u32>,
    },
    /// An unbounded model population.
    #[serde(rename = "population")]
    Population {
        /// The population's canonical member type.
        member_type: CheckedNodeId,
        /// Population declaration ordinal.
        ordinal: u32,
    },
}

impl Encode for DomainKey {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        match self {
            Self::Node { node, path } => {
                writer.name("type")?;
                writer.string("node")?;
                super::encode::encoded(writer, "node", node)?;
                super::encode::member(writer, "path", path)?;
            }
            Self::Population {
                member_type,
                ordinal,
            } => {
                writer.name("type")?;
                writer.string("population")?;
                super::encode::encoded(writer, "member_type", member_type)?;
                super::encode::member(writer, "ordinal", ordinal)?;
            }
        }
        writer.end_object()
    }
}

/// The record's bounded or unbounded classification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RequirementExtent {
    /// No unbounded domains contribute to the request.
    Bounded,
    /// Ordered unbounded domains contributing to the request.
    Unbounded(Vec<(DomainKey, DomainKind)>),
}

/// The JSON representation of a requirement extent.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "classification", deny_unknown_fields)]
pub(super) enum RequirementExtentWire {
    #[serde(rename = "bounded")]
    Bounded,
    #[serde(rename = "unbounded")]
    Unbounded { domains: Vec<RequirementDomain> },
}

/// One ordered domain in an unbounded extent.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RequirementDomain {
    pub domain: DomainKey,
    pub kind: DomainKind,
}

/// A row's claim node and occurrence key.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RequirementClaimKey {
    pub node: CheckedNodeId,
    pub occurrence: CheckedOccurrence,
}

/// One requirement item in package order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequirementItemRow {
    /// The graph node the request item is keyed at.
    pub claim: CheckedNodeId,
    /// The occurrence on the claim node.
    pub occurrence: CheckedOccurrence,
    /// Requested capability.
    pub kind: CapabilityKind,
    /// The request's extent classification.
    pub extent: RequirementExtent,
    /// Value-validity or operation-contract record, when this row carries one.
    pub record: Option<RequirementRecord>,
}

/// One typed requirement record retained from an admitted package.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RequirementRecord {
    /// Scalar-operation definedness and range facts.
    ValueValidity(ValueValidityRecord),
    /// State-operation contract facts.
    OperationContract(OperationContractRecord),
}

impl RequirementRecord {
    fn node(&self) -> &CheckedNodeId {
        match self {
            Self::ValueValidity(record) => &record.node,
            Self::OperationContract(record) => &record.node,
        }
    }
}

impl Encode for RequirementItemRow {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("claim")?;
        writer.begin_object()?;
        super::encode::encoded(writer, "node", &self.claim)?;
        super::encode::member(writer, "occurrence", &self.occurrence)?;
        writer.end_object()?;
        super::encode::member(writer, "kind", &self.kind)?;
        writer.name("extent")?;
        writer.begin_object()?;
        match &self.extent {
            RequirementExtent::Bounded => {}
            RequirementExtent::Unbounded(domains) => {
                writer.name("classification")?;
                writer.string("unbounded")?;
                writer.name("domains")?;
                writer.begin_array()?;
                for (domain, kind) in domains {
                    writer.begin_object()?;
                    super::encode::encoded(writer, "domain", domain)?;
                    super::encode::member(writer, "kind", kind)?;
                    writer.end_object()?;
                }
                writer.end_array()?;
            }
        }
        if matches!(&self.extent, RequirementExtent::Bounded) {
            writer.name("classification")?;
            writer.string("bounded")?;
        }
        writer.end_object()?;
        if let Some(record) = &self.record {
            super::encode::encoded(writer, "record", record.node())?;
        }
        writer.end_object()
    }
}

/// The JSON-facing requirement row retained during intake before graph joins.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RequirementItemWireRow {
    pub claim: RequirementClaimKey,
    pub kind: CapabilityKind,
    pub extent: RequirementExtentWire,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub record: Option<CheckedNodeId>,
}

/// A package-local index into the admitted shared path-condition frame table.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PathConditionFrameId(pub usize);

/// One admitted parent-linked path-condition frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathConditionFrame {
    /// The immediately enclosing condition frame.
    pub parent: Option<PathConditionFrameId>,
    /// Boolean expression or literal/parameter value node used as the guard.
    pub guard: CheckedNodeId,
    /// Expression occurrence of the guard read as this condition.
    pub occurrence: u64,
    /// Outcome required for this branch.
    pub holds: bool,
}

/// One borrowed guard yielded by a path-condition view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RequirementGuard<'a> {
    /// Boolean expression or literal/parameter value node used as the guard.
    pub guard: &'a CheckedNodeId,
    /// Expression occurrence of the guard read as this condition.
    pub occurrence: u64,
    /// Outcome required for this branch.
    pub holds: bool,
}

/// A borrowed outermost-first view over a shared parent-linked condition chain.
#[derive(Clone, Copy, Debug)]
pub struct PathConditionView<'a> {
    frames: &'a [PathConditionFrame],
    terminal: Option<PathConditionFrameId>,
}

impl<'a> PathConditionView<'a> {
    pub(super) fn new(
        frames: &'a [PathConditionFrame],
        terminal: Option<PathConditionFrameId>,
    ) -> Self {
        Self { frames, terminal }
    }

    /// Returns the number of guards in this chain.
    pub fn len(self) -> usize {
        let mut count = 0;
        let mut current = self.terminal;
        while let Some(id) = current {
            let Some(frame) = self.frames.get(id.0) else {
                return 0;
            };
            count += 1;
            current = frame.parent;
        }
        count
    }

    /// Returns whether this path has no guards.
    pub fn is_empty(self) -> bool {
        self.terminal.is_none()
    }

    /// Iterates guards outermost first without allocating a per-record vector.
    pub fn iter(self) -> PathConditionIter<'a> {
        PathConditionIter {
            frames: self.frames,
            terminal: self.terminal,
            remaining: self.len(),
        }
    }
}

/// The allocation-free iterator for a path-condition view.
#[derive(Clone, Debug)]
pub struct PathConditionIter<'a> {
    frames: &'a [PathConditionFrame],
    terminal: Option<PathConditionFrameId>,
    remaining: usize,
}

impl<'a> Iterator for PathConditionIter<'a> {
    type Item = RequirementGuard<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let mut current = self.terminal?;
        for _ in 1..self.remaining {
            current = self.frames.get(current.0)?.parent?;
        }
        let frame = self.frames.get(current.0)?;
        self.remaining -= 1;
        Some(RequirementGuard {
            guard: &frame.guard,
            occurrence: frame.occurrence,
            holds: frame.holds,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl ExactSizeIterator for PathConditionIter<'_> {}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PathConditionFrameWire {
    pub parent: Option<u64>,
    pub guard: PathConditionGuardWire,
    pub holds: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PathConditionGuardWire {
    pub node: CheckedNodeId,
    pub occurrence: CheckedOccurrence,
}

impl Encode for PathConditionFrame {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("parent")?;
        match self.parent {
            Some(parent) => {
                writer.integer(i128::try_from(parent.0).map_err(|_| Error::Internal {
                    invariant: "path-condition frame index exceeds the canonical integer range",
                })?)?
            }
            None => writer.null()?,
        }
        writer.name("guard")?;
        writer.begin_object()?;
        super::encode::encoded(writer, "node", &self.guard)?;
        writer.name("occurrence")?;
        writer.begin_object()?;
        writer.name("role")?;
        writer.string("expression")?;
        super::encode::member(writer, "ordinal", &self.occurrence)?;
        writer.end_object()?;
        writer.end_object()?;
        super::encode::member(writer, "holds", &self.holds)?;
        writer.end_object()
    }
}

fn root() -> Path<'static> {
    Path::root().key("requirement_items")
}

fn site(index: usize, at: Path<'_>) -> Site<'_> {
    Site::new(PackageReadStage::Graph, index, "requirement_items", at)
}

fn first_unnamed_position(
    records: &[usize],
    named_positions: &[usize],
    on_comparison: &mut impl FnMut(),
) -> Option<usize> {
    for &record in records {
        let mut lower = 0;
        let mut upper = named_positions.len();
        let mut found = false;
        while lower < upper {
            let middle = lower + (upper - lower) / 2;
            on_comparison();
            match named_positions[middle].cmp(&record) {
                std::cmp::Ordering::Less => lower = middle + 1,
                std::cmp::Ordering::Greater => upper = middle,
                std::cmp::Ordering::Equal => {
                    found = true;
                    break;
                }
            }
        }
        if found {
            continue;
        }
        if lower < named_positions.len() {
            on_comparison();
            if named_positions[lower] == record {
                continue;
            }
        }
        return Some(record);
    }
    None
}

/// The order of the structural keys at graph positions `left` and `right`.
///
/// The reader's graph index ranks every structural key once, in one pass that
/// enters each operand key before the keys nesting it (hash-consing: a key's
/// entry is compared through its operands' entries, never by walking a shared
/// operand chain). A rank is the key's position in `CheckedNodeId::cmp`
/// order, and equal keys share one entry and so one rank. Two ranked keys
/// therefore compare by their ranks exactly as `CheckedNodeId::cmp` compares
/// them, in O(1) where that comparison walks a chain of `k` nested
/// applications in O(k). An unranked graph compares through the meter, which
/// charges the frame list a nested comparison asks for.
fn key_order(
    index: &GraphIndex<'_>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    left: usize,
    right: usize,
    meter: &mut WorkMeter,
) -> Result<std::cmp::Ordering, ValidationFailure> {
    index
        .resolved_at(left, structural_keys)
        .order(&index.resolved_at(right, structural_keys), meter)
}

/// Sorts `items` by a fallible `compare` with a bottom-up merge sort,
/// returning the first refusal as soon as `compare` returns it. The scratch
/// block is charged to the meter before it is made. A comparator that
/// refuses never feeds a sort inconsistent answers, so the sort cannot
/// panic on a non-total order.
///
/// `compare` must be a strict total order (no two distinct items compare
/// equal). Every correct sort then yields the one ascending permutation, so
/// this sort's result is the one any other sort by the same order yields.
fn sort_charged(
    items: &mut Vec<usize>,
    meter: &mut WorkMeter,
    mut compare: impl FnMut(
        usize,
        usize,
        &mut WorkMeter,
    ) -> Result<std::cmp::Ordering, ValidationFailure>,
) -> Result<(), ValidationFailure> {
    let len = items.len();
    if len < 2 {
        return Ok(());
    }
    let mut scratch: Vec<usize> = meter.vec(len)?;
    scratch.resize(len, 0);
    let mut width = 1;
    while width < len {
        let mut start = 0;
        while start < len {
            let middle = start.saturating_add(width).min(len);
            let end = start.saturating_add(width.saturating_mul(2)).min(len);
            let (mut left, mut right) = (start, middle);
            for slot in &mut scratch[start..end] {
                let take_right = right < end
                    && (left == middle || compare(items[right], items[left], meter)?.is_lt());
                if take_right {
                    *slot = items[right];
                    right += 1;
                } else {
                    *slot = items[left];
                    left += 1;
                }
            }
            start = end;
        }
        std::mem::swap(items, &mut scratch);
        width = width.saturating_mul(2);
    }
    Ok(())
}

/// Sorts graph positions into ascending structural-key order, a key tie
/// (which a valid graph cannot hold: see below) broken by position.
///
/// Every key comparison answers as `CheckedNodeId::cmp` on the two keys does
/// (see [`key_order`]), with O(1) work on a ranked graph. Distinct graph
/// positions hold distinct structural keys: the graph stage refused a
/// repeated node id before this step runs, and a structural key replaces
/// each index segment of the id by its target's key, which keeps distinct
/// ids distinct by induction over the acyclic graph. So the position
/// tie-break never decides, the order is strict and total, and the result is
/// the permutation the retired `sort_unstable_by(CheckedNodeId::cmp)` gave.
fn sort_by_structural_key(
    positions: &mut Vec<usize>,
    index: &GraphIndex<'_>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    sort_charged(positions, meter, |left, right, meter| {
        Ok(key_order(index, structural_keys, left, right, meter)?.then(left.cmp(&right)))
    })
}

/// The order in which named records are checked for dense per-owner
/// ordinals: by owner key, then row index, then record key, as indices into
/// `named_records`. Row indices are distinct, so the order is strict and
/// total and the record-key step never decides.
fn named_record_order(
    named_records: &[(usize, usize, CheckedNodeId)],
    index: &GraphIndex<'_>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    meter: &mut WorkMeter,
) -> Result<Vec<usize>, ValidationFailure> {
    let mut order: Vec<usize> = meter.vec(named_records.len())?;
    order.extend(0..named_records.len());
    sort_charged(&mut order, meter, |left, right, meter| {
        let (left_row, left, left_owner) = &named_records[left];
        let (right_row, right, right_owner) = &named_records[right];
        let left_owner_position = index.get(left_owner).copied().unwrap_or(usize::MAX);
        let right_owner_position = index.get(right_owner).copied().unwrap_or(usize::MAX);
        let order = key_order(
            index,
            structural_keys,
            left_owner_position,
            right_owner_position,
            meter,
        )?
        .then_with(|| left_row.cmp(right_row));
        if order.is_ne() {
            return Ok(order);
        }
        key_order(index, structural_keys, *left, *right, meter)
    })?;
    Ok(order)
}

/// A requirement row's package-order key: the claim's structural key, then
/// its occurrence role, occurrence ordinal and capability, as wire spellings.
type RowKey<'a> = (
    super::graph_storage::ResolvedKey<'a>,
    &'static str,
    u64,
    &'static str,
);

/// The lexicographic order of two row keys, the claim key compared as
/// [`key_order`] compares it.
fn row_key_order(
    left: &RowKey<'_>,
    right: &RowKey<'_>,
    meter: &mut WorkMeter,
) -> Result<std::cmp::Ordering, ValidationFailure> {
    Ok(left
        .0
        .order(&right.0, meter)?
        .then_with(|| (left.1, left.2, left.3).cmp(&(right.1, right.2, right.3))))
}

#[cfg(test)]
mod order_tests;

fn refuse(
    index: usize,
    at: Path<'_>,
    cause: Cause,
    node: Option<&CheckedNodeId>,
    meter: &mut WorkMeter,
) -> ValidationFailure {
    let refusal = Refusal::new(Code::InvalidPackage, at).because(cause);
    let refusal = node.map_or(refusal, |node| refusal.at_node(node));
    stage_storage::refuse(meter, &site(index, at), refusal)
}

#[allow(clippy::too_many_arguments)] // each argument is a separate admitted table the row checks join
pub(super) fn validate(
    rows: Option<&[RequirementItemWireRow]>,
    path_condition_frames: Option<&[PathConditionFrameWire]>,
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[CheckedTerm],
    index: &GraphIndex<'_>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    operand_rows: &[Vec<CheckedOperandEntry>],
    source_map: &[CheckedSourceMapEntry],
    evidence: &super::CheckedPackageEvidence,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    validate_frames(path_condition_frames, nodes, kinds, index, meter)?;
    let frame_index = path_condition_frame_index(path_condition_frames, index, meter)?;
    let mut records: Vec<usize> = kinds
        .iter()
        .enumerate()
        .filter_map(|(position, kind)| {
            matches!(
                kind,
                CheckedNodeKind::Claim(
                    super::ClaimForm::ValueValidity | super::ClaimForm::OperationContract
                )
            )
            .then_some(position)
        })
        .collect();
    sort_by_structural_key(&mut records, index, structural_keys, meter)?;
    let operation_records: Vec<usize> = records
        .iter()
        .copied()
        .filter(|&position| {
            kinds[position] == CheckedNodeKind::Claim(super::ClaimForm::OperationContract)
        })
        .collect();
    let mut owner_paths: Option<Vec<DerivedPath>> = None;
    let mut used_frames =
        meter.vec(path_condition_frames.map_or(0, <[PathConditionFrameWire]>::len))?;
    used_frames.resize(
        path_condition_frames.map_or(0, <[PathConditionFrameWire]>::len),
        false,
    );
    for &position in &records {
        if kinds[position] == CheckedNodeKind::Claim(super::ClaimForm::OperationContract) {
            validate_operation_contract_node(
                position,
                nodes,
                kinds,
                terms,
                index,
                operand_rows,
                evidence,
                meter,
            )?;
            continue;
        }
        let node = &nodes[position];
        let node_at = Path::root()
            .key("semantic_graph")
            .key("nodes")
            .index(position);
        stage_storage::work(meter, &site(position, node_at), 1)?;
        if node.declaration.is_some()
            || node.occurrences.len() != 1
            || node.occurrences[0].role != CheckedOccurrenceRole::Generated
        {
            let at = node_at.key("occurrences");
            return Err(refuse(
                position,
                at,
                Cause::InvalidValue,
                Some(&node.node_id),
                meter,
            ));
        }
        let valid_body = terms.get(position).is_some_and(record_binding_shape);
        if !valid_body {
            let at = node_at.key("body");
            return Err(refuse(
                position,
                at,
                Cause::InvalidValue,
                Some(&node.node_id),
                meter,
            ));
        }
        for member in [5, 6, 7] {
            if let Some(CheckedTerm::Aggregate { members, .. }) =
                terms.get(position).and_then(|term| binding(term, member))
            {
                for entry in 0..members.len() {
                    let at = node_at
                        .key("body")
                        .key("members")
                        .index(member)
                        .key("value")
                        .key("members")
                        .index(entry);
                    stage_storage::work(meter, &site(position, at), 1)?;
                }
            } else {
                let at = node_at
                    .key("body")
                    .key("members")
                    .index(member)
                    .key("value");
                return Err(refuse(
                    position,
                    at,
                    Cause::InvalidValue,
                    Some(&node.node_id),
                    meter,
                ));
            }
        }
        let Some(record) = terms
            .get(position)
            .and_then(|term| read_record(node.node_id.clone(), term))
        else {
            let at = terms.get(position).map_or_else(
                || node_at.key("body"),
                |term| record_body_failure_path(term, node_at),
            );
            return Err(refuse(
                position,
                at,
                Cause::InvalidValue,
                Some(&node.node_id),
                meter,
            ));
        };
        let owner_at = node_at.key("body").key("members").index(0);
        let Some(&owner_position) = index
            .get(&record.owner)
            .filter(|&&owner| matches!(kinds[owner], CheckedNodeKind::Function(_)))
        else {
            return Err(refuse(
                position,
                owner_at,
                Cause::InvalidValue,
                Some(&node.node_id),
                meter,
            ));
        };
        let application_at = node_at.key("body").key("members").index(1);
        let Some(&application_position) = index
            .get(&record.application)
            .filter(|&&application| matches!(kinds[application], CheckedNodeKind::Expression(_)))
        else {
            return Err(refuse(
                position,
                application_at,
                Cause::InvalidValue,
                Some(&node.node_id),
                meter,
            ));
        };
        let scalar_application = matches!(
            terms.get(application_position),
            Some(CheckedTerm::Application { operation, .. })
                if operation.identity().is_some_and(is_scalar_operation)
        );
        let occurrence_present = nodes[application_position]
            .occurrences
            .iter()
            .any(|occurrence| {
                occurrence.role == CheckedOccurrenceRole::Expression
                    && occurrence.ordinal == record.occurrence
            });
        if !scalar_application || !occurrence_present {
            return Err(refuse(
                position,
                if scalar_application {
                    node_at.key("body").key("members").index(2)
                } else {
                    application_at
                },
                Cause::InvalidValue,
                Some(&node.node_id),
                meter,
            ));
        }
        if owner_paths.is_none() {
            owner_paths = Some(derive_package_path_conditions(
                nodes,
                kinds,
                terms,
                index,
                &frame_index,
                source_map,
                position,
                meter,
            )?);
        }
        let expected_path = owner_paths.as_ref().and_then(|paths| {
            paths
                .binary_search_by_key(
                    &(owner_position, application_position, record.occurrence),
                    |path| (path.owner, path.application, path.occurrence),
                )
                .ok()
                .map(|path| paths[path].terminal)
        });
        if expected_path.is_none() {
            return Err(refuse(
                position,
                node_at.key("body").key("members").index(5),
                Cause::InvalidValue,
                nodes.get(position).map(|node| &node.node_id),
                meter,
            ));
        }
        if expected_path != Some(record.path_condition_frame) {
            return Err(refuse(
                position,
                node_at.key("body").key("members").index(5),
                Cause::InvalidValue,
                Some(&node.node_id),
                meter,
            ));
        }
        if identity_path_terminal(&node.node_id)
            != record.path_condition_frame.map(|id| id.0.to_string())
        {
            let at = node_at.key("node_id");
            return Err(stage_storage::refuse(
                meter,
                &site(position, at),
                Refusal::new(Code::InvalidSemanticGraph, at),
            ));
        }
        let bound_at = node_at.key("body").key("members").index(3);
        let valid_bound = if let Some(&bound) = index.get(&record.result_bound) {
            is_type_kind(kinds[bound])
        } else {
            terms
                .get(position)
                .and_then(|term| binding(term, 3))
                .is_some_and(|term| match term {
                    CheckedTerm::DependencyReference { package, node } => evidence
                        .dependency_package(package)
                        .and_then(|dependency| dependency.node_kind(node))
                        .is_some_and(is_type_kind),
                    _ => false,
                })
        };
        if !valid_bound {
            return Err(refuse(
                position,
                bound_at,
                Cause::InvalidValue,
                Some(&node.node_id),
                meter,
            ));
        }
        if let Some((narrow_id, narrow_occurrence)) = &record.conversion {
            let valid_conversion = index.get(narrow_id).is_some_and(|&narrow_position| {
                let wraps_application = matches!(
                    terms.get(narrow_position),
                    Some(CheckedTerm::Application { operation, arguments, .. })
                        if operation.identity() == Some("filament.op.numeric.narrow")
                            && operation.member_key("declaration") == Some(&record.result_bound)
                            && arguments.iter().any(|argument| reference(argument).as_ref() == Some(&record.application))
                );
                wraps_application
                    && nodes[narrow_position].occurrences.iter().any(|occurrence| {
                        occurrence.role == CheckedOccurrenceRole::Expression
                            && occurrence.ordinal == *narrow_occurrence
                    })
            });
            if !valid_conversion {
                return Err(refuse(
                    position,
                    node_at.key("body").key("members").index(4),
                    Cause::InvalidValue,
                    Some(&node.node_id),
                    meter,
                ));
            }
        }
        for (member, ids) in [
            (6, record.extent_roots.as_slice()),
            (7, record.objects.as_slice()),
        ] {
            let mut previous_id: Option<&CheckedNodeId> = None;
            for (entry, id) in ids.iter().enumerate() {
                let at = node_at
                    .key("body")
                    .key("members")
                    .index(member)
                    .key("value")
                    .key("members")
                    .index(entry);
                let target = index.get(id);
                let valid = target.is_some_and(|&target| {
                    if member == 6 {
                        kinds[target] == CheckedNodeKind::Value(ValueForm::Parameter)
                    } else {
                        matches!(kinds[target], CheckedNodeKind::Model(ModelForm::ObjectType))
                    }
                });
                if !valid || previous_id.is_some_and(|previous| previous >= id) {
                    return Err(refuse(
                        position,
                        at,
                        Cause::InvalidValue,
                        Some(&node.node_id),
                        meter,
                    ));
                }
                previous_id = Some(id);
            }
        }
        if record.path_condition_frame.is_some_and(|frame| {
            frame.0 >= path_condition_frames.map_or(0, <[PathConditionFrameWire]>::len)
        }) {
            return Err(refuse(
                position,
                node_at.key("body").key("members").index(5),
                Cause::InvalidValue,
                Some(&node.node_id),
                meter,
            ));
        }
        let mut frame_at = record.path_condition_frame;
        while let Some(frame_id) = frame_at {
            let Some(frame) = path_condition_frames.and_then(|frames| frames.get(frame_id.0))
            else {
                break;
            };
            used_frames[frame_id.0] = true;
            frame_at = frame
                .parent
                .and_then(|parent| usize::try_from(parent).ok())
                .map(PathConditionFrameId);
        }
    }
    let Some(rows) = rows else {
        if let Some(&position) = records.first().or_else(|| operation_records.first()) {
            let at = root();
            return Err(refuse(
                0,
                at,
                Cause::InvalidValue,
                Some(&nodes[position].node_id),
                meter,
            ));
        }
        return Ok(());
    };
    if rows.is_empty() {
        return Err(stage_storage::refuse(
            meter,
            &site(0, root()),
            Refusal::new(Code::MalformedWire, root()),
        ));
    }
    let mut previous: Option<RowKey<'_>> = None;
    let mut named_records: Vec<(usize, usize, CheckedNodeId)> = Vec::new();
    let mut named_operation_records = Vec::new();
    for (row_index, row) in rows.iter().enumerate() {
        let at = root().index(row_index);
        stage_storage::work(meter, &site(row_index, at), 1)?;
        let Some(&claim_position) = index.get(&row.claim.node) else {
            return Err(refuse(
                row_index,
                at.key("claim"),
                Cause::InvalidValue,
                Some(&row.claim.node),
                meter,
            ));
        };
        if !nodes[claim_position]
            .occurrences
            .iter()
            .any(|occurrence| occurrence == &row.claim.occurrence)
        {
            return Err(refuse(
                row_index,
                at.key("claim"),
                Cause::InvalidValue,
                Some(&row.claim.node),
                meter,
            ));
        }
        if let RequirementExtentWire::Unbounded { domains } = &row.extent {
            if domains.is_empty() {
                return Err(stage_storage::refuse(
                    meter,
                    &site(row_index, at.key("extent")),
                    Refusal::new(Code::MalformedWire, at.key("extent")),
                ));
            }
            let mut prior_domain: Option<&DomainKey> = None;
            for (domain_index, entry) in domains.iter().enumerate() {
                let domain_at = at.key("extent").key("domains").index(domain_index);
                stage_storage::work(meter, &site(row_index, domain_at), 1)?;
                let target = match &entry.domain {
                    DomainKey::Node { node, .. } => node,
                    DomainKey::Population { member_type, .. } => member_type,
                };
                if index.get(target).is_none()
                    || prior_domain.is_some_and(|prior| prior >= &entry.domain)
                {
                    return Err(refuse(
                        row_index,
                        domain_at,
                        Cause::InvalidValue,
                        Some(target),
                        meter,
                    ));
                }
                prior_domain = Some(&entry.domain);
            }
        }
        if row.kind == CapabilityKind::ValueValidity {
            let Some(record_id) = row.record.as_ref() else {
                return Err(refuse(
                    row_index,
                    at,
                    Cause::InvalidValue,
                    Some(&row.claim.node),
                    meter,
                ));
            };
            let Some(&record_position) = index.get(record_id) else {
                return Err(refuse(
                    row_index,
                    at.key("record"),
                    Cause::InvalidValue,
                    Some(record_id),
                    meter,
                ));
            };
            if kinds[record_position] != CheckedNodeKind::Claim(super::ClaimForm::ValueValidity) {
                return Err(refuse(
                    row_index,
                    at.key("record"),
                    Cause::InvalidValue,
                    Some(record_id),
                    meter,
                ));
            }
            let Some(record) = terms
                .get(record_position)
                .and_then(|term| read_record(record_id.clone(), term))
            else {
                return Err(refuse(
                    row_index,
                    at.key("record"),
                    Cause::InvalidValue,
                    Some(record_id),
                    meter,
                ));
            };
            if record.application != row.claim.node
                || row.claim.occurrence.role != CheckedOccurrenceRole::Expression
                || record.occurrence != row.claim.occurrence.ordinal
            {
                return Err(refuse(
                    row_index,
                    at.key("record"),
                    Cause::InvalidValue,
                    Some(record_id),
                    meter,
                ));
            }
            if let RequirementExtentWire::Unbounded { domains } = &row.extent {
                for (domain_index, entry) in domains.iter().enumerate() {
                    if match &entry.domain {
                        DomainKey::Population { .. } => true,
                        DomainKey::Node { node, .. } => !record.extent_roots.contains(node),
                    } {
                        return Err(refuse(
                            row_index,
                            at.key("extent").key("domains").index(domain_index),
                            Cause::InvalidValue,
                            Some(record_id),
                            meter,
                        ));
                    }
                }
            }
            named_records.push((row_index, record_position, record.owner));
        } else if row.kind == CapabilityKind::OperationContract {
            let Some(record_id) = row.record.as_ref() else {
                return Err(refuse(
                    row_index,
                    at,
                    Cause::InvalidValue,
                    Some(&row.claim.node),
                    meter,
                ));
            };
            let Some(&record_position) = index.get(record_id) else {
                return Err(refuse(
                    row_index,
                    at.key("record"),
                    Cause::InvalidValue,
                    Some(record_id),
                    meter,
                ));
            };
            if kinds[record_position] != CheckedNodeKind::Claim(super::ClaimForm::OperationContract)
            {
                return Err(refuse(
                    row_index,
                    at.key("record"),
                    Cause::InvalidValue,
                    Some(record_id),
                    meter,
                ));
            }
            let Some((claim, occurrence)) = terms
                .get(record_position)
                .and_then(read_operation_contract_join)
            else {
                return Err(refuse(
                    row_index,
                    at.key("record"),
                    Cause::InvalidValue,
                    Some(record_id),
                    meter,
                ));
            };
            if claim != row.claim.node || occurrence != row.claim.occurrence {
                return Err(refuse(
                    row_index,
                    at.key("record"),
                    Cause::InvalidValue,
                    Some(record_id),
                    meter,
                ));
            }
            let Some(expected_path) = operation_contract_identity_path(
                record_position,
                &terms[record_position],
                &row.extent,
                nodes,
                operand_rows,
                index,
                evidence,
            ) else {
                return Err(refuse(
                    row_index,
                    at.key("extent"),
                    Cause::InvalidValue,
                    Some(record_id),
                    meter,
                ));
            };
            if nodes[record_position].node_id.path != expected_path {
                return Err(refuse(
                    row_index,
                    at.key("extent"),
                    Cause::InvalidValue,
                    Some(record_id),
                    meter,
                ));
            }
            let record_occurrence = &nodes[record_position].occurrences[0];
            let record_regions = source_map.iter().find(|entry| {
                entry.node_id == *record_id
                    && entry.role == record_occurrence.role
                    && entry.ordinal == record_occurrence.ordinal
            });
            let claim_regions = source_map.iter().find(|entry| {
                entry.node_id == row.claim.node
                    && entry.role == row.claim.occurrence.role
                    && entry.ordinal == row.claim.occurrence.ordinal
            });
            if !record_regions
                .zip(claim_regions)
                .is_some_and(|(record, claim)| record.regions == claim.regions)
            {
                let source_at = Path::root()
                    .key("semantic_graph")
                    .key("nodes")
                    .index(record_position)
                    .key("occurrences");
                return Err(refuse(
                    record_position,
                    source_at,
                    Cause::InvalidValue,
                    Some(record_id),
                    meter,
                ));
            }
            named_operation_records.push(record_position);
        } else if row.record.is_some() {
            return Err(refuse(
                row_index,
                at.key("record"),
                Cause::InvalidValue,
                Some(&row.claim.node),
                meter,
            ));
        }
        let key: RowKey<'_> = (
            index.resolved_at(claim_position, structural_keys),
            occurrence_role_wire(&row.claim.occurrence.role),
            row.claim.occurrence.ordinal,
            capability_wire(row.kind),
        );
        let order = match &previous {
            Some(prior) => Some(row_key_order(prior, &key, meter)?),
            None => None,
        };
        if let Some(order) = order.filter(|order| order.is_ge()) {
            let cause = if order.is_eq() {
                Cause::ConflictingDefinition
            } else {
                Cause::InvalidSemanticGraph
            };
            return Err(refuse(row_index, at, cause, Some(&row.claim.node), meter));
        }
        previous = Some(key);
    }
    let mut named_record_positions: Vec<usize> = named_records
        .iter()
        .map(|(_, record_position, _)| *record_position)
        .collect();
    named_record_positions.extend(named_operation_records.iter().copied());
    named_record_positions.sort_unstable();
    named_record_positions.dedup();
    if let Some(position) = first_unnamed_position(&records, &named_record_positions, &mut || {}) {
        return Err(refuse(
            0,
            super::stage_storage::Path::root()
                .key("semantic_graph")
                .key("nodes")
                .index(position)
                .key("body"),
            Cause::InvalidValue,
            Some(&nodes[position].node_id),
            meter,
        ));
    }
    for (expected, &record_position) in named_operation_records.iter().enumerate() {
        let expected = u64::try_from(expected).unwrap_or(u64::MAX);
        if nodes[record_position].occurrences[0].ordinal != expected {
            let node_at = Path::root()
                .key("semantic_graph")
                .key("nodes")
                .index(record_position)
                .key("occurrences");
            return Err(refuse(
                record_position,
                node_at,
                Cause::InvalidValue,
                Some(&nodes[record_position].node_id),
                meter,
            ));
        }
    }
    let order = named_record_order(&named_records, index, structural_keys, meter)?;
    let mut previous_owner: Option<CheckedNodeId> = None;
    let mut owner_ordinal = 0_u64;
    for at in order {
        let (_, record_position, owner) = &named_records[at];
        let record_position = *record_position;
        if previous_owner.as_ref() != Some(owner) {
            previous_owner = Some(owner.clone());
            owner_ordinal = 0;
        }
        if nodes[record_position].occurrences[0].ordinal != owner_ordinal {
            let at = Path::root()
                .key("semantic_graph")
                .key("nodes")
                .index(record_position)
                .key("occurrences");
            return Err(refuse(
                record_position,
                at,
                Cause::InvalidValue,
                Some(&nodes[record_position].node_id),
                meter,
            ));
        }
        owner_ordinal = owner_ordinal.saturating_add(1);
    }
    if let Some((frame_index, _)) = used_frames.iter().enumerate().find(|(_, used)| !**used) {
        let at = Path::root().key("path_condition_frames").index(frame_index);
        return Err(stage_storage::refuse(
            meter,
            &Site::new(
                PackageReadStage::Graph,
                frame_index,
                "path_condition_frames",
                at,
            ),
            Refusal::new(Code::InvalidPackage, at).because(Cause::InvalidValue),
        ));
    }
    Ok(())
}

fn validate_frames(
    frames: Option<&[PathConditionFrameWire]>,
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &GraphIndex<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let Some(frames) = frames else {
        return Ok(());
    };
    if frames.is_empty() {
        return Err(stage_storage::refuse(
            meter,
            &Site::new(
                PackageReadStage::Graph,
                0,
                "path_condition_frames",
                Path::root().key("path_condition_frames"),
            ),
            Refusal::new(
                Code::InvalidPackage,
                Path::root().key("path_condition_frames"),
            )
            .because(Cause::InvalidValue),
        ));
    }
    let mut seen = BTreeSet::new();
    for (frame_index, frame) in frames.iter().enumerate() {
        let at = Path::root().key("path_condition_frames").index(frame_index);
        stage_storage::work(
            meter,
            &Site::new(
                PackageReadStage::Graph,
                frame_index,
                "path_condition_frames",
                at,
            ),
            1,
        )?;
        if frame.parent.is_some_and(|parent| {
            usize::try_from(parent).map_or(true, |parent| parent >= frame_index)
        }) {
            let pointer = at.key("parent");
            return Err(stage_storage::refuse(
                meter,
                &Site::new(
                    PackageReadStage::Graph,
                    frame_index,
                    "path_condition_frames",
                    pointer,
                ),
                Refusal::new(Code::InvalidPackage, pointer).because(Cause::InvalidValue),
            ));
        }
        if frame.guard.occurrence.role != CheckedOccurrenceRole::Expression {
            return Err(stage_storage::refuse(
                meter,
                &Site::new(
                    PackageReadStage::Graph,
                    frame_index,
                    "path_condition_frames",
                    at,
                ),
                Refusal::new(Code::MalformedWire, at).because(Cause::WrongValueKind),
            ));
        }
        let Some(&guard_position) = index.get(&frame.guard.node) else {
            let pointer = at.key("guard");
            return Err(stage_storage::refuse(
                meter,
                &Site::new(
                    PackageReadStage::Graph,
                    frame_index,
                    "path_condition_frames",
                    pointer,
                ),
                Refusal::new(Code::InvalidPackage, pointer).because(Cause::InvalidValue),
            ));
        };
        let guard_kind = kinds.get(guard_position);
        let boolean_type = index
            .get(&nodes[guard_position].semantic_type)
            .and_then(|position| kinds.get(*position))
            == Some(&CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean));
        let guard_occurrence = frame.guard.occurrence.role == CheckedOccurrenceRole::Expression
            && nodes[guard_position].occurrences.iter().any(|occurrence| {
                occurrence.role == CheckedOccurrenceRole::Expression
                    && occurrence.ordinal == frame.guard.occurrence.ordinal
            });
        if !matches!(
            guard_kind,
            Some(
                CheckedNodeKind::Expression(_)
                    | CheckedNodeKind::Value(ValueForm::Literal | ValueForm::Parameter)
            )
        ) || !boolean_type
            || !guard_occurrence
        {
            let pointer = at.key("guard");
            return Err(stage_storage::refuse(
                meter,
                &Site::new(
                    PackageReadStage::Graph,
                    frame_index,
                    "path_condition_frames",
                    pointer,
                ),
                Refusal::new(Code::InvalidPackage, pointer).because(Cause::InvalidValue),
            ));
        }
        if !seen.insert((
            frame.parent,
            frame.guard.node.clone(),
            frame.guard.occurrence.ordinal,
            frame.holds,
        )) {
            return Err(stage_storage::refuse(
                meter,
                &Site::new(
                    PackageReadStage::Graph,
                    frame_index,
                    "path_condition_frames",
                    at,
                ),
                Refusal::new(Code::InvalidPackage, at).because(Cause::ConflictingDefinition),
            ));
        }
    }
    Ok(())
}

fn is_type_kind(kind: CheckedNodeKind) -> bool {
    matches!(
        kind,
        CheckedNodeKind::ScalarType(_)
            | CheckedNodeKind::CompositeType(_)
            | CheckedNodeKind::BoundedDomain(_)
    )
}

fn validate_operation_contract_node(
    position: usize,
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[CheckedTerm],
    index: &GraphIndex<'_>,
    operand_rows: &[Vec<CheckedOperandEntry>],
    evidence: &super::CheckedPackageEvidence,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let node = &nodes[position];
    let node_at = Path::root()
        .key("semantic_graph")
        .key("nodes")
        .index(position);
    stage_storage::work(meter, &site(position, node_at), 1)?;
    if node.declaration.is_some()
        || node.occurrences.len() != 1
        || node.occurrences[0].role != CheckedOccurrenceRole::Generated
    {
        return Err(refuse(
            position,
            node_at.key("occurrences"),
            Cause::InvalidValue,
            Some(&node.node_id),
            meter,
        ));
    }
    if index.get(&node.semantic_type).is_none_or(|&type_position| {
        kinds[type_position] != CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean)
    }) {
        return Err(refuse(
            position,
            node_at.key("semantic_type"),
            Cause::InvalidValue,
            Some(&node.node_id),
            meter,
        ));
    }
    let Some(term) = terms.get(position) else {
        return Err(refuse(
            position,
            node_at.key("body"),
            Cause::InvalidValue,
            Some(&node.node_id),
            meter,
        ));
    };
    if !operation_contract_binding_shape(term) {
        return Err(refuse(
            position,
            node_at.key("body"),
            Cause::InvalidValue,
            Some(&node.node_id),
            meter,
        ));
    }
    let Some(record) = read_operation_contract_record(node.node_id.clone(), term) else {
        return Err(refuse(
            position,
            node_at.key("body"),
            Cause::InvalidValue,
            Some(&node.node_id),
            meter,
        ));
    };
    let Some(path_prefix) = operation_contract_identity_prefix(
        position,
        term,
        &record,
        nodes,
        operand_rows,
        index,
        evidence,
    ) else {
        return Err(refuse(
            position,
            node_at.key("node_id").key("path"),
            Cause::InvalidSemanticGraph,
            Some(&node.node_id),
            meter,
        ));
    };
    if node.node_id.path.get(..path_prefix.len()) != Some(path_prefix.as_slice())
        || !matches!(
            node.node_id.path.get(path_prefix.len()),
            Some(CheckedNodePathSegment::Name(segment)) if segment.as_ref() == "extent"
        )
    {
        return Err(refuse(
            position,
            node_at.key("node_id").key("path"),
            Cause::InvalidSemanticGraph,
            Some(&node.node_id),
            meter,
        ));
    }
    let Some((claim, occurrence)) = read_operation_contract_join(term) else {
        return Err(refuse(
            position,
            node_at.key("body").key("members").index(0),
            Cause::InvalidValue,
            Some(&node.node_id),
            meter,
        ));
    };
    let Some(&claim_position) = index.get(&claim) else {
        return Err(refuse(
            position,
            node_at.key("body").key("members").index(0),
            Cause::InvalidValue,
            Some(&node.node_id),
            meter,
        ));
    };
    if !matches!(
        kinds[claim_position],
        CheckedNodeKind::State(super::StateForm::StateClause | super::StateForm::Frame)
    ) || !nodes[claim_position]
        .occurrences
        .iter()
        .any(|candidate| candidate == &occurrence)
    {
        return Err(refuse(
            position,
            node_at.key("body").key("members").index(0),
            Cause::InvalidValue,
            Some(&node.node_id),
            meter,
        ));
    }
    if !binding(term, 2).is_some_and(|bound| {
        reference_is_kind(
            bound,
            CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean),
            kinds,
            index,
            evidence,
        )
    }) {
        return Err(refuse(
            position,
            node_at.key("body").key("members").index(2),
            Cause::InvalidValue,
            Some(&node.node_id),
            meter,
        ));
    }
    if record.operation.is_some()
        && !binding(term, 4).is_some_and(|operation| {
            reference_is_kind(
                operation,
                CheckedNodeKind::State(super::StateForm::OperationAnchor),
                kinds,
                index,
                evidence,
            )
        })
    {
        return Err(refuse(
            position,
            node_at.key("body").key("members").index(4),
            Cause::InvalidValue,
            Some(&node.node_id),
            meter,
        ));
    }
    let mut previous_object: Option<&CheckedNodeId> = None;
    for (object_index, object) in record.model_objects.iter().enumerate() {
        let valid = binding(term, 5)
            .and_then(|term| match term {
                CheckedTerm::Aggregate { members, .. } => members.get(object_index),
                _ => None,
            })
            .is_some_and(|object_term| {
                reference_is_kind(
                    object_term,
                    CheckedNodeKind::Model(ModelForm::ObjectType),
                    kinds,
                    index,
                    evidence,
                )
            });
        if !valid || previous_object.is_some_and(|previous| previous >= object) {
            return Err(refuse(
                position,
                node_at
                    .key("body")
                    .key("members")
                    .index(5)
                    .key("value")
                    .key("members")
                    .index(object_index),
                Cause::InvalidValue,
                Some(&node.node_id),
                meter,
            ));
        }
        previous_object = Some(object);
    }
    let mut previous_population: Option<(&CheckedNodeId, u64)> = None;
    for (population_index, population) in record.populations.iter().enumerate() {
        let valid = binding(term, 6)
            .and_then(|term| match term {
                CheckedTerm::Aggregate { members, .. } => members.get(population_index),
                _ => None,
            })
            .and_then(|term| match term {
                CheckedTerm::Aggregate { members, .. } => {
                    named_value(members.first()?, "member_type")
                }
                _ => None,
            })
            .is_some_and(|member_term| {
                reference_is_kind(
                    member_term,
                    CheckedNodeKind::Model(ModelForm::ObjectType),
                    kinds,
                    index,
                    evidence,
                )
            });
        let key = (&population.member_type, population.ordinal);
        if !valid || previous_population.is_some_and(|previous| previous >= key) {
            return Err(refuse(
                position,
                node_at
                    .key("body")
                    .key("members")
                    .index(6)
                    .key("value")
                    .key("members")
                    .index(population_index),
                Cause::InvalidValue,
                Some(&node.node_id),
                meter,
            ));
        }
        previous_population = Some(key);
    }
    Ok(())
}

fn operation_contract_binding_shape(term: &CheckedTerm) -> bool {
    let CheckedTerm::Aggregate { members, .. } = term else {
        return false;
    };
    const NAMES: [&str; 7] = [
        "claim",
        "occurrence",
        "result_bound",
        "invariant",
        "operation",
        "model_objects",
        "populations",
    ];
    members.len() == NAMES.len()
        && members.iter().zip(NAMES).all(|(member, expected)| {
            matches!(member, CheckedTerm::Binding { name, .. }
                if matches!(name.kind(), super::CheckedBindingNameKind::Other(actual) if actual == expected))
        })
}

fn operation_contract_identity_prefix(
    holder: usize,
    term: &CheckedTerm,
    record: &OperationContractRecord,
    nodes: &[RawSemanticNodeV2],
    operand_rows: &[Vec<CheckedOperandEntry>],
    index: &GraphIndex<'_>,
    evidence: &super::CheckedPackageEvidence,
) -> Option<Vec<CheckedNodePathSegment>> {
    let row = operand_rows.get(holder)?;
    let mut path = vec![path_name("claim"), path_name("operation_contract")];
    path.push(path_operand(
        operand_slot_for_term(binding(term, 0)?, row, nodes, index, evidence)?,
        row,
    )?);
    let (_, occurrence) = read_operation_contract_join(term)?;
    path.push(path_name(occurrence_role_wire(&occurrence.role)));
    path.push(path_number(occurrence.ordinal));
    path.push(path_name("bound"));
    path.push(path_operand(
        operand_slot_for_term(binding(term, 2)?, row, nodes, index, evidence)?,
        row,
    )?);
    path.push(path_name("invariant"));
    path.push(path_name(if record.invariant { "true" } else { "false" }));
    path.push(path_name("operation"));
    if record.operation.is_some() {
        path.push(path_operand(
            operand_slot_for_term(binding(term, 4)?, row, nodes, index, evidence)?,
            row,
        )?);
    } else {
        path.push(path_name("none"));
    }
    path.push(path_name("objects"));
    path.push(path_number(u64::try_from(record.model_objects.len()).ok()?));
    let CheckedTerm::Aggregate {
        members: object_terms,
        ..
    } = binding(term, 5)?
    else {
        return None;
    };
    if object_terms.len() != record.model_objects.len() {
        return None;
    }
    for object in object_terms {
        path.push(path_operand(
            operand_slot_for_term(object, row, nodes, index, evidence)?,
            row,
        )?);
    }
    path.push(path_name("populations"));
    path.push(path_number(u64::try_from(record.populations.len()).ok()?));
    let CheckedTerm::Aggregate {
        members: population_terms,
        ..
    } = binding(term, 6)?
    else {
        return None;
    };
    if population_terms.len() != record.populations.len() {
        return None;
    }
    for (population_term, population) in population_terms.iter().zip(&record.populations) {
        let CheckedTerm::Aggregate { members, .. } = population_term else {
            return None;
        };
        path.push(path_name("population"));
        path.push(path_operand(
            operand_slot_for_term(
                named_value(members.first()?, "member_type")?,
                row,
                nodes,
                index,
                evidence,
            )?,
            row,
        )?);
        path.push(path_number(population.ordinal));
        path.push(match population.maximum {
            Some(maximum) => path_number(maximum),
            None => path_name("none"),
        });
    }
    Some(path)
}

fn operation_contract_identity_path(
    holder: usize,
    term: &CheckedTerm,
    extent: &RequirementExtentWire,
    nodes: &[RawSemanticNodeV2],
    operand_rows: &[Vec<CheckedOperandEntry>],
    index: &GraphIndex<'_>,
    evidence: &super::CheckedPackageEvidence,
) -> Option<Vec<CheckedNodePathSegment>> {
    let record = read_operation_contract_record(nodes.get(holder)?.node_id.clone(), term)?;
    let mut path = operation_contract_identity_prefix(
        holder,
        term,
        &record,
        nodes,
        operand_rows,
        index,
        evidence,
    )?;
    path.push(path_name("extent"));
    match extent {
        RequirementExtentWire::Bounded => path.push(path_name("bounded")),
        RequirementExtentWire::Unbounded { domains } => {
            path.push(path_name("unbounded"));
            path.push(path_number(u64::try_from(domains.len()).ok()?));
            let row = operand_rows.get(holder)?;
            for entry in domains {
                match &entry.domain {
                    DomainKey::Node {
                        node,
                        path: indices,
                    } => {
                        path.push(path_name("node"));
                        path.push(path_operand(
                            operand_slot_for_id(node, row, nodes, index, evidence)?,
                            row,
                        )?);
                        path.push(path_number(u64::try_from(indices.len()).ok()?));
                        path.extend(
                            indices
                                .iter()
                                .copied()
                                .map(|value| path_number(u64::from(value))),
                        );
                    }
                    DomainKey::Population {
                        member_type,
                        ordinal,
                    } => {
                        path.push(path_name("population"));
                        path.push(path_operand(
                            operand_slot_for_id(member_type, row, nodes, index, evidence)?,
                            row,
                        )?);
                        path.push(path_number(u64::from(*ordinal)));
                    }
                }
                path.push(path_name(domain_kind_wire(entry.kind)));
            }
        }
    }
    Some(path)
}

fn operand_slot_for_term(
    term: &CheckedTerm,
    row: &[CheckedOperandEntry],
    nodes: &[RawSemanticNodeV2],
    index: &GraphIndex<'_>,
    evidence: &super::CheckedPackageEvidence,
) -> Option<u64> {
    match term {
        CheckedTerm::Reference { target } => {
            let target_position = index.get(target)?;
            let target_position = u64::try_from(*target_position).ok()?;
            row.iter()
                .position(|entry| *entry == CheckedOperandEntry::Node(target_position))
                .and_then(|slot| u64::try_from(slot).ok())
        }
        CheckedTerm::DependencyReference { package, node } => {
            let dependency = evidence.dependency_package(package)?;
            let target_position = u64::try_from(dependency.node_position(node)?).ok()?;
            row.iter()
                .position(|entry| {
                    matches!(entry,
                        CheckedOperandEntry::Dependency(operand)
                            if operand.package.as_ref() == package.as_ref()
                                && operand.node == target_position)
                })
                .and_then(|slot| u64::try_from(slot).ok())
        }
        _ => {
            let target = reference(term)?;
            operand_slot_for_id(&target, row, nodes, index, evidence)
        }
    }
}

fn operand_slot_for_id(
    target: &CheckedNodeId,
    row: &[CheckedOperandEntry],
    nodes: &[RawSemanticNodeV2],
    index: &GraphIndex<'_>,
    evidence: &super::CheckedPackageEvidence,
) -> Option<u64> {
    if let Some(&position) = index.get(target) {
        let position = u64::try_from(position).ok()?;
        return row
            .iter()
            .position(|entry| *entry == CheckedOperandEntry::Node(position))
            .and_then(|slot| u64::try_from(slot).ok());
    }
    let _ = nodes;
    for (slot, entry) in row.iter().enumerate() {
        let CheckedOperandEntry::Dependency(operand) = entry else {
            continue;
        };
        let position = usize::try_from(operand.node).ok()?;
        let dependency = evidence.dependency_package(&operand.package)?;
        if dependency.document_held_id(position) == Some(target) {
            return u64::try_from(slot).ok();
        }
    }
    None
}

fn path_name(value: impl Into<Box<str>>) -> CheckedNodePathSegment {
    CheckedNodePathSegment::Name(value.into())
}

fn path_operand(slot: u64, row: &[CheckedOperandEntry]) -> Option<CheckedNodePathSegment> {
    match row.get(usize::try_from(slot).ok()?)? {
        CheckedOperandEntry::Node(position) => Some(CheckedNodePathSegment::Index(*position)),
        CheckedOperandEntry::Dependency(operand) => Some(CheckedNodePathSegment::Dependency {
            package: operand.package.clone(),
            node: operand.node,
        }),
    }
}

fn path_number(value: u64) -> CheckedNodePathSegment {
    path_name(value.to_string().into_boxed_str())
}

fn read_operation_contract_join(
    term: &CheckedTerm,
) -> Option<(CheckedNodeId, super::CheckedOccurrence)> {
    let claim = reference(binding(term, 0)?)?;
    let CheckedTerm::Aggregate { members, .. } = binding(term, 1)? else {
        return None;
    };
    const OCCURRENCE_NAMES: [&str; 2] = ["role", "ordinal"];
    if members.len() != OCCURRENCE_NAMES.len()
        || !members.iter().zip(OCCURRENCE_NAMES).all(|(member, expected)| {
            matches!(member, CheckedTerm::Binding { name, .. }
                if matches!(name.kind(), super::CheckedBindingNameKind::Other(actual) if actual == expected))
        })
    {
        return None;
    }
    let role = named_value(members.first()?, "role")
        .and_then(string)
        .and_then(occurrence_role_from_wire)?;
    let ordinal = named_value(members.get(1)?, "ordinal").and_then(integer)?;
    Some((claim, super::CheckedOccurrence { role, ordinal }))
}

/// The occurrence role a record's `role` literal spells.
#[ix_trace_rs::string_edge]
fn occurrence_role_from_wire(text: &str) -> Option<CheckedOccurrenceRole> {
    match text {
        "declaration" => Some(CheckedOccurrenceRole::Declaration),
        "type" => Some(CheckedOccurrenceRole::Type),
        "expression" => Some(CheckedOccurrenceRole::Expression),
        "anchor" => Some(CheckedOccurrenceRole::Anchor),
        "claim" => Some(CheckedOccurrenceRole::Claim),
        "generated" => Some(CheckedOccurrenceRole::Generated),
        _ => None,
    }
}

enum OwnerWalkTask<'a> {
    /// The walk has finished the body of the node at this position.
    Leave(usize),
    Node(usize, Option<PathConditionFrameId>),
    Term(&'a CheckedTerm, Option<PathConditionFrameId>),
}

type PathConditionFrameIndexEntry = ((Option<usize>, usize, u64, bool), usize);

fn path_condition_frame_index(
    frames: Option<&[PathConditionFrameWire]>,
    index: &GraphIndex<'_>,
    meter: &mut WorkMeter<'_>,
) -> Result<Vec<PathConditionFrameIndexEntry>, ValidationFailure> {
    let frames = frames.unwrap_or_default();
    let at = Path::root().key("path_condition_frames");
    let site = Site::new(PackageReadStage::Graph, 0, "path_condition_frames", at);
    let mut entries = stage_storage::vec(meter, &site, frames.len())?;
    for (frame_id, frame) in frames.iter().enumerate() {
        let guard_position = index
            .get_charged(&frame.guard.node, meter)?
            .ok_or_else(|| {
                refuse(
                    frame_id,
                    Path::root()
                        .key("path_condition_frames")
                        .index(frame_id)
                        .key("guard"),
                    Cause::InvalidValue,
                    Some(&frame.guard.node),
                    meter,
                )
            })?;
        entries.push((
            (
                frame.parent.and_then(|parent| usize::try_from(parent).ok()),
                guard_position,
                frame.guard.occurrence.ordinal,
                frame.holds,
            ),
            frame_id,
        ));
    }
    entries.sort_unstable();
    Ok(entries)
}

#[derive(Clone, Copy)]
struct DerivedPath {
    owner: usize,
    application: usize,
    occurrence: u64,
    terminal: Option<PathConditionFrameId>,
}

/// The terminal a branch carries when its frame is absent from the table: the
/// emitter prunes a frame no record cites (FR-825), so a branch with no
/// scalar-operation record has none. No record can cite this id.
const PRUNED_FRAME: PathConditionFrameId = PathConditionFrameId(usize::MAX);

/// Derive the path condition of every scalar-operation occurrence in the
/// package. Expression occurrences are numbered package-wide (FR-825): the
/// owners are walked in declaration order, one count of visits per node
/// carried across them, each owner's body before its measure.
fn derive_package_path_conditions(
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[CheckedTerm],
    index: &GraphIndex<'_>,
    frame_index: &[PathConditionFrameIndexEntry],
    source_map: &[CheckedSourceMapEntry],
    record_position: usize,
    meter: &mut WorkMeter,
) -> Result<Vec<DerivedPath>, ValidationFailure> {
    let at = Path::root().key("semantic_graph").key("nodes");
    let owner_count = (0..nodes.len())
        .filter(|&position| {
            matches!(kinds[position], CheckedNodeKind::Function(_))
                && terms.get(position).and_then(function_body_root).is_some()
        })
        .count();
    let numbering_site = Site::new(
        PackageReadStage::Graph,
        record_position,
        "requirement_items",
        at,
    );
    let mut declared = stage_storage::vec::<Option<(&CheckedSourceRef, u64, u64)>>(
        meter,
        &numbering_site,
        nodes.len(),
    )?;
    declared.resize(nodes.len(), None);
    for entry in source_map {
        stage_storage::work(meter, &numbering_site, 1)?;
        if entry.role != CheckedOccurrenceRole::Declaration {
            continue;
        }
        let (Some(&position), Some(region)) = (index.get(&entry.node_id), entry.regions.first())
        else {
            continue;
        };
        let start = (&region.source, region.start, region.end);
        if declared[position].is_none_or(|known| start < known) {
            declared[position] = Some(start);
        }
    }
    let mut owners = stage_storage::vec::<(Option<(&CheckedSourceRef, u64, u64)>, usize)>(
        meter,
        &numbering_site,
        owner_count,
    )?;
    for position in 0..nodes.len() {
        if matches!(kinds[position], CheckedNodeKind::Function(_))
            && terms.get(position).and_then(function_body_root).is_some()
        {
            owners.push((declared[position], position));
        }
    }
    owners.sort_unstable_by(
        |(left_position, left_owner), (right_position, right_owner)| {
            left_position
                .cmp(right_position)
                .then_with(|| nodes[*left_owner].node_id.cmp(&nodes[*right_owner].node_id))
        },
    );
    let scalar_occurrence_count = terms
        .iter()
        .enumerate()
        .filter(|(_, term)| {
            matches!(term, CheckedTerm::Application { operation, .. }
                if operation.identity().is_some_and(is_scalar_operation))
        })
        .map(|(position, _)| {
            nodes[position]
                .occurrences
                .iter()
                .filter(|occurrence| occurrence.role == CheckedOccurrenceRole::Expression)
                .count()
        })
        .fold(0_usize, usize::saturating_add);
    // These flat tables are sized from admitted input counts before traversal;
    // no path, frame-order, or occurrence index grows with an uncharged map.
    let mut paths =
        stage_storage::vec::<DerivedPath>(meter, &numbering_site, scalar_occurrence_count)?;
    let mut visit_counts = stage_storage::vec::<usize>(meter, &numbering_site, nodes.len())?;
    visit_counts.resize(nodes.len(), 0);
    let mut charged_owner = stage_storage::vec::<usize>(meter, &numbering_site, nodes.len())?;
    charged_owner.resize(nodes.len(), usize::MAX);
    // The nodes whose bodies a walk is inside. A reference back to one is a
    // recursive call, which the walk does not enter again.
    let mut active = stage_storage::vec::<u8>(meter, &numbering_site, nodes.len())?;
    active.resize(nodes.len(), 0);
    let mut frame_seen = stage_storage::vec::<u8>(meter, &numbering_site, frame_index.len())?;
    frame_seen.resize(frame_index.len(), 0);
    let mut owner_frame_order =
        stage_storage::vec::<usize>(meter, &numbering_site, frame_index.len())?;
    let mut frame_orders =
        stage_storage::vec::<(usize, usize, usize)>(meter, &numbering_site, frame_index.len())?;
    let mut pending = stage_storage::vec::<OwnerWalkTask<'_>>(meter, &numbering_site, 1)?;
    for &(_, owner_position) in &owners {
        let Some(root) = terms.get(owner_position).and_then(function_body_root) else {
            continue;
        };
        let Some(&root_position) = index.get(root) else {
            continue;
        };
        owner_frame_order.clear();
        walk_owner_expression(
            root_position,
            owner_position,
            true,
            nodes,
            terms,
            index,
            frame_index,
            record_position,
            &mut visit_counts,
            &mut charged_owner,
            &mut active,
            &mut paths,
            &mut frame_seen,
            &mut owner_frame_order,
            &mut pending,
            &numbering_site,
            meter,
        )?;
        if let Some(measure) = function_measure_root(&terms[owner_position]) {
            if let Some(&measure_position) = index.get(measure) {
                walk_owner_expression(
                    measure_position,
                    owner_position,
                    false,
                    nodes,
                    terms,
                    index,
                    frame_index,
                    record_position,
                    &mut visit_counts,
                    &mut charged_owner,
                    &mut active,
                    &mut paths,
                    &mut frame_seen,
                    &mut owner_frame_order,
                    &mut pending,
                    &numbering_site,
                    meter,
                )?;
            }
        }
        frame_orders.extend(
            owner_frame_order
                .iter()
                .copied()
                .enumerate()
                .map(|(order, frame_id)| (owner_position, order, frame_id)),
        );
    }
    // The table's canonical order is owner structural key ascending, then
    // first visit; it is checked apart from the numbering order above.
    frame_orders.sort_unstable_by(|left, right| {
        nodes[left.0]
            .node_id
            .cmp(&nodes[right.0].node_id)
            .then_with(|| left.1.cmp(&right.1))
    });
    for (expected, (_, _, frame_id)) in frame_orders.iter().enumerate() {
        if *frame_id != expected {
            return Err(refuse(
                record_position,
                Path::root().key("path_condition_frames"),
                Cause::InvalidValue,
                None,
                meter,
            ));
        }
    }
    if frame_seen.contains(&0) {
        return Err(refuse(
            record_position,
            Path::root().key("path_condition_frames"),
            Cause::InvalidValue,
            None,
            meter,
        ));
    }
    paths.sort_unstable_by_key(|path| (path.owner, path.application, path.occurrence));
    Ok(paths)
}

#[allow(clippy::too_many_arguments)] // the walk state is charged and reused across owner bodies
fn walk_owner_expression<'a>(
    root: usize,
    owner: usize,
    derive: bool,
    nodes: &[RawSemanticNodeV2],
    terms: &'a [CheckedTerm],
    index: &GraphIndex<'_>,
    frame_index: &[PathConditionFrameIndexEntry],
    record_position: usize,
    visit_counts: &mut [usize],
    charged_owner: &mut [usize],
    active: &mut [u8],
    paths: &mut Vec<DerivedPath>,
    frame_seen: &mut [u8],
    frame_order: &mut Vec<usize>,
    pending: &mut Vec<OwnerWalkTask<'a>>,
    walk_site: &Site<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    // Reuse one charged stack across bodies; charge each capacity increase
    // before asking the allocator for it.
    pending.clear();
    push_owner_task(pending, OwnerWalkTask::Node(root, None), meter, walk_site)?;
    while let Some(task) = pending.pop() {
        match task {
            OwnerWalkTask::Leave(position) => {
                active[position] = 0;
            }
            OwnerWalkTask::Node(position, terminal) => {
                if active.get(position).is_none_or(|inside| *inside != 0) {
                    continue;
                }
                let Some(visit) = visit_counts.get_mut(position) else {
                    continue;
                };
                let occurrence = nodes[position]
                    .occurrences
                    .iter()
                    .filter(|occurrence| occurrence.role == CheckedOccurrenceRole::Expression)
                    .nth(*visit)
                    .map(|occurrence| occurrence.ordinal);
                *visit = visit.saturating_add(1);
                if charged_owner[position] != owner {
                    charged_owner[position] = owner;
                    let at = Path::root()
                        .key("semantic_graph")
                        .key("nodes")
                        .index(position);
                    stage_storage::work(meter, &site(position, at), 1)?;
                }
                let Some(term) = terms.get(position) else {
                    continue;
                };
                if let (true, Some(ordinal), CheckedTerm::Application { operation, .. }) =
                    (derive, occurrence, term)
                {
                    if operation.identity().is_some_and(is_scalar_operation) {
                        paths.push(DerivedPath {
                            owner,
                            application: position,
                            occurrence: ordinal,
                            terminal,
                        });
                    }
                }
                if let Some(inside) = active.get_mut(position) {
                    *inside = 1;
                }
                push_owner_task(pending, OwnerWalkTask::Leave(position), meter, walk_site)?;
                push_owner_task(
                    pending,
                    OwnerWalkTask::Term(term, terminal),
                    meter,
                    walk_site,
                )?;
            }
            OwnerWalkTask::Term(term, terminal) => match term {
                CheckedTerm::Reference { target } => {
                    if let Some(&position) = index.get(target) {
                        push_owner_task(
                            pending,
                            OwnerWalkTask::Node(position, terminal),
                            meter,
                            walk_site,
                        )?;
                    }
                }
                CheckedTerm::Application {
                    operation,
                    arguments,
                    ..
                } => {
                    let identity = operation.identity();
                    if derive && identity == Some("filament.op.control.if") && arguments.len() == 3
                    {
                        let guard = reference(&arguments[0]);
                        let Some((guard, occurrence)) = guard.and_then(|guard| {
                            let position = index.get(&guard)?;
                            let ordinal = nodes[*position]
                                .occurrences
                                .iter()
                                .filter(|occurrence| {
                                    occurrence.role == CheckedOccurrenceRole::Expression
                                })
                                .nth(*visit_counts.get(*position).unwrap_or(&0))?
                                .ordinal;
                            Some((guard, ordinal))
                        }) else {
                            for argument in arguments.iter().rev() {
                                push_owner_task(
                                    pending,
                                    OwnerWalkTask::Term(argument, terminal),
                                    meter,
                                    walk_site,
                                )?;
                            }
                            continue;
                        };
                        let then_frame = derived_frame_id(
                            terminal,
                            &guard,
                            occurrence,
                            true,
                            index,
                            frame_index,
                            record_position,
                            frame_seen,
                            frame_order,
                            meter,
                        )?;
                        let else_frame = derived_frame_id(
                            terminal,
                            &guard,
                            occurrence,
                            false,
                            index,
                            frame_index,
                            record_position,
                            frame_seen,
                            frame_order,
                            meter,
                        )?;
                        push_owner_task(
                            pending,
                            OwnerWalkTask::Term(&arguments[2], else_frame),
                            meter,
                            walk_site,
                        )?;
                        push_owner_task(
                            pending,
                            OwnerWalkTask::Term(&arguments[1], then_frame),
                            meter,
                            walk_site,
                        )?;
                        push_owner_task(
                            pending,
                            OwnerWalkTask::Term(&arguments[0], terminal),
                            meter,
                            walk_site,
                        )?;
                    } else if derive
                        && matches!(
                            identity,
                            Some(
                                "filament.op.boolean.and"
                                    | "filament.op.boolean.or"
                                    | "filament.op.boolean.implies"
                            )
                        )
                        && arguments.len() == 2
                    {
                        let holds = identity != Some("filament.op.boolean.or");
                        let guard = reference(&arguments[0]);
                        let right_frame = guard.and_then(|guard| {
                            let position = index.get(&guard)?;
                            let occurrence = nodes[*position]
                                .occurrences
                                .iter()
                                .filter(|occurrence| {
                                    occurrence.role == CheckedOccurrenceRole::Expression
                                })
                                .nth(*visit_counts.get(*position).unwrap_or(&0))?
                                .ordinal;
                            Some((guard, occurrence))
                        });
                        let right_terminal = if let Some((guard, occurrence)) = right_frame {
                            derived_frame_id(
                                terminal,
                                &guard,
                                occurrence,
                                holds,
                                index,
                                frame_index,
                                record_position,
                                frame_seen,
                                frame_order,
                                meter,
                            )?
                        } else {
                            terminal
                        };
                        push_owner_task(
                            pending,
                            OwnerWalkTask::Term(&arguments[1], right_terminal),
                            meter,
                            walk_site,
                        )?;
                        push_owner_task(
                            pending,
                            OwnerWalkTask::Term(&arguments[0], terminal),
                            meter,
                            walk_site,
                        )?;
                    } else {
                        for argument in arguments.iter().rev() {
                            push_owner_task(
                                pending,
                                OwnerWalkTask::Term(argument, terminal),
                                meter,
                                walk_site,
                            )?;
                        }
                    }
                }
                CheckedTerm::Aggregate { members, .. } => {
                    for member in members.iter().rev() {
                        push_owner_task(
                            pending,
                            OwnerWalkTask::Term(member, terminal),
                            meter,
                            walk_site,
                        )?;
                    }
                }
                CheckedTerm::Binding { value, .. } => {
                    push_owner_task(
                        pending,
                        OwnerWalkTask::Term(value, terminal),
                        meter,
                        walk_site,
                    )?;
                }
                _ => {}
            },
        }
    }
    Ok(())
}

fn push_owner_task<'a>(
    pending: &mut Vec<OwnerWalkTask<'a>>,
    task: OwnerWalkTask<'a>,
    meter: &mut WorkMeter,
    site: &Site<'_>,
) -> Result<(), ValidationFailure> {
    if pending.len() == pending.capacity() {
        let capacity = pending
            .len()
            .saturating_add(1)
            .max(pending.capacity().saturating_mul(2))
            .max(4);
        stage_storage::reserve_items::<OwnerWalkTask<'_>>(meter, site, capacity)?;
        pending
            .try_reserve_exact(capacity.saturating_sub(pending.len()))
            .map_err(|_| {
                stage_storage::unavailable(
                    capacity.saturating_mul(std::mem::size_of::<OwnerWalkTask<'_>>()),
                )
            })?;
    }
    pending.push(task);
    Ok(())
}

#[allow(clippy::too_many_arguments)] // the frame key, its lookup tables and the walk state it updates
fn derived_frame_id(
    parent: Option<PathConditionFrameId>,
    guard: &CheckedNodeId,
    occurrence: u64,
    holds: bool,
    index: &GraphIndex<'_>,
    frame_index: &[PathConditionFrameIndexEntry],
    record_position: usize,
    frame_seen: &mut [u8],
    frame_order: &mut Vec<usize>,
    meter: &mut WorkMeter,
) -> Result<Option<PathConditionFrameId>, ValidationFailure> {
    if parent == Some(PRUNED_FRAME) {
        return Ok(parent);
    }
    let parent_index = parent.map(|parent| parent.0);
    let guard_position = index.get_charged(guard, meter)?.ok_or_else(|| {
        refuse(
            record_position,
            Path::root().key("path_condition_frames"),
            Cause::InvalidValue,
            None,
            meter,
        )
    })?;
    let key = (parent_index, guard_position, occurrence, holds);
    let Ok(found) = frame_index.binary_search_by_key(&key, |entry| entry.0) else {
        // FR-825: every table row occurs in a record's chain, so a branch with
        // no scalar-operation record has no row. A record inside such a
        // branch cites a different terminal and is refused at its own binding.
        return Ok(Some(PRUNED_FRAME));
    };
    let frame_id = frame_index[found].1;
    if let Some(seen) = frame_seen.get_mut(frame_id) {
        if *seen == 0 {
            *seen = 1;
            frame_order.push(frame_id);
        }
    }
    Ok(Some(PathConditionFrameId(frame_id)))
}

fn record_binding_shape(term: &CheckedTerm) -> bool {
    let CheckedTerm::Aggregate { members, .. } = term else {
        return false;
    };
    const NAMES: [&str; 8] = [
        "owner",
        "application",
        "occurrence",
        "result_bound",
        "conversion",
        "path_condition",
        "extent_roots",
        "objects",
    ];
    members.len() == NAMES.len()
        && members.iter().zip(NAMES).all(|(member, expected)| {
            matches!(member, CheckedTerm::Binding { name, .. }
                if matches!(name.kind(), super::CheckedBindingNameKind::Other(actual) if actual == expected))
        })
}

fn record_body_failure_path<'a>(term: &CheckedTerm, node_at: Path<'a>) -> Path<'a> {
    let member_path = |member| node_at.key("body").key("members").index(member);
    for (member, valid) in [
        (0, binding(term, 0).and_then(reference).is_some()),
        (1, binding(term, 1).and_then(reference).is_some()),
        (2, binding(term, 2).and_then(integer).is_some()),
        (3, binding(term, 3).and_then(reference).is_some()),
        (
            4,
            binding(term, 4).is_some_and(|conversion| match conversion {
                CheckedTerm::Aggregate { members, .. } if members.is_empty() => true,
                CheckedTerm::Aggregate { members, .. } if members.len() == 2 => {
                    named_value(&members[0], "narrow")
                        .and_then(reference)
                        .is_some()
                        && named_value(&members[1], "occurrence")
                            .and_then(integer)
                            .is_some()
                }
                _ => false,
            }),
        ),
    ] {
        if !valid {
            return member_path(member);
        }
    }
    if let Some(CheckedTerm::Aggregate { members, .. }) = binding(term, 5) {
        if let Some(guard_index) = members.iter().position(|guard| {
            let CheckedTerm::Aggregate { members, .. } = guard else {
                return true;
            };
            members.len() != 3
                || named_value(&members[0], "guard")
                    .and_then(reference)
                    .is_none()
        }) {
            return member_path(5)
                .key("value")
                .key("members")
                .index(guard_index);
        }
    } else {
        return member_path(5);
    }
    for member in [6, 7] {
        if let Some(CheckedTerm::Aggregate { members, .. }) = binding(term, member) {
            if let Some(entry_index) = members
                .iter()
                .position(|entry| reference(member_value(entry).unwrap_or(entry)).is_none())
            {
                return member_path(member)
                    .key("value")
                    .key("members")
                    .index(entry_index);
            }
        } else {
            return member_path(member);
        }
    }
    member_path(0)
}

fn function_body_root(term: &CheckedTerm) -> Option<&CheckedNodeId> {
    let CheckedTerm::Aggregate { members, .. } = term else {
        return None;
    };
    members.iter().find_map(|member| {
        let CheckedTerm::Binding { name, value, .. } = member else {
            return None;
        };
        if !matches!(name.kind(), super::CheckedBindingNameKind::Body) {
            return None;
        }
        match value.as_ref() {
            CheckedTerm::Reference { target } => Some(target),
            _ => None,
        }
    })
}

/// The termination measure's root, which the path-condition walk excludes but
/// whose reads still take expression occurrences.
fn function_measure_root(term: &CheckedTerm) -> Option<&CheckedNodeId> {
    let CheckedTerm::Aggregate { members, .. } = term else {
        return None;
    };
    members.iter().find_map(|member| {
        let CheckedTerm::Binding { name, value, .. } = member else {
            return None;
        };
        if !matches!(
            name.kind(),
            super::CheckedBindingNameKind::Other("decreases" | "measure")
        ) {
            return None;
        }
        match value.as_ref() {
            CheckedTerm::Reference { target } => Some(target),
            _ => None,
        }
    })
}

fn capability_wire(kind: CapabilityKind) -> &'static str {
    match kind {
        CapabilityKind::ValueValidity => "value-validity",
        CapabilityKind::OperationContract => "operation-contract",
        CapabilityKind::FiniteReplay => "finite-replay",
        CapabilityKind::TemporalSatisfaction => "temporal-satisfaction",
        CapabilityKind::GlobalConformance => "global-conformance",
        CapabilityKind::Monitorability => "monitorability",
        CapabilityKind::LocalProjection => "local-projection",
        CapabilityKind::Refinement => "refinement",
        CapabilityKind::Realizability => "realizability",
        CapabilityKind::Composition => "composition",
        CapabilityKind::ProbabilisticSatisfaction => "probabilistic-satisfaction",
        CapabilityKind::Schedulability => "schedulability",
    }
}

fn domain_kind_wire(kind: DomainKind) -> &'static str {
    match kind {
        DomainKind::Collection => "collection",
        DomainKind::Population => "population",
        DomainKind::Integer => "integer",
        DomainKind::Recursive => "recursive",
        DomainKind::Quantity => "quantity",
        DomainKind::Loop => "loop",
        DomainKind::InfiniteTrace => "infinite-trace",
    }
}

#[ix_trace_rs::string_edge]
fn is_scalar_operation(identity: &str) -> bool {
    [
        "filament.op.integer",
        "filament.op.rational",
        "filament.op.decimal",
        "filament.op.ieee",
        "filament.op.quantity",
        "filament.op.numeric",
        "filament.op.text",
        "filament.op.enum",
    ]
    .iter()
    .any(|prefix| {
        identity
            .strip_prefix(prefix)
            .is_some_and(|suffix| suffix.is_empty() || suffix.starts_with('.'))
            && identity != "filament.op.numeric.narrow"
    })
}

fn occurrence_role_wire(role: &CheckedOccurrenceRole) -> &'static str {
    match role {
        CheckedOccurrenceRole::Declaration => "declaration",
        CheckedOccurrenceRole::Type => "type",
        CheckedOccurrenceRole::Expression => "expression",
        CheckedOccurrenceRole::Anchor => "anchor",
        CheckedOccurrenceRole::Claim => "claim",
        CheckedOccurrenceRole::Generated => "generated",
    }
}

/// The facts retained from a `claim`/`value_validity` node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueValidityRecord {
    /// `claim`/`value_validity` record node.
    pub node: CheckedNodeId,
    /// Function owning the application occurrence.
    pub owner: CheckedNodeId,
    /// Scalar operation application node.
    pub application: CheckedNodeId,
    /// Application expression occurrence ordinal.
    pub occurrence: u64,
    /// Admitted result-bound type node.
    pub result_bound: CheckedNodeId,
    /// Wrapping narrow application and its occurrence, when present.
    pub conversion: Option<(CheckedNodeId, u64)>,
    /// The terminal frame encoding this record's path condition, when guarded.
    pub path_condition_frame: Option<PathConditionFrameId>,
    /// Parameter and binder value nodes read by the claim.
    pub extent_roots: Vec<CheckedNodeId>,
    /// Model object types whose members the claim reads.
    pub objects: Vec<CheckedNodeId>,
}

/// Facts retained from a `claim`/`operation_contract` node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationContractRecord {
    /// The operation-contract record node.
    pub node: CheckedNodeId,
    /// Boolean bound type used by the source record.
    pub result_bound: CheckedNodeId,
    /// Whether the record is for an invariant-clause requirement.
    pub invariant: bool,
    /// Associated operation anchor, when present.
    pub operation: Option<CheckedNodeId>,
    /// Model object types read by the source record, in structural order.
    pub model_objects: Vec<CheckedNodeId>,
    /// Population identities and authored bounds.
    pub populations: Vec<OperationPopulation>,
}

/// One population in an operation-contract record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationPopulation {
    /// The population's model/object type.
    pub member_type: CheckedNodeId,
    /// Population declaration ordinal.
    pub ordinal: u64,
    /// Authored maximum, absent when the population is unbounded.
    pub maximum: Option<u64>,
}

pub(super) fn retain_rows(
    rows: Option<&[RequirementItemWireRow]>,
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    operand_rows: &[Vec<CheckedOperandEntry>],
    node_positions: &super::retained_index::PositionIndex,
) -> Vec<RequirementItemRow> {
    // These sidecar keys are document-held IDs, not node-held slots. Preserve
    // them directly; only references inside a record body need the record
    // node's authoritative operand row to reach the document-held space.
    rows.map_or_else(Vec::new, |rows| {
        rows.iter()
            .map(|row| {
                let record = row.record.as_ref().and_then(|id| {
                    let position = node_positions.node(nodes, id)?;
                    let holder_row = operand_rows.get(position)?;
                    let node = nodes.get(position)?;
                    match kinds.get(position)? {
                        CheckedNodeKind::Claim(super::ClaimForm::ValueValidity) => {
                            read_record(node.node_id.clone(), &node.body).map(|record| {
                                RequirementRecord::ValueValidity(resolve_record(record, holder_row))
                            })
                        }
                        CheckedNodeKind::Claim(super::ClaimForm::OperationContract) => {
                            read_operation_contract_record(node.node_id.clone(), &node.body).map(
                                |record| {
                                    RequirementRecord::OperationContract(
                                        resolve_operation_contract_record(record, holder_row),
                                    )
                                },
                            )
                        }
                        _ => None,
                    }
                });
                RequirementItemRow {
                    claim: row.claim.node.clone(),
                    occurrence: row.claim.occurrence.clone(),
                    kind: row.kind,
                    extent: match &row.extent {
                        RequirementExtentWire::Bounded => RequirementExtent::Bounded,
                        RequirementExtentWire::Unbounded { domains } => {
                            RequirementExtent::Unbounded(
                                domains
                                    .iter()
                                    .map(|domain| (domain.domain.clone(), domain.kind))
                                    .collect(),
                            )
                        }
                    },
                    record,
                }
            })
            .collect()
    })
}

pub(super) fn retain_frames(
    rows: Option<&[PathConditionFrameWire]>,
    nodes: &[CheckedSemanticNodeV2],
    operand_rows: &[Vec<CheckedOperandEntry>],
    node_positions: &super::retained_index::PositionIndex,
) -> Vec<PathConditionFrame> {
    rows.map_or_else(Vec::new, |rows| {
        rows.iter()
            .map(|row| PathConditionFrame {
                parent: row.parent.map(|parent| {
                    PathConditionFrameId(
                        usize::try_from(parent).expect("admission checked frame parent width"),
                    )
                }),
                guard: node_positions
                    .node(nodes, &row.guard.node)
                    .and_then(|position| operand_rows.get(position))
                    .map_or_else(
                        || row.guard.node.clone(),
                        |holder_row| resolve_holder_id(row.guard.node.clone(), holder_row),
                    ),
                occurrence: row.guard.occurrence.ordinal,
                holds: row.holds,
            })
            .collect()
    })
}

fn resolve_holder_id(id: CheckedNodeId, row: &[CheckedOperandEntry]) -> CheckedNodeId {
    let mut resolved = id;
    let result = super::node_index::resolve_id_operand_slots(&mut resolved, row, |package| {
        Ok::<_, std::convert::Infallible>(package.into())
    });
    match result {
        Ok(()) => resolved,
        Err(never) => match never {},
    }
}

fn resolve_record(
    mut record: ValueValidityRecord,
    holder_row: &[CheckedOperandEntry],
) -> ValueValidityRecord {
    record.owner = resolve_holder_id(record.owner, holder_row);
    record.application = resolve_holder_id(record.application, holder_row);
    record.result_bound = resolve_holder_id(record.result_bound, holder_row);
    record.conversion = record
        .conversion
        .map(|(id, occurrence)| (resolve_holder_id(id, holder_row), occurrence));
    for root in &mut record.extent_roots {
        *root = resolve_holder_id(root.clone(), holder_row);
    }
    for object in &mut record.objects {
        *object = resolve_holder_id(object.clone(), holder_row);
    }
    record
}

fn resolve_operation_contract_record(
    mut record: OperationContractRecord,
    holder_row: &[CheckedOperandEntry],
) -> OperationContractRecord {
    record.result_bound = resolve_holder_id(record.result_bound, holder_row);
    record.operation = record
        .operation
        .map(|operation| resolve_holder_id(operation, holder_row));
    for object in &mut record.model_objects {
        *object = resolve_holder_id(object.clone(), holder_row);
    }
    for population in &mut record.populations {
        population.member_type = resolve_holder_id(population.member_type.clone(), holder_row);
    }
    record
}

pub(super) fn owner_index(rows: &[RequirementItemRow]) -> Vec<(CheckedNodeId, Vec<usize>)> {
    // One stable sort by owner keeps each owner's rows in row order, then one
    // pass groups equal owners: O(n log n) where inserting each new owner
    // into a sorted vector shifted every later owner.
    let mut owned: Vec<(&CheckedNodeId, usize)> = rows
        .iter()
        .enumerate()
        .filter_map(|(row_index, row)| {
            let RequirementRecord::ValueValidity(record) = row.record.as_ref()? else {
                return None;
            };
            Some((&record.owner, row_index))
        })
        .collect();
    owned.sort_by_key(|&(owner, _)| owner);
    let mut index: Vec<(CheckedNodeId, Vec<usize>)> = Vec::new();
    for (owner, row_index) in owned {
        match index.last_mut() {
            Some((last, owner_rows)) if *last == *owner => owner_rows.push(row_index),
            _ => index.push((owner.clone(), vec![row_index])),
        }
    }
    index
}

fn read_record(node: CheckedNodeId, term: &CheckedTerm) -> Option<ValueValidityRecord> {
    let owner = reference(binding(term, 0)?)?;
    let application = reference(binding(term, 1)?)?;
    let occurrence = integer(binding(term, 2)?)?;
    let result_bound = reference(binding(term, 3)?)?;
    let conversion_term = binding(term, 4)?;
    let conversion = match conversion_term {
        CheckedTerm::Aggregate { members, .. } if members.is_empty() => None,
        CheckedTerm::Aggregate { members, .. } if members.len() == 2 => Some((
            reference(named_value(&members[0], "narrow")?)?,
            integer(named_value(&members[1], "occurrence")?)?,
        )),
        _ => return None,
    };
    let CheckedTerm::Aggregate { members: path, .. } = binding(term, 5)? else {
        return None;
    };
    let path_condition_frame = match path.as_slice() {
        [] => None,
        [terminal] => Some(PathConditionFrameId(
            usize::try_from(integer(terminal)?).ok()?,
        )),
        _ => return None,
    };
    Some(ValueValidityRecord {
        node,
        owner,
        application,
        occurrence,
        result_bound,
        conversion,
        path_condition_frame,
        extent_roots: references(binding(term, 6)?)?,
        objects: references(binding(term, 7)?)?,
    })
}

fn read_operation_contract_record(
    node: CheckedNodeId,
    term: &CheckedTerm,
) -> Option<OperationContractRecord> {
    let result_bound = reference(binding(term, 2)?)?;
    let invariant = boolean(binding(term, 3)?)?;
    let operation = match binding(term, 4)? {
        CheckedTerm::Aggregate { members, .. } if members.is_empty() => None,
        term => Some(reference(term)?),
    };
    let model_objects = references(binding(term, 5)?)?;
    let CheckedTerm::Aggregate {
        members: population_terms,
        ..
    } = binding(term, 6)?
    else {
        return None;
    };
    let mut populations = Vec::with_capacity(population_terms.len());
    for population_term in population_terms {
        let CheckedTerm::Aggregate { members, .. } = population_term else {
            return None;
        };
        const POPULATION_NAMES: [&str; 3] = ["member_type", "ordinal", "maximum"];
        if members.len() != POPULATION_NAMES.len()
            || !members.iter().zip(POPULATION_NAMES).all(|(member, expected)| {
                matches!(member, CheckedTerm::Binding { name, .. }
                    if matches!(name.kind(), super::CheckedBindingNameKind::Other(actual) if actual == expected))
            })
        {
            return None;
        }
        let member_type = reference(named_value(members.first()?, "member_type")?)?;
        let ordinal = integer(named_value(members.get(1)?, "ordinal")?)?;
        let maximum = match named_value(members.get(2)?, "maximum")? {
            CheckedTerm::Aggregate { members, .. } if members.is_empty() => None,
            term => Some(integer(term)?),
        };
        populations.push(OperationPopulation {
            member_type,
            ordinal,
            maximum,
        });
    }
    Some(OperationContractRecord {
        node,
        result_bound,
        invariant,
        operation,
        model_objects,
        populations,
    })
}

fn identity_path_terminal(node: &CheckedNodeId) -> Option<String> {
    let mut path = node.path.iter();
    while let Some(segment) = path.next() {
        if matches!(segment, super::CheckedNodePathSegment::Name(name) if name.as_ref() == "path_condition")
        {
            return match path.next()? {
                super::CheckedNodePathSegment::Name(value) => Some(value.to_string()),
                _ => None,
            };
        }
    }
    None
}

fn binding(term: &CheckedTerm, index: usize) -> Option<&CheckedTerm> {
    let CheckedTerm::Aggregate { members, .. } = term else {
        return None;
    };
    member_value(members.get(index)?)
}

fn member_value(term: &CheckedTerm) -> Option<&CheckedTerm> {
    match term {
        CheckedTerm::Binding { value, .. } => Some(value),
        _ => None,
    }
}

fn named_value<'a>(term: &'a CheckedTerm, expected: &str) -> Option<&'a CheckedTerm> {
    let CheckedTerm::Binding { name, value, .. } = term else {
        return None;
    };
    matches!(name.kind(), super::CheckedBindingNameKind::Other(actual) if actual == expected)
        .then_some(value)
}

fn reference(term: &CheckedTerm) -> Option<CheckedNodeId> {
    match term {
        CheckedTerm::Reference { target } => Some(target.clone()),
        CheckedTerm::DependencyReference { node, .. } => Some(node.clone()),
        _ => None,
    }
}

fn reference_is_kind(
    term: &CheckedTerm,
    expected: CheckedNodeKind,
    kinds: &[CheckedNodeKind],
    index: &GraphIndex<'_>,
    evidence: &super::CheckedPackageEvidence,
) -> bool {
    match term {
        CheckedTerm::Reference { target } => index
            .get(target)
            .is_some_and(|&position| kinds[position] == expected),
        CheckedTerm::DependencyReference { package, node } => {
            evidence
                .dependency_package(package)
                .and_then(|dependency| dependency.node_kind(node))
                == Some(expected)
        }
        _ => false,
    }
}

fn integer(term: &CheckedTerm) -> Option<u64> {
    match term {
        CheckedTerm::Literal {
            value_kind: LiteralKind::Integer,
            value,
            ..
        } => value.as_u64(),
        _ => None,
    }
}

fn string(term: &CheckedTerm) -> Option<&str> {
    match term {
        CheckedTerm::Literal {
            value_kind: LiteralKind::String,
            value,
            ..
        } => value.as_str(),
        _ => None,
    }
}

fn boolean(term: &CheckedTerm) -> Option<bool> {
    match term {
        CheckedTerm::Literal {
            value_kind: LiteralKind::Boolean,
            value,
            ..
        } => value.as_bool(),
        _ => None,
    }
}

fn references(term: &CheckedTerm) -> Option<Vec<CheckedNodeId>> {
    let CheckedTerm::Aggregate { members, .. } = term else {
        return None;
    };
    members.iter().map(reference).collect()
}

#[cfg(test)]
mod tests {
    use super::{
        first_unnamed_position, CapabilityKind, DomainKey, DomainKind, PathConditionFrame,
        PathConditionFrameId, PathConditionView, RequirementExtent, RequirementExtentWire,
        RequirementItemRow, RequirementItemWireRow, ValueValidityRecord,
    };
    use crate::checked_package::shared::{
        CheckedNodeId, CheckedNodePathSegment, CheckedOperandEntry,
    };
    use ix_trace_rs::trace;
    use serde_json::json;

    #[trace("TC-488", "FR-376-AC-3")]
    #[test]
    fn named_record_completeness_uses_logarithmic_membership_work() {
        const RECORDS: usize = 100_000;
        let records = (0..RECORDS).collect::<Vec<_>>();
        let named = records.clone();
        let mut comparisons = 0;
        let found = first_unnamed_position(&records, &named, &mut || comparisons += 1);
        assert_eq!(found, None);
        assert!(comparisons >= RECORDS);
        let comparisons_per_lookup = usize::BITS as usize - RECORDS.leading_zeros() as usize + 1;
        assert!(
            comparisons <= RECORDS * comparisons_per_lookup,
            "named-record membership used {comparisons} comparisons for {RECORDS} rows"
        );

        let first_missing = RECORDS / 2;
        let named_without_one = (0..RECORDS)
            .filter(|position| *position != first_missing)
            .collect::<Vec<_>>();
        let mut missing_comparisons = 0;
        assert_eq!(
            first_unnamed_position(&records, &named_without_one, &mut || {
                missing_comparisons += 1
            }),
            Some(first_missing)
        );
        assert!(missing_comparisons > 0);
        assert!(
            missing_comparisons <= RECORDS * comparisons_per_lookup,
            "first missing record lookup used {missing_comparisons} comparisons"
        );

        const LEGACY_RECORDS: usize = 4_096;
        let mut legacy_comparisons = 0;
        for record in 0..LEGACY_RECORDS {
            for named in 0..LEGACY_RECORDS {
                legacy_comparisons += 1;
                if named == record {
                    break;
                }
            }
        }
        let legacy_logarithmic_bound =
            LEGACY_RECORDS * (usize::BITS as usize - LEGACY_RECORDS.leading_zeros() as usize + 1);
        assert!(legacy_comparisons > legacy_logarithmic_bound);
    }

    #[trace("TC-488", "FR-376-AC-1")]
    #[test]
    fn value_record_references_resolve_through_the_actual_holder_row() {
        let slot_id = |slot| {
            serde_json::from_value::<CheckedNodeId>(json!({
                "package":"pkg", "path":[{"operand":slot}], "ordinal":0
            }))
            .expect("slot reference")
        };
        let record = ValueValidityRecord {
            node: serde_json::from_value(json!({
                "package":"pkg", "path":["record"], "ordinal":0
            }))
            .expect("record ID"),
            owner: slot_id(0),
            application: slot_id(1),
            occurrence: 3,
            result_bound: slot_id(0),
            conversion: Some((slot_id(1), 7)),
            path_condition_frame: Some(PathConditionFrameId(2)),
            extent_roots: vec![slot_id(0)],
            objects: vec![slot_id(1)],
        };
        let holder_row = [CheckedOperandEntry::Node(11), CheckedOperandEntry::Node(4)];
        let resolved = super::resolve_record(record, &holder_row);
        let index_id = |index| CheckedNodeId {
            package: "pkg".into(),
            path: vec![CheckedNodePathSegment::Index(index)],
            ordinal: 0,
        };

        assert_eq!(resolved.owner, index_id(11));
        assert_eq!(resolved.application, index_id(4));
        assert_eq!(resolved.result_bound, index_id(11));
        assert_eq!(resolved.conversion, Some((index_id(4), 7)));
        assert_eq!(resolved.path_condition_frame, Some(PathConditionFrameId(2)));
        assert_eq!(resolved.extent_roots, vec![index_id(11)]);
        assert_eq!(resolved.objects, vec![index_id(4)]);
    }

    #[trace("TC-488", "FR-376-AC-1")]
    #[test]
    fn borrowed_path_view_yields_parent_chain_outermost_first() {
        let guard = |ordinal| CheckedNodeId {
            package: "pkg".into(),
            path: vec![CheckedNodePathSegment::Name(format!("g{ordinal}").into())],
            ordinal: 0,
        };
        let frames = [
            PathConditionFrame {
                parent: None,
                guard: guard(0),
                occurrence: 3,
                holds: true,
            },
            PathConditionFrame {
                parent: Some(PathConditionFrameId(0)),
                guard: guard(1),
                occurrence: 5,
                holds: false,
            },
        ];
        let view = PathConditionView::new(&frames, Some(PathConditionFrameId(1)));
        let chain = view.iter().collect::<Vec<_>>();
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0].guard, &frames[0].guard);
        assert_eq!(chain[0].occurrence, 3);
        assert!(chain[0].holds);
        assert_eq!(chain[1].guard, &frames[1].guard);
        assert_eq!(chain[1].occurrence, 5);
        assert!(!chain[1].holds);
        assert!(PathConditionView::new(&frames, None).is_empty());
    }

    #[trace("TC-2053", "FR-825-AC-5")]
    #[test]
    fn tc_488_decodes_the_requirement_row_wire_shape() {
        let row: RequirementItemWireRow = serde_json::from_str(
            r#"{
                "claim": {
                    "node": {"package":"pkg","path":["plus"],"ordinal":0},
                    "occurrence": {"role":"expression","ordinal":1}
                },
                "kind":"value-validity",
                "extent": {
                    "classification":"unbounded",
                    "domains":[{
                        "domain": {
                            "type":"node",
                            "node":{"package":"pkg","path":["parameter"],"ordinal":0},
                            "path":[]
                        },
                        "kind":"integer"
                    }]
                },
                "record":{"package":"pkg","path":["record"],"ordinal":0}
            }"#,
        )
        .expect("requirement row has the FR-825 checked wire shape");

        assert_eq!(row.kind, CapabilityKind::ValueValidity);
        let RequirementExtentWire::Unbounded { domains } = row.extent else {
            panic!("unbounded extent is retained as a typed unbounded value")
        };
        assert_eq!(domains.len(), 1);
        assert_eq!(domains[0].kind, DomainKind::Integer);
        assert!(matches!(
            &domains[0].domain,
            super::DomainKey::Node { node, path }
                if node.path == [crate::checked_package::shared::CheckedNodePathSegment::Name("parameter".into())]
                    && path.is_empty()
        ));
        assert!(row.record.is_some());
    }

    #[trace("TC-2053", "FR-825-AC-5")]
    #[test]
    fn tc_488_rejects_unknown_requirement_vocabulary_values() {
        let unknown_kind = serde_json::from_str::<CapabilityKind>("\"value-validty\"");
        let unknown_domain = serde_json::from_str::<DomainKind>("\"float\"");

        assert!(unknown_kind.is_err());
        assert!(unknown_domain.is_err());
    }

    #[trace("TC-488", "FR-376-AC-1")]
    #[test]
    fn tc_488_reads_operation_contract_record_facts() {
        let id = json!({"package":"pkg", "path":["node"], "ordinal":0});
        let body = serde_json::from_value(json!({
            "term":"aggregate",
            "members":[
                {"term":"binding", "name":"claim", "value":{"term":"reference", "target":id}},
                {"term":"binding", "name":"occurrence", "value":{"term":"aggregate", "members":[
                    {"term":"binding", "name":"role", "value":{"term":"literal", "type":id, "value_kind":"string", "value":"claim"}},
                    {"term":"binding", "name":"ordinal", "value":{"term":"literal", "type":id, "value_kind":"integer", "value":2}}
                ]}},
                {"term":"binding", "name":"result_bound", "value":{"term":"reference", "target":id}},
                {"term":"binding", "name":"invariant", "value":{"term":"literal", "type":id, "value_kind":"boolean", "value":true}},
                {"term":"binding", "name":"operation", "value":{"term":"aggregate", "members":[]}},
                {"term":"binding", "name":"model_objects", "value":{"term":"aggregate", "members":[
                    {"term":"reference", "target":id}
                ]}},
                {"term":"binding", "name":"populations", "value":{"term":"aggregate", "members":[
                    {"term":"aggregate", "members":[
                        {"term":"binding", "name":"member_type", "value":{"term":"reference", "target":id}},
                        {"term":"binding", "name":"ordinal", "value":{"term":"literal", "type":id, "value_kind":"integer", "value":3}},
                        {"term":"binding", "name":"maximum", "value":{"term":"literal", "type":id, "value_kind":"integer", "value":8}}
                    ]}
                ]}}
            ]
        })).expect("typed operation-contract body");
        let record_id: CheckedNodeId = serde_json::from_value(json!({
            "package":"pkg", "path":["record"], "ordinal":0
        }))
        .expect("record node id");
        let record = super::read_operation_contract_record(record_id.clone(), &body)
            .expect("operation-contract record facts");

        assert_eq!(record.node, record_id);
        assert_eq!(
            record.result_bound,
            super::reference(&id_term(&id)).unwrap()
        );
        assert!(record.invariant);
        assert_eq!(record.operation, None);
        assert_eq!(record.model_objects.len(), 1);
        assert_eq!(record.populations.len(), 1);
        assert_eq!(record.populations[0].ordinal, 3);
        assert_eq!(record.populations[0].maximum, Some(8));
    }

    /// FR-825: each owner body is walked once. A reference back to a node whose
    /// body the walk is inside is a recursive call and is not entered again, so
    /// a recursive owner ends within a small read-storage budget instead of
    /// growing the walk stack without end.
    #[trace("TC-2053", "FR-825-AC-5")]
    #[test]
    fn recursive_owner_body_walk_terminates_within_a_small_budget() {
        use super::super::stage_storage::{Path, Site};
        use super::{walk_owner_expression, OwnerWalkTask};
        use crate::checked_package::shared::{CheckedOccurrence, CheckedOccurrenceRole};
        use crate::checked_package::v2::{
            CheckedJson, GraphIndex, NativeJoinBudget, PackageReadStage, RawSemanticNodeV2,
            WorkMeter,
        };

        let id = |name: &str| -> CheckedNodeId {
            serde_json::from_value(json!({"package":"pkg", "path":[name], "ordinal":0}))
                .expect("node id")
        };
        let reference = |name: &str| json!({"term":"reference", "target":id(name)});
        let terms: Vec<super::CheckedTerm> = vec![
            serde_json::from_value(json!({"term":"aggregate", "members":[
                {"term":"binding", "name":"body", "value":reference("a")}
            ]}))
            .expect("owner body"),
            serde_json::from_value(json!({
                "term":"application", "operator":"conditional",
                "operation":{"identity":"filament.op.control.if", "laws":[], "mode":null,
                    "member":null, "leaves":[]},
                "result_type":id("t"),
                "arguments":[reference("g"), reference("f"), reference("g")]
            }))
            .expect("recursive if"),
            serde_json::from_value(json!({"term":"aggregate", "members":[]})).expect("guard"),
        ];
        let names = ["f", "a", "g"];
        let nodes: Vec<RawSemanticNodeV2> = names
            .iter()
            .zip(&terms)
            .map(|(name, term)| RawSemanticNodeV2 {
                node_id: id(name),
                schema_version: "filament.checked-semantic-graph/v2".into(),
                node_tag: "expression".into(),
                semantic_form: "x".into(),
                semantic_type: id("t"),
                dependencies: Vec::new(),
                occurrences: if *name == "a" {
                    vec![CheckedOccurrence {
                        role: CheckedOccurrenceRole::Expression,
                        ordinal: 0,
                    }]
                } else {
                    Vec::new()
                },
                recursion_group: None,
                declaration: None,
                owner: None,
                owner_model: None,
                check_mode: None,
                body: CheckedJson::from_serializable(term).expect("body json"),
            })
            .collect();
        let ids: Vec<CheckedNodeId> = names.iter().map(|name| id(name)).collect();
        let index: GraphIndex<'_> = ids
            .iter()
            .enumerate()
            .map(|(position, id)| (id, position))
            .collect();
        let mut budget =
            NativeJoinBudget::new(1_000_000, 1_000_000).with_read_storage_limit(1 << 16);
        let mut meter = WorkMeter::over(&mut budget);
        let at = Path::root();
        let site = Site::new(PackageReadStage::Graph, 0, "requirement_items", at);
        let mut visits = vec![0_usize; 3];
        let mut charged = vec![usize::MAX; 3];
        let mut active = vec![0_u8; 3];
        let mut paths = Vec::new();
        let mut frame_seen = Vec::new();
        let mut order = Vec::new();
        let mut pending: Vec<OwnerWalkTask<'_>> = Vec::new();
        let walked = walk_owner_expression(
            1,
            0,
            true,
            &nodes,
            &terms,
            &index,
            &[],
            0,
            &mut visits,
            &mut charged,
            &mut active,
            &mut paths,
            &mut frame_seen,
            &mut order,
            &mut pending,
            &site,
            &mut meter,
        );
        assert!(
            walked.is_ok(),
            "the recursive owner body walk must end: {walked:?}"
        );
        assert_eq!(visits[1], 1, "the recursive node is entered once");
    }

    /// FR-825: a node shared by two parents and no cycle is walked in full each
    /// time it is reached and charged once for the owner: the recursion guard
    /// marks only the nodes on the current path, not every node seen.
    #[trace("TC-2053", "FR-825-AC-5")]
    #[test]
    fn shared_node_without_a_cycle_is_walked_in_full_each_time() {
        use super::super::stage_storage::{Path, Site};
        use super::{walk_owner_expression, OwnerWalkTask};
        use crate::checked_package::shared::{CheckedOccurrence, CheckedOccurrenceRole};
        use crate::checked_package::v2::{
            CheckedJson, GraphIndex, NativeJoinBudget, PackageReadStage, RawSemanticNodeV2,
            WorkMeter,
        };

        let id = |name: &str| -> CheckedNodeId {
            serde_json::from_value(json!({"package":"pkg", "path":[name], "ordinal":0}))
                .expect("node id")
        };
        let reference = |name: &str| json!({"term":"reference", "target":id(name)});
        let call = |arguments: Vec<serde_json::Value>| {
            serde_json::from_value::<super::CheckedTerm>(json!({
                "term":"application", "operator":"call",
                "operation":{"identity":"filament.op.test.call", "laws":[], "mode":null,
                    "member":null, "leaves":[]},
                "result_type":id("t"),
                "arguments":arguments
            }))
            .expect("application")
        };
        // root -> (shared, shared); shared -> leaf. The diamond has no cycle.
        let names = ["root", "shared", "leaf"];
        let terms = vec![
            call(vec![reference("shared"), reference("shared")]),
            call(vec![reference("leaf")]),
            serde_json::from_value(json!({"term":"aggregate", "members":[]})).expect("leaf"),
        ];
        let expression = |count: u64| -> Vec<CheckedOccurrence> {
            (0..count)
                .map(|ordinal| CheckedOccurrence {
                    role: CheckedOccurrenceRole::Expression,
                    ordinal,
                })
                .collect()
        };
        let nodes: Vec<RawSemanticNodeV2> = names
            .iter()
            .zip(&terms)
            .zip([expression(1), expression(2), expression(2)])
            .map(|((name, term), occurrences)| RawSemanticNodeV2 {
                node_id: id(name),
                schema_version: "filament.checked-semantic-graph/v2".into(),
                node_tag: "expression".into(),
                semantic_form: "x".into(),
                semantic_type: id("t"),
                dependencies: Vec::new(),
                occurrences,
                recursion_group: None,
                declaration: None,
                owner: None,
                owner_model: None,
                check_mode: None,
                body: CheckedJson::from_serializable(term).expect("body json"),
            })
            .collect();
        let ids: Vec<CheckedNodeId> = names.iter().map(|name| id(name)).collect();
        let index: GraphIndex<'_> = ids
            .iter()
            .enumerate()
            .map(|(position, id)| (id, position))
            .collect();
        let mut budget =
            NativeJoinBudget::new(1_000_000, 1_000_000).with_read_storage_limit(1 << 16);
        let mut visits = vec![0_usize; 3];
        {
            let mut meter = WorkMeter::over(&mut budget);
            let at = Path::root();
            let site = Site::new(PackageReadStage::Graph, 0, "requirement_items", at);
            let mut charged = vec![usize::MAX; 3];
            let mut active = vec![0_u8; 3];
            let mut pending: Vec<OwnerWalkTask<'_>> = Vec::new();
            walk_owner_expression(
                0,
                0,
                true,
                &nodes,
                &terms,
                &index,
                &[],
                0,
                &mut visits,
                &mut charged,
                &mut active,
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                &mut pending,
                &site,
                &mut meter,
            )
            .expect("the diamond walk ends");
            assert!(
                active.iter().all(|inside| *inside == 0),
                "every node is left"
            );
        }
        assert_eq!(
            visits,
            [1, 2, 2],
            "the shared node and its leaf are walked twice"
        );
        assert_eq!(
            budget.consumed().work,
            3,
            "one work unit per node for the owner, as before the guard"
        );
    }

    /// FSpec FR-825: only a population's `maximum` is an empty aggregate when
    /// absent. The flat wire admits an empty aggregate at any leaf position of
    /// an operation-contract body, so the record read is what refuses one at
    /// `member_type` or `ordinal` (the caller refuses `invalid-value` at the
    /// node's `body`), while an empty `maximum` reads as no maximum.
    #[trace("TC-2053", "FR-825-AC-3")]
    #[test]
    fn operation_contract_population_empty_aggregate_is_read_only_as_maximum() {
        let id = json!({"package":"pkg", "path":["node"], "ordinal":0});
        let empty = json!({"term":"aggregate", "members":[]});
        let reference = json!({"term":"reference", "target":id});
        let integer = json!({"term":"literal", "type":id, "value_kind":"integer", "value":3});
        let read = |member_type: &serde_json::Value,
                    ordinal: &serde_json::Value,
                    maximum: &serde_json::Value| {
            let body = serde_json::from_value(json!({
                "term":"aggregate",
                "members":[
                    {"term":"binding", "name":"claim", "value":reference},
                    {"term":"binding", "name":"occurrence", "value":{"term":"aggregate", "members":[
                        {"term":"binding", "name":"role", "value":{"term":"literal", "type":id, "value_kind":"string", "value":"claim"}},
                        {"term":"binding", "name":"ordinal", "value":integer}
                    ]}},
                    {"term":"binding", "name":"result_bound", "value":reference},
                    {"term":"binding", "name":"invariant", "value":{"term":"literal", "type":id, "value_kind":"boolean", "value":true}},
                    {"term":"binding", "name":"operation", "value":empty},
                    {"term":"binding", "name":"model_objects", "value":{"term":"aggregate", "members":[]}},
                    {"term":"binding", "name":"populations", "value":{"term":"aggregate", "members":[
                        {"term":"aggregate", "members":[
                            {"term":"binding", "name":"member_type", "value":member_type},
                            {"term":"binding", "name":"ordinal", "value":ordinal},
                            {"term":"binding", "name":"maximum", "value":maximum}
                        ]}
                    ]}}
                ]
            }))
            .expect("typed operation-contract body");
            let node = serde_json::from_value(json!({
                "package":"pkg", "path":["record"], "ordinal":0
            }))
            .expect("record node id");
            super::read_operation_contract_record(node, &body)
        };
        let admitted = read(&reference, &integer, &empty).expect("an empty maximum is no maximum");
        assert_eq!(admitted.populations[0].maximum, None);
        assert!(
            read(&empty, &integer, &integer).is_none(),
            "empty member_type refuses"
        );
        assert!(
            read(&reference, &empty, &integer).is_none(),
            "empty ordinal refuses"
        );
    }

    fn id_term(value: &serde_json::Value) -> super::CheckedTerm {
        serde_json::from_value(json!({"term":"reference", "target":value})).expect("reference term")
    }

    #[trace("TC-2053", "FR-825-AC-1")]
    #[test]
    fn bounded_requirement_row_encodes_the_exact_wire_shape() {
        let row = RequirementItemRow {
            claim: serde_json::from_value(json!({
                "package":"pkg", "path":["claim"], "ordinal":0
            }))
            .expect("checked node ID"),
            occurrence: serde_json::from_value(json!({
                "role":"claim", "ordinal":0
            }))
            .expect("checked occurrence"),
            kind: CapabilityKind::OperationContract,
            extent: RequirementExtent::Bounded,
            record: None,
        };
        let bytes = quire_canonical::to_vec(&row, quire_canonical::Limits::new(u64::MAX))
            .expect("canonical requirement row");
        let encoded: serde_json::Value =
            serde_json::from_slice(&bytes).expect("encoded requirement JSON");

        assert_eq!(
            encoded,
            json!({
                "claim": {
                    "node":{"package":"pkg", "path":["claim"], "ordinal":0},
                    "occurrence":{"role":"claim", "ordinal":0}
                },
                "kind":"operation-contract",
                "extent":{"classification":"bounded"}
            })
        );

        let unbounded = RequirementItemRow {
            claim: row.claim.clone(),
            occurrence: row.occurrence.clone(),
            kind: CapabilityKind::ValueValidity,
            extent: RequirementExtent::Unbounded(vec![(
                DomainKey::Node {
                    node: row.claim.clone(),
                    path: vec![2],
                },
                DomainKind::Integer,
            )]),
            record: None,
        };
        let bytes = quire_canonical::to_vec(&unbounded, quire_canonical::Limits::new(u64::MAX))
            .expect("canonical unbounded requirement row");
        let encoded: serde_json::Value =
            serde_json::from_slice(&bytes).expect("encoded unbounded requirement JSON");
        assert_eq!(
            encoded["extent"],
            json!({
                "classification":"unbounded",
                "domains":[{
                    "domain":{
                        "type":"node",
                        "node":{"package":"pkg", "path":["claim"], "ordinal":0},
                        "path":[2]
                    },
                    "kind":"integer"
                }]
            })
        );
    }
}
