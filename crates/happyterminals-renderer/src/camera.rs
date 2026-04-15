//! Orbit camera converting spherical coordinates to a view matrix.
//!
//! The [`OrbitCamera`] uses spherical coordinates (azimuth, elevation, distance)
//! to position the eye around a target point. The view matrix is computed via
//! [`glam::Mat4::look_at_rh`].
//!
//! The camera struct is pure data -- it does not hold `Signal` references.
//! The `Renderer` (Plan 02) reads signals via `.untracked()` and writes the
//! values into the camera fields before calling [`OrbitCamera::view_matrix`].

use glam::{Mat4, Vec3};

/// Orbit camera parameterized by spherical coordinates.
///
/// - `azimuth`: rotation around the Y axis (0 = looking from +Z toward origin).
/// - `elevation`: rotation up from the XZ plane.
/// - `distance`: distance from the target.
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

impl OrbitCamera {
    /// Compute the view matrix from the current spherical coordinates.
    ///
    /// Converts (azimuth, elevation, distance) to a Cartesian eye position,
    /// then uses `Mat4::look_at_rh` with Y-up.
    #[must_use]
    pub fn view_matrix(&self) -> Mat4 {
        let eye = self.target
            + Vec3::new(
                self.distance * self.elevation.cos() * self.azimuth.sin(),
                self.distance * self.elevation.sin(),
                self.distance * self.elevation.cos() * self.azimuth.cos(),
            );
        Mat4::look_at_rh(eye, self.target, Vec3::Y)
    }
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            azimuth: 0.0,
            elevation: 0.0,
            distance: 5.0,
            target: Vec3::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    #[test]
    fn default_camera_eye_at_positive_z() {
        let cam = OrbitCamera::default();
        // azimuth=0, elevation=0, distance=5 -> eye at (0, 0, 5)
        let view = cam.view_matrix();
        // The view matrix inverse gives us the camera position
        let inv = view.inverse();
        let eye = inv.col(3).truncate();
        assert!(
            (eye.x).abs() < 0.001,
            "Eye X should be ~0, got {}",
            eye.x
        );
        assert!(
            (eye.y).abs() < 0.001,
            "Eye Y should be ~0, got {}",
            eye.y
        );
        assert!(
            (eye.z - 5.0).abs() < 0.001,
            "Eye Z should be ~5, got {}",
            eye.z
        );
    }

    #[test]
    fn view_matrix_is_not_nan() {
        let cam = OrbitCamera {
            azimuth: 0.5,
            elevation: 0.3,
            distance: 3.0,
            target: Vec3::ZERO,
        };
        let view = cam.view_matrix();
        for col in 0..4 {
            let c = view.col(col);
            assert!(!c.x.is_nan(), "View matrix has NaN");
            assert!(!c.y.is_nan(), "View matrix has NaN");
            assert!(!c.z.is_nan(), "View matrix has NaN");
            assert!(!c.w.is_nan(), "View matrix has NaN");
        }
    }

    #[test]
    fn view_matrix_is_not_identity_for_nontrivial_input() {
        let cam = OrbitCamera {
            azimuth: 0.5,
            elevation: 0.3,
            distance: 3.0,
            target: Vec3::ZERO,
        };
        let view = cam.view_matrix();
        assert_ne!(view, Mat4::IDENTITY, "View matrix should not be identity");
    }

    #[test]
    fn azimuth_pi_half_moves_eye_to_positive_x() {
        let cam = OrbitCamera {
            azimuth: FRAC_PI_2,
            elevation: 0.0,
            distance: 5.0,
            target: Vec3::ZERO,
        };
        let view = cam.view_matrix();
        let inv = view.inverse();
        let eye = inv.col(3).truncate();
        // At azimuth=PI/2, eye should be at roughly (+5, 0, 0)
        assert!(
            eye.x > 4.0,
            "Eye X should be ~5 at azimuth=PI/2, got {}",
            eye.x
        );
        assert!(
            eye.y.abs() < 0.001,
            "Eye Y should be ~0, got {}",
            eye.y
        );
        assert!(
            eye.z.abs() < 0.1,
            "Eye Z should be ~0 at azimuth=PI/2, got {}",
            eye.z
        );
    }

    #[test]
    fn default_values_are_correct() {
        let cam = OrbitCamera::default();
        assert!((cam.azimuth - 0.0).abs() < f32::EPSILON);
        assert!((cam.elevation - 0.0).abs() < f32::EPSILON);
        assert!((cam.distance - 5.0).abs() < f32::EPSILON);
        assert_eq!(cam.target, Vec3::ZERO);
    }

    #[test]
    fn elevation_moves_eye_upward() {
        let cam = OrbitCamera {
            azimuth: 0.0,
            elevation: FRAC_PI_2 * 0.5, // 45 degrees up
            distance: 5.0,
            target: Vec3::ZERO,
        };
        let view = cam.view_matrix();
        let inv = view.inverse();
        let eye = inv.col(3).truncate();
        assert!(
            eye.y > 1.0,
            "Eye Y should be positive with positive elevation, got {}",
            eye.y
        );
    }
}
