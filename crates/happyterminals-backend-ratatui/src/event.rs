//! Input event mapping — translates `crossterm::event::Event` into
//! [`InputEvent`] for consumption by the signal system.

use crossterm::event::{
    Event as CrosstermEvent, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind,
};

/// Terminal input event, mapped from crossterm events.
///
/// This is the framework's own event type — decoupled from crossterm so that
/// consumers (signals, effects, scene graph) never depend on crossterm
/// directly.
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    /// A key press (only `KeyEventKind::Press`; repeats and releases are filtered).
    Key {
        /// Which key was pressed.
        code: KeyCode,
        /// Active modifier keys (Ctrl, Shift, Alt, etc.).
        modifiers: KeyModifiers,
    },
    /// A mouse event (click, scroll, move, drag).
    Mouse {
        /// The type of mouse action (button down/up, scroll, move, drag).
        kind: MouseEventKind,
        /// Zero-based column where the event occurred.
        column: u16,
        /// Zero-based row where the event occurred.
        row: u16,
        /// Active modifier keys during the mouse event.
        modifiers: KeyModifiers,
    },
    /// Terminal resize.
    Resize {
        /// New terminal width in columns.
        width: u16,
        /// New terminal height in rows.
        height: u16,
    },
    /// Terminal window gained focus.
    FocusGained,
    /// Terminal window lost focus.
    FocusLost,
}

/// Maps a raw crossterm event to our [`InputEvent`].
///
/// Returns `None` for events we don't handle (`Paste`, key repeats/releases,
/// and any future crossterm variants).
#[must_use]
pub fn map_event(ev: &CrosstermEvent) -> Option<InputEvent> {
    match *ev {
        CrosstermEvent::Key(key) if key.kind == KeyEventKind::Press => Some(InputEvent::Key {
            code: key.code,
            modifiers: key.modifiers,
        }),
        CrosstermEvent::Mouse(mouse) => Some(InputEvent::Mouse {
            kind: mouse.kind,
            column: mouse.column,
            row: mouse.row,
            modifiers: mouse.modifiers,
        }),
        CrosstermEvent::Resize(w, h) => Some(InputEvent::Resize {
            width: w,
            height: h,
        }),
        CrosstermEvent::FocusGained => Some(InputEvent::FocusGained),
        CrosstermEvent::FocusLost => Some(InputEvent::FocusLost),
        // Paste, key repeats/releases, and unknown variants are ignored.
        _ => None,
    }
}

/// Returns `true` if the event is Ctrl+C (the conventional quit signal).
#[must_use]
pub fn is_quit_event(ev: &InputEvent) -> bool {
    matches!(
        ev,
        InputEvent::Key {
            code: KeyCode::Char('c'),
            modifiers,
        } if modifiers.contains(KeyModifiers::CONTROL)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent, KeyEventState, MouseEvent};

    fn make_key_event(code: KeyCode, modifiers: KeyModifiers, kind: KeyEventKind) -> CrosstermEvent {
        CrosstermEvent::Key(KeyEvent {
            code,
            modifiers,
            kind,
            state: KeyEventState::NONE,
        })
    }

    #[test]
    fn test_map_key_event() {
        let ev = make_key_event(KeyCode::Char('a'), KeyModifiers::NONE, KeyEventKind::Press);
        let mapped = map_event(&ev);
        assert_eq!(
            mapped,
            Some(InputEvent::Key {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::NONE,
            })
        );
    }

    #[test]
    fn test_map_key_repeat_ignored() {
        let ev = make_key_event(KeyCode::Char('a'), KeyModifiers::NONE, KeyEventKind::Repeat);
        assert_eq!(map_event(&ev), None);
    }

    #[test]
    fn test_map_resize() {
        let ev = CrosstermEvent::Resize(80, 24);
        assert_eq!(
            map_event(&ev),
            Some(InputEvent::Resize {
                width: 80,
                height: 24,
            })
        );
    }

    #[test]
    fn test_map_focus() {
        assert_eq!(map_event(&CrosstermEvent::FocusGained), Some(InputEvent::FocusGained));
        assert_eq!(map_event(&CrosstermEvent::FocusLost), Some(InputEvent::FocusLost));
    }

    #[test]
    fn test_map_mouse() {
        let ev = CrosstermEvent::Mouse(MouseEvent {
            kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: 10,
            row: 5,
            modifiers: KeyModifiers::NONE,
        });
        let mapped = map_event(&ev);
        assert_eq!(
            mapped,
            Some(InputEvent::Mouse {
                kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
                column: 10,
                row: 5,
                modifiers: KeyModifiers::NONE,
            })
        );
    }

    #[test]
    fn test_is_quit_ctrl_c() {
        let ev = InputEvent::Key {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
        };
        assert!(is_quit_event(&ev));
    }

    #[test]
    fn test_is_quit_plain_c() {
        let ev = InputEvent::Key {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::NONE,
        };
        assert!(!is_quit_event(&ev));
    }

    #[test]
    fn test_is_quit_key_q() {
        let ev = InputEvent::Key {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
        };
        assert!(!is_quit_event(&ev));
    }
}
