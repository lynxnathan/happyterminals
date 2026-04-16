//! Color-mode pipeline — Phase 2.2 foundation.
//!
//! - [`ColorMode`] enum: `TrueColor` | `Palette256` | `Ansi16` | `Mono`.
//! - [`detect_color_mode`]: env-var cascade (`NO_COLOR` > override > `$COLORTERM` > `$TERM`).
//! - [`downsample`]: flush-time buffer transform.
//! - [`palette`] submodule: compile-time 256 palette + 256→16 LUT.
//!
//! Detection uses the [`EnvProvider`] trait so tests inject a `FakeEnv`
//! without calling `std::env::set_var` (see RESEARCH §Pitfall 4).

pub mod palette;

pub use palette::{nearest_256, PALETTE_16_LUT, PALETTE_256};

/// Terminal color capability tier.
///
/// - `TrueColor` — 24-bit RGB SGR escapes (`\e[38;2;r;g;b m`).
/// - `Palette256` — xterm 8-bit indexed (`\e[38;5;Nm`).
/// - `Ansi16` — 4-bit named system colors.
/// - `Mono` — no color; terminal default fg/bg; modifiers preserved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    /// 24-bit RGB color (truecolor).
    TrueColor,
    /// xterm 8-bit indexed palette (256 colors).
    Palette256,
    /// 4-bit named system colors (16-color baseline).
    Ansi16,
    /// No color; modifiers (bold/italic/underline/reverse) preserved.
    Mono,
}

/// Minimal env-read abstraction.
///
/// Used by [`detect_color_mode`] so tests can inject a fake env without
/// calling `std::env::set_var` — `set_var` races across parallel test threads
/// (see RESEARCH §Pitfall 4).
pub trait EnvProvider {
    /// Returns the env var value, or `None` if unset OR non-unicode.
    fn var(&self, key: &str) -> Option<String>;
}

/// Production impl — reads via `std::env::var`.
///
/// `VarError::NotUnicode` is treated as unset (pathological case; non-issue
/// on Linux/Windows per RESEARCH §Pattern 1 note).
pub struct RealEnv;

impl EnvProvider for RealEnv {
    fn var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}

/// Follows the cascade (highest priority first):
///
/// 1. `NO_COLOR` present AND non-empty → [`ColorMode::Mono`] (per no-color.org).
/// 2. `override_mode` present → forced.
/// 3. `$COLORTERM` equals `"truecolor"` or `"24bit"` (case-insensitive) → [`ColorMode::TrueColor`].
/// 4. `$TERM` inspection:
///    - `"dumb"` → [`ColorMode::Mono`]
///    - contains `"256color"` → [`ColorMode::Palette256`]
///    - `"xterm-kitty" | "alacritty"` → [`ColorMode::TrueColor`]
///    - anything else → [`ColorMode::Ansi16`]
///    - unset → [`ColorMode::Mono`]
///
/// `NO_COLOR=""` (empty string) is treated as UNSET per spec — the user did
/// NOT opt out of color.
#[must_use]
pub fn detect_color_mode<E: EnvProvider>(
    override_mode: Option<ColorMode>,
    env: &E,
) -> ColorMode {
    // 1. NO_COLOR (highest priority; beats override per no-color.org)
    if let Some(v) = env.var("NO_COLOR") {
        if !v.is_empty() {
            return ColorMode::Mono;
        }
    }
    // 2. Programmatic override
    if let Some(mode) = override_mode {
        return mode;
    }
    // 3. COLORTERM truecolor signal (case-insensitive)
    if let Some(v) = env.var("COLORTERM") {
        let lower = v.to_ascii_lowercase();
        if lower == "truecolor" || lower == "24bit" {
            return ColorMode::TrueColor;
        }
    }
    // 4. TERM fallback
    match env.var("TERM").as_deref() {
        // "dumb" explicitly opts out of color; unset TERM means piped /
        // non-TTY (e.g. output redirected to a log file) — in both cases
        // Mono is the safe default.
        Some("dumb") | None => ColorMode::Mono,
        Some(t) if t.contains("256color") => ColorMode::Palette256,
        Some("xterm-kitty" | "alacritty") => ColorMode::TrueColor,
        Some(_) => ColorMode::Ansi16,
    }
}

/// Convenience wrapper over [`detect_color_mode`] using [`RealEnv`].
#[must_use]
pub fn detect_color_mode_from_real_env(override_mode: Option<ColorMode>) -> ColorMode {
    detect_color_mode(override_mode, &RealEnv)
}

use ratatui::buffer::Buffer;
use ratatui::layout::Position;
use ratatui::style::Color;

/// Applies the color-mode transform to every cell in `buffer`.
///
/// [`ColorMode::TrueColor`] is a no-op fast path. Modifiers
/// (bold/italic/underline/reverse) are preserved across all modes.
///
/// - `Mono` — chromatic fg/bg → [`Color::Reset`] (NOT [`Color::Black`] —
///   see RESEARCH §Pitfall 7; preserves the user's terminal default).
/// - `Palette256` — `Color::Rgb` → `Color::Indexed(nearest_256)`.
/// - `Ansi16` — `Color::Rgb` and `Color::Indexed` → named 16 variant.
pub fn downsample(buffer: &mut Buffer, mode: ColorMode) {
    if matches!(mode, ColorMode::TrueColor) {
        return;
    }
    // Copy the Rect out so the iteration doesn't hold an immutable borrow
    // on `buffer` during `cell_mut` calls.
    let area = *buffer.area();
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buffer.cell_mut(Position::new(x, y)) {
                let fg_out = map_color(cell.fg, mode);
                let bg_out = map_color(cell.bg, mode);
                cell.set_fg(fg_out);
                cell.set_bg(bg_out);
                // modifier preserved — no touch
            }
        }
    }
}

/// Pure color remap for a single `Color` under a given `ColorMode`.
fn map_color(c: Color, mode: ColorMode) -> Color {
    match mode {
        ColorMode::TrueColor => c,
        // Pitfall 7: strip all chromatic colors (and keep existing Reset) to
        // Reset, NOT Color::Black, so the terminal's default fg/bg is honored
        // (important for users on light-theme terminals).
        ColorMode::Mono => Color::Reset,
        ColorMode::Palette256 => match c {
            Color::Rgb(r, g, b) => Color::Indexed(nearest_256((r, g, b))),
            other => other,
        },
        ColorMode::Ansi16 => match c {
            Color::Rgb(r, g, b) => {
                let idx16 = PALETTE_16_LUT[nearest_256((r, g, b)) as usize];
                named_from_u8(idx16)
            }
            Color::Indexed(i) => named_from_u8(PALETTE_16_LUT[i as usize]),
            other => other,
        },
    }
}

/// Maps a low-nibble index (0..16) to the corresponding named `Color` variant.
/// Out-of-range inputs default to `Color::White`.
fn named_from_u8(i: u8) -> Color {
    match i {
        0 => Color::Black,
        1 => Color::Red,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Blue,
        5 => Color::Magenta,
        6 => Color::Cyan,
        7 => Color::Gray,
        8 => Color::DarkGray,
        9 => Color::LightRed,
        10 => Color::LightGreen,
        11 => Color::LightYellow,
        12 => Color::LightBlue,
        13 => Color::LightMagenta,
        14 => Color::LightCyan,
        _ => Color::White,
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::cast_possible_truncation,
    clippy::similar_names
)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;
    use ratatui::style::Modifier;
    use std::collections::HashMap;

    // ----- FakeEnv test double (Pitfall 4: no std::env::set_var) -----

    struct FakeEnv {
        vars: HashMap<String, String>,
    }

    impl FakeEnv {
        fn new() -> Self {
            Self {
                vars: HashMap::new(),
            }
        }
        fn with(pairs: &[(&str, &str)]) -> Self {
            let mut e = Self::new();
            for (k, v) in pairs {
                e.vars.insert((*k).to_string(), (*v).to_string());
            }
            e
        }
    }

    impl EnvProvider for FakeEnv {
        fn var(&self, key: &str) -> Option<String> {
            self.vars.get(key).cloned()
        }
    }

    // ================ detection cascade tests ================

    #[test]
    fn no_color_1_returns_mono() {
        let env = FakeEnv::with(&[("NO_COLOR", "1")]);
        assert_eq!(detect_color_mode(None, &env), ColorMode::Mono);
    }

    #[test]
    fn no_color_zero_returns_mono() {
        // Spec: any non-empty value disables.
        let env = FakeEnv::with(&[("NO_COLOR", "0")]);
        assert_eq!(detect_color_mode(None, &env), ColorMode::Mono);
    }

    #[test]
    fn no_color_whitespace_returns_mono() {
        let env = FakeEnv::with(&[("NO_COLOR", " ")]);
        assert_eq!(detect_color_mode(None, &env), ColorMode::Mono);
    }

    #[test]
    fn no_color_empty_is_unset() {
        // NO_COLOR="" → NOT disabled. Falls through to TERM.
        let env = FakeEnv::with(&[("NO_COLOR", ""), ("TERM", "xterm-256color")]);
        assert_eq!(detect_color_mode(None, &env), ColorMode::Palette256);
    }

    #[test]
    fn no_color_beats_override() {
        let env = FakeEnv::with(&[("NO_COLOR", "1")]);
        assert_eq!(
            detect_color_mode(Some(ColorMode::TrueColor), &env),
            ColorMode::Mono
        );
    }

    #[test]
    fn override_beats_colorterm() {
        let env = FakeEnv::with(&[("COLORTERM", "truecolor")]);
        assert_eq!(
            detect_color_mode(Some(ColorMode::Ansi16), &env),
            ColorMode::Ansi16
        );
    }

    #[test]
    fn colorterm_truecolor() {
        let env = FakeEnv::with(&[("COLORTERM", "truecolor")]);
        assert_eq!(detect_color_mode(None, &env), ColorMode::TrueColor);
    }

    #[test]
    fn colorterm_24bit() {
        let env = FakeEnv::with(&[("COLORTERM", "24bit")]);
        assert_eq!(detect_color_mode(None, &env), ColorMode::TrueColor);
    }

    #[test]
    fn colorterm_truecolor_uppercase() {
        let env = FakeEnv::with(&[("COLORTERM", "TRUECOLOR")]);
        assert_eq!(detect_color_mode(None, &env), ColorMode::TrueColor);
    }

    #[test]
    fn colorterm_1_not_truecolor() {
        // Legacy non-spec "yes colors" value; must NOT claim truecolor.
        let env = FakeEnv::with(&[("COLORTERM", "1"), ("TERM", "xterm-256color")]);
        assert_eq!(detect_color_mode(None, &env), ColorMode::Palette256);
    }

    #[test]
    fn term_256color_detected() {
        let env = FakeEnv::with(&[("TERM", "xterm-256color")]);
        assert_eq!(detect_color_mode(None, &env), ColorMode::Palette256);
    }

    #[test]
    fn term_tmux_256color_detected() {
        let env = FakeEnv::with(&[("TERM", "tmux-256color")]);
        assert_eq!(detect_color_mode(None, &env), ColorMode::Palette256);
    }

    #[test]
    fn term_kitty_is_truecolor() {
        let env = FakeEnv::with(&[("TERM", "xterm-kitty")]);
        assert_eq!(detect_color_mode(None, &env), ColorMode::TrueColor);
    }

    #[test]
    fn term_alacritty_is_truecolor() {
        let env = FakeEnv::with(&[("TERM", "alacritty")]);
        assert_eq!(detect_color_mode(None, &env), ColorMode::TrueColor);
    }

    #[test]
    fn term_dumb_is_mono() {
        let env = FakeEnv::with(&[("TERM", "dumb")]);
        assert_eq!(detect_color_mode(None, &env), ColorMode::Mono);
    }

    #[test]
    fn term_generic_is_ansi16() {
        let env = FakeEnv::with(&[("TERM", "xterm")]);
        assert_eq!(detect_color_mode(None, &env), ColorMode::Ansi16);
    }

    #[test]
    fn no_env_at_all_is_mono() {
        let env = FakeEnv::new();
        assert_eq!(detect_color_mode(None, &env), ColorMode::Mono);
    }

    // ================ downsample tests ================

    fn mk_buf() -> Buffer {
        Buffer::empty(Rect::new(0, 0, 2, 1))
    }

    #[test]
    fn downsample_truecolor_is_identity() {
        let mut buf = mk_buf();
        if let Some(cell) = buf.cell_mut(Position::new(0, 0)) {
            cell.set_fg(Color::Rgb(100, 200, 50));
            cell.set_bg(Color::Rgb(10, 20, 30));
        }
        downsample(&mut buf, ColorMode::TrueColor);
        let cell = buf.cell(Position::new(0, 0)).expect("cell exists");
        assert_eq!(cell.fg, Color::Rgb(100, 200, 50));
        assert_eq!(cell.bg, Color::Rgb(10, 20, 30));
    }

    #[test]
    fn downsample_mono_strips_rgb_to_reset() {
        let mut buf = mk_buf();
        if let Some(cell) = buf.cell_mut(Position::new(0, 0)) {
            cell.set_fg(Color::Rgb(100, 200, 50));
        }
        downsample(&mut buf, ColorMode::Mono);
        let cell = buf.cell(Position::new(0, 0)).expect("cell exists");
        assert_eq!(cell.fg, Color::Reset);
    }

    #[test]
    fn downsample_mono_strips_indexed_to_reset() {
        let mut buf = mk_buf();
        if let Some(cell) = buf.cell_mut(Position::new(0, 0)) {
            cell.set_bg(Color::Indexed(42));
        }
        downsample(&mut buf, ColorMode::Mono);
        let cell = buf.cell(Position::new(0, 0)).expect("cell exists");
        assert_eq!(cell.bg, Color::Reset);
    }

    #[test]
    fn downsample_mono_strips_named_to_reset() {
        let mut buf = mk_buf();
        if let Some(cell) = buf.cell_mut(Position::new(0, 0)) {
            cell.set_fg(Color::Red);
            cell.set_bg(Color::Blue);
        }
        downsample(&mut buf, ColorMode::Mono);
        let cell = buf.cell(Position::new(0, 0)).expect("cell exists");
        assert_eq!(cell.fg, Color::Reset);
        assert_eq!(cell.bg, Color::Reset);
    }

    #[test]
    fn downsample_mono_preserves_reset() {
        // Pitfall 7 regression gate: Reset stays Reset.
        let mut buf = mk_buf();
        if let Some(cell) = buf.cell_mut(Position::new(0, 0)) {
            cell.set_fg(Color::Reset);
        }
        downsample(&mut buf, ColorMode::Mono);
        let cell = buf.cell(Position::new(0, 0)).expect("cell exists");
        assert_eq!(cell.fg, Color::Reset);
    }

    #[test]
    fn downsample_mono_preserves_modifier_bold() {
        let mut buf = mk_buf();
        if let Some(cell) = buf.cell_mut(Position::new(0, 0)) {
            cell.set_fg(Color::Red);
            cell.modifier = Modifier::BOLD;
        }
        downsample(&mut buf, ColorMode::Mono);
        let cell = buf.cell(Position::new(0, 0)).expect("cell exists");
        assert_eq!(cell.fg, Color::Reset);
        assert!(cell.modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn downsample_mono_preserves_modifier_underline_reverse() {
        let mut buf = mk_buf();
        if let Some(cell) = buf.cell_mut(Position::new(0, 0)) {
            cell.set_fg(Color::Rgb(1, 2, 3));
            cell.modifier = Modifier::UNDERLINED | Modifier::REVERSED;
        }
        downsample(&mut buf, ColorMode::Mono);
        let cell = buf.cell(Position::new(0, 0)).expect("cell exists");
        assert!(cell.modifier.contains(Modifier::UNDERLINED));
        assert!(cell.modifier.contains(Modifier::REVERSED));
    }

    #[test]
    fn downsample_palette256_maps_rgb_to_indexed() {
        let mut buf = mk_buf();
        if let Some(cell) = buf.cell_mut(Position::new(0, 0)) {
            cell.set_fg(Color::Rgb(255, 255, 255));
        }
        downsample(&mut buf, ColorMode::Palette256);
        let cell = buf.cell(Position::new(0, 0)).expect("cell exists");
        let expected = Color::Indexed(nearest_256((255, 255, 255)));
        assert_eq!(cell.fg, expected);
    }

    #[test]
    fn downsample_palette256_passes_through_indexed() {
        let mut buf = mk_buf();
        if let Some(cell) = buf.cell_mut(Position::new(0, 0)) {
            cell.set_fg(Color::Indexed(42));
        }
        downsample(&mut buf, ColorMode::Palette256);
        let cell = buf.cell(Position::new(0, 0)).expect("cell exists");
        assert_eq!(cell.fg, Color::Indexed(42));
    }

    #[test]
    fn downsample_palette256_passes_through_named() {
        let mut buf = mk_buf();
        if let Some(cell) = buf.cell_mut(Position::new(0, 0)) {
            cell.set_fg(Color::Red);
        }
        downsample(&mut buf, ColorMode::Palette256);
        let cell = buf.cell(Position::new(0, 0)).expect("cell exists");
        assert_eq!(cell.fg, Color::Red);
    }

    fn is_named_16(c: Color) -> bool {
        matches!(
            c,
            Color::Black
                | Color::Red
                | Color::Green
                | Color::Yellow
                | Color::Blue
                | Color::Magenta
                | Color::Cyan
                | Color::Gray
                | Color::DarkGray
                | Color::LightRed
                | Color::LightGreen
                | Color::LightYellow
                | Color::LightBlue
                | Color::LightMagenta
                | Color::LightCyan
                | Color::White
                | Color::Reset // Reset passes through unchanged
        )
    }

    #[test]
    fn downsample_ansi16_rgb_becomes_named() {
        let mut buf = mk_buf();
        if let Some(cell) = buf.cell_mut(Position::new(0, 0)) {
            cell.set_fg(Color::Rgb(255, 0, 0));
        }
        downsample(&mut buf, ColorMode::Ansi16);
        let cell = buf.cell(Position::new(0, 0)).expect("cell exists");
        assert!(is_named_16(cell.fg), "got {:?}", cell.fg);
    }

    #[test]
    fn downsample_ansi16_indexed_becomes_named() {
        let mut buf = mk_buf();
        if let Some(cell) = buf.cell_mut(Position::new(0, 0)) {
            cell.set_fg(Color::Indexed(200));
        }
        downsample(&mut buf, ColorMode::Ansi16);
        let cell = buf.cell(Position::new(0, 0)).expect("cell exists");
        assert!(is_named_16(cell.fg), "got {:?}", cell.fg);
    }

    #[test]
    fn downsample_ansi16_closed_under_named() {
        // Start with all-named colors; post-downsample all still in 16-named set.
        let mut buf = Buffer::empty(Rect::new(0, 0, 4, 1));
        let names = [Color::Red, Color::Green, Color::Blue, Color::Yellow];
        for (i, c) in names.iter().enumerate() {
            if let Some(cell) = buf.cell_mut(Position::new(i as u16, 0)) {
                cell.set_fg(*c);
            }
        }
        downsample(&mut buf, ColorMode::Ansi16);
        for i in 0..4 {
            let cell = buf.cell(Position::new(i, 0)).expect("cell exists");
            assert!(is_named_16(cell.fg), "position {i}: got {:?}", cell.fg);
        }
    }

    // ================ helper tests ================

    #[test]
    fn named_from_u8_round_trip() {
        let expected = [
            Color::Black,
            Color::Red,
            Color::Green,
            Color::Yellow,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
            Color::Gray,
            Color::DarkGray,
            Color::LightRed,
            Color::LightGreen,
            Color::LightYellow,
            Color::LightBlue,
            Color::LightMagenta,
            Color::LightCyan,
            Color::White,
        ];
        for (i, c) in expected.iter().enumerate() {
            assert_eq!(named_from_u8(i as u8), *c, "index {i}");
        }
    }

    #[test]
    fn named_from_u8_out_of_range() {
        // RESEARCH §Pattern 4 fallback branch: out-of-range → White.
        assert_eq!(named_from_u8(99), Color::White);
        assert_eq!(named_from_u8(16), Color::White);
        assert_eq!(named_from_u8(255), Color::White);
    }

    #[test]
    fn real_env_reads_something() {
        // Smoke test — we don't assert a value (shell-dependent), only that
        // RealEnv is usable. Prefer a var that's virtually always set: PATH.
        let _ = RealEnv.var("PATH");
        // Reading an unset-very-rare var returns None cleanly.
        assert!(RealEnv
            .var("HAPPYTERMINALS_NONEXISTENT_VAR_FOR_TEST")
            .is_none());
    }
}
