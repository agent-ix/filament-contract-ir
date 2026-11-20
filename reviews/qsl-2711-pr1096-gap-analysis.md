---
id: SR-17940486709564847048108
title: "gap-analysis of QSL-2711 PR #1096 against the ticket's main-reds scope"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1096 at 3fcf6a01 (merge base af45a983); QSL-2711 reds: conformance_fsl_ir1052, conformance_fsl_ir1236, conformance_fsl_ir760, selected_population_public, make spec, make deny"
review_set: subset
---

## Summary

Ticket: QSL-2711. Plan completion: not assessed. The PR changes only tests, so this analysis does not run a Quoin matrix pass. It is a manual check of the ticket's listed reds against the PR, and of the PR body's account of what remains red. The ticket asks for contract-ir main to be green on its CI gate. The PR fixes one of the listed reds and accounts for the others in its body.

## Examined scope

- conformance_fsl_ir1052 (examined): fixed by the PR. It fails on main and passes on the PR head against FSL main f040d334. See the code-review SR.
- conformance_fsl_ir1236 (examined): the body says it needs external `IR1236_FSL_PACKAGE`/`IR1236_FSL_MODEL` inputs. This is confirmed by `tests/conformance_fsl_ir1236/main.rs:18-20` (`expect("live FSL output path")`). The body's statement that it "passes with authored inputs" was not reproduced in this review.
- conformance_fsl_ir760 (examined): the body says FSL main's `population_v2_fixtures` example does not compile. Confirmed. `selected_population::duplicate_max_on_reader_admitted_population_is_not_population` panics at `tests/it/selected_population.rs:51` ("FSL producer failed"). Building the example directly against FSL f040d334 with `--features population-fixtures` fails with `E0063 missing field claim_names in initializer of CheckedPackageSidecarsRef` and `E0433 cannot find module or crate fsl_eval`, both in FSL's `src/population_fixtures.rs` (lines 451 and 233).
- selected_population_public (examined): it runs the same FSL producer (`crates/filament-contract-model/tests/selected_population_public.rs:59-87`). Both of its tests fail at line 87 with the same two FSL compile errors. The cause is FSL-side.
- make deny / make spec (context_only): the body's account (deny passes with QUIRE=quire-0.52 as CI runs it; spec is not a hosted gate) was not reproduced.

## Verdict

PASS for the PR's own scope, and the body is honest about what remains red. QSL-2711 is not complete on merge. ir760 and selected_population_public stay red until FSL main's `population_v2_fixtures` compiles again, so the ticket should stay open, or the work should move to an FSL ticket. One low finding: the PR title still lists items the PR does not fix.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | PR title "contract-ir main reds: fsl conformance fixtures, selected_population_public, make spec/deny" claims the whole ticket scope, but the PR fixes only ir1052 and retags six tests. The body is accurate, but a reader of the merged title would assume selected_population_public and make spec/deny were fixed. | PR #1096 title |
