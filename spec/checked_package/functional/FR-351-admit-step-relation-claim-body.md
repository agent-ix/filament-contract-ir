---
id: FR-351
title: "Admit the checked-package step-relation claim body"
type: FR
relationships:
  - target: ix://agent-ix/filament-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/filament-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/filament-contract-ir/FR-040
    type: depends_on
  - target: ix://agent-ix/filament-contract-ir/FR-346
    type: depends_on
  - target: ix://agent-ix/filament-contract-ir/FR-349
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-940
    type: references
  - target: ix://agent-ix/filament-specification/FR-477
    type: references
  - target: ix://agent-ix/filament-specification/FR-322
    type: references
  - target: ix://agent-ix/filament-specification/FR-341
    type: references
  - target: ix://agent-ix/filament-specification/FR-402
    type: references
---
# FR-351: Admit the checked-package step-relation claim body

## Description

When the Contract IR V2 reader receives a `filament.checked-package/v2` package
containing a `claim` node with `semantic_form: "step_relation"`, it shall admit
the node only when its body and joins satisfy FSpec FR-477, and retain the
admitted claim as a closed `ClaimForm::StepRelation` with a typed body. The
reader shall return the first applicable typed refusal under FSpec FR-322's
package order and FR-477's node order, with no partially admitted package.
This requirement specifies IR's consumption and writing of the FSpec artifact;
FSpec FR-477 owns production, and FSpec FR-402 owns obligation identity.

## Inputs

Canonical checked-package V2 bytes, the existing FR-038 read limits and
evidence, including selected model documents needed for declaration joins.

## Outputs

An admitted `CheckedPackageV2` with a typed step-relation claim and its
existing node, occurrence and identity data; or one existing V2 refusal or
incomplete result.

## Behavior

### Form and body

The reader shall add `step_relation` to the closed `claim` form vocabulary and
shall retain its body as a distinct typed production, not as a caller-inspected
JSON object. The body shall be an aggregate of four ordered bindings:
`subjects`, `executions`, `invariants`, `body`, optionally followed by one final
binding `subject_declaration_order`. `subjects` is an aggregate of
bindings from each distinct model alias, in first-named order, to a reference
to its model node. `executions` is an aggregate of bindings in source order,
one per execution variable. Each execution value is an aggregate of exactly
six ordered bindings: `alias` (text literal), `operation` (operation-anchor
reference), `parameters` (aggregate of value/parameter references),
`preconditions` and `postconditions` (aggregates of state-clause references),
and `implementation` (abstraction-relation reference or empty aggregate).
`invariants` is an aggregate of one binding per subject in subject order,
holding invariant-clause references. `body` references a Boolean expression.
The node's `semantic_type` shall name the Boolean scalar type; its
`declaration` shall be the declared relation name; its required source owner
shall be retained; and it shall have one declaration and one claim occurrence.

The reader shall admit an optional fifth binding `subject_declaration_order`: a
Group of bindings `(alias, reference to model root)` in source model
declaration order, using the `subjects` entry production. When the binding is
present, the reader shall require it to be an exact permutation of `subjects`
by alias and model root reference: every subject alias occurs once, none is
unknown, duplicate or missing, and each reference equals that alias's
`subjects` reference. Distinct aliases may reference one model root. The
reader shall retain the supplied sequence and shall not sort it or infer it
from first execution use. The reader shall admit a body without the binding,
and for such a body its borrowed `(alias, root)` view shall be absent. Its
refusals shall be `invalid_semantic_graph` with no cause at
`P = /semantic_graph/nodes/N/body/members/4`: a wrong final binding or name at
`P`; a non-Group value at `P/value`; a malformed entry at `P/value/members/K`;
a duplicate or unknown alias at that entry's `name`; a wrong alias/root join or
reference at that entry's `value/target`; and, only after every supplied entry
is checked in sequence, a missing alias at `P/value/members/LEN`, `LEN` being
the supplied entry count. A sixth body binding shall refuse at `members/5`. The
binding's own shape (`P`, `P/value`) is read after every other body shape, so a
defect in the first four bindings keeps its existing pointer. This is an
explicit narrow extension of the current four-binding profile, not a
seven-binding migration; filament-spec-language FR-940 owns the package form.

The reader shall require every alias to name a subject and every subject to be
named by an execution. It shall require each subject target to be a model node,
each operation target to be a `state`/`operation_anchor` whose context is a
declaration of that alias's model, and each referenced implementation to be a
`correspondence`/`abstraction_relation` with a `frames` entry keyed by that
operation. For every execution it shall require the `parameters` names and
types in FR-341 postcondition binder order: `self`, optional `result`, then
declared operation parameters. Levels shall start at zero and continue without
reset across executions; each execution shall reference its own nodes, even
when two executions name one operation. `preconditions` and `postconditions`
shall be exactly the matching state clauses anchored at that operation,
ascending by structural node id. Each subject's `invariants` shall be exactly the
invariant state clauses anchored at an object type, process or state machine
of its model, ascending by structural node id. The `body` target shall be an
expression node with Boolean `semantic_type` and its reads of each execution's
arguments, receiver states and result shall resolve to that execution's own
parameter nodes. The reader shall derive the code-claim request only when
every execution's `implementation` is a reference; otherwise only the model
claim is requested. It shall preserve the model and code items' distinct
FR-402 obligation identities over the one claim occurrence.

### Identity and order

The reader shall apply FR-322's graph, reference, dependency, owner, source-map and structural-id checks to this form. The node's dependencies shall be exactly the unique structural-id-ascending body reference targets. Its id is the retained `{package,path,ordinal}` label; the reader does not rederive it from owner, declaration or body. A changed invariant, precondition, execution order, binder or implementation changes the complete typed body and package value, while the declared path and package name may remain unchanged. Source whitespace and comments do not alter the typed body. The writer shall emit the same canonical body, dependencies, structural id and V2 bytes from the retained typed package, and its output shall read back with the same typed claim. The public reader shall also admit canonical bytes emitted by the external FSL producer under this FSpec contract.

The step-relation body step shall run after the terminal step and before the
protocol step, in FR-322 body-step order. It shall visit `claim`/
`step_relation` nodes by ascending structural node id. Within one node the reader
shall check shape, aliases, subject targets, the declaration order (each supplied
entry left to right, then omission), the remaining targets in body order,
premises, then Boolean body, reporting the first failure. Shape errors refuse `invalid_semantic_graph` at
the first binding breaking layout. An alias with no subject refuses
`missing_declaration`/`missing-name` at its alias binding; an unused subject
refuses `invalid_semantic_graph` at that subject binding. Wrong target kinds,
operation-to-model joins, implementation frame joins and premise sets or
binder lists refuse `invalid_model_binding`/`malformed-declaration` at the
offending target or aggregate. A non-Boolean body target refuses
`ill_typed`/`non-boolean-root` at that target. Earlier package stages retain
their precedence; a malformed or duplicate structural id or wrong dependency list is refused at its FR-322 stage before this body's semantic joins.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-351-AC-1 | Reading a canonical two-execution determinism relation over `V::Vault::step(i)` with an abstraction binding exposes one typed `claim`/`step_relation` node: four ordered root bindings; `V` once in subjects; `x` and `y` in source order, each with six ordered bindings, the same operation anchor, distinct `self` and `i` nodes at levels `[0, 1]` and `[2, 3]`, the operation's exact precondition/postcondition sets, the Vault invariant set, and a bound implementation; declaration `Det`, its source owner, one declaration and one claim occurrence are retained. Its body reading `pre(x.self.l) = pre(y.self.l)` reaches distinct self nodes. | Test |
| FR-351-AC-2 | Without an abstraction binding, both implementation members are empty aggregates and the reader exposes the model claim alone; with the binding, both model and code claims are requested from the one claim occurrence with different FR-402 obligation identities. | Test |
| FR-351-AC-3 | PLANNED/UNRUN (IR-1141). Adding a Vault invariant, changing step precondition or swapping x and y changes the complete step-relation body; whitespace and comment changes leave that body unchanged. Writing an admitted typed package and reading its canonical bytes preserves body, typed form and structural id; canonical output from the external FSL producer admits. | Test |
| FR-351-AC-4 | A missing execution `implementation` binding refuses `invalid_semantic_graph`; an unknown alias refuses `missing_declaration`/`missing-name`; an operation target naming an object type and an implementation relation lacking the operation's frame each refuse `invalid_model_binding`/`malformed-declaration`; omitted precondition or invariant references refuse the same at their aggregate; an integer-typed body expression refuses `ill_typed`/`non-boolean-root`, each at FR-477's locus and first applicable reader step. | Test |
| FR-351-AC-5 | Replacing `y`'s `self` and `i` parameter references with `x`'s nodes, while updating the exact dependencies, refuses `invalid_model_binding`/`malformed-declaration` at `y`'s `parameters`; a simultaneous earlier-stage defect wins before this semantic refusal. | Test |
| FR-351-AC-6 | A relation with two aliases `A`, `B` over one model root, first-use subjects `[A, B]` and `subject_declaration_order` `[B, A]` admits, and the reader's borrowed view returns `[B, A]` with each alias's root while `step_relation_body` still lists subjects `[A, B]`. | Test |
| FR-351-AC-7 | Writing the AC-6 package and reading its canonical bytes preserves the declaration order `[B, A]`. | Test |
| FR-351-AC-8 | A package whose body has only the four legacy bindings admits, its declaration-order view is absent, and its `step_relation_body` equals that of the same relation with the fifth binding present. | Test |
| FR-351-AC-9 | A wrong final binding name refuses `invalid_semantic_graph`, no cause, at `P`; a non-Group value at `P/value`; a malformed entry at `P/value/members/K`; a duplicate or unknown alias at that entry's `name`; a non-reference value at that entry's `value/target`; a sixth binding at `members/5`. | Test |
| FR-351-AC-10 | With two subjects over two distinct model roots, an entry whose reference is the other subject's already-selected root refuses `invalid_semantic_graph`, no cause, at that entry's `value/target`, while the correctly joined order is not refused there. | Test |
| FR-351-AC-11 | An omitted alias refuses at `P/value/members/LEN`, and a defect in a supplied entry is reported at its own pointer before an omission; of two defective entries the first supplied is reported. | Test |
| FR-351-AC-12 | A declaration-order join defect refuses at its entry pointer before a wrong operation target, while a wrong subject target refuses at the subject's pointer before a declaration-order defect; a non-aggregate `subjects` value together with a non-Group declaration value refuses at `/body/members/0/value`. | Test |
| FR-351-AC-13 | The declaration-order check's heap request and per-entry work are charged to the read budget at the relation node: the exact allowance is admitted and one unit short is denied at the request, and the work of the package with the binding exceeds that of the same package without it by the literal count of its entries plus the binding's own terms. | Test |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns V2 intake, limits,
  identity, generic refusal fields and writer rules.
- [FR-040](./FR-040-admit-frame-entries-and-state-clauses.md) owns the anchor
  and state-clause semantics consumed by the premise joins.
- [FR-346](./FR-346-admit-abstraction-relation-body.md) owns the typed
  implementation relation and its frame entries.
- [FR-349](./FR-349-retain-typed-checked-v2-terms.md) owns typed body retention.
- FSpec FR-477 and TC-962 define the step-relation vectors; FSpec FR-322,
  FR-341 and FR-402 define graph identity, binders and obligation identities.
