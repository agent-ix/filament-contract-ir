// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-485: operation parameters in the source-object identity rows
//! (FR-367-AC-8), and the clock entries of a row's `members` (FR-367-AC-4).

use super::*;

const CLOCK: &str = "ix://filament/native/Clock";

fn range_id() -> CheckedNodeId {
    serde_json::from_value(probabilistic::range()).expect("range id")
}

fn data(name: &str) -> SelectedSourceObjectMember {
    SelectedSourceObjectMember::data(name, range_id(), false)
}

fn clock(name: &str) -> SelectedSourceObjectMember {
    SelectedSourceObjectMember::clock(name, CLOCK)
}

fn operation(
    clock_parameters: &[&str],
    parameters: Vec<SelectedSourceObjectMember>,
) -> SelectedSourceObjectOperation {
    SelectedSourceObjectOperation {
        identity: "O::arm".into(),
        query: false,
        clock_parameters: clock_parameters.iter().map(|name| (*name).into()).collect(),
        parameters,
    }
}

/// The rows of the AC-25 unit's three source types, `O` declaring
/// `arm(x: Clock, k: Int[1, 3])`.
fn rows(arm: SelectedSourceObjectOperation) -> Vec<SelectedSourceObjectRow> {
    ["Order", "O", "Other"]
        .iter()
        .map(|name| {
            let mut row = row(name, vec![], &[]);
            row.members = match *name {
                "Order" => vec![SelectedSourceObjectMember::data(
                    "total",
                    id(&family_key("a3a3")),
                    false,
                )],
                "Other" => vec![SelectedSourceObjectMember::data(
                    "n",
                    id(&family_key("a3a3")),
                    false,
                )],
                _ => vec![],
            };
            row.operations = if *name == "O" {
                vec![arm.clone()]
            } else {
                vec![]
            };
            row
        })
        .collect()
}

fn unit() -> CheckedPackageV2 {
    admitted(&probabilistic::package())
}

fn arm() -> SelectedSourceObjectOperation {
    operation(&["x"], vec![clock("x"), data("k")])
}

/// FR-367-AC-8: the row whose `O::arm` entry lists the clock entry `x` then
/// the data entry `k` joins and exposes both parameters through the typed read.
#[trace("TC-485", "FR-367-AC-8")]
#[test]
fn an_operation_entry_exposes_its_clock_and_data_parameters() {
    let package = unit();
    let joined = SelectedSourceObjects::from_stage_rows(
        &package,
        rows(arm()),
        filament_contract_model::SelectedSourceClockRows::EMPTY,
    )
    .expect("joins");
    let entry = &joined
        .read(&package, &id("qualified:O"))
        .expect("row")
        .operations[0];
    assert_eq!(entry.parameters, [clock("x"), data("k")]);
    assert_eq!(entry.clock_parameters, ["x".into()] as [Box<str>; 1]);
    assert_eq!(entry.parameters[0].type_node, None);
    assert_eq!(entry.parameters[0].member_type.as_deref(), Some(CLOCK));
    assert_eq!(entry.parameters[1].type_node, Some(range_id()));
    assert_eq!(entry.parameters[1].member_type, None);
}

/// FR-367-AC-8: each defect refuses `UnknownIdentityWitness`.
#[trace("TC-485", "FR-367-AC-8")]
#[test]
fn a_defective_operation_parameter_entry_refuses_as_unknown() {
    let package = unit();
    let both = {
        let mut entry = data("k");
        entry.member_type = Some(CLOCK.into());
        entry
    };
    let neither = {
        let mut entry = data("k");
        entry.type_node = None;
        entry
    };
    let unknown_type = SelectedSourceObjectMember::data("k", id("qualified:Nothing"), false);
    // A node of the package that is not a type: the reward function.
    let not_a_type =
        SelectedSourceObjectMember::data("k", id("qualified:Other/poke/reward/cost"), false);
    let layered = {
        let mut entry = data("k");
        entry.collections = vec![filament_contract_model::SelectedClockCollectionLayer {
            kind: "list".into(),
            minimum: "0".into(),
            maximum: "unbounded".into(),
        }];
        entry
    };
    let cases = [
        (
            "clock_parameters empty beside the clock entry",
            operation(&[], vec![clock("x"), data("k")]),
        ),
        (
            "clock_parameters naming the data entry",
            operation(&["k"], vec![clock("x"), data("k")]),
        ),
        (
            "clock_parameters naming both",
            operation(&["x", "k"], vec![clock("x"), data("k")]),
        ),
        (
            "clock_parameters out of order",
            operation(&["y", "x"], vec![clock("x"), clock("y")]),
        ),
        (
            "an entry with both type_node and member_type",
            operation(&["x"], vec![clock("x"), both]),
        ),
        (
            "an entry with neither",
            operation(&["x"], vec![clock("x"), neither]),
        ),
        (
            "two entries named x",
            operation(&["x", "x"], vec![clock("x"), clock("x")]),
        ),
        (
            "a data entry naming no type node",
            operation(&["x"], vec![clock("x"), unknown_type]),
        ),
        (
            "a data entry naming a node that is not a type",
            operation(&["x"], vec![clock("x"), not_a_type]),
        ),
        (
            "a data entry with collection layers",
            operation(&["x"], vec![clock("x"), layered]),
        ),
    ];
    for (case, entry) in cases {
        assert_eq!(
            code_of(&package, rows(entry)),
            RowError::UnknownIdentityWitness,
            "{case}"
        );
    }
    // Two clock parameters in order, and a clock entry of another rendering
    // that clock_parameters does not list, are consistent.
    let mut optional = clock("z");
    optional.member_type = Some("Option<ix://filament/native/Clock>".into());
    SelectedSourceObjects::from_stage_rows(
        &package,
        rows(operation(
            &["x", "y"],
            vec![clock("x"), optional, clock("y")],
        )),
        filament_contract_model::SelectedSourceClockRows::EMPTY,
    )
    .expect("an Option<Clock> entry is not a clock_parameters name");
}

/// FR-367-AC-4: a clock entry among the `members` is not compared with the
/// bindings, shares no name with one and renders a type ending in `Clock`.
#[trace("TC-475", "FR-367-AC-4")]
#[test]
fn clock_entries_of_the_members_are_checked_apart_from_the_bindings() {
    let package = unit();
    let with = |entry: SelectedSourceObjectMember| {
        let mut set = rows(arm());
        let at = set
            .iter()
            .position(|row| &*row.source_identity == "Other")
            .expect("Other");
        set[at].members.push(entry);
        set
    };
    SelectedSourceObjects::from_stage_rows(
        &package,
        with(clock("d")),
        filament_contract_model::SelectedSourceClockRows::EMPTY,
    )
    .expect("a clock member");
    let mut wrapped = clock("d");
    wrapped.member_type = Some("Map<Text, ix://filament/native/Clock>".into());
    SelectedSourceObjects::from_stage_rows(
        &package,
        with(wrapped),
        filament_contract_model::SelectedSourceClockRows::EMPTY,
    )
    .expect("a wrapped clock");
    let mut integer = clock("d");
    integer.member_type = Some("ix://filament/native/Integer".into());
    for (case, entry) in [
        ("a clock entry named as a binding", clock("n")),
        ("a clock entry of another type", integer),
    ] {
        assert_eq!(
            code_of(&package, with(entry)),
            RowError::UnknownIdentityWitness,
            "{case}"
        );
    }
}
