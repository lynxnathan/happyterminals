//! Phase 2.5 STL corpus + proptest byte-fuzz harness.
//!
//! Enforces the "load_stl never panics on any input" contract against:
//! - 2 synthetic fixtures under `tests/fixtures/stl/`
//! - `PROPTEST_CASES` (default 256 in CI) arbitrary byte sequences written to
//!   a temp file.
//!
//! Any panic fails the corresponding test. Parse errors are always acceptable --
//! the contract is *no panics*, not *no errors*.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use happyterminals_renderer::{load_stl, MeshError};

/// STL fixture corpus -- synthetic files under `tests/fixtures/stl/`.
const STL_CORPUS: &[&str] = &[
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/stl/cube.stl"),
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/stl/small_mesh.stl"
    ),
];

#[test]
fn stl_corpus_never_panics() {
    assert_eq!(STL_CORPUS.len(), 2, "STL corpus must have 2 entries");
    for path in STL_CORPUS {
        match load_stl(path) {
            Ok((mesh, _stats)) => {
                assert_eq!(
                    mesh.indices.len(),
                    mesh.normals.len(),
                    "{path}: indices/normals length mismatch"
                );
                for n in &mesh.normals {
                    assert!(n.is_finite(), "{path}: non-finite normal {n:?}");
                }
            }
            Err(_) => {
                // Acceptable -- the contract is "no panic", not "no error".
            }
        }
    }
}

#[test]
fn stl_normals_are_finite() {
    for path in STL_CORPUS {
        let (mesh, _stats) = load_stl(path).unwrap_or_else(|e| panic!("{path}: {e}"));
        for (i, n) in mesh.normals.iter().enumerate() {
            assert!(
                n.is_finite(),
                "{path}: normal {i} is not finite: {n:?}"
            );
            let len = n.length();
            assert!(
                (len - 1.0).abs() < 1e-4,
                "{path}: normal {i} not unit length: {len}"
            );
        }
    }
}

#[test]
fn stl_parse_error_on_nonexistent_file() {
    let result = load_stl("/nonexistent/path/to/file.stl");
    assert!(result.is_err(), "nonexistent file should return Err");
    match result {
        Err(MeshError::Io(_)) => {} // expected
        other => panic!("expected MeshError::Io, got {other:?}"),
    }
}

#[test]
fn stl_cube_has_correct_counts() {
    let cube_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/stl/cube.stl"
    );
    let (mesh, stats) = load_stl(cube_path).expect("cube.stl must load");
    assert_eq!(stats.triangles_loaded, 12, "cube has 12 triangles");
    assert_eq!(mesh.indices.len(), 12);
    assert_eq!(mesh.normals.len(), 12);
    assert_eq!(mesh.indices.len(), mesh.normals.len());
}

#[test]
fn stl_small_mesh_loads() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/stl/small_mesh.stl"
    );
    let (mesh, stats) = load_stl(path).expect("small_mesh.stl must load");
    assert_eq!(stats.triangles_loaded, 4, "pyramid base has 4 triangles");
    assert_eq!(mesh.indices.len(), mesh.normals.len());
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config {
        cases: 256,
        .. proptest::test_runner::Config::default()
    })]

    /// load_stl must not panic on arbitrary byte sequences.
    #[test]
    fn load_stl_never_panics_on_arbitrary_bytes(data: Vec<u8>) {
        let tmp = tempfile::NamedTempFile::new().expect("tmpfile");
        std::fs::write(tmp.path(), &data).expect("write tmpfile");
        let _ = load_stl(tmp.path());
    }
}
