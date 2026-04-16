//! `InputMap` dispatch engine with context stack resolution.
//!
//! The `InputMap` is the central hub of the input system. It holds a registry
//! of actions (each backed by reactive signals) and a stack of
//! [`InputContext`]s. When a crossterm event arrives via [`InputMap::dispatch`],
//! the engine walks the context stack top-down, fires the first matching
//! action, and sets the appropriate signal.

use std::collections::HashMap;
use std::time::Duration;

use crossterm::event::Event;
use glam::Vec2;
use happyterminals_core::Signal;

use crate::action::{ActionEntry, ActionState, ActionValue, ActionValueType};
use crate::binding::Binding;
use crate::context::{FiredAction, InputContext};
use crate::drag::DragStateMachine;
use crate::modifier::apply_chain;

/// The central input dispatch engine.
///
/// Holds a registry of actions (each backed by reactive `Signal`s) and a
/// stack of [`InputContext`]s. When [`dispatch`](InputMap::dispatch) is
/// called with a crossterm event, the engine walks the context stack
/// top-down; the first context that matches fires the corresponding action
/// signal.
pub struct InputMap {
    /// Context stack: index 0 = bottom ("default"), last = top.
    contexts: Vec<InputContext>,
    /// Action registry: action key -> entry with signals.
    actions: HashMap<String, ActionEntry>,
    /// Drag state machine for tracking mouse drag gestures.
    drag_machine: DragStateMachine,
}

impl InputMap {
    /// Creates a new `InputMap` with empty context stack, empty action
    /// registry, and a fresh drag state machine.
    #[must_use]
    pub fn new() -> Self {
        Self {
            contexts: Vec::new(),
            actions: HashMap::new(),
            drag_machine: DragStateMachine::new(),
        }
    }

    /// Registers an action with the given key and value type.
    ///
    /// Creates an [`ActionEntry`] with the appropriate reactive signals
    /// and stores it in the action registry.
    pub fn register_action(&mut self, key: impl Into<String>, value_type: ActionValueType) {
        let key = key.into();
        self.actions.insert(key, ActionEntry::new(value_type));
    }

    /// Pushes a context onto the top of the context stack.
    ///
    /// The topmost context has highest priority during dispatch.
    pub fn push_context(&mut self, context: InputContext) {
        self.contexts.push(context);
    }

    /// Removes a named context from the stack.
    ///
    /// Returns `None` if the name matches the bottom context (index 0),
    /// which is protected from removal. Also returns `None` if the name
    /// is not found.
    pub fn pop_context(&mut self, name: &str) -> Option<InputContext> {
        let pos = self.contexts.iter().rposition(|c| c.name == name)?;
        if pos == 0 {
            return None; // protect bottom (default) context
        }
        Some(self.contexts.remove(pos))
    }

    /// Dispatches a crossterm event through the context stack.
    ///
    /// Phase 1: If the event is a mouse event, updates the drag state
    /// machine and captures any drag output.
    ///
    /// Phase 2: Walks contexts top-down (last to first). The first context
    /// that resolves the event fires the corresponding action signal.
    pub fn dispatch(&mut self, event: &Event) {
        // Phase 1: Update drag machine
        let drag_output = if let Event::Mouse(mouse) = event {
            self.drag_machine.update(mouse)
        } else {
            None
        };

        // Phase 2: Context stack walk (top-down)
        let fired = self
            .contexts
            .iter()
            .rev()
            .find_map(|ctx| ctx.try_resolve(event, drag_output.as_ref()));

        if let Some(fired) = fired {
            self.apply_action(&fired);
        }
    }

    /// Applies a fired action: runs the modifier chain and sets the
    /// appropriate signals on the action entry.
    fn apply_action(&self, fired: &FiredAction) {
        let Some(entry) = self.actions.get(&fired.action_key) else {
            return;
        };

        let value = apply_chain(&fired.modifiers, fired.raw_value.clone());

        match value {
            ActionValue::Bool(b) => {
                if b {
                    entry.state.set(ActionState::JustPressed);
                } else {
                    entry.state.set(ActionState::Released);
                }
            }
            ActionValue::Axis1D(v) => {
                if let Some(sig) = &entry.axis1d {
                    sig.set(v);
                }
                entry.state.set(ActionState::JustPressed);
            }
            ActionValue::Axis2D(v) => {
                if let Some(sig) = &entry.axis2d {
                    sig.set(v);
                }
                entry.state.set(ActionState::JustPressed);
            }
        }
    }

    /// Returns a reference to the state signal for the given action.
    #[must_use]
    pub fn action_state(&self, key: &str) -> Option<&Signal<ActionState>> {
        self.actions.get(key).map(|e| &e.state)
    }

    /// Returns a reference to the `axis1d` signal for the given action.
    #[must_use]
    pub fn action_axis1d(&self, key: &str) -> Option<&Signal<f32>> {
        self.actions.get(key).and_then(|e| e.axis1d.as_ref())
    }

    /// Returns a reference to the `axis2d` signal for the given action.
    #[must_use]
    pub fn action_axis2d(&self, key: &str) -> Option<&Signal<Vec2>> {
        self.actions.get(key).and_then(|e| e.axis2d.as_ref())
    }

    /// Transitions action states for a frame tick.
    ///
    /// - `JustPressed` -> `Held(Duration::ZERO)`
    /// - `Held(d)` -> `Held(d + dt)`
    /// - Other states are unchanged.
    pub fn tick_update(&mut self, dt: Duration) {
        for entry in self.actions.values() {
            let current = entry.state.untracked();
            match current {
                ActionState::JustPressed => {
                    entry.state.set(ActionState::Held(Duration::ZERO));
                }
                ActionState::Held(d) => {
                    entry.state.set(ActionState::Held(d + dt));
                }
                _ => {}
            }
        }
    }

    /// Resets all axis signals to zero.
    ///
    /// Called at the start of each tick before dispatch accumulates new
    /// deltas. Sets all `axis1d` signals to `0.0` and all `axis2d` signals
    /// to `Vec2::ZERO`.
    pub fn reset_axes(&self) {
        for entry in self.actions.values() {
            if let Some(sig) = &entry.axis1d {
                sig.set(0.0);
            }
            if let Some(sig) = &entry.axis2d {
                sig.set(Vec2::ZERO);
            }
        }
    }

    /// Replaces the binding for an action in the named context.
    ///
    /// Finds the context by name and calls `rebind` on it.
    pub fn rebind(&mut self, context_name: &str, action: &str, binding: Binding) {
        if let Some(ctx) = self.contexts.iter_mut().find(|c| c.name == context_name) {
            ctx.rebind(action, binding);
        }
    }
}

impl Default for InputMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{
        KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton,
        MouseEvent, MouseEventKind,
    };
    use happyterminals_core::create_root;

    use crate::binding::{DragAxis, ScrollDirection};
    use crate::modifier::InputModifier;

    fn make_key_event(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    fn make_mouse_event(kind: MouseEventKind, col: u16, row: u16) -> Event {
        Event::Mouse(MouseEvent {
            kind,
            column: col,
            row,
            modifiers: KeyModifiers::NONE,
        })
    }

    #[test]
    fn dispatch_key_event_fires_action() {
        let (_result, _owner) = create_root(|| {
            let mut map = InputMap::new();
            map.register_action("quit", ActionValueType::Bool);

            let mut ctx = InputContext::new("default");
            ctx.bind("quit", Binding::Key(KeyCode::Char('q')), vec![]);
            map.push_context(ctx);

            map.dispatch(&make_key_event(KeyCode::Char('q'), KeyModifiers::NONE));

            let state = map.action_state("quit").expect("quit action should exist");
            assert_eq!(state.untracked(), ActionState::JustPressed);
        });
    }

    #[test]
    fn dispatch_context_stack_top_wins() {
        let (_result, _owner) = create_root(|| {
            let mut map = InputMap::new();
            map.register_action("bottom_action", ActionValueType::Bool);
            map.register_action("top_action", ActionValueType::Bool);

            let mut bottom = InputContext::new("bottom");
            bottom.bind(
                "bottom_action",
                Binding::Key(KeyCode::Char('w')),
                vec![],
            );
            map.push_context(bottom);

            let mut top = InputContext::new("top");
            top.bind("top_action", Binding::Key(KeyCode::Char('w')), vec![]);
            map.push_context(top);

            map.dispatch(&make_key_event(KeyCode::Char('w'), KeyModifiers::NONE));

            // Top context should have fired
            let top_state = map
                .action_state("top_action")
                .expect("top_action should exist");
            assert_eq!(top_state.untracked(), ActionState::JustPressed);

            // Bottom context should NOT have fired
            let bottom_state = map
                .action_state("bottom_action")
                .expect("bottom_action should exist");
            assert_eq!(bottom_state.untracked(), ActionState::Inactive);
        });
    }

    #[test]
    fn dispatch_falls_through_to_bottom() {
        let (_result, _owner) = create_root(|| {
            let mut map = InputMap::new();
            map.register_action("quit", ActionValueType::Bool);

            let mut bottom = InputContext::new("default");
            bottom.bind("quit", Binding::Key(KeyCode::Char('q')), vec![]);
            map.push_context(bottom);

            // Top context does NOT bind 'q'
            let top = InputContext::new("overlay");
            map.push_context(top);

            map.dispatch(&make_key_event(KeyCode::Char('q'), KeyModifiers::NONE));

            let state = map.action_state("quit").expect("quit should exist");
            assert_eq!(state.untracked(), ActionState::JustPressed);
        });
    }

    #[test]
    fn pop_context_default_protected() {
        let mut map = InputMap::new();
        let ctx = InputContext::new("default");
        map.push_context(ctx);

        let result = map.pop_context("default");
        assert!(result.is_none());
        assert_eq!(map.contexts.len(), 1);
    }

    #[test]
    fn pop_context_non_default_works() {
        let mut map = InputMap::new();
        map.push_context(InputContext::new("default"));
        map.push_context(InputContext::new("overlay"));

        let result = map.pop_context("overlay");
        assert!(result.is_some());
        assert_eq!(result.expect("should be Some").name, "overlay");
        assert_eq!(map.contexts.len(), 1);
    }

    #[test]
    fn rebind_changes_active_binding() {
        let (_result, _owner) = create_root(|| {
            let mut map = InputMap::new();
            map.register_action("zoom", ActionValueType::Axis1D);

            let mut ctx = InputContext::new("default");
            ctx.bind(
                "zoom",
                Binding::Scroll(ScrollDirection::Up),
                vec![],
            );
            map.push_context(ctx);

            // Rebind zoom to Key('+')
            map.rebind("default", "zoom", Binding::Key(KeyCode::Char('+')));

            // Old binding should not fire
            map.dispatch(&make_mouse_event(MouseEventKind::ScrollUp, 0, 0));
            let axis = map
                .action_axis1d("zoom")
                .expect("zoom axis1d should exist");
            assert!((axis.untracked() - 0.0).abs() < f32::EPSILON);

            // New binding should fire
            map.dispatch(&make_key_event(KeyCode::Char('+'), KeyModifiers::NONE));
            let state = map.action_state("zoom").expect("zoom should exist");
            assert_eq!(state.untracked(), ActionState::JustPressed);
        });
    }

    #[test]
    fn drag_dispatch_sets_axis2d() {
        let (_result, _owner) = create_root(|| {
            let mut map = InputMap::new();
            map.register_action("orbit", ActionValueType::Axis2D);

            let mut ctx = InputContext::new("default");
            ctx.bind(
                "orbit",
                Binding::Drag {
                    button: MouseButton::Left,
                    axis: DragAxis::Both,
                },
                vec![],
            );
            map.push_context(ctx);

            // Mouse down at (10, 5)
            map.dispatch(&make_mouse_event(
                MouseEventKind::Down(MouseButton::Left),
                10,
                5,
            ));

            // Drag to (15, 8)
            map.dispatch(&make_mouse_event(
                MouseEventKind::Drag(MouseButton::Left),
                15,
                8,
            ));

            let axis = map
                .action_axis2d("orbit")
                .expect("orbit axis2d should exist");
            let val = axis.untracked();
            assert!((val.x - 5.0).abs() < f32::EPSILON);
            assert!((val.y - 3.0).abs() < f32::EPSILON);
        });
    }

    #[test]
    fn scroll_dispatch_sets_axis1d() {
        let (_result, _owner) = create_root(|| {
            let mut map = InputMap::new();
            map.register_action("zoom", ActionValueType::Axis1D);

            let mut ctx = InputContext::new("default");
            ctx.bind(
                "zoom",
                Binding::Scroll(ScrollDirection::Down),
                vec![InputModifier::Negate],
            );
            map.push_context(ctx);

            map.dispatch(&make_mouse_event(MouseEventKind::ScrollDown, 0, 0));

            let axis = map
                .action_axis1d("zoom")
                .expect("zoom axis1d should exist");
            // ScrollDown raw=1.0, Negate -> -1.0
            assert!((axis.untracked() - (-1.0)).abs() < f32::EPSILON);
        });
    }

    #[test]
    fn tick_update_transitions_just_pressed_to_held() {
        let (_result, _owner) = create_root(|| {
            let mut map = InputMap::new();
            map.register_action("test", ActionValueType::Bool);

            let mut ctx = InputContext::new("default");
            ctx.bind("test", Binding::Key(KeyCode::Char('t')), vec![]);
            map.push_context(ctx);

            map.dispatch(&make_key_event(KeyCode::Char('t'), KeyModifiers::NONE));
            assert_eq!(
                map.action_state("test").expect("exists").untracked(),
                ActionState::JustPressed
            );

            // First tick: JustPressed -> Held(ZERO)
            map.tick_update(Duration::from_millis(16));
            assert_eq!(
                map.action_state("test").expect("exists").untracked(),
                ActionState::Held(Duration::ZERO)
            );

            // Second tick: Held(ZERO) -> Held(16ms)
            map.tick_update(Duration::from_millis(16));
            assert_eq!(
                map.action_state("test").expect("exists").untracked(),
                ActionState::Held(Duration::from_millis(16))
            );
        });
    }

    #[test]
    fn reset_axes_zeros_signals() {
        let (_result, _owner) = create_root(|| {
            let mut map = InputMap::new();
            map.register_action("orbit", ActionValueType::Axis2D);
            map.register_action("zoom", ActionValueType::Axis1D);

            // Set some values
            if let Some(sig) = map.action_axis2d("orbit") {
                sig.set(Vec2::new(1.0, 2.0));
            }
            if let Some(sig) = map.action_axis1d("zoom") {
                sig.set(5.0);
            }

            map.reset_axes();

            assert_eq!(
                map.action_axis2d("orbit").expect("exists").untracked(),
                Vec2::ZERO
            );
            assert!(
                (map.action_axis1d("zoom").expect("exists").untracked() - 0.0).abs() < f32::EPSILON
            );
        });
    }

    #[test]
    fn action_accessors_return_none_for_missing() {
        let map = InputMap::new();
        assert!(map.action_state("nonexistent").is_none());
        assert!(map.action_axis1d("nonexistent").is_none());
        assert!(map.action_axis2d("nonexistent").is_none());
    }

    #[test]
    fn action_axis1d_returns_none_for_bool_type() {
        let (_result, _owner) = create_root(|| {
            let mut map = InputMap::new();
            map.register_action("test", ActionValueType::Bool);
            assert!(map.action_axis1d("test").is_none());
            assert!(map.action_axis2d("test").is_none());
        });
    }
}
