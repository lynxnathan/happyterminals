//! Grid: the terminal framebuffer.
//!
//! [`Grid`] is a newtype over [`ratatui_core::buffer::Buffer`] providing
//! read-only `Deref` access (no `DerefMut`) and [`put_str`](Grid::put_str)
//! as the sole text-writing API with grapheme-cluster-correct rendering.

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Style;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// A terminal grid buffer. Wraps `ratatui::Buffer` via `Deref` for
/// read-only access; all writes go through [`put_str`](Self::put_str).
pub struct Grid {
    inner: Buffer,
}

impl Grid {
    /// Creates a new empty grid with the given dimensions.
    #[must_use]
    pub fn new(area: Rect) -> Self {
        todo!("implement Grid::new")
    }

    /// Writes a styled string at `(x, y)`, handling grapheme clusters,
    /// wide characters, and silent out-of-bounds clipping.
    pub fn put_str(&mut self, x: u16, y: u16, s: &str, style: Style) {
        todo!("implement put_str")
    }

    /// Replaces the grid with a fresh empty buffer of the given dimensions.
    pub fn resize(&mut self, area: Rect) {
        todo!("implement resize")
    }

    /// Backend access to the underlying buffer for blit operations.
    pub(crate) fn inner_mut(&mut self) -> &mut Buffer {
        &mut self.inner
    }
}

impl std::ops::Deref for Grid {
    type Target = Buffer;
    fn deref(&self) -> &Buffer {
        &self.inner
    }
}

// NO DerefMut -- writes go through put_str only.

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui_core::style::Color;

    #[test]
    fn test_new_grid_dimensions() {
        let grid = Grid::new(Rect::new(0, 0, 80, 24));
        assert_eq!(grid.area.width, 80);
        assert_eq!(grid.area.height, 24);
    }

    #[test]
    fn test_deref_to_buffer() {
        let grid = Grid::new(Rect::new(0, 0, 10, 10));
        // Proves Deref works: calling Buffer::area() on Grid
        let _area = grid.area;
        // Can access cells through Deref
        let cell = grid.cell(Position::new(0, 0));
        assert!(cell.is_some());
    }

    #[test]
    fn test_put_str_ascii() {
        let mut grid = Grid::new(Rect::new(0, 0, 80, 24));
        grid.put_str(0, 0, "Hello", Style::default());
        for (i, ch) in "Hello".chars().enumerate() {
            let cell = grid.cell(Position::new(i as u16, 0)).unwrap();
            assert_eq!(cell.symbol(), &ch.to_string(), "cell {i}");
        }
    }

    #[test]
    fn test_put_str_cjk() {
        let mut grid = Grid::new(Rect::new(0, 0, 80, 24));
        grid.put_str(0, 0, "\u{4F60}\u{597D}", Style::default()); // "你好"

        let c0 = grid.cell(Position::new(0, 0)).unwrap();
        assert_eq!(c0.symbol(), "\u{4F60}"); // "你"

        let c1 = grid.cell(Position::new(1, 0)).unwrap();
        assert!(c1.skip, "continuation cell for wide char should have skip=true");

        let c2 = grid.cell(Position::new(2, 0)).unwrap();
        assert_eq!(c2.symbol(), "\u{597D}"); // "好"

        let c3 = grid.cell(Position::new(3, 0)).unwrap();
        assert!(c3.skip, "continuation cell for wide char should have skip=true");
    }

    #[test]
    fn test_put_str_emoji() {
        let mut grid = Grid::new(Rect::new(0, 0, 80, 24));
        grid.put_str(0, 0, "\u{1F3A8}", Style::default()); // "🎨"

        let c0 = grid.cell(Position::new(0, 0)).unwrap();
        assert_eq!(c0.symbol(), "\u{1F3A8}");

        let c1 = grid.cell(Position::new(1, 0)).unwrap();
        assert!(c1.skip, "continuation cell for emoji should have skip=true");
    }

    #[test]
    fn test_put_str_zwj() {
        let mut grid = Grid::new(Rect::new(0, 0, 80, 24));
        let family = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}";
        grid.put_str(0, 0, family, Style::default());

        let c0 = grid.cell(Position::new(0, 0)).unwrap();
        assert_eq!(c0.symbol(), family);

        let c1 = grid.cell(Position::new(1, 0)).unwrap();
        assert!(c1.skip, "continuation cell for ZWJ should have skip=true");
    }

    #[test]
    fn test_put_str_clips_x() {
        let mut grid = Grid::new(Rect::new(0, 0, 80, 24));
        // Writing "Hello" at x=78 on 80-wide grid: only 2 chars fit
        grid.put_str(78, 0, "Hello", Style::default());

        let c78 = grid.cell(Position::new(78, 0)).unwrap();
        assert_eq!(c78.symbol(), "H");
        let c79 = grid.cell(Position::new(79, 0)).unwrap();
        assert_eq!(c79.symbol(), "e");
        // No panic -- that's the test
    }

    #[test]
    fn test_put_str_clips_y() {
        let mut grid = Grid::new(Rect::new(0, 0, 80, 24));
        // Writing at y=25 on 24-high grid: nothing written, no panic
        grid.put_str(0, 25, "Hello", Style::default());
        // If we get here, no panic occurred
    }

    #[test]
    fn test_put_str_style() {
        let mut grid = Grid::new(Rect::new(0, 0, 80, 24));
        let style = Style::default().fg(Color::Red);
        grid.put_str(0, 0, "x", style);

        let c0 = grid.cell(Position::new(0, 0)).unwrap();
        assert_eq!(c0.fg, Color::Red);
    }

    #[test]
    fn test_resize() {
        let mut grid = Grid::new(Rect::new(0, 0, 80, 24));
        grid.put_str(0, 0, "Hello", Style::default());
        grid.resize(Rect::new(0, 0, 40, 12));

        assert_eq!(grid.area.width, 40);
        assert_eq!(grid.area.height, 12);
        // Content is fresh (empty)
        let c0 = grid.cell(Position::new(0, 0)).unwrap();
        assert_eq!(c0.symbol(), " "); // default empty cell
    }

    #[test]
    fn test_no_deref_mut() {
        // DerefMut is intentionally NOT implemented.
        // This is a documentation test -- if DerefMut were added,
        // it would be a breaking API change. We verify at review time.
        // A compile-fail test would require trybuild, which is overkill here.
    }
}
