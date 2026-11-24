// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

use filament_contract_model::{
    CertificateNodeId, CertificatePathSegment as Segment, MAX_CERTIFICATE_NODE_ORDINAL,
};
use filament_exact::{NodeKey, PathSegment};
use ix_trace_rs::trace;
use serde_json::{json, Value};

fn canonical_node() -> Value {
    json!({
        "package": "pkg",
        "path": ["claim", {"package": "pkg", "path": ["operand"], "ordinal": 2}],
        "ordinal": 7
    })
}

#[trace("FR-019-AC-31", "FR-412-AC-34")]
#[test]
fn certificate_node_id_round_trips_exact_recursive_shape_and_node_key() {
    let canonical = canonical_node();
    let value: CertificateNodeId = serde_json::from_value(canonical.clone()).unwrap();
    assert_eq!(value.package(), "pkg");
    assert_eq!(value.ordinal(), 7);
    assert_eq!(value.path().len(), 2);
    assert!(matches!(value.path()[0], Segment::Text(ref part) if part == "claim"));
    assert!(matches!(value.path()[1], Segment::Node(ref node)
        if node.package() == "pkg" && node.path() == [Segment::Text("operand".to_owned())] && node.ordinal() == 2));
    assert_eq!(serde_json::to_value(&value).unwrap(), canonical);

    let key = NodeKey::new(
        "pkg",
        vec![
            PathSegment::Text("claim".to_owned()),
            PathSegment::Node(
                NodeKey::new("pkg", vec![PathSegment::Text("operand".to_owned())], 2).unwrap(),
            ),
        ],
        7,
    )
    .unwrap();
    let from_key = CertificateNodeId::from_node_key(&key).unwrap();
    assert_eq!(from_key, value);
    assert_eq!(from_key.to_node_key().unwrap(), key);
}

#[trace("FR-019-AC-31", "FR-412-AC-34")]
#[test]
fn certificate_node_id_reader_refuses_malformed_exact_shape() {
    let valid = canonical_node();
    let mut invalid = Vec::new();
    for member in ["package", "path", "ordinal"] {
        let mut value = valid.clone();
        value.as_object_mut().unwrap().remove(member);
        invalid.push(value);
    }
    let mut extra = valid.clone();
    extra["extra"] = json!(true);
    invalid.push(extra);
    let mut empty_package = valid.clone();
    empty_package["package"] = json!("");
    invalid.push(empty_package);
    let mut empty_path = valid.clone();
    empty_path["path"] = json!([]);
    invalid.push(empty_path);
    let mut nested_malformed = valid.clone();
    nested_malformed["path"][1]["ordinal"] = json!("2");
    invalid.push(nested_malformed);

    for input in invalid {
        assert!(serde_json::from_value::<CertificateNodeId>(input).is_err());
    }
    assert!(CertificateNodeId::new("pkg", vec![], 0).is_err());
    assert!(CertificateNodeId::new("", vec![Segment::Text("x".into())], 0).is_err());
    assert!(CertificateNodeId::new(
        "pkg",
        vec![Segment::Text("x".into())],
        MAX_CERTIFICATE_NODE_ORDINAL + 1,
    )
    .is_err());
}

#[trace("FR-019-AC-31", "FR-412-AC-34")]
#[test]
fn deeply_nested_certificate_identity_round_trips_without_a_depth_cap() {
    const DEPTH: usize = 512;

    let mut key = NodeKey::new("pkg", vec![PathSegment::Text("leaf".into())], 0).unwrap();
    for ordinal in 1..=DEPTH as u64 {
        key = NodeKey::new("pkg", vec![PathSegment::Node(key)], ordinal).unwrap();
    }

    let value = CertificateNodeId::from_node_key(&key).unwrap();
    let json = serde_json::to_value(&value).unwrap();
    let decoded: CertificateNodeId = serde_json::from_value(json).unwrap();
    let recovered = decoded.to_node_key().unwrap();
    assert_eq!(recovered, key);

    let mut node = &decoded;
    for expected_ordinal in (1..=DEPTH as u64).rev() {
        assert_eq!(node.package(), "pkg");
        assert_eq!(node.ordinal(), expected_ordinal);
        let [Segment::Node(child)] = node.path() else {
            panic!("each nested identity has exactly one structural child segment");
        };
        node = child;
    }
    assert_eq!(node.package(), "pkg");
    assert_eq!(node.ordinal(), 0);
    assert_eq!(node.path(), [Segment::Text("leaf".into())]);
}
