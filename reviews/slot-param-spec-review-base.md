---
id: SR-17940486709564847048134
title: "Spec review — FR-402 reserved slot form as exported for QSL-2581"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1104 at 47f92538; reference spec agent-ix/filament-specification at FSPEC-main-6ad873: FR-402 Slot parameters, Reserved slot names, FR-402-AC-9, FR-402-AC-12"
review_set: subset
---

## Summary

Ticket: QSL-2581. The PR changes no FSpec text. This base review checks whether the FR-402 grammar the export implements is complete and unambiguous enough for an FSL checker to rely on.

## Examined scope

- FR-402 "Slot parameters" (examined). `esc(x)` maps an ASCII letter or digit to itself, `_` to `__`, and any other byte to `_x` plus two uppercase hex digits. The names are `slot_s<p>_s<t>`, `op_s<t>`, `rcv_s<t>` and `arg_s<t>_s<T>_s<n>_s<p>`. The tags hold no `_`, and `__`, `_xHH` and `_s` are prefix-free, so a name decodes uniquely.
- FR-402 "Reserved slot names" (examined). The reserved form is the decoder-accepted set: non-empty fields of units, well-formed UTF-8 per field, and the exact field count. Non-canonical escapes (`op_s_x41`) are reserved. Its listed non-reserved look-alikes are all consistent with the implementation.
- FR-402-AC-9 (examined) is the encoding and distinctness criterion. FR-402-AC-12 (examined) is the checker and reader reservation criterion, with explicit reserved and non-reserved examples. Both are testable.

## Verdict

**Clean.** The grammar is complete and unambiguous. The text defines exactly four tags, and the implementation matches it.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
