//! Caller-supplied definition identities used to bind strict package reads.

use super::shared::CheckedArtifactRef;
use ix_trace_rs::implements;
use std::collections::BTreeSet;

/// Definitions available to a catalog-bound checked-package read.
///
/// Profile, extension and other known selection definitions may appear in
/// different lock positions. The reader uses the distinction to reject a
/// profile placed in `definition_selections`; operation-law definitions
/// remain governed by the reader's operation catalog. A reference inserted
/// as both profile and extension is treated as a profile, regardless of
/// insertion order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DefinitionCatalog {
    profiles: BTreeSet<CheckedArtifactRef>,
    extensions: BTreeSet<CheckedArtifactRef>,
    selections: BTreeSet<CheckedArtifactRef>,
}

impl DefinitionCatalog {
    /// Creates an empty definition catalog.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a profile definition known by the caller's catalog.
    pub fn insert_profile(&mut self, definition: CheckedArtifactRef) {
        self.extensions.remove(&definition);
        self.selections.remove(&definition);
        self.profiles.insert(definition);
    }

    /// Adds an extension definition known by the caller's catalog.
    pub fn insert_extension(&mut self, definition: CheckedArtifactRef) {
        if !self.profiles.contains(&definition) {
            self.selections.remove(&definition);
            self.extensions.insert(definition);
        }
    }

    /// Adds a known definition that is valid in `lock.definition_selections`
    /// but is neither a caller-supplied profile nor an extension.
    #[implements("FR-038-AC-495")]
    pub fn insert_selection(&mut self, definition: CheckedArtifactRef) {
        self.profiles.remove(&definition);
        self.extensions.remove(&definition);
        self.selections.insert(definition);
    }

    pub(super) fn is_profile(&self, definition: &CheckedArtifactRef) -> bool {
        self.profiles.contains(definition)
    }

    pub(super) fn is_extension(&self, definition: &CheckedArtifactRef) -> bool {
        self.extensions.contains(definition)
    }

    pub(super) fn is_selection(&self, definition: &CheckedArtifactRef) -> bool {
        self.selections.contains(definition)
    }
}
