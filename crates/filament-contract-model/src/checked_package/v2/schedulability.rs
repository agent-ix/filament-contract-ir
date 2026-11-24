// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Authenticated schedulability claim carrier and FR-482 body admission.

use super::graph_storage::{GraphIndex, ResolvedKey, SortedTable};
use super::native_join_budget::PackageReadStage;
use super::stage_storage::{self, Path, Position, Refusal, Site};
use super::{
    CheckedLiteralRead, CheckedNodeId, CheckedNodeKind, CheckedPackageV2, CheckedSemanticNodeV2,
    CheckedTerm, ClaimForm, LiteralKind, ScalarTypeForm, WorkMeter,
};
use crate::checked_package::common::{Step, Trail, ValidationFailure};
use crate::checked_package::shared::{
    CheckedOccurrence, CheckedOccurrenceRole, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code,
};
use quire_canonical::{Encode, Error, Sink, Writer};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// An untrusted producer row naming one local claim occurrence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedSchedulabilityClaimRow {
    /// Local claim node identity.
    pub declaration: CheckedNodeId,
    /// Ordinal of its claim occurrence.
    pub occurrence: u64,
    /// Exact checked source task-set identifier for this occurrence.
    pub task_set: Box<str>,
    /// Release declarations associated with this occurrence's task set.
    pub release_bindings: Vec<CheckedSchedulabilityReleaseBinding>,
    /// Mode-switch declarations associated with this occurrence's task set.
    pub mode_switch_bindings: Vec<CheckedSchedulabilityModeSwitchBinding>,
}

/// One checked task-to-operation release relation supplied by the producer.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedSchedulabilityReleaseBinding {
    /// Exact task identifier in the claim's task set.
    pub task_name: Box<str>,
    /// Identity of the selected model package that declares the operation.
    pub selected_model_identity: Box<str>,
    /// Structural identity of the operation's declaring model node.
    pub operation_declaring_identity: CheckedNodeId,
    /// Exact operation member identifier, separate from its declaring node.
    pub operation_member: Box<str>,
}

/// One checked claim-to-operation criticality mode-switch relation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedSchedulabilityModeSwitchBinding {
    /// Identity of the selected model package that declares the operation.
    pub selected_model_identity: Box<str>,
    /// Structural identity of the operation's declaring model node.
    pub operation_declaring_identity: CheckedNodeId,
    /// Exact operation member identifier, separate from its declaring node.
    pub operation_member: Box<str>,
}

impl Encode for CheckedSchedulabilityClaimRow {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        super::encode::encoded(writer, "declaration", &self.declaration)?;
        super::encode::member(writer, "occurrence", &self.occurrence)?;
        super::encode::member(writer, "task_set", self.task_set.as_ref())?;
        super::encode::elements(writer, "release_bindings", &self.release_bindings)?;
        super::encode::elements(writer, "mode_switch_bindings", &self.mode_switch_bindings)?;
        writer.end_object()
    }
}

impl Encode for CheckedSchedulabilityReleaseBinding {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        super::encode::member(writer, "task_name", self.task_name.as_ref())?;
        super::encode::member(
            writer,
            "selected_model_identity",
            self.selected_model_identity.as_ref(),
        )?;
        super::encode::encoded(
            writer,
            "operation_declaring_identity",
            &self.operation_declaring_identity,
        )?;
        super::encode::member(writer, "operation_member", self.operation_member.as_ref())?;
        writer.end_object()
    }
}

impl Encode for CheckedSchedulabilityModeSwitchBinding {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        super::encode::member(
            writer,
            "selected_model_identity",
            self.selected_model_identity.as_ref(),
        )?;
        super::encode::encoded(
            writer,
            "operation_declaring_identity",
            &self.operation_declaring_identity,
        )?;
        super::encode::member(writer, "operation_member", self.operation_member.as_ref())?;
        writer.end_object()
    }
}

/// The closed schedulability claim forms.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedSchedulabilityKind {
    /// Whole task-set schedulability.
    Schedulable,
    /// One task's response bound.
    Response,
}
/// The closed scheduling policies.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedSchedulingPolicy {
    /// Static fixed priority.
    FixedPriority,
    /// Earliest deadline first.
    Edf,
    /// Adaptive mixed criticality.
    Amc,
}
/// The closed task arrival modes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedTaskArrival {
    /// Periodic releases.
    Periodic,
    /// Sporadic releases.
    Sporadic,
}
/// The closed task criticalities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedTaskCriticality {
    /// Low criticality.
    Lo,
    /// High criticality.
    Hi,
}

/// A package-borrowed authenticated claim; it cannot be constructed from a wire row.
///
/// ```compile_fail
/// use filament_contract_model::{CheckedSchedulabilityClaimRef, CheckedPackageV2};
/// fn escape(package: CheckedPackageV2) -> CheckedSchedulabilityClaimRef<'static> {
///     package.schedulability_claims().next().unwrap()
/// }
/// ```
///
/// ```compile_fail
/// use filament_contract_model::{CheckedSchedulabilityClaimRef, CheckedNodeId, CheckedOccurrence, CheckedTerm};
/// fn fabricate<'a>(declaration: &'a CheckedNodeId, occurrence: &'a CheckedOccurrence, body: &'a CheckedTerm) {
///     let _ = CheckedSchedulabilityClaimRef { declaration, occurrence, body };
/// }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct CheckedSchedulabilityClaimRef<'a> {
    declaration: &'a CheckedNodeId,
    occurrence: &'a CheckedOccurrence,
    body: &'a CheckedTerm,
    task_set: &'a str,
    release_bindings: &'a [CheckedSchedulabilityReleaseBinding],
    mode_switch_bindings: &'a [CheckedSchedulabilityModeSwitchBinding],
}

/// A package-borrowed, admitted task release binding.
#[derive(Clone, Copy, Debug)]
pub struct CheckedSchedulabilityReleaseBindingRef<'a> {
    row: &'a CheckedSchedulabilityReleaseBinding,
}

impl CheckedSchedulabilityReleaseBindingRef<'_> {
    /// Exact task identifier in the claim's task set.
    pub fn task_name(&self) -> &str {
        &self.row.task_name
    }
    /// Exact selected model package identity.
    pub fn selected_model_identity(&self) -> &str {
        &self.row.selected_model_identity
    }
    /// Exact declaring model node identity.
    pub fn operation_declaring_identity(&self) -> &CheckedNodeId {
        &self.row.operation_declaring_identity
    }
    /// Exact operation member identifier.
    pub fn operation_member(&self) -> &str {
        &self.row.operation_member
    }
}

/// A package-borrowed, admitted criticality mode-switch binding.
#[derive(Clone, Copy, Debug)]
pub struct CheckedSchedulabilityModeSwitchBindingRef<'a> {
    row: &'a CheckedSchedulabilityModeSwitchBinding,
}

impl CheckedSchedulabilityModeSwitchBindingRef<'_> {
    /// Exact selected model package identity.
    pub fn selected_model_identity(&self) -> &str {
        &self.row.selected_model_identity
    }
    /// Exact declaring model node identity.
    pub fn operation_declaring_identity(&self) -> &CheckedNodeId {
        &self.row.operation_declaring_identity
    }
    /// Exact operation member identifier.
    pub fn operation_member(&self) -> &str {
        &self.row.operation_member
    }
}
/// A package-borrowed task whose binding shape and policy premises were admitted.
///
/// ```compile_fail
/// use filament_contract_model::{CheckedSchedulabilityTaskRef, CheckedTerm};
/// fn fabricate(body: &CheckedTerm) {
///     let _ = CheckedSchedulabilityTaskRef { body };
/// }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct CheckedSchedulabilityTaskRef<'a> {
    body: &'a CheckedTerm,
}

fn members(term: &CheckedTerm) -> Option<&[CheckedTerm]> {
    match term {
        CheckedTerm::Aggregate { members, .. } => Some(members),
        _ => None,
    }
}
fn bound(term: &CheckedTerm, index: usize) -> Option<&CheckedTerm> {
    match members(term)?.get(index)? {
        CheckedTerm::Binding { value, .. } => Some(value),
        _ => None,
    }
}
fn admitted_bound(term: &CheckedTerm, index: usize) -> &CheckedTerm {
    bound(term, index).expect("admission established the ordered binding shape")
}
fn text(term: &CheckedTerm) -> Option<&str> {
    match term {
        CheckedTerm::Literal {
            value_kind: LiteralKind::Text,
            value,
            ..
        } => value.as_str(),
        _ => None,
    }
}
fn admitted_text(term: &CheckedTerm) -> &str {
    text(term).expect("admission established the text literal")
}
fn literal(term: &CheckedTerm) -> CheckedLiteralRead<'_> {
    term.literal_read()
        .expect("admission established the exact literal")
}
fn empty(term: &CheckedTerm) -> bool {
    members(term).is_some_and(<[CheckedTerm]>::is_empty)
}
fn kind(term: &CheckedTerm, kind: LiteralKind) -> bool {
    matches!(term, CheckedTerm::Literal { value_kind, .. } if *value_kind == kind)
}
impl<'a> CheckedSchedulabilityClaimRef<'a> {
    /// Exact admitted local declaration identity.
    pub fn declaration(&self) -> &'a CheckedNodeId {
        self.declaration
    }
    /// Exact owned claim occurrence.
    pub fn occurrence(&self) -> &'a CheckedOccurrence {
        self.occurrence
    }
    /// Exact checked source task-set identifier for this claim occurrence.
    pub fn task_set(&self) -> &'a str {
        self.task_set
    }
    /// Admitted task-to-operation release bindings in canonical order.
    pub fn release_bindings(
        &self,
    ) -> impl ExactSizeIterator<Item = CheckedSchedulabilityReleaseBindingRef<'a>> + 'a {
        self.release_bindings
            .iter()
            .map(|row| CheckedSchedulabilityReleaseBindingRef { row })
    }
    /// Admitted claim-to-operation mode-switch bindings in canonical order.
    pub fn mode_switch_bindings(
        &self,
    ) -> impl ExactSizeIterator<Item = CheckedSchedulabilityModeSwitchBindingRef<'a>> + 'a {
        self.mode_switch_bindings
            .iter()
            .map(|row| CheckedSchedulabilityModeSwitchBindingRef { row })
    }
    /// Complete admitted body, borrowed without copying.
    pub fn body(&self) -> &'a CheckedTerm {
        self.body
    }
    /// Closed claim kind.
    #[ix_trace_rs::string_edge]
    pub fn kind(&self) -> CheckedSchedulabilityKind {
        match admitted_text(admitted_bound(self.body, 0)) {
            "schedulable" => CheckedSchedulabilityKind::Schedulable,
            "response" => CheckedSchedulabilityKind::Response,
            _ => unreachable!("admitted claim kind"),
        }
    }
    /// Closed declared policy.
    #[ix_trace_rs::string_edge]
    pub fn policy(&self) -> CheckedSchedulingPolicy {
        match admitted_text(admitted_bound(self.body, 1)) {
            "fixed-priority" => CheckedSchedulingPolicy::FixedPriority,
            "edf" => CheckedSchedulingPolicy::Edf,
            "amc" => CheckedSchedulingPolicy::Amc,
            _ => unreachable!("admitted scheduling policy"),
        }
    }
    /// Response task, present exactly for a response claim.
    pub fn response_task(&self) -> Option<&'a str> {
        text(admitted_bound(self.body, 2))
    }
    /// Exact response bound, present exactly for a response claim.
    pub fn response_bound(&self) -> Option<CheckedLiteralRead<'a>> {
        admitted_bound(self.body, 3).literal_read()
    }
    /// Tasks in the admitted task-name order.
    pub fn tasks(&self) -> impl ExactSizeIterator<Item = CheckedSchedulabilityTaskRef<'a>> + 'a {
        members(admitted_bound(self.body, 4))
            .expect("admitted tasks aggregate")
            .iter()
            .map(|body| CheckedSchedulabilityTaskRef { body })
    }
}
impl<'a> CheckedSchedulabilityTaskRef<'a> {
    /// Exact task name.
    pub fn name(&self) -> &'a str {
        admitted_text(admitted_bound(self.body, 0))
    }
    /// Closed arrival mode.
    #[ix_trace_rs::string_edge]
    pub fn arrival(&self) -> CheckedTaskArrival {
        match admitted_text(admitted_bound(self.body, 1)) {
            "periodic" => CheckedTaskArrival::Periodic,
            "sporadic" => CheckedTaskArrival::Sporadic,
            _ => unreachable!("admitted arrival"),
        }
    }
    /// Closed criticality.
    #[ix_trace_rs::string_edge]
    pub fn criticality(&self) -> CheckedTaskCriticality {
        match admitted_text(admitted_bound(self.body, 7)) {
            "lo" => CheckedTaskCriticality::Lo,
            "hi" => CheckedTaskCriticality::Hi,
            _ => unreachable!("admitted criticality"),
        }
    }
    /// Exact period or minimum inter-arrival time.
    pub fn period(&self) -> CheckedLiteralRead<'a> {
        literal(admitted_bound(self.body, 2))
    }
    /// Exact relative deadline.
    pub fn deadline(&self) -> CheckedLiteralRead<'a> {
        literal(admitted_bound(self.body, 3))
    }
    /// Exact best-case work.
    pub fn bcet(&self) -> CheckedLiteralRead<'a> {
        literal(admitted_bound(self.body, 4))
    }
    /// Exact low-mode worst-case work.
    pub fn wcet(&self) -> CheckedLiteralRead<'a> {
        literal(admitted_bound(self.body, 5))
    }
    /// Exact high-mode work, present exactly for high-criticality tasks.
    pub fn wcet_hi(&self) -> Option<CheckedLiteralRead<'a>> {
        admitted_bound(self.body, 6).literal_read()
    }
    /// Declared priority, with absence preserved.
    pub fn priority(&self) -> Option<CheckedLiteralRead<'a>> {
        admitted_bound(self.body, 8).literal_read()
    }
    /// Exact release jitter.
    pub fn jitter(&self) -> CheckedLiteralRead<'a> {
        literal(admitted_bound(self.body, 9))
    }
    /// Exact blocking time.
    pub fn blocking(&self) -> CheckedLiteralRead<'a> {
        literal(admitted_bound(self.body, 10))
    }
}
impl CheckedPackageV2 {
    /// Borrow the retained producer identity rows for lossless complete encoding.
    pub fn schedulability_claim_rows(&self) -> Option<&[CheckedSchedulabilityClaimRow]> {
        self.wire.schedulability_claims.as_deref()
    }
    /// Authenticated claims in canonical index order, after complete admission.
    pub fn schedulability_claims(
        &self,
    ) -> impl ExactSizeIterator<Item = CheckedSchedulabilityClaimRef<'_>> + '_ {
        self.schedulability_claim_index
            .iter()
            .map(|&(node, occurrence, row_index)| {
                let node = &self.wire.semantic_graph.nodes[node];
                let row = &self
                    .wire
                    .schedulability_claims
                    .as_ref()
                    .expect("indexed rows present")[row_index];
                CheckedSchedulabilityClaimRef {
                    declaration: &node.node_id,
                    occurrence: &node.occurrences[occurrence],
                    body: &node.body,
                    task_set: &row.task_set,
                    release_bindings: &row.release_bindings,
                    mode_switch_bindings: &row.mode_switch_bindings,
                }
            })
    }
}

fn invalid<'a>(
    code: Code,
    cause: Option<Cause>,
    at: impl Into<Position<'a>>,
    id: &CheckedNodeId,
    meter: &mut WorkMeter,
) -> ValidationFailure {
    let at = at.into();
    let refusal = Refusal::new(code, at).at_node(id);
    let refusal = if let Some(cause) = cause {
        refusal.because(cause)
    } else {
        refusal
    };
    let index = meter.site().map_or(0, |site| site.index);
    stage_storage::refuse(
        meter,
        &Site::new(PackageReadStage::Graph, index, "claim", at),
        refusal,
    )
}
fn claim(kind: CheckedNodeKind) -> bool {
    kind == CheckedNodeKind::Claim(ClaimForm::Schedulability)
}

pub(super) fn validate_carrier<Body>(
    package_name: &str,
    rows: Option<&[CheckedSchedulabilityClaimRow]>,
    nodes: &[CheckedSemanticNodeV2<Body>],
    kinds: &[CheckedNodeKind],
    index: &GraphIndex<'_>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    meter: &mut WorkMeter,
) -> Result<Vec<(usize, usize, usize)>, ValidationFailure> {
    let caller = meter.site();
    meter.at_row("schedulability_claims", 0);
    let root = || Path::root().key("schedulability_claims");
    // Per claim node (ascending position): its claim occurrences as
    // `(ordinal, occurrence position)` sorted, the last of equal ordinals
    // being the one a later insertion would have kept.
    let claims = nodes
        .iter()
        .enumerate()
        .filter(|(p, _)| claim(kinds[*p]))
        .count();
    let mut occurrences: Vec<(usize, Vec<(u64, usize)>)> = meter.vec(claims)?;
    for (p, node) in nodes.iter().enumerate().filter(|(p, _)| claim(kinds[*p])) {
        let held = node
            .occurrences
            .iter()
            .filter(|occurrence| occurrence.role == CheckedOccurrenceRole::Claim)
            .count();
        let mut by_ordinal: Vec<(u64, usize)> = meter.vec(held)?;
        for (position, occurrence) in node.occurrences.iter().enumerate() {
            let at = Path::root()
                .key("semantic_graph")
                .key("nodes")
                .index(p)
                .key("occurrences")
                .index(position);
            stage_storage::work(
                meter,
                &Site::new(PackageReadStage::Graph, p, "claim", at),
                1,
            )?;
            if occurrence.role == CheckedOccurrenceRole::Claim {
                by_ordinal.push((occurrence.ordinal, position));
            }
        }
        by_ordinal.sort_unstable();
        occurrences.push((p, by_ordinal));
    }
    let mut joined = meter.vec(rows.map_or(0, <[CheckedSchedulabilityClaimRow]>::len))?;
    let mut seen: Vec<(ResolvedKey<'_>, u64)> = Vec::new();
    let mut previous: Option<(ResolvedKey<'_>, u64)> = None;
    for (i, row) in rows.unwrap_or_default().iter().enumerate() {
        meter.at_row("schedulability_claims", i);
        let at = || root().index(i);
        stage_storage::work(
            meter,
            &Site::new(PackageReadStage::Graph, i, "schedulability_claims", at()),
            1,
        )?;
        let Some(&position) = index
            .get(&row.declaration)
            .filter(|&&p| row.declaration.package.as_ref() == package_name && claim(kinds[p]))
        else {
            return Err(invalid(
                Code::InvalidPackage,
                Some(Cause::InvalidValue),
                at().key("declaration"),
                &row.declaration,
                meter,
            ));
        };
        let Some(&occurrence_position) = occurrences
            .binary_search_by_key(&position, |(held, _)| *held)
            .ok()
            .and_then(|at| {
                let held = &occurrences[at].1;
                let after = held.partition_point(|(ordinal, _)| *ordinal <= row.occurrence);
                held.get(after.checked_sub(1)?)
                    .filter(|(ordinal, _)| *ordinal == row.occurrence)
                    .map(|(_, position)| position)
            })
        else {
            return Err(invalid(
                Code::InvalidPackage,
                Some(Cause::InvalidValue),
                at().key("occurrence"),
                &row.declaration,
                meter,
            ));
        };
        let key = (index.resolved_at(position, structural_keys), row.occurrence);
        let slot = match search_keys(&seen, &key, meter)? {
            Ok(_) => {
                return Err(invalid(
                    Code::InvalidPackage,
                    Some(Cause::ConflictingDefinition),
                    at(),
                    &row.declaration,
                    meter,
                ));
            }
            Err(slot) => slot,
        };
        meter.room(&mut seen, 1)?;
        seen.insert(slot, key);
        if let Some(previous) = previous {
            if compare_keys(&previous, &key, meter)? == Ordering::Greater {
                return Err(invalid(
                    Code::InvalidPackage,
                    Some(Cause::InvalidValue),
                    at(),
                    &row.declaration,
                    meter,
                ));
            }
        }
        previous = Some(key);
        joined.push((position, occurrence_position, i));
    }
    for (p, node) in nodes.iter().enumerate().filter(|(p, _)| claim(kinds[*p])) {
        for occurrence in node
            .occurrences
            .iter()
            .filter(|o| o.role == CheckedOccurrenceRole::Claim)
        {
            let key = (index.resolved_at(p, structural_keys), occurrence.ordinal);
            if search_keys(&seen, &key, meter)?.is_err() {
                let at = root();
                return Err(stage_storage::refuse(
                    meter,
                    &Site::new(PackageReadStage::Graph, 0, "schedulability_claims", at),
                    Refusal::new(Code::InvalidPackage, at).because(Cause::InvalidValue),
                ));
            }
        }
    }
    meter.restore_site(caller);
    Ok(joined)
}

/// Validate the task-set and operation relations after timed-model and claim
/// body admission, retaining every occurrence-scoped row without repair.
#[allow(clippy::too_many_arguments)]
pub(super) fn validate_relations<Body>(
    rows: Option<&[CheckedSchedulabilityClaimRow]>,
    claim_index: &[(usize, usize, usize)],
    nodes: &[CheckedSemanticNodeV2<Body>],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &GraphIndex<'_>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    owners: &super::model_members::ModelOwners<'_>,
    operation_bodies: Option<&[super::OperationBodyRow]>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let rows = rows.unwrap_or_default();
    for &(node_position, _, row_position) in claim_index {
        let row = &rows[row_position];
        let declaration = &row.declaration;
        meter.at_row("schedulability_claims", row_position);
        let at = || {
            Path::root()
                .key("schedulability_claims")
                .index(row_position)
        };
        if !super::identity::is_identifier(&row.task_set) {
            return Err(invalid(
                Code::MalformedWire,
                None,
                at().key("task_set"),
                declaration,
                meter,
            ));
        }
        let body = terms[node_position]
            .as_ref()
            .expect("admitted schedulability body");
        let tasks = members(admitted_bound(body, 4)).expect("admitted task list");

        let mut previous_release: Option<(&CheckedSchedulabilityReleaseBinding, usize)> = None;
        for (binding_position, binding) in row.release_bindings.iter().enumerate() {
            let path = at().key("release_bindings").index(binding_position);
            stage_storage::work(
                meter,
                &Site::new(
                    PackageReadStage::Graph,
                    row_position,
                    "schedulability_claims",
                    path,
                ),
                1,
            )?;
            if !super::identity::is_identifier(&binding.task_name) {
                return Err(invalid(
                    Code::MalformedWire,
                    None,
                    path.key("task_name"),
                    declaration,
                    meter,
                ));
            }
            if !tasks
                .iter()
                .any(|task| text(admitted_bound(task, 0)) == Some(binding.task_name.as_ref()))
            {
                return Err(invalid(
                    Code::InvalidPackage,
                    Some(Cause::InvalidValue),
                    path.key("task_name"),
                    declaration,
                    meter,
                ));
            }
            validate_model_binding(
                &binding.selected_model_identity,
                &binding.operation_declaring_identity,
                &binding.operation_member,
                path,
                declaration,
                nodes,
                kinds,
                terms,
                index,
                owners,
                operation_bodies,
                meter,
            )?;
            if let Some((previous, previous_owner_position)) = previous_release {
                let ordering = previous
                    .task_name
                    .cmp(&binding.task_name)
                    .then_with(|| {
                        previous
                            .selected_model_identity
                            .cmp(&binding.selected_model_identity)
                    })
                    .then(
                        index
                            .resolved_at(previous_owner_position, structural_keys)
                            .order(
                                &index.resolved_at(
                                    *index
                                        .get(&binding.operation_declaring_identity)
                                        .expect("validated declaring identity"),
                                    structural_keys,
                                ),
                                meter,
                            )?,
                    )
                    .then(previous.operation_member.cmp(&binding.operation_member));
                if ordering == Ordering::Greater {
                    return Err(invalid(
                        Code::InvalidPackage,
                        Some(Cause::InvalidValue),
                        path,
                        declaration,
                        meter,
                    ));
                }
            }
            previous_release = Some((
                binding,
                *index
                    .get(&binding.operation_declaring_identity)
                    .expect("validated declaring identity"),
            ));
        }

        let mut previous_switch: Option<(&CheckedSchedulabilityModeSwitchBinding, usize)> = None;
        for (binding_position, binding) in row.mode_switch_bindings.iter().enumerate() {
            let path = at().key("mode_switch_bindings").index(binding_position);
            stage_storage::work(
                meter,
                &Site::new(
                    PackageReadStage::Graph,
                    row_position,
                    "schedulability_claims",
                    path,
                ),
                1,
            )?;
            validate_model_binding(
                &binding.selected_model_identity,
                &binding.operation_declaring_identity,
                &binding.operation_member,
                path,
                declaration,
                nodes,
                kinds,
                terms,
                index,
                owners,
                operation_bodies,
                meter,
            )?;
            if let Some((previous, previous_owner_position)) = previous_switch {
                let ordering = previous
                    .selected_model_identity
                    .cmp(&binding.selected_model_identity)
                    .then(
                        index
                            .resolved_at(previous_owner_position, structural_keys)
                            .order(
                                &index.resolved_at(
                                    *index
                                        .get(&binding.operation_declaring_identity)
                                        .expect("validated declaring identity"),
                                    structural_keys,
                                ),
                                meter,
                            )?,
                    )
                    .then(previous.operation_member.cmp(&binding.operation_member));
                if ordering == Ordering::Greater {
                    return Err(invalid(
                        Code::InvalidPackage,
                        Some(Cause::InvalidValue),
                        path,
                        declaration,
                        meter,
                    ));
                }
            }
            previous_switch = Some((
                binding,
                *index
                    .get(&binding.operation_declaring_identity)
                    .expect("validated declaring identity"),
            ));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_model_binding<Body>(
    model_identity: &str,
    declaring_identity: &CheckedNodeId,
    operation_member: &str,
    binding_path: Path<'_>,
    claim_identity: &CheckedNodeId,
    nodes: &[CheckedSemanticNodeV2<Body>],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &GraphIndex<'_>,
    owners: &super::model_members::ModelOwners<'_>,
    operation_bodies: Option<&[super::OperationBodyRow]>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let model_at = binding_path.key("selected_model_identity");
    let selected_timed = !model_identity.is_empty()
        && kinds.iter().enumerate().any(|(position, kind)| {
            if *kind != CheckedNodeKind::TimedModel(super::TimedModelForm::Model) {
                return false;
            }
            let root = &nodes[position];
            let target_identity = terms
                .get(position)
                .and_then(Option::as_ref)
                .and_then(|body| bound(body, 0))
                .and_then(|model| match model {
                    CheckedTerm::Reference { target, .. } => index.get(target),
                    _ => None,
                })
                .and_then(|target| nodes.get(*target))
                .and_then(|target| target.owner.as_ref())
                .and_then(owner_identity);
            root.owner
                .as_ref()
                .and_then(owner_identity)
                .is_some_and(|identity| identity == model_identity)
                || target_identity.is_some_and(|identity| identity == model_identity)
        });
    if !selected_timed {
        return Err(invalid(
            if model_identity.is_empty() {
                Code::MalformedWire
            } else {
                Code::InvalidPackage
            },
            (!model_identity.is_empty()).then_some(Cause::InvalidValue),
            model_at,
            claim_identity,
            meter,
        ));
    }
    let model_node = index.get(declaring_identity).copied();
    let operation_declared_by_model = owners
        .owner_by_key(declaring_identity)
        .filter(|owner| owner.package.identity.as_ref() == model_identity)
        .map(|owner| {
            let mut budget =
                super::model_members::Budget::new(meter, owner.package.root, owner.package.bytes);
            owner
                .package
                .effective_operation_owner(owner.node, operation_member, &mut budget)
                .map(|declared| declared == Some(owner.node))
        })
        .transpose()?;
    let operation_declared_by_source = model_node.is_some_and(|position| {
        matches!(
            nodes[position].owner.as_ref(),
            Some(super::CheckedNodeOwner::Source { identity, .. })
                if identity.as_ref() == model_identity
        ) && operation_bodies.is_some_and(|rows| {
            rows.iter().any(|row| {
                row.document.as_ref() == model_identity
                    && row.declaring == *declaring_identity
                    && row.operation.as_ref() == operation_member
            })
        })
    });
    if model_node.is_none() {
        return Err(invalid(
            Code::InvalidPackage,
            Some(Cause::InvalidValue),
            binding_path.key("operation_declaring_identity"),
            claim_identity,
            meter,
        ));
    }
    if !super::identity::is_identifier(operation_member) {
        return Err(invalid(
            Code::MalformedWire,
            None,
            binding_path.key("operation_member"),
            claim_identity,
            meter,
        ));
    }
    if !operation_declared_by_model.unwrap_or(false) && !operation_declared_by_source {
        return Err(invalid(
            Code::InvalidPackage,
            Some(Cause::InvalidValue),
            binding_path.key("operation_member"),
            claim_identity,
            meter,
        ));
    }
    Ok(())
}

fn owner_identity(owner: &super::CheckedNodeOwner) -> Option<&str> {
    match owner {
        super::CheckedNodeOwner::ModelRoot { identity }
        | super::CheckedNodeOwner::Source { identity, .. }
        | super::CheckedNodeOwner::Model { identity, .. } => Some(identity),
    }
}

/// The order of two `(structural key, ordinal)` rows: ranks when the graph
/// has them, else a comparison that charges its frame list.
fn compare_keys(
    left: &(ResolvedKey<'_>, u64),
    right: &(ResolvedKey<'_>, u64),
    meter: &mut WorkMeter,
) -> Result<Ordering, ValidationFailure> {
    Ok(left
        .0
        .order(&right.0, meter)?
        .then_with(|| left.1.cmp(&right.1)))
}

/// Binary search of sorted `rows` for `key` (`Ok` position, or `Err` the
/// insertion slot), each comparison charged.
fn search_keys(
    rows: &[(ResolvedKey<'_>, u64)],
    key: &(ResolvedKey<'_>, u64),
    meter: &mut WorkMeter,
) -> Result<Result<usize, usize>, ValidationFailure> {
    let (mut low, mut high) = (0, rows.len());
    while low < high {
        let middle = low + (high - low) / 2;
        match compare_keys(&rows[middle], key, meter)? {
            Ordering::Less => low = middle + 1,
            Ordering::Greater => high = middle,
            Ordering::Equal => return Ok(Ok(middle)),
        }
    }
    Ok(Err(low))
}

const CLAIM_FIELDS: &[&str] = &["kind", "policy", "task", "bound", "tasks"];
const TASK_FIELDS: &[&str] = &[
    "name",
    "arrival",
    "period",
    "deadline",
    "bcet",
    "wcet",
    "wcet_hi",
    "criticality",
    "priority",
    "jitter",
    "blocking",
];
fn shape(term: &CheckedTerm, fields: &[&str]) -> bool {
    members(term).is_some_and(|members| {
        members.len() == fields.len() && members.iter().zip(fields).all(|(term, expected)| {
        matches!(term, CheckedTerm::Binding { name, .. } if name.as_ref() == *expected)
    })
    })
}
fn binding_at<'a>(at: &Path<'a>, i: usize) -> Path<'a> {
    at.key("members").index(i)
}
fn nonnegative(term: &CheckedTerm) -> bool {
    match term {
        CheckedTerm::Literal {
            value_kind: LiteralKind::Integer | LiteralKind::Rational | LiteralKind::Quantity,
            value,
            ..
        } => value
            .as_str()
            .or_else(|| value.number_text())
            .is_some_and(|text| !text.starts_with('-')),
        _ => false,
    }
}
fn zero(term: &CheckedTerm) -> bool {
    match term {
        CheckedTerm::Literal {
            value_kind: LiteralKind::Rational | LiteralKind::Quantity,
            value,
            ..
        } => value
            .as_str()
            .is_some_and(|text| text.split('/').next() == Some("0")),
        _ => false,
    }
}

fn magnitude(term: &CheckedTerm) -> (&str, &str) {
    let CheckedTerm::Literal { value, .. } = term else {
        unreachable!("admitted quantity")
    };
    let text = value.as_str().expect("admitted rational spelling");
    text.split_once('/').unwrap_or((text, "1"))
}
/// The existing signed rational syntax and reduction check without building
/// an intermediate owned refusal that the caller would immediately replace.
#[ix_trace_rs::string_edge]
pub(super) fn rational_parts_are_canonical(
    numerator: &str,
    denominator: &str,
    meter: &mut WorkMeter,
    at: Site<'_>,
) -> Result<bool, ValidationFailure> {
    let positive = |text: &str| matches!(text.as_bytes(), [first, rest @ ..] if (b'1'..=b'9').contains(first) && rest.iter().all(u8::is_ascii_digit));
    let magnitude = numerator.strip_prefix('-').unwrap_or(numerator);
    if !(numerator == "0" || positive(magnitude)) || !positive(denominator) {
        return Ok(false);
    }
    super::natural::coprime(magnitude, denominator, meter, at)
}

fn greater(
    left: &CheckedTerm,
    right: &CheckedTerm,
    meter: &mut WorkMeter,
    at: &Path<'_>,
) -> Result<bool, ValidationFailure> {
    Ok(super::natural::compare_rationals(
        magnitude(left),
        magnitude(right),
        meter,
        super::natural::site(meter, *at),
    )?
    .is_gt())
}

/// The claim nodes in structural-key order: one exact block, charged before
/// it is made. Ranks order the keys without a comparison that allocates;
/// positions are unique, so the unstable sort is deterministic.
fn claim_order(
    kinds: &[CheckedNodeKind],
    index: &GraphIndex<'_>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    meter: &mut WorkMeter,
) -> Result<Vec<usize>, ValidationFailure> {
    let claims = index.values().filter(|&&p| claim(kinds[p])).count();
    let mut order: Vec<usize> = meter.vec(claims)?;
    order.extend(index.values().copied().filter(|&p| claim(kinds[p])));
    order.sort_unstable_by_key(|&p| index.resolved_at(p, structural_keys));
    Ok(order)
}

/// Records a task's integer priority in the sorted set of the claim:
/// `true` when it is new, `false` when it repeats or is not an integer
/// literal. The value is read by a parse that charges its limbs, and the set
/// grows under the meter.
fn record_priority(
    term: &CheckedTerm,
    priorities: &mut Vec<filament_exact::Integer>,
    meter: &mut WorkMeter,
) -> Result<bool, ValidationFailure> {
    let CheckedTerm::Literal {
        value_kind: LiteralKind::Integer,
        value,
        ..
    } = term
    else {
        return Ok(false);
    };
    let priority = match (value.as_str(), value.as_i64(), value.as_u64()) {
        (Some(text), _, _) => meter.exact_integer(text)?,
        (None, Some(number), _) => Some(filament_exact::Integer::from(number)),
        (None, None, Some(number)) => Some(filament_exact::Integer::from(number)),
        _ => None,
    };
    match priority {
        Some(priority) => meter.insert_sorted(priorities, priority),
        None => Ok(false),
    }
}

#[ix_trace_rs::string_edge]
#[allow(clippy::too_many_arguments)]
pub(super) fn validate_bodies<Body>(
    nodes: &[CheckedSemanticNodeV2<Body>],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &GraphIndex<'_>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    models: &[super::model_members::DomainModel],
    claim_rows: Option<&[CheckedSchedulabilityClaimRow]>,
    claim_index: &[(usize, usize, usize)],
    owners: &super::model_members::ModelOwners<'_>,
    operation_bodies: Option<&[super::OperationBodyRow]>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    // The site is the caller's again once the claims are checked.
    let caller = meter.site();
    meter.at_node(0, "claim_order");
    // Of two models with one identity the later one is kept, as a map built
    // by insertion would hold; the slice address orders them.
    let mut identified = meter.vec::<(&str, &super::model_members::DomainModel)>(models.len())?;
    identified.extend(models.iter().map(|model| (model.identity.as_ref(), model)));
    let models = SortedTable::from_vec(identified, |left, right| {
        std::ptr::from_ref(*right).cmp(&std::ptr::from_ref(*left))
    });
    let order = claim_order(kinds, index, structural_keys, meter)?;
    for p in order {
        let node = &nodes[p];
        meter.at_node(p, super::graph_storage::node_form(&node.node_tag));
        if node
            .occurrences
            .iter()
            .any(|occurrence| occurrence.role != CheckedOccurrenceRole::Claim)
        {
            return Err(invalid(
                Code::InvalidSemanticGraph,
                None,
                Path::root()
                    .key("semantic_graph")
                    .key("nodes")
                    .index(p)
                    .key("occurrences"),
                &node.node_id,
                meter,
            ));
        }
        let at = Path::root()
            .key("semantic_graph")
            .key("nodes")
            .index(p)
            .key("body");
        let fail =
            |meter: &mut WorkMeter, code, cause, at| invalid(code, cause, at, &node.node_id, meter);
        let body = terms[p].as_ref().expect("graph admission decoded the term");
        // Six complete sweeps preserve the normative first-defect order.
        if !shape(body, CLAIM_FIELDS) {
            return Err(fail(meter, Code::InvalidSemanticGraph, None, at));
        }
        let task_at = binding_at(&at, 4);
        let Some(tasks) = members(admitted_bound(body, 4)).filter(|tasks| !tasks.is_empty()) else {
            return Err(fail(meter, Code::InvalidSemanticGraph, None, task_at));
        };
        for (i, task) in tasks.iter().enumerate() {
            if !shape(task, TASK_FIELDS) {
                return Err(fail(
                    meter,
                    Code::InvalidSemanticGraph,
                    None,
                    task_at.key("value").key("members").index(i),
                ));
            }
        }
        for (i, allowed) in [
            (0, &["schedulable", "response"][..]),
            (1, &["fixed-priority", "edf", "amc"][..]),
        ] {
            if !text(admitted_bound(body, i)).is_some_and(|s| allowed.contains(&s)) {
                return Err(fail(
                    meter,
                    Code::InvalidPackage,
                    Some(Cause::InvalidValue),
                    binding_at(&at, i),
                ));
            }
        }
        for (i, task) in tasks.iter().enumerate() {
            let task_at = task_at.key("value").key("members").index(i);
            for (j, allowed) in [
                (0, &[][..]),
                (1, &["periodic", "sporadic"][..]),
                (7, &["lo", "hi"][..]),
            ] {
                if !text(admitted_bound(task, j))
                    .is_some_and(|s| !s.is_empty() && (allowed.is_empty() || allowed.contains(&s)))
                {
                    return Err(fail(
                        meter,
                        Code::InvalidPackage,
                        Some(Cause::InvalidValue),
                        binding_at(&task_at, j),
                    ));
                }
            }
            let priority = admitted_bound(task, 8);
            if kind(priority, LiteralKind::Integer) && !nonnegative(priority) {
                return Err(fail(
                    meter,
                    Code::InvalidPackage,
                    Some(Cause::InvalidValue),
                    binding_at(&task_at, 8),
                ));
            }
        }
        let response = text(admitted_bound(body, 0)) == Some("response");
        let policy = admitted_text(admitted_bound(body, 1));
        if response && policy != "fixed-priority" {
            return Err(fail(
                meter,
                Code::InvalidSemanticGraph,
                None,
                binding_at(&at, 1),
            ));
        }
        for i in [2, 3] {
            let term = admitted_bound(body, i);
            let fits = if response {
                kind(
                    term,
                    if i == 2 {
                        LiteralKind::Text
                    } else {
                        LiteralKind::Quantity
                    },
                )
            } else {
                empty(term)
            };
            if !fits {
                return Err(fail(
                    meter,
                    Code::InvalidSemanticGraph,
                    None,
                    binding_at(&at, i),
                ));
            }
        }
        for (i, task) in tasks.iter().enumerate() {
            let task_at = task_at.key("value").key("members").index(i);
            for j in [2, 3, 4, 5, 6, 8, 9, 10] {
                let term = admitted_bound(task, j);
                let fits = match j {
                    6 if text(admitted_bound(task, 7)) == Some("lo") => empty(term),
                    8 => empty(term) || kind(term, LiteralKind::Integer),
                    _ => kind(term, LiteralKind::Quantity),
                };
                if !fits {
                    return Err(fail(
                        meter,
                        Code::InvalidSemanticGraph,
                        None,
                        binding_at(&task_at, j),
                    ));
                }
            }
        }
        let mut previous = None;
        for task in tasks {
            let name = admitted_text(admitted_bound(task, 0));
            if previous.is_some_and(|prev| prev >= name) {
                return Err(fail(meter, Code::InvalidSemanticGraph, None, task_at));
            }
            previous = Some(name);
        }
        if response
            && !tasks
                .iter()
                .any(|task| text(admitted_bound(task, 0)) == text(admitted_bound(body, 2)))
        {
            return Err(fail(
                meter,
                Code::InvalidSemanticGraph,
                None,
                binding_at(&at, 2),
            ));
        }
        let mut common_unit = None;
        let times = (response.then(|| (admitted_bound(body, 3), binding_at(&at, 3).key("value"))))
            .into_iter()
            .chain(tasks.iter().enumerate().flat_map(|(i, task)| {
                let task_at = task_at.key("value").key("members").index(i);
                [2, 3, 4, 5, 6, 9, 10].into_iter().filter_map(move |j| {
                    let term = admitted_bound(task, j);
                    kind(term, LiteralKind::Quantity)
                        .then(|| (term, binding_at(&task_at, j).key("value")))
                })
            }));
        for (time, time_at) in times {
            let CheckedTerm::Literal { type_id, .. } = time else {
                return Err(fail(
                    meter,
                    Code::IllTyped,
                    Some(Cause::TypeMismatch),
                    time_at,
                ));
            };
            let valid = index.get(type_id).is_some_and(|&u| {
                kinds[u] == CheckedNodeKind::ScalarType(ScalarTypeForm::Unit)
                    && index.get(&nodes[u].semantic_type).is_some_and(|&d| {
                        kinds[d] == CheckedNodeKind::ScalarType(ScalarTypeForm::Dimension)
                            && super::timed_model::is_base_time_dimension(
                                &nodes[d],
                                terms.get(d).and_then(Option::as_ref),
                                &models,
                            )
                    })
            });
            if !valid || common_unit.is_some_and(|unit| unit != type_id) {
                return Err(fail(
                    meter,
                    Code::IllTyped,
                    Some(Cause::TypeMismatch),
                    time_at,
                ));
            }
            common_unit = Some(type_id);
        }
        let mut priorities: Vec<filament_exact::Integer> = Vec::new();
        for (i, task) in tasks.iter().enumerate() {
            let task_at = task_at.key("value").key("members").index(i);
            let b = |j| admitted_bound(task, j);
            let hi = text(b(7)) == Some("hi");
            let positive = [2, 3, 5]
                .into_iter()
                .all(|j| nonnegative(b(j)) && !zero(b(j)));
            let nonnegative_times =
                [4, 9, 10].into_iter().all(|j| nonnegative(b(j))) && (!hi || nonnegative(b(6)));
            let mut valid = positive && nonnegative_times;
            if valid {
                valid = !greater(b(4), b(if hi { 6 } else { 5 }), meter, &task_at)?
                    && (!hi || !greater(b(5), b(6), meter, &task_at)?);
            }
            if policy == "edf" {
                valid &= zero(b(9)) && zero(b(10));
            } else {
                valid &= record_priority(b(8), &mut priorities, meter)?;
            }
            if valid && policy == "amc" {
                valid = !greater(b(3), b(2), meter, &task_at)?;
            }
            if !valid {
                return Err(fail(
                    meter,
                    Code::InvalidTimedModel,
                    Some(Cause::InvalidTaskSet),
                    task_at,
                ));
            }
        }
    }
    validate_relations(
        claim_rows,
        claim_index,
        nodes,
        kinds,
        terms,
        index,
        structural_keys,
        owners,
        operation_bodies,
        meter,
    )?;
    meter.restore_site(caller);
    Ok(())
}

/// Graph premises precede body and carrier validation.
pub(super) fn validate_graph<Body>(
    nodes: &[CheckedSemanticNodeV2<Body>],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &GraphIndex<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    for &p in index.values() {
        if !claim(kinds[p]) {
            continue;
        }
        let node = &nodes[p];
        meter.at_node(p, "claim");
        for (bad, field) in [
            (!node.dependencies.is_empty(), "dependencies"),
            (node.owner.is_some(), "owner"),
            (
                !index.get(&node.semantic_type).is_some_and(|&t| {
                    kinds[t] == CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean)
                }),
                "semantic_type",
            ),
        ] {
            if bad {
                return Err(invalid(
                    Code::InvalidSemanticGraph,
                    None,
                    Path::root()
                        .key("semantic_graph")
                        .key("nodes")
                        .index(p)
                        .key(field),
                    &node.node_id,
                    meter,
                ));
            }
        }
        if let Some(body) = terms[p].as_ref() {
            let at = [
                Step::Key("semantic_graph"),
                Step::Key("nodes"),
                Step::Index(p),
                Step::Key("body"),
            ];
            validate_literal_values(body, &at, &node.node_id, meter)?;
        }
    }
    Ok(())
}

fn validate_literal_values(
    body: &CheckedTerm,
    at: &[Step<'_>],
    id: &CheckedNodeId,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    // One paid path and a DFS frame list preserve arbitrary admitted depth.
    // No pointer is materialized on the successful path.
    let mut path = meter.vec(at.len())?;
    path.extend_from_slice(at);
    let mut frames = meter.vec(1)?;
    frames.push((body, 0usize, path.len(), false));
    while let Some((term, child, parent_len, entered)) = frames.last_mut() {
        if !*entered {
            *entered = true;
            if let CheckedTerm::Literal {
                value_kind, value, ..
            } = term
            {
                let trail = Trail::Base(&path);
                let value_at = trail.key("value");
                let bad = |meter: &mut WorkMeter| {
                    invalid(
                        Code::InvalidPackage,
                        Some(Cause::InvalidValue),
                        value_at,
                        id,
                        meter,
                    )
                };
                let spelling = match value_kind {
                    LiteralKind::Quantity => Some(value.as_str().ok_or_else(|| bad(meter))?),
                    LiteralKind::Integer => {
                        if let Some(text) = value.as_str() {
                            Some(text)
                        } else if value.as_i64().is_none() && value.as_u64().is_none() {
                            return Err(bad(meter));
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                if let Some(spelling) = spelling {
                    let (numerator, denominator) = if *value_kind == LiteralKind::Quantity {
                        spelling.split_once('/').unwrap_or((spelling, "1"))
                    } else {
                        (spelling, "1")
                    };
                    if !rational_parts_are_canonical(
                        numerator,
                        denominator,
                        meter,
                        super::natural::site(meter, value_at),
                    )? {
                        return Err(bad(meter));
                    }
                }
            }
        }
        let next = match term {
            CheckedTerm::Aggregate { members, .. } => {
                members.get(*child).map(|member| (member, Some(*child)))
            }
            CheckedTerm::Binding { value, .. } if *child == 0 => Some((value.as_ref(), None)),
            _ => None,
        };
        *child += 1;
        if let Some((next, index)) = next {
            let parent = path.len();
            meter.room(&mut path, if index.is_some() { 2 } else { 1 })?;
            path.push(Step::Key(if index.is_some() { "members" } else { "value" }));
            if let Some(index) = index {
                path.push(Step::Index(index));
            }
            meter.push(&mut frames, (next, 0, parent, false))?;
        } else {
            let parent = *parent_len;
            frames.pop();
            path.truncate(parent);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NativeJoinBudget;
    use ix_trace_rs::trace;
    use serde_json::json;

    /// The claim carrier join requests every table before it makes it: the
    /// bytes the allocator is asked for equal the bytes charged, the measured
    /// requirement admits and one byte below is denied (FR-038-AC-504). The
    /// carrier holds only flat wire IDs, so the join's comparisons never
    /// allocate.
    #[trace("TC-472", "FR-038-AC-504")]
    #[trace("TC-3009", "FR-246-AC-5")]
    #[test]
    fn claim_carrier_join_allocates_exactly_what_it_charges() {
        let claim = |name: &str, occurrences: u64| -> super::super::RawSemanticNodeV2 {
            serde_json::from_value(json!({
                "node_id":{"package":"example","path":[name],"ordinal":0},
                "schema_version":"filament.checked-semantic-graph/v2",
                "node_tag":"claim", "semantic_form":"schedulability",
                "semantic_type":{"package":"example","path":["boolean"],"ordinal":0},
                "dependencies":[],
                "occurrences":(0..occurrences)
                    .map(|ordinal| json!({"role":"claim","ordinal":ordinal}))
                    .collect::<Vec<_>>(),
                "body":{"term":"aggregate","members":[]}
            }))
            .expect("claim node")
        };
        let nodes = [claim("a", 2), claim("b", 1), claim("c", 3)];
        let kinds = [CheckedNodeKind::Claim(ClaimForm::Schedulability); 3];
        let index = nodes
            .iter()
            .enumerate()
            .map(|(position, node)| (&node.node_id, position))
            .collect::<GraphIndex<'_>>();
        let keys: Vec<std::sync::Arc<CheckedNodeId>> = nodes
            .iter()
            .map(|node| std::sync::Arc::new(node.node_id.clone()))
            .collect();
        let rows: Vec<CheckedSchedulabilityClaimRow> =
            [(0, 0), (0, 1), (1, 0), (2, 0), (2, 1), (2, 2)]
                .into_iter()
                .map(|(position, occurrence)| CheckedSchedulabilityClaimRow {
                    declaration: nodes[position].node_id.clone(),
                    occurrence,
                    task_set: "Set".into(),
                    release_bindings: Vec::new(),
                    mode_switch_bindings: Vec::new(),
                })
                .collect();
        let run = |limit: u64| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut joined = None;
            let measured = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                joined = Some(validate_carrier(
                    "example",
                    Some(&rows),
                    &nodes,
                    &kinds,
                    &index,
                    &keys,
                    &mut meter,
                ));
            });
            (
                joined.expect("ran").ok(),
                measured.bytes_total,
                budget.read_storage_consumed(),
            )
        };
        let (joined, allocated, charged) = run(u64::MAX);
        assert_eq!(
            joined,
            Some(vec![
                (0, 0, 0),
                (0, 1, 1),
                (1, 0, 2),
                (2, 0, 3),
                (2, 1, 4),
                (2, 2, 5),
            ])
        );
        assert!(charged > 0);
        assert_eq!(allocated, charged, "allocated == requested");
        assert_eq!(run(charged).0, joined, "the measured limit admits");
        assert_eq!(run(charged - 1).0, None, "one byte short is denied");
    }

    /// The exact occurrence relation row is required and has a strict closed shape.
    #[trace("TC-3009", "FR-246-AC-4", "FR-246-AC-5", "FR-038-AC-618")]
    #[test]
    fn schedulability_claim_relation_rows_are_required_and_strict() {
        let declaration = CheckedNodeId {
            package: "example".into(),
            path: vec![super::super::CheckedNodePathSegment::Name("claim".into())],
            ordinal: 0,
        };
        let row = |release_bindings| CheckedSchedulabilityClaimRow {
            declaration: declaration.clone(),
            occurrence: 0,
            task_set: "Tasks".into(),
            release_bindings,
            mode_switch_bindings: Vec::new(),
        };
        let unbacked = serde_json::to_value(row(Vec::new())).expect("serializes empty bindings");
        let release = CheckedSchedulabilityReleaseBinding {
            task_name: "worker".into(),
            selected_model_identity: "model:one".into(),
            operation_declaring_identity: declaration.clone(),
            operation_member: "step".into(),
        };
        let backed = serde_json::to_value(row(vec![release.clone()])).expect("serializes binding");
        assert!(unbacked["release_bindings"].as_array().unwrap().is_empty());
        assert_eq!(backed["release_bindings"][0]["task_name"], "worker");
        assert_eq!(
            serde_json::from_value::<CheckedSchedulabilityClaimRow>(backed.clone())
                .expect("reads binding"),
            row(vec![release])
        );

        for malformed in [
            json!({"declaration": declaration.clone(), "occurrence": 0}),
            json!({"declaration": declaration.clone(), "occurrence": 0, "task_set":"Tasks", "release_bindings":[], "mode_switch_bindings":[], "extra": false}),
            json!({"declaration": declaration, "occurrence": 0, "task_set":"Tasks", "release_bindings":[]}),
        ] {
            assert!(
                serde_json::from_value::<CheckedSchedulabilityClaimRow>(malformed).is_err(),
                "malformed schedulability relation row is rejected"
            );
        }
    }

    /// The model table is one charged exact block the table keeps without
    /// copying: for several models, two sharing an identity, the bytes the
    /// allocator is asked for equal the bytes charged, the measured
    /// requirement admits and one byte below is denied (FR-038-AC-504). The
    /// claim order and the priority set have their own oracles below.
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn model_table_allocates_exactly_what_it_charges() {
        let document = |identity: &str| {
            json!({
                "contractVersion":"2.0.0",
                "package":{"identity":identity,"version":"1.0.0"},
                "constructs":[], "types":[], "dimensions":[], "units":[]
            })
        };
        let unit_refs = std::sync::Arc::new(Default::default());
        let read = |identity: &str| {
            let selected =
                crate::CheckedJson::from_serializable(&document(identity)).expect("a document");
            let mut meter = WorkMeter::new(u64::MAX);
            let mut budget = super::super::model_members::Budget::new(&mut meter, 0, 1 << 20);
            super::super::model_members::read_semantic_ir_of(
                selected.document(),
                "example",
                &mut budget,
                &unit_refs,
            )
            .expect("selected declarations")
        };
        let models = [read("acme/one"), read("acme/two"), read("acme/one")];
        let index = std::iter::empty::<(&CheckedNodeId, usize)>().collect::<GraphIndex<'_>>();
        let run = |limit: u64| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut outcome = None;
            let measured = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                outcome = Some(validate_bodies::<CheckedTerm>(
                    &[],
                    &[],
                    &[],
                    &index,
                    &[],
                    &models,
                    &mut meter,
                ));
            });
            (
                outcome.expect("ran").ok(),
                measured.bytes_total,
                budget.read_storage_consumed(),
            )
        };
        let (outcome, allocated, charged) = run(u64::MAX);
        assert_eq!(outcome, Some(()));
        assert!(charged > 0);
        assert_eq!(allocated, charged, "allocated == requested");
        assert_eq!(run(charged).0, outcome, "the measured limit admits");
        assert_eq!(run(charged - 1).0, None, "one byte short is denied");
    }

    /// The claim order is one exact block charged before it is made, at the
    /// graph locus: the bytes the allocator is asked for equal the bytes
    /// charged, the measured requirement admits and one byte below is denied
    /// at `PackageRead(Graph, 0)` (FR-038-AC-504). Claims are listed against
    /// the key order and mixed with other nodes.
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn claim_order_allocates_exactly_what_it_charges() {
        use super::super::native_join_budget::{
            NativeJoinLocus, NativeJoinLocusSource, PackageReadStage,
        };
        let id = |name: &str| CheckedNodeId {
            package: "example".into(),
            path: vec![crate::CheckedNodePathSegment::Name(name.into())],
            ordinal: 0,
        };
        let names = ["m", "c", "x", "a", "z", "k", "b"];
        let ids: Vec<CheckedNodeId> = names.iter().map(|name| id(name)).collect();
        let kinds: Vec<CheckedNodeKind> = names
            .iter()
            .map(|name| {
                if matches!(*name, "x" | "k") {
                    CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean)
                } else {
                    CheckedNodeKind::Claim(ClaimForm::Schedulability)
                }
            })
            .collect();
        let index = ids
            .iter()
            .enumerate()
            .map(|(position, id)| (id, position))
            .collect::<GraphIndex<'_>>();
        let keys: Vec<std::sync::Arc<CheckedNodeId>> =
            ids.iter().cloned().map(std::sync::Arc::new).collect();
        let run = |limit: u64| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut order = None;
            let measured = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                meter.at_node(0, "claim_order");
                order = Some(claim_order(&kinds, &index, &keys, &mut meter));
            });
            (
                order.expect("ran").ok(),
                measured.bytes_total,
                budget.read_storage_consumed(),
                budget.first_unpaid_locus(),
            )
        };
        let (order, allocated, charged, _) = run(u64::MAX);
        let names_in_order: Vec<&str> = order
            .as_ref()
            .expect("the order")
            .iter()
            .map(|&p| names[p])
            .collect();
        assert_eq!(names_in_order, ["a", "b", "c", "m", "z"]);
        assert!(charged > 0);
        assert_eq!(allocated, charged, "allocated == requested");
        assert_eq!(run(charged).0, order, "the measured limit admits");
        let (short, _, _, locus) = run(charged - 1);
        assert_eq!(short, None, "one byte short is denied");
        assert_eq!(
            locus,
            Some(NativeJoinLocus {
                source: NativeJoinLocusSource::PackageRead(PackageReadStage::Graph, 0),
                form: "claim_order",
            })
        );
    }

    /// The priority set grows under the meter and its values are read by a
    /// parse that charges its limbs: for priorities that repeat, are
    /// written as JSON numbers, exceed 128 bits or are not integers, the
    /// bytes the allocator is asked for equal the bytes charged, the
    /// measured requirement admits and one byte below is denied at the
    /// claim's graph locus (FR-038-AC-504).
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn priority_set_allocates_exactly_what_it_charges() {
        use super::super::native_join_budget::{NativeJoinLocusSource, PackageReadStage};
        let integer = |value: serde_json::Value| CheckedTerm::Literal {
            type_id: CheckedNodeId {
                package: "example".into(),
                path: vec![crate::CheckedNodePathSegment::Name("integer".into())],
                ordinal: 0,
            },
            value_kind: LiteralKind::Integer,
            value: crate::CheckedJson::from_serializable(&value).expect("a canonical literal"),
        };
        let huge = format!("1{}", "0".repeat(60));
        let mut terms: Vec<CheckedTerm> = (0..40_u64)
            .map(|n| integer(json!(((n * 7) % 40).to_string())))
            .collect();
        terms.push(integer(json!(huge)));
        terms.push(integer(json!(huge)));
        terms.push(integer(json!(12345)));
        terms.push(integer(json!(null)));
        terms.push(CheckedTerm::Aggregate {
            members: Vec::new(),
            temporal_profile: None,
        });
        let run = |limit: u64| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            // The record of outcomes is made before the measure starts.
            let mut outcome = Some(Vec::with_capacity(terms.len()));
            let mut priorities = Vec::new();
            let measured = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                meter.at_node(3, "claim");
                for term in &terms {
                    match record_priority(term, &mut priorities, &mut meter) {
                        Ok(recorded) => outcome.as_mut().expect("running").push(recorded),
                        Err(_) => {
                            outcome = None;
                            break;
                        }
                    }
                }
            });
            let held = priorities.len();
            drop(priorities);
            (
                outcome,
                held,
                measured.bytes_total,
                budget.read_storage_consumed(),
                budget.first_unpaid_locus(),
            )
        };
        let (outcome, held, allocated, charged, _) = run(u64::MAX);
        let outcome = outcome.expect("every priority is read");
        // 40 distinct small values, the huge one and its repeat, a JSON
        // number, a null and an aggregate.
        assert_eq!(held, 42);
        assert!(outcome[..40].iter().all(|&recorded| recorded));
        assert_eq!(outcome[40..], [true, false, true, false, false]);
        assert!(charged > 0);
        assert_eq!(allocated, charged, "allocated == requested");
        assert!(run(charged).0.is_some(), "the measured limit admits");
        let (short, _, _, _, locus) = run(charged - 1);
        assert!(short.is_none(), "one byte short is denied");
        // The set's growth is charged as the claim; a limb block is charged
        // as the exact-number library that parses it.
        let locus = locus.expect("a denied request has a locus");
        assert_eq!(
            locus.source,
            NativeJoinLocusSource::PackageRead(PackageReadStage::Graph, 3)
        );
        assert!(matches!(locus.form, "claim" | "exact_number"));
    }
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn deep_claim_numeric_walk_preserves_exact_values_and_prepays_frames() {
        let id = super::super::test_support::node_id('a');
        let integer = |value: &str| CheckedTerm::Literal {
            type_id: id.clone(),
            value_kind: LiteralKind::Integer,
            value: super::super::CheckedJson::scalar(&json!(value)),
        };
        let mut term = integer("170141183460469231731687303715884105728");
        for _ in 0..40 {
            term = CheckedTerm::Aggregate {
                members: vec![term],
                temporal_profile: None,
            };
        }
        let run = |limit| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let measured = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                meter.at_node(7, "claim");
                result = Some(validate_literal_values(
                    &term,
                    &[Step::Key("body")],
                    &id,
                    &mut meter,
                ));
            });
            (
                result.expect("the numeric walk ran"),
                measured.bytes_total,
                budget,
            )
        };
        let (result, allocated, budget) = run(u64::MAX);
        assert!(result.is_ok());
        let required = budget.read_storage_consumed();
        assert!(required > 0);
        assert_eq!(allocated, required);
        assert!(run(required).0.is_ok());
        let (short, allocated, budget) = run(required - 1);
        assert!(matches!(short, Err(ValidationFailure::Budget(_))));
        assert!(
            allocated <= budget.read_storage_consumed(),
            "denial cannot allocate unpaid material"
        );
        let (zero, allocated, _) = run(0);
        assert!(matches!(zero, Err(ValidationFailure::Budget(_))));
        assert_eq!(allocated, 0);
        let mut meter = WorkMeter::new(u64::MAX);
        let at = Site::new(PackageReadStage::Graph, 0, "claim", Path::root());
        assert!(!rational_parts_are_canonical(
            "170141183460469231731687303715884105728",
            "2",
            &mut meter,
            at
        )
        .expect("exact gcd"));
        assert!(rational_parts_are_canonical(
            "170141183460469231731687303715884105729",
            "2",
            &mut meter,
            at
        )
        .expect("exact gcd"));
        assert!(rational_parts_are_canonical(
            "-170141183460469231731687303715884105729",
            "1",
            &mut meter,
            at
        )
        .expect("signed exact integer"));
    }
    #[trace("TC-472", "FR-038-AC-510")]
    #[test]
    fn deep_numeric_fault_keeps_the_first_leaf_and_charges_its_full_path() {
        let id = super::super::test_support::node_id('a');
        let literal = |spelling: &str| CheckedTerm::Literal {
            type_id: id.clone(),
            value_kind: LiteralKind::Integer,
            value: super::super::CheckedJson::scalar(&json!(spelling)),
        };
        let mut body = CheckedTerm::Aggregate {
            members: vec![literal("01"), literal("-0")],
            temporal_profile: None,
        };
        for _ in 0..40 {
            body = CheckedTerm::Aggregate {
                members: vec![body],
                temporal_profile: None,
            };
        }
        let expected = format!("/body{}/members/0/value", "/members/0".repeat(40));
        let run = |limit| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let observed = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                meter.at_node(7, "claim");
                result = Some(validate_literal_values(
                    &body,
                    &[Step::Key("body")],
                    &id,
                    &mut meter,
                ));
            });
            (
                result.expect("numeric validation ran"),
                observed.bytes_total,
                budget,
            )
        };
        let check = |result| {
            let Err(ValidationFailure::Refused(refusal)) = result else {
                panic!("first invalid leaf refuses");
            };
            assert_eq!(refusal.code, Code::InvalidPackage);
            assert_eq!(refusal.cause, Some(Cause::InvalidValue));
            assert_eq!(refusal.locus.as_ref(), Some(&id));
            assert_eq!(
                refusal.loci[0].path.as_ref().expect("path").as_str(),
                expected
            );
        };
        let (result, allocated, budget) = run(u64::MAX);
        check(result);
        let required = budget.read_storage_consumed();
        assert_eq!(allocated, required);
        check(run(required).0);
        let (short, allocated, budget) = run(required - 1);
        assert!(matches!(short, Err(ValidationFailure::Budget(_))));
        assert!(allocated <= budget.read_storage_consumed());
    }
}
