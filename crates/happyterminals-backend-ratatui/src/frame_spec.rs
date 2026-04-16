//! Frame configuration — [`FrameSpec`] controls fps, title, mouse capture,
//! and color-mode override.

use std::time::Duration;

use crate::color::ColorMode;

/// Configuration for the render loop.
pub struct FrameSpec {
    /// Target frames per second (default: 30).
    pub fps: u32,
    /// Optional window title (applied via crossterm `SetTitle`, best-effort).
    pub title: Option<String>,
    /// Whether to capture mouse events (default: `true`).
    pub mouse_capture: bool,
    /// Programmatic color-mode override.
    ///
    /// - `None` (default) → runtime auto-detection via
    ///   [`crate::color::detect_color_mode_from_real_env`].
    /// - `Some(mode)` → hard override; beaten only by `NO_COLOR=<non-empty>`
    ///   per the no-color.org spec.
    pub color_mode: Option<ColorMode>,
}

impl Default for FrameSpec {
    fn default() -> Self {
        Self {
            fps: 30,
            title: None,
            mouse_capture: true,
            color_mode: None,
        }
    }
}

impl FrameSpec {
    /// Duration between frames based on the configured `fps`.
    #[must_use]
    pub fn frame_duration(&self) -> Duration {
        Duration::from_millis(1000 / u64::from(self.fps))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_fps() {
        assert_eq!(FrameSpec::default().fps, 30);
    }

    #[test]
    fn test_default_mouse() {
        assert!(FrameSpec::default().mouse_capture);
    }

    #[test]
    fn test_frame_duration_30fps() {
        let spec = FrameSpec::default();
        // 1000 / 30 = 33ms (integer division)
        assert_eq!(spec.frame_duration(), Duration::from_millis(33));
    }

    #[test]
    fn test_frame_duration_60fps() {
        let spec = FrameSpec { fps: 60, ..Default::default() };
        // 1000 / 60 = 16ms (integer division)
        assert_eq!(spec.frame_duration(), Duration::from_millis(16));
    }

    #[test]
    fn test_default_color_mode() {
        assert_eq!(FrameSpec::default().color_mode, None);
    }

    #[test]
    fn test_explicit_color_mode() {
        let spec = FrameSpec {
            color_mode: Some(ColorMode::Ansi16),
            ..Default::default()
        };
        assert_eq!(spec.color_mode, Some(ColorMode::Ansi16));
    }
}
