//! [`SceneIr`] -- the root container of the scene intermediate representation.

use crate::node::SceneNode;

/// The scene intermediate representation: a tree of [`SceneNode`]s.
///
/// Every front-end (Rust builder, JSON recipes, Python) produces a `SceneIr`.
/// The runtime walks the tree each frame to render visible nodes.
#[derive(Debug)]
pub struct SceneIr {
    children: Vec<SceneNode>,
}

impl SceneIr {
    /// Creates a new scene IR from a list of root children (typically layers).
    #[must_use]
    pub fn new(children: Vec<SceneNode>) -> Self {
        Self { children }
    }

    /// Returns a slice of the root children.
    #[must_use]
    pub fn nodes(&self) -> &[SceneNode] {
        &self.children
    }

    /// Returns a mutable reference to the root children.
    pub fn nodes_mut(&mut self) -> &mut Vec<SceneNode> {
        &mut self.children
    }
}
