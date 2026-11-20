---
id: SR-17940486709564847048105
title: "gap-analysis of QSL-2653 against FR-038-AC-511 and TC-472"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1089 at 1dc26097b (git diff origin/main...HEAD); FR-038-AC-511, TC-472, spec/checked_package/matrix/tests.md, crates/filament-contract-model/src/checked_package/v2/probabilistic.rs, tests/it/checked_package_v2_whole_read_accounting.rs, tests/it/checked_package_v2_probabilistic.rs, tests/it/checked_package_v2_source_objects/probabilistic.rs"
review_set: subset
---

## Summary

Ticket: QSL-2653. Plan completion: not assessed (planless).

The ticket's expectation is that heap proportional to input is charged or bounded. It is met by not allocating on the success path, which matches FR-038-AC-511. The AC requires non-refusal-diagnostic bytes beyond the admitted read storage to stay within 4096, and says fixed-size diagnostics do not justify single-site charges, so no new charge is needed.

`quire trace --id FR-038-AC-511` lists both new tests as `verifies` through `#[trace("TC-472", "FR-038-AC-511")]`. The ticket's two shapes are now in the whole-read registry: the rational spelling sets of a `discrete` delay and a `random_parameter`, and workloads over 256 rows. The new growth test reads both fixtures at 20/300/600 rows. A probe measured unpaid 0 and 63 B, flat across sizes, with all reads admitted. All 5 named tests pass against FSpec origin/main 73a169ff.

The production change has no code without an owning requirement. There are no stubs, and the coverage claims are not inflated: the matrix row stays 🚧 Partly tested, and the FSpec-dependent half is still described as needing `FILAMENT_SPECIFICATION_DIR`.

## Verdict

**Clean.** FR-038-AC-511 coverage for the QSL-2653 shapes is real and adequate. Examined: FR-038-AC-511 (examined), FR-358-AC-12 (context_only), FR-369-AC-27 (context_only), TC-472 (examined). The low tagging and admission-assertion points are recorded in SR-17940486709564847048104 and not repeated here.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
