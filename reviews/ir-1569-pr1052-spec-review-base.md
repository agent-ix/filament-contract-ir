---
id: SR-20261118225204
title: "spec-review of IR-1569 FR-369 AC-23/24 and TC-476 steps 23-24 (PR #1052)"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1052 at 39d888c3 (base origin/main 836ae95c); spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md, spec/checked_package/matrix/TC-476-probabilistic-node-reader.md, spec/checked_package/matrix/tests.md"
review_set: subset
---

## Summary

This is the base spec review of the spec changes in filament-contract-ir PR #1052. Ticket: IR-1569. The reference text is FSpec FR-640, FR-405 and FR-420 at filament-specification `origin/main` f92839de.

`quire validate` on the three changed files reports 3/3 grammar-clean. That run used quire 0.36.2.

## Examined scope

- The new FR-369 paragraph on Boolean and enumeration support values (examined). It matches FR-640 Properties: "a literal of the parameter's declared type". It also matches FR-405 "Random parameters": "Each value is a literal of the parameter's declared type". The paragraph correctly limits enumerations to variants with no payload case.
- FR-369-AC-23 (examined). It is atomic enough and testable. It names admission and read-back for both types.
- FR-369-AC-24 (examined). It enumerates its refusal cases and states one code, cause and locus. The list is long but is one rule ("value outside the parameter's type", FR-640 Reader rule 2).
- TC-476 steps 23-24 and the status (examined). They agree with the ACs.
- `tests.md` TC-476 row (examined). It lists AC-23/24, and its status names IR-1569.
- FSpec FR-640 scope (context_only). FR-640 says "a literal of the parameter's declared type" for any parameter type. FR-322 has no literal form for a record, tuple, option, collection or payload-variant value, so FSpec itself cannot express those supports. That is an FSpec gap, not an IR defect, and FR-369 correctly records it as refused.

## Verdict

The spec change is sound, and its numbering does not collide with anything else. One clarity nit: the payload-variant refusal names no code.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | "A support value of a declared record, tuple, option or collection type, or of a variant type with a payload case, has no literal kind and is refused" names no code, cause or locus. Two readings are plausible: `invalid_package`/`invalid-value` per FR-640 Reader rule 2 "outside the parameter's type", or the generic literal check's `ill_typed`. The implementation gives `ill_typed`/`operator-ineligible` at the literal's `type` for a payload variant (probe). State the code and pointer the reader actually returns. | spec/checked_package/functional/FR-369-read-probabilistic-nodes-and-their-four-forms.md:183-185 |

## Dispositions

Round 1 was verified at head 9ba38668.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The FR-369 sentence now reads: the value "has no literal kind: FSpec has no literal form for these, and the reader's general literal check refuses it as `ill_typed`/`operator-ineligible` at the literal's `type`, before this join". That states the code and the locus, and it matches the probe and the new public-reader test. |
