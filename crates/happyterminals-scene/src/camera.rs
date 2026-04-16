//! Camera configuration for scene ownership.
//!
//! The camera is owned by the [`Scene`](crate::scene::Scene), not global.
//! [`CameraConfig`] wraps a concrete camera type and delegates to its
//! [`Camera::view_matrix()`](happyterminals_renderer::camera::Camera::view_matrix)
//! implementation.

use glam::Mat4;
use happyterminals_renderer::camera::{FreeLookCamera, FpsCamera, OrbitCamera};

/// Camera configuration wrapping a concrete camera type.
///
/// Supports [`OrbitCamera`], [`FreeLookCamera`], and [`FpsCamera`].
#[derive(Debug, Clone)]
pub enum CameraConfig {
    /// An orbit camera parameterized by spherical coordinates.
    Orbit(OrbitCamera),
    /// A free-look (flight) camera with 6 degrees of freedom.
    FreeLook(FreeLookCamera),
    /// A first-person camera locked to a ground plane.
    Fps(FpsCamera),
}

impl CameraConfig {
    /// Compute the view matrix by delegating to the inner camera.
    #[must_use]
    pub fn view_matrix(&self) -> Mat4 {
        match self {
            Self::Orbit(cam) => cam.view_matrix(),
            Self::FreeLook(cam) => cam.view_matrix(),
            Self::Fps(cam) => cam.view_matrix(),
        }
    }

    /// Returns a mutable reference to the inner [`OrbitCamera`], if this is
    /// the `Orbit` variant.
    pub fn as_orbit_mut(&mut self) -> Option<&mut OrbitCamera> {
        match self {
            Self::Orbit(cam) => Some(cam),
            _ => None,
        }
    }

    /// Returns a mutable reference to the inner [`FreeLookCamera`], if this is
    /// the `FreeLook` variant.
    pub fn as_freelook_mut(&mut self) -> Option<&mut FreeLookCamera> {
        match self {
            Self::FreeLook(cam) => Some(cam),
            _ => None,
        }
    }

    /// Returns a mutable reference to the inner [`FpsCamera`], if this is
    /// the `Fps` variant.
    pub fn as_fps_mut(&mut self) -> Option<&mut FpsCamera> {
        match self {
            Self::Fps(cam) => Some(cam),
            _ => None,
        }
    }
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

impl From<FreeLookCamera> for CameraConfig {
    fn from(cam: FreeLookCamera) -> Self {
        Self::FreeLook(cam)
    }
}

impl From<FpsCamera> for CameraConfig {
    fn from(cam: FpsCamera) -> Self {
        Self::Fps(cam)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_from_freelook() {
        let config = CameraConfig::from(FreeLookCamera::default());
        assert!(matches!(config, CameraConfig::FreeLook(_)));
    }

    #[test]
    fn config_from_fps() {
        let config = CameraConfig::from(FpsCamera::default());
        assert!(matches!(config, CameraConfig::Fps(_)));
    }

    #[test]
    fn config_view_matrix_delegates() {
        let configs = [
            CameraConfig::Orbit(OrbitCamera::default()),
            CameraConfig::FreeLook(FreeLookCamera::default()),
            CameraConfig::Fps(FpsCamera::default()),
        ];
        for config in &configs {
            let view = config.view_matrix();
            for col in 0..4 {
                let c = view.col(col);
                assert!(!c.x.is_nan(), "view_matrix produced NaN for {:?}", config);
                assert!(!c.y.is_nan(), "view_matrix produced NaN for {:?}", config);
                assert!(!c.z.is_nan(), "view_matrix produced NaN for {:?}", config);
                assert!(!c.w.is_nan(), "view_matrix produced NaN for {:?}", config);
            }
        }
    }

    #[test]
    fn config_as_orbit_mut_returns_some() {
        let mut config = CameraConfig::Orbit(OrbitCamera::default());
        assert!(config.as_orbit_mut().is_some());
    }

    #[test]
    fn config_as_orbit_mut_returns_none_for_freelook() {
        let mut config = CameraConfig::FreeLook(FreeLookCamera::default());
        assert!(config.as_orbit_mut().is_none());
    }
}
