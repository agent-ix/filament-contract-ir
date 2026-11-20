// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The identity-row join of FR-369: every operation reference of a
//! `probabilistic` node whose `declaring` node (a workload's `model`) is a
//! source-declared object type is joined through the type's identity row.
//!
//! Implements: FR-369 "Behavior" (identity-row join), FR-358.

use super::model_members::{IntegerBounds, RationalBounds};
use super::native_join_budget::{
    NativeJoinBudget, NativeJoinChargeKind, NativeJoinLocus, NativeJoinLocusSource,
};
use super::probabilistic_join::{
    bound, bounded_not_member, members, target, text, Bounded, NotMember,
};
use super::source_object::{
    is_source_declared_object_type, SelectedSourceObjectError, SelectedSourceObjectErrorCode,
    SelectedSourceObjectMember, SelectedSourceObjectRow, SelectedSourceObjects,
};
use super::{
    BoundedDomainForm, CheckedNodeId, CheckedNodeKind, CheckedPackageV2, CheckedTerm, LiteralKind,
    ProbabilisticForm, ScalarTypeForm, ValueForm, WorkMeter,
};
use crate::checked_package::shared::{CheckedPackageRefusalLocation, JsonPointer};

const FORM: &str = "probabilistic_join";

/// The logical storage one refusal locus is charged, ahead of building it: a
/// pointer of at most a dozen short segments.
const LOCUS_BYTES: u128 = 256;

/// The smallest capacity a charged list grows to, as in the strict read.
const MINIMUM_CAPACITY: usize = 4;

type Failure = SelectedSourceObjectError;

/// Where a refusal or a budget stop is named, built only once it is needed.
type At<'a> = &'a dyn Fn() -> JsonPointer;

/// The `<operation>` of an entry identity `<type>::<operation>` of the row of
/// `type_name`.
#[ix_trace_rs::string_edge]
fn operation_of<'a>(identity: &'a str, type_name: &str) -> Option<&'a str> {
    let (declared, operation) = identity.rsplit_once("::")?;
    (declared == type_name).then_some(operation)
}

/// The literal kind, the unit node position of a quantity and the membership
/// rule of the values of a data entry's type node.
struct Slot<'p> {
    kind: LiteralKind,
    unit: Option<usize>,
    rule: Bounded<'p>,
}

struct Join<'p, 'b> {
    package: &'p CheckedPackageV2,
    selected: &'p SelectedSourceObjects<'p>,
    /// Charges the caller's budget: work, logical storage and the exact
    /// numbers' read storage, as the strict read does.
    meter: WorkMeter<'b>,
}

/// `term` as the integer literal it spells, for the bounds of a type node the
/// strict read has already admitted.
fn integer_of(term: Option<&CheckedTerm>) -> Option<i128> {
    match term? {
        CheckedTerm::Literal { value, .. } => value.as_str()?.parse().ok(),
        _ => None,
    }
}

fn natural_of(term: Option<&CheckedTerm>) -> Option<u64> {
    u64::try_from(integer_of(term)?).ok()
}

impl<'p> Join<'p, '_> {
    fn location(at: At) -> Option<CheckedPackageRefusalLocation> {
        Some(CheckedPackageRefusalLocation {
            path: Some(at()),
            source_region: None,
        })
    }

    /// The stop at `at`, the first member left uncharged. Its locus is a
    /// bounded pointer of static keys and indices.
    fn incomplete(at: At) -> Failure {
        Failure {
            code: SelectedSourceObjectErrorCode::Incomplete,
            locus: Self::location(at),
        }
    }

    /// Admits `amount` of `kind` of the caller's budget ahead of the work at
    /// `at`, of the node at `position`.
    fn pay(
        &mut self,
        kind: NativeJoinChargeKind,
        amount: u128,
        position: usize,
        at: At,
    ) -> Result<(), Failure> {
        let locus = NativeJoinLocus {
            source: NativeJoinLocusSource::GraphNode(position),
            form: FORM,
        };
        match self.meter.budget_mut() {
            Some(budget) => budget
                .accounted_charge(kind, amount, locus)
                .map_err(|_| Self::incomplete(at)),
            None => Ok(()),
        }
    }

    /// One work unit.
    fn charge(&mut self, position: usize, at: At) -> Result<(), Failure> {
        self.pay(NativeJoinChargeKind::Work, 1, position, at)
    }

    /// One work unit and one for each byte of the caller text it compares.
    fn charge_text(&mut self, position: usize, length: usize, at: At) -> Result<(), Failure> {
        let amount = u128::try_from(length)
            .unwrap_or(u128::MAX)
            .saturating_add(1);
        self.pay(NativeJoinChargeKind::Work, amount, position, at)
    }

    /// The refusal at `at`: its pointer is paid before it is built.
    fn refuse(&mut self, position: usize, at: At) -> Failure {
        match self.pay(NativeJoinChargeKind::Storage, LOCUS_BYTES, position, at) {
            Err(stop) => stop,
            Ok(()) => Failure {
                code: SelectedSourceObjectErrorCode::InvalidValue,
                locus: Self::location(at),
            },
        }
    }

    /// Pushes onto `list`, charging each growth of its capacity first.
    fn push<T>(
        &mut self,
        list: &mut Vec<T>,
        item: T,
        position: usize,
        at: At,
    ) -> Result<(), Failure> {
        if list.len() == list.capacity() {
            let capacity = list.capacity();
            let extra = capacity.saturating_mul(2).max(MINIMUM_CAPACITY) - capacity;
            let bytes =
                u128::try_from(extra.saturating_mul(std::mem::size_of::<T>())).unwrap_or(u128::MAX);
            self.pay(NativeJoinChargeKind::Storage, bytes, position, at)?;
            list.try_reserve_exact(extra)
                .map_err(|_| Self::incomplete(at))?;
        }
        list.push(item);
        Ok(())
    }

    fn node_at(position: usize) -> JsonPointer {
        JsonPointer::root()
            .key("semantic_graph")
            .key("nodes")
            .index(position)
    }

    /// The `value` of the binding at `member` of the node at `position`, the
    /// path the strict read names for it.
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
    fn slot(&self, entry: &SelectedSourceObjectMember) -> Option<Slot<'p>> {
        let package = self.package;
        let at = package.position_of(entry.type_node.as_ref()?)?;
        let body = package.term_at(at)?;
        let plain = |kind| {
            Some(Slot {
                kind,
                unit: None,
                rule: Bounded::Free,
            })
        };
        match package.kinds.get(at)? {
            CheckedNodeKind::ScalarType(form) => match form {
                ScalarTypeForm::Boolean => Some(Slot {
                    kind: LiteralKind::Boolean,
                    unit: None,
                    rule: Bounded::Boolean,
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
                    rule: Bounded::Free,
                }),
                ScalarTypeForm::CarriedQuantity => {
                    let unit = bound(body, 0).and_then(target)?;
                    let carrier = package.position_of(bound(body, 1).and_then(target)?)?;
                    Some(Slot {
                        kind: LiteralKind::Quantity,
                        unit: Some(package.position_of(unit)?),
                        rule: Bounded::Rationals(self.rational_bounds(carrier)?),
                    })
                }
                _ => None,
            },
            CheckedNodeKind::BoundedDomain(BoundedDomainForm::IntegerRange) => Some(Slot {
                kind: LiteralKind::Integer,
                unit: None,
                rule: Bounded::Integers(IntegerBounds {
                    lower: integer_of(bound(body, 0))?,
                    upper: integer_of(bound(body, 1))?,
                }),
            }),
            CheckedNodeKind::BoundedDomain(BoundedDomainForm::RationalRange) => Some(Slot {
                kind: LiteralKind::Rational,
                unit: None,
                rule: Bounded::Rationals(self.rational_bounds(at)?),
            }),
            CheckedNodeKind::BoundedDomain(BoundedDomainForm::TextBounds) => Some(Slot {
                kind: LiteralKind::Text,
                unit: None,
                rule: Bounded::Text {
                    lower: natural_of(bound(body, 0))?,
                    upper: natural_of(bound(body, 1))?,
                    profile: text(bound(body, 2)?)?,
                },
            }),
            _ => None,
        }
    }

    /// The numerator and denominator bounds of the `rational_range` node at
    /// `at`.
    fn rational_bounds(&self, at: usize) -> Option<RationalBounds> {
        (self.package.kinds.get(at)?
            == &CheckedNodeKind::BoundedDomain(BoundedDomainForm::RationalRange))
            .then_some(())?;
        let body = self.package.term_at(at)?;
        Some(RationalBounds {
            numerator_min: integer_of(bound(body, 0))?,
            numerator_max: integer_of(bound(body, 1))?,
            denominator_min: integer_of(bound(body, 2))?,
            denominator_max: integer_of(bound(body, 3))?,
        })
    }

    /// The operation entry `name` of the row at `declaring`: one visit for
    /// the reference joined and one for each entry read, with the bytes of the
    /// identity compared.
    fn entry(
        &mut self,
        holder: usize,
        declaring: usize,
        name: Option<&str>,
    ) -> Result<&'p [SelectedSourceObjectMember], Failure> {
        let at = || Self::member_at(holder, 1).key("value");
        self.charge(holder, &at)?;
        let (Some(name), Some(row)) = (name, self.row(declaring)) else {
            return Err(self.refuse(holder, &at));
        };
        for entry in &row.operations {
            self.charge_text(holder, entry.identity.len(), &at)?;
            if operation_of(&entry.identity, &row.source_identity) == Some(name) {
                return Ok(&entry.parameters);
            }
        }
        Err(self.refuse(holder, &at))
    }

    fn random_parameter(
        &mut self,
        holder: usize,
        body: &CheckedTerm,
        parameters: &'p [SelectedSourceObjectMember],
    ) -> Result<(), Failure> {
        let parameter_at = || Self::member_at(holder, 2).key("value");
        let name = bound(body, 2).and_then(text);
        let mut found = None;
        for entry in parameters {
            self.charge_text(holder, entry.name.len(), &parameter_at)?;
            if name == Some(entry.name.as_ref()) {
                found = Some(entry);
                break;
            }
        }
        let Some(entry) = found else {
            return Err(self.refuse(holder, &parameter_at));
        };
        let slot = self.slot(entry);
        for (row, support) in bound(body, 3)
            .map(members)
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            let value_at = || {
                Self::member_at(holder, 3)
                    .key("members")
                    .index(row)
                    .key("members")
                    .index(0)
                    .key("value")
            };
            let literal = bound(support, 0);
            let spelled = match literal {
                Some(CheckedTerm::Literal { value, .. }) => value.as_str().map_or(0, str::len),
                _ => 0,
            };
            self.charge_text(holder, spelled, &value_at)?;
            let Some(CheckedTerm::Literal {
                value_kind,
                type_id,
                value,
            }) = literal
            else {
                return Err(self.refuse(holder, &value_at));
            };
            let Some(slot) = slot.as_ref().filter(|slot| slot.kind == *value_kind) else {
                return Err(self.refuse(holder, &value_at));
            };
            if slot
                .unit
                .is_some_and(|unit| self.package.position_of(type_id) != Some(unit))
            {
                return Err(self.refuse(holder, &value_at));
            }
            self.meter.at_node(holder, FORM);
            let refused = bounded_not_member(&mut self.meter, slot.rule, value)
                .map_err(|_| Self::incomplete(&value_at))?;
            match refused {
                Some(NotMember::Literal) => return Err(self.refuse(holder, &value_at)),
                Some(NotMember::Spelling) => {
                    return Err(self.refuse(holder, &|| value_at().key("value")));
                }
                None => {}
            }
        }
        Ok(())
    }

    /// The reward function's `parameters` against the operation entry's, by
    /// name and in declared order, one visit for each listed parameter and
    /// the bytes of the declared name compared.
    fn reward(
        &mut self,
        holder: usize,
        body: &CheckedTerm,
        parameters: &'p [SelectedSourceObjectMember],
    ) -> Result<(), Failure> {
        let expression_at = || Self::member_at(holder, 4).key("target");
        let function = bound(body, 4)
            .and_then(target)
            .and_then(|id| self.package.position_of(id))
            .and_then(|position| self.package.term_at(position));
        let listed = function
            .and_then(|function| bound(function, 0))
            .map(members)
            .unwrap_or_default();
        for (index, entry) in listed.iter().enumerate() {
            let declared = parameters.get(index);
            self.charge_text(
                holder,
                declared.map_or(0, |declared| declared.name.len()),
                &expression_at,
            )?;
            let name = target(entry)
                .and_then(|id| self.package.position_of(id))
                .filter(|&position| {
                    self.package.kinds.get(position)
                        == Some(&CheckedNodeKind::Value(ValueForm::Parameter))
                })
                .and_then(|position| self.package.term_at(position))
                .and_then(|node| bound(node, 0))
                .and_then(text);
            if declared.is_none() || name != declared.map(|declared| declared.name.as_ref()) {
                return Err(self.refuse(holder, &expression_at));
            }
        }
        if parameters.len() > listed.len() {
            return Err(self.refuse(holder, &expression_at));
        }
        Ok(())
    }

    /// Whether `lineage` holds `needle`, one work unit for each element met.
    fn lineage_has(
        &mut self,
        lineage: &[usize],
        needle: usize,
        holder: usize,
        at: At,
    ) -> Result<bool, Failure> {
        for &position in lineage {
            self.charge(holder, at)?;
            if position == needle {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Whether `declared` lists `name` of `declaring`, one work unit and the
    /// bytes of the name for each element met.
    fn declared_has(
        &mut self,
        declared: &[(usize, &str)],
        declaring: usize,
        name: &str,
        holder: usize,
        at: At,
    ) -> Result<bool, Failure> {
        for &(position, candidate) in declared {
            self.charge_text(holder, name.len(), at)?;
            if position == declaring && candidate == name {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Whether `named` holds `name`, one work unit and the bytes of the name
    /// for each element met.
    fn named_has(
        &mut self,
        named: &[&str],
        name: &str,
        holder: usize,
        at: At,
    ) -> Result<bool, Failure> {
        for &candidate in named {
            self.charge_text(holder, name.len(), at)?;
            if candidate == name {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// A workload of a source-declared `model`: its rows name the operations
    /// of the model's row and of its supertypes, followed transitively, and
    /// omit none.
    fn workload(&mut self, holder: usize, body: &CheckedTerm, model: usize) -> Result<(), Failure> {
        let model_at = || Self::member_at(holder, 0).key("target");
        let weights_at = || Self::member_at(holder, 2);
        let mut lineage: Vec<usize> = Vec::new();
        self.push(&mut lineage, model, holder, &model_at)?;
        let mut next = 0;
        while let Some(&current) = lineage.get(next) {
            next += 1;
            let Some(row) = self.row(current) else {
                continue;
            };
            for supertype in &row.supertypes {
                self.charge(holder, &model_at)?;
                if let Some(position) = self.package.position_of(supertype) {
                    if !self.lineage_has(&lineage, position, holder, &model_at)? {
                        self.push(&mut lineage, position, holder, &model_at)?;
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
                self.charge_text(holder, entry.identity.len(), &weights_at)?;
                if let Some(name) = operation_of(&entry.identity, &row.source_identity) {
                    self.push(&mut declared, (position, name), holder, &weights_at)?;
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
            let row_at = || weights_at().key("members").index(index);
            self.charge(holder, &row_at)?;
            let declaring = bound(row, 0)
                .and_then(target)
                .and_then(|id| self.package.position_of(id));
            let declaring = match declaring {
                Some(position) if self.lineage_has(&lineage, position, holder, &row_at)? => {
                    Some(position)
                }
                _ => None,
            };
            let Some(declaring) = declaring else {
                return Err(self.refuse(holder, &|| {
                    row_at().key("members").index(0).key("value").key("target")
                }));
            };
            let name_at = || row_at().key("members").index(1).key("value").key("value");
            let name = bound(row, 1).and_then(text);
            let name = match name {
                Some(name)
                    if self.declared_has(&declared, declaring, name, holder, &name_at)? =>
                {
                    Some(name)
                }
                _ => None,
            };
            let Some(name) = name else {
                return Err(self.refuse(holder, &name_at));
            };
            self.push(&mut named, name, holder, &row_at)?;
        }
        for &(_, name) in &declared {
            if !self.named_has(&named, name, holder, &|| weights_at().key("members"))? {
                return Err(self.refuse(holder, &|| weights_at().key("members")));
            }
        }
        Ok(())
    }

    /// The join of every node in ascending node-id order, one work unit for
    /// each node met.
    fn run(&mut self) -> Result<(), Failure> {
        let package = self.package;
        for id in package.node_ids_ascending() {
            let Some(position) = package.node_position(id) else {
                continue;
            };
            self.charge(position, &|| Self::node_at(position))?;
            let Some(CheckedNodeKind::Probabilistic(form)) = package.kinds.get(position) else {
                continue;
            };
            let Some(body) = package.term_at(position) else {
                continue;
            };
            let Some(declaring) = self.source_type(bound(body, 0).and_then(target)) else {
                continue;
            };
            match form {
                ProbabilisticForm::Workload => self.workload(position, body, declaring)?,
                ProbabilisticForm::DelayDistribution => {
                    self.entry(position, declaring, bound(body, 1).and_then(text))?;
                }
                ProbabilisticForm::RandomParameter => {
                    let parameters =
                        self.entry(position, declaring, bound(body, 1).and_then(text))?;
                    self.random_parameter(position, body, parameters)?;
                }
                ProbabilisticForm::Reward => {
                    let parameters =
                        self.entry(position, declaring, bound(body, 1).and_then(text))?;
                    self.reward(position, body, parameters)?;
                }
            }
        }
        Ok(())
    }
}

impl SelectedSourceObjects<'_> {
    /// Join every operation reference of a `probabilistic` node whose
    /// `declaring` node (a workload's `model`) is source-declared through the
    /// identity row of that node, in ascending node-id order (FR-369). Each
    /// node met, operation reference joined, operation entry and supertype
    /// edge read, support value checked and reward function parameter
    /// compared is one work unit of `budget`, charged ahead of the work, with
    /// one more for each byte of caller text it compares; the lists the join
    /// keeps and the pointer of a refusal are charged to its logical storage
    /// and the exact numbers it parses to its read storage, as the strict
    /// read charges them.
    ///
    /// `Ok(())` means the typed reads `delay_distributions`,
    /// `random_parameters`, `rewards` and `workloads` of the package are
    /// joined, and the work and storage the join charged are the budget's
    /// last success delta.
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
        let before = budget.consumed();
        let mut join = Join {
            package: self.package(),
            selected: self,
            meter: WorkMeter::over(budget),
        };
        let joined = join.run();
        if joined.is_ok() {
            if let Some(budget) = join.meter.budget_mut() {
                budget.finish(before);
            }
        }
        joined
    }
}
