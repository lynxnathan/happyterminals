//! Security sandbox utilities for JSON recipe loading.
//!
//! Three pure-data utilities that harden [`crate::json::load_recipe_sandboxed`]:
//!
//! - [`EffectRegistry`] -- a static allow-list of transition-effect names so
//!   JSON recipes cannot name arbitrary code to resolve.
//! - [`sanitize_path`] -- a pure-string validator that rejects path traversal
//!   (`..`), current-directory (`.`), absolute paths, and Windows drive
//!   letters before any file I/O occurs.
//! - [`strip_ansi`] -- a byte-level scanner that removes CSI, OSC, and simple
//!   escape sequences from user-provided strings so they cannot reprogram the
//!   terminal when rendered as cell content.
//!
//! None of these utilities perform I/O. `sanitize_path` does not call
//! `canonicalize()` (that would require the file to exist).

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::json::RecipeError;

// ── Effect registry ─────────────────────────────────────────────────────

/// Allow-list of transition-effect names that JSON recipes may reference.
///
/// The registry is a static mapping from name to "resolves" -- it does not
/// hold the effect implementations themselves. Callers look up the name and,
/// on success, map it to the corresponding [`TransitionEffect`][1] instance.
/// This keeps recipe loading free of dynamic code paths.
///
/// [`EffectRegistry::default`] is populated with the three built-ins shipped
/// by `happyterminals-scene`: `"dissolve"`, `"slide-left"`, and
/// `"fade-to-black"`.
///
/// [1]: happyterminals_scene::transition_effect::TransitionEffect
#[derive(Debug, Clone)]
pub struct EffectRegistry {
    names: HashSet<String>,
}

impl EffectRegistry {
    /// Create an empty registry. Prefer [`EffectRegistry::default`] for the
    /// set of built-in effects.
    #[must_use]
    pub fn new() -> Self {
        Self {
            names: HashSet::new(),
        }
    }

    /// Register a custom effect name.
    pub fn register(&mut self, name: &str) {
        self.names.insert(name.to_owned());
    }

    /// Resolve an effect name.
    ///
    /// # Errors
    ///
    /// Returns [`RecipeError::UnknownEffect`] if `name` is not in the
    /// registry. The error preserves the offending name so callers can
    /// report it back to the recipe author.
    pub fn resolve(&self, name: &str) -> Result<(), RecipeError> {
        if self.names.contains(name) {
            Ok(())
        } else {
            Err(RecipeError::UnknownEffect {
                name: name.to_owned(),
            })
        }
    }

    /// Number of registered effects. Mainly useful for testing.
    #[must_use]
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// Whether the registry contains no effects.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}

impl Default for EffectRegistry {
    /// Registry pre-populated with the three built-in transition effects from
    /// `happyterminals-scene`: `dissolve`, `slide-left`, `fade-to-black`.
    fn default() -> Self {
        let mut reg = Self::new();
        reg.register("dissolve");
        reg.register("slide-left");
        reg.register("fade-to-black");
        reg
    }
}

// ── Path sandboxing ─────────────────────────────────────────────────────

/// Validate `relative` and return `asset_root.join(relative)` on success.
///
/// Rejects:
/// - empty strings,
/// - absolute paths (starting with `/`),
/// - Windows drive-letter prefixes (e.g. `C:\`, `c:/`),
/// - any path component equal to `..` or `.`.
///
/// This is a pure-string check; the file does not need to exist. We
/// intentionally do NOT call [`Path::canonicalize`] -- that would require the
/// file to exist and follows symlinks, which would bypass the guarantees we
/// want here.
///
/// # Errors
///
/// Returns [`RecipeError::PathTraversal`] with the offending path on any
/// rejected input.
pub fn sanitize_path(relative: &str, asset_root: &Path) -> Result<PathBuf, RecipeError> {
    if relative.is_empty() {
        return Err(RecipeError::PathTraversal {
            path: relative.to_owned(),
        });
    }

    // Reject absolute paths (POSIX). Rust's `Path::is_absolute` is
    // platform-aware; we do our own check here for two reasons:
    //   1. We want a leading `/` rejected on Windows as well as POSIX.
    //   2. `Path::is_absolute` lets Windows drive letters slip on POSIX.
    if relative.starts_with('/') || relative.starts_with('\\') {
        return Err(RecipeError::PathTraversal {
            path: relative.to_owned(),
        });
    }

    // Reject Windows drive-letter prefixes like "C:" or "c:/" regardless of
    // host platform. Recipes should use forward-slash relative paths.
    let bytes = relative.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        return Err(RecipeError::PathTraversal {
            path: relative.to_owned(),
        });
    }

    // Split on both forward and back slashes so Windows-flavored recipes
    // can't sneak `..` past a forward-slash-only split.
    for component in relative.split(['/', '\\']) {
        if component == ".." || component == "." {
            return Err(RecipeError::PathTraversal {
                path: relative.to_owned(),
            });
        }
    }

    Ok(asset_root.join(relative))
}

// ── ANSI stripping ──────────────────────────────────────────────────────

/// Remove ANSI escape sequences from `input`.
///
/// Handles:
/// - **CSI** sequences: `ESC [` followed by any number of parameter/intermediate
///   bytes, terminated by a final byte in `@..~` (0x40..=0x7E). Covers SGR
///   (colors), cursor movement, screen clear (`ESC[2J`), and private modes
///   like `ESC[?25h`.
/// - **OSC** sequences: `ESC ]` until a `BEL` (0x07) or `ST` (`ESC \`).
/// - **Simple escapes**: `ESC` followed by a single byte in `@..~` (covers
///   SS2, SS3, RIS, charset selection, etc.). This also swallows the 2-byte
///   `ESC \` string terminator harmlessly.
///
/// Operates on UTF-8 bytes and always returns a valid UTF-8 string -- ANSI
/// escape structural bytes are all ASCII, so bytes inside a sequence are
/// discarded without splitting a multi-byte character.
#[must_use]
pub fn strip_ansi(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];
        if b != 0x1B {
            out.push(b);
            i += 1;
            continue;
        }

        // We have an ESC. Look at the next byte to decide sequence kind.
        let Some(&next) = bytes.get(i + 1) else {
            // Lone ESC at end of string -- drop it.
            break;
        };

        match next {
            b'[' => {
                // CSI: consume params/intermediates, stop after a final byte
                // in 0x40..=0x7E.
                let mut j = i + 2;
                while j < bytes.len() {
                    let c = bytes[j];
                    j += 1;
                    if (0x40..=0x7E).contains(&c) {
                        break;
                    }
                }
                i = j;
            }
            b']' => {
                // OSC: consume until BEL (0x07) or ST (ESC \).
                let mut j = i + 2;
                while j < bytes.len() {
                    let c = bytes[j];
                    if c == 0x07 {
                        j += 1;
                        break;
                    }
                    if c == 0x1B && bytes.get(j + 1) == Some(&b'\\') {
                        j += 2;
                        break;
                    }
                    j += 1;
                }
                i = j;
            }
            // Simple ESC + single final byte. Range 0x40..=0x5F covers
            // SS2/SS3/RIS/etc.; we widen to 0x40..=0x7E to also swallow the
            // lone `ESC \` string terminator if one appears outside an OSC.
            0x40..=0x7E => {
                i += 2;
            }
            _ => {
                // Unknown ESC-prefix byte: drop just the ESC, keep the next
                // byte for normal processing. This is defensive -- a well-
                // formed stream shouldn't reach here.
                i += 1;
            }
        }
    }

    // Safety: we only discarded whole ASCII escape sequences, so the
    // remaining bytes are a prefix of the original UTF-8 sequence with
    // ASCII-only cuts.
    String::from_utf8(out).unwrap_or_default()
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── EffectRegistry ──────────────────────────────────────────────

    #[test]
    fn sandbox_default_registry_contains_three_builtins() {
        let reg = EffectRegistry::default();
        assert_eq!(reg.len(), 3, "default registry should contain 3 built-ins");
        assert!(reg.resolve("dissolve").is_ok());
        assert!(reg.resolve("slide-left").is_ok());
        assert!(reg.resolve("fade-to-black").is_ok());
    }

    #[test]
    fn sandbox_registry_resolve_unknown_returns_error_with_name() {
        let reg = EffectRegistry::default();
        let Err(err) = reg.resolve("nonexistent") else {
            panic!("resolving unknown effect should fail");
        };
        let msg = err.to_string();
        assert!(
            msg.contains("nonexistent"),
            "error should mention effect name, got: {msg}"
        );
        assert!(matches!(err, RecipeError::UnknownEffect { .. }));
    }

    #[test]
    fn sandbox_registry_register_custom_then_resolve_ok() {
        let mut reg = EffectRegistry::new();
        reg.register("custom");
        assert!(reg.resolve("custom").is_ok());
    }

    #[test]
    fn sandbox_registry_new_is_empty() {
        let reg = EffectRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    // ── sanitize_path ───────────────────────────────────────────────

    #[test]
    fn sandbox_path_accepts_simple_relative() {
        let root = Path::new("/assets");
        let out = sanitize_path("bunny.obj", root).unwrap();
        assert_eq!(out, PathBuf::from("/assets/bunny.obj"));
    }

    #[test]
    fn sandbox_path_accepts_subdir() {
        let root = Path::new("/assets");
        let out = sanitize_path("subdir/model.obj", root).unwrap();
        assert_eq!(out, PathBuf::from("/assets/subdir/model.obj"));
    }

    #[test]
    fn sandbox_path_rejects_parent_traversal() {
        let root = Path::new("/assets");
        let err = sanitize_path("../etc/passwd", root).unwrap_err();
        assert!(matches!(err, RecipeError::PathTraversal { .. }));
    }

    #[test]
    fn sandbox_path_rejects_nested_parent_traversal() {
        let root = Path::new("/assets");
        let err = sanitize_path("subdir/../bunny.obj", root).unwrap_err();
        assert!(matches!(err, RecipeError::PathTraversal { .. }));
    }

    #[test]
    fn sandbox_path_rejects_absolute() {
        let root = Path::new("/assets");
        let err = sanitize_path("/etc/passwd", root).unwrap_err();
        assert!(matches!(err, RecipeError::PathTraversal { .. }));
    }

    #[test]
    fn sandbox_path_rejects_empty() {
        let root = Path::new("/assets");
        let err = sanitize_path("", root).unwrap_err();
        assert!(matches!(err, RecipeError::PathTraversal { .. }));
    }

    #[test]
    fn sandbox_path_rejects_current_dir_component() {
        let root = Path::new("/assets");
        let err = sanitize_path("./bunny.obj", root).unwrap_err();
        assert!(matches!(err, RecipeError::PathTraversal { .. }));
    }

    #[test]
    fn sandbox_path_rejects_windows_drive_letter() {
        let root = Path::new("/assets");
        let err = sanitize_path("C:\\Windows\\System32", root).unwrap_err();
        assert!(matches!(err, RecipeError::PathTraversal { .. }));
    }

    #[test]
    fn sandbox_path_rejects_backslash_traversal() {
        // Even on POSIX hosts, a `..` hidden behind a backslash separator
        // should be rejected so recipes can't smuggle traversal across
        // platforms.
        let root = Path::new("/assets");
        let err = sanitize_path("subdir\\..\\bunny.obj", root).unwrap_err();
        assert!(matches!(err, RecipeError::PathTraversal { .. }));
    }

    // ── strip_ansi ──────────────────────────────────────────────────

    #[test]
    fn sandbox_ansi_plain_text_unchanged() {
        assert_eq!(strip_ansi("hello"), "hello");
    }

    #[test]
    fn sandbox_ansi_strips_screen_clear() {
        // ESC [ 2 J -- clear screen. Must not be rendered as control.
        assert_eq!(strip_ansi("\x1b[2Jhello"), "hello");
    }

    #[test]
    fn sandbox_ansi_strips_sgr_color() {
        // ESC [ 31 m red ESC [ 0 m -- red foreground then reset.
        assert_eq!(strip_ansi("\x1b[31mred\x1b[0m"), "red");
    }

    #[test]
    fn sandbox_ansi_strips_osc_title() {
        // ESC ] 0 ; title BEL -- set window title.
        assert_eq!(strip_ansi("a\x1b]0;title\x07b"), "ab");
    }

    #[test]
    fn sandbox_ansi_strips_osc_terminated_by_st() {
        // OSC terminated by ST (ESC \) rather than BEL.
        assert_eq!(strip_ansi("a\x1b]0;title\x1b\\b"), "ab");
    }

    #[test]
    fn sandbox_ansi_strips_private_mode_sequence() {
        // ESC [ ? 25 h -- show cursor (private mode with parameter).
        assert_eq!(strip_ansi("\x1b[?25h"), "");
    }

    #[test]
    fn sandbox_ansi_preserves_utf8_multibyte() {
        // Ensure we don't slice through multi-byte characters.
        assert_eq!(strip_ansi("café"), "café");
        assert_eq!(strip_ansi("\x1b[31mcafé\x1b[0m"), "café");
    }

    #[test]
    fn sandbox_ansi_strips_simple_escape() {
        // ESC N (SS2) -- single-shift two. ESC followed by 0x4E.
        assert_eq!(strip_ansi("a\x1bNb"), "ab");
    }

    #[test]
    fn sandbox_ansi_strips_multiple_sequences() {
        let input = "\x1b[2J\x1b[H\x1b[31mhi\x1b[0m\x1b]0;t\x07!";
        assert_eq!(strip_ansi(input), "hi!");
    }

    #[test]
    fn sandbox_ansi_lone_esc_at_end_dropped() {
        assert_eq!(strip_ansi("hello\x1b"), "hello");
    }
}
