// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-830: the selected model-union witness rows and the native join that
//! admits them beside the selected model-tuple rows.
//!
//! The reader validates consistency and preserves the producer's opaque
//! identities; it does NOT claim provenance authentication from parsing. A row
//! set that passes every join is admitted whether or not the producer that wrote
//! it held the declaration.

use super::super::model_members::{DeclarationForm, MemberType};
use super::super::native_join_storage as storage;
use super::super::NativeJoinBudget;
use super::super::{CheckedNodeId, CheckedNodeKind, CheckedPackageV2, ModelForm, ScalarTypeForm};
use super::{
    charge_native_storage, charge_native_work, exact_diagnostic, selected_declaration_error,
    witness_error, ExactDiagnosticForm, ExactTypeProjectionError, ExactTypeProjectionErrorCode,
    SelectedExactIdentities, SelectedExactIdentity, SelectedTupleDeclarationRow,
};
use filament_exact::{NodeKey, OriginalDeclarationKey, UnionMember, UnitId, ValueType};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// The identity of the Clock native type a clock payload carries.
const CLOCK_IDENTITY: &str = "ix://filament/native/Clock";

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

fn node_units(node: &CheckedNodeId) -> u128 {
    storage::node(node)
}

fn row_units(row: &SelectedModelUnionRow) -> u128 {
    storage::model_union(row)
}

/// Every NodeKey edge of one member type that names a node the closure follows.
fn member_edges<'a>(member: &'a MemberType, out: &mut Vec<&'a CheckedNodeId>) {
    let mut current = member;
    loop {
        match current {
            MemberType::Option(inner) => current = inner,
            MemberType::Collection { element, .. } => current = element,
            MemberType::Record(id)
            | MemberType::Tuple(id)
            | MemberType::Variant(id)
            | MemberType::Reference(id) => {
                out.push(id);
                return;
            }
            _ => return,
        }
    }
}

impl SelectedExactIdentities {
    fn runtime_denied(budget: &NativeJoinBudget) -> ExactTypeProjectionError {
        let denied = budget
            .first_unpaid()
            .expect("a denied runtime type read records its unpaid charge");
        ExactTypeProjectionError::compact_native(budget, denied.kind, denied.amount)
    }

    /// The closure of `start`: case payloads, tuple positions and record or
    /// object field types, each node expanded once, each edge examined once.
    fn model_union_closure(
        package: &CheckedPackageV2,
        start: &CheckedNodeId,
        budget: &mut NativeJoinBudget,
    ) -> Result<BTreeSet<CheckedNodeId>, ExactTypeProjectionError> {
        if package.node(start).is_none() {
            return Err(selected_declaration_error(package, start));
        }
        charge_native_storage(budget, start, start, "model_union_closure", 3)?;
        let mut visited = BTreeSet::new();
        let mut pending = vec![start.clone()];
        visited.insert(start.clone());
        while let Some(id) = pending.pop() {
            charge_native_work(budget, start, &id, "model_union_closure_node", 1)?;
            let Some(entry) = package.models.selected_declaration_entry(&id) else {
                continue;
            };
            let Some(model) = package.models.selected_models().get(entry.selection) else {
                continue;
            };
            let mut edges: Vec<&CheckedNodeId> = Vec::new();
            let mut members: Vec<MemberType> = Vec::new();
            match entry.form {
                DeclarationForm::VariantType => {
                    if let Some(cases) = model.variant_types.get(&entry.node) {
                        for case in cases.values() {
                            charge_native_work(budget, start, &id, "model_union_closure_edge", 1)?;
                            if let Some(payload) = &case.payload {
                                if let Some(member) = model
                                    .element_type_runtime(payload, budget)
                                    .map_err(|_| Self::runtime_denied(budget))?
                                {
                                    members.push(member);
                                }
                            }
                        }
                    }
                }
                DeclarationForm::TupleValueType => {
                    if let Some(positions) = model.tuple_types.get(&entry.node) {
                        for position in positions {
                            charge_native_work(budget, start, &id, "model_union_closure_edge", 1)?;
                            if let Some(member) = model
                                .element_type_runtime(position, budget)
                                .map_err(|_| Self::runtime_denied(budget))?
                            {
                                members.push(member);
                            }
                        }
                    }
                }
                DeclarationForm::RecordValueType => {
                    if let Some((_, fields)) = package.models.selected_record_fields(&id) {
                        for field in fields {
                            charge_native_work(budget, start, &id, "model_union_closure_edge", 1)?;
                            if let Some(member) = field.member_type {
                                member_edges(member, &mut edges);
                            }
                        }
                    }
                }
                DeclarationForm::ObjectType => {
                    if let Some((_, fields)) = package.models.selected_object_fields(&id) {
                        for field in fields {
                            charge_native_work(budget, start, &id, "model_union_closure_edge", 1)?;
                            if let Some(member) = field.member_type {
                                member_edges(member, &mut edges);
                            }
                        }
                    }
                }
                _ => {}
            }
            for member in &members {
                member_edges(member, &mut edges);
            }
            for target in edges {
                charge_native_work(budget, start, target, "model_union_closure_edge", 1)?;
                if !visited.contains(target) {
                    charge_native_storage(
                        budget,
                        start,
                        target,
                        "model_union_closure",
                        node_units(target).saturating_add(2),
                    )?;
                    visited.insert(target.clone());
                    pending.push(target.clone());
                }
            }
        }
        Ok(visited)
    }

    /// The package node of a payload member type, FR-322 step 4.
    fn model_union_payload_node<'m>(
        member: &'m MemberType,
        checked_package: &str,
        at: &CheckedNodeId,
        budget: &mut NativeJoinBudget,
    ) -> Result<Option<Cow<'m, CheckedNodeId>>, ExactTypeProjectionError> {
        use super::super::CheckedNodePathSegment as Segment;
        let form = match member {
            MemberType::Record(id)
            | MemberType::Tuple(id)
            | MemberType::Variant(id)
            | MemberType::Quantity(id)
            | MemberType::Reference(id) => return Ok(Some(Cow::Borrowed(id))),
            MemberType::Boolean => "boolean",
            MemberType::Integer => "integer",
            MemberType::Float32 => "float32",
            MemberType::Float64 => "float64",
            MemberType::String => "string",
            MemberType::Uuid => "uuid",
            MemberType::Timestamp => "timestamp",
            MemberType::Bytes => "bytes",
            MemberType::JsonObject => "json_object",
            _ => return Ok(None),
        };
        let units = (checked_package.len() as u128)
            .saturating_add("scalar_type".len() as u128)
            .saturating_add(form.len() as u128)
            .saturating_add(4);
        charge_native_work(budget, at, at, "model_union_payload_node", units)?;
        charge_native_storage(budget, at, at, "model_union_payload_node", units)?;
        Ok(Some(Cow::Owned(CheckedNodeId {
            package: checked_package.into(),
            path: vec![
                Segment::Name("scalar_type".into()),
                Segment::Name(form.into()),
            ],
            ordinal: 0,
        })))
    }

    /// Every key handle held by any retained identity channel, indexed once.
    fn model_union_handles(
        &self,
        first: &CheckedNodeId,
        budget: &mut NativeJoinBudget,
    ) -> Result<BTreeSet<NodeKey>, ExactTypeProjectionError> {
        charge_native_storage(budget, first, first, "model_union_handles", 2)?;
        let mut handles = BTreeSet::new();
        macro_rules! take {
            ($id:expr, $key:expr) => {{
                charge_native_work(budget, first, $id, "model_union_handle", 1)?;
                if !handles.contains($key) {
                    charge_native_storage(budget, first, $id, "model_union_handle", 2)?;
                    handles.insert($key.clone());
                }
            }};
        }
        for row in self.declarations.values() {
            match &row.exact_identity {
                SelectedExactIdentity::Record(key)
                | SelectedExactIdentity::Union(key)
                | SelectedExactIdentity::DeclaredUnit(UnitId::Declared(key)) => {
                    take!(&row.structural_node, key)
                }
                _ => charge_native_work(
                    budget,
                    first,
                    &row.structural_node,
                    "model_union_handle",
                    1,
                )?,
            }
        }
        for (id, row) in self.admitted_tuples.iter() {
            take!(id, &row.node_key);
        }
        for (id, row) in self.admitted_source_units.iter() {
            if let UnitId::Declared(key) = &row.unit_id {
                take!(id, key);
            }
        }
        for (id, row) in self.source_unions.iter() {
            take!(id, &row.type_handle);
            take!(id, &row.settled_key);
        }
        for (id, row) in self.source_records.iter() {
            take!(id, &row.record_key);
        }
        for (id, row) in self.source_tuples.iter() {
            take!(id, &row.type_handle);
        }
        for (id, row) in self.source_enums.iter() {
            take!(id, &row.declaration.node_key);
        }
        for (id, row) in self.model_enum_shapes.iter() {
            take!(id, &row.node_key);
            for case in &row.cases {
                take!(&case.structural_node, &case.member_key);
            }
        }
        for (id, row) in self.model_unions.iter() {
            take!(id, &row.type_handle);
            take!(id, &row.settled_key);
        }
        Ok(handles)
    }

    /// Join the selected model-tuple rows and then the selected model-union
    /// rows to this package under the caller's one shared budget.
    ///
    /// Tuple rows are staged first because a union payload reads its tuple's
    /// witness. Under [`ModelUnionScope::Reached`] only rows of the closure of
    /// the requested node are joined, inspected and retained; the common pass
    /// still reads every supplied row and its work is charged. The result is
    /// published whole or not at all, and a refusal or exhaustion leaves the
    /// charges already admitted consumed.
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
        let first = first.clone();
        let before = budget.begin();
        let extent = match scope {
            ModelUnionScope::Full => Extent::Full,
            ModelUnionScope::Reached(requested) => Extent::Requested(requested),
        };
        let result =
            self.join_model_unions(package, tuple_rows, union_rows, extent, &first, budget);
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
    ) -> Result<Self, ExactTypeProjectionError> {
        self.check_binding(package, budget, first)?;
        charge_native_storage(
            budget,
            first,
            first,
            "selected_identity_handles",
            storage::selected_clone(),
        )?;
        // Preflight: the handle index of every supplied and retained witness.
        let mut handles = self.model_union_handles(first, budget)?;
        // Common pass: every row of both channels, whatever the scope.
        for row in tuple_rows {
            let id = &row.structural_node;
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
        for row in union_rows {
            let id = &row.structural_node;
            charge_native_work(budget, id, id, "model_unions_row", row_units(row))?;
        }
        let computed;
        let closure: Option<&BTreeSet<CheckedNodeId>> = match extent {
            Extent::Full => None,
            Extent::Requested(requested) => {
                computed = Self::model_union_closure(package, requested, budget)?;
                Some(&computed)
            }
            Extent::Set(set) => Some(set),
        };
        let in_scope = |id: &CheckedNodeId| closure.is_none_or(|set| set.contains(id));
        // Tuples first, through the existing typed tuple join.
        let mut joined_tuples = Vec::new();
        for row in tuple_rows
            .iter()
            .filter(|row| in_scope(&row.structural_node))
        {
            charge_native_storage(
                budget,
                first,
                &row.structural_node,
                "model_tuples_stage",
                storage::runtime_tuple(row).saturating_add(1),
            )?;
            joined_tuples.push(row.clone());
        }
        let mut selected = self.with_admitted_type_rows(package, &joined_tuples, &[], budget)?;
        for row in &joined_tuples {
            // A model tuple's key must not collide with a union handle.
            if selected
                .model_unions
                .values()
                .any(|union| union.type_handle == row.node_key || union.settled_key == row.node_key)
            {
                return Err(witness_error(
                    &row.structural_node,
                    ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                ));
            }
        }
        handles.extend(joined_tuples.iter().map(|row| row.node_key.clone()));
        // Union handles of the rows joined here, for payload composite keys.
        let mut joined_handles: BTreeMap<&CheckedNodeId, &NodeKey> = BTreeMap::new();
        for row in union_rows
            .iter()
            .filter(|row| in_scope(&row.structural_node))
        {
            joined_handles.insert(&row.structural_node, &row.type_handle);
        }
        let mut staged: BTreeMap<CheckedNodeId, Arc<SelectedModelUnionRow>> = BTreeMap::new();
        for (index, row) in union_rows
            .iter()
            .enumerate()
            .filter(|(_, row)| in_scope(&row.structural_node))
        {
            let id = &row.structural_node;
            let at = |tail: &str| format!("/model_unions/{index}{tail}");
            charge_native_work(budget, id, id, "model_union_row", 1)?;
            if staged.contains_key(id) || selected.model_unions.contains_key(id) {
                return Err(fail(
                    budget,
                    ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                    id,
                    &at("/structural_node"),
                ));
            }
            if package.node(id).is_none() {
                let mut error = selected_declaration_error(package, id);
                error.form = Some(at("/structural_node").into());
                return Err(error);
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
                return Err(selected_declaration_error(package, id));
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
            if handles.contains(&row.type_handle) {
                return Err(fail(
                    budget,
                    ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                    id,
                    &at("/type_handle"),
                ));
            }
            if handles.contains(&row.settled_key) && row.settled_key != row.type_handle {
                return Err(fail(
                    budget,
                    ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                    id,
                    &at("/settled_key"),
                ));
            }
            let Some(document_cases) = model.variant_types.get(&entry.node) else {
                return Err(selected_declaration_error(package, id));
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
                if case.member.identifier() != case.name.as_ref()
                    || case.member.declaration() != &row.settled_key
                {
                    return Err(fail(
                        budget,
                        ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                        id,
                        &case_at(if case.member.identifier() != case.name.as_ref() {
                            "/member/identifier"
                        } else {
                            "/member/declaration"
                        }),
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
                        let expected_type = selected.model_union_expected_type(
                            package,
                            node,
                            &joined_handles,
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
            // Retention: precharge every copy before it is made.
            charge_native_storage(budget, id, id, "model_union_row", row_units(row))?;
            handles.insert(row.type_handle.clone());
            handles.insert(row.settled_key.clone());
            staged.insert(id.clone(), Arc::new(row.clone()));
        }
        // Step 7: coverage, last.
        for (id, entry) in package.models.selected_declarations() {
            let needs_tuple = entry.form == DeclarationForm::TupleValueType;
            let needs_union = entry.form == DeclarationForm::VariantType
                && package.models.variant_family(id) == Some("union");
            if !(needs_tuple || needs_union) {
                continue;
            }
            charge_native_work(budget, first, id, "model_union_coverage", 1)?;
            let present = if needs_tuple {
                selected.admitted_tuples.contains_key(id)
            } else {
                staged.contains_key(id) || selected.model_unions.contains_key(id)
            };
            if present {
                continue;
            }
            match extent {
                Extent::Full => {
                    return Err(fail(
                        budget,
                        ExactTypeProjectionErrorCode::IncompleteIdentityWitness,
                        id,
                        if needs_tuple {
                            "/model_tuples"
                        } else {
                            "/model_unions"
                        },
                    ));
                }
                Extent::Requested(_) | Extent::Set(_) if in_scope(id) => {
                    return Err(fail(
                        budget,
                        ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                        id,
                        if needs_tuple {
                            "/model_tuples"
                        } else {
                            "/model_unions"
                        },
                    ));
                }
                Extent::Requested(_) | Extent::Set(_) => {}
            }
        }
        // Publish whole.
        if !staged.is_empty() {
            if Arc::strong_count(&selected.model_unions) > 1 {
                charge_native_work(
                    budget,
                    first,
                    first,
                    "model_union_prior_copy",
                    u128::try_from(selected.model_unions.len()).unwrap_or(u128::MAX),
                )?;
                charge_native_storage(
                    budget,
                    first,
                    first,
                    "model_union_prior_copy",
                    storage::prior_arc_map(&selected.model_unions),
                )?;
            }
            let map = Arc::make_mut(&mut selected.model_unions);
            for (id, row) in staged {
                map.insert(id, row);
            }
        }
        if selected.binding.is_none() {
            selected.binding = package.admission_binding.clone();
        }
        Ok(selected)
    }

    /// The Exact type the admitted projection gives one payload node's form,
    /// from the witnesses this selection holds and the union rows being joined.
    fn model_union_expected_type(
        &self,
        package: &CheckedPackageV2,
        node: &CheckedNodeId,
        joined: &BTreeMap<&CheckedNodeId, &NodeKey>,
        locus: &str,
        budget: &mut NativeJoinBudget,
    ) -> Result<ValueType, ExactTypeProjectionError> {
        let unavailable = |budget: &mut NativeJoinBudget| {
            fail(
                budget,
                ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                node,
                locus,
            )
        };
        charge_native_work(budget, node, node, "model_union_payload_type", 2)?;
        match package.node_kind(node) {
            Some(CheckedNodeKind::Model(ModelForm::TupleValueType)) => self
                .admitted_tuples
                .get(node)
                .map(|row| ValueType::Composite(row.node_key.clone()))
                .ok_or_else(|| unavailable(budget)),
            Some(CheckedNodeKind::Model(ModelForm::VariantType)) => {
                if package.models.variant_family(node) != Some("union") {
                    return Err(fail(
                        budget,
                        ExactTypeProjectionErrorCode::UnsupportedExactType,
                        node,
                        locus,
                    ));
                }
                joined
                    .get(node)
                    .map(|key| ValueType::Composite((*key).clone()))
                    .or_else(|| {
                        self.model_unions
                            .get(node)
                            .map(|row| ValueType::Composite(row.type_handle.clone()))
                    })
                    .ok_or_else(|| unavailable(budget))
            }
            Some(CheckedNodeKind::Model(ModelForm::RecordValueType)) => {
                match self.declaration_identity(package, node) {
                    Ok(SelectedExactIdentity::Record(key)) => Ok(ValueType::Composite(key.clone())),
                    Ok(_) => Err(fail(
                        budget,
                        ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                        node,
                        locus,
                    )),
                    Err(code) => Err(fail(budget, code, node, locus)),
                }
            }
            Some(CheckedNodeKind::Model(ModelForm::ObjectType)) => {
                match self.declaration_identity(package, node) {
                    Ok(SelectedExactIdentity::Object(effective)) => {
                        storage::preflight_effective(effective, budget, node)?;
                        charge_native_storage(
                            budget,
                            node,
                            node,
                            "model_union_payload_type",
                            storage::effective_identity(effective).saturating_add(1),
                        )?;
                        Ok(ValueType::Reference(effective.clone()))
                    }
                    Ok(_) => Err(fail(
                        budget,
                        ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                        node,
                        locus,
                    )),
                    Err(code) => Err(fail(budget, code, node, locus)),
                }
            }
            Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Unit)) => {
                match self.declaration_identity(package, node) {
                    Ok(SelectedExactIdentity::DeclaredUnit(unit)) => {
                        storage::preflight_unit(unit, budget, node)?;
                        charge_native_storage(
                            budget,
                            node,
                            node,
                            "model_union_payload_type",
                            storage::unit(unit).saturating_add(1),
                        )?;
                        Ok(ValueType::Quantity(unit.clone()))
                    }
                    Ok(_) => Err(fail(
                        budget,
                        ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                        node,
                        locus,
                    )),
                    Err(code) => Err(fail(budget, code, node, locus)),
                }
            }
            Some(CheckedNodeKind::ScalarType(
                ScalarTypeForm::Boolean
                | ScalarTypeForm::Integer
                | ScalarTypeForm::String
                | ScalarTypeForm::Uuid
                | ScalarTypeForm::Timestamp
                | ScalarTypeForm::Bytes
                | ScalarTypeForm::JsonObject
                | ScalarTypeForm::Float32
                | ScalarTypeForm::Float64,
            )) => package
                .project_exact_type(node, self, budget)
                .map(|projection| projection.value_type().clone()),
            _ => Err(fail(
                budget,
                ExactTypeProjectionErrorCode::UnsupportedExactType,
                node,
                locus,
            )),
        }
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

/// The internal extent of one join: the public scope, or a closure the caller
/// already walked under the same budget.
#[derive(Clone, Copy)]
pub(in crate::checked_package::v2) enum Extent<'a> {
    Full,
    Requested(&'a CheckedNodeId),
    Set(&'a BTreeSet<CheckedNodeId>),
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
        let Some(first) = closure.iter().next().cloned() else {
            return Ok(self.clone());
        };
        let before = budget.begin();
        let result = self.join_model_unions(
            package,
            tuple_rows,
            union_rows,
            Extent::Set(closure),
            &first,
            budget,
        );
        match &result {
            Ok(_) => budget.finish(before),
            Err(_) => budget.abort(),
        }
        result.map_err(|mut error| {
            error.work = budget.consumed().work;
            error
        })
    }
}
