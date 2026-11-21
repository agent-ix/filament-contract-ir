---
id: SR-967175026302197768900474181941935084552
title: "QSL2271 PR 1121 spec-review/base of FR-378 and TC-493 against FSpec FR-831"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; QSL-2271; PR 1121 head 50ba1ace53b6388ff1e41b329b6c0bfb33e25dcb; spec/checked_package/functional/FR-378-read-protocol-correlation-provenance-of-a-checked-package.md, spec/checked_package/matrix/TC-493-protocol-correlation-provenance-read-from-an-admitted-package.md, spec/checked_package/matrix/tests.md (FR-378, TC-493 rows)"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-378", type: references }
---

## Summary

Spec review of the new FR-378 and TC-493, checked against FSpec FR-831 and TC-2113 at merged f6b0480cd. Ticket: QSL-2271. `quire validate --scope .` on both files reports no errors.

## Verdict

**FAIL**: two high findings. In both, FR-378 contradicts the upstream contract it implements.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The FR-378 coverage rule limits coverage to "an eligible control that has any row". FR-831 requires every eligible control occurrence with m >= 1 to be covered, "no row at all included". It also refuses `[]` over nonempty related. As written, FR-378 would admit a control with records and no rows. The code and CO-R20 follow FR-831, so the statement contradicts both. | spec/checked_package/functional/FR-378-read-protocol-correlation-provenance-of-a-checked-package.md:63-65 |
| FND-002 | high | FR-378-AC-4 redefines exact admission: "the result box the strict read charges after the step is paid separately and the read completes with it". FR-831-AC-7 and CO-B11 say the exact offset over the measured baseline admits the strict read. The AC writes the IR's post-step charge into the acceptance test as uncontracted slack. It should not do that without an FSpec amendment or a reader change. | spec/checked_package/functional/FR-378-read-protocol-correlation-provenance-of-a-checked-package.md:105 |

## Coverage

- FR-378 Description, Inputs, Outputs, Public read, Limits and Dependencies were examined. Each agrees with FR-831 apart from FND-001: optional member, never null, closed 5 members, row order (charge, joins, key, ordinal), absent vs `[]`, dependency answers from its own member, and the non-authentication limits.
- AC-1, AC-2, AC-3 and AC-5 were examined, with no defect.
- TC-493 steps 1-5 were examined and follow the ACs.
- The tests.md FR-378 and TC-493 rows were examined. They claim Implemented, subject to FND-002.
