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
    member_name, DeclarationForm, NativeTypeRef, OperationDecl, SelectedTypeRef,
};
use super::native_join_budget::PackageReadStage;
use super::operation_effects::own_operation;
use super::stage_storage::{self, Path, Refusal, Site};
use super::{
    CheckedBindingNameKind, CheckedNodeId, CheckedNodeKind, CheckedNodeOwner, CheckedTerm,
    ExpressionForm, LiteralKind, ProbabilisticForm, RawCheckedPackageWireV2, ScalarTypeForm,
    TimedModelForm, ValueForm, WorkMeter,
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

/// The literal kind a support value of a parameter of `slot`'s type has.
fn literal_kind(slot: &super::model_members::TypedSlot) -> Option<LiteralKind> {
    let single = slot.multiplicity.lower == 1 && slot.multiplicity.upper == Some(1);
    if !single {
        return None;
    }
    Some(match slot.type_ref.as_ref()? {
        SelectedTypeRef::Native(native) => match native {
            NativeTypeRef::Boolean => LiteralKind::Boolean,
            NativeTypeRef::Integer => LiteralKind::Integer,
            NativeTypeRef::Rational => LiteralKind::Rational,
            NativeTypeRef::Decimal => LiteralKind::Decimal,
            NativeTypeRef::Float32 => LiteralKind::Float32Bits,
            NativeTypeRef::Float64 => LiteralKind::Float64Bits,
            NativeTypeRef::Text => LiteralKind::Text,
            NativeTypeRef::String => LiteralKind::String,
            NativeTypeRef::Uuid => LiteralKind::Uuid,
            NativeTypeRef::Timestamp => LiteralKind::Timestamp,
            NativeTypeRef::Duration => LiteralKind::Quantity,
            NativeTypeRef::Bytes | NativeTypeRef::JsonObject | NativeTypeRef::Clock => return None,
        },
        SelectedTypeRef::Unit(_) => LiteralKind::Quantity,
        SelectedTypeRef::Declared(_) => LiteralKind::Enum,
        SelectedTypeRef::Map(..) => return None,
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
    ) -> Result<(&'a CheckedNodeId, &'a OperationDecl), ValidationFailure> {
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
            Some(found) => Ok((&self.nodes[model_position].node_id, found)),
            None => Err(invalid(meter, holder, operation_at)),
        }
    }

    /// Whether a timed model rooted on `model` has a `dense` time source.
    fn is_dense(&self, model: &CheckedNodeId) -> bool {
        let root = (0..self.nodes.len()).find(|&position| {
            self.kinds.get(position) == Some(&CheckedNodeKind::TimedModel(TimedModelForm::Model))
                && bound(&self.terms[position], 0).and_then(target) == Some(model)
        });
        root.and_then(|root| self.position(bound(&self.terms[root], 2).and_then(target)?))
            .is_some_and(|source| bound(&self.terms[source], 0).and_then(text) == Some("dense"))
    }

    /// Whether a function reads state: an expression it reaches by
    /// dependencies, through functions, expressions and values only, reads
    /// the pre-state, a dereferenced object, a population or a presence.
    fn reads_state(
        &self,
        meter: &mut WorkMeter,
        holder: usize,
        start: usize,
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
                CheckedNodeKind::State(_)
                | CheckedNodeKind::Expression(
                    ExpressionForm::PreRead
                    | ExpressionForm::PresenceRead
                    | ExpressionForm::ValueRead
                    | ExpressionForm::Deref
                    | ExpressionForm::Query
                    | ExpressionForm::Reachability,
                ) => return Ok(true),
                CheckedNodeKind::Function(_)
                | CheckedNodeKind::Expression(_)
                | CheckedNodeKind::Value(_) => {
                    for dependency in &self.nodes[position].dependencies {
                        if let Some(next) = self.position(dependency) {
                            if !seen[next] {
                                seen[next] = true;
                                pending.push(next);
                            }
                        }
                    }
                }
                _ => {}
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
                        | ScalarTypeForm::CarriedQuantity
                        | ScalarTypeForm::BoundedQuantity
                )
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
        if !matches_parameters || self.reads_state(meter, holder, function)? {
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
        match self.is_dense(model) {
            true => Ok(()),
            false => Err(invalid(meter, holder, binding_at(holder, 0).key("target"))),
        }
    }

    /// FR-640 Reader rule 3: the weights of a workload cover the operations of
    /// its model in ascending order.
    fn workload(
        &self,
        meter: &mut WorkMeter,
        holder: usize,
        body: &CheckedTerm,
    ) -> Result<(), ValidationFailure> {
        let model_at = binding_at(holder, 0).key("target");
        let model = bound(body, 0)
            .and_then(target)
            .and_then(|id| self.position(id))
            .filter(|&position| matches!(self.kinds[position], CheckedNodeKind::Model(_)));
        let Some(model) = model else {
            return Err(invalid(meter, holder, model_at));
        };
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
                .and_then(|position| {
                    self.models
                        .owner_by_key(&self.structural_keys[position])
                        .filter(|owner| {
                            matches!(
                                owner.form,
                                DeclarationForm::ObjectType
                                    | DeclarationForm::Process
                                    | DeclarationForm::StateMachine
                            )
                        })
                });
            let Some(owner) = declared else {
                return Err(invalid(meter, holder, declaring_at));
            };
            let found = owner
                .package
                .object_types
                .get(owner.node)
                .and_then(|declared| own_operation(declared, name));
            if found.is_none() {
                return Err(invalid(meter, holder, operation_at));
            }
            if self.position(declaring) != Some(model) && !self.inherits(model, declaring) {
                return Err(invalid(meter, holder, declaring_at));
            }
            if previous.is_some_and(|earlier| earlier >= (declaring, name)) {
                return Err(invalid(meter, holder, row_at));
            }
            previous = Some((declaring, name));
            if self.position(declaring) == Some(model) {
                named.push(name);
            }
        }
        // A model that declares supertypes also exposes inherited operations
        // that the weights must cover; those are not judged here.
        let own = self
            .models
            .owner_by_key(&self.structural_keys[model])
            .and_then(|owner| owner.package.object_types.get(owner.node));
        if let Some(declared) = own.filter(|declared| declared.supertypes.is_empty()) {
            for operation in &declared.operations {
                if !named.contains(&member_name(&operation.identity)) {
                    return Err(invalid(meter, holder, weights_at.key("members")));
                }
            }
        }
        Ok(())
    }

    /// Whether the model at `model` declares the node `declaring` among its
    /// supertypes, by the model's declared supertype names.
    fn inherits(&self, model: usize, declaring: &CheckedNodeId) -> bool {
        let Some(owner) = self.models.owner_by_key(&self.structural_keys[model]) else {
            return false;
        };
        let Some(declared) = owner.package.object_types.get(owner.node) else {
            return false;
        };
        let Some(ancestor) = self
            .position(declaring)
            .and_then(|position| self.models.owner_by_key(&self.structural_keys[position]))
        else {
            return false;
        };
        ancestor.package.identity == owner.package.identity
            && declared
                .supertypes
                .iter()
                .any(|supertype| supertype.as_ref() == ancestor.node)
    }
}

/// Checks every `probabilistic` node against the nodes it joins, in ascending
/// node-id order, and refuses the first failure.
#[ix_trace_rs::string_edge]
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
                let (model, operation) = join.operation(meter, position, body, 0, 1)?;
                let parameter_at = binding_at(position, 2).key("value");
                let name = bound(body, 2).and_then(text).unwrap_or_default();
                let Some(declared) = operation
                    .parameters
                    .iter()
                    .find(|declared| declared.name.as_deref() == Some(name))
                else {
                    return Err(invalid(meter, position, parameter_at));
                };
                let expected = literal_kind(declared);
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
                    let kind = match bound(entry, 0) {
                        Some(CheckedTerm::Literal { value_kind, .. }) => Some(*value_kind),
                        _ => None,
                    };
                    if kind.is_none() || kind != expected {
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
                let (model, operation) = join.operation(meter, position, body, 0, 1)?;
                join.reward(meter, position, body, operation)?;
                (
                    2,
                    Some(model),
                    member_name(&operation.identity),
                    bound(body, 2).and_then(text).unwrap_or_default(),
                )
            }
            ProbabilisticForm::DelayDistribution => {
                let (model, operation) = join.operation(meter, position, body, 0, 1)?;
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
