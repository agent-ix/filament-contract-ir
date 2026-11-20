// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Join stage 5: the occurrence of every model declaration node in
//! `source_map` (FSpec FR-322 "Model declaration occurrences",
//! FR-322-AC-486; Contract IR FR-038).
//!
//! A model declaration node (a `model` or `relation` node carrying a
//! `ModelOwner`) has exactly one `source_map` entry: role `generated`, ordinal
//! 0, with exactly the regions of its identity's `model_root` entry.

use super::native_join_budget::PackageReadStage;
use super::stage_storage::{self, Path, Refusal, Site};
use super::{CheckedNodeKind, CheckedNodeOwner, GraphIndex, RawSemanticNodeV2, WorkMeter};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedOccurrenceRole, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    CheckedSourceMapEntry, CheckedSourceRegion,
};

/// A model root's identity, entry position and entry regions.
type Root<'a> = (&'a str, usize, &'a [CheckedSourceRegion]);

/// Whether the node at `position` is a model declaration node.
fn is_declaration(node: &RawSemanticNodeV2, kind: Option<&CheckedNodeKind>) -> bool {
    matches!(
        kind,
        Some(CheckedNodeKind::Model(_) | CheckedNodeKind::Relation(_))
    ) && matches!(node.owner, Some(CheckedNodeOwner::Model { .. }))
}

/// Refuses the first model declaration node whose `source_map` entry is not
/// the one occurrence the specification fixes, else the first with none.
pub(super) fn validate(
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    source_map: &[CheckedSourceMapEntry],
    index: &GraphIndex<'_>,
    meter: &mut WorkMeter<'_>,
) -> Result<(), ValidationFailure> {
    let root = Path::root().key("source_map");
    let site = |at: Path<'static>| Site::new(PackageReadStage::Graph, 0, "model_occurrence", at);
    let declarations = nodes
        .iter()
        .enumerate()
        .filter(|(position, node)| is_declaration(node, kinds.get(*position)))
        .count();
    if declarations == 0 {
        return Ok(());
    }
    let model_roots = kinds
        .iter()
        .filter(|kind| **kind == CheckedNodeKind::Model(super::ModelForm::ModelRoot))
        .count();
    // One flag per graph node: a model root whose entry is already collected,
    // then a model declaration node whose entry has been seen.
    let mut seen = meter.filled(nodes.len(), false)?;
    // The first `generated` ordinal-zero entry of each model root, by identity
    // and entry position. A repeated entry of a root fills no further slot.
    let mut roots: Vec<Root<'_>> = stage_storage::vec(meter, &site(root), model_roots)?;
    for (entry_index, entry) in source_map.iter().enumerate() {
        if entry.role != CheckedOccurrenceRole::Generated || entry.ordinal != 0 {
            continue;
        }
        let Some(&position) = index.get(&entry.node_id) else {
            continue;
        };
        if let (
            Some(CheckedNodeKind::Model(super::ModelForm::ModelRoot)),
            Some(CheckedNodeOwner::ModelRoot { identity }),
            false,
        ) = (
            kinds.get(position),
            nodes[position].owner.as_ref(),
            seen[position],
        ) {
            seen[position] = true;
            stage_storage::work(meter, &site(root), 1)?;
            roots.push((identity.as_ref(), entry_index, entry.regions.as_slice()));
        }
    }
    // Unstable sorts allocate nothing; the entry position makes the order total.
    roots.sort_unstable_by(|left, right| (left.0, left.1).cmp(&(right.0, right.1)));
    let invalid = CheckedPackageRefusalCause::InvalidValue;
    let refuse = |meter: &mut WorkMeter<'_>, at: Path<'_>, id| {
        stage_storage::refuse(
            meter,
            &Site::new(PackageReadStage::Graph, 0, "model_occurrence", at),
            Refusal::new(CheckedPackageRefusalCode::InvalidPackage, at)
                .because(invalid)
                .at_node(id),
        )
    };
    for (entry_index, entry) in source_map.iter().enumerate() {
        let Some(&position) = index.get(&entry.node_id) else {
            continue;
        };
        let node = &nodes[position];
        if !is_declaration(node, kinds.get(position)) {
            continue;
        }
        let Some(CheckedNodeOwner::Model { identity, .. }) = node.owner.as_ref() else {
            continue;
        };
        stage_storage::work(meter, &site(root.index(entry_index)), 1)?;
        let start = roots.partition_point(|(held, _, _)| *held < identity.as_ref());
        let selecting = roots
            .get(start)
            .filter(|(held, _, _)| *held == identity.as_ref())
            .map(|(_, _, regions)| *regions);
        let repeated = seen[position];
        seen[position] = true;
        if entry.role != CheckedOccurrenceRole::Generated
            || entry.ordinal != 0
            || repeated
            || selecting != Some(entry.regions.as_slice())
        {
            return Err(refuse(meter, root.index(entry_index), &entry.node_id));
        }
    }
    for (position, node) in nodes.iter().enumerate() {
        if !is_declaration(node, kinds.get(position)) {
            continue;
        }
        stage_storage::work(meter, &site(root), 1)?;
        if !seen[position] {
            return Err(refuse(meter, root, &node.node_id));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::native_join_budget::NativeJoinBudget;
    use super::super::{test_support::node_id, ModelForm};
    use super::*;
    use ix_trace_rs::trace;
    use serde_json::{json, Value};

    fn region(start: u64) -> Value {
        json!([{"source": {"authority": "agent-ix", "identity": "example"},
            "start": start, "end": start + 1}])
    }

    fn node(digit: char, form: &str, owner: Value) -> RawSemanticNodeV2 {
        let id = node_id(digit);
        serde_json::from_value(json!({
            "node_id": id, "schema_version": "filament.checked-semantic-graph/v2",
            "node_tag": "model", "semantic_form": form, "semantic_type": id,
            "dependencies": [], "occurrences": [{"role": "generated", "ordinal": 0}],
            "owner": owner, "body": {"term": "aggregate", "members": []}
        }))
        .expect("test node")
    }

    fn entry(digit: char, start: u64) -> CheckedSourceMapEntry {
        serde_json::from_value(json!({
            "node_id": node_id(digit), "role": "generated", "ordinal": 0,
            "regions": region(start)
        }))
        .expect("test entry")
    }

    /// Two selected models: roots `a` (model `m`) and `b` (model `n`), then one
    /// declaration of each, `c` and `d`.
    fn two_models() -> (Vec<RawSemanticNodeV2>, Vec<CheckedNodeKind>) {
        let nodes = vec![
            node(
                'a',
                "model_root",
                json!({"kind": "model_root", "identity": "m"}),
            ),
            node(
                'b',
                "model_root",
                json!({"kind": "model_root", "identity": "n"}),
            ),
            node(
                'c',
                "object_type",
                json!({"kind": "model", "identity": "m", "node": "ix://m/C"}),
            ),
            node(
                'd',
                "object_type",
                json!({"kind": "model", "identity": "n", "node": "ix://n/D"}),
            ),
        ];
        let kinds = vec![
            CheckedNodeKind::Model(ModelForm::ModelRoot),
            CheckedNodeKind::Model(ModelForm::ModelRoot),
            CheckedNodeKind::Model(ModelForm::ObjectType),
            CheckedNodeKind::Model(ModelForm::ObjectType),
        ];
        (nodes, kinds)
    }

    fn run(
        nodes: &[RawSemanticNodeV2],
        kinds: &[CheckedNodeKind],
        entries: &[CheckedSourceMapEntry],
        work: u64,
    ) -> (Result<(), ValidationFailure>, NativeJoinBudget) {
        let index: GraphIndex<'_> = nodes
            .iter()
            .enumerate()
            .map(|(position, node)| (&node.node_id, position))
            .collect();
        let mut budget = NativeJoinBudget::new(work, 0).with_read_storage_limit(u64::MAX);
        let result = validate(
            nodes,
            kinds,
            entries,
            &index,
            &mut WorkMeter::over(&mut budget),
        );
        (result, budget)
    }

    /// A repeated entry of the first root fills no slot of the root table, so
    /// the second model's root still selects its declarations' regions.
    #[trace("TC-048", "FR-038-AC-582")]
    #[test]
    fn a_repeated_root_entry_does_not_hide_a_later_models_root() {
        let (nodes, kinds) = two_models();
        let entries = [
            entry('a', 0),
            entry('a', 0),
            entry('b', 5),
            entry('c', 0),
            entry('d', 5),
        ];
        let (result, budget) = run(&nodes, &kinds, &entries, u64::MAX);
        assert!(result.is_ok(), "{result:?}");
        // The repeat is neither charged nor tabled: 2 root entries, 2
        // declaration entries, 2 declaration nodes; one row per model root.
        assert_eq!(budget.consumed().work, 6);
        assert_eq!(
            budget.read_storage_consumed(),
            (2 * std::mem::size_of::<Root<'_>>() + nodes.len()) as u64
        );
    }

    /// Each root's own first entry selects: a declaration of the second model
    /// at the first model's region is refused at its own entry.
    #[trace("TC-048", "FR-038-AC-582")]
    #[test]
    fn each_models_declaration_must_carry_its_own_roots_regions() {
        let (nodes, kinds) = two_models();
        let entries = [entry('a', 0), entry('b', 5), entry('c', 0), entry('d', 0)];
        let (result, _) = run(&nodes, &kinds, &entries, u64::MAX);
        let Err(ValidationFailure::Refused(refusal)) = result else {
            panic!("a declaration at the other model's region must refuse: {result:?}");
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::InvalidValue)
        );
        assert_eq!(
            refusal.loci[0].path.as_ref().expect("path").as_str(),
            "/source_map/3"
        );
    }

    /// A declaration of a model with no root entry is refused even when its
    /// regions equal another model's, and whichever side of it sorts first.
    #[trace("TC-048", "FR-038-AC-582")]
    #[test]
    fn a_declaration_of_a_model_with_no_root_entry_is_refused() {
        for identity in ["a", "z"] {
            let (mut nodes, mut kinds) = two_models();
            nodes.push(node(
                'e',
                "object_type",
                json!({"kind": "model", "identity": identity, "node": "ix://x/E"}),
            ));
            kinds.push(CheckedNodeKind::Model(ModelForm::ObjectType));
            let entries = [
                entry('a', 0),
                entry('b', 5),
                entry('c', 0),
                entry('d', 5),
                entry('e', 0),
            ];
            let (result, _) = run(&nodes, &kinds, &entries, u64::MAX);
            let Err(ValidationFailure::Refused(refusal)) = result else {
                panic!("{identity}: a rootless declaration must refuse: {result:?}");
            };
            assert_eq!(
                refusal.loci[0].path.as_ref().expect("path").as_str(),
                "/source_map/4"
            );
        }
    }

    /// The pass charges one work unit per root entry, per declaration entry
    /// and per declaration node, and exactly its two tables: a root table of
    /// one row per model root and one flag per graph node. One unit less work
    /// is denied inside the stage.
    #[trace("TC-048", "FR-038-AC-582")]
    #[test]
    fn the_stage_charges_exactly_its_work_and_its_two_tables() {
        let (nodes, kinds) = two_models();
        let entries = [entry('a', 0), entry('b', 5), entry('c', 0), entry('d', 5)];
        let (result, budget) = run(&nodes, &kinds, &entries, u64::MAX);
        assert!(result.is_ok(), "{result:?}");
        // 2 root entries, 2 declaration entries, 2 declaration nodes.
        assert_eq!(budget.consumed().work, 6);
        assert_eq!(
            budget.read_storage_consumed(),
            (2 * std::mem::size_of::<Root<'_>>() + nodes.len()) as u64
        );
        let (short, _) = run(&nodes, &kinds, &entries, 5);
        assert!(
            matches!(short, Err(ValidationFailure::Budget(_))),
            "{short:?}"
        );
        // A package with no declaration node is not charged at all.
        let (none, none_kinds) = (&nodes[..2], &kinds[..2]);
        let (result, budget) = run(none, none_kinds, &entries[..2], u64::MAX);
        assert!(result.is_ok());
        assert_eq!(budget.consumed().work, 0);
        assert_eq!(budget.read_storage_consumed(), 0);
    }
}
