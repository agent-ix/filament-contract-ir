// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The identity-row join of FR-369: every operation reference of a
//! `probabilistic` node whose `declaring` node (a workload's `model`) is a
//! source-declared object type is joined through the type's identity row.
//!
//! Implements: FR-369 "Behavior" (identity-row join), FR-358.

use filament_exact::Integer;

use super::model_members::{IntegerBounds, RationalBounds};
use super::native_join_budget::{
    NativeJoinBudget, NativeJoinChargeKind, NativeJoinLocus, NativeJoinLocusSource,
};
use super::probabilistic_join::{
    bound, bounded_not_member, members, target, text, Bounded, ExactParse, NotMember,
};
use super::source_object::{
    is_source_declared_object_type, SelectedSourceObjectError, SelectedSourceObjectErrorCode,
    SelectedSourceObjectMember, SelectedSourceObjectRow, SelectedSourceObjects,
};
use super::SelectedSourceEnumRow;
use super::{
    BoundedDomainForm, CheckedNodeId, CheckedNodeKind, CheckedPackageV2, CheckedTerm, LiteralKind,
    ProbabilisticForm, ScalarTypeForm, ValueForm,
};
use crate::checked_package::shared::{CheckedPackageRefusalLocation, JsonPointer};

const FORM: &str = "probabilistic_join";

/// The logical storage units of the pointer a failure names, charged once
/// ahead of the join: one container and one unit for each byte of a pointer of
/// at most a dozen short segments.
const LOCUS_UNITS: u128 = 1 + 256;

/// The decimal digits a 32-bit limb of an exact number holds at most, so a
/// parse of `n` digits owns at most `n / 9 + 1` limbs.
const DIGITS_PER_LIMB: usize = 9;

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

/// Whether the enumeration case `candidate` is the case `case` names.
#[ix_trace_rs::string_edge]
fn is_case(candidate: &str, case: &str) -> bool {
    candidate == case
}

/// The literal kind, the unit node position of a quantity and the membership
/// rule of the values of a data entry's type node.
struct Slot<'p> {
    kind: LiteralKind,
    unit: Option<usize>,
    rule: Bounded<'p>,
    /// The source enumeration an enumeration entry's type node is, whose cases
    /// its support values name.
    enumeration: Option<&'p CheckedNodeId>,
}

struct Join<'p, 'b> {
    package: &'p CheckedPackageV2,
    selected: &'p SelectedSourceObjects<'p>,
    /// The admitted source enumerations (FR-352) the join checks enumeration
    /// literals against.
    enums: &'b [SelectedSourceEnumRow],
    /// The caller's budget: work and logical storage are charged to it.
    budget: &'b mut NativeJoinBudget,
}

/// The exact-number parses of a support value, each paid in logical storage
/// ahead of being made: one unit for the number and one for each limb.
struct Parser<'a> {
    budget: &'a mut NativeJoinBudget,
    position: usize,
}

impl Parser<'_> {
    fn pay(&mut self, digits: usize) -> Result<(), ()> {
        let units = 1 + digits.div_ceil(DIGITS_PER_LIMB) as u128;
        let locus = NativeJoinLocus {
            source: NativeJoinLocusSource::GraphNode(self.position),
            form: FORM,
        };
        self.budget
            .accounted_charge(NativeJoinChargeKind::Storage, units, locus)
            .map_err(|_| ())
    }
}

impl ExactParse for Parser<'_> {
    type Error = ();

    fn exact_integer(&mut self, text: &str) -> Result<Option<Integer>, ()> {
        self.pay(text.len())?;
        Ok(text.parse().ok())
    }

    fn canonical_rational(&mut self, text: &str) -> Result<Option<(Integer, Integer)>, ()> {
        let Some((numerator, denominator)) = text.split_once('/') else {
            return Ok(self
                .exact_integer(text)?
                .map(|numerator| (numerator, Integer::one())));
        };
        let (Some(numerator), Some(denominator)) = (
            self.exact_integer(numerator)?,
            self.exact_integer(denominator)?,
        ) else {
            return Ok(None);
        };
        if denominator <= Integer::one() {
            return Ok(None);
        }
        // The divisor is a number of the size of the larger operand.
        self.pay(text.len())?;
        Ok((numerator.gcd(&denominator) == Integer::one()).then_some((numerator, denominator)))
    }
}

/// `term` as the integer literal it spells, for the bounds of a type node the
/// strict read has already admitted.
fn integer_of(term: Option<&CheckedTerm>) -> Option<i128> {
    let CheckedTerm::Literal { value, .. } = term? else {
        return None;
    };
    value.as_str()?.parse().ok()
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

    /// The stop at `at`, the first member left uncharged. Its pointer is the
    /// one paid for ahead of the join.
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
        self.budget
            .accounted_charge(kind, amount, locus)
            .map_err(|_| Self::incomplete(at))
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

    /// The refusal at `at`; its pointer was paid for ahead of the join.
    fn refuse(at: At) -> Failure {
        Failure {
            code: SelectedSourceObjectErrorCode::InvalidValue,
            locus: Self::location(at),
        }
    }

    /// Pushes onto `list`, charging a logical storage unit for the element and
    /// one for the container when it is the first, ahead of the push.
    fn push<T>(
        &mut self,
        list: &mut Vec<T>,
        item: T,
        position: usize,
        at: At,
    ) -> Result<(), Failure> {
        let units = if list.is_empty() { 2 } else { 1 };
        self.pay(NativeJoinChargeKind::Storage, units, position, at)?;
        list.try_reserve(1).map_err(|_| Self::incomplete(at))?;
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
    fn slot(&self, entry: &'p SelectedSourceObjectMember) -> Option<Slot<'p>> {
        let package = self.package;
        let at = package.position_of(entry.type_node.as_ref()?)?;
        let body = package.term_at(at)?;
        let plain = |kind| {
            Some(Slot {
                kind,
                unit: None,
                rule: Bounded::Free,
                enumeration: None,
            })
        };
        match package.kinds.get(at)? {
            CheckedNodeKind::ScalarType(form) => match form {
                ScalarTypeForm::Boolean => Some(Slot {
                    kind: LiteralKind::Boolean,
                    unit: None,
                    rule: Bounded::Boolean,
                    enumeration: None,
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
                    enumeration: None,
                }),
                ScalarTypeForm::Enum => Some(Slot {
                    kind: LiteralKind::Enum,
                    unit: None,
                    rule: Bounded::Free,
                    enumeration: entry.type_node.as_ref(),
                }),
                ScalarTypeForm::CarriedQuantity => {
                    let unit = bound(body, 0).and_then(target)?;
                    let carrier = package.position_of(bound(body, 1).and_then(target)?)?;
                    Some(Slot {
                        kind: LiteralKind::Quantity,
                        unit: Some(package.position_of(unit)?),
                        rule: Bounded::Rationals(self.rational_bounds(carrier)?),
                        enumeration: None,
                    })
                }
                ScalarTypeForm::Bytes
                | ScalarTypeForm::JsonObject
                | ScalarTypeForm::Dimension
                | ScalarTypeForm::DimensionEquivalence
                | ScalarTypeForm::CompoundUnit
                | ScalarTypeForm::BoundedQuantity
                | ScalarTypeForm::ClockReference => None,
            },
            CheckedNodeKind::BoundedDomain(BoundedDomainForm::IntegerRange) => Some(Slot {
                kind: LiteralKind::Integer,
                unit: None,
                rule: Bounded::Integers(IntegerBounds {
                    lower: integer_of(bound(body, 0))?,
                    upper: integer_of(bound(body, 1))?,
                }),
                enumeration: None,
            }),
            CheckedNodeKind::BoundedDomain(BoundedDomainForm::RationalRange) => Some(Slot {
                kind: LiteralKind::Rational,
                unit: None,
                rule: Bounded::Rationals(self.rational_bounds(at)?),
                enumeration: None,
            }),
            CheckedNodeKind::BoundedDomain(BoundedDomainForm::TextBounds) => Some(Slot {
                kind: LiteralKind::Text,
                unit: None,
                rule: Bounded::Text {
                    lower: natural_of(bound(body, 0))?,
                    upper: natural_of(bound(body, 1))?,
                    profile: text(bound(body, 2)?)?,
                },
                enumeration: None,
            }),
            CheckedNodeKind::BoundedDomain(
                BoundedDomainForm::DecimalRange
                | BoundedDomainForm::FloatRounding
                | BoundedDomainForm::CollectionBounds
                | BoundedDomainForm::ModelPopulation
                | BoundedDomainForm::FiniteTracePopulation,
            )
            | CheckedNodeKind::Extension(_)
            | CheckedNodeKind::Hybrid(_)
            | CheckedNodeKind::TimedModel(_)
            | CheckedNodeKind::Probabilistic(_)
            | CheckedNodeKind::CompositeType(_)
            | CheckedNodeKind::Value(_)
            | CheckedNodeKind::Expression(_)
            | CheckedNodeKind::Function(_)
            | CheckedNodeKind::Model(_)
            | CheckedNodeKind::Relation(_)
            | CheckedNodeKind::State(_)
            | CheckedNodeKind::Temporal(_)
            | CheckedNodeKind::Protocol(_)
            | CheckedNodeKind::Claim(_)
            | CheckedNodeKind::Correspondence(_) => None,
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
            return Err(Self::refuse(&at));
        };
        for entry in &row.operations {
            self.charge_text(holder, entry.identity.len(), &at)?;
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
            return Err(Self::refuse(&parameter_at));
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
            if let Some(enumeration) = slot.enumeration {
                let type_at = || value_at().key("type");
                self.enum_member(holder, enumeration, type_id, value, &value_at, &type_at)?;
                continue;
            }
            let mut parser = Parser {
                budget: &mut *self.budget,
                position: holder,
            };
            let refused = bounded_not_member(&mut parser, slot.rule, value)
                .map_err(|()| Self::incomplete(&value_at))?;
            match refused {
                Some(NotMember::Literal) => return Err(Self::refuse(&value_at)),
                Some(NotMember::Spelling) => {
                    return Err(Self::refuse(&|| value_at().key("value")));
                }
                None => {}
            }
        }
        Ok(())
    }

    /// The support value `value` typed `type_id` against the cases of the
    /// enumeration `enumeration` its entry is typed by: one visit for each
    /// enumeration row met and, with the bytes of its text, one for each case
    /// compared. An enumeration `enums` does not hold refuses at the literal's
    /// `type` member; a literal typed at another declaration, or naming no
    /// case, at the literal.
    fn enum_member(
        &mut self,
        holder: usize,
        enumeration: &CheckedNodeId,
        type_id: &CheckedNodeId,
        value: &super::CheckedJson,
        value_at: At,
        type_at: At,
    ) -> Result<(), Failure> {
        let mut found = None;
        for row in self.enums {
            self.charge(holder, value_at)?;
            if row.declaration.structural_node == *enumeration {
                found = Some(row);
                break;
            }
        }
        let Some(row) = found else {
            return Err(Self::refuse(type_at));
        };
        if type_id != enumeration {
            return Err(Self::refuse(value_at));
        }
        let Some(case) = value.as_str() else {
            return Err(Self::refuse(value_at));
        };
        for member in &row.members {
            self.charge_text(holder, member.case.len(), value_at)?;
            if is_case(&member.case, case) {
                return Ok(());
            }
        }
        Err(Self::refuse(value_at))
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
                return Err(Self::refuse(&expression_at));
            }
        }
        if parameters.len() > listed.len() {
            return Err(Self::refuse(&expression_at));
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
                return Err(Self::refuse(&|| {
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
                return Err(Self::refuse(&name_at));
            };
            self.push(&mut named, name, holder, &row_at)?;
        }
        for &(_, name) in &declared {
            if !self.named_has(&named, name, holder, &|| weights_at().key("members"))? {
                return Err(Self::refuse(&|| weights_at().key("members")));
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
    /// one more for each byte of caller text it compares. Logical storage
    /// (FR-358 storage units) is charged ahead as well: the pointer of the
    /// failure the join may name first, then one unit for each list the join
    /// keeps and for each element, and for each exact number it parses one
    /// unit and one for each limb. No read storage is charged.
    ///
    /// `Ok(())` means the typed reads `delay_distributions`,
    /// `random_parameters`, `rewards` and `workloads` of the package are
    /// joined, and the work and storage the join charged are the budget's
    /// last success delta. A budget that cannot pay for the failure pointer
    /// stops the join as incomplete with no locus.
    ///
    /// # Errors
    ///
    /// [`SelectedSourceObjectErrorCode::InvalidValue`] at the member the
    /// strict read would name, for the first node that disagrees with its
    /// row; [`SelectedSourceObjectErrorCode::Incomplete`] at the first
    /// uncharged member when `budget` denies a charge, with no partial view.
    pub fn join_probabilistic(
        &self,
        enums: &[SelectedSourceEnumRow],
        budget: &mut NativeJoinBudget,
    ) -> Result<(), SelectedSourceObjectError> {
        budget.resume();
        let before = budget.consumed();
        // The pointer of the failure the join may name is paid for first; a
        // budget that cannot pay for it stops the join with no locus.
        let locus = NativeJoinLocus {
            source: NativeJoinLocusSource::Package,
            form: FORM,
        };
        budget
            .accounted_charge(NativeJoinChargeKind::Storage, LOCUS_UNITS, locus)
            .map_err(|_| Failure {
                code: SelectedSourceObjectErrorCode::Incomplete,
                locus: None,
            })?;
        let mut join = Join {
            package: self.package(),
            selected: self,
            enums,
            budget,
        };
        let joined = join.run();
        if joined.is_ok() {
            join.budget.finish(before);
        }
        joined
    }
}
