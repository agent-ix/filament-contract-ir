---
id: SR-17940486709564847048076
title: "spec-review of QSL-2400 TemporalBinding trace member"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1051 at 93e09980 (base origin/main 71914a07); spec/checked_package/functional/FR-038-consume-checked-package-v2.md (FR-038-AC-561), spec/checked_package/matrix/{TC-480-hyper-clause-wrapper-trace-rows.md,tests.md}"
review_set: subset
---

## Summary

Spec review of the IR spec changes in PR #1051. Ticket: QSL-2400. Upstream text, read only, at filament-specification origin/main 0bf58005: FR-322 "Hyper clause rows", the row-order paragraph, FR-322-AC-294 and FR-322-AC-453, TC-1680, FR-402 quantifier Group, "Body wrapper" and "Hyperproperty step", and the `TemporalBinding` schema def.

The coder read FSpec at f4583226. These sections are unchanged between f4583226 and 0bf58005.

`quire validate` over spec/plan/reviews at head reports the same 10 grammar findings as clean main, none on AC-561 or TC-480. `quire coverage --strict` reports 79 unbacked rows and 0 contradicted statuses, as on main.

Numbering: FR-038-AC-561 is free. The highest AC on main is AC-560, and no open PR adds an FR-038-AC-5xx row.

The coder's two reported gaps (finite-domain rows, and two hyperproperties sharing one wrapper) and the AC-453 two-model vector are settled by the draco:QSL:arch ruling the coordinator relayed. Every hyper row in both domains is keyed on the hyperproperty node, and a wrapper-keyed row refuses at `/declaration`. Finite-domain clauses keep per-trace rows. A new case admits two different `sequence_authority` values. The sources are FSpec FR-322-AC-463/464 and TC-2027/2028 (W52). AC-561's wrapper and finite-domain sentences will be rewritten to that ruling, so they are not judged here. As a reading of FSpec at 0bf58005, both gaps were real upstream contradictions, not IR deviations:
- FR-322 states rows for a finite-domain hyper clause, but FR-402 gives it no `temporal_clause` node to key them.
- FR-402 does not say which hyperproperty owns a shared wrapper.

## Examined scope

- FR-038-AC-561, reader/writer/schema sentence (examined). It matches FR-322 "Hyper clause rows" and schema.json `TemporalBinding.trace: Nonempty`.
- FR-038-AC-561, the refusal list (examined). It matches FR-322's stated loci for absent and unknown trace, misplaced trace, repeated key and missing row. The out-of-order locus is FND-002, and the upstream cause wording is FND-003.
- FR-038-AC-561, wrapper keying and finite-domain sentences (context_only): superseded by W52.
- TC-480 and its tests.md row (examined): FND-001.
- FSpec FR-402 quantifier Group and "Hyperproperty step" shape check (examined): FND-004.

## Verdict

AC-561's trace-member, refusal and order text is a faithful IR consumer of FR-322-AC-294 and AC-453, apart from the parts W52 replaces. The one blocker is the TC-480 id collision with #1049. Three upstream asks go to draco:QSL:arch, and they can ride W52:
- state the locus of an out-of-order wrapper row (FND-002);
- fix the cause wording for a repeated key (FND-003);
- rule on a repeated quantifier variable (FND-004).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-480 id collision. Open PR #1049 (QSL-2347, opened 2026-11-18 20:20Z, before this PR's 21:30Z) adds `spec/checked_package/matrix/TC-480-source-operation-body-rows.md` and binds FR-373-AC-1..5 to TC-480. This PR adds `TC-480-hyper-clause-wrapper-trace-rows.md`. Whichever merges second makes two TC-480 documents, a duplicate id for `make spec` and an ambiguous `#[trace("TC-480", ...)]` binding. #1051 renumbers, because #1049 was opened and reviewed first. TC-481 and TC-482 are taken by open #1050 and TC-477 by #1046, so the next free id is TC-483. Rename the file, its `id`/title, the AC-561 verification cell, the tests.md row and the 9 `#[trace("TC-480", "FR-038-AC-561")]` attributes | spec/checked_package/matrix/TC-480-hyper-clause-wrapper-trace-rows.md:2; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6700; spec/checked_package/matrix/tests.md:182 |
| FND-002 | low | AC-561 states that a row whose trace comes before its predecessor's in quantifier order refuses `invalid_package`/`timed-node-reference` at `/temporal_bindings/{i}/trace`. FSpec FR-322 states the order (row order paragraph and "Hyper clause rows") and that "a row out of order ... is refused", but names no locus for a trace-order defect. The IR choice follows its existing `/declaration` and `/occurrence` order loci and is reasonable. Ask draco:QSL:arch to state the locus in FR-322 (W52 can carry it) so producers' adverse vectors agree | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6700 |
| FND-003 | low | Upstream wording conflict, which the IR resolves correctly. FSpec FR-322 "Hyper clause rows", second paragraph: "A wrapper row with no `trace` member, a `trace` naming no variable of the body, a repeated (`declaration`, `occurrence`, `trace`) key and a missing trace row refuse `invalid_package`/`timed-node-reference`", and later "with `invalid_package`/`conflicting-definition` for a repeated key". AC-294 and AC-453 say `conflicting-definition`, and AC-561 follows them. Ask draco:QSL:arch to drop "a repeated ... key" from the `timed-node-reference` list | filament-specification spec/objects/interfaces/FR-322-checked-package-artifact.md:3936-3944 |
| FND-004 | medium | No FSpec rule for a repeated quantifier variable. FR-402's Group shape check does not refuse two Groups binding one `variable`, and the reader admits such a hyperproperty (code-review FND-002, probe P4b). FR-322 requires "exactly one row per quantifier Group, keyed by its variable". For `s`, `s` that requires two rows with one key, which the same paragraph refuses as a repeat, so no package is valid and FSpec does not say which refusal applies. AC-561 is silent too. Exact ask for draco:QSL:arch: (a) state in FR-402 "Hyperproperty step" 1 (Shape) that a Group whose `variable` repeats an earlier Group's refuses `ambiguous_declaration`/`ambiguous-name` (or `invalid_semantic_graph`) at the later `variable` binding, or (b) state in FR-322 that rows are one per distinct variable. Then pin the ruled outcome in AC-561 | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6700; filament-specification spec/objects/interfaces/FR-402-hyperproperty-obligation-identity.md:51-56,184-193 |

## New findings (disposition pass 1)

Round 1, head f7cf238c. The upstream text is now FSpec origin/main 57574468, which contains W52/W54 f92839de. FR-322 "Hyper clause rows" now states:
- the out-of-order locus;
- `conflicting-definition` alone for a repeated key;
- finite-domain rows, with the profile taken from the lock at `/clock/kind` and per-occurrence `clock_name` agreement;
- a wrapper-keyed row refused at `/declaration`.

FR-402-AC-10 adds the repeated-variable Shape refusal.

AC-561 at head matches each of these and FR-322-AC-294, AC-453, AC-463 and AC-464. It also records the ordering note: the binding step runs before the hyperproperty step, so a repeated variable with two rows refuses `conflicting-definition`. TC-483's 14 steps cover AC-463 (shared wrapper, reversed order, third `match`-only trace) and AC-464 (the finite `Det` clause). quire validate and `coverage --strict` match main.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | medium | A new TC-483 collision, owned by #1053. Open PR #1053 (fsl-spec-w56, opened 2026-11-18T23:12Z; commit 52e711f5 "Spec: TC-483" at 23:10Z) adds `spec/checked_package/matrix/TC-483-body-wrapper-over-a-model-root.md`. This PR took TC-483 in ce11837b at 21:44Z and pushed it before #1053 existed, so #1053 renumbers. #1053 itself holds TC-484 and TC-485, so its next free id is TC-486. Not a blocker for #1051, but whichever merges second fails the duplicate-id check | spec/checked_package/matrix/TC-483-hyperproperty-trace-rows.md:2 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |
| FND-004 | deferred | FSpec f92839de added FR-402-AC-10 (repeated variable refuses `ambiguous_declaration`/`ambiguous-name` at the later Group's `variable`). The IR Shape refusal is ticketed separately (coordinator, outside this PR). AC-561 records the interim reading: two rows give `conflicting-definition` at the later row, and one row is not a missing row |
| FND-005 | still-open | New this round: #1053 must renumber its TC-483 to TC-486 |
| FND-005 | fixed | - |

## Disposition pass 2 notes

Round 2 (disposition pass 2), head 8cded29d: fix commit 0b8cf26e, plus a clean merge of origin/main 9edbd391. FSpec is origin/main ebfcb277. Gates at 8cded29d:
- `make string-edge`, fmt and both clippy lanes: clean.
- Full workspace test: 2251 passed, 21 failed. Clean main 9edbd391: 2231 passed, 21 failed. The failing sets are identical: 18 `monitor_plan::reached_model_read` and 3 `public_interface`.
- `make conformance-fspec`: the same 6 failures on both.
- quire validate and `coverage --strict`: identical to main (89 unbacked rows, 0 contradicted). AC-561 has 20 trace claims.
- Hand mutants M01 to M26 at the new head: 21 killed, now including M19 and M24. M12, M13, M14 and M23 remain equivalent survivors. M18 survives, deferred.

## Disposition pass 3 notes

Round 3 (disposition pass 3); no finding of this SR is open, so no row is added, head 71892dd4. The delta since 8cded29d is fix commit 9d4bcfa7, a merge of origin/main (including #1045 `validate_clock_configuration`), and the 71892dd4 `lock_tables` test literal `trace: None`. Per the arch ruling, a finite-domain hyperproperty always requires its rows. Under a lock whose selected `temporal_profile` is not clock-configured, it refuses `invalid_package`/`timed-node-reference` at `/semantic_graph/nodes/{N}/body`. Test `tc_483_a_finite_domain_node_under_a_profile_without_a_clock_refuses_at_its_body` and the AC-561 note cover this. FSpec is FSPEC-9a181ace.

Focused gates, single slot:
- `make string-edge`, fmt and both clippy lanes: clean.
- `--test it -- tc_483 tc_346`: 32 passed.
- model lib `temporal_binding`: 6 passed.
- `temporal_binding_public`: 1 passed.
- AC-561: 21 trace claims.

Hand mutants on the delta are all killed:
- N1: finite not forced to configured.
- N2: finite configured only under a clock-configured lock, the old M18 behaviour.
- M17, M19, M21, M24, M25 (re-run).

No full suite this round; the coordinator gates it.
