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

/// The rendering of the type node at `position`, `None` for a node whose
/// rendering the specification does not give. `tick` is called once per node
/// visited.
pub(super) fn render<G: TypeGraph + ?Sized, E>(
    graph: &G,
    mut position: usize,
    tick: &mut impl FnMut() -> Result<(), E>,
) -> Result<Option<String>, E> {
    let mut prefix = String::new();
    let mut closing = 0_usize;
    for _ in 0..=graph.size() {
        tick()?;
        let Some(kind) = graph.kind(position) else {
            return Ok(None);
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
            ) => match (graph.owner(position), graph.declaration(position)) {
                (Some(CheckedNodeOwner::Source { identity, .. }), Some(declaration)) => {
                    match declaration.qualified_name.as_slice() {
                        [name] => format!("ix://{identity}/{name}"),
                        _ => return Ok(None),
                    }
                }
                _ => return Ok(None),
            },
            CheckedNodeKind::ScalarType(form) => match native_name(form) {
                Some(name) if graph.owner(position).is_none() => {
                    format!("ix://filament/native/{name}")
                }
                _ => return Ok(None),
            },
            CheckedNodeKind::CompositeType(CompositeTypeForm::Reference) => {
                let Some(next) = element() else {
                    return Ok(None);
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
                    return Ok(None);
                };
                position = next;
                continue;
            }
            CheckedNodeKind::CompositeType(form) => {
                let wrapper = match form {
                    CompositeTypeForm::Option => "Option<",
                    CompositeTypeForm::Sequence => "Sequence<",
                    CompositeTypeForm::Set => "Set<",
                    CompositeTypeForm::Bag => "Bag<",
                    CompositeTypeForm::OrderedSet => "OrderedSet<",
                    _ => return Ok(None),
                };
                let Some(next) = element() else {
                    return Ok(None);
                };
                prefix.push_str(wrapper);
                closing += 1;
                position = next;
                continue;
            }
            _ => return Ok(None),
        };
        prefix.push_str(&leaf);
        prefix.extend(std::iter::repeat_n('>', closing));
        return Ok(Some(prefix));
    }
    Ok(None)
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
        _ => return None,
    })
}

/// The collection kind, minimum and maximum a data type node spells for the
/// `slot` step that follows its field: the kind of the collection node and the
/// bounds of its `collection_bounds` node, or `0` and `unbounded` without one.
pub(super) fn collection_layer<G: TypeGraph + ?Sized>(
    graph: &G,
    mut position: usize,
) -> Option<(&'static str, &str, &str)> {
    let mut bounds = None;
    for _ in 0..=graph.size() {
        match graph.kind(position)? {
            CheckedNodeKind::CompositeType(CompositeTypeForm::Option) => {
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
                let (lower, upper) = bounds.unwrap_or(("0", "unbounded"));
                return Some((name, lower, upper));
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
/// native record, tuple or union.
#[ix_trace_rs::string_edge]
pub(super) fn declaration_node<G: TypeGraph + ?Sized>(
    graph: &G,
    identity: &str,
    declaring: &str,
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
            && matches!(
                kind,
                CheckedNodeKind::Model(ModelForm::ObjectType)
                    | CheckedNodeKind::CompositeType(
                        CompositeTypeForm::Record
                            | CompositeTypeForm::Tuple
                            | CompositeTypeForm::Union
                    )
            )
    })
}
