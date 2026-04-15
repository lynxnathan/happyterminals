//! # happyterminals-renderer
//!
//! Fresh ASCII 3D rasterizer for [happyterminals](https://github.com/lynxnathan/happyterminals):
//! perspective projection with configurable cell aspect ratio, reversed-Z buffer,
//! configurable ASCII shading ramp, OBJ/STL mesh loading, and particle infrastructure.
//!
//! Not a fork of any existing renderer -- see the "fresh implementation" decision
//! in `.eclusa/PROJECT.md` Key Decisions.
//!
//! ## Modules
//!
//! - [`projection`] -- Perspective projection with cell aspect ratio correction.
//! - [`cube`] -- Unit cube primitive (8 vertices, 12 triangles, 6 face normals).
//! - [`shading`] -- ASCII shading ramp mapping `NdotL` to characters.
//! - [`camera`] -- Orbit camera converting spherical coordinates to view matrix.

#![forbid(unsafe_code)]

pub mod camera;
pub mod cube;
pub mod projection;
pub mod shading;

pub use camera::OrbitCamera;
pub use cube::Cube;
pub use projection::Projection;
pub use shading::ShadingRamp;
