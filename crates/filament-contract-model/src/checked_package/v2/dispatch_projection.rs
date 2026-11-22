//! Bounded read of producer-captured model dispatch candidates (FR-354 AC-4).

use super::checked_term_data::{
    CheckedDispatchCandidate, CheckedDispatchCandidateTable, CheckedTermDataV2,
};
use super::model_fields::RetainedModels;
use super::model_members::Budget;
use super::native_join_budget::PackageReadStage;
use super::source_object::is_source_declared_object_type;
use super::stage_storage::{self, Path, Refusal, Site};
use super::{
    ApplicationOperator, CheckedNodeKind, CheckedNodeOwner, CheckedPackageV2, CheckedPackageWireV2,
    CheckedTerm, ClaimForm, ExpressionForm, FunctionForm, ModelForm, ScalarTypeForm, WorkMeter,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedNodeId, CheckedOccurrence, CheckedOccurrenceRole, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode,
};

pub(super) fn data_site(row: usize, at: Path<'static>) -> Site<'static> {
    Site::new(
        PackageReadStage::CheckedTermData,
        row,
        "checked_term_data",
        at,
    )
}

/// A refusal about the value at `at` in sidecar row `row`, its diagnostic
/// charged before it is built.
pub(super) fn fail(
    meter: &mut WorkMeter,
    row: usize,
    code: CheckedPackageRefusalCode,
    cause: Option<CheckedPackageRefusalCause>,
    at: Path<'static>,
) -> ValidationFailure {
    let refusal = Refusal::new(code, at);
    let refusal = match cause {
        Some(cause) => refusal.because(cause),
        None => refusal,
    };
    stage_storage::refuse(meter, &data_site(row, at), refusal)
}

pub(super) fn invalid(meter: &mut WorkMeter, row: usize, at: Path<'static>) -> ValidationFailure {
    fail(
        meter,
        row,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        at,
    )
}

pub(super) fn conflicting(
    meter: &mut WorkMeter,
    row: usize,
    at: Path<'static>,
) -> ValidationFailure {
    fail(
        meter,
        row,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::ConflictingDefinition),
        at,
    )
}

/// The graph's node ids with their positions, ascending by identity.
pub(super) fn node_positions<'w, Body>(
    wire: &'w CheckedPackageWireV2<Body>,
    meter: &mut WorkMeter,
) -> Result<Vec<(&'w CheckedNodeId, usize)>, ValidationFailure> {
    let nodes = &wire.semantic_graph.nodes;
    let mut index = stage_storage::vec(
        meter,
        &data_site(0, Path::root().key("checked_term_data")),
        nodes.len(),
    )?;
    index.extend(
        nodes
            .iter()
            .enumerate()
            .map(|(position, node)| (&node.node_id, position)),
    );
    index.sort_unstable();
    Ok(index)
}

/// The position of `id` among `index`.
pub(super) fn position_of(index: &[(&CheckedNodeId, usize)], id: &CheckedNodeId) -> Option<usize> {
    index
        .binary_search_by(|(held, _)| (*held).cmp(id))
        .ok()
        .map(|found| index[found].1)
}

/// How a dispatch row's receiver family is joined.
enum Route<'a> {
    /// Selected model owners.
    Model {
        model: &'a super::model_members::DomainModel,
        family: &'a str,
    },
    /// Source-declared object types sharing one `SourceOwner`.
    Source { owner: &'a CheckedNodeOwner },
}

/// Whether `node` is a source clause function of `operation` for `clause`: its
/// `declaration` is `[<type>, <operation>, <clause>]` under `owner`.
fn is_source_clause_function<Body>(
    node: &super::CheckedSemanticNodeV2<Body>,
    owner: &CheckedNodeOwner,
    operation: &str,
    clause: &str,
) -> bool {
    node.owner.as_ref() == Some(owner)
        && node.declaration.as_ref().is_some_and(|declaration| {
            matches!(
                declaration.qualified_name.as_slice(),
                [_, name, kind] if name.as_ref() == operation && kind.as_ref() == clause
            )
        })
}

/// The (owner identity, `source_identity`) of every source-declared object
/// type, sorted, built once per read.
fn source_type_index<'w, Body>(
    wire: &'w CheckedPackageWireV2<Body>,
    kinds: &[CheckedNodeKind],
    meter: &mut WorkMeter,
) -> Result<Vec<(&'w str, &'w str)>, ValidationFailure> {
    let mut held = stage_storage::vec(
        meter,
        &data_site(0, Path::root().key("checked_term_data")),
        wire.semantic_graph.nodes.len(),
    )?;
    held.extend(
        wire.semantic_graph
            .nodes
            .iter()
            .zip(kinds)
            .filter(|(node, kind)| is_source_declared_object_type(node, **kind))
            .filter_map(|(node, _)| match (&node.owner, &node.declaration) {
                (Some(CheckedNodeOwner::Source { identity, .. }), Some(declaration)) => declaration
                    .qualified_name
                    .last()
                    .map(|name| (identity.as_ref(), name.as_ref())),
                _ => None,
            }),
    );
    held.sort_unstable();
    Ok(held)
}

pub(super) type Occurrence<'a> = (&'a CheckedNodeId, &'a CheckedOccurrenceRole, u64);

/// The source map's occurrences, ascending.
fn source_occurrences<'w, Body>(
    wire: &'w CheckedPackageWireV2<Body>,
    meter: &mut WorkMeter,
) -> Result<Vec<Occurrence<'w>>, ValidationFailure> {
    let mut held = stage_storage::vec(
        meter,
        &data_site(0, Path::root().key("checked_term_data")),
        wire.source_map.len(),
    )?;
    held.extend(
        wire.source_map
            .iter()
            .map(|entry| (&entry.node_id, &entry.role, entry.ordinal)),
    );
    held.sort_unstable();
    Ok(held)
}

/// Checks the producer-captured sidecar after graph and source-map admission.
pub(super) fn validate_checked_term_data<Body>(
    data: &CheckedTermDataV2,
    wire: &CheckedPackageWireV2<Body>,
    kinds: &[CheckedNodeKind],
    terms: &[CheckedTerm],
    models: &RetainedModels,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let root = Path::root().key("checked_term_data");
    super::equality_projection::validate(data, wire, kinds, terms, meter)?;
    super::protocol_locations::validate(wire, kinds, terms, meter)?;

    let nodes = &wire.semantic_graph.nodes;
    let index = node_positions(wire, meter)?;
    let source_occurrences = source_occurrences(wire, meter)?;
    let tables = root.key("dispatch_tables");
    let mut source_types: Option<Vec<(&str, &str)>> = None;
    let repeated_table = stage_storage::first_repeat(
        meter,
        &data_site(0, tables),
        data.dispatch_tables
            .iter()
            .map(|table| (&table.application, &table.occurrence)),
    )?;
    for (row_index, table) in data.dispatch_tables.iter().enumerate() {
        let row = tables.index(row_index);
        stage_storage::work(meter, &data_site(row_index, row), 1)?;
        if repeated_table == Some(row_index) {
            return Err(conflicting(meter, row_index, row));
        }
        let Some(application_index) = position_of(&index, &table.application) else {
            return Err(invalid(meter, row_index, row.key("application")));
        };
        let Some(application_node) = nodes.get(application_index) else {
            return Err(invalid(meter, row_index, row.key("application")));
        };
        if table.application.package.as_ref() != wire.name.as_ref()
            || kinds.get(application_index)
                != Some(&CheckedNodeKind::Expression(ExpressionForm::Call))
        {
            return Err(invalid(meter, row_index, row.key("application")));
        }
        if !application_node.occurrences.contains(&table.occurrence)
            || source_occurrences
                .binary_search(&(
                    &table.application,
                    &table.occurrence.role,
                    table.occurrence.ordinal,
                ))
                .is_err()
        {
            return Err(invalid(meter, row_index, row.key("occurrence")));
        }
        let Some(CheckedTerm::Application {
            operator: ApplicationOperator::Call,
            operation,
            ..
        }) = terms.get(application_index)
        else {
            return Err(invalid(meter, row_index, row.key("application")));
        };
        let member = operation.member_node();
        let member_name = member
            .and_then(|member| member.get("name"))
            .and_then(|name| name.as_str());
        if member
            .and_then(|member| member.get("kind"))
            .and_then(|kind| kind.as_str())
            != Some("operation")
            || member_name != Some(table.operation.operation.as_ref())
        {
            return Err(invalid(meter, row_index, row.key("operation")));
        }
        let context_position = position_of(&index, &table.context);
        let source_context = context_position.is_some_and(|position| {
            is_source_declared_object_type(&nodes[position], kinds[position])
        });
        let source_declaring =
            position_of(&index, &table.operation.declaring).is_some_and(|position| {
                is_source_declared_object_type(&nodes[position], kinds[position])
            });
        let context_is_model = context_position.is_some_and(|position| {
            matches!(
                kinds.get(position),
                Some(CheckedNodeKind::Model(
                    ModelForm::ObjectType
                        | ModelForm::Process
                        | ModelForm::StateMachine
                        | ModelForm::PersistenceInterface
                ))
            )
        });
        // The member and receiver context name the call's static type,
        // while the selected operation may belong to an ancestor's family.
        if operation.member_declaration().ok().flatten() != Some(&table.context)
            || !(context_is_model || source_context)
        {
            return Err(invalid(meter, row_index, row.key("context")));
        }
        let route = if source_context || source_declaring {
            // FR-366: a source row joins context and declaring through their
            // `SourceOwner`; a row mixing a source and a model family refuses
            // as a model row whose families differ does.
            let owner = |id: &CheckedNodeId| {
                position_of(&index, id)
                    .filter(|&position| {
                        is_source_declared_object_type(&nodes[position], kinds[position])
                    })
                    .and_then(|position| nodes[position].owner.as_ref())
            };
            match (owner(&table.context), owner(&table.operation.declaring)) {
                (Some(context), Some(declaring)) if context == declaring => {
                    Route::Source { owner: context }
                }
                _ => return Err(invalid(meter, row_index, row.key("operation"))),
            }
        } else {
            let Some(context_owner) = models.owner_by_key(&table.context) else {
                return Err(invalid(meter, row_index, row.key("context")));
            };
            let Some(family_owner) = models.owner_by_key(&table.operation.declaring) else {
                return Err(invalid(meter, row_index, row.key("operation")));
            };
            if context_owner.selection != family_owner.selection {
                return Err(invalid(meter, row_index, row.key("operation")));
            }
            let model = context_owner.package;
            let mut budget = Budget::new(meter, model.root, model.bytes);
            if !model.operation_in_family(
                context_owner.node,
                family_owner.node,
                &table.operation.operation,
                &mut budget,
            )? {
                return Err(invalid(meter, row_index, row.key("operation")));
            }
            Route::Model {
                model,
                family: family_owner.node,
            }
        };
        let candidates = row.key("candidates");
        let repeated_key = stage_storage::first_repeat(
            meter,
            &data_site(row_index, candidates),
            table
                .candidates
                .iter()
                .map(|candidate| &candidate.selection_key),
        )?;
        for (candidate_index, candidate) in table.candidates.iter().enumerate() {
            let at = candidates.index(candidate_index);
            stage_storage::work(meter, &data_site(row_index, at), 1)?;
            if candidate.selected_operation != table.operation {
                return Err(invalid(meter, row_index, at.key("selected_operation")));
            }
            if repeated_key == Some(candidate_index) {
                return Err(conflicting(meter, row_index, at.key("selection_key")));
            }
            if candidate.selection_key.owner_effective_type.is_some() {
                // A source declaration key carries no owner effective type.
                if matches!(route, Route::Source { .. }) {
                    return Err(invalid(meter, row_index, at.key("selection_key")));
                }
                return Err(fail(
                    meter,
                    row_index,
                    CheckedPackageRefusalCode::UnsupportedConstruct,
                    None,
                    at.key("selection_key"),
                ));
            }
            if let Route::Model { model, family } = &route {
                let original = &candidate.selection_key.original;
                if model.identity.as_ref() != original.package.as_ref()
                    || !model.object_types.contains_key(original.node.as_ref())
                {
                    return Err(invalid(meter, row_index, at.key("selection_key")));
                }
                let mut budget = Budget::new(meter, model.root, model.bytes);
                if !model.operation_in_family(
                    &original.node,
                    family,
                    &table.operation.operation,
                    &mut budget,
                )? {
                    return Err(invalid(meter, row_index, at.key("selection_key")));
                }
            }
            if let Route::Source {
                owner: CheckedNodeOwner::Source { identity, .. },
            } = &route
            {
                // FR-366: the key is `{package: <owner identity>, node: <the
                // `source_identity` of a source type under that owner>}`; the
                // candidate's own charge covers one lookup.
                let types = match &source_types {
                    Some(types) => types,
                    None => source_types.insert(source_type_index(wire, kinds, meter)?),
                };
                let original = &candidate.selection_key.original;
                let source_type = source_type_identity(&original.node);
                let named = identity.as_ref() == original.package.as_ref()
                    && source_type.is_some_and(|(source_identity, name)| {
                        source_identity == identity.as_ref()
                            && types.binary_search(&(identity.as_ref(), name)).is_ok()
                    });
                if !named {
                    return Err(invalid(meter, row_index, at.key("selection_key")));
                }
            }
            let body_position = position_of(&index, &candidate.operation_body);
            let body_kind = body_position.and_then(|position| kinds.get(position));
            // Model declarations can belong to a dependency package, so the
            // exact ID's package need not be the package being emitted. The
            // graph index and function kind prove that this handle is a
            // retained function node.
            if !matches!(body_kind, Some(CheckedNodeKind::Function(_))) {
                return Err(invalid(meter, row_index, at.key("operation_body")));
            }
            if let Route::Source { owner } = &route {
                if !body_position.is_some_and(|position| {
                    is_source_clause_function(
                        &nodes[position],
                        owner,
                        &table.operation.operation,
                        "body",
                    )
                }) {
                    return Err(invalid(meter, row_index, at.key("operation_body")));
                }
            }
            if let Some(precondition) = candidate.effective_precondition.as_ref() {
                let precondition_position = position_of(&index, precondition);
                let precondition_kind =
                    precondition_position.and_then(|position| kinds.get(position));
                let has_boolean_result = precondition_position
                    .and_then(|position| nodes.get(position))
                    .and_then(|node| position_of(&index, &node.semantic_type))
                    .and_then(|position| kinds.get(position))
                    == Some(&CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean));
                let valid_precondition = match precondition_kind {
                    Some(CheckedNodeKind::Function(FunctionForm::Predicate)) => true,
                    Some(CheckedNodeKind::Function(FunctionForm::PureFunction)) => {
                        has_boolean_result
                    }
                    _ => false,
                };
                if !valid_precondition {
                    return Err(invalid(meter, row_index, at.key("effective_precondition")));
                }
                if let Route::Source { owner } = &route {
                    if !precondition_position.is_some_and(|position| {
                        is_source_clause_function(
                            &nodes[position],
                            owner,
                            &table.operation.operation,
                            "effective_precondition",
                        )
                    }) {
                        return Err(invalid(meter, row_index, at.key("effective_precondition")));
                    }
                }
            }
        }
    }
    validate_probabilistic_claims(data, wire, kinds, &index, &source_occurrences, meter)?;
    super::body_locations::validate(wire, kinds, terms, &data.body_locations, meter)?;
    Ok(())
}

#[ix_trace_rs::string_edge]
fn source_type_identity(node: &str) -> Option<(&str, &str)> {
    node.strip_prefix("ix://")?.rsplit_once('/')
}

fn validate_probabilistic_claims<Body>(
    data: &CheckedTermDataV2,
    wire: &CheckedPackageWireV2<Body>,
    kinds: &[CheckedNodeKind],
    index: &[(&CheckedNodeId, usize)],
    source_occurrences: &[Occurrence<'_>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let root = Path::root()
        .key("checked_term_data")
        .key("probabilistic_claims");
    let nodes = &wire.semantic_graph.nodes;
    let claims = &data.probabilistic_claims;
    // The declared claims, ascending, to find a repeat and to find a claim node
    // no row declares.
    let mut declared: Vec<(&CheckedNodeId, usize)> =
        stage_storage::vec(meter, &data_site(0, root), claims.len())?;
    declared.extend(
        claims
            .iter()
            .enumerate()
            .map(|(position, claim)| (&claim.declaration, position)),
    );
    declared.sort_unstable();
    let repeated = declared
        .windows(2)
        .filter(|pair| pair[0].0 == pair[1].0)
        .map(|pair| pair[1].1)
        .min();
    let mut previous = None;
    for (row_index, claim) in claims.iter().enumerate() {
        let row = root.index(row_index);
        stage_storage::work(meter, &data_site(row_index, row), 1)?;
        if repeated == Some(row_index) {
            return Err(conflicting(meter, row_index, row));
        }
        if previous.is_some_and(|prior| prior > &claim.declaration) {
            return Err(fail(
                meter,
                row_index,
                CheckedPackageRefusalCode::InvalidPackage,
                Some(CheckedPackageRefusalCause::InvalidSemanticGraph),
                row,
            ));
        }
        previous = Some(&claim.declaration);
        let Some(position) = position_of(index, &claim.declaration) else {
            return Err(invalid(meter, row_index, row.key("declaration")));
        };
        let node = &nodes[position];
        if claim.declaration.package.as_ref() != wire.name.as_ref()
            || kinds.get(position) != Some(&CheckedNodeKind::Claim(ClaimForm::ProbabilisticClaim))
        {
            return Err(invalid(meter, row_index, row.key("declaration")));
        }
        if claim.occurrence.role != CheckedOccurrenceRole::Claim
            || !node.occurrences.contains(&claim.occurrence)
            || source_occurrences
                .binary_search(&(
                    &claim.declaration,
                    &claim.occurrence.role,
                    claim.occurrence.ordinal,
                ))
                .is_err()
        {
            return Err(invalid(meter, row_index, row.key("occurrence")));
        }
    }
    for node in nodes.iter().zip(kinds).filter_map(|(node, kind)| {
        (*kind == CheckedNodeKind::Claim(ClaimForm::ProbabilisticClaim)).then_some(node)
    }) {
        if declared
            .binary_search_by(|(held, _)| (*held).cmp(&node.node_id))
            .is_err()
        {
            return Err(invalid(meter, 0, root));
        }
    }
    Ok(())
}

/// A complete admitted dispatch table, borrowing candidates in producer order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckedDispatchView<'a> {
    /// Exact application and occurrence selected by the producer.
    pub table: &'a CheckedDispatchCandidateTable,
}

impl CheckedDispatchView<'_> {
    /// Candidate rows in the producer's order, including an authoritative empty set.
    pub fn candidates(&self) -> &[CheckedDispatchCandidate] {
        &self.table.candidates
    }
}

/// Why a bounded dispatch read returned no complete table.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum CheckedDispatchReadError {
    /// The producer retained no checked-term sidecar.
    #[error("checked-term dispatch data is unavailable")]
    Unavailable,
    /// The requested application belongs to another checked package.
    #[error("dispatch application belongs to another checked package")]
    ForeignApplication,
    /// The requested application does not name an admitted graph node.
    #[error("dispatch application is not admitted")]
    UnknownApplication,
    /// The requested admitted node is not a call application.
    #[error("dispatch application is not a call")]
    WrongApplication,
    /// No admitted table names the requested application and occurrence.
    #[error("no dispatch table names the requested occurrence")]
    Missing,
    /// The first uncharged row or candidate exceeded the caller's work limit.
    #[error("dispatch read exhausted its work limit")]
    WorkExhausted {
        /// Caller-supplied work limit.
        limit: u64,
        /// Work attempted, including the rejected charge.
        consumed: u64,
        /// Sidecar row at which the charge failed.
        row: usize,
        /// Candidate position when the rejected charge was for a candidate.
        candidate: Option<usize>,
    },
}

impl CheckedPackageV2 {
    /// Returns one complete producer-captured dispatch table under a finite work limit.
    ///
    /// The read does not choose a candidate, evaluate a precondition, or derive
    /// a table from the graph. `Missing` differs from a captured empty set.
    pub fn dispatch_candidates(
        &self,
        application: &CheckedNodeId,
        occurrence: &CheckedOccurrence,
        work_limit: u64,
    ) -> Result<CheckedDispatchView<'_>, CheckedDispatchReadError> {
        if application.package.as_ref() != self.wire.name.as_ref() {
            return Err(CheckedDispatchReadError::ForeignApplication);
        }
        let Some(position) = self.node_position(application) else {
            return Err(CheckedDispatchReadError::UnknownApplication);
        };
        if self.kinds.get(position) != Some(&CheckedNodeKind::Expression(ExpressionForm::Call)) {
            return Err(CheckedDispatchReadError::WrongApplication);
        }
        let Some(data) = self.wire.checked_term_data.as_ref() else {
            return Err(CheckedDispatchReadError::Unavailable);
        };
        let mut consumed = 0_u64;
        for (row_index, table) in data.dispatch_tables.iter().enumerate() {
            consumed = consumed.saturating_add(1);
            if consumed > work_limit {
                return Err(CheckedDispatchReadError::WorkExhausted {
                    limit: work_limit,
                    consumed,
                    row: row_index,
                    candidate: None,
                });
            }
            if &table.application != application || &table.occurrence != occurrence {
                continue;
            }
            for candidate_index in 0..table.candidates.len() {
                consumed = consumed.saturating_add(1);
                if consumed > work_limit {
                    return Err(CheckedDispatchReadError::WorkExhausted {
                        limit: work_limit,
                        consumed,
                        row: row_index,
                        candidate: Some(candidate_index),
                    });
                }
            }
            return Ok(CheckedDispatchView { table });
        }
        Err(CheckedDispatchReadError::Missing)
    }
}

#[cfg(test)]
mod metadata_tests {
    use super::super::native_join_budget::{NativeJoinBudget, NativeJoinLocusSource};
    use super::super::{CheckedJson, CheckedProbabilisticClaim};
    use super::*;
    use ix_trace_rs::trace;
    use serde_json::json;

    /// The checked-term-data stage consumes retained document metadata and
    /// already checked terms; this fixture enters that actual stage directly.
    #[trace("TC-472", "FR-038-AC-506")]
    #[test]
    fn document_metadata_sidecar_join_prepays_tables_and_checks_occurrence_values() {
        let id = json!({"package":"test/sidecar", "path":["claim"], "ordinal":0});
        let definition = json!({"authority":"test", "identity":"sidecar-definition"});
        let mut wire: CheckedPackageWireV2<CheckedJson> = serde_json::from_value(json!({
            "contract_version":super::super::CHECKED_PACKAGE_V2, "name":"test/sidecar",
            "lock":{"sources":[], "edition":{"role":"edition", "definition":definition},
                "dependency_selections":[], "definition_selections":[],
                "profile_selections":[], "required_features":[]},
            "semantic_graph":{"graph_version":"filament.checked-semantic-graph/v2", "operand_rows":[[]], "nodes":[{
                "node_id":id, "schema_version":"filament.checked-semantic-graph/v2",
                "node_tag":"claim", "semantic_form":"probabilistic_claim", "semantic_type":id,
                "dependencies":[], "occurrences":[{"role":"claim", "ordinal":7}],
                "body":{"term":"aggregate", "members":[]}
            }]},
            "source_map":[{"node_id":id, "role":"claim", "ordinal":7, "regions":[]}],
            "capability_report":[], "diagnostics":{"catalog":definition, "entries":[]}
        }))
        .expect("retained document metadata");
        let data = CheckedTermDataV2 {
            probabilistic_claims: vec![CheckedProbabilisticClaim {
                declaration: wire.semantic_graph.nodes[0].node_id.clone(),
                occurrence: wire.semantic_graph.nodes[0].occurrences[0].clone(),
            }],
            ..CheckedTermDataV2::default()
        };
        wire.checked_term_data = Some(data.clone());
        let models = RetainedModels::default();
        let terms = [CheckedTerm::Aggregate {
            members: Vec::new(),
        }];
        let run = |data: &CheckedTermDataV2, limit| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let observed = allocation_counter::measure(|| {
                result = Some(validate_checked_term_data(
                    data,
                    &wire,
                    &[CheckedNodeKind::Claim(ClaimForm::ProbabilisticClaim)],
                    &terms,
                    &models,
                    &mut WorkMeter::over(&mut budget),
                ));
            });
            (
                result.expect("the actual sidecar join ran"),
                observed.bytes_total,
                budget,
            )
        };
        let (result, allocated, budget) = run(&data, u64::MAX);
        assert!(
            result.is_ok(),
            "the original claim occurrence joins: {result:?}"
        );
        let required = budget.read_storage_consumed();
        assert!(required > 0);
        assert_eq!(allocated, required);
        assert!(run(&data, required).0.is_ok());
        let (short, _, budget) = run(&data, required - 1);
        assert!(matches!(short, Err(ValidationFailure::Budget(_))));
        let locus = budget
            .first_unpaid_locus()
            .expect("the request that did not fit");
        assert_eq!(
            locus.source,
            NativeJoinLocusSource::PackageRead(PackageReadStage::CheckedTermData, 0)
        );
        assert_eq!(locus.form, "checked_term_data");
        let mut changed = data;
        changed.probabilistic_claims[0].occurrence.ordinal = 8;
        let Err(ValidationFailure::Refused(refusal)) = run(&changed, u64::MAX).0 else {
            panic!("an occurrence absent from the node and source map must refuse");
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::InvalidValue)
        );
        assert_eq!(
            refusal.loci[0].path.as_ref().expect("path").as_str(),
            "/checked_term_data/probabilistic_claims/0/occurrence"
        );
    }
}
