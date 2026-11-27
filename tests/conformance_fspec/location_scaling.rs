// SPDX-License-Identifier: AGPL-3.0-or-later
//! Public-reader work bounds for captured location joins.

use crate::fixture_support::{canonical, evidence_for};
use filament_contract_model::{
    CheckedBodyLocationOrigin, CheckedBodyLocationReadError, CheckedKeyOrdinal,
    CheckedPackageEvidence, CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2ReadResult,
};
use ix_trace_rs::trace;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};
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
    read_serialized_owned(bytes, evidence, work)
}

fn read_serialized_owned(
    bytes: Vec<u8>,
    evidence: CheckedPackageEvidence,
    work: u64,
) -> CheckedPackageV2ReadResult {
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
    println!(
        "body indexed work: 100={small}, 200={large}; exact/one-short passed; 1000 body terms admitted at 1000000"
    );
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
    println!(
        "protocol indexed work: 100={small}, 200={large}; exact/one-short passed; 100000 rows admitted"
    );
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
    println!(
        "lazy paths: lets={lets}, rows={count}, exact_work={exact}, peak_bytes={}, storage_bound={storage_bound}; kth path oracles, cancellation and one-short passed",
        allocations.bytes_max
    );
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

struct NestedLets {
    base: Value,
    count: usize,
    integer: Value,
    text: Value,
    add: Value,
    function: Value,
    source: Value,
}

impl NestedLets {
    fn node_count(&self) -> usize {
        3 * self.count + 4
    }

    fn id(&self, path: Value) -> Value {
        json!({"package":"example","path":path,"ordinal":0})
    }

    fn parameter_id(&self, index: usize) -> Value {
        self.id(json!([
            "value", "parameter", "name", {"node":1}, format!("x{index:06}"),
            "level", {"node":0}, index.to_string(), {"node":0}
        ]))
    }

    fn add_node_index(&self, index: usize) -> usize {
        self.count + 3 + index
    }

    fn let_node_index(&self, index: usize) -> usize {
        2 * self.count + 3 + index
    }

    fn add_id(&self, index: usize) -> Value {
        self.id(json!([
            "expression", "binary", {"node":index + 2}, "1",
            "filament.op.integer.add", "binary", {"node":0}
        ]))
    }

    fn let_id(&self, index: usize) -> Value {
        self.id(json!([
            "expression", "let", {"node":index + 3}, format!("x{:06}", index + 1),
            {"node":self.add_node_index(index)},
            {"node":if index + 1 == self.count {index + 3} else {self.let_node_index(index + 1)}},
            "filament.op.control.let", "let", {"node":0}
        ]))
    }

    fn node(&self, position: usize) -> Value {
        if position == 0 {
            return self.integer.clone();
        }
        if position == 1 {
            return self.text.clone();
        }
        if position < self.count + 3 {
            let index = position - 2;
            let mut node = self.add.clone();
            node["node_id"] = self.parameter_id(index);
            node["node_tag"] = json!("value");
            node["semantic_form"] = json!("parameter");
            node["dependencies"] = json!([]);
            node["body"] = json!({"term":"aggregate","members":[
                {"term":"binding","name":"name","value":{"term":"literal","type":self.text["node_id"],"value_kind":"text","value":format!("x{index:06}")}},
                {"term":"binding","name":"level","value":{"term":"literal","type":self.integer["node_id"],"value_kind":"integer","value":index.to_string()}}
            ]});
            return node;
        }
        if position < 2 * self.count + 3 {
            let index = position - (self.count + 3);
            let mut node = self.add.clone();
            node["node_id"] = self.add_id(index);
            node["dependencies"] = json!([self.parameter_id(index)]);
            node["body"]["arguments"] = json!([
                {"term":"reference","target":self.parameter_id(index)},
                {"term":"literal","type":self.integer["node_id"],"value_kind":"integer","value":"1"}
            ]);
            return node;
        }
        if position < 3 * self.count + 3 {
            let index = position - (2 * self.count + 3);
            let body = if index + 1 == self.count {
                self.parameter_id(index + 1)
            } else {
                self.let_id(index + 1)
            };
            let mut node = self.add.clone();
            node["node_id"] = self.let_id(index);
            node["semantic_form"] = json!("let");
            node["body"]["operator"] = json!("let");
            node["body"]["operation"]["identity"] = json!("filament.op.control.let");
            node["body"]["arguments"] = json!([
                {"term":"binding","name":format!("x{:06}", index + 1),"binder":self.parameter_id(index + 1),"value":{"term":"reference","target":self.add_id(index)}},
                {"term":"reference","target":body}
            ]);
            node["dependencies"] = if index + 1 == self.count {
                json!([self.add_id(index), body])
            } else {
                json!([self.add_id(index), body, self.parameter_id(index + 1)])
            };
            return node;
        }
        let mut function = self.function.clone();
        function["body"] = json!({"term":"aggregate","members":[
            {"term":"binding","name":"parameters","value":{"term":"aggregate","members":[{"term":"reference","target":self.parameter_id(0)}]}},
            {"term":"binding","name":"body","value":{"term":"reference","target":self.let_id(0)}}
        ]});
        function["dependencies"] = json!([self.let_id(0), self.parameter_id(0)]);
        function
    }

    fn wire_node(&self, node: Value) -> (Value, Value) {
        let mut package = json!({
            "contract_version":"filament.checked-package/v2",
            "semantic_graph":{"nodes":[node]}
        });
        crate::fixture_support::wire_operands(&mut package);
        let graph = package.get_mut("semantic_graph").unwrap();
        let node = std::mem::take(&mut graph["nodes"][0]);
        let row = std::mem::take(&mut graph["operand_rows"][0]);
        (node, row)
    }

    fn source_entry(&self, position: usize) -> Value {
        let (node_id, role) = if position == 0 {
            (
                self.integer["node_id"].clone(),
                self.integer["occurrences"][0]["role"].clone(),
            )
        } else if position == 1 {
            (
                self.text["node_id"].clone(),
                self.text["occurrences"][0]["role"].clone(),
            )
        } else if position < self.count + 3 {
            (
                self.parameter_id(position - 2),
                self.add["occurrences"][0]["role"].clone(),
            )
        } else if position < 2 * self.count + 3 {
            (
                self.add_id(position - (self.count + 3)),
                self.add["occurrences"][0]["role"].clone(),
            )
        } else if position < 3 * self.count + 3 {
            (
                self.let_id(position - (2 * self.count + 3)),
                self.add["occurrences"][0]["role"].clone(),
            )
        } else {
            (
                self.function["node_id"].clone(),
                self.function["occurrences"][0]["role"].clone(),
            )
        };
        let mut source = self.source.clone();
        source["node_id"] = node_id;
        source["role"] = role;
        source
    }
}

struct NestedNodes<'a>(&'a NestedLets);

impl Serialize for NestedNodes<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.0.node_count()))?;
        for position in 0..self.0.node_count() {
            let (node, _) = self.0.wire_node(self.0.node(position));
            sequence.serialize_element(&node)?;
        }
        sequence.end()
    }
}

struct NestedOperandRows<'a>(&'a NestedLets);

impl Serialize for NestedOperandRows<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.0.node_count()))?;
        for position in 0..self.0.node_count() {
            let (_, row) = self.0.wire_node(self.0.node(position));
            sequence.serialize_element(&row)?;
        }
        sequence.end()
    }
}

struct NestedSources<'a>(&'a NestedLets);

impl Serialize for NestedSources<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.0.node_count()))?;
        for position in 0..self.0.node_count() {
            sequence.serialize_element(&self.0.source_entry(position))?;
        }
        sequence.end()
    }
}

struct NestedBodyLocations<'a>(&'a NestedLets);

impl Serialize for NestedBodyLocations<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.0.count))?;
        for index in 0..self.0.count {
            sequence.serialize_element(&json!({
                "application":self.0.add_id(index),
                "occurrence":{"role":"expression","ordinal":0},
                "term_ordinal":0,
                "location":{
                    "origin":{"kind":"function_body","function":self.0.function["node_id"],"name":"double","index":0},
                    "up":if index == 0 {0} else {1},
                    "down":if index == 0 {json!([0])} else {json!([1,0])}
                }
            }))?;
        }
        sequence.end()
    }
}

struct NestedGraph<'a> {
    base: &'a Value,
    fixture: &'a NestedLets,
}

impl Serialize for NestedGraph<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let object = self.base.as_object().expect("semantic graph object");
        let mut map = serializer.serialize_map(Some(object.len()))?;
        for (key, value) in object {
            match key.as_str() {
                "nodes" => map.serialize_entry(key, &NestedNodes(self.fixture))?,
                "operand_rows" => map.serialize_entry(key, &NestedOperandRows(self.fixture))?,
                _ => map.serialize_entry(key, value)?,
            }
        }
        map.end()
    }
}

struct NestedLock<'a>(&'a Value);

impl Serialize for NestedLock<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let object = self.0.as_object().expect("lock object");
        let mut map = serializer.serialize_map(Some(object.len()))?;
        for (key, value) in object {
            if key == "definition_selections" {
                map.serialize_entry(key, &[] as &[Value])?;
            } else {
                map.serialize_entry(key, value)?;
            }
        }
        map.end()
    }
}

struct NestedCheckedTermData<'a> {
    base: &'a Value,
    fixture: &'a NestedLets,
}

impl Serialize for NestedCheckedTermData<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let object = self.base.as_object().expect("checked term data object");
        let mut map = serializer.serialize_map(Some(object.len()))?;
        for (key, value) in object {
            if key == "body_locations" {
                map.serialize_entry(key, &NestedBodyLocations(self.fixture))?;
            } else {
                map.serialize_entry(key, value)?;
            }
        }
        map.end()
    }
}

struct NestedPackage<'a>(&'a NestedLets);

impl Serialize for NestedPackage<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let object = self.0.base.as_object().expect("package object");
        let mut map = serializer.serialize_map(Some(object.len()))?;
        for (key, value) in object {
            match key.as_str() {
                "semantic_graph" => map.serialize_entry(
                    key,
                    &NestedGraph {
                        base: value,
                        fixture: self.0,
                    },
                )?,
                "source_map" => map.serialize_entry(key, &NestedSources(self.0))?,
                "lock" => map.serialize_entry(key, &NestedLock(value))?,
                "checked_term_data" => map.serialize_entry(
                    key,
                    &NestedCheckedTermData {
                        base: value,
                        fixture: self.0,
                    },
                )?,
                _ => map.serialize_entry(key, value)?,
            }
        }
        map.end()
    }
}

fn nested_lets(count: usize) -> (Vec<u8>, CheckedPackageEvidence) {
    let mut base =
        crate::read_json(&crate::fixtures_dir().join("positive-operation-identities.json"))
            .unwrap();
    let original = base["semantic_graph"]["nodes"].as_array().unwrap();
    let integer = original[16].clone();
    let text = original
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "text")
        .unwrap()
        .clone();
    let add = original[35].clone();
    let function = original
        .iter()
        .find(|node| node["node_tag"] == "function")
        .unwrap()
        .clone();
    let source = base["source_map"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["node_id"] == add["node_id"])
        .unwrap()
        .clone();

    base["semantic_graph"]["nodes"] = json!([]);
    base["semantic_graph"]["operand_rows"] = json!([]);
    base["source_map"] = json!([]);
    base["lock"]["definition_selections"] = json!([]);
    base["checked_term_data"] = json!({
        "equality_decisions":[],"protocol_locations":[],"dispatch_tables":[],
        "probabilistic_claims":[],"body_locations":[]
    });
    let evidence = evidence_for(&base);
    let fixture = NestedLets {
        base,
        count,
        integer,
        text,
        add,
        function,
        source,
    };
    let mut bytes = Vec::new();
    serde_json::to_writer(&mut bytes, &NestedPackage(&fixture)).unwrap();
    (bytes, evidence)
}

#[test]
#[trace("TC-491", "FR-038-AC-597", "FR-322-AC-510")]
fn bl_d2_one_hundred_thousand_nested_lets_admit_at_the_reader() {
    for count in [10, 100_000] {
        let (bytes, evidence) = nested_lets(count);
        let result = read_serialized_owned(
            bytes,
            evidence,
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
