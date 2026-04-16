//! Action types: typed action values and states for the input system.
//!
//! Actions are identified by string keys and typed as `Bool`, `Axis1D`, or
//! `Axis2D`. Each registered action owns reactive signals that are set when
//! input events match their bindings.

use std::time::Duration;

use glam::Vec2;
use happyterminals_core::Signal;

/// The value produced by an action binding when fired.
#[derive(Debug, Clone, PartialEq)]
pub enum ActionValue {
    /// A boolean action (pressed/released).
    Bool(bool),
    /// A single-axis action (scroll amount, 1D slider).
    Axis1D(f32),
    /// A two-axis action (mouse drag delta, joystick).
    Axis2D(Vec2),
}

/// The type of value an action produces, used during registration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionValueType {
    /// Boolean press/release.
    Bool,
    /// Single-axis float.
    Axis1D,
    /// Two-axis float vector.
    Axis2D,
}

/// The current state of an action in the input system.
#[derive(Debug, Clone, PartialEq)]
pub enum ActionState {
    /// Action is not active.
    Inactive,
    /// Action was just triggered this frame.
    JustPressed,
    /// Action has been held for the given duration.
    Held(Duration),
    /// Action was just released this frame.
    Released,
}

/// A registered action entry with reactive signals for state and axis values.
///
/// Created by `InputMap::register_action` and stored in the action registry.
/// Consumers read the signals to react to input changes.
pub struct ActionEntry {
    /// The type of value this action produces.
    pub value_type: ActionValueType,
    /// The current state of the action.
    pub state: Signal<ActionState>,
    /// Single-axis value signal (present for `Axis1D` actions).
    pub axis1d: Option<Signal<f32>>,
    /// Two-axis value signal (present for `Axis2D` actions).
    pub axis2d: Option<Signal<Vec2>>,
}

impl ActionEntry {
    /// Creates a new action entry with the appropriate signals for the given type.
    ///
    /// - `Bool`: state signal only.
    /// - `Axis1D`: state + axis1d signal.
    /// - `Axis2D`: state + axis2d signal.
    #[must_use]
    pub fn new(value_type: ActionValueType) -> Self {
        let state = Signal::new(ActionState::Inactive);
        let (axis1d, axis2d) = match value_type {
            ActionValueType::Bool => (None, None),
            ActionValueType::Axis1D => (Some(Signal::new(0.0_f32)), None),
            ActionValueType::Axis2D => (None, Some(Signal::new(Vec2::ZERO))),
        };
        Self {
            value_type,
            state,
            axis1d,
            axis2d,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use happyterminals_core::create_root;

    #[test]
    fn action_entry_bool_has_state_only() {
        let (_result, _owner) = create_root(|| {
            let entry = ActionEntry::new(ActionValueType::Bool);
            assert_eq!(entry.value_type, ActionValueType::Bool);
            assert_eq!(entry.state.untracked(), ActionState::Inactive);
            assert!(entry.axis1d.is_none());
            assert!(entry.axis2d.is_none());
        });
    }

    #[test]
    fn action_entry_axis1d_has_state_and_axis1d() {
        let (_result, _owner) = create_root(|| {
            let entry = ActionEntry::new(ActionValueType::Axis1D);
            assert_eq!(entry.value_type, ActionValueType::Axis1D);
            assert_eq!(entry.state.untracked(), ActionState::Inactive);
            assert!(entry.axis1d.is_some());
            assert!(entry.axis2d.is_none());
            let sig = entry.axis1d.as_ref().expect("axis1d should be Some");
            assert!((sig.untracked() - 0.0).abs() < f32::EPSILON);
        });
    }

    #[test]
    fn action_entry_axis2d_has_state_and_axis2d() {
        let (_result, _owner) = create_root(|| {
            let entry = ActionEntry::new(ActionValueType::Axis2D);
            assert_eq!(entry.value_type, ActionValueType::Axis2D);
            assert_eq!(entry.state.untracked(), ActionState::Inactive);
            assert!(entry.axis1d.is_none());
            assert!(entry.axis2d.is_some());
            let sig = entry.axis2d.as_ref().expect("axis2d should be Some");
            assert_eq!(sig.untracked(), Vec2::ZERO);
        });
    }

    #[test]
    fn action_value_variants() {
        let b = ActionValue::Bool(true);
        assert_eq!(b, ActionValue::Bool(true));

        let a1 = ActionValue::Axis1D(0.5);
        assert_eq!(a1, ActionValue::Axis1D(0.5));

        let a2 = ActionValue::Axis2D(Vec2::new(1.0, 2.0));
        assert_eq!(a2, ActionValue::Axis2D(Vec2::new(1.0, 2.0)));
    }

    #[test]
    fn action_state_variants() {
        assert_eq!(ActionState::Inactive, ActionState::Inactive);
        assert_eq!(ActionState::JustPressed, ActionState::JustPressed);
        assert_eq!(ActionState::Released, ActionState::Released);
        assert_eq!(
            ActionState::Held(Duration::from_millis(100)),
            ActionState::Held(Duration::from_millis(100))
        );
    }
}
