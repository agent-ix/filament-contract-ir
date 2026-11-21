---
id: SR-17940486709564847048132
title: "Code review — QSL-2581 export is_slot_parameter_name"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR #1104 at 47f92538; crates/filament-contract-model/src/checked_package/v2/temporal.rs; crates/filament-contract-model/src/checked_package/v2/mod.rs; crates/filament-contract-model/src/lib.rs; spec/model/functional/FR-019-rust-library-interface.md; tests/it/checked_package_v2_temporal.rs"
review_set: subset
---

## Summary

Ticket: QSL-2581. I reviewed `git diff origin/main...HEAD` for PR #1104 at 47f92538 (5 files, +87/-9) against FSpec FR-402 "Slot parameters" and "Reserved slot names" and FR-402-AC-9 and AC-12. The FSpec checkout is FSPEC-main-6ad873. The Rust review lane is folded in.

## Examined scope

- Behavior (examined). The only code change in `temporal.rs` is `fn` becoming `pub fn`, plus the doc comment. The body is byte-identical, `#[ix_trace_rs::string_edge]` is kept, and the reader's one caller (temporal.rs:1317) is unchanged. No reader behavior changes.
- Grammar against FR-402 "Reserved slot form" (examined):
  - Tags are `slot` 2, `op` 1, `rcv` 1, `arg` 4. FR-402 lists no other slot-parameter kind, and FR-322 and FR-370 add none.
  - The tag is the text before the first `_`, which is required to be followed by `s`.
  - Each field is `_s` then one or more units, a unit being an ASCII letter or digit, `__`, or `_x` with two uppercase hex digits. A lowercase hex digit, a truncated escape, a stray `_` or a non-ASCII char refuses.
  - The decoded bytes of each field are checked separately for well-formed UTF-8, and the field count must equal the tag's count.
  - The implementation accepts exactly FR-402's decoder-accepted set, including non-canonical escapes such as `op_s_x41`. That is what the spec and the QSL-2581 ruling require.
- Ambiguity (examined). `__`, `_xHH` and `_s` are prefix-free, so the split is unique. The checker refuses any source identifier the predicate accepts, which is the purpose of the export.
- API placement (examined). It is re-exported from `checked_package::v2` next to the other `temporal*` exports and from the crate root, like the other `checked_package` functions. The `is_` predicate name fits the crate's style.
- Tests (examined). The new test's expectations are literal names, mostly the spec's own examples (`slot_sv_sa`, `op_sa`, `arg_sa_sVault_sstep_si`, `slot_sa__b_sc`, `slot_sa_sb__c`, `slot_sv`, `op_sa_sb`, `rcv_s`, `op_sa_x4`, `op_s_xC3`, `slotted`, `operand`). They are not restated from the implementation. 26/26 tests pass under the `is_slot_parameter_name public_interface tc_058` filter.
- Mutants (examined):
  - Killed (8 of 9): empty field allowed, UTF-8 check dropped, lowercase hex allowed, `count >= fields`, `rcv` taking 2 fields, `arg` taking 3, the `_s` prefix after the tag made optional, and `_x00` refused.
  - **Survived (1)**: refusing a non-canonical escape (an `_xHH` that decodes to a letter, digit or `_`). See FND-001.

## Verdict

**Changes requested (medium).** The code is correct and behavior-neutral. The doc comment misdescribes the accepted set at the one point the FSL checker depends on, and no test pins that point.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No test pins that a non-canonical escape is reserved. FR-402 "Reserved slot names" and FR-402-AC-12 name `op_s_x41` reserved, and the QSL-2581 ruling is "reserved form is the decoder-accepted set". A mutant that refuses `_xHH` decoding to a letter, digit or `_` survives the test. Add `op_s_x41` (and `op_sx` from AC-12) to the accepted names. | tests/it/checked_package_v2_temporal.rs:5576-5591 |
| FND-002 | medium | The doc comment says "A field is the non-empty `esc` of a source name", which describes the canonical encoder output. The function decides the wider reserved form, units of letter, digit, `__` or `_xHH` (for example `op_s_x41`, which no `esc` produces). An FSL checker author reading the doc would under-refuse. The comment also cites FR-402-AC-9 (the encoding) rather than "Reserved slot names" / FR-402-AC-12, which defines this predicate. | crates/filament-contract-model/src/checked_package/v2/temporal.rs:1208-1219 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | de4554fd adds `"op_s_x41"` and `"op_sx"` to the accepted names (tests/it/checked_package_v2_temporal.rs:5588-5590). The canonical-only mutant now fails the test. |
| FND-002 | fixed | de4554fd rewrites the doc comment: "A field is one or more units: an ASCII letter or digit, `__`, or `_x` and two uppercase hexadecimal digits, the bytes the units decode to forming valid UTF-8. This decides the decoder's accepted set, which is wider than the names the encoder writes: a non-canonical escape such as `op_s_x41` (the field `A`) is reserved too." It now cites FR-402 "Reserved slot names" (FR-402-AC-12). |
