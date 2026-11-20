// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-322 "Named type arguments": FSpec's `type-argument-alias-vectors.json`
//! (TA-01 to TA-09) against the production reader (FR-322-AC-492), and that
//! the label changes no admission decision (FR-322-AC-493).

use super::*;

fn vectors() -> Value {
    read_json(&proposal_dir_of(&fixtures_dir()).join("type-argument-alias-vectors.json"))
        .expect("published type-argument alias vectors")
}

/// The package a vector case describes: its base with each `set` applied.
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
    assert!(case["remove"].as_array().expect("remove list").is_empty());
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

fn base_of(published: &Value) -> (Value, Value) {
    let path = proposal_dir_of(&fixtures_dir()).join(published["base"].as_str().expect("base"));
    (
        read_json(&path).expect("vector base fixture"),
        read_json_wire(&path).expect("vector base wire"),
    )
}

#[trace("TC-048", "FR-322-AC-492", "FR-038-AC-583")]
#[test]
fn every_published_type_argument_alias_vector_reads_as_published() {
    let published = vectors();
    let (base, wire) = base_of(&published);
    let schema = schema();
    let cases = published["cases"].as_array().expect("cases");
    assert_eq!(cases.len(), 9, "the published TA-01 to TA-09");
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
            CheckedPackageV2ReadResult::Admitted(_) => assert_eq!(outcome, "admitted", "{id}"),
            CheckedPackageV2ReadResult::Refused(refusal) => {
                let mut given = format!("refused:{}", code_word(refusal.code));
                if let Some(cause) = refusal.cause {
                    if outcome.contains('/') {
                        given = format!("{given}/{}", cause_word(cause));
                    }
                }
                assert_eq!(given, outcome, "{id}");
                let path = refusal.loci[0].path.as_ref().expect("refusal pointer");
                assert_eq!(path.to_string(), case["locus"].as_str().unwrap(), "{id}");
                if id == "TA-09" {
                    let named: Vec<_> = refusal
                        .loci
                        .iter()
                        .map(|locus| locus.path.as_ref().unwrap().to_string())
                        .collect();
                    assert_eq!(
                        named,
                        [
                            "/semantic_graph/nodes/7/body/operation/member/alias",
                            "/semantic_graph/nodes/5/body/operation/member/alias"
                        ],
                        "{id} names both applications"
                    );
                }
            }
            other => panic!("{id}: {other:?}"),
        }
    }
}

/// FR-322-AC-493: the label is read by no check. The base package, TA-02's
/// package and TA-02 with its two keys exchanged are all admitted.
#[trace("TC-048", "FR-322-AC-493", "FR-038-AC-584")]
#[test]
fn an_alias_label_changes_no_admission_decision() {
    let published = vectors();
    let (base, _) = base_of(&published);
    let ta02 = published["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "TA-02")
        .unwrap();
    let labelled = candidate(&base, ta02);
    let mut swapped = labelled.clone();
    for (node, key) in [(5, "Money"), (7, "Row")] {
        swapped["semantic_graph"]["nodes"][node]["body"]["operation"]["member"]["alias"]["node"] =
            json!(format!("ix://example/{key}"));
    }
    for package in [&base, &labelled, &swapped] {
        assert!(matches!(
            read_package(package).expect("evidence"),
            CheckedPackageV2ReadResult::Admitted(_)
        ));
    }
}

/// FR-322-AC-492, empty `package` and `node` texts are shape defects at the
/// member they name, before any graph stage (rule 1).
#[trace("TC-048", "FR-322-AC-492", "FR-038-AC-583")]
#[test]
fn an_empty_alias_text_is_malformed_wire_at_that_member() {
    let published = vectors();
    let (base, _) = base_of(&published);
    for member in ["package", "node"] {
        let mut package = base.clone();
        package["semantic_graph"]["nodes"][7]["body"]["operation"]["member"]["alias"] =
            json!({"package": "example", "node": "ix://example/Money"});
        package["semantic_graph"]["nodes"][7]["body"]["operation"]["member"]["alias"][member] =
            json!("");
        let CheckedPackageV2ReadResult::Refused(refusal) =
            read_package(&package).expect("evidence")
        else {
            panic!("an empty {member} refuses");
        };
        assert_eq!(refusal.code, Code::MalformedWire, "{member}");
        assert_eq!(
            refusal.loci[0].path.as_ref().unwrap().to_string(),
            format!("/semantic_graph/nodes/7/body/operation/member/alias/{member}"),
            "{member}"
        );
    }
}

/// FR-322-AC-492, rule 4: a `model_root` key for an alias whose resolution is
/// another type than the member's `declaration` is `ill_typed`/`type-mismatch`
/// at the member's `alias`.
#[trace("TC-048", "FR-322-AC-492", "FR-038-AC-583")]
#[test]
fn a_model_alias_resolving_to_another_type_than_the_declaration_is_a_type_mismatch() {
    let published = vectors();
    let (base, _) = base_of(&published);
    let source = proposal_dir_of(&fixtures_dir()).join(DOMAIN_DOCUMENT);
    let mut document = read_json(&source).expect("domain document");
    document["types"].as_array_mut().unwrap().push(json!({
        "identity": "ix://acme/orders/Small",
        "kind": "alias",
        "target": "ix://filament/native/Integer"
    }));
    let path = std::env::temp_dir().join(format!("qsl-2708-alias-{}.json", std::process::id()));
    std::fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
    let mut package = base;
    package["semantic_graph"]["nodes"][7]["body"]["operation"]["member"]["alias"] =
        json!({"package": "acme/orders", "node": "ix://acme/orders/Small"});
    let result = read_package_with_document(&package, "alias mismatch", &path).expect("evidence");
    std::fs::remove_file(&path).ok();
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("a mismatching alias refuses: {result:?}");
    };
    assert_eq!(refusal.code, Code::IllTyped, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::TypeMismatch), "{refusal:?}");
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().to_string(),
        "/semantic_graph/nodes/7/body/operation/member/alias"
    );
    // FR-322 rule 4: `expected` is the resolved type's node, `actual` the
    // `declaration`.
    let detail = refusal
        .type_argument_detail
        .as_deref()
        .expect("rule 4 detail");
    assert_eq!(
        serde_json::to_value(detail.expected.as_ref().unwrap()).unwrap(),
        json!({"ordinal": 0, "package": "example", "path": ["scalar_type", "integer"]})
    );
    assert_eq!(
        serde_json::to_value(detail.actual.as_ref().unwrap()).unwrap(),
        published_declaration(7)
    );
}

/// The `declaration` the base package's node `position` names.
fn published_declaration(position: usize) -> Value {
    let (base, _) = base_of(&vectors());
    base["semantic_graph"]["nodes"][position]["body"]["operation"]["member"]["declaration"].clone()
}

/// Reads the base package with node 7's `alias` over a model document that
/// gains the alias `ix://acme/orders/<name>` onto `target`; `edit` may change
/// the package further.
fn read_with_model_alias(
    name: &str,
    target: &str,
    edit: impl FnOnce(&mut Value),
) -> CheckedPackageV2ReadResult {
    let published = vectors();
    let (base, _) = base_of(&published);
    let source = proposal_dir_of(&fixtures_dir()).join(DOMAIN_DOCUMENT);
    let mut document = read_json(&source).expect("domain document");
    document["types"].as_array_mut().unwrap().push(json!({
        "identity": format!("ix://acme/orders/{name}"),
        "kind": "alias",
        "target": target
    }));
    let path =
        std::env::temp_dir().join(format!("qsl-2708-alias-{}-{name}.json", std::process::id()));
    std::fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
    let mut package = base;
    package["semantic_graph"]["nodes"][7]["body"]["operation"]["member"]["alias"] =
        json!({"package": "acme/orders", "node": format!("ix://acme/orders/{name}")});
    edit(&mut package);
    let result = read_package_with_document(&package, "model alias", &path).expect("evidence");
    std::fs::remove_file(&path).ok();
    result
}

/// FR-322-AC-492 rule 4: an alias whose resolution this reader cannot map to
/// a node is not that node, so it refuses like any other mismatch, with no
/// `expected` node.
#[trace("TC-048", "FR-322-AC-492", "FR-038-AC-583")]
#[test]
fn a_model_alias_the_reader_cannot_map_to_a_node_is_a_type_mismatch() {
    let result = read_with_model_alias("Unmapped", "ix://acme/orders/Flow", |_| {});
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("an unmappable alias refuses: {result:?}");
    };
    assert_eq!(refusal.code, Code::IllTyped, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::TypeMismatch), "{refusal:?}");
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().to_string(),
        "/semantic_graph/nodes/7/body/operation/member/alias"
    );
    let detail = refusal
        .type_argument_detail
        .as_deref()
        .expect("rule 4 detail");
    assert!(detail.expected.is_none(), "{detail:?}");
    assert_eq!(
        serde_json::to_value(detail.actual.as_ref().unwrap()).unwrap(),
        published_declaration(7)
    );
}

/// FR-322-AC-492 rule 4: an alias that resolves to the node `declaration`
/// names is never refused at the alias. Node 7 is rewritten to name the
/// `Integer` node (its dependency, result type and semantic type with it), and
/// the alias to `Integer` resolves to that node; whatever the sum then refuses
/// for its operand types, the alias is not at fault.
#[trace("TC-048", "FR-322-AC-492", "FR-038-AC-583")]
#[test]
fn a_model_alias_resolving_to_the_declaration_is_not_refused_at_the_alias() {
    let integer = json!({"ordinal": 0, "package": "example", "path": ["scalar_type", "integer"]});
    let result = read_with_model_alias("Whole", "ix://filament/native/Integer", |package| {
        let node = &mut package["semantic_graph"]["nodes"][7];
        node["body"]["operation"]["member"]["declaration"] = integer.clone();
        node["body"]["result_type"] = integer.clone();
        node["semantic_type"] = integer.clone();
        node["dependencies"][0] = integer.clone();
    });
    if let CheckedPackageV2ReadResult::Refused(refusal) = &result {
        let at = refusal.loci[0].path.as_ref().unwrap().to_string();
        assert!(
            !at.ends_with("/member/alias"),
            "the alias resolves to the declaration: {refusal:?}"
        );
        assert!(refusal.type_argument_detail.is_none(), "{refusal:?}");
    }
}

fn refusal_of(package: &Value) -> (String, Option<String>) {
    let reading = reading_of(package).expect("reading");
    let locus = match read_package(package).expect("evidence") {
        CheckedPackageV2ReadResult::Refused(refusal) => {
            Some(refusal.loci[0].path.as_ref().unwrap().to_string())
        }
        _ => None,
    };
    (reading, locus)
}

/// FR-322-AC-493: two otherwise defective variants of the base package are
/// refused the same way whatever `alias` they carry, and the typed `declaration`
/// of every application is the same with and without it.
#[trace("TC-048", "FR-322-AC-493", "FR-038-AC-584")]
#[test]
fn an_alias_label_changes_no_refusal_and_no_declaration() {
    let published = vectors();
    let (base, _) = base_of(&published);
    let integer = json!({"ordinal": 0, "package": "example", "path": ["scalar_type", "integer"]});
    let variants: [(&str, Box<dyn Fn(&mut Value)>); 2] = [
        (
            "result_type no operator admits",
            Box::new(|package: &mut Value| {
                package["semantic_graph"]["nodes"][7]["body"]["result_type"] = integer.clone();
            }),
        ),
        (
            "declaration another node than the constraints require",
            Box::new(|package: &mut Value| {
                package["semantic_graph"]["nodes"][7]["body"]["operation"]["member"]
                    ["declaration"] = published_declaration(5);
            }),
        ),
    ];
    for (name, defect) in &variants {
        let mut plain = base.clone();
        defect(&mut plain);
        let held = refusal_of(&plain);
        assert!(held.0.starts_with("refused:"), "{name} refuses: {held:?}");
        for (package, node) in [
            ("example", "ix://example/Money"),
            ("example", "ix://example/Other"),
        ] {
            let mut labelled = plain.clone();
            labelled["semantic_graph"]["nodes"][7]["body"]["operation"]["member"]["alias"] =
                json!({"package": package, "node": node});
            assert_eq!(refusal_of(&labelled), held, "{name} with {node}");
        }
    }
    // The typed `declaration` read is the same with and without the label.
    let ta01 = published["cases"][0].clone();
    for position in [5, 7] {
        let read = |package: &Value| {
            let CheckedPackageV2ReadResult::Admitted(admitted) =
                read_package(package).expect("evidence")
            else {
                panic!("admitted");
            };
            let node = &admitted.resolved_graph().nodes[position];
            let CheckedTerm::Application { operation, .. } = &node.body else {
                panic!("application");
            };
            let Some(CheckedOperationMember::TypeArgument { declaration, .. }) =
                operation.read().expect("view").member
            else {
                panic!("type_argument");
            };
            serde_json::to_value(declaration).unwrap()
        };
        assert_eq!(
            read(&base),
            read(&candidate(&base, &ta01)),
            "node {position}"
        );
    }
}

/// FR-322-AC-492 rule 3: the key is `ix://` + package + `/` + identifier
/// segments; a segment that is no identifier, or an empty one, is refused.
#[trace("TC-048", "FR-322-AC-492", "FR-038-AC-583")]
#[test]
fn a_source_key_with_a_segment_that_is_no_identifier_is_invalid_value() {
    let (base, _) = base_of(&vectors());
    for node in [
        "ix://example/Mo ney",
        "ix://example/",
        "ix://example/Money/",
        "ix://example//Money",
        "ix://example/9Money",
        "ix://examplex/Money",
    ] {
        let mut package = base.clone();
        package["semantic_graph"]["nodes"][7]["body"]["operation"]["member"]["alias"] =
            json!({"package": "example", "node": node});
        assert_eq!(
            refusal_of(&package),
            (
                "refused:invalid_package/invalid-value".to_owned(),
                Some("/semantic_graph/nodes/7/body/operation/member/alias/node".to_owned())
            ),
            "{node}"
        );
    }
    let mut nested = base;
    nested["semantic_graph"]["nodes"][7]["body"]["operation"]["member"]["alias"] =
        json!({"package": "example", "node": "ix://example/Mod/Money"});
    assert_eq!(reading_of(&nested).unwrap(), "admitted");
}

/// FR-322-AC-492 rule 2: the owner join also takes a `package` named only by a
/// `lock.dependency_selections` entry.
#[trace("TC-048", "FR-322-AC-492", "FR-038-AC-583")]
#[test]
fn an_alias_package_named_by_a_dependency_selection_joins() {
    let (base, _) = base_of(&vectors());
    let mut package = base;
    package["lock"]["dependency_selections"] = json!([{"identity": "test/predicates"}]);
    package["semantic_graph"]["nodes"][7]["body"]["operation"]["member"]["alias"] =
        json!({"package": "test/predicates", "node": "ix://test/predicates/Money"});
    assert_eq!(reading_of(&package).unwrap(), "admitted");
}
