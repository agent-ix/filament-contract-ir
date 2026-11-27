// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded, lazy reconstruction of admitted function-body locations.

use super::body_locations::{body_binding, is_term, is_walk_node};
use super::checked_term_data::{
    CheckedBodyLocationOrigin, CheckedBodySemanticLocation, CheckedKeyOrdinal,
};
use super::{CheckedNodeId, CheckedNodeKind, CheckedPackageV2, CheckedTerm};
use ix_trace_rs::implements;

/// Why a function's reconstructed body locations could not be read.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum CheckedBodyLocationReadError {
    /// The package retained no captured body-location array.
    #[error("body location data is unavailable")]
    Unavailable,
    /// The selected local function has no body walk.
    #[error("the selected function has no body walk")]
    MissingFunction,
    /// An expected captured row is unavailable.
    #[error("the body walk has no captured row")]
    MissingRow,
    /// The first unpaid setup, walk or path step exceeded the caller's limit.
    #[error("body location reconstruction exhausted its work allowance")]
    WorkExhausted {
        /// Caller-supplied finite work limit.
        limit: u64,
        /// Attempted cumulative work, including the denied step.
        consumed: u64,
    },
}

/// One walk visit, borrowing its captured row and reconstructed child path.
/// The path borrow expires before the cursor can advance again.
pub struct CheckedBodyLocationRead<'a> {
    /// The exact admitted application, occurrence and term coordinate.
    pub row: &'a CheckedBodySemanticLocation,
    /// The rebuilt checker path, rather than its relative wire encoding.
    pub path: &'a [CheckedKeyOrdinal],
}

enum Task<'a> {
    Node(usize),
    Term(&'a CheckedTerm),
}

/// A lending cursor over one function's locations in body-walk order.
///
/// Only the current rebuilt path is retained. Setup and traversal storage are
/// linear in graph nodes and captured rows; visiting all rows costs the walk
/// steps and encoded delta entries, without copying every full path. Dropping
/// the cursor cancels reconstruction without changing the admitted package.
pub struct CheckedBodyLocations<'a> {
    package: &'a CheckedPackageV2,
    rows: &'a [CheckedBodySemanticLocation],
    slots: Vec<(usize, usize)>,
    visits: Vec<usize>,
    pending: Vec<Task<'a>>,
    path: Vec<CheckedKeyOrdinal>,
    limit: u64,
    consumed: u64,
    finished: bool,
}

impl CheckedPackageV2 {
    /// Lazily rebuild one local function's captured paths in body-walk order.
    ///
    /// The caller's work limit prepays every setup entry, traversal step,
    /// pending task and removed/appended path entry. A wide decimal ordinal
    /// also charges its byte length before cloning it into the current path.
    /// No full paths are materialized during construction.
    #[implements("FR-322-AC-539")]
    #[implements("FR-322-AC-541")]
    #[implements("FR-322-AC-542")]
    pub fn body_locations(
        &self,
        function: &CheckedNodeId,
        work_limit: u64,
    ) -> Result<CheckedBodyLocations<'_>, CheckedBodyLocationReadError> {
        let rows = &self
            .wire
            .checked_term_data
            .as_ref()
            .ok_or(CheckedBodyLocationReadError::Unavailable)?
            .body_locations;
        let mut cursor = CheckedBodyLocations {
            package: self,
            rows,
            slots: Vec::new(),
            visits: Vec::new(),
            pending: Vec::new(),
            path: Vec::new(),
            limit: work_limit,
            consumed: 0,
            finished: false,
        };
        cursor.charge(1)?;
        let position = self
            .node_index
            .node(&self.wire.semantic_graph.nodes, function)
            .ok_or(CheckedBodyLocationReadError::MissingFunction)?;
        if function.package != self.wire.name
            || !matches!(self.kinds.get(position), Some(CheckedNodeKind::Function(_)))
        {
            return Err(CheckedBodyLocationReadError::MissingFunction);
        }
        let start = self
            .wire
            .semantic_graph
            .nodes
            .get(position)
            .and_then(|node| body_binding(&node.body))
            .ok_or(CheckedBodyLocationReadError::MissingFunction)?;
        let Some(start) = start.filter(|start| start.package == self.wire.name) else {
            return Ok(cursor);
        };
        cursor.charge(1)?;
        let start = self
            .node_index
            .node(&self.wire.semantic_graph.nodes, start)
            .ok_or(CheckedBodyLocationReadError::MissingFunction)?;
        if !self.kinds.get(start).copied().is_some_and(is_walk_node) {
            return Ok(cursor);
        }
        cursor.charge_count(self.wire.semantic_graph.nodes.len())?;
        cursor.visits = vec![0; self.wire.semantic_graph.nodes.len()];
        cursor.charge_count(rows.len())?;
        cursor.slots = Vec::with_capacity(rows.len());
        for (row_index, row) in rows.iter().enumerate() {
            cursor.charge(1)?;
            let CheckedBodyLocationOrigin::FunctionBody {
                function: owner, ..
            } = &row.location.origin;
            if owner == function {
                cursor.charge(1)?;
                let application = self
                    .node_index
                    .node(&self.wire.semantic_graph.nodes, &row.application)
                    .ok_or(CheckedBodyLocationReadError::MissingRow)?;
                cursor.slots.push((application, row_index));
            }
        }
        cursor.slots.sort_unstable();
        cursor.push(Task::Node(start))?;
        Ok(cursor)
    }
}

impl<'a> CheckedBodyLocations<'a> {
    fn charge(&mut self, amount: u64) -> Result<(), CheckedBodyLocationReadError> {
        let next = self.consumed.checked_add(amount);
        self.consumed = next.unwrap_or(u64::MAX);
        if next.is_none() || self.consumed > self.limit {
            self.finished = true;
            return Err(CheckedBodyLocationReadError::WorkExhausted {
                limit: self.limit,
                consumed: self.consumed,
            });
        }
        Ok(())
    }

    fn charge_count(&mut self, count: usize) -> Result<(), CheckedBodyLocationReadError> {
        self.charge(u64::try_from(count).unwrap_or(u64::MAX))
    }

    fn push(&mut self, task: Task<'a>) -> Result<(), CheckedBodyLocationReadError> {
        self.charge(1)?;
        self.reserve_pending(1)?;
        self.pending.push(task);
        Ok(())
    }

    fn reserve_pending(&mut self, additional: usize) -> Result<(), CheckedBodyLocationReadError> {
        let required = self.pending.len().saturating_add(additional);
        if required > self.pending.capacity() {
            let capacity = self
                .pending
                .capacity()
                .saturating_mul(2)
                .max(4)
                .max(required);
            self.charge_count(capacity - self.pending.capacity())?;
            self.pending.reserve_exact(capacity - self.pending.len());
        }
        Ok(())
    }

    /// Cumulative setup, traversal and reconstruction work charged so far.
    pub fn work_consumed(&self) -> u64 {
        self.consumed
    }

    /// Return the next captured row with its rebuilt path, borrowing this
    /// cursor until the view is released. Exhaustion returns no partial view
    /// and permanently ends the cursor; subsequent calls return `Ok(None)`.
    pub fn next_location(
        &mut self,
    ) -> Result<Option<CheckedBodyLocationRead<'_>>, CheckedBodyLocationReadError> {
        if self.finished {
            return Ok(None);
        }
        let row_index = match self.advance() {
            Ok(Some(index)) => index,
            Ok(None) => {
                self.finished = true;
                return Ok(None);
            }
            Err(error) => {
                self.finished = true;
                return Err(error);
            }
        };
        Ok(Some(CheckedBodyLocationRead {
            row: &self.rows[row_index],
            path: &self.path,
        }))
    }

    fn advance(&mut self) -> Result<Option<usize>, CheckedBodyLocationReadError> {
        while !self.pending.is_empty() {
            self.charge(1)?;
            let Some(task) = self.pending.pop() else {
                break;
            };
            match task {
                Task::Node(position) => {
                    let node = self
                        .package
                        .wire
                        .semantic_graph
                        .nodes
                        .get(position)
                        .ok_or(CheckedBodyLocationReadError::MissingRow)?;
                    self.push(Task::Term(&node.body))?;
                    if self
                        .package
                        .kinds
                        .get(position)
                        .is_some_and(|kind| is_term(*kind, &node.body))
                    {
                        let first = self
                            .slots
                            .partition_point(|(application, _)| *application < position);
                        let rank = self
                            .visits
                            .get_mut(position)
                            .ok_or(CheckedBodyLocationReadError::MissingRow)?;
                        let selected = first
                            .checked_add(*rank)
                            .ok_or(CheckedBodyLocationReadError::MissingRow)?;
                        let row_index = self
                            .slots
                            .get(selected)
                            .filter(|(application, _)| *application == position)
                            .map(|(_, row)| *row)
                            .ok_or(CheckedBodyLocationReadError::MissingRow)?;
                        *rank += 1;
                        let row = &self.rows[row_index];
                        self.charge(1)?;
                        self.charge(row.location.up)?;
                        let keep = usize::try_from(row.location.up)
                            .ok()
                            .and_then(|up| self.path.len().checked_sub(up))
                            .ok_or(CheckedBodyLocationReadError::MissingRow)?;
                        self.charge_count(row.location.down.len())?;
                        for entry in &row.location.down {
                            if let CheckedKeyOrdinal::BeyondSafe(text) = entry {
                                self.charge_count(text.len())?;
                            }
                        }
                        self.path.truncate(keep);
                        let required = keep.saturating_add(row.location.down.len());
                        if required > self.path.capacity() {
                            let capacity =
                                self.path.capacity().saturating_mul(2).max(4).max(required);
                            self.charge_count(capacity - self.path.capacity())?;
                            self.path.reserve_exact(capacity - self.path.len());
                        }
                        self.path.extend(row.location.down.iter().cloned());
                        return Ok(Some(row_index));
                    }
                }
                Task::Term(term) => match term {
                    CheckedTerm::Reference { target } => {
                        self.charge(1)?;
                        if target.package == self.package.wire.name {
                            if let Some(position) = self
                                .package
                                .node_index
                                .node(&self.package.wire.semantic_graph.nodes, target)
                            {
                                if self
                                    .package
                                    .kinds
                                    .get(position)
                                    .copied()
                                    .is_some_and(is_walk_node)
                                {
                                    self.push(Task::Node(position))?;
                                }
                            }
                        }
                    }
                    CheckedTerm::Application { arguments, .. } => {
                        self.charge_count(arguments.len())?;
                        self.reserve_pending(arguments.len())?;
                        self.pending.extend(arguments.iter().rev().map(Task::Term));
                    }
                    CheckedTerm::Aggregate { members, .. } => {
                        self.charge_count(members.len())?;
                        self.reserve_pending(members.len())?;
                        self.pending.extend(members.iter().rev().map(Task::Term));
                    }
                    CheckedTerm::Binding { value, .. } => self.push(Task::Term(value))?,
                    _ => {}
                },
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;
    use serde_json::json;

    fn fixture_package() -> CheckedPackageV2 {
        let caller = super::super::test_support::id('c');
        let outer = super::super::test_support::id('o');
        let app = super::super::test_support::id('a');
        let node = |node_id: serde_json::Value, tag: &str, form: &str, body| {
            json!({
                "node_id":node_id,
                "schema_version":"filament.checked-semantic-graph/v2",
                "node_tag":tag,
                "semantic_form":form,
                "semantic_type":node_id,
                "dependencies":[],
                "occurrences":[{"role":"expression","ordinal":0}],
                "body":body
            })
        };
        let wire = super::super::test_support::private_wire(json!({
            "contract_version":"filament.checked-package/v2",
            "name":"test",
            "lock":{
                "sources":[],
                "edition":{"role":"edition","definition":{"authority":"test","identity":"edition"}},
                "profile_selections":[],"definition_selections":[],
                "required_features":[],"dependency_selections":[]
            },
            "semantic_graph":{
                "graph_version":"filament.checked-semantic-graph/v2",
                "nodes":[
                    node(caller.clone(), "function", "pure_function", json!({
                        "term":"aggregate",
                        "members":[{"term":"binding","name":"body","value":{"term":"reference","target":outer}}]
                    })),
                    node(outer, "value", "option_value", json!({
                        "term":"aggregate",
                        "members":[{"term":"binding","name":"some","value":{"term":"reference","target":app}}]
                    })),
                    node(app.clone(), "expression", "binary", json!({
                        "term":"application","operator":"binary",
                        "operation":{"identity":"filament.op.test","laws":[],"mode":null,"member":null,"leaves":[]},
                        "result_type":app.clone(),"arguments":[]
                    }))
                ]
            },
            "checked_term_data":{
                "equality_decisions":[],"protocol_locations":[],"dispatch_tables":[],
                "body_locations":[{
                    "application":app,
                    "occurrence":{"role":"expression","ordinal":0},
                    "term_ordinal":0,
                    "location":{
                        "origin":{"kind":"function_body","function":caller,"name":"caller","index":0},
                        "up":0,"down":[0]
                    }
                }],
                "probabilistic_claims":[]
            },
            "source_map":[],"capability_report":[],
            "diagnostics":{"catalog":{"authority":"test","identity":"diagnostics"},"entries":[]}
        }));
        let kinds = wire
            .semantic_graph
            .nodes
            .iter()
            .map(|node| {
                let tag =
                    super::super::CheckedNodeTag::from_wire(&node.node_tag).expect("fixture tag");
                CheckedNodeKind::decode(tag, &node.semantic_form).expect("fixture form")
            })
            .collect();
        CheckedPackageV2 {
            correlation: Default::default(),
            schedulability_claim_index: Vec::new(),
            monitor_plan_index: Vec::new(),
            terminal_rows: std::sync::Arc::new(Vec::new()),
            probabilistic_claims: std::sync::Arc::new(Vec::new()),
            timed_model_roots: Vec::new(),
            node_index: std::sync::Arc::new(super::super::retained_node_index(&wire)),
            structural_keys: Vec::new(),
            slot_graph: Default::default(),
            source_index: std::sync::Arc::new(super::super::retained_source_index(&wire)),
            lowering_facts: Vec::new(),
            wire: std::sync::Arc::new(wire),
            kinds: std::sync::Arc::new(kinds),
            bytes: 1 << 20,
            models: std::sync::Arc::new(super::super::RetainedModels::default()),
            selected_documents: std::sync::Arc::new(Vec::new()),
            direct_dependencies: Vec::new(),
            closure_index: Default::default(),
            catalog: super::super::operation_catalog::owned_operation_catalog(),
            admission_binding: Default::default(),
            body_documents: Vec::new(),
        }
        .retain_original_admission()
    }

    #[trace("FR-322-AC-539", "FR-322-AC-541")]
    #[test]
    fn body_cursor_pairs_option_payload_row_with_rebuilt_path() {
        let package = fixture_package();
        let function = super::super::test_support::node_id('c');
        let expected_application = super::super::test_support::node_id('a');
        let mut cursor = package
            .body_locations(&function, 1_000)
            .expect("function body cursor");
        let location = cursor
            .next_location()
            .expect("cursor step")
            .expect("option payload application row");

        assert_eq!(location.row.application, expected_application);
        assert_eq!(location.row.occurrence.ordinal, 0);
        assert_eq!(location.row.term_ordinal, 0);
        assert_eq!(location.path, [CheckedKeyOrdinal::Safe(0)]);
        assert!(cursor.next_location().expect("cursor exhaustion").is_none());
    }
}
