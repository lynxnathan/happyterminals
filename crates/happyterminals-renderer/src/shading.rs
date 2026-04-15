//! ASCII shading ramp -- maps face normal dot light direction to a character.
//!
//! The [`ShadingRamp`] maps the cosine of the angle between a surface normal
//! and the light direction to one of N characters, from darkest to brightest.
//! The default ramp uses a dotted progression so unlit faces remain visible
//! (instead of disappearing into whitespace) while lit faces fill in with
//! denser glyphs.

use glam::Vec3;

/// Default shading characters, from darkest to brightest.
///
/// Dotted progression — unlit faces render as `·` (middle dot, visible but
/// subtle) rather than blank space, so a cube's back face is still legible.
pub const DEFAULT_RAMP: &[char] = &['·', '.', ':', ';', '+', 'o', 'O', '●', '█'];

/// Maps the dot product of a face normal and light direction to an ASCII character.
#[derive(Debug, Clone)]
pub struct ShadingRamp<'ramp> {
    /// The character ramp from darkest (index 0) to brightest.
    pub ramp: &'ramp [char],
    /// Normalized light direction.
    pub light_dir: Vec3,
}

impl ShadingRamp<'_> {
    /// Shade a surface with the given outward-facing normal.
    ///
    /// Computes `NdotL = clamp(normal . light_dir, 0, 1)` and maps it
    /// to a ramp index. Normals facing away from the light get the darkest
    /// character (space by default).
    #[must_use]
    pub fn shade(&self, normal: Vec3) -> char {
        let ndotl = normal.dot(self.light_dir).clamp(0.0, 1.0);
        let max_idx = self.ramp.len().saturating_sub(1);
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let idx = (ndotl * max_idx as f32).round() as usize;
        self.ramp[idx.min(max_idx)]
    }
}

impl Default for ShadingRamp<'_> {
    fn default() -> Self {
        Self {
            ramp: DEFAULT_RAMP,
            light_dir: Vec3::new(1.0, 1.0, 1.0).normalize(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_ramp_is_dotted_progression() {
        assert_eq!(DEFAULT_RAMP.len(), 9);
        assert_eq!(
            DEFAULT_RAMP,
            &['·', '.', ':', ';', '+', 'o', 'O', '●', '█']
        );
    }

    #[test]
    fn shade_normal_facing_light_returns_bright() {
        let ramp = ShadingRamp::default();
        // Vec3::Y dotted with normalized (1,1,1) = 1/sqrt(3) ~ 0.577
        let ch = ramp.shade(Vec3::Y);
        // Should be in the brighter half, definitely not the darkest dot
        assert_ne!(ch, '·', "Normal facing light should not be darkest");
    }

    #[test]
    fn shade_normal_facing_away_returns_darkest() {
        let ramp = ShadingRamp::default();
        // -Y is mostly away from (1,1,1).normalize()
        let ch = ramp.shade(-Vec3::Y);
        assert_eq!(ch, '·', "Normal facing away from light should be darkest (middle dot)");
    }

    #[test]
    fn shade_directly_at_light_returns_brightest() {
        let ramp = ShadingRamp::default();
        let light = Vec3::new(1.0, 1.0, 1.0).normalize();
        let ch = ramp.shade(light);
        assert_eq!(ch, '█', "Normal aligned with light should be brightest (full block)");
    }

    #[test]
    fn shade_never_panics_for_any_normal() {
        let ramp = ShadingRamp::default();
        // Test various normals including edge cases
        let normals = [
            Vec3::X,
            Vec3::Y,
            Vec3::Z,
            -Vec3::X,
            -Vec3::Y,
            -Vec3::Z,
            Vec3::ZERO, // degenerate
            Vec3::new(1.0, 1.0, 1.0).normalize(),
            Vec3::new(-1.0, -1.0, -1.0).normalize(),
        ];
        for normal in &normals {
            let ch = ramp.shade(*normal);
            assert!(
                DEFAULT_RAMP.contains(&ch),
                "shade({normal}) returned '{ch}' which is not in the ramp"
            );
        }
    }

    #[test]
    fn custom_ramp_with_three_characters() {
        let custom = &['a', 'b', 'c'];
        let ramp = ShadingRamp {
            ramp: custom,
            light_dir: Vec3::Y,
        };
        // Normal aligned with light -> brightest
        assert_eq!(ramp.shade(Vec3::Y), 'c');
        // Normal perpendicular -> darkest (NdotL=0)
        assert_eq!(ramp.shade(Vec3::X), 'a');
        // Normal opposite -> darkest
        assert_eq!(ramp.shade(-Vec3::Y), 'a');
    }

    #[test]
    fn single_char_ramp_always_returns_that_char() {
        let ramp = ShadingRamp {
            ramp: &['X'],
            light_dir: Vec3::Y,
        };
        assert_eq!(ramp.shade(Vec3::Y), 'X');
        assert_eq!(ramp.shade(-Vec3::Y), 'X');
    }
}
