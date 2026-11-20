// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The type renderings of FSpec FR-789 "Rendering" over the type nodes of a
//! package, shared by the strict-read half (contract-ir FR-374) and the
//! identity-row join of a source timed model's clock places.

use super::{
    BoundedDomainForm, CheckedNodeKind, CheckedNodeOwner, CheckedTerm, CompositeTypeForm,
    ModelForm, ScalarTypeForm,
};
use crate::checked_package::shared::CheckedNodeId;

/// The graph a rendering reads.
pub(super) trait TypeGraph {
    fn kind(&self, position: usize) -> Option<CheckedNodeKind>;
    fn owner(&self, position: usize) -> Option<&CheckedNodeOwner>;
    fn declaration(&self, position: usize) -> Option<&super::CheckedDeclaration>;
    fn dependencies(&self, position: usize) -> &[CheckedNodeId];
    fn body(&self, position: usize) -> Option<&CheckedTerm>;
    fn locate(&self, id: &CheckedNodeId) -> Option<usize>;
    fn size(&self) -> usize;
}

fn members(term: &CheckedTerm) -> &[CheckedTerm] {
    match term {
        CheckedTerm::Aggregate { members } => members,
        _ => &[],
    }
}

fn target(term: &CheckedTerm) -> Option<&CheckedNodeId> {
    match term {
        CheckedTerm::Reference { target } => Some(target),
        _ => None,
    }
}

/// The text of the literal bound at `index` of an aggregate body.
fn literal_text(body: &CheckedTerm, index: usize) -> Option<&str> {
    match members(body).get(index)? {
        CheckedTerm::Binding { value, .. } => match value.as_ref() {
            CheckedTerm::Literal { value, .. } => value.as_str(),
            _ => None,
        },
        _ => None,
    }
}

/// The binding of `name` in the body of the declared type at `node`: whether
/// it is `optional` and the type node it references; `None` for a name with
/// no binding.
pub(super) fn binding_of<'g, G: TypeGraph + ?Sized>(
    graph: &'g G,
    node: usize,
    name: &str,
) -> Option<(bool, &'g CheckedNodeId)> {
    members(graph.body(node)?).iter().find_map(|member| {
        let CheckedTerm::Binding {
            name: bound_name,
            value,
        } = member
        else {
            return None;
        };
        if bound_name.as_ref() != name {
            return None;
        }
        match value.as_ref() {
            CheckedTerm::Reference { target } => Some((false, target)),
            CheckedTerm::Aggregate { members } => match members.as_slice() {
                [CheckedTerm::Binding { value, .. }] => target(value).map(|target| (true, target)),
                _ => None,
            },
            _ => None,
        }
    })
}

/// What FSpec FR-789 "Rendering" says of a type node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum Rendered {
    /// The node's rendering.
    Known(String),
    /// A node of a form FR-789 gives no rendering and the architecture rules
    /// interim: an integer or other range, a text bound, a quantity. Its
    /// field's `member_type` is not compared.
    Interim,
    /// A node no rendering names: a row over it does not join.
    Unknown,
}

/// The step of a clock path a declaring type is looked up for.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum DeclaringStep {
    Field,
    Tuple,
    Union,
}

/// The first reference bound as `name` in an aggregate body.
fn named_target<'g>(body: &'g CheckedTerm, name: &str) -> Option<&'g CheckedNodeId> {
    members(body).iter().find_map(|member| match member {
        CheckedTerm::Binding {
            name: bound_name,
            value,
        } if bound_name.as_ref() == name => target(value),
        _ => None,
    })
}

/// The one-segment source declaration key `ix://I/<name>` of a source-owned
/// node of any form.
fn source_key<G: TypeGraph + ?Sized>(graph: &G, position: usize) -> Option<String> {
    match (graph.owner(position), graph.declaration(position)) {
        (Some(CheckedNodeOwner::Source { identity, .. }), Some(declaration)) => {
            match declaration.qualified_name.as_slice() {
                [name] => Some(format!("ix://{identity}/{name}")),
                _ => None,
            }
        }
        _ => None,
    }
}

/// A map key's rendering: the bare name of a native scalar, otherwise the
/// key type's source declaration key.
fn key_rendering<G: TypeGraph + ?Sized>(graph: &G, position: usize) -> Option<String> {
    if let Some(key) = source_key(graph, position) {
        return Some(key);
    }
    match graph.kind(position)? {
        CheckedNodeKind::ScalarType(form) if graph.owner(position).is_none() => {
            native_name(form).map(str::to_owned)
        }
        _ => None,
    }
}

/// The rendering of the type node at `position`. `tick` is called once per
/// node visited.
pub(super) fn render<G: TypeGraph + ?Sized, E>(
    graph: &G,
    mut position: usize,
    tick: &mut impl FnMut() -> Result<(), E>,
) -> Result<Rendered, E> {
    let mut prefix = String::new();
    let mut closing = 0_usize;
    for _ in 0..=graph.size() {
        tick()?;
        let Some(kind) = graph.kind(position) else {
            return Ok(Rendered::Unknown);
        };
        let element = || {
            graph
                .body(position)
                .and_then(|body| members(body).first())
                .and_then(target)
                .and_then(|id| graph.locate(id))
        };
        let leaf = match kind {
            CheckedNodeKind::Model(ModelForm::ObjectType)
            | CheckedNodeKind::CompositeType(
                CompositeTypeForm::Record | CompositeTypeForm::Tuple | CompositeTypeForm::Union,
            )
            | CheckedNodeKind::ScalarType(_)
                if graph.owner(position).is_some() =>
            {
                match source_key(graph, position) {
                    Some(key) => key,
                    None => return Ok(Rendered::Unknown),
                }
            }
            CheckedNodeKind::ScalarType(
                ScalarTypeForm::CarriedQuantity
                | ScalarTypeForm::BoundedQuantity
                | ScalarTypeForm::CompoundUnit,
            )
            | CheckedNodeKind::BoundedDomain(
                BoundedDomainForm::IntegerRange
                | BoundedDomainForm::RationalRange
                | BoundedDomainForm::DecimalRange
                | BoundedDomainForm::FloatRounding
                | BoundedDomainForm::TextBounds,
            ) => return Ok(Rendered::Interim),
            CheckedNodeKind::ScalarType(form) => match native_name(form) {
                Some(name) => format!("ix://filament/native/{name}"),
                None => return Ok(Rendered::Unknown),
            },
            CheckedNodeKind::CompositeType(
                CompositeTypeForm::Reference | CompositeTypeForm::Alias,
            ) => {
                let Some(next) = element() else {
                    return Ok(Rendered::Unknown);
                };
                position = next;
                continue;
            }
            CheckedNodeKind::BoundedDomain(BoundedDomainForm::CollectionBounds) => {
                let Some(next) = graph
                    .dependencies(position)
                    .first()
                    .and_then(|id| graph.locate(id))
                else {
                    return Ok(Rendered::Unknown);
                };
                position = next;
                continue;
            }
            CheckedNodeKind::CompositeType(CompositeTypeForm::Map) => {
                let operands = graph.body(position).and_then(|body| {
                    let key = named_target(body, "key").and_then(|id| graph.locate(id))?;
                    let value = named_target(body, "value").and_then(|id| graph.locate(id))?;
                    Some((key, value))
                });
                let Some((key, value)) = operands else {
                    return Ok(Rendered::Unknown);
                };
                let Some(key) = key_rendering(graph, key) else {
                    return Ok(Rendered::Unknown);
                };
                prefix.push_str("Map<");
                prefix.push_str(&key);
                prefix.push_str(", ");
                closing += 1;
                position = value;
                continue;
            }
            CheckedNodeKind::CompositeType(form) => {
                let wrapper = match form {
                    CompositeTypeForm::Option => "Option<",
                    CompositeTypeForm::Sequence => "Sequence<",
                    CompositeTypeForm::Set => "Set<",
                    CompositeTypeForm::Bag => "Bag<",
                    CompositeTypeForm::OrderedSet => "OrderedSet<",
                    _ => return Ok(Rendered::Unknown),
                };
                let Some(next) = element() else {
                    return Ok(Rendered::Unknown);
                };
                prefix.push_str(wrapper);
                closing += 1;
                position = next;
                continue;
            }
            _ => return Ok(Rendered::Unknown),
        };
        prefix.push_str(&leaf);
        prefix.extend(std::iter::repeat_n('>', closing));
        return Ok(Rendered::Known(prefix));
    }
    Ok(Rendered::Unknown)
}

#[ix_trace_rs::string_edge]
fn native_name(form: ScalarTypeForm) -> Option<&'static str> {
    Some(match form {
        ScalarTypeForm::Boolean => "Boolean",
        ScalarTypeForm::Integer => "Integer",
        ScalarTypeForm::Text => "Text",
        ScalarTypeForm::String => "String",
        ScalarTypeForm::Rational => "Rational",
        ScalarTypeForm::Decimal => "Decimal",
        ScalarTypeForm::Float32 => "Float32",
        ScalarTypeForm::Float64 => "Float64",
        ScalarTypeForm::Bytes => "Bytes",
        ScalarTypeForm::JsonObject => "JsonObject",
        ScalarTypeForm::Uuid => "UUID",
        ScalarTypeForm::Timestamp => "Timestamp",
        _ => return None,
    })
}

/// The collection kind, minimum and maximum a data type node spells for the
/// `slot` step at `depth` collections down: the kind of that collection node
/// and the bounds of its `collection_bounds` node, or `0` and `unbounded`
/// without one.
pub(super) fn collection_layer<G: TypeGraph + ?Sized>(
    graph: &G,
    mut position: usize,
    mut depth: usize,
) -> Option<(&'static str, &str, &str)> {
    let mut bounds = None;
    for _ in 0..=graph.size() {
        match graph.kind(position)? {
            CheckedNodeKind::CompositeType(
                CompositeTypeForm::Option | CompositeTypeForm::Alias | CompositeTypeForm::Reference,
            ) => {
                position = graph
                    .body(position)
                    .and_then(|body| members(body).first())
                    .and_then(target)
                    .and_then(|id| graph.locate(id))?;
            }
            CheckedNodeKind::BoundedDomain(BoundedDomainForm::CollectionBounds) => {
                let body = graph.body(position)?;
                bounds = Some((literal_text(body, 0)?, literal_text(body, 1)?));
                position = graph
                    .dependencies(position)
                    .first()
                    .and_then(|id| graph.locate(id))?;
            }
            CheckedNodeKind::CompositeType(form) => {
                let name = match form {
                    CompositeTypeForm::Sequence => "sequence",
                    CompositeTypeForm::Set => "set",
                    CompositeTypeForm::Bag => "bag",
                    CompositeTypeForm::OrderedSet => "ordered_set",
                    CompositeTypeForm::Map => "map",
                    _ => return None,
                };
                if depth == 0 {
                    let (lower, upper) = bounds.unwrap_or(("0", "unbounded"));
                    return Some((name, lower, upper));
                }
                depth -= 1;
                bounds = None;
                let body = graph.body(position)?;
                let inner = if form == CompositeTypeForm::Map {
                    named_target(body, "value")
                } else {
                    members(body).first().and_then(target)
                };
                position = inner.and_then(|id| graph.locate(id))?;
            }
            _ => return None,
        }
    }
    None
}

/// Whether `text` is a well-formed type rendering: the wrappers `Option<..>`,
/// `Sequence<..>`, `Set<..>`, `Bag<..>`, `OrderedSet<..>` and `Map<K, ..>`
/// around a source declaration key, a native type, or a product `(..)`.
#[ix_trace_rs::string_edge]
pub(super) fn well_formed(text: &str) -> bool {
    let mut rest = text;
    loop {
        let mut stripped = false;
        for prefix in ["Option<", "Sequence<", "Set<", "Bag<", "OrderedSet<"] {
            if let Some(inner) = rest
                .strip_prefix(prefix)
                .and_then(|inner| inner.strip_suffix('>'))
            {
                rest = inner;
                stripped = true;
                break;
            }
        }
        if let Some(inner) = rest
            .strip_prefix("Map<")
            .and_then(|inner| inner.strip_suffix('>'))
        {
            let Some((_, value)) = inner.split_once(", ") else {
                return false;
            };
            rest = value;
            stripped = true;
        }
        if !stripped {
            break;
        }
    }
    (rest.starts_with("ix://") && rest.len() > "ix://".len() && !rest.contains(char::is_whitespace))
        || (rest.starts_with('(') && rest.ends_with(')'))
}

/// The source type node `ix://<identity>/<name>` names: a node of the
/// `SourceOwner` `identity` whose declaration is `[<name>]`, an object type,
/// native record, tuple or union that the step's kind names: an object type
/// or record for a `field`, a tuple for a `tuple`, a union for a `union`.
#[ix_trace_rs::string_edge]
pub(super) fn declaration_node<G: TypeGraph + ?Sized>(
    graph: &G,
    identity: &str,
    declaring: &str,
    step: DeclaringStep,
) -> Option<usize> {
    let name = declaring
        .strip_prefix("ix://")?
        .strip_prefix(identity)?
        .strip_prefix('/')?;
    (0..graph.size()).find(|&position| {
        let Some(kind) = graph.kind(position) else {
            return false;
        };
        matches!(
            graph.owner(position),
            Some(CheckedNodeOwner::Source { identity: owner, .. }) if owner.as_ref() == identity
        ) && graph
            .declaration(position)
            .is_some_and(|d| matches!(d.qualified_name.as_slice(), [only] if only.as_ref() == name))
            && match step {
                DeclaringStep::Field => matches!(
                    kind,
                    CheckedNodeKind::Model(ModelForm::ObjectType)
                        | CheckedNodeKind::CompositeType(CompositeTypeForm::Record)
                ),
                DeclaringStep::Tuple => {
                    kind == CheckedNodeKind::CompositeType(CompositeTypeForm::Tuple)
                }
                DeclaringStep::Union => {
                    kind == CheckedNodeKind::CompositeType(CompositeTypeForm::Union)
                }
            }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checked_package::v2::{CheckedDeclaration, CheckedNodeOwner};
    use ix_trace_rs::trace;
    use serde_json::json;

    struct Mock {
        nodes: Vec<Entry>,
    }

    struct Entry {
        kind: CheckedNodeKind,
        owner: Option<CheckedNodeOwner>,
        declaration: Option<CheckedDeclaration>,
        dependencies: Vec<CheckedNodeId>,
        body: Option<CheckedTerm>,
    }

    fn id(position: usize) -> CheckedNodeId {
        serde_json::from_value(
            json!({"package": "p", "path": [position.to_string()], "ordinal": 0}),
        )
        .expect("id")
    }

    fn reference(position: usize) -> serde_json::Value {
        json!({"term": "reference", "target": {"package": "p", "path": [position.to_string()], "ordinal": 0}})
    }

    fn aggregate(members: Vec<serde_json::Value>) -> CheckedTerm {
        serde_json::from_value(json!({"term": "aggregate", "members": members})).expect("term")
    }

    fn bound(name: &str, value: serde_json::Value) -> serde_json::Value {
        json!({"term": "binding", "name": name, "value": value})
    }

    impl Mock {
        fn push(
            &mut self,
            kind: CheckedNodeKind,
            source: Option<&str>,
            body: Option<CheckedTerm>,
        ) -> usize {
            self.nodes.push(Entry {
                kind,
                owner: source.map(|_| CheckedNodeOwner::Source {
                    authority: "a".into(),
                    identity: "I".into(),
                }),
                declaration: source.map(|name| CheckedDeclaration {
                    qualified_name: vec![name.into()],
                }),
                dependencies: vec![],
                body,
            });
            self.nodes.len() - 1
        }
    }

    impl TypeGraph for Mock {
        fn kind(&self, position: usize) -> Option<CheckedNodeKind> {
            self.nodes.get(position).map(|node| node.kind)
        }
        fn owner(&self, position: usize) -> Option<&CheckedNodeOwner> {
            self.nodes.get(position)?.owner.as_ref()
        }
        fn declaration(&self, position: usize) -> Option<&CheckedDeclaration> {
            self.nodes.get(position)?.declaration.as_ref()
        }
        fn dependencies(&self, position: usize) -> &[CheckedNodeId] {
            self.nodes
                .get(position)
                .map_or(&[][..], |node| &node.dependencies)
        }
        fn body(&self, position: usize) -> Option<&CheckedTerm> {
            self.nodes.get(position)?.body.as_ref()
        }
        fn locate(&self, wanted: &CheckedNodeId) -> Option<usize> {
            (0..self.nodes.len()).find(|&position| &id(position) == wanted)
        }
        fn size(&self) -> usize {
            self.nodes.len()
        }
    }

    fn scalar(graph: &mut Mock, form: ScalarTypeForm) -> usize {
        graph.push(CheckedNodeKind::ScalarType(form), None, None)
    }

    fn wrapped(graph: &mut Mock, form: CompositeTypeForm, element: usize) -> usize {
        let body = aggregate(vec![reference(element)]);
        graph.push(CheckedNodeKind::CompositeType(form), None, Some(body))
    }

    fn render_at(graph: &Mock, position: usize) -> Rendered {
        render(graph, position, &mut || Ok::<(), ()>(())).expect("no meter")
    }

    fn known(text: &str) -> Rendered {
        Rendered::Known(text.to_owned())
    }

    /// FSpec FR-789 "Rendering": the natives, a map with a bare scalar key
    /// and with a source key, and the collection wrappers.
    #[trace("TC-481", "FR-374-AC-2")]
    #[test]
    fn natives_maps_and_collections_render_as_fr_789_spells_them() {
        let mut graph = Mock { nodes: vec![] };
        for (form, name) in [
            (ScalarTypeForm::Uuid, "UUID"),
            (ScalarTypeForm::Timestamp, "Timestamp"),
            (ScalarTypeForm::Bytes, "Bytes"),
            (ScalarTypeForm::JsonObject, "JsonObject"),
            (ScalarTypeForm::Text, "Text"),
        ] {
            let at = scalar(&mut graph, form);
            assert_eq!(
                render_at(&graph, at),
                known(&format!("ix://filament/native/{name}"))
            );
        }
        let text = scalar(&mut graph, ScalarTypeForm::Text);
        let record = graph.push(
            CheckedNodeKind::CompositeType(CompositeTypeForm::Record),
            Some("R"),
            Some(aggregate(vec![])),
        );
        let map = graph.push(
            CheckedNodeKind::CompositeType(CompositeTypeForm::Map),
            None,
            Some(aggregate(vec![
                bound("key", reference(text)),
                bound("value", reference(record)),
            ])),
        );
        assert_eq!(render_at(&graph, map), known("Map<Text, ix://I/R>"));
        let enumeration = graph.push(
            CheckedNodeKind::ScalarType(ScalarTypeForm::Enum),
            Some("E"),
            None,
        );
        let keyed = graph.push(
            CheckedNodeKind::CompositeType(CompositeTypeForm::Map),
            None,
            Some(aggregate(vec![
                bound("key", reference(enumeration)),
                bound("value", reference(text)),
            ])),
        );
        let sequence = wrapped(&mut graph, CompositeTypeForm::Sequence, keyed);
        let optional = wrapped(&mut graph, CompositeTypeForm::Option, sequence);
        assert_eq!(
            render_at(&graph, optional),
            known("Option<Sequence<Map<ix://I/E, ix://filament/native/Text>>>")
        );
        assert_eq!(render_at(&graph, enumeration), known("ix://I/E"));
    }

    /// A node FR-789 gives no rendering is `Unknown`, except the interim
    /// forms the architecture rules skipped.
    #[trace("TC-481", "FR-374-AC-2")]
    #[test]
    fn unrenderable_nodes_are_unknown_and_ranges_and_quantities_interim() {
        let mut graph = Mock { nodes: vec![] };
        let integer = scalar(&mut graph, ScalarTypeForm::Integer);
        let dimension = scalar(&mut graph, ScalarTypeForm::Dimension);
        assert_eq!(render_at(&graph, dimension), Rendered::Unknown);
        let map = graph.push(
            CheckedNodeKind::CompositeType(CompositeTypeForm::Map),
            None,
            Some(aggregate(vec![bound("key", reference(integer))])),
        );
        assert_eq!(render_at(&graph, map), Rendered::Unknown);
        let tuple_key = graph.push(
            CheckedNodeKind::CompositeType(CompositeTypeForm::Tuple),
            None,
            Some(aggregate(vec![])),
        );
        let bad_key = graph.push(
            CheckedNodeKind::CompositeType(CompositeTypeForm::Map),
            None,
            Some(aggregate(vec![
                bound("key", reference(tuple_key)),
                bound("value", reference(integer)),
            ])),
        );
        assert_eq!(render_at(&graph, bad_key), Rendered::Unknown);
        for form in [
            BoundedDomainForm::IntegerRange,
            BoundedDomainForm::RationalRange,
            BoundedDomainForm::TextBounds,
            BoundedDomainForm::DecimalRange,
            BoundedDomainForm::FloatRounding,
        ] {
            let at = graph.push(CheckedNodeKind::BoundedDomain(form), None, None);
            assert_eq!(render_at(&graph, at), Rendered::Interim);
        }
        for form in [
            ScalarTypeForm::CarriedQuantity,
            ScalarTypeForm::BoundedQuantity,
            ScalarTypeForm::CompoundUnit,
        ] {
            let at = scalar(&mut graph, form);
            assert_eq!(render_at(&graph, at), Rendered::Interim);
        }
        let interim = graph.push(
            CheckedNodeKind::BoundedDomain(BoundedDomainForm::IntegerRange),
            None,
            None,
        );
        let of_range = wrapped(&mut graph, CompositeTypeForm::Set, interim);
        assert_eq!(render_at(&graph, of_range), Rendered::Interim);
    }

    /// The layer at `depth` collections down, not the outermost one.
    #[trace("TC-481", "FR-374-AC-3")]
    #[test]
    fn a_nested_collection_is_read_layer_by_layer() {
        let mut graph = Mock { nodes: vec![] };
        let integer = scalar(&mut graph, ScalarTypeForm::Integer);
        let inner = wrapped(&mut graph, CompositeTypeForm::Set, integer);
        let limits = graph.push(
            CheckedNodeKind::BoundedDomain(BoundedDomainForm::CollectionBounds),
            None,
            Some(aggregate(vec![
                bound("min", json!({"term": "literal", "type": {"package": "p", "path": ["0"], "ordinal": 0}, "value": "1", "value_kind": "integer"})),
                bound("max", json!({"term": "literal", "type": {"package": "p", "path": ["0"], "ordinal": 0}, "value": "2", "value_kind": "integer"})),
            ])),
        );
        graph.nodes[limits].dependencies = vec![id(inner)];
        let outer_body = aggregate(vec![reference(limits)]);
        let outer = graph.push(
            CheckedNodeKind::CompositeType(CompositeTypeForm::Sequence),
            None,
            Some(outer_body),
        );
        assert_eq!(
            collection_layer(&graph, outer, 0),
            Some(("sequence", "0", "unbounded"))
        );
        assert_eq!(collection_layer(&graph, outer, 1), Some(("set", "1", "2")));
        assert_eq!(collection_layer(&graph, outer, 2), None);
    }

    /// A source-owned node of any scalar, object or composite form renders
    /// as its declaration key; the same form without an owner is a native.
    #[trace("TC-481", "FR-374-AC-2")]
    #[test]
    fn every_source_owned_declaration_renders_as_its_key() {
        let mut graph = Mock { nodes: vec![] };
        let unit = graph.push(
            CheckedNodeKind::ScalarType(ScalarTypeForm::Unit),
            Some("ms"),
            None,
        );
        assert_eq!(render_at(&graph, unit), known("ix://I/ms"));
        let object = graph.push(
            CheckedNodeKind::Model(ModelForm::ObjectType),
            Some("O"),
            Some(aggregate(vec![])),
        );
        assert_eq!(render_at(&graph, object), known("ix://I/O"));
        let union = graph.push(
            CheckedNodeKind::CompositeType(CompositeTypeForm::Union),
            Some("U"),
            Some(aggregate(vec![])),
        );
        assert_eq!(render_at(&graph, union), known("ix://I/U"));
        let native = scalar(&mut graph, ScalarTypeForm::Unit);
        assert_eq!(render_at(&graph, native), Rendered::Unknown);
    }

    /// The step's kind picks the declaration: a field's declaring type is an
    /// object type or a record, a tuple step's a tuple, a union step's a union.
    #[trace("TC-481", "FR-374-AC-3")]
    #[test]
    fn a_declaring_type_is_found_by_the_kind_the_step_names() {
        let mut graph = Mock { nodes: vec![] };
        let record = graph.push(
            CheckedNodeKind::CompositeType(CompositeTypeForm::Record),
            Some("R"),
            Some(aggregate(vec![])),
        );
        let object = graph.push(
            CheckedNodeKind::Model(ModelForm::ObjectType),
            Some("O"),
            Some(aggregate(vec![])),
        );
        let tuple = graph.push(
            CheckedNodeKind::CompositeType(CompositeTypeForm::Tuple),
            Some("P"),
            Some(aggregate(vec![])),
        );
        let union = graph.push(
            CheckedNodeKind::CompositeType(CompositeTypeForm::Union),
            Some("U"),
            Some(aggregate(vec![])),
        );
        let found =
            |name: &str, step| declaration_node(&graph, "I", &format!("ix://I/{name}"), step);
        assert_eq!(found("R", DeclaringStep::Field), Some(record));
        assert_eq!(found("O", DeclaringStep::Field), Some(object));
        assert_eq!(found("P", DeclaringStep::Field), None);
        assert_eq!(found("U", DeclaringStep::Field), None);
        assert_eq!(found("P", DeclaringStep::Tuple), Some(tuple));
        assert_eq!(found("R", DeclaringStep::Tuple), None);
        assert_eq!(found("U", DeclaringStep::Union), Some(union));
        assert_eq!(found("P", DeclaringStep::Union), None);
        assert_eq!(found("Nope", DeclaringStep::Field), None);
    }

    /// An inner collection without its own bounds is unbounded, not the
    /// outer collection's.
    #[trace("TC-481", "FR-374-AC-3")]
    #[test]
    fn an_inner_collection_does_not_inherit_the_outer_bounds() {
        let mut graph = Mock { nodes: vec![] };
        let integer = scalar(&mut graph, ScalarTypeForm::Integer);
        let inner = wrapped(&mut graph, CompositeTypeForm::Set, integer);
        let outer = wrapped(&mut graph, CompositeTypeForm::Sequence, inner);
        let limits = graph.push(
            CheckedNodeKind::BoundedDomain(BoundedDomainForm::CollectionBounds),
            None,
            Some(aggregate(vec![
                bound("min", json!({"term": "literal", "type": {"package": "p", "path": ["0"], "ordinal": 0}, "value": "1", "value_kind": "integer"})),
                bound("max", json!({"term": "literal", "type": {"package": "p", "path": ["0"], "ordinal": 0}, "value": "2", "value_kind": "integer"})),
            ])),
        );
        graph.nodes[limits].dependencies = vec![id(outer)];
        assert_eq!(
            collection_layer(&graph, limits, 0),
            Some(("sequence", "1", "2"))
        );
        assert_eq!(
            collection_layer(&graph, limits, 1),
            Some(("set", "0", "unbounded"))
        );
    }
}
