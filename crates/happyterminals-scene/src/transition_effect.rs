//! [`TransitionEffect`] trait and built-in effects for scene-to-scene transitions.
//!
//! Each effect blends two ratatui [`Buffer`]s into an output buffer based on
//! a progress value in `[0.0, 1.0]`.

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Position;

/// A transition effect that blends two buffers based on progress.
///
/// Progress `0.0` = fully scene A, `1.0` = fully scene B.
pub trait TransitionEffect: Send {
    /// Blend buffer `a` (outgoing scene) and `b` (incoming scene) into `output`.
    ///
    /// # Panics
    ///
    /// Implementations should panic if `a`, `b`, and `output` have different areas.
    fn blend(&self, a: &Buffer, b: &Buffer, progress: f32, output: &mut Buffer);
}

/// Crossfade dissolve: switches cells from A to B at the midpoint.
pub struct Dissolve;

impl TransitionEffect for Dissolve {
    fn blend(&self, _a: &Buffer, _b: &Buffer, _progress: f32, _output: &mut Buffer) {
        todo!()
    }
}

/// Slide-left wipe: scene A slides out to the left, scene B enters from the right.
pub struct SlideLeft;

impl TransitionEffect for SlideLeft {
    fn blend(&self, _a: &Buffer, _b: &Buffer, _progress: f32, _output: &mut Buffer) {
        todo!()
    }
}

/// Fade through black: scene A fades to black, then scene B fades in.
pub struct FadeToBlack;

impl TransitionEffect for FadeToBlack {
    fn blend(&self, _a: &Buffer, _b: &Buffer, _progress: f32, _output: &mut Buffer) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui_core::layout::Rect;
    use ratatui_core::style::Style;

    /// Create a 4x2 buffer where every cell has the given symbol.
    fn make_buffer(symbol: &str) -> Buffer {
        let area = Rect::new(0, 0, 4, 2);
        let mut buf = Buffer::empty(area);
        for y in 0..2 {
            for x in 0..4 {
                if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                    cell.set_symbol(symbol);
                    cell.set_style(Style::default());
                }
            }
        }
        buf
    }

    fn all_cells_eq(buf: &Buffer, symbol: &str) -> bool {
        let area = buf.area;
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                if let Some(cell) = buf.cell(Position::new(x, y)) {
                    if cell.symbol() != symbol {
                        return false;
                    }
                }
            }
        }
        true
    }

    // --- Dissolve tests ---

    #[test]
    fn dissolve_progress_0_is_all_a() {
        let a = make_buffer("A");
        let b = make_buffer("B");
        let mut out = Buffer::empty(Rect::new(0, 0, 4, 2));
        Dissolve.blend(&a, &b, 0.0, &mut out);
        assert!(all_cells_eq(&out, "A"));
    }

    #[test]
    fn dissolve_progress_1_is_all_b() {
        let a = make_buffer("A");
        let b = make_buffer("B");
        let mut out = Buffer::empty(Rect::new(0, 0, 4, 2));
        Dissolve.blend(&a, &b, 1.0, &mut out);
        assert!(all_cells_eq(&out, "B"));
    }

    #[test]
    fn dissolve_progress_half_has_b_cells() {
        let a = make_buffer("A");
        let b = make_buffer("B");
        let mut out = Buffer::empty(Rect::new(0, 0, 4, 2));
        Dissolve.blend(&a, &b, 0.5, &mut out);
        // At exactly 0.5, the spec says "progress < 0.5 => A, else B"
        // so progress=0.5 should yield B
        assert!(all_cells_eq(&out, "B"));
    }

    // --- SlideLeft tests ---

    #[test]
    fn slide_left_progress_0_is_all_a() {
        let a = make_buffer("A");
        let b = make_buffer("B");
        let mut out = Buffer::empty(Rect::new(0, 0, 4, 2));
        SlideLeft.blend(&a, &b, 0.0, &mut out);
        assert!(all_cells_eq(&out, "A"));
    }

    #[test]
    fn slide_left_progress_1_is_all_b() {
        let a = make_buffer("A");
        let b = make_buffer("B");
        let mut out = Buffer::empty(Rect::new(0, 0, 4, 2));
        SlideLeft.blend(&a, &b, 1.0, &mut out);
        assert!(all_cells_eq(&out, "B"));
    }

    #[test]
    fn slide_left_progress_half_split() {
        let a = make_buffer("A");
        let b = make_buffer("B");
        let mut out = Buffer::empty(Rect::new(0, 0, 4, 2));
        SlideLeft.blend(&a, &b, 0.5, &mut out);
        // At 0.5: split_x = 0 + (4 * 0.5) = 2
        // x < 2 => A, x >= 2 => B
        for y in 0..2 {
            assert_eq!(out.cell(Position::new(0, y)).unwrap().symbol(), "A");
            assert_eq!(out.cell(Position::new(1, y)).unwrap().symbol(), "A");
            assert_eq!(out.cell(Position::new(2, y)).unwrap().symbol(), "B");
            assert_eq!(out.cell(Position::new(3, y)).unwrap().symbol(), "B");
        }
    }

    // --- FadeToBlack tests ---

    #[test]
    fn fade_to_black_progress_0_is_all_a() {
        let a = make_buffer("A");
        let b = make_buffer("B");
        let mut out = Buffer::empty(Rect::new(0, 0, 4, 2));
        FadeToBlack.blend(&a, &b, 0.0, &mut out);
        assert!(all_cells_eq(&out, "A"));
    }

    #[test]
    fn fade_to_black_progress_quarter_has_some_black() {
        let a = make_buffer("A");
        let b = make_buffer("B");
        let mut out = Buffer::empty(Rect::new(0, 0, 4, 2));
        FadeToBlack.blend(&a, &b, 0.25, &mut out);
        // sub-progress p = 0.25 / 0.5 = 0.5
        // Cells where (x / width) < p become black (space)
        // x=0: 0.0 < 0.5 => space; x=1: 0.25 < 0.5 => space
        // x=2: 0.5 < 0.5 => false => A; x=3: 0.75 < 0.5 => false => A
        let has_space = (0..4).any(|x| out.cell(Position::new(x, 0)).unwrap().symbol() == " ");
        let has_a = (0..4).any(|x| out.cell(Position::new(x, 0)).unwrap().symbol() == "A");
        assert!(has_space, "should have some faded cells at progress=0.25");
        assert!(has_a, "should still have some A cells at progress=0.25");
    }

    #[test]
    fn fade_to_black_progress_three_quarters_has_b() {
        let a = make_buffer("A");
        let b = make_buffer("B");
        let mut out = Buffer::empty(Rect::new(0, 0, 4, 2));
        FadeToBlack.blend(&a, &b, 0.75, &mut out);
        // sub-progress p = (0.75 - 0.5) / 0.5 = 0.5
        // Cells where (x / width) < p show B, else space
        let has_b = (0..4).any(|x| out.cell(Position::new(x, 0)).unwrap().symbol() == "B");
        assert!(has_b, "should have B cells emerging at progress=0.75");
    }

    #[test]
    fn fade_to_black_progress_1_is_all_b() {
        let a = make_buffer("A");
        let b = make_buffer("B");
        let mut out = Buffer::empty(Rect::new(0, 0, 4, 2));
        FadeToBlack.blend(&a, &b, 1.0, &mut out);
        assert!(all_cells_eq(&out, "B"));
    }

    // --- Area assertion ---

    #[test]
    #[should_panic]
    fn blend_panics_on_mismatched_areas() {
        let a = make_buffer("A");
        let b = Buffer::empty(Rect::new(0, 0, 8, 4));
        let mut out = Buffer::empty(Rect::new(0, 0, 4, 2));
        Dissolve.blend(&a, &b, 0.5, &mut out);
    }
}
