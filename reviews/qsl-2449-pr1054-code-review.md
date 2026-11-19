---
id: SR-17940486709564847048077
title: "code-review and rust-review of QSL-2449 role-body boundary admission"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR #1054 at 12eab4ef (base origin/main a15bf349); crates/filament-contract-model/src/checked_package/v2/{protocol_body.rs,mod.rs}, crates/filament-contract-model/src/lib.rs, tests/it/checked_package_v2_protocol_body.rs"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in, for filament-contract-ir PR #1054. Ticket: QSL-2449. Reference: filament-specification origin/main 551aa784 (the pin the coder used; `git fetch` showed no newer main), FR-465 "Role" (argument table row `boundary`, reader check 6), FR-466 check 7, FR-322-AC-462, FR-465-AC-14, FR-466-AC-19, TC-2018.

Commands run in a scratch worktree at 12eab4ef with its own target dir: `cargo test --features conformance-cli --test it -- boundary public_interface` (47 passed), `fsl-string-edge` with the Makefile's arguments (clean, on head and main), `cargo fmt --all --check` (clean), quire validate on the four changed spec files (no errors; same four `ac:non-canonical-shape` warnings as main), `quire coverage --strict` (70 unbacked rows, the same as main). 13 hand-applied mutants (the Mutants section below). A probe test (not committed) added a `boundary` to every control node of the fixture and checked the role `node_id`.

## Examined scope

- `role_tail_fits` (examined): it admits `[]`, `[model_member]`, `[boundary]` and `[model_member, boundary]`, and nothing else. This matches the FR-465 row "the text `observe-only`, after `model_member` when both are present". The probe confirmed that `[boundary, model_member]` and a repeated `[boundary, boundary]` without a member both refuse `malformed_wire` at the role node.
- `validate_layout` change to `arguments.len() >= expected.len()` (examined): it is correct. The change to `>` (M10) behaves the same, because for a length equal to the expected length the first disjunct already passes.
- `role_boundary_defect` through `enum_value_defect` (examined): an unknown text returns `EnumValue`, which maps to `malformed_wire` with no cause at the node pointer. That is what FR-465 check 6 says ("else `malformed_wire`") and the node pointer that FR-464-AC-24 sets for layout and enum violations. "Typed, located" in the ticket is satisfied by the `malformed_wire` code plus the node-pointer locus.
- `check_role_member` now reads `binding_value(arguments, "model_member")` instead of position 2 (examined): it is needed, because a member-less role now has `boundary` at position 2. Mutant M7 (the old position read) is killed.
- `CheckedProtocolBoundary` (`#[non_exhaustive]`, `Copy`, `Hash`) and `CheckedPackageV2::protocol_role_boundary` (examined): borrowed, with no new carrier. They are listed in the FR-019 "V2 reader" row and the prose, and the `public_interface` tests pass.
- Role `node_id` stability (examined, probe): adding `boundary` leaves the role node's `node_id` unchanged after `settle`, as FR-322-AC-462 requires.
- String-edge (examined): `role_boundary`, `role_tail_fits`, `role_boundary_defect` and `validate_layout` carry `#[string_edge]`, and the checker is clean.
- Control-node `boundary` (examined, probe): a `boundary` `observe-only` appended to the fixture's `sequence` control refuses `malformed_wire` at `/semantic_graph/nodes/22`. This is expected, since the PR leaves control nodes out of scope, but it matters for the gap-analysis artifact.

## Mutants

13 run, re-run with failing-test names. Filter: `role_boundary observe_only_role_boundary role_port port_role protocol_role`.

| Mutant | Result |
| --- | --- |
| M1 pair arm `&&` to `\|\|` | killed (AC-565 test) |
| M2 single arm drops the `boundary` alternative | killed (AC-564 test) |
| M3 `_ => true` | killed (AC-565) |
| M4 text defect never fires | killed (AC-565) |
| M5 `.skip(2)` to `.skip(3)` | killed (AC-565) |
| M7 member read at position 2 | killed (AC-564) |
| M8 drop the Role-kind check in `protocol_role_boundary` | survives: behaves the same for admitted packages whose only `boundary` binding is on a role |
| M10 `>=` to `>` | survives: equivalent (see above) |
| M11 single arm admits any `boundary` value shape | **survives: a real test gap, FND-001** |
| M12 pair arm admits any `boundary` value shape | killed (AC-565) |
| M13 pair order swap also admitted | killed (AC-565) |
| M14 `role_boundary` returns `ObserveOnly` for any text | survives: equivalent, since an admitted boundary is always `observe-only` |
| M15 `protocol_role_boundary` returns `None` | killed (AC-564) |

The coder reported two equivalents. M10 and M14 are equivalent. M8 only behaves the same while the reader never admits a `boundary` binding on a non-role node, and the FR-322-AC-462 probe below shows that does not hold for every node form.

## Verdict

The role-body admission, position rules, refusal code and locus, writer round-trip, public API and string-edge compliance are correct and match FR-465 "Role". There are two defects in this method: the non-text branch of FR-038-AC-565 is untested for a role without `model_member`, and the reader now admits an observe-only role without the refusal check 6 attaches to it (FND-002). The acceptance and scope consequences are recorded in the gap-analysis and spec-review artifacts.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The `non-text value` case of FR-038-AC-565 is exercised only on a role that has `model_member`. The single-member arm's `Shape::Text` check can be dropped (M11) and every test still passes. | crates/filament-contract-model/src/checked_package/v2/protocol_body.rs:1039; tests/it/checked_package_v2_protocol_body.rs:14303 |
| FND-002 | high | The reader now admits a role carrying `boundary` `observe-only` but never applies FR-465 check 6's consequence: an `attempt`, `effect`, `send`, `commit`, an `event` with a `compensation` or a `compensate` template behind that role is still admitted, although FSpec requires `ill_typed`/`operator-ineligible`. Before this PR such a package refused (fail-closed). Now it admits a package the spec forbids (fail-open). No code outside the role layout reads `boundary`. | crates/filament-contract-model/src/checked_package/v2/protocol_body.rs:118-129,1270-1276 |

## New findings (disposition pass 1)

Disposition pass 1 ran at head 51166c4b, which is a merge of origin/main e0327d7c, so merge-tree is clean. PR delta: `git diff origin/main...51166c4b`. 22 hand mutants were run on the new branches (filter: `checked_package_v2_protocol_body checked_package_v2_flat_wire public_interface`, 251 tests). 16 were killed. N11 (channel allowed to carry `boundary`) and N20 (role-kind check dropped) are equivalent. N9, N13, N14 and N15 survive and are recorded below.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | FR-038-AC-566 claims that a `recommend` binder "is never a decision fact" (FSpec FR-466 check 7, last sentence). The rule holds only because `recommend` falls through `fact_roles` to `None`, as `send` does, and no test covers it. Mutant N15 adds `"recommend"` to the decision-fact roles and all 251 tests still pass. | crates/filament-contract-model/src/checked_package/v2/protocol_body.rs:4686-4699 |
| FND-004 | medium | The send performer join is untested on the side that matters. The fixture channel has `from` and `to` set to the same role, so mutant N9 (performer read from `to` instead of `from`) survives. No test admits a `send` on a channel whose `to`, but not `from`, is the observe-only role (the inverse case of FSpec FR-822-AC-2). | crates/filament-contract-model/src/checked_package/v2/protocol_body.rs:6766-6772; tests/it/checked_package_v2_protocol_body.rs:7492-7495 |
| FND-005 | low | The `recommend` canonical-order checks for `contracts` and `related` are untested. N13 (contracts order check removed) and N14 (related order check removed) both survive, although AC-566 says these check as an `attempt`'s do. | crates/filament-contract-model/src/checked_package/v2/protocol_body.rs:1140-1142,1161 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `non-text value without member` case added to the AC-565 test; mutant N12 (boundary shape unchecked) is now killed by the role and control tests |
| FND-002 | fixed | `check_observe_only` and `observe_regions` refuse `ill_typed`/`operator-ineligible` with `ObserveOnlyBoundary {action, boundary}`; mutants N1 to N8 are all killed |
