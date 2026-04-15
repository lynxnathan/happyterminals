//! Grapheme-cluster utilities for terminal cells.
//!
//! [`Cell`] is a zero-sized utility namespace providing display-width computation
//! and grapheme iteration. The actual cell storage is [`ratatui_core::buffer::Cell`],
//! accessed via [`Grid::deref()`](crate::Grid) -> `&Buffer`.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Utility namespace for grapheme-cluster cell operations.
///
/// Cells are not constructed directly -- they are written via
/// [`Grid::put_str`](crate::Grid::put_str) and read via
/// `Grid::deref() -> &Buffer`.
pub struct Cell;

impl Cell {
    /// Returns the display width (in terminal columns) of a grapheme cluster string.
    ///
    /// - ASCII characters return 1
    /// - CJK ideographs return 2
    /// - Emoji return 2
    /// - Combining marks (zero-width) return 0
    #[must_use]
    pub fn display_width(s: &str) -> usize {
        s.width()
    }

    /// Iterates over extended grapheme clusters in `s`.
    pub fn graphemes(s: &str) -> impl Iterator<Item = &str> {
        s.graphemes(true)
    }
}

// Re-export ratatui types for consumer convenience.
pub use ratatui_core::buffer::Cell as RatatuiCell;
pub use ratatui_core::style::{Color, Modifier, Style};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_width_ascii() {
        assert_eq!(Cell::display_width("a"), 1);
    }

    #[test]
    fn test_display_width_cjk() {
        assert_eq!(Cell::display_width("\u{4F60}"), 2); // "你"
    }

    #[test]
    fn test_display_width_emoji() {
        assert_eq!(Cell::display_width("\u{1F3A8}"), 2); // "🎨"
    }

    #[test]
    fn test_display_width_combining() {
        assert_eq!(Cell::display_width("\u{0301}"), 0); // combining acute accent
    }
}
