// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FSpec FR-640 `probabilistic` node wire shapes: the closed record, the
//! ordered bindings of each of the four forms and the value rules that need
//! no other node.

use super::native_join_budget::NativeJoinBudget;
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
            for (index, row) in support.enumerate() {
                let row_at = b[3].1.key("members").index(index);
                visit(budget, row_at)?;
                let r = bind(row, ["value", "weight"], row_at)?;
                literal(r[0])?;
                weight(r[1], true)?;
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
                weight(r[2], false)?;
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
                    if aggregate(value, value_at)?.len() != 0 {
                        return Err(wire(value_at.key("members")));
                    }
                }
                "uniform_interval" => {
                    let p = bind(value, ["lower", "upper"], value_at)?;
                    quantity(p[0])?;
                    quantity(p[1])?;
                }
                "exponential" => {
                    let p = bind(value, ["rate"], value_at)?;
                    weight(p[0], true)?;
                }
                "discrete" => {
                    for (index, row) in aggregate(value, value_at)?.enumerate() {
                        let row_at = value_at.key("members").index(index);
                        visit(budget, row_at)?;
                        let r = bind(row, ["delay", "weight"], row_at)?;
                        quantity(r[0])?;
                        weight(r[1], true)?;
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

fn literal((value, at): Bound<'_>) -> Result<(), Fail> {
    match value.view() {
        Some(TermView::Literal { .. }) => Ok(()),
        _ => Err(wire(at)),
    }
}

/// Whether `spelling` is an exact non-negative rational `n` or `n/d`, and
/// whether it is zero.
#[ix_trace_rs::string_edge]
fn exact_zero(spelling: &str) -> Option<bool> {
    let natural = |s: &str| {
        s == "0"
            || (s.starts_with(|c: char| ('1'..='9').contains(&c))
                && s.bytes().all(|c| c.is_ascii_digit()))
    };
    let (numerator, denominator) = spelling.split_once('/').unwrap_or((spelling, "1"));
    (natural(numerator) && natural(denominator) && denominator != "0").then_some(numerator == "0")
}

/// An exact weight or rate; `positive` refuses zero.
fn weight(bound: Bound<'_>, positive: bool) -> Result<(), Fail> {
    let at = bound.1;
    match exact_zero(spelling(bound, LiteralKind::Rational)?) {
        None => Err(wire(at.key("value"))),
        Some(true) if positive => Err(invalid(at.key("value"))),
        Some(_) => Ok(()),
    }
}

fn quantity(bound: Bound<'_>) -> Result<(), Fail> {
    let at = bound.1;
    exact_zero(spelling(bound, LiteralKind::Quantity)?)
        .map(|_| ())
        .ok_or(wire(at.key("value")))
}

#[cfg(test)]
mod tests {
    use super::super::flat_wire::check;
    use super::super::{
        CheckedNodeKind, CheckedNodeTag, JsonPointer, NativeJoinBudget, ProbabilisticForm,
        RawCheckedPackageWireV2,
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
        let mut row = discrete_row("1", "90");
        row["members"]
            .as_array_mut()
            .expect("row")
            .push(binding("extra", text("x")));
        refusal(
            &wire(vec![node(
                "delay_distribution",
                delay("discrete", aggregate(vec![row])),
            )]),
            CheckedPackageRefusalCode::MalformedWire,
            None,
            "/semantic_graph/nodes/0/body/members/3/value/members/0/members",
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
}
