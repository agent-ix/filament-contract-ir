// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Work and order of the requirement-record structural-key orderings.

use super::{
    key_order, named_record_order, row_key_order, sort_by_structural_key, sort_charged, GraphIndex,
    RowKey, WorkMeter,
};
use crate::checked_package::shared::checked_node_comparisons;
use crate::checked_package::shared::{CheckedNodeId, CheckedNodePathSegment as Segment};
use ix_trace_rs::trace;
use std::cmp::Ordering;
use std::sync::Arc;

/// The flat wire identity of the node at `position`.
fn wire_id(position: usize) -> CheckedNodeId {
    CheckedNodeId {
        package: "p".into(),
        path: vec![
            Segment::Name("node".into()),
            Segment::Index(position as u64),
        ],
        ordinal: 0,
    }
}

/// The package-order key of an `expression` value-validity row claimed at
/// graph `position`.
fn row_key<'a>(
    index: &GraphIndex<'_>,
    keys: &'a [Arc<CheckedNodeId>],
    position: usize,
    ordinal: u64,
) -> RowKey<'a> {
    (
        index.resolved_at(position, keys),
        "expression",
        ordinal,
        "value-validity",
    )
}

/// A function owner, a chain of `length` scalar applications each nesting
/// the previous one, and one value-validity record per application whose key
/// nests the owner and its application: the shape the FSL producer writes for
/// `x + 1 + 1 + ...`.
struct Chain {
    wire: Vec<CheckedNodeId>,
    keys: Vec<Arc<CheckedNodeId>>,
    applications: Vec<usize>,
    records: Vec<usize>,
}

fn chain(length: usize) -> Chain {
    let mut keys = vec![Arc::new(CheckedNodeId {
        package: "p".into(),
        path: vec![Segment::Name("function".into()), Segment::Name("f".into())],
        ordinal: 0,
    })];
    let owner = Arc::clone(&keys[0]);
    let (mut applications, mut records) = (Vec::new(), Vec::new());
    let mut previous: Option<Arc<CheckedNodeId>> = None;
    for _ in 0..length {
        let operand = previous
            .take()
            .map_or_else(|| Segment::Name("x".into()), Segment::Node);
        let application = Arc::new(CheckedNodeId {
            package: "p".into(),
            path: vec![
                Segment::Name("expression".into()),
                Segment::Name("integer_add".into()),
                operand,
                Segment::Name("1".into()),
            ],
            ordinal: 0,
        });
        applications.push(keys.len());
        keys.push(Arc::clone(&application));
        records.push(keys.len());
        keys.push(Arc::new(CheckedNodeId {
            package: "p".into(),
            path: vec![
                Segment::Name("claim".into()),
                Segment::Name("value_validity".into()),
                Segment::Node(Arc::clone(&owner)),
                Segment::Node(Arc::clone(&application)),
                Segment::Name("0".into()),
            ],
            ordinal: 0,
        }));
        previous = Some(application);
    }
    Chain {
        wire: (0..keys.len()).map(wire_id).collect(),
        keys,
        applications,
        records,
    }
}

/// A fixed permutation that moves every entry: odd positions first, each
/// half reversed.
fn scrambled(positions: &[usize]) -> Vec<usize> {
    let mut odd: Vec<usize> = positions.iter().copied().skip(1).step_by(2).collect();
    let mut even: Vec<usize> = positions.iter().copied().step_by(2).collect();
    odd.reverse();
    even.reverse();
    odd.extend(even);
    odd
}

/// Work of the record sort and the row-order scan over one chain: comparator
/// calls plus nested node comparisons, for the ranked ordering and for the
/// retired `CheckedNodeId::cmp` ordering, which is kept here only as the
/// reference.
struct Work {
    ranked: u64,
    retired: Option<u64>,
}

fn chain_work(length: usize, with_retired: bool) -> Work {
    let chain = chain(length);
    let mut meter = WorkMeter::new(u64::MAX);
    let index = GraphIndex::build(chain.wire.iter(), &mut meter)
        .and_then(|index| index.with_ranks(&chain.keys, &mut meter))
        .unwrap_or_else(|_| panic!("chain graph indexes"));
    assert!(
        (0..chain.keys.len()).all(|position| index.rank_at(position).is_some()),
        "every operand of the chain is itself a key, so the graph is ranked"
    );

    // The record sort.
    let input = scrambled(&chain.records);
    let mut ranked = input.clone();
    let mut calls = 0_u64;
    checked_node_comparisons::reset();
    sort_charged(&mut ranked, &mut meter, |left, right, meter| {
        calls += 1;
        Ok(key_order(&index, &chain.keys, left, right, meter)?.then(left.cmp(&right)))
    })
    .unwrap_or_else(|_| panic!("ranked sort admits"));
    let mut ranked_work = calls + checked_node_comparisons::count();
    let mut production = input.clone();
    sort_by_structural_key(&mut production, &index, &chain.keys, &mut meter)
        .unwrap_or_else(|_| panic!("production sort admits"));
    assert_eq!(production, ranked);

    assert_eq!(ranked, chain.records, "records follow their applications");

    // The row-order scan: one comparison per adjacent pair of claims.
    checked_node_comparisons::reset();
    for pair in chain.applications.windows(2) {
        let order = row_key_order(
            &row_key(&index, &chain.keys, pair[0], 0),
            &row_key(&index, &chain.keys, pair[1], 0),
            &mut meter,
        )
        .unwrap_or_else(|_| panic!("ranked row comparison admits"));
        assert_eq!(order, Ordering::Less);
    }
    ranked_work +=
        (chain.applications.len() as u64).saturating_sub(1) + checked_node_comparisons::count();
    if !with_retired {
        return Work {
            ranked: ranked_work,
            retired: None,
        };
    }

    let mut retired = input;
    let mut retired_calls = 0_u64;
    checked_node_comparisons::reset();
    retired.sort_unstable_by(|left, right| {
        retired_calls += 1;
        chain.keys[*left].cmp(&chain.keys[*right])
    });
    let mut retired_work = retired_calls + checked_node_comparisons::count();
    assert_eq!(ranked, retired, "ranks keep the canonical record order");
    checked_node_comparisons::reset();
    for pair in chain.applications.windows(2) {
        let left = (
            chain.keys[pair[0]].as_ref(),
            "expression",
            0_u64,
            "value-validity",
        );
        let right = (
            chain.keys[pair[1]].as_ref(),
            "expression",
            0_u64,
            "value-validity",
        );
        assert!(left < right);
    }
    retired_work +=
        (chain.applications.len() as u64).saturating_sub(1) + checked_node_comparisons::count();
    Work {
        ranked: ranked_work,
        retired: Some(retired_work),
    }
}

/// IR-1588: ordering the value-validity records of a chain of `n` scalar
/// applications, and checking their rows' package order, costs O(n log n)
/// comparisons of O(1) work each. The retired order walked the shared
/// application chain on every comparison: O(n^2 log n).
///
/// Work counted, for the ranked order and for the retired reference alike:
/// (1) the record sort (`sort_charged` with the production comparator, or
/// the retired `sort_unstable_by(CheckedNodeId::cmp)`): one unit per
/// comparator call plus one per
/// `checked_node_comparisons` visit (each `CheckedNodeId` comparison entered
/// and each distinct nested operand pair it descends into); and (2) the
/// row-order scan: one unit per adjacent row pair compared by
/// `row_key_order` plus its node visits. The named-record sort is not
/// counted: on this chain every record has the same owner, whose key
/// compares by identity, and the row index decides.
#[trace("TC-488", "FR-376-AC-3")]
#[test]
fn record_ordering_work_is_linearithmic_on_a_chain_of_scalar_applications() {
    // The retired reference is measured at the two smaller sizes only: at
    // 8,000 records it alone takes most of a minute in a debug build.
    let sizes = [2_000_usize, 4_000, 8_000];
    let work: Vec<Work> = sizes
        .iter()
        .map(|&length| chain_work(length, length <= 4_000))
        .collect();
    for (length, work) in sizes.iter().zip(&work) {
        println!(
            "IR-1588 chain n={length}: ranked work={} retired work={:?}",
            work.ranked, work.retired
        );
    }
    let ranked_ratio = work[2].ranked as f64 / work[0].ranked as f64;
    // n log n grows 4 * log2(8000) / log2(2000) ~ 4.7 times from 2,000 to
    // 8,000; quadratic growth is 16 times.
    assert!(
        ranked_ratio < 6.0,
        "ranked ordering work grew {ranked_ratio:.2} times from 2,000 to 8,000 records"
    );
    let (Some(retired_small), Some(retired_large)) = (work[0].retired, work[1].retired) else {
        panic!("the retired reference is measured at 2,000 and 4,000 records");
    };
    let retired_ratio = retired_large as f64 / retired_small as f64;
    // Doubling n grows n log n work ~2.2 times and quadratic work 4 times.
    assert!(
        retired_ratio > 3.0,
        "the retired order must be the quadratic one this test rejects: {retired_ratio:.2}"
    );
}

/// A small deterministic generator for the order-equivalence property.
struct Random(u64);

impl Random {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }
}

/// A random structural key over a small alphabet, so equal keys, shared
/// operands and every segment kind occur. A nested operand names an earlier
/// key, or (when `unlisted`) a fresh key the graph does not list.
fn random_key(
    random: &mut Random,
    earlier: &[Arc<CheckedNodeId>],
    unlisted: bool,
) -> CheckedNodeId {
    let segments = 1 + random.below(3);
    let mut path = Vec::with_capacity(segments);
    for _ in 0..segments {
        path.push(match random.below(6) {
            0 => Segment::Name(["a", "b"][random.below(2)].into()),
            1 => Segment::Index(random.below(2) as u64),
            2 => Segment::Operand(random.below(2) as u64),
            3 => Segment::Dependency {
                package: ["d", "e"][random.below(2)].into(),
                node: random.below(2) as u64,
            },
            _ if unlisted && random.below(4) == 0 => Segment::Node(Arc::new(CheckedNodeId {
                package: "p".into(),
                path: vec![Segment::Name("unlisted".into())],
                ordinal: random.below(2) as u64,
            })),
            _ if !earlier.is_empty() => {
                Segment::Node(Arc::clone(&earlier[random.below(earlier.len())]))
            }
            _ => Segment::Name("leaf".into()),
        });
    }
    CheckedNodeId {
        package: ["p", "q"][random.below(2)].into(),
        path,
        ordinal: random.below(2) as u64,
    }
}

/// IR-1588: over random small graphs, ranked and unranked, the record sort,
/// the row-key order and the per-pair key order answer exactly as the
/// retired `CheckedNodeId::cmp` order, so the canonical record order and
/// every refusal it selects are unchanged.
#[trace("TC-488", "FR-376-AC-3")]
#[test]
fn ranked_record_order_equals_the_structural_id_order_on_random_graphs() {
    let mut random = Random(0x1588_0000_2026_1122);
    let (mut ranked_graphs, mut unranked_graphs) = (0, 0);
    for case in 0..400 {
        let unlisted = case % 4 == 3;
        let size = 1 + random.below(24);
        let mut keys: Vec<Arc<CheckedNodeId>> = Vec::with_capacity(size);
        for _ in 0..size {
            let key = random_key(&mut random, &keys, unlisted);
            keys.push(Arc::new(key));
        }
        let wire: Vec<CheckedNodeId> = (0..size).map(wire_id).collect();
        let mut meter = WorkMeter::new(u64::MAX);
        let index = GraphIndex::build(wire.iter(), &mut meter)
            .and_then(|index| index.with_ranks(&keys, &mut meter))
            .unwrap_or_else(|_| panic!("random graph indexes"));
        if index.rank_at(0).is_some() {
            ranked_graphs += 1;
        } else {
            unranked_graphs += 1;
        }

        for left in 0..size {
            for right in 0..size {
                let order = key_order(&index, &keys, left, right, &mut meter)
                    .unwrap_or_else(|_| panic!("key order admits"));
                assert_eq!(
                    order,
                    keys[left].cmp(&keys[right]),
                    "case {case}: {left} vs {right}"
                );
                let (left_ordinal, right_ordinal) =
                    (random.below(2) as u64, random.below(2) as u64);
                let order = row_key_order(
                    &row_key(&index, &keys, left, left_ordinal),
                    &row_key(&index, &keys, right, right_ordinal),
                    &mut meter,
                )
                .unwrap_or_else(|_| panic!("row order admits"));
                let retired = (
                    keys[left].as_ref(),
                    "expression",
                    left_ordinal,
                    "value-validity",
                )
                    .cmp(&(
                        keys[right].as_ref(),
                        "expression",
                        right_ordinal,
                        "value-validity",
                    ));
                assert_eq!(order, retired, "case {case}: rows {left} vs {right}");
            }
        }

        let mut records: Vec<usize> = (0..size).filter(|_| random.below(3) != 0).collect();
        for at in (1..records.len()).rev() {
            records.swap(at, random.below(at + 1));
        }
        let mut retired = records.clone();
        retired.sort_unstable_by(|left, right| keys[*left].cmp(&keys[*right]));
        let mut tie_broken = records.clone();
        tie_broken
            .sort_unstable_by(|left, right| keys[*left].cmp(&keys[*right]).then(left.cmp(right)));
        sort_by_structural_key(&mut records, &index, &keys, &mut meter)
            .unwrap_or_else(|_| panic!("record sort admits"));
        // Random graphs, unlike admitted ones, can hold equal keys at two
        // positions; the key sequence is the retired one either way.
        assert_eq!(records, tie_broken, "case {case}: record order");
        assert!(
            records
                .iter()
                .zip(&retired)
                .all(|(new, old)| keys[*new] == keys[*old]),
            "case {case}: record key sequence"
        );

        // Named records: distinct rows, any record positions, owners named
        // by their wire ids as the validated rows name them.
        let count = random.below(12);
        let mut rows: Vec<usize> = (0..count).collect();
        for at in (1..rows.len()).rev() {
            rows.swap(at, random.below(at + 1));
        }
        let named: Vec<(usize, usize, CheckedNodeId)> = rows
            .into_iter()
            .map(|row| (row, random.below(size), wire[random.below(size)].clone()))
            .collect();
        let mut retired_named = named.clone();
        retired_named.sort_unstable_by(
            |(left_row, left, left_owner), (right_row, right, right_owner)| {
                let left_owner_position = index.get(left_owner).copied().unwrap_or(usize::MAX);
                let right_owner_position = index.get(right_owner).copied().unwrap_or(usize::MAX);
                keys[left_owner_position]
                    .cmp(&keys[right_owner_position])
                    .then_with(|| left_row.cmp(right_row))
                    .then_with(|| keys[*left].cmp(&keys[*right]))
            },
        );
        let order = named_record_order(&named, &index, &keys, &mut meter)
            .unwrap_or_else(|_| panic!("named-record order admits"));
        let ordered: Vec<(usize, usize, CheckedNodeId)> =
            order.iter().map(|&at| named[at].clone()).collect();
        assert_eq!(ordered, retired_named, "case {case}: named-record order");
    }
    assert!(ranked_graphs > 0 && unranked_graphs > 0);
}

/// IR-1588: on an unranked graph the record sort compares through the
/// meter. A read budget that runs out at any point of the sort, the scratch
/// block or any comparison's frame list, returns the typed read-storage
/// stop, which the budgeted entry reports as `incomplete`; the sort never
/// panics and never answers a comparison after a refusal.
#[trace("TC-488", "FR-376-AC-3")]
#[test]
fn record_sort_on_an_unranked_graph_stops_typed_wherever_the_budget_runs_out() {
    use super::super::{BudgetStop, NativeJoinBudget, PackageReadResource};
    use crate::checked_package::common::ValidationFailure;

    // Records nest copies of their applications, which are not themselves
    // listed keys, so the graph is unranked and comparisons walk chains.
    let length = 64;
    let mut keys: Vec<Arc<CheckedNodeId>> = Vec::new();
    let mut application = Arc::new(CheckedNodeId {
        package: "p".into(),
        path: vec![Segment::Name("x".into())],
        ordinal: 0,
    });
    for _ in 0..length {
        application = Arc::new(CheckedNodeId {
            package: "p".into(),
            path: vec![
                Segment::Name("expression".into()),
                Segment::Node(Arc::clone(&application)),
            ],
            ordinal: 0,
        });
        keys.push(Arc::new(CheckedNodeId {
            package: "p".into(),
            path: vec![
                Segment::Name("claim".into()),
                Segment::Node(Arc::new(CheckedNodeId::clone(&application))),
            ],
            ordinal: 0,
        }));
    }
    let wire: Vec<CheckedNodeId> = (0..keys.len()).map(wire_id).collect();
    let mut own = WorkMeter::new(u64::MAX);
    let index = GraphIndex::build(wire.iter(), &mut own)
        .and_then(|index| index.with_ranks(&keys, &mut own))
        .unwrap_or_else(|_| panic!("unranked graph indexes"));
    assert!(index.rank_at(0).is_none(), "the graph must be unranked");
    let input = scrambled(&(0..keys.len()).collect::<Vec<_>>());

    let sort = |limit: u64| {
        let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
        let mut records = input.clone();
        let result = {
            let mut meter = WorkMeter::over(&mut budget);
            sort_by_structural_key(&mut records, &index, &keys, &mut meter)
        };
        (result, records, budget.read_storage_consumed())
    };
    let (admitted, sorted, required) = sort(u64::MAX);
    assert!(admitted.is_ok());
    let mut retired = input.clone();
    retired.sort_unstable_by(|left, right| keys[*left].cmp(&keys[*right]));
    assert_eq!(sorted, retired);
    let scratch = (keys.len() * std::mem::size_of::<usize>()) as u64;
    assert!(
        required > 2 * scratch,
        "comparisons charge frame lists beyond the scratch block: {required}"
    );
    let cuts = [
        0,
        scratch - 1,
        scratch,
        scratch + 1,
        required / 2,
        required - 1,
    ];
    for limit in cuts {
        let (result, _, _) = sort(limit);
        match result {
            Err(ValidationFailure::Budget(BudgetStop {
                resource: PackageReadResource::ReadStorage,
                ..
            })) => {}
            other => panic!("budget {limit} of {required}: {other:?}"),
        }
    }
}
