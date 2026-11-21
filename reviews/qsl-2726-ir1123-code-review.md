---
id: SR-20261121001123001
title: "QSL-2726 IR PR 1123 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; PR 1123; model_members.rs and model_members/tests/intake_retention.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: reviews
---

## Summary

Ticket: QSL-2726. Entire frozen PR #1123 diff reviewed: two Rust files, 48 insertions and 5 deletions. Captured base 7e4f54095816d51aeb034be8675959e3ef3dbe72; head ac73b4f7c5400299e27cd9e66210d6dfeb5de3ed. This is a bounded PR review, not a repository assurance or downstream FSL acceptance claim.

## Verdict

PASS for the reviewed diff; no defects found. Final current-main rebase and all-target gates remain lead-owned and pending.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

The new private byte predicate exactly implements ASCII initial letter followed by ASCII letters, digits, underscore or hyphen; empty text, leading punctuation/digits, dots, Unicode and slash-separated suffixes fail. Prefix validation remains exact. Both type and population classification reach the predicate. Other is_object_id uses retain their strict grammar. No public API, dependency, wire owner, budget charging, limit or validation-order change is introduced. Searched the V2 model implementation for existing artifact-ID predicates: no equivalent reusable predicate was found; the member-name predicate has intentionally different behavior.

The positive regression exercises actual selected-document admission and checks admitted object, population and membership. Dot population refusal and the existing dot type retention tests cover negative admission. Updating bad-id to bad.id repairs the old negative premise while retaining identity/origin and work-limit assertions. The separate bad-id in the metadata charging test directly constructs SelectionRefusal and does not assert identity rejection; it remains valid.

Author receipts inspected: affected intake_retention 8 passed, 0 failed; operation_parameter_names_are_validated_at_selection 1 passed, 0 failed. No reviewer Cargo/build/test/gate ran. Genuine FSL source-emission acceptance awaits the consuming pin PR and is not claimed here. The untriaged field-name acceptance observation concerns unchanged member validation; admitting the now-legal enclosing type can expose that existing defect, but this diff does not alter that member rule. It is not classified as a new regression.

No applicable AssuranceProfile was found. No CI or gate weakening, copied schema, source fixture vendoring, stub, unsafe operation, panic on input, new concurrency or unowned behavior was added. No spec/plan files changed, so spec-review and its sub-analyses do not apply.


- FR-038-AC-174, role: examined, path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md. Excerpt: IMPLEMENTED; final-head runtime/Clippy checks UNRUN, full gates pending (IR-663 identity retention). When FR-154 intake selects an offending declaration with a supplied string identity, the reader retains that exact FCD identity in the private refusal returned by `admit_selection`, including refused object-id spelling. Missing or non-string identity yields None; no graph node id, inferred owner, repaired string or separate artifact id supplies it.

- FR-038-AC-175, role: examined, path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md. Excerpt: IMPLEMENTED; final-head runtime/Clippy checks UNRUN, full gates pending (IR-663 private constructors). Each private `SelectionRefusal` constructor and declaration-to-`SelectionFailure::Refused` conversion preserves the selected retained metadata record through the private `admit_selection` return after document release. Independently dropping available identity/origin or substituting a different declaration's record fails the typed return assertion. Pre-declaration constructors preserve genuine metadata absence.

- FR-038-AC-186, role: examined, path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md. Excerpt: IMPLEMENTED; final-head runtime/Clippy checks UNRUN, full gates pending (IR-663 source origin). Given a located refusal with a valid FCD Source origin, the reader returns its exact typed source member values and optional-member presence at the private intake boundary, including all four end-coordinate presence combinations and the existing admitted 2^53 boundary. Retention adds no paired-end or span-order constraint.

- FR-154-artifact-id, role: examined, path: agent-ix/filament-specification/spec/functional/type-model/FR-154-admit-domain-package-model.md. Excerpt: A type definition's or population node's artifact id does not match `^[A-Za-z][A-Za-z0-9_-]*$`

- FR-154-member-name, role: examined, path: agent-ix/filament-specification/spec/functional/type-model/FR-154-admit-domain-package-model.md. Excerpt: A field's or an operation's `name` is absent, is not a string, is empty, does not match `^[A-Za-z][A-Za-z0-9_]*$`, or differs from the last segment of its identity `<owner identity>/<member name>`, on a member not refused as a repeated identity of its owner (checked after the "Two nodes share one identity" row below)

- FR-322-selection-evidence, role: context_only, path: agent-ix/filament-specification/spec/objects/interfaces/FR-322-checked-package-artifact.md. Excerpt: After strict syntax and member validation and before graph admission, the reader admits the document of each model_root node, in node order, under FR-154's admission checks in FR-154's order.

- FR-065, role: context_only, path: spec/checked_package/functional/FR-065-carry-selected-model-root.md. Excerpt: The public typed CheckedNodeOwner vocabulary SHALL include the closed ModelRootOwner {kind: "model_root", identity}.
