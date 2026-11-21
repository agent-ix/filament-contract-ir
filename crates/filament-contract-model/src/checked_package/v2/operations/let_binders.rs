// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-038-AC-595: shared, scope-delimited parameter-read summaries for let links.
//!
//! A summary is a persistent sparse tree over the graph's distinct parameter
//! (name, level) pairs. Union shares unchanged subtrees; a nested let does not
//! walk its body again. Two distinct node identities per key suffice to find a
//! read different from any linked binder. Function references are boundaries;
//! expression and value-container references propagate their term summaries.
//! All arenas and worklists are prepaid. Paths are parent edges, materialized
//! only for a refused read, after the diagnostic reservation.

use super::super::stage_storage::{self, Path, Refusal, Site};
use super::*;
use crate::checked_package::common::{Step, Trail};
use std::ops::Range;

#[derive(Clone, Copy)]
enum Edge {
    Body,
    Argument(usize),
    Member(usize),
    Value,
}

struct Term<'g> {
    value: &'g CheckedTerm,
    owner: usize,
    parent: Option<usize>,
    edge: Edge,
    children: Range<usize>,
    summary: usize,
    body: usize,
    state: u8,
}

#[derive(Clone, Copy)]
struct Read {
    target: usize,
    term: usize,
}

#[derive(Clone, Copy, Default)]
struct Summary {
    left: usize,
    right: usize,
    reads: [Option<Read>; 2],
}

pub(super) struct BinderReads<'g> {
    terms: Vec<Term<'g>>,
    roots: Vec<Option<usize>>,
    keys: Vec<Option<usize>>,
    key_count: usize,
    summaries: Vec<Summary>,
    stray: Vec<Option<usize>>,
}

impl<'g> BinderReads<'g> {
    pub(super) fn new(graph: &Graph<'g>, meter: &mut WorkMeter) -> Result<Self, ValidationFailure> {
        let mut parameters = meter.vec(graph.nodes.len())?;
        for position in 0..graph.nodes.len() {
            meter.at_node(
                position,
                super::super::graph_storage::node_form(&graph.nodes[position].node_tag),
            );
            work_at_path(
                meter,
                &Application {
                    node: &graph.nodes[position],
                    position,
                }
                .read_node(),
                1,
            )?;
            if graph.kinds[position] == CheckedNodeKind::Value(ValueForm::Parameter) {
                if let Some((name, level)) = parameter_name_level(position, graph) {
                    parameters.push((name, level, position));
                }
            }
        }
        parameters.sort_unstable();
        let mut keys = meter.vec(graph.nodes.len())?;
        keys.resize(graph.nodes.len(), None);
        let mut key_count = 0;
        let mut previous = None;
        for (name, level, position) in parameters {
            if previous != Some((name, level)) {
                key_count += 1;
                previous = Some((name, level));
            }
            keys[position] = Some(key_count - 1);
        }
        let mut index = Self {
            terms: Vec::new(),
            roots: meter.vec(graph.nodes.len())?,
            keys,
            key_count,
            summaries: meter.vec(1)?,
            stray: meter.vec(graph.nodes.len())?,
        };
        index.summaries.push(Summary::default()); // zero denotes an empty subtree
        index.stray.resize(graph.nodes.len(), None);
        for (position, term) in graph.terms.iter().enumerate() {
            meter.at_node(
                position,
                super::super::graph_storage::node_form(&graph.nodes[position].node_tag),
            );
            let root = if let Some(term) = term {
                Some(index.push_term(term, position, None, Edge::Body, meter)?)
            } else {
                None
            };
            index.roots.push(root);
        }
        let mut at = 0;
        while at < index.terms.len() {
            let entry = &index.terms[at];
            let owner = entry.owner;
            let term = entry.value;
            meter.at_node(
                owner,
                super::super::graph_storage::node_form(&graph.nodes[owner].node_tag),
            );
            let begin = index.terms.len();
            match term {
                CheckedTerm::Application { arguments, .. } => {
                    for (i, child) in arguments.iter().enumerate() {
                        index.push_term(child, owner, Some(at), Edge::Argument(i), meter)?;
                    }
                }
                CheckedTerm::Aggregate { members } => {
                    for (i, child) in members.iter().enumerate() {
                        index.push_term(child, owner, Some(at), Edge::Member(i), meter)?;
                    }
                }
                CheckedTerm::Binding { binder, value, .. } => {
                    let is_let = match (entry.parent, entry.edge) {
                        (Some(parent), Edge::Argument(0)) => matches!(index.terms[parent].value,
                            CheckedTerm::Application { operation, .. }
                            if operation.identity() == Some("filament.op.control.let")),
                        _ => false,
                    };
                    if binder.is_some() && !is_let && index.stray[owner].is_none() {
                        index.stray[owner] = Some(at);
                    }
                    index.push_term(value, owner, Some(at), Edge::Value, meter)?;
                }
                _ => {}
            }
            index.terms[at].children = begin..index.terms.len();
            at += 1;
        }
        let mut pending = Vec::new();
        // Memoize every term, including shared DAG targets, once. The graph
        // stage already refused cycles before the operation stage.
        for root in 0..index.terms.len() {
            if index.terms[root].state == 2 {
                continue;
            }
            meter.push(&mut pending, (root, false))?;
            while let Some((at, finish)) = pending.pop() {
                let owner = index.terms[at].owner;
                let application = Application {
                    node: &graph.nodes[owner],
                    position: owner,
                };
                meter.at_node(
                    owner,
                    super::super::graph_storage::node_form(&application.node.node_tag),
                );
                work_at_path(meter, &application.read_node(), 1)?;
                if finish {
                    let mut summary = 0;
                    let mut body = 0;
                    if let CheckedTerm::Reference { target } = index.terms[at].value {
                        if let Some(&position) = graph.index.get(target) {
                            if let Some(key) = index.keys[position] {
                                summary = index.singleton(
                                    key,
                                    Read {
                                        target: position,
                                        term: at,
                                    },
                                    0..key_count,
                                    meter,
                                )?;
                            } else if follows(graph.kinds[position]) {
                                summary = index.roots[position]
                                    .map_or(0, |root| index.terms[root].summary);
                            }
                        }
                    }
                    for child in index.terms[at].children.clone() {
                        let child_summary = index.terms[child].summary;
                        summary = index.union(summary, child_summary, 0..key_count, meter)?;
                        if matches!(index.terms[child].edge, Edge::Argument(i) if i > 0) {
                            body = index.union(body, child_summary, 0..key_count, meter)?;
                        }
                    }
                    index.terms[at].summary = summary;
                    index.terms[at].body = body;
                    index.terms[at].state = 2;
                } else if index.terms[at].state == 0 {
                    index.terms[at].state = 1;
                    meter.push(&mut pending, (at, true))?;
                    if let CheckedTerm::Reference { target } = index.terms[at].value {
                        if let Some(&position) = graph.index.get(target) {
                            if follows(graph.kinds[position]) {
                                if let Some(root) = index.roots[position] {
                                    if index.terms[root].state == 0 {
                                        meter.push(&mut pending, (root, false))?;
                                    }
                                }
                            }
                        }
                    }
                    for child in index.terms[at].children.clone().rev() {
                        if index.terms[child].state == 0 {
                            meter.push(&mut pending, (child, false))?;
                        }
                    }
                }
            }
        }
        Ok(index)
    }

    fn push_term(
        &mut self,
        value: &'g CheckedTerm,
        owner: usize,
        parent: Option<usize>,
        edge: Edge,
        meter: &mut WorkMeter,
    ) -> Result<usize, ValidationFailure> {
        let at = self.terms.len();
        meter.push(
            &mut self.terms,
            Term {
                value,
                owner,
                parent,
                edge,
                children: 0..0,
                summary: 0,
                body: 0,
                state: 0,
            },
        )?;
        Ok(at)
    }

    fn store(
        &mut self,
        summary: Summary,
        meter: &mut WorkMeter,
    ) -> Result<usize, ValidationFailure> {
        let at = self.summaries.len();
        meter.push(&mut self.summaries, summary)?;
        Ok(at)
    }

    // Recursion is bounded by the bit width of usize: each call halves the
    // compressed parameter-key range, independently of expression depth.
    fn singleton(
        &mut self,
        key: usize,
        read: Read,
        range: Range<usize>,
        meter: &mut WorkMeter,
    ) -> Result<usize, ValidationFailure> {
        work_at_path(meter, &Path::root(), 1)?;
        let mut summary = Summary::default();
        if range.len() == 1 {
            summary.reads[0] = Some(read);
        } else {
            let middle = range.start + range.len() / 2;
            if key < middle {
                summary.left = self.singleton(key, read, range.start..middle, meter)?;
            } else {
                summary.right = self.singleton(key, read, middle..range.end, meter)?;
            }
        }
        self.store(summary, meter)
    }

    fn union(
        &mut self,
        left: usize,
        right: usize,
        range: Range<usize>,
        meter: &mut WorkMeter,
    ) -> Result<usize, ValidationFailure> {
        if left == right || right == 0 {
            return Ok(left);
        }
        if left == 0 {
            return Ok(right);
        }
        work_at_path(meter, &Path::root(), 1)?;
        let a = self.summaries[left];
        let b = self.summaries[right];
        let mut result = a;
        if range.len() == 1 {
            for read in b.reads.into_iter().flatten() {
                if !result
                    .reads
                    .iter()
                    .flatten()
                    .any(|prior| prior.target == read.target)
                {
                    if let Some(slot) = result.reads.iter_mut().find(|slot| slot.is_none()) {
                        *slot = Some(read);
                    }
                }
            }
            if result.reads.iter().flatten().map(|r| r.target).eq(a
                .reads
                .iter()
                .flatten()
                .map(|r| r.target))
            {
                return Ok(left);
            }
        } else {
            let middle = range.start + range.len() / 2;
            result.left = self.union(a.left, b.left, range.start..middle, meter)?;
            result.right = self.union(a.right, b.right, middle..range.end, meter)?;
            if result.left == a.left && result.right == a.right {
                return Ok(left);
            }
            if result.left == b.left && result.right == b.right {
                return Ok(right);
            }
        }
        self.store(result, meter)
    }

    pub(super) fn wrong_read(
        &self,
        owner: usize,
        binder: usize,
        graph: &Graph<'_>,
        meter: &mut WorkMeter,
    ) -> Result<Option<ValidationFailure>, ValidationFailure> {
        let (Some(root), Some(key)) = (self.roots[owner], self.keys[binder]) else {
            return Ok(None);
        };
        let mut summary = self.terms[root].body;
        let mut range = 0..self.key_count;
        while summary != 0 && range.len() > 1 {
            work_at_path(
                meter,
                &Application {
                    node: &graph.nodes[owner],
                    position: owner,
                }
                .read_node(),
                1,
            )?;
            let middle = range.start + range.len() / 2;
            if key < middle {
                summary = self.summaries[summary].left;
                range.end = middle;
            } else {
                summary = self.summaries[summary].right;
                range.start = middle;
            }
        }
        if let Some(read) = self.summaries[summary]
            .reads
            .iter()
            .flatten()
            .find(|read| read.target != binder)
        {
            return Ok(Some(self.refuse(read.term, None, graph, meter)?));
        }
        Ok(None)
    }

    pub(super) fn stray_defect(
        &self,
        owner: usize,
        graph: &Graph<'_>,
        meter: &mut WorkMeter,
    ) -> Result<Option<ValidationFailure>, ValidationFailure> {
        self.stray[owner]
            .map(|at| self.refuse(at, Some("binder"), graph, meter))
            .transpose()
    }

    fn refuse(
        &self,
        at: usize,
        member: Option<&str>,
        graph: &Graph<'_>,
        meter: &mut WorkMeter,
    ) -> Result<ValidationFailure, ValidationFailure> {
        let owner = self.terms[at].owner;
        let node = &graph.nodes[owner];
        let path = Application {
            node,
            position: owner,
        }
        .read_node();
        let site = Site::new(
            super::super::PackageReadStage::Graph,
            owner,
            super::super::graph_storage::node_form(&node.node_tag),
            path,
        );
        let mut count = 3 + usize::from(member.is_some());
        let mut cursor = Some(at);
        while let Some(at) = cursor {
            count += match self.terms[at].edge {
                Edge::Argument(_) | Edge::Member(_) => 2,
                _ => 1,
            };
            cursor = self.terms[at].parent;
        }
        let mut steps = stage_storage::vec(meter, &site, count)?;
        let mut cursor = Some(at);
        while let Some(at) = cursor {
            match self.terms[at].edge {
                Edge::Body => steps.push(Step::Key("body")),
                Edge::Argument(i) => {
                    steps.push(Step::Index(i));
                    steps.push(Step::Key("arguments"));
                }
                Edge::Member(i) => {
                    steps.push(Step::Index(i));
                    steps.push(Step::Key("members"));
                }
                Edge::Value => steps.push(Step::Key("value")),
            }
            cursor = self.terms[at].parent;
        }
        steps.extend([
            Step::Index(owner),
            Step::Key("nodes"),
            Step::Key("semantic_graph"),
        ]);
        steps.reverse();
        if let Some(member) = member {
            steps.push(Step::Key(member));
        }
        Ok(stage_storage::refuse(
            meter,
            &site,
            Refusal::new(
                CheckedPackageRefusalCode::InvalidPackage,
                Trail::Base(&steps),
            )
            .because(CheckedPackageRefusalCause::InvalidValue)
            .at_node(&node.node_id),
        ))
    }
}

fn follows(kind: CheckedNodeKind) -> bool {
    matches!(
        kind,
        CheckedNodeKind::Expression(_)
            | CheckedNodeKind::Value(
                ValueForm::RecordValue
                    | ValueForm::TupleValue
                    | ValueForm::CollectionValue
                    | ValueForm::UnionValue
                    | ValueForm::OptionValue
            )
    )
}
