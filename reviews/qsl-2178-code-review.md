---
id: SR-20261121101006000003
title: "QSL-2178 code-review disposition round 1"
type: SpecReview
analysis: code-review
scope: "QSL-2178; filament-contract-ir#1108; original SR-2026112106202500000001"
review_set: base
---

## Summary

PASS / CLEAR: 5 original findings closed, zero still open. Disposition round 1 of the existing completed review of QSL-2178 / filament-contract-ir#1108. Verified frozen head `b43165e9be3aeb6209a79c69af19f2eb36411332` against integrated base `25637dae5fd7f829944a3873678750b0099c57ff`; original reviewed head `289381ccb9da2e157caf3a7466b987c3ac1641fc`. This is a bounded recheck of the original ten findings, not a new first review. Original review artifacts were unavailable; the full original immutable Linear comments are reproduced below, including their finding rows verbatim. Fresh document IDs identify these disposition documents and do not replace the original review IDs.

No applicable AssuranceProfile was found under spec/. No new finding was identified in the bounded fixes. This pass inspected source, exact finding regressions and retained execution logs; it ran no Cargo command, full gate, strict matrix or baseline campaign. It creates no worktree, source change or canonical-wire change.

Original method: code-review; original review ID: SR-2026112106202500000001; original Linear comment: dd64f80a-04c5-4b63-aac7-833060029442. The code-review disposition includes its Rust lane. The specification disposition preserves the original base method; no new analysis lenses were launched.

## Findings

The summaries below retain the original finding text. Current outcomes are in Dispositions.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The new public variant CheckedMemberType::ClockReference is not added to the repo's own exhaustive matches. The `it` test binary fails to compile with E0004 at two sites, ty_of and tc_227_the_public_shape_is_closed_and_takes_one_node_id (FR-038-AC-142), so the public_interface and tc_058 tests cannot run at the PR head. On a scratch copy with the two arms added, the binary compiles. | tests/it/checked_package_v2_model_fields.rs |
| FND-002 | high | names_a_native treats any row whose `kind` is not a string as a bound value type when it has a native `target`. It runs before classify, so it never checks that `kind` resolves to filament.meaning.model.value-type/v1. Measured: an entity object type Gadget that carries a target of ix://filament/native/Integer becomes an alias of Integer. On main a field typed Gadget is Reference(Gadget); on the PR it is Integer, and Gadget's fields are dropped. Rows with a kind that names an undeclared meaning, or with kind 42, were refused malformed-declaration on main and are admitted on the PR. | crates/filament-contract-model/src/checked_package/v2/model_members.rs |
| FND-003 | low | The arm Self::ClockReference => "clock_reference" in the primitive fallback of node_id_in is unreachable, because line 1014 already handles ClockReference. | crates/filament-contract-model/src/checked_package/v2/model_members.rs |
| FND-004 | low | The Reference(Native(Clock)) => break arm in the layer rebuild of the charged derivation is redundant. The general Reference(target) arm moves current to Native(Clock) and the while-let Declared loop ends anyway, so mutant M4 is equivalent. | crates/filament-contract-model/src/checked_package/v2/model_members/derive.rs |
| FND-005 | medium | The node-id check asserts only id.is_some(). FSpec FR-154 Clock reference types and FR-829-AC-7 require the one scalar_type/clock_reference node. M5 (node_id_in gives scalar_type/boolean) survives. M7 (the same change in the uncharged node_id, line 1212) survives, because no test calls that path for ClockReference. | crates/filament-contract-model/src/checked_package/v2/model_members/tests.rs |

## Dispositions

Round 1, verified at b43165e9be3aeb6209a79c69af19f2eb36411332.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | All closed matches now carry ClockReference, including Ty conversion, type-node conversion, perturbation and public closed-shape matching. The integrated conformance-cli it binary compiled; the exact closed-shape test and seven public-interface tests pass. |
| FND-002 | fixed | The native-target gate resolves the meaning and requires ValueType. Raw alias remains admitted. The Gadget object preserves its reference and fields; numeric and undeclared kinds refuse. The retained old-gate mutation fails both regression tests. |
| FND-003 | fixed | The duplicate charged primitive fallback is removed. The dedicated ClockReference derived-node arm remains; the separate plain node_id arm remains necessary. |
| FND-004 | fixed | The redundant charged rebuild special case is removed, while the bottom traversal still stops at Reference<Clock> and the leaf derivation returns ClockReference. Runtime, charged and plain tests remain green. |
| FND-005 | fixed | The test asserts the exact package, scalar_type/clock_reference path and ordinal zero for both charged and plain derivation. Separate retained Boolean-node mutations fail each assertion; restored integrated tests pass. |

## Evidence

Evidence read: integrated native-target tests 3 passed, Clock tests 2 passed, prepaid-storage test 1 passed, exact public-shape test 1 passed and public-interface tests 7 passed, all exit 0. Logs: /private/tmp/ir1108-integrated-native-target.log, /private/tmp/ir1108-integrated-clock.log, /private/tmp/ir1108-integrated-budget.log, /private/tmp/ir1108-integrated-public-shape.log and /private/tmp/ir1108-integrated-public-interface.log. Prior scoped Clippy logs /private/tmp/ir1108-final-model-clippy.log and /private/tmp/ir1108-final-it-clippy.log finish successfully. Production files are unchanged from the prior frozen integration head 914ddd639113b976054c7ca6d8a475804376b6fa; the integration only renumbers this PR's native-target criterion/traces to AC-600.

The old-native-gate mutation and independent charged/plain Boolean-node mutations each fail the intended test in /private/tmp/ir1108-old-native-gate-red.log, /private/tmp/ir1108-charged-boolean-red.log and /private/tmp/ir1108-plain-boolean-red.log. Restored final-base tests pass. Scoped specification validation passed with four pre-existing grammar warnings outside the changed acceptance criteria. git diff --check passed.

Plan completion: not assessed. Repository-wide matrix completeness and aggregate gates were not reassessed in this disposition round. PASS means all original findings have a closed disposition, not a new whole-repository audit.

## Machine-readable dispositions

```yaml
{
  "original_review": {
    "id": "SR-2026112106202500000001",
    "comment_id": "dd64f80a-04c5-4b63-aac7-833060029442",
    "method": "code-review",
    "head": "289381ccb9da2e157caf3a7466b987c3ac1641fc"
  },
  "disposition_review": {
    "id": "SR-20261121101006000003",
    "round": 1,
    "head": "b43165e9be3aeb6209a79c69af19f2eb36411332",
    "base": "25637dae5fd7f829944a3873678750b0099c57ff",
    "verdict": "PASS",
    "model": "gpt-6.2-astra",
    "run": "52ce559c-05d1-4ace-a8a1-1feb69753663"
  },
  "scope": [
    {
      "id": "element_type_runtime_inner",
      "path": "crates/filament-contract-model/src/checked_package/v2/model_members.rs",
      "role": "examined",
      "excerpt": "SelectedTypeDefinition::Reference(SelectedTypeRef::Native(NativeTypeRef::Clock)) => { clock_reference = true; break; }"
    },
    {
      "id": "DomainModel::bottom/leaf/rebuild",
      "path": "crates/filament-contract-model/src/checked_package/v2/model_members/derive.rs",
      "role": "examined",
      "excerpt": "// `Reference<Clock>` is a data leaf, never the clock itself. SelectedTypeDefinition::Reference(SelectedTypeRef::Native(NativeTypeRef::Clock)) => break,"
    },
    {
      "id": "MemberType::node_id_in",
      "path": "crates/filament-contract-model/src/checked_package/v2/model_members.rs",
      "role": "examined",
      "excerpt": "Self::ClockReference => derived(package, \"scalar_type\", \"clock_reference\", [], budget)?,"
    },
    {
      "id": "names_a_native",
      "path": "crates/filament-contract-model/src/checked_package/v2/model_members.rs",
      "role": "examined",
      "excerpt": "fn names_a_native(row: View<'_>) -> bool {\n    match row.get(\"kind\") {\n        Some(kind) if kind.as_str() == Some(\"alias\") => true,\n        Some(kind) if kind.as_str().is_none() => row\n            .get(\"target\")\n            .and_then(View::as_str)\n            .is_some_and(|target| target.starts_with(NATIVE_PREFIX)),\n        _ => false,\n    }\n}"
    },
    {
      "id": "CheckedMemberType::ClockReference",
      "path": "crates/filament-contract-model/src/checked_package/v2/model_fields.rs",
      "role": "examined",
      "excerpt": "/// `Reference<Clock>`, a data value naming one clock place.\nClockReference,"
    },
    {
      "id": "FR-154-AC-53",
      "path": "FSpec spec/functional/type-model/FR-154-admit-domain-package-model.md",
      "role": "context_only",
      "excerpt": "A domain package `acme/rpc` whose object type `Call` has clock fields `x` and `y` and a field `r` typed `ix://acme/rpc/ClockRef`, a type definition of `kind` `reference` with `target` `ix://filament/native/Clock`, is admitted with one declaration for `ClockRef`, and `r` is a data member of type `Reference<Clock>` with no clock member shape: `r` with presence `optional` is `Option<Reference<Clock>>` and with multiplicity `{0, 3, true, false}` is `Sequence<Reference<Clock>>[0, 3]`; the same holds with `target` an alias of `Clock` or a value type bound to it by `target`. The field typed `ix://fil"
    },
    {
      "id": "timed_model clock place detection",
      "path": "crates/filament-contract-model/src/checked_package/v2/timed_model.rs",
      "role": "context_only",
      "excerpt": "matches!(field.slot.type_ref, Some(SelectedTypeRef::Native(NativeTypeRef::Clock)))"
    }
  ],
  "original_findings": [
    {
      "fnd": "FND-001",
      "severity": "high",
      "confidence": "high",
      "check": "code-bug",
      "artifact_id": "tests/it/checked_package_v2_model_fields.rs",
      "related_ids": [
        "FR-038-AC-142",
        "CheckedMemberType::ClockReference"
      ],
      "path": "tests/it/checked_package_v2_model_fields.rs",
      "lines": "978, 6190",
      "excerpt": "fn ty_of(member: &CheckedMemberType) -> Ty {\n    match member {\n        ...\n        CheckedMemberType::Clock => Ty::Clock,\n        CheckedMemberType::IntRange { lower, upper } => Ty::Range(*lower, *upper),",
      "finding": "The new public variant CheckedMemberType::ClockReference is not added to the repo's own exhaustive matches. The `it` test binary fails to compile with E0004 at two sites, ty_of and tc_227_the_public_shape_is_closed_and_takes_one_node_id (FR-038-AC-142), so the public_interface and tc_058 tests cannot run at the PR head. On a scratch copy with the two arms added, the binary compiles.",
      "method": "code-review"
    },
    {
      "fnd": "FND-002",
      "severity": "high",
      "confidence": "high",
      "check": "code-bug",
      "artifact_id": "names_a_native",
      "related_ids": [
        "FR-154"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/model_members.rs",
      "lines": "841-850, 4791",
      "excerpt": "fn names_a_native(row: View<'_>) -> bool {\n    match row.get(\"kind\") {\n        Some(kind) if kind.as_str() == Some(\"alias\") => true,\n        Some(kind) if kind.as_str().is_none() => row\n            .get(\"target\")\n            .and_then(View::as_str)\n            .is_some_and(|target| target.starts_with(NATIVE_PREFIX)),\n        _ => false,\n    }\n}",
      "finding": "names_a_native treats any row whose `kind` is not a string as a bound value type when it has a native `target`. It runs before classify, so it never checks that `kind` resolves to filament.meaning.model.value-type/v1. Measured: an entity object type Gadget that carries a target of ix://filament/native/Integer becomes an alias of Integer. On main a field typed Gadget is Reference(Gadget); on the PR it is Integer, and Gadget's fields are dropped. Rows with a kind that names an undeclared meaning, or with kind 42, were refused malformed-declaration on main and are admitted on the PR.",
      "method": "code-review"
    },
    {
      "fnd": "FND-003",
      "severity": "low",
      "confidence": "high",
      "check": "other",
      "artifact_id": "MemberType::node_id_in",
      "related_ids": [],
      "path": "crates/filament-contract-model/src/checked_package/v2/model_members.rs",
      "lines": "1072",
      "excerpt": "Self::JsonObject => \"json_object\",\n                    Self::ClockReference => \"clock_reference\",\n                    _ => return Ok(None),",
      "finding": "The arm Self::ClockReference => \"clock_reference\" in the primitive fallback of node_id_in is unreachable, because line 1014 already handles ClockReference.",
      "method": "code-review"
    },
    {
      "fnd": "FND-004",
      "severity": "low",
      "confidence": "high",
      "check": "other",
      "artifact_id": "DomainModel derived member type layer rebuild",
      "related_ids": [],
      "path": "crates/filament-contract-model/src/checked_package/v2/model_members/derive.rs",
      "lines": "309-311",
      "excerpt": "SelectedTypeDefinition::Reference(SelectedTypeRef::Native(\n    NativeTypeRef::Clock,\n)) => break,",
      "finding": "The Reference(Native(Clock)) => break arm in the layer rebuild of the charged derivation is redundant. The general Reference(target) arm moves current to Native(Clock) and the while-let Declared loop ends anyway, so mutant M4 is equivalent.",
      "method": "code-review"
    },
    {
      "fnd": "FND-005",
      "severity": "medium",
      "confidence": "high",
      "check": "test-intent",
      "artifact_id": "tc_234_a_reference_to_clock_is_a_data_member",
      "related_ids": [
        "FR-829-AC-7",
        "FR-154-AC-53"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/model_members/tests.rs",
      "lines": "2214-2222",
      "excerpt": "let id = MemberType::ClockReference\n    .node_id_in(\n        &model.checked_package,\n        &mut Budget::new(&mut meter, 0, BYTES),\n    )\n    .expect(\"no limit is reached\");\nassert!(id.is_some(), \"the one clock_reference scalar node\");",
      "finding": "The node-id check asserts only id.is_some(). FSpec FR-154 Clock reference types and FR-829-AC-7 require the one scalar_type/clock_reference node. M5 (node_id_in gives scalar_type/boolean) survives. M7 (the same change in the uncharged node_id, line 1212) survives, because no test calls that path for ClockReference.",
      "method": "code-review"
    }
  ],
  "dispositions": [
    {
      "fnd": "FND-001",
      "outcome": "fixed",
      "reason": "All closed matches now carry ClockReference, including Ty conversion, type-node conversion, perturbation and public closed-shape matching. The integrated conformance-cli it binary compiled; the exact closed-shape test and seven public-interface tests pass.",
      "after_excerpt": "        CheckedMemberType::Clock => Ty::Clock,\n        CheckedMemberType::ClockReference => Ty::ClockReference,\n        CheckedMemberType::IntRange { lower, upper } => Ty::Range(*lower, *upper),"
    },
    {
      "fnd": "FND-002",
      "outcome": "fixed",
      "reason": "The native-target gate resolves the meaning and requires ValueType. Raw alias remains admitted. The Gadget object preserves its reference and fields; numeric and undeclared kinds refuse. The retained old-gate mutation fails both regression tests.",
      "after_excerpt": "    let meaning = kind\n        .get(\"module\")\n        .and_then(View::as_str)\n        .zip(kind.get(\"name\").and_then(View::as_str))\n        .and_then(|key| meanings.get(&key).copied())\n        .and_then(meaning::Kind::from_wire);\n    meaning == Some(meaning::Kind::ValueType)\n        && row\n            .get(\"target\")\n            .and_then(View::as_str)\n            .is_some_and(|target| target.starts_with(NATIVE_PREFIX))"
    },
    {
      "fnd": "FND-003",
      "outcome": "fixed",
      "reason": "The duplicate charged primitive fallback is removed. The dedicated ClockReference derived-node arm remains; the separate plain node_id arm remains necessary.",
      "after_excerpt": "                    Self::Timestamp => \"timestamp\",\n                    Self::Bytes => \"bytes\",\n                    Self::JsonObject => \"json_object\",\n                    _ => return Ok(None),"
    },
    {
      "fnd": "FND-004",
      "outcome": "fixed",
      "reason": "The redundant charged rebuild special case is removed, while the bottom traversal still stops at Reference<Clock> and the leaf derivation returns ClockReference. Runtime, charged and plain tests remain green.",
      "after_excerpt": "                SelectedTypeDefinition::Reference(target) => {\n                    current = target;\n                    continue;\n                }\n                SelectedTypeDefinition::Map(_) => return Ok(None),"
    },
    {
      "fnd": "FND-005",
      "outcome": "fixed",
      "reason": "The test asserts the exact package, scalar_type/clock_reference path and ordinal zero for both charged and plain derivation. Separate retained Boolean-node mutations fail each assertion; restored integrated tests pass.",
      "after_excerpt": "    let expected = Some(CheckedNodeId {\n        package: model.checked_package.clone(),\n        path: vec![\n            CheckedNodePathSegment::Name(\"scalar_type\".into()),\n            CheckedNodePathSegment::Name(\"clock_reference\".into()),\n        ],\n        ordinal: 0,\n    });\n    assert_eq!(id, expected, \"charged clock-reference identity\");\n    assert_eq!(\n        MemberType::ClockReference.node_id(&model.checked_package),\n        expected,\n        \"plain clock-reference identity\"\n    );"
    }
  ],
  "bindings": []
}
```

## Original review comment (verbatim)

Source: Linear QSL-2178 comment dd64f80a-04c5-4b63-aac7-833060029442, original SR-2026112106202500000001, original method code-review, original head 289381ccb9da2e157caf3a7466b987c3ac1641fc. The fenced content is immutable source material; original findings and metadata are preserved exactly.

````markdown
<!-- reviewer repo=agent-ix/filament-contract-ir visibility=public quoin=0.29.1 module=spec-artifacts-process@b82e42e893612b4b4ee94127714b4577ee893502 id=SR-2026112106202500000001 method=code-review lang=rust pr=filament-contract-ir#1108 model=claude-opus-5-5 run=707c7059-9da6-4142-a493-1be8aa68c420 -->

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | high | code-bug | The new public variant CheckedMemberType::ClockReference is not added to the repo's own exhaustive matches |
| FND-002 | high | code-bug | names_a_native treats any row whose `kind` is not a string as a bound value type when it has a native `target` |
| FND-003 | low | other | The arm Self::ClockReference => "clock_reference" in the primitive fallback of node_id_in is unreachable, because line 1014 already handles ClockReference. |
| FND-004 | low | other | The Reference(Native(Clock)) => break arm in the layer rebuild of the charged derivation is redundant |
| FND-005 | medium | test-intent | The node-id check asserts only id.is_some() |

+++ [reviewer data]

```yaml
clean: false
scope:
- id: element_type_runtime_inner
  path: crates/filament-contract-model/src/checked_package/v2/model_members.rs
  role: examined
  excerpt: SelectedTypeDefinition::Reference(SelectedTypeRef::Native(NativeTypeRef::Clock)) => { clock_reference = true; break; }
- id: DomainModel::bottom/leaf/rebuild
  path: crates/filament-contract-model/src/checked_package/v2/model_members/derive.rs
  role: examined
  excerpt: // `Reference<Clock>` is a data leaf, never the clock itself. SelectedTypeDefinition::Reference(SelectedTypeRef::Native(NativeTypeRef::Clock)) => break,
- id: MemberType::node_id_in
  path: crates/filament-contract-model/src/checked_package/v2/model_members.rs
  role: examined
  excerpt: Self::ClockReference => derived(package, "scalar_type", "clock_reference", [], budget)?,
- id: names_a_native
  path: crates/filament-contract-model/src/checked_package/v2/model_members.rs
  role: examined
  excerpt: "fn names_a_native(row: View<'_>) -> bool {\n    match row.get(\"kind\") {\n        Some(kind) if kind.as_str() == Some(\"alias\") => true,\n        Some(kind) if kind.as_str().is_none() => row\n            .get(\"target\")\n            .and_then(View::as_str)\n            .is_some_and(|target| target.starts_with(NATIVE_PREFIX)),\n        _ => false,\n    }\n}"
- id: CheckedMemberType::ClockReference
  path: crates/filament-contract-model/src/checked_package/v2/model_fields.rs
  role: examined
  excerpt: '/// `Reference<Clock>`, a data value naming one clock place.

    ClockReference,'
- id: FR-154-AC-53
  path: FSpec spec/functional/type-model/FR-154-admit-domain-package-model.md
  role: context_only
  excerpt: 'A domain package `acme/rpc` whose object type `Call` has clock fields `x` and `y` and a field `r` typed `ix://acme/rpc/ClockRef`, a type definition of `kind` `reference` with `target` `ix://filament/native/Clock`, is admitted with one declaration for `ClockRef`, and `r` is a data member of type `Reference<Clock>` with no clock member shape: `r` with presence `optional` is `Option<Reference<Clock>>` and with multiplicity `{0, 3, true, false}` is `Sequence<Reference<Clock>>[0, 3]`; the same holds with `target` an alias of `Clock` or a value type bound to it by `target`. The field typed `ix://fil'
- id: timed_model clock place detection
  path: crates/filament-contract-model/src/checked_package/v2/timed_model.rs
  role: context_only
  excerpt: matches!(field.slot.type_ref, Some(SelectedTypeRef::Native(NativeTypeRef::Clock)))
findings:
- fnd: FND-001
  severity: high
  confidence: high
  check: code-bug
  artifact_id: tests/it/checked_package_v2_model_fields.rs
  related_ids:
  - FR-038-AC-142
  - CheckedMemberType::ClockReference
  path: tests/it/checked_package_v2_model_fields.rs
  lines: 978, 6190
  excerpt: "fn ty_of(member: &CheckedMemberType) -> Ty {\n    match member {\n        ...\n        CheckedMemberType::Clock => Ty::Clock,\n        CheckedMemberType::IntRange { lower, upper } => Ty::Range(*lower, *upper),"
  finding: The new public variant CheckedMemberType::ClockReference is not added to the repo's own exhaustive matches. The `it` test binary fails to compile with E0004 at two sites, ty_of and tc_227_the_public_shape_is_closed_and_takes_one_node_id (FR-038-AC-142), so the public_interface and tc_058 tests cannot run at the PR head. On a scratch copy with the two arms added, the binary compiles.
  method: code-review
- fnd: FND-002
  severity: high
  confidence: high
  check: code-bug
  artifact_id: names_a_native
  related_ids:
  - FR-154
  path: crates/filament-contract-model/src/checked_package/v2/model_members.rs
  lines: 841-850, 4791
  excerpt: "fn names_a_native(row: View<'_>) -> bool {\n    match row.get(\"kind\") {\n        Some(kind) if kind.as_str() == Some(\"alias\") => true,\n        Some(kind) if kind.as_str().is_none() => row\n            .get(\"target\")\n            .and_then(View::as_str)\n            .is_some_and(|target| target.starts_with(NATIVE_PREFIX)),\n        _ => false,\n    }\n}"
  finding: 'names_a_native treats any row whose `kind` is not a string as a bound value type when it has a native `target`. It runs before classify, so it never checks that `kind` resolves to filament.meaning.model.value-type/v1. Measured: an entity object type Gadget that carries a target of ix://filament/native/Integer becomes an alias of Integer. On main a field typed Gadget is Reference(Gadget); on the PR it is Integer, and Gadget''s fields are dropped. Rows with a kind that names an undeclared meaning, or with kind 42, were refused malformed-declaration on main and are admitted on the PR.'
  method: code-review
- fnd: FND-003
  severity: low
  confidence: high
  check: other
  artifact_id: MemberType::node_id_in
  related_ids: []
  path: crates/filament-contract-model/src/checked_package/v2/model_members.rs
  lines: '1072'
  excerpt: "Self::JsonObject => \"json_object\",\n                    Self::ClockReference => \"clock_reference\",\n                    _ => return Ok(None),"
  finding: The arm Self::ClockReference => "clock_reference" in the primitive fallback of node_id_in is unreachable, because line 1014 already handles ClockReference.
  method: code-review
- fnd: FND-004
  severity: low
  confidence: high
  check: other
  artifact_id: DomainModel derived member type layer rebuild
  related_ids: []
  path: crates/filament-contract-model/src/checked_package/v2/model_members/derive.rs
  lines: 309-311
  excerpt: "SelectedTypeDefinition::Reference(SelectedTypeRef::Native(\n    NativeTypeRef::Clock,\n)) => break,"
  finding: The Reference(Native(Clock)) => break arm in the layer rebuild of the charged derivation is redundant. The general Reference(target) arm moves current to Native(Clock) and the while-let Declared loop ends anyway, so mutant M4 is equivalent.
  method: code-review
- fnd: FND-005
  severity: medium
  confidence: high
  check: test-intent
  artifact_id: tc_234_a_reference_to_clock_is_a_data_member
  related_ids:
  - FR-829-AC-7
  - FR-154-AC-53
  path: crates/filament-contract-model/src/checked_package/v2/model_members/tests.rs
  lines: 2214-2222
  excerpt: "let id = MemberType::ClockReference\n    .node_id_in(\n        &model.checked_package,\n        &mut Budget::new(&mut meter, 0, BYTES),\n    )\n    .expect(\"no limit is reached\");\nassert!(id.is_some(), \"the one clock_reference scalar node\");"
  finding: The node-id check asserts only id.is_some(). FSpec FR-154 Clock reference types and FR-829-AC-7 require the one scalar_type/clock_reference node. M5 (node_id_in gives scalar_type/boolean) survives. M7 (the same change in the uncharged node_id, line 1212) survives, because no test calls that path for ClockReference.
  method: code-review
bindings: []
```

+++
````

