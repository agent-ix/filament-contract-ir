---
id: SR-17940486709564847048079
title: "spec-review/base of QSL-2449 FR-038-AC-564/565, TC-048 and FR-019 edits"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1054 at 12eab4ef; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md, spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

Spec review of the spec edits in filament-contract-ir PR #1054. Ticket: QSL-2449. Reference: FSpec origin/main 551aa784.

AC numbering: FR-038-AC-561 (#1051) and AC-562/563 (#1053) are taken by open PRs. No other open PR (#1045, #1049, #1050, #1052) adds an FR-038-AC-56x id, so 564 and 565 do not collide. quire validate reports no errors on the four files, the `ac:non-canonical-shape` warning count on FR-038 is unchanged from main (4), and `quire coverage --strict` is unchanged (70 unbacked rows).

## Examined scope

- FR-038-AC-564 (examined): Implemented (QSL-2449). The public reader admits an FSpec FR-465 role whose last argument is the `boundary` binding, the text `observe-only`, after `model_member` when both are present and directly after `replication` otherwise; ... A role without the binding reads `None` and keeps its bytes. This agrees with the FR-465 row.
- FR-038-AC-565 (examined): A role `boundary` whose text is not `observe-only` ..., whose value is not a text, that precedes `replication` or `model_member`, or that repeats refuses `malformed_wire` with no cause at `/semantic_graph/nodes/{n}` of the role node (FSpec FR-465 "Role" check 6). The `attempt`-behind-boundary `ill_typed`/`operator-ineligible` refusals and the `boundary` member of `control` nodes (FSpec FR-465-AC-14 tail, FR-466-AC-19) are not part of this AC. The code, cause and locus match FR-465 check 6 and FR-464-AC-24.
- TC-048 "Role boundary" section (examined): the procedure matches the two tests, and the excluded scope is stated.
- tests.md row for AC-564 and AC-565 (examined): its column layout is the same as the neighbouring AC-555 to AC-560 row.
- FR-019 "V2 reader" row and prose (examined): `CheckedProtocolBoundary` is listed and `protocol_role_boundary` is described. The `public_interface` tests pass.
- FSpec FR-465 check 6 (context_only): A `boundary` argument is the last argument and its text is `observe-only`, else `malformed_wire`. A role that carries it is observe-only: ... the reader refuses an `attempt`, `effect`, `send`, `commit`, an `event` with a `compensation` and a `compensate` template behind it with `ill_typed`/`operator-ineligible` at that control or template node, retaining `action` and `boundary` (this role node).

## Verdict

The edits are well-formed and consistent with the code. The one defect: the ACs consume FSpec FR-465 check 6 and FR-465-AC-14 only in part, without an owner for the rest.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-564 and AC-565 cite FSpec FR-465-AC-14 and TC-2018 as their source, but cover only the layout half of check 6. The carved-out half (the observe-only `operator-ineligible` refusals) and the FR-466 control boundary are named as "not part of this AC" with no IR criterion, ticket or `later` marker owning them. So the IR spec now records FR-465-AC-14 as consumed while the reader admits packages that AC says must refuse. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6701 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AC-564 now reads "FR-465-AC-14, first half", AC-565 reads "check 6, first sentence, and FR-466 check 7, first sentence", and the rest is consumed by AC-566 to AC-569 with the TC-048 section and the tests.md row updated. No AC-id collision with open #1051 (561), #1058 or #1061 |
