---
id: SR-17940486709564847071
title: "Code review — filament-contract-ir QSL-2333/QSL-2319 batch"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; Cargo.lock, Cargo.toml, crates/filament-contract-model/Cargo.toml, crates/filament-contract-model/src/checked_package/v2/{flat_wire.rs,operation_catalog.rs,operation_catalog_index.rs,operations.rs,timed_model.rs,vocabulary.rs}"
review_set: subset
---

## Summary

Reviewed PR #1059's diff from base `d2f7c767b4289cbe4b1418cce9a9a2dcabf1775e`, including the FR-772 slot-domain reader, FR-322 point-anchor validation, catalog pin and test coverage. The focused tests and formatting pass, but a non-`presence_read` slots application bypasses the new structural validator, the quantifier-domain test is untraced, and the parent-reported CI `make lint` fails on five string-edge occurrences.

## Verdict

**CONDITIONAL** — three medium findings remain, including a failed CI lint gate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A `filament.op.option.slots` application with a semantic form other than `presence_read` returns `false` from the special validator and falls through to generic catalog operand checks, admitting it without the required place/owner/selector shape and bounded `Set<D>` result checks. | crates/filament-contract-model/src/checked_package/v2/operations.rs:4757,2065 |
| FND-002 | medium | `make lint` fails on five new unmarked/unlisted string-edge occurrences, preventing the PR's Rust Checks gate from passing. | crates/filament-contract-model/src/checked_package/v2/operations.rs:4923,4927,5081; crates/filament-contract-model/src/checked_package/v2/timed_model.rs:205,1774 |
| FND-003 | medium | The test that verifies slots are usable only as a quantifier domain has no `#[trace]` tag, so this FR-772-AC-2 behavior has no test binding in the changed code. | crates/filament-contract-model/src/checked_package/v2/operations.rs:13137 |

## Rust review

- `OperationConstraint` decoding permits an absent operand/family only for `quantifier_domain_only`; catalog tests verify that other operandless constraints remain rejected.
- The slot-domain handler checks that the operation is a `presence_read`, verifies a timed-model place, final `slot` path step, selector count, bounded `Set<D>` result, owner model type and remaining selectors. The early return on other semantic forms is the bypass in FND-001.
- Quantifier use checks restrict direct references to operand 0 of `forall`/`exists`; binder checking compares against the final place selector type.
- Point-anchor validation admits only the final binding at guard/reset-place/reset-parameter positions, and checks same model/operation plus pre for guards and post for resets. The canonical ordering key includes optional point-anchor presence and content.
- The catalog pin is the exact merged revision `2714219621cfbfabdd8d411c39d9109d956b85dd`; it is a rev pin, with no tag move.

## Gates

| Gate | Result |
| --- | --- |
| `cargo fmt --all -- --check` | pass |
| `cargo test -p filament-contract-model fr772_ -- --nocapture` | pass; 3 tests |
| `cargo test -p filament-contract-model point_anchor_must_be_final_and_match_the_clause_execution_point -- --nocapture` | pass; 1 test |
| GitHub Rust Checks `make lint` | fail; parent reports five new unmarked/unlisted string-edge occurrences at the paths listed in FND-002 |

## Assurance Context

No applicable AssuranceProfile was found for the changed behavior.
