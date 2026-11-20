---
id: SR-17940486709564847048084
title: "gap-analysis of QSL-2464 model-root over for hyper body wrappers"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1074 at cf3c7807 (base origin/main c7b62038); FR-038-AC-562, FR-038-AC-563, TC-486 against crates/filament-contract-model/src/checked_package/v2/temporal.rs and tests/it/checked_package_v2_temporal.rs"
review_set: subset
---

## Summary

Gap analysis for PR #1074. Ticket: QSL-2464. Planless: plan completion was not assessed. Upstream: FSpec origin/main 3b5fc9e0, FR-402-AC-11, TC-2034, FR-402 "Reader order" step 3, and FR-370's `over` step.

Two tests carry trace tags: `tc_486_a_body_wrapper_over_its_first_subjects_model_root_reads_and_writes` (`#[trace("TC-486", "FR-038-AC-562")]`) and `tc_486_a_model_root_over_refuses_everywhere_the_spec_names` (`#[trace("TC-486", "FR-038-AC-563")]`). Both bindings are correct. Each AC has one tagged test.

Hand mutants: cargo-mutants was not used. The coder's own six mutants are not recorded in the PR or the ticket, so they were not re-run. Eight mutants of my own were hand-applied to temporal.rs, and each was run against `--test it -- checked_package_v2_temporal` (89 tests). The source was restored after each one.

- Killed:
  - M2: the body-wrapper condition removed from the temporal step.
  - M3: the slot check removed.
  - M4: the `rcv_s` prefix dropped.
  - M6: the hyperproperty first-subject check disabled.
  - M7: the traversal stops at the formula's direct dependencies.
- Survived, real:
  - M1: any `subjects` binding's root accepted, not only the first (FND-001).
  - M5: the prefixes loosened to `slot_`/`op_`/`rcv_`/`arg_` (FND-003).
- Survived, equivalent on the fixtures: M8, which removes the `value`/`parameter` kind filter from the traversal. No other reachable node carries a slot-named `name` binding.

## Examined scope

- FR-038-AC-562 (examined). Covered by the positive test: admission, two hyper rows keyed by the hyperproperty node, no wrapper name row, typed re-encode and an accounted re-read. The fixture is the synthetic hyper-clock fixture with `over` edited to the root. It is not FR-402-AC-11's `Whole` (`always holds(true)`) (FND-004).
- FR-038-AC-563 (examined). Covered: a root no subject names (hyper step), a single-trace clause (temporal step), four slot-named parameters plus two non-slot names, and an undeclared root keeping `missing-name`. Code, cause, pointer and locus are asserted.
- FR-402 "Reader order" step 3, check order (examined): untested (FND-002).

## Verdict

Both ACs are covered on their main paths, and the tests assert the full refusal tuple. The gaps are listed below.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | "First" in "the first `subjects` binding" is untested. Every fixture has one subject (`m`), so a reader that accepts any subject's root passes. Add a two-subject hyperproperty whose wrapper `over` is the second subject's root, expected `invalid_model_binding`/`malformed-declaration` at `arguments/0` | tests/it/checked_package_v2_temporal.rs:3370-3395,4920-5062; crates/filament-contract-model/src/checked_package/v2/temporal.rs:2697-2713 |
| FND-002 | low | The step-3 check order is unpinned. No case has both a non-empty wrapper `fairness` argument and a non-first-subject root `over`, so swapping the two checks passes every test (code-review FND-001) | crates/filament-contract-model/src/checked_package/v2/temporal.rs:2695-2752 |
| FND-003 | low | The slot-name negatives (`slotted`, `operand`) never contain `_s`, so the tests cannot tell a prefix check from FR-402's decoding. Add names that start with a slot prefix but do not decode: `slot_sv` (wrong field count), `op_sa_sb`, and `op_` with an invalid escape (code-review FND-002) | tests/it/checked_package_v2_temporal.rs:5017-5040 |
| FND-004 | low | AC-562 names FR-402-AC-11's object-free `Whole`, but the test reads a synthetic hyper-clock fixture with `over` edited, whose formula is not `always holds(true)`. Either build `Whole` (or use an FSL-emitted package) or reword the AC and TC-486 step 1 to the fixture actually used | tests/it/checked_package_v2_temporal.rs:4935-4943,4962-5003 |


## Disposition pass 1 notes

Round 1, head 8bca5ece. New tests:

- `tc_486_a_model_root_over_is_the_first_subjects_never_a_later_ones`: two subjects; the second subject's root is refused.
- `tc_486_the_wrapper_fairness_refusal_precedes_the_model_root_refusal`.
- 14 more names in the AC-563 slot table, eight of them prefix-alike names that are not valid encodings. All carry the correct `#[trace("TC-486", "FR-038-AC-563")]` tag.

My mutants at this head, against `--test it -- checked_package_v2_temporal`:

- Killed:
  - N1: any subject's root accepted.
  - N3: `slot` takes one field.
  - N4: lowercase hex accepted.
- Survived, equivalent:
  - N5: the memo shortcut removed. This changes performance only.
  - N6: the parameter-kind filter removed. As with M8, no other reachable node carries a slot-named `name`.

The coder's 13 mutants are listed in the PR body as killed. I did not re-run them.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |
| FND-004 | rejected | The fixture has the shape FR-402-AC-11 and AC-562 specify: a two-trace forall/forall behaviours clause over one subject, a wrapper `over` the first subject's root among its dependencies, and a formula that reads no slot parameter. The `always holds(true)` formula text is not part of the admitted rule, so the AC is exercised as written |
