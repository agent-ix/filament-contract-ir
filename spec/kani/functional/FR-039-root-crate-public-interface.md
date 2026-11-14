---
id: FR-039
title: "Expose the filament-contract-ir root crate's bounded-Kani interface"
type: FR
relationships:
  - target: ix://agent-ix/filament-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/filament-contract-ir/FR-019
    type: depends_on
  - target: ix://agent-ix/filament-contract-ir/FR-028
    type: depends_on
  - target: ix://agent-ix/filament-contract-ir/AD-001
    type: references
  - target: ix://agent-ix/filament-spec-language/ADR-011
    type: references
---
# FR-039: Expose the filament-contract-ir root crate's bounded-Kani interface

## Contract

```yaml
name: ContractIrRootApi
ownership: filament-contract-ir
modules:
  - kani            # FR-029..FR-031 kani-bounded/1 profile boundary
re_exports: [] # model items are imported from filament_contract_model (FR-019)
invariants:
  - no public item is a filament_contract_model item under a second path
  - no public item is a FSL runtime type, and nothing calls filament_spec_language::runtime
  - replay, witness, counterexample-envelope, terminal-record and obligation-identity types are fsl_replay's
  - every conversion from untrusted input is fallible and returns a typed error
```

## Description

The `filament-contract-ir` root crate shall expose exactly the public items
listed below and shall re-export no `filament_contract_model` item: the model's
public items have one import path, `filament_contract_model`, which
[FR-019](../../model/functional/FR-019-rust-library-interface.md) governs. It shall expose no
counterexample packet, witness, replay source, replay agreement, terminal
record or obligation identity of its own: those are FSL `fsl-replay` types,
and where the root crate names one it names FSL's.

## Inputs

A checked clause, a `kani-bounded/1` profile selection and a finite input.

## Outputs

The `kani-bounded/1` profile selection, validated finite
inputs, dispatch routes and typed `KaniOutcome`s. Nothing is FSL-typed: the
`KaniOutcome` to `TerminalValue` map is owned by
`agent-ix/filament-contract-codegen`.

## Behavior

### Public items

| Module | Public items | Owning requirement |
| --- | --- | --- |
| `kani` | constant `PROFILE`; types `KaniProfile`, `ProfileSelection`, `ProfileError`, `CapabilityDisposition`, `CapabilityEntry` | FR-029 |
| `kani` | types `FiniteInput`, `FiniteObject`, `FiniteReference`, `PopulationCompleteness`, `ResourceBounds`, `ValidatedFiniteInput`, `KaniOutcome`, `KaniOutcomeKind`, `KaniCancellationSource`, `KaniOutcomeError` | FR-030, STD-001 |
| `kani` | types `DispatchIndex`, `DispatchError`, `ModuleDescriptor`, `SemanticFamily` | FR-031 |

An item's own public fields, variants and inherent methods are part of the
item and are not listed separately. This includes the nine public infallible
non-success constructors on `KaniOutcome` specified by
[FR-030](./FR-030-bounded-kani-domain-and-outcomes.md), AC-8 and AC-9.
`NonSuccessKind` remains private; this addition exports no new outcome-kind
type. These constructors accept typed, already selected inputs rather than
untrusted wire input, and their signatures return `KaniOutcome` directly.

The root crate depends on no FSL crate (FR-028), so it has no dependency on
`filament_spec_language::runtime` or `fsl-replay`; the native-runtime replay
entry point `replay_with_native_runtime` and its `Native*` agreement types are
removed (IR#140, FSL ADR-011). The map from a `KaniOutcome` to a
`fsl_replay::TerminalValue` is not part of this interface: it is owned by
`agent-ix/filament-contract-codegen`, which depends on both sides, and is tracked
there under Linear IR-358. Contract IR exposes only its own Kani outcome types.

### Items codegen owns

`lower_checked_arithmetic`, `lower_query`, `lower_reaches`,
`ArithmeticLowering`, `CheckedArithmeticRequest`, `CollectionLowering`,
`CollectionQuery`, `QueryKind`, `GraphLowering` and `GraphRequest` are not
part of this interface. The family lowerings for checked arithmetic,
collections and objects belong to the backend adapter in codegen, and the
root crate has no `arithmetic`, `collections` or `objects` module.

### Items FSL owns

`CounterexamplePacket`, `PacketIdentity`, `ReplaySource`, `ReplayAgreement`,
`WitnessReplayAgreement`, `InputReplayAgreement`,
`replay_counterexample`, `Witness`,
`WitnessBinding`, `WitnessCheck`, `WitnessValue`, `WitnessValueType`,
`KaniProviderResult` and `KaniProviderRecord` are not part of this interface.
Their roles are FSL's `WitnessEnvelope`, `ReplaySource`, `Witness`,
`ReplayResult`, `TerminalValue`, `TerminalRecord`, `ObligationIdentity` and
`replay` (FSL FR-069 through FR-073, FR-098); the Kani transcript parser is
the codegen backend adapter's.

### Error surface

Every public function over untrusted input returns a typed error rather than
panicking; `KaniOutcomeError` codes are registered in STD-001. The `code` of a
`KaniOutcome` and of a `KaniOutcomeError` is `filament_contract_model::Std001Code`
(FR-044), which the root crate names but does not re-export.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-039-AC-1 | The root crate's public items are exactly those the Public items table lists, checked by a public-signature inventory of `src/` that fails on an added or missing item; `src/lib.rs` re-exports no `filament_contract_model` item, and code naming a model item through `filament_contract_ir` fails to compile. | Test (TC-055) |
| FR-039-AC-2 | No public item names a `filament_spec_language::runtime` type and no source file under `src/` names `filament_spec_language::runtime`. | Test (TC-055) |
| FR-039-AC-3 | Code naming any item the "Items FSL owns" or "Items codegen owns" section lists through `filament_contract_ir` fails to compile, and no public item of the root crate names `fsl_replay::TerminalValue` or maps a `KaniOutcome` to one. | Test (TC-055) |
| FR-039-AC-4 | Every public `kani` function over untrusted input returns a typed error with no public panic path. | Test (TC-055) |

## Dependencies

[FR-019](../../model/functional/FR-019-rust-library-interface.md) governs the model crate's
surface, which this crate does not re-export.
[FR-028](../../model/functional/FR-028-separate-cycle-free-contract-model.md) fixes the
model/root package split. [AD-001](../../assurance/AD-001-contract-ir-architecture.md)
records the module boundaries, the one-path rule for public items, the
FSL replay-type ownership and the placement of the Kani family lowerings in
codegen's backend adapter.
