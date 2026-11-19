// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The FSpec FR-640 Reader rules 6 and 7 for the `claim`/`probabilistic_claim`
//! node: the nine bindings and what the claim reads (rule 6), then what it
//! states (rule 7, six checks in order over bindings 6 to 9).
//!
//! Implements: FR-377 "Behavior".

use filament_exact::Integer as Exact;

use super::super::operation_effects::own_operation;
use super::super::stage_storage::{self, Path, Refusal, Site};
use super::super::{
    CheckedNodeId, CheckedNodeKind, CheckedNodeOwner, CheckedTerm, ClaimForm, LiteralKind,
    ModelForm, ProbabilisticForm, ScalarTypeForm, TemporalForm, TimedModelForm, ValueForm,
    WorkMeter,
};
use super::{
    binding_at, binding_name, bound, invalid, members, node_at, target, text, visit, Join, FORM,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
};

/// The classes of probabilistic node a claim's reads name.
const REWARD: u8 = 0;
const DISTRIBUTION: u8 = 1;
const DELAY: u8 = 2;

/// One probabilistic node that a claim may read: its class, the identity of
/// the selected model it belongs to, its name (a reward's, else empty) and its
/// rank in ascending node-id order.
type Row<'a> = (u8, &'a str, &'a str, usize);

/// A `timed_model` root of one selected model.
struct Timed<'a> {
    identity: &'a str,
    dense: bool,
    unit: &'a CheckedNodeId,
}

/// The probabilistic nodes and timed roots by selected model, built once.
struct Index<'a> {
    rows: Vec<Row<'a>>,
    timed: Vec<Timed<'a>>,
}

/// The identity of the selected model a model node or model root belongs to.
fn identity_of<'a>(join: &Join<'a>, id: &CheckedNodeId) -> Option<&'a str> {
    let position = join.position(id)?;
    match join.kinds.get(position)? {
        CheckedNodeKind::Model(ModelForm::ModelRoot) => match join.nodes[position].owner.as_ref() {
            Some(CheckedNodeOwner::ModelRoot { identity }) => Some(identity),
            _ => None,
        },
        CheckedNodeKind::Model(_) => {
            let models: &'a super::RetainedModels = join.models;
            let owner = models.owner_by_key(&join.structural_keys[position])?;
            Some(&owner.package.identity)
        }
        _ => None,
    }
}

impl<'a> Index<'a> {
    fn build(join: &Join<'a>, meter: &mut WorkMeter) -> Result<Self, ValidationFailure> {
        let site = Site::new(super::PackageReadStage::Graph, 0, FORM, Path::root());
        let probabilistic = join
            .kinds
            .iter()
            .filter(|kind| matches!(kind, CheckedNodeKind::Probabilistic(_)))
            .count();
        let mut rows: Vec<Row<'a>> = stage_storage::vec(meter, &site, probabilistic)?;
        for (rank, &position) in join.by_id.iter().enumerate() {
            let class = match join.kinds[position] {
                CheckedNodeKind::Probabilistic(ProbabilisticForm::Reward) => REWARD,
                CheckedNodeKind::Probabilistic(ProbabilisticForm::RandomParameter) => DISTRIBUTION,
                CheckedNodeKind::Probabilistic(ProbabilisticForm::DelayDistribution) => DELAY,
                _ => continue,
            };
            let body = &join.terms[position];
            let Some(identity) = bound(body, 0)
                .and_then(target)
                .and_then(|model| identity_of(join, model))
            else {
                continue;
            };
            let name = if class == REWARD {
                bound(body, 2).and_then(text).unwrap_or_default()
            } else {
                ""
            };
            rows.push((class, identity, name, rank));
        }
        rows.sort_unstable();
        let roots = join
            .kinds
            .iter()
            .filter(|kind| **kind == CheckedNodeKind::TimedModel(TimedModelForm::Model))
            .count();
        let mut timed: Vec<Timed<'a>> = stage_storage::vec(meter, &site, roots)?;
        for &position in &join.by_id {
            if join.kinds[position] != CheckedNodeKind::TimedModel(TimedModelForm::Model) {
                continue;
            }
            let body = &join.terms[position];
            let Some(identity) = bound(body, 0)
                .and_then(target)
                .and_then(|model| identity_of(join, model))
            else {
                continue;
            };
            let Some(source) = bound(body, 2)
                .and_then(target)
                .and_then(|source| join.position(source))
            else {
                continue;
            };
            let source = &join.terms[source];
            let Some(unit) = bound(source, 1).and_then(target) else {
                continue;
            };
            timed.push(Timed {
                identity,
                dense: bound(source, 0).and_then(text) == Some("dense"),
                unit,
            });
        }
        Ok(Self { rows, timed })
    }

    /// The rows of `class` of the model `identity`, and for a reward the name
    /// `name`, in ascending node-id order.
    fn group(&self, class: u8, identity: &str, name: &str) -> &[Row<'a>] {
        let start = self
            .rows
            .partition_point(|row| (row.0, row.1, row.2) < (class, identity, name));
        let end = self
            .rows
            .partition_point(|row| (row.0, row.1, row.2) <= (class, identity, name));
        &self.rows[start..end]
    }

    fn time(&self, identity: &str) -> Option<&Timed<'a>> {
        self.timed.iter().find(|root| root.identity == identity)
    }
}

/// Checks every `claim`/`probabilistic_claim` node in ascending node-id order,
/// rule 6 and then rule 7 on each, and refuses the first failure.
pub(super) fn validate(join: &Join<'_>, meter: &mut WorkMeter) -> Result<(), ValidationFailure> {
    let claim = CheckedNodeKind::Claim(ClaimForm::ProbabilisticClaim);
    if !join.kinds.contains(&claim) {
        return Ok(());
    }
    let index = Index::build(join, meter)?;
    for &position in &join.by_id {
        if join.kinds[position] == claim {
            Check {
                join,
                index: &index,
                position,
            }
            .run(meter)?;
        }
    }
    Ok(())
}

/// The names of the nine bindings, in order.
const BINDINGS: [&str; 9] = [
    "subject",
    "scheduler",
    "reward",
    "distributions",
    "delays",
    "form",
    "over",
    "activation",
    "confidence",
];

struct Check<'j, 'a> {
    join: &'j Join<'a>,
    index: &'j Index<'a>,
    position: usize,
}

/// What a reward's `value_type` makes of a threshold on the reward.
#[derive(Clone, Copy)]
enum RewardType<'a> {
    /// An integer reward: a `rational` threshold.
    Integer,
    /// A quantity reward: a `quantity` at this unit node.
    Quantity(&'a CheckedNodeId),
}

impl<'j, 'a> Check<'j, 'a> {
    fn run(&self, meter: &mut WorkMeter) -> Result<(), ValidationFailure> {
        let position = self.position;
        visit(meter, position, node_at(position))?;
        let body = &self.join.terms[position];
        let shaped = matches!(body, CheckedTerm::Aggregate { .. })
            && members(body).len() == BINDINGS.len()
            && members(body)
                .iter()
                .zip(BINDINGS)
                .all(|(member, name)| binding_name(member) == Some(name));
        if !shaped {
            return Err(invalid(meter, position, node_at(position).key("body")));
        }
        let subject = self.subject(meter, body)?;
        self.scheduler(meter, body, subject)?;
        let rewards = self.rewards(meter, body, subject)?;
        self.exact_set(meter, body, 3, DISTRIBUTION, subject, "")?;
        self.exact_set(meter, body, 4, DELAY, subject, "")?;
        self.statement(meter, body, subject, rewards)
    }

    /// Rule 6, `subject`: a `reference` to a `model`/`model_root` node.
    fn subject(
        &self,
        meter: &mut WorkMeter,
        body: &CheckedTerm,
    ) -> Result<&'a str, ValidationFailure> {
        let join = self.join;
        let root = bound(body, 0)
            .and_then(target)
            .and_then(|id| join.position(id))
            .filter(|&root| join.kinds[root] == CheckedNodeKind::Model(ModelForm::ModelRoot));
        match root.and_then(|root| join.nodes[root].owner.as_ref()) {
            Some(CheckedNodeOwner::ModelRoot { identity }) => Ok(identity),
            _ => Err(invalid(
                meter,
                self.position,
                binding_at(self.position, 0).key("target"),
            )),
        }
    }

    /// Rule 6, `scheduler`: a workload of the subject's model or the
    /// aggregate of exactly one `fairness` binding.
    fn scheduler(
        &self,
        meter: &mut WorkMeter,
        body: &CheckedTerm,
        subject: &str,
    ) -> Result<(), ValidationFailure> {
        let join = self.join;
        let position = self.position;
        let at = binding_at(position, 1);
        match bound(body, 1) {
            Some(CheckedTerm::Reference { target: id }) => {
                let model = join
                    .position(id)
                    .filter(|&workload| {
                        join.kinds[workload]
                            == CheckedNodeKind::Probabilistic(ProbabilisticForm::Workload)
                    })
                    .and_then(|workload| bound(&join.terms[workload], 0))
                    .and_then(target)
                    .and_then(|model| identity_of(join, model));
                if model != Some(subject) {
                    return Err(invalid(meter, position, at.key("target")));
                }
                Ok(())
            }
            Some(term @ CheckedTerm::Aggregate { .. })
                if members(term).len() == 1
                    && binding_name(&members(term)[0]) == Some("fairness") =>
            {
                let Some(set) =
                    bound(term, 0).filter(|set| matches!(set, CheckedTerm::Aggregate { .. }))
                else {
                    return Err(invalid(meter, position, at));
                };
                let set_at = at.key("members").index(0).key("value");
                self.fairness(meter, set, set_at, subject)
            }
            _ => Err(invalid(meter, position, at)),
        }
    }

    /// Rule 6, the fairness members: each a `kind`, `granularity`, `declaring`
    /// and `operation` of the subject's model, in strictly ascending order.
    fn fairness(
        &self,
        meter: &mut WorkMeter,
        set: &CheckedTerm,
        set_at: Path<'static>,
        subject: &str,
    ) -> Result<(), ValidationFailure> {
        let join = self.join;
        let position = self.position;
        let mut previous: Option<(&CheckedNodeId, &str, &str, &str)> = None;
        for (row, member) in members(set).iter().enumerate() {
            let member_at = set_at.key("members").index(row);
            visit(meter, position, member_at)?;
            let shaped = matches!(member, CheckedTerm::Aggregate { .. })
                && members(member).len() == 4
                && members(member)
                    .iter()
                    .zip(["kind", "granularity", "declaring", "operation"])
                    .all(|(part, name)| binding_name(part) == Some(name));
            let parts = shaped
                .then(|| {
                    Some((
                        bound(member, 0)
                            .and_then(text)
                            .filter(|kind| matches!(*kind, "weak" | "strong"))?,
                        bound(member, 1)
                            .and_then(text)
                            .filter(|g| matches!(*g, "whole" | "each"))?,
                        bound(member, 2).and_then(target)?,
                        bound(member, 3).and_then(text)?,
                    ))
                })
                .flatten();
            let Some((kind, granularity, declaring, operation)) = parts else {
                return Err(invalid(meter, position, member_at));
            };
            let declared = join
                .position(declaring)
                .filter(|&at| matches!(join.kinds[at], CheckedNodeKind::Model(_)))
                .and_then(|at| join.models.owner_by_key(&join.structural_keys[at]))
                .filter(|owner| {
                    super::is_operation_owner(owner.form) && &*owner.package.identity == subject
                })
                .and_then(|owner| owner.package.object_types.get(owner.node))
                .and_then(|declared| own_operation(declared, operation));
            // Plain node-id order, as the join's own `by_id` sort already runs
            // over every node of the package (QSL-2554 review FND-003).
            let key = (declaring, operation, kind, granularity);
            if declared.is_none() || previous.is_some_and(|earlier| earlier >= key) {
                return Err(invalid(meter, position, member_at));
            }
            previous = Some(key);
        }
        Ok(())
    }

    /// Rule 6, `reward`: the reward nodes of the subject's model that carry
    /// the one reward name the claim reads, or none. The reward type of the
    /// first member, when there is one.
    fn rewards(
        &self,
        meter: &mut WorkMeter,
        body: &CheckedTerm,
        subject: &str,
    ) -> Result<Option<RewardType<'a>>, ValidationFailure> {
        let join = self.join;
        let first = members(bound(body, 2).unwrap_or(body))
            .first()
            .and_then(target)
            .and_then(|id| join.position(id))
            .filter(|&reward| {
                join.kinds[reward] == CheckedNodeKind::Probabilistic(ProbabilisticForm::Reward)
            });
        let name = first
            .and_then(|reward| bound(&join.terms[reward], 2))
            .and_then(text)
            .unwrap_or_default();
        self.exact_set(meter, body, 2, REWARD, subject, name)?;
        Ok(first.and_then(|reward| self.reward_type(reward)))
    }

    /// The type a reward's threshold has.
    fn reward_type(&self, reward: usize) -> Option<RewardType<'a>> {
        let join = self.join;
        let value_type = bound(&join.terms[reward], 3).and_then(target)?;
        let at = join.position(value_type)?;
        match join.kinds[at] {
            CheckedNodeKind::ScalarType(ScalarTypeForm::Integer)
            | CheckedNodeKind::BoundedDomain(super::BoundedDomainForm::IntegerRange) => {
                Some(RewardType::Integer)
            }
            CheckedNodeKind::ScalarType(ScalarTypeForm::Unit | ScalarTypeForm::CompoundUnit) => {
                Some(RewardType::Quantity(&join.nodes[at].node_id))
            }
            CheckedNodeKind::ScalarType(
                ScalarTypeForm::CarriedQuantity | ScalarTypeForm::BoundedQuantity,
            ) => members(&join.terms[at])
                .iter()
                .find(|member| binding_name(member) == Some("unit"))
                .and_then(|member| match member {
                    CheckedTerm::Binding { value, .. } => target(value),
                    _ => None,
                })
                .map(RewardType::Quantity),
            _ => None,
        }
    }

    /// Rule 6, a set binding: `references` at binding `at` are exactly the
    /// nodes of `class` of the subject's model (and `name`), in ascending
    /// node-id order. One visit is charged for each member, ahead of it.
    fn exact_set(
        &self,
        meter: &mut WorkMeter,
        body: &CheckedTerm,
        at: usize,
        class: u8,
        subject: &str,
        name: &str,
    ) -> Result<(), ValidationFailure> {
        let join = self.join;
        let position = self.position;
        let set_at = binding_at(position, at);
        let Some(CheckedTerm::Aggregate { members: listed }) = bound(body, at) else {
            return Err(invalid(meter, position, set_at));
        };
        let expected = self.index.group(class, subject, name);
        let mut exact = listed.len() == expected.len();
        for (row, member) in listed.iter().enumerate() {
            visit(meter, position, set_at.key("members").index(row))?;
            exact &= target(member).is_some_and(|id| {
                expected
                    .get(row)
                    .is_some_and(|expected| join.nodes[join.by_id[expected.3]].node_id == *id)
            });
        }
        if exact {
            Ok(())
        } else {
            Err(invalid(meter, position, set_at))
        }
    }

    /// Rule 7 over bindings 6 to 9.
    fn statement(
        &self,
        meter: &mut WorkMeter,
        body: &CheckedTerm,
        subject: &str,
        reward: Option<RewardType<'a>>,
    ) -> Result<(), ValidationFailure> {
        let site = Site::new(
            super::PackageReadStage::Graph,
            self.position,
            FORM,
            Path::root(),
        );
        let items = stage_storage::vec(meter, &site, ITEMS)?;
        let mut parse = Parse {
            check: self,
            meter,
            items,
            reads_reward: false,
            theta: None,
            reward,
            time: self.index.time(subject),
        };
        parse.run(body)?;
        let Parse {
            items,
            reads_reward,
            reward: held,
            time,
            ..
        } = parse;
        let position = self.position;
        // 2. Time.
        for item in &items {
            if let Item::Time { at, dense } = item {
                if time.is_none_or(|root| *dense && !root.dense) {
                    return Err(invalid(meter, position, *at));
                }
            }
        }
        // 3. Exact values.
        for item in &items {
            if let Item::Exact {
                term, at, expect, ..
            } = item
            {
                if self.exact_defect(term, expect) {
                    return Err(invalid(meter, position, *at));
                }
            }
        }
        // 4. Ranges.
        for item in &items {
            if let Item::Exact {
                term,
                at,
                range,
                theta,
                ..
            } = item
            {
                if self.range_defect(meter, term, *range, *theta)? {
                    return Err(invalid(meter, position, *at));
                }
            }
        }
        // 5. Targets.
        for item in &items {
            if let Item::Target { id, at, slot } = item {
                self.target_defect(meter, id, *at, *slot)?;
            }
        }
        // 6. Reward.
        let named = members(bound(body, 2).unwrap_or(body)).is_empty();
        if named == reads_reward {
            return Err(invalid(meter, position, binding_at(position, 2)));
        }
        let _ = held;
        Ok(())
    }

    fn kind_of(&self, id: &CheckedNodeId) -> Option<CheckedNodeKind> {
        self.join
            .position(id)
            .map(|position| self.join.kinds[position])
    }

    /// Whether a value is not what its position names (rule 7, check 3).
    fn exact_defect(&self, term: &CheckedTerm, expect: &Expect<'_>) -> bool {
        let CheckedTerm::Literal {
            type_id,
            value_kind,
            ..
        } = term
        else {
            return true;
        };
        let typed =
            |form: ScalarTypeForm| self.kind_of(type_id) == Some(CheckedNodeKind::ScalarType(form));
        match expect {
            Expect::Any => !matches!(
                value_kind,
                LiteralKind::Integer | LiteralKind::Rational | LiteralKind::Quantity
            ),
            Expect::Rational => {
                *value_kind != LiteralKind::Rational || !typed(ScalarTypeForm::Rational)
            }
            Expect::Integer => {
                *value_kind != LiteralKind::Integer || !typed(ScalarTypeForm::Integer)
            }
            Expect::Quantity(unit) => *value_kind != LiteralKind::Quantity || type_id != *unit,
        }
    }

    /// Whether a value is out of its range (rule 7, check 4).
    fn range_defect(
        &self,
        meter: &mut WorkMeter,
        term: &CheckedTerm,
        range: Range,
        theta: Option<&CheckedTerm>,
    ) -> Result<bool, ValidationFailure> {
        let spelled = |term: &CheckedTerm| match term {
            CheckedTerm::Literal { value, .. } => value.as_str().map(str::to_owned),
            _ => None,
        };
        let Some(text) = spelled(term) else {
            return Ok(true);
        };
        let zero = Exact::from(0_i64);
        match range {
            Range::None => Ok(false),
            Range::NonNegative => Ok(text.starts_with('-')),
            Range::Open01 => {
                let Some((numerator, denominator)) = meter.canonical_rational(&text)? else {
                    return Ok(true);
                };
                Ok(!(numerator > zero && numerator < denominator))
            }
            Range::Indifference => {
                let Some((numerator, denominator)) = meter.canonical_rational(&text)? else {
                    return Ok(true);
                };
                if !(numerator > zero) {
                    return Ok(true);
                }
                let Some(theta) = theta.and_then(spelled) else {
                    return Ok(false);
                };
                // θ − ι <= 0 or θ + ι >= 1, with θ = a/b and ι = c/d.
                let Some((a, b)) = meter.canonical_rational(&theta)? else {
                    return Ok(true);
                };
                let (c, d) = (numerator, denominator);
                let left = meter.exact_mul(&a, &d)?;
                let right = meter.exact_mul(&c, &b)?;
                let whole = meter.exact_mul(&b, &d)?;
                let sum = meter.exact_add(&left, &right)?;
                Ok(left <= right || sum >= whole)
            }
        }
    }

    /// Rule 7, check 5: the target of a predicate, formula or parameter slot.
    fn target_defect(
        &self,
        meter: &mut WorkMeter,
        id: &CheckedNodeId,
        at: Path<'static>,
        slot: Slot,
    ) -> Result<(), ValidationFailure> {
        let join = self.join;
        let found = join.position(id);
        let kind = found.map(|found| join.kinds[found]);
        let mut refused = |code: Code, cause: Cause| {
            let site = Site::new(super::PackageReadStage::Graph, self.position, FORM, at);
            stage_storage::refuse(
                meter,
                &site,
                Refusal::new(code, at).because(cause).at_node(id),
            )
        };
        let admitted = match slot {
            Slot::Predicate => {
                matches!(
                    kind,
                    Some(
                        CheckedNodeKind::Expression(_) | CheckedNodeKind::Value(ValueForm::Literal)
                    )
                ) && found.is_some_and(|found| {
                    self.kind_of(&join.nodes[found].semantic_type)
                        == Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean))
                })
            }
            Slot::Formula => kind == Some(CheckedNodeKind::Temporal(TemporalForm::Formula)),
            Slot::Parameter => kind == Some(CheckedNodeKind::Value(ValueForm::Parameter)),
        };
        if admitted {
            return Ok(());
        }
        Err(match slot {
            Slot::Parameter => refused(Code::InvalidModelBinding, Cause::MalformedDeclaration),
            _ => refused(Code::IllTyped, Cause::OperatorIneligible),
        })
    }
}

/// The number of items a statement holds at most.
const ITEMS: usize = 32;

/// What a position takes of an exact value.
enum Expect<'a> {
    /// A `rational` at the builtin `scalar_type`/`rational` node.
    Rational,
    /// An `integer` at the builtin Integer node.
    Integer,
    /// A `quantity` at this unit node.
    Quantity(&'a CheckedNodeId),
    /// Any exact value: the reward the threshold reads is unnamed, and
    /// check 6 refuses the claim.
    Any,
}

/// The range an exact value must lie in.
#[derive(Clone, Copy)]
enum Range {
    None,
    /// Strictly between 0 and 1.
    Open01,
    /// Not negative.
    NonNegative,
    Indifference,
}

/// Where a reference stands.
#[derive(Clone, Copy)]
enum Slot {
    /// A `from`, `until`, `predicate`, `left`, `right` or `guard`.
    Predicate,
    /// The `formula` of an event.
    Formula,
    /// An `over` or `trigger`.
    Parameter,
}

/// One thing rule 7 checks, in body order.
enum Item<'a> {
    /// Check 2: the claim reads model time (`dense` when `dense`).
    Time { at: Path<'static>, dense: bool },
    /// Checks 3 and 4: an exact value.
    Exact {
        term: &'a CheckedTerm,
        at: Path<'static>,
        expect: Expect<'a>,
        range: Range,
        theta: Option<&'a CheckedTerm>,
    },
    /// Check 5: a reference.
    Target {
        id: &'a CheckedNodeId,
        at: Path<'static>,
        slot: Slot,
    },
}

/// The kind of a measure.
#[derive(Clone, Copy, Eq, PartialEq)]
enum Measure {
    Accumulate,
    Steps,
    Elapsed,
    Fraction,
}

/// The first check of rule 7, over bindings 6 to 9 in body order: the shape.
/// One validation visit is charged for each of the four bindings and each
/// binding nested in them, ahead of inspecting it.
struct Parse<'c, 'm, 'b, 'j, 'a> {
    check: &'c Check<'j, 'a>,
    meter: &'m mut WorkMeter<'b>,
    items: Vec<Item<'a>>,
    reads_reward: bool,
    /// The probability a confidence's `indifference` must keep clear of 0 and 1.
    theta: Option<&'a CheckedTerm>,
    reward: Option<RewardType<'a>>,
    time: Option<&'c Timed<'a>>,
}

type Step<T> = Result<T, ValidationFailure>;

impl<'c, 'm, 'b, 'j, 'a> Parse<'c, 'm, 'b, 'j, 'a>
where
    'j: 'a,
{
    fn bad(&mut self, at: Path<'static>) -> ValidationFailure {
        invalid(self.meter, self.check.position, at)
    }

    fn charge(&mut self, at: Path<'static>) -> Step<()> {
        visit(self.meter, self.check.position, at)
    }

    #[ix_trace_rs::string_edge]
    fn run(&mut self, body: &'a CheckedTerm) -> Step<()> {
        let position = self.check.position;
        // 6. `form`.
        let value = bound(body, 5).expect("nine bindings");
        self.charge(node_at(position).key("body").key("members").index(5))?;
        self.form(value, binding_at(position, 5))?;
        // 7. `over`.
        self.charge(node_at(position).key("body").key("members").index(6))?;
        let value = bound(body, 6).expect("nine bindings");
        let at = binding_at(position, 6);
        match value {
            CheckedTerm::Reference { target } => self.items.push(Item::Target {
                id: target,
                at: at.key("target"),
                slot: Slot::Parameter,
            }),
            CheckedTerm::Aggregate { members } if members.is_empty() => {}
            _ => return Err(self.bad(at)),
        }
        // 8. `activation`.
        self.charge(node_at(position).key("body").key("members").index(7))?;
        let value = bound(body, 7).expect("nine bindings");
        let at = binding_at(position, 7);
        let (parts, kind) = self.open(value, at)?;
        let kind_at = at.key("members").index(0).key("value");
        match kind {
            "origin" => self.end(parts, 1, at)?,
            "each" => {
                let (trigger, trigger_at) = self.slot(parts, 1, "trigger", at)?;
                self.target(trigger, trigger_at, Slot::Parameter)?;
                let (guard, guard_at) = self.slot(parts, 2, "guard", at)?;
                self.target(guard, guard_at, Slot::Predicate)?;
                self.end(parts, 3, at)?;
            }
            _ => return Err(self.bad(kind_at)),
        }
        // 9. `confidence`.
        self.charge(node_at(position).key("body").key("members").index(8))?;
        let value = bound(body, 8).expect("nine bindings");
        let at = binding_at(position, 8);
        match value {
            CheckedTerm::Aggregate { members } if members.is_empty() => {}
            CheckedTerm::Aggregate { members: parts } => {
                for (row, (name, role)) in [
                    ("alpha", Range::Open01),
                    ("beta", Range::Open01),
                    ("indifference", Range::Indifference),
                ]
                .into_iter()
                .enumerate()
                {
                    let (value, value_at) = self.slot(parts, row, name, at)?;
                    let theta = self.theta;
                    self.items.push(Item::Exact {
                        term: value,
                        at: value_at,
                        expect: Expect::Rational,
                        range: role,
                        theta: if name == "indifference" { theta } else { None },
                    });
                }
                self.end(parts, 3, at)?;
            }
            _ => return Err(self.bad(at)),
        }
        Ok(())
    }

    /// Opens an aggregate whose first binding is `kind`, a text literal:
    /// charges and reads that binding and returns the members and the kind.
    fn open(
        &mut self,
        term: &'a CheckedTerm,
        at: Path<'static>,
    ) -> Step<(&'a [CheckedTerm], &'a str)> {
        let CheckedTerm::Aggregate { members: parts } = term else {
            return Err(self.bad(at));
        };
        let Some(first) = parts.first() else {
            return Err(self.bad(at));
        };
        self.charge(at.key("members").index(0))?;
        if binding_name(first) != Some("kind") {
            return Err(self.bad(at));
        }
        let kind_at = at.key("members").index(0).key("value");
        match bound(term, 0).and_then(text) {
            Some(kind) => Ok((parts, kind)),
            None => Err(self.bad(kind_at)),
        }
    }

    /// Charges and reads the binding `row` of `parts`, which must be `name`.
    fn slot(
        &mut self,
        parts: &'a [CheckedTerm],
        row: usize,
        name: &str,
        at: Path<'static>,
    ) -> Step<(&'a CheckedTerm, Path<'static>)> {
        let Some(part) = parts.get(row) else {
            return Err(self.bad(at));
        };
        self.charge(at.key("members").index(row))?;
        match part {
            CheckedTerm::Binding { value, .. } if binding_name(part) == Some(name) => {
                Ok((value, at.key("members").index(row).key("value")))
            }
            _ => Err(self.bad(at)),
        }
    }

    /// Refuses a binding beyond the `count` the aggregate holds.
    fn end(&mut self, parts: &'a [CheckedTerm], count: usize, at: Path<'static>) -> Step<()> {
        if parts.len() > count {
            self.charge(at.key("members").index(count))?;
            return Err(self.bad(at));
        }
        Ok(())
    }

    /// A reference to a state predicate, a formula or a parameter.
    fn target(&mut self, term: &'a CheckedTerm, at: Path<'static>, slot: Slot) -> Step<()> {
        match term {
            CheckedTerm::Reference { target } => {
                self.items.push(Item::Target {
                    id: target,
                    at: at.key("target"),
                    slot,
                });
                Ok(())
            }
            _ => Err(self.bad(at)),
        }
    }

    fn time_item(&mut self, at: Path<'static>, dense: bool) {
        self.items.push(Item::Time { at, dense });
    }

    fn exact(
        &mut self,
        term: &'a CheckedTerm,
        at: Path<'static>,
        expect: Expect<'a>,
        range: Range,
    ) {
        self.items.push(Item::Exact {
            term,
            at,
            expect,
            range,
            theta: None,
        });
    }

    /// The threshold type of a bound on `measure`.
    fn threshold(&self, measure: Measure) -> (Expect<'a>, Range) {
        match measure {
            Measure::Steps => (Expect::Integer, Range::NonNegative),
            Measure::Fraction => (Expect::Rational, Range::Open01),
            Measure::Accumulate => (
                match self.reward {
                    Some(RewardType::Integer) => Expect::Rational,
                    Some(RewardType::Quantity(unit)) => Expect::Quantity(unit),
                    None => Expect::Any,
                },
                Range::None,
            ),
            Measure::Elapsed => (
                self.time
                    .map_or(Expect::Any, |root| Expect::Quantity(root.unit)),
                Range::None,
            ),
        }
    }

    /// A `bound`: its `direction` and its `value`, of the type `expect`.
    /// Returns the value, so a form can keep it as its probability.
    fn bound(
        &mut self,
        term: &'a CheckedTerm,
        at: Path<'static>,
        expect: Expect<'a>,
        range: Range,
    ) -> Step<&'a CheckedTerm> {
        let CheckedTerm::Aggregate { members: parts } = term else {
            return Err(self.bad(at));
        };
        let (direction, direction_at) = self.slot(parts, 0, "direction", at)?;
        if !matches!(direction, CheckedTerm::Literal { .. })
            || !matches!(text(direction), Some("at_least" | "at_most"))
        {
            return Err(self.bad(direction_at));
        }
        let (value, value_at) = self.slot(parts, 1, "value", at)?;
        self.exact(value, value_at, expect, range);
        self.end(parts, 2, at)?;
        Ok(value)
    }

    #[ix_trace_rs::string_edge]
    fn form(&mut self, value: &'a CheckedTerm, at: Path<'static>) -> Step<()> {
        let (parts, kind) = self.open(value, at)?;
        let kind_at = at.key("members").index(0).key("value");
        match kind {
            "probability" => {
                let (event, event_at) = self.slot(parts, 1, "event", at)?;
                let (event_parts, event_kind) = self.open(event, event_at)?;
                let event_kind_at = event_at.key("members").index(0).key("value");
                match event_kind {
                    "formula" => {
                        let (formula, formula_at) =
                            self.slot(event_parts, 1, "formula", event_at)?;
                        self.target(formula, formula_at, Slot::Formula)?;
                        self.end(event_parts, 2, event_at)?;
                    }
                    "compare" => {
                        let (measure, measure_at) =
                            self.slot(event_parts, 1, "measure", event_at)?;
                        let measured = self.measure(measure, measure_at)?;
                        let (bound, bound_at) = self.slot(event_parts, 2, "bound", event_at)?;
                        let (expect, range) = self.threshold(measured);
                        self.bound(bound, bound_at, expect, range)?;
                        self.end(event_parts, 3, event_at)?;
                    }
                    _ => return Err(self.bad(event_kind_at)),
                }
                let (bound, bound_at) = self.slot(parts, 2, "bound", at)?;
                self.probability(bound, bound_at)?;
                self.end(parts, 3, at)
            }
            "quantile" => {
                let (level, level_at) = self.slot(parts, 1, "level", at)?;
                self.exact(level, level_at, Expect::Rational, Range::Open01);
                self.theta = Some(level);
                let (measure, measure_at) = self.slot(parts, 2, "measure", at)?;
                let measured = self.measure(measure, measure_at)?;
                if measured == Measure::Fraction {
                    return Err(self.bad(measure_at));
                }
                let (bound, bound_at) = self.slot(parts, 3, "bound", at)?;
                let (expect, range) = self.threshold(measured);
                self.bound(bound, bound_at, expect, range)?;
                self.end(parts, 4, at)
            }
            "mean_of_fraction" => {
                let (measure, measure_at) = self.slot(parts, 1, "measure", at)?;
                if self.measure(measure, measure_at)? != Measure::Fraction {
                    return Err(self.bad(measure_at));
                }
                let (bound, bound_at) = self.slot(parts, 2, "bound", at)?;
                self.probability(bound, bound_at)?;
                self.end(parts, 3, at)
            }
            "long_run_fraction" => {
                let (predicate, predicate_at) = self.slot(parts, 1, "predicate", at)?;
                self.target(predicate, predicate_at, Slot::Predicate)?;
                let (weighting, weighting_at) = self.slot(parts, 2, "weighting", at)?;
                self.weighting(weighting, weighting_at)?;
                let (bound, bound_at) = self.slot(parts, 3, "bound", at)?;
                self.probability(bound, bound_at)?;
                self.end(parts, 4, at)
            }
            "reach" => {
                let (reach, reach_at) = self.slot(parts, 1, "target", at)?;
                let (reach_parts, reach_kind) = self.open(reach, reach_at)?;
                let reach_kind_at = reach_at.key("members").index(0).key("value");
                match reach_kind {
                    "eventually" | "always" => {
                        let (predicate, predicate_at) =
                            self.slot(reach_parts, 1, "predicate", reach_at)?;
                        self.target(predicate, predicate_at, Slot::Predicate)?;
                        self.end(reach_parts, 2, reach_at)?;
                    }
                    "until" => {
                        let (left, left_at) = self.slot(reach_parts, 1, "left", reach_at)?;
                        self.target(left, left_at, Slot::Predicate)?;
                        let (right, right_at) = self.slot(reach_parts, 2, "right", reach_at)?;
                        self.target(right, right_at, Slot::Predicate)?;
                        self.end(reach_parts, 3, reach_at)?;
                    }
                    _ => return Err(self.bad(reach_kind_at)),
                }
                let (bound, bound_at) = self.slot(parts, 2, "bound", at)?;
                self.probability(bound, bound_at)?;
                self.end(parts, 3, at)
            }
            "expected_reward" | "expected_elapsed" => {
                let elapsed = kind == "expected_elapsed";
                if elapsed {
                    self.time_item(kind_at, false);
                } else {
                    self.reads_reward = true;
                }
                let (until, until_at) = self.slot(parts, 1, "until", at)?;
                self.target(until, until_at, Slot::Predicate)?;
                let (bound, bound_at) = self.slot(parts, 2, "bound", at)?;
                let (expect, range) = self.threshold(if elapsed {
                    Measure::Elapsed
                } else {
                    Measure::Accumulate
                });
                self.bound(bound, bound_at, expect, range)?;
                self.end(parts, 3, at)
            }
            _ => Err(self.bad(kind_at)),
        }
    }

    /// The `bound` of a form whose threshold is a probability.
    fn probability(&mut self, term: &'a CheckedTerm, at: Path<'static>) -> Step<()> {
        let value = self.bound(term, at, Expect::Rational, Range::Open01)?;
        self.theta = Some(value);
        Ok(())
    }

    #[ix_trace_rs::string_edge]
    fn measure(&mut self, term: &'a CheckedTerm, at: Path<'static>) -> Step<Measure> {
        let (parts, kind) = self.open(term, at)?;
        let kind_at = at.key("members").index(0).key("value");
        let measure = match kind {
            "accumulate" => Measure::Accumulate,
            "steps" => Measure::Steps,
            "elapsed" => Measure::Elapsed,
            "fraction" => Measure::Fraction,
            _ => return Err(self.bad(kind_at)),
        };
        match measure {
            Measure::Fraction => {
                let (predicate, predicate_at) = self.slot(parts, 1, "predicate", at)?;
                self.target(predicate, predicate_at, Slot::Predicate)?;
                let (window, window_at) = self.slot(parts, 2, "window", at)?;
                self.window(window, window_at);
                let (weighting, weighting_at) = self.slot(parts, 3, "weighting", at)?;
                self.weighting(weighting, weighting_at)?;
                self.end(parts, 4, at)?;
            }
            _ => {
                match measure {
                    Measure::Accumulate => self.reads_reward = true,
                    Measure::Elapsed => self.time_item(kind_at, false),
                    _ => {}
                }
                let (from, from_at) = self.slot(parts, 1, "from", at)?;
                self.target(from, from_at, Slot::Predicate)?;
                let (until, until_at) = self.slot(parts, 2, "until", at)?;
                self.target(until, until_at, Slot::Predicate)?;
                let (within, within_at) = self.slot(parts, 3, "within", at)?;
                self.window(within, within_at);
                self.end(parts, 4, at)?;
            }
        }
        Ok(measure)
    }

    /// A `within` or `window`: an integer over an untimed subject and a time
    /// quantity over a timed one.
    fn window(&mut self, term: &'a CheckedTerm, at: Path<'static>) {
        let expect = self
            .time
            .map_or(Expect::Integer, |root| Expect::Quantity(root.unit));
        self.exact(term, at, expect, Range::NonNegative);
    }

    #[ix_trace_rs::string_edge]
    fn weighting(&mut self, term: &'a CheckedTerm, at: Path<'static>) -> Step<()> {
        let (parts, kind) = self.open(term, at)?;
        let kind_at = at.key("members").index(0).key("value");
        match kind {
            "none" => {}
            "reward" => self.reads_reward = true,
            "delay" => self.time_item(kind_at, true),
            _ => return Err(self.bad(kind_at)),
        }
        self.end(parts, 1, at)
    }
}
