//! Input context: named groups of action-to-binding mappings.
//!
//! An [`InputContext`] maps bindings to action keys. Multiple contexts are
//! stacked in [`crate::input_map::InputMap`] and resolved top-down -- the
//! first context that matches an event wins.

use crossterm::event::{Event, KeyEventKind, KeyModifiers, MouseEventKind};

use crate::action::ActionValue;
use crate::binding::{Binding, DragAxis, ScrollDirection};
use crate::drag::DragOutput;
use crate::modifier::InputModifier;

/// A binding entry within a context: maps a binding + modifiers to an action key.
#[derive(Debug, Clone)]
pub struct ContextBinding {
    /// The action key this binding fires.
    pub action_key: String,
    /// The raw event binding to match.
    pub binding: Binding,
    /// Modifiers to apply to the raw value before setting the signal.
    pub modifiers: Vec<InputModifier>,
}

/// The result of a successful binding resolution.
#[derive(Debug, Clone)]
pub struct FiredAction {
    /// The action key that matched.
    pub action_key: String,
    /// The raw value produced by the binding before modifiers.
    pub raw_value: ActionValue,
    /// The modifiers to apply to the raw value.
    pub modifiers: Vec<InputModifier>,
}

/// A named group of action-to-binding mappings.
///
/// Contexts are stacked in [`crate::input_map::InputMap`]. During dispatch,
/// contexts are walked top-down; the first context that matches the event
/// fires the corresponding action.
pub struct InputContext {
    /// The name of this context (e.g., "default", "`orbit_mode`").
    pub name: String,
    bindings: Vec<ContextBinding>,
}

impl InputContext {
    /// Creates a new empty context with the given name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bindings: Vec::new(),
        }
    }

    /// Adds a binding that maps the given event to an action key.
    ///
    /// Returns `&mut Self` for chaining.
    pub fn bind(
        &mut self,
        action: impl Into<String>,
        binding: Binding,
        modifiers: Vec<InputModifier>,
    ) -> &mut Self {
        self.bindings.push(ContextBinding {
            action_key: action.into(),
            binding,
            modifiers,
        });
        self
    }

    /// Attempts to resolve a crossterm event (and optional drag output) against
    /// this context's bindings.
    ///
    /// Returns `Some(FiredAction)` on the first matching binding, or `None`
    /// if no binding matches.
    #[must_use]
    pub fn try_resolve(
        &self,
        event: &Event,
        drag: Option<&DragOutput>,
    ) -> Option<FiredAction> {
        // Check drag output first (if present)
        if let Some(drag_out) = drag {
            for cb in &self.bindings {
                if let Binding::Drag { button, axis } = &cb.binding {
                    if *button == drag_out.button {
                        let raw_value = match axis {
                            DragAxis::Both => ActionValue::Axis2D(drag_out.delta),
                            DragAxis::Horizontal => {
                                ActionValue::Axis1D(drag_out.delta.x)
                            }
                            DragAxis::Vertical => {
                                ActionValue::Axis1D(drag_out.delta.y)
                            }
                        };
                        return Some(FiredAction {
                            action_key: cb.action_key.clone(),
                            raw_value,
                            modifiers: cb.modifiers.clone(),
                        });
                    }
                }
            }
        }

        // Then match the raw event
        match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                for cb in &self.bindings {
                    match &cb.binding {
                        Binding::Key(code) if *code == key_event.code
                            && key_event.modifiers == KeyModifiers::NONE =>
                        {
                            return Some(FiredAction {
                                action_key: cb.action_key.clone(),
                                raw_value: ActionValue::Bool(true),
                                modifiers: cb.modifiers.clone(),
                            });
                        }
                        Binding::KeyWithModifier { key, modifier }
                            if *key == key_event.code
                                && key_event.modifiers.contains(*modifier) =>
                        {
                            return Some(FiredAction {
                                action_key: cb.action_key.clone(),
                                raw_value: ActionValue::Bool(true),
                                modifiers: cb.modifiers.clone(),
                            });
                        }
                        _ => {}
                    }
                }
            }
            Event::Mouse(mouse_event) => {
                let scroll_dir = match mouse_event.kind {
                    MouseEventKind::ScrollUp => Some(ScrollDirection::Up),
                    MouseEventKind::ScrollDown => Some(ScrollDirection::Down),
                    MouseEventKind::ScrollLeft => Some(ScrollDirection::Left),
                    MouseEventKind::ScrollRight => Some(ScrollDirection::Right),
                    _ => None,
                };
                if let Some(dir) = scroll_dir {
                    for cb in &self.bindings {
                        if let Binding::Scroll(sd) = &cb.binding {
                            if *sd == dir {
                                return Some(FiredAction {
                                    action_key: cb.action_key.clone(),
                                    raw_value: ActionValue::Axis1D(1.0),
                                    modifiers: cb.modifiers.clone(),
                                });
                            }
                        }
                    }
                }
                // Mouse button press
                if let MouseEventKind::Down(btn) = mouse_event.kind {
                    for cb in &self.bindings {
                        if let Binding::MouseButton(mb) = &cb.binding {
                            if *mb == btn {
                                return Some(FiredAction {
                                    action_key: cb.action_key.clone(),
                                    raw_value: ActionValue::Bool(true),
                                    modifiers: cb.modifiers.clone(),
                                });
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        None
    }

    /// Replaces the binding for the first entry matching the given action key.
    ///
    /// If no entry matches, this is a no-op.
    pub fn rebind(&mut self, action: &str, binding: Binding) {
        if let Some(entry) = self.bindings.iter_mut().find(|cb| cb.action_key == action)
        {
            entry.binding = binding;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventState, MouseButton, MouseEvent};
    use glam::Vec2;

    fn key_event(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    fn scroll_event(kind: MouseEventKind) -> Event {
        Event::Mouse(MouseEvent {
            kind,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        })
    }

    #[test]
    fn context_new_has_name() {
        let ctx = InputContext::new("orbit_mode");
        assert_eq!(ctx.name, "orbit_mode");
    }

    #[test]
    fn try_resolve_key_matches() {
        let mut ctx = InputContext::new("test");
        ctx.bind("quit", Binding::Key(KeyCode::Char('q')), vec![]);

        let ev = key_event(KeyCode::Char('q'), KeyModifiers::NONE);
        let fired = ctx.try_resolve(&ev, None);
        assert!(fired.is_some());
        let fired = fired.expect("should match");
        assert_eq!(fired.action_key, "quit");
        assert_eq!(fired.raw_value, ActionValue::Bool(true));
    }

    #[test]
    fn try_resolve_key_no_match() {
        let mut ctx = InputContext::new("test");
        ctx.bind("quit", Binding::Key(KeyCode::Char('q')), vec![]);

        let ev = key_event(KeyCode::Char('w'), KeyModifiers::NONE);
        let fired = ctx.try_resolve(&ev, None);
        assert!(fired.is_none());
    }

    #[test]
    fn try_resolve_key_with_modifier() {
        let mut ctx = InputContext::new("test");
        ctx.bind(
            "quit",
            Binding::KeyWithModifier {
                key: KeyCode::Char('c'),
                modifier: KeyModifiers::CONTROL,
            },
            vec![],
        );

        let ev = key_event(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let fired = ctx.try_resolve(&ev, None);
        assert!(fired.is_some());
        assert_eq!(fired.expect("should match").action_key, "quit");
    }

    #[test]
    fn try_resolve_scroll() {
        let mut ctx = InputContext::new("test");
        ctx.bind(
            "zoom",
            Binding::Scroll(ScrollDirection::Up),
            vec![],
        );

        let ev = scroll_event(MouseEventKind::ScrollUp);
        let fired = ctx.try_resolve(&ev, None);
        assert!(fired.is_some());
        let fired = fired.expect("should match");
        assert_eq!(fired.action_key, "zoom");
        assert_eq!(fired.raw_value, ActionValue::Axis1D(1.0));
    }

    #[test]
    fn try_resolve_drag() {
        let mut ctx = InputContext::new("test");
        ctx.bind(
            "orbit",
            Binding::Drag {
                button: MouseButton::Left,
                axis: DragAxis::Both,
            },
            vec![InputModifier::Deadzone(1.0)],
        );

        let drag = DragOutput {
            button: MouseButton::Left,
            delta: Vec2::new(5.0, 3.0),
        };
        // Event doesn't matter for drag resolution, but we need one
        let ev = Event::Mouse(MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: 15,
            row: 8,
            modifiers: KeyModifiers::NONE,
        });
        let fired = ctx.try_resolve(&ev, Some(&drag));
        assert!(fired.is_some());
        let fired = fired.expect("should match");
        assert_eq!(fired.action_key, "orbit");
        assert_eq!(fired.raw_value, ActionValue::Axis2D(Vec2::new(5.0, 3.0)));
        assert_eq!(fired.modifiers.len(), 1);
    }

    #[test]
    fn rebind_changes_binding() {
        let mut ctx = InputContext::new("test");
        ctx.bind("quit", Binding::Key(KeyCode::Char('q')), vec![]);

        // Rebind quit to Escape
        ctx.rebind("quit", Binding::Key(KeyCode::Esc));

        // Old binding should no longer match
        let ev_old = key_event(KeyCode::Char('q'), KeyModifiers::NONE);
        assert!(ctx.try_resolve(&ev_old, None).is_none());

        // New binding should match
        let ev_new = key_event(KeyCode::Esc, KeyModifiers::NONE);
        let fired = ctx.try_resolve(&ev_new, None);
        assert!(fired.is_some());
        assert_eq!(fired.expect("should match").action_key, "quit");
    }

    #[test]
    fn try_resolve_drag_horizontal_axis() {
        let mut ctx = InputContext::new("test");
        ctx.bind(
            "pan_h",
            Binding::Drag {
                button: MouseButton::Left,
                axis: DragAxis::Horizontal,
            },
            vec![],
        );

        let drag = DragOutput {
            button: MouseButton::Left,
            delta: Vec2::new(5.0, 3.0),
        };
        let ev = Event::Mouse(MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        });
        let fired = ctx.try_resolve(&ev, Some(&drag)).expect("should match");
        assert_eq!(fired.raw_value, ActionValue::Axis1D(5.0));
    }
}
