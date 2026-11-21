// SPDX-License-Identifier: AGPL-3.0-or-later
//! Public-reader work bounds for captured location joins.

use crate::fixture_support::{canonical, evidence_for};
use filament_contract_model::{
    CheckedBodyLocationOrigin, CheckedBodyLocationReadError, CheckedKeyOrdinal,
    CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use ix_trace_rs::trace;
use serde_json::{json, Value};

fn chain(count: usize, body: bool) -> Value {
    let mut package =
        crate::read_json(&crate::fixtures_dir().join("positive-operation-identities.json"))
            .expect("published arithmetic fixture");
    let original = package["semantic_graph"]["nodes"].as_array().unwrap();
    let integer = original[16].clone();
    let template = original[35].clone();
    let function = original
        .iter()
        .find(|node| node["node_tag"] == "function")
        .unwrap()
        .clone();
    let original_sources = package["source_map"].as_array().unwrap();
    let mut nodes = vec![integer.clone()];
    let mut sources = vec![original_sources[16].clone()];
    let mut rows = Vec::new();
    for index in 1..=count {
        let mut node = template.clone();
        node["node_id"]["path"] = json!(["expression", "binary", {"node": index - 1}]);
        if index > 1 {
            let previous = nodes[index - 1]["node_id"].clone();
            node["body"]["arguments"][0] = json!({"term":"reference","target":previous});
            node["dependencies"] = json!([previous]);
        }
        let mut source = original_sources[35].clone();
        source["node_id"] = node["node_id"].clone();
        let location = if body {
            json!({"origin":{"kind":"function_body","function":function["node_id"],"name":"double","index":0},
                "up":0,"down":if index == count {json!([])} else {json!([0])}})
        } else {
            json!({"origin":{"kind":"type_declaration","name":"monitor_root"},"path":[]})
        };
        rows.push(json!({"application":node["node_id"],"occurrence":node["occurrences"][0],"term_ordinal":0,"location":location}));
        nodes.push(node);
        sources.push(source);
    }
    if body {
        let mut function = function;
        function["body"] = json!({"term":"aggregate","members":[{"term":"binding","name":"body",
            "value":{"term":"reference","target":nodes[count]["node_id"]}}]});
        function["dependencies"] = json!([nodes[count]["node_id"]]);
        let mut source = original_sources
            .iter()
            .find(|source| source["node_id"] == function["node_id"])
            .unwrap()
            .clone();
        source["node_id"] = function["node_id"].clone();
        nodes.push(function);
        sources.push(source);
    }
    package["semantic_graph"]["nodes"] = json!(nodes);
    package["source_map"] = json!(sources);
    package["lock"]["definition_selections"] = json!([]);
    // Index coordinates are in structural node order.
    package["checked_term_data"] = json!({"equality_decisions":[],"dispatch_tables":[],
        "body_locations":if body {rows.clone()} else {vec![]},
        "protocol_locations":if body {vec![]} else {rows},"probabilistic_claims":[]});
    package
}

fn read(package: &Value, work: u64) -> CheckedPackageV2ReadResult {
    let bytes = canonical(package);
    let limits = CheckedPackageReadLimits {
        bytes: u64::try_from(bytes.len()).unwrap() + 1024,
        nodes: 300_010,
        edges: 600_010,
        occurrences: 600_010,
        diagnostics: 1,
        work,
    };
    CheckedPackageV2::read(&bytes, limits, &evidence_for(package))
}

// Large fixture builders relinquish their JSON before admission. Retaining
// both a producer-sized Value tree and the admitted reader state would measure
// fixture duplication rather than the reader's linear storage.
fn read_owned(mut package: Value, work: u64) -> CheckedPackageV2ReadResult {
    let evidence = evidence_for(&package);
    crate::fixture_support::wire_operands(&mut package);
    let bytes = serde_json::to_vec(&package).unwrap();
    drop(package);
    let limits = CheckedPackageReadLimits {
        bytes: u64::try_from(bytes.len()).unwrap() + 1024,
        nodes: 300_010,
        edges: 600_010,
        occurrences: 600_010,
        diagnostics: 1,
        work,
    };
    CheckedPackageV2::read(&bytes, limits, &evidence)
}

fn least_work(package: &Value, bound: u64) -> u64 {
    assert!(
        matches!(
            read(package, bound),
            CheckedPackageV2ReadResult::Admitted(_)
        ),
        "must admit within linear bound: {:?}",
        read(package, bound)
    );
    let (mut low, mut high) = (0, bound);
    while low < high {
        let mid = low + (high - low) / 2;
        if matches!(read(package, mid), CheckedPackageV2ReadResult::Admitted(_)) {
            high = mid;
        } else {
            low = mid + 1;
        }
    }
    assert!(matches!(
        read(package, low),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let CheckedPackageV2ReadResult::Incomplete(stopped) = read(package, low - 1) else {
        panic!("one unit below the least admitting work must be incomplete");
    };
    assert_eq!(stopped.limit_kind, CheckedPackageLimit::Work);
    assert_eq!(stopped.limit, low - 1);
    assert!(stopped.consumed > stopped.limit);
    low
}

#[test]
#[trace("TC-491", "FR-038-AC-597")]
fn body_location_join_scales_linearly_and_admits_a_thousand_terms() {
    let small = least_work(&chain(100, true), 1_000_000);
    let large = least_work(&chain(200, true), 1_000_000);
    assert!(large <= small * 2 + 100, "{small} -> {large}");
    assert!(matches!(
        read(&chain(1_000, true), 1_000_000),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    println!("body indexed work: 100={small}, 200={large}; exact/one-short passed; 1000 body terms admitted at 1000000");
}

#[test]
#[trace("TC-491", "FR-038-AC-597")]
fn protocol_location_join_scales_linearly_and_admits_one_hundred_thousand_rows() {
    let small = least_work(&chain(100, false), 1_000_000);
    let large = least_work(&chain(200, false), 1_000_000);
    assert!(large <= small * 2 + 100, "{small} -> {large}");
    assert!(matches!(
        read(&chain(100_000, false), 100_000_000),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    println!("protocol indexed work: 100={small}, 200={large}; exact/one-short passed; 100000 rows admitted");
}

#[test]
#[trace("TC-491", "FR-038-AC-597", "FR-322-AC-510")]
fn bl_d1_one_hundred_thousand_left_nested_applications_admit_at_the_reader() {
    let result = read_owned(chain(100_000, true), 100_000_000);
    let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
        panic!("BL-D1 must admit: {result:?}");
    };
    assert_eq!(
        admitted.checked_term_data().unwrap().body_locations.len(),
        100_000
    );
    assert_lazy_paths(&admitted, 100_000, false);
}

fn assert_lazy_paths(package: &CheckedPackageV2, count: usize, lets: bool) {
    let rows = &package.checked_term_data().unwrap().body_locations;
    let CheckedBodyLocationOrigin::FunctionBody { function, .. } = &rows[0].location.origin;
    let bound = u64::try_from(count).unwrap() * 100 + 10_000;
    // Cancel a partial read before measuring the independent full traversal.
    {
        let mut cancelled = package.body_locations(function, bound).unwrap();
        assert!(cancelled.next_location().unwrap().is_some());
    }
    let mut exact = 0;
    let allocations = allocation_counter::measure(|| {
        let mut cursor = package.body_locations(function, bound).unwrap();
        let mut seen = 0;
        while let Some(location) = cursor.next_location().unwrap() {
            assert_eq!(location.path.len(), seen + usize::from(lets));
            if [0, count / 2, count - 1].contains(&seen) {
                if lets {
                    assert!(location.path[..seen]
                        .iter()
                        .all(|entry| *entry == CheckedKeyOrdinal::Safe(1)));
                    assert_eq!(location.path.last(), Some(&CheckedKeyOrdinal::Safe(0)));
                } else {
                    assert!(location
                        .path
                        .iter()
                        .all(|entry| *entry == CheckedKeyOrdinal::Safe(0)));
                }
            }
            seen += 1;
        }
        assert_eq!(seen, count);
        exact = cursor.work_consumed();
    });
    let storage_bound =
        u64::try_from(package.resolved_graph().nodes.len() + rows.len()).unwrap() * 512 + 65_536;
    assert!(
        allocations.bytes_max <= storage_bound,
        "lazy cursor storage is linear: {allocations:?}, bound {storage_bound}"
    );
    assert_eq!(
        allocations.bytes_current, 0,
        "cursor storage is released on drop"
    );
    assert!(
        exact <= bound,
        "lazy reconstruction stays linear: {exact}/{bound}"
    );
    let mut exact_cursor = package.body_locations(function, exact).unwrap();
    let mut seen = 0;
    while exact_cursor.next_location().unwrap().is_some() {
        seen += 1;
    }
    assert_eq!(seen, count);
    assert_eq!(exact_cursor.work_consumed(), exact);
    let mut short = package.body_locations(function, exact - 1).unwrap();
    loop {
        match short.next_location() {
            Ok(Some(_)) => {}
            Err(CheckedBodyLocationReadError::WorkExhausted { limit, consumed }) => {
                assert_eq!(limit, exact - 1);
                assert!(consumed > limit);
                break;
            }
            other => panic!("one-short lazy cursor must exhaust: {}", other.is_ok()),
        }
    }
    assert!(
        short.next_location().unwrap().is_none(),
        "exhaustion is fused"
    );
    assert!(matches!(
        package.body_locations(function, 0),
        Err(CheckedBodyLocationReadError::WorkExhausted {
            limit: 0,
            consumed: 1
        })
    ));
    println!("lazy paths: lets={lets}, rows={count}, exact_work={exact}, peak_bytes={}, storage_bound={storage_bound}; kth path oracles, cancellation and one-short passed", allocations.bytes_max);
}

#[test]
#[trace("TC-491", "FR-038-AC-597")]
fn many_occurrences_of_one_application_join_without_a_per_row_scan() {
    let mut package = chain(1, false);
    let template = package["checked_term_data"]["protocol_locations"][0].clone();
    let source = package["source_map"][1].clone();
    let mut rows = Vec::new();
    let mut occurrences = Vec::new();
    let mut sources = vec![package["source_map"][0].clone()];
    for ordinal in 0..1_000 {
        let occurrence = json!({"role":"expression","ordinal":ordinal});
        let mut row = template.clone();
        row["occurrence"] = occurrence.clone();
        let mut entry = source.clone();
        entry["ordinal"] = json!(ordinal);
        rows.push(row);
        occurrences.push(occurrence);
        sources.push(entry);
    }
    package["semantic_graph"]["nodes"][1]["occurrences"] = json!(occurrences);
    package["source_map"] = json!(sources);
    package["checked_term_data"]["protocol_locations"] = json!(rows);
    let needed = least_work(&package, 100_000);
    println!("1000 occurrences of one node admit at exact work {needed}; one short incomplete");
}

fn nested_lets(count: usize) -> Value {
    let mut package =
        crate::read_json(&crate::fixtures_dir().join("positive-operation-identities.json"))
            .unwrap();
    let original = package["semantic_graph"]["nodes"].as_array().unwrap();
    let integer = original[16].clone();
    let text = original
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "text")
        .unwrap()
        .clone();
    let add = original[35].clone();
    let mut function = original
        .iter()
        .find(|node| node["node_tag"] == "function")
        .unwrap()
        .clone();
    let source = package["source_map"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["node_id"] == add["node_id"])
        .unwrap()
        .clone();
    let id = |path: Value| json!({"package":"example","path":path,"ordinal":0});
    let int_id = integer["node_id"].clone();
    let text_id = text["node_id"].clone();
    let parameter = |index: usize| {
        id(
            json!(["value","parameter","name",{"node":1},format!("x{index:06}"),"level",{"node":0},index.to_string(),{"node":0}]),
        )
    };
    let add_at = |index: usize| count + 3 + index;
    let let_at = |index: usize| 2 * count + 3 + index;
    let add_id = |index: usize| {
        id(
            json!(["expression","binary",{"node":index + 2},"1","filament.op.integer.add","binary",{"node":0}]),
        )
    };
    let let_id = |index: usize| {
        id(
            json!(["expression","let",{"node":index+3},format!("x{:06}",index+1),{"node":add_at(index)},{"node":if index+1 == count {index+3} else {let_at(index+1)}},"filament.op.control.let","let",{"node":0}]),
        )
    };
    let reference = |target: Value| json!({"term":"reference","target":target});
    let mut nodes = vec![integer, text];
    for index in 0..=count {
        let mut node = add.clone();
        node["node_id"] = parameter(index);
        node["node_tag"] = json!("value");
        node["semantic_form"] = json!("parameter");
        node["dependencies"] = json!([]);
        node["body"] = json!({"term":"aggregate","members":[
            {"term":"binding","name":"name","value":{"term":"literal","type":text_id,"value_kind":"text","value":format!("x{index:06}")}},
            {"term":"binding","name":"level","value":{"term":"literal","type":int_id,"value_kind":"integer","value":index.to_string()}}
        ]});
        nodes.push(node);
    }
    for index in 0..count {
        let mut node = add.clone();
        node["node_id"] = add_id(index);
        node["dependencies"] = json!([parameter(index)]);
        node["body"]["arguments"] = json!([reference(parameter(index)),{"term":"literal","type":int_id,"value_kind":"integer","value":"1"}]);
        nodes.push(node);
    }
    for index in 0..count {
        let mut node = add.clone();
        let body = if index + 1 == count {
            parameter(index + 1)
        } else {
            let_id(index + 1)
        };
        node["node_id"] = let_id(index);
        node["semantic_form"] = json!("let");
        node["body"]["operator"] = json!("let");
        node["body"]["operation"]["identity"] = json!("filament.op.control.let");
        node["body"]["arguments"] = json!([
            {"term":"binding","name":format!("x{:06}",index+1),"binder":parameter(index+1),"value":reference(add_id(index))},reference(body.clone())]);
        node["dependencies"] = if index + 1 == count {
            json!([add_id(index), body])
        } else {
            json!([add_id(index), body, parameter(index + 1)])
        };
        nodes.push(node);
    }
    function["body"] = json!({"term":"aggregate","members":[
        {"term":"binding","name":"parameters","value":{"term":"aggregate","members":[reference(parameter(0))]}},
        {"term":"binding","name":"body","value":reference(let_id(0))}]});
    function["dependencies"] = json!([let_id(0), parameter(0)]);
    let rows: Vec<_> = (0..count).map(|index| json!({"application":add_id(index),"occurrence":{"role":"expression","ordinal":0},"term_ordinal":0,
        "location":{"origin":{"kind":"function_body","function":function["node_id"],"name":"double","index":0},"up":if index == 0 {0} else {1},"down":if index == 0 {json!([0])} else {json!([1,0])}}})).collect();
    nodes.push(function);
    let sources: Vec<_> = nodes
        .iter()
        .map(|node| {
            let mut entry = source.clone();
            entry["node_id"] = node["node_id"].clone();
            entry["role"] = node["occurrences"][0]["role"].clone();
            entry
        })
        .collect();
    package["semantic_graph"]["nodes"] = json!(nodes);
    package["source_map"] = json!(sources);
    package["lock"]["definition_selections"] = json!([]);
    package["checked_term_data"] = json!({"equality_decisions":[],"protocol_locations":[],"dispatch_tables":[],"probabilistic_claims":[],"body_locations":rows});
    package
}

#[test]
#[trace("TC-491", "FR-038-AC-597", "FR-322-AC-510")]
fn bl_d2_one_hundred_thousand_nested_lets_admit_at_the_reader() {
    for count in [10, 100_000] {
        let result = read_owned(
            nested_lets(count),
            u64::try_from(count).unwrap() * 1_000 + 10_000,
        );
        let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
            panic!("BL-D2 {count} must admit: {result:?}");
        };
        assert_eq!(
            admitted.checked_term_data().unwrap().body_locations.len(),
            count
        );
        assert_lazy_paths(&admitted, count, true);
    }
}
