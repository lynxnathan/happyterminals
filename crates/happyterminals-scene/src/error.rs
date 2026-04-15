//! Scene construction errors.

use crate::node::NodeId;

/// Errors that can occur during scene construction and validation.
#[derive(Debug, thiserror::Error)]
pub enum SceneError {
    /// The scene has no camera configured.
    #[error("missing camera: scene requires exactly one camera")]
    MissingCamera,

    /// The scene has no layers or nodes.
    #[error("empty scene: at least one layer with one node is required")]
    EmptyScene,

    /// Two or more nodes share the same ID.
    #[error("duplicate node ID {node_id:?}")]
    DuplicateId {
        /// The duplicated node ID.
        node_id: NodeId,
    },

    /// A signal binding on a node is invalid.
    #[error("invalid binding on node {node_id:?}, prop '{prop_name}': {reason}")]
    InvalidBinding {
        /// The node with the invalid binding.
        node_id: NodeId,
        /// The property name that has an invalid binding.
        prop_name: String,
        /// Description of why the binding is invalid.
        reason: String,
    },

    /// A property's runtime type does not match the expected type for the node kind.
    #[error("prop type mismatch on node {node_id:?}, prop '{prop_name}': expected {expected}")]
    PropTypeMismatch {
        /// The node with the mismatched prop.
        node_id: NodeId,
        /// The property name.
        prop_name: String,
        /// The expected type description.
        expected: String,
    },
}
