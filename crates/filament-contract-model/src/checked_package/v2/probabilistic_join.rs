// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The FSpec FR-640 Reader rules that join a `probabilistic` node to other
//! nodes: the operation reference, the parameter and its support values, the
//! workload's operations, the reward's function and value type, the delay's
//! model time, and the `conflicting-definition` between two nodes.
//!
//! Implements: FR-369 "Behavior" (join checks).

use unicode_normalization::{is_nfc, is_nfd, is_nfkc, is_nfkd};

use super::model_fields::RetainedModels;
use super::model_members::{
    member_name, DeclarationForm, DomainModel, MemberType, NativeTypeRef, OperationDecl,
    SelectedTypeRef, TypedSlot,
};
use super::native_join_budget::PackageReadStage;
use super::operation_effects::own_operation;
use super::source_object::is_source_declared_object_type;
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

mod claims;

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

/// The operation an operation reference names: a selected model's operation
/// or, for a source-declared `declaring` node, the operation's name, which the
/// identity-row join resolves.
enum Operation<'a> {
    Selected {
        model: &'a CheckedNodeId,
        operation: &'a OperationDecl,
        package: &'a DomainModel,
    },
    Source {
        model: &'a CheckedNodeId,
        position: usize,
        name: &'a str,
    },
}

impl<'a> Operation<'a> {
    fn model(&self) -> &'a CheckedNodeId {
        match self {
            Self::Selected { model, .. } | Self::Source { model, .. } => model,
        }
    }

    fn name(&self) -> &'a str {
        match self {
            Self::Selected { operation, .. } => member_name(&operation.identity),
            Self::Source { name, .. } => name,
        }
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
        MemberType::Variant(_) => LiteralKind::Enum,
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

/// The scalar count of `value` as written, or its UTF-8 byte count under
/// `binary-utf8`, when `value` is in the normal form `profile` names; `None`
/// when it is not (FSpec FR-322 "Text literal membership": the reader applies
/// no normalization). The profile is the declaration's wire text, so its words
/// gate the branch.
#[ix_trace_rs::string_edge]
fn text_length(value: &str, profile: &str) -> Option<usize> {
    let normal = match profile {
        "nfc" => is_nfc(value),
        "nfd" => is_nfd(value),
        "nfkc" => is_nfkc(value),
        "nfkd" => is_nfkd(value),
        _ => true,
    };
    normal.then(|| match profile {
        "binary-utf8" => value.len(),
        _ => value.chars().count(),
    })
}

/// Where a support value that is no member of its parameter's type is refused.
pub(super) enum NotMember {
    /// At the literal: outside the bounds, or a non-canonical integer or text.
    Literal,
    /// At the literal's `value` member: a rational spelled other than its one
    /// canonical spelling (FSpec FR-322 "Scalar literal values").
    Spelling,
}

/// The membership rule of the values of a type: the bounds
/// `bounded_not_member` checks, shared by the strict read and the identity-row
/// join of FR-369.
#[derive(Clone, Copy)]
pub(super) enum Bounded<'a> {
    Boolean,
    Integers(super::model_members::IntegerBounds),
    Rationals(super::model_members::RationalBounds),
    Text {
        lower: u64,
        upper: u64,
        profile: &'a str,
    },
    Free,
}

impl<'a> Bounded<'a> {
    /// The rule of the member type `kind`.
    fn of(kind: &'a MemberType) -> Self {
        match kind {
            MemberType::Boolean => Self::Boolean,
            MemberType::IntRange(bounds) => Self::Integers(*bounds),
            MemberType::RationalRange(bounds) | MemberType::CarriedRational(bounds, _) => {
                Self::Rationals(*bounds)
            }
            MemberType::TextBounds(bounds) => Self::Text {
                lower: bounds.lower,
                upper: bounds.upper,
                profile: &bounds.profile,
            },
            MemberType::Integer
            | MemberType::Float32
            | MemberType::Float64
            | MemberType::String
            | MemberType::Uuid
            | MemberType::Timestamp
            | MemberType::Bytes
            | MemberType::JsonObject
            | MemberType::Clock
            | MemberType::Reference(_)
            | MemberType::Record(_)
            | MemberType::Tuple(_)
            | MemberType::Variant(_)
            | MemberType::Quantity(_)
            | MemberType::Option(_)
            | MemberType::Collection { .. } => Self::Free,
        }
    }
}

/// The exact-number parses a membership check asks of its meter: the strict
/// read parses through its read-storage reservation, the identity-row join
/// pays the logical storage of each parse ahead of it.
pub(super) trait ExactParse {
    /// What a refused charge returns.
    type Error;

    /// `text` as an exact integer, `None` when it is not the canonical spelling.
    fn exact_integer(&mut self, text: &str)
        -> Result<Option<filament_exact::Integer>, Self::Error>;

    /// `text` as a canonical reduced rational: its numerator and denominator.
    fn canonical_rational(
        &mut self,
        text: &str,
    ) -> Result<Option<(filament_exact::Integer, filament_exact::Integer)>, Self::Error>;
}

impl ExactParse for WorkMeter<'_> {
    type Error = ValidationFailure;

    fn exact_integer(
        &mut self,
        text: &str,
    ) -> Result<Option<filament_exact::Integer>, ValidationFailure> {
        WorkMeter::exact_integer(self, text)
    }

    fn canonical_rational(
        &mut self,
        text: &str,
    ) -> Result<Option<(filament_exact::Integer, filament_exact::Integer)>, ValidationFailure> {
        WorkMeter::canonical_rational(self, text)
    }
}

/// Why `value` is no member of the type `rule` bounds, or `None` when it is
/// one: a spelling that is not the one canonical spelling of an integer or
/// rational (FSpec FR-322), a number outside `Int[lo, hi]` or the bounds of
/// `Rational[n1, n2; d1, d2]` (also for a quantity over one), a text that is
/// not in its profile's normal form or whose length is outside
/// `Text[min, max; profile]`, or a Boolean whose value is no JSON boolean.
/// Every exact number is parsed through `meter`.
pub(super) fn bounded_not_member<M: ExactParse>(
    meter: &mut M,
    rule: Bounded<'_>,
    value: &super::CheckedJson,
) -> Result<Option<NotMember>, M::Error> {
    use filament_exact::Integer as Exact;
    if matches!(rule, Bounded::Boolean) {
        return Ok(value.as_bool().is_none().then_some(NotMember::Literal));
    }
    let Some(spelled) = value.as_str() else {
        return Ok(Some(NotMember::Literal));
    };
    let outside = match rule {
        Bounded::Integers(bounds) => match meter.exact_integer(spelled)? {
            Some(found) => found < Exact::from(bounds.lower) || found > Exact::from(bounds.upper),
            None => true,
        },
        Bounded::Rationals(bounds) => match meter.canonical_rational(spelled)? {
            Some((numerator, denominator)) => {
                numerator < Exact::from(bounds.numerator_min)
                    || numerator > Exact::from(bounds.numerator_max)
                    || denominator < Exact::from(bounds.denominator_min)
                    || denominator > Exact::from(bounds.denominator_max)
            }
            None => return Ok(Some(NotMember::Spelling)),
        },
        Bounded::Text {
            lower,
            upper,
            profile,
        } => text_length(spelled, profile)
            .and_then(|length| u64::try_from(length).ok())
            .is_none_or(|length| length < lower || length > upper),
        Bounded::Boolean | Bounded::Free => false,
    };
    Ok(outside.then_some(NotMember::Literal))
}

/// Why the support value `term` is no member of the type `slot` has, or `None`
/// when it is one: the bounds of `bounded_not_member`, and for an enumeration
/// a literal typed at another declaration or naming a case the declaration
/// lacks.
fn not_member(
    meter: &mut WorkMeter,
    models: &RetainedModels,
    package: &DomainModel,
    slot: &TypedSlot,
    term: &CheckedTerm,
) -> Result<Option<NotMember>, ValidationFailure> {
    let CheckedTerm::Literal { value, type_id, .. } = term else {
        return Ok(None);
    };
    let member = package.slot_type(slot);
    if let Some(MemberType::Variant(declaration)) = &member {
        let declared = value
            .as_str()
            .is_some_and(|case| models.selected_variant_case(declaration, case).is_some());
        return Ok((!declared || type_id != declaration).then_some(NotMember::Literal));
    }
    bounded_not_member(
        meter,
        member.as_ref().map_or(Bounded::Free, Bounded::of),
        value,
    )
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
        body: &'a CheckedTerm,
        declaring: usize,
        operation: usize,
    ) -> Result<Operation<'a>, ValidationFailure> {
        let declaring_at = binding_at(holder, declaring).key("target");
        let operation_at = binding_at(holder, operation).key("value");
        let Some(model) = bound(body, declaring).and_then(target) else {
            return Err(invalid(meter, holder, declaring_at));
        };
        let Some(model_position) = self.position(model) else {
            return Err(invalid(meter, holder, declaring_at));
        };
        if self.is_source_object(model_position) {
            // A source-declared node has no domain document: the strict read
            // checks its kind and `SourceOwner` only, and the identity-row
            // join resolves the operation (FSpec FR-640 Reader rule 1).
            let Some(name) = bound(body, operation).and_then(text) else {
                return Err(invalid(meter, holder, operation_at));
            };
            return Ok(Operation::Source {
                model: &self.nodes[model_position].node_id,
                position: model_position,
                name,
            });
        }
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
            Some(found) => Ok(Operation::Selected {
                model: &self.nodes[model_position].node_id,
                operation: found,
                package: owner.package,
            }),
            None => Err(invalid(meter, holder, operation_at)),
        }
    }

    /// Whether the node at `position` is a source-declared object type: a
    /// `model`/`object_type` node with a `declaration` and a `SourceOwner`
    /// (FSpec FR-322 "Source-declared object types").
    fn is_source_object(&self, position: usize) -> bool {
        is_source_declared_object_type(&self.nodes[position], self.kinds[position])
    }

    /// The unit node of the `dense` time source of the timed model whose root
    /// `model` `accepts` names, or `None` when no such root has a `dense`
    /// source. With `any` false the first accepted root decides, whatever its
    /// source; with `any` true the scan goes on to the next accepted root
    /// while the source is not `dense`. One validation visit is charged for
    /// each graph node scanned for the root.
    fn dense_unit(
        &self,
        meter: &mut WorkMeter,
        holder: usize,
        accepts: impl Fn(&CheckedNodeId) -> bool,
        any: bool,
    ) -> Result<Option<&'a CheckedNodeId>, ValidationFailure> {
        for position in 0..self.nodes.len() {
            visit(meter, holder, node_at(holder))?;
            if self.kinds.get(position) == Some(&CheckedNodeKind::TimedModel(TimedModelForm::Model))
                && bound(&self.terms[position], 0)
                    .and_then(target)
                    .is_some_and(&accepts)
            {
                let source = bound(&self.terms[position], 2)
                    .and_then(target)
                    .and_then(|source| self.position(source));
                let unit = source.and_then(|source| {
                    (bound(&self.terms[source], 0).and_then(text) == Some("dense"))
                        .then(|| bound(&self.terms[source], 1).and_then(target))
                        .flatten()
                });
                if unit.is_some() || !any {
                    return Ok(unit);
                }
            }
        }
        Ok(None)
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
        operation: &Operation<'_>,
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
        let operation = match operation {
            Operation::Selected { operation, .. } => *operation,
            Operation::Source { position, name, .. } => {
                return self.source_reward(
                    meter,
                    holder,
                    body,
                    (*position, name),
                    (function, value_type),
                );
            }
        };
        let owned = matches!(
            node.owner.as_ref(),
            Some(CheckedNodeOwner::Model { node: member, .. })
                if member.as_ref() == operation.identity.as_ref()
        );
        // A reward's expression is a `reward` clause function, never a
        // `precondition` or `body` one (FSpec FR-640 rule 4).
        if !owned
            || &node.semantic_type != value_type
            || !super::owner::is_reward_clause(&self.terms[function])
        {
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

    /// FR-640 Reader rule 4 for a reward whose `declaring` node (at
    /// `position`, the operation `name`) is source-declared: `function` is a
    /// function node that carries the declaring node's `SourceOwner`, the
    /// `declaration` `[<type>, <operation>, "reward", <name>]`, the clause
    /// kind `reward` and the reward's `value_type` as its own type, and the
    /// parameters it lists are read by the walk of its body. The parameter list is
    /// compared with the operation's at the identity-row join.
    fn source_reward(
        &self,
        meter: &mut WorkMeter,
        holder: usize,
        body: &CheckedTerm,
        (declaring, operation): (usize, &str),
        (function, value_type): (usize, &CheckedNodeId),
    ) -> Result<(), ValidationFailure> {
        let expression_at = binding_at(holder, 4).key("target");
        let node = &self.nodes[function];
        let name = bound(body, 2).and_then(text).unwrap_or_default();
        let declared = &self.nodes[declaring];
        let named = node
            .declaration
            .as_ref()
            .zip(declared.declaration.as_ref())
            .is_some_and(|(found, declaring)| {
                let (type_name, rest) = found
                    .qualified_name
                    .split_at_checked(declaring.qualified_name.len())
                    .unwrap_or_default();
                type_name == declaring.qualified_name.as_slice()
                    && matches!(rest, [op, kind, reward]
                        if op.as_ref() == operation
                            && kind.as_ref() == "reward"
                            && reward.as_ref() == name)
            });
        if node.owner != declared.owner
            || !named
            || &node.semantic_type != value_type
            || !super::owner::is_reward_clause(&self.terms[function])
        {
            return Err(invalid(meter, holder, expression_at));
        }
        let entries = bound(&self.terms[function], 0)
            .map(members)
            .unwrap_or_default();
        let mut listed: Vec<usize> = stage_storage::vec(
            meter,
            &Site::new(PackageReadStage::Graph, holder, FORM, expression_at),
            entries.len(),
        )?;
        for entry in entries {
            let position = target(entry)
                .and_then(|id| self.position(id))
                .filter(|&position| {
                    self.kinds[position] == CheckedNodeKind::Value(ValueForm::Parameter)
                });
            let Some(position) = position else {
                return Err(invalid(meter, holder, expression_at));
            };
            listed.push(position);
        }
        if self.reads_beyond_parameters(meter, holder, function, &listed)? {
            return Err(invalid(meter, holder, expression_at));
        }
        Ok(())
    }

    /// FR-640 Reader rule 2 for a random parameter of a selected operation:
    /// the parameter is one of the operation's and each support value is a
    /// member of its type. One validation visit is charged for each value.
    fn random_parameter(
        &self,
        meter: &mut WorkMeter,
        position: usize,
        body: &CheckedTerm,
        operation: &OperationDecl,
        package: &DomainModel,
    ) -> Result<(), ValidationFailure> {
        let join = self;
        let name = bound(body, 2).and_then(text).unwrap_or_default();
        let parameter_at = binding_at(position, 2).key("value");
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
            visit(meter, position, value_at)?;
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
            let verdict = match bound(entry, 0) {
                Some(value) => not_member(meter, join.models, package, declared, value)?,
                None => None,
            };
            match verdict {
                Some(NotMember::Literal) => {
                    return Err(invalid(meter, position, value_at));
                }
                Some(NotMember::Spelling) => {
                    return Err(invalid(meter, position, value_at.key("value")));
                }
                None => {}
            }
        }
        Ok(())
    }

    /// FR-640 Reader rule 5: the delay's model is `time dense`, and each time
    /// quantity of its parameters is in the unit of that source (FR-640
    /// "Time quantities are in the timed model's time unit"). One validation
    /// visit is charged for each parameter member inspected.
    fn delay(
        &self,
        meter: &mut WorkMeter,
        holder: usize,
        operation: &Operation<'_>,
        body: &CheckedTerm,
    ) -> Result<(), ValidationFailure> {
        let unit = match operation {
            Operation::Selected { model, .. } => {
                self.dense_unit(meter, holder, |root| root == *model, false)?
            }
            // One source time member times every source object type of the
            // unit (FSpec FR-640 Reader rule 5, FR-415): any `dense` root
            // whose model is a source-declared node of the same `SourceOwner`.
            Operation::Source { position, .. } => {
                let owner = self.nodes[*position].owner.as_ref();
                self.dense_unit(
                    meter,
                    holder,
                    |root| {
                        self.position(root).is_some_and(|root| {
                            self.is_source_object(root) && self.nodes[root].owner.as_ref() == owner
                        })
                    },
                    true,
                )?
            }
        };
        let Some(unit) = unit else {
            return Err(invalid(meter, holder, binding_at(holder, 0).key("target")));
        };
        let parameters_at = binding_at(holder, 3);
        let in_unit = |term: Option<&CheckedTerm>| match term {
            Some(CheckedTerm::Literal {
                value_kind: LiteralKind::Quantity,
                type_id,
                ..
            }) => type_id == unit,
            _ => true,
        };
        for (index, member) in bound(body, 3)
            .map(members)
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            let member_at = parameters_at.key("members").index(index);
            let (quantity, quantity_at) = match member {
                CheckedTerm::Binding { value, .. } => {
                    (Some(value.as_ref()), member_at.key("value"))
                }
                row => (
                    bound(row, 0),
                    member_at.key("members").index(0).key("value"),
                ),
            };
            visit(meter, holder, quantity_at)?;
            if !in_unit(quantity) {
                return Err(invalid(meter, holder, quantity_at));
            }
        }
        Ok(())
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
        if let Some(model) = bound(body, 0)
            .and_then(target)
            .and_then(|id| self.position(id))
            .filter(|&position| self.is_source_object(position))
        {
            return self.source_workload(meter, holder, body, model);
        }
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
        let mut named: Vec<(_, &str)> = stage_storage::vec(
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
            // A row is one operation of one declaring node (FR-640 Reader rule
            // 3): strict ascent of (declaring, name) refuses a repeat, while
            // two inherited operations of one name stay distinct.
            if previous.is_some_and(|earlier| earlier >= (declaring, name)) {
                return Err(invalid(meter, holder, row_at));
            }
            previous = Some((declaring, name));
            named.push((declared.node, name));
        }
        for node in &lineage {
            let Some(declared) = owner.package.object_types.get(*node) else {
                continue;
            };
            for operation in &declared.operations {
                if !named.contains(&(*node, member_name(&operation.identity))) {
                    return Err(invalid(meter, holder, weights_at.key("members")));
                }
            }
        }
        Ok(())
    }
}

impl Join<'_> {
    /// FR-640 Reader rule 3 for a workload whose `model` (at `model`) is
    /// source-declared. The identity-row join resolves each row's operation
    /// and the coverage of the model's own and inherited operations; the strict
    /// read checks the order of the weights, that no (declaring, operation)
    /// pair repeats and that each row's `declaring` node is a source-declared node of the
    /// model's `SourceOwner`. One validation visit is charged for each row.
    fn source_workload(
        &self,
        meter: &mut WorkMeter,
        holder: usize,
        body: &CheckedTerm,
        model: usize,
    ) -> Result<(), ValidationFailure> {
        let weights_at = binding_at(holder, 2);
        let rows = bound(body, 2).map(members).unwrap_or_default();
        let mut previous: Option<(&CheckedNodeId, &str)> = None;
        for (index, row) in rows.iter().enumerate() {
            let row_at = weights_at.key("members").index(index);
            visit(meter, holder, row_at)?;
            let (Some(declaring), Some(name)) =
                (bound(row, 0).and_then(target), bound(row, 1).and_then(text))
            else {
                return Err(invalid(meter, holder, row_at));
            };
            let declaring_at = row_at.key("members").index(0).key("value").key("target");
            let declared = self.position(declaring).is_some_and(|position| {
                self.is_source_object(position)
                    && self.nodes[position].owner == self.nodes[model].owner
            });
            if !declared {
                return Err(invalid(meter, holder, declaring_at));
            }
            if previous.is_some_and(|earlier| earlier >= (declaring, name)) {
                return Err(invalid(meter, holder, row_at));
            }
            previous = Some((declaring, name));
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
    let claims = kinds
        .iter()
        .filter(|kind| **kind == CheckedNodeKind::Claim(super::ClaimForm::ProbabilisticClaim))
        .count();
    if count == 0 && claims == 0 {
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
                let operation = join.operation(meter, position, body, 0, 1)?;
                let name = bound(body, 2).and_then(text).unwrap_or_default();
                match operation {
                    Operation::Selected {
                        model,
                        operation,
                        package,
                    } => {
                        join.random_parameter(meter, position, body, operation, package)?;
                        (0, Some(model), member_name(&operation.identity), name)
                    }
                    // The parameter and its support are checked against the
                    // operation entry's `parameters` at the identity-row join.
                    Operation::Source {
                        model, name: op, ..
                    } => (0, Some(model), op, name),
                }
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
                let operation = join.operation(meter, position, body, 0, 1)?;
                join.reward(meter, position, body, &operation)?;
                (
                    2,
                    Some(operation.model()),
                    operation.name(),
                    bound(body, 2).and_then(text).unwrap_or_default(),
                )
            }
            ProbabilisticForm::DelayDistribution => {
                let operation = join.operation(meter, position, body, 0, 1)?;
                join.delay(meter, position, &operation, body)?;
                (3, Some(operation.model()), operation.name(), "")
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
    claims::validate(&join, meter)
}
