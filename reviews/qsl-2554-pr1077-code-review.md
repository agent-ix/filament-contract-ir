---
id: SR-17940486709564847048092
title: "code-review and rust-review of QSL-2554 probabilistic claim node obligation identity"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1077 at e0e47c8a (merge base 383857b6, reviewed also as a local merge with origin/main 2ec296a1); crates/filament-contract-model/src/checked_package/v2/{probabilistic_join/claims.rs,probabilistic_claim_read.rs,probabilistic_join.rs,temporal.rs,flat_wire.rs,stage_storage.rs,mod.rs}, crates/filament-contract-model/src/lib.rs, tests/it/checked_package_v2_probabilistic_claim.rs, tests/conformance_fspec/{probabilistic_claim_timed.rs,probabilistic_claims.rs}, tests/it/support/checked_package.rs"
review_set: subset
---

## Summary

Code review, with the rust-review lane folded in, for filament-contract-ir PR #1077. Ticket: QSL-2554. Spec authority: FSpec origin/main (cdebe572) FR-640 "Probabilistic claim node", Reader rules 6 and 7, AC-8 to AC-11; FR-370 "Placement and profile fit" and FR-370-AC-22; FR-322-AC-474; contract-ir FR-377, TC-489 and FR-019.

Gates. The PR head's base predates #1074 and #1076, so the head was merged locally with origin/main 2ec296a1 (clean merge, never pushed) and gated there, in the scratch worktree CIR-1077-review with its own target:

- `cargo fmt --all -- --check`: clean. `make string-edge`: clean.
- `cargo clippy --workspace --all-targets --features conformance-cli -- -D warnings`: clean.
- Full `cargo test --workspace --features conformance-cli --no-fail-fast`: 32 failures, exactly the known baseline. These are 18 `monitor_plan::reached_model_read` lib tests, 13 `checked_package_v2_protocol_body` await tests and `refinement_catalog_joins_timeout_on_await_node`. `public_interface` and `whole_read_accounting` are green. `read_storage_alloc`: 32 passed. At the unmerged head, `public_interface` fails on the `CheckedTerminalDuplicate` drift that #1076 fixed on main. That failure is not from this PR.
- `--test conformance_fspec` (full binary, merged tree): 8 failures. They are the same 8 on clean origin/main (see gap-analysis).

Mutants: 40 hand mutants were run against `checked_package_v2_probabilistic_claim` and `probabilistic_claim_timed`. Of the coder's 17, 15 were killed. Two survived:

- `c12`: charge `form` after its nested bindings.
- `c17`: subject kind widened to any `model` node. This mutant is equivalent, because the `ModelRoot` owner check refuses the same inputs.

Of my 22, 12 were killed: the placement Slot map for the form body, the event body and the formula value; the trigger slot; turning the flat-wire claim branch off; dropping the sort; the fairness `declared` check; reward check 6; `NonNegative`; the binding-name check; the `elapsed` time item; and turning the claim validation off. Ten survived: `r21` (extra binding after `origin`), `r23b` (predicate slot admits any target), `r24` (quantity unit unchecked), `r25` (`CAPACITY` 18), `r30` (`weighted by R` reads no reward), `r32` (Open01 lower bound dropped), `r33` (θ not set for probability forms), `r35` (foreign-model workload), `r37` (steps bound type) and `r38` (rational type node unchecked). The test-strength consequences are recorded in the gap-analysis artifact.

Reviewer probes (scratch only, never pushed):

- `over` referencing `Steady` refuses `invalid_model_binding`/`malformed-declaration` at `over`'s target: correct.
- `from` referencing `Steady` refuses `ill_typed`/`operator-ineligible`: correct.
- `Avail` with `indifference` 1/2 (θ − ι = 0) refuses at the literal: correct.
- `Avail` with its `delays` binding removed: refuses at the wrong rule (FND-002).
- A whole-read allocation proof over the four-claim package: shows unpaid heap per claim (FND-001).

## Examined scope

- `probabilistic_join/claims.rs` `Check::run`, `subject`, `scheduler`, `fairness`, `rewards`, `exact_set` (examined).
  - Rule 6 matches FR-640. The body must be exactly the nine names in order, else the refusal is at `/body`.
  - `subject` must be a `model_root` reference.
  - `scheduler` must be a workload of the subject's model or the aggregate of exactly one `fairness` binding.
  - Each fairness member must have kind `weak`/`strong` and granularity `whole`/`each`, and must name an operation the subject's model declares. Members must strictly ascend by (declaring, operation, kind, granularity).
  - `reward`, `distributions` and `delays` must be exactly the subject model's nodes, with a single reward name, in ascending node-id order.
  - One visit is charged per claim node and per set or fairness member, ahead of inspecting it.
- `claims.rs` `statement` and `Parse` (examined).
  - Rule 7 runs as one shape pass over bindings 6 to 9 in body order, a binding before the bindings nested in it. The time, exact-value, range, target and reward checks follow in that order, each over the collected items in body order.
  - `θ − ι <= 0` is computed as `a·d <= c·b` and `θ + ι >= 1` as `a·d + c·b >= b·d`, which is exact for canonical positive denominators. `ι` must be above 0. θ is the probability bound, or the quantile's `level`. `expected_*` forms take no θ.
  - A rational exact value must be typed at the `scalar_type`/`rational` node and an integer at the `scalar_type`/`integer` node. An integer typed at an `integer_range` node refuses. A quantity must be typed at the reward unit or the timed root's source unit.
  - A subject is timed when it has a dense or tick root. Only a `delay` weighting needs dense. The reader's timed-root lookup reads source binding 1 as the unit for both sources, which matches `timed_model.rs`'s `[kind, unit, …]` layout.
- `probabilistic_claim_read.rs` (examined). `retain_claims` reads each field back from the admitted body in FR-640's binding order and resolves ids through `resolve_id_charged`. See FND-001 and FND-003.
- `temporal.rs` placement (examined). The formula slot is reached only through the claim root's member 5, then a `binding` named `event` in the form body, then a `binding` named `formula` in the event body. Fairness references are misplaced everywhere in a claim. Placement is FR-038 step 1, which runs before `probabilistic_join`, so FR-370-AC-22's order holds. See FND-002.
- `flat_wire.rs` `Place::Claim` (examined). A claim body admits Leafs, aggregates and bindings at any depth. A nested application still refuses. Other bodies keep their strata: the new arms are reached only from the claim-node dispatch, and mutant `r26` (dispatch off) is killed. No other reader path regresses: the full suite and `read_storage_alloc` are green.
- `stage_storage::Path::CAPACITY` 16 to 20 (examined). Raising it is necessary. The deepest claim locus the reader names is 19 steps (FND-004), which exceeds 16. No other path changes.
- Public API (examined).
  - Thirteen new public types and `CheckedPackageV2::probabilistic_claims`, all listed in FR-019's new row.
  - Un-backticking the method name follows the existing FR-019 convention: the `random_parameters, rewards, workloads and delay_distributions` row does the same. The inventory parser treats backticked words as items, and methods are not crate-root items, so nothing is hidden. All 13 types are backticked and the inventory test passes on the merged tree.
  - `CheckedPackageV2`'s new field is private, so external construction is unaffected. The seven patched constructors are crate-internal unit tests. Not a breaking change.
- `tests/conformance_fspec/probabilistic_claims.rs` fixture change (examined). It is legitimate, not weakening. The PC-row vectors test sidecar coverage, not the claim body. The former empty bodies are now refused by rule 6 as FR-640 requires. The minimal nine-binding body (a probability-of-formula claim under an empty fairness set) keeps the row-coverage assertions meaningful.

## Verdict

The rule 6 and rule 7 logic, the exact θ ± ι arithmetic, the placement walk, the flat-wire stratum and the charge order are correct against FR-640 on every path I traced. The gates are clean, and the failing set equals clean main's.

Two medium defects:

- FND-001: the retained claim read allocates uncharged heap per claim.
- FND-002: placement keys on member index rather than on the `form` binding name, so a formula claim with a rule-6 body defect refuses at the wrong rule.

Two low findings follow. None is a crash or a wrong admission, but FND-001 and FND-002 should be fixed in this PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `retain_claims` clones the claim's structural key uncharged (`(*self.keys[position]).clone()`). The terminal-row precedent charges `read_charge::charge_node_id_clone` first. Also uncharged: each rational's two `Box<str>`, the fairness `operation` `Box<str>` and the `BigInt` parse. Measured with the whole-read allocation counter: unpaid read memory grows by about 140 to 190 bytes per claim (none 8032, Avail 8175, P95 8224, all four 8656). It scales with the claim count and with the claim key's size, and real FR-640 keys carry every operand segment. No claim package is registered in `whole_read_accounting`, so the gate cannot see it | crates/filament-contract-model/src/checked_package/v2/probabilistic_claim_read.rs:283-289,327,457,475 |
| FND-002 | medium | The placement walk gives the formula slot to the claim root's member at index 5 without checking that its name is `form`. FR-370 places the formula at "the `formula` binding of `form`'s `event`", by name. FR-640-AC-11 says a body that also breaks rule 6 refuses at rule 6's member. Reproduced: `Avail` with `delays` removed refuses `ill_typed`/`operator-ineligible` at the claim node instead of `invalid_package`/`invalid-value` at `/body`. P95-style claims without a formula are unaffected. Check `binding_named(member_term, "form")` as the event and formula arms do | crates/filament-contract-model/src/checked_package/v2/temporal.rs:893 |
| FND-003 | low | The read path uses plain `CheckedNodeId` ordering, which is uncharged and spills frames on distinct nested operand chains. This happens in the fairness order check (`earlier >= key`) and in `retain_claims`' sort. Claim keys hold operand segments, so two claims with one name compare nested operands. The precedent is the same (PR #1076 FND-002, accepted-no-change), but claim keys widen the exposure. Compare by graph rank (`join.by_id` position) | crates/filament-contract-model/src/checked_package/v2/probabilistic_join/claims.rs:334; crates/filament-contract-model/src/checked_package/v2/probabilistic_claim_read.rs:533 |
| FND-004 | low | The new `CAPACITY` comment, and FR-377's grammar paragraph, say the deepest member is a `from` at 17 steps. The reader actually names 19: a `weighting` `kind` inside a `fraction` measure of a `compare` event (`…/members/5/value/members/1/value/members/1/value/members/3/value/members/0/value`, the delay-weighting time refusal). 20 still suffices, so this is no crash. But the margin is 1, not 3, and a `CAPACITY = 18` mutant survives the whole claim suite: no test reaches a path deeper than 18 steps | crates/filament-contract-model/src/checked_package/v2/stage_storage.rs:44-49 |
