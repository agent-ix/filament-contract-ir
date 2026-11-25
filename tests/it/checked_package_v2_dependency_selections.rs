// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Structural V2 dependency selections: exact names, closed entries, ordering,
//! and complete-value conflicts in the admitted dependency closure.

use crate::support::checked_package;
use crate::support::checked_package::{
    admitted_dependency, canonical, evidence_for, pointer, read_with_dependencies,
};
#[path = "support/dependency_selection_inputs.rs"]
pub(crate) mod dependency_selection_inputs;
use dependency_selection_inputs::dependency_selection_package as empty_package;
use filament_contract_model::{
    CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use ix_trace_rs::trace;
use serde_json::{json, Value};

fn entry(identity: &str) -> Value {
    json!({"identity": identity})
}

fn with_selections(mut package: Value, entries: Vec<Value>) -> Value {
    package["lock"]["dependency_selections"] = Value::Array(entries);
    package
}

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    crate::read_and_account(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

fn refused(package: &Value) -> CheckedPackageRefusal {
    match read(package) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected refusal, got {other:?}"),
    }
}

fn expect_refusal(package: &Value, code: Code, path: &str, cause: Option<Cause>) {
    let refusal = refused(package);
    assert_eq!(refusal.code, code, "{refusal:?}");
    assert_eq!(refusal.cause, cause, "{refusal:?}");
    assert!(
        refusal
            .loci
            .iter()
            .any(|locus| locus.path == Some(pointer(path))),
        "{refusal:?}"
    );
}

/// FR-038-AC-31/218/460: two selected packages retain complete values in lock
/// order, including source material that differs beyond their names.
#[trace(
    "TC-048",
    "FR-038-AC-31",
    "FR-038-AC-218",
    "FR-038-AC-366",
    "FR-038-AC-460"
)]
#[test]
fn ascending_dependency_names_admit_and_are_retained() {
    let geometry_wire = empty_package("test/geometry");
    let mut units_wire = empty_package("test/units");
    units_wire["source_map"][0]["regions"][0]["end"] = json!(100);
    let (_, geometry) = admitted_dependency(&geometry_wire);
    let (_, units) = admitted_dependency(&units_wire);
    assert_ne!(
        geometry.source_map()[0].regions[0].end,
        units.source_map()[0].regions[0].end
    );
    let entries = vec![entry("test/geometry"), entry("test/units")];
    let importer = with_selections(empty_package("test/importer"), entries.clone());
    let CheckedPackageV2ReadResult::Admitted(admitted) = read_with_dependencies(
        &importer,
        &[("test/geometry", &geometry), ("test/units", &units)],
    ) else {
        panic!("two selected packages with matching names must admit");
    };
    assert_eq!(
        serde_json::to_value(&admitted.lock().dependency_selections).expect("selections"),
        json!(entries)
    );
    assert_eq!(admitted.name(), "test/importer");
    assert_eq!(
        admitted.direct_dependencies().collect::<Vec<_>>(),
        vec![&geometry, &units]
    );
    assert_eq!(admitted.direct_dependencies().len(), 2);
    assert_eq!(geometry.direct_dependencies().len(), 0);
    assert!(importer.get("package_id").is_none());
    assert!(importer.get("identity_preimage").is_none());
}

/// FR-038-AC-460: a parent exposes its child while the child's own accessor
/// exposes the grandchild; neither iteration fabricates a transitive edge.
#[trace("TC-048", "FR-038-AC-460")]
#[test]
fn direct_dependency_accessor_stops_at_one_edge() {
    let (_, grandchild) = admitted_dependency(&empty_package("test/grandchild"));
    let child_wire = with_selections(empty_package("test/child"), vec![entry(grandchild.name())]);
    let CheckedPackageV2ReadResult::Admitted(child) =
        read_with_dependencies(&child_wire, &[(grandchild.name(), &grandchild)])
    else {
        panic!("child with a grandchild must admit");
    };
    let root_wire = with_selections(empty_package("test/root"), vec![entry(child.name())]);
    let CheckedPackageV2ReadResult::Admitted(root) =
        read_with_dependencies(&root_wire, &[(child.name(), &child)])
    else {
        panic!("root with a child must admit");
    };
    assert_eq!(
        root.direct_dependencies().collect::<Vec<_>>(),
        vec![child.as_ref()]
    );
    assert_eq!(
        child.direct_dependencies().collect::<Vec<_>>(),
        vec![&grandchild]
    );
    assert_eq!(grandchild.direct_dependencies().len(), 0);
}

/// FR-038-AC-32/63: `{identity}` is the complete selection entry. An absent
/// or empty identity is malformed, and every removed legacy member is refused.
#[trace("TC-048", "FR-038-AC-32", "FR-038-AC-63", "FR-038-AC-366")]
#[test]
fn selection_shape_is_closed() {
    let base = empty_package("test/importer");
    for (shape, path) in [
        (json!({}), "/lock/dependency_selections/0"),
        (
            json!({"identity": ""}),
            "/lock/dependency_selections/0/identity",
        ),
    ] {
        expect_refusal(
            &with_selections(base.clone(), vec![shape]),
            Code::MalformedWire,
            path,
            None,
        );
    }
    for member in ["package_id", "version", "digest", "role", "definition"] {
        let mut selection = entry("test/geometry");
        selection[member] = json!("removed");
        expect_refusal(
            &with_selections(base.clone(), vec![selection]),
            Code::UnknownMember,
            &format!("/lock/dependency_selections/0/{member}"),
            None,
        );
    }
}

/// FR-038-AC-33: duplicate detection precedes monotonicity for an identity
/// repeated at any position; distinct entries use UTF-8 byte order.
#[trace("TC-048", "FR-038-AC-33", "FR-038-AC-366")]
#[test]
fn duplicate_and_misordered_names_refuse_at_the_repeating_entry() {
    let base = empty_package("test/importer");
    for (entries, at, cause) in [
        (
            vec![entry("test/a"), entry("test/a")],
            1,
            Cause::ConflictingDefinition,
        ),
        (
            vec![entry("test/a"), entry("test/b"), entry("test/a")],
            2,
            Cause::ConflictingDefinition,
        ),
        (
            vec![entry("test/b"), entry("test/a")],
            1,
            Cause::InvalidValue,
        ),
    ] {
        expect_refusal(
            &with_selections(base.clone(), entries),
            Code::InvalidPackage,
            &format!("/lock/dependency_selections/{at}"),
            Some(cause),
        );
    }
    let first_wire = empty_package("test/\u{ff61}");
    let second_wire = empty_package("test/\u{1f600}");
    let (_, first) = admitted_dependency(&first_wire);
    let (_, second) = admitted_dependency(&second_wire);
    let ordered = with_selections(base, vec![entry("test/\u{ff61}"), entry("test/\u{1f600}")]);
    assert!(matches!(
        read_with_dependencies(
            &ordered,
            &[("test/\u{ff61}", &first), ("test/\u{1f600}", &second)]
        ),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// FR-038-AC-64/217/218: selection binding uses package name; a selected
/// name without an identically named admitted package is a typed missing import.
#[trace(
    "TC-048",
    "FR-038-AC-64",
    "FR-038-AC-217",
    "FR-038-AC-218",
    "FR-038-AC-366"
)]
#[test]
fn selection_binds_by_exact_package_name() {
    let wire = empty_package("test/geometry");
    let (_, geometry) = admitted_dependency(&wire);
    let importer = with_selections(empty_package("test/importer"), vec![entry("test/geometry")]);
    assert!(matches!(
        read_with_dependencies(&importer, &[("test/geometry", &geometry)]),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    for supplied in [
        Vec::<(&str, &CheckedPackageV2)>::new(),
        vec![("test/other", &geometry)],
    ] {
        let CheckedPackageV2ReadResult::Refused(refusal) =
            read_with_dependencies(&importer, &supplied)
        else {
            panic!("unsupplied selection must refuse");
        };
        assert_eq!(refusal.code, Code::MissingImport);
        assert_eq!(refusal.cause, Some(Cause::MissingSelection));
        assert_eq!(
            refusal.loci[0].path,
            Some(pointer("/lock/dependency_selections/0"))
        );
    }
}

/// FR-038-AC-293: the same name reached through a diamond must bind one
/// complete package value; the two importer-rooted loci remain ordered.
#[trace("TC-048", "FR-038-AC-293", "FR-038-AC-366")]
#[test]
fn diamond_with_unequal_complete_leaf_packages_refuses() {
    let first_wire = empty_package("test/families");
    let mut second_wire = first_wire.clone();
    second_wire["source_map"][0]["regions"][0]["end"] = json!(100);
    let (_, first_leaf) = admitted_dependency(&first_wire);
    let (_, second_leaf) = admitted_dependency(&second_wire);
    assert_eq!(first_leaf.name(), second_leaf.name());
    assert_ne!(canonical(&first_wire), canonical(&second_wire));

    let left_wire = with_selections(empty_package("test/left"), vec![entry(first_leaf.name())]);
    let right_wire = with_selections(empty_package("test/right"), vec![entry(second_leaf.name())]);
    let CheckedPackageV2ReadResult::Admitted(left) =
        read_with_dependencies(&left_wire, &[(first_leaf.name(), &first_leaf)])
    else {
        panic!("left dependency admits");
    };
    let CheckedPackageV2ReadResult::Admitted(right) =
        read_with_dependencies(&right_wire, &[(second_leaf.name(), &second_leaf)])
    else {
        panic!("right dependency admits");
    };
    let root = with_selections(
        empty_package("test/root"),
        vec![entry(left.name()), entry(right.name())],
    );
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with_dependencies(&root, &[(left.name(), &left), (right.name(), &right)])
    else {
        panic!("unequal complete values for one transitive name must refuse");
    };
    assert_eq!(refusal.code, Code::InvalidPackage);
    assert_eq!(refusal.cause, Some(Cause::ConflictingDefinition));
    assert_eq!(
        refusal.loci[0].path,
        Some(pointer(
            "/lock/dependency_selections/0/lock/dependency_selections/0"
        ))
    );
    assert_eq!(
        refusal.loci[1].path,
        Some(pointer(
            "/lock/dependency_selections/1/lock/dependency_selections/0"
        ))
    );
}

/// FR-038-AC-293: two values of one package name that differ only in a
/// dimension equivalence's `root_scale`, and so share the equivalence's node
/// id, refuse in the lock stage when a diamond reaches both.
#[trace("TC-048", "FR-038-AC-293")]
#[test]
fn diamond_with_leaf_packages_differing_only_in_root_scale_refuses() {
    let first_wire = crate::checked_package_v2_dimension_equivalence::package();
    let mut second_wire = first_wire.clone();
    let scale = "/semantic_graph/nodes/3/body/root_scale/numerator";
    *second_wire.pointer_mut(scale).expect("equivalence ratio") = json!("2");
    assert_eq!(first_wire.pointer(scale), Some(&json!("1")));
    let mut restored = second_wire.clone();
    *restored.pointer_mut(scale).expect("equivalence ratio") = json!("1");
    assert_eq!(restored, first_wire, "the ratio is the only difference");
    let (_, first_leaf) = admitted_dependency(&first_wire);
    let (_, second_leaf) = admitted_dependency(&second_wire);
    assert_eq!(first_leaf.name(), second_leaf.name());
    assert_eq!(
        first_leaf.document_held_id(3),
        second_leaf.document_held_id(3)
    );

    let branch = |name: &str, leaf: &CheckedPackageV2| {
        let wire = with_selections(empty_package(name), vec![entry(leaf.name())]);
        let CheckedPackageV2ReadResult::Admitted(admitted) =
            read_with_dependencies(&wire, &[(leaf.name(), leaf)])
        else {
            panic!("{name} admits over its own leaf");
        };
        admitted
    };
    let left = branch("test/left", &first_leaf);
    let agreeing = branch("test/right", &first_leaf);
    let conflicting = branch("test/right", &second_leaf);
    let root = with_selections(
        empty_package("test/root"),
        vec![entry("test/left"), entry("test/right")],
    );
    assert!(matches!(
        read_with_dependencies(&root, &[("test/left", &left), ("test/right", &agreeing)]),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with_dependencies(&root, &[("test/left", &left), ("test/right", &conflicting)])
    else {
        panic!("leaf values differing only in root_scale must refuse");
    };
    assert_eq!(refusal.code, Code::InvalidPackage);
    assert_eq!(refusal.cause, Some(Cause::ConflictingDefinition));
    assert_eq!(
        refusal.loci[0].path,
        Some(pointer(
            "/lock/dependency_selections/0/lock/dependency_selections/0"
        ))
    );
    assert_eq!(
        refusal.loci[1].path,
        Some(pointer(
            "/lock/dependency_selections/1/lock/dependency_selections/0"
        ))
    );
}

/// AC-366 equal-value diamond and direct-lock-only projection slice. Both
/// branches supply the same admitted leaf value under one selected name.
#[trace("TC-048", "FR-038-AC-366")]
#[test]
fn equal_complete_leaf_diamond_admits_without_a_transitive_lock_row() {
    let leaf_wire = empty_package("test/leaf");
    let (_, leaf) = admitted_dependency(&leaf_wire);
    let left_wire = with_selections(empty_package("test/left"), vec![entry(leaf.name())]);
    let right_wire = with_selections(empty_package("test/right"), vec![entry(leaf.name())]);
    let CheckedPackageV2ReadResult::Admitted(left) =
        read_with_dependencies(&left_wire, &[(leaf.name(), &leaf)])
    else {
        panic!("left branch must admit");
    };
    let CheckedPackageV2ReadResult::Admitted(right) =
        read_with_dependencies(&right_wire, &[(leaf.name(), &leaf)])
    else {
        panic!("right branch must admit");
    };
    let direct = vec![entry(left.name()), entry(right.name())];
    let root = with_selections(empty_package("test/root"), direct.clone());
    let CheckedPackageV2ReadResult::Admitted(admitted) =
        read_with_dependencies(&root, &[(left.name(), &left), (right.name(), &right)])
    else {
        panic!("equal complete leaf values in a diamond must admit");
    };
    assert_eq!(
        serde_json::to_value(&admitted.lock().dependency_selections).expect("direct selections"),
        json!(direct),
        "the root lock retains only its two direct selections"
    );
}

/// An explicit direct claim for a name already reached transitively is valid
/// when it supplies the identical complete admitted package value.
#[trace("TC-048", "FR-038-AC-31", "FR-038-AC-217", "FR-038-AC-366")]
#[test]
fn overlisted_transitive_selection_admits_when_values_agree() {
    let leaf_wire = empty_package("test/leaf");
    let (_, leaf) = admitted_dependency(&leaf_wire);
    let direct_wire = with_selections(empty_package("test/direct"), vec![entry(leaf.name())]);
    let CheckedPackageV2ReadResult::Admitted(direct) =
        read_with_dependencies(&direct_wire, &[(leaf.name(), &leaf)])
    else {
        panic!("direct dependency admits");
    };
    let root = with_selections(
        empty_package("test/root"),
        vec![entry(direct.name()), entry(leaf.name())],
    );
    assert!(matches!(
        read_with_dependencies(&root, &[(direct.name(), &direct), (leaf.name(), &leaf)]),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// Admits, on its own read, a package named `name` that selects each of
/// `supplied`, which must be in ascending name order.
fn admitted_selecting(name: &str, supplied: &[&CheckedPackageV2]) -> CheckedPackageV2 {
    match read_selecting(name, supplied) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("{name} admits on its own read: {other:?}"),
    }
}

fn read_selecting(name: &str, supplied: &[&CheckedPackageV2]) -> CheckedPackageV2ReadResult {
    let wire = with_selections(
        empty_package(name),
        supplied
            .iter()
            .map(|package| entry(package.name()))
            .collect(),
    );
    let evidence: Vec<_> = supplied
        .iter()
        .map(|package| (package.name(), *package))
        .collect();
    read_with_dependencies(&wire, &evidence)
}

/// Reads `name` over `supplied` and requires the one-location import-cycle
/// refusal at `path`.
fn expect_cycle(name: &str, supplied: &[&CheckedPackageV2], path: &str) {
    let CheckedPackageV2ReadResult::Refused(refusal) = read_selecting(name, supplied) else {
        panic!("a closure holding {name} through {path} must refuse");
    };
    assert_eq!(refusal.code, Code::InvalidPackage, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::DefinitionCycle), "{refusal:?}");
    let loci: Vec<_> = refusal.loci.iter().map(|locus| &locus.path).collect();
    assert_eq!(loci, [&Some(pointer(path))], "{refusal:?}");
}

/// A package named `name` with no selections, distinct from the one every
/// other helper builds when `end` differs.
fn leaf_ending_at(name: &str, end: u64) -> CheckedPackageV2 {
    let mut wire = empty_package(name);
    wire["source_map"][0]["regions"][0]["end"] = json!(end);
    admitted_dependency(&wire).1
}

/// A package's closure cannot hold its own name, selected directly or through
/// one or two libraries. The refused package is a child to any importer, so it
/// refuses on its own read.
#[trace("TC-048", "FR-038-AC-637", "FR-038-AC-293")]
#[test]
fn a_closure_holding_the_package_name_refuses_at_the_entry_that_names_it() {
    let own = leaf_ending_at("test/own", 100);
    expect_cycle("test/own", &[&own], "/lock/dependency_selections/0");

    let library = admitted_selecting("test/library", &[&own]);
    expect_cycle(
        "test/own",
        &[&library],
        "/lock/dependency_selections/0/lock/dependency_selections/0",
    );

    let outer = admitted_selecting("test/outer", &[&library]);
    expect_cycle(
        "test/own",
        &[&outer],
        "/lock/dependency_selections/0/lock/dependency_selections/0/lock/dependency_selections/0",
    );
}

/// Two libraries that select unequal packages under the reading package's name
/// are an import cycle at the first, not a diamond conflict.
#[trace("TC-048", "FR-038-AC-637")]
#[test]
fn unequal_packages_under_the_own_name_are_a_cycle_not_a_conflict() {
    let first = leaf_ending_at("test/own", 100);
    let second = leaf_ending_at("test/own", 101);
    let left = admitted_selecting("test/left", &[&first]);
    let right = admitted_selecting("test/right", &[&second]);
    expect_cycle(
        "test/own",
        &[&left, &right],
        "/lock/dependency_selections/0/lock/dependency_selections/0",
    );
}

/// Supply is checked first: an unsupplied direct entry naming the reading
/// package is a missing selection.
#[trace("TC-048", "FR-038-AC-637")]
#[test]
fn an_unsupplied_entry_naming_the_package_is_a_missing_selection() {
    let root = with_selections(empty_package("test/own"), vec![entry("test/own")]);
    let CheckedPackageV2ReadResult::Refused(refusal) = read_with_dependencies(&root, &[]) else {
        panic!("an unsupplied selection must refuse");
    };
    assert_eq!(refusal.code, Code::MissingImport, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::MissingSelection), "{refusal:?}");
    let loci: Vec<_> = refusal.loci.iter().map(|locus| &locus.path).collect();
    assert_eq!(loci, [&Some(pointer("/lock/dependency_selections/0"))]);
}

/// Two copies of one library with equal wire values, admitted over unequal
/// packages: only the second reaches the reading package. The walk follows the
/// second copy and refuses at the first unequal pair it meets.
#[trace("TC-048", "FR-038-AC-293")]
#[test]
fn a_repeated_library_that_reaches_the_importer_is_walked() {
    let own = leaf_ending_at("test/own", 100);
    let plain = leaf_ending_at("test/n", 100);
    let reaching = admitted_selecting("test/n", &[&own]);
    let first_copy = admitted_selecting("test/m", &[&plain]);
    let second_copy = admitted_selecting("test/m", &[&reaching]);
    let left = admitted_selecting("test/a", &[&first_copy]);
    let right = admitted_selecting("test/b", &[&second_copy]);
    let CheckedPackageV2ReadResult::Refused(refusal) = read_selecting("test/own", &[&left, &right])
    else {
        panic!("a closure holding the package's own name must refuse");
    };
    assert_eq!(refusal.code, Code::InvalidPackage, "{refusal:?}");
    assert_eq!(
        refusal.cause,
        Some(Cause::ConflictingDefinition),
        "{refusal:?}"
    );
    let loci: Vec<_> = refusal.loci.iter().map(|locus| &locus.path).collect();
    assert_eq!(
        loci,
        [
            &Some(pointer(
                "/lock/dependency_selections/0/lock/dependency_selections/0/lock/dependency_selections/0"
            )),
            &Some(pointer(
                "/lock/dependency_selections/1/lock/dependency_selections/0/lock/dependency_selections/0"
            )),
        ]
    );
}
