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
    CheckedOccurrenceRole, CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
};
use quire_canonical::{Encode, Error, FixedShape, Sink, Writer};
use serde::{Deserialize, Serialize};

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
    /// Value-validity record, when this row carries one.
    pub record: Option<ValueValidityRecord>,
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
            super::encode::encoded(writer, "record", &record.node)?;
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

fn root() -> Path<'static> {
    Path::root().key("requirement_items")
}

fn site(index: usize, at: Path<'_>) -> Site<'_> {
    Site::new(PackageReadStage::Graph, index, "requirement_items", at)
}

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

pub(super) fn validate(
    rows: Option<&[RequirementItemWireRow]>,
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[CheckedTerm],
    index: &GraphIndex<'_>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    evidence: &super::CheckedPackageEvidence,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut records: Vec<usize> = kinds
        .iter()
        .enumerate()
        .filter_map(|(position, kind)| {
            (*kind == CheckedNodeKind::Claim(super::ClaimForm::ValueValidity)).then_some(position)
        })
        .collect();
    records.sort_unstable_by(|left, right| structural_keys[*left].cmp(&structural_keys[*right]));
    let mut owner_reach: Vec<(CheckedNodeId, Vec<CheckedNodeId>)> = Vec::new();
    for &position in &records {
        let node = &nodes[position];
        let node_at = Path::root()
            .key("semantic_graph")
            .key("nodes")
            .index(position);
        stage_storage::work(meter, &site(position, node_at.clone()), 1)?;
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
            if let Some(CheckedTerm::Aggregate { members }) =
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
                |term| record_body_failure_path(term, node_at.clone()),
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
        let reach_index = if let Some((reach_index, _)) = owner_reach
            .iter()
            .enumerate()
            .find(|(_, (owner, _))| owner == &record.owner)
        {
            reach_index
        } else {
            let reached = owner_body_reach(owner_position, nodes, terms, index, position, meter)?;
            owner_reach.push((record.owner.clone(), reached));
            owner_reach.len() - 1
        };
        if !owner_reach
            .get(reach_index)
            .is_some_and(|(_, reached)| reached.contains(&record.application))
        {
            return Err(refuse(
                position,
                application_at,
                Cause::InvalidValue,
                Some(&node.node_id),
                meter,
            ));
        }
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
        for (guard_index, guard) in record.path_condition.iter().enumerate() {
            let at = node_at
                .key("body")
                .key("members")
                .index(5)
                .key("value")
                .key("members")
                .index(guard_index);
            let valid = index.get(&guard.guard).is_some_and(|&guard_position| {
                matches!(kinds[guard_position], CheckedNodeKind::Expression(_))
                    && owner_reach
                        .get(reach_index)
                        .is_some_and(|(_, reached)| reached.contains(&guard.guard))
                    && kinds.get(
                        index
                            .get(&nodes[guard_position].semantic_type)
                            .copied()
                            .unwrap_or(usize::MAX),
                    ) == Some(&CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean))
                    && nodes[guard_position].occurrences.iter().any(|occurrence| {
                        occurrence.role == CheckedOccurrenceRole::Expression
                            && occurrence.ordinal == guard.occurrence
                    })
            });
            if !valid {
                return Err(refuse(
                    position,
                    at,
                    Cause::InvalidValue,
                    Some(&node.node_id),
                    meter,
                ));
            }
        }
    }
    let Some(rows) = rows else {
        if let Some(&position) = records.first() {
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
    let mut previous: Option<(&CheckedNodeId, &'static str, u64, &'static str)> = None;
    let mut named_records: Vec<(usize, usize, CheckedNodeId)> = Vec::new();
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
        } else if row.record.is_some() {
            return Err(refuse(
                row_index,
                at.key("record"),
                Cause::InvalidValue,
                Some(&row.claim.node),
                meter,
            ));
        }
        let key = (
            structural_keys[claim_position].as_ref(),
            occurrence_role_wire(&row.claim.occurrence.role),
            row.claim.occurrence.ordinal,
            capability_wire(row.kind),
        );
        if previous.is_some_and(|prior| prior >= key) {
            let duplicate = previous.is_some_and(|prior| {
                prior.0 == key.0 && prior.1 == key.1 && prior.2 == key.2 && prior.3 == key.3
            });
            let cause = if duplicate {
                Cause::ConflictingDefinition
            } else {
                Cause::InvalidSemanticGraph
            };
            return Err(refuse(row_index, at, cause, Some(&row.claim.node), meter));
        }
        previous = Some(key);
    }
    if records
        .iter()
        .any(|record| !named_records.iter().any(|(_, named, _)| named == record))
    {
        let position = records
            .iter()
            .copied()
            .find(|record| !named_records.iter().any(|(_, named, _)| named == record))
            .unwrap_or_default();
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
    named_records.sort_unstable_by(
        |(left_row, left, left_owner), (right_row, right, right_owner)| {
            let left_owner_position = index.get(left_owner).copied().unwrap_or(usize::MAX);
            let right_owner_position = index.get(right_owner).copied().unwrap_or(usize::MAX);
            structural_keys[left_owner_position]
                .cmp(&structural_keys[right_owner_position])
                .then_with(|| left_row.cmp(right_row))
                .then_with(|| structural_keys[*left].cmp(&structural_keys[*right]))
        },
    );
    let mut previous_owner: Option<CheckedNodeId> = None;
    let mut owner_ordinal = 0_u64;
    for (_, record_position, owner) in named_records {
        if previous_owner.as_ref() != Some(&owner) {
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

fn record_binding_shape(term: &CheckedTerm) -> bool {
    let CheckedTerm::Aggregate { members } = term else {
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
                CheckedTerm::Aggregate { members } if members.is_empty() => true,
                CheckedTerm::Aggregate { members } if members.len() == 2 => {
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
    if let Some(CheckedTerm::Aggregate { members }) = binding(term, 5) {
        if let Some(guard_index) = members.iter().position(|guard| {
            let CheckedTerm::Aggregate { members } = guard else {
                return true;
            };
            members.len() != 3
                || named_value(&members[0], "guard")
                    .and_then(reference)
                    .is_none()
                || named_value(&members[1], "occurrence")
                    .and_then(integer)
                    .is_none()
                || named_value(&members[2], "holds")
                    .and_then(boolean)
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
        if let Some(CheckedTerm::Aggregate { members }) = binding(term, member) {
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

fn owner_body_reach(
    owner_position: usize,
    nodes: &[RawSemanticNodeV2],
    terms: &[CheckedTerm],
    index: &GraphIndex<'_>,
    record_position: usize,
    meter: &mut WorkMeter,
) -> Result<Vec<CheckedNodeId>, ValidationFailure> {
    let root_id = terms
        .get(owner_position)
        .and_then(function_body_root)
        .ok_or_else(|| {
            refuse(
                record_position,
                Path::root()
                    .key("semantic_graph")
                    .key("nodes")
                    .index(record_position)
                    .key("body")
                    .key("members")
                    .index(1),
                Cause::InvalidValue,
                Some(&nodes[record_position].node_id),
                meter,
            )
        })?;
    let mut pending = vec![root_id];
    let mut reached = Vec::new();
    while let Some(id) = pending.pop() {
        if reached.contains(&id) {
            continue;
        }
        let Some(&position) = index.get(&id) else {
            continue;
        };
        let at = Path::root()
            .key("semantic_graph")
            .key("nodes")
            .index(position);
        stage_storage::work(meter, &site(position, at), 1)?;
        reached.push(id);
        if let Some(term) = terms.get(position) {
            term.visit_reference_targets(&mut |target| pending.push(target.clone()));
        }
    }
    Ok(reached)
}

fn function_body_root(term: &CheckedTerm) -> Option<CheckedNodeId> {
    let CheckedTerm::Aggregate { members } = term else {
        return None;
    };
    members.iter().find_map(|member| {
        let CheckedTerm::Binding { name, value } = member else {
            return None;
        };
        if !matches!(name.kind(), super::CheckedBindingNameKind::Body) {
            return None;
        }
        match value.as_ref() {
            CheckedTerm::Reference { target } => Some(target.clone()),
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

/// One admitted path-condition guard for a value-validity record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequirementGuard {
    /// Boolean expression node of the guard.
    pub guard: CheckedNodeId,
    /// Guard expression occurrence ordinal.
    pub occurrence: u64,
    /// Outcome required for this path.
    pub holds: bool,
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
    /// Path-condition guards, outermost first.
    pub path_condition: Vec<RequirementGuard>,
    /// Parameter and binder value nodes read by the claim.
    pub extent_roots: Vec<CheckedNodeId>,
    /// Model object types whose members the claim reads.
    pub objects: Vec<CheckedNodeId>,
}

pub(super) fn retain_rows(
    rows: Option<&[RequirementItemWireRow]>,
    nodes: &[CheckedSemanticNodeV2],
) -> Vec<RequirementItemRow> {
    rows.map_or_else(Vec::new, |rows| {
        rows.iter()
            .map(|row| {
                let record = row.record.as_ref().and_then(|id| {
                    let position = nodes.iter().position(|node| &node.node_id == id)?;
                    read_record(id.clone(), &nodes.get(position)?.body)
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

pub(super) fn owner_index(rows: &[RequirementItemRow]) -> Vec<(CheckedNodeId, Vec<usize>)> {
    let mut index: Vec<(CheckedNodeId, Vec<usize>)> = Vec::new();
    for (row_index, row) in rows.iter().enumerate() {
        let Some(record) = &row.record else {
            continue;
        };
        match index.binary_search_by(|(owner, _)| owner.cmp(&record.owner)) {
            Ok(slot) => index[slot].1.push(row_index),
            Err(slot) => index.insert(slot, (record.owner.clone(), vec![row_index])),
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
        CheckedTerm::Aggregate { members } if members.is_empty() => None,
        CheckedTerm::Aggregate { members } if members.len() == 2 => Some((
            reference(named_value(&members[0], "narrow")?)?,
            integer(named_value(&members[1], "occurrence")?)?,
        )),
        _ => return None,
    };
    let CheckedTerm::Aggregate { members: guards } = binding(term, 5)? else {
        return None;
    };
    let mut path_condition = Vec::with_capacity(guards.len());
    for guard in guards {
        let guard = member_value(guard)?;
        let CheckedTerm::Aggregate { members } = guard else {
            return None;
        };
        path_condition.push(RequirementGuard {
            guard: reference(named_value(members.first()?, "guard")?)?,
            occurrence: integer(named_value(members.get(1)?, "occurrence")?)?,
            holds: boolean(named_value(members.get(2)?, "holds")?)?,
        });
    }
    Some(ValueValidityRecord {
        node,
        owner,
        application,
        occurrence,
        result_bound,
        conversion,
        path_condition,
        extent_roots: references(binding(term, 6)?)?,
        objects: references(binding(term, 7)?)?,
    })
}

fn binding(term: &CheckedTerm, index: usize) -> Option<&CheckedTerm> {
    let CheckedTerm::Aggregate { members } = term else {
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
    let CheckedTerm::Binding { name, value } = term else {
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
    let CheckedTerm::Aggregate { members } = term else {
        return None;
    };
    members
        .iter()
        .map(|member| reference(member_value(member)?))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        CapabilityKind, DomainKey, DomainKind, RequirementExtent, RequirementExtentWire,
        RequirementItemRow, RequirementItemWireRow,
    };
    use ix_trace_rs::trace;
    use serde_json::json;

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
