---
id: SR-17940486709564847048125
title: "spec-review of the QSL-2726 FR-372 prose for the FIFO key model walk"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; branch code/fifo-key-model-record at 0ce5cc0d; spec/checked_package/functional/FR-372-admit-bounded-channels-population-captures-reference-carriers-and-post-test-repeats.md (lines 80-96, 186-196); FSpec FR-726, FR-322 equality-stable types, FR-465 reader step 2 (context)"
review_set: subset
---

## Summary

Ticket: QSL-2726. The diff edits two FR-372 passages:

- the reader-rule prose: a model record, tuple or variant "is walked through its members, which ... are the fields (own and inherited), positions and payloads its selected model document declares, each declaration expanded once";
- the test-status note, which ends "(a tuple is not an admitted `carries` type, so it is reached through a record field or a variant payload)".

FR-372-AC-2 and FR-372-AC-3 are unchanged, and they agree with FSpec FR-726-AC-2 and FR-726-AC-3.

## Verdict

**FINDINGS.** The rule prose states a behaviour the code does not have (inherited fields). The test note records, as a fact, a reader limit that contradicts FSpec FR-465. The ACs themselves are accurate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-372 says the walk covers record fields "own and inherited". The implementation reads only own fields (code-review SR FND-001), so the spec claims a refusal that does not happen. Either fix the code (preferred: FR-726 requires every leaf) or drop "and inherited" | spec/checked_package/functional/FR-372-admit-bounded-channels-population-captures-reference-carriers-and-post-test-repeats.md:89-92 |
| FND-002 | medium | The test note says "a tuple is not an admitted `carries` type". FSpec FR-465 reader step 2 lists `model` nodes of form `record_value_type`, `tuple_value_type`, `event_type` or `variant_type` as admitted `carries`. The reader's match (protocol_body.rs:3408-3412) omits `TupleValueType`, so this is a reader defect, not a spec limit, and FR-372 must not record it as a premise. File or fix the `carries` defect, and reword the note as a known gap | spec/checked_package/functional/FR-372-admit-bounded-channels-population-captures-reference-carriers-and-post-test-repeats.md:192-196; crates/filament-contract-model/src/checked_package/v2/protocol_body.rs:3408-3412 |
| FND-003 | low | The rule prose does not say what happens to a member whose type has no FR-322 step-4 member type (a map, or a value type with a float scalar). The code treats such a member as stable. FR-726 needs it treated as unstable, or walked. State the rule so that an implementer cannot read silence as admission | spec/checked_package/functional/FR-372-admit-bounded-channels-population-captures-reference-carriers-and-post-test-repeats.md:88-92 |

## Dispositions

Round 2, at head ae150b09 (fix commits be6ed5f9 and ae150b09). Checked against the FR-372 text at that head and against the reviewer probes run in a snapshot with its own target dir.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Lines 89-92 still say "fields (own and inherited)", and the code now honours it: `holds_key_unstable_leaf` expands `std::iter::once(name).chain(ancestors)` (model_members.rs:1910). The probes for a float on a supertype and two supertypes up both refuse ill_typed/operator-ineligible. after_excerpt: "its members, which for a record, tuple or variant are the fields (own and inherited), positions and payloads its selected model document declares, each declaration expanded once." |
| FND-002 | fixed | The "a tuple is not an admitted `carries` type" clause is removed, and `ModelForm::TupleValueType` is added to the `carries` match (protocol_body.rs:3341), per FSpec FR-465 step 2. The tuple-root probes admit `Pair(Boolean, Integer)` and refuse `Pair(Boolean, Float32)`. after_excerpt: "a model-owned `record_value_type`, tuple and variant are tested for a document-declared `float32` or `float64` leaf in a record field, an inherited field, ... a tuple position, a variant payload, a map key or value, an alias and a float-scalar value type, for a `Clock` member" |
| FND-003 | fixed | The prose now states the rule for members with no FR-322 step-4 member type. after_excerpt: "A member whose `typeRef` is a native `Decimal`, `Rational` or `Text`, or a model value type over any scalar but `float32` and `float64`, is a stable leaf though FR-322 step 4 gives it no member type. A map is walked through its key and value types, and a model value type whose `scalar` is `float32` or `float64` holds an IEEE leaf. A `Clock` member is not equality-stable." |
