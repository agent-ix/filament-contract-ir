// SPDX-License-Identifier: AGPL-3.0-or-later
//! Validation of the captured locations of function-body terms (FSpec FR-322).
//!
//! A function's body walk is computed from the admitted graph alone. The
//! captured `name`, `index` and `path` of a row are kept as the producer's
//! checker gave them and are checked against no derivation.

use super::checked_term_data::{CheckedBodyLocationOrigin, CheckedBodySemanticLocation};
use super::dispatch_projection::{data_site, invalid, node_positions, position_of};
use super::protocol_locations::{refuse_missing_predecessor, validate_rows};
use super::stage_storage::{self, Path};
use super::{
    CheckedBindingNameKind, CheckedNodeKind, CheckedPackageWireV2, CheckedTerm, ExpressionForm,
    WorkMeter,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::CheckedNodeId;
use std::collections::BTreeMap;

const ARRAY: &str = "body_locations";

/// The control applications a body walk passes through without counting a term.
const WALKED_THROUGH: [&str; 3] = [
    "filament.op.control.let",
    "filament.op.control.if",
    "filament.op.control.case",
];

fn array_path() -> Path<'static> {
    Path::root().key("checked_term_data").key(ARRAY)
}

fn charge(meter: &mut WorkMeter, row: usize, at: Path<'static>) -> Result<(), ValidationFailure> {
    stage_storage::work(meter, &data_site(row, at), 1)
}

/// Terms visited per application node position along one function's body walk.
type Walk = BTreeMap<usize, u64>;

/// The `body` binding of a function node: `Some(target)` when the aggregate
/// body has one, where `target` is the node it references, if it references one.
fn body_binding(term: &CheckedTerm) -> Option<Option<&CheckedNodeId>> {
    let CheckedTerm::Aggregate { members } = term else {
        return None;
    };
    members.iter().find_map(|member| match member {
        CheckedTerm::Binding { name, value } if name.kind() == CheckedBindingNameKind::Body => {
            Some(match value.as_ref() {
                CheckedTerm::Reference { target } => Some(target),
                _ => None,
            })
        }
        _ => None,
    })
}

/// Whether a visit of `kind`/`term` is a term of the walk.
fn is_term(kind: CheckedNodeKind, term: &CheckedTerm) -> bool {
    match (kind, term) {
        (CheckedNodeKind::Expression(ExpressionForm::Reference), _) => false,
        (CheckedNodeKind::Expression(_), CheckedTerm::Application { operation, .. }) => !operation
            .identity()
            .is_some_and(|identity| WALKED_THROUGH.contains(&identity)),
        (CheckedNodeKind::Expression(_), _) => true,
        _ => false,
    }
}

/// The graph a walk reads.
struct Graph<'a, 'w, Body> {
    wire: &'w CheckedPackageWireV2<Body>,
    kinds: &'a [CheckedNodeKind],
    terms: &'a [CheckedTerm],
    index: &'a [(&'w CheckedNodeId, usize)],
}

impl<Body> Graph<'_, '_, Body> {
    /// The positions of the local `expression` nodes `term` references, in the
    /// order a pre-order traversal of the term meets them.
    fn expression_references(&self, term: &CheckedTerm) -> Vec<usize> {
        let mut found = Vec::new();
        let mut stack = vec![term];
        while let Some(next) = stack.pop() {
            match next {
                CheckedTerm::Reference { target } => {
                    if target.package == self.wire.name {
                        if let Some(position) = position_of(self.index, target) {
                            if matches!(
                                self.kinds.get(position),
                                Some(CheckedNodeKind::Expression(_))
                            ) {
                                found.push(position);
                            }
                        }
                    }
                }
                CheckedTerm::Application { arguments, .. } => stack.extend(arguments.iter().rev()),
                CheckedTerm::Aggregate { members } => stack.extend(members.iter().rev()),
                CheckedTerm::Binding { value, .. } => stack.push(value),
                _ => {}
            }
        }
        found
    }

    /// Walk the body of the function at `function`. A node reached along two
    /// references is visited, and counted, twice.
    fn walk(
        &self,
        function: usize,
        row: usize,
        meter: &mut WorkMeter,
    ) -> Result<Walk, ValidationFailure> {
        let mut walk = Walk::new();
        let Some(Some(start)) = self.terms.get(function).and_then(body_binding) else {
            return Ok(walk);
        };
        let Some(start) = (start.package == self.wire.name)
            .then(|| position_of(self.index, start))
            .flatten()
        else {
            return Ok(walk);
        };
        let mut pending = vec![start];
        while let Some(position) = pending.pop() {
            charge(meter, row, array_path())?;
            let (Some(kind), Some(term)) = (self.kinds.get(position), self.terms.get(position))
            else {
                continue;
            };
            if is_term(*kind, term) {
                *walk.entry(position).or_default() += 1;
            }
            let mut next = self.expression_references(term);
            charge_many(meter, row, next.len())?;
            next.reverse();
            pending.extend(next);
        }
        Ok(walk)
    }
}

fn charge_many(meter: &mut WorkMeter, row: usize, count: usize) -> Result<(), ValidationFailure> {
    stage_storage::work(
        meter,
        &data_site(row, array_path()),
        u64::try_from(count).unwrap_or(u64::MAX),
    )
}

/// Check `rows` against the admitted graph; an absent sidecar is `rows == []`.
pub(super) fn validate<Body>(
    wire: &CheckedPackageWireV2<Body>,
    kinds: &[CheckedNodeKind],
    terms: &[CheckedTerm],
    rows: &[CheckedBodySemanticLocation],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let gap = validate_rows(
        ARRAY,
        &wire.name,
        &wire.semantic_graph.nodes,
        kinds,
        terms,
        &wire.source_map,
        rows,
        meter,
    )?;
    let index = node_positions(wire, meter)?;
    let graph = Graph {
        wire,
        kinds,
        terms,
        index: &index,
    };
    let mut names: BTreeMap<usize, (&str, u64)> = BTreeMap::new();
    let mut owners: BTreeMap<u64, usize> = BTreeMap::new();
    let mut counts: BTreeMap<(usize, usize), u64> = BTreeMap::new();
    let mut walks: BTreeMap<usize, Walk> = BTreeMap::new();
    for (row_index, row) in rows.iter().enumerate() {
        let here = array_path().index(row_index);
        let origin = here.key("location").key("origin");
        let CheckedBodyLocationOrigin::FunctionBody {
            function,
            name,
            index: function_index,
        } = &row.location.origin;
        charge(meter, row_index, here)?;
        let function_position = (function.package == wire.name)
            .then(|| position_of(&index, function))
            .flatten()
            .filter(|position| {
                matches!(kinds.get(*position), Some(CheckedNodeKind::Function(_)))
                    && terms.get(*position).and_then(body_binding).is_some()
            });
        let Some(function_position) = function_position else {
            return Err(invalid(meter, row_index, origin.key("function")));
        };
        match names.get(&function_position) {
            Some((held_name, held_index)) => {
                if *held_name != name.as_ref() {
                    return Err(invalid(meter, row_index, origin.key("name")));
                }
                if held_index != function_index {
                    return Err(invalid(meter, row_index, origin.key("index")));
                }
            }
            None => {
                names.insert(function_position, (name, *function_index));
            }
        }
        match owners.get(function_index) {
            Some(owner) if *owner != function_position => {
                return Err(invalid(meter, row_index, origin.key("index")));
            }
            Some(_) => {}
            None => {
                owners.insert(*function_index, function_position);
            }
        }
        let application = position_of(&index, &row.application)
            .ok_or_else(|| invalid(meter, row_index, here.key("application")))?;
        let held = counts.entry((function_position, application)).or_default();
        *held += 1;
        let held = *held;
        if let std::collections::btree_map::Entry::Vacant(slot) = walks.entry(function_position) {
            slot.insert(graph.walk(function_position, row_index, meter)?);
        }
        let terms_at = walks
            .get(&function_position)
            .and_then(|walk| walk.get(&application))
            .copied()
            .unwrap_or(0);
        if held > terms_at {
            return Err(invalid(meter, row_index, here.key("application")));
        }
    }
    refuse_missing_predecessor(ARRAY, gap, meter)?;
    for (position, term) in terms.iter().enumerate() {
        let is_function = matches!(kinds.get(position), Some(CheckedNodeKind::Function(_)))
            && wire
                .semantic_graph
                .nodes
                .get(position)
                .is_some_and(|node| node.node_id.package == wire.name);
        if !is_function || body_binding(term).is_none() {
            continue;
        }
        if let std::collections::btree_map::Entry::Vacant(slot) = walks.entry(position) {
            slot.insert(graph.walk(position, 0, meter)?);
        }
        let Some(walk) = walks.get(&position) else {
            continue;
        };
        for (application, terms_at) in walk {
            let held = counts.get(&(position, *application)).copied().unwrap_or(0);
            if held < *terms_at {
                return Err(invalid(meter, 0, array_path()));
            }
        }
    }
    Ok(())
}
