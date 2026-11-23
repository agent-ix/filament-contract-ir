// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Work and order of the requirement-record structural-key orderings.

use super::{
    key_order, row_key_order, sort_by_structural_key, sort_charged, GraphIndex, RowKey, WorkMeter,
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
        key_order(&index, &chain.keys, *left, *right, meter)
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
        sort_by_structural_key(&mut records, &index, &keys, &mut meter)
            .unwrap_or_else(|_| panic!("record sort admits"));
        assert_eq!(records, retired, "case {case}: record order");
    }
    assert!(ranked_graphs > 0 && unranked_graphs > 0);
}
