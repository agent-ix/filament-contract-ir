// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-770: FSpec's `check-mode-vectors.json` against the production reader.

use super::*;

fn vectors() -> Value {
    read_json(&proposal_dir_of(&fixtures_dir()).join("check-mode-vectors.json"))
        .expect("published check-mode vectors")
}

/// The package a vector case describes: its base with each `set` applied and
/// each `remove` deleted, no identity refreshed.
fn candidate(base: &Value, case: &Value) -> Value {
    let mut package = base.clone();
    for set in case["set"].as_array().expect("set list") {
        let pointer = set["pointer"].as_str().expect("set pointer");
        let (parent, key) = pointer.rsplit_once('/').expect("member pointer");
        package
            .pointer_mut(parent)
            .and_then(Value::as_object_mut)
            .expect("set parent object")
            .insert(key.to_owned(), set["value"].clone());
    }
    for remove in case["remove"].as_array().expect("remove list") {
        let pointer = remove.as_str().expect("remove pointer");
        let (parent, key) = pointer.rsplit_once('/').expect("member pointer");
        package
            .pointer_mut(parent)
            .and_then(Value::as_object_mut)
            .expect("remove parent object")
            .remove(key)
            .unwrap_or_else(|| panic!("{pointer} is absent from the base"));
    }
    package
}

fn schema() -> jsonschema::JSONSchema {
    let schema = read_json(&proposal_dir_of(&fixtures_dir()).join("schema.json"))
        .expect("FSpec checked-package schema");
    jsonschema::JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .compile(&schema)
        .expect("published schema compiles")
}

#[trace("TC-1860", "FR-770-AC-1", "FR-770-AC-2")]
#[test]
fn check_mode_vectors_read_as_published() {
    let published = vectors();
    let base = read_json(
        &proposal_dir_of(&fixtures_dir()).join(published["base"].as_str().expect("vector base")),
    )
    .expect("vector base fixture");
    let wire = read_json_wire(
        &proposal_dir_of(&fixtures_dir()).join(published["base"].as_str().expect("vector base")),
    )
    .expect("vector base wire");
    let schema = schema();
    let cases = published["cases"].as_array().expect("check-mode cases");
    assert_eq!(cases.len(), 4, "the published CM-01 to CM-04");
    let mut admitted = Vec::new();
    for case in cases {
        let id = case["id"].as_str().expect("case id");
        let package = candidate(&base, case);
        assert_eq!(
            schema.is_valid(&candidate(&wire, case)),
            case["schema"] == "valid",
            "{id}: schema verdict"
        );
        let outcome = case["outcome"].as_str().expect("outcome");
        match read_package(&package).expect("selection evidence") {
            CheckedPackageV2ReadResult::Admitted(read) => {
                assert_eq!(outcome, "admitted", "{id}");
                admitted.push(read);
            }
            CheckedPackageV2ReadResult::Refused(refusal) => {
                let given = format!("refused:{}", code_word(refusal.code));
                assert_eq!(given, outcome, "{id}");
                let path = refusal.loci[0].path.as_ref().expect("refusal pointer");
                assert_eq!(path.to_string(), case["locus"].as_str().unwrap(), "{id}");
            }
            other => panic!("{id}: {other:?}"),
        }
    }
    // FR-770-AC-1: the two admitted packages differ only in check_mode and
    // are two packages.
    assert_eq!(admitted.len(), 2);
    assert_ne!(
        super::timed_model::encode_package(&admitted[0]),
        super::timed_model::encode_package(&admitted[1])
    );
}
