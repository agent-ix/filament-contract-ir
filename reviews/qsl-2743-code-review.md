---
id: SR-967175026302197768900474181941935084514
title: "QSL-2743 PR 1117 code-review (Rust lane) of the model-union reader"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; QSL-2743; PR 1117 head 31a74a3c6d48f8f272f82092cd28ba9e377d2516; crates/filament-contract-model/src/checked_package/v2/{exact_projection.rs,exact_projection/*,native_join_storage.rs,runtime_type_projection.rs,model_members.rs,monitor_plan/*}, tests/conformance_fspec/model_union_witness.rs, tests/conformance_fspec/main.rs, tests/conformance_fsl_ir1384/tests/receipt.rs, crates/filament-contract-model/tests/monitor_plan_public.rs, spec rows"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-352", type: references }
  - { target: "ix://agent-ix/filament-contract-ir/FR-358", type: references }
  - { target: "ix://agent-ix/filament-contract-ir/FR-069", type: references }
---

## Summary

Code review of PR 1117 (agent-ix/filament-contract-ir), the FR-830 model-union reader, diff origin/main...31a74a3. Ticket: QSL-2743. The core join is sound, but refusal loci, budget vectors, retention, row-index loci, payload coverage and storage precharge fall short of the FR-830 contract, and the tests cannot detect several of these gaps.

## Verdict

**FAIL** — two high findings (FND-001 still open after a partial fix, FND-002) plus medium findings on payload forms, precharge, loci and coverage. MERGEABLE=FALSE.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Refusal loci do not match the published vectors and no test checks them. The refusal test compares only expected.code and ignores expected.locus, while FR-830 AC-7/8/9/13/14 and FR-352-AC-31 require code and locus. Wrong pointers as found: model_union.rs:672 emits /member (MUW-R-17/18 expect /member/declaration and /member/identifier); 860/876/896/918/940/962 emit a bare /payload/node with no /model_unions/{k}/cases/{j} prefix (R-43/46/51/54/55, TC-02/03/05); 497 (R-01) and every tuple refusal from with_admitted_type_rows (TR-01..07) carry form selected_identity, not a /model_unions or /model_tuples/{k} pointer. A mutant that swaps /rank and /position passes. | tests/conformance_fspec/model_union_witness.rs:393-401 (e8ba885f7); crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:497,672,860-962 (e8ba885f7) |
| FND-002 | high | The published budget_cases MUW-B-01..B-08 are never executed. The budget test asserts only outcome codes for baseline, one-below and zero limits on three positives; it never asserts the first unpaid kind, full attempted amount or locus, and omits B-02 (closure-edge boundary Ec-1/Ec), B-03 (cumulative across joins), B-04/B-06/B-07 (charges kept after a refusal, no rows), B-05 (tuple/union channel split at Wt) and B-08 (reached with an unreached row: surplus only common work). FR-830-AC-11 and FR-358-AC-15 are largely unverified. | tests/conformance_fspec/model_union_witness.rs:426-462 (e8ba885f7) |
| FND-003 | medium | The positive test only checks each returned row is contained in the input. A change that retains out-of-closure rows under reached scope (MUW-03, MUW-16) or returns extra rows (MUW-19 returns only the model row) passes; dropping all retention passes for every reached vector. The published closure_edges counts (AC-3: each node expanded once) are never asserted. Only the MUW-01 full-scope projection test asserts an exact count. | tests/conformance_fspec/model_union_witness.rs:330-336 (e8ba885f7) |
| FND-004 | medium | Native denials carry no sidecar row-index locus. The union phase never sets a locus source; in the reached monitor path with_admitted_type_rows sets TupleRow(index) over the scope-filtered joined_tuples, not the original sidecar index, and that source stays active, so a later union-phase denial is attributed to the last tuple row. FR-830 Work and storage requires the denial's compact locus to name the channel (model_tuples vs model_unions), the sidecar row index and the form. | crates/filament-contract-model/src/checked_package/v2/exact_projection/complete_witness.rs:85; crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:456,487-890 |
| FND-005 | medium | Admitted non-Clock payload forms are refused. model_union_payload_node returns None for Option, Collection, IntRange, RationalRange, TextBounds, CarriedRational and ClockReference, and model_union_expected_type refuses an enum-family variant payload; all become UnsupportedExactType. FR-830 says a data payload is any admitted type but Clock and the original value-type grammar includes option, collection, map, bounded and carried-quantity forms. A full join over a package with a Rational-range payload (TK-36, FR-069-AC-8) cannot succeed. No vector covers these forms; that does not narrow the contract. | crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:247-263, 871-877 (e8ba885f7 numbering 247-263, 870-878) |
| FND-006 | medium | The monitor and runtime threading has no test with real rows. Every ReachedProducerRows and SelectedMonitorClosureRows site passes model_unions: &[] (reached_model_read.rs tests, monitor_plan_public.rs:70, receipt.rs:284), so with_model_union_rows_over and SelectedTypeDeclarations::unions() never run with a row. FR-352-AC-31 also claims a record or object field of an admitted union returns Composite(type_handle) (exact_projection.rs:3429); no test exercises it, nor a tuple position or optional field of union type. | crates/filament-contract-model/src/checked_package/v2/monitor_plan/reached_model_read.rs:708-719; crates/filament-contract-model/src/checked_package/v2/runtime_type_projection.rs:1211-1220; crates/filament-contract-model/src/checked_package/v2/exact_projection.rs:3429-3437 |
| FND-007 | medium | The preflight handle index covers only rows already retained by the receiver, not the supplied rows. FR-830 Work and storage puts the handles of every supplied tuple and union witness, across both channels, in the preflight index of the common pass. Under reached scope a joined row whose type_handle or settled_key collides with an unreached supplied row's handle is not detected, and the B-08 claim that the surplus is preflight-index work is not what the code does. | crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:281-345 |
| FND-008 | low | Misleading comment: it says the fixture-C source-row non-substitution controls are asserted by the native tests of the crate, but no such native tests exist in the diff or the crate. | tests/conformance_fspec/model_union_witness.rs:356-357 (e8ba885f7) |
| FND-009 | low | One clone is charged twice and one check is duplicated. selected_identity_handles charges storage::selected_clone() at 405-411 while with_admitted_type_rows charges it again for the only clone (complete_witness.rs:73-80). The tuple-vs-union key check at 457-469 duplicates the model_unions comparison check_handle already makes (complete_witness.rs:302-305) and uses the unbudgeted witness_error with form selected_identity. | crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:405-411, 457-469 |
| FND-010 | low | validate_and_rebind ignores model unions. validate_contents does not re-check model_unions rows and validate_completeness requires no row for a union-family variant_type node, while it does require admitted_tuples rows. FR-352-AC-28 says missing emitted declarations or unsupported identity channels refuse rather than yield a partial complete witness, and FR-830 full scope requires one row per union-family node; a rebound witness can be complete without the rows direct projection needs. Matching the older model-enum omission is not contract scope. | crates/filament-contract-model/src/checked_package/v2/exact_projection/complete_witness.rs:652-740; 409 |
| FND-011 | low | A shared helper changed behaviour for every caller. selected_declaration_error and its budgeted twin now return UnknownNode instead of ForeignNode for any node whose package equals a selected model document identity. This affects all joins and projections that use the helper, not just the model-union join, and only the FR-830 vectors exercise it. | crates/filament-contract-model/src/checked_package/v2/exact_projection.rs:2976-2990, 2999 |
| FND-012 | low | Spec-row wording. FR-352-AC-31 says a foreign selection refuses UnknownNode, which is ambiguous beside FR-352-AC-30 (identities from an independent admission refuse ForeignNode). FR-358-AC-15 says every row is charged in the common pass before allocation, but FR-830 and the code allocate the preflight handle index first. None of the three new rows carried the status prefix sibling rows use. | spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md:627 (e8ba885f7) |
| FND-013 | medium | The reached closure clones before it pays. The start node is cloned twice (pending and visited) under a flat charge of 3 storage units, not its node size; each new target is cloned twice (visited.insert and pending.push) under one node_units+2 charge; the per-node edges and members Vecs grow with no storage charge. A long start or target CheckedNodeId under a tight storage limit allocates beyond what the budget admitted, violating precharge-before-clone. | crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:150-157, 165-166, 221-233 |
| FND-014 | medium | Staged and index storage is under-charged. staged.insert clones the structural node as the map key, but row_units (native_join_storage::model_union) counts the structural node once, for the row copy; the tuple path charges the key separately (complete_witness.rs:121-124, storage::node(id)). The handle-set inserts at 470 and 775-776 and the joined_handles BTreeMap at 472-478 allocate with no storage charge. | crates/filament-contract-model/src/checked_package/v2/native_join_storage.rs:605-612; crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:470,477,774-777 |
| FND-015 | medium | Uncharged scans. The coverage loop walks every selected declaration and calls variant_family (which scans the variant's cases) before any work charge, charging only matching tuple and union nodes; the tuple-vs-prior-union check at 457-469 scans every retained union for every joined tuple with no charge (T x U). These are whole-model and quadratic blind spots in the shared work budget. | crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:457-469, 780-787 |
| FND-016 | medium | The typed source non-substitution controls are skipped as if producer-only. MUW-19, MUW-20 and MUW-R-59..R-62 need only typed SelectedSourceUnionRow and SelectedSourceTupleRow rows and the existing native source joins, which IR already owns; the FSL parser boundary covers the closed format only. Nothing shows a valid source_unions or source_tuples row never supplies a missing model row, or that MUW-20 admits beside a source tuple. | tests/conformance_fspec/model_union_witness.rs:377-381 |
| FND-017 | medium | The -TR- locus exemption drops required coverage. IR owns the typed model_tuples join and knows each row's sidecar index before filtering, so /model_tuples/{k}/... is IR's coordinate to report, not an FSL adapter pointer. The substring match also exempts MUW-TR-08, whose expected locus is the union pointer /model_unions/1/type_handle. Coverage vectors' expected.node is never compared with error.at. | tests/conformance_fspec/model_union_witness.rs:420-428 |
| FND-018 | low | FR-019 inventory omits part of the public change: the new public field SelectedMonitorClosureRows.model_unions is a breaking change for struct-literal consumers (receipt.rs and monitor_plan_public.rs had to change) but is not in the breaking-change prose, and the associated with_model_union_rows join, model_union_rows/model_union_row accessors and SelectedTypeDeclarations::unions() are not named. | spec/model/functional/FR-019-rust-library-interface.md:311, 316, 318 |

## Scope and evidence

Rust lane (rust-review, rust-style baseline) folded into this artifact. FND-001 to FND-012 are the original review findings as found at e8ba885f7 (their lines and excerpts cite e8ba885f7). FND-013 to FND-018 were added in this formal pass at frozen head 31a74a3, covering the lead verification items: closure precharge, staged and index storage, uncharged scans, typed source controls, the -TR- exemption and the public inventory. Round-1 dispositions cover only the partial fixes in 31a74a3: locus (FND-001), the misleading comment (FND-008) and spec wording (FND-012). Every other finding is open.

Held clean (examined, no finding): atomic tuple-first join and single copy-on-write publish (model_union.rs:456, 824-846); prior-Arc copy charged before Arc::make_mut; union rows on source-union, enum-family or tuple nodes refuse Mismatched; union projection branches precede the enum-shape lookup (exact_projection.rs:3429, 4284); type_handle and settled_key kept in separate roles; canonical rank and authored position checked apart; UnionMember never an enum VariantId; nullary, Clock and data payloads never substitute; the closure keeps a visited set and expands each node once; the FR-531-AC-13 authority wording is verbatim at model_union.rs:5-6 and 357-358; the model-enum test supersession only replaces the old unsupported refusal with the missing-witness refusal, and an enum-only mask is still refused; source-union, source-tuple and dimensional APIs are unchanged.

Gates: no cargo was run by this reviewer (host thermal rule; no aggregate, workspace or clippy gates). Coder receipts read, not re-executed: /tmp/ir2743-t4.log (conformance_fspec model_union_witness, 4 passed) and /tmp/ir2743-t5.log (exact_projection, 16 passed). t4 is timestamped 06:51 and head 31a74a3 was committed 06:55, so the receipt may predate the final commit by one edit.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | Partial fix at 31a74a3: member/identifier vs member/declaration, the /model_unions/{k}/cases/{j} payload prefix and the R-01 structural_node pointer are corrected, and the refusal test now compares error.form with expected.locus. Still missing: every -TR- vector is exempted from the locus check, so tuple refusals keep the tuple-stage form selected_identity (TR-01..07) and TR-08 is unasserted; coverage vectors' expected.node is not compared with error.at. |
| FND-008 | fixed | Fixed at 31a74a3; after_excerpt in the round-1 tracker comment. |
| FND-012 | still-open | Partial fix at 31a74a3: the three rows now carry the Implemented (QSL-2743) status prefix. Still unchanged: FR-352-AC-31 still says a foreign selection refuses UnknownNode beside FR-352-AC-30's ForeignNode, and FR-358-AC-15 still says the common pass is charged before allocation although the preflight handle index is allocated first. |

## Machine-readable data

```yaml
{
  "clean": false,
  "scope": [
    {
      "id": "FR-352-AC-31",
      "path": "spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md",
      "role": "examined",
      "excerpt": "Implemented (QSL-2743). Joining the selected model tuple and model union witness rows (FSpec FR-830) returns the complete rows keyed by structural node, retaining each type handle, settled key, case rank, position, name, original case, member and payload as the producer's opaque identities. Tuple rows are admitted first, then union identity, case census, per-case checks, payload original type against the side-input witness table, and coverage: a full scope refuses an uncovered union with `IncompleteIdentityWitness` and a reached scope with `UnavailableIdentityWitness`. Direct projection and a "
    },
    {
      "id": "FR-358-AC-15",
      "path": "spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md",
      "role": "examined",
      "excerpt": "Implemented (QSL-2743). The model tuple and model union row join charges one shared `NativeJoinBudget`: every row of both channels is charged in the common pass before allocation, then the selected clone and the prior-copy publish. The measured exact cumulative work and storage succeed and one unit below either refuses at the first unpaid kind and locus with no partial publish; a zero budget refuses before any row is retained."
    },
    {
      "id": "FR-069-AC-11",
      "path": "spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md",
      "role": "examined",
      "excerpt": "Implemented (QSL-2743). A model union variant payload naming an object type of the selected package is admitted as a `Reference` payload, so a union such as `Mix.Held(Holder)` passes intake instead of being refused for an undeclared payload type."
    },
    {
      "id": "FR-352-AC-28",
      "path": "spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md",
      "role": "context_only",
      "excerpt": "Implemented (IR-1360). A default-feature external consumer calls the checked public `with_admitted_type_rows` and single `validate_and_rebind` signatures with `&mut NativeJoinBudget`. The selected-type declaration projection uses the same checked admitted-type join; the old unchecked copy path is absent. The public surface exposes no private-map getter, unchecked rebind, completeness flag, wrapper or second validator. Full finite package-local checks use native admitted facts, terminate iteratively, perform no source parse, and acquire no FSL/runtime dependency or pin. Missing emitted declarat"
    },
    {
      "id": "FR-352-AC-30",
      "path": "spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md",
      "role": "context_only",
      "excerpt": "Direct projection of an admitted selected `model/variant_type` with its complete installed model enum shape returns `ValueType::Enum` equal to that shape, retaining ordering and every opaque variant identity. A package clone agrees. The reached-model join and following projection share one `NativeJoinBudget`: their measured exact cumulative work and storage succeed, and one unit below either refuses at the first unpaid kind and locus without a partial projection. An absent shape returns `UnavailableIdentityWitness`, and identities installed for an independent admission refuse `ForeignNode`. A "
    },
    {
      "id": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs",
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs",
      "role": "examined",
      "excerpt": "Atomic tuple-first join: tuples via with_admitted_type_rows, union staging, coverage, single COW publish; type_handle/settled_key roles; rank vs position; UnionMember not VariantId; Clock vs nullary; cycle-aware closure."
    },
    {
      "id": "crates/filament-contract-model/src/checked_package/v2/exact_projection.rs",
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection.rs",
      "role": "examined",
      "excerpt": "Union branches of direct projection (4284) and record-field projection (3429) ahead of the enum-shape lookup; selected_declaration_error classification."
    },
    {
      "id": "crates/filament-contract-model/src/checked_package/v2/exact_projection/complete_witness.rs",
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection/complete_witness.rs",
      "role": "examined",
      "excerpt": "check_handle and check_admitted_handle cover model_unions in both directions; with_admitted_type_rows locus source; validate_and_rebind."
    },
    {
      "id": "crates/filament-contract-model/src/checked_package/v2/native_join_storage.rs",
      "path": "crates/filament-contract-model/src/checked_package/v2/native_join_storage.rs",
      "role": "examined",
      "excerpt": "storage::model_union per-row units."
    },
    {
      "id": "crates/filament-contract-model/src/checked_package/v2/monitor_plan/reached_model_read.rs",
      "path": "crates/filament-contract-model/src/checked_package/v2/monitor_plan/reached_model_read.rs",
      "role": "examined",
      "excerpt": "with_model_union_rows_over threaded after the enum-shape join under one budget."
    },
    {
      "id": "crates/filament-contract-model/src/checked_package/v2/runtime_type_projection.rs",
      "path": "crates/filament-contract-model/src/checked_package/v2/runtime_type_projection.rs",
      "role": "examined",
      "excerpt": "selected_union requirement and unions() output with storage precharge."
    },
    {
      "id": "crates/filament-contract-model/src/checked_package/v2/model_members.rs",
      "path": "crates/filament-contract-model/src/checked_package/v2/model_members.rs",
      "role": "examined",
      "excerpt": "scope.object_types admitted as a declared payload name (FR-322 Reference<O>)."
    },
    {
      "id": "tests/conformance_fspec/model_union_witness.rs",
      "path": "tests/conformance_fspec/model_union_witness.rs",
      "role": "examined",
      "excerpt": "Conformance harness: strict read of complete published inputs, production with_model_union_rows, positives, refusals with code and locus, budget baseline, projection."
    },
    {
      "id": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_enum_tests.rs",
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_enum_tests.rs",
      "role": "examined",
      "excerpt": "Supersession: absent union row now UnavailableIdentityWitness at selected_union; enum-only mask still refused."
    }
  ],
  "findings": [
    {
      "fnd": "FND-001",
      "severity": "high",
      "confidence": "high",
      "method": "code-review",
      "check": "test-intent",
      "artifact_id": "tests/conformance_fspec/model_union_witness.rs::published_reader_refusals_keep_their_codes",
      "related_ids": [
        "FR-352-AC-31",
        "FR-830-AC-7",
        "FR-830-AC-8",
        "FR-830-AC-9",
        "FR-830-AC-13",
        "FR-830-AC-14"
      ],
      "path": "tests/conformance_fspec/model_union_witness.rs",
      "lines": "393-401 (e8ba885f7); crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:497,672,860-962 (e8ba885f7)",
      "excerpt": "        let expected = row[\"expected\"][\"code\"].as_str().expect(\"code\");\n        if expected == \"malformed\" {\n            // The closed-format codec is the FSL identity-file parser's.\n            continue;\n        }\n        let actual = run(&vectors, id, None);\n        if actual != expected {\n            mismatches.push(format!(\"{id}: expected {expected}, got {actual}\"));\n        }",
      "finding": "Refusal loci do not match the published vectors and no test checks them. The refusal test compares only expected.code and ignores expected.locus, while FR-830 AC-7/8/9/13/14 and FR-352-AC-31 require code and locus. Wrong pointers as found: model_union.rs:672 emits /member (MUW-R-17/18 expect /member/declaration and /member/identifier); 860/876/896/918/940/962 emit a bare /payload/node with no /model_unions/{k}/cases/{j} prefix (R-43/46/51/54/55, TC-02/03/05); 497 (R-01) and every tuple refusal from with_admitted_type_rows (TR-01..07) carry form selected_identity, not a /model_unions or /model_tuples/{k} pointer. A mutant that swaps /rank and /position passes."
    },
    {
      "fnd": "FND-002",
      "severity": "high",
      "confidence": "high",
      "method": "code-review",
      "check": "test-intent",
      "artifact_id": "tests/conformance_fspec/model_union_witness.rs::the_measured_baseline_admits_and_one_unit_less_exhausts_each_counter",
      "related_ids": [
        "FR-358-AC-15",
        "FR-830-AC-11"
      ],
      "path": "tests/conformance_fspec/model_union_witness.rs",
      "lines": "426-462 (e8ba885f7)",
      "excerpt": "        assert_eq!(\n            run(&vectors, id, Some((work, storage))),\n            \"admitted\",\n            \"{id}: baseline\"\n        );\n        assert_eq!(\n            run(&vectors, id, Some((work - 1, storage))),\n            \"exhausted_work\",\n            \"{id}: one unit less work\"\n        );\n        assert_eq!(",
      "finding": "The published budget_cases MUW-B-01..B-08 are never executed. The budget test asserts only outcome codes for baseline, one-below and zero limits on three positives; it never asserts the first unpaid kind, full attempted amount or locus, and omits B-02 (closure-edge boundary Ec-1/Ec), B-03 (cumulative across joins), B-04/B-06/B-07 (charges kept after a refusal, no rows), B-05 (tuple/union channel split at Wt) and B-08 (reached with an unreached row: surplus only common work). FR-830-AC-11 and FR-358-AC-15 are largely unverified."
    },
    {
      "fnd": "FND-003",
      "severity": "medium",
      "confidence": "high",
      "method": "code-review",
      "check": "test-intent",
      "artifact_id": "tests/conformance_fspec/model_union_witness.rs::published_positive_vectors_are_admitted_with_their_rows_returned",
      "related_ids": [
        "FR-830-AC-2",
        "FR-830-AC-3",
        "FR-830-AC-10",
        "FR-352-AC-31"
      ],
      "path": "tests/conformance_fspec/model_union_witness.rs",
      "lines": "330-336 (e8ba885f7)",
      "excerpt": "        match identities.with_model_union_rows(&admitted, &tuples, &unions, scope, &mut budget) {\n            Ok(joined) => {\n                // Every joined union row is returned unchanged.\n                for row in joined.model_union_rows() {\n                    assert!(\n                        unions.contains(row),\n                        \"{id}: a returned row differs from its input\"\n                    );\n                }\n                \"admitted\".into()",
      "finding": "The positive test only checks each returned row is contained in the input. A change that retains out-of-closure rows under reached scope (MUW-03, MUW-16) or returns extra rows (MUW-19 returns only the model row) passes; dropping all retention passes for every reached vector. The published closure_edges counts (AC-3: each node expanded once) are never asserted. Only the MUW-01 full-scope projection test asserts an exact count."
    },
    {
      "fnd": "FND-004",
      "severity": "medium",
      "confidence": "high",
      "method": "code-review",
      "check": "code-bug",
      "artifact_id": "crates/filament-contract-model/src/checked_package/v2/exact_projection/complete_witness.rs::with_admitted_type_rows",
      "related_ids": [
        "FR-358-AC-15",
        "FR-830-AC-11",
        "FR-352-AC-31"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection/complete_witness.rs",
      "lines": "85; crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:456,487-890",
      "excerpt": "            for (index, row) in tuple_rows.iter().enumerate() {\n                budget.set_locus_source(NativeJoinLocusSource::TupleRow(index));\n                validate_admitted_tuple(package, row, budget)?;",
      "finding": "Native denials carry no sidecar row-index locus. The union phase never sets a locus source; in the reached monitor path with_admitted_type_rows sets TupleRow(index) over the scope-filtered joined_tuples, not the original sidecar index, and that source stays active, so a later union-phase denial is attributed to the last tuple row. FR-830 Work and storage requires the denial's compact locus to name the channel (model_tuples vs model_unions), the sidecar row index and the form."
    },
    {
      "fnd": "FND-005",
      "severity": "medium",
      "confidence": "high",
      "method": "code-review",
      "check": "code-bug",
      "artifact_id": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs::model_union_payload_node",
      "related_ids": [
        "FR-352-AC-31",
        "FR-830-AC-9",
        "FR-069-AC-8"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs",
      "lines": "247-263, 871-877 (e8ba885f7 numbering 247-263, 870-878)",
      "excerpt": "            | MemberType::Reference(id) => return Ok(Some(Cow::Borrowed(id))),\n            MemberType::Boolean => \"boolean\",\n            MemberType::Integer => \"integer\",\n            MemberType::Float32 => \"float32\",\n            MemberType::Float64 => \"float64\",\n            MemberType::String => \"string\",\n            MemberType::Uuid => \"uuid\",\n            MemberType::Timestamp => \"timestamp\",\n            MemberType::Bytes => \"bytes\",\n            MemberType::JsonObject => \"json_object\",\n            _ => return Ok(None),",
      "finding": "Admitted non-Clock payload forms are refused. model_union_payload_node returns None for Option, Collection, IntRange, RationalRange, TextBounds, CarriedRational and ClockReference, and model_union_expected_type refuses an enum-family variant payload; all become UnsupportedExactType. FR-830 says a data payload is any admitted type but Clock and the original value-type grammar includes option, collection, map, bounded and carried-quantity forms. A full join over a package with a Rational-range payload (TK-36, FR-069-AC-8) cannot succeed. No vector covers these forms; that does not narrow the contract."
    },
    {
      "fnd": "FND-006",
      "severity": "medium",
      "confidence": "high",
      "method": "code-review",
      "check": "coverage",
      "artifact_id": "crates/filament-contract-model/src/checked_package/v2/monitor_plan/reached_model_read.rs::with_model_union_rows_over call",
      "related_ids": [
        "FR-352-AC-31",
        "FR-019-AC-27"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/monitor_plan/reached_model_read.rs",
      "lines": "708-719; crates/filament-contract-model/src/checked_package/v2/runtime_type_projection.rs:1211-1220; crates/filament-contract-model/src/checked_package/v2/exact_projection.rs:3429-3437",
      "excerpt": "            let identities = identities\n                .with_model_union_rows_over(\n                    self.package,\n                    rows.tuples,\n                    rows.model_unions,\n                    &reached,\n                    budget,\n                )",
      "finding": "The monitor and runtime threading has no test with real rows. Every ReachedProducerRows and SelectedMonitorClosureRows site passes model_unions: &[] (reached_model_read.rs tests, monitor_plan_public.rs:70, receipt.rs:284), so with_model_union_rows_over and SelectedTypeDeclarations::unions() never run with a row. FR-352-AC-31 also claims a record or object field of an admitted union returns Composite(type_handle) (exact_projection.rs:3429); no test exercises it, nor a tuple position or optional field of union type."
    },
    {
      "fnd": "FND-007",
      "severity": "medium",
      "confidence": "medium",
      "method": "code-review",
      "check": "soundness",
      "artifact_id": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs::model_union_handles",
      "related_ids": [
        "FR-830-AC-2",
        "FR-830-AC-7",
        "FR-830-AC-14"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs",
      "lines": "281-345",
      "excerpt": "    fn model_union_handles(\n        &self,\n        first: &CheckedNodeId,\n        budget: &mut NativeJoinBudget,\n    ) -> Result<BTreeSet<NodeKey>, ExactTypeProjectionError> {\n        charge_native_storage(budget, first, first, \"model_union_handles\", 2)?;\n        let mut handles = BTreeSet::new();\n        macro_rules! take {\n            ($id:expr, $key:expr) => {{\n                charge_native_work(budget, first, $id, \"model_union_handle\", 1)?;\n                if !handles.contains($key) {\n                    charge_native_storage(budget, first, $id, \"model_union_handle\", 2)?;",
      "finding": "The preflight handle index covers only rows already retained by the receiver, not the supplied rows. FR-830 Work and storage puts the handles of every supplied tuple and union witness, across both channels, in the preflight index of the common pass. Under reached scope a joined row whose type_handle or settled_key collides with an unreached supplied row's handle is not detected, and the B-08 claim that the surplus is preflight-index work is not what the code does."
    },
    {
      "fnd": "FND-008",
      "severity": "low",
      "confidence": "high",
      "method": "code-review",
      "check": "other",
      "artifact_id": "tests/conformance_fspec/model_union_witness.rs::uses_source_rows",
      "related_ids": [],
      "path": "tests/conformance_fspec/model_union_witness.rs",
      "lines": "356-357 (e8ba885f7)",
      "excerpt": "/// Fixture C holds source rows this adapter does not build; the source-row\n/// non-substitution controls are asserted by the native tests of the crate.\nfn uses_source_rows(vectors: &Value, id: &str) -> bool {\n    find(vectors, id).to_string().contains(\"/source_\") || resolve_fixtures(vectors, id) == \"C\"\n}",
      "finding": "Misleading comment: it says the fixture-C source-row non-substitution controls are asserted by the native tests of the crate, but no such native tests exist in the diff or the crate."
    },
    {
      "fnd": "FND-009",
      "severity": "low",
      "confidence": "high",
      "method": "code-review",
      "check": "code-bug",
      "artifact_id": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs::join_model_unions",
      "related_ids": [
        "FR-358-AC-15"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs",
      "lines": "405-411, 457-469",
      "excerpt": "        for row in &joined_tuples {\n            // A model tuple's key must not collide with a union handle.\n            if selected\n                .model_unions\n                .values()\n                .any(|union| union.type_handle == row.node_key || union.settled_key == row.node_key)\n            {\n                return Err(witness_error(\n                    &row.structural_node,\n                    ExactTypeProjectionErrorCode::ConflictingIdentityWitness,\n                ));\n            }\n        }",
      "finding": "One clone is charged twice and one check is duplicated. selected_identity_handles charges storage::selected_clone() at 405-411 while with_admitted_type_rows charges it again for the only clone (complete_witness.rs:73-80). The tuple-vs-union key check at 457-469 duplicates the model_unions comparison check_handle already makes (complete_witness.rs:302-305) and uses the unbudgeted witness_error with form selected_identity."
    },
    {
      "fnd": "FND-010",
      "severity": "low",
      "confidence": "medium",
      "method": "code-review",
      "check": "soundness",
      "artifact_id": "crates/filament-contract-model/src/checked_package/v2/exact_projection/complete_witness.rs::validate_completeness",
      "related_ids": [
        "FR-352-AC-28",
        "FR-830-AC-10"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection/complete_witness.rs",
      "lines": "652-740; 409",
      "excerpt": "    fn validate_completeness(\n        &self,\n        package: &CheckedPackageV2,\n        budget: &mut NativeJoinBudget,\n    ) -> Result<(), ExactTypeProjectionError> {\n        // Required rows are derived below from admitted tables, never from the\n        // number of rows retained by this witness.\n        self.complete_selected_documents(package, budget)?;\n        // Declarations, including model-owned units, share one ascending",
      "finding": "validate_and_rebind ignores model unions. validate_contents does not re-check model_unions rows and validate_completeness requires no row for a union-family variant_type node, while it does require admitted_tuples rows. FR-352-AC-28 says missing emitted declarations or unsupported identity channels refuse rather than yield a partial complete witness, and FR-830 full scope requires one row per union-family node; a rebound witness can be complete without the rows direct projection needs. Matching the older model-enum omission is not contract scope."
    },
    {
      "fnd": "FND-011",
      "severity": "low",
      "confidence": "medium",
      "method": "code-review",
      "check": "soundness",
      "artifact_id": "crates/filament-contract-model/src/checked_package/v2/exact_projection.rs::selected_declaration_error",
      "related_ids": [
        "FR-352-AC-31",
        "FR-352-AC-30"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection.rs",
      "lines": "2976-2990, 2999",
      "excerpt": "\nfn selected_declaration_error(\n    package: &CheckedPackageV2,\n    node: &CheckedNodeId,\n) -> ExactTypeProjectionError {\n    let code = if package.node(node).is_some() {\n        ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration\n    } else if node.package == package.wire.name\n        || package.models.selected_model(&node.package).is_some()\n    {\n        ExactTypeProjectionErrorCode::UnknownNode\n    } else {\n        ExactTypeProjectionErrorCode::ForeignNode\n    };\n    witness_error(node, code)",
      "finding": "A shared helper changed behaviour for every caller. selected_declaration_error and its budgeted twin now return UnknownNode instead of ForeignNode for any node whose package equals a selected model document identity. This affects all joins and projections that use the helper, not just the model-union join, and only the FR-830 vectors exercise it."
    },
    {
      "fnd": "FND-012",
      "severity": "low",
      "confidence": "high",
      "method": "code-review",
      "check": "other",
      "artifact_id": "FR-352-AC-31",
      "related_ids": [
        "FR-358-AC-15",
        "FR-069-AC-11"
      ],
      "path": "spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md",
      "lines": "627 (e8ba885f7)",
      "excerpt": "Joining the selected model tuple and model union witness rows (FSpec FR-830) returns the complete rows keyed by structural node, retaining each type handle, settled key, case rank, position, name, original case, member and payload as the producer's opaque identities. Tuple rows are admitted first, then union identity, case census, per-case checks, payload original type against the side-input witness table, and coverage: a full scope refuses an uncovered union with `IncompleteIdentityWitness` and a reached scope with `UnavailableIdentityWitness`. Direct projection and a record or object field o",
      "finding": "Spec-row wording. FR-352-AC-31 says a foreign selection refuses UnknownNode, which is ambiguous beside FR-352-AC-30 (identities from an independent admission refuse ForeignNode). FR-358-AC-15 says every row is charged in the common pass before allocation, but FR-830 and the code allocate the preflight handle index first. None of the three new rows carried the status prefix sibling rows use."
    },
    {
      "fnd": "FND-013",
      "severity": "medium",
      "confidence": "high",
      "method": "code-review",
      "check": "code-bug",
      "artifact_id": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs::model_union_closure",
      "related_ids": [
        "FR-358-AC-15",
        "FR-830-AC-3"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs",
      "lines": "150-157, 165-166, 221-233",
      "excerpt": "        if package.node(start).is_none() {\n            return Err(selected_declaration_error(package, start));\n        }\n        charge_native_storage(budget, start, start, \"model_union_closure\", 3)?;\n        let mut visited = BTreeSet::new();\n        let mut pending = vec![start.clone()];\n        visited.insert(start.clone());\n        while let Some(id) = pending.pop() {",
      "finding": "The reached closure clones before it pays. The start node is cloned twice (pending and visited) under a flat charge of 3 storage units, not its node size; each new target is cloned twice (visited.insert and pending.push) under one node_units+2 charge; the per-node edges and members Vecs grow with no storage charge. A long start or target CheckedNodeId under a tight storage limit allocates beyond what the budget admitted, violating precharge-before-clone."
    },
    {
      "fnd": "FND-014",
      "severity": "medium",
      "confidence": "high",
      "method": "code-review",
      "check": "code-bug",
      "artifact_id": "crates/filament-contract-model/src/checked_package/v2/native_join_storage.rs::model_union",
      "related_ids": [
        "FR-358-AC-15"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/native_join_storage.rs",
      "lines": "605-612; crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:470,477,774-777",
      "excerpt": "            }\n            // Retention: precharge every copy before it is made.\n            charge_native_storage(budget, id, id, \"model_union_row\", row_units(row))?;\n            handles.insert(row.type_handle.clone());\n            handles.insert(row.settled_key.clone());\n            staged.insert(id.clone(), Arc::new(row.clone()));",
      "finding": "Staged and index storage is under-charged. staged.insert clones the structural node as the map key, but row_units (native_join_storage::model_union) counts the structural node once, for the row copy; the tuple path charges the key separately (complete_witness.rs:121-124, storage::node(id)). The handle-set inserts at 470 and 775-776 and the joined_handles BTreeMap at 472-478 allocate with no storage charge."
    },
    {
      "fnd": "FND-015",
      "severity": "medium",
      "confidence": "high",
      "method": "code-review",
      "check": "code-bug",
      "artifact_id": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs::join_model_unions coverage",
      "related_ids": [
        "FR-358-AC-15",
        "FR-830-AC-11"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs",
      "lines": "457-469, 780-787",
      "excerpt": "        for (id, entry) in package.models.selected_declarations() {\n            let needs_tuple = entry.form == DeclarationForm::TupleValueType;\n            let needs_union = entry.form == DeclarationForm::VariantType\n                && package.models.variant_family(id) == Some(\"union\");\n            if !(needs_tuple || needs_union) {\n                continue;\n            }\n            charge_native_work(budget, first, id, \"model_union_coverage\", 1)?;",
      "finding": "Uncharged scans. The coverage loop walks every selected declaration and calls variant_family (which scans the variant's cases) before any work charge, charging only matching tuple and union nodes; the tuple-vs-prior-union check at 457-469 scans every retained union for every joined tuple with no charge (T x U). These are whole-model and quadratic blind spots in the shared work budget."
    },
    {
      "fnd": "FND-016",
      "severity": "medium",
      "confidence": "high",
      "method": "code-review",
      "check": "coverage",
      "artifact_id": "tests/conformance_fspec/model_union_witness.rs::uses_source_rows",
      "related_ids": [
        "FR-830-AC-10",
        "FR-830-AC-15",
        "FR-352-AC-31"
      ],
      "path": "tests/conformance_fspec/model_union_witness.rs",
      "lines": "377-381",
      "excerpt": "/// Fixture C holds source rows this adapter does not build; the source-row\n/// non-substitution controls are not executed here and remain an open boundary.\nfn uses_source_rows(vectors: &Value, id: &str) -> bool {\n    find(vectors, id).to_string().contains(\"/source_\") || resolve_fixtures(vectors, id) == \"C\"\n}",
      "finding": "The typed source non-substitution controls are skipped as if producer-only. MUW-19, MUW-20 and MUW-R-59..R-62 need only typed SelectedSourceUnionRow and SelectedSourceTupleRow rows and the existing native source joins, which IR already owns; the FSL parser boundary covers the closed format only. Nothing shows a valid source_unions or source_tuples row never supplies a missing model row, or that MUW-20 admits beside a source tuple."
    },
    {
      "fnd": "FND-017",
      "severity": "medium",
      "confidence": "high",
      "method": "code-review",
      "check": "test-intent",
      "artifact_id": "tests/conformance_fspec/model_union_witness.rs::published_reader_refusals_keep_their_codes",
      "related_ids": [
        "FR-830-AC-14",
        "FR-830-AC-7",
        "FR-352-AC-31"
      ],
      "path": "tests/conformance_fspec/model_union_witness.rs",
      "lines": "420-428",
      "excerpt": "        let want = row[\"expected\"][\"locus\"].as_str().unwrap_or(\"\");\n        // A tuple-channel refusal keeps the existing tuple stage form; the\n        // model-tuple row pointers are reported as a boundary, not asserted.\n        let tuple_stage = id.contains(\"-TR-\");\n        if actual != expected || (!tuple_stage && locus.as_deref().unwrap_or(\"\") != want) {\n            mismatches.push(format!(\n                \"{id}: expected {expected} at {want}, got {actual} at {locus:?}\"\n            ));\n        }",
      "finding": "The -TR- locus exemption drops required coverage. IR owns the typed model_tuples join and knows each row's sidecar index before filtering, so /model_tuples/{k}/... is IR's coordinate to report, not an FSL adapter pointer. The substring match also exempts MUW-TR-08, whose expected locus is the union pointer /model_unions/1/type_handle. Coverage vectors' expected.node is never compared with error.at."
    },
    {
      "fnd": "FND-018",
      "severity": "low",
      "confidence": "high",
      "method": "code-review",
      "check": "other",
      "artifact_id": "spec/model/functional/FR-019-rust-library-interface.md",
      "related_ids": [
        "FR-019-AC-27"
      ],
      "path": "spec/model/functional/FR-019-rust-library-interface.md",
      "lines": "311, 316, 318",
      "excerpt": "| `checked_package` (V2 selected monitor type closure) | types `SelectedMonitorClosureRows`, `SelectedMonitorTypeClosure`, `SelectedMonitorTypeClosureError`, `SelectedMonitorTypeClosureErrorCode`; associated bounded project_selected_type_closure read over one admitted root, selected producer rows and shared native budget | FR-357-AC-1 through FR-357-AC-4 |",
      "finding": "FR-019 inventory omits part of the public change: the new public field SelectedMonitorClosureRows.model_unions is a breaking change for struct-literal consumers (receipt.rs and monitor_plan_public.rs had to change) but is not in the breaking-change prose, and the associated with_model_union_rows join, model_union_rows/model_union_row accessors and SelectedTypeDeclarations::unions() are not named."
    }
  ],
  "bindings": []
}
```
