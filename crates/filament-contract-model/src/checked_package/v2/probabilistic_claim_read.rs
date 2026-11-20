// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The typed read of FSpec FR-640 "Typed read" for the claim node: what each
//! `claim`/`probabilistic_claim` node reads and states, retained at admission.
//!
//! Implements: FR-377 "Public claim read".

use std::sync::Arc;

use num_bigint::BigInt;

use super::native_join_budget::{NativeJoinBudget, PackageReadStage};
use super::probabilistic_join::{bound, members, target, text};
use super::{
    CheckedFairnessGranularity, CheckedFairnessKind, CheckedNodeId, CheckedNodeKind,
    CheckedPackageV2, CheckedRational, CheckedSemanticNodeV2, CheckedTerm, ClaimForm, WorkMeter,
};
use crate::checked_package::common::ValidationFailure;

/// One constraint of an `every scheduler` claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FairnessMember {
    /// `weak` or `strong`.
    pub kind: CheckedFairnessKind,
    /// `whole` or `each`.
    pub granularity: CheckedFairnessGranularity,
    /// The node that declares the operation.
    pub declaring: CheckedNodeId,
    /// The operation's name.
    pub operation: Box<str>,
}

/// The scheduler a claim ranges over.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProbabilisticScheduler {
    /// `under W`: the `workload` node.
    Workload(CheckedNodeId),
    /// `under every scheduler` with its fairness set, in body order.
    Every(Vec<FairnessMember>),
}

/// An exact value of a claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProbabilisticExact {
    /// An `integer` literal.
    Integer(BigInt),
    /// A `rational` literal.
    Rational(CheckedRational),
    /// A `quantity` literal: its magnitude in the unit node `unit`.
    Quantity {
        /// The unit node the quantity is typed at.
        unit: CheckedNodeId,
        /// The exact magnitude.
        magnitude: CheckedRational,
    },
}

/// The direction of a threshold.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundDirection {
    /// `>=`.
    AtLeast,
    /// `<=`.
    AtMost,
}

/// A threshold with its direction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProbabilisticBound {
    /// The direction.
    pub direction: BoundDirection,
    /// The exact value.
    pub value: ProbabilisticExact,
}

/// How a fraction or long-run fraction is weighted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProbabilisticWeighting {
    /// Unweighted.
    None,
    /// `weighted by R`, the reward the claim's `rewards` name.
    Reward,
    /// `weighted by delay`.
    Delay,
}

/// A bounded measure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProbabilisticMeasure {
    /// `accumulate R from .. until .. within h`.
    Accumulate {
        /// The `from` predicate node.
        from: CheckedNodeId,
        /// The `until` predicate node.
        until: CheckedNodeId,
        /// The horizon.
        within: ProbabilisticExact,
    },
    /// `steps from .. until .. within h`.
    Steps {
        /// The `from` predicate node.
        from: CheckedNodeId,
        /// The `until` predicate node.
        until: CheckedNodeId,
        /// The horizon.
        within: ProbabilisticExact,
    },
    /// `elapsed from .. until .. within h`.
    Elapsed {
        /// The `from` predicate node.
        from: CheckedNodeId,
        /// The `until` predicate node.
        until: CheckedNodeId,
        /// The horizon.
        within: ProbabilisticExact,
    },
    /// A fraction over a window.
    Fraction {
        /// The predicate node.
        predicate: CheckedNodeId,
        /// The window.
        window: ProbabilisticExact,
        /// The weighting.
        weighting: ProbabilisticWeighting,
    },
}

/// The event of a `probability` form.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::large_enum_variant)] // the shape FR-377 states
pub enum ProbabilisticEvent {
    /// A temporal formula node.
    Formula(CheckedNodeId),
    /// `M <= c` or `M >= c`.
    Compare {
        /// The measure.
        measure: ProbabilisticMeasure,
        /// The comparison.
        bound: ProbabilisticBound,
    },
}

/// The target of a `reach` form.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProbabilisticReachTarget {
    /// `eventually holds(P)`.
    Eventually(CheckedNodeId),
    /// `always holds(P)`.
    Always(CheckedNodeId),
    /// `holds(A) until holds(B)`.
    Until(CheckedNodeId, CheckedNodeId),
}

/// A claim's form (FSpec FR-640 binding 6). Named apart from the node form
/// `ProbabilisticForm` of the declarations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProbabilisticClaimForm {
    /// `probability >= θ [E]`.
    Probability {
        /// The event.
        event: ProbabilisticEvent,
        /// The probability threshold.
        bound: ProbabilisticBound,
    },
    /// `quantile q of M <= c`.
    Quantile {
        /// The level.
        level: CheckedRational,
        /// The measure.
        measure: ProbabilisticMeasure,
        /// The threshold.
        bound: ProbabilisticBound,
    },
    /// `mean of F >= θ`.
    MeanOfFraction {
        /// The fraction measure.
        measure: ProbabilisticMeasure,
        /// The threshold.
        bound: ProbabilisticBound,
    },
    /// `long_run_fraction holds(P) >= θ`.
    LongRunFraction {
        /// The predicate node.
        predicate: CheckedNodeId,
        /// The weighting.
        weighting: ProbabilisticWeighting,
        /// The threshold.
        bound: ProbabilisticBound,
    },
    /// A reachability form.
    Reach {
        /// The target.
        target: ProbabilisticReachTarget,
        /// The threshold.
        bound: ProbabilisticBound,
    },
    /// `expected accumulate R until holds(B) <= c`.
    ExpectedReward {
        /// The `until` predicate node.
        until: CheckedNodeId,
        /// The threshold.
        bound: ProbabilisticBound,
    },
    /// `expected elapsed until holds(B) <= c`.
    ExpectedElapsed {
        /// The `until` predicate node.
        until: CheckedNodeId,
        /// The threshold.
        bound: ProbabilisticBound,
    },
}

/// A claim's activation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProbabilisticActivation {
    /// `on origin`.
    Origin,
    /// `on each (x: T) when (g)`.
    Each {
        /// The binder's `value`/`parameter` node.
        trigger: CheckedNodeId,
        /// The guard's predicate node.
        guard: CheckedNodeId,
    },
}

/// A claim's confidence parameters.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProbabilisticConfidence {
    /// `alpha`.
    pub alpha: CheckedRational,
    /// `beta`.
    pub beta: CheckedRational,
    /// `indifference`.
    pub indifference: CheckedRational,
}

/// One admitted `claim`/`probabilistic_claim` node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedProbabilisticClaimNode {
    /// The claim node.
    pub node: CheckedNodeId,
    /// The `model`/`model_root` node.
    pub subject: CheckedNodeId,
    /// The scheduler.
    pub scheduler: ProbabilisticScheduler,
    /// The reward nodes the form reads, ascending.
    pub rewards: Vec<CheckedNodeId>,
    /// The random parameter nodes of the subject's model, ascending.
    pub distributions: Vec<CheckedNodeId>,
    /// The delay nodes of the subject's model, ascending.
    pub delays: Vec<CheckedNodeId>,
    /// The form.
    pub form: ProbabilisticClaimForm,
    /// The `over` parameter node.
    pub over: Option<CheckedNodeId>,
    /// The activation.
    pub activation: ProbabilisticActivation,
    /// The confidence parameters, when stated.
    pub confidence: Option<ProbabilisticConfidence>,
}

impl CheckedPackageV2 {
    /// The admitted probabilistic claim nodes in ascending structural node-id
    /// order; empty when the package has none.
    pub fn probabilistic_claims(&self) -> &[CheckedProbabilisticClaimNode] {
        self.probabilistic_claims.as_slice()
    }
}

struct Reader<'k, 'm, 'b> {
    keys: &'k [Arc<CheckedNodeId>],
    meter: &'m mut WorkMeter<'b>,
    site: super::read_charge::Site,
}

type Read<T> = Result<T, ValidationFailure>;

fn value(term: &CheckedTerm, row: usize) -> &CheckedTerm {
    bound(term, row).expect("admitted claim binding")
}

fn spelled(term: &CheckedTerm) -> &str {
    match term {
        CheckedTerm::Literal { value, .. } => value.as_str().expect("admitted exact spelling"),
        _ => unreachable!("admitted exact value is a literal"),
    }
}

impl Reader<'_, '_, '_> {
    /// Pays `bytes` of heap about to be requested.
    fn pay(&mut self, bytes: usize) -> Read<()> {
        use super::read_charge::ReadStorage;
        ReadStorage::reserve(&mut *self.meter, bytes as u128, self.site)
            .map_err(|_| ReadStorage::denial(&*self.meter))
    }

    /// An owned rational, its two boxed spellings paid first.
    fn rational(&mut self, spelling: &str) -> Read<CheckedRational> {
        self.pay(spelling.len().saturating_add(1))?;
        let (numerator, denominator) = spelling.split_once('/').unwrap_or((spelling, "1"));
        Ok(CheckedRational {
            numerator: numerator.into(),
            denominator: denominator.into(),
        })
    }

    fn id(&mut self, term: &CheckedTerm) -> Read<CheckedNodeId> {
        let id = target(term).expect("admitted claim reference");
        Ok(
            super::node_index::resolve_id_charged(id, self.keys, self.meter)?
                .expect("admitted claim target resolves in the checked graph"),
        )
    }

    fn ids(&mut self, term: &CheckedTerm) -> Read<Vec<CheckedNodeId>> {
        let mut ids = self.meter.vec(members(term).len())?;
        for member in members(term) {
            ids.push(self.id(member)?);
        }
        Ok(ids)
    }

    fn exact(&mut self, term: &CheckedTerm) -> Read<ProbabilisticExact> {
        let CheckedTerm::Literal {
            type_id,
            value_kind,
            ..
        } = term
        else {
            unreachable!("admitted exact value is a literal")
        };
        let spelling = spelled(term);
        Ok(match value_kind {
            super::LiteralKind::Integer => {
                // The digits parse into at most one byte of limb per digit,
                // and the parse holds one transient copy: two bytes a digit
                // bounds both.
                self.pay(spelling.len().saturating_mul(2).saturating_add(32))?;
                ProbabilisticExact::Integer(spelling.parse().expect("canonical integer"))
            }
            super::LiteralKind::Quantity => {
                let unit = super::node_index::resolve_id_charged(type_id, self.keys, self.meter)?
                    .expect("admitted unit resolves");
                ProbabilisticExact::Quantity {
                    unit,
                    magnitude: self.rational(spelling)?,
                }
            }
            _ => ProbabilisticExact::Rational(self.rational(spelling)?),
        })
    }

    fn bound(&mut self, term: &CheckedTerm) -> Read<ProbabilisticBound> {
        let direction = match text(value(term, 0)) {
            Some("at_least") => BoundDirection::AtLeast,
            _ => BoundDirection::AtMost,
        };
        Ok(ProbabilisticBound {
            direction,
            value: self.exact(value(term, 1))?,
        })
    }

    fn weighting(term: &CheckedTerm) -> ProbabilisticWeighting {
        match text(value(term, 0)) {
            Some("reward") => ProbabilisticWeighting::Reward,
            Some("delay") => ProbabilisticWeighting::Delay,
            _ => ProbabilisticWeighting::None,
        }
    }

    #[ix_trace_rs::string_edge]
    fn measure(&mut self, term: &CheckedTerm) -> Read<ProbabilisticMeasure> {
        let kind = text(value(term, 0)).expect("admitted measure kind");
        if kind == "fraction" {
            return Ok(ProbabilisticMeasure::Fraction {
                predicate: self.id(value(term, 1))?,
                window: self.exact(value(term, 2))?,
                weighting: Self::weighting(value(term, 3)),
            });
        }
        let from = self.id(value(term, 1))?;
        let until = self.id(value(term, 2))?;
        let within = self.exact(value(term, 3))?;
        Ok(match kind {
            "accumulate" => ProbabilisticMeasure::Accumulate {
                from,
                until,
                within,
            },
            "steps" => ProbabilisticMeasure::Steps {
                from,
                until,
                within,
            },
            _ => ProbabilisticMeasure::Elapsed {
                from,
                until,
                within,
            },
        })
    }

    #[ix_trace_rs::string_edge]
    fn form(&mut self, term: &CheckedTerm) -> Read<ProbabilisticClaimForm> {
        let kind = text(value(term, 0)).expect("admitted form kind");
        Ok(match kind {
            "probability" => {
                let event = value(term, 1);
                let event = match text(value(event, 0)) {
                    Some("formula") => ProbabilisticEvent::Formula(self.id(value(event, 1))?),
                    _ => ProbabilisticEvent::Compare {
                        measure: self.measure(value(event, 1))?,
                        bound: self.bound(value(event, 2))?,
                    },
                };
                ProbabilisticClaimForm::Probability {
                    event,
                    bound: self.bound(value(term, 2))?,
                }
            }
            "quantile" => ProbabilisticClaimForm::Quantile {
                level: self.rational(spelled(value(term, 1)))?,
                measure: self.measure(value(term, 2))?,
                bound: self.bound(value(term, 3))?,
            },
            "mean_of_fraction" => ProbabilisticClaimForm::MeanOfFraction {
                measure: self.measure(value(term, 1))?,
                bound: self.bound(value(term, 2))?,
            },
            "long_run_fraction" => ProbabilisticClaimForm::LongRunFraction {
                predicate: self.id(value(term, 1))?,
                weighting: Self::weighting(value(term, 2)),
                bound: self.bound(value(term, 3))?,
            },
            "reach" => {
                let reach = value(term, 1);
                let reach = match text(value(reach, 0)) {
                    Some("eventually") => {
                        ProbabilisticReachTarget::Eventually(self.id(value(reach, 1))?)
                    }
                    Some("always") => ProbabilisticReachTarget::Always(self.id(value(reach, 1))?),
                    _ => ProbabilisticReachTarget::Until(
                        self.id(value(reach, 1))?,
                        self.id(value(reach, 2))?,
                    ),
                };
                ProbabilisticClaimForm::Reach {
                    target: reach,
                    bound: self.bound(value(term, 2))?,
                }
            }
            "expected_reward" => ProbabilisticClaimForm::ExpectedReward {
                until: self.id(value(term, 1))?,
                bound: self.bound(value(term, 2))?,
            },
            _ => ProbabilisticClaimForm::ExpectedElapsed {
                until: self.id(value(term, 1))?,
                bound: self.bound(value(term, 2))?,
            },
        })
    }

    fn claim(
        &mut self,
        position: usize,
        body: &CheckedTerm,
    ) -> Read<CheckedProbabilisticClaimNode> {
        super::read_charge::charge_node_id_clone(
            &mut *self.meter,
            self.site,
            &self.keys[position],
        )?;
        let node = (*self.keys[position]).clone();
        let scheduler = value(body, 1);
        let scheduler = match scheduler {
            CheckedTerm::Reference { .. } => ProbabilisticScheduler::Workload(self.id(scheduler)?),
            _ => {
                let set = value(scheduler, 0);
                let mut every = self.meter.vec(members(set).len())?;
                for member in members(set) {
                    let operation = text(value(member, 3)).expect("admitted operation");
                    self.pay(operation.len())?;
                    every.push(FairnessMember {
                        kind: match text(value(member, 0)) {
                            Some("weak") => CheckedFairnessKind::Weak,
                            _ => CheckedFairnessKind::Strong,
                        },
                        granularity: match text(value(member, 1)) {
                            Some("each") => CheckedFairnessGranularity::Each,
                            _ => CheckedFairnessGranularity::Whole,
                        },
                        declaring: self.id(value(member, 2))?,
                        operation: operation.into(),
                    });
                }
                ProbabilisticScheduler::Every(every)
            }
        };
        let over = value(body, 6);
        let activation = value(body, 7);
        let confidence = value(body, 8);
        Ok(CheckedProbabilisticClaimNode {
            node,
            subject: self.id(value(body, 0))?,
            scheduler,
            rewards: self.ids(value(body, 2))?,
            distributions: self.ids(value(body, 3))?,
            delays: self.ids(value(body, 4))?,
            form: self.form(value(body, 5))?,
            over: match over {
                CheckedTerm::Reference { .. } => Some(self.id(over)?),
                _ => None,
            },
            activation: match text(value(activation, 0)) {
                Some("each") => ProbabilisticActivation::Each {
                    trigger: self.id(value(activation, 1))?,
                    guard: self.id(value(activation, 2))?,
                },
                _ => ProbabilisticActivation::Origin,
            },
            confidence: if members(confidence).is_empty() {
                None
            } else {
                Some(ProbabilisticConfidence {
                    alpha: self.rational(spelled(value(confidence, 0)))?,
                    beta: self.rational(spelled(value(confidence, 1)))?,
                    indifference: self.rational(spelled(value(confidence, 2)))?,
                })
            },
        })
    }
}

/// Retains the admitted claim nodes once, after the reader admitted them.
pub(super) fn retain_claims(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    structural_keys: &[Arc<CheckedNodeId>],
    budget: &mut NativeJoinBudget,
) -> Result<Arc<Vec<CheckedProbabilisticClaimNode>>, ValidationFailure> {
    let site = super::read_charge::Site::new(PackageReadStage::Retained, 0, "probabilistic_claims");
    let kind = CheckedNodeKind::Claim(ClaimForm::ProbabilisticClaim);
    let count = kinds.iter().filter(|held| **held == kind).count();
    let mut rows = super::read_charge::with_capacity(budget, site, count)?;
    let mut meter = WorkMeter::over(budget);
    let mut reader = Reader {
        keys: structural_keys,
        meter: &mut meter,
        site,
    };
    for (position, (node, held)) in nodes.iter().zip(kinds).enumerate() {
        if *held == kind {
            rows.push(reader.claim(position, &node.body)?);
        }
    }
    // The sort compares plain node ids, as the join's own `by_id` sort does
    // for every node of the package before this point; a claim pair adds no
    // comparison class that sort has not already run.
    rows.sort_unstable_by(|left, right| left.node.cmp(&right.node));
    super::read_charge::new_arc(budget, site, rows)
}
