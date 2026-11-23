---
id: SR-2026112305284800000002
title: "gap-analysis of IR-1586 FR-322-AC-464 finite hyper body profile"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1136 at e7b81bbb; FR-322-AC-464, TC-2028; tests/it/checked_package_v2_temporal.rs, tests/conformance_fspec/temporal_clock_bindings.rs"
review_set: subset
---

## Summary

Gap analysis scoped to PR #1136 for ticket IR-1586. Plan completion: not assessed. This was a manual AC-to-test check limited to the PR diff; it was not a repo-wide matrix run.

## Examined scope

- FR-322-AC-464 (examined): "with lock [T, D] and the hyperproperty body's temporal_profile equal to D ... two rows ... trace a then b, each with clock_name t ... admitted. With the same body member D but lock selections [T], it refuses invalid_package/operation-law-unselected at /semantic_graph/nodes/{N}/body/temporal_profile (TCB-15). With the admitted [T, D] lock and a fixed_sample row, it refuses invalid_package/timed-node-reference at /temporal_bindings/0/clock/kind (TCB-16). Removing the b row ... trace c ... clock_name u ..."
- Bindings: TCB-14 maps to `tcb_14_...` (correct). TCB-15 maps to `tcb_15_...` (correct mechanism, but it uses lock [T, D] with a body naming fixed-sample, not the vector's lock [T] with body D; the in-repo fixture's temporal clause needs D). TCB-16 maps to `tcb_16_...` (correct). The b-row, trace c and clock_name u cases are in `tc_483_a_finite_domain_row_is_refused_at_its_own_defect` (correct, tagged TC-483).
- No stubs and no unowned production code in the diff.
- The published TCB vectors are not executed against the reader because their base package comes from the FSL emitter. The PR states this. Code-review FND-005 covers the metadata-only conformance test.

## Verdict

AC-464 is covered by reader tests that assert exact codes, causes and loci. The gaps are recorded in the code-review SR (FND-002 untested guard, FND-005). There is no separate gap finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
