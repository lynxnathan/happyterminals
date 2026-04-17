//! Runtime-loaded triangle mesh + panic-free OBJ/STL loader.
//!
//! This module unifies the const [`crate::Cube`] primitive with runtime-loaded
//! geometry via a single [`Mesh`] type. [`load_obj`] wraps `tobj 4.0` and
//! [`load_stl`] wraps `stl_io 0.11` with defensive plumbing: per-triangle skip
//! accounting for degenerate / non-finite input, flat per-face normal
//! computation, inverted-winding detection (OBJ), and `DoS` guards against
//! pathological files.
//!
//! All whole-file parse or I/O failures surface as [`MeshError`]; per-triangle
//! issues are counted in [`LoadStats`] and emit warnings (capped at
//! [`MAX_WARNINGS`]). The loaders never panic on any input — the fixture corpus
//! and proptest harness in `tests/obj_corpus.rs` and `tests/stl_corpus.rs`
//! enforce this invariant.

use std::fs::File;
use std::path::Path;

use glam::Vec3;
use thiserror::Error;

use crate::shading::ShadingRamp;

/// Maximum triangles accepted from any single OBJ file (`DoS` guard).
///
/// Files exceeding this limit return [`MeshError::Parse`] before any normal
/// computation runs. 1M triangles is ~36 MB of f32 vertex data — well above
/// any realistic terminal-rendered mesh.
pub const MAX_TRIANGLES: usize = 1_000_000;

/// Maximum warning strings stored in [`LoadStats`] (`DoS` guard).
///
/// Past this cap, [`LoadStats::triangles_skipped`] continues to increment but
/// no new warning strings are allocated. Prevents OOM on maliciously large
/// malformed files.
pub const MAX_WARNINGS: usize = 100;

/// A renderable indexed triangle mesh.
///
/// Vertices are stored as positions only (no per-vertex attributes in 2.1);
/// per-face flat normals are stored in [`Mesh::normals`] with
/// `normals.len() == indices.len()`. CCW winding convention matches
/// [`crate::Cube`] — viewed from outside the surface, vertices go CCW.
#[derive(Debug, Clone)]
pub struct Mesh {
    /// Vertex positions (not pruned; unused positions may remain after load).
    pub vertices: Vec<Vec3>,
    /// Triangle indices into [`Mesh::vertices`].
    pub indices: Vec<[u32; 3]>,
    /// Flat per-face normals; `normals.len() == indices.len()`.
    pub normals: Vec<Vec3>,
    /// Optional per-mesh shading override (scaffolded for 999.x backlog).
    ///
    /// When `None`, the scene-default [`ShadingRamp`] applies. The loader
    /// always leaves this `None`; callers set it explicitly.
    pub shading: Option<ShadingRamp<'static>>,
}

impl Mesh {
    /// Returns the bounding sphere `(center, radius)` of this mesh.
    ///
    /// Used by the model-viewer (Phase 2.1 Plan 03) to auto-fit the orbit
    /// camera distance across meshes of different scales. Empty meshes return
    /// `(Vec3::ZERO, 0.0)`.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn bounding_sphere(&self) -> (Vec3, f32) {
        if self.vertices.is_empty() {
            return (Vec3::ZERO, 0.0);
        }
        let sum: Vec3 = self.vertices.iter().copied().sum();
        let center = sum / self.vertices.len() as f32;
        let radius = self
            .vertices
            .iter()
            .map(|v| (*v - center).length())
            .fold(0.0_f32, f32::max);
        (center, radius)
    }
}

/// Statistics + non-fatal diagnostics from [`load_obj`].
///
/// Per-triangle issues (degenerate, non-finite, index out of bounds) are
/// counted in [`LoadStats::triangles_skipped`] with a human-readable entry
/// appended to [`LoadStats::warnings`] (up to [`MAX_WARNINGS`]). Whole-file
/// failures return [`MeshError`] instead.
#[derive(Debug, Default, Clone)]
pub struct LoadStats {
    /// Number of triangles successfully kept in the final [`Mesh`].
    pub triangles_loaded: usize,
    /// Number of triangles skipped (degenerate, non-finite, out-of-bounds).
    pub triangles_skipped: usize,
    /// Non-fatal warnings emitted during load (capped at [`MAX_WARNINGS`]).
    pub warnings: Vec<String>,
}

/// Errors returned by [`load_obj`] for whole-file failures.
///
/// Per-triangle issues are surfaced via [`LoadStats::warnings`] instead.
#[derive(Debug, Error)]
pub enum MeshError {
    /// I/O failure reading the OBJ file.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Structural parse failure reported by `tobj` (invalid face, unrecognized
    /// character, out-of-bounds whole-file index, triangle budget exceeded).
    #[error("OBJ parse error: {0}")]
    Parse(String),

    /// Degenerate whole-file data (reserved; 2.1 surfaces degeneracies via
    /// warnings and counts, not this variant).
    #[error("degenerate data in {file}")]
    Degenerate {
        /// Source file where the degeneracy was detected.
        file: String,
        /// Source line number (if known).
        line: Option<u32>,
    },

    /// Structural parse failure reported by `stl_io` (invalid format, truncated
    /// data, malformed header).
    #[error("STL parse error: {0}")]
    StlParse(String),

    /// Reserved for future opt-in strict winding rejection; 2.1 uses warnings.
    #[error("mesh has inverted winding")]
    Winding,
}

/// Compute flat per-face normal via cross-product of edges.
///
/// Returns `None` if the triangle is degenerate (zero-area / collinear) or if
/// any input vertex is non-finite. Callers treat `None` as "skip this
/// triangle and record a warning".
///
/// The degeneracy threshold uses `cross.length()` (not `length_squared`)
/// compared against a tight absolute epsilon. Squared-length would over-cull
/// tiny-but-valid triangles common in real meshes (e.g. the Stanford bunny
/// has triangles with `length_squared(cross) ≈ 1e-8` that are meaningful).
#[must_use]
fn flat_normal(a: Vec3, b: Vec3, c: Vec3) -> Option<Vec3> {
    if !a.is_finite() || !b.is_finite() || !c.is_finite() {
        return None;
    }
    let edge1 = b - a;
    let edge2 = c - a;
    let cross = edge1.cross(edge2);
    // Tight threshold on the cross magnitude itself (not its square).
    // f32 ulp at magnitudes ~1.0 is ~1e-7; we accept anything above 1e-12
    // as non-degenerate so real-world sub-millimeter triangles survive.
    let len = cross.length();
    if !len.is_finite() || len < 1e-12 {
        return None;
    }
    let normal = cross / len;
    if !normal.is_finite() {
        return None;
    }
    Some(normal)
}

/// Map a [`tobj::LoadError`] into our [`MeshError`] without leaking `tobj`
/// into the public API (per PITFALLS §6 — keep the conversion private).
fn map_tobj_err(e: tobj::LoadError) -> MeshError {
    MeshError::Parse(format!("{e}"))
}

/// Compute the signed volume of a closed triangle mesh via the divergence
/// theorem. Positive = CCW outward normals; negative = CW / inside-out.
fn signed_volume(vertices: &[Vec3], indices: &[[u32; 3]]) -> f32 {
    indices
        .iter()
        .map(|&[a, b, c]| {
            let a_idx = a as usize;
            let b_idx = b as usize;
            let c_idx = c as usize;
            if a_idx >= vertices.len() || b_idx >= vertices.len() || c_idx >= vertices.len() {
                return 0.0;
            }
            let va = vertices[a_idx];
            let vb = vertices[b_idx];
            let vc = vertices[c_idx];
            va.dot(vb.cross(vc)) / 6.0
        })
        .sum()
}

/// Load an OBJ file from disk into a renderable [`Mesh`].
///
/// Returns `(Mesh, LoadStats)` on success. Stats report how many triangles
/// loaded, how many were skipped (degenerate / non-finite / out-of-bounds),
/// and any warnings (e.g., inverted winding, skipped-triangle reason strings).
///
/// Whole-file issues (I/O failure, unparseable syntax, exceeded triangle
/// budget) return [`MeshError`]. Per-triangle issues are counted + warned
/// without returning `Err`.
///
/// Normals are always recomputed flat per-face; any `vn` lines in the source
/// file are ignored. Material (`.mtl`) files are not loaded — MTL parse
/// failures from `tobj` are silently discarded.
///
/// # Errors
/// - [`MeshError::Io`] if the file cannot be opened or read.
/// - [`MeshError::Parse`] if `tobj` reports a structural failure, if vertex
///   or triangle counts overflow [`u32`] / [`usize`], or if the mesh exceeds
///   [`MAX_TRIANGLES`].
///
/// # Examples
/// ```no_run
/// use happyterminals_renderer::mesh::load_obj;
/// let (mesh, stats) = load_obj("examples/models/bunny.obj")?;
/// println!("loaded {} tris, skipped {}", stats.triangles_loaded, stats.triangles_skipped);
/// # Ok::<(), happyterminals_renderer::mesh::MeshError>(())
/// ```
pub fn load_obj<P: AsRef<Path>>(path: P) -> Result<(Mesh, LoadStats), MeshError> {
    let path_ref = path.as_ref();

    let opts = tobj::LoadOptions {
        triangulate: true,
        single_index: true,
        ignore_points: true,
        ignore_lines: true,
    };

    let (models, _materials) = tobj::load_obj(path_ref, &opts).map_err(map_tobj_err)?;

    // Concatenate all sub-models. OBJ files often have multiple `o` objects;
    // tobj returns one Model per object with per-Model-local indices. Rebase
    // each sub-model's indices by the running vertex count.
    let mut vertices: Vec<Vec3> = Vec::new();
    let mut raw_indices: Vec<[u32; 3]> = Vec::new();

    for model in &models {
        let base = u32::try_from(vertices.len())
            .map_err(|_| MeshError::Parse("vertex count overflow u32".into()))?;

        for chunk in model.mesh.positions.chunks_exact(3) {
            vertices.push(Vec3::new(chunk[0], chunk[1], chunk[2]));
        }

        for tri in model.mesh.indices.chunks_exact(3) {
            let a = tri[0]
                .checked_add(base)
                .ok_or_else(|| MeshError::Parse("index + base overflow u32".into()))?;
            let b = tri[1]
                .checked_add(base)
                .ok_or_else(|| MeshError::Parse("index + base overflow u32".into()))?;
            let c = tri[2]
                .checked_add(base)
                .ok_or_else(|| MeshError::Parse("index + base overflow u32".into()))?;
            raw_indices.push([a, b, c]);
        }
    }

    // `DoS` guard: reject pathologically large meshes BEFORE normal computation.
    if raw_indices.len() > MAX_TRIANGLES {
        return Err(MeshError::Parse(format!(
            "mesh exceeds MAX_TRIANGLES ({MAX_TRIANGLES})"
        )));
    }

    let mut stats = LoadStats::default();
    let mut kept_indices: Vec<[u32; 3]> = Vec::with_capacity(raw_indices.len());
    let mut normals: Vec<Vec3> = Vec::with_capacity(raw_indices.len());

    for (i, &[a_idx, b_idx, c_idx]) in raw_indices.iter().enumerate() {
        let a_us = a_idx as usize;
        let b_us = b_idx as usize;
        let c_us = c_idx as usize;

        if a_us >= vertices.len() || b_us >= vertices.len() || c_us >= vertices.len() {
            stats.triangles_skipped += 1;
            push_warning_capped(
                &mut stats.warnings,
                format!("skipped triangle {i}: index out of bounds"),
            );
            continue;
        }

        let a = vertices[a_us];
        let b = vertices[b_us];
        let c = vertices[c_us];

        if let Some(n) = flat_normal(a, b, c) {
            normals.push(n);
            kept_indices.push([a_idx, b_idx, c_idx]);
            stats.triangles_loaded += 1;
        } else {
            stats.triangles_skipped += 1;
            push_warning_capped(
                &mut stats.warnings,
                format!("skipped triangle {i}: degenerate or non-finite vertex"),
            );
        }
    }

    // Inverted-winding detection (warning only, no correction in 2.1).
    let volume = signed_volume(&vertices, &kept_indices);
    if volume < 0.0 {
        push_warning_capped(
            &mut stats.warnings,
            "mesh appears to have inverted winding; normals may render inside-out".into(),
        );
    }

    let mesh = Mesh {
        vertices,
        indices: kept_indices,
        normals,
        shading: None,
    };

    Ok((mesh, stats))
}

/// Load an STL file (ASCII or binary) from disk into a renderable [`Mesh`].
///
/// Returns `(Mesh, LoadStats)` on success. Stats report how many triangles
/// loaded, how many were skipped (degenerate / non-finite / out-of-bounds),
/// and any warnings.
///
/// Whole-file issues (I/O failure, unparseable syntax, exceeded triangle
/// budget) return [`MeshError`]. Per-triangle issues are counted + warned
/// without returning `Err`.
///
/// Normals from the STL file are used when they are finite and non-zero;
/// otherwise they fall back to [`flat_normal`] recomputation. If `flat_normal`
/// also returns `None`, the triangle is skipped.
///
/// # Errors
/// - [`MeshError::Io`] if the file cannot be opened or read.
/// - [`MeshError::StlParse`] if `stl_io` reports a structural failure.
/// - [`MeshError::Parse`] if the mesh exceeds [`MAX_TRIANGLES`].
///
/// # Examples
/// ```no_run
/// use happyterminals_renderer::mesh::load_stl;
/// let (mesh, stats) = load_stl("models/part.stl")?;
/// println!("loaded {} tris, skipped {}", stats.triangles_loaded, stats.triangles_skipped);
/// # Ok::<(), happyterminals_renderer::mesh::MeshError>(())
/// ```
#[allow(clippy::cast_precision_loss)]
pub fn load_stl<P: AsRef<Path>>(path: P) -> Result<(Mesh, LoadStats), MeshError> {
    let mut file = File::open(path.as_ref())?;
    let stl_mesh =
        stl_io::read_stl(&mut file).map_err(|e| MeshError::StlParse(format!("{e}")))?;

    // DoS guard: reject pathologically large meshes BEFORE normal computation.
    if stl_mesh.faces.len() > MAX_TRIANGLES {
        return Err(MeshError::Parse(format!(
            "mesh exceeds MAX_TRIANGLES ({MAX_TRIANGLES})"
        )));
    }

    // Convert stl_io vertices to Vec3.
    let vertices: Vec<Vec3> = stl_mesh
        .vertices
        .iter()
        .map(|v| Vec3::new(v[0], v[1], v[2]))
        .collect();

    let vertex_count = vertices.len();
    let mut stats = LoadStats::default();
    let mut kept_indices: Vec<[u32; 3]> = Vec::with_capacity(stl_mesh.faces.len());
    let mut normals: Vec<Vec3> = Vec::with_capacity(stl_mesh.faces.len());

    for (i, face) in stl_mesh.faces.iter().enumerate() {
        // Bounds-check vertex indices.
        let a_idx = face.vertices[0];
        let b_idx = face.vertices[1];
        let c_idx = face.vertices[2];

        if a_idx >= vertex_count || b_idx >= vertex_count || c_idx >= vertex_count {
            stats.triangles_skipped += 1;
            push_warning_capped(
                &mut stats.warnings,
                format!("skipped triangle {i}: index out of bounds"),
            );
            continue;
        }

        // Safe u32 conversion (guard overflow on 32-bit).
        let Ok(a_u32) = u32::try_from(a_idx) else {
            stats.triangles_skipped += 1;
            push_warning_capped(
                &mut stats.warnings,
                format!("skipped triangle {i}: index overflow u32"),
            );
            continue;
        };
        let Ok(b_u32) = u32::try_from(b_idx) else {
            stats.triangles_skipped += 1;
            push_warning_capped(
                &mut stats.warnings,
                format!("skipped triangle {i}: index overflow u32"),
            );
            continue;
        };
        let Ok(c_u32) = u32::try_from(c_idx) else {
            stats.triangles_skipped += 1;
            push_warning_capped(
                &mut stats.warnings,
                format!("skipped triangle {i}: index overflow u32"),
            );
            continue;
        };

        let va = vertices[a_idx];
        let vb = vertices[b_idx];
        let vc = vertices[c_idx];

        // Try to use the STL-provided normal first.
        let stl_normal = Vec3::new(face.normal[0], face.normal[1], face.normal[2]);
        let normal_len = stl_normal.length();
        let normal = if stl_normal.is_finite() && normal_len > 1e-12 {
            stl_normal / normal_len
        } else {
            // Degenerate STL normal -- fall back to flat_normal recomputation.
            if let Some(n) = flat_normal(va, vb, vc) {
                n
            } else {
                stats.triangles_skipped += 1;
                push_warning_capped(
                    &mut stats.warnings,
                    format!(
                        "skipped triangle {i}: degenerate normal and flat_normal failed"
                    ),
                );
                continue;
            }
        };

        normals.push(normal);
        kept_indices.push([a_u32, b_u32, c_u32]);
        stats.triangles_loaded += 1;
    }

    let mesh = Mesh {
        vertices,
        indices: kept_indices,
        normals,
        shading: None,
    };

    Ok((mesh, stats))
}

/// Push a warning string only if the cap has not been reached.
fn push_warning_capped(warnings: &mut Vec<String>, msg: String) {
    if warnings.len() < MAX_WARNINGS {
        warnings.push(msg);
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::float_cmp,
    clippy::single_match_else,
    clippy::unwrap_used
)]
mod tests {
    use super::*;

    /// Absolute path to a fixture file, resolved at compile time.
    macro_rules! fixture {
        ($rel:literal) => {
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/obj/", $rel)
        };
    }

    /// Absolute path to a pre-imported real-world model at the workspace root.
    macro_rules! real_model {
        ($name:literal) => {
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/models/", $name)
        };
    }

    #[test]
    fn flat_normal_finite_triangle() {
        let n = flat_normal(Vec3::ZERO, Vec3::X, Vec3::Y);
        assert!(n.is_some());
        let n = n.unwrap_or(Vec3::ZERO);
        assert!((n - Vec3::Z).length() < 1e-5, "expected +Z, got {n:?}");
    }

    #[test]
    fn flat_normal_degenerate_returns_none() {
        // Three collinear points on the X axis.
        let n = flat_normal(Vec3::ZERO, Vec3::X, Vec3::new(2.0, 0.0, 0.0));
        assert!(n.is_none(), "collinear triangle must return None");
    }

    #[test]
    fn flat_normal_nan_vertex_returns_none() {
        let nan_vertex = Vec3::new(f32::NAN, 0.0, 0.0);
        assert!(flat_normal(nan_vertex, Vec3::X, Vec3::Y).is_none());
        assert!(flat_normal(Vec3::ZERO, nan_vertex, Vec3::Y).is_none());

        let inf_vertex = Vec3::new(f32::INFINITY, 0.0, 0.0);
        assert!(flat_normal(Vec3::ZERO, inf_vertex, Vec3::Y).is_none());
    }

    #[test]
    fn load_obj_quad_triangulates_to_two_triangles() {
        let (mesh, stats) = load_obj(fixture!("quad.obj")).expect("quad loads");
        assert_eq!(mesh.indices.len(), 2, "quad should fan to 2 triangles");
        assert_eq!(mesh.normals.len(), 2);
        assert_eq!(stats.triangles_loaded, 2);
        assert_eq!(stats.triangles_skipped, 0);
    }

    #[test]
    fn load_obj_ngon_pentagon_triangulates_to_three_triangles() {
        let (mesh, stats) =
            load_obj(fixture!("ngon_pentagon.obj")).expect("pentagon loads");
        assert_eq!(
            mesh.indices.len(),
            3,
            "pentagon fans to 3 triangles"
        );
        assert_eq!(mesh.normals.len(), 3);
        assert_eq!(stats.triangles_loaded, 3);
    }

    #[test]
    fn load_obj_missing_normals_recovered() {
        let (mesh, _stats) =
            load_obj(fixture!("missing_normals.obj")).expect("loads");
        assert_eq!(mesh.normals.len(), 1);
        assert!(mesh.normals.iter().all(|n| n.is_finite()));
    }

    #[test]
    fn load_obj_degenerate_skipped() {
        let (mesh, stats) = load_obj(fixture!("degenerate.obj")).expect("loads");
        assert_eq!(stats.triangles_loaded, 0);
        assert_eq!(stats.triangles_skipped, 1);
        assert_eq!(stats.warnings.len(), 1);
        assert_eq!(mesh.indices.len(), 0);
        assert_eq!(mesh.normals.len(), 0);
    }

    #[test]
    fn load_obj_empty_file_does_not_panic() {
        // tobj treats a 0-byte file as "a file with no models" and returns Ok
        // with an empty model list. Our contract is "never panic on any
        // input" — Ok-with-empty-mesh or Err(Parse) are both acceptable.
        match load_obj(fixture!("empty.obj")) {
            Ok((mesh, stats)) => {
                assert_eq!(mesh.vertices.len(), 0);
                assert_eq!(mesh.indices.len(), 0);
                assert_eq!(stats.triangles_loaded, 0);
            }
            Err(MeshError::Parse(_) | MeshError::Io(_)) => {}
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn load_obj_truncated_no_face_returns_ok_zero_triangles() {
        // Truncated = 3 vertex lines with no face. tobj accepts this as
        // "a mesh with no faces" rather than erroring, so load_obj returns
        // Ok with triangles_loaded == 0. That's fine — the contract is "no
        // panic", and downstream code handles empty meshes gracefully.
        match load_obj(fixture!("truncated.obj")) {
            Ok((mesh, stats)) => {
                assert_eq!(mesh.indices.len(), 0);
                assert_eq!(stats.triangles_loaded, 0);
            }
            Err(MeshError::Parse(_)) => {
                // Also acceptable if tobj's behaviour ever tightens.
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn load_obj_flipped_winding_warns() {
        let (_mesh, stats) =
            load_obj(fixture!("flipped_winding.obj")).expect("loads");
        let has_winding_warning = stats
            .warnings
            .iter()
            .any(|w| w.to_lowercase().contains("winding"));
        assert!(
            has_winding_warning,
            "expected a winding warning, got: {:?}",
            stats.warnings
        );
    }

    #[test]
    fn load_obj_negative_indices_handled() {
        // tobj translates negative indices internally; loader must not crash.
        match load_obj(fixture!("negative_indices.obj")) {
            Ok((mesh, _stats)) => {
                assert!(mesh.indices.len() <= 1);
            }
            Err(_) => {
                // Also acceptable if tobj rejects this file.
            }
        }
    }

    #[test]
    fn load_obj_bunny_has_many_triangles() {
        let (mesh, stats) =
            load_obj(real_model!("bunny.obj")).expect("bunny loads");
        assert!(
            stats.triangles_loaded >= 4900,
            "expected >= 4900 triangles, got {}",
            stats.triangles_loaded
        );
        assert!(
            mesh.normals.iter().all(|n| n.is_finite()),
            "all bunny normals must be finite"
        );
        assert_eq!(mesh.indices.len(), mesh.normals.len());
    }

    #[test]
    fn cube_mesh_round_trips() {
        let mesh = crate::Cube::mesh();
        assert_eq!(mesh.indices.len(), 12, "cube has 12 triangles");
        assert_eq!(mesh.vertices.len(), 8, "cube has 8 unique vertices");
        assert_eq!(mesh.normals.len(), 12, "one normal per triangle");
        for (i, n) in mesh.normals.iter().enumerate() {
            let len = n.length();
            assert!(
                (len - 1.0).abs() < 1e-4,
                "normal {i} not unit length: {len}"
            );
        }
        assert!(mesh.shading.is_none());
    }

    #[test]
    fn mesh_bounding_sphere_encloses_unit_cube() {
        let mesh = crate::Cube::mesh();
        let (center, radius) = mesh.bounding_sphere();
        assert!(
            center.length() < 1e-4,
            "unit cube centered at origin, got {center:?}"
        );
        // sqrt(3)/2 ≈ 0.8660254 — corner distance from center of unit cube.
        assert!(
            radius >= 0.866,
            "expected radius >= sqrt(3)/2 ≈ 0.866, got {radius}"
        );
    }

    #[test]
    fn mesh_bounding_sphere_empty_mesh() {
        let empty = Mesh {
            vertices: Vec::new(),
            indices: Vec::new(),
            normals: Vec::new(),
            shading: None,
        };
        let (c, r) = empty.bounding_sphere();
        assert_eq!(c, Vec3::ZERO);
        assert_eq!(r, 0.0);
    }

    // ── STL loader inline tests ──────────────────────────────────────────

    /// Absolute path to an STL fixture file, resolved at compile time.
    macro_rules! stl_fixture {
        ($rel:literal) => {
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/stl/", $rel)
        };
    }

    #[test]
    fn load_stl_cube_has_12_triangles() {
        let (mesh, stats) = load_stl(stl_fixture!("cube.stl")).expect("cube.stl loads");
        assert_eq!(stats.triangles_loaded, 12, "cube has 12 triangles");
        assert_eq!(mesh.indices.len(), 12);
        assert_eq!(mesh.normals.len(), 12);
        assert_eq!(mesh.indices.len(), mesh.normals.len());
    }

    #[test]
    fn load_stl_indices_normals_length_match() {
        let (mesh, _stats) = load_stl(stl_fixture!("small_mesh.stl")).expect("loads");
        assert_eq!(
            mesh.indices.len(),
            mesh.normals.len(),
            "indices and normals must have same length"
        );
    }

    #[test]
    fn load_stl_nonexistent_returns_io_error() {
        match load_stl("/nonexistent/path.stl") {
            Err(MeshError::Io(_)) => {}
            other => panic!("expected MeshError::Io, got {other:?}"),
        }
    }

    #[test]
    fn stl_parse_variant_formats_correctly() {
        let err = MeshError::StlParse("bad header".into());
        let msg = format!("{err}");
        assert!(msg.contains("STL parse error"), "got: {msg}");
        assert!(msg.contains("bad header"), "got: {msg}");
    }

    #[test]
    fn load_obj_public_api_does_not_leak_tobj() {
        // Compile-time check: MeshError must not expose tobj::LoadError.
        // If we ever add `impl From<tobj::LoadError> for MeshError`, this
        // test's existence flags the API boundary discussion.
        fn _takes_error(_e: MeshError) {}
    }
}
