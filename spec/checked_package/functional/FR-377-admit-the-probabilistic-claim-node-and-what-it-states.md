---
id: FR-377
title: "Admit the probabilistic claim node and what it states"
type: FR
relationships:
  - target: ix://agent-ix/filament-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/filament-contract-ir/FR-369
    type: depends_on
  - target: ix://agent-ix/filament-specification/FR-640
    type: references
  - target: ix://agent-ix/filament-specification/FR-407
    type: references
  - target: ix://agent-ix/filament-specification/FR-322
    type: references
---

# FR-377: Admit the probabilistic claim node and what it states

## Description

When a V2 checked package carries a `claim`/`probabilistic_claim` node, the
Contract IR reader SHALL admit exactly FSpec FR-640's nine-binding body
("Probabilistic claim node"), or refuse the whole package at the first defect
in FR-640's reader rules 6 and 7. An admitted package SHALL expose each claim
node's reads and statement through one typed read, so a provider, a replay or
a certificate checker that holds only the package bytes reads the claim's
subject, scheduler, rewards, distributions and delays, and its form, event or
measure, exact thresholds, `over` parameter, activation and confidence
parameters, from the package and never from the source text.

## Inputs

The untrusted `filament.checked-package/v2` bytes, evidence and limits of
[FR-038](./FR-038-consume-checked-package-v2.md). The body, its path and the
reader rules are specified by `ix://agent-ix/filament-specification/FR-640`,
what the claim's identity covers by `ix://agent-ix/filament-specification/FR-407`
"Obligation identity", and literal spelling, identity and check order by
`ix://agent-ix/filament-specification/FR-322`.

## Outputs

An admitted package retaining every claim node with its exact body and
dependencies, or one typed refusal with code, cause when defined and RFC 6901
pointer. No partial package.

## Behavior

The reader SHALL run FSpec FR-640's rule 6 and then rule 7 on each
`claim`/`probabilistic_claim` node, in ascending structural node-id order,
after every `probabilistic` node has passed rules 1 to 5 and before the
requirement-record step of [FR-376](./FR-376-admit-the-requirement-records-of-a-checked-package.md).
In this crate the reader SHALL run the claim checks inside
`probabilistic_join::validate`, after its rules 1 to 5, so that the step order
FR-376 states stays unchanged. For one node, rule 6 runs before rule 7, and
rule 7's six checks (shape, time, exact values, ranges, targets, reward) run
in that order, each over bindings 6 to 9 in body order, a binding before the
bindings nested in it. The reader SHALL refuse the first failure with exactly the code, cause and pointer FR-640
gives.

Rule 7's exact-value check SHALL read a literal's `value` only after FR-322's
strict literal read has admitted its spelling, so a non-canonical rational,
integer or quantity magnitude keeps FR-322's refusal at the literal's `value`
and is never reported by rule 7. Rule 7 SHALL check every exact value's `type` by
node id: the builtin `scalar_type`/`rational` node for a `rational`, the
builtin Integer node for an `integer` (an `integer` typed at an
`integer_range` node refuses, though FR-322's family check admits it
elsewhere), and for a `quantity` the unit node of the reward nodes'
`value_type` that binding 3 holds, or the source unit node of the subject
model's `timed_model` root. A subject is timed when such a root exists
whatever its time source, `dense` or `tick` (FSpec FR-640 "Probabilistic
claim node", FR-415): `elapsed`, `expected_elapsed` and a time-quantity
`within` or `window` are admitted over either source, and only a `delay`
weighting needs the `dense` root FR-369's delay rule finds.

The temporal placement pass ([FR-038](./FR-038-consume-checked-package-v2.md)
step 1, FSpec FR-370 "Placement and profile fit") SHALL admit a `reference`
to a `temporal`/`formula` node as the `formula` binding of the `event` of a
`claim`/`probabilistic_claim` node's `form`, and SHALL keep refusing such a
reference anywhere else in that body, and any `temporal`/`fairness`
reference in it, `ill_typed`/`operator-ineligible` at the claim node, before
FR-640's rules run. This crate's placement walk (`temporal.rs`) gives that
one binding the formula slot.

The reader SHALL NOT compare a claim node's path with its body, re-derive a
horizon from a formula, or check a formula's operators
against the temporal profile, an event's finite window or the object type of
an `over` or `on each` parameter (FSpec FR-640 rule 7, last paragraph): those
are the frontend's checks.

The reader SHALL charge the caller's reader budget as FR-640 "Reader" states,
one visit per claim node, per `reward`, `distributions`, `delays` and fairness
member, and per binding of `form`, `over`, `activation` and `confidence` and
per binding nested in them, each before inspecting it, a binding before the
bindings nested in it, and on exhaustion return its `incomplete` result at the
first uncharged member with no partial package.

### Body grammar of the claim node

The claim body is a closed grammar of its own, as the `probabilistic` bodies
are: FSpec FR-322 "Body grammar" gives the five strata of every other body, and
its Group, Tuple and Member productions stop one aggregate short of FR-640's
`form`, `over`, `activation` and `confidence` bindings, whose values are
aggregates of bindings that hold aggregates of bindings (`form`, then `event`
or `measure`, then `bound`, then the exact value). The reader SHALL read the
claim body as a Leaf, an `aggregate` whose members are each such a term, or a
`binding` of one; no `application` stands in it, and a body outside
FR-640's nine bindings is refused by rule 6 or 7, not by the grammar. The
nesting is fixed by FR-640's forms, so the JSON depth of a claim node is fixed
whatever the model's size: the deepest member the reader names, a predicate
`from` inside the `measure` of a `compare` event inside the `form`, stands at
seventeen JSON Pointer steps below the package root, and the reader's pointer
builder holds twenty.

### Public claim read

An admitted package SHALL expose `CheckedPackageV2::probabilistic_claims(&self)
-> &[CheckedProbabilisticClaimNode]`, one entry per claim node in ascending
structural node-id order:

```rust
pub struct CheckedProbabilisticClaimNode {
    pub node: CheckedNodeId,
    pub subject: CheckedNodeId,                 // the model_root
    pub scheduler: ProbabilisticScheduler,              // Workload(node) | Every(Vec<FairnessMember>)
    pub rewards: Vec<CheckedNodeId>,
    pub distributions: Vec<CheckedNodeId>,
    pub delays: Vec<CheckedNodeId>,
    pub form: ProbabilisticForm,                        // FR-640 binding 6
    pub over: Option<CheckedNodeId>,            // a value/parameter node
    pub activation: ProbabilisticActivation,            // Origin | Each { trigger, guard }
    pub confidence: Option<ProbabilisticConfidence>,    // alpha, beta, indifference
}
pub enum ProbabilisticForm {
    Probability { event: ProbabilisticEvent, bound: ProbabilisticBound },
    Quantile { level: CheckedRational, measure: ProbabilisticMeasure, bound: ProbabilisticBound },
    MeanOfFraction { measure: ProbabilisticMeasure, bound: ProbabilisticBound },
    LongRunFraction { predicate: CheckedNodeId, weighting: ProbabilisticWeighting, bound: ProbabilisticBound },
    Reach { target: ProbabilisticReachTarget, bound: ProbabilisticBound },
    ExpectedReward { until: CheckedNodeId, bound: ProbabilisticBound },
    ExpectedElapsed { until: CheckedNodeId, bound: ProbabilisticBound },
}
pub enum ProbabilisticEvent { Formula(CheckedNodeId), Compare { measure: ProbabilisticMeasure, bound: ProbabilisticBound } }
pub enum ProbabilisticMeasure {
    Accumulate { from: CheckedNodeId, until: CheckedNodeId, within: ProbabilisticExact },
    Steps { from: CheckedNodeId, until: CheckedNodeId, within: ProbabilisticExact },
    Elapsed { from: CheckedNodeId, until: CheckedNodeId, within: ProbabilisticExact },
    Fraction { predicate: CheckedNodeId, window: ProbabilisticExact, weighting: ProbabilisticWeighting },
}
pub enum ProbabilisticWeighting { None, Reward, Delay }
pub enum ProbabilisticReachTarget { Eventually(CheckedNodeId), Always(CheckedNodeId), Until(CheckedNodeId, CheckedNodeId) }
pub struct ProbabilisticBound { pub direction: BoundDirection, pub value: ProbabilisticExact } // AtLeast | AtMost
pub enum ProbabilisticExact { Integer(BigInt), Rational(CheckedRational), Quantity { unit: CheckedNodeId, magnitude: CheckedRational } }
```

- Every field SHALL be read from the admitted node's body in FR-640's binding
  order; every id is in the resolved-graph form the admitted package exposes
  and compares by structural key (FSpec FR-322 "Rebuilding the key").
- `ProbabilisticWeighting::Reward` and every `accumulate` measure and `ExpectedReward`
  read the reward named by `rewards`; the read does not repeat the name.
- The read SHALL allocate nothing per call and return what admission
  retained. A package with no claim node returns the empty slice.

The writer SHALL write each claim node exactly as admitted.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-377-AC-1 | The reader admits FSpec TC-1563's packages of FR-640-AC-8 and FR-640-AC-10. `probabilistic_claims()` returns `Avail`, `Cost`, `Fair` and `P95` in that order; `P95` gives `Quantile` with `level` `19/20`, an `Accumulate` measure whose `within` is `Integer(10)`, `bound` `AtMost` `Quantity` `5` at the unit node of `ms`, `over` `None`, `Origin` and confidence `1/100` three times; `Avail` gives `Probability` over a `Formula` event and `AtLeast` `Rational(1/2)`; `Cost` gives `ExpectedReward` with `AtMost` `Rational(3)`; `Fair` gives `Every` with its one fairness member. Each package of FR-640-AC-10's spellings returns equal entries, and each of its identity changes returns an entry with another `node`. The writer's output read back returns equal entries, and the same package with its nodes in another graph order returns equal entries. | Test (TC-489) |
| FR-377-AC-2 | Each refusing mutation of FSpec FR-640-AC-9 and FR-640-AC-11 refuses with FR-640's exact code, cause and pointer, observed through the public reader, with no partial package: among them `P95`'s `form` `kind` `median` at that `kind`, `P95`'s `bound` `value` `Rational(5)` at the literal, `P95`'s `level` `1` at the literal, `P95`'s `level` spelled `38/40` at the literal's `value` (FR-322's refusal, not rule 7's), `Avail`'s `event` `formula` referencing the `true` literal node as `ill_typed`/`operator-ineligible`, `Avail`'s `trigger` referencing it as `invalid_model_binding`/`malformed-declaration`, `Cost`'s empty `reward` at `reward`, and the node with both a rule-6 and a rule-7 defect at rule 6's member. FR-640-AC-11's path rewrite is admitted. | Test (TC-489) |
| FR-377-AC-3 | The claim checks run inside `probabilistic_join::validate` after rules 1 to 5 and before FR-376's step: a package with a rule-7 defect and a requirement-record defect refuses at the rule-7 defect. FR-640-AC-8's package is admitted with exactly its full charged count, which includes FR-376's visits for its `requirement_items` rows and their domains; a reader budget one visit short of the visits charged up to the end of FR-640's reader (every step before it included, FR-376's step excluded) returns `incomplete` at `P95`'s `confidence` `indifference` member, the last member FR-640's reader charges (FSpec FR-640-AC-9), with no partial package. | Test (TC-489) |
| FR-377-AC-4 | The placement pass admits FSpec FR-640-AC-8's package, whose `Avail` and `Fair` `event` `formula` bindings reference a `temporal`/`formula` node, and refuses `ill_typed`/`operator-ineligible` at the claim node, before FR-640's rules, each package of FSpec FR-370-AC-22: the formula node referenced as `P95`'s `measure` `from`, and a `temporal`/`fairness` node referenced as `Avail`'s `event` `formula`. Over FSL FR-233-AC-3's `tick` `Rpc` model, the `expected_elapsed` and `elapsed` claims of FSpec FR-640-AC-11 are admitted with quantities at `ms`, and the `delay` weighting refuses at its `kind`; a `within` `integer` `10` typed at an `Int[0, 10]` node refuses `invalid-value` at the literal. | Test (TC-489) |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns the V2 reader, writer,
  identity and shared refusal machinery; [FR-369](./FR-369-read-probabilistic-nodes-and-their-four-forms.md)
  admits the `probabilistic` nodes a claim reads.
- `ix://agent-ix/filament-specification/FR-640` owns the claim node body, its
  path, rules 6 and 7 and their refusals; `ix://agent-ix/filament-specification/FR-322`
  owns literal spelling, structural identity and check order.
