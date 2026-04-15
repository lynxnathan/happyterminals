//! Node builders: [`LayerBuilder`], [`CubeBuilder`], [`GroupBuilder`].
//!
//! Each builder uses the consuming-self pattern. `CubeBuilder` holds a reference
//! back to its parent `LayerBuilder` so that chaining `.cube().position(v).cube()`
//! automatically finalizes the first cube and starts a new one.

use std::collections::HashMap;

use glam::Vec3;
use happyterminals_core::Signal;
use happyterminals_pipeline::Pipeline;
use happyterminals_scene::node::{NodeId, NodeKind, SceneNode, Transform};
use happyterminals_scene::prop::PropValue;
/// Builder for a compositing layer (`NodeKind::Layer`).
///
/// Created by [`SceneBuilder::layer()`](crate::builder::SceneBuilder::layer).
pub struct LayerBuilder {
    z_order: i16,
    children: Vec<SceneNode>,
    pipeline: Option<Pipeline>,
}

impl LayerBuilder {
    /// Create a new layer builder with the given z-order.
    #[must_use]
    pub(crate) fn new(z_order: i16) -> Self {
        Self {
            z_order,
            children: Vec::new(),
            pipeline: None,
        }
    }

    /// Start building a cube child node.
    ///
    /// Returns a [`CubeBuilder`] that holds this layer as its parent.
    /// When the cube is finalized (via another `.cube()` call or
    /// `Into<LayerBuilder>` conversion), it pushes itself onto the
    /// layer's children.
    #[must_use]
    pub fn cube(self) -> CubeBuilder {
        CubeBuilder::new(self)
    }

    /// Start building a group child node.
    ///
    /// The closure receives a [`GroupBuilder`] and returns a `GroupBuilder`.
    /// The group is finalized and pushed as a child when the closure returns.
    #[must_use]
    pub fn group<R: Into<GroupBuilder>>(mut self, f: impl FnOnce(GroupBuilder) -> R) -> Self {
        let gb = GroupBuilder::new();
        let gb: GroupBuilder = f(gb).into();
        self.children.push(gb.into_node());
        self
    }

    /// Set an effect pipeline on this layer.
    #[must_use]
    pub fn pipeline(mut self, pipeline: Pipeline) -> Self {
        self.pipeline = Some(pipeline);
        self
    }

    /// Consume the builder and produce a [`SceneNode`].
    pub(crate) fn into_node(self) -> SceneNode {
        SceneNode {
            id: NodeId::next(),
            kind: NodeKind::Layer {
                z_order: self.z_order,
            },
            transform: Transform::default(),
            props: HashMap::new(),
            children: self.children,
            pipeline: self.pipeline,
        }
    }

    /// Push a finished child node.
    fn push_child(&mut self, node: SceneNode) {
        self.children.push(node);
    }
}

/// Builder for a cube primitive (`NodeKind::Cube`).
///
/// Holds a reference back to the parent [`LayerBuilder`]. When the cube is
/// finalized (by calling `.cube()` for a sibling or converting back to
/// `LayerBuilder`), the in-progress node is pushed onto the parent's children.
pub struct CubeBuilder {
    parent: LayerBuilder,
    transform: Transform,
    props: HashMap<String, PropValue>,
    pipeline: Option<Pipeline>,
}

impl CubeBuilder {
    fn new(parent: LayerBuilder) -> Self {
        Self {
            parent,
            transform: Transform::default(),
            props: HashMap::new(),
            pipeline: None,
        }
    }

    /// Set the cube's position.
    #[must_use]
    pub fn position(mut self, v: impl Into<Vec3>) -> Self {
        self.transform.position = v.into();
        self
    }

    /// Bind the cube's rotation to a reactive signal.
    ///
    /// Stores a `PropValue::Reactive` in the props map under `"rotation"`.
    #[must_use]
    pub fn rotation(mut self, sig: &Signal<f32>) -> Self {
        self.props.insert(
            "rotation".to_owned(),
            PropValue::Reactive(Box::new(sig.clone())),
        );
        self
    }

    /// Set the cube's rotation to a static value.
    #[must_use]
    pub fn rotation_static(mut self, v: impl Into<Vec3>) -> Self {
        self.transform.rotation = v.into();
        self
    }

    /// Set the cube's scale.
    #[must_use]
    pub fn scale(mut self, v: impl Into<Vec3>) -> Self {
        self.transform.scale = v.into();
        self
    }

    /// Finalize this cube and start a new sibling cube.
    ///
    /// The current cube is pushed onto the parent layer's children,
    /// and a fresh `CubeBuilder` is returned for the next cube.
    #[must_use]
    pub fn cube(self) -> CubeBuilder {
        let parent = self.finalize();
        CubeBuilder::new(parent)
    }

    /// Start building a group child on the parent layer.
    ///
    /// Finalizes this cube first, then delegates to [`LayerBuilder::group`].
    #[must_use]
    pub fn group<R: Into<GroupBuilder>>(self, f: impl FnOnce(GroupBuilder) -> R) -> LayerBuilder {
        let parent = self.finalize();
        parent.group(f)
    }

    /// Consume this builder, push the node to parent, return the parent.
    fn finalize(mut self) -> LayerBuilder {
        let node = SceneNode {
            id: NodeId::next(),
            kind: NodeKind::Cube,
            transform: self.transform,
            props: self.props,
            children: Vec::new(),
            pipeline: self.pipeline,
        };
        self.parent.push_child(node);
        self.parent
    }
}

/// Convert a `CubeBuilder` back into its parent `LayerBuilder`,
/// finalizing the in-progress cube.
impl From<CubeBuilder> for LayerBuilder {
    fn from(cb: CubeBuilder) -> Self {
        cb.finalize()
    }
}

/// Builder for a group node (`NodeKind::Group`).
pub struct GroupBuilder {
    children: Vec<SceneNode>,
}

impl GroupBuilder {
    fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    /// Start building a cube child inside this group.
    #[must_use]
    pub fn cube(self) -> GroupCubeBuilder {
        GroupCubeBuilder::new(self)
    }

    /// Consume the builder and produce a [`SceneNode`].
    fn into_node(self) -> SceneNode {
        SceneNode {
            id: NodeId::next(),
            kind: NodeKind::Group,
            transform: Transform::default(),
            props: HashMap::new(),
            children: self.children,
            pipeline: None,
        }
    }

    /// Push a finished child node.
    fn push_child(&mut self, node: SceneNode) {
        self.children.push(node);
    }
}

/// Builder for a cube inside a [`GroupBuilder`].
///
/// Same API as [`CubeBuilder`] but returns to `GroupBuilder` instead of
/// `LayerBuilder`.
pub struct GroupCubeBuilder {
    parent: GroupBuilder,
    transform: Transform,
    props: HashMap<String, PropValue>,
}

impl GroupCubeBuilder {
    fn new(parent: GroupBuilder) -> Self {
        Self {
            parent,
            transform: Transform::default(),
            props: HashMap::new(),
        }
    }

    /// Set the cube's position.
    #[must_use]
    pub fn position(mut self, v: impl Into<Vec3>) -> Self {
        self.transform.position = v.into();
        self
    }

    /// Bind the cube's rotation to a reactive signal.
    #[must_use]
    pub fn rotation(mut self, sig: &Signal<f32>) -> Self {
        self.props.insert(
            "rotation".to_owned(),
            PropValue::Reactive(Box::new(sig.clone())),
        );
        self
    }

    /// Set the cube's scale.
    #[must_use]
    pub fn scale(mut self, v: impl Into<Vec3>) -> Self {
        self.transform.scale = v.into();
        self
    }

    /// Finalize this cube and start a new sibling cube in the group.
    #[must_use]
    pub fn cube(self) -> GroupCubeBuilder {
        let parent = self.finalize();
        GroupCubeBuilder::new(parent)
    }

    /// Consume this builder, push the node to parent, return the parent.
    fn finalize(self) -> GroupBuilder {
        let node = SceneNode {
            id: NodeId::next(),
            kind: NodeKind::Cube,
            transform: self.transform,
            props: self.props,
            children: Vec::new(),
            pipeline: None,
        };
        let mut parent = self.parent;
        parent.push_child(node);
        parent
    }
}

/// Convert a `GroupCubeBuilder` back into its parent `GroupBuilder`,
/// finalizing the in-progress cube.
impl From<GroupCubeBuilder> for GroupBuilder {
    fn from(gcb: GroupCubeBuilder) -> Self {
        gcb.finalize()
    }
}
