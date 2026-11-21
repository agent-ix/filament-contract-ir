// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The member coordinates of a `correspondence`/`component_composition` node
//! (FSpec FR-491 "Composition member occurrences"; Contract IR FR-038).
//!
//! A composition carries one `declaration` occurrence at ordinal 0, whose
//! `source_map` entry holds the whole declaration as one region, and one
//! `generated` occurrence per body member, ordinal = the member's position in
//! the canonical concatenation of its body arrays. Each member entry holds
//! exactly one region, under the declaration's source, strictly inside the
//! declaration region and disjoint from every earlier member entry's region.
//!
//! The node's entries are examined in `source_map` order and the first
//! invalid one is refused as `invalid_package`/`invalid-value` at
//! `/source_map/{i}`; when every entry passes, a missing required occurrence
//! is refused at `/source_map`. The reader runs this after FR-322's generic
//! source-map checks. It reads only the admitted regions: no source text is
//! parsed and no member inherits the declaration's region.

use super::declaration_region::SourceEntries;
use super::native_join_budget::PackageReadStage;
use super::stage_storage::{self, Path, Refusal, Site};
use super::{CheckedNodeId, CheckedOccurrenceRole, WorkMeter};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedSourceMapEntry, CheckedSourceRef,
    CheckedSourceRegion,
};

fn site(at: Path<'static>) -> Site<'static> {
    Site::new(PackageReadStage::Graph, 0, "composition_coordinates", at)
}

fn refuse(at: Path<'static>, id: &CheckedNodeId, meter: &mut WorkMeter) -> ValidationFailure {
    stage_storage::refuse(
        meter,
        &site(at),
        Refusal::new(CheckedPackageRefusalCode::InvalidPackage, at)
            .because(CheckedPackageRefusalCause::InvalidValue)
            .at_node(id),
    )
}

/// Whether `member` lies strictly inside `declaration`, under its source.
fn strictly_inside(member: &CheckedSourceRegion, declaration: &CheckedSourceRegion) -> bool {
    member.source == declaration.source
        && member.start < member.end
        && declaration.start < member.start
        && member.end < declaration.end
}

/// One member entry's region: source, start, end and the entry's position.
type Candidate<'a> = (&'a CheckedSourceRef, u64, u64, usize);

/// A Fenwick tree over the candidates' ranks, counting the accepted ones. Its
/// table is charged once, at construction; each operation walks at most
/// `steps` table cells.
struct Accepted {
    tree: Vec<u32>,
    total: u32,
    steps: u64,
}

impl Accepted {
    fn new(candidates: usize, meter: &mut WorkMeter) -> Result<Self, ValidationFailure> {
        let mut tree: Vec<u32> =
            stage_storage::vec(meter, &site(Path::root().key("source_map")), candidates + 1)?;
        tree.resize(candidates + 1, 0);
        let steps = u64::from(usize::BITS - candidates.leading_zeros()) + 1;
        Ok(Self {
            tree,
            total: 0,
            steps,
        })
    }

    /// Accepted ranks at or below `rank` (1-based).
    fn prefix(&self, mut rank: usize) -> u32 {
        let mut count = 0;
        while rank > 0 {
            count += self.tree[rank];
            rank &= rank - 1;
        }
        count
    }

    fn insert(&mut self, mut rank: usize) {
        self.total += 1;
        while rank < self.tree.len() {
            self.tree[rank] += 1;
            rank += rank & rank.wrapping_neg();
        }
    }

    /// The rank of the `k`-th accepted entry (1-based `k`, at most `total`).
    fn nth(&self, mut k: u32) -> usize {
        let mut at = 0;
        let mut bit = self.tree.len().next_power_of_two();
        while bit > 0 {
            let next = at + bit;
            if next < self.tree.len() && self.tree[next] < k {
                at = next;
                k -= self.tree[next];
            }
            bit >>= 1;
        }
        at + 1
    }

    /// The nearest accepted ranks below and above `rank`, if any.
    fn neighbours(&self, rank: usize) -> (Option<usize>, Option<usize>) {
        let below = self.prefix(rank - 1);
        let through = self.prefix(rank);
        (
            (below > 0).then(|| self.nth(below)),
            (through < self.total).then(|| self.nth(through + 1)),
        )
    }
}

/// Checks the entries of composition node `id`, whose body holds `members`
/// members, against the coordinates the module documentation fixes.
///
/// Disjointness is judged against the accepted earlier entries through the
/// candidates' sorted ranks and a charged Fenwick tree, so a package whose
/// entries list the members in any order pays `O(n log n)` work and one
/// charged table per structure; no step moves a shifted tail.
pub(super) fn validate(
    id: &CheckedNodeId,
    members: usize,
    sources: &SourceEntries<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let root = Path::root().key("source_map");
    let run = sources.of_node(id);
    // The node's entries in `source_map` order.
    let mut order: Vec<usize> = stage_storage::vec(meter, &site(root), run.len())?;
    order.extend(run.iter().map(|(_, position)| *position));
    order.sort_unstable();
    let declaration = run
        .iter()
        .find(|((_, role, ordinal), _)| {
            **role == CheckedOccurrenceRole::Declaration && *ordinal == 0
        })
        .map(|(_, position)| sources.at(*position))
        .and_then(|entry| match entry.regions.as_slice() {
            [region] => Some(region),
            _ => None,
        });
    let is_member = |entry: &CheckedSourceMapEntry| {
        entry.role == CheckedOccurrenceRole::Generated
            && usize::try_from(entry.ordinal).is_ok_and(|ordinal| ordinal < members)
    };
    // The member entries holding one region, by source then interval.
    let mut candidates: Vec<Candidate<'_>> = stage_storage::vec(meter, &site(root), run.len())?;
    for (_, position) in run {
        let entry = sources.at(*position);
        if let (true, [region]) = (is_member(entry), entry.regions.as_slice()) {
            candidates.push((&region.source, region.start, region.end, *position));
        }
    }
    candidates.sort_unstable();
    let mut accepted = Accepted::new(candidates.len(), meter)?;
    let mut passed = 0_usize;
    for position in order {
        let at = root.index(position);
        stage_storage::work(meter, &site(at), 1)?;
        let entry = sources.at(position);
        let [region] = entry.regions.as_slice() else {
            return Err(refuse(at, id, meter));
        };
        match (&entry.role, entry.ordinal) {
            (CheckedOccurrenceRole::Declaration, 0) => {}
            _ if is_member(entry) => {
                // Without the declaration region the source and the interval
                // cannot be judged; the missing declaration is refused below.
                if declaration.is_some_and(|declaration| !strictly_inside(region, declaration)) {
                    return Err(refuse(at, id, meter));
                }
                // One search, two prefix sums and two selections, then one
                // insertion, each walking at most `steps` cells.
                stage_storage::work(meter, &site(at), 6 * accepted.steps)?;
                let key = (&region.source, region.start, region.end, position);
                let rank = candidates
                    .binary_search(&key)
                    .map_or_else(|slot| slot, |found| found)
                    + 1;
                let (below, above) = accepted.neighbours(rank);
                let touches_below = below.is_some_and(|rank| {
                    let (source, _, end, _) = candidates[rank - 1];
                    source == &region.source && end > region.start
                });
                let touches_above = above.is_some_and(|rank| {
                    let (source, start, _, _) = candidates[rank - 1];
                    source == &region.source && start < region.end
                });
                if touches_below || touches_above {
                    return Err(refuse(at, id, meter));
                }
                accepted.insert(rank);
            }
            _ => return Err(refuse(at, id, meter)),
        }
        passed += 1;
    }
    // Every entry is one of the declaration and member keys, once each, so a
    // short count is a missing required occurrence.
    if passed != members + 1 || declaration.is_none() {
        return Err(refuse(root, id, meter));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::native_join_budget::NativeJoinBudget;
    use super::super::test_support::{id, node_id};
    use super::*;
    use crate::checked_package::shared::CheckedSourceMapEntry;
    use ix_trace_rs::trace;
    use serde_json::{json, Value};

    const MEMBERS: usize = 64;

    fn region(start: u64, end: u64) -> Value {
        json!({"source": {"authority": "agent-ix", "identity": "example"},
            "start": start, "end": end})
    }

    /// The declaration entry, then `MEMBERS` member entries listed by
    /// descending start: the order that moves the most under an insertion.
    fn descending(extra: Option<(u64, u64)>) -> Vec<CheckedSourceMapEntry> {
        let mut entries = vec![json!({"node_id": id('a'), "role": "declaration",
            "ordinal": 0, "regions": [region(1, 1_000_000)]})];
        for member in 0..MEMBERS as u64 {
            let start = 900_000 - 1_000 * member;
            entries.push(json!({"node_id": id('a'), "role": "generated",
                "ordinal": member, "regions": [region(start, start + 10)]}));
        }
        if let Some((start, end)) = extra {
            // The last entry repeats the first member's slot and overlaps it.
            entries.push(json!({"node_id": id('a'), "role": "generated",
                "ordinal": MEMBERS as u64, "regions": [region(start, end)]}));
        }
        serde_json::from_value(Value::Array(entries)).expect("entries")
    }

    fn run(
        entries: &[CheckedSourceMapEntry],
        members: usize,
        work_limit: u64,
        storage_limit: u64,
    ) -> (Result<(), ValidationFailure>, NativeJoinBudget) {
        let mut index = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(u64::MAX);
        let table = SourceEntries::build(
            entries,
            true,
            &Site::new(PackageReadStage::Graph, 0, "test", Path::root()),
            &mut WorkMeter::over(&mut index),
        )
        .expect("index");
        let mut budget =
            NativeJoinBudget::new(work_limit, 0).with_read_storage_limit(storage_limit);
        let result = validate(
            &node_id('a'),
            members,
            &table,
            &mut WorkMeter::over(&mut budget),
        );
        (result, budget)
    }

    /// The tables the check holds, from their element sizes: the entry
    /// positions, the member candidates and the Fenwick tree of ranks.
    fn expected_storage(entries: usize, candidates: usize) -> u64 {
        (entries * size_of::<usize>()
            + entries * size_of::<(&CheckedSourceRef, u64, u64, usize)>()
            + (candidates + 1) * size_of::<u32>()) as u64
    }

    #[trace("TC-474", "FR-038-AC-611")]
    #[test]
    fn descending_member_entries_are_charged_exactly_and_refuse_when_one_short() {
        let entries = descending(None);
        // 1 declaration + 64 members; every member is a candidate.
        let storage = expected_storage(entries.len(), MEMBERS);
        // One unit per entry, plus per member one search, two prefix sums, two
        // selections and one insertion of at most `log2(n) + 2` cells each.
        let steps = u64::from(MEMBERS.ilog2()) + 2;
        let work = entries.len() as u64 + MEMBERS as u64 * 6 * steps;
        let (result, budget) = run(&entries, MEMBERS, u64::MAX, storage);
        result.expect("a package whose members descend is valid");
        assert_eq!(budget.read_storage_consumed(), storage);
        assert_eq!(budget.consumed().work, work);
        // One byte of storage short is refused; so is one unit of work.
        let (short_storage, _) = run(&entries, MEMBERS, u64::MAX, storage - 1);
        assert!(matches!(short_storage, Err(ValidationFailure::Budget(_))));
        let (short_work, _) = run(&entries, MEMBERS, work - 1, storage);
        assert!(matches!(short_work, Err(ValidationFailure::Budget(_))));
    }

    #[trace("TC-474", "FR-038-AC-611")]
    #[test]
    fn an_overlap_among_descending_entries_refuses_at_the_first_overlapping_entry() {
        // The extra entry has ordinal 64 against 64 members: out of range, so
        // use a 65-member count and overlap the slot of member 3.
        let start = 900_000 - 1_000 * 3 + 5;
        let entries = descending(Some((start, start + 10)));
        let (result, _) = run(&entries, MEMBERS + 1, u64::MAX, u64::MAX);
        let Err(ValidationFailure::Refused(refusal)) = result else {
            panic!("the overlap refuses");
        };
        assert_eq!(
            refusal.primary_path().map(|path| path.as_str().to_owned()),
            Some(format!("/source_map/{}", entries.len() - 1))
        );
        // The same entries without the overlapping one admit.
        let plain = descending(None);
        run(&plain, MEMBERS, u64::MAX, u64::MAX)
            .0
            .expect("no overlap");
    }

    fn member(ordinal: u64, start: u64, end: u64) -> Value {
        json!({"node_id": id('a'), "role": "generated", "ordinal": ordinal,
            "regions": [region(start, end)]})
    }

    fn entries_of(entries: Vec<Value>) -> Vec<CheckedSourceMapEntry> {
        serde_json::from_value(Value::Array(entries)).expect("entries")
    }

    fn refused_at(result: Result<(), ValidationFailure>, expected: &str) {
        let Err(ValidationFailure::Refused(refusal)) = result else {
            panic!("expected a refusal at {expected}");
        };
        assert_eq!(
            refusal.primary_path().map(|path| path.as_str().to_owned()),
            Some(expected.to_owned())
        );
    }

    #[trace("TC-474", "FR-038-AC-613")]
    #[test]
    fn an_absent_declaration_still_refuses_an_overlap_at_its_entry_and_a_gap_at_the_map() {
        // Without the declaration entry the interval cannot be judged, but
        // two member regions that overlap are still refused at the later one.
        let overlapping = entries_of(vec![member(0, 100, 200), member(1, 150, 250)]);
        refused_at(run(&overlapping, 2, u64::MAX, u64::MAX).0, "/source_map/1");
        // Disjoint members pass entry by entry; the missing declaration is
        // the whole `/source_map`.
        let disjoint = entries_of(vec![member(0, 100, 200), member(1, 200, 300)]);
        refused_at(run(&disjoint, 2, u64::MAX, u64::MAX).0, "/source_map");
    }
}
