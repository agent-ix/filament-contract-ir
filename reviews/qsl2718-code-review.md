---
id: SR-2026112105511900000001
title: "code-review and rust-review of QSL-2718 model_root terminal self binding"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1109 at de9833fe (base origin/main 7b76c8be); crates/filament-contract-model/src/checked_package/v2/claim_terminal.rs, tests/it/checked_package_v2_claim_terminal.rs"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in, for filament-contract-ir PR #1109. Ticket: QSL-2718. Oracle (read only): filament-specification main 6ad873d5, FR-476 ("Properties", "Several terminal nodes on one model node", "Reader order", AC-4, AC-6), FR-366 "`self` in `P`" and TC-961 MT-10, MT-15 and MT-18. FR-476 and FR-366 are byte-identical on the #778 branch (06e768a1). Focused run: `cargo test --features conformance-cli --test it checked_package_v2_claim_terminal` (see Verdict for the result).

## Examined scope

- `self_object_key`, `model_root` arm (examined). It takes `self`'s object from the selected model's populations. It counts (population, member) pairs and admits only when exactly one pair exists and that member is an object type of the package. Members are a `SortedSet` per population, so pair count 1 means exactly one object type covered by exactly one population. This is FR-366's rule ("populations cover exactly one object type `T` and exactly one population covers `T`"). One object in two populations gives 2 pairs and refuses, which matches FR-366's ambiguous case. Two objects give at least 2 pairs and refuse, which matches the "several" case. MT-15 (a second object type that no population covers leaves `C` as `T`) holds, because only population members are counted.
- Refusal for none or several (examined): `expected = None`, then `ill_typed`/`operator-ineligible` at `body/members/2/value/members/0/target`. This is FR-476 Reader order step 4 ("whose type is not `Reference<M::T>` for FR-366's `T` ... refuses `ill_typed`/`operator-ineligible` ... The locus is the target"). The FR-366 source-level codes (`ambiguous_declaration`, `missing_declaration`) belong to the checker, not the reader.
- Metering (examined): one charge per population and one per member, the same as the `object_type` arm. `declaration_id` is a map lookup. `terminal_self_population_scan_obeys_work_limit` still passes.
- `object_type` arm (examined): the old body is unchanged, so every existing refusal is unchanged.
- `validate_terminal` model-target check, line 470 (examined, unchanged): `matches!(model_kind, Some(CheckedNodeKind::Model(_)))`. See FND-001.
- New tests (examined): the positive admission, its `model` row key and a wrong-typed `self`; no population and an absent `populations` key; two objects in one population and two populations with one distinct object each, each typed for both objects. They are not tautological: each asserts the code, the cause and the locus, and the `for_other` variants kill the "drop the count == 1 filter" mutant.

## Verdict

Focused run at de9833fe: 19 passed, 0 failed (exit 0). Mergeable as a step toward QSL-2718. The PR does not complete QSL-2718. The `model_root` admission is correct against FR-476 and FR-366, and every existing refusal is unchanged. FND-001 is the ticket's own remaining acceptance item; the lead takes the ruling on it to arch. FND-002 and FND-003 are test gaps to fix in this PR.

Recommendation on FND-001: tighten the reader in this PR. Change line 470 to accept only `Model(ModelRoot)`, delete the `object_type` arm of `self_object_key`, move the fixtures that bind `object_type` to `model_root`, and add MT-10's `object_type` case. The FSL pin bump that consumes this PR must carry the one-line emitter fix (`model_root_key(selection)`) in any case, because the FSL test `a_terminal_row_names_the_selected_model_root_instead_of_its_object_type` un-ignores only then. Doing both in one bump opens no extra broken window. If arch rules instead that the reader stays lenient until FSL has flipped, then this PR stays `refs`, and QSL-2718 stays open with the tightening as its one remaining contract-ir item. The ticket is parked with "no follow-up tickets", so do not file a new ticket for it.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Reader still admits a `model`/`object_type` terminal `model` target (any `Model(_)` kind). FR-476 Reader order step 4, FR-476-AC-4 and TC-961 MT-10 require `invalid_model_binding`/`malformed-declaration`, and QSL-2718's expected completion names this refusal. The PR keeps and documents the `object_type` arm, so the leniency persists. | crates/filament-contract-model/src/checked_package/v2/claim_terminal.rs:470, crates/filament-contract-model/src/checked_package/v2/claim_terminal.rs:179-211 |
| FND-002 | medium | No test covers the `model_root` arm with one object covered by two populations (FR-366 ambiguous `self`). A mutant that counts distinct objects instead of (population, object) pairs survives all four new tests. The `AccountsAgain` case at line 723 exercises only the `object_type` arm. | tests/it/checked_package_v2_claim_terminal.rs:1549-1571 |
| FND-003 | low | The positive admission test `model_root_terminal_with_self_takes_the_one_population_object` is tagged only `FR-060-AC-2`, which is the refusal AC. The admission of a `model_root`-bound MT-01 is FR-060-AC-1 or FR-060-AC-8 (MT-18). | tests/it/checked_package_v2_claim_terminal.rs:1506 |

## Dispositions

Round 1, at b201ff1d (delta since de9833fe: one commit, tests only). Focused run: 21 passed, 0 failed. The FND-002 mutant (the model_root arm counting distinct objects instead of pairs) fails exactly `model_root_terminal_with_self_refuses_one_object_in_two_populations`, 20 passed and 1 failed, so the mutant dies.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | deferred | arch ruling: #1109 admits both ModelRoot and ObjectType; the object_type refusal is follow-up PR scope remaining on QSL-2718, after the FSL emitter emits model_root |
| FND-002 | fixed | New test `model_root_terminal_with_self_refuses_one_object_in_two_populations` (Accounts and AccountsAgain both cover Account) kills the distinct-object mutant |
| FND-003 | fixed | `model_root_terminal_with_self_takes_the_one_population_object` is retagged `#[trace("TC-961", "FR-060-AC-8")]` |
