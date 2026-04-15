//! The run loop — entry point for the happyterminals backend.
//!
//! [`run`] drives a `tokio::select!` loop between a frame ticker and
//! `crossterm::EventStream`. Input events are propagated into [`InputSignals`]
//! so that the render callback (and any scene code) can observe them as
//! reactive signal reads.

use std::io;
use std::ops::Deref;

use crossterm::event::EventStream;
use crossterm::terminal::SetTitle;
use futures::StreamExt;
use ratatui::Terminal;
use ratatui_crossterm::CrosstermBackend;
use tokio::time::{interval, MissedTickBehavior};

use happyterminals_core::grid::Grid;
use happyterminals_core::Rect;

use crate::event::{is_quit_event, map_event, InputEvent, InputSignals};
use crate::frame_spec::FrameSpec;
use crate::guard::{install_panic_hook, TerminalGuard};

/// Runs the terminal event loop.
///
/// Creates a [`TerminalGuard`] on entry (RAII — terminal is restored on drop,
/// including panics). Drives a `tokio::select!` loop between:
///
/// - **Frame tick:** calls `render_fn`, copies the [`Grid`] into ratatui's
///   frame buffer, and lets ratatui diff + flush only changed cells.
/// - **Event stream:** maps crossterm events into [`InputEvent`] and writes
///   them into [`InputSignals`] so the render callback can observe them.
///
/// Ctrl+C breaks the loop. The guard drops on return, restoring the terminal.
///
/// # Errors
///
/// Returns an error if terminal acquisition fails or an I/O error occurs
/// during rendering.
pub async fn run<F>(mut render_fn: F, spec: FrameSpec) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(&mut Grid, &InputSignals),
{
    install_panic_hook();

    let _guard = TerminalGuard::acquire()?;

    // Best-effort window title
    if let Some(ref title) = spec.title {
        let _ = crossterm::execute!(io::stdout(), SetTitle(title.as_str()));
    }

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut events = EventStream::new();

    let mut tick = interval(spec.frame_duration());
    tick.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let (w, h) = crossterm::terminal::size()?;
    let input_signals = InputSignals::new(w, h);
    let mut grid = Grid::new(Rect::new(0, 0, w, h));

    loop {
        tokio::select! {
            _ = tick.tick() => {
                let _span = tracing::trace_span!("frame").entered();
                terminal.draw(|frame| {
                    grid.resize(frame.area());
                    render_fn(&mut grid, &input_signals);
                    *frame.buffer_mut() = grid.deref().clone();
                })?;
            }
            maybe_event = events.next() => {
                match maybe_event {
                    Some(Ok(ev)) => {
                        if let Some(input) = map_event(&ev) {
                            if is_quit_event(&input) {
                                break;
                            }
                            match &input {
                                InputEvent::Key { .. } => {
                                    input_signals.last_key.set(Some(input));
                                }
                                InputEvent::Mouse { .. } => {
                                    input_signals.last_mouse.set(Some(input));
                                }
                                InputEvent::Resize { width, height } => {
                                    input_signals.terminal_size.set((*width, *height));
                                    // Grid resizes on next tick via frame.area()
                                }
                                InputEvent::FocusGained => {
                                    input_signals.focused.set(true);
                                }
                                InputEvent::FocusLost => {
                                    input_signals.focused.set(false);
                                }
                            }
                        }
                    }
                    Some(Err(_)) | None => break,
                }
            }
        }
    }

    Ok(())
    // _guard drops here, restoring terminal
}
