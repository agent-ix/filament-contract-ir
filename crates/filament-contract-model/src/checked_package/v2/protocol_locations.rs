// SPDX-License-Identifier: AGPL-3.0-or-later
//! Validation and borrowed reads of the canonical captured protocol locations.
//! The common checked-term-data module owns all wire types and decoding.

use super::checked_term_data::{
    CheckedBodyLocationOrigin, CheckedBodySemanticLocation, CheckedKeyOrdinal,
    CheckedProtocolLocationOrigin, CheckedProtocolSemanticLocation,
};
use super::dispatch_projection::{data_site, fail, invalid};
use super::stage_storage::{self, Path};
use super::{
    CheckedNodeKind, CheckedPackageV2, CheckedPackageWireV2, CheckedSemanticNodeV2, CheckedTerm,
    ExpressionForm, WorkMeter,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedNodeId, CheckedOccurrence, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    CheckedSourceMapEntry,
};
use std::collections::BTreeSet;

/// Sorted borrowed node identities, shared with the body walk.
pub(super) type NodeIndex<'a> = Vec<(&'a CheckedNodeId, usize)>;

const MAX_SAFE: u64 = 9_007_199_254_740_991;

fn at(array: &'static str, row: usize) -> Path<'static> {
    Path::root().key("checked_term_data").key(array).index(row)
}

/// One captured location row of a retained array: the shared key, join and
/// path shape of `protocol_locations` and `body_locations`.
pub(super) trait LocationRow {
    /// The local application node.
    fn application(&self) -> &CheckedNodeId;
    /// The occurrence of that application.
    fn occurrence(&self) -> &CheckedOccurrence;
    /// The term coordinate within the occurrence.
    fn term_ordinal(&self) -> u64;
    /// Body locations are independently keyed by function; protocol rows have
    /// no function owner.
    fn function(&self) -> Option<&CheckedNodeId>;
    /// The captured child path entries, as written.
    fn path(&self) -> &[CheckedKeyOrdinal];
    /// The wire member that holds `path()`.
    fn path_member(&self) -> &'static str;
    /// Check the origin wire grammar, without graph decisions.
    fn check_origin(
        &self,
        array: &'static str,
        index: usize,
        meter: &mut WorkMeter,
    ) -> Result<(), ValidationFailure>;
}

impl LocationRow for CheckedProtocolSemanticLocation {
    fn application(&self) -> &CheckedNodeId {
        &self.application
    }
    fn occurrence(&self) -> &CheckedOccurrence {
        &self.occurrence
    }
    fn term_ordinal(&self) -> u64 {
        self.term_ordinal
    }
    fn function(&self) -> Option<&CheckedNodeId> {
        None
    }
    fn path(&self) -> &[CheckedKeyOrdinal] {
        &self.location.path
    }
    fn path_member(&self) -> &'static str {
        "path"
    }
    fn check_origin(
        &self,
        array: &'static str,
        index: usize,
        meter: &mut WorkMeter,
    ) -> Result<(), ValidationFailure> {
        intake_protocol_origin(&self.location.origin, array, index, meter)
    }
}

impl LocationRow for CheckedBodySemanticLocation {
    fn application(&self) -> &CheckedNodeId {
        &self.application
    }
    fn occurrence(&self) -> &CheckedOccurrence {
        &self.occurrence
    }
    fn term_ordinal(&self) -> u64 {
        self.term_ordinal
    }
    fn function(&self) -> Option<&CheckedNodeId> {
        match &self.location.origin {
            CheckedBodyLocationOrigin::FunctionBody { function, .. } => Some(function),
        }
    }
    fn path(&self) -> &[CheckedKeyOrdinal] {
        &self.location.down
    }
    fn path_member(&self) -> &'static str {
        "down"
    }
    fn check_origin(
        &self,
        array: &'static str,
        index: usize,
        meter: &mut WorkMeter,
    ) -> Result<(), ValidationFailure> {
        let up = at(array, index).key("location").key("up");
        work(meter, index, up, 1)?;
        if self.location.up > MAX_SAFE {
            return Err(malformed(meter, index, at(array, index).key("location")));
        }
        intake_body_origin(&self.location.origin, array, index, meter)
    }
}

/// A malformed-wire refusal at `path`, its diagnostic charged before it is built.
fn malformed(meter: &mut WorkMeter, row: usize, path: Path<'static>) -> ValidationFailure {
    fail(
        meter,
        row,
        CheckedPackageRefusalCode::MalformedWire,
        Some(CheckedPackageRefusalCause::InvalidValue),
        path,
    )
}

/// One unit of work for the value at `path` in row `row`.
fn work(
    meter: &mut WorkMeter,
    row: usize,
    path: Path<'static>,
    amount: u64,
) -> Result<(), ValidationFailure> {
    stage_storage::work(meter, &data_site(row, path), amount)
}

/// Validate retained row keys, canonical ordinal shapes and local graph joins.
/// The envelope owner calls this after the typed node terms are available.
/// No graph interpretation or source-location reconstruction occurs here.
pub(super) fn validate<Body>(
    wire: &CheckedPackageWireV2<Body>,
    kinds: &[CheckedNodeKind],
    terms: &[CheckedTerm],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let Some(data) = wire.checked_term_data.as_ref() else {
        return Ok(());
    };
    let (gap, _) = validate_rows(
        "protocol_locations",
        &wire.name,
        &wire.semantic_graph.nodes,
        kinds,
        terms,
        &wire.source_map,
        &data.protocol_locations,
        meter,
    )?;
    refuse_missing_predecessor("protocol_locations", gap, meter)
}

/// A retained successor witnesses a gap; a final row cannot witness an absent
/// successor. Existing row shape, join, duplicate and ordering refusals keep
/// their precedence over this array-level refusal.
pub(super) fn refuse_missing_predecessor(
    array: &'static str,
    gap: bool,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    if gap {
        return Err(invalid(
            meter,
            0,
            Path::root().key("checked_term_data").key(array),
        ));
    }
    Ok(())
}

/// Check every row of `array` in order and report whether a row follows a
/// missing predecessor ordinal.
#[allow(clippy::too_many_arguments)]
pub(super) fn validate_rows<'a, Body, Row: LocationRow>(
    array: &'static str,
    name: &str,
    nodes: &'a [CheckedSemanticNodeV2<Body>],
    kinds: &[CheckedNodeKind],
    terms: &[CheckedTerm],
    source_map: &[CheckedSourceMapEntry],
    rows: &[Row],
    meter: &mut WorkMeter,
) -> Result<(bool, NodeIndex<'a>), ValidationFailure> {
    if rows.is_empty() {
        return Ok((false, Vec::new()));
    }
    let site = data_site(0, Path::root().key("checked_term_data").key(array));
    stage_storage::work(meter, &site, u64::try_from(nodes.len()).unwrap_or(u64::MAX))?;
    let mut node_index = stage_storage::vec(meter, &site, nodes.len())?;
    node_index.extend(nodes.iter().enumerate().map(|(i, node)| (&node.node_id, i)));
    node_index.sort_unstable();
    stage_storage::work(
        meter,
        &site,
        u64::try_from(source_map.len()).unwrap_or(u64::MAX),
    )?;
    let mut sources = stage_storage::vec(meter, &site, source_map.len())?;
    sources.extend(
        source_map
            .iter()
            .map(|entry| (&entry.node_id, &entry.role, entry.ordinal)),
    );
    sources.sort_unstable();
    let occurrence_count = nodes.iter().fold(0_usize, |total, node| {
        total.saturating_add(node.occurrences.len())
    });
    stage_storage::work(
        meter,
        &site,
        u64::try_from(occurrence_count).unwrap_or(u64::MAX),
    )?;
    let mut occurrences = stage_storage::vec(meter, &site, occurrence_count)?;
    occurrences.extend(nodes.iter().enumerate().flat_map(|(i, node)| {
        node.occurrences
            .iter()
            .map(move |occurrence| (i, occurrence))
    }));
    occurrences.sort_unstable();
    let mut previous: Option<&Row> = None;
    let mut ordinal_rows = stage_storage::vec(meter, &site, rows.len())?;
    type SeenKey<'a> = (
        Option<&'a CheckedNodeId>,
        &'a CheckedNodeId,
        &'a CheckedOccurrence,
        u64,
    );
    stage_storage::reserve_items::<SeenKey<'_>>(meter, &site, rows.len())?;
    let mut seen_keys = BTreeSet::<SeenKey<'_>>::new();
    let mut missing_predecessor = false;
    for (index, row) in rows.iter().enumerate() {
        let here = at(array, index);
        work(meter, index, here, 1)?;
        intake_local_application(row.application(), name, array, index, meter)?;
        let term_ordinal = intake_term_ordinal(row.term_ordinal(), array, index, meter)?;
        work(meter, index, here, 1)?;
        if !seen_keys.insert((
            row.function(),
            row.application(),
            row.occurrence(),
            term_ordinal,
        )) {
            return Err(fail(
                meter,
                index,
                CheckedPackageRefusalCode::InvalidPackage,
                Some(CheckedPackageRefusalCause::ConflictingDefinition),
                here,
            ));
        }
        if let Some(before) = previous {
            let key = (row.application(), row.occurrence(), term_ordinal);
            let prior = (
                before.application(),
                before.occurrence(),
                before.term_ordinal(),
            );
            if key < prior {
                return Err(invalid(meter, index, here));
            }
        }
        previous = Some(row);
        work(meter, index, here.key("application"), 1)?;
        let found = node_index.partition_point(|(id, _)| *id < row.application());
        let Some((_, position)) = node_index
            .get(found)
            .filter(|(id, _)| *id == row.application())
        else {
            return Err(invalid(meter, index, here.key("application")));
        };
        let position = *position;
        // A leaf owns its location on its graph node. Its occurrence and a
        // matching enclosing application location cannot make a row valid.
        match (kinds.get(position), terms.get(position)) {
            (Some(CheckedNodeKind::Expression(form)), Some(CheckedTerm::Application { .. }))
                if *form != ExpressionForm::Reference => {}
            _ => return Err(invalid(meter, index, here.key("application"))),
        }
        work(meter, index, here.key("occurrence"), 1)?;
        if occurrences
            .binary_search(&(position, row.occurrence()))
            .is_err()
        {
            return Err(invalid(meter, index, here.key("occurrence")));
        }
        if sources
            .binary_search(&(
                row.application(),
                &row.occurrence().role,
                row.occurrence().ordinal,
            ))
            .is_err()
        {
            return Err(invalid(meter, index, here.key("occurrence")));
        }
        intake_location_value(row, array, index, meter)?;
        ordinal_rows.push((
            row.function(),
            row.application(),
            row.occurrence(),
            term_ordinal,
        ));
    }
    stage_storage::work(
        meter,
        &site,
        u64::try_from(ordinal_rows.len()).unwrap_or(u64::MAX),
    )?;
    ordinal_rows.sort_unstable_by(|left, right| {
        (left.0, left.1, left.2, left.3).cmp(&(right.0, right.1, right.2, right.3))
    });
    let mut prior: Option<(Option<&CheckedNodeId>, &CheckedNodeId, &CheckedOccurrence)> = None;
    let mut expected = 0_u64;
    for (function, application, occurrence, ordinal) in ordinal_rows {
        let key = (function, application, occurrence);
        if prior == Some(key) {
            missing_predecessor |= ordinal != expected;
        } else {
            missing_predecessor |= ordinal != 0;
            prior = Some(key);
        }
        expected = ordinal.saturating_add(1);
    }
    Ok((missing_predecessor, node_index))
}

/// Convert an incoming numeric term coordinate to its safe wire range.
#[ix_trace_rs::string_edge]
fn intake_term_ordinal(
    wire: u64,
    array: &'static str,
    index: usize,
    meter: &mut WorkMeter,
) -> Result<u64, ValidationFailure> {
    if wire > MAX_SAFE {
        return Err(malformed(
            meter,
            index,
            at(array, index).key("term_ordinal"),
        ));
    }
    Ok(wire)
}

/// Bind the incoming sidecar NodeRef's namespace before any graph join.
/// This checks untrusted wire selection; it does not classify an admitted node.
#[ix_trace_rs::string_edge]
fn intake_local_application<'a>(
    wire_id: &'a CheckedNodeId,
    root_package: &str,
    array: &'static str,
    index: usize,
    meter: &mut WorkMeter,
) -> Result<&'a CheckedNodeId, ValidationFailure> {
    if wire_id.package.as_ref() != root_package {
        return Err(invalid(meter, index, at(array, index).key("application")));
    }
    Ok(wire_id)
}

/// Check the captured origin and ordinal wire grammar, without graph decisions.
#[ix_trace_rs::string_edge]
fn intake_location_value<Row: LocationRow>(
    row: &Row,
    array: &'static str,
    index: usize,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    row.check_origin(array, index, meter)?;
    for (position, ordinal) in row.path().iter().enumerate() {
        let locus = at(array, index)
            .key("location")
            .key(row.path_member())
            .index(position);
        work(meter, index, locus, 1)?;
        match ordinal {
            CheckedKeyOrdinal::Safe(value) if *value <= MAX_SAFE => {}
            CheckedKeyOrdinal::BeyondSafe(text) => {
                work(
                    meter,
                    index,
                    locus,
                    u64::try_from(text.len()).unwrap_or(u64::MAX),
                )?;
                let beyond =
                    text.len() > 16 || (text.len() == 16 && text.as_ref() > "9007199254740991");
                if !beyond || text.starts_with('0') || !text.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(malformed(meter, index, locus));
                }
            }
            CheckedKeyOrdinal::Safe(_) => return Err(malformed(meter, index, locus)),
        }
    }
    Ok(())
}

/// Check the protocol origin wire grammar.
#[ix_trace_rs::string_edge]
fn intake_protocol_origin(
    origin_value: &CheckedProtocolLocationOrigin,
    array: &'static str,
    index: usize,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let origin = at(array, index).key("location").key("origin");
    match origin_value {
        CheckedProtocolLocationOrigin::TypeDeclaration { name } => {
            work(
                meter,
                index,
                origin,
                u64::try_from(name.len()).unwrap_or(u64::MAX),
            )?;
            if name.is_empty() {
                return Err(malformed(meter, index, origin));
            }
        }
        CheckedProtocolLocationOrigin::ProtocolAttempt { protocol, attempt } => {
            if *protocol > MAX_SAFE || *attempt > MAX_SAFE {
                return Err(malformed(meter, index, origin));
            }
        }
    }
    Ok(())
}

/// Check the body origin wire grammar: a non-empty name, charged by length, and
/// a safe function index.
#[ix_trace_rs::string_edge]
fn intake_body_origin(
    origin_value: &CheckedBodyLocationOrigin,
    array: &'static str,
    index: usize,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let CheckedBodyLocationOrigin::FunctionBody {
        name,
        index: function_index,
        ..
    } = origin_value;
    let origin = at(array, index).key("location").key("origin");
    work(
        meter,
        index,
        origin.key("name"),
        u64::try_from(name.len()).unwrap_or(u64::MAX),
    )?;
    if name.is_empty() {
        return Err(malformed(meter, index, origin.key("name")));
    }
    if *function_index > MAX_SAFE {
        return Err(malformed(meter, index, origin.key("index")));
    }
    Ok(())
}

/// Why a selected captured semantic location could not be returned.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum CheckedProtocolLocationReadError {
    /// The admitted package retained no checked-term-data sidecar.
    #[error("protocol semantic location data is unavailable")]
    Unavailable,
    /// The requested exact application/occurrence/term key has no captured row.
    #[error("the selected protocol term has no captured location")]
    Missing,
    /// The first uncharged row or child-path entry exceeded the caller limit.
    #[error("protocol semantic location read exhausted its work allowance")]
    WorkExhausted {
        /// Caller-supplied finite work limit.
        limit: u64,
        /// Attempted work including the denied charge.
        consumed: u64,
        /// Sidecar row where work stopped.
        row: usize,
        /// Child-path position when that entry could not be charged.
        child: Option<usize>,
    },
}

impl CheckedPackageV2 {
    /// Borrow the captured row for one exact local application occurrence and
    /// semantic term coordinate. An absent mapping is never inferred from a
    /// source-map region or another term's mapping.
    pub fn protocol_location(
        &self,
        application: &CheckedNodeId,
        occurrence: &CheckedOccurrence,
        term_ordinal: u64,
        work_limit: u64,
    ) -> Result<&CheckedProtocolSemanticLocation, CheckedProtocolLocationReadError> {
        let data = self
            .wire
            .checked_term_data
            .as_ref()
            .ok_or(CheckedProtocolLocationReadError::Unavailable)?;
        let mut consumed = 0_u64;
        for (index, row) in data.protocol_locations.iter().enumerate() {
            charge(&mut consumed, work_limit, index, None)?;
            if &row.application != application
                || &row.occurrence != occurrence
                || row.term_ordinal != term_ordinal
            {
                continue;
            }
            for child in 0..row.location.path.len() {
                charge(&mut consumed, work_limit, index, Some(child))?;
            }
            return Ok(row);
        }
        Err(CheckedProtocolLocationReadError::Missing)
    }
}

fn charge(
    consumed: &mut u64,
    limit: u64,
    row: usize,
    child: Option<usize>,
) -> Result<(), CheckedProtocolLocationReadError> {
    let next = consumed.checked_add(1);
    *consumed = next.unwrap_or(u64::MAX);
    if next.is_none() || *consumed > limit {
        return Err(CheckedProtocolLocationReadError::WorkExhausted {
            limit,
            consumed: *consumed,
            row,
            child,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::ValueForm;
    use super::*;
    use ix_trace_rs::trace;
    use serde_json::json;

    #[test]
    #[trace("TC-491", "FR-038-AC-597")]
    fn empty_location_rows_build_no_join_indexes_or_work() {
        let mut meter = WorkMeter::new(0);
        let node: CheckedSemanticNodeV2 = serde_json::from_value(json!({
            "node_id":{"package":"example","path":["leaf"],"ordinal":0},
            "schema_version":"filament.checked-semantic-graph/v2",
            "node_tag":"value","semantic_form":"literal",
            "semantic_type":{"package":"example","path":["leaf"],"ordinal":0},
            "dependencies":[],"occurrences":[{"role":"expression","ordinal":0}],
            "body":{"term":"aggregate","members":[]}
        }))
        .unwrap();
        let nodes = std::slice::from_ref(&node);
        let (gap, index) = validate_rows::<_, CheckedProtocolSemanticLocation>(
            "protocol_locations",
            "example",
            nodes,
            &[],
            &[],
            &[],
            &[],
            &mut meter,
        )
        .unwrap();
        assert!(!gap);
        assert!(index.is_empty());
        assert_eq!(meter.consumed(), 0);
    }

    #[test]
    #[trace("TC-048", "FR-038-AC-448")]
    fn every_value_form_and_reference_is_ineligible_for_a_location_row() {
        let id = json!({"package":"example","path":["value","literal","leaf"],"ordinal":0});
        let node: CheckedSemanticNodeV2 = serde_json::from_value(json!({
            "node_id": id, "schema_version":"filament.checked-semantic-graph/v2",
            "node_tag":"value", "semantic_form":"literal", "semantic_type":id,
            "dependencies":[], "occurrences":[{"role":"expression","ordinal":0}],
            "body":{"term":"reference","target":id}
        }))
        .unwrap();
        let row: CheckedProtocolSemanticLocation = serde_json::from_value(json!({
            "application":id, "occurrence":{"role":"expression","ordinal":0},
            "term_ordinal":0,
            "location":{"origin":{"kind":"type_declaration","name":"protocol P"},"path":[]}
        }))
        .unwrap();
        let term = CheckedTerm::Reference {
            target: node.node_id.clone(),
        };
        for kind in ValueForm::ALL
            .iter()
            .copied()
            .map(CheckedNodeKind::Value)
            .chain([CheckedNodeKind::Expression(ExpressionForm::Reference)])
        {
            let result = validate_rows(
                "protocol_locations",
                "example",
                std::slice::from_ref(&node),
                &[kind],
                std::slice::from_ref(&term),
                &[],
                std::slice::from_ref(&row),
                &mut WorkMeter::new(u64::MAX),
            );
            let Err(ValidationFailure::Refused(refusal)) = result else {
                panic!("{kind:?} must refuse its location row: {result:?}");
            };
            assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
            assert_eq!(
                refusal.cause,
                Some(CheckedPackageRefusalCause::InvalidValue)
            );
            assert_eq!(
                refusal.loci[0].path.as_ref().unwrap().as_str(),
                "/checked_term_data/protocol_locations/0/application",
                "{kind:?}"
            );
        }
    }
}
