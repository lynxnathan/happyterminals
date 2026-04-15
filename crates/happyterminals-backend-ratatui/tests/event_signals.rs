use crossterm::event::{KeyCode, KeyModifiers};
use happyterminals_backend_ratatui::{InputEvent, InputSignals};
use happyterminals_core::create_root;

#[test]
fn resize_event_updates_terminal_size_signal() {
    let _owner = create_root(|| {
        let signals = InputSignals::new(80, 24);
        assert_eq!(signals.terminal_size.untracked(), (80, 24));

        // Simulate resize
        signals.terminal_size.set((120, 40));
        assert_eq!(signals.terminal_size.untracked(), (120, 40));
    });
}

#[test]
fn key_event_observable_via_signal() {
    let _owner = create_root(|| {
        let signals = InputSignals::new(80, 24);
        assert!(signals.last_key.untracked().is_none());

        // Simulate key event written by run()
        signals.last_key.set(Some(InputEvent::Key {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
        }));
        let key = signals.last_key.untracked();
        assert!(key.is_some());
        assert!(matches!(
            key,
            Some(InputEvent::Key {
                code: KeyCode::Char('a'),
                ..
            })
        ));
    });
}

#[test]
fn focus_event_observable_via_signal() {
    let _owner = create_root(|| {
        let signals = InputSignals::new(80, 24);
        assert!(signals.focused.untracked()); // starts focused

        signals.focused.set(false); // FocusLost
        assert!(!signals.focused.untracked());

        signals.focused.set(true); // FocusGained
        assert!(signals.focused.untracked());
    });
}
