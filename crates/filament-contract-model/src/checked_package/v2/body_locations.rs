// SPDX-License-Identifier: AGPL-3.0-or-later
//! Validation of the captured locations of function-body terms (FSpec FR-322).
//!
//! A function's body walk is computed from the admitted graph alone. The
//! captured `name`, `index` and path of a row are kept as the producer's
//! checker gave them and are checked against no derivation. A row writes its
//! path relative to the previous term of the same walk (`{up, down}`); the
//! reader rebuilds the paths in walk order, pairing the `k`-th visit at an
//! application with that application's row of rank `k`, and checks only that
//! each `up` fits the previous path and is the least one (FR-322-AC-510).

use super::checked_term_data::{
    CheckedBodyLocationOrigin, CheckedBodySemanticLocation, CheckedKeyOrdinal,
};
use super::dispatch_projection::{data_site, invalid, node_positions, position_of};
use super::protocol_locations::{refuse_missing_predecessor, validate_rows};
use super::stage_storage::{self, Path};
use super::{
    CheckedBindingNameKind, CheckedNodeKind, CheckedPackageWireV2, CheckedTerm, ExpressionForm,
    ValueForm, WorkMeter,
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

/// One function's body walk: the node position of each term in visit order,
/// and how many terms the walk has at each application position.
#[derive(Default)]
struct Walk {
    order: Vec<usize>,
    counts: BTreeMap<usize, u64>,
}

/// The `body` binding of a function node: `Some(target)` when the aggregate
/// body has one, where `target` is the node it references, if it references one.
fn body_binding(term: &CheckedTerm) -> Option<Option<&CheckedNodeId>> {
    let CheckedTerm::Aggregate { members } = term else {
        return None;
    };
    members.iter().find_map(|member| match member {
        CheckedTerm::Binding { name, value, .. } if name.kind() == CheckedBindingNameKind::Body => {
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
    /// The positions of local expression and composite-value nodes `term`
    /// references, in the order a pre-order traversal meets them. Record and
    /// tuple aggregates expose their children without contributing a term.
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
                                    | Some(CheckedNodeKind::Value(
                                        ValueForm::RecordValue | ValueForm::TupleValue
                                    ))
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
        let mut walk = Walk::default();
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
                *walk.counts.entry(position).or_default() += 1;
                walk.order.push(position);
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
    let (gap, index) = validate_rows(
        ARRAY,
        &wire.name,
        &wire.semantic_graph.nodes,
        kinds,
        terms,
        &wire.source_map,
        rows,
        meter,
    )?;
    let index = if rows.is_empty() {
        node_positions(wire, meter)?
    } else {
        index
    };
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
    // The rows of each function and application, in array order, which is rank
    // order: rows are stored in (`application`, `occurrence`, `term_ordinal`)
    // order.
    let mut slots: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
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
                stage_storage::work(
                    meter,
                    &data_site(row_index, origin.key("name")),
                    u64::try_from(name.len()).unwrap_or(u64::MAX),
                )?;
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
        slots
            .entry((function_position, application))
            .or_default()
            .push(row_index);
        if let std::collections::btree_map::Entry::Vacant(slot) = walks.entry(function_position) {
            slot.insert(graph.walk(function_position, row_index, meter)?);
        }
        let terms_at = walks
            .get(&function_position)
            .and_then(|walk| walk.counts.get(&application))
            .copied()
            .unwrap_or(0);
        if held > terms_at {
            return Err(invalid(meter, row_index, here.key("application")));
        }
    }
    refuse_missing_predecessor(ARRAY, gap, meter)?;
    let mut functions = Vec::new();
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
        for (application, terms_at) in &walk.counts {
            let held = counts.get(&(position, *application)).copied().unwrap_or(0);
            if held < *terms_at {
                return Err(invalid(meter, 0, array_path()));
            }
        }
        functions.push(position);
    }
    // Every row-level and missing-row check has passed: rebuild the paths in
    // walk order. Each row pairs with exactly one visit, so the work is linear
    // in the walk and the entries the rows hold.
    for position in functions {
        let Some(walk) = walks.get(&position) else {
            continue;
        };
        let mut previous: Vec<CheckedKeyOrdinal> = Vec::new();
        let mut visited: BTreeMap<usize, usize> = BTreeMap::new();
        for application in &walk.order {
            let rank = visited.entry(*application).or_default();
            let row_index = slots
                .get(&(position, *application))
                .and_then(|rows| rows.get(*rank))
                .copied()
                .ok_or_else(|| invalid(meter, 0, array_path()))?;
            *rank += 1;
            let location = &rows[row_index].location;
            let up_at = array_path().index(row_index).key("location").key("up");
            charge(meter, row_index, up_at)?;
            if !step(&mut previous, location.up, &location.down, |amount| {
                stage_storage::work(meter, &data_site(row_index, up_at), amount)
            })? {
                return Err(invalid(meter, row_index, up_at));
            }
        }
    }
    Ok(())
}

/// Replace `previous` with the path `{up, down}` writes after it. `Ok(false)`
/// is a row the reader refuses: its `up` exceeds `previous`, or is not the
/// least one because `down` begins with the entry `up` removed last.
fn step(
    previous: &mut Vec<CheckedKeyOrdinal>,
    up: u64,
    down: &[CheckedKeyOrdinal],
    mut charge_up: impl FnMut(u64) -> Result<(), ValidationFailure>,
) -> Result<bool, ValidationFailure> {
    let Some(keep) = usize::try_from(up)
        .ok()
        .and_then(|up| previous.len().checked_sub(up))
    else {
        return Ok(false);
    };
    charge_up(up)?;
    if let (Some(first), Some(removed)) = (down.first(), previous.get(keep)) {
        if first == removed {
            return Ok(false);
        }
    }
    previous.truncate(keep);
    previous.extend(down.iter().cloned());
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;

    fn key(value: u64) -> CheckedKeyOrdinal {
        CheckedKeyOrdinal::Safe(value)
    }

    fn apply(previous: &mut Vec<CheckedKeyOrdinal>, up: u64, down: &[u64]) -> bool {
        let down: Vec<_> = down.iter().copied().map(key).collect();
        step(previous, up, &down, |_| Ok(())).expect("no meter")
    }

    /// FR-322-AC-510: the path is the previous path less its last `up`
    /// entries, then `down`; BL-01's walk order gives `[0]`, `[1]`, `[]`.
    #[test]
    #[trace("TC-491", "FR-038-AC-597", "FR-322-AC-510")]
    fn a_row_rebuilds_its_path_from_the_previous_one() {
        let mut path = Vec::new();
        assert!(apply(&mut path, 0, &[0]));
        assert_eq!(path, [key(0)]);
        assert!(apply(&mut path, 1, &[1]));
        assert_eq!(path, [key(1)]);
        assert!(apply(&mut path, 1, &[]));
        assert!(path.is_empty());
        assert!(apply(&mut path, 0, &[]));
        assert!(path.is_empty());
    }

    /// BL-R9: an `up` beyond the previous path's length is refused, and the
    /// first row of a walk has the empty path before it.
    #[test]
    #[trace("TC-491", "FR-038-AC-597", "FR-322-AC-510")]
    fn an_up_beyond_the_previous_path_is_refused() {
        let mut path = vec![key(0)];
        assert!(!apply(&mut path, 2, &[1]));
        let mut empty = Vec::new();
        assert!(!apply(&mut empty, 1, &[]));
        assert!(apply(&mut vec![key(0)], 1, &[]));
    }

    /// BL-R10: a `down` that begins with the entry `up` removed last gives the
    /// same path with a smaller `up`, so the row is not in least form; any other
    /// first entry, or `up` 0, is least.
    #[test]
    #[trace("TC-491", "FR-038-AC-597", "FR-322-AC-510")]
    fn an_up_that_is_not_the_least_is_refused() {
        assert!(!apply(&mut vec![key(0)], 1, &[0]));
        assert!(!apply(&mut vec![key(3), key(1), key(2)], 2, &[1, 5]));
        assert!(apply(&mut vec![key(3), key(1), key(2)], 2, &[2, 5]));
        assert!(apply(&mut vec![key(0)], 0, &[0]));
        assert!(apply(&mut vec![key(0)], 1, &[1]));
    }

    /// BL-D1 and BL-D2: a left-nested 100,000-term sum and 100,000 nested
    /// `let`s rebuild the paths the vector names, and the metered `up` work
    /// stays linear (at most the entries the rows hold).
    #[test]
    #[trace("TC-491", "FR-038-AC-597", "FR-322-AC-510")]
    fn deep_bodies_rebuild_in_linear_work() {
        let mut path = Vec::new();
        let mut charged = 0_u64;
        for k in 0..100_000_usize {
            let (up, down) = if k == 0 {
                (0, vec![])
            } else {
                (0, vec![key(0)])
            };
            assert!(step(&mut path, up, &down, |amount| {
                charged += amount;
                Ok(())
            })
            .expect("no meter"));
            assert_eq!(path.len(), k);
        }
        assert_eq!(charged, 0);
        let mut path = Vec::new();
        for k in 0..100_000_usize {
            let (up, down) = if k == 0 {
                (0, vec![key(0)])
            } else {
                (1, vec![key(1), key(0)])
            };
            assert!(step(&mut path, up, &down, |amount| {
                charged += amount;
                Ok(())
            })
            .expect("no meter"));
            // `k` ones then a zero.
            assert_eq!(path.len(), k + 1);
            assert_eq!(path.last(), Some(&key(0)));
        }
        assert!(charged <= 100_000);
    }
}
