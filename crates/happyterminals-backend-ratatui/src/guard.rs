//! `TerminalGuard` — RAII wrapper that enters raw mode + alternate screen on
//! creation and restores the terminal on `Drop` (including during panics).
//!
//! # Panic safety
//!
//! Call [`install_panic_hook`] before creating a guard. The hook runs
//! [`TerminalGuard::restore`] before the original panic handler, so the
//! terminal is always left in a usable state even if a panic unwinds.

use std::io::{self, Stdout, Write};

use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

/// RAII guard that owns the terminal's raw-mode / alternate-screen state.
///
/// Created via [`TerminalGuard::acquire`]. When dropped, the terminal is
/// restored: raw mode disabled, alternate screen left, cursor shown, mouse
/// capture disabled, and an SGR reset (`ESC[0m`) emitted to prevent color
/// leakage into the shell.
pub struct TerminalGuard {
    stdout: Stdout,
}

impl TerminalGuard {
    /// Enters raw mode, alternate screen, enables mouse capture, and hides the
    /// cursor. Returns the guard that will undo all of this on drop.
    pub fn acquire() -> io::Result<Self> {
        let mut stdout = io::stdout();
        terminal::enable_raw_mode()?;
        execute!(
            stdout,
            EnterAlternateScreen,
            EnableMouseCapture,
            cursor::Hide
        )?;
        Ok(Self { stdout })
    }

    /// Best-effort terminal restore. Every operation ignores errors so that
    /// cleanup always runs to completion — even during panics or when the
    /// terminal is already partially restored.
    ///
    /// Idempotent: safe to call multiple times (duplicate restores are harmless).
    pub fn restore(stdout: &mut Stdout) {
        let _ = execute!(stdout, cursor::Show, DisableMouseCapture, LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
        // SGR reset prevents color leakage into the shell after exit.
        let _ = stdout.write_all(b"\x1b[0m");
        let _ = stdout.flush();
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        Self::restore(&mut self.stdout);
    }
}

/// Installs a panic hook that restores the terminal before the original
/// handler runs. Call this once, early in `main()`, before creating any
/// `TerminalGuard`.
pub fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let mut stdout = io::stdout();
        TerminalGuard::restore(&mut stdout);
        original(info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restore_is_callable() {
        // Verify the restore code-path doesn't panic. We cannot test actual
        // terminal state in CI (no TTY), but the best-effort design means
        // every call returns successfully regardless of environment.
        let mut stdout = io::stdout();
        TerminalGuard::restore(&mut stdout);
    }

    #[test]
    fn test_install_panic_hook() {
        // Verify the hook can be installed without panic. We don't trigger
        // a panic here — that would abort the test runner.
        install_panic_hook();
    }
}
