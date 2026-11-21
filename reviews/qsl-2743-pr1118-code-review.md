---
id: SR-967175026302197768900474181941935084518
title: "QSL-2743 PR 1118 code-review (Rust lane) of held-model equality locality, semantic-value pin and population source lookup"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; QSL-2743 (QSL-2711 rides); PR 1118 head b7dcf6666bb715253e4c8af6f7920b7cd1784c7c base 743b5e5f1983642a118dc1db2ee945d370e4a3ee; Cargo.lock, crates/filament-contract-model/Cargo.toml, crates/filament-contract-model/src/checked_package/{selected_population.rs,v2/equality_projection.rs,v2/mod.rs}, crates/filament-contract-model/tests/{selected_population_public.rs,fixtures/population-slot-spelling/*}, tests/it/checked_package_v2_model_members.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-053", type: references }
  - { target: "ix://agent-ix/filament-contract-ir/FR-353", type: references }
  - { target: "ix://agent-ix/filament-contract-ir/FR-038", type: references }
---

## Summary

Code review of PR 1118 (agent-ix/filament-contract-ir), diff origin/main...b7dcf666 (base 743b5e5f1). Ticket: QSL-2743; the QSL-2711 population-lookup fix rides in the same PR. The Rust lane (rust-review, rust-style baseline) is part of this artifact. The equality-locality change and the semantic-value pin are sound. The population lookup fix reaches the right source entry. But for a selector spelled the way `graph()` shows the node, it returns a binding ID that differs from the selector's, and the original FR-053-AC-1 test oracle was rewritten to accept that.

## Verdict

**FAIL**: one high finding (FND-001) and one low finding (FND-002). Merge only after FND-001 is fixed or the spec owner rules on it.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-053-AC-1 says a selector naming the population node's exact structural ID "returns the same package name, node ID, occurrence". The new lookup accepts the `graph()`-spelled selector (operand slots, e.g. Operand(1)/Operand(2)) but still returns `binding: node.node_id.clone()`, which is the document-held wire ID (Index(7)/Index(5)). So the result's node ID differs from the selector's node ID. The original test selected_population_is_bound_to_the_emitted_node_and_source_occurrence asserted `selected.binding == node.node_id` (graph spelling). That assert was replaced by `selected.binding == document_held_id(position)`, and the new regression asserts the divergence ("binding is the document-held ID, as before"). The two original FSL tests are green only because the AC-1 oracle was changed. Suggested fix: return the selector's accepted binding (`selector.binding.clone()`, which equals the document-held ID for `reaches_population` selectors) and restore the original oracle. Otherwise get an explicit FR-053 spec decision that the result binding is normalised to the document-held ID. | crates/filament-contract-model/src/checked_package/selected_population.rs:396,406; crates/filament-contract-model/tests/selected_population_public.rs:202-212,547-550 |
| FND-002 | low | The new identity conversion does a second lookup that is not needed. `package.node(&selector.binding)` (node_with_position) already resolved `position_of` and returned the wire node, whose `node_id` is by definition `document_held_id(position)`. So `&node.node_id` is the held ID. Because of the second `position_of` call: (a) `position_of` was widened from private to `pub(in crate::checked_package)` (v2/mod.rs:1963) for no need; (b) a new `StaleBinding` arm was added that can never run, since `position_of` is deterministic and has already succeeded; (c) the slot-view binary search runs twice. | crates/filament-contract-model/src/checked_package/selected_population.rs:359-366; crates/filament-contract-model/src/checked_package/v2/mod.rs:1963 |

## Scope and evidence

Examined clean (no finding):

- **Equality locality** (equality_projection.rs:693-716). Locality is now `package.node(&row.structural_node)` membership. An unheld node refuses ForeignEqualityWitness at the row's own structural node. A held node with the wrong owner still refuses MismatchedEqualityWitness. The application, source-map-entry, occurrence and owner joins are unchanged, and the work charge comes before both checks. One precedence change is intended: an unheld node whose package name matches the wire name used to refuse Mismatched and now refuses Foreign. FR-353-AC-7 ("foreign-document … row returns a typed refusal at its first structural locus") covers this.
- **Equality tests.** held_model_enum_equality_facts_are_local_for_eq_and_ne covers eq and ne, operator, Enum schedule, location, both operand sources and member order, case, rank and variant. unheld_or_wrong_owner_model_enum_fact_refuses_with_node changes only the last member row's structural_node, to an ID that is genuinely unheld and has the same original package name. Earlier held rows therefore pass, and refusal.at is the unheld node. That guards against an early package-name refusal hiding the result. The old guard fails both tests (coder log /tmp/qsl2743-old.log: 2 failed, rc=101).
- **The other locality site.** quantity_unit (equality_projection.rs:1216) keeps the package-name check. No held-model quantity case was shown, so I raise no finding.
- **Semantic-value pin.** Cargo.toml changes the rev only: 6e809434d5fd7bde4c5b88ec40cd809efcca595d -> fa5e78aa71fcea5418492b67f13a33124af84459. Cargo.lock changes one source line. fa5e78aa is reachable from filament-semantic-value origin/main and descends from 6e809434 (3 commits: #119, #120, #121 QSL-2734 derived roots). Its [dependencies] are identical, so there is no extra dependency, mixed source or patch.
- **Population lookup.** It uses the retained source index (no linear scan and no fallback). StaleBinding, MissingOccurrence, ForeignPackage and NotPopulation keep their order. When slot IDs have equal bytes, position_of returns None, so node() refuses StaleBinding before any lookup and the wrong node is never chosen.
- **Fixture.** population-slot-spelling/* matches /home/peter/pop2711 byte for byte (cmp). It contains no digest or hash catalog.
- **Rust lane.** No unsafe code, no new production panic, no new integer casts and no new public API.

Reviewer receipts at final head b7dcf666:

- Run with FILAMENT_SPEC_LANGUAGE_DIR set to the FSL replay at 158cc43285 (contains f89c2e03; clean): `/tmp/ir-dynamic-build cargo test -p filament-contract-model --test selected_population_public -- selected_population_` gave 3 passed, 0 failed, rc=0 (/tmp/ir-qsl2743-equality-reviews/rev-population.log).
- `cargo test --locked --features conformance-cli --test it -- held_model_enum_equality_facts_are_local_for_eq_and_ne unheld_or_wrong_owner_model_enum_fact_refuses_with_node reaches_population` gave 20 passed, rc=0 (/tmp/ir-qsl2743-equality-reviews/rev-it.log).
- Arch's old-code failure of both original tests: /home/peter/arch-test-logs/qsl2711-c1.log (2 failed, at lines 199 and 435).

CI:

- Rust Checks / Lint (fsl-string-edge, 5 occurrences in model_members.rs:858,4807, operations.rs:4563-4564 and protocol_locations.rs:102) fails at b7dcf666. The same 5 occurrences fail on merged PR 1116's head 4c61cb05 (run 44904729249). None of these files is in this diff, so this is a pre-existing baseline failure.
- origin/main has since advanced to e979980ad (PR 1117). git merge-tree reports a clean textual merge. The merged tree was not built.

## Dispositions

Round 1 was checked at frozen head 688d827b9e5c795d917d634cdbfb861fbb63280f. That head is fix commit 688d827b9 on top of merge commit d08d6af30, which merged origin/main e979980ad (PR 1117). d08d6af30 is identical to the automatic `git merge-tree` result. Reviewer reruns at this head:

- selected_population_public: 3 passed, rc=0 (r1-population.log).
- it, filtered to the two equality tests plus `reaches_population`: 20 passed, rc=0, including tc_048_population_maximum_is_canonical_or_explicitly_unbounded (r1-it.log).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fixed at 688d827b9. selected_population.rs:389 and :399 now return `binding: selector.binding.clone()` in Finite and in AdmittedUnbounded. The original test asserts `selected.binding == node.node_id` again (selected_population_public.rs:212). The regression passes both spellings and asserts the result binding equals the exact input (selected_population_public.rs:547-550). The `reaches_population` oracles, which use document-held selectors, are unchanged and pass. |
| FND-002 | fixed | Fixed at 688d827b9. The source lookup uses `&node.node_id` (selected_population.rs:359-364). The second position_of call and the unreachable StaleBinding arm are gone, and `position_of` is private again. v2/mod.rs is identical to origin/main. |
