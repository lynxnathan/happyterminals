//! [`SceneGraph`] -- layer sorting and tree traversal utilities.

use crate::ir::SceneIr;
use crate::node::{NodeKind, SceneNode};

/// A view over a [`SceneIr`] that provides layer sorting and traversal.
#[derive(Debug)]
pub struct SceneGraph<'a> {
    ir: &'a SceneIr,
}

impl<'a> SceneGraph<'a> {
    /// Creates a new scene graph view over the given IR.
    #[must_use]
    pub fn new(ir: &'a SceneIr) -> Self {
        Self { ir }
    }

    /// Returns layers sorted by z-order ascending (lowest first = rendered
    /// first = furthest back). Non-layer root nodes are excluded.
    #[must_use]
    pub fn sorted_layers(&self) -> Vec<&'a SceneNode> {
        let mut layers: Vec<&SceneNode> = self
            .ir
            .nodes()
            .iter()
            .filter(|n| matches!(n.kind, NodeKind::Layer { .. }))
            .collect();
        layers.sort_by_key(|n| match n.kind {
            NodeKind::Layer { z_order } => z_order,
            _ => 0,
        });
        layers
    }

    /// Depth-first traversal of all nodes in the IR tree.
    ///
    /// The callback receives each node and its depth (0 for root children).
    pub fn walk<F: FnMut(&'a SceneNode, usize)>(&self, mut f: F) {
        fn walk_recursive<'b>(node: &'b SceneNode, depth: usize, f: &mut impl FnMut(&'b SceneNode, usize)) {
            f(node, depth);
            for child in &node.children {
                walk_recursive(child, depth + 1, f);
            }
        }
        for node in self.ir.nodes() {
            walk_recursive(node, 0, &mut f);
        }
    }
}
