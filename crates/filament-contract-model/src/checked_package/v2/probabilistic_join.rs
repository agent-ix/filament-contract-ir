// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The FSpec FR-640 Reader rules that join a `probabilistic` node to other
//! nodes: the operation reference, the parameter and its support values, the
//! workload's operations, the reward's function and value type, the delay's
//! model time, and the `conflicting-definition` between two nodes.
//!
//! Implements: FR-369 "Behavior" (join checks).

use super::model_fields::RetainedModels;
use super::model_members::{
    member_name, DeclarationForm, DomainModel, MemberType, NativeTypeRef, OperationDecl,
    SelectedTypeRef, TypedSlot,
};
use super::native_join_budget::PackageReadStage;
use super::operation_effects::own_operation;
use super::stage_storage::{self, Path, Refusal, Site};
use super::{
    BoundedDomainForm, CheckedBindingNameKind, CheckedNodeId, CheckedNodeKind, CheckedNodeOwner,
    CheckedTerm, ExpressionForm, LiteralKind, ProbabilisticForm, RawCheckedPackageWireV2,
    ScalarTypeForm, TimedModelForm, ValueForm, WorkMeter,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
};

const FORM: &str = "probabilistic";

/// The term bound to the `index`th member of an aggregate body.
pub(super) fn bound(term: &CheckedTerm, index: usize) -> Option<&CheckedTerm> {
    match members(term).get(index)? {
        CheckedTerm::Binding { value, .. } => Some(value),
        _ => None,
    }
}

pub(super) fn members(term: &CheckedTerm) -> &[CheckedTerm] {
    match term {
        CheckedTerm::Aggregate { members } => members,
        _ => &[],
    }
}

pub(super) fn target(term: &CheckedTerm) -> Option<&CheckedNodeId> {
    match term {
        CheckedTerm::Reference { target } => Some(target),
        _ => None,
    }
}

pub(super) fn text(term: &CheckedTerm) -> Option<&str> {
    match term {
        CheckedTerm::Literal {
            value_kind: LiteralKind::Text,
            value,
            ..
        } => value.as_str(),
        _ => None,
    }
}

/// The name a binding member carries.
pub(super) fn binding_name(term: &CheckedTerm) -> Option<&str> {
    match term {
        CheckedTerm::Binding { name, .. } => Some(match name.kind() {
            CheckedBindingNameKind::Parameters => "parameters",
            CheckedBindingNameKind::Body => "body",
            CheckedBindingNameKind::Optional => "optional",
            CheckedBindingNameKind::Clause => "clause",
            CheckedBindingNameKind::Other(name) => name,
        }),
        _ => None,
    }
}

/// One conflict key: the form, the declaring node, the operation and the
/// further name the form's definition is unique by.
type Key<'a> = (u8, Option<&'a CheckedNodeId>, &'a str, &'a str);

struct Join<'a> {
    nodes: &'a [super::RawSemanticNodeV2],
    kinds: &'a [CheckedNodeKind],
    terms: &'a [CheckedTerm],
    models: &'a RetainedModels,
    /// Graph positions in node-id order.
    by_id: Vec<usize>,
    structural_keys: &'a [std::sync::Arc<CheckedNodeId>],
}

fn node_at(position: usize) -> Path<'static> {
    Path::root()
        .key("semantic_graph")
        .key("nodes")
        .index(position)
}

fn binding_at(position: usize, member: usize) -> Path<'static> {
    node_at(position)
        .key("body")
        .key("members")
        .index(member)
        .key("value")
}

fn fail(
    meter: &mut WorkMeter,
    holder: usize,
    at: Path<'static>,
    cause: Cause,
) -> ValidationFailure {
    stage_storage::refuse(
        meter,
        &Site::new(PackageReadStage::Graph, holder, FORM, at),
        Refusal::new(Code::InvalidPackage, at).because(cause),
    )
}

fn invalid(meter: &mut WorkMeter, holder: usize, at: Path<'static>) -> ValidationFailure {
    fail(meter, holder, at, Cause::InvalidValue)
}

fn visit(meter: &mut WorkMeter, holder: usize, at: Path<'static>) -> Result<(), ValidationFailure> {
    stage_storage::work(
        meter,
        &Site::new(PackageReadStage::Graph, holder, FORM, at),
        1,
    )
}

/// The literal kind of a value of the member type `kind`.
fn member_literal_kind(kind: &MemberType) -> Option<LiteralKind> {
    Some(match kind {
        MemberType::Boolean => LiteralKind::Boolean,
        MemberType::Integer | MemberType::IntRange(_) => LiteralKind::Integer,
        MemberType::Float32 => LiteralKind::Float32Bits,
        MemberType::Float64 => LiteralKind::Float64Bits,
        MemberType::String => LiteralKind::String,
        MemberType::Uuid => LiteralKind::Uuid,
        MemberType::Timestamp => LiteralKind::Timestamp,
        MemberType::RationalRange(_) => LiteralKind::Rational,
        MemberType::CarriedRational(..) | MemberType::Quantity(_) => LiteralKind::Quantity,
        MemberType::TextBounds(_) => LiteralKind::Text,
        _ => return None,
    })
}

/// The unit node a quantity support value of a parameter of `slot`'s type is
/// typed at, when the type is a quantity.
fn literal_unit(package: &DomainModel, slot: &TypedSlot) -> Option<CheckedNodeId> {
    match package.slot_type(slot)? {
        MemberType::Quantity(unit) | MemberType::CarriedRational(_, unit) => Some(unit),
        _ => None,
    }
}

/// The literal kind a support value of a parameter of `slot`'s type has: the
/// kind of the member type FR-322 step 4 derives, and for a type that step
/// gives no member type, the kind of the declared name (a native Rational,
/// Decimal or Text, or a declared enumeration).
fn literal_kind(package: &DomainModel, slot: &TypedSlot) -> Option<LiteralKind> {
    if let Some(kind) = package.slot_type(slot) {
        return member_literal_kind(&kind);
    }
    let single = slot.multiplicity.lower == 1 && slot.multiplicity.upper == Some(1);
    if !single {
        return None;
    }
    Some(match slot.type_ref.as_ref()? {
        SelectedTypeRef::Native(NativeTypeRef::Rational) => LiteralKind::Rational,
        SelectedTypeRef::Native(NativeTypeRef::Decimal) => LiteralKind::Decimal,
        SelectedTypeRef::Native(NativeTypeRef::Text) => LiteralKind::Text,
        SelectedTypeRef::Declared(_) => LiteralKind::Enum,
        _ => return None,
    })
}

impl<'a> Join<'a> {
    fn position(&self, id: &CheckedNodeId) -> Option<usize> {
        let found = self
            .by_id
            .binary_search_by(|&position| self.nodes[position].node_id.cmp(id))
            .ok()?;
        Some(self.by_id[found])
    }

    /// FR-640 Reader rule 1: the operation of an operation reference whose
    /// `declaring` and `operation` bindings are the members at `declaring`
    /// and `operation` of `body`.
    fn operation(
        &self,
        meter: &mut WorkMeter,
        holder: usize,
        body: &CheckedTerm,
        declaring: usize,
        operation: usize,
    ) -> Result<(&'a CheckedNodeId, &'a OperationDecl, &'a DomainModel), ValidationFailure> {
        let declaring_at = binding_at(holder, declaring).key("target");
        let operation_at = binding_at(holder, operation).key("value");
        let Some(model) = bound(body, declaring).and_then(target) else {
            return Err(invalid(meter, holder, declaring_at));
        };
        let Some(model_position) = self.position(model) else {
            return Err(invalid(meter, holder, declaring_at));
        };
        let owner = matches!(
            self.kinds.get(model_position),
            Some(CheckedNodeKind::Model(_))
        )
        .then(|| {
            self.models
                .owner_by_key(&self.structural_keys[model_position])
        })
        .flatten()
        .filter(|owner| {
            matches!(
                owner.form,
                DeclarationForm::ObjectType
                    | DeclarationForm::Process
                    | DeclarationForm::StateMachine
            )
        });
        let Some(owner) = owner else {
            return Err(invalid(meter, holder, declaring_at));
        };
        let declared = owner.package.object_types.get(owner.node);
        let Some(name) = bound(body, operation).and_then(text) else {
            return Err(invalid(meter, holder, operation_at));
        };
        match declared.and_then(|declared| own_operation(declared, name)) {
            Some(found) => Ok((&self.nodes[model_position].node_id, found, owner.package)),
            None => Err(invalid(meter, holder, operation_at)),
        }
    }

    /// Whether a timed model rooted on `model` has a `dense` time source.
    /// One validation visit is charged for each graph node scanned for the
    /// root.
    fn is_dense(
        &self,
        meter: &mut WorkMeter,
        holder: usize,
        model: &CheckedNodeId,
    ) -> Result<bool, ValidationFailure> {
        for position in 0..self.nodes.len() {
            visit(meter, holder, node_at(holder))?;
            if self.kinds.get(position) == Some(&CheckedNodeKind::TimedModel(TimedModelForm::Model))
                && bound(&self.terms[position], 0).and_then(target) == Some(model)
            {
                return Ok(bound(&self.terms[position], 2)
                    .and_then(target)
                    .and_then(|source| self.position(source))
                    .is_some_and(|source| {
                        bound(&self.terms[source], 0).and_then(text) == Some("dense")
                    }));
            }
        }
        Ok(false)
    }

    /// Whether a function reads anything but the operation's parameters and
    /// constants (FSpec FR-405 "Rewards"). The walk follows the dependencies of
    /// functions, values and the expressions that compute a value; it admits a
    /// parameter of the operation (one of `parameters`), a type and a constant, and it stops at the first node of any
    /// other kind, among them every `state` node and the `pre_read`,
    /// `presence_read`, `value_read`, `deref`, `query` and `reachability`
    /// expressions. One validation visit is charged for each node reached.
    fn reads_beyond_parameters(
        &self,
        meter: &mut WorkMeter,
        holder: usize,
        start: usize,
        parameters: &[usize],
    ) -> Result<bool, ValidationFailure> {
        let at = node_at(holder);
        let site = Site::new(PackageReadStage::Graph, holder, FORM, at);
        let mut seen: Vec<bool> = stage_storage::vec(meter, &site, self.nodes.len())?;
        seen.resize(self.nodes.len(), false);
        let mut pending: Vec<usize> = stage_storage::vec(meter, &site, self.nodes.len())?;
        pending.push(start);
        seen[start] = true;
        while let Some(position) = pending.pop() {
            stage_storage::work(meter, &site, 1)?;
            match self.kinds[position] {
                CheckedNodeKind::ScalarType(_)
                | CheckedNodeKind::CompositeType(_)
                | CheckedNodeKind::BoundedDomain(_) => {}
                CheckedNodeKind::Value(ValueForm::Parameter) => {
                    if !parameters.contains(&position) {
                        return Ok(true);
                    }
                }
                CheckedNodeKind::Function(_)
                | CheckedNodeKind::Value(_)
                | CheckedNodeKind::Expression(
                    ExpressionForm::Reference
                    | ExpressionForm::Call
                    | ExpressionForm::Unary
                    | ExpressionForm::Binary
                    | ExpressionForm::Conditional
                    | ExpressionForm::Case
                    | ExpressionForm::Let
                    | ExpressionForm::Quantify
                    | ExpressionForm::Collection
                    | ExpressionForm::Conversion,
                ) => {
                    for dependency in &self.nodes[position].dependencies {
                        if let Some(next) = self.position(dependency) {
                            if !seen[next] {
                                seen[next] = true;
                                pending.push(next);
                            }
                        }
                    }
                }
                _ => return Ok(true),
            }
        }
        Ok(false)
    }

    /// FR-640 Reader rule 4: the reward's `value_type` and `expression`.
    fn reward(
        &self,
        meter: &mut WorkMeter,
        holder: usize,
        body: &CheckedTerm,
        operation: &OperationDecl,
    ) -> Result<(), ValidationFailure> {
        let value_type_at = binding_at(holder, 3).key("target");
        let Some(value_type) = bound(body, 3).and_then(target) else {
            return Err(invalid(meter, holder, value_type_at));
        };
        let typed = self.position(value_type).is_some_and(|position| {
            matches!(
                self.kinds[position],
                CheckedNodeKind::ScalarType(
                    ScalarTypeForm::Integer
                        | ScalarTypeForm::Unit
                        | ScalarTypeForm::CompoundUnit
                        | ScalarTypeForm::CarriedQuantity
                        | ScalarTypeForm::BoundedQuantity
                ) | CheckedNodeKind::BoundedDomain(BoundedDomainForm::IntegerRange)
            )
        });
        if !typed {
            return Err(invalid(meter, holder, value_type_at));
        }
        let expression_at = binding_at(holder, 4).key("target");
        let function = bound(body, 4)
            .and_then(target)
            .and_then(|function| self.position(function))
            .filter(|&position| matches!(self.kinds[position], CheckedNodeKind::Function(_)));
        let Some(function) = function else {
            return Err(invalid(meter, holder, expression_at));
        };
        let node = &self.nodes[function];
        let owned = matches!(
            node.owner.as_ref(),
            Some(CheckedNodeOwner::Model { node: member, .. })
                if member.as_ref() == operation.identity.as_ref()
        );
        if !owned || &node.semantic_type != value_type {
            return Err(invalid(meter, holder, expression_at));
        }
        let parameters = bound(&self.terms[function], 0).map(members);
        let matches_parameters = parameters.is_some_and(|listed| {
            listed.len() == operation.parameters.len()
                && listed
                    .iter()
                    .zip(&operation.parameters)
                    .all(|(listed, declared)| {
                        let Some(position) = target(listed).and_then(|id| self.position(id)) else {
                            return false;
                        };
                        self.kinds[position] == CheckedNodeKind::Value(ValueForm::Parameter)
                            && bound(&self.terms[position], 0).and_then(text)
                                == declared.name.as_deref()
                    })
        });
        let listed: Vec<usize> = parameters
            .unwrap_or_default()
            .iter()
            .filter_map(|listed| target(listed).and_then(|id| self.position(id)))
            .collect();
        if !matches_parameters || self.reads_beyond_parameters(meter, holder, function, &listed)? {
            return Err(invalid(meter, holder, expression_at));
        }
        Ok(())
    }

    /// FR-640 Reader rule 5: the delay's model is `time dense`.
    fn delay(
        &self,
        meter: &mut WorkMeter,
        holder: usize,
        model: &CheckedNodeId,
    ) -> Result<(), ValidationFailure> {
        match self.is_dense(meter, holder, model)? {
            true => Ok(()),
            false => Err(invalid(meter, holder, binding_at(holder, 0).key("target"))),
        }
    }

    /// The selected declaration `node` and its supertypes, transitively, as
    /// identities of `package`. One validation visit is charged for each
    /// supertype edge followed.
    fn lineage(
        &self,
        meter: &mut WorkMeter,
        holder: usize,
        package: &'a DomainModel,
        node: &'a str,
    ) -> Result<Vec<&'a str>, ValidationFailure> {
        let mut reached = vec![node];
        let mut next = 0;
        while let Some(&current) = reached.get(next) {
            next += 1;
            let Some(declared) = package.object_types.get(current) else {
                continue;
            };
            for supertype in &declared.supertypes {
                visit(meter, holder, node_at(holder))?;
                if !reached.contains(&supertype.as_ref()) {
                    reached.push(supertype);
                }
            }
        }
        Ok(reached)
    }

    /// FR-640 Reader rule 3: the weights of a workload cover the operations of
    /// its model, own and inherited, once each and in ascending order.
    fn workload(
        &self,
        meter: &mut WorkMeter,
        holder: usize,
        body: &CheckedTerm,
    ) -> Result<(), ValidationFailure> {
        let model_at = binding_at(holder, 0).key("target");
        let owner = bound(body, 0)
            .and_then(target)
            .and_then(|id| self.position(id))
            .filter(|&position| matches!(self.kinds[position], CheckedNodeKind::Model(_)))
            .and_then(|position| self.models.owner_by_key(&self.structural_keys[position]))
            .filter(|owner| is_operation_owner(owner.form));
        let Some(owner) = owner else {
            return Err(invalid(meter, holder, model_at));
        };
        let lineage = self.lineage(meter, holder, owner.package, owner.node)?;
        let weights_at = binding_at(holder, 2);
        let rows = bound(body, 2).map(members).unwrap_or_default();
        let mut previous: Option<(&CheckedNodeId, &str)> = None;
        let mut named: Vec<&str> = stage_storage::vec(
            meter,
            &Site::new(PackageReadStage::Graph, holder, FORM, weights_at),
            rows.len(),
        )?;
        for (index, row) in rows.iter().enumerate() {
            let row_at = weights_at.key("members").index(index);
            visit(meter, holder, row_at)?;
            let (Some(declaring), Some(name)) =
                (bound(row, 0).and_then(target), bound(row, 1).and_then(text))
            else {
                return Err(invalid(meter, holder, row_at));
            };
            let declaring_at = row_at.key("members").index(0).key("value").key("target");
            let operation_at = row_at.key("members").index(1).key("value").key("value");
            let declared = self
                .position(declaring)
                .filter(|&position| matches!(self.kinds[position], CheckedNodeKind::Model(_)))
                .and_then(|position| self.models.owner_by_key(&self.structural_keys[position]))
                .filter(|declared| {
                    is_operation_owner(declared.form)
                        && declared.package.identity == owner.package.identity
                        && lineage.contains(&declared.node)
                });
            let Some(declared) = declared else {
                return Err(invalid(meter, holder, declaring_at));
            };
            let found = declared
                .package
                .object_types
                .get(declared.node)
                .and_then(|declared| own_operation(declared, name));
            if found.is_none() {
                return Err(invalid(meter, holder, operation_at));
            }
            if previous.is_some_and(|earlier| earlier >= (declaring, name)) || named.contains(&name)
            {
                return Err(invalid(meter, holder, row_at));
            }
            previous = Some((declaring, name));
            named.push(name);
        }
        for node in &lineage {
            let Some(declared) = owner.package.object_types.get(*node) else {
                continue;
            };
            for operation in &declared.operations {
                if !named.contains(&member_name(&operation.identity)) {
                    return Err(invalid(meter, holder, weights_at.key("members")));
                }
            }
        }
        Ok(())
    }
}

fn is_operation_owner(form: DeclarationForm) -> bool {
    matches!(
        form,
        DeclarationForm::ObjectType | DeclarationForm::Process | DeclarationForm::StateMachine
    )
}

/// Checks every `probabilistic` node against the nodes it joins, in ascending
/// node-id order, and refuses the first failure.
pub(super) fn validate(
    wire: &RawCheckedPackageWireV2,
    kinds: &[CheckedNodeKind],
    terms: &[CheckedTerm],
    models: &RetainedModels,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let nodes = &wire.semantic_graph.nodes;
    let count = kinds
        .iter()
        .filter(|kind| matches!(kind, CheckedNodeKind::Probabilistic(_)))
        .count();
    if count == 0 {
        return Ok(());
    }
    let site = Site::new(PackageReadStage::Graph, 0, FORM, Path::root());
    let mut by_id: Vec<usize> = stage_storage::vec(meter, &site, nodes.len())?;
    by_id.extend(0..nodes.len());
    by_id.sort_unstable_by(|&left, &right| nodes[left].node_id.cmp(&nodes[right].node_id));
    let join = Join {
        nodes,
        kinds,
        terms,
        models,
        by_id,
        structural_keys,
    };
    let mut seen: Vec<Key<'_>> = stage_storage::vec(meter, &site, count)?;
    for &position in &join.by_id {
        let CheckedNodeKind::Probabilistic(form) = kinds[position] else {
            continue;
        };
        let body = &terms[position];
        visit(meter, position, node_at(position))?;
        let key: Key<'_> = match form {
            ProbabilisticForm::RandomParameter => {
                let (model, operation, package) = join.operation(meter, position, body, 0, 1)?;
                let parameter_at = binding_at(position, 2).key("value");
                let name = bound(body, 2).and_then(text).unwrap_or_default();
                let Some(declared) = operation
                    .parameters
                    .iter()
                    .find(|declared| declared.name.as_deref() == Some(name))
                else {
                    return Err(invalid(meter, position, parameter_at));
                };
                let expected = literal_kind(package, declared);
                let unit = literal_unit(package, declared);
                let support_at = binding_at(position, 3);
                for (row, entry) in bound(body, 3)
                    .map(members)
                    .unwrap_or_default()
                    .iter()
                    .enumerate()
                {
                    let value_at = support_at
                        .key("members")
                        .index(row)
                        .key("members")
                        .index(0)
                        .key("value");
                    let (kind, typed_at) = match bound(entry, 0) {
                        Some(CheckedTerm::Literal {
                            value_kind,
                            type_id,
                            ..
                        }) => (Some(*value_kind), Some(type_id)),
                        _ => (None, None),
                    };
                    if kind.is_none()
                        || kind != expected
                        || unit.as_ref().is_some_and(|unit| typed_at != Some(unit))
                    {
                        return Err(invalid(meter, position, value_at));
                    }
                }
                (0, Some(model), member_name(&operation.identity), name)
            }
            ProbabilisticForm::Workload => {
                join.workload(meter, position, body)?;
                (
                    1,
                    None,
                    "",
                    bound(body, 1).and_then(text).unwrap_or_default(),
                )
            }
            ProbabilisticForm::Reward => {
                let (model, operation, _) = join.operation(meter, position, body, 0, 1)?;
                join.reward(meter, position, body, operation)?;
                (
                    2,
                    Some(model),
                    member_name(&operation.identity),
                    bound(body, 2).and_then(text).unwrap_or_default(),
                )
            }
            ProbabilisticForm::DelayDistribution => {
                let (model, operation, _) = join.operation(meter, position, body, 0, 1)?;
                join.delay(meter, position, model)?;
                (3, Some(model), member_name(&operation.identity), "")
            }
        };
        match seen.binary_search(&key) {
            Ok(_) => {
                return Err(fail(
                    meter,
                    position,
                    node_at(position),
                    Cause::ConflictingDefinition,
                ))
            }
            Err(at) => seen.insert(at, key),
        }
    }
    Ok(())
}
