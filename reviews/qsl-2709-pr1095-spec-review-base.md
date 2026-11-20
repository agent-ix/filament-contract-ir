---
id: SR-2026112013214800000003
title: "spec-review/base of QSL-2709 PR #1095 IR spec edits"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1095 at cf214d40; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (FR-038-AC-586..589), spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md (integer containment paragraph), spec/checked_package/matrix/TC-2099-integer-containment-at-result-positions.md, spec/checked_package/matrix/tests.md; checked against filament-specification b90d84ec FR-322 'Integer containment', FR-322-AC-495..502, TC-2099"
review_set: subset
---

## Summary

This spec review covers the IR spec edits in PR #1095. Ticket: QSL-2709. The edits were checked against FSpec b90d84ec. `quire validate --scope .` was run on the four edited files: no errors, and one new grammar warning (FR-038-AC-588, line 6368). `quire matrix` shows FR-038-AC-586..589 each `tagged`, bound to the `tc_2099_*` tests. The edits are:

- Four new ACs, FR-038-AC-586..589.
- One FR-347 paragraph.
- A new local TC-2099.
- Two matrix rows.

## Examined scope

- FR-038-AC-586. It consumes FR-322-AC-495, 496, 497 and 499, and the "Results" expression-root sentence. Codes and loci agree. Clean, except the constructor case (FND-004 of the gap analysis concerns the test, not this AC).
- FR-038-AC-587. It consumes FR-322-AC-498 and keeps the exact match at a call argument, with the `body/arguments/1` locus. Clean.
- FR-038-AC-588. It consumes FR-322-AC-500 and AC-502, the `if` check over both branches and the hull case IC-17. Its content is clean; it carries the grammar warning (FND-003).
- FR-347 paragraph. It lists the same positions as FR-322 "Integer containment" and keeps call arguments, payloads, tuples and slots exact. Clean.
- TC-2099 (local). It verifies FR-038 and FR-347 and references FR-322-AC-495. Its steps cover IC-01..IC-20 as built in-repo, because FSpec b90d84ec ships no `integer-containment-vectors.json` (FND-004).

## Verdict

Changes required. FR-038-AC-589 writes the binder-by-name rule into the IR spec, and that rule is weaker than FSpec's "The `let` binder check" (FND-001). The remaining findings are low.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-589 says "A `let` binder is the `value`/`parameter` node named by the binding. The `let` is admitted when a binder of that name has the bound value's type node, ...". That makes any same-named parameter anywhere in the graph sufficient. FSpec FR-322 "The `let` binder check" refuses "a binder whose `semantic_type` is none of them", meaning the let's own binder. The AC should name the let's own binder, as the code-review FND-001 fix does: the same-named `value`/`parameter` node that the let body reads, every such node admitted. It should also add the masking case (IC-15 plus an unrelated same-named parameter refuses) | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6369 |
| FND-002 | low | TC-2099 reuses FSpec's TC id inside contract-ir, whose local TCs are numbered sequentially (TC-491 is the latest, and PR #1094 allocates in the same file). A local TC whose id equals an FSpec TC id is ambiguous in cross-repo `ix://` references and trace output. Use the next local id, or state that the reuse is deliberate | spec/checked_package/matrix/TC-2099-integer-containment-at-result-positions.md:2; spec/checked_package/matrix/tests.md:25,91 |
| FND-003 | low | FR-038-AC-588 raises quire's `ac:non-canonical-shape` (given-when-then) warning at line 6368. The new matrix row at tests.md:25 has no closing `\|`, unlike every other row | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6368; spec/checked_package/matrix/tests.md:25 |
| FND-004 | low | Spec question for FSpec, not an IR defect. FR-322-AC-495..502 and FSpec TC-2099 cite `proposals/checked-package-v2/integer-containment-vectors.json` and `tests/checked_package_v2.rs tc_2099_integer_containment`, and neither exists at b90d84ec. FSpec review FND-005 on #774 already noted this, and no ticket was filed. Contract-ir therefore builds the IC packages node by node, so they are not the published bytes, and IC-08's `Sequence` is built as a `Set` (gap analysis FND-002). An FSpec ticket should publish the vectors, and the IR should switch TC-2099 to read them by reference, like the other corpus TCs | spec/checked_package/matrix/TC-2099-integer-containment-at-result-positions.md:18-23 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | medium | The test-case table row reads `TC-492 ... FR-038-AC-586, ..., FR-038-AC-589 ... ✅ Implemented (QSL-2709)`, while the trace row for the same ACs (tests.md:25), FR-038-AC-589 and TC-492 step 4 all say the `let` binder check (IC-15) is not made. The status column contradicts the rest of the file. It should read `🚧 Partial (QSL-2709): IC-15 let binder check not made` | spec/checked_package/matrix/tests.md:91 |

## Dispositions

Round 1, at head 515f738a (fix commit 515f738a4 over cf214d40). The private review worktree was used with a private target dir, via cargo-slot. `it` (`--features conformance-cli`), name-filtered (operation_arguments, lowering, union, catalogue_differential): 57 pass and 0 fail, with no compiler warnings. `quire validate`: no errors, and the FR-038-AC-588 grammar warning is gone. TC-492 does not collide with main or with open PR #1094. The disposition of the `let` binder check follows the arch ruling, posted as the fix-round comment on QSL-2709: the wire gives the `let` application no node-id link to its binder, a binder is never found by name, and FR-322 is to pin such a link. The check was removed rather than weakened. FR-038-AC-589, the FR-347 paragraph, TC-492 step 4 and the trace row of `tests.md` (Partial) now state that IC-15 is not enforced. No AC, doc comment or trace row claims a binder check. That is an honest deferral. No FSpec ticket id is cited for the id link yet, so the lead should file one and cite it.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |
| FND-004 | deferred | FSpec b90d84ec still publishes no integer-containment-vectors.json; TC-492 now states this and that the reader switches to the published bytes when FSpec ships them. The FSpec ticket for the vectors is still to be filed and cited |
