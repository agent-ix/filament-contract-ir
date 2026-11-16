// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The typed read of FSpec FR-640 "Typed read": the random parameters,
//! rewards, workloads and delay distributions of an admitted package.
//!
//! Implements: FR-369 "Behavior" (typed read).

use super::probabilistic_join::{binding_name, bound, members, target, text};
use super::{CheckedNodeId, CheckedNodeKind, CheckedPackageV2, CheckedTerm, ProbabilisticForm};
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Zero};

/// One support value of a random parameter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedSupportValue<'a> {
    /// The support value, a literal of the parameter's type.
    pub value: &'a CheckedTerm,
    /// The exact weight.
    pub weight: &'a str,
    /// The exact probability, the weight over the sum of the weights, in
    /// lowest terms.
    pub probability: Box<str>,
}

/// A random parameter of one model operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedRandomParameter<'a> {
    /// The `probabilistic` node.
    pub node: &'a CheckedNodeId,
    /// The parameter's name.
    pub parameter: &'a str,
    /// The support in declared order.
    pub support: Vec<CheckedSupportValue<'a>>,
}

/// A reward of one model operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckedReward<'a> {
    /// The `probabilistic` node.
    pub node: &'a CheckedNodeId,
    /// The reward's name.
    pub name: &'a str,
    /// The integer or quantity type node of the reward.
    pub value_type: &'a CheckedNodeId,
    /// The clause function evaluated on a transition's arguments.
    pub function: &'a CheckedNodeId,
}

/// One operation's weight in a workload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckedWorkloadWeight<'a> {
    /// The node that declares the operation.
    pub declaring: &'a CheckedNodeId,
    /// The operation's name.
    pub operation: &'a str,
    /// The exact weight.
    pub weight: &'a str,
}

/// A workload of one model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedWorkload<'a> {
    /// The `probabilistic` node.
    pub node: &'a CheckedNodeId,
    /// The workload's name.
    pub name: &'a str,
    /// The weights in ascending order of declaring node and operation.
    pub weights: Vec<CheckedWorkloadWeight<'a>>,
}

/// The family of a delay distribution with its parameters.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedDelayFamily<'a> {
    /// `uniform`.
    Uniform,
    /// `uniform_interval` over two time quantities.
    UniformInterval {
        /// The lower bound.
        lower: &'a str,
        /// The upper bound.
        upper: &'a str,
    },
    /// `exponential` with an exact rate per time unit.
    Exponential {
        /// The rate.
        rate: &'a str,
    },
    /// `discrete`: delays with exact weights in declared order.
    Discrete(Vec<(&'a str, &'a str)>),
}

/// A delay distribution of one model operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedDelayDistribution<'a> {
    /// The `probabilistic` node.
    pub node: &'a CheckedNodeId,
    /// The model operation's name.
    pub operation: &'a str,
    /// The family and its parameters.
    pub family: CheckedDelayFamily<'a>,
}

fn quantity(term: &CheckedTerm) -> Option<&str> {
    match term {
        CheckedTerm::Literal { value, .. } => value.as_str(),
        _ => None,
    }
}

/// An exact rational spelling `n` or `n/d`.
fn rational(spelling: &str) -> Option<(BigInt, BigInt)> {
    let (n, d) = spelling.split_once('/').unwrap_or((spelling, "1"));
    Some((n.parse().ok()?, d.parse().ok()?))
}

/// Each weight over the sum of the weights, in lowest terms.
fn probabilities(weights: &[&str]) -> Option<Vec<Box<str>>> {
    let parts: Vec<(BigInt, BigInt)> =
        weights.iter().map(|w| rational(w)).collect::<Option<_>>()?;
    let common = parts
        .iter()
        .fold(BigInt::one(), |common, (_, d)| common.lcm(d));
    let scaled: Vec<BigInt> = parts.iter().map(|(n, d)| n * (&common / d)).collect();
    let sum: BigInt = scaled.iter().sum();
    if sum.is_zero() {
        return None;
    }
    Some(
        scaled
            .iter()
            .map(|n| {
                let g = n.gcd(&sum);
                let (n, d) = (n / &g, &sum / &g);
                if d.is_one() {
                    n.to_string().into()
                } else {
                    format!("{n}/{d}").into()
                }
            })
            .collect(),
    )
}

impl CheckedPackageV2 {
    fn probabilistic(
        &self,
        wanted: ProbabilisticForm,
    ) -> impl Iterator<Item = (&CheckedNodeId, &CheckedTerm)> {
        self.wire
            .semantic_graph
            .nodes
            .iter()
            .zip(self.kinds.iter())
            .filter(move |(_, kind)| **kind == CheckedNodeKind::Probabilistic(wanted))
            .map(|(node, _)| (&node.node_id, &node.body))
    }

    /// The random parameters of operation `operation` of the model node
    /// `declaring`, with their supports and exact probabilities, each keyed
    /// by its node.
    pub fn random_parameters(
        &self,
        declaring: &CheckedNodeId,
        operation: &str,
    ) -> Vec<CheckedRandomParameter<'_>> {
        self.probabilistic(ProbabilisticForm::RandomParameter)
            .filter(|(_, body)| {
                bound(body, 0).and_then(target) == Some(declaring)
                    && bound(body, 1).and_then(text) == Some(operation)
            })
            .filter_map(|(node, body)| {
                let rows = members(bound(body, 3)?);
                let weights: Vec<&str> = rows
                    .iter()
                    .map(|row| bound(row, 1).and_then(quantity))
                    .collect::<Option<_>>()?;
                let probabilities = probabilities(&weights)?;
                let support = rows
                    .iter()
                    .zip(weights)
                    .zip(probabilities)
                    .map(|((row, weight), probability)| {
                        Some(CheckedSupportValue {
                            value: bound(row, 0)?,
                            weight,
                            probability,
                        })
                    })
                    .collect::<Option<_>>()?;
                Some(CheckedRandomParameter {
                    node,
                    parameter: bound(body, 2).and_then(text)?,
                    support,
                })
            })
            .collect()
    }

    /// The rewards of operation `operation` of the model node `declaring`,
    /// each keyed by its node.
    pub fn rewards(&self, declaring: &CheckedNodeId, operation: &str) -> Vec<CheckedReward<'_>> {
        self.probabilistic(ProbabilisticForm::Reward)
            .filter(|(_, body)| {
                bound(body, 0).and_then(target) == Some(declaring)
                    && bound(body, 1).and_then(text) == Some(operation)
            })
            .filter_map(|(node, body)| {
                Some(CheckedReward {
                    node,
                    name: bound(body, 2).and_then(text)?,
                    value_type: bound(body, 3).and_then(target)?,
                    function: bound(body, 4).and_then(target)?,
                })
            })
            .collect()
    }

    /// The workloads of the model node `model`, each keyed by its node.
    pub fn workloads(&self, model: &CheckedNodeId) -> Vec<CheckedWorkload<'_>> {
        self.probabilistic(ProbabilisticForm::Workload)
            .filter(|(_, body)| bound(body, 0).and_then(target) == Some(model))
            .filter_map(|(node, body)| {
                let weights = members(bound(body, 2)?)
                    .iter()
                    .map(|row| {
                        Some(CheckedWorkloadWeight {
                            declaring: bound(row, 0).and_then(target)?,
                            operation: bound(row, 1).and_then(text)?,
                            weight: bound(row, 2).and_then(quantity)?,
                        })
                    })
                    .collect::<Option<_>>()?;
                Some(CheckedWorkload {
                    node,
                    name: bound(body, 1).and_then(text)?,
                    weights,
                })
            })
            .collect()
    }

    /// The delay distributions of the operations of the model node `model`,
    /// each keyed by its node.
    pub fn delay_distributions(&self, model: &CheckedNodeId) -> Vec<CheckedDelayDistribution<'_>> {
        self.probabilistic(ProbabilisticForm::DelayDistribution)
            .filter(|(_, body)| bound(body, 0).and_then(target) == Some(model))
            .filter_map(|(node, body)| {
                let parameters = bound(body, 3)?;
                let named = |name: &str| {
                    members(parameters)
                        .iter()
                        .find(|member| binding_name(member) == Some(name))
                        .and_then(|member| match member {
                            CheckedTerm::Binding { value, .. } => quantity(value),
                            _ => None,
                        })
                };
                let family = match bound(body, 2).and_then(text)? {
                    "uniform" => CheckedDelayFamily::Uniform,
                    "uniform_interval" => CheckedDelayFamily::UniformInterval {
                        lower: named("lower")?,
                        upper: named("upper")?,
                    },
                    "exponential" => CheckedDelayFamily::Exponential {
                        rate: named("rate")?,
                    },
                    "discrete" => CheckedDelayFamily::Discrete(
                        members(parameters)
                            .iter()
                            .map(|row| {
                                Some((
                                    bound(row, 0).and_then(quantity)?,
                                    bound(row, 1).and_then(quantity)?,
                                ))
                            })
                            .collect::<Option<_>>()?,
                    ),
                    _ => return None,
                };
                Some(CheckedDelayDistribution {
                    node,
                    operation: bound(body, 1).and_then(text)?,
                    family,
                })
            })
            .collect()
    }
}
