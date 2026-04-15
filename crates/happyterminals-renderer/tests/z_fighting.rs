//! Z-fighting regression tests for the reversed-Z depth buffer.
//!
//! Verifies that the rasterizer's depth testing is deterministic and that
//! reversed-Z correctly resolves closer objects over farther ones.

use happyterminals_renderer::rasterizer::rasterize_triangle;

/// Two coplanar triangles at the same depth: the second one rasterized wins
/// deterministically (no flickering).
#[test]
fn coplanar_triangles_are_deterministic() {
    let grid_w = 20;
    let grid_h = 10;
    let size = grid_w * grid_h;
    let mut z_buffer = vec![0.0_f32; size];
    let mut cell_chars = vec![' '; size];

    // Triangle A covering center area at depth 0.5
    let tri_a = [
        (8.0_f32, 2.0, 0.5),
        (14.0, 8.0, 0.5),
        (4.0, 8.0, 0.5),
    ];
    rasterize_triangle(&tri_a, &mut z_buffer, grid_w, grid_h, 'A', &mut cell_chars);

    // Count A cells
    let a_count: usize = cell_chars.iter().filter(|&&c| c == 'A').count();
    assert!(a_count > 0, "Triangle A should have rasterized some cells");

    // Triangle B at same position and depth
    let tri_b = tri_a; // Exact same triangle
    rasterize_triangle(&tri_b, &mut z_buffer, grid_w, grid_h, 'B', &mut cell_chars);

    // With reversed-Z and >= comparison, B at same depth would NOT overwrite A
    // because depth == z_buffer (not strictly >). So A should still be there.
    let a_after: usize = cell_chars.iter().filter(|&&c| c == 'A').count();
    let b_after: usize = cell_chars.iter().filter(|&&c| c == 'B').count();

    // The key property: deterministic. Either all A or all B (or a consistent mix).
    // With our > test, same depth does NOT overwrite, so A stays.
    assert_eq!(a_after, a_count, "Same-depth triangle should not overwrite (reversed-Z uses >)");
    assert_eq!(b_after, 0, "Second triangle at same depth should not overwrite first");
}

/// A triangle at higher reversed-Z depth (closer to camera) overwrites a farther one.
#[test]
fn closer_triangle_overwrites_farther() {
    let grid_w = 20;
    let grid_h = 10;
    let size = grid_w * grid_h;
    let mut z_buffer = vec![0.0_f32; size];
    let mut cell_chars = vec![' '; size];

    // Farther triangle at depth 0.5 (in reversed-Z, lower = farther)
    let far = [
        (8.0_f32, 2.0, 0.5),
        (14.0, 8.0, 0.5),
        (4.0, 8.0, 0.5),
    ];
    rasterize_triangle(&far, &mut z_buffer, grid_w, grid_h, 'F', &mut cell_chars);

    let far_count: usize = cell_chars.iter().filter(|&&c| c == 'F').count();
    assert!(far_count > 0, "Far triangle should have rasterized");

    // Closer triangle at depth 0.8 (higher = closer in reversed-Z)
    let close = [
        (8.0_f32, 2.0, 0.8),
        (14.0, 8.0, 0.8),
        (4.0, 8.0, 0.8),
    ];
    rasterize_triangle(&close, &mut z_buffer, grid_w, grid_h, 'C', &mut cell_chars);

    let close_count: usize = cell_chars.iter().filter(|&&c| c == 'C').count();
    let remaining_far: usize = cell_chars.iter().filter(|&&c| c == 'F').count();

    assert!(close_count > 0, "Close triangle should have overwritten far");
    assert_eq!(remaining_far, 0, "All far cells should be overwritten by closer triangle");

    // Verify z_buffer values where close triangle wrote
    for (i, &ch) in cell_chars.iter().enumerate() {
        if ch == 'C' {
            assert!(
                (z_buffer[i] - 0.8).abs() < 0.01,
                "Z-buffer at close cell should be 0.8, got {}",
                z_buffer[i]
            );
        }
    }
}

/// Farther triangle cannot overwrite closer one.
#[test]
fn farther_triangle_does_not_overwrite_closer() {
    let grid_w = 20;
    let grid_h = 10;
    let size = grid_w * grid_h;
    let mut z_buffer = vec![0.0_f32; size];
    let mut cell_chars = vec![' '; size];

    // Close triangle first at depth 0.8
    let close = [
        (8.0_f32, 2.0, 0.8),
        (14.0, 8.0, 0.8),
        (4.0, 8.0, 0.8),
    ];
    rasterize_triangle(&close, &mut z_buffer, grid_w, grid_h, 'C', &mut cell_chars);

    // Far triangle at depth 0.5 should NOT overwrite
    let far = [
        (8.0_f32, 2.0, 0.5),
        (14.0, 8.0, 0.5),
        (4.0, 8.0, 0.5),
    ];
    rasterize_triangle(&far, &mut z_buffer, grid_w, grid_h, 'F', &mut cell_chars);

    let far_count: usize = cell_chars.iter().filter(|&&c| c == 'F').count();
    assert_eq!(far_count, 0, "Farther triangle should not overwrite closer one");
}
