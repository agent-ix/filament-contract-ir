---
id: SR-17940486709564847048133
title: "Gap analysis — QSL-2581 is_slot_parameter_name trace and inventory"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-contract-ir; PR #1104 at 47f92538; spec/model/functional/FR-019-rust-library-interface.md; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (FR-038-AC-563); tests/it/checked_package_v2_temporal.rs"
review_set: subset
---

## Summary

Ticket: QSL-2581. This is a planless gap analysis scoped to the criteria and inventory rows the PR touches. Plan completion: not assessed.

## Examined scope

- FR-019 public items row "V2 slot parameter names" (examined). It names function `is_slot_parameter_name` and is consistent with the export, and the public_interface inventory tests pass. The wording "a value-parameter name" avoids backticks only because the inventory parser takes every backticked token as a public item. It is accurate (FR-402 calls these `value`/`parameter` names) and consistent with neighbouring rows, which also cite FSpec ids (FR-322) in the last column. "No behavior of the reader changes" is true.
- FR-038-AC-563 (examined). This AC is the reader's root-`over` refusal at `/semantic_graph/nodes/{w}/body/arguments/0`, tagged by temporal tests 5005, 5099 and 5113, which build packages.
- FSpec FR-402-AC-12 (examined, context). It holds both the reserved form and the checker/reader examples. No local criterion mirrors it.

## Verdict

**PASS with findings.** The inventory row is truthful. The new test is bound to a criterion it does not exercise.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new unit test is tagged `#[trace("TC-486", "FR-038-AC-563")]`, but it reads no package and checks no `over` refusal. It tests only name classification, which is FSpec FR-402 "Reserved slot names" / FR-402-AC-12. The tag inflates FR-038-AC-563's binders, and the predicate's real criterion goes untraced. Tag it `FR-402-AC-12`, as other tests tag FSpec ids (for example `TC-1862`/`FR-772-AC-2`), or add a local AC. | tests/it/checked_package_v2_temporal.rs:5572 |
| FND-002 | low | The FR-019 row says the function reports whether a name is "the reserved slot-parameter encoding of FSpec FR-402 'Slot parameters'". The reserved set is defined in FR-402 "Reserved slot names" and is wider than the encoding: it includes non-canonical escapes. Naming that section would keep the row exact. | spec/model/functional/FR-019-rust-library-interface.md:328 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | de4554fd retags the test `#[trace("TC-2082", "FR-402-AC-12")]` (tests/it/checked_package_v2_temporal.rs:5572), the FSpec criterion and test case for the reserved slot form. |
| FND-002 | fixed | de4554fd changes the FR-019 row to "the reserved slot form of FSpec FR-402 \"Reserved slot names\"" (spec/model/functional/FR-019-rust-library-interface.md:328). |
