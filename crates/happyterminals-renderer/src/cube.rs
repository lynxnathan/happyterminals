//! Unit cube primitive for the ASCII 3D renderer.
//!
//! Defines a cube centered at the origin with side length 1.0:
//! 8 vertices, 12 triangles (2 per face), 6 axis-aligned face normals.
//! All winding is counter-clockwise when viewed from outside the cube.

use glam::Vec3;

/// A unit cube primitive centered at the origin.
///
/// The cube spans from (-0.5, -0.5, -0.5) to (0.5, 0.5, 0.5).
/// Each face is split into two triangles with CCW winding (viewed from outside).
pub struct Cube;

impl Cube {
    /// 8 vertices of the unit cube, centered at origin.
    pub const VERTICES: [Vec3; 8] = [
        Vec3::new(-0.5, -0.5, -0.5), // 0: left-bottom-back
        Vec3::new(0.5, -0.5, -0.5),  // 1: right-bottom-back
        Vec3::new(0.5, 0.5, -0.5),   // 2: right-top-back
        Vec3::new(-0.5, 0.5, -0.5),  // 3: left-top-back
        Vec3::new(-0.5, -0.5, 0.5),  // 4: left-bottom-front
        Vec3::new(0.5, -0.5, 0.5),   // 5: right-bottom-front
        Vec3::new(0.5, 0.5, 0.5),    // 6: right-top-front
        Vec3::new(-0.5, 0.5, 0.5),   // 7: left-top-front
    ];

    /// 12 triangles (2 per face), CCW winding when viewed from outside.
    /// Each triple indexes into [`Self::VERTICES`].
    pub const INDICES: [[usize; 3]; 12] = [
        // Front face (+Z)
        [4, 5, 6],
        [4, 6, 7],
        // Back face (-Z)
        [1, 0, 3],
        [1, 3, 2],
        // Right face (+X)
        [5, 1, 2],
        [5, 2, 6],
        // Left face (-X)
        [0, 4, 7],
        [0, 7, 3],
        // Top face (+Y)
        [7, 6, 2],
        [7, 2, 3],
        // Bottom face (-Y)
        [0, 1, 5],
        [0, 5, 4],
    ];

    /// One normal per face (pairs of triangles share a normal).
    /// Order: front, back, right, left, top, bottom.
    pub const FACE_NORMALS: [Vec3; 6] = [
        Vec3::new(0.0, 0.0, 1.0),  // front (+Z)
        Vec3::new(0.0, 0.0, -1.0), // back (-Z)
        Vec3::new(1.0, 0.0, 0.0),  // right (+X)
        Vec3::new(-1.0, 0.0, 0.0), // left (-X)
        Vec3::new(0.0, 1.0, 0.0),  // top (+Y)
        Vec3::new(0.0, -1.0, 0.0), // bottom (-Y)
    ];

    /// Returns the face normal index for a given triangle index.
    ///
    /// Each face has 2 triangles, so triangle `i` belongs to face `i / 2`.
    #[must_use]
    pub const fn face_normal_index(triangle_index: usize) -> usize {
        triangle_index / 2
    }

    /// Construct a heap-allocated [`Mesh`](crate::mesh::Mesh) equivalent to
    /// the `Cube` primitive.
    ///
    /// Unifies the const-data cube with runtime-loaded OBJ meshes so the
    /// rasterizer can consume `&Mesh` uniformly (refactor lands in Plan 02).
    /// Every triangle gets its owning face's normal — 12 triangles, 12
    /// normals, 8 unique vertex positions.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn mesh() -> crate::mesh::Mesh {
        let vertices = Self::VERTICES.to_vec();
        let indices: Vec<[u32; 3]> = Self::INDICES
            .iter()
            .map(|&[a, b, c]| [a as u32, b as u32, c as u32])
            .collect();
        let normals: Vec<glam::Vec3> = (0..Self::INDICES.len())
            .map(|tri_idx| Self::FACE_NORMALS[Self::face_normal_index(tri_idx)])
            .collect();
        crate::mesh::Mesh {
            vertices,
            indices,
            normals,
            shading: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertex_count_is_eight() {
        assert_eq!(Cube::VERTICES.len(), 8);
    }

    #[test]
    fn index_count_is_twelve_triangles() {
        assert_eq!(Cube::INDICES.len(), 12);
        // Total index values = 36
        let total_indices: usize = Cube::INDICES.len() * 3;
        assert_eq!(total_indices, 36);
    }

    #[test]
    fn face_normal_count_is_six() {
        assert_eq!(Cube::FACE_NORMALS.len(), 6);
    }

    #[test]
    fn all_normals_are_unit_length() {
        for (i, normal) in Cube::FACE_NORMALS.iter().enumerate() {
            let len = normal.length();
            assert!(
                (len - 1.0).abs() < f32::EPSILON,
                "Normal {i} has length {len}, expected 1.0"
            );
        }
    }

    #[test]
    fn all_normals_are_axis_aligned() {
        for (i, normal) in Cube::FACE_NORMALS.iter().enumerate() {
            let abs = normal.abs();
            // Exactly one component should be 1.0, the others 0.0
            let ones = [abs.x, abs.y, abs.z]
                .iter()
                .filter(|&&v| (v - 1.0).abs() < f32::EPSILON)
                .count();
            let zeros = [abs.x, abs.y, abs.z]
                .iter()
                .filter(|&&v| v.abs() < f32::EPSILON)
                .count();
            assert_eq!(
                ones, 1,
                "Normal {i} should have exactly one +/-1.0 component"
            );
            assert_eq!(
                zeros, 2,
                "Normal {i} should have exactly two 0.0 components"
            );
        }
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn vertices_centered_at_origin() {
        let sum: Vec3 = Cube::VERTICES.iter().copied().sum();
        let center = sum / Cube::VERTICES.len() as f32;
        assert!(
            center.length() < f32::EPSILON,
            "Center of mass should be at origin, got {center}"
        );
    }

    #[test]
    fn all_indices_in_range() {
        for (tri_idx, tri) in Cube::INDICES.iter().enumerate() {
            for &idx in tri {
                assert!(
                    idx < Cube::VERTICES.len(),
                    "Triangle {tri_idx} has out-of-range index {idx}"
                );
            }
        }
    }

    #[test]
    fn face_normal_index_maps_correctly() {
        // Triangles 0,1 -> face 0 (front)
        assert_eq!(Cube::face_normal_index(0), 0);
        assert_eq!(Cube::face_normal_index(1), 0);
        // Triangles 2,3 -> face 1 (back)
        assert_eq!(Cube::face_normal_index(2), 1);
        assert_eq!(Cube::face_normal_index(3), 1);
        // Last triangle -> face 5 (bottom)
        assert_eq!(Cube::face_normal_index(11), 5);
    }
}
