---
id: SR-2026112109235102
title: "PR 1111 changed-surface trace and evidence gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; QSL-2400; PR 1111 changed behavior, FR-038-AC-598/599 and FR-019"
review_set: subset
relationships:
  - {target: "ix://agent-ix/filament-contract-ir/FR-038", type: references}
---

## Summary

Repository-driven matrix, reverse-gap and stub inspection with merge findings restricted to PR 1111's changed behavior. Plan completion: not assessed. Static tags exist for both added criteria, but important behavioral evidence is absent.

## Verdict

CONDITIONAL for the changed surface. This is not a repository-wide assurance PASS: the full matrix includes pre-existing untagged and ignored-only criteria outside this review's diff.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The hyper source-unit tests never exercise timestamp-unit or duration-bound resolution in the stated source unit. | tests/it/checked_package_v2_temporal.rs:5813 |
| FND-002 | medium | Wrapper binder evidence covers only seq, and that test lacks the FR-038-AC-598 trace; let, quantifier and fold levels in wrapper leaves have no new coverage. | tests/it/checked_package_v2_temporal.rs:5452 |

## Finding details

FND-001: The added AC-599 tests use hyper_clock_fixture's event-position profile and inspect source-unit row presence, locking, repeated occurrence identity and forbidden carriers. They never declare source-owned Time/s/ms nodes, configure a timestamped clock, or place a duration aggregate in the wrapper. Add the FR-322-AC-511/FR-778-AC-13 positive, foreign-source missing-name negative, wrong bound owner/unit negative and missing-source-before-unit-resolution precedence case. Include shared-wrapper contexts to prevent the implementation defect recorded separately in SR-2026112109235101.

FND-002: The modified tc_048_a_seq_binder_in_a_clause_leaf_resolves_its_level_from_the_binders_in_scope checks wrapper seq bases 0, 1 and 2, but is tagged only TC-048, so none of that binder behavior binds to AC-598. The three tests the matrix binds to AC-598 cover slot order, duplicates and over, not leaf binders. Add the AC-598 trace to the seq test and focused wrapper cases for let, a quantifier, and fold (accumulator n, element n+1), with wrong levels refused. Coordinate with the preceding binder-link PR rather than fixing its baseline here.

## Scope

- FR-038-AC-598, examined: "A binder a leaf of the wrapper introduces (`seq`, `let`, a quantifier, `fold`) takes level `n` for its first, replacing the refusal of binder forms in wrapper leaves."
- FR-038-AC-599, examined: "The clause's `temporal_bindings` rows carry none, and its `timestamp_unit` and its wrapper's duration bounds resolve only in that unit, after these checks, never inferred."
- FR-019, context_only: "`ClaimNameRow` gains the optional field source_unit (FR-322 Hyper clause source unit), a breaking change that rides the next contract-ir tag (FR-371 Compatibility)."

## Coverage

Reconciliation: quire matrix (CLI 0.52.0, engine 0.62.2), static axis only; no run-evidence store read. Full matrix: tagged 1150; untagged 115; tagged-by-ignored-test 1; method-without-symbol 26; retired 8. Both new criteria are tagged by three tests each. The full-tree ignored-only row is FR-028-AC-4, unrelated to this patch. Coverage also reports 286 untracked symbols and 1956 diagnostics across the repository; the two tags in the changed temporal test file name external FSpec/TC ids on an unchanged test and are not introduced by this patch.

Plan completion: not assessed

Reverse-gap inventory of changed behavior: locked hyper source carrier, source-scope clock/bound propagation, wrapper scope validation and wrapper binder base, all owned by FR-038-AC-598/599 and upstream FR-322/402/778. No unowned changed behavior or source/test stub was found. The behavior tests use the public reader and real graph data. Existing whole-tree extraction is reported for context, not represented as a full-tree semantic audit. Optional separate semantic fan-out was not run; direct code-test intent checking belongs to the accompanying independent code review. No tests or gates were executed by this reviewer.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, head f23a39ec8693f1b0d32b68794c18db1262252b20: new public-reader tests declare source-owned Time/s/ms, admit matching timestamp/duration units, refuse a wrong bound unit and a foreign source lacking the clock unit, and prove missing claim source precedes unit resolution. Shared-wrapper contexts have positive and negative controls in both owner orders, and finite-domain timestamp scope is covered. |
| FND-002 | fixed | Round 1, same head: seq now carries FR-038-AC-598. New traced let/forall and fold/reduce cases put seq within each binder body, admitting only depths n+1 or n+2 and refusing adjacent wrong levels. The matrix now binds these behavioral tests to AC-598. |

## Round 1 verification

CLEAR for the reviewed changed surface at f23a39ec8693f1b0d32b68794c18db1262252b20; no new gap finding. This does not assert a repository-wide assurance PASS or qualification of unrelated baseline criteria.

FND-001 after excerpts: `fn hyper_wrapper_duration_resolves_only_in_its_claim_source_unit() {` and `fn shared_hyper_wrapper_checks_every_source_and_clock_in_both_orders() {`. Their bodies assert admission with own declared units, exact foreign-source and wrong-bound refusals, missing-carrier precedence, and both owner orders. The additional finite-domain case admits ms in its claim source and refuses an unrelated locked source.

FND-002 after excerpt: `#[trace("TC-048", "FR-038-AC-598")]`. The wrapper tests admit let/forall nested seq only at level 3 for a two-slot scope and fold/reduce nested seq only at level 4, with adjacent levels refused. These witness binder depth, including both fold step binders, rather than merely asserting that the form exists.

Reconciliation: quire matrix CLI 0.52.0, engine 0.62.2, static axis only. Current matrix binds AC-598 to seven tests, including seq, let/quantifier and fold/reduce depth tests, and AC-599 to six tests, including source-owned duration, shared-owner and finite-domain tests. The engine emitted existing unmarked trace-grammar diagnostics across the tree; these did not prevent extraction of the new criterion bindings and are not relabeled as new gaps. No full-tree baseline repair is required by this bounded disposition.

Plan completion: not assessed

Execution evidence is the coordinator's retained 20-pass focused log and two-regression red-control log. No test command was run by this reviewer. The source-identity regression uses valid distinct unit names: same-name cross-source declarations are blocked earlier by the existing global declaration-name rule, so that exact hypothetical case remains unexecuted and is not claimed as tested. Final integration after the compact-body API rebase remains with the coordinator.
