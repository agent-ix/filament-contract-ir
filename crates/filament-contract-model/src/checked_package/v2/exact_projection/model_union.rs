// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-830: the selected model-union witness rows and the native join that
//! admits them beside the selected model-tuple rows.
//!
//! The reader validates consistency and preserves the producer's opaque
//! identities; it does NOT claim provenance authentication from parsing. A row
//! set that passes every join is admitted whether or not the producer that wrote
//! it held the declaration.

use super::super::model_members::{
    CollectionKind as CheckedCollectionKind, DeclarationForm, MemberType,
};
use super::super::native_join_storage as storage;
use super::super::{
    CheckedNodeId, CheckedPackageV2, NativeJoinBudget, NativeJoinLocus, NativeJoinLocusSource,
};
use super::{
    charge_native_storage, charge_native_work, exact_diagnostic, ExactDiagnosticForm,
    ExactTypeProjectionError, ExactTypeProjectionErrorCode, SelectedExactIdentities,
    SelectedExactIdentity, SelectedTupleDeclarationRow,
};
use filament_exact::{
    CardinalityBound, CarriedQuantityType, CollectionKind, CollectionType, FloatType, IeeeWidth,
    Integer, IntegerInterval, NodeKey, OriginalDeclarationKey, QuantityCarrier, RationalDomain,
    TextProfile, TextType, UnionMember, UnitId, ValueType,
};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// The identity of the Clock native type a clock payload carries.
const CLOCK_IDENTITY: &str = "ix://filament/native/Clock";

type Res<T> = Result<T, ExactTypeProjectionError>;

/// One case's payload, in one of the three closed forms of FR-830.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectedModelUnionPayload {
    /// A nullary case with no `payloadType`.
    None,
    /// A case whose alias-resolved payload is any admitted type but Clock.
    Data {
        /// The package node FR-322 step 4 gives the payload type.
        node: CheckedNodeId,
        /// The producer's original value type of the payload.
        original_type: ValueType,
    },
    /// A case whose payload is Clock: data arity 0, never a nullary case, and
    /// carrying neither a node nor an original value type.
    Clock {
        /// The Clock identity, `ix://filament/native/Clock`.
        payload_type: Box<str>,
    },
}

/// One complete case of a selected model union.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedModelUnionCaseRow {
    /// The case's index in ascending UTF-8 byte order of `name`.
    pub rank: u64,
    /// The case's authored index among the selected document's variants.
    pub position: u64,
    /// The case name.
    pub name: Box<str>,
    /// The selected document's declaration key of the case; never an enum
    /// `VariantId` and never a case node.
    pub original_case: OriginalDeclarationKey,
    /// The sealed union member: `declaration` equals the row's `settled_key`.
    pub member: UnionMember,
    /// The case payload.
    pub payload: SelectedModelUnionPayload,
}

/// One selected model union's producer witness.
///
/// IR validates the structural join, the case census and the payload
/// correspondence. The opaque keys are the producer's: a caller-created row
/// establishes consistency only, never provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedModelUnionRow {
    /// The union's `model`/`variant_type` node.
    pub structural_node: CheckedNodeId,
    /// The identity of the node's `ModelOwner`.
    pub selected_document: Box<str>,
    /// The producer's original composite type key, named by `composite`.
    pub type_handle: NodeKey,
    /// The settled declaration key the sealed members carry.
    pub settled_key: NodeKey,
    /// Every case, in ascending UTF-8 byte order of name.
    pub cases: Vec<SelectedModelUnionCaseRow>,
}

/// The extent of one model-union join.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelUnionScope<'a> {
    /// Every union-family and tuple node of the package needs its row.
    Full,
    /// Only the closure of this node needs rows.
    Reached(&'a CheckedNodeId),
}

fn fail(
    budget: &mut NativeJoinBudget,
    code: ExactTypeProjectionErrorCode,
    node: &CheckedNodeId,
    pointer: &str,
) -> ExactTypeProjectionError {
    exact_diagnostic(
        budget,
        code,
        node,
        Some(node),
        ExactDiagnosticForm::Literal(pointer),
    )
}

/// A row names a node the package does not hold: `unknown_node` for a node of
/// this package or of a selected model document, `foreign_node` for any other
/// package. This classification is local to the FR-830 channels; the shared
/// selected-declaration helpers keep their own.
fn absent_node(
    package: &CheckedPackageV2,
    budget: &mut NativeJoinBudget,
    node: &CheckedNodeId,
    pointer: &str,
) -> ExactTypeProjectionError {
    let code = if node.package == package.wire.name
        || package.models.selected_model(&node.package).is_some()
    {
        ExactTypeProjectionErrorCode::UnknownNode
    } else {
        ExactTypeProjectionErrorCode::ForeignNode
    };
    fail(budget, code, node, pointer)
}

fn node_units(node: &CheckedNodeId) -> u128 {
    storage::node(node)
}

fn row_units(row: &SelectedModelUnionRow) -> u128 {
    storage::model_union(row)
}

fn at_locus(budget: &mut NativeJoinBudget, source: NativeJoinLocusSource, form: &'static str) {
    budget.set_active_locus(Some(NativeJoinLocus { source, form }));
}

/// The one package node a member type's edge names, when it names one.
fn member_target(member: &MemberType) -> Option<&CheckedNodeId> {
    let mut current = member;
    loop {
        match current {
            MemberType::Option(inner) => current = inner,
            MemberType::Collection { element, .. } => current = element,
            MemberType::Record(id)
            | MemberType::Tuple(id)
            | MemberType::Variant(id)
            | MemberType::Reference(id) => return Some(id),
            _ => return None,
        }
    }
}

/// The sidecar channel a handle owner row belongs to; tuples are joined first.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Channel {
    Tuple,
    Union,
}

/// The owner of one indexed key handle.
#[derive(Clone, Copy)]
enum Owner<'a> {
    /// A witness this selection already retains, by its structural node.
    Retained(&'a CheckedNodeId),
    /// A supplied row, by channel and its original sidecar index.
    Row(Channel, usize),
}

/// Every key handle of every retained witness and of every supplied tuple and
/// union row, indexed once with borrowed keys.
struct HandleIndex<'a>(BTreeMap<&'a NodeKey, Vec<Owner<'a>>>);

impl<'a> HandleIndex<'a> {
    /// Index one handle: its work, then its container storage, before it lands.
    fn add(
        &mut self,
        first: &CheckedNodeId,
        at: &CheckedNodeId,
        key: &'a NodeKey,
        owner: Owner<'a>,
        form: &'static str,
        budget: &mut NativeJoinBudget,
    ) -> Res<()> {
        charge_native_work(budget, first, at, form, 1)?;
        let present = self.0.contains_key(key);
        charge_native_storage(budget, first, at, form, if present { 1 } else { 3 })?;
        self.0.entry(key).or_default().push(owner);
        Ok(())
    }
}

/// The nodes a join inspects: all of them, or a closure.
enum InScope<'c, 'p> {
    Full,
    Walked(BTreeSet<&'p CheckedNodeId>),
    Given(&'c BTreeSet<CheckedNodeId>),
}

impl InScope<'_, '_> {
    fn contains(&self, node: &CheckedNodeId) -> bool {
        match self {
            Self::Full => true,
            Self::Walked(set) => set.contains(node),
            Self::Given(set) => set.contains(node),
        }
    }
}

/// The internal extent of one join: the public scope, or a closure the caller
/// already walked under the same budget.
#[derive(Clone, Copy)]
pub(in crate::checked_package::v2) enum Extent<'a> {
    Full,
    Requested(&'a CheckedNodeId),
    Set(&'a BTreeSet<CheckedNodeId>),
}

/// How one coverage node refuses when it has no row.
struct Coverage {
    pointer: &'static str,
    tuple: bool,
}

impl SelectedExactIdentities {
    fn runtime_denied(budget: &NativeJoinBudget) -> ExactTypeProjectionError {
        let denied = budget
            .first_unpaid()
            .expect("a denied runtime type read records its unpaid charge");
        ExactTypeProjectionError::compact_native(budget, denied.kind, denied.amount)
    }

    /// The members a visited declaration scans: its cases, tuple positions or
    /// effective fields, plus the node itself.
    fn model_union_scan_units(package: &CheckedPackageV2, id: &CheckedNodeId) -> u128 {
        let Some(entry) = package.models.selected_declaration_entry(id) else {
            return 1;
        };
        let Some(model) = package.models.selected_models().get(entry.selection) else {
            return 1;
        };
        let members = match entry.form {
            DeclarationForm::VariantType => model
                .variant_types
                .get(&entry.node)
                .map_or(0, |cases| cases.len()),
            DeclarationForm::TupleValueType => model
                .tuple_types
                .get(&entry.node)
                .map_or(0, |positions| positions.len()),
            DeclarationForm::RecordValueType => package
                .models
                .selected_record_fields(id)
                .map_or(0, |(_, fields)| fields.len()),
            DeclarationForm::ObjectType => package
                .models
                .selected_object_fields(id)
                .map_or(0, |(_, fields)| fields.len()),
            _ => 0,
        };
        1u128.saturating_add(u128::try_from(members).unwrap_or(u128::MAX))
    }

    /// Enter one newly visited declaration: its whole scan (node, cases,
    /// positions or fields) is paid as work and its set entry and stack slot as
    /// storage before either is made. The node is borrowed from the package.
    fn model_union_enter<'p>(
        package: &CheckedPackageV2,
        target: &'p CheckedNodeId,
        start: &CheckedNodeId,
        visited: &mut BTreeSet<&'p CheckedNodeId>,
        pending: &mut Vec<&'p CheckedNodeId>,
        budget: &mut NativeJoinBudget,
    ) -> Res<()> {
        let units = Self::model_union_scan_units(package, target);
        charge_native_work(budget, start, target, "model_union_closure_node", units)?;
        charge_native_storage(
            budget,
            start,
            target,
            "model_union_closure",
            units.saturating_add(3),
        )?;
        visited.insert(target);
        pending.push(target);
        Ok(())
    }

    /// Follow one typed member's edge. One edge per typed member is paid (a
    /// scalar leaf's edge ends at its type node); only a declaration or
    /// reference target is entered, and the edge is the last charge of its step.
    fn model_union_follow<'p>(
        package: &'p CheckedPackageV2,
        member: &MemberType,
        at: &CheckedNodeId,
        start: &CheckedNodeId,
        visited: &mut BTreeSet<&'p CheckedNodeId>,
        pending: &mut Vec<&'p CheckedNodeId>,
        budget: &mut NativeJoinBudget,
    ) -> Res<()> {
        // A Clock payload names no node: it has no edge.
        if matches!(member, MemberType::Clock) {
            return Ok(());
        }
        if let Some(node) = member_target(member).and_then(|target| package.node(target)) {
            if !visited.contains(&node.node_id) {
                Self::model_union_enter(package, &node.node_id, start, visited, pending, budget)?;
            }
        }
        charge_native_work(budget, start, at, "model_union_closure_edge", 1)
    }

    /// The closure of `start`: case payloads, tuple positions and record or
    /// object field types, each node expanded once, each edge examined once.
    /// Every set entry and stack slot is paid before it is made; the nodes are
    /// borrowed from the package, never cloned.
    fn model_union_closure<'p>(
        package: &'p CheckedPackageV2,
        start: &CheckedNodeId,
        budget: &mut NativeJoinBudget,
    ) -> Res<BTreeSet<&'p CheckedNodeId>> {
        at_locus(
            budget,
            NativeJoinLocusSource::Package,
            "model_union_closure",
        );
        let Some(start_node) = package.node(start) else {
            return Err(absent_node(package, budget, start, "/scope/request"));
        };
        charge_native_storage(budget, start, start, "model_union_closure", 5)?;
        let mut visited = BTreeSet::new();
        let mut pending = Vec::new();
        Self::model_union_enter(
            package,
            &start_node.node_id,
            start,
            &mut visited,
            &mut pending,
            budget,
        )?;
        while let Some(id) = pending.pop() {
            let Some(entry) = package.models.selected_declaration_entry(id) else {
                continue;
            };
            let Some(model) = package.models.selected_models().get(entry.selection) else {
                continue;
            };
            match entry.form {
                DeclarationForm::VariantType => {
                    if let Some(cases) = model.variant_types.get(&entry.node) {
                        for case in cases.values() {
                            let Some(payload) = &case.payload else {
                                continue;
                            };
                            at_locus(
                                budget,
                                NativeJoinLocusSource::Package,
                                "model_union_closure_type",
                            );
                            let member = model
                                .element_type_runtime(payload, budget)
                                .map_err(|_| Self::runtime_denied(budget))?;
                            if let Some(member) = member {
                                Self::model_union_follow(
                                    package,
                                    &member,
                                    id,
                                    start,
                                    &mut visited,
                                    &mut pending,
                                    budget,
                                )?;
                            }
                        }
                    }
                }
                DeclarationForm::TupleValueType => {
                    if let Some(positions) = model.tuple_types.get(&entry.node) {
                        for position in positions {
                            at_locus(
                                budget,
                                NativeJoinLocusSource::Package,
                                "model_union_closure_type",
                            );
                            let member = model
                                .element_type_runtime(position, budget)
                                .map_err(|_| Self::runtime_denied(budget))?;
                            if let Some(member) = member {
                                Self::model_union_follow(
                                    package,
                                    &member,
                                    id,
                                    start,
                                    &mut visited,
                                    &mut pending,
                                    budget,
                                )?;
                            }
                        }
                    }
                }
                DeclarationForm::RecordValueType => {
                    if let Some((_, fields)) = package.models.selected_record_fields(id) {
                        for field in fields {
                            if let Some(member) = field.member_type {
                                Self::model_union_follow(
                                    package,
                                    member,
                                    id,
                                    start,
                                    &mut visited,
                                    &mut pending,
                                    budget,
                                )?;
                            }
                        }
                    }
                }
                DeclarationForm::ObjectType => {
                    if let Some((_, fields)) = package.models.selected_object_fields(id) {
                        for field in fields {
                            if let Some(member) = field.member_type {
                                Self::model_union_follow(
                                    package,
                                    member,
                                    id,
                                    start,
                                    &mut visited,
                                    &mut pending,
                                    budget,
                                )?;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(visited)
    }

    /// Index every retained witness handle and the handles of every supplied
    /// tuple and union row, across both channels, before the first row.
    fn model_union_handle_index<'a>(
        &'a self,
        tuple_rows: &'a [SelectedTupleDeclarationRow],
        union_rows: &'a [SelectedModelUnionRow],
        first: &CheckedNodeId,
        budget: &mut NativeJoinBudget,
    ) -> Res<HandleIndex<'a>> {
        at_locus(
            budget,
            NativeJoinLocusSource::Package,
            "model_union_handle_index",
        );
        charge_native_storage(budget, first, first, "model_union_handle_index", 2)?;
        let mut index = HandleIndex(BTreeMap::new());
        let form = "model_union_handle";
        for row in self.declarations.values() {
            let node = &row.structural_node;
            match &row.exact_identity {
                SelectedExactIdentity::Record(key)
                | SelectedExactIdentity::Union(key)
                | SelectedExactIdentity::DeclaredUnit(UnitId::Declared(key)) => {
                    index.add(first, node, key, Owner::Retained(node), form, budget)?
                }
                _ => charge_native_work(budget, first, node, form, 1)?,
            }
        }
        for (id, row) in self.admitted_tuples.iter() {
            index.add(first, id, &row.node_key, Owner::Retained(id), form, budget)?;
        }
        for (id, row) in self.admitted_source_units.iter() {
            if let UnitId::Declared(key) = &row.unit_id {
                index.add(first, id, key, Owner::Retained(id), form, budget)?;
            }
        }
        for (id, row) in self.source_unions.iter() {
            index.add(
                first,
                id,
                &row.type_handle,
                Owner::Retained(id),
                form,
                budget,
            )?;
            index.add(
                first,
                id,
                &row.settled_key,
                Owner::Retained(id),
                form,
                budget,
            )?;
        }
        for (id, row) in self.source_records.iter() {
            index.add(
                first,
                id,
                &row.record_key,
                Owner::Retained(id),
                form,
                budget,
            )?;
        }
        for (id, row) in self.source_tuples.iter() {
            index.add(
                first,
                id,
                &row.type_handle,
                Owner::Retained(id),
                form,
                budget,
            )?;
        }
        for (id, row) in self.source_enums.iter() {
            let key = &row.declaration.node_key;
            index.add(first, id, key, Owner::Retained(id), form, budget)?;
        }
        for (id, row) in self.model_enum_shapes.iter() {
            index.add(first, id, &row.node_key, Owner::Retained(id), form, budget)?;
            for case in &row.cases {
                let node = &case.structural_node;
                let key = &case.member_key;
                index.add(first, node, key, Owner::Retained(node), form, budget)?;
            }
        }
        for (id, row) in self.model_unions.iter() {
            index.add(
                first,
                id,
                &row.type_handle,
                Owner::Retained(id),
                form,
                budget,
            )?;
            index.add(
                first,
                id,
                &row.settled_key,
                Owner::Retained(id),
                form,
                budget,
            )?;
        }
        for (k, row) in tuple_rows.iter().enumerate() {
            at_locus(
                budget,
                NativeJoinLocusSource::ModelTupleRow(k),
                "model_tuple_handle",
            );
            let owner = Owner::Row(Channel::Tuple, k);
            let node = &row.structural_node;
            index.add(
                first,
                node,
                &row.node_key,
                owner,
                "model_tuple_handle",
                budget,
            )?;
        }
        for (k, row) in union_rows.iter().enumerate() {
            at_locus(
                budget,
                NativeJoinLocusSource::ModelUnionRow(k),
                "model_union_handle",
            );
            let owner = Owner::Row(Channel::Union, k);
            let node = &row.structural_node;
            index.add(first, node, &row.type_handle, owner, form, budget)?;
            if row.settled_key != row.type_handle {
                index.add(first, node, &row.settled_key, owner, form, budget)?;
            }
        }
        Ok(index)
    }

    /// Join the selected model-tuple rows and then the selected model-union
    /// rows to this package under the caller's one shared budget.
    ///
    /// Tuple rows are joined first because a union payload reads its tuple's
    /// witness. Under [`ModelUnionScope::Reached`] only rows of the closure of
    /// the requested node are joined, inspected and retained; the preflight
    /// handle index and the common pass still read and charge every supplied row.
    /// The result is published whole or not at all, and a refusal or exhaustion
    /// leaves the charges already admitted consumed. The caller's budget is
    /// never reset; its previous active locus is restored on return.
    ///
    /// The reader validates consistency and preserves the producer's opaque
    /// identities; it does NOT claim provenance authentication from parsing.
    pub fn with_model_union_rows(
        &self,
        package: &CheckedPackageV2,
        tuple_rows: &[SelectedTupleDeclarationRow],
        union_rows: &[SelectedModelUnionRow],
        scope: ModelUnionScope<'_>,
        budget: &mut NativeJoinBudget,
    ) -> Result<Self, ExactTypeProjectionError> {
        let first = match scope {
            ModelUnionScope::Reached(requested) => Some(requested),
            ModelUnionScope::Full => None,
        }
        .or_else(|| tuple_rows.first().map(|row| &row.structural_node))
        .or_else(|| union_rows.first().map(|row| &row.structural_node))
        .or_else(|| package.resolved_graph().nodes.first().map(|n| &n.node_id));
        let Some(first) = first else {
            return Ok(self.clone());
        };
        let extent = match scope {
            ModelUnionScope::Full => Extent::Full,
            ModelUnionScope::Reached(requested) => Extent::Requested(requested),
        };
        self.join_model_unions_framed(package, tuple_rows, union_rows, extent, first, budget)
    }

    fn join_model_unions_framed(
        &self,
        package: &CheckedPackageV2,
        tuple_rows: &[SelectedTupleDeclarationRow],
        union_rows: &[SelectedModelUnionRow],
        extent: Extent<'_>,
        first: &CheckedNodeId,
        budget: &mut NativeJoinBudget,
    ) -> Res<Self> {
        let before = budget.begin();
        let previous_locus = budget.active_locus();
        let result = self.join_model_unions(package, tuple_rows, union_rows, extent, first, budget);
        budget.set_active_locus(previous_locus);
        match &result {
            Ok(_) => budget.finish(before),
            Err(_) => budget.abort(),
        }
        result.map_err(|mut error| {
            error.work = budget.consumed().work;
            error
        })
    }

    fn join_model_unions(
        &self,
        package: &CheckedPackageV2,
        tuple_rows: &[SelectedTupleDeclarationRow],
        union_rows: &[SelectedModelUnionRow],
        extent: Extent<'_>,
        first: &CheckedNodeId,
        budget: &mut NativeJoinBudget,
    ) -> Res<Self> {
        at_locus(
            budget,
            NativeJoinLocusSource::Package,
            "model_union_binding",
        );
        self.check_binding(package, budget, first)?;
        // The one clone of the identities this join returns, paid once.
        charge_native_storage(
            budget,
            first,
            first,
            "selected_identity_handles",
            storage::selected_clone(),
        )?;
        // Preflight: the handle index of every retained and supplied witness.
        let index = self.model_union_handle_index(tuple_rows, union_rows, first, budget)?;
        // Common pass: every row of both channels, whatever the scope.
        for (k, row) in tuple_rows.iter().enumerate() {
            let id = &row.structural_node;
            at_locus(
                budget,
                NativeJoinLocusSource::ModelTupleRow(k),
                "model_tuples_row",
            );
            charge_native_work(
                budget,
                id,
                id,
                "model_tuples_row",
                node_units(id)
                    .saturating_add(storage::text_parts(&[&row.selected_document]))
                    .saturating_add(storage::text_parts(&[&row.original_identity])),
            )?;
        }
        for (k, row) in union_rows.iter().enumerate() {
            let id = &row.structural_node;
            at_locus(
                budget,
                NativeJoinLocusSource::ModelUnionRow(k),
                "model_unions_row",
            );
            charge_native_work(budget, id, id, "model_unions_row", row_units(row))?;
        }
        let scope = match extent {
            Extent::Full => InScope::Full,
            Extent::Requested(requested) => {
                InScope::Walked(Self::model_union_closure(package, requested, budget)?)
            }
            Extent::Set(set) => InScope::Given(set),
        };
        let in_row = |channel: Channel, k: usize| match channel {
            Channel::Tuple => scope.contains(&tuple_rows[k].structural_node),
            Channel::Union => scope.contains(&union_rows[k].structural_node),
        };
        let mut selected = self.clone();
        if selected.binding.is_none() {
            selected.binding = package.admission_binding.clone();
        }
        selected.join_tuple_stage(package, tuple_rows, &scope, &index, &in_row, first, budget)?;
        selected.join_union_stage(
            self, package, union_rows, &scope, &index, &in_row, first, budget,
        )?;
        selected.check_coverage(package, &scope, extent, first, budget)?;
        Ok(selected)
    }

    /// Whether `key` collides with a handle that an earlier joined row or a
    /// retained witness already owns. A supplied row outside the scope is never
    /// inspected, so its handle is indexed and paid for but never refuses.
    #[allow(clippy::too_many_arguments)]
    fn model_union_collides(
        index: &HandleIndex<'_>,
        key: &NodeKey,
        me: (Channel, usize),
        exempt: Option<&CheckedNodeId>,
        in_row: &dyn Fn(Channel, usize) -> bool,
        first: &CheckedNodeId,
        at: &CheckedNodeId,
        budget: &mut NativeJoinBudget,
    ) -> Res<bool> {
        let Some(owners) = index.0.get(key) else {
            return Ok(false);
        };
        charge_native_work(
            budget,
            first,
            at,
            "model_union_handle_probe",
            u128::try_from(owners.len()).unwrap_or(u128::MAX),
        )?;
        Ok(owners.iter().any(|owner| match *owner {
            Owner::Retained(node) => exempt != Some(node),
            Owner::Row(channel, k) => (channel, k) < me && in_row(channel, k),
        }))
    }

    /// The model-tuple stage: every tuple row of the scope, in sidecar order,
    /// with the sidecar pointer of the row's original index.
    #[allow(clippy::too_many_arguments)]
    fn join_tuple_stage(
        &mut self,
        package: &CheckedPackageV2,
        tuple_rows: &[SelectedTupleDeclarationRow],
        scope: &InScope<'_, '_>,
        index: &HandleIndex<'_>,
        in_row: &dyn Fn(Channel, usize) -> bool,
        first: &CheckedNodeId,
        budget: &mut NativeJoinBudget,
    ) -> Res<()> {
        charge_native_storage(budget, first, first, "model_tuples_seen", 2)?;
        let mut seen: BTreeSet<&CheckedNodeId> = BTreeSet::new();
        for (k, row) in tuple_rows.iter().enumerate() {
            let id = &row.structural_node;
            if !scope.contains(id) {
                continue;
            }
            at_locus(
                budget,
                NativeJoinLocusSource::ModelTupleRow(k),
                "model_tuple_row",
            );
            let at = |tail: &str| format!("/model_tuples/{k}{tail}");
            charge_native_work(budget, id, id, "model_tuple_row", 1)?;
            let Some((document, original, form)) = package.models.selected_declaration(id) else {
                return Err(if package.node(id).is_some() {
                    fail(
                        budget,
                        ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                        id,
                        &at("/structural_node"),
                    )
                } else {
                    absent_node(package, budget, id, &at("/structural_node"))
                });
            };
            charge_native_work(
                budget,
                id,
                id,
                "model_tuple_document",
                storage::text_parts(&[document, &row.selected_document]),
            )?;
            charge_native_work(
                budget,
                id,
                id,
                "model_tuple_original",
                storage::text_parts(&[original, &row.original_identity]),
            )?;
            for (wrong, tail) in [
                (form != DeclarationForm::TupleValueType, "/structural_node"),
                (
                    document != row.selected_document.as_ref(),
                    "/selected_document",
                ),
                (
                    original != row.original_identity.as_ref(),
                    "/original_identity",
                ),
            ] {
                if wrong {
                    return Err(fail(
                        budget,
                        ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                        id,
                        &at(tail),
                    ));
                }
            }
            charge_native_work(budget, id, id, "model_tuple_unique", 1)?;
            if seen.contains(id) {
                return Err(fail(
                    budget,
                    ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                    id,
                    &at("/structural_node"),
                ));
            }
            charge_native_storage(budget, id, id, "model_tuples_seen", 3)?;
            seen.insert(id);
            let mut exempt = None;
            if let Some(prior) = self.admitted_tuples.get(id) {
                charge_native_work(
                    budget,
                    id,
                    id,
                    "model_tuple_existing",
                    storage::runtime_tuple(prior).saturating_add(storage::runtime_tuple(row)),
                )?;
                if prior.as_ref() != row {
                    return Err(fail(
                        budget,
                        ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                        id,
                        &at("/structural_node"),
                    ));
                }
                exempt = Some(id);
            }
            if Self::model_union_collides(
                index,
                &row.node_key,
                (Channel::Tuple, k),
                exempt,
                in_row,
                first,
                id,
                budget,
            )? {
                return Err(fail(
                    budget,
                    ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                    id,
                    &at("/node_key"),
                ));
            }
            if exempt.is_some() {
                continue;
            }
            charge_native_storage(
                budget,
                id,
                id,
                "model_tuple_row",
                storage::runtime_tuple(row)
                    .saturating_add(storage::node(id))
                    .saturating_add(2),
            )?;
            if Arc::strong_count(&self.admitted_tuples) > 1 {
                charge_native_work(
                    budget,
                    id,
                    id,
                    "model_tuple_copy",
                    u128::try_from(self.admitted_tuples.len()).unwrap_or(u128::MAX),
                )?;
                charge_native_storage(
                    budget,
                    id,
                    id,
                    "model_tuple_copy",
                    storage::prior_arc_map(&self.admitted_tuples),
                )?;
            }
            Arc::make_mut(&mut self.admitted_tuples).insert(id.clone(), Arc::new(row.clone()));
        }
        Ok(())
    }

    /// The model-union stage. Every in-scope row is first retained as a claim
    /// (its handle is what a payload's `composite` names, in any row order),
    /// then each row is inspected in sidecar order; this value is the local
    /// clone of the caller's identities, so a refusal publishes nothing.
    #[allow(clippy::too_many_arguments)]
    fn join_union_stage(
        &mut self,
        before: &Self,
        package: &CheckedPackageV2,
        union_rows: &[SelectedModelUnionRow],
        scope: &InScope<'_, '_>,
        index: &HandleIndex<'_>,
        in_row: &dyn Fn(Channel, usize) -> bool,
        first: &CheckedNodeId,
        budget: &mut NativeJoinBudget,
    ) -> Res<()> {
        let first_row = union_rows
            .iter()
            .position(|row| scope.contains(&row.structural_node));
        if let (Some(k), true) = (first_row, Arc::strong_count(&self.model_unions) > 1) {
            at_locus(
                budget,
                NativeJoinLocusSource::ModelUnionRow(k),
                "model_union_prior_copy",
            );
            charge_native_work(
                budget,
                first,
                first,
                "model_union_prior_copy",
                u128::try_from(self.model_unions.len()).unwrap_or(u128::MAX),
            )?;
            charge_native_storage(
                budget,
                first,
                first,
                "model_union_prior_copy",
                storage::prior_arc_map(&self.model_unions),
            )?;
        }
        // Retention: every copy is paid before it is made.
        for (k, row) in union_rows.iter().enumerate() {
            let id = &row.structural_node;
            if !scope.contains(id) {
                continue;
            }
            at_locus(
                budget,
                NativeJoinLocusSource::ModelUnionRow(k),
                "model_union_retain",
            );
            charge_native_work(budget, id, id, "model_union_retain", 1)?;
            if self.model_unions.contains_key(id) {
                continue;
            }
            charge_native_storage(
                budget,
                id,
                id,
                "model_union_row",
                row_units(row)
                    .saturating_add(storage::node(id))
                    .saturating_add(2),
            )?;
            Arc::make_mut(&mut self.model_unions).insert(id.clone(), Arc::new(row.clone()));
        }
        charge_native_storage(budget, first, first, "model_unions_seen", 2)?;
        let mut seen: BTreeSet<&CheckedNodeId> = BTreeSet::new();
        for (k, row) in union_rows.iter().enumerate() {
            let id = &row.structural_node;
            if !scope.contains(id) {
                continue;
            }
            at_locus(
                budget,
                NativeJoinLocusSource::ModelUnionRow(k),
                "model_union_row",
            );
            charge_native_work(budget, id, id, "model_union_row", 1)?;
            if before.model_unions.contains_key(id) || seen.contains(id) {
                return Err(fail(
                    budget,
                    ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                    id,
                    &format!("/model_unions/{k}/structural_node"),
                ));
            }
            charge_native_storage(budget, id, id, "model_unions_seen", 3)?;
            seen.insert(id);
            let handles = |budget: &mut NativeJoinBudget| -> Res<()> {
                let me = (Channel::Union, k);
                for (key, tail) in [
                    (&row.type_handle, "/type_handle"),
                    (&row.settled_key, "/settled_key"),
                ] {
                    if Self::model_union_collides(index, key, me, None, in_row, first, id, budget)?
                    {
                        return Err(fail(
                            budget,
                            ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                            id,
                            &format!("/model_unions/{k}{tail}"),
                        ));
                    }
                }
                Ok(())
            };
            self.validate_union_row(package, k, row, handles, budget)?;
        }
        Ok(())
    }

    /// Coverage, last: every tuple or union-family node the scope needs has its
    /// row. A full scope walks every selected declaration; a reached scope
    /// walks only its closure. Each visited node and each family scan is paid.
    fn check_coverage(
        &self,
        package: &CheckedPackageV2,
        scope: &InScope<'_, '_>,
        extent: Extent<'_>,
        first: &CheckedNodeId,
        budget: &mut NativeJoinBudget,
    ) -> Res<()> {
        at_locus(
            budget,
            NativeJoinLocusSource::Package,
            "model_union_coverage",
        );
        let visit = |id: &CheckedNodeId, budget: &mut NativeJoinBudget| -> Res<()> {
            charge_native_work(budget, first, id, "model_union_coverage", 1)?;
            let Some(entry) = package.models.selected_declaration_entry(id) else {
                return Ok(());
            };
            let need = match entry.form {
                DeclarationForm::TupleValueType => Coverage {
                    pointer: "/model_tuples",
                    tuple: true,
                },
                DeclarationForm::VariantType => {
                    let cases = package
                        .models
                        .selected_models()
                        .get(entry.selection)
                        .and_then(|model| model.variant_types.get(&entry.node))
                        .map_or(0, |cases| cases.len());
                    charge_native_work(
                        budget,
                        first,
                        id,
                        "model_union_coverage_family",
                        u128::try_from(cases).unwrap_or(u128::MAX),
                    )?;
                    if package.models.variant_family(id) != Some("union") {
                        return Ok(());
                    }
                    Coverage {
                        pointer: "/model_unions",
                        tuple: false,
                    }
                }
                _ => return Ok(()),
            };
            let present = if need.tuple {
                self.admitted_tuples.contains_key(id)
            } else {
                self.model_unions.contains_key(id)
            };
            if present {
                return Ok(());
            }
            let code = match extent {
                Extent::Full => ExactTypeProjectionErrorCode::IncompleteIdentityWitness,
                Extent::Requested(_) | Extent::Set(_) => {
                    ExactTypeProjectionErrorCode::UnavailableIdentityWitness
                }
            };
            Err(fail(budget, code, id, need.pointer))
        };
        match scope {
            InScope::Full => {
                for (id, _) in package.models.selected_declarations() {
                    visit(id, budget)?;
                }
            }
            InScope::Walked(set) => {
                for id in set {
                    visit(id, budget)?;
                }
            }
            InScope::Given(set) => {
                for id in *set {
                    visit(id, budget)?;
                }
            }
        }
        Ok(())
    }

    /// Steps 3 to 6 of FR-830 for one row against the package and these
    /// identities: node, owner document, the caller's handle check, the case
    /// census, the per-case checks and the payload. `index` is the row's
    /// sidecar position, used only for the refusal pointer. Shared by the join
    /// and by the retained-row validation of `validate_and_rebind`.
    pub(super) fn validate_union_row(
        &self,
        package: &CheckedPackageV2,
        index: usize,
        row: &SelectedModelUnionRow,
        between: impl FnOnce(&mut NativeJoinBudget) -> Res<()>,
        budget: &mut NativeJoinBudget,
    ) -> Res<()> {
        let id = &row.structural_node;
        let at = |tail: &str| format!("/model_unions/{index}{tail}");
        if package.node(id).is_none() {
            return Err(absent_node(package, budget, id, &at("/structural_node")));
        }
        let Some(entry) = package.models.selected_declaration_entry(id) else {
            return Err(fail(
                budget,
                ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                id,
                &at("/structural_node"),
            ));
        };
        if entry.form != DeclarationForm::VariantType
            || package.models.variant_family(id) != Some("union")
        {
            return Err(fail(
                budget,
                ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                id,
                &at("/structural_node"),
            ));
        }
        let Some(model) = package.models.selected_models().get(entry.selection) else {
            return Err(absent_node(package, budget, id, &at("/structural_node")));
        };
        charge_native_work(
            budget,
            id,
            id,
            "model_union_document",
            (row.selected_document.len() as u128).saturating_add(model.identity.len() as u128),
        )?;
        if row.selected_document.as_ref() != model.identity.as_ref() {
            return Err(fail(
                budget,
                ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                id,
                &at("/selected_document"),
            ));
        }
        between(budget)?;
        let Some(document_cases) = model.variant_types.get(&entry.node) else {
            return Err(absent_node(package, budget, id, &at("/structural_node")));
        };
        // Step 4: the case census.
        charge_native_storage(budget, id, id, "model_union_census", 2)?;
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for (j, case) in row.cases.iter().enumerate() {
            charge_native_work(
                budget,
                id,
                id,
                "model_union_census",
                storage::text_parts(&[&case.name]),
            )?;
            if seen.contains(case.name.as_ref()) {
                return Err(fail(
                    budget,
                    ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                    id,
                    &at(&format!("/cases/{j}/name")),
                ));
            }
            if document_cases.get(case.name.as_ref()).is_none() {
                return Err(fail(
                    budget,
                    ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                    id,
                    &at(&format!("/cases/{j}/name")),
                ));
            }
            charge_native_storage(
                budget,
                id,
                id,
                "model_union_census",
                storage::text_parts(&[&case.name]),
            )?;
            seen.insert(case.name.as_ref());
        }
        charge_native_work(
            budget,
            id,
            id,
            "model_union_census",
            u128::try_from(document_cases.len()).unwrap_or(u128::MAX),
        )?;
        if seen.len() != document_cases.len() {
            return Err(fail(
                budget,
                ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                id,
                &at("/cases"),
            ));
        }
        for (j, pair) in row.cases.windows(2).enumerate() {
            charge_native_work(
                budget,
                id,
                id,
                "model_union_order",
                storage::text_parts(&[&pair[0].name, &pair[1].name]),
            )?;
            if pair[0].name.as_bytes() >= pair[1].name.as_bytes() {
                return Err(fail(
                    budget,
                    ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                    id,
                    &at(&format!("/cases/{}/name", j + 1)),
                ));
            }
        }
        // Steps 5 and 6, in array order.
        for (j, case) in row.cases.iter().enumerate() {
            let document_case = document_cases
                .get(case.name.as_ref())
                .expect("the census proved the case");
            charge_native_work(
                budget,
                id,
                id,
                "model_union_case",
                storage::text_parts(&[
                    &case.name,
                    case.member.identifier(),
                    document_case.identity.as_ref(),
                    case.original_case.node(),
                    case.original_case.package(),
                ]),
            )?;
            let case_at = |tail: &str| at(&format!("/cases/{j}{tail}"));
            if u64::try_from(j) != Ok(case.rank)
                || u64::try_from(document_case.position) != Ok(case.position)
            {
                return Err(fail(
                    budget,
                    ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                    id,
                    &case_at(if u64::try_from(j) != Ok(case.rank) {
                        "/rank"
                    } else {
                        "/position"
                    }),
                ));
            }
            if case.original_case.package() != row.selected_document.as_ref()
                || case.original_case.node() != document_case.identity.as_ref()
            {
                return Err(fail(
                    budget,
                    ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                    id,
                    &case_at("/original_case"),
                ));
            }
            if case.member.identifier() != case.name.as_ref() {
                return Err(fail(
                    budget,
                    ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                    id,
                    &case_at("/member/identifier"),
                ));
            }
            if case.member.declaration() != &row.settled_key {
                return Err(fail(
                    budget,
                    ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                    id,
                    &case_at("/member/declaration"),
                ));
            }
            let member = match &document_case.payload {
                None => None,
                Some(type_ref) => model
                    .element_type_runtime(type_ref, budget)
                    .map_err(|_| Self::runtime_denied(budget))?,
            };
            let is_clock = matches!(member, Some(MemberType::Clock));
            match (&case.payload, &document_case.payload) {
                (SelectedModelUnionPayload::None, None) => {}
                (SelectedModelUnionPayload::Clock { payload_type }, Some(_)) if is_clock => {
                    if payload_type.as_ref() != CLOCK_IDENTITY {
                        return Err(fail(
                            budget,
                            ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                            id,
                            &case_at("/payload/payload_type"),
                        ));
                    }
                }
                (
                    SelectedModelUnionPayload::Data {
                        node,
                        original_type,
                    },
                    Some(_),
                ) if !is_clock => {
                    let Some(member) = member else {
                        return Err(fail(
                            budget,
                            ExactTypeProjectionErrorCode::UnsupportedExactType,
                            id,
                            &case_at("/payload"),
                        ));
                    };
                    let expected = Self::model_union_payload_node(
                        &member,
                        &model.checked_package,
                        id,
                        budget,
                    )?
                    .ok_or_else(|| {
                        fail(
                            budget,
                            ExactTypeProjectionErrorCode::UnsupportedExactType,
                            id,
                            &case_at("/payload"),
                        )
                    })?;
                    if package.node(expected.as_ref()).is_none() {
                        return Err(fail(
                            budget,
                            ExactTypeProjectionErrorCode::UnknownNode,
                            node,
                            &case_at("/payload/node"),
                        ));
                    }
                    charge_native_work(budget, id, node, "model_union_payload_node", 1)?;
                    if node != expected.as_ref() {
                        let code = if package.node(node).is_none() {
                            ExactTypeProjectionErrorCode::UnknownNode
                        } else {
                            ExactTypeProjectionErrorCode::MismatchedIdentityWitness
                        };
                        return Err(fail(budget, code, node, &case_at("/payload/node")));
                    }
                    let expected_type = self.model_union_expected_type(
                        package,
                        &member,
                        node,
                        &case_at("/payload/node"),
                        budget,
                    )?;
                    storage::preflight_value_type(original_type, budget, node)?;
                    if original_type != &expected_type {
                        return Err(fail(
                            budget,
                            ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                            node,
                            &case_at("/payload/original_type"),
                        ));
                    }
                }
                _ => {
                    return Err(fail(
                        budget,
                        ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                        id,
                        &case_at("/payload/kind"),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Charge one structural id about to be built, before it is built.
    fn payload_id_units(package: &str, parts: &[&str], nodes: u128) -> u128 {
        parts
            .iter()
            .fold(
                storage::text_parts(&[package]).saturating_add(nodes),
                |units, part| units.saturating_add(storage::text_parts(&[part])),
            )
            .saturating_add(2)
    }

    /// The package node FR-322 step 4 gives a payload member type. A direct
    /// declaration node (record, tuple, variant, unit, object type) is borrowed;
    /// every other form is built under the caller's budget, each id paid for
    /// before it is allocated. `None` is the Clock marker, which names no node.
    fn model_union_payload_node<'m>(
        member: &'m MemberType,
        checked_package: &str,
        at: &CheckedNodeId,
        budget: &mut NativeJoinBudget,
    ) -> Res<Option<Cow<'m, CheckedNodeId>>> {
        use super::super::CheckedNodePathSegment as Segment;
        match member {
            MemberType::Clock => return Ok(None),
            MemberType::Record(id)
            | MemberType::Tuple(id)
            | MemberType::Variant(id)
            | MemberType::Quantity(id)
            | MemberType::Reference(id) => return Ok(Some(Cow::Borrowed(id))),
            _ => {}
        }
        // Wrappers, outermost first.
        let mut wrappers: Vec<&MemberType> = Vec::new();
        let mut leaf = member;
        loop {
            match leaf {
                MemberType::Option(inner) => {
                    charge_native_storage(budget, at, at, "model_union_payload_node", 1)?;
                    wrappers.push(leaf);
                    leaf = inner;
                }
                MemberType::Collection { element, .. } => {
                    charge_native_storage(budget, at, at, "model_union_payload_node", 1)?;
                    wrappers.push(leaf);
                    leaf = element;
                }
                _ => break,
            }
        }
        let name = |text: &str| Segment::Name(text.into());
        let digits = |value: i128| -> u128 {
            u128::from(value.unsigned_abs().checked_ilog10().map_or(1, |d| d + 1))
                + u128::from(value < 0)
        };
        let derived = |budget: &mut NativeJoinBudget,
                       tag: &str,
                       form: &str,
                       extra: Vec<Segment>,
                       nodes: u128,
                       extra_text: u128|
         -> Res<CheckedNodeId> {
            let units = Self::payload_id_units(checked_package, &[tag, form], nodes)
                .saturating_add(extra_text);
            charge_native_work(budget, at, at, "model_union_payload_node", units)?;
            charge_native_storage(budget, at, at, "model_union_payload_node", units)?;
            let mut path = vec![name(tag), name(form)];
            path.extend(extra);
            Ok(CheckedNodeId {
                package: checked_package.into(),
                path,
                ordinal: 0,
            })
        };
        let integer = |budget: &mut NativeJoinBudget| {
            derived(budget, "scalar_type", "integer", Vec::new(), 0, 0)
        };
        let leaf_id = match leaf {
            MemberType::Clock => return Ok(None),
            MemberType::ClockReference => {
                derived(budget, "scalar_type", "clock_reference", Vec::new(), 0, 0)?
            }
            MemberType::Record(id)
            | MemberType::Tuple(id)
            | MemberType::Variant(id)
            | MemberType::Quantity(id) => {
                charge_native_work(budget, at, at, "model_union_payload_node", node_units(id))?;
                charge_native_storage(budget, at, at, "model_union_payload_node", node_units(id))?;
                id.clone()
            }
            MemberType::Reference(id) => {
                charge_native_work(budget, at, at, "model_union_payload_node", node_units(id))?;
                charge_native_storage(budget, at, at, "model_union_payload_node", node_units(id))?;
                let target = Segment::Node(Arc::new(id.clone()));
                derived(budget, "composite_type", "reference", vec![target], 1, 0)?
            }
            MemberType::IntRange(bounds) => {
                let integer = integer(budget)?;
                let numbers = digits(bounds.lower) + digits(bounds.upper);
                derived(
                    budget,
                    "bounded_domain",
                    "integer_range",
                    vec![
                        name("min"),
                        Segment::Node(Arc::new(integer)),
                        name(&bounds.lower.to_string()),
                        name("max"),
                        name(&bounds.upper.to_string()),
                    ],
                    1,
                    numbers + 8,
                )?
            }
            MemberType::TextBounds(bounds) => {
                let text = derived(budget, "scalar_type", "text", Vec::new(), 0, 0)?;
                let integer = integer(budget)?;
                let numbers = u128::try_from(bounds.profile.len()).unwrap_or(u128::MAX)
                    + digits(bounds.lower.into())
                    + digits(bounds.upper.into());
                derived(
                    budget,
                    "bounded_domain",
                    "text_bounds",
                    vec![
                        name("min"),
                        Segment::Node(Arc::new(integer)),
                        name(&bounds.lower.to_string()),
                        name("max"),
                        name(&bounds.upper.to_string()),
                        name("text_profile"),
                        Segment::Node(Arc::new(text)),
                        name(&bounds.profile),
                    ],
                    2,
                    numbers + 24,
                )?
            }
            MemberType::RationalRange(bounds) => {
                let integer = integer(budget)?;
                let numbers = digits(bounds.numerator_min)
                    + digits(bounds.numerator_max)
                    + digits(bounds.denominator_min)
                    + digits(bounds.denominator_max);
                derived(
                    budget,
                    "bounded_domain",
                    "rational_range",
                    vec![
                        name("numerator_min"),
                        Segment::Node(Arc::new(integer)),
                        name(&bounds.numerator_min.to_string()),
                        name("numerator_max"),
                        name(&bounds.numerator_max.to_string()),
                        name("denominator_min"),
                        name(&bounds.denominator_min.to_string()),
                        name("denominator_max"),
                        name(&bounds.denominator_max.to_string()),
                    ],
                    1,
                    numbers + 56,
                )?
            }
            MemberType::CarriedRational(bounds, unit) => {
                let integer = integer(budget)?;
                let numbers = digits(bounds.numerator_min)
                    + digits(bounds.numerator_max)
                    + digits(bounds.denominator_min)
                    + digits(bounds.denominator_max);
                let carrier = derived(
                    budget,
                    "bounded_domain",
                    "rational_range",
                    vec![
                        name("numerator_min"),
                        Segment::Node(Arc::new(integer)),
                        name(&bounds.numerator_min.to_string()),
                        name("numerator_max"),
                        name(&bounds.numerator_max.to_string()),
                        name("denominator_min"),
                        name(&bounds.denominator_min.to_string()),
                        name("denominator_max"),
                        name(&bounds.denominator_max.to_string()),
                    ],
                    1,
                    numbers + 56,
                )?;
                charge_native_work(budget, at, at, "model_union_payload_node", node_units(unit))?;
                charge_native_storage(
                    budget,
                    at,
                    at,
                    "model_union_payload_node",
                    node_units(unit),
                )?;
                let text = derived(budget, "scalar_type", "text", Vec::new(), 0, 0)?;
                derived(
                    budget,
                    "scalar_type",
                    "carried_quantity",
                    vec![
                        Segment::Node(Arc::new(unit.clone())),
                        Segment::Node(Arc::new(carrier)),
                        name("rounding"),
                        Segment::Node(Arc::new(text)),
                        name("exact"),
                    ],
                    3,
                    16,
                )?
            }
            MemberType::Option(_) | MemberType::Collection { .. } => unreachable!("unwrapped"),
            primitive => {
                let form = match primitive {
                    MemberType::Boolean => "boolean",
                    MemberType::Integer => "integer",
                    MemberType::Float32 => "float32",
                    MemberType::Float64 => "float64",
                    MemberType::String => "string",
                    MemberType::Uuid => "uuid",
                    MemberType::Timestamp => "timestamp",
                    MemberType::Bytes => "bytes",
                    _ => "json_object",
                };
                derived(budget, "scalar_type", form, Vec::new(), 0, 0)?
            }
        };
        let mut id = leaf_id;
        while let Some(wrapper) = wrappers.pop() {
            id = match wrapper {
                MemberType::Option(_) => derived(
                    budget,
                    "composite_type",
                    "option",
                    vec![Segment::Node(Arc::new(id))],
                    1,
                    0,
                )?,
                MemberType::Collection { kind, bounds, .. } => {
                    let collection = derived(
                        budget,
                        "composite_type",
                        kind.form(),
                        vec![Segment::Node(Arc::new(id))],
                        1,
                        0,
                    )?;
                    match bounds {
                        Some((lower, upper)) => {
                            let integer = integer(budget)?;
                            let numbers = digits(i128::from(*lower)) + digits(i128::from(*upper));
                            derived(
                                budget,
                                "bounded_domain",
                                "collection_bounds",
                                vec![
                                    name("min"),
                                    Segment::Node(Arc::new(integer)),
                                    name(&lower.to_string()),
                                    name("max"),
                                    name(&upper.to_string()),
                                    Segment::Node(Arc::new(collection)),
                                ],
                                2,
                                numbers + 16,
                            )?
                        }
                        None => collection,
                    }
                }
                _ => unreachable!("only unary member wrappers are stacked"),
            };
        }
        Ok(Some(Cow::Owned(id)))
    }

    /// The Exact type one payload's member type has: the wrappers, then the
    /// leaf. A leaf the shared member-type projection represents is read from
    /// it; the native keyless leaves and the bounded numeric and text leaves
    /// FR-322 step 4 gives are built here. A leaf with no faithful Exact
    /// representation (a clock-place reference) is `UnsupportedExactType`.
    fn model_union_expected_type(
        &self,
        package: &CheckedPackageV2,
        member: &MemberType,
        node: &CheckedNodeId,
        locus: &str,
        budget: &mut NativeJoinBudget,
    ) -> Res<ValueType> {
        enum Wrap {
            Option,
            Collection(CheckedCollectionKind, Option<(u64, u64)>),
        }
        let mut wrappers = Vec::new();
        let mut leaf = member;
        loop {
            match leaf {
                MemberType::Option(inner) => {
                    charge_native_work(budget, node, node, "model_union_payload_type", 1)?;
                    charge_native_storage(budget, node, node, "model_union_payload_type", 1)?;
                    wrappers.push(Wrap::Option);
                    leaf = inner;
                }
                MemberType::Collection {
                    kind,
                    element,
                    bounds,
                } => {
                    charge_native_work(budget, node, node, "model_union_payload_type", 1)?;
                    charge_native_storage(budget, node, node, "model_union_payload_type", 1)?;
                    wrappers.push(Wrap::Collection(*kind, *bounds));
                    leaf = element;
                }
                _ => break,
            }
        }
        charge_native_work(budget, node, node, "model_union_payload_type", 2)?;
        let refuse = |budget: &mut NativeJoinBudget, code| fail(budget, code, node, locus);
        let mut value = match leaf {
            MemberType::Float32 | MemberType::Float64 => {
                charge_native_storage(budget, node, node, "model_union_payload_type", 3)?;
                let width = if matches!(leaf, MemberType::Float32) {
                    IeeeWidth::Binary32
                } else {
                    IeeeWidth::Binary64
                };
                ValueType::Float(FloatType::exact(width))
            }
            MemberType::Bytes => {
                charge_native_storage(budget, node, node, "model_union_payload_type", 1)?;
                ValueType::Bytes
            }
            MemberType::JsonObject => {
                charge_native_storage(budget, node, node, "model_union_payload_type", 1)?;
                ValueType::JsonObject
            }
            MemberType::RationalRange(bounds) => {
                charge_native_storage(budget, node, node, "model_union_payload_type", 80)?;
                let numerator = IntegerInterval::new(
                    Integer::from(bounds.numerator_min),
                    Integer::from(bounds.numerator_max),
                );
                let denominator = IntegerInterval::new(
                    Integer::from(bounds.denominator_min),
                    Integer::from(bounds.denominator_max),
                );
                let domain = numerator
                    .ok()
                    .zip(denominator.ok())
                    .and_then(|(n, d)| RationalDomain::new(n, d).ok());
                match domain {
                    Some(domain) => ValueType::Rational(domain),
                    None => {
                        return Err(refuse(
                            budget,
                            ExactTypeProjectionErrorCode::UnsupportedExactType,
                        ))
                    }
                }
            }
            MemberType::TextBounds(bounds) => {
                charge_native_storage(budget, node, node, "model_union_payload_type", 8)?;
                let text = TextProfile::from_code(&bounds.profile)
                    .and_then(|profile| TextType::new(bounds.lower, bounds.upper, profile).ok());
                match text {
                    Some(text) => ValueType::Text(text),
                    None => {
                        return Err(refuse(
                            budget,
                            ExactTypeProjectionErrorCode::UnsupportedExactType,
                        ))
                    }
                }
            }
            MemberType::CarriedRational(bounds, unit_node) => {
                let quantity = self.model_union_leaf(
                    package,
                    &MemberType::Quantity(unit_node.clone()),
                    node,
                    locus,
                    budget,
                )?;
                let ValueType::Quantity(unit) = &quantity else {
                    return Err(refuse(
                        budget,
                        ExactTypeProjectionErrorCode::UnsupportedExactType,
                    ));
                };
                let unit = unit.clone();
                charge_native_storage(budget, node, node, "model_union_payload_type", 100)?;
                let numerator = IntegerInterval::new(
                    Integer::from(bounds.numerator_min),
                    Integer::from(bounds.numerator_max),
                );
                let denominator = IntegerInterval::new(
                    Integer::from(bounds.denominator_min),
                    Integer::from(bounds.denominator_max),
                );
                let domain = numerator
                    .ok()
                    .zip(denominator.ok())
                    .and_then(|(n, d)| RationalDomain::new(n, d).ok());
                match domain {
                    Some(domain) => ValueType::CarriedQuantity(Arc::new(CarriedQuantityType::new(
                        unit,
                        QuantityCarrier::Rational(domain),
                    ))),
                    None => {
                        return Err(refuse(
                            budget,
                            ExactTypeProjectionErrorCode::UnsupportedExactType,
                        ))
                    }
                }
            }
            leaf => self.model_union_leaf(package, leaf, node, locus, budget)?,
        };
        for wrapper in wrappers.into_iter().rev() {
            value = match wrapper {
                Wrap::Option => {
                    charge_native_storage(budget, node, node, "model_union_payload_type", 1)?;
                    ValueType::option(value)
                }
                Wrap::Collection(kind, bounds) => {
                    let kind = match kind {
                        CheckedCollectionKind::Sequence => CollectionKind::Sequence,
                        CheckedCollectionKind::Set => CollectionKind::Set,
                        CheckedCollectionKind::Bag => CollectionKind::Bag,
                        CheckedCollectionKind::OrderedSet => CollectionKind::OrderedSet,
                    };
                    let Ok(bound) = bounds
                        .map(|(lower, upper)| CardinalityBound::new(lower, upper))
                        .transpose()
                    else {
                        return Err(refuse(
                            budget,
                            ExactTypeProjectionErrorCode::UnsupportedExactType,
                        ));
                    };
                    charge_native_storage(budget, node, node, "model_union_payload_type", 1)?;
                    ValueType::collection(CollectionType::new(kind, value, bound))
                }
            };
        }
        Ok(value)
    }

    /// One leaf through the shared member-type projection, its refusal located
    /// at the payload node's sidecar pointer.
    fn model_union_leaf(
        &self,
        package: &CheckedPackageV2,
        leaf: &MemberType,
        node: &CheckedNodeId,
        locus: &str,
        budget: &mut NativeJoinBudget,
    ) -> Res<ValueType> {
        super::project_record_field_type(package, self, leaf, node, budget).map_err(|mut error| {
            if error.denied.is_none() {
                error.form = Some(locus.into());
            }
            error
        })
    }

    /// The model-tuple rows this selection retains, keyed by structural node.
    pub fn model_tuple_rows(&self) -> impl Iterator<Item = &SelectedTupleDeclarationRow> {
        self.admitted_tuples.values().map(Arc::as_ref)
    }

    /// The validated model-union rows, keyed by the full structural node.
    pub fn model_union_rows(&self) -> impl Iterator<Item = &SelectedModelUnionRow> {
        self.model_unions.values().map(Arc::as_ref)
    }

    /// The model-union row of one `variant_type` node, borrowed.
    pub fn model_union_row(&self, node: &CheckedNodeId) -> Option<&SelectedModelUnionRow> {
        self.model_unions.get(node).map(Arc::as_ref)
    }
}

impl SelectedExactIdentities {
    /// The same join over a closure the caller already walked and paid for.
    pub(in crate::checked_package::v2) fn with_model_union_rows_over(
        &self,
        package: &CheckedPackageV2,
        tuple_rows: &[SelectedTupleDeclarationRow],
        union_rows: &[SelectedModelUnionRow],
        closure: &BTreeSet<CheckedNodeId>,
        budget: &mut NativeJoinBudget,
    ) -> Result<Self, ExactTypeProjectionError> {
        let Some(first) = closure.iter().next() else {
            return Ok(self.clone());
        };
        self.join_model_unions_framed(
            package,
            tuple_rows,
            union_rows,
            Extent::Set(closure),
            first,
            budget,
        )
    }
}
