//! Color-mode pipeline — Phase 2.2 foundation.
//!
//! - [`ColorMode`] enum: `TrueColor` | `Palette256` | `Ansi16` | `Mono` (Task 2).
//! - `detect_color_mode`: env-var cascade (Task 2).
//! - `downsample`: flush-time buffer transform (Task 2).
//! - [`palette`] submodule: compile-time 256 palette + 256→16 LUT (Task 1).

pub mod palette;

pub use palette::{nearest_256, PALETTE_16_LUT, PALETTE_256};

// ColorMode, EnvProvider, RealEnv, detect_color_mode, downsample, map_color,
// named_from_u8 all land in Task 2.
