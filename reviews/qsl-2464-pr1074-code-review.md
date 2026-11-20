---
id: SR-17940486709564847048083
title: "code-review and rust-review of QSL-2464 model-root over for hyper body wrappers"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1074 at cf3c7807 (base origin/main c7b62038); crates/filament-contract-model/src/checked_package/v2/temporal.rs, tests/it/checked_package_v2_temporal.rs"
review_set: subset
---

## Summary

Code review of filament-contract-ir PR #1074, with the rust-review lane folded in. Ticket: QSL-2464. Authority: a private FSpec worktree at origin/main 3b5fc9e0. The rules checked are FR-402 "Body wrapper", "Slot parameters" and "Reader order" step 3, FR-370 argument 1 (`over`) and its temporal-step `over` check, and FR-402-AC-11/TC-2034. The contract-ir ACs are FR-038-AC-562/563 and TC-486.

The coder made three claims. Each was checked against the spec text:

- **Claim 1: the `over` is the model/model_root of the first `subjects` binding when the formula reads no slot parameter.** The temporal step admits a declared `model_root` `over` only on a body wrapper whose formula reaches no slot-named parameter (temporal.rs:1296-1307). The hyperproperty step then requires the root to be the first subject's (temporal.rs:2695-2724). This matches FR-370's `over` step and FR-402 step 3. Slot detection is by name prefix (FND-002).
- **Claim 2: every other model-root `over` refuses `invalid_model_binding`/`malformed-declaration` at `/semantic_graph/nodes/{w}/body/arguments/0`.** Verified by reading: the else branch at temporal.rs:1304 and the hyper refusal at temporal.rs:2714-2722. The tests pin code, cause, pointer and locus.
- **Claim 3: an `over` not among the wrapper's dependencies keeps `missing_declaration`/`missing-name`.** Verified. The existing `!node.dependencies.contains(over)` arm precedes the new arm (temporal.rs:1293), and a test pins it.

Gates run in a scratch worktree at cf3c7807, with its own target and `cargo-slot`, single slot:

- `cargo fmt --all --check`: clean.
- `make string-edge`: clean.
- The focused `--test it` run (`checked_package_v2_temporal whole_read_accounting claim_names`, with FILAMENT_SPECIFICATION_DIR set to FSpec 3b5fc9e0): 136 passed, 0 failed. Both `tc_486_*` tests and every `whole_read_accounting` test passed.

More gates:

- `cargo clippy --locked --workspace --all-targets --features conformance-cli -- -D warnings`: clean.
- Filtered lib run `-p filament-contract-model --lib -- temporal hyper monitor_plan`: 43 passed, 18 failed. The 18 failures are exactly the known-red `monitor_plan::reached_model_read` tests.
- The same `--test it` filter on clean origin/main (358e9e2d, since main has moved): 134 passed, 0 failed. Head adds exactly the two `tc_486_*` tests, all passing, so the PR introduces no new failure.
- Hand mutants: see the gap-analysis SR.

## Examined scope

- temporal.rs `check_clause` `over` arm (examined): it handles the order of declared versus model_root versus parameter and keeps the `missing-name` precedence.
- temporal.rs `is_body_wrapper`, `member_binding`, `is_slot_parameter_name` and `formula_reads_slot_parameter` (examined).
- temporal.rs `check_hyper_shape` step 3 (examined): the new first-subject check and its position relative to the body-kind and fairness checks.
- tests/it/checked_package_v2_temporal.rs `tc_486_*` and their helpers (examined).
- The work and storage metering of the new traversal (examined).

## Verdict

The main path is correct and well located. The three claims hold. The FSpec rule is implemented, apart from the points below.

Not mergeable as is: FND-001 (check order inside step 3) and FND-002 (slot detection by prefix rather than by decoding the name) should be resolved first, or explicitly deferred with an arch ruling (spec-review FND-001/FND-002). The medium metering point (FND-003) needs either a work charge or a stated rationale.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Wrong order inside the hyperproperty step. The new first-subject root check runs before the existing non-empty-fairness check, so a wrapper whose `fairness` argument is non-empty and whose `over` is a non-first-subject root refuses `invalid_model_binding` at `arguments/0`. FR-402 "Reader order" step 3 lists the fairness refusal (`invalid_semantic_graph` at `arguments/4`) first, then the model-root refusal. Move the root check after the fairness check, or get an arch ruling (spec-review FND-002) and pin the order with a test | crates/filament-contract-model/src/checked_package/v2/temporal.rs:2695-2752 |
| FND-002 | medium | Slot detection by name prefix. `is_slot_parameter_name` matches the prefixes `slot_s`, `op_s`, `rcv_s` and `arg_s` and does not decode the name as FR-402 "Slot parameters" requires ("a consumer SHALL decode a slot's role and fields from the name by the same encoding"). So names that are not valid encodings count as slot parameters: `slot_sv` has one field where `slot` needs two, `op_sa_sb` has two where `op` needs one, and so does any name whose fields hold an invalid `_` escape. A non-slot `value`/`parameter` (a bound or captured variable) whose source name begins with one of the prefixes also flips an admissible root `over` to a refusal. FSpec gives no structural alternative (spec-review FND-001). At minimum, decode the tag, the field count and the escapes | crates/filament-contract-model/src/checked_package/v2/temporal.rs:1153-1160 |
| FND-003 | medium | The new traversal charges no work. `formula_reads_slot_parameter` walks the formula's transitive `dependencies` closure, and `is_body_wrapper` scans every node for hyperproperty nodes. Both run once per clause whose `over` is a model root, so CPU cost is O(K x (N + E)) for K such clauses, and none of it is charged to the work budget FR-038 says bounds the V2 read's work. Each call also charges a fresh N-byte `seen` vector to read storage, O(K x N) cumulative. Charge one work unit per visited node and per examined edge, or compute the wrapper set and slot reachability once per step | crates/filament-contract-model/src/checked_package/v2/temporal.rs:1134-1201,1296-1307 |
| FND-004 | low | The hyper step decodes the wrapper body twice: once for `over` (2702) and again for `fairness` (2726). Each decode charges read storage. Both sites discard a budget `ValidationFailure` through `.ok()?`, so a budget stop on the new decode silently skips the check. The swallowing matches the existing line 2726. Decode once and propagate the budget failure | crates/filament-contract-model/src/checked_package/v2/temporal.rs:2701-2708,2725-2726 |


## Disposition pass 1 notes

Round 1, head 8bca5ece: fix commit 8bca5ece on top of cf3c7807, base still c7b62038. Gates at this head, single slot:

- fmt and `make string-edge`: clean. `is_slot_parameter_name` now carries `#[ix_trace_rs::string_edge]`.
- `cargo clippy --locked --workspace --all-targets --features conformance-cli -- -D warnings`: clean.
- Focused `--test it -- checked_package_v2_temporal whole_read_accounting claim_names`: 138 passed, 0 failed. That is main's 134 plus the four `tc_486_*` tests. Those four include the new `root_wrapper_fixture` registered as a `whole_read_accounting` positive.
- Filtered lib run: 43 passed, 18 failed. The failures are identical to clean main: the known `monitor_plan::reached_model_read` set.

How the fix round was judged:

- **Slot decoding.** The decoder parses FR-402's grammar: the tag, the field count, `__`, `_x` with two uppercase hex digits, `_s`, and UTF-8 in each field. It treats an empty field (`rcv_s`) as non-slot. That is consistent with the spec: every field is a source name (a trace variable, object parameter, type, operation or parameter name), and none of these can be empty. It accepts non-canonical escapes such as `_x41`. Any collision this allows is a case of the name-collision class in spec-review FND-001, so it is not raised again here.
- **Work charging.** The `clean` memo is set at enqueue. That is sound because a slot find always ends the read with the refusal. With it, the per-read traversal is O(N+E) over the already-decoded and already-charged graph. Stack growth and the three N-sized vectors are charged through the meter. No other temporal-step graph walk charges work per visit either, and FR-038 states no per-visit work rule for this step, so the quadratic uncharged cost FND-003 named is gone.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |
| FND-004 | fixed | - |
