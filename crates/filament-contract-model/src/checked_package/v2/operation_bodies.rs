// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Authored operation bodies carried by the checked package (TC-987).
//!
//! Implements: FR-322 "Operation-body carrier", FR-322-AC-397.

use super::identity::is_identifier;
use super::model_fields::RetainedModels;
use super::model_members::Budget;
use super::native_join_budget::PackageReadStage;
use super::operation_effects::{
    first_repeated, has_uncovered_operation, join_member, key_of, sorted_keys, Key,
};
use super::stage_storage::{self, Path, Refusal, Site};
use super::{CheckedJson, CheckedSemanticNodeV2, RawCheckedPackageWireV2};
use super::{CheckedNodeId, CheckedNodeKind, CheckedNodeOwner, CheckedTerm, WorkMeter};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
};
use quire_canonical::{Encode, Error, Sink, Writer};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

/// The authored body of one model operation member, or its stated absence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OperationBody {
    /// The checked body function node of an operation that has a body.
    Function {
        /// The package `function` node holding the body.
        node: CheckedNodeId,
    },
    /// An operation with no authored body.
    NoBody,
}

impl<'de> Deserialize<'de> for OperationBody {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "snake_case")]
        enum Kind {
            Function,
            NoBody,
        }
        // An internally tagged unit variant ignores a member it does not name,
        // so the closed shape is checked here.
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            kind: Kind,
            #[serde(default, deserialize_with = "present")]
            node: Option<CheckedNodeId>,
        }
        fn present<'de, D: Deserializer<'de>>(
            deserializer: D,
        ) -> Result<Option<CheckedNodeId>, D::Error> {
            CheckedNodeId::deserialize(deserializer).map(Some)
        }
        match Wire::deserialize(deserializer)? {
            Wire {
                kind: Kind::Function,
                node: Some(node),
            } => Ok(Self::Function { node }),
            Wire {
                kind: Kind::NoBody,
                node: None,
            } => Ok(Self::NoBody),
            Wire {
                kind: Kind::Function,
                node: None,
            } => Err(D::Error::missing_field("node")),
            Wire {
                kind: Kind::NoBody,
                node: Some(_),
            } => Err(D::Error::unknown_field("node", &["kind"])),
        }
    }
}

/// A producer-constructible operation body row; admission validates its claims.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationBodyRow {
    /// Selected document identity.
    pub document: Box<str>,
    /// Structural ID of the declaring model object, process or state machine.
    pub declaring: CheckedNodeId,
    /// Original operation name on that declaring node.
    pub operation: Box<str>,
    /// The body function node, or `no_body`; never omitted.
    pub body: OperationBody,
}

impl Encode for OperationBody {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        match self {
            Self::Function { node } => {
                super::encode::member(writer, "kind", "function")?;
                super::encode::encoded(writer, "node", node)?;
            }
            Self::NoBody => super::encode::member(writer, "kind", "no_body")?,
        }
        writer.end_object()
    }
}

impl Encode for OperationBodyRow {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        super::encode::member(writer, "document", self.document.as_ref())?;
        super::encode::encoded(writer, "declaring", &self.declaring)?;
        super::encode::member(writer, "operation", self.operation.as_ref())?;
        super::encode::encoded(writer, "body", &self.body)?;
        writer.end_object()
    }
}

impl super::CheckedPackageV2 {
    /// The admitted body of one operation member, keyed as `operation_effects`
    /// is. Absent when the package carries no `operation_bodies` member or no
    /// row names the key.
    pub fn operation_body(
        &self,
        document: &str,
        declaring: &CheckedNodeId,
        operation: &str,
    ) -> Option<&OperationBody> {
        let rows = self.wire.operation_bodies.as_ref()?;
        let index = rows
            .binary_search_by(|row| {
                key_of(&row.document, &row.declaring, &row.operation)
                    .cmp(&(document, declaring, operation))
            })
            .ok()?;
        rows.get(index).map(|row| &row.body)
    }

    /// Admitted operation body rows for borrowed typed re-encoding.
    ///
    /// Absence means the optional carrier was omitted, so bodies are
    /// unavailable; a present slice, possibly empty, preserves the admitted
    /// key order.
    pub fn operation_body_rows(&self) -> Option<&[OperationBodyRow]> {
        self.wire.operation_bodies.as_deref()
    }
}

const STAGE_FORM: &str = "operation_bodies";

fn rows_at() -> Path<'static> {
    Path::root().key("operation_bodies")
}

fn site(row: usize, at: Path<'static>) -> Site<'static> {
    Site::new(PackageReadStage::OperationBodies, row, STAGE_FORM, at)
}

fn refused(
    meter: &mut WorkMeter,
    row: usize,
    code: Code,
    cause: Option<Cause>,
    at: Path<'static>,
) -> ValidationFailure {
    let refusal = Refusal::new(code, at);
    let refusal = match cause {
        Some(cause) => refusal.because(cause),
        None => refusal,
    };
    stage_storage::refuse(meter, &site(row, at), refusal)
}

fn invalid(meter: &mut WorkMeter, row: usize, at: Path<'static>) -> ValidationFailure {
    refused(
        meter,
        row,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        at,
    )
}

/// Whether `node` is owned by the operation member `member`. The member
/// identity names its document, and the owner join has already matched the
/// owner's document, so the owner node alone is compared.
fn owned_by(node: &CheckedSemanticNodeV2<CheckedJson>, member: &str) -> bool {
    matches!(
        node.owner.as_ref(),
        Some(CheckedNodeOwner::Model { node, .. }) if node.as_ref() == member
    )
}

/// The source-declared object types as (owner identity, type name, node).
type SourceTypes<'a> = Vec<(&'a str, &'a str, &'a CheckedNodeId)>;

/// A source body clause function: its (owner identity, type, operation) and node.
type SourceBody<'a> = ((&'a str, &'a str, &'a str), &'a CheckedNodeId);

/// A source dispatch candidate's joined row key and its body node.
type SourceCandidate<'a> = (&'a str, &'a CheckedNodeId, &'a str, &'a CheckedNodeId);

/// The `SourceOwner` identity of `node`, when it has one.
fn source_identity(node: &CheckedSemanticNodeV2<CheckedJson>) -> Option<&str> {
    match node.owner.as_ref() {
        Some(CheckedNodeOwner::Source { identity, .. }) => Some(identity.as_ref()),
        _ => None,
    }
}

/// `[<type>, <operation>, "body"]` of a source clause function, when `node`
/// is one: its `declaration` has exactly those three segments. The last
/// segment is the clause label of the declaration key (FSpec FR-610), a
/// classified wire string boundary.
#[ix_trace_rs::string_edge]
fn source_body_declaration(node: &CheckedSemanticNodeV2<CheckedJson>) -> Option<(&str, &str)> {
    match node.declaration.as_ref()?.qualified_name.as_slice() {
        [type_name, operation, clause] if clause.as_ref() == "body" => {
            Some((type_name.as_ref(), operation.as_ref()))
        }
        _ => None,
    }
}

/// A dispatch candidate's effective declaration (package and node), its
/// selected operation name, and its operation body.
///
/// A candidate joins the row of the declaration that declares the operation
/// its receiver (`selection_key.original`) effectively exposes: the receiver
/// itself when it redefines the operation, else the ancestor it inherits it
/// from. Candidates of one dispatch table share the selected operation but
/// carry different bodies, so the family's `selected_operation` is not the
/// join (FR-322 "Candidate join").
type Candidate<'a> = (&'a str, &'a str, &'a str, &'a CheckedNodeId);

/// Validate a present carrier against the selected documents and the package
/// graph before exposing any borrowed row.
pub(super) fn validate(
    rows: Option<&[OperationBodyRow]>,
    wire: &RawCheckedPackageWireV2,
    kinds: &[CheckedNodeKind],
    terms: &[CheckedTerm],
    models: &RetainedModels,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let Some(rows) = rows else {
        return Ok(());
    };
    let nodes = &wire.semantic_graph.nodes;
    // Graph positions by node ID, to resolve a body reference.
    let mut by_id: Vec<usize> = stage_storage::vec(meter, &site(0, rows_at()), nodes.len())?;
    by_id.extend(0..nodes.len());
    by_id.sort_unstable_by(|&left, &right| nodes[left].node_id.cmp(&nodes[right].node_id));
    // Dispatch candidates by selection-key declaration, to join a row to its candidates.
    let tables = wire
        .checked_term_data
        .as_ref()
        .map_or(&[][..], |data| data.dispatch_tables.as_slice());
    let count = tables.iter().map(|table| table.candidates.len()).sum();
    let mut candidates: Vec<Candidate<'_>> = stage_storage::vec(meter, &site(0, rows_at()), count)?;
    // The carrier runs before `checked_term_data` validation, so it joins only
    // a candidate whose receiver is in its table's family (the family owner's
    // package, and `operation_in_family`); any other candidate is left to
    // dispatch validation to refuse (FSpec OBD-01).
    for table in tables {
        let Some(family) = models.owner_by_key(&table.operation.declaring) else {
            continue;
        };
        let model = family.package;
        let name = table.operation.operation.as_ref();
        for candidate in &table.candidates {
            let original = &candidate.selection_key.original;
            if original.package.as_ref() != model.identity.as_ref() {
                continue;
            }
            let mut budget = Budget::new(meter, model.root, model.bytes);
            if !model.operation_in_family(&original.node, family.node, name, &mut budget)? {
                continue;
            }
            if let Some(owner) =
                model.effective_operation_owner(&original.node, name, &mut budget)?
            {
                candidates.push((
                    model.identity.as_ref(),
                    owner,
                    name,
                    &candidate.operation_body,
                ));
            }
        }
    }
    candidates.sort_unstable();
    let mut source_types: SourceTypes<'_> =
        stage_storage::vec(meter, &site(0, rows_at()), nodes.len())?;
    source_types.extend(nodes.iter().enumerate().filter_map(|(position, node)| {
        let identity = source_identity(node)?;
        let name = node.declaration.as_ref()?.qualified_name.last()?;
        super::source_object::is_source_declared_object_type(node, kinds[position]).then_some((
            identity,
            name.as_ref(),
            &node.node_id,
        ))
    }));
    source_types.sort_unstable();
    let mut source_bodies: Vec<SourceBody<'_>> =
        stage_storage::vec(meter, &site(0, rows_at()), nodes.len())?;
    source_bodies.extend(nodes.iter().enumerate().filter_map(|(position, node)| {
        if !matches!(kinds[position], CheckedNodeKind::Function(_)) {
            return None;
        }
        let (type_name, operation) = source_body_declaration(node)?;
        Some((
            (source_identity(node)?, type_name, operation),
            &node.node_id,
        ))
    }));
    source_bodies.sort_unstable();
    // FR-322 item 9: a source candidate joins the row of the type its body
    // clause function is declared under, whose `declaration` names it.
    let mut source_candidates: Vec<SourceCandidate<'_>> =
        stage_storage::vec(meter, &site(0, rows_at()), count)?;
    for table in tables {
        let operation = table.operation.operation.as_ref();
        for candidate in &table.candidates {
            stage_storage::work(meter, &site(0, rows_at()), 1)?;
            let Ok(found) = by_id.binary_search_by(|&position| {
                nodes[position].node_id.cmp(&candidate.operation_body)
            }) else {
                continue;
            };
            let body = &nodes[by_id[found]];
            let (Some(identity), Some((type_name, name))) =
                (source_identity(body), source_body_declaration(body))
            else {
                continue;
            };
            if name != operation {
                continue;
            }
            if let Ok(at) =
                source_types.binary_search_by(|held| (held.0, held.1).cmp(&(identity, type_name)))
            {
                source_candidates.push((
                    identity,
                    source_types[at].2,
                    name,
                    &candidate.operation_body,
                ));
            }
        }
    }
    source_candidates.sort_unstable();
    let covered = sorted_keys(
        meter,
        &site(0, rows_at()),
        rows.iter()
            .map(|row| key_of(&row.document, &row.declaring, &row.operation)),
    )?;
    let repeated = first_repeated(&covered);
    // The members that own a body clause function.
    let mut owned: Vec<&str> = stage_storage::vec(meter, &site(0, rows_at()), nodes.len())?;
    owned.extend(nodes.iter().enumerate().filter_map(|(position, node)| {
        match (
            node.owner.as_ref(),
            kinds.get(position),
            terms.get(position),
        ) {
            (Some(CheckedNodeOwner::Model { node: member, .. }), Some(kind), Some(term))
                if matches!(kind, CheckedNodeKind::Function(_))
                    && super::owner::is_body_clause(term) =>
            {
                Some(member.as_ref())
            }
            _ => None,
        }
    }));
    owned.sort_unstable();
    let mut previous: Option<Key<'_>> = None;
    for (index, row) in rows.iter().enumerate() {
        let at = rows_at().index(index);
        stage_storage::work(meter, &site(index, at), 1)?;
        if row.document.is_empty() {
            return Err(refused(
                meter,
                index,
                Code::MalformedWire,
                None,
                at.key("document"),
            ));
        }
        if !is_identifier(&row.operation) {
            return Err(refused(
                meter,
                index,
                Code::MalformedWire,
                None,
                at.key("operation"),
            ));
        }
        let source_declaring = by_id
            .binary_search_by(|&position| nodes[position].node_id.cmp(&row.declaring))
            .ok()
            .map(|found| by_id[found])
            .filter(|&position| {
                super::source_object::is_source_declared_object_type(
                    &nodes[position],
                    kinds[position],
                )
            });
        if let Some(declaring) = source_declaring {
            check_source_row(
                meter,
                index,
                row,
                &nodes[declaring],
                SourceJoin {
                    nodes,
                    kinds,
                    by_id: &by_id,
                    bodies: &source_bodies,
                },
            )?;
            let key = key_of(&row.document, &row.declaring, &row.operation);
            if repeated == Some(index) {
                return Err(refused(
                    meter,
                    index,
                    Code::InvalidPackage,
                    Some(Cause::ConflictingDefinition),
                    at,
                ));
            }
            if previous.is_some_and(|earlier| earlier >= key) {
                return Err(invalid(meter, index, at));
            }
            previous = Some(key);
            continue;
        }
        let (owner, operation) = join_member(
            models,
            &row.document,
            &row.declaring,
            &row.operation,
            at,
            meter,
            &mut |meter, code, cause, locus| refused(meter, index, code, Some(cause), locus),
        )?;
        let has_body = owned.binary_search(&operation.identity.as_ref()).is_ok();
        let body_at = at.key("body");
        let node = match &row.body {
            OperationBody::Function { node } if has_body => Some(node),
            OperationBody::NoBody if !has_body => None,
            _ => return Err(invalid(meter, index, body_at)),
        };
        if let Some(node) = node {
            let node_at = body_at.key("node");
            let Ok(found) = by_id.binary_search_by(|&position| nodes[position].node_id.cmp(node))
            else {
                return Err(refused(
                    meter,
                    index,
                    Code::MissingDeclaration,
                    Some(Cause::MissingName),
                    node_at,
                ));
            };
            let position = by_id[found];
            if !matches!(kinds.get(position), Some(CheckedNodeKind::Function(_))) {
                return Err(refused(
                    meter,
                    index,
                    Code::IllTyped,
                    Some(Cause::OperatorIneligible),
                    node_at,
                ));
            }
            let own = terms
                .get(position)
                .is_some_and(super::owner::is_body_clause)
                && owned_by(&nodes[position], &operation.identity);
            if !own {
                return Err(invalid(meter, index, node_at));
            }
        }
        let joined = (owner.package.identity.as_ref(), owner.node, &*row.operation);
        let start = candidates.partition_point(|held| (held.0, held.1, held.2) < joined);
        let agrees = candidates[start..]
            .iter()
            .take_while(|held| (held.0, held.1, held.2) == joined)
            .all(|held| node == Some(held.3));
        if !agrees {
            return Err(invalid(meter, index, body_at));
        }
        if repeated == Some(index) {
            return Err(refused(
                meter,
                index,
                Code::InvalidPackage,
                Some(Cause::ConflictingDefinition),
                at,
            ));
        }
        let key = key_of(&row.document, &row.declaring, &row.operation);
        if previous.is_some_and(|earlier| earlier >= key) {
            return Err(invalid(meter, index, at));
        }
        previous = Some(key);
    }
    let uncovered_source = source_candidates
        .iter()
        .any(|(identity, node, operation, _)| {
            covered
                .binary_search_by(|(key, _)| {
                    (key.0, key.1, key.2).cmp(&(*identity, *node, *operation))
                })
                .is_err()
        });
    if uncovered_source || has_uncovered_operation(models, &covered) {
        return Err(invalid(meter, 0, rows_at()));
    }
    Ok(())
}

/// The package facts a source row joins.
struct SourceJoin<'a> {
    nodes: &'a [CheckedSemanticNodeV2<CheckedJson>],
    kinds: &'a [CheckedNodeKind],
    by_id: &'a [usize],
    bodies: &'a [SourceBody<'a>],
}

/// FR-322 item 9 over one row whose `declaring` is a source-declared object
/// type: `document` is the node's `SourceOwner` identity, and `body` is a
/// `function` exactly when the package holds the clause function declared
/// `[<type>, <operation>, "body"]` under that owner, naming it.
fn check_source_row(
    meter: &mut WorkMeter,
    index: usize,
    row: &OperationBodyRow,
    declaring: &CheckedSemanticNodeV2<CheckedJson>,
    join: SourceJoin<'_>,
) -> Result<(), ValidationFailure> {
    let at = rows_at().index(index);
    let Some(identity) = source_identity(declaring) else {
        return Err(invalid(meter, index, at.key("declaring")));
    };
    let document_at = at.key("document");
    if row.document.as_ref() != identity {
        let owned = join
            .nodes
            .iter()
            .any(|node| source_identity(node) == Some(row.document.as_ref()));
        return Err(if owned {
            invalid(meter, index, document_at)
        } else {
            refused(
                meter,
                index,
                Code::MissingDeclaration,
                Some(Cause::MissingSelection),
                document_at,
            )
        });
    }
    let type_name = declaring
        .declaration
        .as_ref()
        .and_then(|declaration| declaration.qualified_name.last())
        .map(AsRef::as_ref);
    let held = type_name.and_then(|type_name| {
        let key = (identity, type_name, row.operation.as_ref());
        join.bodies
            .binary_search_by(|(held, _)| (held.0, held.1, held.2).cmp(&key))
            .ok()
            .map(|found| join.bodies[found].1)
    });
    let body_at = at.key("body");
    match (&row.body, held) {
        (OperationBody::Function { node }, Some(expected)) => {
            let node_at = body_at.key("node");
            let Ok(found) = join
                .by_id
                .binary_search_by(|&position| join.nodes[position].node_id.cmp(node))
            else {
                return Err(refused(
                    meter,
                    index,
                    Code::MissingDeclaration,
                    Some(Cause::MissingName),
                    node_at,
                ));
            };
            if !matches!(
                join.kinds.get(join.by_id[found]),
                Some(CheckedNodeKind::Function(_))
            ) {
                return Err(refused(
                    meter,
                    index,
                    Code::IllTyped,
                    Some(Cause::OperatorIneligible),
                    node_at,
                ));
            }
            if node != expected {
                return Err(invalid(meter, index, node_at));
            }
        }
        (OperationBody::NoBody, None) => {}
        _ => return Err(invalid(meter, index, body_at)),
    }
    Ok(())
}
