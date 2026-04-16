//! Drag state machine: tracks mouse drag gestures and produces deltas.
//!
//! The state machine transitions through `Idle -> Pressed -> Dragging` as
//! mouse events arrive. Each drag frame produces a [`DragOutput`] with the
//! delta since the last position.

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use glam::Vec2;

/// Current state of the drag state machine.
#[derive(Debug, Clone)]
pub enum DragState {
    /// No drag in progress.
    Idle,
    /// Mouse button pressed, waiting for first drag movement.
    Pressed {
        /// Which button was pressed.
        button: MouseButton,
        /// Column where the press occurred.
        start_col: u16,
        /// Row where the press occurred.
        start_row: u16,
    },
    /// Active drag in progress.
    Dragging {
        /// Which button is being held.
        button: MouseButton,
        /// Last known column position.
        last_col: u16,
        /// Last known row position.
        last_row: u16,
    },
}

/// Output from the drag state machine when a drag delta is produced.
#[derive(Debug, Clone)]
pub struct DragOutput {
    /// Which mouse button is being dragged.
    pub button: MouseButton,
    /// Delta movement since the last position.
    pub delta: Vec2,
}

/// Tracks mouse drag gestures and produces position deltas.
///
/// Feed mouse events via [`DragStateMachine::update`]. When a drag gesture
/// is active, it returns [`DragOutput`] with the movement delta.
pub struct DragStateMachine {
    state: DragState,
}

impl DragStateMachine {
    /// Creates a new drag state machine in the `Idle` state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: DragState::Idle,
        }
    }

    /// Processes a mouse event and returns a drag output if a drag delta
    /// was produced.
    ///
    /// State transitions:
    /// - `(Idle, Down(btn))` -> `Pressed`, returns `None`
    /// - `(Pressed, Drag(btn))` -> `Dragging`, returns `Some(delta from start)`
    /// - `(Dragging, Drag(btn))` -> `Dragging`, returns `Some(delta from last)`
    /// - `(*, Up(_))` -> `Idle`, returns `None`
    pub fn update(&mut self, mouse: &MouseEvent) -> Option<DragOutput> {
        match (&self.state, mouse.kind) {
            (DragState::Idle, MouseEventKind::Down(btn)) => {
                self.state = DragState::Pressed {
                    button: btn,
                    start_col: mouse.column,
                    start_row: mouse.row,
                };
                None
            }
            (
                DragState::Pressed {
                    button,
                    start_col,
                    start_row,
                },
                MouseEventKind::Drag(btn),
            ) if btn == *button => {
                let delta = Vec2::new(
                    f32::from(mouse.column) - f32::from(*start_col),
                    f32::from(mouse.row) - f32::from(*start_row),
                );
                let out_button = *button;
                self.state = DragState::Dragging {
                    button: out_button,
                    last_col: mouse.column,
                    last_row: mouse.row,
                };
                Some(DragOutput {
                    button: out_button,
                    delta,
                })
            }
            (
                DragState::Dragging {
                    button,
                    last_col,
                    last_row,
                },
                MouseEventKind::Drag(btn),
            ) if btn == *button => {
                let delta = Vec2::new(
                    f32::from(mouse.column) - f32::from(*last_col),
                    f32::from(mouse.row) - f32::from(*last_row),
                );
                let out_button = *button;
                self.state = DragState::Dragging {
                    button: out_button,
                    last_col: mouse.column,
                    last_row: mouse.row,
                };
                Some(DragOutput {
                    button: out_button,
                    delta,
                })
            }
            (_, MouseEventKind::Up(_)) => {
                self.state = DragState::Idle;
                None
            }
            _ => None,
        }
    }

    /// Returns the current drag state.
    #[must_use]
    pub fn state(&self) -> &DragState {
        &self.state
    }

    /// Resets the state machine to `Idle`.
    pub fn reset(&mut self) {
        self.state = DragState::Idle;
    }
}

impl Default for DragStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    fn mouse_event(kind: MouseEventKind, col: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind,
            column: col,
            row,
            modifiers: KeyModifiers::NONE,
        }
    }

    #[test]
    fn idle_to_pressed_on_down() {
        let mut sm = DragStateMachine::new();
        let result = sm.update(&mouse_event(MouseEventKind::Down(MouseButton::Left), 10, 5));
        assert!(result.is_none());
        assert!(matches!(
            sm.state(),
            DragState::Pressed {
                button: MouseButton::Left,
                start_col: 10,
                start_row: 5,
            }
        ));
    }

    #[test]
    fn pressed_to_dragging_on_drag() {
        let mut sm = DragStateMachine::new();
        sm.update(&mouse_event(MouseEventKind::Down(MouseButton::Left), 10, 5));
        let result = sm.update(&mouse_event(
            MouseEventKind::Drag(MouseButton::Left),
            13,
            7,
        ));
        let output = result.expect("should produce drag output");
        assert_eq!(output.button, MouseButton::Left);
        assert!((output.delta.x - 3.0).abs() < f32::EPSILON);
        assert!((output.delta.y - 2.0).abs() < f32::EPSILON);
        assert!(matches!(sm.state(), DragState::Dragging { .. }));
    }

    #[test]
    fn dragging_to_dragging_produces_delta() {
        let mut sm = DragStateMachine::new();
        sm.update(&mouse_event(MouseEventKind::Down(MouseButton::Left), 10, 5));
        sm.update(&mouse_event(
            MouseEventKind::Drag(MouseButton::Left),
            13,
            7,
        ));
        let result = sm.update(&mouse_event(
            MouseEventKind::Drag(MouseButton::Left),
            15,
            8,
        ));
        let output = result.expect("should produce drag output");
        assert!((output.delta.x - 2.0).abs() < f32::EPSILON);
        assert!((output.delta.y - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn drag_up_returns_to_idle() {
        let mut sm = DragStateMachine::new();
        sm.update(&mouse_event(MouseEventKind::Down(MouseButton::Left), 10, 5));
        sm.update(&mouse_event(
            MouseEventKind::Drag(MouseButton::Left),
            13,
            7,
        ));
        let result = sm.update(&mouse_event(MouseEventKind::Up(MouseButton::Left), 13, 7));
        assert!(result.is_none());
        assert!(matches!(sm.state(), DragState::Idle));
    }

    #[test]
    fn mismatched_button_ignored() {
        let mut sm = DragStateMachine::new();
        sm.update(&mouse_event(MouseEventKind::Down(MouseButton::Left), 10, 5));
        // Drag with Right button while Left is pressed -- should be ignored
        let result = sm.update(&mouse_event(
            MouseEventKind::Drag(MouseButton::Right),
            13,
            7,
        ));
        assert!(result.is_none());
        // State should still be Pressed
        assert!(matches!(sm.state(), DragState::Pressed { .. }));
    }

    #[test]
    fn reset_returns_to_idle() {
        let mut sm = DragStateMachine::new();
        sm.update(&mouse_event(MouseEventKind::Down(MouseButton::Left), 10, 5));
        sm.reset();
        assert!(matches!(sm.state(), DragState::Idle));
    }

    #[test]
    fn idle_ignores_drag_without_press() {
        let mut sm = DragStateMachine::new();
        let result = sm.update(&mouse_event(
            MouseEventKind::Drag(MouseButton::Left),
            10,
            5,
        ));
        assert!(result.is_none());
        assert!(matches!(sm.state(), DragState::Idle));
    }
}
