// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Typed clock configuration rows for temporal and compensation declarations.

use super::graph_storage::GraphIndex;
use super::term_view::TermNode;
use super::{
    CheckedNodeId, CheckedNodeKind, CheckedOccurrenceRole, CheckedPackageLockV2,
    CheckedSelectionRole, CheckedSemanticNodeV2, LawRole, ProtocolForm, TemporalForm,
    TemporalProfile, WorkMeter,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{CheckedPackageRefusalCause, CheckedPackageRefusalCode};
use quire_canonical::{Encode, Error, FixedShape, Sink, Writer};
use serde::{Deserialize, Serialize};

/// An exact number in a fixed-sample clock configuration.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, FixedShape)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClockExactNumber {
    /// A canonical signed decimal integer.
    Integer {
        /// Decimal spelling without leading zeroes or negative zero.
        decimal: Box<str>,
    },
    /// A reduced rational with a positive denominator.
    Rational {
        /// Canonical signed numerator.
        numerator: Box<str>,
        /// Canonical positive denominator.
        denominator: Box<str>,
    },
}

/// The four closed clock configurations selected by temporal profiles.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, FixedShape)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClockConfiguration {
    /// Discrete positions from a named event sequence.
    EventPosition {
        /// Nonempty sequence authority.
        sequence_authority: Box<str>,
    },
    /// Exact samples at a fixed period from an epoch.
    FixedSample {
        /// Exact epoch.
        epoch: ClockExactNumber,
        /// Strictly positive exact period.
        period: ClockExactNumber,
        /// Nonempty unit name.
        unit: Box<str>,
    },
    /// Timestamps supplied by events.
    TimestampedEvent {
        /// Nonempty timestamp unit.
        timestamp_unit: Box<str>,
    },
    /// Time-stamped positions of a trace subject under the timed profile.
    Timed {
        /// Nonempty time unit the caller's clock selection binds the role to.
        unit: Box<str>,
    },
}

/// One admitted clock selection keyed by a declaration and source occurrence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalBinding {
    /// Exact ID of the temporal clause, compensation, await or
    /// `claim`/`hyperproperty` node the row selects a clock for; never a
    /// hyperproperty's body wrapper.
    pub declaration: CheckedNodeId,
    /// Ordinal of the clause's or hyperproperty's claim occurrence, or of the
    /// compensation's declaration occurrence.
    pub occurrence: u64,
    /// Exact nonempty clock name: the declaration's clock argument (a
    /// hyperproperty's wrapper's, under a behaviours domain) or, for a
    /// finite-domain hyperproperty, the clock role its occurrence's rows share.
    pub clock_name: Box<str>,
    /// Closed clock configuration.
    pub clock: ClockConfiguration,
    /// For a `claim`/`hyperproperty` node's row, the exact name of the quantifier
    /// Group variable whose trace the row's clock serves; absent on every other
    /// row.
    #[serde(
        default,
        deserialize_with = "present_trace",
        skip_serializing_if = "Option::is_none"
    )]
    pub trace: Option<Box<str>>,
}

/// A `trace` member that is present is a string; `null` is not an absent one.
fn present_trace<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Box<str>>, D::Error> {
    Box::<str>::deserialize(deserializer).map(Some)
}

impl Encode for TemporalBinding {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        super::encode::encoded(writer, "declaration", &self.declaration)?;
        super::encode::member(writer, "occurrence", &self.occurrence)?;
        super::encode::member(writer, "clock_name", self.clock_name.as_ref())?;
        super::encode::member(writer, "clock", &self.clock)?;
        if let Some(trace) = &self.trace {
            super::encode::member(writer, "trace", trace.as_ref())?;
        }
        writer.end_object()
    }
}

fn row_path(index: usize) -> super::stage_storage::Path<'static> {
    super::stage_storage::Path::root()
        .key("temporal_bindings")
        .index(index)
}

fn binding_work(
    meter: &mut WorkMeter,
    path: super::stage_storage::Path<'_>,
) -> Result<(), ValidationFailure> {
    let site = super::natural::site(meter, path);
    super::natural::work_at(meter, site, 1)
}

fn malformed(meter: &mut WorkMeter, at: super::stage_storage::Path<'_>) -> ValidationFailure {
    super::stage_storage::refuse(
        meter,
        &super::natural::site(meter, at),
        super::stage_storage::Refusal::new(CheckedPackageRefusalCode::MalformedWire, at),
    )
}

fn invalid(
    meter: &mut WorkMeter,
    at: super::stage_storage::Path<'_>,
    declaration: &CheckedNodeId,
) -> ValidationFailure {
    binding_refusal(
        meter,
        at,
        declaration,
        CheckedPackageRefusalCause::TimedNodeReference,
    )
}

fn binding_refusal(
    meter: &mut WorkMeter,
    at: super::stage_storage::Path<'_>,
    declaration: &CheckedNodeId,
    cause: CheckedPackageRefusalCause,
) -> ValidationFailure {
    super::stage_storage::refuse(
        meter,
        &super::natural::site(meter, at),
        super::stage_storage::Refusal::new(CheckedPackageRefusalCode::InvalidPackage, at)
            .because(cause)
            .at_node(declaration),
    )
}

#[ix_trace_rs::string_edge]
fn decimal_magnitude(text: &str) -> Option<(&str, bool)> {
    if text == "0" {
        return Some(("0", true));
    }
    let digits = text.strip_prefix('-').unwrap_or(text);
    let mut bytes = digits.bytes();
    (bytes
        .next()
        .is_some_and(|first| (b'1'..=b'9').contains(&first))
        && bytes.all(|byte| byte.is_ascii_digit()))
    .then_some((digits, false))
}

fn validate_number(
    number: &ClockExactNumber,
    positive: bool,
    at: super::stage_storage::Path<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    match number {
        ClockExactNumber::Integer { decimal } => {
            let Some((_, zero)) = decimal_magnitude(decimal) else {
                return Err(malformed(meter, at.key("decimal")));
            };
            if positive && (zero || decimal.starts_with('-')) {
                return Err(malformed(meter, at.key("decimal")));
            }
        }
        ClockExactNumber::Rational {
            numerator,
            denominator,
        } => {
            let Some((digits, zero)) = decimal_magnitude(numerator) else {
                return Err(malformed(meter, at.key("numerator")));
            };
            let valid_denominator = denominator
                .bytes()
                .next()
                .is_some_and(|first| (b'1'..=b'9').contains(&first))
                && denominator
                    .bytes()
                    .skip(1)
                    .all(|byte| byte.is_ascii_digit());
            if !valid_denominator {
                return Err(malformed(meter, at.key("denominator")));
            }
            if positive && (zero || numerator.starts_with('-')) {
                return Err(malformed(meter, at.key("numerator")));
            }
            if !super::natural::coprime(
                digits,
                denominator,
                meter,
                super::natural::site(meter, at),
            )? {
                return Err(malformed(meter, at));
            }
        }
    }
    Ok(())
}

fn validate_shape(
    rows: &[TemporalBinding],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    for (index, row) in rows.iter().enumerate() {
        meter.at_row("temporal_bindings", index);
        let at = || row_path(index);
        binding_work(
            meter,
            super::stage_storage::Path::root()
                .key("temporal_bindings")
                .index(index),
        )?;
        if row.clock_name.is_empty() {
            return Err(malformed(meter, at().key("clock_name")));
        }
        if row.trace.as_deref().is_some_and(str::is_empty) {
            return Err(malformed(meter, at().key("trace")));
        }
        let clock_at = || at().key("clock");
        match &row.clock {
            ClockConfiguration::EventPosition { sequence_authority } => {
                if sequence_authority.is_empty() {
                    return Err(malformed(meter, clock_at().key("sequence_authority")));
                }
            }
            ClockConfiguration::FixedSample {
                epoch,
                period,
                unit,
            } => {
                if unit.is_empty() {
                    return Err(malformed(meter, clock_at().key("unit")));
                }
                validate_number(
                    epoch,
                    false,
                    super::stage_storage::Path::root()
                        .key("temporal_bindings")
                        .index(index)
                        .key("clock")
                        .key("epoch"),
                    meter,
                )?;
                validate_number(
                    period,
                    true,
                    super::stage_storage::Path::root()
                        .key("temporal_bindings")
                        .index(index)
                        .key("clock")
                        .key("period"),
                    meter,
                )?;
            }
            ClockConfiguration::TimestampedEvent { timestamp_unit } => {
                if timestamp_unit.is_empty() {
                    return Err(malformed(meter, clock_at().key("timestamp_unit")));
                }
            }
            ClockConfiguration::Timed { unit } => {
                if unit.is_empty() {
                    return Err(malformed(meter, clock_at().key("unit")));
                }
            }
        }
    }
    Ok(())
}

/// Whether `value` is a string.
fn is_text(value: Option<TermNode<'_>>) -> bool {
    value.is_some_and(|value| value.as_str().is_some())
}

/// Whether `law` is an operation law: a `role` text and a `definition` of
/// exactly an `authority` and an `identity`.
fn is_operation_law(law: TermNode<'_>) -> bool {
    law.exact_members(&["role", "definition"])
        && is_text(law.get("role"))
        && law.get("definition").is_some_and(|definition| {
            definition.exact_members(&["authority", "identity"])
                && is_text(definition.get("authority"))
                && is_text(definition.get("identity"))
        })
}

/// Whether `mode` is null or exactly a `kind` and a `value` text.
fn is_operation_mode(mode: TermNode<'_>) -> bool {
    mode.is_null()
        || (mode.exact_members(&["kind", "value"])
            && is_text(mode.get("kind"))
            && is_text(mode.get("value")))
}

/// Whether `leaf` is an operation leaf: a `path` of texts, `laws`, and an
/// optional `mode`, with no other member.
#[ix_trace_rs::string_edge]
fn is_operation_leaf(leaf: TermNode<'_>) -> bool {
    let Some(members) = leaf.members() else {
        return false;
    };
    let (mut path, mut laws) = (false, false);
    for (name, value) in members {
        let fine = match name {
            "path" => {
                path = true;
                value
                    .items()
                    .is_some_and(|mut items| items.all(|item| item.as_str().is_some()))
            }
            "laws" => {
                laws = true;
                value
                    .items()
                    .is_some_and(|mut items| items.all(is_operation_law))
            }
            "mode" => is_operation_mode(value),
            _ => false,
        };
        if !fine {
            return false;
        }
    }
    path && laws
}

/// Whether `operation` has the closed shape `OperationWire` decodes: the five
/// members, `mode` and `member` present (null allowed).
fn is_operation_wire(operation: TermNode<'_>) -> bool {
    operation.exact_members(&["identity", "laws", "mode", "member", "leaves"])
        && is_text(operation.get("identity"))
        && operation
            .get("laws")
            .and_then(TermNode::items)
            .is_some_and(|mut laws| laws.all(is_operation_law))
        && operation.get("mode").is_some_and(is_operation_mode)
        && operation
            .get("leaves")
            .and_then(TermNode::items)
            .is_some_and(|mut leaves| leaves.all(is_operation_leaf))
}

/// The one temporal-profile law of the operation of graph node `position`,
/// read from the retained document; its two texts are charged before they are
/// copied. `None` when the body is not a decodable operation, or does not
/// carry exactly one such law.
pub(super) fn profile_law(
    position: usize,
    meter: &mut WorkMeter,
) -> Result<Option<super::CheckedArtifactRef>, ValidationFailure> {
    let Some((body, _)) = meter.body(position) else {
        return Ok(None);
    };
    let Some(operation) = body.get("operation").filter(|op| is_operation_wire(*op)) else {
        return Ok(None);
    };
    let Some(mut laws) = operation.get("laws").and_then(TermNode::items) else {
        return Ok(None);
    };
    let (Some(law), None) = (laws.next(), laws.next()) else {
        return Ok(None);
    };
    let role = law.get("role").and_then(TermNode::as_str);
    if role.and_then(LawRole::from_wire) != Some(LawRole::TemporalProfile) {
        return Ok(None);
    }
    let Some(definition) = law.get("definition") else {
        return Ok(None);
    };
    let (Some(authority), Some(identity)) = (
        definition.get("authority").and_then(TermNode::as_str),
        definition.get("identity").and_then(TermNode::as_str),
    ) else {
        return Ok(None);
    };
    meter.at_node(position, "profile_law");
    let authority = meter.boxed_str(authority)?;
    let identity = meter.boxed_str(identity)?;
    Ok(Some(super::CheckedArtifactRef {
        authority,
        identity,
    }))
}

fn clock_configured(profile: TemporalProfile) -> Option<TemporalProfile> {
    matches!(
        profile,
        TemporalProfile::EventPositionFalseExtension
            | TemporalProfile::FixedSampleFalseExtension
            | TemporalProfile::TimestampedEventFiniteWindow
    )
    .then_some(profile)
}

/// The clock-configured profile of the clause at `position`. The timed
/// profile is clock-configured only over a trace subject: a timed-model
/// subject takes its time binding from the model and has no row.
fn configured_profile(
    position: usize,
    model_subject: bool,
    meter: &mut WorkMeter,
) -> Result<Option<TemporalProfile>, ValidationFailure> {
    Ok(profile_law(position, meter)?
        .and_then(|law| TemporalProfile::from_wire(&law.identity))
        .and_then(|profile| {
            if profile == TemporalProfile::Timed {
                (!model_subject).then_some(profile)
            } else {
                clock_configured(profile)
            }
        }))
}

fn selected_profile(lock: &CheckedPackageLockV2) -> Option<TemporalProfile> {
    let selection = lock
        .profile_selections
        .iter()
        .find(|selection| selection.role == CheckedSelectionRole::TemporalProfile)?;
    clock_configured(TemporalProfile::from_wire(&selection.definition.identity)?)
}

/// The text of argument `argument` of the clause at `position`, a literal
/// text or a binding of one, borrowed from the retained document.
#[ix_trace_rs::string_edge]
pub(super) fn clock_name<'b>(
    meter: &WorkMeter<'b>,
    position: usize,
    argument: usize,
) -> Option<&'b str> {
    let (body, _) = meter.body(position)?;
    let term = body.get("arguments")?.items()?.nth(argument)?;
    let term = if term.get("term")?.as_str()? == "binding" {
        term.get("value")?
    } else {
        term
    };
    (term.get("term")?.as_str()? == "literal" && term.get("value_kind")?.as_str()? == "text")
        .then(|| term.get("value")?.as_str())?
}

fn clock_matches(profile: Option<TemporalProfile>, clock: &ClockConfiguration) -> bool {
    matches!(
        (profile, clock),
        (
            Some(TemporalProfile::EventPositionFalseExtension),
            ClockConfiguration::EventPosition { .. }
        ) | (
            Some(TemporalProfile::FixedSampleFalseExtension),
            ClockConfiguration::FixedSample { .. }
        ) | (
            Some(TemporalProfile::TimestampedEventFiniteWindow),
            ClockConfiguration::TimestampedEvent { .. }
        ) | (
            Some(TemporalProfile::Timed),
            ClockConfiguration::Timed { .. }
        )
    )
}

/// One `claim`/`hyperproperty` node: its quantified trace variables in
/// `quantifiers` order, whether its domain is finite, and, under a behaviours
/// domain, the body wrapper it references. A node whose body is not readable as
/// that shape lists no variables; the hyperproperty step owns its refusal.
struct HyperNode<'t> {
    position: usize,
    variables: Vec<&'t str>,
    finite: bool,
    wrapper: Option<usize>,
}

#[ix_trace_rs::string_edge]
fn hyper_nodes<'t>(
    kinds: &[CheckedNodeKind],
    terms: &'t [Option<super::CheckedTerm>],
    index: &GraphIndex<'_>,
    meter: &mut WorkMeter,
) -> Result<Vec<HyperNode<'t>>, ValidationFailure> {
    use super::{CheckedTerm, LiteralKind};
    let mut hypers: Vec<HyperNode<'t>> = meter.vec(0)?;
    for (position, kind) in kinds.iter().enumerate() {
        if *kind != CheckedNodeKind::Claim(super::ClaimForm::Hyperproperty) {
            continue;
        }
        let mut variables = meter.vec(0)?;
        let mut finite = false;
        let mut wrapper = None;
        if let Some(Some(CheckedTerm::Aggregate { members })) = terms.get(position) {
            let bound = |name: &str| {
                members.iter().find_map(|member| match member {
                    CheckedTerm::Binding { name: found, value }
                        if AsRef::<str>::as_ref(found) == name =>
                    {
                        Some(value.as_ref())
                    }
                    _ => None,
                })
            };
            finite = bound("domain")
                .and_then(super::structural::typed_reference)
                .is_some();
            if !finite {
                wrapper = bound("body")
                    .and_then(super::structural::typed_reference)
                    .and_then(|target| index.get(target).copied())
                    .filter(|target| {
                        kinds[*target] == CheckedNodeKind::Temporal(TemporalForm::TemporalClause)
                    });
            }
            if let Some(CheckedTerm::Aggregate { members: groups }) = bound("quantifiers") {
                for group in groups {
                    let CheckedTerm::Aggregate { members } = group else {
                        continue;
                    };
                    let variable = members.iter().find_map(|member| match member {
                        CheckedTerm::Binding { name, value }
                            if AsRef::<str>::as_ref(name) == "variable" =>
                        {
                            match value.as_ref() {
                                CheckedTerm::Literal {
                                    value_kind: LiteralKind::Text,
                                    value,
                                    ..
                                } => value.as_str(),
                                _ => None,
                            }
                        }
                        _ => None,
                    });
                    if let Some(variable) = variable {
                        meter.push(&mut variables, variable)?;
                    }
                }
            }
        }
        meter.push(
            &mut hypers,
            HyperNode {
                position,
                variables,
                finite,
                wrapper,
            },
        )?;
    }
    Ok(hypers)
}

/// Validate row coverage before per-node joins and compare names before kinds.
pub(super) fn validate<B>(
    rows: Option<&[TemporalBinding]>,
    nodes: &[CheckedSemanticNodeV2<B>],
    kinds: &[CheckedNodeKind],
    terms: &[Option<super::CheckedTerm>],
    index: &GraphIndex<'_>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    lock: &CheckedPackageLockV2,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let rows = rows.unwrap_or_default();
    validate_shape(rows, meter)?;
    let compensation_profile = selected_profile(lock);
    let hypers = hyper_nodes(kinds, terms, index, meter)?;
    // `hypers` ascends by graph position, so a node is found by bisection; the
    // wrappers are listed once and sorted.
    let hyper_of = |position: usize| {
        hypers
            .binary_search_by_key(&position, |hyper| hyper.position)
            .ok()
            .map(|at| &hypers[at])
    };
    let mut wrappers: Vec<usize> = meter.vec(hypers.len())?;
    for hyper in &hypers {
        if let Some(wrapper) = hyper.wrapper {
            meter.push(&mut wrappers, wrapper)?;
        }
    }
    wrappers.sort_unstable();
    let is_wrapper = |position: usize| wrappers.binary_search(&position).is_ok();
    let traces_of = |position: usize| hyper_of(position).map(|hyper| hyper.variables.as_slice());
    let mut seen: Vec<(usize, u64, usize)> = meter.vec(rows.len())?;
    let mut previous: Option<(super::graph_storage::ResolvedKey<'_>, u64, usize)> = None;
    let mut resolved = meter.vec(rows.len())?;
    for (row_index, row) in rows.iter().enumerate() {
        let at = || row_path(row_index);
        let Some(&position) = index.get(&row.declaration) else {
            return Err(invalid(meter, at().key("declaration"), &row.declaration));
        };
        if !matches!(
            kinds[position],
            CheckedNodeKind::Temporal(TemporalForm::TemporalClause)
                | CheckedNodeKind::Protocol(ProtocolForm::Compensation)
                | CheckedNodeKind::Claim(super::ClaimForm::Hyperproperty)
        ) || is_wrapper(position)
        {
            return Err(invalid(meter, at().key("declaration"), &row.declaration));
        }
        if kinds[position] == CheckedNodeKind::Temporal(TemporalForm::TemporalClause)
            && profile_law(position, meter)?
                .as_ref()
                .and_then(|law| TemporalProfile::from_wire(&law.identity))
                == Some(TemporalProfile::Timed)
            && super::timed_model::is_model_subject_clause(nodes, kinds, terms, index, position)
        {
            return Err(invalid(meter, at(), &row.declaration));
        }
        let role = if kinds[position] == CheckedNodeKind::Protocol(ProtocolForm::Compensation) {
            CheckedOccurrenceRole::Declaration
        } else {
            CheckedOccurrenceRole::Claim
        };
        let mut owned = false;
        for occurrence in &nodes[position].occurrences {
            binding_work(
                meter,
                super::stage_storage::Path::root()
                    .key("temporal_bindings")
                    .index(row_index)
                    .key("occurrence"),
            )?;
            if occurrence.role == role && occurrence.ordinal == row.occurrence {
                owned = true;
                break;
            }
        }
        if !owned {
            return Err(invalid(meter, at().key("occurrence"), &row.declaration));
        }
        // A hyperproperty row names one of the node's quantified traces, and
        // no other row names a trace.
        let trace_at = match (traces_of(position), row.trace.as_deref()) {
            (None, None) => 0,
            (Some(variables), Some(trace)) => {
                let Some(trace_at) = variables.iter().position(|name| *name == trace) else {
                    return Err(invalid(meter, at().key("trace"), &row.declaration));
                };
                trace_at
            }
            _ => return Err(invalid(meter, at().key("trace"), &row.declaration)),
        };
        let key = (position, row.occurrence, trace_at);
        let insertion = seen.binary_search(&key);
        if insertion.is_ok() {
            return Err(binding_refusal(
                meter,
                at(),
                &row.declaration,
                CheckedPackageRefusalCause::ConflictingDefinition,
            ));
        }
        seen.insert(insertion.expect_err("duplicate refused above"), key);
        let structural = index.resolved_at(position, structural_keys);
        if let Some((previous_id, previous_occurrence, previous_trace)) = previous {
            if previous_id > structural {
                return Err(invalid(meter, at().key("declaration"), &row.declaration));
            }
            if previous_id == structural && previous_occurrence > row.occurrence {
                return Err(invalid(meter, at().key("occurrence"), &row.declaration));
            }
            if previous_id == structural
                && previous_occurrence == row.occurrence
                && previous_trace > trace_at
            {
                return Err(invalid(meter, at().key("trace"), &row.declaration));
            }
        }
        previous = Some((structural, row.occurrence, trace_at));
        resolved.push(position);
    }
    let mut missing = None;
    for (position, node) in nodes.iter().enumerate() {
        let role = if is_wrapper(position) {
            continue;
        } else if let Some(hyper) = hyper_of(position) {
            let configured = if hyper.finite {
                compensation_profile.is_some()
            } else {
                match hyper.wrapper {
                    Some(wrapper) => configured_profile(wrapper, true, meter)?.is_some(),
                    None => false,
                }
            };
            if !configured {
                continue;
            }
            CheckedOccurrenceRole::Claim
        } else if kinds[position] == CheckedNodeKind::Temporal(TemporalForm::TemporalClause)
            && configured_profile(
                position,
                super::timed_model::is_model_subject_clause(nodes, kinds, terms, index, position),
                meter,
            )?
            .is_some()
        {
            CheckedOccurrenceRole::Claim
        } else if kinds[position] == CheckedNodeKind::Protocol(ProtocolForm::Compensation)
            && compensation_profile.is_some()
        {
            CheckedOccurrenceRole::Declaration
        } else {
            continue;
        };
        for occurrence in &node.occurrences {
            binding_work(
                meter,
                super::stage_storage::Path::root()
                    .key("semantic_graph")
                    .key("nodes")
                    .index(position)
                    .key("body"),
            )?;
            let rows_needed = traces_of(position).map_or(1, <[&str]>::len);
            if occurrence.role == role
                && (0..rows_needed).any(|trace| {
                    // A variable a later Group repeats is the key of its first.
                    let repeated = traces_of(position)
                        .is_some_and(|variables| variables[..trace].contains(&variables[trace]));
                    !repeated
                        && seen
                            .binary_search(&(position, occurrence.ordinal, trace))
                            .is_err()
                })
            {
                missing = Some(position);
                break;
            }
        }
        if missing.is_some() {
            break;
        }
    }
    if let Some(position) = missing {
        return Err(invalid(
            meter,
            super::stage_storage::Path::root()
                .key("semantic_graph")
                .key("nodes")
                .index(position)
                .key("body"),
            &nodes[position].node_id,
        ));
    }
    for (row_index, (row, &position)) in rows.iter().zip(&resolved).enumerate() {
        let at = || row_path(row_index);
        let hyper = hyper_of(position);
        if hyper.is_some_and(|hyper| hyper.finite) {
            // The rows of one occurrence share one clock role.
            if row_index > 0
                && rows[row_index - 1].declaration == row.declaration
                && rows[row_index - 1].occurrence == row.occurrence
                && rows[row_index - 1].clock_name != row.clock_name
            {
                return Err(invalid(meter, at().key("clock_name"), &row.declaration));
            }
            if !clock_matches(compensation_profile, &row.clock) {
                return Err(invalid(
                    meter,
                    at().key("clock").key("kind"),
                    &row.declaration,
                ));
            }
            continue;
        }
        // A behaviours node reads its profile and clock name from its wrapper.
        let position = hyper.and_then(|hyper| hyper.wrapper).unwrap_or(position);
        if kinds[position] == CheckedNodeKind::Protocol(ProtocolForm::Compensation) {
            if clock_name(meter, position, 4) != Some(row.clock_name.as_ref()) {
                return Err(invalid(meter, at().key("clock_name"), &row.declaration));
            }
            if !clock_matches(compensation_profile, &row.clock) {
                return Err(invalid(
                    meter,
                    at().key("clock").key("kind"),
                    &row.declaration,
                ));
            }
            continue;
        }
        let law = profile_law(position, meter)?;
        if law
            .as_ref()
            .is_some_and(|law| TemporalProfile::from_wire(&law.identity).is_none())
        {
            // The temporal step owns unknown-profile classification at the
            // law definition. Do not let this row's profile join preempt it.
            continue;
        }
        if clock_name(meter, position, 1) != Some(row.clock_name.as_ref()) {
            return Err(invalid(meter, at().key("clock_name"), &row.declaration));
        }
        let profile = configured_profile(position, false, meter)?;
        if profile == Some(TemporalProfile::Timed)
            && (!matches!(row.clock, ClockConfiguration::Timed { .. })
                || matches!(row.clock_name.as_ref(), "model-time" | "model-steps"))
        {
            // A trace-subject timed clause takes a `timed` row, and a clock
            // role never takes a model-subject binding name: the whole row
            // is refused, as FSpec's TN vectors place it.
            return Err(invalid(meter, at(), &row.declaration));
        }
        let mut selected = false;
        if let Some(law) = law.as_ref() {
            for selection in &lock.profile_selections {
                binding_work(
                    meter,
                    super::stage_storage::Path::root()
                        .key("temporal_bindings")
                        .index(row_index)
                        .key("clock")
                        .key("kind"),
                )?;
                if selection.role == CheckedSelectionRole::TemporalProfile
                    && selection.definition == *law
                {
                    selected = true;
                    break;
                }
            }
        }
        if !selected || !clock_matches(profile, &row.clock) {
            return Err(invalid(
                meter,
                at().key("clock").key("kind"),
                &row.declaration,
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::graph_storage::OwnedBodies;
    use super::super::native_join_budget::NativeJoinBudget;
    use super::*;
    use ix_trace_rs::trace;
    use serde_json::json;

    fn node_body(operation: serde_json::Value, clock: &str) -> serde_json::Value {
        json!({"term":"application","operation":operation,"arguments":[
            {"term":"binding","name":"clock","value":
                {"term":"literal","kind":"text","value_kind":"text","value":clock}},
            {"term":"literal","value_kind":"text","value":clock}]})
    }

    fn operation(laws: serde_json::Value) -> serde_json::Value {
        json!({"identity":"filament.op.temporal.always","laws":laws,"mode":null,"member":null,"leaves":[]})
    }

    fn law(role: &str, identity: &str) -> serde_json::Value {
        json!({"role":role,"definition":{"authority":"filament","identity":identity}})
    }

    /// The profile law and the clock name read from the document agree with
    /// the typed decode of the same bodies, and the law's two texts are
    /// allocated exactly as charged (FR-038-AC-504).
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn profile_law_and_clock_name_read_the_document_as_the_decode_does() {
        let profile = law("temporal_profile", "filament.profile.timed");
        let cases = [
            operation(json!([profile.clone()])),
            operation(json!([])),
            operation(json!([profile.clone(), profile.clone()])),
            operation(json!([law("other", "x")])),
            json!({"identity":"x","laws":[profile.clone()],"mode":null,"leaves":[]}),
            json!({"identity":"x","laws":[profile.clone()],"mode":null,"member":null,"leaves":[],"extra":1}),
            json!({"identity":"x","laws":[profile.clone()],"mode":{"kind":"k","value":3},"member":null,"leaves":[]}),
            json!({"identity":"x","laws":[profile.clone()],"mode":null,"member":null,
                   "leaves":[{"path":["a"],"laws":[],"mode":null}]}),
            json!({"identity":"x","laws":[profile.clone()],"mode":null,"member":null,
                   "leaves":[{"path":["a"]}]}),
            json!({"identity":"x","laws":[profile.clone()],"mode":null,"member":null,
                   "leaves":[{"path":[1],"laws":[]}]}),
            json!({"identity":"x","laws":[{"role":"temporal_profile","definition":
                   {"authority":"q","identity":"i","more":1}}],"mode":null,"member":null,"leaves":[]}),
        ];
        for (n, case) in cases.iter().enumerate() {
            let body = node_body(case.clone(), "model-time");
            let expected =
                serde_json::from_value::<super::super::operations::OperationWire>(case.clone())
                    .ok()
                    .and_then(|wire| {
                        let [law] = wire.laws.as_slice() else {
                            return None;
                        };
                        (law.role_class() == Some(LawRole::TemporalProfile))
                            .then(|| law.definition.clone())
                    });
            let bodies = OwnedBodies::of_values(&[body], vec![Vec::new()]);
            let mut meter = WorkMeter::new(u64::MAX);
            bodies.attach(&mut meter);
            let read = profile_law(0, &mut meter).expect("an unlimited meter admits");
            assert_eq!(read, expected, "case {n}");
            assert_eq!(clock_name(&meter, 0, 1), Some("model-time"), "case {n}");
            assert_eq!(clock_name(&meter, 0, 0), Some("model-time"), "case {n}");
            assert_eq!(clock_name(&meter, 0, 2), None, "case {n}");
        }

        // The law's two texts are charged before they are copied, to the byte.
        let body = node_body(cases[0].clone(), "model-time");
        let bodies = OwnedBodies::of_values(&[body], vec![Vec::new()]);
        let run = |limit: u64| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut outcome = None;
            let measured = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                bodies.attach(&mut meter);
                outcome = Some(profile_law(0, &mut meter));
            });
            (
                outcome.expect("ran").ok().flatten(),
                measured.bytes_total,
                budget.read_storage_consumed(),
            )
        };
        let (law, allocated, charged) = run(u64::MAX);
        assert!(law.is_some() && charged > 0);
        assert_eq!(allocated, charged, "allocated == requested");
        assert!(run(charged).0.is_some(), "the measured limit admits");
        assert!(run(charged - 1).0.is_none(), "one byte short is denied");
    }
    #[trace("TC-472", "FR-038-AC-504", "FR-038-AC-510", "FR-038-AC-511")]
    #[test]
    fn temporal_binding_tables_allocate_only_charged_storage() {
        let id = super::super::test_support::node_id('1');
        let nodes = [CheckedSemanticNodeV2 {
            node_id: id.clone(),
            schema_version: "test".into(),
            node_tag: "protocol".into(),
            semantic_form: "compensation".into(),
            semantic_type: id.clone(),
            dependencies: vec![],
            occurrences: vec![
                crate::checked_package::shared::CheckedOccurrence {
                    role: CheckedOccurrenceRole::Declaration,
                    ordinal: 0,
                },
                crate::checked_package::shared::CheckedOccurrence {
                    role: CheckedOccurrenceRole::Declaration,
                    ordinal: 1,
                },
            ],
            recursion_group: None,
            declaration: None,
            owner: None,
            owner_model: None,
            check_mode: None,
            body: (),
        }];
        let rows: Vec<_> = (0..2)
            .map(|occurrence| TemporalBinding {
                declaration: id.clone(),
                occurrence,
                clock_name: "events".into(),
                clock: ClockConfiguration::EventPosition {
                    sequence_authority: "source".into(),
                },
                trace: None,
            })
            .collect();
        let lock: CheckedPackageLockV2 = serde_json::from_value(json!({
            "sources":[], "edition":{"role":"edition","definition":{"authority":"test","identity":"edition"}},
            "profile_selections":[{"role":"temporal_profile","definition":{"authority":"filament","identity":"filament.temporal.event-position.false-extension/v1"}}],
            "definition_selections":[],"required_features":[],"dependency_selections":[]
        })).expect("fixture lock");
        let bodies = OwnedBodies::of_values(
            &[json!({"arguments":[null,null,null,null,
            {"term":"literal","value_kind":"text","value":"events"}]})],
            vec![Vec::new()],
        );
        let mut build_meter = WorkMeter::new(u64::MAX);
        let index = GraphIndex::build(nodes.iter().map(|node| &node.node_id), &mut build_meter)
            .expect("index");
        let keys = [std::sync::Arc::new(id)];
        let kinds = [CheckedNodeKind::Protocol(ProtocolForm::Compensation)];
        let run = |limit| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let measured = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                bodies.attach(&mut meter);
                result = Some(validate(
                    Some(&rows),
                    &nodes,
                    &kinds,
                    &[None],
                    &index,
                    &keys,
                    &lock,
                    &mut meter,
                ));
            });
            (
                result.expect("ran"),
                measured.bytes_total,
                budget.read_storage_consumed(),
            )
        };
        let (outcome, allocated, charged) = run(u64::MAX);
        outcome.expect("valid rows");
        assert!(charged > 0);
        assert_eq!(allocated, charged);
        run(charged).0.expect("exact allowance admits");
        assert!(run(charged - 1).0.is_err(), "one byte short refuses");

        // Mutate the existing admitted table to reach the actual strict shape,
        // reference, conflict, missing-binding and clock-name refusal paths.
        for case in 0..6 {
            let mut refused_rows = rows.clone();
            let (path, cause, located) = match case {
                0 => {
                    refused_rows[0].clock_name = "".into();
                    ("/temporal_bindings/0/clock_name", None, false)
                }
                1 => {
                    refused_rows[0].declaration = super::super::test_support::node_id('2');
                    (
                        "/temporal_bindings/0/declaration",
                        Some(CheckedPackageRefusalCause::TimedNodeReference),
                        true,
                    )
                }
                2 => {
                    refused_rows[1].occurrence = 0;
                    (
                        "/temporal_bindings/1",
                        Some(CheckedPackageRefusalCause::ConflictingDefinition),
                        true,
                    )
                }
                3 => {
                    refused_rows.pop();
                    (
                        "/semantic_graph/nodes/0/body",
                        Some(CheckedPackageRefusalCause::TimedNodeReference),
                        true,
                    )
                }
                4 => {
                    refused_rows[0].clock_name = "wrong".into();
                    (
                        "/temporal_bindings/0/clock_name",
                        Some(CheckedPackageRefusalCause::TimedNodeReference),
                        true,
                    )
                }
                _ => {
                    refused_rows.swap(0, 1);
                    (
                        "/temporal_bindings/1/occurrence",
                        Some(CheckedPackageRefusalCause::TimedNodeReference),
                        true,
                    )
                }
            };
            let run_refused = |budget: &mut NativeJoinBudget| {
                let mut result = None;
                let allocated = allocation_counter::measure(|| {
                    let mut meter = WorkMeter::over(budget);
                    bodies.attach(&mut meter);
                    result = Some(validate(
                        Some(&refused_rows),
                        &nodes,
                        &kinds,
                        &[None],
                        &index,
                        &keys,
                        &lock,
                        &mut meter,
                    ));
                })
                .bytes_total;
                (result.unwrap(), allocated)
            };
            let mut ample = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(u64::MAX);
            let (expected, allocated) = run_refused(&mut ample);
            let Err(ValidationFailure::Refused(refusal)) = &expected else {
                panic!("semantic fault {case}")
            };
            assert_eq!(
                refusal.code,
                if case == 0 {
                    CheckedPackageRefusalCode::MalformedWire
                } else {
                    CheckedPackageRefusalCode::InvalidPackage
                }
            );
            assert_eq!(refusal.cause, cause);
            assert_eq!(refusal.loci[0].path.as_ref().unwrap().as_str(), path);
            let expected_id = if case == 1 {
                &refused_rows[0].declaration
            } else {
                &nodes[0].node_id
            };
            assert_eq!(refusal.locus.as_ref(), located.then_some(expected_id));
            let required = ample.read_storage_consumed();
            assert_eq!(allocated, required, "case {case}");
            let mut exact = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(required);
            assert_eq!(run_refused(&mut exact), (expected.clone(), required));
            for limit in [0, required - 1] {
                let mut short = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
                let (result, allocated) = run_refused(&mut short);
                assert!(matches!(result, Err(ValidationFailure::Budget(_))));
                assert_eq!(allocated, short.read_storage_consumed());
                assert!(short.first_unpaid_read_storage().is_some());
            }
            let mut repeated =
                NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(2 * required);
            assert_eq!(run_refused(&mut repeated), (expected.clone(), required));
            assert_eq!(run_refused(&mut repeated), (expected, required));
            assert_eq!(repeated.read_storage_consumed(), 2 * required);
            assert_eq!(repeated.consumed().work, 2 * ample.consumed().work);
        }
    }
}

impl<'de> Deserialize<'de> for ClockExactNumber {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
        enum Wire {
            Integer {
                decimal: Box<str>,
            },
            Rational {
                numerator: Box<str>,
                denominator: Box<str>,
            },
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "snake_case")]
        enum Tag {
            Integer,
            Rational,
        }
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = ClockExactNumber;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a closed ClockExactNumber record")
            }
            fn visit_newtype_struct<D2: serde::Deserializer<'de>>(
                self,
                inner: D2,
            ) -> Result<Self::Value, D2::Error> {
                Ok(match Wire::deserialize(inner)? {
                    Wire::Integer { decimal } => ClockExactNumber::Integer { decimal },
                    Wire::Rational {
                        numerator,
                        denominator,
                    } => ClockExactNumber::Rational {
                        numerator,
                        denominator,
                    },
                })
            }
            fn visit_enum<A: serde::de::EnumAccess<'de>>(
                self,
                record: A,
            ) -> Result<Self::Value, A::Error> {
                use serde::de::VariantAccess;
                let (tag, fields) = record.variant::<Tag>()?;
                match tag {
                    Tag::Integer => {
                        #[derive(Deserialize)]
                        #[serde(deny_unknown_fields)]
                        struct Fields {
                            decimal: Box<str>,
                        }
                        let Fields { decimal } = fields.newtype_variant::<Fields>()?;
                        Ok(ClockExactNumber::Integer { decimal })
                    }
                    Tag::Rational => {
                        #[derive(Deserialize)]
                        #[serde(deny_unknown_fields)]
                        struct Fields {
                            numerator: Box<str>,
                            denominator: Box<str>,
                        }
                        let Fields {
                            numerator,
                            denominator,
                        } = fields.newtype_variant::<Fields>()?;
                        Ok(ClockExactNumber::Rational {
                            numerator,
                            denominator,
                        })
                    }
                }
            }
        }
        deserializer.deserialize_newtype_struct(super::document_de::CLOSED_ENUM_TOKEN, Visitor)
    }
}

impl<'de> Deserialize<'de> for ClockConfiguration {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
        enum Wire {
            EventPosition {
                sequence_authority: Box<str>,
            },
            FixedSample {
                epoch: ClockExactNumber,
                period: ClockExactNumber,
                unit: Box<str>,
            },
            TimestampedEvent {
                timestamp_unit: Box<str>,
            },
            Timed {
                unit: Box<str>,
            },
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "snake_case")]
        enum Tag {
            EventPosition,
            FixedSample,
            TimestampedEvent,
            Timed,
        }
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = ClockConfiguration;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a closed ClockConfiguration record")
            }
            fn visit_newtype_struct<D2: serde::Deserializer<'de>>(
                self,
                inner: D2,
            ) -> Result<Self::Value, D2::Error> {
                Ok(match Wire::deserialize(inner)? {
                    Wire::EventPosition { sequence_authority } => {
                        ClockConfiguration::EventPosition { sequence_authority }
                    }
                    Wire::FixedSample {
                        epoch,
                        period,
                        unit,
                    } => ClockConfiguration::FixedSample {
                        epoch,
                        period,
                        unit,
                    },
                    Wire::TimestampedEvent { timestamp_unit } => {
                        ClockConfiguration::TimestampedEvent { timestamp_unit }
                    }
                    Wire::Timed { unit } => ClockConfiguration::Timed { unit },
                })
            }
            fn visit_enum<A: serde::de::EnumAccess<'de>>(
                self,
                record: A,
            ) -> Result<Self::Value, A::Error> {
                use serde::de::VariantAccess;
                let (tag, fields) = record.variant::<Tag>()?;
                match tag {
                    Tag::EventPosition => {
                        #[derive(Deserialize)]
                        #[serde(deny_unknown_fields)]
                        struct Fields {
                            sequence_authority: Box<str>,
                        }
                        let Fields { sequence_authority } = fields.newtype_variant::<Fields>()?;
                        Ok(ClockConfiguration::EventPosition { sequence_authority })
                    }
                    Tag::FixedSample => {
                        #[derive(Deserialize)]
                        #[serde(deny_unknown_fields)]
                        struct Fields {
                            epoch: ClockExactNumber,
                            period: ClockExactNumber,
                            unit: Box<str>,
                        }
                        let Fields {
                            epoch,
                            period,
                            unit,
                        } = fields.newtype_variant::<Fields>()?;
                        Ok(ClockConfiguration::FixedSample {
                            epoch,
                            period,
                            unit,
                        })
                    }
                    Tag::TimestampedEvent => {
                        #[derive(Deserialize)]
                        #[serde(deny_unknown_fields)]
                        struct Fields {
                            timestamp_unit: Box<str>,
                        }
                        let Fields { timestamp_unit } = fields.newtype_variant::<Fields>()?;
                        Ok(ClockConfiguration::TimestampedEvent { timestamp_unit })
                    }
                    Tag::Timed => {
                        #[derive(Deserialize)]
                        #[serde(deny_unknown_fields)]
                        struct Fields {
                            unit: Box<str>,
                        }
                        let Fields { unit } = fields.newtype_variant::<Fields>()?;
                        Ok(ClockConfiguration::Timed { unit })
                    }
                }
            }
        }
        deserializer.deserialize_newtype_struct(super::document_de::CLOSED_ENUM_TOKEN, Visitor)
    }
}

#[cfg(test)]
mod closed_record_storage_tests {
    use super::*;
    use crate::checked_package::common::decode_closed_document;
    use ix_trace_rs::trace;
    use std::sync::Arc;

    #[trace("TC-472", "FR-038-AC-511")]
    #[test]
    fn clock_records_decode_owned_fields_without_tagged_content_buffers() {
        for bytes in [
            br#"{"kind":"event_position","sequence_authority":"orders"}"#.as_slice(),
            br#"{"kind":"timestamped_event","timestamp_unit":"seconds"}"#.as_slice(),
            br#"{"kind":"timed","unit":"ms"}"#.as_slice(),
            br#"{"epoch":{"decimal":"0","kind":"integer"},"kind":"fixed_sample","period":{"denominator":"2","kind":"rational","numerator":"1"},"unit":"s"}"#.as_slice(),
        ] {
            let expected: ClockConfiguration = serde_json::from_slice(bytes).unwrap();
            let document=Arc::new(quire_canonical::read(bytes,u64::MAX).unwrap());
            let run=|limit| {
                let mut budget=super::super::NativeJoinBudget::new(u64::MAX,0).with_read_storage_limit(limit);
                let mut result=None;
                let allocated=allocation_counter::measure(||result=Some(decode_closed_document::<ClockConfiguration>(&document,&mut budget))).bytes_total;
                (result.unwrap(),allocated,budget)
            };
            let (result,allocated,budget)=run(u64::MAX);
            assert_eq!(result.unwrap(),expected);
            let required=budget.read_storage_consumed();
            assert!(required>0);
            assert_eq!(allocated,required);
            let (result,allocated,budget)=run(required);
            assert_eq!(result.unwrap(),expected);
            assert_eq!(allocated,budget.read_storage_consumed());
            let (result,allocated,budget)=run(required-1);
            assert!(matches!(result,Err(ValidationFailure::Budget(_))));
            assert_eq!(allocated,budget.read_storage_consumed());
            assert!(budget.first_unpaid_read_storage().is_some());
        }
    }
}

#[cfg(test)]
mod timed_clock_tests {
    use super::*;
    use ix_trace_rs::trace;

    /// The `timed` alternative is exactly `{kind, unit}`: it round-trips, and
    /// a missing or mistyped unit, an extra member and an unknown kind are
    /// refused rather than defaulted or substituted.
    #[trace("TC-048", "FR-038-AC-545", "FR-038-AC-546")]
    #[test]
    fn timed_clock_round_trips_and_refuses_violating_documents() {
        let timed: ClockConfiguration =
            serde_json::from_str(r#"{"kind":"timed","unit":"ms"}"#).unwrap();
        assert_eq!(timed, ClockConfiguration::Timed { unit: "ms".into() });
        assert_eq!(
            serde_json::to_string(&timed).unwrap(),
            r#"{"kind":"timed","unit":"ms"}"#
        );
        for violating in [
            r#"{"kind":"timed"}"#,
            r#"{"kind":"timed","unit":3}"#,
            r#"{"kind":"timed","unit":"ms","period":"1"}"#,
            r#"{"kind":"wall_clock","unit":"ms"}"#,
        ] {
            assert!(
                serde_json::from_str::<ClockConfiguration>(violating).is_err(),
                "{violating}"
            );
        }
    }

    /// The `trace` member is a present non-null string or absent, is written
    /// only when present, and a null, numeric or repeated member is refused.
    #[trace("TC-483", "FR-038-AC-561")]
    #[test]
    fn trace_member_round_trips_and_refuses_violating_documents() {
        let row = |extra: &str| {
            format!(
                r#"{{"declaration":{{"package":"p","path":["a"],"ordinal":0}},"occurrence":0,"clock_name":"c","clock":{{"kind":"event_position","sequence_authority":"s"}}{extra}}}"#
            )
        };
        let with: TemporalBinding = serde_json::from_str(&row(r#","trace":"s""#)).unwrap();
        assert_eq!(with.trace.as_deref(), Some("s"));
        assert!(serde_json::to_string(&with)
            .unwrap()
            .ends_with(r#","trace":"s"}"#));
        let without: TemporalBinding = serde_json::from_str(&row("")).unwrap();
        assert_eq!(without.trace, None);
        assert!(!serde_json::to_string(&without).unwrap().contains("trace"));
        for violating in [
            r#","trace":null"#,
            r#","trace":3"#,
            r#","trace":"s","trace":"n""#,
        ] {
            assert!(
                serde_json::from_str::<TemporalBinding>(&row(violating)).is_err(),
                "{violating}"
            );
        }
    }
}

#[cfg(test)]
mod occurrence_work_storage_tests {
    use super::super::{NativeJoinBudget, PackageReadStage};
    use super::*;
    use ix_trace_rs::trace;

    #[trace("TC-472", "FR-038-AC-508", "FR-038-AC-511")]
    #[test]
    fn occurrence_work_denial_prepays_only_its_final_pointer() {
        use super::super::stage_storage::Path;
        for borrowed in [
            Path::root().key("temporal_bindings").index(0),
            Path::root()
                .key("temporal_bindings")
                .index(0)
                .key("occurrence"),
            Path::root()
                .key("semantic_graph")
                .key("nodes")
                .index(7)
                .key("body"),
            Path::root()
                .key("temporal_bindings")
                .index(0)
                .key("clock")
                .key("kind"),
        ] {
            let expected_path = borrowed.build();
            let path = expected_path.as_str();
            let run = |budget: &mut NativeJoinBudget| {
                budget.set_active_locus(Some(super::super::NativeJoinLocus {
                    source: super::super::NativeJoinLocusSource::PackageRead(
                        PackageReadStage::Graph,
                        7,
                    ),
                    form: "temporal_clause",
                }));
                let mut result = None;
                let allocated = allocation_counter::measure(|| {
                    let mut meter = WorkMeter::over(budget);
                    meter.at_node(7, "temporal_clause");
                    result = Some(binding_work(&mut meter, borrowed));
                })
                .bytes_total;
                (result.unwrap(), allocated)
            };
            let mut admitted = NativeJoinBudget::new(1, 0);
            assert_eq!(run(&mut admitted), (Ok(()), 0));
            assert_eq!(admitted.consumed().work, 1);
            let mut ample = NativeJoinBudget::new(0, 0).with_read_storage_limit(u64::MAX);
            let (expected, allocated) = run(&mut ample);
            let Err(ValidationFailure::Budget(stop)) = &expected else {
                panic!("work denial")
            };
            assert_eq!(stop.path.as_ref().unwrap().as_str(), path);
            assert_eq!(stop.resource, super::super::PackageReadResource::Work);
            assert_eq!(stop.amount, 1);
            let required = ample.read_storage_consumed();
            assert_eq!(required, path.len() as u64);
            assert_eq!(allocated, required);
            let locus = ample.first_unpaid_locus();
            assert!(locus.is_some());
            assert_eq!(
                locus.unwrap().source,
                super::super::NativeJoinLocusSource::PackageRead(PackageReadStage::Graph, 7)
            );
            let mut exact = NativeJoinBudget::new(0, 0).with_read_storage_limit(required);
            assert_eq!(run(&mut exact), (expected.clone(), required));
            for limit in [0, required - 1] {
                let mut short = NativeJoinBudget::new(0, 0).with_read_storage_limit(limit);
                let (result, allocated) = run(&mut short);
                let Err(ValidationFailure::Budget(stop)) = result else {
                    panic!("work denial")
                };
                assert_eq!(stop.path, None);
                assert_eq!(allocated, 0);
                assert_eq!(short.read_storage_consumed(), 0);
                assert_eq!(short.first_unpaid_locus(), locus);
                assert_eq!(short.consumed().work, ample.consumed().work);
            }
            let mut repeated = NativeJoinBudget::new(0, 0).with_read_storage_limit(2 * required);
            assert_eq!(run(&mut repeated), (expected.clone(), required));
            assert_eq!(run(&mut repeated), (expected, required));
            assert_eq!(repeated.read_storage_consumed(), 2 * required);
            assert_eq!(repeated.first_unpaid_locus(), locus);
            assert_eq!(repeated.consumed().work, ample.consumed().work);
        }
    }
}
