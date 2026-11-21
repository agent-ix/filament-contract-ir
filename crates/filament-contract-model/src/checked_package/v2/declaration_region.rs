// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The declaration region a name row carries (FSpec FR-322-AC-460 and
//! FR-322-AC-461): one source interval covering the whole declaration, under
//! the source of the occurrence's own `source_map` region and containing it.

use super::stage_storage::{self, Site};
use super::{CheckedNodeId, CheckedOccurrenceRole, WorkMeter};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{CheckedSourceMapEntry, CheckedSourceRegion};

pub(super) type Key<'a> = (&'a CheckedNodeId, &'a CheckedOccurrenceRole, u64);

/// The source-map entries of a package by (node, role, ordinal), ascending.
pub(super) struct SourceEntries<'a> {
    keys: Vec<(Key<'a>, usize)>,
    entries: &'a [CheckedSourceMapEntry],
}

impl<'a> SourceEntries<'a> {
    /// Indexes `entries`, charging the table at `site` first. A package with
    /// no name row needs no table, and none is built.
    pub(super) fn build(
        entries: &'a [CheckedSourceMapEntry],
        needed: bool,
        site: &Site<'_>,
        meter: &mut WorkMeter,
    ) -> Result<Self, ValidationFailure> {
        let mut keys: Vec<(Key<'a>, usize)> = Vec::new();
        if needed {
            keys = stage_storage::vec(meter, site, entries.len())?;
            keys.extend(
                entries.iter().enumerate().map(|(position, entry)| {
                    ((&entry.node_id, &entry.role, entry.ordinal), position)
                }),
            );
            keys.sort_unstable();
        }
        Ok(Self { keys, entries })
    }

    fn entry(
        &self,
        id: &CheckedNodeId,
        role: &CheckedOccurrenceRole,
        ordinal: u64,
    ) -> Option<&'a CheckedSourceMapEntry> {
        let start = self
            .keys
            .partition_point(|(key, _)| *key < (id, role, ordinal));
        self.keys
            .get(start)
            .filter(|(key, _)| *key == (id, role, ordinal))
            .map(|(_, position)| &self.entries[*position])
    }

    /// Whether `region` is under the source of every region of the occurrence's
    /// `source_map` entry and contains every byte of each. An occurrence with
    /// no entry is left to the package-wide source-map check, which refuses it.
    /// Each region compared costs one validation visit at `site`.
    pub(super) fn covers(
        &self,
        region: &CheckedSourceRegion,
        id: &CheckedNodeId,
        role: &CheckedOccurrenceRole,
        ordinal: u64,
        site: &Site<'_>,
        meter: &mut WorkMeter,
    ) -> Result<bool, ValidationFailure> {
        let Some(entry) = self.entry(id, role, ordinal) else {
            return Ok(true);
        };
        for held in &entry.regions {
            stage_storage::work(meter, site, 1)?;
            if held.source != region.source || held.start < region.start || region.end < held.end {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Whether the occurrence's `source_map` entry holds exactly the one region
    /// `region`. An occurrence with no entry, or an entry with no region, is
    /// left to the package-wide source-map check, which refuses it. Each region
    /// compared costs one validation visit at `site`.
    pub(super) fn is_exactly(
        &self,
        region: &CheckedSourceRegion,
        id: &CheckedNodeId,
        role: &CheckedOccurrenceRole,
        ordinal: u64,
        site: &Site<'_>,
        meter: &mut WorkMeter,
    ) -> Result<bool, ValidationFailure> {
        let Some(entry) = self.entry(id, role, ordinal) else {
            return Ok(true);
        };
        if entry.regions.is_empty() {
            return Ok(true);
        }
        for held in &entry.regions {
            stage_storage::work(meter, site, 1)?;
            if held != region {
                return Ok(false);
            }
        }
        Ok(entry.regions.len() == 1)
    }

    /// The `source_map` entries of node `id`, by role then ordinal: each key's
    /// position in the package's entry array.
    pub(super) fn of_node(&self, id: &CheckedNodeId) -> &[(Key<'a>, usize)] {
        let start = self.keys.partition_point(|((held, _, _), _)| *held < id);
        let end = start + self.keys[start..].partition_point(|((held, _, _), _)| *held == id);
        &self.keys[start..end]
    }

    /// The entry at `position` of the package's `source_map` array.
    pub(super) fn at(&self, position: usize) -> &'a CheckedSourceMapEntry {
        &self.entries[position]
    }
}
