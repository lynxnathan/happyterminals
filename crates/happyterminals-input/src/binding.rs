//! Binding types: map raw terminal events to actions.
//!
//! A [`Binding`] describes which terminal event triggers an action. Bindings
//! are stored inside [`crate::context::InputContext`] and resolved during
//! [`crate::input_map::InputMap::dispatch`].

use crossterm::event::{KeyCode, KeyModifiers, MouseButton};

/// Which drag axis to capture from mouse drag events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragAxis {
    /// Only horizontal (column) movement.
    Horizontal,
    /// Only vertical (row) movement.
    Vertical,
    /// Both axes.
    Both,
}

/// Which scroll direction to match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    /// Scroll wheel up.
    Up,
    /// Scroll wheel down.
    Down,
    /// Scroll wheel left (horizontal scroll).
    Left,
    /// Scroll wheel right (horizontal scroll).
    Right,
}

/// A binding that maps a raw terminal event to an action.
#[derive(Debug, Clone, PartialEq)]
pub enum Binding {
    /// A keyboard key press (no modifier requirement).
    Key(KeyCode),
    /// A keyboard key press with required modifiers (e.g., Ctrl+C).
    KeyWithModifier {
        /// The key code.
        key: KeyCode,
        /// Required modifier keys.
        modifier: KeyModifiers,
    },
    /// A mouse button press.
    MouseButton(MouseButton),
    /// A scroll wheel event.
    Scroll(ScrollDirection),
    /// A mouse drag event.
    Drag {
        /// Which mouse button must be held during the drag.
        button: MouseButton,
        /// Which axis/axes to capture.
        axis: DragAxis,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binding_key_construct() {
        let b = Binding::Key(KeyCode::Char('w'));
        assert_eq!(b, Binding::Key(KeyCode::Char('w')));
    }

    #[test]
    fn binding_key_with_modifier_construct() {
        let b = Binding::KeyWithModifier {
            key: KeyCode::Char('c'),
            modifier: KeyModifiers::CONTROL,
        };
        assert!(matches!(b, Binding::KeyWithModifier { .. }));
    }

    #[test]
    fn binding_drag_construct() {
        let b = Binding::Drag {
            button: MouseButton::Left,
            axis: DragAxis::Both,
        };
        assert_eq!(
            b,
            Binding::Drag {
                button: MouseButton::Left,
                axis: DragAxis::Both,
            }
        );
    }

    #[test]
    fn binding_scroll_construct() {
        let b = Binding::Scroll(ScrollDirection::Up);
        assert_eq!(b, Binding::Scroll(ScrollDirection::Up));
    }

    #[test]
    fn binding_mouse_button_construct() {
        let b = Binding::MouseButton(MouseButton::Right);
        assert_eq!(b, Binding::MouseButton(MouseButton::Right));
    }
}
