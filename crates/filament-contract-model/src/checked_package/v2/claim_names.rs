// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Declared names of state-clause, refinement and non-temporal claim
//! declarations.

use super::declaration_region::SourceEntries;
use super::graph_storage::GraphIndex;
use super::stage_storage::{self, Path, Refusal, Site};
use super::temporal_clause_names::TemporalClauseName;
use super::{
    identity::is_identifier, CheckedNodeId, CheckedNodeKind, CheckedOccurrence,
    CheckedOccurrenceRole, CheckedTerm, RawSemanticNodeV2, RelationForm, StateForm, WorkMeter,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
};
use quire_canonical::{Encode, Error, Sink, Writer};
use serde::{Deserialize, Serialize};

/// One declared name of a state-clause, refinement or claim declaration.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimNameRow {
    /// Local structural identity of the claim-bearing node.
    pub declaration: CheckedNodeId,
    /// The `declaration` or `claim` occurrence this row names.
    pub occurrence: CheckedOccurrence,
    /// Exact source identifier, independent of the node identity.
    pub name: Box<str>,
    /// Alias-free package-local path whose last segment is `name`.
    pub qualified_name: Vec<Box<str>>,
    /// The whole source declaration that introduced the occurrence, under the
    /// source of the occurrence's `source_map` region and containing it.
    pub declaration_region: crate::checked_package::shared::CheckedSourceRegion,
}

impl Encode for ClaimNameRow {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        super::encode::encoded(writer, "declaration", &self.declaration)?;
        super::encode::member(writer, "occurrence", &self.occurrence)?;
        super::encode::member(writer, "name", self.name.as_ref())?;
        writer.name("qualified_name")?;
        writer.begin_array()?;
        for segment in &self.qualified_name {
            writer.string(segment)?;
        }
        writer.end_array()?;
        super::encode::member(writer, "declaration_region", &self.declaration_region)?;
        writer.end_object()
    }
}

fn root() -> Path<'static> {
    Path::root().key("claim_names")
}

fn site(index: usize, path: Path<'_>) -> Site<'_> {
    Site::new(super::PackageReadStage::Graph, index, "claim_names", path)
}

fn is_claim_bearing(kind: CheckedNodeKind) -> bool {
    matches!(
        kind,
        CheckedNodeKind::State(StateForm::StateClause)
            | CheckedNodeKind::Relation(RelationForm::Refinement)
            | CheckedNodeKind::Claim(_)
    )
}

fn is_named_role(role: &CheckedOccurrenceRole) -> bool {
    matches!(
        role,
        CheckedOccurrenceRole::Declaration | CheckedOccurrenceRole::Claim
    )
}

fn refuse_row(
    code: Code,
    cause: Cause,
    at: Path<'_>,
    declaration: &CheckedNodeId,
    index: usize,
    meter: &mut WorkMeter,
) -> ValidationFailure {
    stage_storage::refuse(
        meter,
        &site(index, at),
        Refusal::new(code, at).because(cause).at_node(declaration),
    )
}

fn invalid(
    at: Path<'_>,
    declaration: &CheckedNodeId,
    index: usize,
    meter: &mut WorkMeter,
) -> ValidationFailure {
    refuse_row(
        Code::InvalidPackage,
        Cause::InvalidValue,
        at,
        declaration,
        index,
        meter,
    )
}

fn malformed(index: usize, at: Path<'_>, meter: &mut WorkMeter) -> ValidationFailure {
    stage_storage::refuse(
        meter,
        &site(index, at),
        Refusal::new(Code::MalformedWire, at),
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn validate(
    rows: Option<&[ClaimNameRow]>,
    temporal: Option<&[TemporalClauseName]>,
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    sources: &SourceEntries<'_>,
    index: &GraphIndex<'_>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let capacity = rows.map_or(0, <[ClaimNameRow]>::len);
    let mut seen: Vec<(&CheckedNodeId, &CheckedOccurrence)> = Vec::new();
    let mut names: Vec<(&[Box<str>], usize)> = Vec::new();
    let mut previous: Option<(super::graph_storage::ResolvedKey<'_>, &CheckedOccurrence)> = None;
    // The closed schema is checked across every row before the reader visits
    // any row, so a malformed later row wins over a dangling earlier one.
    // Each row costs one validation visit here.
    for (row_index, row) in rows.unwrap_or_default().iter().enumerate() {
        let at = root().index(row_index);
        stage_storage::work(meter, &site(row_index, at), 1)?;
        if !is_identifier(&row.name) {
            return Err(malformed(row_index, at.key("name"), meter));
        }
        if row.qualified_name.is_empty() || !row.qualified_name.iter().all(|s| is_identifier(s)) {
            return Err(malformed(row_index, at.key("qualified_name"), meter));
        }
        if !is_named_role(&row.occurrence.role) {
            return Err(malformed(
                row_index,
                at.key("occurrence").key("role"),
                meter,
            ));
        }
    }
    for (row_index, row) in rows.unwrap_or_default().iter().enumerate() {
        let at = root().index(row_index);
        let Some(&position) = index
            .get(&row.declaration)
            .filter(|&&position| is_claim_bearing(kinds[position]))
        else {
            return Err(invalid(
                at.key("declaration"),
                &row.declaration,
                row_index,
                meter,
            ));
        };
        let mut owned = false;
        for occurrence in &nodes[position].occurrences {
            stage_storage::work(meter, &site(row_index, at.key("occurrence")), 1)?;
            if occurrence == &row.occurrence {
                owned = true;
                break;
            }
        }
        if !owned {
            return Err(invalid(
                at.key("occurrence"),
                &row.declaration,
                row_index,
                meter,
            ));
        }
        if !sources.covers(
            &row.declaration_region,
            &row.declaration,
            &row.occurrence.role,
            row.occurrence.ordinal,
            &site(row_index, at.key("declaration_region")),
            meter,
        )? {
            return Err(invalid(
                at.key("declaration_region"),
                &row.declaration,
                row_index,
                meter,
            ));
        }
        if row.qualified_name.last() != Some(&row.name) {
            return Err(invalid(
                at.key("qualified_name"),
                &row.declaration,
                row_index,
                meter,
            ));
        }
        if kinds[position] == CheckedNodeKind::Relation(RelationForm::Refinement) {
            // A body the typed decode refused is reported by the refinement
            // step that owns it, not by this row.
            if let Some(CheckedTerm::Refinement { name, .. }) = terms[position].as_ref() {
                if name != &row.name {
                    return Err(invalid(at.key("name"), &row.declaration, row_index, meter));
                }
            }
        }
        let key = (&row.declaration, &row.occurrence);
        match seen.binary_search(&key) {
            Ok(_) => {
                return Err(refuse_row(
                    Code::InvalidPackage,
                    Cause::ConflictingDefinition,
                    at,
                    &row.declaration,
                    row_index,
                    meter,
                ))
            }
            Err(position) => {
                if seen.is_empty() {
                    seen = stage_storage::vec(meter, &site(row_index, at), capacity)?;
                }
                seen.insert(position, key);
            }
        }
        let ordered = (
            index.resolved_at(position, structural_keys),
            &row.occurrence,
        );
        if previous
            .as_ref()
            .is_some_and(|previous| previous > &ordered)
        {
            return Err(invalid(at, &row.declaration, row_index, meter));
        }
        previous = Some(ordered);
        let path: &[Box<str>] = &row.qualified_name;
        match names.binary_search_by(|(earlier, _)| (*earlier).cmp(path)) {
            Ok(found) => {
                // The refusal names both rows: the later one is its locus and
                // the earlier one the package-document position.
                let earlier = names[found].1;
                return Err(stage_storage::refuse(
                    meter,
                    &site(row_index, at),
                    Refusal::new(Code::AmbiguousDeclaration, at.key("qualified_name"))
                        .because(Cause::AmbiguousName)
                        .at_node(&row.declaration)
                        .in_document(root().index(earlier).key("qualified_name")),
                ));
            }
            Err(position) => {
                if names.is_empty() {
                    names = stage_storage::vec(meter, &site(row_index, at), capacity)?;
                }
                names.insert(position, (path, row_index));
            }
        }
    }
    for (position, node) in nodes.iter().enumerate() {
        if !is_claim_bearing(kinds[position]) {
            continue;
        }
        for occurrence in &node.occurrences {
            stage_storage::work(meter, &site(position, root()), 1)?;
            if is_named_role(&occurrence.role)
                && seen.binary_search(&(&node.node_id, occurrence)).is_err()
            {
                return Err(stage_storage::refuse(
                    meter,
                    &site(position, root()),
                    Refusal::new(Code::InvalidPackage, root()).because(Cause::InvalidValue),
                ));
            }
        }
    }
    one_namespace(
        rows.unwrap_or_default(),
        temporal.unwrap_or_default(),
        meter,
    )
}

/// Two refinement nodes of one package may not share a `name`
/// (FR-322-AC-200): the later node, in graph order, refuses
/// `ambiguous_declaration`/`ambiguous-name` at its `body/name`, and the refusal
/// names the earlier node's `body/name` as well. The pass walks nodes the
/// graph stage already charged, and charges the storage it keeps.
pub(super) fn validate_refinement_names(
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    nodes: &[RawSemanticNodeV2],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut taken: Vec<(&str, usize)> = Vec::new();
    for (position, kind) in kinds.iter().enumerate() {
        if *kind != CheckedNodeKind::Relation(RelationForm::Refinement) {
            continue;
        }
        let at = Path::root()
            .key("semantic_graph")
            .key("nodes")
            .index(position)
            .key("body")
            .key("name");
        let refinement_site = || {
            Site::new(
                super::PackageReadStage::Graph,
                position,
                "refinement_names",
                at,
            )
        };
        let Some(CheckedTerm::Refinement { name, .. }) = terms[position].as_ref() else {
            continue;
        };
        let name: &str = name;
        match taken.binary_search_by(|(earlier, _)| (*earlier).cmp(name)) {
            Ok(found) => {
                let earlier = taken[found].1;
                return Err(stage_storage::refuse(
                    meter,
                    &refinement_site(),
                    Refusal::new(Code::AmbiguousDeclaration, at)
                        .because(Cause::AmbiguousName)
                        .at_node(&nodes[position].node_id)
                        .in_document(
                            Path::root()
                                .key("semantic_graph")
                                .key("nodes")
                                .index(earlier)
                                .key("body")
                                .key("name"),
                        ),
                ));
            }
            Err(slot) => {
                if taken.is_empty() {
                    taken = stage_storage::vec(meter, &refinement_site(), kinds.len())?;
                }
                taken.insert(slot, (name, position));
            }
        }
    }
    Ok(())
}

/// `claim_names` and `temporal_clause_names` are two tables of one namespace
/// (FR-322-AC-459): once both have passed their own row checks, the first
/// `claim_names` row whose `name` equals a `temporal_clause_names` row's `name`
/// is ambiguous, and the refusal names both rows. Each visited row costs one
/// validation visit.
fn one_namespace(
    rows: &[ClaimNameRow],
    temporal: &[TemporalClauseName],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    if rows.is_empty() || temporal.is_empty() {
        return Ok(());
    }
    let mut taken: Vec<(&str, usize)> =
        stage_storage::vec(meter, &site(0, root()), temporal.len())?;
    for (temporal_index, row) in temporal.iter().enumerate() {
        let name: &str = &row.name;
        if let Err(position) = taken.binary_search_by(|(earlier, _)| (*earlier).cmp(name)) {
            taken.insert(position, (name, temporal_index));
        }
    }
    for (row_index, row) in rows.iter().enumerate() {
        let at = root().index(row_index);
        stage_storage::work(meter, &site(row_index, at.key("name")), 1)?;
        let name: &str = &row.name;
        if let Ok(found) = taken.binary_search_by(|(earlier, _)| (*earlier).cmp(name)) {
            let first = taken[found].1;
            return Err(stage_storage::refuse(
                meter,
                &site(row_index, at.key("name")),
                Refusal::new(Code::AmbiguousDeclaration, at.key("name"))
                    .because(Cause::AmbiguousName)
                    .at_node(&row.declaration)
                    .in_document(
                        Path::root()
                            .key("temporal_clause_names")
                            .index(first)
                            .key("name"),
                    ),
            ));
        }
    }
    Ok(())
}
