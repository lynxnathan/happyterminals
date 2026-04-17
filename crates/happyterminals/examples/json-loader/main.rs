//! JSON recipe loader — load a scene from a JSON file through the sandboxed loader.
//!
//! Parses `examples/recipes/hello.json` via `load_recipe_sandboxed()` with an
//! asset root pointed at `examples/models/`, builds a Scene, and renders it.
//! Demonstrates the safe path for untrusted JSON input (DSL-05 / DSL-08): path
//! traversal is rejected before any file I/O, ANSI escapes are stripped from
//! string props, and effect names resolve through a static registry.
//!
//! The recipe is minimal by design:
//!
//!     {
//!       "$version": "1.0",
//!       "camera": { "type": "orbit", "azimuth": 0.5, "elevation": 0.3, "distance": 5.0, "target": [0.0, 0.0, 0.0] },
//!       "layers": [{ "z_order": 0, "children": [{ "type": "cube", "transform": { "position": [0,0,0], "scale": [1,1,1] } }] }]
//!     }
//!
//! Features exercised:
//! - `happyterminals_dsl::json::load_recipe_sandboxed` (sandboxed JSON → `SceneIr`)
//! - `SandboxConfig` (`asset_root` + static `EffectRegistry`)
//! - `Scene::new` (assembling `SceneIr` + `CameraConfig` into a renderable `Scene`)
//! - `run_scene` driving `walk_and_render` for `Cube` / `Layer` nodes
//!
//! Controls:
//! - Ctrl-C or Q: quit
//!
//! Run from the workspace root:
//!
//!     cargo run --example json-loader -p happyterminals
//!
//! Why this exists:
//! A JSON recipe is the portable, LLM-friendly surface of happyterminals:
//! describe the scene in a version-stamped file, validate it against the
//! schema, then render it. This example closes the round-trip loop —
//! file → bytes → `SceneIr` → `Scene` → pixels. Today `walk_and_render` dispatches
//! Cube, Layer, and Group nodes; mesh recipe rendering is scheduled for post-v1.

use std::path::PathBuf;
use std::time::Duration;

use happyterminals::prelude::*;

const RECIPE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/recipes/hello.json"
);
const ASSET_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/models");

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the recipe off disk. Using runtime read_to_string (not include_str!)
    // keeps path resolution consistent with how model-viewer loads bunny.obj.
    let json = std::fs::read_to_string(RECIPE_PATH)
        .map_err(|e| format!("failed to read {RECIPE_PATH}: {e}"))?;

    // Sandbox config: asset_root pinned to examples/models/, default effect registry
    // (dissolve, slide-left, fade-to-black).
    let cfg = SandboxConfig {
        asset_root: PathBuf::from(ASSET_ROOT),
        ..SandboxConfig::default()
    };

    // Happy path: parse + sandbox-validate the recipe.
    let (ir, camera_config) = load_recipe_sandboxed(&json, &cfg)?;

    // ----- Educational: uncomment this block to see the sandbox reject a traversal path.
    // Left COMMENTED per D-07 so the example renders cleanly by default.
    // Uncommenting and running will print a one-line rejection to stderr and exit Ok.
    // -----
    // let bad = r#"{
    //   "$version": "1.0",
    //   "camera": { "type": "orbit", "azimuth": 0.0, "elevation": 0.0, "distance": 3.0, "target": [0.0, 0.0, 0.0] },
    //   "layers": [{ "z_order": 0, "children": [
    //     { "type": "mesh", "path": "../../etc/passwd" }
    //   ]}]
    // }"#;
    // match load_recipe_sandboxed(bad, &cfg) {
    //     Err(RecipeError::PathTraversal { path }) => {
    //         eprintln!("sandbox blocked path traversal: {path}");
    //     }
    //     Err(other) => eprintln!("sandbox rejected recipe: {other}"),
    //     Ok(_) => unreachable!("sandbox MUST reject path traversal"),
    // }

    // Build the Scene from (SceneIr, CameraConfig). No scene-level pipeline.
    // create_root gives us the Owner that drives reactive disposal; we don't
    // need reactive state here, but the Scene machinery expects the caller
    // to be inside a reactive root when constructing.
    let (scene_result, _owner) = create_root(|| Scene::new(ir, camera_config, None));
    let scene = scene_result?;

    run_scene(
        scene,
        FrameSpec {
            title: Some("happyterminals - JSON Loader".into()),
            ..FrameSpec::default()
        },
        |_dt: Duration, _input| {
            // Static scene — nothing to update per frame.
        },
    )
    .await
}
