//! Exact I04 contract-version refusal on the direct V2 byte reader.

use super::definition_catalog::DefinitionCatalog;
use super::evidence::CheckedPackageEvidence;
use super::shared::{
    CheckedPackageIncomplete, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, JsonPointer, RefusalFields,
};
use super::v2::{
    CheckedPackageBudgetedDispatchResult, CheckedPackageV2, CheckedPackageV2BudgetedReadResult,
    CheckedPackageV2ReadResult, NativeJoinBudget,
};

/// The closed result of dispatching untrusted checked-package bytes.
// The required inline typed refusal makes this public result larger than its
// incomplete case; changing the variant to a box would change its API.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedPackageDispatchResult {
    /// A `filament.checked-package/v2` package admitted by the V2 reader.
    AdmittedV2(Box<CheckedPackageV2>),
    /// The input is invalid for its selected version, or selects none.
    Refused(CheckedPackageRefusal),
    /// The input exceeded a caller limit before a conclusion.
    Incomplete(CheckedPackageIncomplete),
    /// The reader's operation catalog is unreadable; no input was judged.
    CatalogUnavailable(super::v2::CatalogReadError),
    /// The host refused an allocation the read needed; no input was judged.
    HostAllocationFailed(super::v2::CheckedPackageHostAllocationFailed),
}

/// Reads `contract_version` from borrowed source bytes after strict syntax and
/// canonical checks, then either admits the current contract or refuses any
/// other version with a typed
/// [`super::shared::CheckedPackageRefusalCode::UnknownContractVersion`] code. This is a
/// refusal control, not a compatibility layer: it never relabels or widens
/// the admitted contract.
pub fn read_checked_package(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
    evidence: &CheckedPackageEvidence,
) -> CheckedPackageDispatchResult {
    match CheckedPackageV2::read(bytes, limits, evidence) {
        CheckedPackageV2ReadResult::Admitted(package) => {
            CheckedPackageDispatchResult::AdmittedV2(package)
        }
        CheckedPackageV2ReadResult::Refused(refusal) => {
            CheckedPackageDispatchResult::Refused(refusal)
        }
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            CheckedPackageDispatchResult::Incomplete(incomplete)
        }
        CheckedPackageV2ReadResult::CatalogUnavailable(error) => {
            CheckedPackageDispatchResult::CatalogUnavailable(error)
        }
        CheckedPackageV2ReadResult::HostAllocationFailed(failure) => {
            CheckedPackageDispatchResult::HostAllocationFailed(failure)
        }
    }
}

/// Reads a strict checked package and binds its selected definitions to the
/// caller's catalog.
///
/// The operation catalog remains authoritative for operation-law roles. The
/// supplied catalog admits extension selections and distinguishes profile
/// identities from unsupported lock-only selections. If a paired operation
/// law is itself outside its operation role, that more specific mismatch is
/// retained at the operation-law definition. For an operation-law refusal, a
/// catalog-only structural pass skips the lock join internally so the caller's
/// catalog can classify the lock row; that structural package is never
/// returned as admitted.
pub fn read_checked_package_with_catalog(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
    evidence: &CheckedPackageEvidence,
    catalog: &DefinitionCatalog,
) -> CheckedPackageDispatchResult {
    let strict = read_checked_package(bytes, limits, evidence);
    match strict {
        CheckedPackageDispatchResult::AdmittedV2(package) => catalog_refusal(&package, catalog)
            .map_or(
                CheckedPackageDispatchResult::AdmittedV2(package),
                CheckedPackageDispatchResult::Refused,
            ),
        CheckedPackageDispatchResult::Refused(refusal)
            if matches!(
                refusal.cause,
                Some(
                    CheckedPackageRefusalCause::OperationLawUnselected
                        | CheckedPackageRefusalCause::OperationLawMismatch
                )
            ) =>
        {
            let structural =
                CheckedPackageV2::read_catalog_bound_structural(bytes, limits, evidence);
            match structural {
                CheckedPackageV2ReadResult::Admitted(package) => catalog_refusal(&package, catalog)
                    .map_or(
                        CheckedPackageDispatchResult::Refused(refusal),
                        CheckedPackageDispatchResult::Refused,
                    ),
                _ => CheckedPackageDispatchResult::Refused(refusal),
            }
        }
        other => other,
    }
}

fn catalog_refusal(
    package: &CheckedPackageV2,
    catalog: &DefinitionCatalog,
) -> Option<CheckedPackageRefusal> {
    let operation_laws = package
        .graph()
        .nodes
        .iter()
        .filter_map(|node| match &node.body {
            super::v2::CheckedTerm::Application { operation, .. } => {
                operation.law_definitions().ok()
            }
            _ => None,
        })
        .flatten()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();

    for (index, definition) in package.lock().definition_selections.iter().enumerate() {
        let operation_law = operation_laws.contains(definition);
        if let Some(refusal) = catalog_selection_refusal(index, definition, operation_law, catalog)
        {
            return Some(refusal);
        }
    }
    None
}

fn catalog_selection_refusal(
    index: usize,
    definition: &super::shared::CheckedArtifactRef,
    operation_law: bool,
    catalog: &DefinitionCatalog,
) -> Option<CheckedPackageRefusal> {
    if catalog.is_profile(definition) || (operation_law && catalog.is_extension(definition)) {
        return Some(definition_refusal(
            index,
            CheckedPackageRefusalCause::WrongSelectionRole,
        ));
    }
    if operation_law || catalog.is_extension(definition) {
        return None;
    }
    Some(definition_refusal(
        index,
        CheckedPackageRefusalCause::UnsupportedSelection,
    ))
}

fn definition_refusal(index: usize, cause: CheckedPackageRefusalCause) -> CheckedPackageRefusal {
    CheckedPackageRefusal::new(RefusalFields {
        code: CheckedPackageRefusalCode::UnknownProfile,
        path: Some(
            JsonPointer::root()
                .key("lock")
                .key("definition_selections")
                .index(index),
        ),
        cause: Some(cause),
        locus: None,
        contract_version: None,
        document_pointer: None,
    })
}

/// Reads a strict package with opaque value-law definitions.
///
/// Operation and law roles remain catalog checked. For value-law roles this
/// entry admits an otherwise unknown `{authority, identity}` pair only when
/// the exact pair is present in `lock.definition_selections`; the pair is
/// retained unchanged in the admitted package. Use [`read_checked_package`]
/// when the caller requires the catalogued law definition.
pub fn read_checked_package_structural(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
    evidence: &CheckedPackageEvidence,
) -> CheckedPackageDispatchResult {
    match CheckedPackageV2::read_structural(bytes, limits, evidence) {
        CheckedPackageV2ReadResult::Admitted(package) => {
            CheckedPackageDispatchResult::AdmittedV2(package)
        }
        CheckedPackageV2ReadResult::Refused(refusal) => {
            CheckedPackageDispatchResult::Refused(refusal)
        }
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            CheckedPackageDispatchResult::Incomplete(incomplete)
        }
        CheckedPackageV2ReadResult::CatalogUnavailable(error) => {
            CheckedPackageDispatchResult::CatalogUnavailable(error)
        }
        CheckedPackageV2ReadResult::HostAllocationFailed(failure) => {
            CheckedPackageDispatchResult::HostAllocationFailed(failure)
        }
    }
}

/// [`read_checked_package`] under the caller's budget: the budgeted V2 read,
/// with each variant mapped one to one. There is no V1 branch.
pub fn read_checked_package_with_budget(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
    evidence: &CheckedPackageEvidence,
    budget: &mut NativeJoinBudget,
) -> CheckedPackageBudgetedDispatchResult {
    match CheckedPackageV2::read_with_budget(bytes, limits, evidence, budget) {
        CheckedPackageV2BudgetedReadResult::Admitted(package) => {
            CheckedPackageBudgetedDispatchResult::AdmittedV2(package)
        }
        CheckedPackageV2BudgetedReadResult::Refused(refusal) => {
            CheckedPackageBudgetedDispatchResult::Refused(refusal)
        }
        CheckedPackageV2BudgetedReadResult::Incomplete(incomplete) => {
            CheckedPackageBudgetedDispatchResult::Incomplete(incomplete)
        }
        CheckedPackageV2BudgetedReadResult::BudgetIncomplete(incomplete) => {
            CheckedPackageBudgetedDispatchResult::BudgetIncomplete(incomplete)
        }
        CheckedPackageV2BudgetedReadResult::Exhausted(exhausted) => {
            CheckedPackageBudgetedDispatchResult::Exhausted(exhausted)
        }
        CheckedPackageV2BudgetedReadResult::CatalogUnavailable(error) => {
            CheckedPackageBudgetedDispatchResult::CatalogUnavailable(error)
        }
        CheckedPackageV2BudgetedReadResult::HostAllocationFailed(failure) => {
            CheckedPackageBudgetedDispatchResult::HostAllocationFailed(failure)
        }
    }
}
