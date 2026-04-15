//! Scene node types: [`NodeId`], [`NodeKind`], [`SceneNode`], [`Transform`].

use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use glam::Vec3;
use happyterminals_pipeline::Pipeline;

use crate::prop::PropValue;

/// Monotonic counter for unique node IDs.
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// A unique identifier for a scene node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u64);

impl NodeId {
    /// Returns the next unique node ID.
    #[must_use]
    pub fn next() -> Self {
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeId({})", self.0)
    }
}

/// The kind of a scene node.
#[derive(Debug, Clone)]
pub enum NodeKind {
    /// A 3D cube primitive.
    Cube,
    /// A compositing layer with explicit z-order.
    Layer {
        /// Z-order: lower values render first (furthest back).
        z_order: i16,
    },
    /// A grouping node for organizing children.
    Group,
    /// A user-defined custom node kind.
    Custom(String),
}

/// A 3D transform: position, rotation (Euler angles), and scale.
#[derive(Debug, Clone)]
pub struct Transform {
    /// Translation in world space.
    pub position: Vec3,
    /// Euler rotation angles (radians).
    pub rotation: Vec3,
    /// Scale factors per axis.
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        }
    }
}

/// A type alias for the node property map.
pub type PropMap = HashMap<String, PropValue>;

/// A single node in the scene IR tree.
#[derive(Debug)]
pub struct SceneNode {
    /// Unique identifier for this node.
    pub id: NodeId,
    /// The kind of node (Cube, Layer, Group, Custom).
    pub kind: NodeKind,
    /// The node's local transform.
    pub transform: Transform,
    /// Type-erased property map.
    pub props: PropMap,
    /// Child nodes forming the tree.
    pub children: Vec<SceneNode>,
    /// Optional visual-effect pipeline attached to this node.
    pub pipeline: Option<Pipeline>,
}
