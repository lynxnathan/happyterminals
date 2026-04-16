//! Camera types producing view matrices for the 3D renderer.
//!
//! All camera types implement the [`Camera`] trait, which provides a single
//! [`view_matrix()`](Camera::view_matrix) method returning a `Mat4`.
//!
//! - [`OrbitCamera`] — spherical coordinates around a target point.
//! - [`FreeLookCamera`] — 6-DOF flight camera (yaw/pitch + free translation).
//! - [`FpsCamera`] — first-person camera locked to a ground plane (Y = `ground_y`).
//!
//! Camera structs are pure data -- they do not hold `Signal` references.
//! The `Renderer` (or scene loop) reads signals via `.untracked()` and writes
//! the values into the camera fields before calling `view_matrix()`.

use glam::{Mat4, Vec3};

/// Epsilon added/subtracted from PI/2 to prevent gimbal lock at poles.
const ELEVATION_EPSILON: f32 = 0.01;

/// Trait for camera types that produce a view matrix.
///
/// Implemented by [`OrbitCamera`], [`FreeLookCamera`], and [`FpsCamera`].
pub trait Camera {
    /// Compute the view matrix from the camera's current state.
    fn view_matrix(&self) -> Mat4;
}

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
    /// then uses `Mat4::look_at_rh` with Y-up. Elevation is clamped to
    /// avoid gimbal lock at the poles.
    #[must_use]
    pub fn view_matrix(&self) -> Mat4 {
        let elevation = self.elevation.clamp(
            -std::f32::consts::FRAC_PI_2 + ELEVATION_EPSILON,
            std::f32::consts::FRAC_PI_2 - ELEVATION_EPSILON,
        );
        let eye = self.target
            + Vec3::new(
                self.distance * elevation.cos() * self.azimuth.sin(),
                self.distance * elevation.sin(),
                self.distance * elevation.cos() * self.azimuth.cos(),
            );
        Mat4::look_at_rh(eye, self.target, Vec3::Y)
    }
}

impl Camera for OrbitCamera {
    fn view_matrix(&self) -> Mat4 {
        OrbitCamera::view_matrix(self)
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

/// Free-look (flight) camera with 6 degrees of freedom.
///
/// Uses yaw/pitch Euler angles to define orientation. Pitch is clamped to
/// avoid gimbal lock at the poles.
#[derive(Debug, Clone)]
pub struct FreeLookCamera {
    /// World-space position of the camera.
    pub position: Vec3,
    /// Yaw in radians. 0 = looking along -Z.
    pub yaw: f32,
    /// Pitch in radians. Clamped to avoid gimbal lock.
    pub pitch: f32,
    /// Movement speed (units per second).
    pub speed: f32,
}

impl FreeLookCamera {
    /// Compute the view matrix from position and yaw/pitch orientation.
    ///
    /// Pitch is clamped before computing the forward vector.
    #[must_use]
    pub fn view_matrix(&self) -> Mat4 {
        let pitch = self.pitch.clamp(
            -std::f32::consts::FRAC_PI_2 + ELEVATION_EPSILON,
            std::f32::consts::FRAC_PI_2 - ELEVATION_EPSILON,
        );
        let forward = Vec3::new(
            self.yaw.sin() * pitch.cos(),
            pitch.sin(),
            -self.yaw.cos() * pitch.cos(),
        );
        let target = self.position + forward;
        Mat4::look_at_rh(self.position, target, Vec3::Y)
    }

    /// Unit forward direction derived from yaw and pitch.
    #[must_use]
    pub fn forward(&self) -> Vec3 {
        let pitch = self.pitch.clamp(
            -std::f32::consts::FRAC_PI_2 + ELEVATION_EPSILON,
            std::f32::consts::FRAC_PI_2 - ELEVATION_EPSILON,
        );
        Vec3::new(
            self.yaw.sin() * pitch.cos(),
            pitch.sin(),
            -self.yaw.cos() * pitch.cos(),
        )
    }

    /// Unit right direction (forward x Y, normalized).
    #[must_use]
    pub fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize()
    }

    /// Translate the camera along its local axes.
    ///
    /// - `forward`: movement along the forward axis (positive = forward).
    /// - `strafe`: movement along the right axis (positive = right).
    /// - `up`: movement along the world Y axis.
    /// - `dt`: frame delta time in seconds.
    pub fn translate(&mut self, forward: f32, strafe: f32, up: f32, dt: f32) {
        self.position +=
            (self.forward() * forward + self.right() * strafe + Vec3::Y * up) * self.speed * dt;
    }
}

impl Camera for FreeLookCamera {
    fn view_matrix(&self) -> Mat4 {
        FreeLookCamera::view_matrix(self)
    }
}

impl Default for FreeLookCamera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 5.0),
            yaw: 0.0,
            pitch: 0.0,
            speed: 5.0,
        }
    }
}

/// First-person camera locked to a ground plane.
///
/// Movement is constrained to the XZ plane at `ground_y`. Pitch controls
/// where the camera looks vertically but does not affect position.
#[derive(Debug, Clone)]
pub struct FpsCamera {
    /// World-space position of the camera.
    pub position: Vec3,
    /// Yaw in radians. 0 = looking along -Z.
    pub yaw: f32,
    /// Pitch in radians. Clamped to avoid gimbal lock.
    pub pitch: f32,
    /// Movement speed (units per second).
    pub speed: f32,
    /// Y coordinate the camera is locked to.
    pub ground_y: f32,
}

impl FpsCamera {
    /// Compute the view matrix.
    ///
    /// Forward is computed on the XZ plane from yaw only; pitch offsets the
    /// look target vertically.
    #[must_use]
    pub fn view_matrix(&self) -> Mat4 {
        let pitch = self.pitch.clamp(
            -std::f32::consts::FRAC_PI_2 + ELEVATION_EPSILON,
            std::f32::consts::FRAC_PI_2 - ELEVATION_EPSILON,
        );
        let forward_xz = Vec3::new(self.yaw.sin(), 0.0, -self.yaw.cos()).normalize();
        let pitch_offset = Vec3::new(0.0, pitch.sin(), 0.0);
        let target = self.position + forward_xz + pitch_offset;
        Mat4::look_at_rh(self.position, target, Vec3::Y)
    }

    /// Move on the XZ plane, locking Y to `ground_y`.
    ///
    /// - `forward_amount`: movement along the forward axis on XZ.
    /// - `strafe_amount`: movement along the right axis on XZ.
    /// - `dt`: frame delta time in seconds.
    pub fn translate_xz(&mut self, forward_amount: f32, strafe_amount: f32, dt: f32) {
        let forward_xz = Vec3::new(self.yaw.sin(), 0.0, -self.yaw.cos()).normalize();
        let right = forward_xz.cross(Vec3::Y).normalize();
        self.position += (forward_xz * forward_amount + right * strafe_amount) * self.speed * dt;
        self.position.y = self.ground_y;
    }
}

impl Camera for FpsCamera {
    fn view_matrix(&self) -> Mat4 {
        FpsCamera::view_matrix(self)
    }
}

impl Default for FpsCamera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 1.6, 5.0),
            yaw: 0.0,
            pitch: 0.0,
            speed: 5.0,
            ground_y: 1.6,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    // ── OrbitCamera tests (existing + new) ──────────────────────────────

    #[test]
    fn default_camera_eye_at_positive_z() {
        let cam = OrbitCamera::default();
        let view = cam.view_matrix();
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
            elevation: FRAC_PI_2 * 0.5,
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

    #[test]
    fn orbit_elevation_clamp() {
        // Set elevation exactly to PI/2 (pole) -- clamping should prevent NaN
        let cam = OrbitCamera {
            azimuth: 0.0,
            elevation: FRAC_PI_2,
            distance: 5.0,
            target: Vec3::ZERO,
        };
        let view = cam.view_matrix();
        for col in 0..4 {
            let c = view.col(col);
            assert!(!c.x.is_nan(), "OrbitCamera at pole should not produce NaN");
            assert!(!c.y.is_nan(), "OrbitCamera at pole should not produce NaN");
            assert!(!c.z.is_nan(), "OrbitCamera at pole should not produce NaN");
            assert!(!c.w.is_nan(), "OrbitCamera at pole should not produce NaN");
        }
    }

    // ── Camera trait tests ──────────────────────────────────────────────

    #[test]
    fn camera_trait_orbit() {
        let cam = OrbitCamera::default();
        let dyn_cam: &dyn Camera = &cam;
        let view = dyn_cam.view_matrix();
        assert_ne!(view, Mat4::IDENTITY);
    }

    #[test]
    fn camera_trait_freelook() {
        let cam = FreeLookCamera::default();
        let dyn_cam: &dyn Camera = &cam;
        let view = dyn_cam.view_matrix();
        assert_ne!(view, Mat4::IDENTITY);
    }

    #[test]
    fn camera_trait_fps() {
        let cam = FpsCamera::default();
        let dyn_cam: &dyn Camera = &cam;
        let view = dyn_cam.view_matrix();
        assert_ne!(view, Mat4::IDENTITY);
    }

    // ── FreeLookCamera tests ────────────────────────────────────────────

    #[test]
    fn freelook_default_eye_at_positive_z() {
        let cam = FreeLookCamera::default();
        let view = cam.view_matrix();
        let inv = view.inverse();
        let eye = inv.col(3).truncate();
        assert!(
            (eye - Vec3::new(0.0, 0.0, 5.0)).length() < 0.01,
            "Default FreeLookCamera eye should be at (0,0,5), got {:?}",
            eye
        );
    }

    #[test]
    fn freelook_view_matrix_not_nan() {
        let cam = FreeLookCamera {
            yaw: 0.5,
            pitch: 0.3,
            ..FreeLookCamera::default()
        };
        let view = cam.view_matrix();
        for col in 0..4 {
            let c = view.col(col);
            assert!(!c.x.is_nan(), "FreeLookCamera view matrix has NaN");
            assert!(!c.y.is_nan(), "FreeLookCamera view matrix has NaN");
            assert!(!c.z.is_nan(), "FreeLookCamera view matrix has NaN");
            assert!(!c.w.is_nan(), "FreeLookCamera view matrix has NaN");
        }
    }

    #[test]
    fn freelook_forward_at_default() {
        let cam = FreeLookCamera::default();
        let fwd = cam.forward();
        // yaw=0, pitch=0 -> forward = (0, 0, -1)
        assert!(
            (fwd - Vec3::new(0.0, 0.0, -1.0)).length() < 0.01,
            "Default forward should be (0,0,-1), got {:?}",
            fwd
        );
    }

    #[test]
    fn freelook_translate_moves_position() {
        let mut cam = FreeLookCamera::default();
        let initial_z = cam.position.z;
        cam.translate(1.0, 0.0, 0.0, 1.0);
        // Forward is (0, 0, -1), so moving forward should decrease z
        assert!(
            cam.position.z < initial_z,
            "Translating forward should decrease z, got {} (was {})",
            cam.position.z,
            initial_z
        );
    }

    #[test]
    fn freelook_pitch_clamp() {
        let cam = FreeLookCamera {
            pitch: std::f32::consts::PI,
            ..FreeLookCamera::default()
        };
        let view = cam.view_matrix();
        for col in 0..4 {
            let c = view.col(col);
            assert!(!c.x.is_nan(), "FreeLookCamera extreme pitch should not produce NaN");
            assert!(!c.y.is_nan(), "FreeLookCamera extreme pitch should not produce NaN");
            assert!(!c.z.is_nan(), "FreeLookCamera extreme pitch should not produce NaN");
            assert!(!c.w.is_nan(), "FreeLookCamera extreme pitch should not produce NaN");
        }
    }

    // ── FpsCamera tests ─────────────────────────────────────────────────

    #[test]
    fn fps_ground_lock() {
        let mut cam = FpsCamera {
            ground_y: 0.0,
            position: Vec3::new(0.0, 0.0, 5.0),
            ..FpsCamera::default()
        };
        cam.translate_xz(1.0, 0.0, 1.0);
        assert!(
            (cam.position.y - 0.0).abs() < f32::EPSILON,
            "FpsCamera Y should be locked to ground_y=0.0, got {}",
            cam.position.y
        );
    }

    #[test]
    fn fps_pitch_clamp() {
        let cam = FpsCamera {
            pitch: std::f32::consts::PI,
            ..FpsCamera::default()
        };
        let view = cam.view_matrix();
        for col in 0..4 {
            let c = view.col(col);
            assert!(!c.x.is_nan(), "FpsCamera extreme pitch should not produce NaN");
            assert!(!c.y.is_nan(), "FpsCamera extreme pitch should not produce NaN");
            assert!(!c.z.is_nan(), "FpsCamera extreme pitch should not produce NaN");
            assert!(!c.w.is_nan(), "FpsCamera extreme pitch should not produce NaN");
        }
    }

    #[test]
    fn fps_default_position() {
        let cam = FpsCamera::default();
        assert!(
            (cam.position.y - cam.ground_y).abs() < f32::EPSILON,
            "Default FpsCamera position.y should equal ground_y"
        );
        assert!((cam.ground_y - 1.6).abs() < f32::EPSILON);
    }

    #[test]
    fn fps_translate_xz_stays_on_ground() {
        let mut cam = FpsCamera::default();
        cam.translate_xz(1.0, 0.5, 0.5);
        assert!(
            (cam.position.y - cam.ground_y).abs() < f32::EPSILON,
            "FpsCamera Y should remain at ground_y after translate_xz, got {}",
            cam.position.y
        );
    }
}
