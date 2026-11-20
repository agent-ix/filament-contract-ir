// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The identity-row join of FR-369: every operation reference of a
//! `probabilistic` node whose `declaring` node (a workload's `model`) is a
//! source-declared object type is joined through the type's identity row.
//!
//! Implements: FR-369 "Behavior" (identity-row join), FR-358.

use filament_exact::Integer;

use super::native_join_budget::{
    NativeJoinBudget, NativeJoinChargeKind, NativeJoinLocus, NativeJoinLocusSource,
};
use super::probabilistic_join::{bound, members, target, text, text_length};
use super::source_object::{
    is_source_declared_object_type, SelectedSourceObjectError, SelectedSourceObjectErrorCode,
    SelectedSourceObjectMember, SelectedSourceObjectRow, SelectedSourceObjects,
};
use super::{
    BoundedDomainForm, CheckedNodeId, CheckedNodeKind, CheckedPackageV2, CheckedTerm, LiteralKind,
    ProbabilisticForm, ScalarTypeForm, ValueForm,
};
use crate::checked_package::shared::{CheckedPackageRefusalLocation, JsonPointer};

const FORM: &str = "probabilistic_join";

type Failure = SelectedSourceObjectError;

/// The `<operation>` of an entry identity `<type>::<operation>` of the row of
/// `type_name`.
#[ix_trace_rs::string_edge]
fn operation_of<'a>(identity: &'a str, type_name: &str) -> Option<&'a str> {
    let (declared, operation) = identity.rsplit_once("::")?;
    (declared == type_name).then_some(operation)
}

/// What the entry's type node says about a support value.
enum Membership {
    /// Any value of the literal kind.
    Any,
    Boolean,
    Integers(Integer, Integer),
    Rationals([Integer; 4]),
    Text(u64, u64, Box<str>),
}

/// The literal kind, the unit node position of a quantity and the membership
/// rule of the values of a data entry's type node, `None` when the type has no
/// literal kind.
struct Slot {
    kind: LiteralKind,
    unit: Option<usize>,
    membership: Membership,
}

/// Why a support value is no member of its type.
enum NotMember {
    Literal,
    Spelling,
}

struct Join<'p, 'b> {
    package: &'p CheckedPackageV2,
    selected: &'p SelectedSourceObjects<'p>,
    budget: &'b mut NativeJoinBudget,
}

fn integer_of(term: Option<&CheckedTerm>) -> Option<Integer> {
    match term? {
        CheckedTerm::Literal { value, .. } => value.as_str()?.parse().ok(),
        _ => None,
    }
}

fn natural_of(term: Option<&CheckedTerm>) -> Option<u64> {
    u64::try_from(integer_of(term)?.to_string().parse::<i128>().ok()?).ok()
}

impl<'p> Join<'p, '_> {
    fn location(at: &JsonPointer) -> Option<CheckedPackageRefusalLocation> {
        Some(CheckedPackageRefusalLocation {
            path: Some(at.clone()),
            source_region: None,
        })
    }

    fn refuse(at: &JsonPointer) -> Failure {
        Failure {
            code: SelectedSourceObjectErrorCode::InvalidValue,
            locus: Self::location(at),
        }
    }

    /// Admits one work unit of the caller's budget ahead of the work at `at`,
    /// of the node at `position`.
    fn charge(&mut self, position: usize, at: &JsonPointer) -> Result<(), Failure> {
        let locus = NativeJoinLocus {
            source: NativeJoinLocusSource::GraphNode(position),
            form: FORM,
        };
        self.budget
            .accounted_charge(NativeJoinChargeKind::Work, 1, locus)
            .map_err(|_| Failure {
                code: SelectedSourceObjectErrorCode::Incomplete,
                locus: Self::location(at),
            })
    }

    fn node_at(position: usize) -> JsonPointer {
        JsonPointer::root()
            .key("semantic_graph")
            .key("nodes")
            .index(position)
    }

    fn member_at(position: usize, member: usize) -> JsonPointer {
        Self::node_at(position)
            .key("body")
            .key("members")
            .index(member)
            .key("value")
    }

    /// The position of a source-declared object type `id` names.
    fn source_type(&self, id: Option<&CheckedNodeId>) -> Option<usize> {
        let position = self.package.position_of(id?)?;
        let node = self.package.resolved_graph().nodes.get(position)?;
        is_source_declared_object_type(node, *self.package.kinds.get(position)?).then_some(position)
    }

    fn row(&self, position: usize) -> Option<&'p SelectedSourceObjectRow> {
        self.selected.row_at(position)
    }

    /// The literal kind and membership rule of a data entry.
    fn slot(&self, entry: &SelectedSourceObjectMember) -> Option<Slot> {
        let at = self.package.position_of(entry.type_node.as_ref()?)?;
        let body = self.package.term_at(at)?;
        let plain = |kind| {
            Some(Slot {
                kind,
                unit: None,
                membership: Membership::Any,
            })
        };
        match self.package.kinds.get(at)? {
            CheckedNodeKind::ScalarType(form) => match form {
                ScalarTypeForm::Boolean => Some(Slot {
                    kind: LiteralKind::Boolean,
                    unit: None,
                    membership: Membership::Boolean,
                }),
                ScalarTypeForm::Integer => plain(LiteralKind::Integer),
                ScalarTypeForm::Rational => plain(LiteralKind::Rational),
                ScalarTypeForm::Decimal => plain(LiteralKind::Decimal),
                ScalarTypeForm::Float32 => plain(LiteralKind::Float32Bits),
                ScalarTypeForm::Float64 => plain(LiteralKind::Float64Bits),
                ScalarTypeForm::Text => plain(LiteralKind::Text),
                ScalarTypeForm::String => plain(LiteralKind::String),
                ScalarTypeForm::Uuid => plain(LiteralKind::Uuid),
                ScalarTypeForm::Timestamp => plain(LiteralKind::Timestamp),
                ScalarTypeForm::Unit => Some(Slot {
                    kind: LiteralKind::Quantity,
                    unit: Some(at),
                    membership: Membership::Any,
                }),
                ScalarTypeForm::CarriedQuantity => {
                    let unit = bound(body, 0).and_then(target)?;
                    let carrier = self.package.position_of(bound(body, 1).and_then(target)?)?;
                    let rational = self.rational_bounds(carrier)?;
                    Some(Slot {
                        kind: LiteralKind::Quantity,
                        unit: Some(self.package.position_of(unit)?),
                        membership: Membership::Rationals(rational),
                    })
                }
                _ => None,
            },
            CheckedNodeKind::BoundedDomain(BoundedDomainForm::IntegerRange) => Some(Slot {
                kind: LiteralKind::Integer,
                unit: None,
                membership: Membership::Integers(
                    integer_of(bound(body, 0))?,
                    integer_of(bound(body, 1))?,
                ),
            }),
            CheckedNodeKind::BoundedDomain(BoundedDomainForm::RationalRange) => Some(Slot {
                kind: LiteralKind::Rational,
                unit: None,
                membership: Membership::Rationals(self.rational_bounds(at)?),
            }),
            CheckedNodeKind::BoundedDomain(BoundedDomainForm::TextBounds) => Some(Slot {
                kind: LiteralKind::Text,
                unit: None,
                membership: Membership::Text(
                    natural_of(bound(body, 0))?,
                    natural_of(bound(body, 1))?,
                    text(bound(body, 2)?)?.into(),
                ),
            }),
            _ => None,
        }
    }

    /// The numerator and denominator bounds of the `rational_range` node at
    /// `at`.
    fn rational_bounds(&self, at: usize) -> Option<[Integer; 4]> {
        (self.package.kinds.get(at)?
            == &CheckedNodeKind::BoundedDomain(BoundedDomainForm::RationalRange))
            .then_some(())?;
        let body = self.package.term_at(at)?;
        Some([
            integer_of(bound(body, 0))?,
            integer_of(bound(body, 1))?,
            integer_of(bound(body, 2))?,
            integer_of(bound(body, 3))?,
        ])
    }

    /// Why `value` is no member of the slot's type, `None` when it is one.
    fn not_member(slot: &Slot, value: &super::CheckedJson) -> Option<NotMember> {
        if matches!(slot.membership, Membership::Boolean) {
            return value.as_bool().is_none().then_some(NotMember::Literal);
        }
        let Some(spelled) = value.as_str() else {
            return Some(NotMember::Literal);
        };
        let outside = match &slot.membership {
            Membership::Integers(lower, upper) => match spelled.parse::<Integer>() {
                Ok(found) => found < *lower || found > *upper,
                Err(_) => true,
            },
            Membership::Rationals([n_min, n_max, d_min, d_max]) => {
                let Some((numerator, denominator)) = canonical_rational(spelled) else {
                    return Some(NotMember::Spelling);
                };
                numerator < *n_min
                    || numerator > *n_max
                    || denominator < *d_min
                    || denominator > *d_max
            }
            Membership::Text(lower, upper, profile) => text_length(spelled, profile)
                .and_then(|length| u64::try_from(length).ok())
                .is_none_or(|length| length < *lower || length > *upper),
            Membership::Any | Membership::Boolean => false,
        };
        outside.then_some(NotMember::Literal)
    }

    /// The operation entry `name` of the row at `declaring`: one visit for
    /// the reference joined and one for each entry read.
    fn entry(
        &mut self,
        holder: usize,
        declaring: usize,
        name: Option<&str>,
    ) -> Result<&'p [SelectedSourceObjectMember], Failure> {
        let at = Self::member_at(holder, 1);
        self.charge(holder, &at)?;
        let (Some(name), Some(row)) = (name, self.row(declaring)) else {
            return Err(Self::refuse(&at));
        };
        for entry in &row.operations {
            self.charge(holder, &at)?;
            if operation_of(&entry.identity, &row.source_identity) == Some(name) {
                return Ok(&entry.parameters);
            }
        }
        Err(Self::refuse(&at))
    }

    fn random_parameter(
        &mut self,
        holder: usize,
        body: &CheckedTerm,
        parameters: &'p [SelectedSourceObjectMember],
    ) -> Result<(), Failure> {
        let parameter_at = Self::member_at(holder, 2);
        let name = bound(body, 2).and_then(text);
        let mut found = None;
        for entry in parameters {
            self.charge(holder, &parameter_at)?;
            if name == Some(entry.name.as_ref()) {
                found = Some(entry);
                break;
            }
        }
        let Some(entry) = found else {
            return Err(Self::refuse(&parameter_at));
        };
        let slot = self.slot(entry);
        let support_at = Self::member_at(holder, 3);
        for (row, support) in bound(body, 3)
            .map(members)
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            let value_at = support_at
                .clone()
                .key("members")
                .index(row)
                .key("members")
                .index(0)
                .key("value");
            self.charge(holder, &value_at)?;
            let Some(CheckedTerm::Literal {
                value_kind,
                type_id,
                value,
            }) = bound(support, 0)
            else {
                return Err(Self::refuse(&value_at));
            };
            let Some(slot) = slot.as_ref().filter(|slot| slot.kind == *value_kind) else {
                return Err(Self::refuse(&value_at));
            };
            if slot
                .unit
                .is_some_and(|unit| self.package.position_of(type_id) != Some(unit))
            {
                return Err(Self::refuse(&value_at));
            }
            match Self::not_member(slot, value) {
                Some(NotMember::Literal) => return Err(Self::refuse(&value_at)),
                Some(NotMember::Spelling) => return Err(Self::refuse(&value_at.key("value"))),
                None => {}
            }
        }
        Ok(())
    }

    /// The reward function's `parameters` against the operation entry's, by
    /// name and in declared order, one visit for each listed parameter.
    fn reward(
        &mut self,
        holder: usize,
        body: &CheckedTerm,
        parameters: &'p [SelectedSourceObjectMember],
    ) -> Result<(), Failure> {
        let expression_at = Self::member_at(holder, 4).key("target");
        let function = bound(body, 4)
            .and_then(target)
            .and_then(|id| self.package.position_of(id))
            .and_then(|position| self.package.term_at(position));
        let listed = function
            .and_then(|function| bound(function, 0))
            .map(members)
            .unwrap_or_default();
        for (index, entry) in listed.iter().enumerate() {
            self.charge(holder, &expression_at)?;
            let name = target(entry)
                .and_then(|id| self.package.position_of(id))
                .filter(|&position| {
                    self.package.kinds.get(position)
                        == Some(&CheckedNodeKind::Value(ValueForm::Parameter))
                })
                .and_then(|position| self.package.term_at(position))
                .and_then(|node| bound(node, 0))
                .and_then(text);
            let declared = parameters.get(index);
            if declared.is_none() || name != declared.map(|declared| declared.name.as_ref()) {
                return Err(Self::refuse(&expression_at));
            }
        }
        if parameters.len() > listed.len() {
            return Err(Self::refuse(&expression_at));
        }
        Ok(())
    }

    /// A workload of a source-declared `model`: its rows name the operations
    /// of the model's row and of its supertypes, followed transitively, and
    /// omit none.
    fn workload(&mut self, holder: usize, body: &CheckedTerm, model: usize) -> Result<(), Failure> {
        let model_at = Self::member_at(holder, 0).key("target");
        let weights_at = Self::member_at(holder, 2);
        let mut lineage = vec![model];
        let mut next = 0;
        while let Some(&current) = lineage.get(next) {
            next += 1;
            let Some(row) = self.row(current) else {
                continue;
            };
            for supertype in &row.supertypes {
                self.charge(holder, &model_at)?;
                if let Some(position) = self.package.position_of(supertype) {
                    if !lineage.contains(&position) {
                        lineage.push(position);
                    }
                }
            }
        }
        let mut declared: Vec<(usize, &str)> = Vec::new();
        for &position in &lineage {
            let Some(row) = self.row(position) else {
                continue;
            };
            for entry in &row.operations {
                self.charge(holder, &weights_at)?;
                if let Some(name) = operation_of(&entry.identity, &row.source_identity) {
                    declared.push((position, name));
                }
            }
        }
        let mut named: Vec<&str> = Vec::new();
        for (index, row) in bound(body, 2)
            .map(members)
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            let row_at = weights_at.clone().key("members").index(index);
            self.charge(holder, &row_at)?;
            let declaring = bound(row, 0)
                .and_then(target)
                .and_then(|id| self.package.position_of(id))
                .filter(|position| lineage.contains(position));
            let Some(declaring) = declaring else {
                return Err(Self::refuse(
                    &row_at.key("members").index(0).key("value").key("target"),
                ));
            };
            let name = bound(row, 1).and_then(text);
            let Some(name) = name.filter(|name| declared.contains(&(declaring, *name))) else {
                return Err(Self::refuse(
                    &row_at.key("members").index(1).key("value").key("value"),
                ));
            };
            named.push(name);
        }
        if declared.iter().any(|(_, name)| !named.contains(name)) {
            return Err(Self::refuse(&weights_at.key("members")));
        }
        Ok(())
    }
}

/// `text` as a canonical reduced rational, `n` or `n/d` with `d > 1` coprime
/// to `n`.
fn canonical_rational(text: &str) -> Option<(Integer, Integer)> {
    let Some((numerator, denominator)) = text.split_once('/') else {
        return Some((text.parse().ok()?, Integer::one()));
    };
    let (numerator, denominator): (Integer, Integer) =
        (numerator.parse().ok()?, denominator.parse().ok()?);
    (denominator > Integer::one() && numerator.gcd(&denominator) == Integer::one())
        .then_some((numerator, denominator))
}

impl SelectedSourceObjects<'_> {
    /// Join every operation reference of a `probabilistic` node whose
    /// `declaring` node (a workload's `model`) is source-declared through the
    /// identity row of that node, in ascending node-id order (FR-369). Each
    /// operation reference joined, operation entry and supertype edge read,
    /// support value checked and reward function parameter compared is one
    /// work unit of `budget`, charged ahead of the work.
    ///
    /// `Ok(())` means the typed reads `delay_distributions`,
    /// `random_parameters`, `rewards` and `workloads` of the package are
    /// joined.
    ///
    /// # Errors
    ///
    /// [`SelectedSourceObjectErrorCode::InvalidValue`] at the member the
    /// strict read would name, for the first node that disagrees with its
    /// row; [`SelectedSourceObjectErrorCode::Incomplete`] at the first
    /// uncharged member when `budget` denies a charge, with no partial view.
    pub fn join_probabilistic(
        &self,
        budget: &mut NativeJoinBudget,
    ) -> Result<(), SelectedSourceObjectError> {
        budget.resume();
        let package = self.package();
        let nodes = &package.resolved_graph().nodes;
        let mut order: Vec<usize> = (0..nodes.len())
            .filter(|&position| {
                matches!(
                    package.kinds.get(position),
                    Some(CheckedNodeKind::Probabilistic(_))
                )
            })
            .collect();
        order.sort_by(|&left, &right| nodes[left].node_id.cmp(&nodes[right].node_id));
        let mut join = Join {
            package,
            selected: self,
            budget,
        };
        for position in order {
            let Some(CheckedNodeKind::Probabilistic(form)) = package.kinds.get(position) else {
                continue;
            };
            let Some(body) = package.term_at(position) else {
                continue;
            };
            let Some(declaring) = join.source_type(bound(body, 0).and_then(target)) else {
                continue;
            };
            match form {
                ProbabilisticForm::Workload => join.workload(position, body, declaring)?,
                ProbabilisticForm::DelayDistribution => {
                    join.entry(position, declaring, bound(body, 1).and_then(text))?;
                }
                ProbabilisticForm::RandomParameter => {
                    let parameters =
                        join.entry(position, declaring, bound(body, 1).and_then(text))?;
                    join.random_parameter(position, body, parameters)?;
                }
                ProbabilisticForm::Reward => {
                    let parameters =
                        join.entry(position, declaring, bound(body, 1).and_then(text))?;
                    join.reward(position, body, parameters)?;
                }
            }
        }
        Ok(())
    }
}
