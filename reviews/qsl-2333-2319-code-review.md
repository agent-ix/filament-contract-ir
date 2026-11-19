---
id: SR-17940486709564847071
title: "Code review — filament-contract-ir QSL-2333/QSL-2319 batch"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; Cargo.lock, Cargo.toml, crates/filament-contract-model/Cargo.toml, crates/filament-contract-model/src/checked_package/v2/{flat_wire.rs,operation_catalog.rs,operation_catalog_index.rs,operations.rs,timed_model.rs,vocabulary.rs}, tests/it/checked_package_v2_catalogue_differential.rs, tests/it/checked_package_v2_typed_terms.rs"
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

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | `tc_048_every_catalogue_entry_retains_its_constructible_admission` claims FR-772-AC-1, but explicitly classifies `filament.op.option.slots` as unavailable instead of testing strict-reader admission and structural sharing. | tests/it/checked_package_v2_catalogue_differential.rs:417,1175 |
| FND-005 | medium | The same catalog-wide test claims FR-772-AC-2 while recording slots as unavailable; it does not test any of the required slot-domain refusals. | tests/it/checked_package_v2_catalogue_differential.rs:417,1175 |
| FND-006 | medium | `admitted_controls_expose_missing_catalog_constraint_productions` claims FR-772-AC-1, but its control list has no slots operation and it does not admit the required quantifier package. | tests/it/checked_package_v2_typed_terms.rs:463,472 |
| FND-007 | medium | The same controls test claims FR-772-AC-2, but does not exercise a slots consumer/refusal; its `quantifier_domain_only` operand assertion is unreachable for the listed controls. | tests/it/checked_package_v2_typed_terms.rs:463,525 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1: non-`presence_read` slot operations now refuse at `semantic_form`. After excerpt: `if application.node.semantic_form.as_ref() != "presence_read" { return Err(application.refuse_read(` |
| FND-002 | fixed | Round 1: the string-edge scan and both workspace/model Clippy lanes passed on head `7982bcb`. After excerpt: `string-edge: no unmarked, unlisted string comparison or string match found` |
| FND-003 | fixed | Round 1: the consumer-exclusion test carries `#[trace("TC-1862", "FR-772-AC-2")]`. |
| FND-004 | still-open | The new trace tag remains on a test whose catalog loop marks slots unavailable; remove the FR-772-AC-1 binding or add a test that asserts the AC. |
| FND-005 | still-open | The new trace tag remains on a test whose catalog loop marks slots unavailable; remove the FR-772-AC-2 binding or add a test that asserts the refusal cases. |
| FND-006 | still-open | The new trace tag remains on a controls test whose listed operations omit slots; remove the FR-772-AC-1 binding or exercise strict-reader admission. |
| FND-007 | still-open | The new trace tag remains on a controls test that exercises no slots consumer/refusal; remove the FR-772-AC-2 binding or exercise those refusals. |
| FND-004 | fixed | The catalog census now has only TC-1862 and FR-038-AC-251; the FR-772 AC tags were removed. Dedicated FR-772 model tests remain tagged. |
| FND-005 | fixed | The catalog census no longer claims FR-772-AC-2; dedicated tagged tests exercise slot-domain refusals. |
| FND-006 | fixed | The typed-terms census now carries only FR-349-AC-8/9 and TC-1862; the FR-772 AC tags were removed. |
| FND-007 | fixed | The unreachable operandless operands branch is removed; no FR-772 AC tags remain on the controls census. Dedicated tagged tests remain. |

## Disposition pass 2 verification

- Head/base: `9a733e625456679f6d881859b1599b80dbdf9618` / `c6dc5c3e5abf3d245a91ac79546d8bafda1e2e56`.
- FND-001–003 remain fixed on the rebased head.
- `cargo fmt --all -- --check`, `make lint`, four `fr772_` model tests, the point-anchor test, and both changed catalog/typed-terms integration test filters pass.
- Full aggregate tests were not run in this disposition pass.
## Disposition pass 3 verification

- Head/base: `265ce2bc1ecfe36e7f62f03251dc302100b84d2b` / `c6dc5c3e5abf3d245a91ac79546d8bafda1e2e56`.
- FND-004–007 fixed by removing unsupported trace claims; dedicated FR-772 tests continue to cover the behavior.
- Prior FND-001–003 remain fixed.
- `tc_048_every_catalogue_entry_retains_its_constructible_admission`, `admitted_controls_expose_missing_catalog_constraint_productions`, four `fr772_` model tests, point-anchor test, formatting, `make lint`, and `git diff --check` pass.
- `quoin matrix` reports 1,257 criteria: 1,104 tagged, 122 untagged, one tagged only by an ignored test, 26 method-without-symbol, and four retired. This remains a repository-wide baseline outside the changed criteria.
- `quire coverage --scope . --json` exited 0, reported 143 repository-wide unbacked rows and existing symbol-id-collision diagnostics in `common.rs`, `checked_term_data.rs`, `owner.rs`, `refinement.rs`, and `temporal_binding.rs`; no FR-772 AC or changed-test rows were unbacked.
- No new findings. Full aggregate tests were not run.

## Disposition pass 4 recheck

- Reviewed current main base `a461f0a1b18542b0a47e1e1676ef9015af48e8f4` against head `01fffc2720fb80c111d7e87af828500f1edb2cbf`.
- The rebase-only manifest changes align all filament-exact references to `b26a8e4c592503fccdc4a0e3d2cce23dd8487616`, semantic-value to `1df71c27e6db56c8118cf017689aa7fab6156cde`, and verification-contracts to `2714219621cfbfabdd8d411c39d9109d956b85dd`; Cargo.lock matches. No new code issue found.
- FND-001–007 remain fixed, including the four removed FR-772 census bindings. Dedicated FR-772 tests remain tagged.
- On this head, `cargo test --locked -p filament-contract-model fr772_ -- --nocapture` passed (4 tests); `cargo fmt --all -- --check` and `git diff --check` passed. The coder reports both focused integration filters and `make lint` pass on this head. The reviewer-run catalogue integration filter was stopped while recompiling after the pin update; it produced no failure output.
- No new findings. Full aggregate tests were not run.
