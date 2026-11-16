---
id: SR-17940486709564847048034
title: "spec-review of QSL-2254 FR-038 timed clock kind text"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1030 at 68d9732e; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (temporal bindings prose, FR-038-AC-482..484)"
review_set: subset
---

## Summary

Spec review of the FR-038 change in PR #1030. Ticket: QSL-2254. Upstream sources read: filament-specification origin/main 959a1982 FR-322 "Temporal clock bindings" (the four-kind `clock` table, the two `filament.temporal.timed/v1` cases, the row check order), FR-540, FR-541, and `proposals/checked-package-v2/timed-clock-no-row-vectors.json` version 2. `quire validate` of the repo spec tree passes.

## Examined scope

- FR-038 `ClockConfiguration` paragraph (examined): the four closed kinds, `timed` exactly `{kind, unit}` with non-empty unit, malformed_wire for a missing, non-text or empty unit or extra member, unknown kind refused. Agrees with the FR-322 table. Its missing AC is FND-001 of the gap-analysis SR.
- FR-038 `filament.temporal.timed/v1` paragraph (examined): model subject has `model-time` and no row; trace subject has exactly one `timed` row per claim occurrence; missing trace row at the clause body; non-`timed` or `model-time`/`model-steps` row at the whole row. Agrees with the FR-322 timed bullets and TN-01..TN-09.
- FR-038 coverage sentence (examined): a timed clause is clock-configured over a trace subject only. Agrees with FR-322.
- FR-038 monitor read paragraph (examined): model clause reads ModelTime, trace clause reads its `timed` row. Agrees with the removal of `TraceTime`.
- FR-038-AC-482, AC-483, AC-484 (examined): each names its TN cases, code, cause and locus; marked QSL-2254.
- FR-322 row check order (context_only): "a profile/configuration kind mismatch refuses with the same code and cause at `/temporal_bindings/{i}/clock/kind`".

## Verdict

The FR-038 text is a faithful consumer of FR-322 and the TN vectors. Following TN-09 (whole row) over FR-322's general `/clock/kind` rule is right: FR-322's own timed bullet places these refusals "at the clause body or the row", and the vector is the executable oracle. But the conflict is not recorded anywhere: not in FR-038, not in the PR body. Separately, neither FR-038 nor FR-322 says which case a hybrid-model subject falls under, although the code decides it.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-038 places a non-`timed` trace row (TN-09) at the whole row without noting that this departs from FR-322's general rule (kind mismatch at `/temporal_bindings/{i}/clock/kind`); FR-322's timed bullet says only "at the clause body or the row". Neither FR-038 nor the PR body flags the discrepancy for the spec lane | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1958-1963 |
| FND-002 | medium | FR-038 splits timed clauses into "model subject" and "trace subject" but does not say a hybrid-model `over` parameter counts as a model subject (no row, model-time); the code decides it in `hybrid_subject`, and FSpec FR-322 is silent too, so a second implementer could require a row for it | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1950-1966; crates/filament-contract-model/src/checked_package/v2/timed_model.rs:2713-2783 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-038 now records that TN-09 is placed at `/temporal_bindings/{i}` following the vector, where FR-322's general rule names `/clock/kind`, and the PR body records it too. Both still say "to be raised with the planner". The planner has since filed QSL-2310 (item 1), so the text should cite QSL-2310 instead. That is a wording refresh, not a blocker. |
| FND-002 | fixed | FR-038 now states that an `over` parameter typed by a model, a model reference or a hybrid model has a model subject. It marks the hybrid case as this repository's reading because FR-322 does not name it. FR-038-AC-547 and `hybrid_subject_timed_clause_takes_no_clock_row` pin the rule, and mutation H3 makes the pin test fail. The upstream statement is QSL-2310 item 2; citing it here is the same wording refresh as FND-001. |

