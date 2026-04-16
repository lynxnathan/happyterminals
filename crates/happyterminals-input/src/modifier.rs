//! Modifier pipeline: transform action values before they reach signals.
//!
//! Modifiers are applied left-to-right in a chain via [`apply_chain`]. Each
//! modifier transforms an [`ActionValue`] in place -- for example, negating
//! an axis, scaling it, or applying a deadzone threshold.

use glam::Vec2;

use crate::action::ActionValue;

/// A value modifier applied to an action's raw value before it reaches
/// the action's signal.
#[derive(Debug, Clone, PartialEq)]
pub enum InputModifier {
    /// Negates the value: Bool(!b), Axis1D(-v), Axis2D(-v).
    Negate,
    /// Scales axis values by the given factor. Bool values are unchanged.
    Scale(f32),
    /// Zeroes axis values whose magnitude is below the threshold. Bool
    /// values are unchanged.
    Deadzone(f32),
    /// Swaps X and Y for `Axis2D` values. Other types are unchanged.
    Swizzle,
}

impl InputModifier {
    /// Applies this modifier to the given action value, returning the
    /// transformed value.
    #[must_use]
    pub fn apply(&self, value: ActionValue) -> ActionValue {
        match self {
            Self::Negate => match value {
                ActionValue::Bool(b) => ActionValue::Bool(!b),
                ActionValue::Axis1D(v) => ActionValue::Axis1D(-v),
                ActionValue::Axis2D(v) => ActionValue::Axis2D(-v),
            },
            Self::Scale(s) => match value {
                ActionValue::Bool(_) => value,
                ActionValue::Axis1D(v) => ActionValue::Axis1D(v * s),
                ActionValue::Axis2D(v) => ActionValue::Axis2D(v * s),
            },
            Self::Deadzone(threshold) => match value {
                ActionValue::Bool(_) => value,
                ActionValue::Axis1D(v) => {
                    if v.abs() < *threshold {
                        ActionValue::Axis1D(0.0)
                    } else {
                        value
                    }
                }
                ActionValue::Axis2D(v) => {
                    if v.length() < *threshold {
                        ActionValue::Axis2D(Vec2::ZERO)
                    } else {
                        value
                    }
                }
            },
            Self::Swizzle => match value {
                ActionValue::Axis2D(v) => ActionValue::Axis2D(Vec2::new(v.y, v.x)),
                _ => value,
            },
        }
    }
}

/// Applies a chain of modifiers left-to-right, folding the value through
/// each modifier in sequence.
#[must_use]
pub fn apply_chain(modifiers: &[InputModifier], value: ActionValue) -> ActionValue {
    modifiers
        .iter()
        .fold(value, |val, modifier| modifier.apply(val))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn negate_bool() {
        assert_eq!(
            InputModifier::Negate.apply(ActionValue::Bool(true)),
            ActionValue::Bool(false)
        );
        assert_eq!(
            InputModifier::Negate.apply(ActionValue::Bool(false)),
            ActionValue::Bool(true)
        );
    }

    #[test]
    fn negate_axis1d() {
        assert_eq!(
            InputModifier::Negate.apply(ActionValue::Axis1D(1.0)),
            ActionValue::Axis1D(-1.0)
        );
    }

    #[test]
    fn negate_axis2d() {
        let result = InputModifier::Negate.apply(ActionValue::Axis2D(Vec2::new(1.0, -2.0)));
        assert_eq!(result, ActionValue::Axis2D(Vec2::new(-1.0, 2.0)));
    }

    #[test]
    fn scale_axis1d() {
        assert_eq!(
            InputModifier::Scale(2.0).apply(ActionValue::Axis1D(3.0)),
            ActionValue::Axis1D(6.0)
        );
    }

    #[test]
    fn scale_axis2d() {
        let result = InputModifier::Scale(0.5).apply(ActionValue::Axis2D(Vec2::new(4.0, 6.0)));
        assert_eq!(result, ActionValue::Axis2D(Vec2::new(2.0, 3.0)));
    }

    #[test]
    fn scale_bool_unchanged() {
        assert_eq!(
            InputModifier::Scale(2.0).apply(ActionValue::Bool(true)),
            ActionValue::Bool(true)
        );
    }

    #[test]
    fn deadzone_axis1d_below_threshold() {
        assert_eq!(
            InputModifier::Deadzone(0.5).apply(ActionValue::Axis1D(0.3)),
            ActionValue::Axis1D(0.0)
        );
    }

    #[test]
    fn deadzone_axis1d_above_threshold() {
        assert_eq!(
            InputModifier::Deadzone(0.5).apply(ActionValue::Axis1D(0.7)),
            ActionValue::Axis1D(0.7)
        );
    }

    #[test]
    fn deadzone_axis2d_below_threshold() {
        let result =
            InputModifier::Deadzone(0.5).apply(ActionValue::Axis2D(Vec2::new(0.3, 0.2)));
        assert_eq!(result, ActionValue::Axis2D(Vec2::ZERO));
    }

    #[test]
    fn deadzone_axis2d_above_threshold() {
        let result =
            InputModifier::Deadzone(0.5).apply(ActionValue::Axis2D(Vec2::new(1.0, 0.0)));
        assert_eq!(result, ActionValue::Axis2D(Vec2::new(1.0, 0.0)));
    }

    #[test]
    fn deadzone_bool_unchanged() {
        assert_eq!(
            InputModifier::Deadzone(0.5).apply(ActionValue::Bool(true)),
            ActionValue::Bool(true)
        );
    }

    #[test]
    fn swizzle_axis2d() {
        let result = InputModifier::Swizzle.apply(ActionValue::Axis2D(Vec2::new(1.0, 2.0)));
        assert_eq!(result, ActionValue::Axis2D(Vec2::new(2.0, 1.0)));
    }

    #[test]
    fn swizzle_axis1d_unchanged() {
        assert_eq!(
            InputModifier::Swizzle.apply(ActionValue::Axis1D(5.0)),
            ActionValue::Axis1D(5.0)
        );
    }

    #[test]
    fn apply_chain_left_to_right() {
        // [Scale(2.0), Negate] on Axis1D(1.0) -> 2.0 -> -2.0
        let result = apply_chain(
            &[InputModifier::Scale(2.0), InputModifier::Negate],
            ActionValue::Axis1D(1.0),
        );
        assert_eq!(result, ActionValue::Axis1D(-2.0));
    }

    #[test]
    fn apply_chain_empty() {
        let result = apply_chain(&[], ActionValue::Axis1D(5.0));
        assert_eq!(result, ActionValue::Axis1D(5.0));
    }

    proptest! {
        #[test]
        fn scale_axis1d_proptest(s in -100.0f32..100.0, v in -100.0f32..100.0) {
            let result = InputModifier::Scale(s).apply(ActionValue::Axis1D(v));
            if let ActionValue::Axis1D(r) = result {
                let expected = v * s;
                // Allow NaN == NaN for edge cases
                if expected.is_nan() {
                    prop_assert!(r.is_nan());
                } else {
                    prop_assert!((r - expected).abs() < 1e-4,
                        "Scale({s}) on Axis1D({v}) = {r}, expected {expected}");
                }
            } else {
                prop_assert!(false, "Scale should produce Axis1D");
            }
        }
    }
}
