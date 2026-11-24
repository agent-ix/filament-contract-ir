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
    /// The recursive identity exceeds the supported structural depth.
    #[error("certificate node identity exceeds the maximum depth")]
    TooDeep,
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
        Self::validate_shape(&package, &path, ordinal, 0)?;
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
        depth: usize,
    ) -> Result<(), CertificateNodeIdError> {
        if depth > 256 {
            return Err(CertificateNodeIdError::TooDeep);
        }
        if package.is_empty() {
            return Err(CertificateNodeIdError::EmptyPackage);
        }
        if path.is_empty() {
            return Err(CertificateNodeIdError::EmptyPath);
        }
        if ordinal > MAX_CERTIFICATE_NODE_ORDINAL {
            return Err(CertificateNodeIdError::OrdinalOutOfRange);
        }
        for segment in path {
            if let CertificatePathSegment::Node(node) = segment {
                Self::validate_shape(&node.package, &node.path, node.ordinal, depth + 1)?;
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

    /// Copy an existing exact structural identity without changing its key.
    pub fn from_node_key(key: &NodeKey) -> Result<Self, CertificateNodeIdError> {
        let path = key
            .path()
            .into_iter()
            .map(|segment| match segment {
                PathSegment::Text(text) => Ok(CertificatePathSegment::Text(text)),
                PathSegment::Node(node) => Self::from_node_key(&node)
                    .map(|node| CertificatePathSegment::Node(Box::new(node))),
            })
            .collect::<Result<Vec<_>, _>>()?;
        Self::new(key.package(), path, key.ordinal())
    }

    /// Recover the exact structural key represented by this certificate identity.
    pub fn to_node_key(&self) -> Result<NodeKey, InvalidNodeKey> {
        let path = self
            .path
            .iter()
            .map(|segment| match segment {
                CertificatePathSegment::Text(text) => Ok(PathSegment::Text(text.clone())),
                CertificatePathSegment::Node(node) => node.to_node_key().map(PathSegment::Node),
            })
            .collect::<Result<Vec<_>, _>>()?;
        NodeKey::new(self.package.clone(), path, self.ordinal)
    }
}
