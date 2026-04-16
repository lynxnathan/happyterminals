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

use crate::color::ColorMode;

/// RAII guard that owns the terminal's raw-mode / alternate-screen state
/// and caches the resolved [`ColorMode`] for the lifetime of the run.
///
/// Created via [`TerminalGuard::acquire`] or [`TerminalGuard::acquire_with_color_mode`].
/// When dropped, the terminal is restored: raw mode disabled, alternate screen
/// left, cursor shown, mouse capture disabled, and an SGR reset (`ESC[0m`)
/// emitted to prevent color leakage into the shell.
pub struct TerminalGuard {
    stdout: Stdout,
    /// Cached color-mode resolved once at acquire-time. Read by the flush
    /// path to drive the downsample pass; never mutated after construction.
    pub color_mode: ColorMode,
    /// Test-only flag: when `true`, skip raw-mode/alt-screen restore on drop.
    /// Set exclusively by `with_mode_uninitialized_for_test`. Guarded at use
    /// sites with `#[cfg(test)]` so production `Drop` unconditionally restores.
    #[cfg(test)]
    uninitialized: bool,
}

impl TerminalGuard {
    /// Enters raw mode, alternate screen, enables mouse capture, and hides the
    /// cursor. Returns the guard that will undo all of this on drop.
    ///
    /// Back-compat shim that caches [`ColorMode::TrueColor`]. Prefer
    /// [`TerminalGuard::acquire_with_color_mode`] for explicit mode selection.
    pub fn acquire() -> io::Result<Self> {
        Self::acquire_with_color_mode(ColorMode::TrueColor)
    }

    /// Enters raw mode, alternate screen, enables mouse capture, and hides the
    /// cursor. Caches the given [`ColorMode`] on the returned guard so the
    /// flush path can read it without re-detecting.
    pub fn acquire_with_color_mode(color_mode: ColorMode) -> io::Result<Self> {
        let mut stdout = io::stdout();
        terminal::enable_raw_mode()?;
        execute!(
            stdout,
            EnterAlternateScreen,
            EnableMouseCapture,
            cursor::Hide
        )?;
        Ok(Self {
            stdout,
            color_mode,
            #[cfg(test)]
            uninitialized: false,
        })
    }

    /// Test-only constructor that sets the cached `color_mode` field WITHOUT
    /// entering raw mode / alternate screen. `Drop` is a no-op for this
    /// variant so the test runner's terminal state is untouched.
    ///
    /// Used to verify field caching in CI (no TTY available).
    #[cfg(test)]
    pub(crate) fn with_mode_uninitialized_for_test(color_mode: ColorMode) -> Self {
        Self {
            stdout: io::stdout(),
            color_mode,
            uninitialized: true,
        }
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
        #[cfg(test)]
        {
            if self.uninitialized {
                return;
            }
        }
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

    #[test]
    fn test_guard_caches_color_mode() {
        // Uses `with_mode_uninitialized_for_test` to avoid TTY requirement.
        // The Drop impl is a no-op for this variant, so the test runner's
        // terminal state is untouched.
        let g = TerminalGuard::with_mode_uninitialized_for_test(ColorMode::Ansi16);
        assert_eq!(g.color_mode, ColorMode::Ansi16);
        drop(g);

        let g2 = TerminalGuard::with_mode_uninitialized_for_test(ColorMode::Mono);
        assert_eq!(g2.color_mode, ColorMode::Mono);

        let g3 = TerminalGuard::with_mode_uninitialized_for_test(ColorMode::TrueColor);
        assert_eq!(g3.color_mode, ColorMode::TrueColor);

        let g4 = TerminalGuard::with_mode_uninitialized_for_test(ColorMode::Palette256);
        assert_eq!(g4.color_mode, ColorMode::Palette256);
    }
}
