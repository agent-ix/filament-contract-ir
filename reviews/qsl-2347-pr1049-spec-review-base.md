---
id: SR-17940486709564847048073
title: "spec-review of QSL-2347 source dispatch keys, operation bodies and identity-row operations"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1049 at 3d9417a7 (base origin/main 71914a07); spec/checked_package/functional/{FR-365,FR-366,FR-367,FR-373}-*.md, spec/checked_package/matrix/{TC-480-source-operation-body-rows.md,tests.md}, spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

Spec review of the IR spec changes in PR #1049. Ticket: QSL-2347. Upstream text, read only: filament-specification origin/main 118febea. Sections read: FR-788, FR-789, FR-790, FR-322 "Source-declared object types" items 1-9, the member-type table's `Clock` row, FR-322-AC-124, AC-157, AC-290 and AC-397, and FR-610 "Identity rows". Also read: FSL FR-569 "Source-object identity rows" on filament-spec-language origin/main.

`make spec` at head and at clean main both report 79 unbacked rows and 0 contradicted statuses; FR-365/366/367 are 6/6 and FR-373 is 5/5 backed. FR-373 and TC-480 are free ids: no FR-373+ or TC-480+ exists on main. Open PR #1046 holds FR-371/TC-477, FR-372/TC-479 is on main, and IR-1569 is in Backlog with no branch or PR.

## Examined scope

- FR-366 Description, the strict-reader paragraph, the source-key paragraph and FR-366-AC-3 (examined). They match FSpec FR-788 and FR-322 item 7, `{owner_effective_type: null, original: {package: I, node: "ix://I/<type>"}}` with exact recompute, and the five FR-788-AC-2 refusals. They also match the lead's draft amendment on QSL-2347 word for word in substance.
- FR-366 family paragraph (examined): the `ix://I/` prefix rule matches the code's `source_type_identity`.
- FR-365 coverage paragraph (examined): the reader does not join `operations` to `operation_effects`, which matches FSL FR-569 ("the reader checks only the member set and types of `operations` ... the correspondence above is the writer's obligation").
- FR-367 Inputs and FR-367-AC-5 (examined): the `operations` member `{identity, query}` matches FSL FR-569.
- FR-367 Behavior, producer-facts sentence (examined): FND-001.
- FR-373 Description, Inputs, Outputs (examined): match FSpec FR-790 and FR-322 item 9.
- FR-373 Behavior items 1-2 (examined): match the FR-790 Behavior bullets. For the untested arms, see FND-003.
- FR-373 Behavior item 3 (examined): FND-002.
- FR-373 Behavior item 4 (examined).
- FR-373 coverage paragraph (examined): FND-004 and FND-005.
- FR-373-AC-1 to AC-5 (examined): AC-1 and AC-2 match FR-790-AC-1 and AC-2, with `deposit`/`quote` for `size`/`bump`. AC-5 goes beyond FSpec and is consistent with FR-322-AC-397's "row of the type that declares that effective operation".
- TC-480 and the tests.md row (examined): FND-006.
- FR-019 public-items row (examined): lists `SelectedSourceObjectOperation`.
- FSpec FR-789 and FR-322 item 8 (context_only): read for the seam-2 judgement in the gap-analysis SR.
- FSpec FR-610 "Identity rows" (context_only): lists no `operations` member, although FR-322 item 3 says the identity rows supply the operation set. That is an upstream FSpec inconsistency for draco:QSL:arch. It is not an IR defect, because FR-367 cites FSL FR-569, which does define the member.

## Verdict

The FR-366 amendment and FR-367/FR-365 updates are faithful to FSpec FR-788 and FSL FR-569. FR-373 is a faithful IR consumer of FR-790 for the rows it checks. Its one deliberate departure (FND-004) is a reader-side relaxation of FR-322-AC-397 coverage, and it comes from the same missing input as seam 2. draco:QSL:arch's option-(c) ruling (post-read join, W51) covers it, so it is a cite-the-ruling fix and not a blocker. No spec file in the diff claims seam 2: no FR-789 or clock-place text is added. Every finding here is low except FND-003 (medium).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-367 Behavior still says "`identity_fields` and `populations` are producer facts read only from a joined row". The PR adds `operations` to Inputs and AC-5 as a producer fact the reader does not join, but this sentence does not name it | spec/checked_package/functional/FR-367-join-source-object-rows-and-field-reads.md:85-87 |
| FND-002 | low | FR-373 item 3's refusal "`invalid_package`/`invalid-value` at `body` when the bodies differ" cannot occur. A candidate joins the row keyed by its own body's `declaration` (type, operation) under its `SourceOwner`. Item 2 has already required that row's `body.node` to be the one clause function with that declaration, and declarations are unique per package (FR-367 Behavior). So a differing row refuses at `body/node` or `body` under item 2 first. The clause states an outcome no package can produce | spec/checked_package/functional/FR-373-carry-operation-body-rows-of-source-declared-types.md:59-64 |
| FND-003 | medium | FR-373 Behavior states two refusals that no AC covers. Item 1: "another owner's identity refuses `invalid_package`/`invalid-value` at `document`". Item 2: `body/node` naming "another owner's function". FR-373-AC-4 covers only the identity of no `SourceOwner`, and AC-3 only the precondition and `Sub.deposit` cases. FSpec FR-790-AC-2 has the same hole, but the IR AC set is where the reader's branches are pinned. The `document` arm is unkilled by tests (code-review FND-002) | spec/checked_package/functional/FR-373-carry-operation-body-rows-of-source-declared-types.md:46-58 |
| FND-004 | low | FR-373 says the reader "SHALL NOT require a row for an operation that no dispatch candidate joins". FSpec FR-790 and FR-322-AC-397 say the carrier holds exactly one row per operation member of every source-declared `object_type`, and AC-397 lists "missing coverage" among its refusals. The IR relaxes reader-side coverage for source rows on the stated ground that the package carries no operation list. The identity rows now carry `operations` (FR-367), but they reach the reader only in the FR-367 stage read, after the package read. This is the same missing input as seam 2 (FR-789). draco:QSL:arch has since ruled option (c): FR-790 coverage moves to the post-read row join and W51 will spec it. The FR-373 text still states the departure only as prose, with no cite of that ruling or of W51 | spec/checked_package/functional/FR-373-carry-operation-body-rows-of-source-declared-types.md:67-72 |
| FND-005 | low | `quire validate` (make spec) raises three new EARS warnings on FR-373, which clean main does not have. Line 62 has two `SHALL`s in one statement (`[ears:non-singular]`). Line 68 is `[ears:unclassifiable]` and `[ears:missing-subject]` (the SHALL NOT paragraph) | spec/checked_package/functional/FR-373-carry-operation-body-rows-of-source-declared-types.md:62,68 |
| FND-006 | low | TC-480 `## Status` reads "Planned (QSL-2347)" and the tests.md row reads "🚧 Planned (QSL-2347)". Yet all of FR-373-AC-1 to AC-5 are bound by `#[trace("TC-480", ...)]` tests that pass at head (quire coverage reports FR-373 5/5). The status is stale | spec/checked_package/matrix/TC-480-source-operation-body-rows.md:30-32; spec/checked_package/matrix/tests.md:12 |

## Dispositions

Round 1, checked at ab9dc00d.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | after: "`identity_fields`, `populations` and `operations` (each entry `{identity, query, clock_parameters}`) are producer facts read only from a joined row; the package has no `extends`, identity, population or operation information against which to check them." |
| FND-002 | fixed | after: item 3 now says "require that row to exist (`invalid_package`/`invalid-value` at `/operation_bodies` otherwise); the row's `body.node` is then the candidate's `operation_body`, since declarations are unique per package and item 2 holds" |
| FND-003 | fixed | after: FR-373-AC-6 covers `document` `src/other-unit`, and `body.node` naming another operation's or another owner's function, refusing at `document` and `body/node`, plus the non-function node, row order and the dispatch-side refusal |
| FND-004 | fixed | after: "This is the arch ruling for QSL-2347 (option c): coverage of FSpec FR-790 and FR-322-AC-397 for source types moves to the identity-row join ... and FSpec W51 and FR-375 specify it." The text is scoped to `CheckedPackageV2::read`, and FR-365 is likewise scoped |
| FND-005 | fixed | after: `make spec` at ab9dc00d raises no EARS warning for FR-373 |
| FND-006 | fixed | after: TC-480 Status "Implemented (QSL-2347): FR-373-AC-1 through FR-373-AC-6 are tested." and the tests.md row is "✅ Implemented" |

Round 2, checked at 51edebff: no finding has a still-open latest outcome, so this round adds no rows. The spec files are unchanged since ab9dc00d apart from merged main.
