---
id: SR-20261121101006000002
title: "QSL-2178 gap-analysis disposition round 1"
type: SpecReview
analysis: gap-analysis
scope: "QSL-2178; filament-contract-ir#1108; original SR-2026112106202500000002"
review_set: base
---

## Summary

PASS / CLEAR: 3 original findings closed, zero still open. Disposition round 1 of the existing completed review of QSL-2178 / filament-contract-ir#1108. Verified frozen head `b43165e9be3aeb6209a79c69af19f2eb36411332` against integrated base `25637dae5fd7f829944a3873678750b0099c57ff`; original reviewed head `289381ccb9da2e157caf3a7466b987c3ac1641fc`. This is a bounded recheck of the original ten findings, not a new first review. Original review artifacts were unavailable; the full original immutable Linear comments are reproduced below, including their finding rows verbatim. Fresh document IDs identify these disposition documents and do not replace the original review IDs.

No applicable AssuranceProfile was found under spec/. No new finding was identified in the bounded fixes. This pass inspected source, exact finding regressions and retained execution logs; it ran no Cargo command, full gate, strict matrix or baseline campaign. It creates no worktree, source change or canonical-wire change.

Original method: gap-analysis; original review ID: SR-2026112106202500000002; original Linear comment: 28be4ad9-c06e-4506-aabf-726bed949656. The code-review disposition includes its Rust lane. The specification disposition preserves the original base method; no new analysis lenses were launched.

## Findings

The summaries below retain the original finding text. Current outcomes are in Dispositions.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The FR-154 constraint rules for a target-bound value type are not implemented. The new alias branch continues past the row's constraints, so they are silently dropped. Measured on the PR (and also admitted on main): Count (target Integer) with min, and Watch (target Clock) with maxLength, are both admitted, but FR-154-AC-51 says each refuses malformed-declaration at Constraint.keyword. Watch with max is admitted, but the max bound is discarded, not carried to the clock places. | crates/filament-contract-model/src/checked_package/v2/model_members.rs |
| FND-002 | medium | tc_234_bound_value_type_refusals_and_option_over_a_reference is traced to FR-154-AC-53 and FR-038-AC-591. What it asserts is FR-154-AC-51 (TC-2048): target Int64 refuses at /types/1/target, and target together with scalar refuses at /types/1/scalar. It also asserts FR-154 Option value types. No FR-038 AC states those refusals. | crates/filament-contract-model/src/checked_package/v2/model_members/tests.rs |
| FND-003 | low | FR-154-AC-53's loci, Field.typeRef for ix://filament/native/ClockReference and TypeDefinition.keys for the map key, are not asserted. The test calls read(), which returns only the ModelRefusal and no document pointer. | crates/filament-contract-model/src/checked_package/v2/model_members/tests.rs |

## Dispositions

Round 1, verified at b43165e9be3aeb6209a79c69af19f2eb36411332.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Mixed resolution: the actual keyword-validation defect is fixed at the exact Constraint.keyword, including a second constraint index. The requested IR max-bound carrier is rejected as not applicable under the 09:45 architecture ruling and normative FSpec FR-154 lines 331-349: the selected raw document is retained, the FSL checker bounds clock places by max and refuses min as invalid_timed_model/time-invariant-shape. No new IR wire member or typed carrier is required. |
| FND-002 | fixed | The target/scalar refusal test now traces TC-2048, FR-154-AC-51 and its own native-target contract FR-038-AC-600, with TC-048. It no longer claims the Clock-reference AC-53/AC-591 binding. |
| FND-003 | fixed | The Clock-reference test uses located_refusal, asserting malformed-declaration at /types/0/fields/1/typeRef and ill_typed/operator-ineligible at /types/6/keys, rather than discarding the document pointer. |

## Evidence

Evidence read: integrated native-target tests 3 passed, Clock tests 2 passed, prepaid-storage test 1 passed, exact public-shape test 1 passed and public-interface tests 7 passed, all exit 0. Logs: /private/tmp/ir1108-integrated-native-target.log, /private/tmp/ir1108-integrated-clock.log, /private/tmp/ir1108-integrated-budget.log, /private/tmp/ir1108-integrated-public-shape.log and /private/tmp/ir1108-integrated-public-interface.log. Prior scoped Clippy logs /private/tmp/ir1108-final-model-clippy.log and /private/tmp/ir1108-final-it-clippy.log finish successfully. Production files are unchanged from the prior frozen integration head 914ddd639113b976054c7ca6d8a475804376b6fa; the integration only renumbers this PR's native-target criterion/traces to AC-600.

The old-native-gate mutation and independent charged/plain Boolean-node mutations each fail the intended test in /private/tmp/ir1108-old-native-gate-red.log, /private/tmp/ir1108-charged-boolean-red.log and /private/tmp/ir1108-plain-boolean-red.log. Restored final-base tests pass. Scoped specification validation passed with four pre-existing grammar warnings outside the changed acceptance criteria. git diff --check passed.

Plan completion: not assessed. Repository-wide matrix completeness and aggregate gates were not reassessed in this disposition round. PASS means all original findings have a closed disposition, not a new whole-repository audit.

## Architecture ownership

The confirmed 09:45 architecture ruling, relayed by the dispatching lead, states:

> scope confirmed as you recommend. The reader validates the constraint keywords on a native-target (Clock) value at the precise constraint pointer (min/max admitted per FR-154; any other keyword, or a constraint on a non-Clock target, refuses at Constraint.keyword) and retains the raw selected document; the alias branch must not skip that validation. No new Clock-bound wire member and no typed bound carrier in the IR: bounding clock places by max (and refusing min) stays the FSL checker’s job, as FR-154 assigns it. Record the reviewer’s gap as not applicable with that reason.

Normative source inspected: /private/tmp/ir1110-fspec-247a3941/spec/functional/type-model/FR-154-admit-domain-package-model.md, lines 331-349. It assigns max-bound interpretation and the invalid_timed_model/time-invariant-shape min refusal to the checker; intake validates keywords. The legacy shared FSpec checkout is older and was not used as authority for native-target semantics.

## Machine-readable dispositions

```yaml
{
  "original_review": {
    "id": "SR-2026112106202500000002",
    "comment_id": "28be4ad9-c06e-4506-aabf-726bed949656",
    "method": "gap-analysis",
    "head": "289381ccb9da2e157caf3a7466b987c3ac1641fc"
  },
  "disposition_review": {
    "id": "SR-20261121101006000002",
    "round": 1,
    "head": "b43165e9be3aeb6209a79c69af19f2eb36411332",
    "base": "25637dae5fd7f829944a3873678750b0099c57ff",
    "verdict": "PASS",
    "model": "gpt-6.2-astra",
    "run": "52ce559c-05d1-4ace-a8a1-1feb69753663"
  },
  "scope": [
    {
      "id": "FR-154-AC-53",
      "path": "FSpec spec/functional/type-model/FR-154-admit-domain-package-model.md",
      "role": "examined",
      "excerpt": "A domain package `acme/rpc` whose object type `Call` has clock fields `x` and `y` and a field `r` typed `ix://acme/rpc/ClockRef`, a type definition of `kind` `reference` with `target` `ix://filament/native/Clock`, is admitted with one declaration for `ClockRef`, and `r` is a data member of type `Reference<Clock>` with no clock member shape: `r` with presence `optional` is `Option<Reference<Clock>>` and with multiplicity `{0, 3, true, false}` is `Sequence<Reference<Clock>>[0, 3]`; the same holds with `target` an alias of `Clock` or a value type bound to it by `target`. The field typed `ix://fil"
    },
    {
      "id": "FR-154-AC-51",
      "path": "FSpec spec/functional/type-model/FR-154-admit-domain-package-model.md",
      "role": "examined",
      "excerpt": "... a value type `Watch` with `target` `ix://filament/native/Clock` typing a required field and an optional field is admitted as two clock members, a `max` constraint whose `appliesTo` is `Watch` is admitted, `maxLength` on it refuses `invalid_model_binding`/`malformed-declaration` at `Constraint.keyword`, ...; a `min` constraint on `Count` refuses `malformed-declaration` at `Constraint.keyword`; a value type with `target` `ix://filament/native/Int64` refuses `malformed-declaration` at `TypeDefinition.target`, and one with both `target` and `scalar` at `TypeDefinition.scalar`."
    },
    {
      "id": "FR-154 Option value types",
      "path": "FSpec spec/functional/type-model/FR-154-admit-domain-package-model.md",
      "role": "examined",
      "excerpt": "`inner` may name a native value type other than `Clock`, a unit, or any package value type, including a record, tuple, variant, sequence, map or reference type and an alias of those"
    },
    {
      "id": "FR-829-AC-7",
      "path": "FSpec spec/objects/interfaces/FR-829-carry-a-clock-place-reference-as-a-place-id-term.md",
      "role": "context_only",
      "excerpt": "Over FR-154-AC-53's `acme/rpc` package, the field `r` typed by the `reference` type `ClockRef` over `Clock` is typed in the checked package by the one `scalar_type`/`clock_reference` node, no other node stands for `ClockRef`, ..."
    },
    {
      "id": "FR-038-AC-591",
      "path": "spec/checked_package/functional/FR-038-consume-checked-package-v2.md",
      "role": "context_only",
      "excerpt": "Implemented (QSL-2178). Consumes FSpec FR-154 \"Clock reference types\" (FR-154-AC-53; TC-234 I37) and FR-322 \"Model-owned members\". A field typed by a `reference` type definition whose `target` is `ix://filament/native/Clock`, an alias of it or a value type bound to it by `target` is a data member of type `Reference<Clock>` (`CheckedMemberType::ClockReference`, the package's one `scalar_type`/`clock_reference` node), `Option<Reference<Clock>>` under presence `optional` and `Sequence<Reference<Clock>>[0, 3]` under multiplicity `{0, 3, true, false}`; ..."
    }
  ],
  "original_findings": [
    {
      "fnd": "FND-001",
      "severity": "medium",
      "confidence": "high",
      "check": "coverage",
      "artifact_id": "FR-154-AC-51",
      "related_ids": [
        "FR-154"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/model_members.rs",
      "lines": "4791-4825",
      "excerpt": "... a value type `Watch` with `target` `ix://filament/native/Clock` typing a required field and an optional field is admitted as two clock members, a `max` constraint whose `appliesTo` is `Watch` is admitted, `maxLength` on it refuses `invalid_model_binding`/`malformed-declaration` at `Constraint.keyword`, ...; a `min` constraint on `Count` refuses `malformed-declaration` at `Constraint.keyword`; a value type with `target` `ix://filament/native/Int64` refuses `malformed-declaration` at `TypeDefinition.target`, and one with both `target` and `scalar` at `TypeDefinition.scalar`.",
      "finding": "The FR-154 constraint rules for a target-bound value type are not implemented. The new alias branch continues past the row's constraints, so they are silently dropped. Measured on the PR (and also admitted on main): Count (target Integer) with min, and Watch (target Clock) with maxLength, are both admitted, but FR-154-AC-51 says each refuses malformed-declaration at Constraint.keyword. Watch with max is admitted, but the max bound is discarded, not carried to the clock places.",
      "method": "gap-analysis"
    },
    {
      "fnd": "FND-002",
      "severity": "medium",
      "confidence": "high",
      "check": "trace",
      "artifact_id": "FR-154-AC-51",
      "related_ids": [
        "FR-154-AC-53",
        "FR-038-AC-591"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/model_members/tests.rs",
      "lines": "2251",
      "excerpt": "... a value type `Watch` with `target` `ix://filament/native/Clock` typing a required field and an optional field is admitted as two clock members, a `max` constraint whose `appliesTo` is `Watch` is admitted, `maxLength` on it refuses `invalid_model_binding`/`malformed-declaration` at `Constraint.keyword`, ...; a `min` constraint on `Count` refuses `malformed-declaration` at `Constraint.keyword`; a value type with `target` `ix://filament/native/Int64` refuses `malformed-declaration` at `TypeDefinition.target`, and one with both `target` and `scalar` at `TypeDefinition.scalar`.",
      "test_id": "tc_234_bound_value_type_refusals_and_option_over_a_reference",
      "claimed_ac": "FR-154-AC-53",
      "correct_ac": "FR-154-AC-51",
      "finding": "tc_234_bound_value_type_refusals_and_option_over_a_reference is traced to FR-154-AC-53 and FR-038-AC-591. What it asserts is FR-154-AC-51 (TC-2048): target Int64 refuses at /types/1/target, and target together with scalar refuses at /types/1/scalar. It also asserts FR-154 Option value types. No FR-038 AC states those refusals.",
      "method": "gap-analysis"
    },
    {
      "fnd": "FND-003",
      "severity": "low",
      "confidence": "high",
      "check": "test-intent",
      "artifact_id": "FR-154-AC-53",
      "related_ids": [
        "tc_234_a_reference_to_clock_is_a_data_member"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/model_members/tests.rs",
      "lines": "2224-2243",
      "excerpt": "A domain package `acme/rpc` whose object type `Call` has clock fields `x` and `y` and a field `r` typed `ix://acme/rpc/ClockRef`, a type definition of `kind` `reference` with `target` `ix://filament/native/Clock`, is admitted with one declaration for `ClockRef`, and `r` is a data member of type `Reference<Clock>` with no clock member shape: `r` with presence `optional` is `Option<Reference<Clock>>` and with multiplicity `{0, 3, true, false}` is `Sequence<Reference<Clock>>[0, 3]`; the same holds with `target` an alias of `Clock` or a value type bound to it by `target`. The field typed `ix://fil",
      "finding": "FR-154-AC-53's loci, Field.typeRef for ix://filament/native/ClockReference and TypeDefinition.keys for the map key, are not asserted. The test calls read(), which returns only the ModelRefusal and no document pointer.",
      "method": "gap-analysis"
    }
  ],
  "dispositions": [
    {
      "fnd": "FND-001",
      "outcome": "fixed",
      "reason": "Mixed resolution: the actual keyword-validation defect is fixed at the exact Constraint.keyword, including a second constraint index. The requested IR max-bound carrier is rejected as not applicable under the 09:45 architecture ruling and normative FSpec FR-154 lines 331-349: the selected raw document is retained, the FSL checker bounds clock places by max and refuses min as invalid_timed_model/time-invariant-shape. No new IR wire member or typed carrier is required.",
      "after_excerpt": "                let clock =\n                    row.get(\"target\").and_then(View::as_str) == Some(\"ix://filament/native/Clock\");\n                for (index, constraint) in list(row, \"constraints\")?.iter().enumerate() {\n                    if !clock\n                        || !matches!(\n                            constraint.get(\"keyword\").and_then(View::as_str),\n                            Some(\"min\" | \"max\")\n                        )",
      "resolution_parts": [
        {
          "part": "keyword validation",
          "outcome": "fixed"
        },
        {
          "part": "IR max-bound typed carrier",
          "outcome": "rejected",
          "reason": "Not applicable: FSpec FR-154 and the confirmed architecture ruling assign interpretation of max and timed refusal of min to the FSL checker."
        }
      ]
    },
    {
      "fnd": "FND-002",
      "outcome": "fixed",
      "reason": "The target/scalar refusal test now traces TC-2048, FR-154-AC-51 and its own native-target contract FR-038-AC-600, with TC-048. It no longer claims the Clock-reference AC-53/AC-591 binding.",
      "after_excerpt": "#[trace(\"TC-2048\", \"FR-154-AC-51\", \"TC-048\", \"FR-038-AC-600\")]\n#[test]\nfn tc_234_bound_value_type_refusals_and_option_over_a_reference() {"
    },
    {
      "fnd": "FND-003",
      "outcome": "fixed",
      "reason": "The Clock-reference test uses located_refusal, asserting malformed-declaration at /types/0/fields/1/typeRef and ill_typed/operator-ineligible at /types/6/keys, rather than discarding the document pointer.",
      "after_excerpt": "    let (refused, pointer) = located_refusal(&document(types));\n    assert_eq!(\n        refused,\n        ModelRefusal::new(Code::IllTyped, Cause::OperatorIneligible)\n    );\n    assert_eq!(pointer, \"/types/6/keys\");"
    }
  ],
  "bindings": [
    {
      "test_id": "tc_234_bound_value_type_refusals_and_option_over_a_reference",
      "ac_id": "FR-154-AC-51",
      "trace": "correct"
    },
    {
      "test_id": "tc_234_bound_value_type_refusals_and_option_over_a_reference",
      "ac_id": "FR-038-AC-600",
      "trace": "correct"
    },
    {
      "test_id": "tc_234_a_reference_to_clock_is_a_data_member",
      "ac_id": "FR-154-AC-53",
      "trace": "correct"
    },
    {
      "test_id": "tc_234_a_reference_to_clock_is_a_data_member",
      "ac_id": "FR-038-AC-591",
      "trace": "correct"
    }
  ]
}
```

## Original review comment (verbatim)

Source: Linear QSL-2178 comment 28be4ad9-c06e-4506-aabf-726bed949656, original SR-2026112106202500000002, original method gap-analysis, original head 289381ccb9da2e157caf3a7466b987c3ac1641fc. The fenced content is immutable source material; original findings and metadata are preserved exactly.

````markdown
<!-- reviewer repo=agent-ix/filament-contract-ir visibility=public quoin=0.29.1 module=spec-artifacts-process@b82e42e893612b4b4ee94127714b4577ee893502 id=SR-2026112106202500000002 method=gap-analysis lang=rust pr=filament-contract-ir#1108 model=claude-opus-5-5 run=707c7059-9da6-4142-a493-1be8aa68c420 -->

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | medium | coverage | The FR-154 constraint rules for a target-bound value type are not implemented |
| FND-002 | medium | trace | tc_234_bound_value_type_refusals_and_option_over_a_reference is traced to FR-154-AC-53 and FR-038-AC-591 |
| FND-003 | low | test-intent | FR-154-AC-53's loci, Field.typeRef for ix://filament/native/ClockReference and TypeDefinition.keys for the map key, are not asserted |

+++ [reviewer data]

```yaml
clean: false
scope:
- id: FR-154-AC-53
  path: FSpec spec/functional/type-model/FR-154-admit-domain-package-model.md
  role: examined
  excerpt: 'A domain package `acme/rpc` whose object type `Call` has clock fields `x` and `y` and a field `r` typed `ix://acme/rpc/ClockRef`, a type definition of `kind` `reference` with `target` `ix://filament/native/Clock`, is admitted with one declaration for `ClockRef`, and `r` is a data member of type `Reference<Clock>` with no clock member shape: `r` with presence `optional` is `Option<Reference<Clock>>` and with multiplicity `{0, 3, true, false}` is `Sequence<Reference<Clock>>[0, 3]`; the same holds with `target` an alias of `Clock` or a value type bound to it by `target`. The field typed `ix://fil'
- id: FR-154-AC-51
  path: FSpec spec/functional/type-model/FR-154-admit-domain-package-model.md
  role: examined
  excerpt: '... a value type `Watch` with `target` `ix://filament/native/Clock` typing a required field and an optional field is admitted as two clock members, a `max` constraint whose `appliesTo` is `Watch` is admitted, `maxLength` on it refuses `invalid_model_binding`/`malformed-declaration` at `Constraint.keyword`, ...; a `min` constraint on `Count` refuses `malformed-declaration` at `Constraint.keyword`; a value type with `target` `ix://filament/native/Int64` refuses `malformed-declaration` at `TypeDefinition.target`, and one with both `target` and `scalar` at `TypeDefinition.scalar`.'
- id: FR-154 Option value types
  path: FSpec spec/functional/type-model/FR-154-admit-domain-package-model.md
  role: examined
  excerpt: '`inner` may name a native value type other than `Clock`, a unit, or any package value type, including a record, tuple, variant, sequence, map or reference type and an alias of those'
- id: FR-829-AC-7
  path: FSpec spec/objects/interfaces/FR-829-carry-a-clock-place-reference-as-a-place-id-term.md
  role: context_only
  excerpt: Over FR-154-AC-53's `acme/rpc` package, the field `r` typed by the `reference` type `ClockRef` over `Clock` is typed in the checked package by the one `scalar_type`/`clock_reference` node, no other node stands for `ClockRef`, ...
- id: FR-038-AC-591
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: Implemented (QSL-2178). Consumes FSpec FR-154 "Clock reference types" (FR-154-AC-53; TC-234 I37) and FR-322 "Model-owned members". A field typed by a `reference` type definition whose `target` is `ix://filament/native/Clock`, an alias of it or a value type bound to it by `target` is a data member of type `Reference<Clock>` (`CheckedMemberType::ClockReference`, the package's one `scalar_type`/`clock_reference` node), `Option<Reference<Clock>>` under presence `optional` and `Sequence<Reference<Clock>>[0, 3]` under multiplicity `{0, 3, true, false}`; ...
findings:
- fnd: FND-001
  severity: medium
  confidence: high
  check: coverage
  artifact_id: FR-154-AC-51
  related_ids:
  - FR-154
  path: crates/filament-contract-model/src/checked_package/v2/model_members.rs
  lines: 4791-4825
  excerpt: '... a value type `Watch` with `target` `ix://filament/native/Clock` typing a required field and an optional field is admitted as two clock members, a `max` constraint whose `appliesTo` is `Watch` is admitted, `maxLength` on it refuses `invalid_model_binding`/`malformed-declaration` at `Constraint.keyword`, ...; a `min` constraint on `Count` refuses `malformed-declaration` at `Constraint.keyword`; a value type with `target` `ix://filament/native/Int64` refuses `malformed-declaration` at `TypeDefinition.target`, and one with both `target` and `scalar` at `TypeDefinition.scalar`.'
  finding: 'The FR-154 constraint rules for a target-bound value type are not implemented. The new alias branch continues past the row''s constraints, so they are silently dropped. Measured on the PR (and also admitted on main): Count (target Integer) with min, and Watch (target Clock) with maxLength, are both admitted, but FR-154-AC-51 says each refuses malformed-declaration at Constraint.keyword. Watch with max is admitted, but the max bound is discarded, not carried to the clock places.'
  method: gap-analysis
- fnd: FND-002
  severity: medium
  confidence: high
  check: trace
  artifact_id: FR-154-AC-51
  related_ids:
  - FR-154-AC-53
  - FR-038-AC-591
  path: crates/filament-contract-model/src/checked_package/v2/model_members/tests.rs
  lines: '2251'
  excerpt: '... a value type `Watch` with `target` `ix://filament/native/Clock` typing a required field and an optional field is admitted as two clock members, a `max` constraint whose `appliesTo` is `Watch` is admitted, `maxLength` on it refuses `invalid_model_binding`/`malformed-declaration` at `Constraint.keyword`, ...; a `min` constraint on `Count` refuses `malformed-declaration` at `Constraint.keyword`; a value type with `target` `ix://filament/native/Int64` refuses `malformed-declaration` at `TypeDefinition.target`, and one with both `target` and `scalar` at `TypeDefinition.scalar`.'
  test_id: tc_234_bound_value_type_refusals_and_option_over_a_reference
  claimed_ac: FR-154-AC-53
  correct_ac: FR-154-AC-51
  finding: 'tc_234_bound_value_type_refusals_and_option_over_a_reference is traced to FR-154-AC-53 and FR-038-AC-591. What it asserts is FR-154-AC-51 (TC-2048): target Int64 refuses at /types/1/target, and target together with scalar refuses at /types/1/scalar. It also asserts FR-154 Option value types. No FR-038 AC states those refusals.'
  method: gap-analysis
- fnd: FND-003
  severity: low
  confidence: high
  check: test-intent
  artifact_id: FR-154-AC-53
  related_ids:
  - tc_234_a_reference_to_clock_is_a_data_member
  path: crates/filament-contract-model/src/checked_package/v2/model_members/tests.rs
  lines: 2224-2243
  excerpt: 'A domain package `acme/rpc` whose object type `Call` has clock fields `x` and `y` and a field `r` typed `ix://acme/rpc/ClockRef`, a type definition of `kind` `reference` with `target` `ix://filament/native/Clock`, is admitted with one declaration for `ClockRef`, and `r` is a data member of type `Reference<Clock>` with no clock member shape: `r` with presence `optional` is `Option<Reference<Clock>>` and with multiplicity `{0, 3, true, false}` is `Sequence<Reference<Clock>>[0, 3]`; the same holds with `target` an alias of `Clock` or a value type bound to it by `target`. The field typed `ix://fil'
  finding: FR-154-AC-53's loci, Field.typeRef for ix://filament/native/ClockReference and TypeDefinition.keys for the map key, are not asserted. The test calls read(), which returns only the ModelRefusal and no document pointer.
  method: gap-analysis
bindings:
- test_id: tc_234_a_reference_to_clock_is_a_data_member
  ac_id: FR-154-AC-53
  trace: correct
- test_id: tc_234_a_reference_to_clock_is_a_data_member
  ac_id: FR-038-AC-591
  trace: correct
- test_id: tc_234_bound_value_type_refusals_and_option_over_a_reference
  ac_id: FR-154-AC-53
  trace: wrong
```

+++
````

