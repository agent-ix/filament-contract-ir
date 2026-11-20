// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Join stage 4: the existence of the `semantic_type` and `dependencies`
//! edges of a node (FSpec FR-322 "One total order of the join stages",
//! FR-322-AC-275; Contract IR FR-038).
//!
//! A node of tag `scalar_type`, `composite_type`, `bounded_domain`, `value`,
//! `expression`, `function` or `model` whose `semantic_type` or `dependencies`
//! entry names no node refuses `invalid_semantic_graph`, with no cause, at the
//! edge member. The stage runs over every node, in graph order and the
//! `semantic_type` before the `dependencies` of each, after the owner join and
//! before the model declaration occurrence check. A node of any other tag is
//! outside the stage: its own artifact defines the refusal of a dangling edge.

use super::native_join_budget::PackageReadStage;
use super::stage_storage::{self, Path, Refusal, Site};
use super::{
    CheckedNodeKind, CheckedNodeTag, GraphIndex, RawSemanticNodeV2, ScalarTypeForm, WorkMeter,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::CheckedPackageRefusalCode;

/// Whether stage 4 reads the edges of a node of `kind`.
fn in_stage(kind: CheckedNodeKind) -> bool {
    matches!(
        kind.tag(),
        CheckedNodeTag::ScalarType
            | CheckedNodeTag::CompositeType
            | CheckedNodeTag::BoundedDomain
            | CheckedNodeTag::Value
            | CheckedNodeTag::Expression
            | CheckedNodeTag::Function
            | CheckedNodeTag::Model
    )
}

/// Whether the `dependencies` of a node of `kind` are read by the nominal
/// join, not here: a dimension, unit or dimension equivalence may depend on a
/// nominal node of a selected dependency package, and its absent operand is
/// refused at the operand of its body.
fn nominal_dependencies(kind: CheckedNodeKind) -> bool {
    matches!(
        kind,
        CheckedNodeKind::ScalarType(
            ScalarTypeForm::Dimension | ScalarTypeForm::Unit | ScalarTypeForm::DimensionEquivalence
        )
    )
}

/// Refuses the first edge of a stage 4 node that names no node.
pub(super) fn validate(
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &GraphIndex<'_>,
    meter: &mut WorkMeter<'_>,
) -> Result<(), ValidationFailure> {
    let root = Path::root().key("semantic_graph").key("nodes");
    let site = |position: usize, at: Path<'static>| {
        Site::new(PackageReadStage::Graph, position, "edge_existence", at)
    };
    let code = CheckedPackageRefusalCode::InvalidSemanticGraph;
    for (position, node) in nodes.iter().enumerate() {
        let Some(&kind) = kinds.get(position).filter(|kind| in_stage(**kind)) else {
            continue;
        };
        let at = root.index(position);
        // One unit for the node and one for each `dependencies` entry the
        // stage reads: an exempt node's entries are left to the nominal join.
        let exempt = nominal_dependencies(kind);
        let read = if exempt { 0 } else { node.dependencies.len() };
        let work = u64::try_from(read).map_or(u64::MAX, |read| read.saturating_add(1));
        stage_storage::work(meter, &site(position, at), work)?;
        if !index.contains_key(&node.semantic_type) {
            let edge = at.key("semantic_type");
            return Err(stage_storage::refuse(
                meter,
                &site(position, edge),
                Refusal::new(code, edge),
            ));
        }
        if exempt {
            continue;
        }
        for (edge_index, dependency) in node.dependencies.iter().enumerate() {
            if index.contains_key(dependency) {
                continue;
            }
            let edge = at.key("dependencies").index(edge_index);
            return Err(stage_storage::refuse(
                meter,
                &site(position, edge),
                Refusal::new(code, edge),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::native_join_budget::NativeJoinBudget;
    use super::super::test_support::id;
    use super::*;
    use crate::checked_package::shared::CheckedPackageRefusal;
    use ix_trace_rs::trace;
    use serde_json::{json, Value};

    use super::super::{
        ClaimForm, CompositeTypeForm, CorrespondenceForm, ExpressionForm, ExtensionForm,
        HybridForm, ProtocolForm, RelationForm, StateForm, TemporalForm, TimedModelForm, ValueForm,
    };

    /// A node of `kind` whose id is `digit`, typed by `semantic_type` and
    /// depending on `dependencies`, all named by their digit.
    fn node(
        digit: char,
        kind: CheckedNodeKind,
        semantic_type: char,
        dependencies: &[char],
    ) -> (RawSemanticNodeV2, CheckedNodeKind) {
        let raw = serde_json::from_value(json!({
            "node_id": id(digit), "schema_version": "filament.checked-semantic-graph/v2",
            "node_tag": kind.tag().as_wire(), "semantic_form": "x",
            "semantic_type": id(semantic_type),
            "dependencies": dependencies.iter().map(|d| id(*d)).collect::<Vec<Value>>(),
            "occurrences": [{"role": "generated", "ordinal": 0}],
            "body": {"term": "aggregate", "members": []}
        }))
        .expect("test node");
        (raw, kind)
    }

    fn run(
        graph: &[(RawSemanticNodeV2, CheckedNodeKind)],
        work: u64,
    ) -> (Result<(), ValidationFailure>, NativeJoinBudget) {
        let nodes: Vec<RawSemanticNodeV2> = graph.iter().map(|(node, _)| node.clone()).collect();
        let kinds: Vec<CheckedNodeKind> = graph.iter().map(|(_, kind)| *kind).collect();
        let index: GraphIndex<'_> = nodes
            .iter()
            .enumerate()
            .map(|(position, node)| (&node.node_id, position))
            .collect();
        let mut budget = NativeJoinBudget::new(work, 0).with_read_storage_limit(u64::MAX);
        let result = validate(&nodes, &kinds, &index, &mut WorkMeter::over(&mut budget));
        (result, budget)
    }

    fn refusal(result: Result<(), ValidationFailure>) -> CheckedPackageRefusal {
        match result {
            Err(ValidationFailure::Refused(refusal)) => *refusal,
            other => panic!("expected a refusal, got {other:?}"),
        }
    }

    fn at(refusal: &CheckedPackageRefusal) -> String {
        refusal.loci[0]
            .path
            .as_ref()
            .map(ToString::to_string)
            .expect("a locus")
    }

    fn integer() -> CheckedNodeKind {
        CheckedNodeKind::ScalarType(ScalarTypeForm::ALL[0])
    }

    /// The tags of the stage are charged one unit and one per dependency.
    #[trace("TC-048", "FR-038-AC-582")]
    #[test]
    fn a_node_of_the_stage_is_charged_one_plus_its_dependencies() {
        let graph = [
            node('a', integer(), 'a', &[]),
            node(
                'b',
                CheckedNodeKind::CompositeType(CompositeTypeForm::ALL[0]),
                'a',
                &['a'],
            ),
            node(
                'c',
                CheckedNodeKind::Expression(ExpressionForm::ALL[0]),
                'a',
                &['a', 'b'],
            ),
            node(
                'd',
                CheckedNodeKind::Value(ValueForm::ALL[0]),
                'a',
                &['a', 'b', 'c'],
            ),
        ];
        let (result, budget) = run(&graph, u64::MAX);
        assert!(result.is_ok(), "{result:?}");
        assert_eq!(budget.consumed().work, 1 + 2 + 3 + 4);
        // Exactly that work is enough; one unit less is denied.
        assert!(run(&graph, 10).0.is_ok());
        assert!(run(&graph, 9).0.is_err());
    }

    /// A node outside the stage is neither read nor charged, whatever its
    /// edges name.
    #[trace("TC-048", "FR-038-AC-582")]
    #[test]
    fn a_node_outside_the_stage_is_neither_read_nor_charged() {
        let kinds = [
            CheckedNodeKind::State(StateForm::ALL[0]),
            CheckedNodeKind::Temporal(TemporalForm::ALL[0]),
            CheckedNodeKind::Protocol(ProtocolForm::ALL[0]),
            CheckedNodeKind::Relation(RelationForm::ALL[0]),
            CheckedNodeKind::Claim(ClaimForm::ALL[0]),
            CheckedNodeKind::Correspondence(CorrespondenceForm::ALL[0]),
            CheckedNodeKind::Extension(ExtensionForm::ALL[0]),
            CheckedNodeKind::Hybrid(HybridForm::ALL[0]),
            CheckedNodeKind::TimedModel(TimedModelForm::ALL[0]),
        ];
        for kind in kinds {
            // `z` names no node, in both edges.
            let graph = [node('a', kind, 'z', &['z'])];
            let (result, budget) = run(&graph, u64::MAX);
            assert!(result.is_ok(), "{kind:?}: {result:?}");
            assert_eq!(budget.consumed().work, 0, "{kind:?}");
        }
    }

    /// Every tag the stage names refuses a dangling `semantic_type`.
    #[trace("TC-048", "FR-038-AC-582")]
    #[test]
    fn every_tag_of_the_stage_refuses_a_dangling_edge() {
        use super::super::{BoundedDomainForm, FunctionForm, ModelForm};
        let kinds = [
            integer(),
            CheckedNodeKind::CompositeType(CompositeTypeForm::ALL[0]),
            CheckedNodeKind::BoundedDomain(BoundedDomainForm::ALL[0]),
            CheckedNodeKind::Value(ValueForm::ALL[0]),
            CheckedNodeKind::Expression(ExpressionForm::ALL[0]),
            CheckedNodeKind::Function(FunctionForm::ALL[0]),
            CheckedNodeKind::Model(ModelForm::ALL[0]),
        ];
        for kind in kinds {
            let refused = refusal(run(&[node('a', kind, 'z', &[])], u64::MAX).0);
            assert_eq!(
                refused.code,
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "{kind:?} semantic_type"
            );
            assert_eq!(refused.cause, None, "{kind:?}");
            assert_eq!(at(&refused), "/semantic_graph/nodes/0/semantic_type");
            let refused = refusal(run(&[node('a', kind, 'a', &['z'])], u64::MAX).0);
            assert_eq!(
                refused.code,
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "{kind:?} dependency"
            );
            assert_eq!(at(&refused), "/semantic_graph/nodes/0/dependencies/0");
        }
    }

    /// Within a node the `semantic_type` is read before the `dependencies`
    /// and the dependencies in order; across nodes, in graph order.
    #[trace("TC-048", "FR-038-AC-582")]
    #[test]
    fn the_edges_are_read_in_node_then_edge_order() {
        // Both of node 0's edges dangle: the semantic type is first.
        let both = [node('a', integer(), 'y', &['a', 'z'])];
        assert_eq!(
            at(&refusal(run(&both, u64::MAX).0)),
            "/semantic_graph/nodes/0/semantic_type"
        );
        // A present first dependency is skipped, the second dangles.
        let second = [node('a', integer(), 'a', &['a', 'z', 'y'])];
        assert_eq!(
            at(&refusal(run(&second, u64::MAX).0)),
            "/semantic_graph/nodes/0/dependencies/1"
        );
        // A dangling edge of node 0 outranks one of node 1.
        let graph = [
            node('a', integer(), 'a', &['z']),
            node('b', integer(), 'y', &[]),
        ];
        assert_eq!(
            at(&refusal(run(&graph, u64::MAX).0)),
            "/semantic_graph/nodes/0/dependencies/0"
        );
        // The refusal of the first dangling edge is charged only the work up
        // to it: node 0 only.
        let (_, budget) = run(&graph, u64::MAX);
        assert_eq!(budget.consumed().work, 2);
    }

    /// A dimension, unit or dimension equivalence leaves its dependencies to
    /// the nominal join: they are neither read nor charged, but its
    /// `semantic_type` is read.
    #[trace("TC-048", "FR-038-AC-582")]
    #[test]
    fn a_nominal_scalar_leaves_its_dependencies_to_the_nominal_join() {
        for form in [
            ScalarTypeForm::Dimension,
            ScalarTypeForm::Unit,
            ScalarTypeForm::DimensionEquivalence,
        ] {
            let kind = CheckedNodeKind::ScalarType(form);
            let (result, budget) = run(&[node('a', kind, 'a', &['z', 'y'])], u64::MAX);
            assert!(result.is_ok(), "{form:?}: {result:?}");
            assert_eq!(budget.consumed().work, 1, "{form:?}");
            let refused = refusal(run(&[node('a', kind, 'z', &[])], u64::MAX).0);
            assert_eq!(at(&refused), "/semantic_graph/nodes/0/semantic_type");
        }
        // An ordinary scalar does not share the exemption.
        let refused = refusal(run(&[node('a', integer(), 'a', &['z'])], u64::MAX).0);
        assert_eq!(at(&refused), "/semantic_graph/nodes/0/dependencies/0");
    }
}
