//! Integration tests for the color-mode detection cascade.
//!
//! Uses a local `FakeEnv` implementing `EnvProvider` so we never call
//! `std::env::set_var` (RESEARCH §Pitfall 4: parallel test env-leak).
//!
//! Each test name is referenced verbatim in the phase RESEARCH §"Phase
//! Requirements → Test Map" table. Do not rename without updating RESEARCH.

use std::collections::HashMap;

use happyterminals_backend_ratatui::color::{detect_color_mode, ColorMode, EnvProvider};

struct FakeEnv {
    vars: HashMap<String, String>,
}

impl FakeEnv {
    fn empty() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    fn with(pairs: &[(&str, &str)]) -> Self {
        let mut e = Self::empty();
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

#[test]
fn no_color_strips_all_chromatic() {
    // NO_COLOR=1 — regardless of COLORTERM/TERM, result is Mono.
    let env = FakeEnv::with(&[
        ("NO_COLOR", "1"),
        ("COLORTERM", "truecolor"),
        ("TERM", "xterm-256color"),
    ]);
    assert_eq!(detect_color_mode(None, &env), ColorMode::Mono);
}

#[test]
fn no_color_empty_is_unset() {
    // Per no-color.org: empty string means NOT disabled.
    // Falls through to TERM → xterm-256color → Palette256.
    let env = FakeEnv::with(&[("NO_COLOR", ""), ("TERM", "xterm-256color")]);
    assert_eq!(detect_color_mode(None, &env), ColorMode::Palette256);
}

#[test]
fn no_color_zero_disables() {
    // Any non-empty value (including "0") triggers Mono.
    let env = FakeEnv::with(&[("NO_COLOR", "0"), ("COLORTERM", "truecolor")]);
    assert_eq!(detect_color_mode(None, &env), ColorMode::Mono);
}

#[test]
fn override_beats_env() {
    let env = FakeEnv::with(&[("COLORTERM", "truecolor")]);
    assert_eq!(
        detect_color_mode(Some(ColorMode::Ansi16), &env),
        ColorMode::Ansi16
    );
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
fn colorterm_1_not_truecolor() {
    // Legacy non-spec "yes colors" value. Must NOT claim truecolor.
    let env = FakeEnv::with(&[("COLORTERM", "1"), ("TERM", "xterm-256color")]);
    assert_eq!(detect_color_mode(None, &env), ColorMode::Palette256);
}

#[test]
fn term_256color_detected() {
    let env = FakeEnv::with(&[("TERM", "xterm-256color")]);
    assert_eq!(detect_color_mode(None, &env), ColorMode::Palette256);
}

#[test]
fn term_dumb_is_mono() {
    let env = FakeEnv::with(&[("TERM", "dumb")]);
    assert_eq!(detect_color_mode(None, &env), ColorMode::Mono);
}

#[test]
fn empty_env_is_mono() {
    // No TERM at all → piped / non-tty → plain text (Mono).
    assert_eq!(detect_color_mode(None, &FakeEnv::empty()), ColorMode::Mono);
}
