//! Orbit camera converting spherical coordinates to a view matrix.
//!
//! Implementation lands in Task 2.

use glam::Vec3;

/// Orbit camera parameterized by spherical coordinates.
#[derive(Debug, Clone)]
pub struct OrbitCamera {
    /// Azimuth angle in radians (rotation around Y axis).
    pub azimuth: f32,
    /// Elevation angle in radians (rotation up from the XZ plane).
    pub elevation: f32,
    /// Distance from the target point.
    pub distance: f32,
    /// The point the camera looks at.
    pub target: Vec3,
}
