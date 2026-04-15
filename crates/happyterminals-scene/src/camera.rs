//! Camera configuration for scene ownership.
//!
//! The camera is owned by the [`Scene`](crate::scene::Scene), not global.
//! [`CameraConfig`] is an enum to allow future camera types (free, FPS)
//! while currently supporting only orbit cameras.

use happyterminals_renderer::camera::OrbitCamera;

/// Camera configuration wrapping a concrete camera type.
///
/// Currently only supports [`OrbitCamera`]. Additional variants (free camera,
/// FPS camera) can be added in future versions without breaking existing code.
#[derive(Debug, Clone)]
pub enum CameraConfig {
    /// An orbit camera parameterized by spherical coordinates.
    Orbit(OrbitCamera),
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self::Orbit(OrbitCamera::default())
    }
}

impl From<OrbitCamera> for CameraConfig {
    fn from(cam: OrbitCamera) -> Self {
        Self::Orbit(cam)
    }
}
