//! Phase 2.1 OBJ corpus + proptest byte-fuzz harness.
//!
//! Enforces the "load_obj never panics on any input" contract against:
//! - 3 real-world models (bunny, cow, teapot) at the workspace root
//! - 8 synthetic quirk fixtures under `tests/fixtures/obj/`
//! - `PROPTEST_CASES` (default 256 in CI; set `PROPTEST_CASES=10000` for the
//!   overnight sweep) arbitrary byte sequences written to a temp file.
//!
//! Any panic fails the corresponding test. Parse errors are always acceptable —
//! the contract is *no panics*, not *no errors*.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use happyterminals_renderer::{Cube, load_obj};

/// Real + synthetic corpus (11 files total) — see PLAN 02.1-01 §must_haves.
///
/// Real models live at workspace-root `examples/models/`, so relative to this
/// crate they sit at `../../examples/models/`. Synthetic fixtures live next to
/// this test file in `tests/fixtures/obj/`.
const CORPUS: &[&str] = &[
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/models/bunny.obj"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/models/cow.obj"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/models/teapot.obj"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/obj/quad.obj"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/obj/ngon_pentagon.obj"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/obj/missing_normals.obj"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/obj/degenerate.obj"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/obj/empty.obj"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/obj/truncated.obj"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/obj/flipped_winding.obj"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/obj/negative_indices.obj"),
];

#[test]
fn corpus_never_panics() {
    assert_eq!(CORPUS.len(), 11, "corpus must have 11 entries");
    for path in CORPUS {
        match load_obj(path) {
            Ok((mesh, _stats)) => {
                assert_eq!(
                    mesh.indices.len(),
                    mesh.normals.len(),
                    "{path}: indices/normals length mismatch"
                );
                for n in &mesh.normals {
                    assert!(
                        n.is_finite(),
                        "{path}: non-finite normal {n:?}"
                    );
                }
            }
            Err(_) => {
                // Acceptable — the contract is "no panic", not "no error".
            }
        }
    }
}

#[test]
fn corpus_loads_bunny() {
    let bunny_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/models/bunny.obj"
    );
    let (mesh, stats) = load_obj(bunny_path).expect("bunny loads");
    assert!(
        stats.triangles_loaded >= 4900,
        "expected >= 4900 triangles, got {}",
        stats.triangles_loaded
    );
    assert_eq!(mesh.indices.len(), mesh.normals.len());
}

#[test]
fn corpus_cube_mesh_round_trips() {
    let mesh = Cube::mesh();
    assert_eq!(mesh.indices.len(), 12);
    assert_eq!(mesh.vertices.len(), 8);
    assert_eq!(mesh.normals.len(), 12);
    for (i, n) in mesh.normals.iter().enumerate() {
        let len = n.length();
        assert!(
            (len - 1.0).abs() < 1e-4,
            "normal {i} not unit length: {len}"
        );
    }
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config {
        // 256 cases keeps this under the VALIDATION.md 30s sampling budget.
        // For the overnight 10k sweep, run with `PROPTEST_CASES=10000`.
        cases: 256,
        .. proptest::test_runner::Config::default()
    })]

    /// load_obj must not panic on arbitrary byte sequences. Parse errors are
    /// fine; the harness only fails if tobj or our mapping panics.
    #[test]
    fn load_obj_never_panics_on_arbitrary_bytes(data: Vec<u8>) {
        let tmp = tempfile::NamedTempFile::new().expect("tmpfile");
        std::fs::write(tmp.path(), &data).expect("write tmpfile");
        let _ = load_obj(tmp.path());
    }
}
