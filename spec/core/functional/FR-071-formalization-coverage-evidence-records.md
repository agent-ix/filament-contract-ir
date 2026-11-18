---
id: FR-071
title: "Address clauses and record formalization coverage as evidence"
type: FR
relationships:
  - target: ix://agent-ix/filament-contract-ir/StR-004
    type: traces_to
  - target: ix://agent-ix/filament-contract-ir/FR-045
    type: depends_on
  - target: ix://agent-ix/filament-contract-ir/FR-052
    type: depends_on
  - target: ix://agent-ix/filament-contract-ir/FR-011
    type: references
  - target: ix://agent-ix/filament-contract-ir/FR-012
    type: references
  - target: ix://agent-ix/quoin/FR-204
    type: references
  - target: ix://agent-ix/quoin/FR-205
    type: references
---
# FR-071: Address clauses and record formalization coverage as evidence

## Description

When formalization coverage is reported for an ISO criterion, the Contract IR
accounting boundary shall retain exactly one explicit coverage-state record for
that criterion and shall identify every associated formal clause by a stable,
revision-free clause address.

## Inputs

- The selected package identity and its admitted formal clauses and source spans.
- The selected Quire criterion inventory and its typed criterion identities as
  defined by FR-052.
- Formalization-stage and terminal-outcome observations from their authoritative
  producers.
- Engineering Assurance `EvidenceEnvelope` values and Quoin evidence references
  when a producer has an observation or retained evidence.

## Outputs

- One formalization coverage-state record per criterion, retaining its exact
  criterion identity, one state, an optional clause address, and optional
  Engineering Assurance evidence metadata.
- Deterministic roll-ups grouped independently by language, profile, module,
  repository, and criterion class, with claims, supporting evidence, and gaps.

## Behavior

The coverage-state record has exactly these members:

| Field | Type | Meaning |
| --- | --- | --- |
| `criterion_id` | `SourceAuthoredCriterionId` | The Quire-authored criterion identity defined by FR-052. |
| `state` | `FormalizationState` | Exactly one value from the closed state vocabulary below. |
| `clause_address` | `Option<ClauseAddress>` | The exact address of the associated formal clause when one exists. |
| `reason` | `Option<String>` | A formalization-specific explanation for a non-success or unknown state, when required by the state rules below. |
| `evidence` | `Option<EvidenceEnvelope>` | Shared Engineering Assurance producer evidence; absent only when no producer evidence exists. |

`FormalizationState` is the closed set `eligible`, `authored`, `parsed`,
`typechecked`, `lowered`, `backend_supported`, `proved`, `inconclusive`,
`rejected`, `unknown`, `unavailable`, and `not-applicable`. These exact wire
tokens describe the criterion's formalization coverage and do not replace the
shared evidence-availability state. When present, `evidence` SHALL conform to
Engineering Assurance FR-004 and FR-009. Its producer observation, availability,
provenance, validation errors, and Quoin reference retain their shared meanings;
the formalization-specific `reason` does not replace or overload any of them.

The address-presence invariant is state-aware. When a coverage-state record has
state `eligible` or `not-applicable`, the record SHALL omit `clause_address`.
When a coverage-state record has state `authored`, `parsed`, `typechecked`,
`lowered`, `backend_supported`, `proved`, `rejected`, or `inconclusive`, the
record SHALL include `clause_address`. When a coverage-state record has state
`unknown` or `unavailable` and a formal clause exists in the selected package,
the record SHALL include `clause_address`; when no such clause is present, the
record SHALL omit it. Every present formal clause SHALL therefore have its
deterministic address even when its coverage state is `unknown` or
`unavailable`.

When a coverage-state record has state `rejected`, `inconclusive`, `unknown`,
`unavailable`, or `not-applicable`, the record SHALL include a non-empty
`reason`. When its state is `eligible`, `authored`, `parsed`, `typechecked`,
`lowered`, `backend_supported`, or `proved`, the record SHALL omit `reason`.
`reason` explains the formalization-state disposition. Engineering Assurance `validation_errors`
remain limited to invalid evidence-envelope input, `diagnostic_category` to the
operator-observed producer outcome, and `boundary_rationale` to the evidence
producer's applicability boundary. None substitutes for `reason`.

`ClauseAddress` has exactly these members:

| Field | Type | Meaning |
| --- | --- | --- |
| `package_identity` | `PackageId` | Identity of the package containing the clause, as defined by FR-011. |
| `criterion_id` | `SourceAuthoredCriterionId` | The Quire-authored criterion identity defined by FR-052. |
| `clause_id` | `ClauseId` | Identity of the clause as defined by FR-012. |
| `source_span` | `SourceSpan` | The admitted clause source range defined by FR-012. |

The clause address SHALL NOT contain a requirement revision, statement hash,
digest, or file hash. The source span retains the source identity required by
FR-012; it does not add the requirement revision to the address.

The accounting boundary SHALL report each criterion exactly once, including
criteria whose state is `unknown`, `unavailable`, or `not-applicable`. It SHALL
preserve `rejected` and `inconclusive` as distinct first-class outcomes and
the accounting boundary SHALL NOT fold either into `proved`. A missing producer observation SHALL remain
`unavailable` or `unknown`, as appropriate, rather than being inferred as a
successful result. A state of `not-applicable` SHALL carry its required
formalization `reason`; a producer boundary rationale, when present, remains in
the shared evidence metadata.

The accounting boundary SHALL retain explicit gaps beside claims and evidence
and SHALL NOT replace them with an aggregate score. When the accounting boundary
reports that no conflicts were found over 40% of the selected specification, the
accounting boundary SHALL display the coverage as `40%`, with the percentage unit visible.
Roll-ups SHALL keep
their language, profile, module, repository, and criterion-class populations
separate and SHALL not combine a missing or unavailable population into a
proved count.

The shared Engineering Assurance evidence contract owns generic evidence
availability and provenance. This requirement defines only the domain-specific
criterion state and clause address; it does not define a second generic evidence
envelope, evidence store, or report protocol. Quoin FR-204 remains the owner of
its corpus-wide `quoin.formalization-coverage/v1` report, and FR-205 consumes the
record and address defined here.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-071-AC-1 | Given a selected fixture with criteria at every formalization state, the output contains exactly one record for every criterion, no criterion is omitted or duplicated, and `unknown`, `unavailable`, and `not-applicable` remain explicit states. | Test |
| FR-071-AC-2 | Each `ClauseAddress` has exactly `package_identity`, `criterion_id`, `clause_id`, and `source_span`; two clauses with different package, criterion, clause, or source-span identity have different addresses, and no requirement revision, statement hash, digest, or file hash is an address member. | Test |
| FR-071-AC-3 | A clause address round-trips using the FR-011 package identity, FR-052 Quire criterion identity, FR-012 clause identity and source span; the address contains no requirement revision, while the source span retains its FR-012 source identity. | Test |
| FR-071-AC-4 | Evidence metadata validates against the shared Engineering Assurance `EvidenceEnvelope` and `AvailabilityState` contract; missing or invalid producer evidence is retained as a gap and cannot become `proved`. | Test |
| FR-071-AC-5 | A fixture with `rejected`, `inconclusive`, `unknown`, `unavailable`, and `not-applicable` records requires a non-empty formalization `reason`; each reason remains distinct from evidence `validation_errors`, `diagnostic_category`, and `boundary_rationale`, and none of these states is counted as proved. | Test |
| FR-071-AC-6 | Reports group coverage independently by language, profile, module, repository, and criterion class, preserve claims, evidence, and gaps without an aggregate score, and render a 40-percent finding as `40%` rather than an unlabelled `0.4`. | Test |
| FR-071-AC-7 | A fixture omits `clause_address` for `eligible` and `not-applicable`, requires it for `authored`, `parsed`, `typechecked`, `lowered`, `backend_supported`, `proved`, `rejected`, and `inconclusive`, and includes it for `unknown` or `unavailable` exactly when the selected package contains a formal clause; every existing clause therefore has one deterministic address. | Test |

## Dependencies

- [FR-011](./FR-011-package-identity.md) defines package identity.
- [FR-012](./FR-012-anchors-clauses-dependencies.md) defines clause identity
  and source spans.
- [FR-045](./FR-045-derive-proof-coverage-baseline.md) defines proof-coverage
  accounting and its claims/evidence/gaps boundary.
- [FR-052](../../model/functional/FR-052-source-obligation-correspondence.md)
  defines the Quire-authored criterion identity.
- Engineering Assurance [FR-004](ix://agent-ix/engineering-assurance/FR-004)
  and [FR-009](ix://agent-ix/engineering-assurance/FR-009) define the shared
  evidence availability, observation, provenance, and non-success contracts.
- Quoin [FR-204](ix://agent-ix/quoin/FR-204) defines corpus formalization
  states, and [FR-205](ix://agent-ix/quoin/FR-205) consumes this record.

## Status

Planned. The first producer integration and its evidence remain separate
implementation and verification work.
