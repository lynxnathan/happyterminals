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

/// Assert that all three buffers have identical areas.
fn assert_areas_match(a: &Buffer, b: &Buffer, output: &Buffer) {
    assert!(
        a.area == b.area && b.area == output.area,
        "TransitionEffect::blend requires all buffers to have the same area; \
         got a={:?}, b={:?}, output={:?}",
        a.area,
        b.area,
        output.area,
    );
}

/// Copy symbol and style from `src` cell at `pos` to `dst` cell at `pos`.
fn copy_cell(src: &Buffer, dst: &mut Buffer, pos: Position) {
    if let (Some(s), Some(d)) = (src.cell(pos), dst.cell_mut(pos)) {
        d.set_symbol(s.symbol());
        d.set_style(s.style());
    }
}

/// Set the cell at `pos` to a space (black/empty).
fn blank_cell(dst: &mut Buffer, pos: Position) {
    if let Some(d) = dst.cell_mut(pos) {
        d.set_symbol(" ");
        d.set_style(ratatui_core::style::Style::default());
    }
}

// ---------------------------------------------------------------------------
// Built-in effects
// ---------------------------------------------------------------------------

/// Crossfade dissolve: switches cells from A to B at the midpoint.
///
/// At `progress < 0.5` all cells show A; at `progress >= 0.5` all cells show B.
pub struct Dissolve;

impl TransitionEffect for Dissolve {
    fn blend(&self, a: &Buffer, b: &Buffer, progress: f32, output: &mut Buffer) {
        assert_areas_match(a, b, output);
        let src = if progress < 0.5 { a } else { b };
        let area = a.area;
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                copy_cell(src, output, Position::new(x, y));
            }
        }
    }
}

/// Slide-left wipe: scene A slides out to the left, scene B enters from the right.
pub struct SlideLeft;

impl TransitionEffect for SlideLeft {
    fn blend(&self, a: &Buffer, b: &Buffer, progress: f32, output: &mut Buffer) {
        assert_areas_match(a, b, output);
        let area = a.area;
        let width = f32::from(area.width);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let split_x = area.x + ((width * (1.0 - progress)) as u16).min(area.width);

        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                let src = if x < split_x { a } else { b };
                copy_cell(src, output, Position::new(x, y));
            }
        }
    }
}

/// Fade through black: scene A fades to black left-to-right, then scene B
/// fades in left-to-right.
pub struct FadeToBlack;

impl TransitionEffect for FadeToBlack {
    fn blend(&self, a: &Buffer, b: &Buffer, progress: f32, output: &mut Buffer) {
        assert_areas_match(a, b, output);
        let area = a.area;
        let width = f32::from(area.width);

        if progress < 0.5 {
            // Phase 1: fade A to black, left-to-right
            let p = progress / 0.5; // sub-progress 0..1
            for y in area.y..area.y + area.height {
                for x in area.x..area.x + area.width {
                    let col_ratio = f32::from(x - area.x) / width;
                    if col_ratio < p {
                        blank_cell(output, Position::new(x, y));
                    } else {
                        copy_cell(a, output, Position::new(x, y));
                    }
                }
            }
        } else if (progress - 1.0).abs() < f32::EPSILON {
            // Exactly 1.0: all B
            for y in area.y..area.y + area.height {
                for x in area.x..area.x + area.width {
                    copy_cell(b, output, Position::new(x, y));
                }
            }
        } else {
            // Phase 2: fade in B from black, left-to-right
            let p = (progress - 0.5) / 0.5; // sub-progress 0..1
            for y in area.y..area.y + area.height {
                for x in area.x..area.x + area.width {
                    let col_ratio = f32::from(x - area.x) / width;
                    if col_ratio < p {
                        copy_cell(b, output, Position::new(x, y));
                    } else {
                        blank_cell(output, Position::new(x, y));
                    }
                }
            }
        }
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
