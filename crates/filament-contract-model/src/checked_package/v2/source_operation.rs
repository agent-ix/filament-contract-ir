// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-368: join producer-selected source-operation rows to admitted anchors
//! whose context is a source-declared object type.

use super::callable::{
    CheckedCallable, CheckedCallableIdentity, CheckedCallableRead, CheckedCallableReadError,
    CheckedCallableReadErrorCode,
};
use super::source_object::is_source_declared_object_type;
use super::structural::typed_anchor_body;
use super::{
    CheckedNodeId, CheckedNodeKind, CheckedNodeOwner, CheckedPackageV2, CheckedTerm,
    SelectedOperationMemberError, SelectedOperationMemberErrorCode as Code, StateForm,
};
use std::collections::BTreeMap;

/// One FSL-stage source-operation identity row for an admitted
/// `state`/`operation_anchor` whose context is a source-declared object type.
///
/// The row is a producer fact: this type validates its correspondence to the
/// admitted anchor and mints no operation identity of its own.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedSourceOperationRow {
    /// Authority of the declaring type's `SourceOwner`.
    pub source_authority: Box<str>,
    /// `<type>::<operation>`, `<type>` the final segment of the declaring
    /// type's `declaration`.
    pub source_identity: Box<str>,
    /// Full structural identity of the admitted operation anchor.
    pub anchor_id: CheckedNodeId,
    /// The admitted operation anchor; equal to `anchor_id`.
    pub structural_node: CheckedNodeId,
}

/// Validated source-operation rows for one admitted checked package instance.
///
/// A missing row remains missing: no row is ever derived from an operation's
/// name, and no source anchor is matched to a model operation member.
#[derive(Clone, Debug)]
pub struct SelectedSourceOperations<'a> {
    package: &'a CheckedPackageV2,
    rows: BTreeMap<CheckedNodeId, SelectedSourceOperationRow>,
}

fn refusal(code: Code, anchor: &CheckedNodeId, consumed_work: u64) -> SelectedOperationMemberError {
    SelectedOperationMemberError {
        code,
        anchor: anchor.clone(),
        consumed_work,
        attempted_work: 0,
    }
}

fn charge(
    consumed: &mut u64,
    limit: u64,
    anchor: &CheckedNodeId,
) -> Result<(), SelectedOperationMemberError> {
    if *consumed >= limit {
        return Err(SelectedOperationMemberError {
            code: Code::WorkLimit,
            anchor: anchor.clone(),
            consumed_work: *consumed,
            attempted_work: 1,
        });
    }
    *consumed += 1;
    Ok(())
}

impl<'a> SelectedSourceOperations<'a> {
    /// Validate producer rows against the admitted package under a finite work
    /// limit, charging one unit for each row and each `operation_effects` row
    /// inspected. Each defect has the outcome the model operation-member row
    /// gets for it. No partially validated index escapes on refusal. An empty
    /// index is valid and makes every later read return `UnavailableWitness`.
    pub fn from_stage_rows(
        package: &'a CheckedPackageV2,
        rows: impl IntoIterator<Item = SelectedSourceOperationRow>,
        work_limit: u64,
    ) -> Result<Self, SelectedOperationMemberError> {
        let mut selected = Self {
            package,
            rows: BTreeMap::new(),
        };
        let mut consumed = 0_u64;
        for row in rows {
            let anchor = &row.anchor_id;
            charge(&mut consumed, work_limit, anchor)?;
            if selected.rows.contains_key(anchor) {
                return Err(refusal(Code::ConflictingWitness, anchor, consumed));
            }
            if package.node(anchor).is_none() {
                return Err(refusal(Code::UnknownAnchor, anchor, consumed));
            }
            if package.node_kind(anchor) != Some(CheckedNodeKind::State(StateForm::OperationAnchor))
            {
                return Err(refusal(Code::WrongAnchorForm, anchor, consumed));
            }
            let Some(body) = package.term(anchor).and_then(typed_anchor_body) else {
                return Err(refusal(Code::WrongAnchorForm, anchor, consumed));
            };
            if row.structural_node != row.anchor_id {
                return Err(refusal(Code::MismatchedWitness, anchor, consumed));
            }
            let context = package
                .node(body.context)
                .zip(package.node_kind(body.context))
                .filter(|(node, kind)| is_source_declared_object_type(node, *kind));
            let Some((context, _)) = context else {
                return Err(refusal(Code::UnavailableWitness, anchor, consumed));
            };
            let type_name = context
                .declaration
                .as_ref()
                .and_then(|declaration| declaration.qualified_name.last());
            let authority_matches = matches!(
                &context.owner,
                Some(CheckedNodeOwner::Source { authority, .. })
                    if *authority == row.source_authority
            );
            let identity_matches = type_name.is_some_and(|name| {
                row.source_identity
                    .strip_prefix(name.as_ref())
                    .and_then(|rest| rest.strip_prefix("::"))
                    == Some(body.operation)
            });
            if !authority_matches || !identity_matches {
                return Err(refusal(Code::MismatchedWitness, anchor, consumed));
            }
            if let Some(effects) = package.operation_effect_rows() {
                let mut named = false;
                for effect in effects {
                    charge(&mut consumed, work_limit, anchor)?;
                    if effect.declaring == *body.context
                        && effect.operation.as_ref() == body.operation
                    {
                        named = true;
                        break;
                    }
                }
                if !named {
                    return Err(refusal(Code::MismatchedWitness, anchor, consumed));
                }
            }
            selected.rows.insert(anchor.clone(), row);
        }
        Ok(selected)
    }

    /// Read the validated producer row for an admitted anchor, never finding
    /// an operation by name or accepting a row from another admission.
    pub fn read(
        &self,
        package: &CheckedPackageV2,
        anchor: &CheckedNodeId,
    ) -> Result<&SelectedSourceOperationRow, SelectedOperationMemberError> {
        if !std::ptr::eq(self.package, package) {
            return Err(refusal(Code::ForeignPackage, anchor, 0));
        }
        if package.node(anchor).is_none() {
            return Err(refusal(Code::UnknownAnchor, anchor, 0));
        }
        self.rows
            .get(anchor)
            .ok_or_else(|| refusal(Code::UnavailableWitness, anchor, 0))
    }
}

impl CheckedPackageV2 {
    /// Read one source-declared operation anchor joined to its source-operation
    /// row: the callable of its `[<type>, <operation>, "body"]` clause function
    /// under the type's `SourceOwner`, with the anchor's context and frame.
    ///
    /// A source anchor without a row returns `UnavailableWitness`, as
    /// [`CheckedPackageV2::callable_table`] returns it. No partial row is
    /// exposed after a refusal.
    pub fn source_operation_callable(
        &self,
        source_operations: &SelectedSourceOperations<'_>,
        anchor: &CheckedNodeId,
        work_limit: u64,
    ) -> Result<CheckedCallableRead, CheckedCallableReadError> {
        let mut budget = super::callable::Budget::new(work_limit);
        budget.charge(anchor)?;
        let error = |code| super::callable::Budget::refusal(code, anchor, 0);
        let row = source_operations.read(self, anchor).map_err(|error| {
            super::callable::Budget::refusal(
                match error.code {
                    Code::UnavailableWitness | Code::UnavailableModelMember => {
                        CheckedCallableReadErrorCode::UnavailableWitness
                    }
                    Code::ForeignPackage | Code::MismatchedWitness | Code::ConflictingWitness => {
                        CheckedCallableReadErrorCode::ForeignSelectedJoin
                    }
                    Code::UnknownAnchor | Code::WrongAnchorForm => {
                        CheckedCallableReadErrorCode::MalformedBody
                    }
                    Code::WorkLimit => CheckedCallableReadErrorCode::WorkLimit,
                },
                anchor,
                budget.work(),
            )
        })?;
        let body = self
            .term(anchor)
            .and_then(typed_anchor_body)
            .ok_or_else(|| error(CheckedCallableReadErrorCode::MalformedBody))?;
        let owner = self
            .node(body.context)
            .and_then(|context| context.owner.as_ref())
            .ok_or_else(|| error(CheckedCallableReadErrorCode::MalformedBody))?;
        let mut function = None;
        for node in &self.resolved_graph().nodes {
            budget.charge(&node.node_id)?;
            if node.owner.as_ref() == Some(owner)
                && matches!(
                    self.node_kind(&node.node_id),
                    Some(CheckedNodeKind::Function(_))
                )
                && node.declaration.as_ref().is_some_and(|declaration| {
                    matches!(
                        declaration.qualified_name.as_slice(),
                        [_, name, clause]
                            if name.as_ref() == body.operation && clause.as_ref() == "body"
                    )
                })
            {
                function = Some(&node.node_id);
                break;
            }
        }
        let function =
            function.ok_or_else(|| error(CheckedCallableReadErrorCode::MissingSelectedJoin))?;
        let parsed = super::callable::parse_function(self, function, &mut budget)?;
        let callable_body = parsed
            .body
            .ok_or_else(|| error(CheckedCallableReadErrorCode::MissingBody))?;
        if self.node_kind(body.frame) != Some(CheckedNodeKind::State(StateForm::Frame))
            || !matches!(self.term(body.frame), Some(CheckedTerm::Frame { .. }))
        {
            return Err(error(CheckedCallableReadErrorCode::MissingFrame));
        }
        Ok(CheckedCallableRead::new(
            Some(CheckedCallable {
                identity: CheckedCallableIdentity::SourceOperation {
                    anchor: anchor.clone(),
                    source_authority: row.source_authority.clone(),
                    source_identity: row.source_identity.clone(),
                },
                parameters: parsed.parameters,
                body: callable_body,
                context: Some(body.context.clone()),
                frame: Some(body.frame.clone()),
            }),
            budget.work(),
        ))
    }
}
