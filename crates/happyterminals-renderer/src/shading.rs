//! ASCII shading ramp -- maps face normal dot light direction to a character.
//!
//! Implementation lands in Task 2.

use glam::Vec3;

/// Maps the dot product of a face normal and light direction to an ASCII character.
#[derive(Debug, Clone)]
pub struct ShadingRamp<'a> {
    /// The character ramp from darkest (index 0) to brightest.
    pub ramp: &'a [char],
    /// Normalized light direction.
    pub light_dir: Vec3,
}
