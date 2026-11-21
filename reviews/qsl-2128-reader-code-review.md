---
id: SR-967175026302197768900474181941935084580
title: "code-review and rust-review of QSL-2128 subject_declaration_order reader half"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR #1122 at 5e39d58b88a836483732a351b1ecee0bb97ab03c (base a03c86b3b); crates/filament-contract-model/src/checked_package/v2/step_relation.rs, tests/it/checked_package_v2_frame_entries.rs"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in, for filament-contract-ir PR #1122. Ticket: QSL-2128 (HP-1 reader half). Contract: FSL FR-940 "Declaration-order member contract" at reviewed head f0a8e7f69406305c31152c1a8ac7c372e4aedaa4 (read with `git show`). The same paragraph and FR-940-AC-27 are byte-identical at b891628febd332ebbeffed13d8c24e86897e8a5b, which was verified locally. Model claude-opus-5-5, run 05d86976-7b49-4d52-aca2-f22ca0ff74ad.

Commands run on the frozen head, using only this worktree's target: `cargo test --locked --test it --features conformance-cli -- step_relation_declaration_order step_relation_first_use_subject step_relation_reader_refuses_layout step_relation_reader_retains_two_executions`. RC=0, 6 passed, 1777 filtered out. Log: /tmp/ir-qsl2128-reader-reviews/focused-test.log. `rustfmt --check` on step_relation.rs: clean. The coder's 11-test receipt (/tmp/ir-qsl2128-reader-test.log, RC=0, 10:43:40 -0800) predates the final source mtimes (10:44:19 / 10:44:25), so this run re-qualifies the 4 new tests and 2 layout and roundtrip tests on the final bytes. The 219 s budget test and the other 3 step_relation tests were not re-run. They take the legacy path, which adds no new charge. No clippy, gates or workspace build were run, per the brief.

## Examined scope

- `view` length and layout (examined). 4 or 5 members are admitted. With fewer than 4, the refusal points to `members/LEN`. With 6 or more, it points to `members/5`, which is tested. The four prefix names are checked first, as before, and the fifth name points to `P`. The non-Group check is the defect in FND-001.
- `validate_step_relations` declaration block, lines 727-787 (examined). Entries are checked in sequence and the first offender wins. A malformed entry points to `P/value/members/K`. A duplicate or unknown alias points to `.../name`, using a binary search over the already-sorted, pre-charged `aliases` table and a `covered` flag. A wrong join or a non-reference value points to `.../value/target` by per-alias equality with `subjects[slot]`. Omission is checked only after the loop, at `members/LEN`. Every refusal is `invalid_semantic_graph` with cause `None`. These match the agreed pointers.
- Budget (examined). `covered` comes from `meter.filled` and is charged before it is allocated. `charge_at` is called per entry before any work on it. The lookup is O(log S). There is no N² scan. The loop ends by iteration S+1, because a repeat refuses. No new Vec is allocated without a charge, and legacy 4-binding bodies run no new code. See FND-002 for the test gap.
- Generic precedence and a bare `Reference` entry (examined statically, not run). `rekey_step_relation` recomputes dependencies. A bare Reference to the valid, already-selected model root dedups into the unchanged dependency set, so reference resolution and FR-322 dependency equality (mod.rs:4214-4245) both pass. `named_binding` then fails, and the refusal points to `P/value/members/K`, as promised. The coder's earlier refusal at `<ref>/target` came from an unknown model id, which is a legitimate earlier generic stage. It is not evidence about this contract.
- Public getter `step_relation_subject_declaration_order` (examined). It returns `Option<impl ExactSizeIterator<Item=(&str,&CheckedNodeId)> + '_>` and borrows from the node body, with no owned clones. It returns `None` for other forms, the legacy 4-binding body, a body that does not parse, and any shape-malformed entry, because construction requires every entry to be `declared` first. Because of that guard, `next()` cannot stop early, and the slice `size_hint` is exact. There is no indexing and no panic path. Allocation: each call re-runs `view` with an unmetered meter and builds and drops three transient tables of size O(S+E+I). This is the same pattern as `step_relation_body`, and the doc comment promises no allocation bound. The getter does not re-check joins on a raw node, which is also the existing pattern. Admission enforces the joins.
- `step_relation_body` output and first-use subjects (examined): unchanged with and without the member, as tested.
- Repo rules (examined): no hash, digest or pin was added, there are no new dependencies, and model crate layering is unchanged.

## Verdict

Not clean: one medium finding and one low. The per-entry contract, the generic-stage precedence, the borrowed API and its count are correct. The fifth member's Group-value check runs ahead of the earlier members' value-shape checks.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `view` checks the fifth binding's Group value at lines 308-321, before the value shapes of `subjects`, `executions`, `invariants` and `body` (line 325 onward). Take a 5-member body whose `subjects` value is not an aggregate and whose `subject_declaration_order` value is not a Group. It refuses at `/body/members/4/value`, not at `/body/members/0/value`. FR-351 says "Shape errors refuse `invalid_semantic_graph` at the first binding breaking layout", in body order, and the brief requires the earlier body priority to be kept. Fix: move the `members(value)` Group check after the `body` reference check (line 424). Optionally keep the name check where it is, alongside the prefix names. Add a two-defect test. | crates/filament-contract-model/src/checked_package/v2/step_relation.rs:308-325 |
| FND-002 | low | No budget test reaches the new strict-read charges: `meter.filled` at line 735 and per-entry `charge_at` at line 738. The exact-and-one-short test `step_relation_tables_are_charged_to_the_read_budget_at_the_relation_node` (TC-472, FR-038-AC-504) uses the legacy 4-binding fixture. Deleting either charge therefore passes every test. Fix: run the same exact and one-short assertion over `declaration_order_package(Some(&["W", "V"]))`. | crates/filament-contract-model/src/checked_package/v2/step_relation.rs:735-738; tests/it/checked_package_v2_frame_entries.rs:5760 |
