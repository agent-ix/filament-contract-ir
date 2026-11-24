//! Exact I04 contract-version refusal on the direct V2 byte reader.

use super::evidence::CheckedPackageEvidence;
use super::shared::{CheckedPackageIncomplete, CheckedPackageReadLimits, CheckedPackageRefusal};
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
