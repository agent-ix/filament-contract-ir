//! FR-059 and FR-060: state-graph claims and model terminal declarations.
//!
//! These are separate body steps. Shape and form fit are checked before target
//! meaning. Several terminal nodes on one model node are admitted; rows are
//! ordered by model node key, then terminal node key (FR-060-AC-8).

use super::frame::StepGraph;
use super::graph_storage::GraphIndex;
use super::model_members::DeclarationForm;
use super::native_join_budget::{NativeJoinBudget, PackageReadStage};
use super::structural::{reference_type_target_ref, typed_parameter_binding};
use super::{
    CheckedNodeId, CheckedNodeKind, CheckedNodeOwner, CheckedPackageV2,
    CheckedSemanticNodeV2 as AdmittedSemanticNodeV2, CheckedTerm, ClaimForm, CompositeTypeForm,
    LiteralKind, ModelForm, RawSemanticNodeV2 as CheckedSemanticNodeV2, ScalarTypeForm,
    TemporalForm, ValueForm, WorkMeter,
};
use crate::checked_package::common::{node_pointer, ValidationFailure};
use crate::checked_package::shared::{
    CheckedOccurrenceRole, CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
    JsonPointer,
};
use std::sync::Arc;

/// The admitted terminal declaration on one model node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TerminalDeclaration {
    /// Every terminal state is intended.
    Any,
    /// A terminal state is intended when `condition` holds.
    When {
        /// The level-0 `self` parameter node, present exactly when the
        /// condition reads `self`.
        parameter: Option<CheckedNodeId>,
        /// The Boolean `expression` or `value`/`literal` condition node.
        condition: CheckedNodeId,
    },
}

/// One admitted model terminal declaration from a checked package.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalRow {
    /// The `temporal`/`terminal` node carrying the declaration.
    pub node: CheckedNodeId,
    /// The model node the declaration belongs to.
    pub model: CheckedNodeId,
    /// The zero-based position of this row among the rows of `model`, in
    /// terminal node key order; (`model`, `ordinal`) is the terminal key.
    pub ordinal: u32,
    /// The declaration's mode and optional condition.
    pub declaration: TerminalDeclaration,
}

fn failure(
    code: Code,
    cause: Option<Cause>,
    path: JsonPointer,
    locus: &CheckedNodeId,
) -> ValidationFailure {
    ValidationFailure::refused_at(code, path, cause, locus.clone())
}

fn body_path(position: usize) -> JsonPointer {
    node_pointer(position).key("body")
}

fn binding_path(position: usize, slot: usize) -> JsonPointer {
    body_path(position).key("members").index(slot)
}

fn target_path(position: usize, slot: usize) -> JsonPointer {
    binding_path(position, slot).key("value").key("target")
}

fn aggregate(term: &CheckedTerm) -> Option<&[CheckedTerm]> {
    match term {
        CheckedTerm::Aggregate { members } => Some(members),
        _ => None,
    }
}

fn binding_value<'a>(member: &'a CheckedTerm, expected: &str) -> Option<&'a CheckedTerm> {
    match member {
        CheckedTerm::Binding { name, value, .. } if name.as_ref() == expected => Some(value),
        _ => None,
    }
}

fn bindings<'a>(term: &'a CheckedTerm, names: [&str; 4]) -> Option<[&'a CheckedTerm; 4]> {
    let [first, second, third, fourth] = aggregate(term)? else {
        return None;
    };
    Some([
        binding_value(first, names[0])?,
        binding_value(second, names[1])?,
        binding_value(third, names[2])?,
        binding_value(fourth, names[3])?,
    ])
}

fn reference(term: &CheckedTerm) -> Option<&CheckedNodeId> {
    match term {
        CheckedTerm::Reference { target } => Some(target),
        _ => None,
    }
}

fn empty(term: &CheckedTerm) -> bool {
    aggregate(term).is_some_and(<[CheckedTerm]>::is_empty)
}

fn word<'a>(term: &'a CheckedTerm, graph: &StepGraph<'_, '_>) -> Option<&'a str> {
    let CheckedTerm::Literal {
        type_id,
        value_kind: LiteralKind::Text,
        value,
    } = term
    else {
        return None;
    };
    graph
        .node(type_id)
        .filter(|(_, kind)| *kind == CheckedNodeKind::ScalarType(ScalarTypeForm::Text))?;
    value.as_str()
}

fn boolean_type(node: &CheckedSemanticNodeV2, graph: &StepGraph<'_, '_>) -> bool {
    graph
        .node(&node.semantic_type)
        .is_some_and(|(_, kind)| kind == CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean))
}

/// The object type node a terminal's `self` is bound to, `None` when the
/// binding names no single object.
///
/// A `model_root` binding (FR-476 "Several terminal nodes on one model node")
/// takes the object from the selection's one population object; an
/// `object_type` binding names its own object, which must appear in exactly
/// one population.
fn self_object_key(
    graph: &StepGraph<'_, '_>,
    model: &CheckedNodeId,
    meter: &mut WorkMeter,
    position: usize,
) -> Result<Option<CheckedNodeId>, ValidationFailure> {
    let Some((model_node, kind)) = graph.node(model) else {
        return Ok(None);
    };
    match (kind, &model_node.owner) {
        (
            CheckedNodeKind::Model(ModelForm::ModelRoot),
            Some(CheckedNodeOwner::ModelRoot { identity }),
        ) => {
            let Some(package) = graph.owners.model(identity) else {
                return Ok(None);
            };
            let mut only: Option<&str> = None;
            let mut count = 0;
            for members in package.population_members.values() {
                meter.charge(1, || binding_path(position, 2))?;
                for object in members {
                    meter.charge(1, || binding_path(position, 2))?;
                    count += 1;
                    only = Some(object.as_ref());
                }
            }
            let Some(object) = only.filter(|_| count == 1) else {
                return Ok(None);
            };
            if !package.object_types.contains_key(object) {
                return Ok(None);
            }
            Ok(graph
                .owners
                .declaration_id(identity, object, DeclarationForm::ObjectType)
                .cloned())
        }
        (
            CheckedNodeKind::Model(ModelForm::ObjectType),
            Some(CheckedNodeOwner::Model {
                identity,
                node: declaration,
            }),
        ) => {
            let Ok(owner) = graph.owners.recover(model_node) else {
                return Ok(None);
            };
            if owner.package.identity.as_ref() != identity.as_ref()
                || !owner
                    .package
                    .object_types
                    .contains_key(declaration.as_ref())
            {
                return Ok(None);
            }
            // Count this model's declaration in every population. Other
            // selected models in the same package do not make this model's
            // `self` ambiguous.
            let mut count = 0;
            for members in owner.package.population_members.values() {
                meter.charge(1, || binding_path(position, 2))?;
                for object in members {
                    meter.charge(1, || binding_path(position, 2))?;
                    if object.as_ref() == declaration.as_ref() {
                        count += 1;
                    }
                }
            }
            Ok((count == 1).then(|| model.clone()))
        }
        _ => Ok(None),
    }
}

fn self_reads<'g>(
    condition: &'g CheckedNodeId,
    graph: &'g StepGraph<'_, '_>,
    terms: &[Option<CheckedTerm>],
    meter: &mut WorkMeter,
    position: usize,
) -> Result<Vec<&'g CheckedNodeId>, ValidationFailure> {
    // Wire IDs are flat, so ordering them never allocates; the tables are
    // charged sorted slices of borrowed IDs.
    let mut seen: Vec<&CheckedNodeId> = Vec::new();
    let mut pending: Vec<&CheckedNodeId> = Vec::new();
    meter.push(&mut pending, condition)?;
    let mut reads: Vec<&CheckedNodeId> = Vec::new();
    while let Some(key) = pending.pop() {
        if !meter.insert_sorted(&mut seen, key)? {
            continue;
        }
        meter.charge(1, || binding_path(position, 3))?;
        let Some((node, kind)) = graph.node(key) else {
            continue;
        };
        if kind == CheckedNodeKind::Value(ValueForm::Parameter) {
            let declaration = graph
                .index
                .get(key)
                .and_then(|&at| terms.get(at))
                .and_then(Option::as_ref)
                .and_then(|body| {
                    typed_parameter_binding(body, &|id| graph.node(id).map(|(_, kind)| kind))
                });
            if declaration == Some(("self", "0")) {
                meter.insert_sorted(&mut reads, key)?;
            }
        } else {
            meter.room(&mut pending, node.dependencies.len())?;
            pending.extend(node.dependencies.iter());
        }
    }
    Ok(reads)
}

fn claim_occurrences(
    node: &CheckedSemanticNodeV2,
    position: usize,
) -> Result<(), ValidationFailure> {
    for (ordinal, occurrence) in node.occurrences.iter().enumerate() {
        if occurrence.role != CheckedOccurrenceRole::Claim {
            return Err(failure(
                Code::InvalidSemanticGraph,
                None,
                node_pointer(position)
                    .key("occurrences")
                    .index(ordinal)
                    .key("role"),
                &node.node_id,
            ));
        }
    }
    Ok(())
}

fn graph_nodes<'a, 'm>(
    nodes: &'a [CheckedSemanticNodeV2],
    kinds: &'a [CheckedNodeKind],
    index: &'a GraphIndex<'a>,
    owners: &'a super::model_members::ModelOwners<'m>,
) -> StepGraph<'a, 'm> {
    StepGraph {
        nodes,
        kinds,
        index,
        owners,
    }
}

/// FR-475's state-graph body step, before the temporal step.
pub(super) fn validate_state_graph(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &GraphIndex<'_>,
    owners: &super::model_members::ModelOwners<'_>,
) -> Result<(), ValidationFailure> {
    let graph = graph_nodes(nodes, kinds, index, owners);
    for (position, node) in graph.each(CheckedNodeKind::Claim(ClaimForm::StateGraph)) {
        let [kind, over, from, target] = terms
            .get(position)
            .and_then(Option::as_ref)
            .and_then(|term| bindings(term, ["kind", "over", "from", "target"]))
            .ok_or_else(|| {
                failure(
                    Code::InvalidSemanticGraph,
                    None,
                    body_path(position),
                    &node.node_id,
                )
            })?;
        let form = word(kind, &graph);
        if !matches!(form, Some("possible" | "always-possible" | "unique-path")) {
            return Err(failure(
                Code::InvalidPackage,
                Some(Cause::InvalidValue),
                binding_path(position, 0),
                &node.node_id,
            ));
        }
        if !(empty(over) || reference(over).is_some()) {
            return Err(failure(
                Code::InvalidSemanticGraph,
                None,
                binding_path(position, 1),
                &node.node_id,
            ));
        }
        if !(if form == Some("possible") {
            empty(from)
        } else {
            reference(from).is_some()
        }) {
            return Err(failure(
                Code::InvalidSemanticGraph,
                None,
                binding_path(position, 2),
                &node.node_id,
            ));
        }
        let Some(target) = reference(target) else {
            return Err(failure(
                Code::InvalidSemanticGraph,
                None,
                binding_path(position, 3),
                &node.node_id,
            ));
        };
        if let Some(over) = reference(over) {
            let valid = graph
                .node(over)
                .is_some_and(|(_, kind)| kind == CheckedNodeKind::Value(ValueForm::Parameter));
            if !valid {
                return Err(failure(
                    Code::InvalidModelBinding,
                    Some(Cause::MalformedDeclaration),
                    target_path(position, 1),
                    over,
                ));
            }
        }
        for (slot, predicate) in [(2, reference(from)), (3, Some(target))] {
            let Some(predicate) = predicate else { continue };
            let valid = graph.node(predicate).is_some_and(|(target_node, kind)| {
                matches!(
                    kind,
                    CheckedNodeKind::Expression(_) | CheckedNodeKind::Value(ValueForm::Literal)
                ) && boolean_type(target_node, &graph)
            });
            if !valid {
                return Err(failure(
                    Code::IllTyped,
                    Some(Cause::OperatorIneligible),
                    target_path(position, slot),
                    predicate,
                ));
            }
        }
        if !boolean_type(node, &graph) || node.owner.is_some() {
            return Err(failure(
                Code::InvalidSemanticGraph,
                None,
                node_pointer(position),
                &node.node_id,
            ));
        }
        claim_occurrences(node, position)?;
    }
    Ok(())
}

/// FR-476's terminal body step, after abstraction and before later body steps.
pub(super) fn validate_terminal(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &GraphIndex<'_>,
    owners: &super::model_members::ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let graph = graph_nodes(nodes, kinds, index, owners);
    for (position, node) in graph.each(CheckedNodeKind::Temporal(TemporalForm::Terminal)) {
        let [model, mode, parameters, condition] = terms
            .get(position)
            .and_then(Option::as_ref)
            .and_then(|term| bindings(term, ["model", "mode", "parameters", "condition"]))
            .ok_or_else(|| {
                failure(
                    Code::InvalidSemanticGraph,
                    None,
                    body_path(position),
                    &node.node_id,
                )
            })?;
        let Some(model) = reference(model) else {
            return Err(failure(
                Code::InvalidSemanticGraph,
                None,
                binding_path(position, 0),
                &node.node_id,
            ));
        };
        let mode = word(mode, &graph);
        if !matches!(mode, Some("when" | "any")) {
            return Err(failure(
                Code::InvalidPackage,
                Some(Cause::InvalidValue),
                binding_path(position, 1),
                &node.node_id,
            ));
        }
        let parameters = aggregate(parameters).ok_or_else(|| {
            failure(
                Code::InvalidSemanticGraph,
                None,
                binding_path(position, 2),
                &node.node_id,
            )
        })?;
        if parameters.len() > 1 || (mode == Some("any") && !parameters.is_empty()) {
            return Err(failure(
                Code::InvalidSemanticGraph,
                None,
                binding_path(position, 2),
                &node.node_id,
            ));
        }
        let parameter = parameters.first().and_then(reference);
        if !parameters.is_empty() && parameter.is_none() {
            return Err(failure(
                Code::InvalidSemanticGraph,
                None,
                binding_path(position, 2),
                &node.node_id,
            ));
        }
        if !(if mode == Some("when") {
            reference(condition).is_some()
        } else {
            empty(condition)
        }) {
            return Err(failure(
                Code::InvalidSemanticGraph,
                None,
                binding_path(position, 3),
                &node.node_id,
            ));
        }
        let model_kind = graph.node(model).map(|(_, kind)| kind);
        if !matches!(model_kind, Some(CheckedNodeKind::Model(_))) {
            return Err(failure(
                Code::InvalidModelBinding,
                Some(Cause::MalformedDeclaration),
                target_path(position, 0),
                model,
            ));
        }
        if let Some(parameter) = parameter {
            let candidate = graph.node(parameter).and_then(|(target_node, kind)| {
                (kind == CheckedNodeKind::Value(ValueForm::Parameter)).then_some(target_node)
            });
            let target_position = graph.index.get(parameter).copied();
            let name_level = target_position
                .and_then(|at| terms.get(at))
                .and_then(Option::as_ref)
                .and_then(|term| {
                    typed_parameter_binding(term, &|id| graph.node(id).map(|(_, kind)| kind))
                });
            let reference_target = candidate
                .and_then(|target_node| graph.node(&target_node.semantic_type))
                .filter(|(_, kind)| {
                    *kind == CheckedNodeKind::CompositeType(CompositeTypeForm::Reference)
                })
                .and_then(|(type_node, _)| {
                    graph
                        .index
                        .get(&type_node.node_id)
                        .and_then(|at| terms.get(*at))
                        .and_then(Option::as_ref)
                })
                .and_then(reference_type_target_ref)
                .cloned();
            let expected = if name_level == Some(("self", "0")) && reference_target.is_some() {
                self_object_key(&graph, model, meter, position)?
            } else {
                None
            };
            let valid = if name_level == Some(("self", "0")) && reference_target.is_some() {
                reference_target
                    .as_ref()
                    .is_some_and(|target| expected.as_ref() == Some(target))
            } else {
                false
            };
            if !valid {
                return Err(failure(
                    Code::IllTyped,
                    Some(Cause::OperatorIneligible),
                    binding_path(position, 2)
                        .key("value")
                        .key("members")
                        .index(0)
                        .key("target"),
                    parameter,
                ));
            }
        }
        if let Some(condition) = reference(condition) {
            let Some((target_node, kind)) = graph.node(condition) else {
                return Err(failure(
                    Code::IllTyped,
                    Some(Cause::OperatorIneligible),
                    target_path(position, 3),
                    condition,
                ));
            };
            if !matches!(
                kind,
                CheckedNodeKind::Expression(_) | CheckedNodeKind::Value(ValueForm::Literal)
            ) {
                return Err(failure(
                    Code::IllTyped,
                    Some(Cause::OperatorIneligible),
                    target_path(position, 3),
                    condition,
                ));
            }
            if !boolean_type(target_node, &graph) {
                return Err(failure(
                    Code::IllTyped,
                    Some(Cause::NonBooleanRoot),
                    target_path(position, 3),
                    condition,
                ));
            }
            let reads = self_reads(condition, &graph, terms, meter, position)?;
            if reads.len() != usize::from(parameter.is_some())
                || parameter.is_some_and(|parameter| reads.binary_search(&parameter).is_err())
            {
                return Err(failure(
                    Code::InvalidSemanticGraph,
                    None,
                    binding_path(position, 2),
                    &node.node_id,
                ));
            }
        }
        if !boolean_type(node, &graph) {
            return Err(failure(
                Code::InvalidSemanticGraph,
                None,
                node_pointer(position).key("semantic_type"),
                &node.node_id,
            ));
        }
        claim_occurrences(node, position)?;
    }
    Ok(())
}

impl CheckedPackageV2 {
    /// The admitted model terminal declarations ordered by the model node's
    /// structural key, then the terminal node's. A model without a terminal
    /// node has no row; a model with several has one row each.
    pub fn terminal_rows(&self) -> &[TerminalRow] {
        self.terminal_rows.as_slice()
    }
}

/// Retain the public terminal rows once, after all terminal nodes have passed
/// FR-060 admission. The body shape and references are therefore invariants
/// here; every returned ID is rebuilt against the admitted graph's structural
/// keys before the rows are sorted and stored.
pub(super) fn retain_rows(
    nodes: &[AdmittedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    structural_keys: &[Arc<CheckedNodeId>],
    budget: &mut NativeJoinBudget,
) -> Result<Arc<Vec<TerminalRow>>, ValidationFailure> {
    let site = super::read_charge::Site::new(PackageReadStage::Retained, 0, "terminal_rows");
    let terminal_kind = CheckedNodeKind::Temporal(TemporalForm::Terminal);
    let count = kinds.iter().filter(|kind| **kind == terminal_kind).count();
    let mut rows = super::read_charge::with_capacity(budget, site, count)?;
    let mut meter = WorkMeter::over(budget);
    for (position, (node, kind)) in nodes.iter().zip(kinds).enumerate() {
        if *kind != terminal_kind {
            continue;
        }
        let [model, mode, parameters, condition] =
            bindings(&node.body, ["model", "mode", "parameters", "condition"])
                .expect("admitted terminal body has four ordered bindings");
        let CheckedTerm::Literal {
            value_kind: LiteralKind::Text,
            value: mode,
            ..
        } = mode
        else {
            unreachable!("admitted terminal mode is text")
        };
        let node_key = structural_keys
            .get(position)
            .expect("admitted terminal has a structural key");
        super::read_charge::charge_node_id_clone(&mut meter, site, node_key)?;
        let node_key = node_key.as_ref().clone();
        let model = resolve_target(
            reference(model).expect("admitted terminal model is a reference"),
            structural_keys,
            &mut meter,
        )?;
        let declaration = match mode.as_str() {
            Some("any") => TerminalDeclaration::Any,
            Some("when") => {
                let parameters =
                    aggregate(parameters).expect("admitted terminal parameters are an aggregate");
                let parameter = parameters
                    .first()
                    .map(|parameter| {
                        resolve_target(
                            reference(parameter).expect("admitted self parameter is a reference"),
                            structural_keys,
                            &mut meter,
                        )
                    })
                    .transpose()?;
                let condition = resolve_target(
                    reference(condition).expect("admitted terminal condition is a reference"),
                    structural_keys,
                    &mut meter,
                )?;
                TerminalDeclaration::When {
                    parameter,
                    condition,
                }
            }
            _ => unreachable!("admitted terminal mode is when or any"),
        };
        rows.push(TerminalRow {
            node: node_key,
            model,
            ordinal: 0,
            declaration,
        });
    }
    rows.sort_unstable_by(|left, right| {
        left.model
            .cmp(&right.model)
            .then_with(|| left.node.cmp(&right.node))
    });
    let mut ordinal = 0_u32;
    for at in 0..rows.len() {
        ordinal = if at > 0 && rows[at - 1].model == rows[at].model {
            ordinal
                .checked_add(1)
                .expect("terminal row count is bounded by the node limit")
        } else {
            0
        };
        rows[at].ordinal = ordinal;
    }
    super::read_charge::new_arc(budget, site, rows)
}

fn resolve_target(
    target: &CheckedNodeId,
    structural_keys: &[Arc<CheckedNodeId>],
    meter: &mut WorkMeter<'_>,
) -> Result<CheckedNodeId, ValidationFailure> {
    // The reader has already checked each target's presence and semantic
    // family; resolution now replaces graph-local indices with shared keys.
    Ok(
        super::node_index::resolve_id_charged(target, structural_keys, meter)?
            .expect("admitted terminal target resolves in the checked graph"),
    )
}
