---
id: SR-20261121101006000001
title: "QSL-2178 spec-review/base disposition round 1"
type: SpecReview
analysis: base
scope: "QSL-2178; filament-contract-ir#1108; original SR-2026112106202500000003"
review_set: base
---

## Summary

PASS / CLEAR: 2 original findings closed, zero still open. Disposition round 1 of the existing completed review of QSL-2178 / filament-contract-ir#1108. Verified frozen head `b43165e9be3aeb6209a79c69af19f2eb36411332` against integrated base `25637dae5fd7f829944a3873678750b0099c57ff`; original reviewed head `289381ccb9da2e157caf3a7466b987c3ac1641fc`. This is a bounded recheck of the original ten findings, not a new first review. Original review artifacts were unavailable; the full original immutable Linear comments are reproduced below, including their finding rows verbatim. Fresh document IDs identify these disposition documents and do not replace the original review IDs.

No applicable AssuranceProfile was found under spec/. No new finding was identified in the bounded fixes. This pass inspected source, exact finding regressions and retained execution logs; it ran no Cargo command, full gate, strict matrix or baseline campaign. It creates no worktree, source change or canonical-wire change.

Original method: spec-review/base; original review ID: SR-2026112106202500000003; original Linear comment: 00574a74-9d6d-44b0-be5c-2ab79740de17. The code-review disposition includes its Rust lane. The specification disposition preserves the original base method; no new analysis lenses were launched.

## Findings

The summaries below retain the original finding text. Current outcomes are in Dispositions.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-591 collides with open PR #1110 (head e09e01db), which also adds FR-038-AC-591 for QSL-2691 body locations and cites it in TC-491 and the matrix. It is still colliding at review time, so whichever PR merges second carries two rows with one id. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md |
| FND-002 | medium | Neither FR-038-AC-591 nor item 22 states the second behaviour change in this PR. A value type with a native target is now read as that native, so a field typed by it gains the native's type where it had none. And the reader now refuses invalid_model_binding/malformed-declaration at /types/<i>/target and at /types/<i>/scalar. The PR consumes FSpec FR-154 Value types bound to a native by target (FR-154-AC-51) with no contract-ir criterion. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md |

## Dispositions

Round 1, verified at b43165e9be3aeb6209a79c69af19f2eb36411332.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The integrated base renumbered #1110 compact body locations to AC-597. This PR retains Clock AC-591 and assigns native-target behavior AC-600, avoiding main hyper AC-598/599. Rows 591, 595, 597, 598, 599 and 600 each appear exactly once. |
| FND-002 | fixed | New AC-600 states ValueType meaning gating, native-target field derivation, unknown target and simultaneous scalar refusals, exact constraint keyword validation and checker ownership. TC-048 links AC-600/TC-2048 and supplies the admission/refusal procedure; production tests trace AC-600. |

## Evidence

Evidence read: integrated native-target tests 3 passed, Clock tests 2 passed, prepaid-storage test 1 passed, exact public-shape test 1 passed and public-interface tests 7 passed, all exit 0. Logs: /private/tmp/ir1108-integrated-native-target.log, /private/tmp/ir1108-integrated-clock.log, /private/tmp/ir1108-integrated-budget.log, /private/tmp/ir1108-integrated-public-shape.log and /private/tmp/ir1108-integrated-public-interface.log. Prior scoped Clippy logs /private/tmp/ir1108-final-model-clippy.log and /private/tmp/ir1108-final-it-clippy.log finish successfully. Production files are unchanged from the prior frozen integration head 914ddd639113b976054c7ca6d8a475804376b6fa; the integration only renumbers this PR's native-target criterion/traces to AC-600.

The old-native-gate mutation and independent charged/plain Boolean-node mutations each fail the intended test in /private/tmp/ir1108-old-native-gate-red.log, /private/tmp/ir1108-charged-boolean-red.log and /private/tmp/ir1108-plain-boolean-red.log. Restored final-base tests pass. Scoped specification validation passed with four pre-existing grammar warnings outside the changed acceptance criteria. git diff --check passed.

Plan completion: not assessed. Repository-wide matrix completeness and aggregate gates were not reassessed in this disposition round. PASS means all original findings have a closed disposition, not a new whole-repository audit.

## Machine-readable dispositions

```yaml
{
  "original_review": {
    "id": "SR-2026112106202500000003",
    "comment_id": "00574a74-9d6d-44b0-be5c-2ab79740de17",
    "method": "spec-review/base",
    "head": "289381ccb9da2e157caf3a7466b987c3ac1641fc"
  },
  "disposition_review": {
    "id": "SR-20261121101006000001",
    "round": 1,
    "head": "b43165e9be3aeb6209a79c69af19f2eb36411332",
    "base": "25637dae5fd7f829944a3873678750b0099c57ff",
    "verdict": "PASS",
    "model": "gpt-6.2-astra",
    "run": "52ce559c-05d1-4ace-a8a1-1feb69753663"
  },
  "scope": [
    {
      "id": "FR-038-AC-591",
      "path": "spec/checked_package/functional/FR-038-consume-checked-package-v2.md",
      "role": "examined",
      "excerpt": "Implemented (QSL-2178). Consumes FSpec FR-154 \"Clock reference types\" (FR-154-AC-53; TC-234 I37) and FR-322 \"Model-owned members\". A field typed by a `reference` type definition whose `target` is `ix://filament/native/Clock`, an alias of it or a value type bound to it by `target` is a data member of type `Reference<Clock>` (`CheckedMemberType::ClockReference`, the package's one `scalar_type`/`clock_reference` node), `Option<Reference<Clock>>` under presence `optional` and `Sequence<Reference<Clock>>[0, 3]` under multiplicity `{0, 3, true, false}`; ..."
    },
    {
      "id": "FR-038 item 22",
      "path": "spec/checked_package/functional/FR-038-consume-checked-package-v2.md",
      "role": "examined",
      "excerpt": "it mirrors `MemberType` variant for variant. A field typed by a `reference` type whose `target` is the native `Clock`, an alias of it or a value type bound to it is a data member of the variant `ClockReference` (`Reference<Clock>`), never the clock member (FSpec FR-154 \"Clock reference types\"; FR-038-AC-591)."
    },
    {
      "id": "FR-154 Value types bound to a native by target",
      "path": "FSpec spec/functional/type-model/FR-154-admit-domain-package-model.md",
      "role": "context_only",
      "excerpt": "A value type ... binds its native ... by a `target` naming `ix://filament/native/<Name>` ... Intake SHALL admit a value type with such a `target`, and the value type SHALL be that native under its own name, exactly as an alias whose `target` is the native is ... A `target` that begins `ix://filament/native/` and names no native of the set refuses ... at `TypeDefinition.target` ...; a value type carrying both `target` and `scalar` refuses the same at `TypeDefinition.scalar`."
    }
  ],
  "original_findings": [
    {
      "fnd": "FND-001",
      "severity": "medium",
      "confidence": "high",
      "check": "other",
      "artifact_id": "FR-038-AC-591",
      "related_ids": [
        "filament-contract-ir#1110"
      ],
      "path": "spec/checked_package/functional/FR-038-consume-checked-package-v2.md",
      "lines": "6377",
      "excerpt": "Implemented (QSL-2178). Consumes FSpec FR-154 \"Clock reference types\" (FR-154-AC-53; TC-234 I37) and FR-322 \"Model-owned members\". A field typed by a `reference` type definition whose `target` is `ix://filament/native/Clock`, an alias of it or a value type bound to it by `target` is a data member of type `Reference<Clock>` (`CheckedMemberType::ClockReference`, the package's one `scalar_type`/`clock_reference` node), `Option<Reference<Clock>>` under presence `optional` and `Sequence<Reference<Clock>>[0, 3]` under multiplicity `{0, 3, true, false}`; ...",
      "finding": "FR-038-AC-591 collides with open PR #1110 (head e09e01db), which also adds FR-038-AC-591 for QSL-2691 body locations and cites it in TC-491 and the matrix. It is still colliding at review time, so whichever PR merges second carries two rows with one id.",
      "method": "spec-review/base"
    },
    {
      "fnd": "FND-002",
      "severity": "medium",
      "confidence": "high",
      "check": "coverage",
      "artifact_id": "FR-038-AC-591",
      "related_ids": [
        "FR-154-AC-51"
      ],
      "path": "spec/checked_package/functional/FR-038-consume-checked-package-v2.md",
      "lines": "6377",
      "excerpt": "Implemented (QSL-2178). Consumes FSpec FR-154 \"Clock reference types\" (FR-154-AC-53; TC-234 I37) and FR-322 \"Model-owned members\". A field typed by a `reference` type definition whose `target` is `ix://filament/native/Clock`, an alias of it or a value type bound to it by `target` is a data member of type `Reference<Clock>` (`CheckedMemberType::ClockReference`, the package's one `scalar_type`/`clock_reference` node), `Option<Reference<Clock>>` under presence `optional` and `Sequence<Reference<Clock>>[0, 3]` under multiplicity `{0, 3, true, false}`; ...",
      "finding": "Neither FR-038-AC-591 nor item 22 states the second behaviour change in this PR. A value type with a native target is now read as that native, so a field typed by it gains the native's type where it had none. And the reader now refuses invalid_model_binding/malformed-declaration at /types/<i>/target and at /types/<i>/scalar. The PR consumes FSpec FR-154 Value types bound to a native by target (FR-154-AC-51) with no contract-ir criterion.",
      "method": "spec-review/base"
    }
  ],
  "dispositions": [
    {
      "fnd": "FND-001",
      "outcome": "fixed",
      "reason": "The integrated base renumbered #1110 compact body locations to AC-597. This PR retains Clock AC-591 and assigns native-target behavior AC-600, avoiding main hyper AC-598/599. Rows 591, 595, 597, 598, 599 and 600 each appear exactly once.",
      "after_excerpt": "| FR-038-AC-600 | Consumes FSpec FR-154 \"Value types bound to a native by target\" (FR-154-AC-51; TC-2048). A type whose kind resolves to `filament.meaning.model.value-type/v1` and whose `target` names a native is read as that native under its own name, including required and optional Clock members and Integer data members."
    },
    {
      "fnd": "FND-002",
      "outcome": "fixed",
      "reason": "New AC-600 states ValueType meaning gating, native-target field derivation, unknown target and simultaneous scalar refusals, exact constraint keyword validation and checker ownership. TC-048 links AC-600/TC-2048 and supplies the admission/refusal procedure; production tests trace AC-600.",
      "after_excerpt": "4. Refuse an unknown native at `target` and a simultaneous scalar binding at\n   `scalar` (FSpec FR-154-AC-51; TC-2048). Verify an object with a native\n   `target` remains an object reference and retains its fields; numeric and\n   undeclared kinds must refuse malformed declaration."
    }
  ],
  "bindings": []
}
```

## Original review comment (verbatim)

Source: Linear QSL-2178 comment 00574a74-9d6d-44b0-be5c-2ab79740de17, original SR-2026112106202500000003, original method spec-review/base, original head 289381ccb9da2e157caf3a7466b987c3ac1641fc. The fenced content is immutable source material; original findings and metadata are preserved exactly.

````markdown
<!-- reviewer repo=agent-ix/filament-contract-ir visibility=public quoin=0.29.1 module=spec-artifacts-process@b82e42e893612b4b4ee94127714b4577ee893502 id=SR-2026112106202500000003 method=spec-review/base lang=spec pr=filament-contract-ir#1108 model=claude-opus-5-5 run=707c7059-9da6-4142-a493-1be8aa68c420 -->

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | medium | other | FR-038-AC-591 collides with open PR #1110 (head e09e01db), which also adds FR-038-AC-591 for QSL-2691 body locations and cites it in TC-491 and the matrix |
| FND-002 | medium | coverage | Neither FR-038-AC-591 nor item 22 states the second behaviour change in this PR |

+++ [reviewer data]

```yaml
clean: false
scope:
- id: FR-038-AC-591
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (QSL-2178). Consumes FSpec FR-154 "Clock reference types" (FR-154-AC-53; TC-234 I37) and FR-322 "Model-owned members". A field typed by a `reference` type definition whose `target` is `ix://filament/native/Clock`, an alias of it or a value type bound to it by `target` is a data member of type `Reference<Clock>` (`CheckedMemberType::ClockReference`, the package's one `scalar_type`/`clock_reference` node), `Option<Reference<Clock>>` under presence `optional` and `Sequence<Reference<Clock>>[0, 3]` under multiplicity `{0, 3, true, false}`; ...
- id: FR-038 item 22
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: it mirrors `MemberType` variant for variant. A field typed by a `reference` type whose `target` is the native `Clock`, an alias of it or a value type bound to it is a data member of the variant `ClockReference` (`Reference<Clock>`), never the clock member (FSpec FR-154 "Clock reference types"; FR-038-AC-591).
- id: FR-154 Value types bound to a native by target
  path: FSpec spec/functional/type-model/FR-154-admit-domain-package-model.md
  role: context_only
  excerpt: A value type ... binds its native ... by a `target` naming `ix://filament/native/<Name>` ... Intake SHALL admit a value type with such a `target`, and the value type SHALL be that native under its own name, exactly as an alias whose `target` is the native is ... A `target` that begins `ix://filament/native/` and names no native of the set refuses ... at `TypeDefinition.target` ...; a value type carrying both `target` and `scalar` refuses the same at `TypeDefinition.scalar`.
findings:
- fnd: FND-001
  severity: medium
  confidence: high
  check: other
  artifact_id: FR-038-AC-591
  related_ids:
  - filament-contract-ir#1110
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  lines: '6377'
  excerpt: Implemented (QSL-2178). Consumes FSpec FR-154 "Clock reference types" (FR-154-AC-53; TC-234 I37) and FR-322 "Model-owned members". A field typed by a `reference` type definition whose `target` is `ix://filament/native/Clock`, an alias of it or a value type bound to it by `target` is a data member of type `Reference<Clock>` (`CheckedMemberType::ClockReference`, the package's one `scalar_type`/`clock_reference` node), `Option<Reference<Clock>>` under presence `optional` and `Sequence<Reference<Clock>>[0, 3]` under multiplicity `{0, 3, true, false}`; ...
  finding: 'FR-038-AC-591 collides with open PR #1110 (head e09e01db), which also adds FR-038-AC-591 for QSL-2691 body locations and cites it in TC-491 and the matrix. It is still colliding at review time, so whichever PR merges second carries two rows with one id.'
  method: spec-review/base
- fnd: FND-002
  severity: medium
  confidence: high
  check: coverage
  artifact_id: FR-038-AC-591
  related_ids:
  - FR-154-AC-51
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  lines: '6377'
  excerpt: Implemented (QSL-2178). Consumes FSpec FR-154 "Clock reference types" (FR-154-AC-53; TC-234 I37) and FR-322 "Model-owned members". A field typed by a `reference` type definition whose `target` is `ix://filament/native/Clock`, an alias of it or a value type bound to it by `target` is a data member of type `Reference<Clock>` (`CheckedMemberType::ClockReference`, the package's one `scalar_type`/`clock_reference` node), `Option<Reference<Clock>>` under presence `optional` and `Sequence<Reference<Clock>>[0, 3]` under multiplicity `{0, 3, true, false}`; ...
  finding: Neither FR-038-AC-591 nor item 22 states the second behaviour change in this PR. A value type with a native target is now read as that native, so a field typed by it gains the native's type where it had none. And the reader now refuses invalid_model_binding/malformed-declaration at /types/<i>/target and at /types/<i>/scalar. The PR consumes FSpec FR-154 Value types bound to a native by target (FR-154-AC-51) with no contract-ir criterion.
  method: spec-review/base
bindings: []
```

+++
````

