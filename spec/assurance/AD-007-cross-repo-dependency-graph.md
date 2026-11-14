---
id: AD-007
title: "Cross-repo dependency graph and shared concept owners"
type: ArchitectureDescription
status: accepted
owner: kreneskyp
system: the Cargo dependency edges among filament-spec-language, filament-contract-ir, filament-contract-codegen, filament-contract-runtime, filament-specification, quire-canonical, filament-verification-contracts and filament-walk, the crate that owns each shared concept, and their remaining duplicate-owner gaps
relationships:
  - target: ix://agent-ix/filament-contract-ir/AD-001
    type: references
  - target: ix://agent-ix/filament-contract-ir/AD-004
    type: references
  - target: ix://agent-ix/filament-contract-ir/AD-005
    type: references
  - target: ix://agent-ix/filament-contract-ir/AD-006
    type: references
  - target: ix://agent-ix/filament-contract-ir/FR-028
    type: references
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/filament-contract-ir/FR-039
    type: references
  - target: ix://agent-ix/filament-spec-language/ADR-011
    type: references
---
# Cross-repo dependency graph

AD-004 describes the FSpec-to-IR reader seam; AD-005 and AD-006 describe IR's FSL
and codegen consumers. This AD records the current repository edges and concept
owners. The dependency manifests and source locations below identify the
owner boundaries considered by this decision.

## System Boundary

In scope are first-party Cargo dependencies among IR, CG, RT, FSL, FSpec,
quire-canonical, filament-walk, filament-verification-contracts, filament-exact,
filament-semantic-value and filament-eval, plus the ownership of concepts they share.
This AD does not prescribe internals of another repository or a compatibility
path. FSpec owns the published wire contract; it is not a runtime dependency
of IR.

This is acceptance of the staged dependency and ownership architecture. RT's
accepted AD-001 through AD-004 in `filament-contract-runtime`
record one temporary expression-port exception under IR-1009. The exception
does not complete the FR-275 one-owner end state. FSpec's source-integrity
text still conflicts with IR's two-member reader; IR-1273 was canceled and
AD-004 routes the owner-spec correction.

## Views

The repository edges give the dependency direction; the concept table assigns
the owning crate and records the one remaining duplicate implementation.

### Repository edges and direction

| Consumer | Direct first-party production edges relevant to this seam | Evidence and disposition |
| --- | --- | --- |
| IR model | quire-canonical, filament-walk, filament-verification-contracts | `crates/filament-contract-model/Cargo.toml`; IR has no production edge to FSL, CG or RT. |
| FSL | IR's `filament-contract-model`, filament-exact, filament-semantic-value, filament-eval, filament-walk, quire-canonical | FSL `Cargo.toml:7-28`; the shared leaves are separate Git repositories. FSL's local package name is now `filament-contract-model`. |
| CG | IR root and model, RT, FSL `fsl-replay`, filament-exact, filament-semantic-value, filament-eval and quire-canonical | CG `Cargo.toml:28-46`; the root crate supplies Kani outcome types, while checked-package types come from the model crate (AD-006). |
| RT | optional filament-exact and filament-semantic-value under its `exact` feature | RT `Cargo.toml:30-38`; neither is fetched from FSL's repository. |
| FSpec qualification | verification contracts and trace tooling as dev edges | FSpec's specification remains the authority for wire semantics, not a linked library in the reader. |

The normal dependency direction through the main seam is FSpec's contract to
IR's reader, then FSL's producer verification and CG's consumer. CG also uses
RT's exact runtime and FSL's replay facade. No IR manifest points back to
FSL, CG or RT, and no FSL manifest points to CG or RT. These edges therefore
do not form a repository dependency cycle. A code reference to another owner
is not itself a Cargo edge.

### Concept owners and remaining duplicate

| Concept | Owner | Current source and finding |
| --- | --- | --- |
| Exact scalar, text, composite kernel, `Meter` and `Outcome` | standalone filament-exact | FSL `Cargo.toml:12-15`, CG `Cargo.toml:31-32`, RT `Cargo.toml:35-38` take the shared crate. RT `src/exact/mod.rs:6-9` says its scalar and metering copies have moved. The former count of 23 duplicate RT files and 124 shared public names no longer describes current main. |
| Semantic values, declarations and shared checking vocabulary | standalone filament-semantic-value | The same three consumers declare direct dependencies. RT `src/exact/mod.rs:24-30` imports shared definition, node and value types. |
| Expression checking and evaluation | shared evaluator as end-state owner; RT runtime-maintainers hold a temporary exception | RT `src/exact/mod.rs:8-9,33-39` retains the existing `expression.rs` implementation and `mod.rs` public surface. IR-1009 and RT AD-004:139-159 bound this staged exception to those existing paths, with no new copy, semantic expansion or consumer use. The usable shared `no_std + alloc` evaluator is deferred; the port is not a second settled owner. |
| Checked-package wire types, strict reader and lowering | IR model, with FSpec owning the published wire | FSL and CG consume the model crate; AD-004 routes correction of FSpec's unsupported source-integrity rule, with no IR reader change scheduled. |
| Compile-side package and emitter | FSL `fsl-package` | FSL produces and verifies through IR's model reader (AD-005). |
| Kani outcome type | IR root crate | CG imports the root for this surface (AD-006); FSL `fsl-replay` owns replay envelopes and terminal records. |
| Generated-code runtime | RT | Native exact execution is distinct from FSL's source evaluation and CG's generation. |
| Canonical JSON encoding | quire-canonical | IR, CG and FSL consume the owned encoder. QVC also depends on it (`filament-verification-contracts/Cargo.toml:37`); QVC's `jcs_canonicalize` delegates to `quire_canonical::to_vec` (`filament-verification-contracts/src/lib.rs:616-623`). |
| Verification-facing JCS equality and SHA API | filament-verification-contracts | QVC FR-001:13-15,25-37,54-79 owns the public helpers over quire-canonical's `Encode` contract; `src/lib.rs:629-644` implements the equality and SHA wrappers over canonical bytes. They are not a second encoder implementation or an unresolved migration. |
| First-party lock coherence command | quire-cli `quire check-lock` | IR `Makefile:188-192` invokes the owned command. The copied IR `scripts/check_one_copy.awk` was removed by IR-641. Each repository qualifies its own lock. |

The FSL workspace still contains `fsl-walk-grow`; it is not the extracted
filament-walk crate. Any question about consolidating stack-growth calls is
separate from the exact-value extraction and cannot justify another owner in
IR. The shared leaves are already separate repositories, so the former
question of whether to extract them is closed by current manifests.

## Decisions

- One crate owns each shared exact and semantic-value concept. Consumers use
  the standalone filament-exact and filament-semantic-value packages directly.
  RT's remaining `expression.rs` and `mod.rs` public path are the one bounded
  temporary exception recorded in IR-1009 and RT AD-004:150-159, owned by
  runtime-maintainers. No new evaluator copy, semantic expansion or consumer
  use is covered. The exception expires when the shared evaluator's usable
  `no_std + alloc` implementation and ownership land and pass RT's governed
  checked-input and generated-body acceptance; IR-1009 then deletes the port
  and old public path in the paired RT/CG cutover. IR-590 is a deferred
  predecessor, so an interface alone does not satisfy expiry. FR-275's
  end-state criteria remain open until that evidence exists.
- IR owns its reader and model API, not FSL's emitter or CG's generation.
  CG's root IR edge is deliberate for Kani outcomes; its model edge serves
  checked-package types. FSL's model edge points only from FSL to IR.
- The published FSpec wire is read by reference. Its source-integrity revision
  remains inconsistent with IR's two-member source references after IR-1273
  was canceled; AD-004 routes the owner-spec correction without changing
  dependency direction.
- IR uses the released quire-cli lock checker. Its removal of the copied AWK
  checker does not establish lock qualification for FSL, CG or RT.

## Risks

| Risk or open work | Owner and control |
| --- | --- |
| RT's ported expression checker remains under `src/exact/expression.rs` beside the shared evaluator target | RT runtime-maintainers own IR-1009's bounded exception and its expiry at the usable shared `no_std + alloc` evaluator cutover; RT AD-004:150-159,204-206 leaves FR-275 end-state completion open. |
| FSpec's source-integrity text conflicts with the canceled IR-1273 decision | FSpec FR-322/FR-323 and FSL FR-518 require owner-led correction; AD-004 keeps published and implemented shapes distinct until then. |
| CG's direct IR-root edge outlives its Kani outcome use | CG owns the import audit (AD-006); no bridge is added to IR. |
| A first-party lock resolves two revisions of the same crate | Each consuming repository owns its lock qualification. IR invokes `quire check-lock` (`Makefile:188-192`); CG `make deny` currently invokes only `cargo deny check` (`filament-contract-codegen/Makefile:145-147`), and IR-1292 owns that missing CG gate. |
