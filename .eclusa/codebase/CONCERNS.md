# Codebase Concerns — Public API Leak Audit

**Analysis Date:** 2026-04-17
**Scope:** Public surface of the `happyterminals` meta-crate and its prelude, audited against the core value: *"Terminal art should feel like magic, not plumbing."*
**Method:** Read every re-export in `crates/happyterminals/src/lib.rs` and every sub-crate root `lib.rs`; traced all 5 headline examples against what is / is not reachable via `use happyterminals::prelude::*;` alone.

A *leak* = the public surface forcing users to know about upstream crates (ratatui, crossterm, tachyonfx) in order to do work that the library claims to own.

---

## BLOCK — Violates core value

### B1. `Color` / `Style` / `Modifier` / `Rect` are bare ratatui re-exports, not happyterminals types

- **Call site:** `crates/happyterminals-core/src/lib.rs:66-67`
  ```rust
  pub use ratatui_core::layout::Rect;
  pub use ratatui_core::style::{Color, Modifier, Style};
  ```
  → surfaced at `crates/happyterminals/src/lib.rs:26` as
  `pub use happyterminals_core::{Color, Modifier, Style, Rect};`
- **What leaks:** The prelude exposes `Color::Rgb(180, 200, 255)`, `Color::Reset`, `Style::default().fg(...)`, `Modifier::BOLD`, `Rect::new(...)` — all `ratatui_core` types verbatim. `particles/main.rs:49-50`, `text-reveal/main.rs:184-188`, `model-viewer/main.rs:226` all construct them by name, inheriting ratatui's API (the `Color::Reset` sentinel, the `Modifier` bitflag semantics, `Rect`'s `x/y/width/height` layout conventions).
- **Why this violates "magic, not plumbing":** `PROJECT.md:9` explicitly says users should not have to know that ratatui is underneath. Today any doc example that mentions `Color` is *literally* documenting `ratatui_core::style::Color`. When ratatui's 0.30 → 0.31 bump changes anything on these types, it is a public-API break for happyterminals with no insulation layer. The `Rect` leak is the worst — it trains users to reach into low-level layout math (`grid.area.width`, `.height`, `Rect::new(x,y,w,h)`) when the narrative is "describe scenes, not rectangles."
- **Cleaner shape:** Introduce `happyterminals::Color`, `happyterminals::Style`, `happyterminals::Modifier` as wrapping newtypes (or `pub use` under our own path with a private-module redirection we control). `Rect` stays — it is unavoidable for bounded-effect regions — but re-export under our own name (`happyterminals::Region`) so we can swap the internal type later without SemVer break. `impl From<Color> for ratatui_core::style::Color` keeps the adapter clean.

### B2. `crossterm::event::KeyCode` is the only way to bind a key

- **Call site:** `crates/happyterminals-input/src/binding.rs:7`
  `use crossterm::event::{KeyCode, KeyModifiers, MouseButton};` → forces every `Binding::Key(...)` constructor to take a crossterm type.
- **User-side evidence:** 4 of 5 headline examples import crossterm directly:
  - `text-reveal/main.rs:169,174` — `Binding::Key(crossterm::event::KeyCode::Char(' '))`
  - `particles/main.rs:69-70` — same pattern for Space / R
  - `transitions/main.rs:66` — `KeyCode::Tab`
  - `model-viewer/main.rs:84,89,94,99,104,109` — six bindings, every single one is `crossterm::event::KeyCode::*`
- **Why this violates "magic, not plumbing":** A user who reads `happyterminals::prelude::*` and wants to bind Space to an action *must* either guess that `crossterm` is in play or grep through sub-crate docs. The prelude advertises `Binding`, `InputMap`, `register_default_actions` — the one critical piece needed to *use* them is smuggled in from an unmentioned crate. This is also the exact leak Python bindings will hit hardest: PyO3 cannot wrap `crossterm::event::KeyCode` without either vendoring crossterm or inventing a wrapper enum. Doing the wrapper now costs one afternoon; doing it after v1 publish costs a major-version break.
- **Cleaner shape:** Ship `happyterminals::Key` as a Rust enum mirroring the common `KeyCode` variants (`Char(char)`, `Tab`, `Enter`, `Esc`, `Backspace`, `F(u8)`, arrows, …) and `happyterminals::KeyMod` as a bitflag. `Binding::Key(Key::Char(' '))` replaces `Binding::Key(crossterm::event::KeyCode::Char(' '))`. Internally `impl From<Key> for crossterm::event::KeyCode` keeps the match glue trivial. Same treatment for `MouseButton` → `happyterminals::MouseButton`.

### B3. `Renderer` is NOT in the prelude — but users still have to import it

- **Call site:** `crates/happyterminals/src/lib.rs:67-69` re-exports `OrbitCamera`, `Projection`, `ShadingRamp`, `Cube`, `Mesh`, `load_obj`, `load_stl`, `Particle`, `ParticleEmitter` — but **not `Renderer`**. Yet 3 of 5 headline examples build one by hand:
  - `model-viewer/main.rs:26` — `use happyterminals_renderer::Renderer;`
  - `particles/main.rs:27` — same
  - `text-reveal/main.rs:39` — same
- **Why this violates "magic, not plumbing":** The DSL path (`scene().layer(...).build()` + `run_scene`) hides `Renderer` correctly — see `spinning-cube/main.rs` and `json-loader/main.rs`, neither of which mentions a renderer at all. The imperative path (`run_with_input` with a raw closure) requires the user to construct a `Renderer`, call `renderer.draw(grid, mesh, camera, projection, shading)`, and manage its state across frames. That is exactly "cell-level addressing when trying to do scene-level work" from the intent anchor. Either `Renderer` should be in the prelude (admitting it is a first-class concept), or `run_with_input` should grow a higher-level form so hero-level examples never need to touch it.
- **Cleaner shape:** Add `pub use happyterminals_renderer::Renderer;` to the prelude *and* offer a `MeshView` scene node / builder so the text-reveal / particles / model-viewer examples can describe "render this mesh with this camera" declaratively and skip the explicit `Renderer::new()` / `renderer.draw(...)` plumbing entirely. The DSL already handles `Cube`; extending to `Mesh` is the natural next step and it erases 3 of 3 current Renderer-leak call sites.

---

## FLAG — Could be tighter

### F1. `TachyonAdapter` is a required import for bounded effects

- **Call site (user):** `crates/happyterminals/examples/text-reveal/main.rs:38`
  `use happyterminals_pipeline::TachyonAdapter;` — forced because the convenience wrappers in `happyterminals_pipeline::effects` (see `effects/mod.rs:16-92`) all call `TachyonAdapter::new(fx)` with *no area override*. If you want a rect-bounded effect, your only option is `TachyonAdapter::with_area(tachyonfx::fx::fade_from(...), rect)` — which forces both `TachyonAdapter` and raw `tachyonfx::fx` into the user namespace.
- **User-side evidence:** `text-reveal/main.rs:54-78` — three ready-made reveal functions, each one a 3-line boilerplate around `tachyonfx::Duration::from(...)` + `tachyonfx::fx::fade_from(...)` + `TachyonAdapter::with_area(...)`. The `effects::fade_from` wrapper at `pipeline/src/effects/mod.rs:23` does all this except the `with_area` override.
- **Why this is a flag, not a block:** The "unbounded effect" path (`Pipeline::new().with(effects::fade_from(fg, bg, dt))` in `spinning-cube/main.rs:43`) is clean — `tachyonfx` is fully hidden. The leak only bites when users want a region-bounded effect, which the hero example exists specifically to demonstrate. So the paradigm-defining use case is where the wrapper fails.
- **Cleaner shape:** Add an `in_rect(rect)` builder method to every `effects::*` constructor's return type. E.g.
  ```rust
  effects::fade_from(Color::Black, Color::Reset, dt).in_rect(title_rect)
  ```
  This requires making `TachyonAdapter` a returned-but-not-imported type (users get the method without needing the name). Also add `effects::fade_from_bounded(fg, bg, rect, dt)` as a direct free fn for discoverability. Either eliminates the `use happyterminals_pipeline::TachyonAdapter;` line and the three `tachyonfx::*` references in `text-reveal/main.rs`.

### F2. No unified `happyterminals::Error`; each crate surfaces its own

- **Error types in play:** `CoreError` (`core/src/error.rs:12`), `RecipeError` (`dsl/src/json.rs:28`), `PipelineError` (`pipeline/src/error.rs:7`), `MeshError` (`renderer/src/mesh.rs:102`), `SceneError` (`scene/src/error.rs:7`). All are re-exported through the prelude individually: `MeshError`, `SceneError`, `RecipeError` appear at `crates/happyterminals/src/lib.rs:50,63,68`.
- **Why this is a flag:** Every headline example dodges the problem by typing its `main` return as `Result<(), Box<dyn std::error::Error>>` (see `spinning-cube/main.rs:30`, `json-loader/main.rs:49`, `text-reveal/main.rs:146`). That works for quick demos but fails downstream: a caller wrapping happyterminals in a larger app has five error types to match on and no trait-object story beyond `Box<dyn Error>`. For Python bindings, PyO3's `PyErr` conversion wants one source error per `impl From`, not five scattered across five crates.
- **Cleaner shape:** Add `pub enum happyterminals::Error` to the meta-crate with `#[from]` conversions from each sub-crate error:
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum Error {
      #[error(transparent)] Core(#[from] CoreError),
      #[error(transparent)] Scene(#[from] SceneError),
      #[error(transparent)] Recipe(#[from] RecipeError),
      #[error(transparent)] Mesh(#[from] MeshError),
      #[error(transparent)] Pipeline(#[from] PipelineError),
      #[error(transparent)] Io(#[from] std::io::Error),
  }
  pub type Result<T> = std::result::Result<T, Error>;
  ```
  Examples become `async fn main() -> happyterminals::Result<()>`. The individual error types remain exported for users who want fine-grained matching — they become *optional* knowledge.

### F3. Scene construction from JSON takes two steps: `load_recipe_sandboxed` → `Scene::new`

- **Call site:** `crates/happyterminals/examples/json-loader/main.rs:63,88-89`
  ```rust
  let (ir, camera_config) = load_recipe_sandboxed(&json, &cfg)?;
  let (scene_result, _owner) = create_root(|| Scene::new(ir, camera_config, None));
  let scene = scene_result?;
  ```
- **Why this is a flag:** The user has to know that `load_recipe_sandboxed` returns a *tuple* `(SceneIr, CameraConfig)`, that `Scene::new` takes three arguments (`ir, camera, pipeline: Option<Pipeline>`), and that the whole thing must run inside `create_root(...)`. Four types (`SceneIr`, `CameraConfig`, `Pipeline`, `Owner`) and one reactive-context rule participate in what the user framed as "load JSON → show scene". The decision note in `STATE.md:59` ("load_recipe returns tuple, not Scene — avoids Scene validation at recipe load time") is sound architecturally, but the meta-crate should still offer a one-line convenience.
- **Cleaner shape:** Add
  ```rust
  pub fn scene_from_recipe(json: &str, cfg: &SandboxConfig) -> Result<(Scene, Owner), Error> { … }
  ```
  to the meta-crate. Internally it does the `load_recipe_sandboxed` + `create_root(|| Scene::new(...))` dance and returns both the ready-to-render `Scene` and the `Owner` the caller needs to keep alive for reactive disposal. `json-loader/main.rs` collapses to 3 lines of real logic.

### F4. `Grid::buffer_mut()` is public and leaks `ratatui_core::buffer::Buffer`

- **Call site:** `crates/happyterminals-core/src/grid.rs:94` — `pub fn buffer_mut(&mut self) -> &mut Buffer`. Doc-comment says "Used by the pipeline crate's `TachyonAdapter`" and "Prefer `put_str`" but the method is fully public and the signature is `ratatui_core::buffer::Buffer`.
- **Why this is a flag, not a block:** It's documented as an escape hatch for effect authors, and the internal consumer (`TachyonAdapter::apply` at `pipeline/src/adapter.rs:61`) needs it. `Deref<Target = Buffer>` on `Grid` (line 108) already exposes read-only `Buffer` anyway, so half the cat is out of the bag.
- **Cleaner shape:** Make `buffer_mut` `pub(crate)` or guard behind a `pub mod internal { }` module with a `#[doc(hidden)]` attribute, so casual users browsing the prelude never see it as a suggested path. Effect authors writing `impl Effect` in user code still need *something* — that could be a thin `EffectCtx<'a>` struct passed to `apply` that exposes only `put_str`-level writes plus a narrow `with_raw_buffer(f: impl FnOnce(&mut Buffer))` escape hatch, pushing the leak behind a doc-signal that says "here be dragons."

---

## INFO — Reasonable as-is, worth documenting

### I1. `vec3` from `glam` is in the prelude by design

- **Call site:** `crates/happyterminals/src/lib.rs:76` — `pub use glam::vec3;`
- **Assessment:** `glam` is the de-facto Rust linear-algebra crate and re-exporting `vec3` (not `Vec3`) keeps the per-node `position(vec3(0., 0., 0.))` call ergonomic. The core value says no ANSI / buffers / draw calls — it doesn't say no math. A scene description without `vec3` would either invent a weaker coordinate type or force `(f32, f32, f32)` tuples everywhere. This leak is a feature.
- **Document:** In the prelude docstring, explicitly call out "we re-export `glam::vec3` because 3D math is part of the scene description, not plumbing."

### I2. `Fx` (aliased `tachyonfx::Effect`) is in the prelude

- **Call site:** `crates/happyterminals-pipeline/src/lib.rs:29` — `pub use tachyonfx::Effect as Fx;`. Re-surfaced via meta-crate prelude at `crates/happyterminals/src/lib.rs:72`.
- **Assessment:** The alias exists specifically to disambiguate from our own `Effect` trait (documented at `pipeline/src/lib.rs:11-12`). `Fx` is the tachyonfx-native type; exposing it lets power users build custom effects. The alternative — hiding tachyonfx entirely — would mean reimplementing the 50+ effects we're building on top of, which `PROJECT.md` key decisions forbids. So this leak is a deliberate trade: you can live in our `effects::*` wrappers entirely (5 of 5 examples do for the *unbounded* path) and never touch `Fx`, but if you want to compose a raw tachyonfx effect, one name is exposed to let you.
- **Document:** Prelude docstring should note "`Fx` = `tachyonfx::Effect`, exposed for power users; prefer `effects::*` constructors for ~all use cases."

### I3. `OrbitCamera` / `FreeLookCamera` / `FpsCamera` are happyterminals-owned structs

- **Verified:** Defined in `crates/happyterminals-renderer/src/camera.rs`, re-exported through scene crate's `CameraConfig` enum (`scene/src/camera.rs:15-22`) and through the meta-crate prelude (`crates/happyterminals/src/lib.rs:45`). `From<OrbitCamera> for CameraConfig` (line 82-98 of scene's camera.rs) is the ergonomic adapter the `SceneBuilder::camera()` method leans on.
- **Assessment:** No leak. The camera surface is fully owned, documented, and unified behind `CameraConfig`. Python bindings will wrap these three structs directly.

### I4. `PropValue::Static(Box<serde_json::Value>)` is in the scene crate's public API

- **Noted in `STATE.md:63`** as a decision. `serde_json::Value` is re-exposed through `PropValue` (`scene/src/prop.rs`).
- **Assessment:** For recipe round-tripping this is unavoidable — JSON props need to survive the `load_recipe` → `Scene` → `scene_ir_to_recipe` loop intact. `serde_json` is the pragmatic carrier. For the v1 surface this is fine; for Python bindings it will become a PyO3 `Py<PyAny>` conversion, which is also fine.
- **Document:** No action needed. Worth a one-line README note that the JSON recipe format commits users to `serde_json::Value` as the prop carrier.

---

## Top 3 surface fixes that would most tighten the "magic" feel

**1. Wrap `KeyCode` / `KeyModifiers` / `MouseButton` behind `happyterminals::Key` + `KeyMod` + `MouseButton` enums. (B2)**
Kills the single most visible leak: 4 of 5 headline examples import `crossterm` directly, for zero functional reason. This is also the leak that will cost the most at the Python milestone — `crossterm::event::KeyCode` cannot cross the PyO3 boundary without a wrapper. Doing the wrapper now (~1 day of enum mirroring + `From` impls) removes the leak from every example and locks the input API for Python without a future break.

**2. Add `pub use happyterminals_renderer::Renderer;` to the prelude AND ship a `mesh()` scene-builder node. (B3)**
Today the imperative examples (`model-viewer`, `particles`, `text-reveal`) reach into `happyterminals_renderer` for `Renderer::new()` and call `renderer.draw(grid, mesh, camera, projection, shading)` by hand — pure plumbing the DSL was supposed to hide. The DSL already handles `cube()`; extending to `mesh()` (closure form: `l.mesh(&bunny).rotation(&rot)`, or path form: `l.mesh_from("bunny.obj")`) collapses the three leak sites and makes the DSL the canonical path instead of one of two parallel paths. Secondary benefit: recipe JSON gets a real `{"type": "mesh", ...}` node with full rendering, not the current partial-support placeholder the json-loader example ships with.

**3. Introduce `happyterminals::Color`, `Style`, `Modifier`, `Region` wrapper types, and a unified `happyterminals::Error`. (B1 + F2)**
Two small cleanups that together insulate the *entire* v1 public surface from ratatui/crossterm/tachyonfx version churn. Users get one error type, one color type, one style type — all under the `happyterminals` namespace. Upstream crates can bump majors without it being a happyterminals breaking change. This is also the difference between "we happen to use ratatui" (strong signal, controllable dependency) and "we expose ratatui" (dependency bleeds into every user's SemVer contract).

Collectively these three fixes:
- Remove every `use crossterm::…` from example code.
- Remove every `use happyterminals_renderer::…` and `use happyterminals_pipeline::TachyonAdapter;` from example code.
- Let every example return `happyterminals::Result<()>` instead of `Box<dyn Error>`.
- Leave users with a single import, `use happyterminals::prelude::*;`, that fully backs the 5 headline examples — matching what the `lib.rs` docstring at `crates/happyterminals/src/lib.rs:6` already promises.

---

*Public-API leak audit: 2026-04-17*
