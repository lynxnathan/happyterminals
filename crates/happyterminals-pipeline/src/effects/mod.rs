//! Convenience constructors wrapping tachyonfx effects in [`TachyonAdapter`](crate::TachyonAdapter).
//!
//! Each function returns a ready-to-use adapter that implements our [`Effect`](crate::Effect) trait.
//! Usage: `Pipeline::new().with(effects::dissolve(Duration::from_millis(500)))`

use std::time::Duration;

use ratatui_core::style::Color;
use tachyonfx::fx::EvolveSymbolSet;
use tachyonfx::Motion;

use crate::adapter::TachyonAdapter;

/// Dissolves foreground content over the given duration.
#[must_use]
pub fn dissolve(duration: Duration) -> TachyonAdapter {
    let tfx_dur = tachyonfx::Duration::from(duration);
    TachyonAdapter::new(tachyonfx::fx::dissolve(tfx_dur))
}

/// Fades in from the specified foreground and background colors.
#[must_use]
pub fn fade_from(fg: Color, bg: Color, duration: Duration) -> TachyonAdapter {
    let tfx_dur = tachyonfx::Duration::from(duration);
    TachyonAdapter::new(tachyonfx::fx::fade_from(fg, bg, tfx_dur))
}

/// Fades out to the specified foreground and background colors.
#[must_use]
pub fn fade_to(fg: Color, bg: Color, duration: Duration) -> TachyonAdapter {
    let tfx_dur = tachyonfx::Duration::from(duration);
    TachyonAdapter::new(tachyonfx::fx::fade_to(fg, bg, tfx_dur))
}

/// Sweeps content in from the given direction with a gradient.
#[must_use]
pub fn sweep_in(
    direction: Motion,
    gradient_length: u16,
    faded_color: Color,
    duration: Duration,
) -> TachyonAdapter {
    let tfx_dur = tachyonfx::Duration::from(duration);
    TachyonAdapter::new(tachyonfx::fx::sweep_in(direction, gradient_length, 0, faded_color, tfx_dur))
}

/// Slides content in from the given direction with a gradient.
#[must_use]
pub fn slide_in(
    direction: Motion,
    gradient_length: u16,
    color_behind: Color,
    duration: Duration,
) -> TachyonAdapter {
    let tfx_dur = tachyonfx::Duration::from(duration);
    TachyonAdapter::new(tachyonfx::fx::slide_in(direction, gradient_length, 0, color_behind, tfx_dur))
}

/// Reverse dissolve: reforms dissolved foreground content.
#[must_use]
pub fn coalesce(duration: Duration) -> TachyonAdapter {
    let tfx_dur = tachyonfx::Duration::from(duration);
    TachyonAdapter::new(tachyonfx::fx::coalesce(tfx_dur))
}

/// Shifts hue, saturation, and lightness over time (CRT-like color shift).
#[must_use]
pub fn hsl_shift(hsl_fg: [f32; 3], duration: Duration) -> TachyonAdapter {
    let tfx_dur = tachyonfx::Duration::from(duration);
    TachyonAdapter::new(tachyonfx::fx::hsl_shift(Some(hsl_fg), None, tfx_dur))
}

/// Transforms characters through a symbol set progression.
#[must_use]
pub fn evolve(symbols: EvolveSymbolSet, duration: Duration) -> TachyonAdapter {
    let tfx_dur = tachyonfx::Duration::from(duration);
    TachyonAdapter::new(tachyonfx::fx::evolve(symbols, tfx_dur))
}

/// Decreases lightness over time (vignette-like darkening).
#[must_use]
pub fn darken(amount: f32, duration: Duration) -> TachyonAdapter {
    let tfx_dur = tachyonfx::Duration::from(duration);
    TachyonAdapter::new(tachyonfx::fx::darken(Some(amount), None, tfx_dur))
}

/// Paints foreground and background colors over time.
#[must_use]
pub fn paint(fg: Color, bg: Color, duration: Duration) -> TachyonAdapter {
    let tfx_dur = tachyonfx::Duration::from(duration);
    TachyonAdapter::new(tachyonfx::fx::paint(fg, bg, tfx_dur))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::Effect;

    /// Verify each convenience constructor compiles and produces a valid Effect.
    #[test]
    fn all_constructors_produce_box_dyn_effect() {
        let dur = Duration::from_millis(500);

        let effects: Vec<Box<dyn Effect>> = vec![
            Box::new(dissolve(dur)),
            Box::new(fade_from(Color::Black, Color::Black, dur)),
            Box::new(fade_to(Color::White, Color::White, dur)),
            Box::new(sweep_in(Motion::LeftToRight, 5, Color::Black, dur)),
            Box::new(slide_in(Motion::UpToDown, 5, Color::Black, dur)),
            Box::new(coalesce(dur)),
            Box::new(hsl_shift([30.0, 0.0, 0.0], dur)),
            Box::new(evolve(EvolveSymbolSet::Circles, dur)),
            Box::new(darken(0.5, dur)),
            Box::new(paint(Color::Red, Color::Blue, dur)),
        ];

        assert_eq!(effects.len(), 10);
    }
}
