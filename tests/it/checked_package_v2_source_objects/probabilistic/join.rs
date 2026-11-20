// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-484: the identity-row join half of FR-369-AC-25 and FR-369-AC-26
//! (`SelectedSourceObjects::join_probabilistic`, QSL-2645).

use super::super::*;
use super::{
    aggregate, declared, id_value, int_literal, package, range, rational, redeclare, redepend,
    reference_to, reward_function, segments, target_at,
};
use filament_contract_model::{NativeJoinBudget, NativeJoinChargeKind, SelectedSourceObjectError};

/// An allowance that bites on work only: storage never runs out.
fn work_only(work: u64) -> NativeJoinBudget {
    NativeJoinBudget::new(work, u64::MAX)
}

const CLOCK: &str = "ix://filament/native/Clock";

fn data(name: &str, node: &Value) -> SelectedSourceObjectMember {
    SelectedSourceObjectMember::data(
        name,
        serde_json::from_value(node.clone()).expect("type node"),
        false,
    )
}

fn operation(
    identity: &str,
    parameters: Vec<SelectedSourceObjectMember>,
) -> SelectedSourceObjectOperation {
    SelectedSourceObjectOperation {
        identity: identity.into(),
        query: false,
        clock_parameters: parameters
            .iter()
            .filter(|entry| entry.member_type.is_some())
            .map(|entry| entry.name.clone())
            .collect(),
        parameters,
    }
}

/// The rows of the AC-25 unit: `Order`, `O` declaring `step` and `Other`
/// declaring `poke(parameters)`, the supertypes of `Other` being `supers`.
fn rows(
    poke: Vec<SelectedSourceObjectMember>,
    supers: Vec<CheckedNodeId>,
) -> Vec<SelectedSourceObjectRow> {
    ["Order", "O", "Other"]
        .iter()
        .map(|name| {
            let mut row = row(name, vec![], &[]);
            row.members = match *name {
                "Order" => vec![data("total", &integer())],
                "Other" => vec![data("n", &integer())],
                _ => vec![],
            };
            row.operations = match *name {
                "O" => vec![operation("O::step", vec![])],
                "Other" => vec![operation("Other::poke", poke.clone())],
                _ => vec![],
            };
            if *name == "Other" {
                row.supertypes = supers.clone();
            }
            row
        })
        .collect()
}

fn poke_rows() -> Vec<SelectedSourceObjectRow> {
    rows(vec![data("k", &range())], vec![])
}

fn joined(
    package: &Value,
    rows: Vec<SelectedSourceObjectRow>,
    budget: &mut NativeJoinBudget,
) -> Result<(), SelectedSourceObjectError> {
    let package = admitted(package);
    let selected = SelectedSourceObjects::from_stage_rows(&package, rows).expect("rows join");
    selected.join_probabilistic(&[], budget)
}

fn join(
    package: &Value,
    rows: Vec<SelectedSourceObjectRow>,
) -> Result<(), SelectedSourceObjectError> {
    joined(package, rows, &mut NativeJoinBudget::unlimited())
}

fn locus_of(error: &SelectedSourceObjectError) -> &str {
    error
        .locus
        .as_ref()
        .and_then(|locus| locus.path.as_ref())
        .map(|path| path.as_str())
        .expect("a located refusal")
}

/// Asserts the join refuses `InvalidValue` at `path`.
fn refuses(case: &str, package: &Value, rows: Vec<SelectedSourceObjectRow>, path: &str) {
    let error = join(package, rows).expect_err(case);
    assert_eq!(error.code, RowError::InvalidValue, "{case}: {error:?}");
    assert_eq!(locus_of(&error), path, "{case}");
}

fn by_label(package: &Value, label: &str) -> usize {
    nodes(package)
        .iter()
        .position(|node| node["node_id"]["path"][2] == label)
        .expect("probabilistic node")
}

fn edit_label(package: &mut Value, label: &str, change: impl FnOnce(&mut Value)) {
    let position = by_label(package, label);
    change(&mut nodes_mut(package)[position]);
    settle(package);
}

fn member_at(package: &Value, label: &str, member: usize) -> String {
    format!(
        "/semantic_graph/nodes/{}/body/members/{member}/value",
        by_label(package, label)
    )
}

/// The `value` of the literal at `member`: where the strict read names an
/// `operation` or a `parameter` it refuses (`member_at` is the literal).
fn literal_value_at(package: &Value, label: &str, member: usize) -> String {
    format!("{}/value", member_at(package, label, member))
}

/// The AC-25 package with the reward function listing `listed`, the parameter
/// nodes `added` (label, name) being added to the graph.
fn with_reward(listed: &[&str], added: &[(&str, &str)]) -> Value {
    let mut package = package();
    for (label, name) in added {
        nodes_mut(&mut package).push(super::parameter(label, name));
    }
    let function = id_value("qualified:Other/poke/reward/cost");
    nodes_mut(&mut package).retain(|node| node["node_id"] != function);
    nodes_mut(&mut package).push(reward_function(listed));
    settle(&mut package);
    package
}

/// FR-369-AC-25: the AC-25 package is admitted by the strict read and by the
/// identity-row join, and the typed read is the checked-in golden one.
#[trace("TC-484", "FR-369-AC-25")]
#[test]
fn the_golden_package_joins_and_its_typed_read_is_the_golden_one() {
    let package = indexed(&package());
    if std::env::var_os("QSL_2645_WRITE_GOLDEN").is_some() {
        let write = |name: &str, value: &Value| {
            std::fs::create_dir_all(golden_path("")).expect("golden directory");
            std::fs::write(
                golden_path(name),
                serde_json::to_string_pretty(value).expect("json") + "\n",
            )
            .expect("write golden");
        };
        write("package.json", &package);
        write("typed-read.json", &typed_read(&self::admitted(&package)));
        panic!(
            "the golden pair is rewritten: review its diff, then run without QSL_2645_WRITE_GOLDEN"
        );
    }
    let golden = golden_value("package.json");
    assert_eq!(golden, package, "the golden package is stale");
    let admitted = self::admitted(&golden);
    let selected =
        SelectedSourceObjects::from_stage_rows(&admitted, poke_rows()).expect("rows join");
    let mut budget = NativeJoinBudget::unlimited();
    selected
        .join_probabilistic(&[], &mut budget)
        .expect("joined");
    let read = typed_read(&admitted);
    assert_eq!(read, golden_value("typed-read.json"));
    let independent: CheckedNodeId = serde_json::from_value(json!({
        "package": node_id("x")["package"],
        "path": super::POKE_REWARD, "ordinal": 0}))
    .expect("declared node id");
    let rewards = admitted.rewards(&id("qualified:Other"), "poke");
    assert_eq!(rewards[0].function, &independent);
}

fn golden_path(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/qsl-2645-golden")
        .join(name)
}

fn golden_value(name: &str) -> Value {
    serde_json::from_slice(&std::fs::read(golden_path(name)).expect("golden file"))
        .expect("golden json")
}

/// The typed read of every declaration of the AC-25 unit.
fn typed_read(package: &CheckedPackageV2) -> Value {
    let (o, other) = (id("qualified:O"), id("qualified:Other"));
    let rational = |value: &filament_contract_model::CheckedRational| {
        json!([&*value.numerator, &*value.denominator])
    };
    let delays: Vec<Value> = [&o, &other]
        .iter()
        .flat_map(|model| package.delay_distributions(model))
        .map(|delay| json!({"operation": delay.operation, "family": format!("{:?}", delay.family)}))
        .collect();
    let parameters: Vec<Value> = package
        .random_parameters(&other, "poke")
        .iter()
        .map(|parameter| {
            json!({"parameter": parameter.parameter,
                "support": parameter.support.iter()
                    .map(|row| rational(&row.probability)).collect::<Vec<_>>()})
        })
        .collect();
    let rewards: Vec<Value> = package
        .rewards(&other, "poke")
        .iter()
        .map(|reward| {
            json!({"name": reward.name,
                "value_type": reward.value_type,
                "function": reward.function})
        })
        .collect();
    let workloads: Vec<Value> = package
        .workloads(&o)
        .iter()
        .map(|workload| {
            json!({"name": workload.name,
                "weights": workload.weights.iter()
                    .map(|weight| json!([weight.declaring, weight.operation]))
                    .collect::<Vec<_>>()})
        })
        .collect();
    json!({"delay_distributions": delays, "random_parameters": parameters,
        "rewards": rewards, "workloads": workloads})
}

/// FR-369-AC-26 at the identity-row join: an `operation` that `Other`'s row
/// lists no entry for, a `parameter` `j` that `Other::poke` lacks, a support
/// value `4` outside `Int[1, 3]` and a workload that omits `O::step`, and an
/// operation the reward or the delay names that no entry carries.
#[trace("TC-484", "FR-369-AC-26")]
#[test]
fn the_join_refuses_each_unjoined_member_at_the_strict_reads_locus() {
    for label in ["delay_poke", "random_k", "reward_cost"] {
        let mut broken = package();
        if label == "reward_cost" {
            // The reward's own function is declared under the operation.
            redeclare(&mut broken, &["Other", "nudge", "reward", "cost"]);
        }
        edit_label(&mut broken, label, |node| {
            node["body"]["members"][1]["value"]["value"] = json!("nudge");
        });
        let path = literal_value_at(&broken, label, 1);
        refuses(&format!("{label} operation"), &broken, poke_rows(), &path);
    }
    let mut broken = package();
    edit_label(&mut broken, "random_k", |node| {
        node["body"]["members"][2]["value"]["value"] = json!("j");
    });
    let path = literal_value_at(&broken, "random_k", 2);
    refuses("a parameter Other::poke lacks", &broken, poke_rows(), &path);
    let mut broken = package();
    edit_label(&mut broken, "random_k", |node| {
        node["body"]["members"][3]["value"]["members"][1]["members"][0]["value"]["value"] =
            json!("4");
    });
    let path = format!(
        "{}/members/1/members/0/value",
        member_at(&broken, "random_k", 3)
    );
    refuses(
        "a support value outside the range",
        &broken,
        poke_rows(),
        &path,
    );
    let mut broken = package();
    edit_label(&mut broken, "workload_w", |node| {
        node["body"]["members"][2]["value"] = aggregate(vec![]);
    });
    let path = format!("{}/members", member_at(&broken, "workload_w", 2));
    refuses("a workload omitting O::step", &broken, poke_rows(), &path);
}

/// The member path below the node of the strict read's refusal of a node of
/// the selected model `Account` (which the join leaves to the strict read),
/// with `form`'s body edited by `edit`.
fn strict_locus(form: &str, body: Vec<Value>, edit: impl FnOnce(&mut Value)) -> String {
    let account = node_id(&family_key("1515"));
    let mut members = vec![binding(
        "declaring",
        json!({"term": "reference", "target": account}),
    )];
    members.extend(body);
    let mut package = super::package_over(mixed_package());
    let mut node = super::probabilistic("selected", form, aggregate(members));
    edit(&mut node["body"]);
    let mut dependencies = node["dependencies"]
        .as_array()
        .expect("dependencies")
        .clone();
    dependencies.sort_by_key(segments);
    node["dependencies"] = json!(dependencies);
    nodes_mut(&mut package).push(node);
    settle(&mut package);
    let refusal = refused(&package);
    let path = refusal.loci[0].path.as_ref().expect("a located refusal");
    let path = path.as_str();
    let body = path.find("/body/").expect("a member of a node body");
    path[body..].to_owned()
}

/// FR-369-AC-26: the join names the member the strict read names, the
/// `value` of the literal for an `operation` and a `parameter`, derived here
/// from the strict read's own refusal of the same edit over a selected model.
#[trace("TC-484", "FR-369-AC-26")]
#[test]
fn the_join_names_the_loci_the_strict_read_names() {
    let strict_operation = strict_locus(
        "delay_distribution",
        vec![
            binding("operation", text("missing")),
            binding("family", text("uniform")),
            binding("parameters", aggregate(vec![])),
        ],
        |_| {},
    );
    let strict_parameter = strict_locus(
        "random_parameter",
        vec![
            binding("operation", text("deposit")),
            binding("parameter", text("missing")),
            binding(
                "support",
                aggregate(vec![aggregate(vec![
                    binding("value", int_literal(&integer(), "1")),
                    binding("weight", rational("1")),
                ])]),
            ),
        ],
        |_| {},
    );
    let below = |path: String| path[path.find("/body/").expect("body")..].to_owned();
    let mut operation = package();
    edit_label(&mut operation, "delay_poke", |node| {
        node["body"]["members"][1]["value"]["value"] = json!("nudge");
    });
    let error = join(&operation, poke_rows()).expect_err("an operation Other lacks");
    assert_eq!(below(locus_of(&error).to_owned()), strict_operation);
    let mut parameter = package();
    edit_label(&mut parameter, "random_k", |node| {
        node["body"]["members"][2]["value"]["value"] = json!("j");
    });
    let error = join(&parameter, poke_rows()).expect_err("a parameter poke lacks");
    assert_eq!(below(locus_of(&error).to_owned()), strict_parameter);
}

/// FR-369-AC-26: a reward whose function lists `j` in place of `k`, or lists
/// none, refuses at the reward's `expression`.
#[trace("TC-484", "FR-369-AC-26")]
#[test]
fn a_reward_function_that_does_not_list_the_operations_parameters_refuses() {
    let admitted = with_reward(&["fixture/src/k"], &[]);
    join(&admitted, poke_rows()).expect("k alone is the operation's parameter list");
    for (case, listed) in [
        ("j in place of k", vec!["fixture/src/j"]),
        ("k then j", vec!["fixture/src/k", "fixture/src/j"]),
        ("no parameter", vec![]),
    ] {
        let broken = with_reward(&listed, &[("fixture/src/j", "j")]);
        let path = target_at(&broken, "reward", 4);
        refuses(case, &broken, poke_rows(), &path);
    }
}

/// The AC-25 package whose reward function lists `listed` (the parameter nodes
/// `added` being added) and computes `body`: the node of that label, or when
/// `constant` the new node `fixture/src/one`, the constant `1` typed
/// `Int[1, 3]`. The function is the one of the reward's own operation.
fn with_reward_body(listed: &[&str], added: &[(&str, &str)], body: &str, constant: bool) -> Value {
    let mut package = with_reward(listed, added);
    if constant {
        nodes_mut(&mut package).push(plain(
            body,
            "value",
            "literal",
            &range(),
            &[],
            "generated",
            int_literal(&range(), "1"),
        ));
    }
    let function = id_value("qualified:Other/poke/reward/cost");
    let at = nodes(&package)
        .iter()
        .position(|node| node["node_id"] == function)
        .expect("function");
    let node = &mut nodes_mut(&mut package)[at];
    node["body"]["members"]
        .as_array_mut()
        .expect("members")
        .insert(1, binding("body", reference_to(body)));
    redepend(node);
    settle(&mut package);
    package
}

/// FR-369-AC-26 step 5 of TC-484: a reward function that has a body. The
/// strict read admits each package (so the refusal is the join's); a function
/// listing `j` read by its body in place of `k`, and one with no parameter and
/// the constant body `1`, refuse at the reward's `expression`, and the
/// reward's `operation` rewritten to `nudge` refuses at the operation.
#[trace("TC-484", "FR-369-AC-26")]
#[test]
fn a_reward_function_with_a_body_is_joined_by_its_listed_parameters() {
    let k = with_reward_body(&["fixture/src/k"], &[], "fixture/src/k", false);
    admitted(&k);
    join(&k, poke_rows()).expect("k read by the body is the operation's parameter list");
    let j = with_reward_body(
        &["fixture/src/j"],
        &[("fixture/src/j", "j")],
        "fixture/src/j",
        false,
    );
    admitted(&j);
    refuses(
        "j read by the body",
        &j,
        poke_rows(),
        &target_at(&j, "reward", 4),
    );
    let constant = with_reward_body(&[], &[], "fixture/src/one", true);
    admitted(&constant);
    refuses(
        "no parameter, the constant body 1",
        &constant,
        poke_rows(),
        &target_at(&constant, "reward", 4),
    );
    let mut nudged = with_reward_body(&["fixture/src/k"], &[], "fixture/src/k", false);
    redeclare(&mut nudged, &["Other", "nudge", "reward", "cost"]);
    edit_label(&mut nudged, "reward_cost", |node| {
        node["body"]["members"][1]["value"]["value"] = json!("nudge");
    });
    admitted(&nudged);
    refuses(
        "a reward of an operation Other lacks",
        &nudged,
        poke_rows(),
        &literal_value_at(&nudged, "reward_cost", 1),
    );
}

/// FR-369-AC-26: the clock-parameter variant. A clock entry is a parameter
/// like a data entry: the reward function lists `x` then `k`; `k` alone refuses.
#[trace("TC-484", "FR-369-AC-26")]
#[test]
fn a_clock_parameter_is_listed_by_the_reward_like_a_data_parameter() {
    let rows = || {
        rows(
            vec![
                SelectedSourceObjectMember::clock("x", CLOCK),
                data("k", &range()),
            ],
            vec![],
        )
    };
    let listed = with_reward(
        &["fixture/src/x", "fixture/src/k"],
        &[("fixture/src/x", "x")],
    );
    join(&listed, rows()).expect("x then k joins");
    let alone = with_reward(&["fixture/src/k"], &[("fixture/src/x", "x")]);
    let path = target_at(&alone, "reward", 4);
    refuses("k alone", &alone, rows(), &path);
    let swapped = with_reward(
        &["fixture/src/k", "fixture/src/x"],
        &[("fixture/src/x", "x")],
    );
    refuses("k then x", &swapped, rows(), &path);
    // A random parameter of a clock entry has no literal kind.
    let mut clocked = package();
    edit_label(&mut clocked, "random_k", |node| {
        node["body"]["members"][2]["value"]["value"] = json!("x");
    });
    let path = format!(
        "{}/members/0/members/0/value",
        member_at(&clocked, "random_k", 3)
    );
    refuses("a random parameter of a clock", &clocked, rows(), &path);
}

/// FR-369-AC-26: a workload over a model with supertypes covers its own and
/// the inherited operations, and refuses an omitted one, a row naming a type
/// outside the lineage and one naming an operation its type lacks.
#[trace("TC-484", "FR-369-AC-26")]
#[test]
fn a_source_workload_covers_the_inherited_operations() {
    let row = |declaring: &str, operation: &str| {
        aggregate(vec![
            binding("declaring", reference_to(declaring)),
            binding("operation", text(operation)),
            binding("weight", rational("1")),
        ])
    };
    let with_workload = |rows: Vec<Value>| {
        let mut package = package();
        nodes_mut(&mut package).push(super::probabilistic(
            "workload_other",
            "workload",
            aggregate(vec![
                binding("model", reference_to("qualified:Other")),
                binding("name", text("W2")),
                binding("weights", aggregate(rows)),
            ]),
        ));
        settle(&mut package);
        package
    };
    let supers = || rows(vec![data("k", &range())], vec![id("qualified:O")]);
    let both = vec![row("qualified:O", "step"), row("qualified:Other", "poke")];
    join(&with_workload(both), supers()).expect("own and inherited operations");
    let omitted = with_workload(vec![row("qualified:Other", "poke")]);
    let weights = format!("{}/members", member_at(&omitted, "workload_other", 2));
    refuses(
        "an omitted inherited operation",
        &omitted,
        supers(),
        &weights,
    );
    // Without the supertype the same rows name an operation outside the lineage.
    let both = with_workload(vec![
        row("qualified:O", "step"),
        row("qualified:Other", "poke"),
    ]);
    let row_at = format!("{}/members/0", member_at(&both, "workload_other", 2));
    refuses(
        "a type outside the lineage",
        &both,
        poke_rows(),
        &format!("{row_at}/members/0/value/target"),
    );
    // `step` is an operation of `O`, not of `Other`.
    let wrong = with_workload(vec![
        row("qualified:Other", "poke"),
        row("qualified:Other", "step"),
    ]);
    let row_at = format!("{}/members/1", member_at(&wrong, "workload_other", 2));
    refuses(
        "an operation of another type of the lineage",
        &wrong,
        supers(),
        &format!("{row_at}/members/1/value/value"),
    );
    let ghost = with_workload(vec![
        row("qualified:O", "step"),
        row("qualified:Other", "ghost"),
    ]);
    let row_at = format!("{}/members/1", member_at(&ghost, "workload_other", 2));
    refuses(
        "an operation the type lacks",
        &ghost,
        supers(),
        &format!("{row_at}/members/1/value/value"),
    );
}

/// FR-640 Reader rule 3 (QSL-2707): a workload's rows are keyed by (declaring
/// node, operation name), so two inherited operations of one name, declared
/// by two supertypes (a diamond), are two rows and the strict read admits
/// them, while one pair twice is a repeat.
#[trace("TC-484", "FR-369-AC-26")]
#[test]
fn a_workload_row_is_keyed_by_its_declaring_node_and_operation_name() {
    let row = |declaring: &str, operation: &str| {
        aggregate(vec![
            binding("declaring", reference_to(declaring)),
            binding("operation", text(operation)),
            binding("weight", rational("1")),
        ])
    };
    let with_workload = |rows: Vec<Value>| {
        let mut package = package();
        nodes_mut(&mut package).push(super::probabilistic(
            "workload_other",
            "workload",
            aggregate(vec![
                binding("model", reference_to("qualified:Other")),
                binding("name", text("W2")),
                binding("weights", aggregate(rows)),
            ]),
        ));
        settle(&mut package);
        package
    };
    // `O::step` and `Other::step`: one name, two declaring nodes.
    let same_name = vec![row("qualified:O", "step"), row("qualified:Other", "step")];
    admitted(&with_workload(same_name));
    // One (declaring, name) pair twice is a repeat, refused at the second row.
    let twice = with_workload(vec![row("qualified:O", "step"), row("qualified:O", "step")]);
    let at = format!("{}/members/1", member_at(&twice, "workload_other", 2));
    let refusal = refused(&twice);
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|path| path.as_str()),
        Some(at.as_str()),
        "{refusal:?}"
    );
}

/// QSL-2707 at the identity-row join: workload coverage is keyed by (declaring
/// node, operation name) like the rows are. In a diamond `D` over `P` and `Q`,
/// each declaring `step`, a workload weighing only one `step` omits the other
/// and is refused, and weighing both is admitted. Where `Other` extends `O`
/// and redeclares `step`, the nearest declaration is the one operation: the
/// workload of `Other::poke` and `Other::step` is admitted, and omitting
/// `Other::step` or weighing the shadowed `O::step` is refused.
#[trace("TC-484", "FR-369-AC-26")]
#[test]
fn a_diamond_workload_must_weigh_every_declaring_nodes_operation_of_one_name() {
    let weigh = |declaring: &str, operation: &str| {
        aggregate(vec![
            binding("declaring", reference_to(declaring)),
            binding("operation", text(operation)),
            binding("weight", rational("1")),
        ])
    };
    let with_workload = |model: &str, extra: &[&str], rows: Vec<Value>| {
        let mut package = package();
        for name in extra {
            let mut node = source_type(name, &[], &fixture_source());
            node["dependencies"] = json!([integer()]);
            nodes_mut(&mut package).push(node);
        }
        nodes_mut(&mut package).push(super::probabilistic(
            "workload_other",
            "workload",
            aggregate(vec![
                binding("model", reference_to(model)),
                binding("name", text("W2")),
                binding("weights", aggregate(rows)),
            ]),
        ));
        settle(&mut package);
        package
    };
    let diamond_rows = || {
        let mut all = poke_rows();
        for (name, operations, supertypes) in [
            ("P", vec![operation("P::step", vec![])], vec![]),
            ("Q", vec![operation("Q::step", vec![])], vec![]),
            ("D", vec![], vec![id("qualified:P"), id("qualified:Q")]),
        ] {
            let mut entry = row(name, vec![], &[]);
            entry.members = vec![];
            entry.operations = operations;
            entry.supertypes = supertypes;
            all.push(entry);
        }
        all
    };
    let weights = |package: &Value| format!("{}/members", member_at(package, "workload_other", 2));
    let both = with_workload(
        "qualified:D",
        &["P", "Q", "D"],
        vec![weigh("qualified:P", "step"), weigh("qualified:Q", "step")],
    );
    join(&both, diamond_rows()).expect("both steps of the diamond");
    for (case, kept) in [("Q::step omitted", "P"), ("P::step omitted", "Q")] {
        let package = with_workload(
            "qualified:D",
            &["P", "Q", "D"],
            vec![weigh(&format!("qualified:{kept}"), "step")],
        );
        refuses(case, &package, diamond_rows(), &weights(&package));
    }
    // `Other` extends `O` and redeclares `step` (FSL FR-185 "Workloads"): the
    // redeclared `O::step` is counted once, as `Other::step`.
    let chain_rows = || {
        let mut all = rows(vec![data("k", &range())], vec![id("qualified:O")]);
        all[2].operations.push(operation("Other::step", vec![]));
        all
    };
    let nearest = with_workload(
        "qualified:Other",
        &[],
        vec![
            weigh("qualified:Other", "poke"),
            weigh("qualified:Other", "step"),
        ],
    );
    join(&nearest, chain_rows()).expect("the redeclared ancestor operation is one operation");
    let omitted = with_workload(
        "qualified:Other",
        &[],
        vec![weigh("qualified:Other", "poke")],
    );
    refuses(
        "the redeclaring Other::step omitted",
        &omitted,
        chain_rows(),
        &weights(&omitted),
    );
    // A row for the redeclared ancestor operation names no operation of the model.
    let both = with_workload(
        "qualified:Other",
        &[],
        vec![
            weigh("qualified:O", "step"),
            weigh("qualified:Other", "poke"),
            weigh("qualified:Other", "step"),
        ],
    );
    let name_at = format!("{}/0/members/1/value/value", weights(&both));
    refuses(
        "the shadowed O::step weighed",
        &both,
        chain_rows(),
        &name_at,
    );
    // A diamond below the redeclaration keeps both nearest declarations: `D`
    // over `P` and `Q`, `P` extending `O`, all three declaring `step`.
    let nested_rows = || {
        let mut all = diamond_rows();
        let p = all
            .iter()
            .position(|row| row.source_identity.as_ref() == "P")
            .unwrap();
        all[p].supertypes = vec![id("qualified:O")];
        all
    };
    let nested = with_workload(
        "qualified:D",
        &["P", "Q", "D"],
        vec![weigh("qualified:P", "step"), weigh("qualified:Q", "step")],
    );
    join(&nested, nested_rows()).expect("P::step and Q::step, O::step redeclared by P");
    let lone = with_workload(
        "qualified:D",
        &["P", "Q", "D"],
        vec![weigh("qualified:P", "step")],
    );
    refuses("Q::step omitted", &lone, nested_rows(), &weights(&lone));
}

/// The work of the AC-25 join of `package`, counted independently of the
/// reader: one unit for each node met, and for the probabilistic ones, in
/// bytes of the caller text compared as well:
///
/// - `delay_poke`: the reference (1) and the entry `Other::poke` (1 + 11);
/// - `delay_step`: the reference (1) and the entry `O::step` (1 + 7);
/// - `random_k`: the reference (1), the entry (1 + 11), the parameter entry
///   `k` (1 + 1) and two support values of one byte each (2 + 2);
/// - `reward_cost`: the reference (1), the entry (1 + 11) and the one listed
///   parameter `k` against the declared name (1 + 1);
/// - `workload_w`: the entry `O::step` of the lineage (1 + 7), its row (1),
///   the lineage lookup of the row's type (1), the operation `step` against
///   the declared ones (1 + 4), the effective-operation check of `step`
///   against the declared ones (1 + 4) and the named ones (1 + 4).
fn ac25_work(package: &Value) -> u64 {
    let delays = (1 + 1 + 11) + (1 + 1 + 7);
    let random = 1 + (1 + 11) + (1 + 1) + (2 + 2);
    let reward = 1 + (1 + 11) + (1 + 1);
    let workload = (1 + 7) + 1 + 1 + (1 + 4) + (1 + 4) + (1 + 4);
    nodes(package).len() as u64 + delays + random + reward + workload
}

/// The logical storage units of the AC-25 join: the pointer of the failure it
/// may name, paid first (one container and 256 bytes), and for the workload
/// four lists (lineage, declared, effective, named), each a container (1) and its one
/// element (1), and for `random_k` the parse of each of the two support
/// values of one digit against `Int[1, 3]` (a number and one limb).
const LOCUS_UNITS: u64 = 1 + 256;
const AC25_STORAGE: u64 = LOCUS_UNITS + 4 * (1 + 1) + 2 * (1 + 1);

/// FR-369-AC-26: the join's charges are ahead of the work: the full count
/// admits, and every smaller work budget stops as incomplete at the first
/// uncharged member, the budget recording the first unpaid charge.
#[trace("TC-484", "FR-369-AC-26")]
#[test]
fn a_budget_one_visit_short_stops_the_join_as_incomplete() {
    let package = package();
    let total = ac25_work(&package);
    let mut budget = work_only(total);
    joined(&package, poke_rows(), &mut budget).expect("the full count admits");
    assert_eq!(budget.consumed().work, total);
    let mut stops = Vec::new();
    for limit in 0..total {
        let mut budget = work_only(limit);
        let error = joined(&package, poke_rows(), &mut budget).expect_err("short");
        assert_eq!(error.code, RowError::Incomplete, "limit {limit}");
        let denied = budget.first_unpaid().expect("first unpaid");
        assert_eq!(denied.kind, NativeJoinChargeKind::Work);
        assert!(budget.consumed().work <= limit, "limit {limit}");
        assert!(
            u128::from(budget.consumed().work) + denied.amount > u128::from(limit),
            "limit {limit}: the denied charge would have fit"
        );
        assert!(budget.first_unpaid_locus().is_some());
        assert_eq!(budget.last_success_delta(), None, "limit {limit}");
        stops.push(locus_of(&error).to_owned());
    }
    // The first charge is a node met; the workload's lookup of `step` among
    // the named operations is charged at the weights.
    assert!(stops[0].starts_with("/semantic_graph/nodes/"));
    assert!(!stops[0].contains("/body"));
    let weights = format!("{}/members", member_at(&package, "workload_w", 2));
    assert!(stops.contains(&weights));
    // The first delay's operation reference is charged at its literal.
    assert!(stops.contains(&literal_value_at(&package, "delay_poke", 1)));
    // The reward's one listed parameter is charged at its `expression`.
    assert!(stops.contains(&target_at(&package, "reward", 4)));
    // A support value is charged at the value, each of the two rows.
    let support = member_at(&package, "random_k", 3);
    for row in 0..2 {
        assert!(stops.contains(&format!("{support}/members/{row}/members/0/value")));
    }
}

/// FR-358: a join that succeeds records what it charged as the budget's last
/// success delta, on top of whatever the budget had consumed before it, and a
/// join that stops records none.
#[trace("TC-484", "FR-369-AC-26")]
#[test]
fn a_joined_package_records_its_charges_as_the_success_delta() {
    let package = package();
    let mut budget = NativeJoinBudget::unlimited();
    let locus = filament_contract_model::NativeJoinLocus {
        source: filament_contract_model::NativeJoinLocusSource::Package,
        form: "test",
    };
    budget
        .accounted_charge(NativeJoinChargeKind::Work, 7, locus)
        .expect("admitted");
    joined(&package, poke_rows(), &mut budget).expect("joined");
    let delta = budget.last_success_delta().expect("a success delta");
    assert_eq!(delta.work, ac25_work(&package));
    assert_eq!(delta.storage, AC25_STORAGE);
    assert_eq!(budget.consumed().work, 7 + delta.work);
    assert!(budget.first_unpaid().is_none());
}

/// FR-358: the lists the join keeps are logical storage of the caller's
/// budget, one unit for each list and each element, charged ahead of the push,
/// after the pointer of the failure: the full count admits and every smaller
/// storage budget stops the join as incomplete, with no locus when the pointer
/// itself cannot be paid, else at the workload.
#[trace("TC-484", "FR-369-AC-26")]
#[test]
fn a_storage_budget_one_unit_short_stops_the_join_as_incomplete() {
    let package = package();
    let budget = |storage| NativeJoinBudget::new(u64::MAX, storage);
    joined(&package, poke_rows(), &mut budget(AC25_STORAGE)).expect("the full count admits");
    let weights = member_at(&package, "workload_w", 2);
    let row = format!("{weights}/members/0");
    let model = format!("{}/target", member_at(&package, "workload_w", 0));
    for limit in 0..AC25_STORAGE {
        let mut short = budget(limit);
        let error = joined(&package, poke_rows(), &mut short).expect_err("short");
        assert_eq!(error.code, RowError::Incomplete, "limit {limit}");
        let denied = short.first_unpaid().expect("first unpaid");
        assert_eq!(denied.kind, NativeJoinChargeKind::Storage);
        assert!(short.consumed().storage <= limit, "limit {limit}");
        if limit < LOCUS_UNITS {
            assert!(error.locus.is_none(), "limit {limit}");
            assert_eq!(denied.amount, u128::from(LOCUS_UNITS));
        } else {
            let at = locus_of(&error);
            let support = member_at(&package, "random_k", 3);
            assert!(
                at == model
                    || at == weights
                    || at == row
                    || at.starts_with(&format!("{support}/members/")),
                "limit {limit}: {at}"
            );
        }
    }
}

/// FR-358: the pointer of the failure is logical storage, paid before the
/// join starts: a budget with the storage of a refusal's join names the member,
/// and one without the pointer's units stops with no locus.
#[trace("TC-484", "FR-369-AC-26")]
#[test]
fn the_pointer_of_a_refusal_is_paid_before_the_join_starts() {
    let mut broken = package();
    edit_label(&mut broken, "random_k", |node| {
        node["body"]["members"][2]["value"]["value"] = json!("j");
    });
    let mut full = NativeJoinBudget::unlimited();
    let error = joined(&broken, poke_rows(), &mut full).expect_err("refused");
    assert_eq!(error.code, RowError::InvalidValue);
    let paid = full.consumed().storage;
    assert!(paid >= LOCUS_UNITS, "the pointer was charged: {paid}");
    let mut exact = NativeJoinBudget::new(u64::MAX, paid);
    let error = joined(&broken, poke_rows(), &mut exact).expect_err("refused");
    assert_eq!(error.code, RowError::InvalidValue);
    let mut short = NativeJoinBudget::new(u64::MAX, LOCUS_UNITS - 1);
    let error = joined(&broken, poke_rows(), &mut short).expect_err("stopped");
    assert_eq!(error.code, RowError::Incomplete);
    assert!(error.locus.is_none());
}

/// FR-358: the exact numbers the join parses are logical storage, paid ahead
/// of the parse (one unit for the number and one for each nine-digit limb), and
/// no read storage: a support value of a hundred digits against `Int[1, 3]`
/// costs 1 + 12 units, is refused when they are paid, and one unit less stops
/// the join as incomplete at the value.
#[trace("TC-484", "FR-369-AC-26")]
#[test]
fn the_exact_numbers_the_join_parses_are_charged_as_logical_storage() {
    let mut package = package();
    edit_label(&mut package, "random_k", |node| {
        node["body"]["members"][3]["value"]["members"][0]["members"][0]["value"]["value"] =
            json!("9".repeat(100));
    });
    let mut full = NativeJoinBudget::unlimited();
    let error = joined(&package, poke_rows(), &mut full).expect_err("outside the range");
    assert_eq!(error.code, RowError::InvalidValue);
    assert_eq!(full.read_storage_consumed(), 0);
    let paid = full.consumed().storage;
    let value = format!(
        "{}/members/0/members/0/value",
        member_at(&package, "random_k", 3)
    );
    let mut short = NativeJoinBudget::new(u64::MAX, paid - 1);
    let error = joined(&package, poke_rows(), &mut short).expect_err("short");
    assert_eq!(error.code, RowError::Incomplete);
    assert_eq!(locus_of(&error), value);
    let denied = short.first_unpaid().expect("first unpaid");
    assert_eq!(denied.kind, NativeJoinChargeKind::Storage);
    assert_eq!(denied.amount, 1 + 12);
    assert!(short.first_unpaid_read_storage().is_none());
    let at = short.first_unpaid_locus().expect("a locus");
    let holder = by_label(&package, "random_k");
    assert!(
        format!("{at:?}").contains(&format!("GraphNode({holder})")),
        "{at:?}"
    );
}

/// FR-358: the join starts its own accounting phase on the caller's allowance:
/// a denial recorded by an earlier operation does not stop it.
#[trace("TC-484", "FR-369-AC-26")]
#[test]
fn the_join_resumes_a_budget_that_an_earlier_operation_denied() {
    let mut budget = work_only(5);
    let locus = filament_contract_model::NativeJoinLocus {
        source: filament_contract_model::NativeJoinLocusSource::Package,
        form: "test",
    };
    budget
        .accounted_charge(NativeJoinChargeKind::Work, 6, locus)
        .expect_err("denied");
    joined(&package(), poke_rows(), &mut budget).expect_err("short");
    assert!(budget.consumed().work <= 5);
    assert_ne!(budget.first_unpaid().map(|denied| denied.amount), Some(6));
}

/// A package that holds no probabilistic node joins at one work unit for each
/// node it meets.
#[trace("TC-484", "FR-369-AC-25")]
#[test]
fn a_package_without_a_probabilistic_node_joins_at_one_unit_a_node() {
    let mut package = package();
    nodes_mut(&mut package).retain(|node| node["node_tag"] != "probabilistic");
    settle(&mut package);
    let met = nodes(&package).len() as u64;
    let mut budget = work_only(met);
    joined(&package, poke_rows(), &mut budget).expect("nothing to join");
    assert_eq!(budget.consumed().work, met);
    let error = joined(&package, poke_rows(), &mut work_only(met - 1)).expect_err("one short");
    assert_eq!(error.code, RowError::Incomplete);
}

/// FR-369-AC-26: a support value is a member of the entry's type by the rules
/// the strict read applies to a selected parameter.
#[trace("TC-484", "FR-369-AC-26")]
#[test]
fn support_values_are_checked_against_the_entry_type_node() {
    let literal = |type_id: &Value, kind: &str, value: Value| json!({"term": "literal", "type": type_id, "value_kind": kind, "value": value});
    let with_support = |values: Vec<Value>| {
        let mut package = package();
        nodes_mut(&mut package).retain(|node| node["node_id"]["path"][2] != "reward_cost");
        let mut second = nodes(&package)
            .iter()
            .find(|node| node["node_id"] == id_value("qualified:ms"))
            .cloned()
            .expect("the unit");
        second["node_id"] = id_value("qualified:s");
        second["declaration"] = json!({"qualified_name": ["s"]});
        second["body"]["target_unit_node_id"] = id_value("qualified:ms");
        second["body"]["scale"] = json!({"numerator": "1000", "denominator": "1"});
        second["dependencies"] = json!([id_value("qualified:Time"), id_value("qualified:ms")]);
        nodes_mut(&mut package).push(second);
        edit_label(&mut package, "random_k", |node| {
            node["body"]["members"][2]["value"]["value"] = json!("p");
            node["body"]["members"][3]["value"] = aggregate(
                values
                    .into_iter()
                    .map(|value| {
                        aggregate(vec![
                            binding("value", value),
                            binding("weight", rational("1")),
                        ])
                    })
                    .collect(),
            );
            redepend(node);
        });
        package
    };
    // `Text[1, 3; nfc]` and `Rational[0, 5; 1, 3]`, over the text and rational
    // scalar nodes.
    let lit = |kind: &str, ty: &Value, value: &str| literal(ty, kind, json!(value));
    let bind = |name: &str, value: Value| binding(name, value);
    let text_scalar = id_value("builtin/text");
    let text_body = aggregate(vec![
        bind("min", lit("integer", &integer(), "1")),
        bind("max", lit("integer", &integer(), "3")),
        bind("text_profile", lit("text", &text_scalar, "nfc")),
    ]);
    let text_key = crate::support::checked_package::structural_key(
        "bounded_domain",
        "text_bounds",
        Some("builtin/text"),
        &text_body,
    );
    let rational_scalar = id_value("builtin/rational");
    let rational_body = aggregate(vec![
        bind("numerator_min", lit("integer", &integer(), "0")),
        bind("numerator_max", lit("integer", &integer(), "5")),
        bind("denominator_min", lit("integer", &integer(), "1")),
        bind("denominator_max", lit("integer", &integer(), "3")),
    ]);
    let rational_key = crate::support::checked_package::structural_key(
        "bounded_domain",
        "rational_range",
        Some("builtin/rational"),
        &rational_body,
    );
    let carried_body = aggregate(vec![
        bind("unit", reference(&id_value("qualified:ms"))),
        bind("carrier", reference(&node_id(&rational_key))),
        bind("rounding", lit("text", &text_scalar, "exact")),
    ]);
    // A carried quantity keeps its operands in its structural id.
    let carried_id = crate::support::checked_package::structural_id(
        node_id("x")["package"].as_str().expect("package"),
        vec![
            json!("scalar_type"),
            json!("carried_quantity"),
            id_value("qualified:ms"),
            node_id(&rational_key),
            json!("rounding"),
            text_scalar.clone(),
            json!("exact"),
        ],
        0,
    );
    let with_support = |values: Vec<Value>| {
        let mut package = with_support(values);
        let mut added = vec![
            plain(
                "builtin/text",
                "scalar_type",
                "text",
                &text_scalar,
                &[],
                "generated",
                aggregate(vec![]),
            ),
            plain(
                &text_key,
                "bounded_domain",
                "text_bounds",
                &text_scalar,
                &[&integer(), &text_scalar],
                "type",
                text_body.clone(),
            ),
            plain(
                &rational_key,
                "bounded_domain",
                "rational_range",
                &rational_scalar,
                &[&integer(), &rational_scalar],
                "type",
                rational_body.clone(),
            ),
        ];
        let mut carried = plain(
            "x",
            "scalar_type",
            "carried_quantity",
            &carried_id,
            &[&id_value("qualified:ms"), &node_id(&rational_key)],
            "type",
            carried_body.clone(),
        );
        carried["node_id"] = carried_id.clone();
        added.push(carried);
        for node in &mut added {
            let mut ids: Vec<Value> = node["dependencies"]
                .as_array()
                .expect("dependencies")
                .clone();
            ids.sort_by_key(segments);
            node["dependencies"] = json!(ids);
        }
        nodes_mut(&mut package).extend(added);
        settle(&mut package);
        package
    };
    let text_node = node_id(&text_key);
    let carried_node = carried_id.clone();
    let rational_node = node_id(&rational_key);
    let entry_rows = |node: &Value| rows(vec![data("k", &range()), data("p", node)], vec![]);
    let boolean = super::super::boolean();
    let ms = id_value("qualified:ms");
    // Each case: the entry's type node, the values, and the first row refused.
    let cases: Vec<(&str, Value, Vec<Value>, Option<usize>)> = vec![
        (
            "a plain integer",
            integer(),
            vec![int_literal(&integer(), "7"), int_literal(&integer(), "-9")],
            None,
        ),
        (
            "an integer given as a rational",
            integer(),
            vec![rational("1/2")],
            Some(0),
        ),
        (
            "booleans",
            boolean.clone(),
            vec![
                literal(&boolean, "boolean", json!(true)),
                literal(&boolean, "boolean", json!(false)),
            ],
            None,
        ),
        (
            "a boolean given as text",
            boolean.clone(),
            vec![literal(&boolean, "boolean", json!("true"))],
            Some(0),
        ),
        (
            "an integer in the range, then outside it",
            range(),
            vec![int_literal(&integer(), "2"), int_literal(&integer(), "0")],
            Some(1),
        ),
        (
            "an integer spelt as a JSON number",
            range(),
            vec![literal(&integer(), "integer", json!(2))],
            Some(0),
        ),
        (
            "an integer above the range",
            range(),
            vec![int_literal(&integer(), "4")],
            Some(0),
        ),
        (
            "a non-canonical integer",
            range(),
            vec![int_literal(&integer(), "+2")],
            Some(0),
        ),
        (
            "quantities of the unit",
            ms.clone(),
            vec![
                literal(&ms, "quantity", json!("1")),
                literal(&ms, "quantity", json!("5")),
            ],
            None,
        ),
        (
            "a quantity of another unit",
            ms.clone(),
            vec![literal(&id_value("qualified:s"), "quantity", json!("1"))],
            Some(0),
        ),
        (
            "a quantity given as an integer",
            ms.clone(),
            vec![int_literal(&integer(), "1")],
            Some(0),
        ),
        (
            "texts of the length range",
            text_node.clone(),
            vec![
                lit("text", &text_scalar, "a"),
                lit("text", &text_scalar, "abc"),
            ],
            None,
        ),
        (
            "a text below the length range",
            text_node.clone(),
            vec![lit("text", &text_scalar, "")],
            Some(0),
        ),
        (
            "a text above the length range",
            text_node.clone(),
            vec![
                lit("text", &text_scalar, "a"),
                lit("text", &text_scalar, "abcd"),
            ],
            Some(1),
        ),
        (
            "rationals in the bounds",
            rational_node.clone(),
            vec![
                lit("rational", &rational_scalar, "5"),
                lit("rational", &rational_scalar, "0"),
                lit("rational", &rational_scalar, "1/3"),
            ],
            None,
        ),
        (
            "a numerator above the bound",
            rational_node.clone(),
            vec![lit("rational", &rational_scalar, "6")],
            Some(0),
        ),
        (
            "a denominator above the bound",
            rational_node.clone(),
            vec![lit("rational", &rational_scalar, "1/4")],
            Some(0),
        ),
        (
            "a numerator below the bound",
            rational_node.clone(),
            vec![lit("rational", &rational_scalar, "-1")],
            Some(0),
        ),
        (
            "a rational spelt with the denominator 1",
            rational_node.clone(),
            vec![lit("rational", &rational_scalar, "3/1")],
            Some(0),
        ),
        (
            "a rational that is not in lowest terms",
            rational_node.clone(),
            vec![lit("rational", &rational_scalar, "2/4")],
            Some(0),
        ),
        (
            "carried quantities of the unit in the bounds",
            carried_node.clone(),
            vec![
                literal(&ms, "quantity", json!("1/3")),
                literal(&ms, "quantity", json!("5")),
            ],
            None,
        ),
        (
            "a carried quantity of another unit",
            carried_node.clone(),
            vec![literal(&id_value("qualified:s"), "quantity", json!("1"))],
            Some(0),
        ),
        (
            "a carried quantity outside the bounds",
            carried_node.clone(),
            vec![
                literal(&ms, "quantity", json!("1")),
                literal(&ms, "quantity", json!("6")),
            ],
            Some(1),
        ),
        (
            "a type node that has no literal kind",
            id_value("qualified:Order"),
            vec![int_literal(&integer(), "1")],
            Some(0),
        ),
    ];
    for (case, node, values, refused) in cases {
        let package = with_support(values);
        // A structural id is read from the indexed package.
        let node = if case.contains("carried") {
            nodes(&indexed(&package))
                .iter()
                .find(|node| node["semantic_form"] == "carried_quantity")
                .map(|node| node["node_id"].clone())
                .expect("the carried quantity node")
        } else {
            node
        };
        let result = join(&package, entry_rows(&node));
        match refused {
            None => result.expect(case),
            Some(row) => {
                let error = result.expect_err(case);
                assert_eq!(error.code, RowError::InvalidValue, "{case}");
                // A misspelt rational refuses at the literal's `value` member.
                let tail = if case.contains("lowest terms") || case.contains("denominator 1") {
                    "/value"
                } else {
                    ""
                };
                let at = format!(
                    "{}/members/{row}/members/0/value{tail}",
                    member_at(&package, "random_k", 3)
                );
                assert_eq!(locus_of(&error), at, "{case}");
            }
        }
    }
    // A rational `n/d` costs a number each for `n` and `d` and one for the
    // divisor (a number and a limb apiece); an integer costs one number.
    let storage = |value: &str| {
        let package = with_support(vec![lit("rational", &rational_scalar, value)]);
        let mut budget = NativeJoinBudget::unlimited();
        joined(&package, entry_rows(&rational_node), &mut budget).expect("joins");
        budget.consumed().storage
    };
    assert_eq!(storage("1/3"), storage("1") + 2 + 2);
}

/// A source enumeration `name` with the `cases`, as the package node and the
/// identity row the join reads it by.
fn enumeration(
    package: &mut Value,
    name: &str,
    cases: &[&str],
) -> filament_contract_model::SelectedSourceEnumRow {
    use filament_contract_model::{SelectedEnumDeclarationRow, SelectedEnumMemberRow};
    use filament_exact::{EnumShape, NodeKey, PathSegment, VariantId};
    let label = format!("qualified:{name}");
    nodes_mut(package).push(declared(
        plain(
            &label,
            "scalar_type",
            "enum",
            &id_value(&label),
            &[],
            "declaration",
            json!({"term": "enum", "ordered": false, "members": cases}),
        ),
        &[name],
    ));
    let key = |text: &str| {
        NodeKey::new("join-fixture", vec![PathSegment::Text(text.into())], 0).expect("key")
    };
    let variants: Vec<VariantId> = cases
        .iter()
        .map(|case| VariantId::new(key("owner"), *case).expect("variant"))
        .collect();
    let node = id(&label);
    let source = source_identity();
    filament_contract_model::SelectedSourceEnumRow {
        declaration: SelectedEnumDeclarationRow {
            structural_node: node.clone(),
            source_authority: fixture_source()["authority"]
                .as_str()
                .expect("authority")
                .into(),
            source_identity: source.as_str().into(),
            qualified_declaration: vec![name.into()],
            node_key: key(name),
            shape: EnumShape::new(false, variants.clone()),
        },
        members: cases
            .iter()
            .zip(variants)
            .map(|(case, variant)| SelectedEnumMemberRow {
                structural_node: id(&format!("value/enum_value/{name}/{case}")),
                source_authority: fixture_source()["authority"]
                    .as_str()
                    .expect("authority")
                    .into(),
                source_identity: source.as_str().into(),
                declaration_node: node.clone(),
                declaration_qualified_name: vec![name.into()],
                case: (*case).into(),
                member_key: key(case),
                variant,
            })
            .collect(),
    }
}

/// FR-369-AC-23, FR-369-AC-26: a support value of an enumeration-typed entry
/// is admitted when its literal is typed at the enumeration and names one of
/// its cases, and refused otherwise; the cases compared are charged ahead.
#[trace("TC-484", "FR-369-AC-23", "FR-369-AC-26")]
#[test]
fn an_enumeration_entry_checks_the_literal_against_the_cases_it_names() {
    let mut package = package();
    nodes_mut(&mut package).retain(|node| node["node_id"]["path"][2] != "reward_cost");
    let status = enumeration(&mut package, "Status", &["ACTIVE", "DONE"]);
    let other = enumeration(&mut package, "Shade", &["RED"]);
    let long_case = "L".repeat(2000);
    let long = enumeration(&mut package, "Long", &[long_case.as_str(), "Y"]);
    let status_node = id_value("qualified:Status");
    let other_node = id_value("qualified:Shade");
    let long_node = id_value("qualified:Long");
    let with_value = |value: Value| {
        let mut package = package.clone();
        edit_label(&mut package, "random_k", |node| {
            node["body"]["members"][2]["value"]["value"] = json!("p");
            node["body"]["members"][3]["value"] = aggregate(vec![aggregate(vec![
                binding("value", value),
                binding("weight", rational("1")),
            ])]);
            redepend(node);
        });
        settle(&mut package);
        package
    };
    let entries = |node: &Value| rows(vec![data("k", &range()), data("p", node)], vec![]);
    let run = |value: Value, enums: &[filament_contract_model::SelectedSourceEnumRow]| {
        let package = with_value(value);
        let mut budget = NativeJoinBudget::unlimited();
        let admitted = self::admitted(&package);
        let selected = SelectedSourceObjects::from_stage_rows(&admitted, entries(&status_node))
            .expect("rows join");
        let result = selected.join_probabilistic(enums, &mut budget);
        (package, result, budget.consumed())
    };
    let named = |ty: &Value, case: &str| json!({"term": "literal", "type": ty, "value_kind": "enum", "value": case});
    let both = [status.clone(), other.clone()];

    // An admitted case, the first and the last.
    let (_, first, first_cost) = run(named(&status_node, "ACTIVE"), &both);
    first.expect("the first case is admitted");
    let (_, last, last_cost) = run(named(&status_node, "DONE"), &both);
    last.expect("the last case is admitted");
    // The spelling of the value costs its bytes ("DONE" two fewer than
    // "ACTIVE"), and each case compared a visit and the bytes of its text: the
    // second case costs one more visit and the bytes of "DONE".
    assert_eq!(
        last_cost.work + ("ACTIVE".len() - "DONE".len()) as u64,
        first_cost.work + 1 + "DONE".len() as u64
    );

    // A case the enumeration lacks, a case of another enumeration, and a
    // literal typed at another declaration refuse at the literal.
    for (case, value) in [
        ("an unknown case", named(&status_node, "LATE")),
        ("a case of another enumeration", named(&status_node, "RED")),
        (
            "a value that is not text",
            json!({"term": "literal", "type": status_node, "value_kind": "enum", "value": 1}),
        ),
        (
            "a literal typed at another enumeration",
            named(&other_node, "ACTIVE"),
        ),
    ] {
        let (package, result, _) = run(value, &both);
        let error = result.expect_err(case);
        assert_eq!(error.code, RowError::InvalidValue, "{case}");
        let at = format!(
            "{}/members/0/members/0/value",
            member_at(&package, "random_k", 3)
        );
        assert_eq!(locus_of(&error), at, "{case}");
    }

    // An enumeration `enums` does not hold refuses at the literal's type
    // member.
    let (package, result, _) = run(named(&status_node, "ACTIVE"), std::slice::from_ref(&other));
    let error = result.expect_err("an absent enumeration");
    assert_eq!(error.code, RowError::InvalidValue);
    assert_eq!(
        locus_of(&error),
        format!(
            "{}/members/0/members/0/value/type",
            member_at(&package, "random_k", 3)
        )
    );

    // The scan stops at the row of the enumeration: rows after it cost nothing.
    let (_, alone, alone_cost) = run(named(&status_node, "DONE"), std::slice::from_ref(&status));
    alone.expect("the enumeration alone");
    assert_eq!(alone_cost.work, last_cost.work);
    // A row met before the enumeration's row costs one visit.
    let (_, behind, behind_cost) = run(
        named(&status_node, "DONE"),
        &[other.clone(), status.clone()],
    );
    behind.expect("the enumeration behind another");
    assert_eq!(behind_cost.work, alone_cost.work + 1);

    // A denied charge of a long case that is not the match stops the join
    // although the shorter cases after it would fit: every allowance short of
    // the visits of the case "Y" behind a 2000-byte case is `Incomplete`.
    let long_package = with_value(named(&long_node, "Y"));
    let long_admitted = self::admitted(&long_package);
    let long_selected =
        SelectedSourceObjects::from_stage_rows(&long_admitted, entries(&long_node)).expect("rows");
    let long_run = |allowed: u64| {
        let mut budget = NativeJoinBudget::new(allowed, u64::MAX);
        let result = long_selected.join_probabilistic(std::slice::from_ref(&long), &mut budget);
        (result, budget.consumed())
    };
    let (whole, long_cost) = long_run(u64::MAX);
    whole.expect("the long enumeration joins");
    for allowed in 0..long_cost.work {
        assert_eq!(
            long_run(allowed).0.expect_err("a short allowance").code,
            RowError::Incomplete,
            "{allowed}"
        );
    }

    // Every work allowance short of the visits stops the join as `Incomplete`
    // and the allowance of them all joins it.
    for allowed in 0..=last_cost.work {
        let package = with_value(named(&status_node, "DONE"));
        let admitted = self::admitted(&package);
        let selected = SelectedSourceObjects::from_stage_rows(&admitted, entries(&status_node))
            .expect("rows join");
        let mut budget = NativeJoinBudget::new(allowed, u64::MAX);
        let result = selected.join_probabilistic(&both, &mut budget);
        if allowed < last_cost.work {
            assert_eq!(
                result.expect_err("a short allowance").code,
                RowError::Incomplete,
                "{allowed}"
            );
        } else {
            result.expect("the whole allowance");
        }
    }
}

/// A row naming another type's operation by identity is no entry of the
/// operation (`<type>::<operation>` of the row's own type).
#[trace("TC-484", "FR-369-AC-26")]
#[test]
fn an_entry_of_another_type_is_no_entry_of_the_operation() {
    let mut rows = poke_rows();
    rows[2].operations[0].identity = "O::poke".into();
    let package = package();
    let path = literal_value_at(&package, "delay_poke", 1);
    refuses("an entry named for another type", &package, rows, &path);
}

/// A node whose `declaring` node is a selected model is not the join's.
#[trace("TC-484", "FR-369-AC-25")]
#[test]
fn a_selected_models_probabilistic_node_is_left_to_the_strict_read() {
    let account = node_id(&family_key("1515"));
    let row = |operation: &str| {
        aggregate(vec![
            binding("declaring", json!({"term": "reference", "target": account})),
            binding("operation", text(operation)),
            binding("weight", rational("1")),
        ])
    };
    let mut package = super::package_over(mixed_package());
    nodes_mut(&mut package).push(super::probabilistic(
        "workload_selected",
        "workload",
        aggregate(vec![
            binding("model", json!({"term": "reference", "target": account})),
            binding("name", text("Steady")),
            binding("weights", aggregate(vec![row("deposit"), row("withdraw")])),
        ]),
    ));
    let at = nodes(&package).len() - 1;
    let dependencies = nodes_mut(&mut package)[at]["dependencies"]
        .as_array_mut()
        .expect("dependencies");
    dependencies.reverse();
    settle(&mut package);
    let mut budget = NativeJoinBudget::unlimited();
    joined(&package, poke_rows(), &mut budget).expect("selected nodes are not joined");
    // The selected node is met (one unit) and not joined: the count is the
    // AC-25 one of the nodes of this package.
    assert_eq!(budget.consumed().work, ac25_work(&package));
}

/// The workload of a model with supertypes charges one visit for each
/// supertype edge read, one for each operation entry of the lineage and one
/// for each row; a repeated supertype edge is read and charged each time.
#[trace("TC-484", "FR-369-AC-26")]
#[test]
fn a_workloads_supertype_edges_are_charged() {
    let row = |declaring: &str, operation: &str| {
        aggregate(vec![
            binding("declaring", reference_to(declaring)),
            binding("operation", text(operation)),
            binding("weight", rational("1")),
        ])
    };
    let mut package = package();
    nodes_mut(&mut package).push(super::probabilistic(
        "workload_other",
        "workload",
        aggregate(vec![
            binding("model", reference_to("qualified:Other")),
            binding("name", text("W2")),
            binding(
                "weights",
                aggregate(vec![
                    row("qualified:O", "step"),
                    row("qualified:Other", "poke"),
                ]),
            ),
        ]),
    ));
    settle(&mut package);
    let o = || id("qualified:O");
    // The workload of `Other` over the lineage `Other`, `O`, on top of the
    // AC-25 count (which meets the added node): each supertype edge read (1)
    // and looked up among the lineage so far (1 for the first edge, 2 for a
    // repeat of it); the entries `Other::poke` (1 + 11) and `O::step` (1 + 7);
    // the row `O::step`: itself (1), its type in the lineage (2), `step`
    // among the declared (2 * (1 + 4)); the row `Other::poke`: itself (1), its
    // type (1), `poke` among the declared (1 + 4); `poke` among the named
    // (2 * (1 + 4)) and `step` among them (1 + 4); and the effective
    // operations: each of the 2 declared operations against the 2 declared
    // ones (4 * (1 + 4)).
    let rest = (1 + 11) + (1 + 7) + (1 + 2 + 10) + (1 + 1 + 5) + 10 + 5 + 20;
    for (supertypes, edges) in [(vec![o()], 1 + 1), (vec![o(), o()], (1 + 1) + (1 + 2))] {
        let rows = || rows(vec![data("k", &range())], supertypes.clone());
        let total = ac25_work(&package) + edges + rest;
        let mut budget = work_only(total);
        joined(&package, rows(), &mut budget).expect("the full count admits");
        assert_eq!(budget.consumed().work, total);
        // Each list of the second workload holds two elements.
        assert_eq!(budget.consumed().storage, AC25_STORAGE + 4 * (1 + 2));
        let mut budget = work_only(total - 1);
        let error = joined(&package, rows(), &mut budget).expect_err("one short");
        assert_eq!(error.code, RowError::Incomplete);
    }
}
