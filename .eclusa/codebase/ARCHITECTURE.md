# Architecture

**Analysis Date:** 2026-04-17
**Scope:** Rust workspace architecture mapped against v1 intent ("3D world on a teletyper" — UX is the product; z-axis validated; API generality over paradigm enforcement; 7 crates are provisional).

---

## 1. Pattern Overview

**Overall:** Layered workspace with a thin meta crate (`happyterminals`) re-exporting a curated prelude. Each lower crate owns a clean responsibility — reactive core, 3D rasterizer, effect pipeline, scene IR/graph, builder DSL + JSON, input actions, backend event loop.

**Key characteristics:**
- `SceneIr` as the single middle-of-the-funnel: every front-end (Rust builder, JSON recipes, future Python) lowers to it, and the backend walks it.
- Fine-grained reactivity (`Signal`/`Memo`/`Effect`) is a cross-cutting concern — present in `core`, surfaced into `scene` props, `input` actions, `backend` run loops.
- Zero-allocation-after-warmup discipline in hot paths (renderer, pipeline, particles) is enforced by capacity-stability tests.
- `!Send + !Sync` reactive types pin everything to a single "render thread"; cross-thread writes go through `SignalSetter` queue.

---

## 2. Crate Map (the 7-crate question)

| Crate | One-line purpose | Deps on (internal) | Standalone value? |
|-------|------------------|--------------------|-------------------|
| `happyterminals-core` | Reactive primitives (`Signal`/`Memo`/`Effect`/`Owner`) + `Grid` + `Cell` + `Rect`/`Color`/`Style` re-exports | — (only `reactive_graph`, `ratatui-core`) | YES — usable alone for any reactive terminal UI. Clean. |
| `happyterminals-renderer` | ASCII 3D rasterizer: projection, reversed-Z buffer, `Camera` trait (Orbit/FreeLook/Fps), `Mesh` (OBJ/STL), `Renderer` struct, `Particle`/`ParticleEmitter` | `core` | YES — 3D ASCII rendering is the defining primitive. |
| `happyterminals-pipeline` | `Effect` trait + `Pipeline` executor + `TachyonAdapter` (tachyonfx bridge) + named effect constructors (`effects::dissolve`, etc.) | `core` | MAYBE — small surface; coupling to `tachyonfx` is the only reason it's separate from `core`. |
| `happyterminals-scene` | `SceneIr`, `SceneNode`, `SceneGraph`, `Scene` (validated), `CameraConfig`, `TransitionManager`, `TransitionEffect` trait + 3 built-ins, easing | `core`, `renderer`, `pipeline` | YES — but note it depends on `renderer` for `CameraConfig` and on `pipeline` because `SceneNode::pipeline: Option<Pipeline>`. See FLAG-2. |
| `happyterminals-dsl` | Rust builder (`scene().camera().layer().cube()`), JSON recipe loader, schema, `EffectRegistry`, `SandboxConfig` | `scene`, `core`, `renderer`, `pipeline` | YES — clean "front-end → SceneIr" layer. |
| `happyterminals-backend-ratatui` | `run` / `run_scene` / `run_scenes` / `run_with_input`, `TerminalGuard`, `FrameSpec`, `InputSignals`, color-mode cascade | all of the above (except `dsl` at top) | YES — but see BLOCK-2: it currently **contains scene-rendering logic** (`walk_and_render`, `render_node`, `render_scene_to_grid`). |
| `happyterminals-input` | InputMap, actions (`Bool`/`Axis1D`/`Axis2D`), bindings, default viewer context | `core` | YES — standalone, no transitive pull-in of renderer/scene. Clean seam. |
| `happyterminals` (meta) | Curated `prelude::*` re-export | all | YES — this is the advertised front door. |

**Verdict on the 7 crates:** the split is **mostly defensible**. `core`, `renderer`, `input`, `backend-ratatui` are all natural seams. `scene` + `dsl` + `pipeline` are where the friction lives (see BLOCK-1, FLAG-1, FLAG-2).

No crate is pure scaffolding — every one earns its weight. The future `happyterminals-py` PyO3 crate (already stubbed in `Cargo.toml` line 12) validates keeping the split: Python should bind against `scene`/`dsl`/`renderer`/`pipeline` without dragging `backend-ratatui`.

---

## 3. Public API Surface (prelude vs. reality)

**Source of truth:** `crates/happyterminals/src/lib.rs` lines 15-77.

**The 5 example files tell the story:**
- `spinning-cube/main.rs` — clean: **only** `use happyterminals::prelude::*;`
- `color-test/main.rs` — clean: prelude only
- `transitions/main.rs` — clean: prelude only (plus `crossterm::event::KeyCode` for binding literals)
- `json-loader/main.rs` — clean: prelude only
- `static_grid.rs` — clean: prelude only
- `model-viewer/main.rs` — **LEAKS**: needs `use happyterminals_renderer::Renderer;`
- `particles/main.rs` — **LEAKS**: needs `use happyterminals_renderer::Renderer;`
- `text-reveal/main.rs` — **LEAKS**: needs `use happyterminals_pipeline::TachyonAdapter;` AND `use happyterminals_renderer::Renderer;`

So **3 of the 7 hero examples break the single-import promise.** Two distinct omissions:

### BLOCK-1: `Renderer` is not in the prelude

**File:** `crates/happyterminals/src/lib.rs`
**Missing line:** `pub use happyterminals_renderer::Renderer;`

`Renderer` is the workhorse used directly by any example that mixes manual draw calls with `run_with_input` (as opposed to the higher-level `run_scene`). The prelude re-exports `Cube`, `Mesh`, `OrbitCamera`, `Projection`, `ShadingRamp` — all the **inputs** to `Renderer::draw` — but not `Renderer` itself. Three of the core examples have to punch through the abstraction.

Impact: users writing their own render loop (which is the documented "use `run_with_input` + draw yourself" path) cannot stay inside `prelude::*`. Violates the "Terminal art should feel like magic, not plumbing" promise.

### BLOCK-2: `TachyonAdapter::with_area` is not reachable from prelude

**File:** `crates/happyterminals/src/lib.rs`
**Missing lines:** `pub use happyterminals_pipeline::TachyonAdapter;` and re-export of `tachyonfx::{Motion, fx, Duration as FxDuration}` (or at minimum `TachyonAdapter`).

The named constructors in `happyterminals_pipeline::effects::{dissolve, fade_from, sweep_in, coalesce, ...}` **do not expose an area-bounded variant**. They all call `TachyonAdapter::new(...)` which uses full-grid area. The text-reveal example (the hero-of-heroes for z-axis narrative) requires a **bounded** title rect — so it reaches for `TachyonAdapter::with_area` directly.

This is a real API hole, not a cosmetic one: bounded effects are not a niche — they're the whole "effects layered over 3D" story.

---

## 4. Layer Coherence Findings

### FLAG-1: `happyterminals-dsl::node_builder` is the builder for *scene nodes*, not DSL sugar

**Files:** `crates/happyterminals-dsl/src/node_builder.rs`, `crates/happyterminals-scene/src/node.rs`

`CubeBuilder`/`LayerBuilder`/`GroupBuilder` live in `dsl`, but they construct `SceneNode`/`NodeKind`/`Transform` which live in `scene`. The split is justifiable (builders are the Rust-front-end-specific consuming-self API; the scene crate stays front-end-agnostic for Python/JSON), but the naming *feels* inverted — `dsl::node_builder` could be read as "DSL for building nodes" when it's actually "Rust-front-end builder that lowers to scene nodes."

Not a move, but worth noting in crate-level docs: `dsl` = Rust front-end only; JSON is the Python-compatible front-end also living in `dsl::json`.

### FLAG-2: `happyterminals-scene` depends on `happyterminals-pipeline`

**File:** `crates/happyterminals-scene/src/node.rs` line 8: `use happyterminals_pipeline::Pipeline;`
**File:** `crates/happyterminals-scene/src/scene.rs` line 9: `use happyterminals_pipeline::Pipeline;`

`SceneNode::pipeline: Option<Pipeline>` and `Scene::pipeline: Option<Pipeline>` anchor the pipeline type into the scene layer. This forces `scene` to carry `tachyonfx` transitively (pipeline → tachyonfx).

Alternative: make it `Option<Box<dyn Effect>>` or parametrize `Scene<P>`. Neither is free — `Pipeline` is a concrete `Vec<Box<dyn Effect>>` that JSON recipes build at runtime and `Scene::into_parts()` hands back.

**Ruling:** this is acceptable coupling (not circular, just strong), but if `scene` ever needs to exist without `tachyonfx` (e.g. a truly minimal Python crate that doesn't bind effects), this is the seam to revisit. Keep as-is for v1; flag for v2 if publish-time bloat becomes an issue.

### FLAG-3: `happyterminals-scene` depends on `happyterminals-renderer` for `CameraConfig`

**File:** `crates/happyterminals-scene/src/Cargo.toml` line 17
**File:** `crates/happyterminals-scene/src/camera.rs` (re-exports `CameraConfig` which internally holds `OrbitCamera`)

Same class of coupling — `scene` pulls `renderer` solely so `CameraConfig` can carry a concrete `OrbitCamera`. Could be inverted with a `Camera` trait object, but the current `camera_config.as_orbit_mut()` / `as_camera()` pattern is exactly what the transition + render loops depend on.

**Ruling:** acceptable. The 7-crate split survives — `renderer` is small enough that `scene` carrying it is fine.

### FLAG-4: `backend-ratatui` re-exports `happyterminals-input` wholesale

**File:** `crates/happyterminals-backend-ratatui/src/lib.rs` lines 23-31

The backend crate already re-exports the entire `happyterminals-input` API. Then the meta crate re-exports the input types again (`src/lib.rs` lines 37-42). This is redundant-but-not-wrong — the backend wants users to be able to read inputs without a separate dep. Harmless, but the meta prelude should pick one source and stick with it. Currently the meta reads directly from `happyterminals_input::*` bypassing the backend's re-export.

### INFO-1: Scene rendering logic lives in the backend crate

**File:** `crates/happyterminals-backend-ratatui/src/run.rs` lines 602-670

`walk_and_render`, `render_node`, `render_node_immutable`, `render_scene_to_grid` — the tree-walking render dispatcher for `NodeKind::Cube` / `Layer` / `Group` — all live in the **backend** crate. This is fine for v1 (the only backend is ratatui), but if a second backend ever ships (wgpu-testing harness, captured-frame recorder, web-ssh terminal), this walker should move into `scene` as `SceneWalker` or similar.

Not a v1 blocker. Noting because the intent says "7 crates are provisional" — this is the one non-obvious placement.

### INFO-2: `SceneNode::pipeline` field exists but is never read by the walker

**Files:** `crates/happyterminals-scene/src/node.rs` line 87; `crates/happyterminals-backend-ratatui/src/run.rs` lines 640-670

`SceneNode` carries an `Option<Pipeline>`, but `render_node` only applies the scene-level `pipeline` (via `Scene::into_parts()` in `run_scene`). Per-node pipelines are shipped scaffolding. Fine — the builder constructs them and tests cover them — but current render loops don't dispatch per-node effects.

**If** v2 demands per-node effects (text-reveal suggests the answer is yes), the walker needs extension. Logged.

---

## 5. Intent Coherence Findings

The stated intent:
1. "Terminal art should feel like magic, not plumbing"
2. "3D world on a teletyper" — UX is the product
3. z-axis spatial navigation is the validated differentiator
4. API generality over paradigm enforcement (don't redesign around one use case)

### How the code expresses the intent

| Intent | Evidence | Status |
|--------|---------|--------|
| Magic not plumbing | `spinning-cube` = 43 LOC including comments; `run_scene` + `scene()` DSL eliminates explicit render loops | **Met** — when the prelude works (see BLOCK-1, BLOCK-2). |
| 3D world on a teletyper | `Renderer` + `Cube` + `Mesh` + `OrbitCamera` + reversed-Z buffer are all first-class | **Met**. |
| z-axis spatial navigation | `Layer { z_order: i16 }` + `TransitionManager` + `SceneNode::pipeline` scaffold | **Partially met** — z-order is data but there's no built-in "layer transition" effect or compositor. See INFO-3. |
| API generality | `Camera` trait, `Effect` trait, `TransitionEffect` trait, `SceneIr` as universal target, `NodeKind::Custom(String)` for user node kinds | **Met** — the surface isn't narrowly shaped around any one demo. |

### INFO-3: Z-axis is a data model, not yet a navigation primitive

Per memory note: "Z-axis paradigm shift validated — Discord community confirmed spatial depth in terminals is a game changer."

Current state:
- `Layer { z_order }` defines compositing order — **static** z.
- `TransitionManager` blends whole scenes — **scene-level** transitions, not z-layer transitions.
- No "spatial z-layer scroll" or "push layer in/out of depth" primitive exists.

This is NOT a v1 regression (the community validated z as a *concept*, not as a shipped API), and v2.0 is in progress. But the architecture should note: **when z-as-navigation ships, it likely belongs in `scene` as a `LayerStack` type**, not in a new crate, not in `dsl`. This follows the "API generality" rule — a `LayerStack` that's orthogonal to any particular transition effect.

### INFO-4: No shipped scaffolding that screams "we'll delete this"

I looked for stubs, half-built placeholders, unused trait methods. The `NodeKind::Custom(String)` branch in `render_node` is a no-op (it just matches and does nothing), but that's by design — users who add custom kinds own the rendering. Not scaffolding, just an extension point.

`NodeKind::Layer { z_order }` is used for children traversal but `z_order` is never read by the walker. That's because layers iterate in IR order (which the builder already sorts). This is a latent data field — either wire it to actually sort, or document that the builder is the sort authority. Low priority.

---

## 6. Cross-Crate API Warts (full inventory)

Grep of `^use happyterminals_` inside `crates/happyterminals/examples/`:

```
model-viewer/main.rs:26:    use happyterminals_renderer::Renderer;
particles/main.rs:27:       use happyterminals_renderer::Renderer;
text-reveal/main.rs:38:     use happyterminals_pipeline::TachyonAdapter;
text-reveal/main.rs:39:     use happyterminals_renderer::Renderer;
text-reveal/main.rs:257:    use happyterminals_core::create_root;  (test-only, already in prelude)
text-reveal/main.rs:433:    use happyterminals_core::Grid;          (test-only, already in prelude)
```

Only two unique holes:
- `Renderer` (hit by 3 examples)
- `TachyonAdapter` (hit by 1 example, but it's THE text-reveal example)

Both are **fixable in a single commit** editing `crates/happyterminals/src/lib.rs`. See §8.

---

## 7. Severity Summary

### BLOCK (violates intent, must fix before v1 publish)

- **BLOCK-1** — `Renderer` missing from prelude. Breaks 3 of 7 hero examples. Fix: one `pub use` line.
- **BLOCK-2** — `TachyonAdapter` missing from prelude. Breaks the text-reveal hero example (z-axis narrative flagship). Fix: one `pub use` line + consider exposing a bounded `effects::*` variant.

### FLAG (smells, discuss)

- **FLAG-1** — `dsl::node_builder` naming could confuse readers into thinking `scene` owns node builders. Docs fix.
- **FLAG-2** — `scene` → `pipeline` coupling via `SceneNode::pipeline`. Acceptable for v1, revisit if Python bindings or minimal-scene publishing matters.
- **FLAG-3** — `scene` → `renderer` coupling via `CameraConfig`. Acceptable.
- **FLAG-4** — Duplicate input re-exports through backend + meta. Cosmetic.

### INFO (nice to know)

- **INFO-1** — Scene walker lives in backend crate. Moves to `scene` if second backend ships.
- **INFO-2** — `SceneNode::pipeline` is shipped-but-not-walked. Per-node effects need walker extension if v2 needs them.
- **INFO-3** — Z-axis is data, not yet a navigation primitive. v2+ work; when it ships, put it in `scene`.
- **INFO-4** — `NodeKind::Layer.z_order` is a latent field (not used by walker).

### Alignment statement

**The workspace is substantially aligned with its stated intent.** The 7-crate split is defensible — every crate earns its weight, seams are natural, dependencies are DAG-shaped (no cycles), and the `SceneIr`-as-universal-target architecture is exactly what the "API generality" principle demands. The two BLOCK items are both `pub use` omissions in the meta crate, not structural problems.

---

## 8. Recommended Refactors (concrete, actionable)

### R-1: Close the prelude gap (v1 BLOCKER)

**File:** `crates/happyterminals/src/lib.rs`

Add to the `prelude` module, after line 73 (`pub use happyterminals_pipeline::effects;`):

```rust
pub use happyterminals_pipeline::TachyonAdapter;
```

Add to `prelude`, after line 69 (`pub use happyterminals_renderer::{Particle, ParticleEmitter, lerp_color};`):

```rust
pub use happyterminals_renderer::Renderer;
```

Then update the prelude compile-check test (same file, lines 88-113) to name `Renderer` and `TachyonAdapter`.

Then update the three leaky examples to remove the direct sub-crate imports:
- `crates/happyterminals/examples/model-viewer/main.rs` — delete line 26
- `crates/happyterminals/examples/particles/main.rs` — delete line 27
- `crates/happyterminals/examples/text-reveal/main.rs` — delete lines 38, 39

### R-2: Ship a bounded `effects::*` variant (v1 BLOCKER companion to R-1)

**File:** `crates/happyterminals-pipeline/src/effects/mod.rs`

Every existing constructor (`dissolve`, `fade_from`, `sweep_in`, `coalesce`, `hsl_shift`, `evolve`, `darken`, `paint`) currently returns a full-grid `TachyonAdapter::new(...)`. Ship an `_in` suffix variant (or a `.bounded(rect)` method on `TachyonAdapter`) so the text-reveal example doesn't need to construct the adapter manually.

Concrete proposal — add a method:

```rust
impl TachyonAdapter {
    pub fn bounded(self, area: Rect) -> Self {
        Self { area: Some(area), ..self }
    }
}
```

Then `effects::fade_from(fg, bg, dur).bounded(title_rect)` is the clean path, and users don't need to name `TachyonAdapter` at all. Keep `TachyonAdapter::with_area` as the lower-level constructor.

**File touch list:** `crates/happyterminals-pipeline/src/adapter.rs` — add `bounded` method. Test stays the same; docs updated.

### R-3: Drop the duplicate input re-exports from the meta prelude

**File:** `crates/happyterminals/src/lib.rs` lines 36-42

The backend crate already re-exports all input types (`crates/happyterminals-backend-ratatui/src/lib.rs` lines 23-31). The meta prelude can pick its source — either import from `happyterminals_input::*` OR from `happyterminals_backend_ratatui::*`, but not both. Currently it imports from `happyterminals_input` directly, which is fine; just delete the now-unnecessary re-exports in the backend OR drop them from the meta prelude. Pick one — current state works but carries a pointless redundancy a curious reader will stumble over.

**Cheapest fix:** delete the `happyterminals_input::*` re-exports from `crates/happyterminals-backend-ratatui/src/lib.rs` lines 23-31. The meta prelude is the canonical consumer surface; the backend crate's job is the run loop, not input surface.

### R-4 (NOT RECOMMENDED): Do NOT merge any crates for v1

`pipeline` + `scene` merger: tempting (strong coupling, small pipeline crate) but would force `scene` to own `tachyonfx`. Keep split.

`dsl` + `scene` merger: tempting (dsl literally builds scene nodes) but would force `scene` to own `serde_json`/`jsonschema` for its JSON half. Keep split.

`backend-ratatui` + `happyterminals` (meta): would conflate "event loop" and "re-export hub." Keep split.

No merges justified by current evidence.

### R-5 (v2+): Move `walk_and_render` to `happyterminals-scene`

**File from:** `crates/happyterminals-backend-ratatui/src/run.rs` lines 602-670
**File to:** a new `crates/happyterminals-scene/src/walker.rs`

Defer until a second backend (non-ratatui) is considered. Today the walker's only caller is `run_scene`/`run_scenes` in the ratatui backend. Premature move.

### R-6 (v2+): Name the z-layer navigation primitive

When the compositor-level z-layer navigation ships (per Discord validation), put the new type in `happyterminals-scene`, probably as `LayerStack` or `DepthStack`. Do NOT create a new crate for it. Do NOT special-case it in `dsl` — expose it as a plain type constructor. This honors "API generality over paradigm enforcement": the z-axis is a *tool*, not a framework tenet.

---

*Architecture analysis: 2026-04-17*
