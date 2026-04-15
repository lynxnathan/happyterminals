//! Snapshot tests for the ASCII 3D renderer.
//!
//! Uses insta to capture and verify the rendered output at known camera poses.

use happyterminals_core::{Grid, Rect};
use happyterminals_renderer::{OrbitCamera, Projection, Renderer, ShadingRamp};
use ratatui_core::layout::Position;
use std::f32::consts::{FRAC_PI_4, FRAC_PI_6};

/// Extract the grid content as a multi-line string for snapshot comparison.
fn grid_to_string(grid: &Grid, width: u16, height: u16) -> String {
    let mut result = String::with_capacity((width as usize + 1) * height as usize);
    for y in 0..height {
        for x in 0..width {
            if let Some(cell) = grid.cell(Position::new(x, y)) {
                let sym = cell.symbol();
                if let Some(ch) = sym.chars().next() {
                    result.push(ch);
                } else {
                    result.push(' ');
                }
            } else {
                result.push(' ');
            }
        }
        if y + 1 < height {
            result.push('\n');
        }
    }
    result
}

#[test]
fn snapshot_cube_default_ramp() {
    let mut grid = Grid::new(Rect::new(0, 0, 80, 24));
    let camera = OrbitCamera {
        azimuth: FRAC_PI_4,
        elevation: FRAC_PI_6,
        distance: 5.0,
        target: glam::Vec3::ZERO,
    };
    let projection = Projection {
        viewport_w: 80,
        viewport_h: 24,
        ..Projection::default()
    };
    let shading = ShadingRamp::default();
    let mut renderer = Renderer::new();

    renderer.draw(&mut grid, &camera, &projection, &shading);

    let output = grid_to_string(&grid, 80, 24);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_cube_custom_ramp() {
    let mut grid = Grid::new(Rect::new(0, 0, 80, 24));
    let camera = OrbitCamera {
        azimuth: FRAC_PI_4,
        elevation: FRAC_PI_6,
        distance: 5.0,
        target: glam::Vec3::ZERO,
    };
    let projection = Projection {
        viewport_w: 80,
        viewport_h: 24,
        ..Projection::default()
    };
    let custom_ramp = &['_', '/', '#'];
    let shading = ShadingRamp {
        ramp: custom_ramp,
        light_dir: glam::Vec3::new(1.0, 1.0, 1.0).normalize(),
    };
    let mut renderer = Renderer::new();

    renderer.draw(&mut grid, &camera, &projection, &shading);

    let output = grid_to_string(&grid, 80, 24);

    // Custom ramp should produce different output than default
    insta::assert_snapshot!(output);
}
