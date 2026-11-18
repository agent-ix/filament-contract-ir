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
| `criterion_id` | `SourceAuthoredCriterionId` | Quire-authored criterion identity: `obligation_declaration: String`, `scoped_document: String`, and `criterion_id: String`. |
| `state` | `FormalizationState` | Exactly one value from the closed state vocabulary below. |
| `clause_address` | `Option<ClauseAddress>` | Absent until a formal clause exists; otherwise the exact address of that clause. |
| `evidence` | `Option<EvidenceEnvelope>` | The shared Engineering Assurance record for producer availability, observation, ownership, and Quoin handoff; absent only when no producer evidence exists. |

`FormalizationState` is the closed set `eligible`, `authored`, `parsed`,
`typechecked`, `lowered`, `backend_supported`, `proved`, `inconclusive`,
`rejected`, `unknown`, `unavailable`, and `not_applicable`. These values describe
the criterion's formalization coverage; they do not replace the shared evidence
availability state. The `EvidenceEnvelope` uses the Engineering Assurance
contract fields `producer_id: String`, `availability: Option<AvailabilityState>`,
`observation: OperatorObservation`, `governing: Option<GoverningVersions>`,
`next_action: Option<String>`, `owner: Option<String>`,
`boundary_rationale: Option<String>`, `quoin_reference: Option<String>`, and
`validation_errors: Vec<String>`. `AvailabilityState` is `observed`,
`unavailable`, `not_computed`, or `not_applicable`; the record's `state` supplies
the additional formalization-specific `unknown` and terminal classifications.

`ClauseAddress` has exactly these members:

| Field | Type | Meaning |
| --- | --- | --- |
| `package_identity` | `PackageId` (`String`) | Identity of the package containing the clause. |
| `criterion_id` | `SourceAuthoredCriterionId` | The same complete Quire criterion identity as the coverage-state record. |
| `clause_id` | `ClauseId` (`String`) | Identity of the clause within its requirement. |
| `source_span` | `SourceSpan` | The admitted clause source range, with `start` and `end` `SourceLocation` values. Each location carries a `SourceIdentity`, one-based `line: u32` and `column: u32`, and `byte_offset: u64`. |

The clause address SHALL NOT contain a requirement revision, statement hash,
digest, or file hash. The source span retains the source identity required by
FR-012; it does not add the requirement revision to the address.

The accounting boundary SHALL report each criterion exactly once, including
criteria whose state is `unknown`, `unavailable`, or `not_applicable`. It SHALL
preserve `rejected` and `inconclusive` as distinct first-class outcomes and
the accounting boundary SHALL NOT fold either into `proved`. A missing producer observation SHALL remain
`unavailable` or `unknown`, as appropriate, rather than being inferred as a
successful result. A state of `not_applicable` SHALL carry the shared
`boundary_rationale` when the non-applicability comes from a producer boundary.

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
| FR-071-AC-1 | Given a selected fixture with criteria at every formalization state, the output contains exactly one record for every criterion, no criterion is omitted or duplicated, and `unknown`, `unavailable`, and `not_applicable` remain explicit states. | Test |
| FR-071-AC-2 | Each `ClauseAddress` has exactly `package_identity`, `criterion_id`, `clause_id`, and `source_span`; two clauses with different package, criterion, clause, or source-span identity have different addresses, and no requirement revision, statement hash, digest, or file hash is an address member. | Test |
| FR-071-AC-3 | A clause address round-trips using the FR-011 package identity, FR-052 Quire criterion identity, FR-012 clause identity and source span; the address contains no requirement revision, while the source span retains its FR-012 source identity. | Test |
| FR-071-AC-4 | Evidence metadata validates against the shared Engineering Assurance `EvidenceEnvelope` and `AvailabilityState` contract; missing or invalid producer evidence is retained as a gap and cannot become `proved`. | Test |
| FR-071-AC-5 | A fixture with `rejected`, `inconclusive`, `unavailable`, `unknown`, and `not_applicable` records preserves each distinct state and its applicable reason or boundary rationale; none is counted as proved. | Test |
| FR-071-AC-6 | Reports group coverage independently by language, profile, module, repository, and criterion class, preserve claims, evidence, and gaps without an aggregate score, and render a 40-percent finding as `40%` rather than an unlabelled `0.4`. | Test |

## Dependencies

- [FR-011](./FR-011-package-identity.md) defines package identity.
- [FR-012](./FR-012-anchors-clauses-dependencies.md) defines clause identity
  and source spans.
- [FR-045](./FR-045-derive-proof-coverage-baseline.md) defines proof-coverage
  accounting and its claims/evidence/gaps boundary.
- [FR-052](../../model/functional/FR-052-source-obligation-correspondence.md)
  defines the Quire-authored criterion identity.
- Quoin [FR-204](ix://agent-ix/quoin/FR-204) defines corpus formalization
  states, and [FR-205](ix://agent-ix/quoin/FR-205) consumes this record.

## Status

Planned. The first producer integration and its evidence remain separate
implementation and verification work.
