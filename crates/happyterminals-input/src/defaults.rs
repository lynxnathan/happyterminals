//! Default viewer context with standard orbit/pan/zoom/quit bindings.
//!
//! Provides [`default_viewer_context`] which creates an [`InputContext`]
//! named "default" with bindings for orbit (left-drag), pan (middle-drag),
//! zoom (scroll), and quit (Ctrl+C / Q).

use crossterm::event::{KeyCode, KeyModifiers, MouseButton};

use crate::action::ActionValueType;
use crate::binding::{Binding, DragAxis, ScrollDirection};
use crate::context::InputContext;
use crate::input_map::InputMap;
use crate::modifier::InputModifier;

/// Creates the default viewer input context with standard bindings.
///
/// Bindings:
/// - `"orbit"` -> Left-drag (both axes), deadzone 1.0
/// - `"pan"` -> Middle-drag (both axes), deadzone 1.0
/// - `"zoom"` -> Scroll down (negated) + Scroll up
/// - `"quit"` -> Ctrl+C + Q
#[must_use]
pub fn default_viewer_context() -> InputContext {
    let mut ctx = InputContext::new("default");

    // Orbit: left-drag
    ctx.bind(
        "orbit",
        Binding::Drag {
            button: MouseButton::Left,
            axis: DragAxis::Both,
        },
        vec![InputModifier::Deadzone(1.0)],
    );

    // Pan: right-drag (raw pixels — callback applies zoom-aware scaling)
    // Deadzone(2.0) filters right-click micro-jitter
    ctx.bind(
        "pan",
        Binding::Drag {
            button: MouseButton::Right,
            axis: DragAxis::Both,
        },
        vec![InputModifier::Deadzone(2.0)],
    );

    // Pan: also middle-drag
    ctx.bind(
        "pan",
        Binding::Drag {
            button: MouseButton::Middle,
            axis: DragAxis::Both,
        },
        vec![InputModifier::Deadzone(2.0)],
    );

    // Zoom: scroll down (negated = zoom out) and scroll up (zoom in)
    ctx.bind(
        "zoom",
        Binding::Scroll(ScrollDirection::Down),
        vec![InputModifier::Negate],
    );
    ctx.bind(
        "zoom",
        Binding::Scroll(ScrollDirection::Up),
        vec![],
    );

    // Quit: Ctrl+C and Q
    ctx.bind(
        "quit",
        Binding::KeyWithModifier {
            key: KeyCode::Char('c'),
            modifier: KeyModifiers::CONTROL,
        },
        vec![],
    );
    ctx.bind("quit", Binding::Key(KeyCode::Char('q')), vec![]);

    ctx
}

/// Registers the default viewer actions in the given `InputMap`.
///
/// Registers: `"orbit"` (`Axis2D`), `"pan"` (`Axis2D`), `"zoom"` (`Axis1D`),
/// `"quit"` (`Bool`).
pub fn register_default_actions(input_map: &mut InputMap) {
    input_map.register_action("orbit", ActionValueType::Axis2D);
    input_map.register_action("pan", ActionValueType::Axis2D);
    input_map.register_action("zoom", ActionValueType::Axis1D);
    input_map.register_action("quit", ActionValueType::Bool);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{Event, KeyEvent, KeyEventKind, KeyEventState, MouseEvent, MouseEventKind};
    use glam::Vec2;
    use happyterminals_core::create_root;

    use crate::action::ActionState;
    use crate::drag::DragOutput;

    #[test]
    fn default_viewer_context_has_name_default() {
        let ctx = default_viewer_context();
        assert_eq!(ctx.name, "default");
    }

    #[test]
    fn default_viewer_context_has_orbit_binding() {
        let ctx = default_viewer_context();
        let drag = DragOutput {
            button: MouseButton::Left,
            delta: Vec2::new(5.0, 3.0),
        };
        let ev = Event::Mouse(MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: 15,
            row: 8,
            modifiers: KeyModifiers::NONE,
        });
        let fired = ctx.try_resolve(&ev, Some(&drag));
        assert!(fired.is_some());
        assert_eq!(fired.expect("should match").action_key, "orbit");
    }

    #[test]
    fn default_viewer_context_has_pan_binding() {
        let ctx = default_viewer_context();
        let drag = DragOutput {
            button: MouseButton::Middle,
            delta: Vec2::new(2.0, 1.0),
        };
        let ev = Event::Mouse(MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Middle),
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        });
        let fired = ctx.try_resolve(&ev, Some(&drag));
        assert!(fired.is_some());
        assert_eq!(fired.expect("should match").action_key, "pan");
    }

    #[test]
    fn default_viewer_context_has_quit_ctrl_c() {
        let ctx = default_viewer_context();
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        let fired = ctx.try_resolve(&ev, None);
        assert!(fired.is_some());
        assert_eq!(fired.expect("should match").action_key, "quit");
    }

    #[test]
    fn default_viewer_context_has_quit_q() {
        let ctx = default_viewer_context();
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        let fired = ctx.try_resolve(&ev, None);
        assert!(fired.is_some());
        assert_eq!(fired.expect("should match").action_key, "quit");
    }

    #[test]
    fn default_viewer_context_has_zoom_scroll_down() {
        let ctx = default_viewer_context();
        let ev = Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        });
        let fired = ctx.try_resolve(&ev, None);
        assert!(fired.is_some());
        assert_eq!(fired.expect("should match").action_key, "zoom");
    }

    #[test]
    fn default_viewer_context_has_zoom_scroll_up() {
        let ctx = default_viewer_context();
        let ev = Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        });
        let fired = ctx.try_resolve(&ev, None);
        assert!(fired.is_some());
        assert_eq!(fired.expect("should match").action_key, "zoom");
    }

    #[test]
    fn register_default_actions_creates_all_four() {
        let (_result, _owner) = create_root(|| {
            let mut map = InputMap::new();
            register_default_actions(&mut map);

            assert!(map.action_state("orbit").is_some());
            assert!(map.action_state("pan").is_some());
            assert!(map.action_state("zoom").is_some());
            assert!(map.action_state("quit").is_some());

            assert!(map.action_axis2d("orbit").is_some());
            assert!(map.action_axis2d("pan").is_some());
            assert!(map.action_axis1d("zoom").is_some());
        });
    }

    #[test]
    fn full_integration_dispatch_with_defaults() {
        let (_result, _owner) = create_root(|| {
            let mut map = InputMap::new();
            register_default_actions(&mut map);
            map.push_context(default_viewer_context());

            // Dispatch quit
            let ev = Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            });
            map.dispatch(&ev);

            let state = map.action_state("quit").expect("quit should exist");
            assert_eq!(state.untracked(), ActionState::JustPressed);
        });
    }
}
