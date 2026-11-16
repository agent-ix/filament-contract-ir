// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FSpec FR-640 `probabilistic` node wire shapes: the closed record, the
//! ordered bindings of each of the four forms and the value rules that need
//! no other node.

use super::native_join_budget::NativeJoinBudget;
use super::natural::coprime;
use super::read_charge;
use super::stage_storage::{self, Refusal, Site};
use super::{
    BodyTerm, LiteralKind, PackageReadStage, ProbabilisticForm,
    RawSemanticNodeV2 as CheckedSemanticNodeV2, TermNode, TermView, WorkMeter,
};
use crate::checked_package::common::{Step, Trail, ValidationFailure};
use crate::checked_package::shared::{
    CheckedOccurrenceRole, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
};

/// The deepest closed position is a delay-distribution `discrete` member's
/// binding value, below the node, `body`, `members`, an index, `value`,
/// `members`, an index, `members` and an index.
#[derive(Clone, Copy)]
struct Path {
    steps: [Step<'static>; 16],
    len: usize,
}
impl Path {
    fn key(mut self, key: &'static str) -> Self {
        self.steps[self.len] = Step::Key(key);
        self.len += 1;
        self
    }
    fn index(mut self, index: usize) -> Self {
        self.steps[self.len] = Step::Index(index);
        self.len += 1;
        self
    }
    fn trail(&self) -> Trail<'_> {
        Trail::Base(&self.steps[..self.len])
    }
    fn holder(&self) -> usize {
        match self.steps[..self.len] {
            [Step::Key("semantic_graph"), Step::Key("nodes"), Step::Index(index), ..] => index,
            _ => unreachable!("probabilistic positions are graph-node positions"),
        }
    }
    fn site(&self) -> Site<'_> {
        Site::new(
            PackageReadStage::WireChecks,
            self.holder(),
            "probabilistic",
            self.trail(),
        )
    }
}
fn node_path(position: usize) -> Path {
    Path {
        steps: [Step::Index(0); 16],
        len: 0,
    }
    .key("semantic_graph")
    .key("nodes")
    .index(position)
}

/// The first failure, kept on the stack until the refusal is built.
enum Fail {
    /// A shape outside the closed record: `malformed_wire`.
    Wire(Box<Path>),
    /// A well-shaped member whose value is outside its rule: `invalid_package`
    /// with cause `invalid-value`.
    Value(Box<Path>),
    Storage(ValidationFailure),
}
impl From<ValidationFailure> for Fail {
    fn from(failure: ValidationFailure) -> Self {
        Self::Storage(failure)
    }
}

fn wire(at: Path) -> Fail {
    Fail::Wire(Box::new(at))
}
fn invalid(at: Path) -> Fail {
    Fail::Value(Box::new(at))
}

/// Validates one node: the closed outer record, then the ordered body.
#[ix_trace_rs::string_edge]
pub(super) fn check_wire(
    node: &CheckedSemanticNodeV2,
    form: ProbabilisticForm,
    position: usize,
    budget: &mut NativeJoinBudget,
) -> Result<(), ValidationFailure> {
    let at = node_path(position);
    match check_node(node, form, at, budget) {
        Ok(()) => Ok(()),
        Err(Fail::Storage(failure)) => Err(failure),
        Err(Fail::Wire(failed)) => Err(stage_storage::refuse(
            &mut WorkMeter::over(budget),
            &failed.site(),
            Refusal::new(CheckedPackageRefusalCode::MalformedWire, failed.trail()),
        )),
        Err(Fail::Value(failed)) => {
            let mut refusal =
                Refusal::new(CheckedPackageRefusalCode::InvalidPackage, failed.trail());
            refusal.cause = Some(CheckedPackageRefusalCause::InvalidValue);
            Err(stage_storage::refuse(
                &mut WorkMeter::over(budget),
                &failed.site(),
                refusal,
            ))
        }
    }
}

fn visit(budget: &mut NativeJoinBudget, at: Path) -> Result<(), Fail> {
    stage_storage::work(&mut WorkMeter::over(budget), &at.site(), 1)?;
    Ok(())
}

#[ix_trace_rs::string_edge]
fn check_node(
    node: &CheckedSemanticNodeV2,
    form: ProbabilisticForm,
    at: Path,
    budget: &mut NativeJoinBudget,
) -> Result<(), Fail> {
    visit(budget, at)?;
    if node.recursion_group.is_some() {
        return Err(wire(at.key("recursion_group")));
    }
    if node.owner.is_some() {
        return Err(wire(at.key("owner")));
    }
    if node.declaration.is_some() {
        return Err(wire(at.key("declaration")));
    }
    if node.occurrences.len() != 1 {
        return Err(wire(at.key("occurrences")));
    }
    if node.occurrences[0].role != CheckedOccurrenceRole::Declaration {
        return Err(wire(at.key("occurrences").index(0).key("role")));
    }
    let body_at = at.key("body");
    let body = TermNode::of(&node.body);
    match form {
        ProbabilisticForm::RandomParameter => {
            let b = bind(
                body,
                ["declaring", "operation", "parameter", "support"],
                body_at,
            )?;
            reference(b[0])?;
            text(b[1])?;
            text(b[2])?;
            let support = aggregate(b[3].0, b[3].1)?;
            if support.len() == 0 {
                return Err(invalid(b[3].1.key("members")));
            }
            let mut seen = read_charge::with_capacity(
                budget,
                read_charge::Site::new(PackageReadStage::WireChecks, at.holder(), "probabilistic"),
                support.len(),
            )?;
            for (index, row) in support.enumerate() {
                let row_at = b[3].1.key("members").index(index);
                visit(budget, row_at)?;
                let r = bind(row, ["value", "weight"], row_at)?;
                let key = literal(r[0], index)?;
                weight(budget, r[1], true)?;
                seen.push((key, r[0].1));
            }
            // A support value that repeats an earlier one is refused at the
            // later row, in declared order.
            let mut ordered = read_charge::with_capacity(
                budget,
                read_charge::Site::new(PackageReadStage::WireChecks, at.holder(), "probabilistic"),
                seen.len(),
            )?;
            ordered.extend(
                seen.iter()
                    .enumerate()
                    .map(|(index, (key, _))| (*key, index)),
            );
            ordered.sort_unstable();
            let mut repeated: Option<usize> = None;
            for pair in ordered.windows(2) {
                if pair[0].0 == pair[1].0 {
                    repeated = Some(repeated.map_or(pair[1].1, |first| first.min(pair[1].1)));
                }
            }
            if let Some(index) = repeated {
                return Err(invalid(seen[index].1));
            }
        }
        ProbabilisticForm::Workload => {
            let b = bind(body, ["model", "name", "weights"], body_at)?;
            reference(b[0])?;
            text(b[1])?;
            for (index, row) in aggregate(b[2].0, b[2].1)?.enumerate() {
                let row_at = b[2].1.key("members").index(index);
                visit(budget, row_at)?;
                let r = bind(row, ["declaring", "operation", "weight"], row_at)?;
                reference(r[0])?;
                text(r[1])?;
                weight(budget, r[2], false)?;
            }
        }
        ProbabilisticForm::Reward => {
            let b = bind(
                body,
                ["declaring", "operation", "name", "value_type", "expression"],
                body_at,
            )?;
            reference(b[0])?;
            text(b[1])?;
            text(b[2])?;
            reference(b[3])?;
            reference(b[4])?;
        }
        ProbabilisticForm::DelayDistribution => {
            let b = bind(
                body,
                ["declaring", "operation", "family", "parameters"],
                body_at,
            )?;
            reference(b[0])?;
            text(b[1])?;
            let family = text(b[2])?;
            let (value, value_at) = b[3];
            match family {
                "uniform" => {
                    if family_members(value, value_at)?.len() != 0 {
                        return Err(invalid(value_at.key("members")));
                    }
                }
                "uniform_interval" => {
                    let p = family_bind(value, ["lower", "upper"], value_at)?;
                    quantity(p[0])?;
                    quantity(p[1])?;
                }
                "exponential" => {
                    let p = family_bind(value, ["rate"], value_at)?;
                    weight(budget, p[0], true)?;
                }
                "discrete" => {
                    for (index, row) in family_members(value, value_at)?.enumerate() {
                        let row_at = value_at.key("members").index(index);
                        visit(budget, row_at)?;
                        let r = family_bind(row, ["delay", "weight"], row_at)?;
                        quantity(r[0])?;
                        weight(budget, r[1], true)?;
                    }
                }
                _ => return Err(invalid(b[2].1.key("value"))),
            }
        }
    }
    Ok(())
}

type Bound<'a> = (TermNode<'a>, Path);

/// An aggregate term's members.
fn aggregate(value: TermNode<'_>, at: Path) -> Result<super::TermItems<'_>, Fail> {
    if !value.exact_members(&["term", "members"]) || value.kind() != Some(BodyTerm::Aggregate) {
        return Err(wire(at));
    }
    value
        .get("members")
        .and_then(TermNode::items)
        .ok_or(wire(at.key("members")))
}

/// An aggregate of exactly the bindings `names`, in that order, with each
/// bound value and its position.
fn bind<'a, const N: usize>(
    value: TermNode<'a>,
    names: [&str; N],
    at: Path,
) -> Result<[Bound<'a>; N], Fail> {
    let members = aggregate(value, at)?;
    if members.len() != N {
        return Err(wire(at.key("members")));
    }
    let mut bound = [(value, at); N];
    for (index, (member, name)) in members.zip(names).enumerate() {
        let member_at = at.key("members").index(index);
        match member.view() {
            Some(TermView::Binding {
                name: actual,
                value,
            }) if actual == name => {
                bound[index] = (value, member_at.key("value"));
            }
            _ => return Err(wire(member_at)),
        }
    }
    Ok(bound)
}

/// The members of a delay `parameters` aggregate. The value must be an
/// aggregate term; what it holds is judged against the family.
fn family_members(value: TermNode<'_>, at: Path) -> Result<super::TermItems<'_>, Fail> {
    aggregate(value, at)
}

/// A delay `parameters` aggregate, or a member row of one, that does not hold
/// exactly the bindings its family states: FR-640 Reader rule 5,
/// `invalid-value`.
fn family_bind<'a, const N: usize>(
    value: TermNode<'a>,
    names: [&str; N],
    at: Path,
) -> Result<[Bound<'a>; N], Fail> {
    aggregate(value, at)?;
    bind(value, names, at).map_err(|failure| match failure {
        Fail::Wire(failed) => Fail::Value(failed),
        other => other,
    })
}

fn reference((value, at): Bound<'_>) -> Result<(), Fail> {
    match value.view() {
        Some(TermView::Reference { .. }) => Ok(()),
        _ => Err(wire(at)),
    }
}

/// The spelling of a literal of `kind`.
fn spelling<'a>((value, at): Bound<'a>, kind: LiteralKind) -> Result<&'a str, Fail> {
    match value.view() {
        Some(TermView::Literal {
            value_kind, value, ..
        }) if value_kind == kind => value.as_str().ok_or(wire(at.key("value"))),
        _ => Err(wire(at)),
    }
}

fn text<'a>(bound: Bound<'a>) -> Result<&'a str, Fail> {
    spelling(bound, LiteralKind::Text)
}

/// What identifies a support value: its literal kind and spelling. A value
/// that is neither text nor a natural number is never equal to another.
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
enum SupportKey<'a> {
    Text(&'a str, &'a str),
    Natural(&'a str, u64),
    Distinct(usize),
}

/// A literal term, as the key of the support row at `index`.
fn literal<'a>((value, at): Bound<'a>, index: usize) -> Result<SupportKey<'a>, Fail> {
    match value.view() {
        Some(TermView::Literal {
            value_kind, value, ..
        }) => Ok(match (value.as_str(), value.as_u64()) {
            (Some(text), _) => SupportKey::Text(value_kind.as_wire(), text),
            (None, Some(number)) => SupportKey::Natural(value_kind.as_wire(), number),
            (None, None) => SupportKey::Distinct(index),
        }),
        _ => Err(wire(at)),
    }
}

/// The numerator and denominator digits of an exact rational spelling `n`,
/// `n/d` or either with a leading `-`, and whether it is negative.
#[ix_trace_rs::string_edge]
fn exact_parts(spelling: &str) -> Option<(bool, &str, &str)> {
    let natural = |s: &str| {
        s == "0"
            || (s.starts_with(|c: char| ('1'..='9').contains(&c))
                && s.bytes().all(|c| c.is_ascii_digit()))
    };
    let (negative, unsigned) = match spelling.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, spelling),
    };
    let (numerator, denominator) = unsigned.split_once('/').unwrap_or((unsigned, "1"));
    (natural(numerator) && natural(denominator) && denominator != "0").then_some((
        negative,
        numerator,
        denominator,
    ))
}

/// An exact weight or rate (FR-205): an exact rational in lowest terms. A
/// spelling that is not one, a negative value and, when `positive`, zero are
/// `invalid-value` (FR-640 Reader rules 2, 3 and 5).
#[ix_trace_rs::string_edge]
fn weight(budget: &mut NativeJoinBudget, bound: Bound<'_>, positive: bool) -> Result<(), Fail> {
    let at = bound.1;
    let spelling = spelling(bound, LiteralKind::Rational)?;
    let Some((negative, numerator, denominator)) = exact_parts(spelling) else {
        return Err(invalid(at.key("value")));
    };
    let zero = numerator == "0";
    if negative || (zero && positive) {
        return Err(invalid(at.key("value")));
    }
    if !coprime(
        numerator,
        denominator,
        &mut WorkMeter::over(budget),
        at.site(),
    )? {
        return Err(invalid(at.key("value")));
    }
    Ok(())
}

/// A time quantity: an exact non-negative rational spelling.
fn quantity(bound: Bound<'_>) -> Result<(), Fail> {
    let at = bound.1;
    match exact_parts(spelling(bound, LiteralKind::Quantity)?) {
        Some((false, _, _)) => Ok(()),
        _ => Err(wire(at.key("value"))),
    }
}

#[cfg(test)]
mod tests {
    use super::super::flat_wire::check;
    use super::super::{
        CheckedNodeKind, CheckedNodeTag, JsonPointer, NativeJoinBudget, ProbabilisticForm,
        RawCheckedPackageWireV2, TermNode,
    };
    use crate::checked_package::common::ValidationFailure;
    use crate::checked_package::shared::{CheckedPackageRefusalCause, CheckedPackageRefusalCode};
    use ix_trace_rs::trace;
    use serde_json::{json, Value};

    fn id(name: &str) -> Value {
        json!({"package": "test/probabilistic", "path": [name], "ordinal": 0})
    }
    fn binding(name: &str, value: Value) -> Value {
        json!({"term": "binding", "name": name, "value": value})
    }
    fn aggregate(members: Vec<Value>) -> Value {
        json!({"term": "aggregate", "members": members})
    }
    fn reference(name: &str) -> Value {
        json!({"term": "reference", "target": id(name)})
    }
    fn literal(kind: &str, value: &str) -> Value {
        json!({"term": "literal", "type": id("type"), "value_kind": kind, "value": value})
    }
    fn text(value: &str) -> Value {
        literal("text", value)
    }
    fn operation(name: &str) -> Vec<Value> {
        vec![
            binding("declaring", reference("model")),
            binding("operation", text(name)),
        ]
    }
    fn body(mut head: Vec<Value>, tail: Vec<Value>) -> Value {
        head.extend(tail);
        aggregate(head)
    }
    fn support_row(value: &str, weight: &str) -> Value {
        aggregate(vec![
            binding("value", literal("integer", value)),
            binding("weight", literal("rational", weight)),
        ])
    }
    fn random_parameter(rows: Vec<Value>) -> Value {
        body(
            operation("attempt"),
            vec![
                binding("parameter", text("n")),
                binding("support", aggregate(rows)),
            ],
        )
    }
    fn workload(weight: &str) -> Value {
        body(
            vec![
                binding("model", reference("model")),
                binding("name", text("Steady")),
            ],
            vec![binding(
                "weights",
                aggregate(vec![aggregate(vec![
                    binding("declaring", reference("model")),
                    binding("operation", text("attempt")),
                    binding("weight", literal("rational", weight)),
                ])]),
            )],
        )
    }
    fn reward() -> Value {
        body(
            operation("attempt"),
            vec![
                binding("name", text("duration")),
                binding("value_type", reference("millis")),
                binding("expression", reference("duration")),
            ],
        )
    }
    fn delay(family: &str, parameters: Value) -> Value {
        body(
            operation("reply"),
            vec![
                binding("family", text(family)),
                binding("parameters", parameters),
            ],
        )
    }
    fn quantity(name: &str, value: &str) -> Value {
        binding(name, literal("quantity", value))
    }
    fn discrete_row(delay: &str, weight: &str) -> Value {
        aggregate(vec![
            quantity("delay", delay),
            binding("weight", literal("rational", weight)),
        ])
    }

    fn node(form: &str, body: Value) -> Value {
        let id =
            json!({"package": "test/probabilistic", "path": ["probabilistic", form], "ordinal": 0});
        json!({
            "schema_version": super::super::GRAPH_V2, "node_id": id.clone(), "semantic_type": id,
            "node_tag": "probabilistic", "semantic_form": form, "dependencies": [],
            "occurrences": [{"role": "declaration", "ordinal": 0}], "body": body,
        })
    }

    fn wire(nodes: Vec<Value>) -> Value {
        json!({
            "contract_version": "filament.checked-package/v2",
            "name": "test/probabilistic",
            "lock": {
                "sources": [], "edition": {"role": "edition", "definition": {"authority": "test", "identity": "edition"}},
                "profile_selections": [], "definition_selections": [],
                "required_features": [], "dependency_selections": [],
            },
            "semantic_graph": {"graph_version": "filament.checked-semantic-graph/v2", "nodes": nodes, "operand_rows": vec![Vec::<u64>::new(); nodes.len()]},
            "source_map": [], "capability_report": [],
            "diagnostics": {"catalog": {"authority": "test", "identity": "diagnostics"}, "entries": []},
        })
    }

    fn read(input: &Value) -> Result<RawCheckedPackageWireV2, ValidationFailure> {
        let wire: RawCheckedPackageWireV2 =
            serde_json::from_value(input.clone()).expect("private wire input");
        let bodies = wire
            .semantic_graph
            .nodes
            .iter()
            .map(|node| node.body.clone())
            .collect::<Vec<_>>();
        check(&wire, &bodies, &mut NativeJoinBudget::unlimited()).map(|_| wire)
    }

    fn refusal(
        input: &Value,
        code: CheckedPackageRefusalCode,
        cause: Option<CheckedPackageRefusalCause>,
        path: &str,
    ) {
        let Err(ValidationFailure::Refused(refusal)) = read(input) else {
            panic!("the strict reader refuses {path}");
        };
        assert_eq!(refusal.code, code);
        assert_eq!(refusal.cause, cause);
        assert_eq!(refusal.loci[0].path.clone(), JsonPointer::parse(path));
    }

    fn shapes() -> Vec<(&'static str, Value)> {
        vec![
            (
                "random_parameter",
                random_parameter(vec![support_row("1", "9"), support_row("3", "7/10")]),
            ),
            ("workload", workload("1")),
            ("reward", reward()),
            ("delay_distribution", delay("uniform", aggregate(vec![]))),
            (
                "delay_distribution",
                delay(
                    "uniform_interval",
                    aggregate(vec![quantity("lower", "1"), quantity("upper", "4")]),
                ),
            ),
            (
                "delay_distribution",
                delay(
                    "exponential",
                    aggregate(vec![binding("rate", literal("rational", "1/2"))]),
                ),
            ),
            (
                "delay_distribution",
                delay(
                    "discrete",
                    aggregate(vec![discrete_row("1", "90"), discrete_row("3", "10")]),
                ),
            ),
        ]
    }

    /// FR-640's tag and four forms decode, and every shape (each delay family
    /// included) reads and encodes back to the document it was read from.
    #[trace("TC-476", "FR-369-AC-1")]
    #[test]
    fn each_probabilistic_shape_decodes_and_round_trips() {
        assert_eq!(
            CheckedNodeTag::from_wire("probabilistic"),
            Some(CheckedNodeTag::Probabilistic)
        );
        for form in [
            "random_parameter",
            "workload",
            "reward",
            "delay_distribution",
        ] {
            let kind = CheckedNodeKind::decode(CheckedNodeTag::Probabilistic, form)
                .expect("an FR-640 form");
            assert_eq!(kind.tag(), CheckedNodeTag::Probabilistic);
            assert_eq!(kind.form_wire(), form);
            assert!(CheckedNodeKind::all().contains(&kind));
        }
        assert_eq!(ProbabilisticForm::ALL.len(), 4);
        for (form, body) in shapes() {
            let input = wire(vec![node(form, body)]);
            let read = read(&input).expect("an FR-640 shape reads");
            assert_eq!(serde_json::to_value(&read).expect("encodes"), input);
        }
    }

    /// A probabilistic node with one member missing, one member added or the
    /// members out of order is refused as malformed wire at that member.
    #[trace("TC-476", "FR-369-AC-2")]
    #[test]
    fn a_missing_or_extra_member_is_refused() {
        for (form, body) in shapes() {
            let members = body["members"].as_array().expect("aggregate").clone();
            let last = members.len() - 1;
            let at = "/semantic_graph/nodes/0/body/members";

            let mut missing = members.clone();
            missing.pop();
            refusal(
                &wire(vec![node(form, aggregate(missing))]),
                CheckedPackageRefusalCode::MalformedWire,
                None,
                at,
            );

            let mut extra = members.clone();
            extra.push(binding("extra", text("x")));
            refusal(
                &wire(vec![node(form, aggregate(extra))]),
                CheckedPackageRefusalCode::MalformedWire,
                None,
                at,
            );

            if last > 0 {
                let mut swapped = members.clone();
                swapped.swap(0, 1);
                refusal(
                    &wire(vec![node(form, aggregate(swapped))]),
                    CheckedPackageRefusalCode::MalformedWire,
                    None,
                    &format!("{at}/0"),
                );
            }

            let mut declared = node(form, body.clone());
            declared["declaration"] = json!({"qualified_name": ["Main"]});
            refusal(
                &wire(vec![declared]),
                CheckedPackageRefusalCode::MalformedWire,
                None,
                "/semantic_graph/nodes/0/declaration",
            );
        }
        // A member inside a support, a workload weight and a delay row.
        let mut row = support_row("1", "9");
        row["members"].as_array_mut().expect("row").pop();
        refusal(
            &wire(vec![node("random_parameter", random_parameter(vec![row]))]),
            CheckedPackageRefusalCode::MalformedWire,
            None,
            "/semantic_graph/nodes/0/body/members/3/value/members/0/members",
        );
        let workload_row = |edit: &dyn Fn(&mut Vec<Value>)| {
            let mut body = workload("1");
            let rows = body["members"][2]["value"]["members"]
                .as_array_mut()
                .expect("weights");
            edit(rows[0]["members"].as_array_mut().expect("weights row"));
            body
        };
        let weights_at = "/semantic_graph/nodes/0/body/members/2/value/members/0/members";
        refusal(
            &wire(vec![node("workload", workload_row(&|row| drop(row.pop())))]),
            CheckedPackageRefusalCode::MalformedWire,
            None,
            weights_at,
        );
        refusal(
            &wire(vec![node(
                "workload",
                workload_row(&|row| row.push(binding("extra", text("x")))),
            )]),
            CheckedPackageRefusalCode::MalformedWire,
            None,
            weights_at,
        );
    }

    /// Values outside FR-640's rules that need no other node are refused as
    /// `invalid-value`.
    #[trace("TC-476", "FR-369-AC-3")]
    #[test]
    fn a_value_outside_the_form_rules_is_refused() {
        let invalid = |form, body, path: &str| {
            refusal(
                &wire(vec![node(form, body)]),
                CheckedPackageRefusalCode::InvalidPackage,
                Some(CheckedPackageRefusalCause::InvalidValue),
                path,
            );
        };
        invalid(
            "random_parameter",
            random_parameter(vec![]),
            "/semantic_graph/nodes/0/body/members/3/value/members",
        );
        invalid(
            "random_parameter",
            random_parameter(vec![support_row("1", "0")]),
            "/semantic_graph/nodes/0/body/members/3/value/members/0/members/1/value/value",
        );
        invalid(
            "delay_distribution",
            delay("gamma", aggregate(vec![])),
            "/semantic_graph/nodes/0/body/members/2/value/value",
        );
        invalid(
            "delay_distribution",
            delay("discrete", aggregate(vec![discrete_row("1", "0")])),
            "/semantic_graph/nodes/0/body/members/3/value/members/0/members/1/value/value",
        );
        // A workload admits weight 0 and a rate must be above 0.
        read(&wire(vec![node("workload", workload("0"))])).expect("weight 0 admits");
        invalid(
            "delay_distribution",
            delay(
                "exponential",
                aggregate(vec![binding("rate", literal("rational", "0"))]),
            ),
            "/semantic_graph/nodes/0/body/members/3/value/members/0/value/value",
        );
        // A negative or non-exact weight or rate, and one not in lowest terms.
        for spelling in ["-1", "-1/2", "-0", "2/2", "0/5", "01", "1/0", "x"] {
            invalid(
                "workload",
                workload(spelling),
                "/semantic_graph/nodes/0/body/members/2/value/members/0/members/2/value/value",
            );
        }
        invalid(
            "delay_distribution",
            delay(
                "exponential",
                aggregate(vec![binding("rate", literal("rational", "-1"))]),
            ),
            "/semantic_graph/nodes/0/body/members/3/value/members/0/value/value",
        );
        // Parameters that do not match the family.
        let parameters_at = "/semantic_graph/nodes/0/body/members/3/value";
        invalid(
            "delay_distribution",
            delay("uniform", aggregate(vec![quantity("lower", "1")])),
            &format!("{parameters_at}/members"),
        );
        invalid(
            "delay_distribution",
            delay("uniform_interval", aggregate(vec![quantity("lower", "1")])),
            &format!("{parameters_at}/members"),
        );
        invalid(
            "delay_distribution",
            delay(
                "exponential",
                aggregate(vec![quantity("lower", "1"), quantity("upper", "2")]),
            ),
            &format!("{parameters_at}/members"),
        );
        invalid(
            "delay_distribution",
            delay(
                "exponential",
                aggregate(vec![binding("lambda", literal("rational", "1/2"))]),
            ),
            &format!("{parameters_at}/members/0"),
        );
        let mut row = discrete_row("1", "90");
        row["members"]
            .as_array_mut()
            .expect("row")
            .push(binding("extra", text("x")));
        invalid(
            "delay_distribution",
            delay("discrete", aggregate(vec![row])),
            &format!("{parameters_at}/members/0/members"),
        );
        // A support that repeats a value is refused at the later row.
        invalid(
            "random_parameter",
            random_parameter(vec![
                support_row("1", "9"),
                support_row("3", "1"),
                support_row("1", "2"),
            ]),
            "/semantic_graph/nodes/0/body/members/3/value/members/2/members/0/value",
        );
        read(&wire(vec![node(
            "random_parameter",
            random_parameter(vec![support_row("1", "9"), support_row("3", "9")]),
        )]))
        .expect("equal weights over distinct values admit");
    }

    /// A tag outside the vocabulary is still refused with the decoder's
    /// existing `unsupported_node_tag`, and a form outside a family is not
    /// decoded as a probabilistic form.
    #[trace("TC-476", "FR-369-AC-4")]
    #[test]
    fn an_unknown_tag_still_fails_decode_with_the_existing_error() {
        assert_eq!(CheckedNodeTag::from_wire("probabilistic_model"), None);
        assert_eq!(
            CheckedNodeKind::decode(CheckedNodeTag::Probabilistic, "random-parameter"),
            None
        );
        assert_eq!(
            CheckedNodeKind::decode(CheckedNodeTag::Probabilistic, "monitor_plan"),
            None
        );
        let mut unknown = node(
            "random_parameter",
            random_parameter(vec![support_row("1", "9")]),
        );
        unknown["node_tag"] = json!("probabilistic_model");
        refusal(
            &wire(vec![unknown]),
            CheckedPackageRefusalCode::UnsupportedNodeTag,
            None,
            "/semantic_graph/nodes/0/node_tag",
        );
    }

    fn wire_refusal(input: &Value, path: &str) {
        refusal(input, CheckedPackageRefusalCode::MalformedWire, None, path);
    }

    /// Each guard of the closed node record refuses, and a guard removed
    /// leaves the package admitted.
    #[trace("TC-476", "FR-369-AC-2")]
    #[test]
    fn the_closed_node_record_guards_each_refuse() {
        let base = || node("workload", workload("1"));
        let at = |member: &str| format!("/semantic_graph/nodes/0/{member}");
        let mut owned = base();
        owned["owner"] = json!({"kind": "model_root", "identity": "test/probabilistic"});
        wire_refusal(&wire(vec![owned]), &at("owner"));
        let mut grouped = base();
        grouped["recursion_group"] = json!("group");
        wire_refusal(&wire(vec![grouped]), &at("recursion_group"));
        let mut none = base();
        none["occurrences"] = json!([]);
        wire_refusal(&wire(vec![none]), &at("occurrences"));
        let mut two = base();
        two["occurrences"] = json!([
            {"role": "declaration", "ordinal": 0},
            {"role": "declaration", "ordinal": 1}
        ]);
        wire_refusal(&wire(vec![two]), &at("occurrences"));
        let mut role = base();
        role["occurrences"][0]["role"] = json!("expression");
        wire_refusal(&wire(vec![role]), &at("occurrences/0/role"));
        read(&wire(vec![base()])).expect("the unedited record admits");
    }

    /// A binding of the wrong term kind refuses at that binding.
    #[trace("TC-476", "FR-369-AC-2")]
    #[test]
    fn a_binding_of_the_wrong_term_kind_is_refused() {
        // The reward's value_type and expression are references.
        for member in [3, 4] {
            let mut body = reward();
            body["members"][member]["value"] = text("x");
            wire_refusal(
                &wire(vec![node("reward", body)]),
                &format!("/semantic_graph/nodes/0/body/members/{member}/value"),
            );
        }
        // A time quantity is a quantity literal.
        let mut interval = delay(
            "uniform_interval",
            aggregate(vec![quantity("lower", "1"), quantity("upper", "4")]),
        );
        interval["members"][3]["value"]["members"][0]["value"] = literal("rational", "1");
        wire_refusal(
            &wire(vec![node("delay_distribution", interval)]),
            "/semantic_graph/nodes/0/body/members/3/value/members/0/value",
        );
        // A weight is a rational literal.
        let mut row = support_row("1", "9");
        row["members"][1]["value"] = literal("integer", "9");
        wire_refusal(
            &wire(vec![node("random_parameter", random_parameter(vec![row]))]),
            "/semantic_graph/nodes/0/body/members/3/value/members/0/members/1/value",
        );
    }

    /// The reader charges one visit per node and one per support, weight or
    /// delay member before inspecting it, and stops at the first uncharged one.
    #[trace("TC-476", "FR-369-AC-5")]
    #[test]
    fn one_visit_is_charged_per_node_and_per_member() {
        let ample = || NativeJoinBudget::new(u64::MAX, u64::MAX).with_read_storage_limit(u64::MAX);
        let limited =
            |work| NativeJoinBudget::new(work, u64::MAX).with_read_storage_limit(u64::MAX);
        let wire_of = |form: &str, body: Value| {
            let wire: RawCheckedPackageWireV2 =
                serde_json::from_value(wire(vec![node(form, body)])).expect("wire");
            wire
        };
        let run = |budget: &mut NativeJoinBudget, form: &str, body: Value| {
            let wire = wire_of(form, body);
            let CheckedNodeKind::Probabilistic(kind) =
                CheckedNodeKind::decode(CheckedNodeTag::Probabilistic, form).expect("a form")
            else {
                panic!("a probabilistic kind");
            };
            super::check_wire(&wire.semantic_graph.nodes[0], kind, 0, budget)
        };
        let work = |form: &str, body: Value| {
            let mut budget = ample();
            run(&mut budget, form, body).expect("admits");
            budget.consumed().work
        };
        // The work one exact weight costs, measured on its own.
        let weight_work = |spelling: &str| {
            let wire = wire_of("workload", workload(spelling));
            let body = TermNode::of(&wire.semantic_graph.nodes[0].body);
            let rows = body
                .get("members")
                .and_then(TermNode::items)
                .expect("members");
            let weights = rows
                .last()
                .expect("weights binding")
                .get("value")
                .expect("value");
            let row = weights
                .get("members")
                .and_then(TermNode::items)
                .and_then(|mut rows| rows.next())
                .expect("a weights row");
            let weight_term = row
                .get("members")
                .and_then(TermNode::items)
                .and_then(|rows| rows.last())
                .and_then(|binding| binding.get("value"))
                .expect("the weight");
            let mut budget = ample();
            super::weight(&mut budget, (weight_term, super::node_path(0)), false)
                .unwrap_or_else(|_| panic!("a valid weight"));
            budget.consumed().work
        };
        let w = weight_work("1");
        assert!(w > 0);
        let workload_rows = |rows: usize| {
            let mut body = workload("1");
            let row = body["members"][2]["value"]["members"][0].clone();
            body["members"][2]["value"]["members"] = json!(vec![row; rows]);
            body
        };
        // The node is one visit; each row is one visit and its weight.
        assert_eq!(work("workload", workload_rows(0)), 1);
        assert_eq!(work("workload", workload_rows(1)), 1 + (1 + w));
        assert_eq!(work("workload", workload_rows(3)), 1 + 3 * (1 + w));
        assert_eq!(work("reward", reward()), 1);
        let support = |rows: Vec<&str>| {
            random_parameter(
                rows.into_iter()
                    .map(|value| support_row(value, "1"))
                    .collect(),
            )
        };
        assert_eq!(
            work("random_parameter", support(vec!["1", "2", "3"]))
                - work("random_parameter", support(vec!["1", "2"])),
            1 + w
        );
        let discrete = |rows: Vec<&str>| {
            delay(
                "discrete",
                aggregate(
                    rows.into_iter()
                        .map(|delay| discrete_row(delay, "1"))
                        .collect(),
                ),
            )
        };
        assert_eq!(
            work("delay_distribution", discrete(vec!["1", "2", "3"]))
                - work("delay_distribution", discrete(vec!["1", "2"])),
            1 + w
        );
        // A budget that holds the node visit but not the first row's refuses
        // at that row, with no partial view.
        let mut budget = limited(1);
        let failure = run(&mut budget, "workload", workload_rows(2)).expect_err("short");
        assert!(
            matches!(failure, ValidationFailure::Budget(_)),
            "{failure:?}"
        );
        assert_eq!(budget.consumed().work, 1);
        assert!(budget.first_unpaid().is_some());
        let mut none = limited(0);
        run(&mut none, "reward", reward()).expect_err("no allowance for the node");
        assert_eq!(none.consumed().work, 0);
    }
}
