// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Structural identities used by backend-provider certificates (FR-412).

use filament_exact::{InvalidNodeKey, NodeKey, PathSegment};
use serde::{Deserialize, Deserializer, Serialize};

/// Maximum ordinal admitted by the backend-provider certificate schema.
pub const MAX_CERTIFICATE_NODE_ORDINAL: u64 = 9_007_199_254_740_991;

/// A validated structural node identity for a backend-provider certificate.
///
/// Its wire form is exactly `{package, path, ordinal}`. Nested node identities
/// in `path` are represented by [`CertificatePathSegment::Node`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CertificateNodeId {
    package: String,
    path: Vec<CertificatePathSegment>,
    ordinal: u64,
}

impl Drop for CertificateNodeId {
    fn drop(&mut self) {
        let mut pending = Vec::new();
        drain_nested_nodes(&mut self.path, &mut pending);
        while let Some(mut node) = pending.pop() {
            drain_nested_nodes(&mut node.path, &mut pending);
        }
    }
}

fn drain_nested_nodes(
    path: &mut Vec<CertificatePathSegment>,
    pending: &mut Vec<CertificateNodeId>,
) {
    for segment in path.drain(..) {
        if let CertificatePathSegment::Node(node) = segment {
            pending.push(*node);
        }
    }
}

/// One segment in a certificate node identity path.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CertificatePathSegment {
    /// A literal path segment.
    Text(String),
    /// A complete nested structural identity.
    Node(Box<CertificateNodeId>),
}

/// Invalid certificate structural identity shape.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum CertificateNodeIdError {
    /// Package names must be nonempty.
    #[error("certificate node identity package must be nonempty")]
    EmptyPackage,
    /// Paths must contain at least one segment.
    #[error("certificate node identity path must be nonempty")]
    EmptyPath,
    /// Ordinals must be exactly representable by the schema's JSON number domain.
    #[error("certificate node identity ordinal exceeds the schema maximum")]
    OrdinalOutOfRange,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CertificateNodeIdWire {
    package: String,
    path: Vec<CertificatePathSegment>,
    ordinal: u64,
}

impl<'de> Deserialize<'de> for CertificateNodeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = CertificateNodeIdWire::deserialize(deserializer)?;
        Self::new(wire.package, wire.path, wire.ordinal).map_err(serde::de::Error::custom)
    }
}

impl CertificateNodeId {
    /// Construct a certificate identity, refusing empty fields or an out-of-range ordinal.
    pub fn new(
        package: impl Into<String>,
        path: Vec<CertificatePathSegment>,
        ordinal: u64,
    ) -> Result<Self, CertificateNodeIdError> {
        let package = package.into();
        Self::validate_shape(&package, &path, ordinal)?;
        Ok(Self {
            package,
            path,
            ordinal,
        })
    }

    fn validate_shape(
        package: &str,
        path: &[CertificatePathSegment],
        ordinal: u64,
    ) -> Result<(), CertificateNodeIdError> {
        if package.is_empty() {
            return Err(CertificateNodeIdError::EmptyPackage);
        }
        if path.is_empty() {
            return Err(CertificateNodeIdError::EmptyPath);
        }
        if ordinal > MAX_CERTIFICATE_NODE_ORDINAL {
            return Err(CertificateNodeIdError::OrdinalOutOfRange);
        }
        let mut pending = Vec::new();
        for segment in path {
            if let CertificatePathSegment::Node(node) = segment {
                pending.push(node.as_ref());
            }
        }
        while let Some(node) = pending.pop() {
            if node.package.is_empty() {
                return Err(CertificateNodeIdError::EmptyPackage);
            }
            if node.path.is_empty() {
                return Err(CertificateNodeIdError::EmptyPath);
            }
            if node.ordinal > MAX_CERTIFICATE_NODE_ORDINAL {
                return Err(CertificateNodeIdError::OrdinalOutOfRange);
            }
            for segment in &node.path {
                if let CertificatePathSegment::Node(child) = segment {
                    pending.push(child.as_ref());
                }
            }
        }
        Ok(())
    }

    /// Owning package name.
    pub fn package(&self) -> &str {
        &self.package
    }

    /// Structural path segments, in order.
    pub fn path(&self) -> &[CertificatePathSegment] {
        &self.path
    }

    /// Structural ordinal.
    pub fn ordinal(&self) -> u64 {
        self.ordinal
    }

    /// Read a certificate identity from JSON bytes without imposing a nesting-depth cap.
    pub fn from_json_slice(input: &[u8]) -> Result<Self, serde_json::Error> {
        let mut deserializer = serde_json::Deserializer::from_slice(input);
        deserializer.disable_recursion_limit();
        let value = Self::deserialize(serde_stacker::Deserializer::new(&mut deserializer))?;
        deserializer.end()?;
        Ok(value)
    }

    /// Write a certificate identity as JSON bytes without recursion-depth stack growth.
    pub fn to_json_vec(&self) -> Result<Vec<u8>, serde_json::Error> {
        let mut output = Vec::new();
        let mut serializer = serde_json::Serializer::new(&mut output);
        self.serialize(serde_stacker::Serializer::new(&mut serializer))?;
        Ok(output)
    }

    /// Copy an existing exact structural identity without changing its key.
    pub fn from_node_key(key: &NodeKey) -> Result<Self, CertificateNodeIdError> {
        struct Frame {
            key: NodeKey,
            segments: Vec<PathSegment>,
            next: usize,
            converted: Vec<CertificatePathSegment>,
        }

        impl Frame {
            fn new(key: NodeKey) -> Self {
                let segments = key.path();
                Self {
                    key,
                    segments,
                    next: 0,
                    converted: Vec::new(),
                }
            }
        }

        let mut frames = vec![Frame::new(key.clone())];
        loop {
            let finished = frames
                .last()
                .is_some_and(|frame| frame.next == frame.segments.len());
            if finished {
                let frame = frames.pop().expect("a conversion frame is present");
                let node = Self {
                    package: frame.key.package().to_owned(),
                    path: frame.converted,
                    ordinal: frame.key.ordinal(),
                };
                if let Some(parent) = frames.last_mut() {
                    parent
                        .converted
                        .push(CertificatePathSegment::Node(Box::new(node)));
                } else {
                    return Ok(node);
                }
                continue;
            }

            enum Action {
                Text(String),
                Node(NodeKey),
            }
            let action = {
                let frame = frames.last_mut().expect("a conversion frame is present");
                let segment = &frame.segments[frame.next];
                frame.next += 1;
                match segment {
                    PathSegment::Text(text) => Action::Text(text.clone()),
                    PathSegment::Node(child) => Action::Node(child.clone()),
                }
            };
            match action {
                Action::Text(text) => frames
                    .last_mut()
                    .expect("a conversion frame is present")
                    .converted
                    .push(CertificatePathSegment::Text(text)),
                Action::Node(child) => frames.push(Frame::new(child)),
            }
        }
    }

    /// Recover the exact structural key represented by this certificate identity.
    pub fn to_node_key(&self) -> Result<NodeKey, InvalidNodeKey> {
        struct Frame<'a> {
            node: &'a CertificateNodeId,
            next: usize,
            converted: Vec<PathSegment>,
        }

        let mut frames = vec![Frame {
            node: self,
            next: 0,
            converted: Vec::new(),
        }];
        loop {
            let finished = frames
                .last()
                .is_some_and(|frame| frame.next == frame.node.path.len());
            if finished {
                let frame = frames.pop().expect("a conversion frame is present");
                let key = NodeKey::new(
                    frame.node.package.clone(),
                    frame.converted,
                    frame.node.ordinal,
                )?;
                if let Some(parent) = frames.last_mut() {
                    parent.converted.push(PathSegment::Node(key));
                } else {
                    return Ok(key);
                }
                continue;
            }

            enum Action<'a> {
                Text(String),
                Node(&'a CertificateNodeId),
            }
            let action = {
                let frame = frames.last_mut().expect("a conversion frame is present");
                let segment = &frame.node.path[frame.next];
                frame.next += 1;
                match segment {
                    CertificatePathSegment::Text(text) => Action::Text(text.clone()),
                    CertificatePathSegment::Node(child) => Action::Node(child),
                }
            };
            match action {
                Action::Text(text) => frames
                    .last_mut()
                    .expect("a conversion frame is present")
                    .converted
                    .push(PathSegment::Text(text)),
                Action::Node(child) => frames.push(Frame {
                    node: child,
                    next: 0,
                    converted: Vec::new(),
                }),
            }
        }
    }
}
