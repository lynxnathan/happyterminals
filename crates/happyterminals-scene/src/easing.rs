//! Easing functions for transition progress interpolation.
//!
//! Each function maps `t` in `[0.0, 1.0]` to an output in `[0.0, 1.0]`.

/// Linear interpolation (identity).
pub fn linear(t: f32) -> f32 {
    todo!()
}

/// Cubic ease-in-out: slow start, fast middle, slow end.
pub fn ease_in_out(t: f32) -> f32 {
    todo!()
}

/// Cubic ease-out: fast start, decelerating to a stop.
pub fn ease_out_cubic(t: f32) -> f32 {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-6;

    #[test]
    fn linear_boundaries() {
        assert!((linear(0.0) - 0.0).abs() < EPSILON);
        assert!((linear(0.5) - 0.5).abs() < EPSILON);
        assert!((linear(1.0) - 1.0).abs() < EPSILON);
    }

    #[test]
    fn ease_in_out_boundaries() {
        assert!((ease_in_out(0.0) - 0.0).abs() < EPSILON);
        assert!((ease_in_out(1.0) - 1.0).abs() < EPSILON);
        assert!((ease_in_out(0.5) - 0.5).abs() < EPSILON);
    }

    #[test]
    fn ease_out_cubic_boundaries() {
        assert!((ease_out_cubic(0.0) - 0.0).abs() < EPSILON);
        assert!((ease_out_cubic(1.0) - 1.0).abs() < EPSILON);
    }

    #[test]
    fn ease_out_cubic_decelerating() {
        // Decelerating curve: at t=0.5, output should be > 0.5
        assert!(ease_out_cubic(0.5) > 0.5);
    }

    #[test]
    fn ease_in_out_monotonic() {
        let mut prev = 0.0;
        for i in 1..=100 {
            let t = i as f32 / 100.0;
            let v = ease_in_out(t);
            assert!(v >= prev, "ease_in_out must be monotonically increasing");
            prev = v;
        }
    }
}
