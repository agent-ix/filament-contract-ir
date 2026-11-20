// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Declared names of state-clause, refinement and non-temporal claim
//! declarations.

use super::declaration_region::SourceEntries;
use super::graph_storage::GraphIndex;
use super::stage_storage::{self, Path, Refusal, Site};
use super::temporal_clause_names::TemporalClauseName;
use super::{
    identity::is_identifier, structural::typed_admitted_reference, CheckedNodeId, CheckedNodeKind,
    CheckedOccurrence, CheckedOccurrenceRole, CheckedTerm, ClaimForm, RawSemanticNodeV2,
    RelationForm, StateForm, TemporalForm, WorkMeter,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code, CheckedSourceRef,
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
            | CheckedNodeKind::Claim(
                ClaimForm::VerificationClaim
                    | ClaimForm::AnalysisClaim
                    | ClaimForm::Hyperproperty
                    | ClaimForm::SynthesisRequest
                    | ClaimForm::ProbabilisticClaim
                    | ClaimForm::StateGraph
                    | ClaimForm::StepRelation
                    | ClaimForm::Schedulability
            )
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
    let all = rows.unwrap_or_default();
    let capacity = all.len();
    let mut seen: Vec<(&CheckedNodeId, &CheckedOccurrence)> = Vec::new();
    let mut names: Vec<(&[Box<str>], usize)> = Vec::new();
    // A declaration is told by its `declaration_region`; `declarations` holds
    // the first row of each one, and `row_declaration` and `row_name` the first
    // row of the declaration and of the qualified name each visited row carries.
    let mut declarations: Vec<(DeclarationKey<'_>, usize)> = Vec::new();
    let mut row_declaration: Vec<usize> = Vec::new();
    let mut row_name: Vec<usize> = Vec::new();
    let mut row_node: Vec<usize> = Vec::new();
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
    for (row_index, row) in all.iter().enumerate() {
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
        // One declaration has one name: a row of a declaration already seen
        // repeats its name and path.
        let key = declaration_key(row);
        let first_of_declaration = match declarations.binary_search_by(|(held, _)| held.cmp(&key)) {
            Ok(found) => {
                let first = declarations[found].1;
                if all[first].name != row.name || all[first].qualified_name != row.qualified_name {
                    return Err(invalid(at.key("name"), &row.declaration, row_index, meter));
                }
                first
            }
            Err(slot) => {
                if declarations.is_empty() {
                    declarations = stage_storage::vec(meter, &site(row_index, at), capacity)?;
                    row_declaration = stage_storage::vec(meter, &site(row_index, at), capacity)?;
                    row_name = stage_storage::vec(meter, &site(row_index, at), capacity)?;
                    row_node = stage_storage::vec(meter, &site(row_index, at), capacity)?;
                }
                declarations.insert(slot, (key, row_index));
                row_index
            }
        };
        let path: &[Box<str>] = &row.qualified_name;
        let first_of_name = match names.binary_search_by(|(earlier, _)| (*earlier).cmp(path)) {
            Ok(found) => {
                let earlier = names[found].1;
                // Rows of one declaration are not duplicates of each other.
                if row_declaration[earlier] != first_of_declaration {
                    // The refusal names both rows: the later one is its locus
                    // and the earlier one the package-document position.
                    return Err(stage_storage::refuse(
                        meter,
                        &site(row_index, at),
                        Refusal::new(Code::AmbiguousDeclaration, at.key("qualified_name"))
                            .because(Cause::AmbiguousName)
                            .at_node(&row.declaration)
                            .in_document(root().index(earlier).key("qualified_name")),
                    ));
                }
                earlier
            }
            Err(slot) => {
                if names.is_empty() {
                    names = stage_storage::vec(meter, &site(row_index, at), capacity)?;
                }
                names.insert(slot, (path, row_index));
                row_index
            }
        };
        row_declaration.push(first_of_declaration);
        row_name.push(first_of_name);
        row_node.push(position);
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
    own_nodes(all, &row_node, &row_declaration, &row_name, meter)?;
    one_namespace(all, temporal.unwrap_or_default(), meter)
}

/// The graph positions of the body wrappers (FR-402): the `temporal_clause`
/// nodes that a `claim`/`hyperproperty` node's `body` binding references. A
/// wrapper has no `temporal_clause_names` row; the hyper clause's name rows are
/// the `claim_names` rows of its `hyperproperty` node. A node whose body is not
/// the hyperproperty aggregate is left to the hyperproperty step that owns it.
/// The nodes were charged by the graph stage; the sorted table is charged.
// The one string comparison is the closed `body` binding name FSpec FR-402
// fixes for the hyperproperty aggregate, not an open key.
#[ix_trace_rs::string_edge]
pub(super) fn body_wrappers(
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &GraphIndex<'_>,
    meter: &mut WorkMeter,
) -> Result<Vec<usize>, ValidationFailure> {
    let hyper = CheckedNodeKind::Claim(ClaimForm::Hyperproperty);
    let count = kinds.iter().filter(|kind| **kind == hyper).count();
    if count == 0 {
        return Ok(Vec::new());
    }
    let mut wrappers: Vec<usize> = stage_storage::vec(meter, &site(0, root()), count)?;
    for (position, kind) in kinds.iter().enumerate() {
        if *kind != hyper {
            continue;
        }
        let Some(CheckedTerm::Aggregate { members }) = terms[position].as_ref() else {
            continue;
        };
        let target = members
            .iter()
            .filter_map(|member| match member {
                CheckedTerm::Binding { name, value } if name.as_ref() == "body" => {
                    typed_admitted_reference(value)
                }
                _ => None,
            })
            .next();
        if let Some(&wrapper) = target.and_then(|target| index.get(target)) {
            if kinds[wrapper] == CheckedNodeKind::Temporal(TemporalForm::TemporalClause) {
                wrappers.push(wrapper);
            }
        }
    }
    wrappers.sort_unstable();
    wrappers.dedup();
    Ok(wrappers)
}

/// The identity of a declaration: its `declaration_region`, compared as
/// (`source`, `start`, `end`).
type DeclarationKey<'a> = (&'a CheckedSourceRef, u64, u64);

fn declaration_key(row: &ClaimNameRow) -> DeclarationKey<'_> {
    (
        &row.declaration_region.source,
        row.declaration_region.start,
        row.declaration_region.end,
    )
}

/// A name whose rows name two or more nodes needs an own node: the node whose
/// declaration set (the declaration keys of its rows) is strictly contained in
/// that of every other node the name's rows name (FR-322 "Rows of one
/// declaration"). A name without one is ambiguous at its last row, naming its
/// first and last rows. The rows are paid for once to build the node sets, once
/// to group them by name, and each set element compared is one visit.
fn own_nodes(
    rows: &[ClaimNameRow],
    row_node: &[usize],
    row_declaration: &[usize],
    row_name: &[usize],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    if rows.is_empty() {
        return Ok(());
    }
    let count = rows.len();
    // (node, declaration) pairs, sorted: each node's set is a contiguous run.
    let mut sets: Vec<(usize, usize)> = stage_storage::vec(meter, &site(0, root()), count)?;
    for (row_index, (node, declaration)) in row_node.iter().zip(row_declaration).enumerate() {
        stage_storage::work(meter, &site(row_index, root().index(row_index)), 1)?;
        sets.push((*node, *declaration));
    }
    sets.sort_unstable();
    sets.dedup();
    // Rows by the first row of their name, in row order within a name.
    let mut grouped: Vec<(usize, usize)> = stage_storage::vec(meter, &site(0, root()), count)?;
    for (row_index, name) in row_name.iter().enumerate() {
        stage_storage::work(meter, &site(row_index, root().index(row_index)), 1)?;
        grouped.push((*name, row_index));
    }
    grouped.sort_unstable();
    let set_of = |node: usize| -> &[(usize, usize)] {
        let start = sets.partition_point(|(held, _)| *held < node);
        let end = sets.partition_point(|(held, _)| *held <= node);
        &sets[start..end]
    };
    let mut nodes: Vec<usize> = stage_storage::vec(meter, &site(0, root()), count)?;
    let mut at = 0;
    while at < grouped.len() {
        let name = grouped[at].0;
        let end = at + grouped[at..].partition_point(|(held, _)| *held == name);
        nodes.clear();
        nodes.extend(grouped[at..end].iter().map(|(_, row)| row_node[*row]));
        nodes.sort_unstable();
        nodes.dedup();
        if nodes.len() > 1 {
            // The own node, if any, has the strictly smallest set.
            let smallest = nodes.iter().map(|node| set_of(*node).len()).min();
            let mut candidates = nodes
                .iter()
                .filter(|node| Some(set_of(**node).len()) == smallest);
            let own = match (candidates.next(), candidates.next()) {
                (Some(own), None) => Some(*own),
                _ => None,
            };
            let nested = match own {
                Some(own) => {
                    let inner = set_of(own);
                    let mut nested = true;
                    for node in nodes.iter().filter(|node| **node != own) {
                        let outer = set_of(*node);
                        stage_storage::work(
                            meter,
                            &site(name, root().index(name)),
                            (inner.len() + outer.len()) as u64,
                        )?;
                        if !inner.iter().all(|(_, key)| {
                            outer.binary_search_by(|(_, held)| held.cmp(key)).is_ok()
                        }) {
                            nested = false;
                            break;
                        }
                    }
                    nested
                }
                None => false,
            };
            if !nested {
                let last = grouped[end - 1].1;
                let at_last = root().index(last);
                return Err(stage_storage::refuse(
                    meter,
                    &site(last, at_last.key("qualified_name")),
                    Refusal::new(Code::AmbiguousDeclaration, at_last.key("qualified_name"))
                        .because(Cause::AmbiguousName)
                        .at_node(&rows[last].declaration)
                        .in_document(root().index(name).key("qualified_name")),
                ));
            }
        }
        at = end;
    }
    Ok(())
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
