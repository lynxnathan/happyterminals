//! JSON recipe loader, schema generation, and validation for happyterminals scenes.
//!
//! Provides [`load_recipe`] to parse and validate a JSON recipe string into a
//! [`SceneIr`], [`recipe_schema`] to generate a JSON Schema document describing
//! the recipe format, and [`RecipeError`] for structured error reporting.

use std::collections::HashMap;
use std::fmt;

use glam::Vec3;
use happyterminals_renderer::camera::{FpsCamera, FreeLookCamera, OrbitCamera};
use happyterminals_scene::node::{NodeId, NodeKind, SceneNode, Transform};
use happyterminals_scene::prop::PropValue;
use happyterminals_scene::{CameraConfig, SceneIr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The current recipe format version.
const RECIPE_VERSION: &str = "1.0";

// ── Error type ──────────────────────────────────────────────────────────

/// Errors produced by [`load_recipe`].
#[derive(Debug)]
pub enum RecipeError {
    /// The recipe specifies an unsupported format version.
    UnsupportedVersion {
        /// The version string found in the recipe.
        found: String,
    },
    /// The recipe JSON failed schema validation.
    SchemaValidation(String),
    /// The recipe JSON could not be deserialized.
    Deserialize(String),
}

impl fmt::Display for RecipeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedVersion { found } => {
                write!(f, "unsupported recipe version: {found}")
            }
            Self::SchemaValidation(msg) => write!(f, "schema validation failed: {msg}"),
            Self::Deserialize(msg) => write!(f, "deserialization failed: {msg}"),
        }
    }
}

impl std::error::Error for RecipeError {}

// ── Recipe serde types ──────────────────────────────────────────────────

/// Root recipe structure deserialized from JSON.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Recipe {
    /// Recipe format version (must be "1.0").
    #[serde(rename = "$version")]
    pub version: String,
    /// Camera configuration for the scene.
    pub camera: RecipeCamera,
    /// Compositing layers containing scene nodes.
    pub layers: Vec<RecipeLayer>,
}

/// Camera configuration as a tagged enum.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum RecipeCamera {
    /// Orbit camera with spherical coordinates around a target.
    Orbit {
        /// Azimuth angle in radians.
        azimuth: f32,
        /// Elevation angle in radians.
        elevation: f32,
        /// Distance from target.
        distance: f32,
        /// Look-at target as [x, y, z].
        target: [f32; 3],
    },
    /// Free-look camera with 6 degrees of freedom.
    #[serde(rename = "freelook")]
    FreeLook {
        /// Position as [x, y, z].
        position: [f32; 3],
        /// Yaw in radians.
        yaw: f32,
        /// Pitch in radians.
        pitch: f32,
        /// Movement speed.
        speed: f32,
    },
    /// First-person camera locked to a ground plane.
    Fps {
        /// Position as [x, y, z].
        position: [f32; 3],
        /// Yaw in radians.
        yaw: f32,
        /// Pitch in radians.
        pitch: f32,
        /// Movement speed.
        speed: f32,
        /// Ground Y coordinate.
        ground_y: f32,
    },
}

/// A compositing layer containing child nodes.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RecipeLayer {
    /// Z-order for compositing (lower = further back).
    pub z_order: i16,
    /// Child nodes within this layer.
    pub children: Vec<RecipeNode>,
}

/// A scene node as a tagged enum.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum RecipeNode {
    /// A cube primitive.
    Cube {
        /// Optional transform override.
        #[serde(default)]
        transform: Option<RecipeTransform>,
        /// Optional property map.
        #[serde(default)]
        props: Option<HashMap<String, serde_json::Value>>,
    },
    /// A mesh loaded from a file path.
    Mesh {
        /// Path to the mesh file.
        path: String,
        /// Optional transform override.
        #[serde(default)]
        transform: Option<RecipeTransform>,
        /// Optional property map.
        #[serde(default)]
        props: Option<HashMap<String, serde_json::Value>>,
    },
    /// A grouping node containing children.
    Group {
        /// Child nodes.
        children: Vec<RecipeNode>,
        /// Optional transform override.
        #[serde(default)]
        transform: Option<RecipeTransform>,
    },
}

/// Transform with optional position, rotation, and scale.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RecipeTransform {
    /// Position as [x, y, z]. Defaults to [0, 0, 0].
    #[serde(default)]
    pub position: Option<[f32; 3]>,
    /// Rotation as [x, y, z] Euler angles. Defaults to [0, 0, 0].
    #[serde(default)]
    pub rotation: Option<[f32; 3]>,
    /// Scale as [x, y, z]. Defaults to [1, 1, 1].
    #[serde(default)]
    pub scale: Option<[f32; 3]>,
}

// ── Schema generation ───────────────────────────────────────────────────

/// Returns a JSON Schema document fully describing the recipe format.
///
/// The schema includes camera variants, node types, transforms, and props.
/// LLMs and editors can use this schema for autocompletion and validation.
#[must_use]
pub fn recipe_schema() -> serde_json::Value {
    let schema = schemars::schema_for!(Recipe);
    serde_json::to_value(schema).unwrap_or_default()
}

// ── Recipe loading ──────────────────────────────────────────────────────

/// Parse, validate, and convert a JSON recipe string into a [`SceneIr`].
///
/// # Steps
///
/// 1. Parse JSON string into a generic value.
/// 2. Validate against the recipe JSON Schema.
/// 3. Deserialize into [`Recipe`] and check version.
/// 4. Convert recipe types into scene IR types.
///
/// # Errors
///
/// - [`RecipeError::Deserialize`] if the JSON is malformed.
/// - [`RecipeError::SchemaValidation`] if the JSON fails schema validation.
/// - [`RecipeError::UnsupportedVersion`] if the version is not "1.0".
pub fn load_recipe(json: &str) -> Result<(SceneIr, CameraConfig), RecipeError> {
    // Step 1: Parse JSON
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| RecipeError::Deserialize(e.to_string()))?;

    // Step 2: Schema validation
    let schema_value = recipe_schema();
    let validator = jsonschema::validator_for(&schema_value)
        .map_err(|e| RecipeError::SchemaValidation(e.to_string()))?;

    let errors: Vec<String> = validator
        .iter_errors(&value)
        .map(|e| {
            let path = e.instance_path().to_string();
            if path.is_empty() {
                e.to_string()
            } else {
                format!("{path}: {e}")
            }
        })
        .collect();

    if !errors.is_empty() {
        return Err(RecipeError::SchemaValidation(errors.join("; ")));
    }

    // Step 3: Deserialize and check version
    let recipe: Recipe =
        serde_json::from_value(value).map_err(|e| RecipeError::Deserialize(e.to_string()))?;

    if recipe.version != RECIPE_VERSION {
        return Err(RecipeError::UnsupportedVersion {
            found: recipe.version,
        });
    }

    // Step 4: Convert to SceneIr
    let camera = recipe_camera_to_config(&recipe.camera);
    let layer_nodes: Vec<SceneNode> = recipe
        .layers
        .into_iter()
        .map(|layer| recipe_layer_to_node(layer))
        .collect();

    Ok((SceneIr::new(layer_nodes), camera))
}

// ── Conversion helpers ──────────────────────────────────────────────────

fn recipe_camera_to_config(cam: &RecipeCamera) -> CameraConfig {
    match cam {
        RecipeCamera::Orbit {
            azimuth,
            elevation,
            distance,
            target,
        } => CameraConfig::Orbit(OrbitCamera {
            azimuth: *azimuth,
            elevation: *elevation,
            distance: *distance,
            target: Vec3::from_array(*target),
        }),
        RecipeCamera::FreeLook {
            position,
            yaw,
            pitch,
            speed,
        } => CameraConfig::FreeLook(FreeLookCamera {
            position: Vec3::from_array(*position),
            yaw: *yaw,
            pitch: *pitch,
            speed: *speed,
        }),
        RecipeCamera::Fps {
            position,
            yaw,
            pitch,
            speed,
            ground_y,
        } => CameraConfig::Fps(FpsCamera {
            position: Vec3::from_array(*position),
            yaw: *yaw,
            pitch: *pitch,
            speed: *speed,
            ground_y: *ground_y,
        }),
    }
}

fn recipe_layer_to_node(layer: RecipeLayer) -> SceneNode {
    let children: Vec<SceneNode> = layer
        .children
        .into_iter()
        .map(recipe_node_to_scene_node)
        .collect();

    SceneNode {
        id: NodeId::next(),
        kind: NodeKind::Layer {
            z_order: layer.z_order,
        },
        transform: Transform::default(),
        props: HashMap::new(),
        children,
        pipeline: None,
    }
}

fn recipe_node_to_scene_node(node: RecipeNode) -> SceneNode {
    match node {
        RecipeNode::Cube { transform, props } => SceneNode {
            id: NodeId::next(),
            kind: NodeKind::Cube,
            transform: recipe_transform_to_transform(transform),
            props: recipe_props_to_propmap(props),
            children: Vec::new(),
            pipeline: None,
        },
        RecipeNode::Mesh {
            path,
            transform,
            props,
        } => {
            let mut prop_map = recipe_props_to_propmap(props);
            prop_map.insert(
                "path".to_owned(),
                PropValue::Static(Box::new(path)),
            );
            SceneNode {
                id: NodeId::next(),
                kind: NodeKind::Custom("mesh".into()),
                transform: recipe_transform_to_transform(transform),
                props: prop_map,
                children: Vec::new(),
                pipeline: None,
            }
        }
        RecipeNode::Group {
            children,
            transform,
        } => {
            let child_nodes: Vec<SceneNode> = children
                .into_iter()
                .map(recipe_node_to_scene_node)
                .collect();
            SceneNode {
                id: NodeId::next(),
                kind: NodeKind::Group,
                transform: recipe_transform_to_transform(transform),
                props: HashMap::new(),
                children: child_nodes,
                pipeline: None,
            }
        }
    }
}

fn recipe_transform_to_transform(t: Option<RecipeTransform>) -> Transform {
    match t {
        None => Transform::default(),
        Some(rt) => Transform {
            position: rt.position.map_or(Vec3::ZERO, Vec3::from_array),
            rotation: rt.rotation.map_or(Vec3::ZERO, Vec3::from_array),
            scale: rt.scale.map_or(Vec3::ONE, Vec3::from_array),
        },
    }
}

fn recipe_props_to_propmap(props: Option<HashMap<String, serde_json::Value>>) -> HashMap<String, PropValue> {
    props
        .unwrap_or_default()
        .into_iter()
        .map(|(k, v)| (k, PropValue::Static(Box::new(v))))
        .collect()
}

// ── SceneIr -> Recipe conversion (for round-trip tests) ─────────────────

/// Convert a [`SceneIr`] and [`CameraConfig`] back into a [`Recipe`].
///
/// Used for round-trip testing: Rust builder -> SceneIr -> Recipe JSON -> load_recipe.
/// Pipeline fields are not represented in JSON and are skipped.
#[must_use]
pub fn scene_ir_to_recipe(ir: &SceneIr, camera: &CameraConfig) -> Recipe {
    let layers: Vec<RecipeLayer> = ir
        .nodes()
        .iter()
        .filter_map(|node| match &node.kind {
            NodeKind::Layer { z_order } => Some(RecipeLayer {
                z_order: *z_order,
                children: node
                    .children
                    .iter()
                    .map(scene_node_to_recipe_node)
                    .collect(),
            }),
            _ => None,
        })
        .collect();

    Recipe {
        version: RECIPE_VERSION.to_owned(),
        camera: camera_config_to_recipe(camera),
        layers,
    }
}

fn camera_config_to_recipe(config: &CameraConfig) -> RecipeCamera {
    match config {
        CameraConfig::Orbit(cam) => RecipeCamera::Orbit {
            azimuth: cam.azimuth,
            elevation: cam.elevation,
            distance: cam.distance,
            target: cam.target.to_array(),
        },
        CameraConfig::FreeLook(cam) => RecipeCamera::FreeLook {
            position: cam.position.to_array(),
            yaw: cam.yaw,
            pitch: cam.pitch,
            speed: cam.speed,
        },
        CameraConfig::Fps(cam) => RecipeCamera::Fps {
            position: cam.position.to_array(),
            yaw: cam.yaw,
            pitch: cam.pitch,
            speed: cam.speed,
            ground_y: cam.ground_y,
        },
    }
}

fn scene_node_to_recipe_node(node: &SceneNode) -> RecipeNode {
    let transform = transform_to_recipe(&node.transform);
    match &node.kind {
        NodeKind::Cube => RecipeNode::Cube {
            transform: Some(transform),
            props: scene_props_to_recipe(&node.props),
        },
        NodeKind::Custom(name) if name == "mesh" => {
            let path = node
                .props
                .get("path")
                .and_then(|v| v.get::<String>())
                .cloned()
                .unwrap_or_default();
            let props_without_path = node.props.iter()
                .filter(|(k, _)| k.as_str() != "path")
                .filter_map(|(k, v)| {
                    v.get::<serde_json::Value>()
                        .map(|jv| (k.clone(), jv.clone()))
                })
                .collect::<HashMap<_, _>>();
            let props = if props_without_path.is_empty() {
                None
            } else {
                Some(props_without_path)
            };
            RecipeNode::Mesh {
                path,
                transform: Some(transform),
                props,
            }
        }
        NodeKind::Group => RecipeNode::Group {
            children: node.children.iter().map(scene_node_to_recipe_node).collect(),
            transform: Some(transform),
        },
        _ => RecipeNode::Cube {
            transform: Some(transform),
            props: None,
        },
    }
}

fn transform_to_recipe(t: &Transform) -> RecipeTransform {
    RecipeTransform {
        position: Some(t.position.to_array()),
        rotation: Some(t.rotation.to_array()),
        scale: Some(t.scale.to_array()),
    }
}

fn scene_props_to_recipe(props: &HashMap<String, PropValue>) -> Option<HashMap<String, serde_json::Value>> {
    let map: HashMap<String, serde_json::Value> = props
        .iter()
        .filter(|(k, _)| k.as_str() != "path")
        .filter_map(|(k, v)| {
            v.get::<serde_json::Value>()
                .map(|jv| (k.clone(), jv.clone()))
        })
        .collect();
    if map.is_empty() {
        None
    } else {
        Some(map)
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to build a minimal valid recipe JSON string
    fn valid_recipe_json() -> String {
        r#"{
            "$version": "1.0",
            "camera": {
                "type": "orbit",
                "azimuth": 0.5,
                "elevation": 0.3,
                "distance": 5.0,
                "target": [0.0, 0.0, 0.0]
            },
            "layers": [
                {
                    "z_order": 0,
                    "children": [
                        { "type": "cube" }
                    ]
                }
            ]
        }"#
        .to_owned()
    }

    // ── Schema tests ────────────────────────────────────────────────

    #[test]
    fn json_schema_has_schema_key_and_properties() {
        let schema = recipe_schema();
        // schemars 1.x uses "$id" or "type" at root; check for "properties"
        assert!(
            schema.get("properties").is_some() || schema.get("$defs").is_some(),
            "Schema should have properties or $defs: {schema}"
        );
        let props = schema.get("properties").unwrap_or(&serde_json::Value::Null);
        assert!(
            props.get("$version").is_some(),
            "Schema should describe $version: {props}"
        );
        assert!(
            props.get("camera").is_some(),
            "Schema should describe camera: {props}"
        );
        assert!(
            props.get("layers").is_some(),
            "Schema should describe layers: {props}"
        );
    }

    #[test]
    fn json_schema_validation_accepts_well_formed_recipe() {
        let schema = recipe_schema();
        let value: serde_json::Value = serde_json::from_str(&valid_recipe_json()).unwrap();
        let validator = jsonschema::validator_for(&schema).unwrap();
        let errors: Vec<_> = validator.iter_errors(&value).collect();
        assert!(errors.is_empty(), "Valid recipe should pass schema: {errors:?}");
    }

    #[test]
    fn json_schema_validation_rejects_missing_version() {
        let schema = recipe_schema();
        let json = r#"{
            "camera": { "type": "orbit", "azimuth": 0.0, "elevation": 0.0, "distance": 5.0, "target": [0,0,0] },
            "layers": []
        }"#;
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let validator = jsonschema::validator_for(&schema).unwrap();
        let errors: Vec<String> = validator.iter_errors(&value).map(|e| e.to_string()).collect();
        assert!(!errors.is_empty(), "Missing $version should fail");
        let joined = errors.join(" ");
        assert!(
            joined.contains("$version") || joined.contains("version") || joined.contains("required"),
            "Error should mention $version: {joined}"
        );
    }

    #[test]
    fn json_schema_validation_rejects_layers_as_string() {
        let schema = recipe_schema();
        let json = r#"{
            "$version": "1.0",
            "camera": { "type": "orbit", "azimuth": 0.0, "elevation": 0.0, "distance": 5.0, "target": [0,0,0] },
            "layers": "not-an-array"
        }"#;
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let validator = jsonschema::validator_for(&schema).unwrap();
        let errors: Vec<String> = validator.iter_errors(&value).map(|e| e.to_string()).collect();
        assert!(!errors.is_empty(), "layers as string should fail");
        let joined = errors.join(" ");
        assert!(
            joined.to_lowercase().contains("layers") || joined.contains("array") || joined.contains("type"),
            "Error should mention layers or array: {joined}"
        );
    }

    // ── load_recipe error tests ──────────────────────────────────────

    #[test]
    fn json_load_recipe_unsupported_version() {
        let json = r#"{
            "$version": "99.0",
            "camera": { "type": "orbit", "azimuth": 0.0, "elevation": 0.0, "distance": 5.0, "target": [0,0,0] },
            "layers": []
        }"#;
        let result = load_recipe(json);
        assert!(result.is_err());
        match result.unwrap_err() {
            RecipeError::UnsupportedVersion { found } => {
                assert_eq!(found, "99.0");
            }
            other => panic!("Expected UnsupportedVersion, got: {other}"),
        }
    }

    #[test]
    fn json_load_recipe_missing_version() {
        let json = r#"{
            "camera": { "type": "orbit", "azimuth": 0.0, "elevation": 0.0, "distance": 5.0, "target": [0,0,0] },
            "layers": []
        }"#;
        let result = load_recipe(json);
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.to_lowercase().contains("version") || err_msg.contains("$version") || err_msg.contains("required"),
            "Error should mention version: {err_msg}"
        );
    }

    // ── Camera conversion tests ──────────────────────────────────────

    #[test]
    fn json_load_orbit_camera() {
        let json = r#"{
            "$version": "1.0",
            "camera": { "type": "orbit", "azimuth": 1.5, "elevation": 0.7, "distance": 10.0, "target": [1.0, 2.0, 3.0] },
            "layers": [{ "z_order": 0, "children": [] }]
        }"#;
        let (_, camera) = load_recipe(json).unwrap();
        match camera {
            CameraConfig::Orbit(cam) => {
                assert!((cam.azimuth - 1.5).abs() < 1e-6);
                assert!((cam.elevation - 0.7).abs() < 1e-6);
                assert!((cam.distance - 10.0).abs() < 1e-6);
                assert!((cam.target.x - 1.0).abs() < 1e-6);
                assert!((cam.target.y - 2.0).abs() < 1e-6);
                assert!((cam.target.z - 3.0).abs() < 1e-6);
            }
            other => panic!("Expected Orbit, got: {other:?}"),
        }
    }

    #[test]
    fn json_load_freelook_camera() {
        let json = r#"{
            "$version": "1.0",
            "camera": { "type": "freelook", "position": [1.0, 2.0, 3.0], "yaw": 0.5, "pitch": 0.3, "speed": 8.0 },
            "layers": [{ "z_order": 0, "children": [] }]
        }"#;
        let (_, camera) = load_recipe(json).unwrap();
        match camera {
            CameraConfig::FreeLook(cam) => {
                assert!((cam.position.x - 1.0).abs() < 1e-6);
                assert!((cam.yaw - 0.5).abs() < 1e-6);
                assert!((cam.pitch - 0.3).abs() < 1e-6);
                assert!((cam.speed - 8.0).abs() < 1e-6);
            }
            other => panic!("Expected FreeLook, got: {other:?}"),
        }
    }

    #[test]
    fn json_load_fps_camera() {
        let json = r#"{
            "$version": "1.0",
            "camera": { "type": "fps", "position": [0.0, 1.6, 5.0], "yaw": 0.0, "pitch": 0.0, "speed": 5.0, "ground_y": 1.6 },
            "layers": [{ "z_order": 0, "children": [] }]
        }"#;
        let (_, camera) = load_recipe(json).unwrap();
        match camera {
            CameraConfig::Fps(cam) => {
                assert!((cam.position.y - 1.6).abs() < 1e-6);
                assert!((cam.speed - 5.0).abs() < 1e-6);
                assert!((cam.ground_y - 1.6).abs() < 1e-6);
            }
            other => panic!("Expected Fps, got: {other:?}"),
        }
    }

    // ── Node structure tests ─────────────────────────────────────────

    #[test]
    fn json_load_layer_with_cube_child() {
        let (ir, _) = load_recipe(&valid_recipe_json()).unwrap();
        assert_eq!(ir.nodes().len(), 1, "Should have one layer");
        let layer = &ir.nodes()[0];
        assert!(matches!(layer.kind, NodeKind::Layer { z_order: 0 }));
        assert_eq!(layer.children.len(), 1, "Layer should have one child");
        assert!(matches!(layer.children[0].kind, NodeKind::Cube));
    }

    #[test]
    fn json_load_transform_fields() {
        let json = r#"{
            "$version": "1.0",
            "camera": { "type": "orbit", "azimuth": 0.0, "elevation": 0.0, "distance": 5.0, "target": [0,0,0] },
            "layers": [{
                "z_order": 0,
                "children": [{
                    "type": "cube",
                    "transform": {
                        "position": [1.0, 2.0, 3.0],
                        "rotation": [0.1, 0.2, 0.3],
                        "scale": [2.0, 2.0, 2.0]
                    }
                }]
            }]
        }"#;
        let (ir, _) = load_recipe(json).unwrap();
        let cube = &ir.nodes()[0].children[0];
        assert!((cube.transform.position.x - 1.0).abs() < 1e-6);
        assert!((cube.transform.position.y - 2.0).abs() < 1e-6);
        assert!((cube.transform.position.z - 3.0).abs() < 1e-6);
        assert!((cube.transform.rotation.x - 0.1).abs() < 1e-6);
        assert!((cube.transform.scale.x - 2.0).abs() < 1e-6);
    }

    #[test]
    fn json_load_props() {
        let json = r#"{
            "$version": "1.0",
            "camera": { "type": "orbit", "azimuth": 0.0, "elevation": 0.0, "distance": 5.0, "target": [0,0,0] },
            "layers": [{
                "z_order": 0,
                "children": [{
                    "type": "cube",
                    "props": { "color": "red" }
                }]
            }]
        }"#;
        let (ir, _) = load_recipe(json).unwrap();
        let cube = &ir.nodes()[0].children[0];
        assert!(cube.props.contains_key("color"), "Should have 'color' prop");
        let color_val = cube.props.get("color").unwrap();
        let json_val = color_val.get::<serde_json::Value>().unwrap();
        assert_eq!(json_val, &serde_json::Value::String("red".into()));
    }

    #[test]
    fn json_load_mesh_node() {
        let json = r#"{
            "$version": "1.0",
            "camera": { "type": "orbit", "azimuth": 0.0, "elevation": 0.0, "distance": 5.0, "target": [0,0,0] },
            "layers": [{
                "z_order": 0,
                "children": [{
                    "type": "mesh",
                    "path": "bunny.obj"
                }]
            }]
        }"#;
        let (ir, _) = load_recipe(json).unwrap();
        let node = &ir.nodes()[0].children[0];
        assert!(
            matches!(&node.kind, NodeKind::Custom(name) if name == "mesh"),
            "Should be Custom(\"mesh\"), got: {:?}",
            node.kind
        );
        let path = node.props.get("path").unwrap().get::<String>().unwrap();
        assert_eq!(path, "bunny.obj");
    }

    // ── Round-trip tests (Task 2) ────────────────────────────────────

    /// Helper: structurally compare two `SceneIr` trees.
    /// Ignores NodeId and Pipeline (not in JSON).
    fn nodes_structurally_equal(a: &[SceneNode], b: &[SceneNode]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        for (na, nb) in a.iter().zip(b.iter()) {
            if !node_kind_equal(&na.kind, &nb.kind) {
                return false;
            }
            if !transform_approx_equal(&na.transform, &nb.transform) {
                return false;
            }
            if !nodes_structurally_equal(&na.children, &nb.children) {
                return false;
            }
        }
        true
    }

    fn node_kind_equal(a: &NodeKind, b: &NodeKind) -> bool {
        match (a, b) {
            (NodeKind::Cube, NodeKind::Cube) => true,
            (NodeKind::Group, NodeKind::Group) => true,
            (NodeKind::Layer { z_order: za }, NodeKind::Layer { z_order: zb }) => za == zb,
            (NodeKind::Custom(na), NodeKind::Custom(nb)) => na == nb,
            _ => false,
        }
    }

    fn transform_approx_equal(a: &Transform, b: &Transform) -> bool {
        let eps = 1e-6;
        (a.position - b.position).length() < eps
            && (a.rotation - b.rotation).length() < eps
            && (a.scale - b.scale).length() < eps
    }

    #[test]
    fn json_round_trip_single_cube() {
        let scene = crate::scene()
            .camera(OrbitCamera::default())
            .layer(0, |l| l.cube())
            .build()
            .unwrap();
        let (ir, camera, _) = scene.into_parts();
        let recipe = scene_ir_to_recipe(&ir, &camera);
        let json_str = serde_json::to_string_pretty(&recipe).unwrap();
        let (ir2, _camera2) = load_recipe(&json_str).unwrap();
        assert!(
            nodes_structurally_equal(ir.nodes(), ir2.nodes()),
            "Round-trip single cube failed.\nOriginal: {:#?}\nLoaded: {:#?}",
            ir.nodes(),
            ir2.nodes()
        );
    }

    #[test]
    fn json_round_trip_multi_layer_multi_cube() {
        let scene = crate::scene()
            .camera(OrbitCamera::default())
            .layer(0, |l| l.cube().cube())
            .layer(1, |l| l.cube().cube())
            .build()
            .unwrap();
        let (ir, camera, _) = scene.into_parts();
        let recipe = scene_ir_to_recipe(&ir, &camera);
        let json_str = serde_json::to_string_pretty(&recipe).unwrap();
        let (ir2, _) = load_recipe(&json_str).unwrap();
        assert!(
            nodes_structurally_equal(ir.nodes(), ir2.nodes()),
            "Round-trip multi-layer failed"
        );
    }

    #[test]
    fn json_round_trip_positioned_scaled_cubes() {
        let scene = crate::scene()
            .camera(OrbitCamera::default())
            .layer(0, |l| {
                l.cube()
                    .position(Vec3::new(1.0, 2.0, 3.0))
                    .scale(Vec3::new(0.5, 0.5, 0.5))
            })
            .build()
            .unwrap();
        let (ir, camera, _) = scene.into_parts();
        let recipe = scene_ir_to_recipe(&ir, &camera);
        let json_str = serde_json::to_string_pretty(&recipe).unwrap();
        let (ir2, _) = load_recipe(&json_str).unwrap();
        assert!(
            nodes_structurally_equal(ir.nodes(), ir2.nodes()),
            "Round-trip transforms failed"
        );
    }

    #[test]
    fn json_round_trip_freelook_camera() {
        let cam = FreeLookCamera {
            position: Vec3::new(1.0, 2.0, 3.0),
            yaw: 0.5,
            pitch: 0.3,
            speed: 8.0,
        };
        let scene = crate::scene()
            .camera(cam)
            .layer(0, |l| l.cube())
            .build()
            .unwrap();
        let (ir, camera, _) = scene.into_parts();
        let recipe = scene_ir_to_recipe(&ir, &camera);
        let json_str = serde_json::to_string_pretty(&recipe).unwrap();
        let (ir2, camera2) = load_recipe(&json_str).unwrap();
        assert!(nodes_structurally_equal(ir.nodes(), ir2.nodes()));
        match camera2 {
            CameraConfig::FreeLook(c) => {
                assert!((c.yaw - 0.5).abs() < 1e-6);
                assert!((c.pitch - 0.3).abs() < 1e-6);
            }
            _ => panic!("Expected FreeLook camera after round-trip"),
        }
    }

    #[test]
    fn json_round_trip_fps_camera() {
        let cam = FpsCamera {
            position: Vec3::new(0.0, 1.6, 5.0),
            yaw: 0.0,
            pitch: 0.0,
            speed: 5.0,
            ground_y: 1.6,
        };
        let scene = crate::scene()
            .camera(cam)
            .layer(0, |l| l.cube())
            .build()
            .unwrap();
        let (ir, camera, _) = scene.into_parts();
        let recipe = scene_ir_to_recipe(&ir, &camera);
        let json_str = serde_json::to_string_pretty(&recipe).unwrap();
        let (ir2, camera2) = load_recipe(&json_str).unwrap();
        assert!(nodes_structurally_equal(ir.nodes(), ir2.nodes()));
        match camera2 {
            CameraConfig::Fps(c) => {
                assert!((c.ground_y - 1.6).abs() < 1e-6);
            }
            _ => panic!("Expected Fps camera after round-trip"),
        }
    }
}
