// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Declared names of state-clause, refinement and non-temporal claim
//! declarations.

use super::graph_storage::GraphIndex;
use super::stage_storage::{self, Path, Refusal, Site};
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

pub(super) fn validate(
    rows: Option<&[ClaimNameRow]>,
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &GraphIndex<'_>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let capacity = rows.map_or(0, <[ClaimNameRow]>::len);
    let mut seen: Vec<(&CheckedNodeId, &CheckedOccurrence)> = Vec::new();
    let mut names: Vec<&[Box<str>]> = Vec::new();
    let mut previous: Option<(super::graph_storage::ResolvedKey<'_>, &CheckedOccurrence)> = None;
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
                if name.as_deref() != Some(row.name.as_ref()) {
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
        match names.binary_search(&path) {
            Ok(_) => {
                return Err(refuse_row(
                    Code::AmbiguousDeclaration,
                    Cause::AmbiguousName,
                    at.key("qualified_name"),
                    &row.declaration,
                    row_index,
                    meter,
                ))
            }
            Err(position) => {
                if names.is_empty() {
                    names = stage_storage::vec(meter, &site(row_index, at), capacity)?;
                }
                names.insert(position, path);
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
    Ok(())
}
