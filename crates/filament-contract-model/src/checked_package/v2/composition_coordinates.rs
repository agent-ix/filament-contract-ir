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
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedSourceRegion,
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

/// Checks the entries of composition node `id`, whose body holds `members`
/// members, against the coordinates the module documentation fixes.
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
    // The regions of the member entries passed so far, disjoint and ascending.
    let mut earlier: Vec<(u64, u64)> = stage_storage::vec(meter, &site(root), members)?;
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
            (CheckedOccurrenceRole::Generated, ordinal)
                if usize::try_from(ordinal).is_ok_and(|ordinal| ordinal < members) =>
            {
                // Without the declaration region no member can be judged; the
                // missing declaration is refused below.
                if let Some(declaration) = declaration {
                    if !strictly_inside(region, declaration) {
                        return Err(refuse(at, id, meter));
                    }
                    let slot = earlier.partition_point(|(start, _)| *start < region.start);
                    let touches_before = slot > 0 && earlier[slot - 1].1 > region.start;
                    let touches_after = earlier
                        .get(slot)
                        .is_some_and(|(start, _)| *start < region.end);
                    if touches_before || touches_after {
                        return Err(refuse(at, id, meter));
                    }
                    earlier.insert(slot, (region.start, region.end));
                }
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
