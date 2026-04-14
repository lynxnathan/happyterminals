# Architecture Research — happyterminals

**Domain:** Declarative, reactive terminal scene manager (Rust workspace + future PyO3 binding)
**Researched:** 2026-04-14
**Confidence:** HIGH on crate split / data flow / pipeline; MEDIUM on reactive-core "build vs reuse" (recommendation given, both viable); MEDIUM on PyO3 asyncio integration (ecosystem still maturing).

---

## 0. TL;DR for the Roadmapper

- **Crates** — split the workspace into **six** crates, not three. Add `happyterminals-backend-ratatui`, `happyterminals-dsl`, and a thin meta-crate `happyterminals` that re-exports a curated public API. Keep the Python bindings in a *separate* `happyterminals-py` crate that depends on the meta-crate but is built only when the `python` feature is enabled (or as its own member excluded from the default workspace build). This keeps PyO3's compile-time cost off the hot path during inner-loop development.
- **Reactive core** — build our own minimal SolidJS-style runtime (the existing `reactive.rs` stub is on the right track) and design it to be **swap-out-able** behind a small trait (`ReactiveRuntime`). If complexity grows, drop in `reactive_graph` (the standalone Leptos crate) without touching downstream code. Recommend single-threaded with thread-local owner stack (mirrors Leptos), and a *separate* render thread that receives diffs over a channel — the renderer never touches the reactive graph directly.
- **Data flow** — strict one-way: `Signal → Memo/Effect → Scene IR (dirty-flagged) → Renderer fills Grid → Pipeline transforms Grid → Backend diffs into ratatui::Buffer → terminal`. Async only at the I/O edges (input event stream, frame ticker, optional async loaders); the entire scene-build / render / pipeline pass is **synchronous** and runs on a single "render thread."
- **Grid model** — one cell type (`char + fg + bg + Modifier + skip-bit`), single `Grid` owned by the scene, *no* internal double-buffering — let ratatui's `Buffer` do the diff. Keep the second buffer at the backend boundary, not in core.
- **Pipeline** — `dyn Effect`-based trait objects (boxed), data-driven order, with tachyonfx adapter that wraps a `tachyonfx::Shader` so any tachyonfx effect plugs straight in. Trait objects beat generic chains here because JSON recipes and Python both need runtime composition.
- **Scene graph** — flat layered list of `SceneNode`s with explicit z-order, plus a top-level transition manager that owns *two* scenes (from/to) and a transition effect. Camera is a property of the renderer pass, not the scene.
- **DSL & JSON** — both compile to one **internal IR** (`scene::Ir`). Builder API and JSON loader are two front-ends; Python is a third front-end built on the same IR. This is the single most important shared abstraction.
- **PyO3 boundary** — cross at the IR + Signal handle level. Expose `Signal`, `Scene`, `Effect`, `run()`. Use `Python::allow_threads` around the entire frame loop. Defer asyncio integration to a v0.2 follow-up — sync `run()` first.
- **Build order** — bottom-up, vertical-slice first: reactive primitives → Grid → ratatui backend (degenerate static Grid) → tachyonfx adapter → minimal renderer (one primitive) → DSL/IR → spinning-cube end-to-end demo (Milestone 1 exit). Then deepen each layer. PyO3 last, as already decided.

---

## 1. Crate Boundaries

### 1.1 Recommended workspace layout

```
happyterminals/
├── Cargo.toml                      # workspace root
├── crates/
│   ├── happyterminals-core/        # Signals, Effect, Memo, Grid, GridDiff, errors
│   ├── happyterminals-pipeline/    # Effect trait, Pipeline, tachyonfx adapter
│   ├── happyterminals-renderer/    # 3D rasterizer (voxcii-inspired), camera, primitives
│   ├── happyterminals-scene/       # Scene IR, scene graph, transition manager
│   ├── happyterminals-dsl/         # Rust builder API + JSON recipe loader/validator
│   ├── happyterminals-backend-ratatui/  # Grid → ratatui::Buffer, crossterm event loop
│   ├── happyterminals/             # META-CRATE: re-exports curated public API
│   └── happyterminals-py/          # PyO3 bindings (final milestone, optional member)
├── examples/                       # spinning-cube/, particles/, transitions/
└── recipes/                        # JSON recipe fixtures
```

### 1.2 Why this split (not the current three)

| Concern | Three-crate split (current) | Six-crate split (recommended) |
|---|---|---|
| Compile-time isolation of PyO3 | PyO3 in `core` ⇒ every rebuild pays its cost | Isolated in `-py` crate; doesn't touch `core` |
| Reasoning about boundaries | `core` carries reactive + Grid + Python at once | Each crate has one clear job |
| Backend swap (e.g. termion later) | Hard — backend logic mixed into core | Swap `-backend-*` independently |
| Public API surface | Each crate exports ad-hoc — no canonical "use happyterminals::" | Meta-crate `happyterminals` curates exports |
| JSON DSL evolution | Lives somewhere undefined | Lives in `-dsl`; can iterate without touching renderer |
| tachyonfx coupling | Coupled to compositor; swap path unclear | Adapter lives in `-pipeline` behind one trait |

### 1.3 Splits considered and rejected

- **Single mega-crate.** Rejected: harms compile times, doesn't let users opt in to subsets, makes the PyO3 boundary murky.
- **Keep `happyterminals-compositor` as a separate name.** Rejected: the word "compositor" overlaps confusingly with both the scene graph and the pipeline. Rename to `-pipeline` (effect chain) and `-scene` (graph + transitions). The compositor concept becomes a small struct inside `-scene` that owns layer ordering.
- **Vendor tachyonfx into the workspace.** Rejected: depend on it from crates.io. The adapter is small (one trait impl).
- **Separate `-effects` crate that wraps each tachyonfx effect with our own builders.** Defer — not needed until we have user data showing which effects need ergonomic Rust wrappers.

### 1.4 Migration from the current scaffold

The existing crates have already been seeded with workable code (`reactive.rs`, `grid.rs`). Migration is mostly *renames* and *cargo move* operations:

- `happyterminals-core` keeps `reactive.rs` + `grid.rs`, **drops** `python.rs` and the `pyo3`/`tui-vfx`/`ratatui` deps. Core should depend only on `std` (and `slotmap`/`smallvec` if useful). Currently it pulls all three; that's wrong.
- `happyterminals-compositor` becomes `happyterminals-pipeline`; its current `Compositor::compose(layers)` moves to `-scene` as `LayerStack::flatten(&mut Grid)`.
- `happyterminals-renderer` keeps its name and depends on `-core`.
- New crates created empty initially: `-scene`, `-dsl`, `-backend-ratatui`, `happyterminals` (meta), `happyterminals-py`.
- `Cargo.toml` (root) currently lists three members; add the new ones; exclude `-py` from default-members so `cargo build` skips it.

### 1.5 Public API curation (meta-crate)

```rust
// crates/happyterminals/src/lib.rs (sketch)
pub use happyterminals_core::{Signal, Effect, Memo, Grid, GridCell};
pub use happyterminals_pipeline::{Effect as PipelineEffect, Pipeline};
pub use happyterminals_scene::{Scene, SceneBuilder, Transition, Camera};
pub use happyterminals_renderer::{Renderer3D, Mesh, Primitive};
pub use happyterminals_dsl::{recipe, json};
pub use happyterminals_backend_ratatui::{Terminal, run};

pub mod fx { pub use happyterminals_pipeline::tachyon::*; }
```

Users write `use happyterminals::*;` and never need to know there are six crates underneath.

---

## 2. System Overview

```
┌──────────────────────────────────────────────────────────────────────┐
│                     User code (Rust DSL or Python)                   │
│   let r = Signal::new(0.0); scene().cube().rotation(r) ...           │
└──────────────────────────────┬───────────────────────────────────────┘
                               │ (builder calls or JSON recipe)
                               ▼
┌──────────────────────────────────────────────────────────────────────┐
│                    happyterminals-dsl                                 │
│   Builder API ─┐                                                      │
│   JSON loader ─┼──► Scene IR  (single canonical representation)      │
│   Python (-py)─┘                                                      │
└──────────────────────────────┬───────────────────────────────────────┘
                               ▼
┌──────────────────────────────────────────────────────────────────────┐
│                  happyterminals-scene                                 │
│   SceneGraph  ──  LayerStack  ──  TransitionManager  ──  Camera ref  │
│         ▲                                                             │
│         │ subscribes (Effect)                                         │
└─────────┼────────────────────────────────────────────────────────────┘
          │ Signals / Memos
┌─────────┴────────────────────────────────────────────────────────────┐
│                   happyterminals-core                                 │
│   Signal<T>   Memo<T>   Effect    Owner    Grid    GridCell           │
│   thread-local observer stack, weak-ref subscribers                   │
└─────────┬────────────────────────────────────────────────────────────┘
          │ (reads dirty scene, writes cells)
          ▼
┌──────────────────────────────────────────────────────────────────────┐
│                  happyterminals-renderer                              │
│   Renderer3D::draw(&Scene, &Camera, &mut Grid)                        │
│   z-buffer, perspective projection, ASCII shading ramp                │
└─────────┬────────────────────────────────────────────────────────────┘
          ▼
┌──────────────────────────────────────────────────────────────────────┐
│                  happyterminals-pipeline                              │
│   Pipeline { effects: Vec<Box<dyn Effect>> }                          │
│   trait Effect { fn apply(&mut self, &mut Grid, dt, area); }          │
│   ↑ adapter: TachyonAdapter wraps tachyonfx::Shader/Effect            │
└─────────┬────────────────────────────────────────────────────────────┘
          ▼
┌──────────────────────────────────────────────────────────────────────┐
│              happyterminals-backend-ratatui                           │
│   Grid → ratatui::Buffer (cell-by-cell copy, set skip flag)           │
│   Terminal::draw uses ratatui's built-in diff → minimal ANSI          │
│   crossterm EventStream feeds input Signals                           │
└──────────────────────────────────────────────────────────────────────┘
                               ▼
                    ANSI escapes on stdout
```

### Component responsibilities

| Component | Owns | Consumes | Produces |
|---|---|---|---|
| `core::Signal/Effect/Memo` | Reactive graph, owner stack | `T: Clone + 'static` | Re-runs of dependent observers |
| `core::Grid` | Cell buffer (Vec\<GridCell\>) | mutations | borrowed views |
| `scene::SceneGraph` | Node tree, z-ordered layers | Signals (via Effects on node properties) | Dirty scene snapshot per frame |
| `scene::TransitionManager` | Two scenes + transition effect | Signal\<TransitionState\> | Composite scene-of-scenes |
| `renderer::Renderer3D` | Z-buffer, projection matrices, mesh cache | Scene, Camera, &mut Grid | Cells written into Grid |
| `pipeline::Pipeline` | Vec\<Box\<dyn Effect\>\> | &mut Grid, dt | Transformed Grid |
| `pipeline::TachyonAdapter` | A `tachyonfx::Shader` | Grid view as `Buffer` | Effect applied |
| `dsl` | Builder structs, JSON schema, validator | User input | Scene IR |
| `backend-ratatui` | `ratatui::Terminal`, event stream | Final Grid | ANSI on stdout, Events into Signals |
| `happyterminals` (meta) | Re-exports | — | Public API surface |
| `happyterminals-py` | PyO3 module | Meta-crate types | Python wheel |

---

## 3. Data Flow — Signal Change to Rendered Cell

This is the load-bearing diagram. Trace it carefully because the entire framework's correctness rests on it.

```
                     ┌────────────────────────┐
   user code:        │ rotation.set(0.5)      │
                     └──────────┬─────────────┘
                                ▼
   ┌────────────────────────────────────────────────────────────────┐
   │ core: SignalState.value = 0.5                                  │
   │       walk weak observers, for each upgradable: enqueue        │
   └──────────┬─────────────────────────────────────────────────────┘
              ▼
   ┌────────────────────────────────────────────────────────────────┐
   │ Effect "cube_node.rotation = rotation()"  re-runs              │
   │   → writes 0.5 into SceneNode::Cube { rotation: Cell<f32> }    │
   │   → marks scene_dirty = true (atomic bool or version counter)  │
   └──────────┬─────────────────────────────────────────────────────┘
              ▼
   ┌────────────────────────────────────────────────────────────────┐
   │ FRAME TICK (33ms / 30fps) on render thread                     │
   │ 1. event-loop wakes from tokio::select! { tick, input }        │
   │ 2. drain pending input events into input Signals (set())       │
   │    -> may trigger more Effects, more scene mutations           │
   │ 3. if scene_dirty || pipeline_has_animated_effects:            │
   │      grid.clear()                                               │
   │      renderer.draw(&scene, &camera, &mut grid)                 │
   │      pipeline.apply(&mut grid, dt)                             │
   │      backend.blit(&grid)  // copies cells into ratatui::Buffer │
   │      terminal.flush()     // ratatui diffs → ANSI              │
   │      scene_dirty = false                                       │
   └────────────────────────────────────────────────────────────────┘
```

### Where the boundaries are

1. **Reactive ↔ scene** — Effects close over scene-node cells. *Effects mutate scene state but never directly touch the Grid.* This is the single most important rule: it prevents reactive callbacks from racing the renderer and lets us batch all state changes into one frame.
2. **Scene ↔ renderer** — The renderer takes `&Scene` (immutable read) and `&mut Grid` (write). It does *not* subscribe to signals itself. It runs once per frame, top to bottom.
3. **Renderer ↔ pipeline** — Pipeline takes `&mut Grid` only. No knowledge of scene. This is what makes effects composable across any source (3D scene, sprite scene, plain text widget).
4. **Pipeline ↔ backend** — Backend reads the final Grid and copies cells into ratatui's `Buffer`. ratatui's existing diff-against-previous-buffer mechanism handles minimal ANSI emission ([Ratatui rendering docs](https://ratatui.rs/concepts/rendering/under-the-hood/)).

### Where async fits

Async lives **only** at the I/O perimeter. Specifically:

- `crossterm::event::EventStream` (async input).
- `tokio::time::interval` for the 30 fps tick.
- Optional `tokio::fs` for async asset loading (OBJ/STL/JSON recipes).
- Optional user-supplied async tasks (e.g. fetching a value over the network and `signal.set()`-ing it).

Inside one frame, everything is **synchronous** on the render thread. This avoids the entire class of "what if an Effect awaits in the middle of a draw" bugs. Pattern is identical to the [Ratatui async-event-stream tutorial](https://ratatui.rs/tutorials/counter-async-app/async-event-stream/) and `fiadtui`-style wrappers.

### Backpressure when a frame takes too long

- Use `tokio::select!` with `tick = interval.tick(); input = stream.next()`.
- If render takes longer than the tick, **the next tick fires immediately** but we render at most once per loop iteration. Dropped ticks = naturally lower fps; we don't queue them.
- Expose `Telemetry { last_frame_us, dropped_ticks }` from the run loop so users can detect overruns.
- Long-running computations belong inside `Memo`s (cached) or in user-spawned tasks that `signal.set()` results when ready — *never* in Effects called by the render loop.

---

## 4. Reactive Core Design

### 4.1 Build vs reuse

Two viable paths:

| Path | Pros | Cons |
|---|---|---|
| **Build our own** (current `reactive.rs`) | ~150 LOC, no deps, exact control over Send/Sync, ASCII-art-debuggable | We maintain it; risk of subtle GC/cycle bugs |
| **Use `reactive_graph`** (Leptos's standalone crate, [crates.io](https://crates.io/crates/reactive_graph)) | Production-tested, owner trees, ArcRwSignal/ArcMemo, free upgrades | Bigger surface, indirect API, couples our public API to Leptos releases |

**Recommendation: build our own initially, behind a `ReactiveRuntime` trait that gives us a zero-cost migration path.** Reasons:
- Our needs are narrow: Signal\<T\>, Memo\<T\>, Effect, with `'static + Clone` bounds. The 150-LOC version covers this.
- Public API uses our own `Signal<T>`, not Leptos types. Users (and Python) get a stable surface independent of Leptos's release cadence.
- If we later hit edge cases (cycle detection, batching, async signals), swapping in `reactive_graph` is a localized refactor in `core`.
- Reference design is well-documented: Leptos's [reactive graph appendix](https://book.leptos.dev/appendix_reactive_graph.html) and [signal lifecycle](https://book.leptos.dev/appendix_life_cycle.html) describe slotmap-based ownership and thread-local observer stacks.

### 4.2 Send/Sync strategy

Three options, ordered by simplicity:

1. **Single-threaded core, message-passing across threads.** Reactive graph lives entirely on the render thread. Other threads (input, network, async loaders) communicate via `mpsc::Sender<Command>` where `Command` is "set this signal to this value." Signals stay `!Send`. This mirrors Leptos's thread-local owner pattern and is the cleanest model for a frame-driven app. **Recommended.**
2. **`Send + Sync` everything via `Arc<Mutex<_>>`.** Simpler API (signals usable anywhere) but lock contention and harder to reason about reentrancy.
3. **`reactive_graph`'s `ArcRwSignal`.** Already solved this trade-off; supports both.

The current `reactive.rs` uses `Rc<RefCell<_>>` and a `thread_local!` observer stack — that's option 1, which is right. Document the rule: **"all `Signal::set()` calls must happen on the render thread; cross-thread updates use the command channel."**

### 4.3 Public API sketch

```rust
// happyterminals-core
pub struct Signal<T>(/* Rc<RefCell<...>> */);
impl<T: Clone + 'static> Signal<T> {
    pub fn new(value: T) -> Self;
    pub fn get(&self) -> T;          // tracks current observer
    pub fn set(&self, value: T);     // re-runs subscribers eagerly
    pub fn update(&self, f: impl FnOnce(&mut T));
    pub fn untracked(&self) -> T;    // get without tracking — needed for renderer
}

pub struct Memo<T>(Signal<T>);
impl<T: Clone + PartialEq + 'static> Memo<T> {
    pub fn new(f: impl Fn() -> T + 'static) -> Self; // skips notifies if equal
    pub fn get(&self) -> T;
}

pub struct Effect;
impl Effect {
    pub fn new(f: impl FnMut() + 'static) -> Self;
    pub fn dispose(self);
}

// For cross-thread updates:
pub struct SignalSetter<T>(/* sender + signal id */);
impl<T: Send + 'static> SignalSetter<T> { pub fn send(&self, v: T); }
```

Two refinements to the existing stub before it's production-ready:
- **Memo equality skip** — currently every Effect re-run unconditionally `set()`s the inner signal. Add `PartialEq` bound and skip propagation when unchanged. Otherwise downstream effects fire on every parent recompute.
- **Untracked reads** — the renderer needs to walk scene state every frame *without* registering itself as an observer of every signal. Add `Signal::untracked()` (and a `with_untracked` scope guard). Mirrors Leptos's `Signal::get_untracked`.

---

## 5. Grid Model

### 5.1 Cell representation

Current stub is right:

```rust
pub struct GridCell {
    pub ch: char,
    pub style: ratatui::style::Style,  // packs fg/bg/Modifier
}
```

Two additions to consider before milestone 1:

- `pub skip: bool` — mirror ratatui's `Cell::skip` so we can punch holes for terminal-graphics protocols (Sixel/Kitty) and avoid clobbering them. See [ratatui Cell docs](https://docs.rs/ratatui/latest/ratatui/buffer/struct.Cell.html).
- Eventually: support for grapheme clusters / wide chars (CJK, emoji). Defer until needed; document as a known limitation. Wide chars require occupying two adjacent cells with the second marked "continuation."

### 5.2 Buffering strategy

**Recommendation: single Grid in core, no internal double buffering.** Let ratatui's `Terminal` do the diff at the backend boundary — that's what it's designed for, and it already produces minimal ANSI output ([Ratatui rendering](https://ratatui.rs/concepts/rendering/under-the-hood/)).

Sequence per frame:
1. `grid.clear()` (or partial clear if scene marks regions clean).
2. Renderer + pipeline write into `grid`.
3. Backend copies `grid` cells into `terminal.current_buffer_mut()`.
4. `terminal.flush()` diffs current vs previous buffer, emits ANSI.
5. ratatui swaps buffers.

Adding our own diff layer would duplicate ratatui's work. The only reason to add one would be if we wanted to skip the renderer pass entirely when no signals changed — that's the `scene_dirty` flag, not a second buffer.

### 5.3 Resize handling

- Backend listens for `Event::Resize(w, h)` from crossterm.
- Resize updates a `Signal<(u16, u16)>` exposed as `viewport()`.
- Memos that depend on `viewport()` recompute (e.g. camera aspect ratio).
- Backend reallocates `Grid` (cheap: `Vec::resize_with`).
- Next frame renders into the new size.

---

## 6. Pipeline

### 6.1 Trait design

```rust
pub trait Effect {
    /// Apply this effect to the grid in-place.
    /// `dt` is the elapsed time since the last frame.
    /// Returns whether the effect is still animating (false = can be removed).
    fn apply(&mut self, grid: &mut Grid, dt: Duration, area: Rect) -> EffectStatus;

    /// Optional: short name for debug output / JSON recipe round-trip.
    fn name(&self) -> &'static str { "effect" }
}

pub enum EffectStatus { Running, Complete }

pub struct Pipeline {
    effects: Vec<Box<dyn Effect>>,
}

impl Pipeline {
    pub fn new() -> Self;
    pub fn push(&mut self, fx: impl Effect + 'static);
    pub fn apply(&mut self, grid: &mut Grid, dt: Duration, area: Rect);
    pub fn from_recipe(recipe: &PipelineRecipe) -> Result<Self>;
}
```

### 6.2 Trait objects vs generic chains vs data-driven passes

| Approach | Compile-time perf | Runtime perf | DSL/JSON friendly | Verdict |
|---|---|---|---|---|
| `Vec<Box<dyn Effect>>` | Best | One vtable call per effect (negligible) | Yes — runtime construction | **Pick this** |
| `Pipeline<E1, E2, E3>` generics | Worst (monomorphization explosion) | Marginally faster | No — can't build at runtime from JSON | Reject |
| Data-driven pass list (enum) | Good | Fast | Yes but closed-set | Hybrid: built-in effects as enum variants for hot path, `dyn` for user-supplied |

The slight runtime cost of `dyn` is irrelevant — we're emitting ANSI to a 30 fps terminal. Flexibility wins.

### 6.3 tachyonfx integration

tachyonfx defines a `Shader` trait that operates on `&mut ratatui::Buffer`. Adapter:

```rust
// happyterminals-pipeline::tachyon
pub struct TachyonAdapter<S: tachyonfx::Shader>(S);

impl<S: tachyonfx::Shader> Effect for TachyonAdapter<S> {
    fn apply(&mut self, grid: &mut Grid, dt: Duration, area: Rect) -> EffectStatus {
        // Borrow grid as a ratatui Buffer view (or copy in/out if borrow doesn't fit)
        let mut buf = grid.as_ratatui_buffer_mut(area);
        self.0.process(dt.into(), &mut buf, area);
        if self.0.done() { EffectStatus::Complete } else { EffectStatus::Running }
    }
}
```

The "borrow `Grid` as `&mut ratatui::Buffer`" trick depends on whether our `GridCell` can be made layout-compatible with `ratatui::buffer::Cell`. **Recommendation:** make `Grid` *be* a `ratatui::Buffer` internally (newtype it). Saves the conversion entirely and makes the adapter trivial. See [tachyonfx docs](https://docs.rs/tachyonfx) and [ratatui Cell](https://docs.rs/ratatui/latest/ratatui/buffer/struct.Cell.html). Tradeoff: we inherit ratatui's Cell API surface, but that's fine — we already depend on ratatui's `Style`.

### 6.4 Built-in effects (beyond tachyonfx)

The pipeline crate ships a few of our own effects to prove the trait works without tachyonfx:
- `Clear` (set all cells to default).
- `Fill { ch, style }`.
- `Mask { region, inner: Box<dyn Effect> }` for clipping.

Everything else is tachyonfx via the adapter.

---

## 7. Scene Graph

### 7.1 Structure

```rust
pub struct Scene {
    pub layers: Vec<Layer>,         // back-to-front
    pub camera: Camera,
    pub viewport: Signal<(u16, u16)>,
}

pub struct Layer {
    pub z: i32,
    pub objects: Vec<SceneNode>,
    pub local_pipeline: Option<Pipeline>,  // per-layer effects (e.g. blur this layer only)
}

pub enum SceneNode {
    Mesh(MeshNode),       // 3D mesh (renderer-fed)
    Primitive(Primitive), // cube/sphere/plane (renderer-fed)
    Particles(ParticleSystem),
    Sprite(SpriteNode),   // 2D static text sprite
    Text(TextNode),
    Group(Vec<SceneNode>),
}
```

### 7.2 Z-order and compositing

- Layers render back-to-front into the same Grid.
- Within a layer, the renderer's z-buffer sorts pixels.
- Layer's optional `local_pipeline` runs after that layer is drawn but before the next layer.
- Scene's top-level `pipeline` runs after all layers (the global pipeline lives on `Scene`, not `Layer`).

### 7.3 Camera

Camera is **owned by the scene** but consumed by the renderer. Holds:
- Position (Vec3), look-at (Vec3), up (Vec3).
- FOV, near, far.
- Projection mode (perspective | orthographic).

Camera fields can be Signals (e.g. `position: Signal<Vec3>`) for reactive camera animations.

### 7.4 Transitions

```rust
pub struct TransitionManager {
    from: Option<Scene>,
    to: Scene,
    progress: Signal<f32>,        // 0.0 → 1.0
    transition: Box<dyn Effect>,  // a special effect that takes TWO Grids
}
```

Transition pattern:
1. Render `from` into `grid_a`, `to` into `grid_b`.
2. Transition effect blends them: `apply(&mut grid_a, &grid_b, progress) → grid_a`.
3. When `progress == 1.0`, drop `from`, replace with `to`, become a normal scene.

This means the transition is *itself* a generalization of `Effect` — call it `BlendEffect` and it's a separate trait or a marker variant.

---

## 8. DSL & JSON Recipes

### 8.1 Single internal IR

The single most important architectural decision in `-dsl`:

```rust
// happyterminals-dsl::ir
#[derive(Serialize, Deserialize)]
pub struct SceneIr {
    pub layers: Vec<LayerIr>,
    pub camera: CameraIr,
    pub pipeline: Vec<EffectIr>,
}

#[derive(Serialize, Deserialize)]
pub enum NodeIr {
    Cube { position: [f32;3], rotation: BindIr<f32>, ... },
    Mesh { path: String, ... },
    Particles { count: u32, gravity: f32, ... },
    // ...
}

#[derive(Serialize, Deserialize)]
pub enum BindIr<T> {
    Const(T),
    Signal(SignalRef),  // reference by name; resolved at scene-construction time
}
```

Both front-ends compile to `SceneIr`:

```
Rust builder API ──┐
                   ├──► SceneIr ──► validate ──► Scene
JSON recipe ──────┘                              ▲
                                                 │
Python builder ─────────────────────────────────┘  (later milestone)
```

### 8.2 Builder API (Rust)

```rust
let r = Signal::new(0.0);
let scene = scene()
    .layer(|l| l
        .cube(cube().position([40.0, 12.0, 0.0]).rotation(&r))
        .particles(particles().count(50).gravity(-0.1)))
    .camera(camera().orbit().distance(20.0))
    .effect(fx::vignette(0.3))
    .effect(fx::color_ramp("dracula"))
    .build()?;
```

Builder methods produce IR fragments; `.build()` validates and instantiates a `Scene`.

### 8.3 JSON schema (versioned)

```json
{
  "$version": 1,
  "scene": { ... }   // mirrors SceneIr
}
```

Use `serde` for round-trip. Validation lives in `dsl::validate(&SceneIr) -> Result<()>`. Version field future-proofs schema migrations.

### 8.4 Signal binding from JSON

JSON can't carry closures, so signal bindings reference named signals registered with the runtime:

```json
{ "rotation": { "$signal": "user.rotation" } }
```

User code:
```rust
let r = Signal::new(0.0);
let scene = json::load(path)?.bind("user.rotation", &r).build()?;
```

This is the cleanest way to bridge declarative (JSON) and reactive (signals).

---

## 9. PyO3 Boundary

The Python milestone is last, but architecture today must not block it. Key decisions:

### 9.1 What crosses the FFI boundary

- `Signal<T>` for `T ∈ {f32, f64, i32, bool, String}` exposed as `PySignal`. Generic `T` doesn't cross FFI; we expose a fixed set of variants.
- `SceneBuilder` exposed as a Python class with chainable methods.
- `Pipeline`/`Effect` references — opaque handles; effects constructed via factory functions (`fx.vignette(0.3)`).
- `run(scene, fps=30)` — the event loop. Releases GIL for the entire frame loop.
- IR loader: `happyterminals.load_recipe(path)` parses JSON in Rust, returns a `Scene` handle.

### 9.2 Zero-copy

For `Grid` access from Python (e.g. custom Python effects), expose via Python's buffer protocol. `Grid`'s internal `Vec<GridCell>` is contiguous and `repr(C)`-able; `PyBuffer<u8>` lets numpy view it as a struct array. See [PyO3 buffer protocol](https://docs.rs/pyo3/latest/pyo3/buffer/index.html). This is a v0.2 feature; v0.1 should expose Grid only as opaque.

### 9.3 GIL handling

```rust
#[pyfunction]
fn run(py: Python<'_>, scene: PyScene, fps: u32) -> PyResult<()> {
    py.allow_threads(|| {
        // entire event loop runs without holding GIL
        // re-acquire GIL only when calling back into Python (e.g. user effect)
        run_event_loop(scene, fps)
    });
    Ok(())
}
```

User-supplied Python callbacks (e.g. an `on_click` handler) require GIL re-acquisition — wrap those in `Python::with_gil`. Pattern is standard; see [PyO3 GIL discussion](https://github.com/PyO3/pyo3/discussions/1912).

### 9.4 asyncio integration

Punt to v0.2. Initial Python API is sync `run()`. asyncio integration via `pyo3-asyncio` is feasible but the ecosystem is not stable (per PyO3 maintainers' own comments). Sync run + Python threads covers the 90% case for creative coders.

### 9.5 Architectural implications today

To not paint ourselves into a corner:
- Reactive runtime must not assume the calling thread is the main thread (i.e. don't use `std::main_thread!`-style assertions).
- All public types in the meta-crate need to be `'static` and mostly `Clone` so PyO3 wrappers can hold them.
- Avoid lifetimes in public APIs — Python wrappers can't carry Rust lifetimes.
- Effects and Renderers should not require `Send`/`Sync` (PyO3 enforces this only at the module export boundary, and we run single-threaded inside the loop anyway). But tachyonfx effects *are* `Send` already, so this is a non-issue for built-ins.

---

## 10. Event Loop & Frame Scheduling

### 10.1 Skeleton (in `-backend-ratatui`)

```rust
pub async fn run(scene: Scene, fps: u32) -> Result<()> {
    let mut terminal = ratatui::Terminal::new(...)?;
    let mut events = crossterm::event::EventStream::new();
    let frame_dur = Duration::from_millis(1000 / fps as u64);
    let mut tick = tokio::time::interval(frame_dur);
    let mut grid = Grid::new(/* viewport */);
    let mut last = Instant::now();

    loop {
        tokio::select! {
            _ = tick.tick() => {
                let now = Instant::now();
                let dt = now - last; last = now;

                // Drain any pending input that arrived between ticks
                while let Ok(Some(ev)) = events.try_next().now_or_never() { dispatch(ev); }

                if scene.is_dirty() || pipeline.is_animating() {
                    grid.clear();
                    renderer.draw(&scene, &scene.camera, &mut grid);
                    pipeline.apply(&mut grid, dt, grid.area());
                    backend::blit(&grid, terminal.current_buffer_mut());
                    terminal.flush()?;
                    scene.mark_clean();
                }
            }
            ev = events.next() => {
                if let Some(Ok(ev)) = ev { dispatch(ev); }
            }
        }
    }
}
```

### 10.2 Backpressure

- Render budget = `1000/fps` ms (33 ms at 30 fps).
- If render exceeds budget, the next `tick.tick()` fires immediately; we render once more, then drift back to schedule. `tokio::time::interval` uses `MissedTickBehavior::Burst` by default — switch to `Delay` to skip missed ticks rather than burst.
- Track `last_frame_us` in a Telemetry struct; expose as `Signal<FrameStats>` so user code can react (e.g. lower particle count if frames drop).
- `scene.is_dirty()` short-circuits idle frames — when nothing changed, we skip the entire pipeline. The pipeline's `is_animating()` check handles continuous effects (e.g. vignette pulsing).

### 10.3 Input dispatch

- `crossterm::Event::Key(k)` → resolve binding → call user callback (which may `signal.set()`).
- `Event::Mouse(m)` → similar.
- `Event::Resize(w,h)` → `viewport_signal.set((w,h))`; reallocate grid before next render.
- Provide a built-in `keyboard()` Signal\<Option\<KeyEvent\>\> for users who want raw access.

---

## 11. Build Order & Milestone Implications

### 11.1 Dependency DAG

```
core ──► pipeline ──► renderer ──► scene ──► dsl ──► backend-ratatui ──► happyterminals (meta) ──► happyterminals-py
                  └──────────────────────────────────────────────────► happyterminals (meta)
```

Strictly bottom-up at the crate level. Within a milestone, the sensible *vertical-slice* path is:

### 11.2 Suggested phase decomposition

| Phase | Crates touched | Exit criterion |
|---|---|---|
| **1.0 Reactive core** | `core` | `Signal/Effect/Memo` work; `untracked()` added; Memo skips on equality; tests cover diamond deps and disposal |
| **1.1 Grid + ratatui backend (static)** | `core`, `backend-ratatui` | Render a hand-built static Grid to the terminal via ratatui; resize works |
| **1.2 Pipeline + tachyonfx adapter** | `pipeline` | Apply a tachyonfx effect (e.g. vignette) to a static Grid in the demo |
| **1.3 Minimal renderer (one primitive)** | `renderer` | Draw a single rotating cube primitive with z-buffer + ASCII shading; no mesh loading yet |
| **1.4 Scene IR + builder API** | `scene`, `dsl` | Build a Scene programmatically via the Rust builder; render it through the full pipeline |
| **1.5 Spinning cube end-to-end demo** | All above + `happyterminals` meta | Project's stated Milestone 1 exit: signal-driven rotation → 3D → vignette → ratatui → terminal in a single binary |
| **2.x Renderer deepening** | `renderer` | OBJ/STL loading, camera modes, particles, L-systems |
| **2.x Scene deepening** | `scene` | Multi-layer compositing, transitions between scenes |
| **2.x DSL deepening** | `dsl` | JSON recipe loader + validator + signal binding |
| **3.x PyO3 bindings** | `happyterminals-py` | Python wheel ships; spinning cube demo rewrittten in Python |

### 11.3 Why this order

- Each phase produces a **runnable artifact** (test, example, or demo).
- Dependencies always point *down* the stack — no phase needs a later one.
- The renderer can be the simplest possible cube primitive at first; deepening it (mesh loading, particles) is independent of everything above.
- DSL and JSON wait until Scene IR exists, which waits until Scene exists, which waits until Pipeline + Renderer exist.
- PyO3 last is the right call — it locks in the public API, which we don't want to lock in until we've used it ourselves.

### 11.4 Risk hotspots for the roadmapper

- **Reactive core memory model.** Risk of accidental cycles between Signals and Effects. Mitigation: weak refs (already done), explicit `Effect::dispose`, integration tests that count live observers.
- **tachyonfx Buffer compatibility.** If `GridCell` and `ratatui::buffer::Cell` aren't layout-compatible, the adapter requires a copy in/out per effect — fine for correctness, bad for performance with chained effects. Mitigation: newtype `Grid` over `ratatui::Buffer` from the start.
- **PyO3 + reactive single-thread invariant.** Python users will expect to call `signal.set()` from any thread. Need a SignalSetter / channel pattern documented and tested before the Python milestone.
- **Frame budget overruns at high effect counts.** No mitigation needed at MVP, but instrument from day one (`last_frame_us` telemetry signal).

---

## 12. Anti-Patterns

### 12.1 Effects mutating the Grid directly

**What people do:** Inside an `Effect` callback, write into the Grid.
**Why it's wrong:** Effects can run at arbitrary times (any `signal.set()`); writing to the Grid races the renderer and produces tearing/flicker.
**Do this instead:** Effects mutate scene state only. The render loop walks the scene once per frame.

### 12.2 Subscribing the renderer to signals

**What people do:** Inside `Renderer3D::draw`, call `signal.get()` so the renderer "auto-rebuilds when signals change."
**Why it's wrong:** This makes the renderer an Observer of every signal in the scene; one change triggers a full re-render synchronously, defeating the frame model and possibly stack-overflowing on diamond deps.
**Do this instead:** Use `signal.untracked()` inside the render path. Subscriptions belong to scene-node Effects.

### 12.3 Per-effect double buffering

**What people do:** Each effect allocates its own scratch Grid, copies in, transforms, copies out.
**Why it's wrong:** Allocation per-effect-per-frame ruins cache + GC pressure; chained effects all pay copy cost.
**Do this instead:** All effects mutate the same `&mut Grid` in place. Effects that need a "before" view of the Grid take it as a separate `&Grid` argument (the transition pattern).

### 12.4 Putting PyO3 in the core crate

**What people do:** Add `#[pyclass]` annotations to `core::Signal`. (The current scaffold does this.)
**Why it's wrong:** Drags PyO3 + Python headers into every downstream crate's compilation, slows iteration, couples API design to PyO3 constraints.
**Do this instead:** Keep PyO3 in `happyterminals-py`. That crate wraps core types in newtype `PySignal(pub Signal<f64>)` and exposes them.

### 12.5 Generic `Pipeline<E1, E2, E3, ...>` chains

**What people do:** Use type-state / generics so the pipeline is monomorphized.
**Why it's wrong:** JSON recipes and Python both need runtime construction, which generics block. Monomorphization explodes compile time for marginal runtime gain on a 30 fps loop.
**Do this instead:** `Vec<Box<dyn Effect>>`. Vtable cost is irrelevant at this scale.

### 12.6 Splitting Grid into "logical" and "physical"

**What people do:** Two grids — one in "scene units," one in "terminal cells" — with a transform pass between them.
**Why it's wrong:** Doubles the work, doubles the bugs, and the renderer already projects to terminal cells natively.
**Do this instead:** One Grid, in terminal cells. Renderer projects 3D coords directly into cell space.

### 12.7 Building our own ANSI diff

**What people do:** "Optimize" by writing a custom Grid → ANSI diffing layer.
**Why it's wrong:** ratatui already does this and does it well ([Ratatui rendering](https://ratatui.rs/concepts/rendering/under-the-hood/)). Reinventing it adds bugs without performance wins.
**Do this instead:** Copy our Grid into `terminal.current_buffer_mut()` and let ratatui flush.

---

## 13. Integration Points

### 13.1 External crates we depend on

| Crate | Where used | Integration pattern | Notes |
|---|---|---|---|
| `ratatui` | `core` (Style, Color, Modifier), `backend-ratatui` (Buffer, Terminal) | Direct dep. Newtype Grid over Buffer. | Pulls in `crossterm` transitively. |
| `tachyonfx` | `pipeline` (adapter only) | Wrap `tachyonfx::Shader` in `TachyonAdapter` impl Effect. | Their effect DSL gives us 50+ effects free. |
| `crossterm` | `backend-ratatui` (event stream) | Async `EventStream` feeds input Signals via setter channel. | `event-stream` feature required. |
| `serde` / `serde_json` | `dsl` | Round-trip `SceneIr ↔ JSON`. | Versioned schema. |
| `pyo3` | `happyterminals-py` only | Newtype wrappers around core types; `Python::allow_threads` for run loop. | Optional workspace member. |
| `slotmap` (optional) | `core` | Allocate signals/effects in arena (Leptos-style). | Defer until perf demands it. |
| `tokio` | `backend-ratatui` | Async runtime for event loop only. | Could also use `smol`; document choice. |

### 13.2 Internal boundaries

| Boundary | Communication | Notes |
|---|---|---|
| `core` ↔ `pipeline` | Direct call (Pipeline takes `&mut Grid`) | No reactive coupling; pipeline doesn't subscribe |
| `core` ↔ `scene` | Effects subscribe to signals | Scene-node properties are signals or constants |
| `scene` ↔ `renderer` | Renderer takes `&Scene, &mut Grid` | Renderer reads scene with `untracked()`; never subscribes |
| `renderer` ↔ `pipeline` | Sequenced — renderer writes Grid, pipeline mutates it | No shared state |
| `pipeline` ↔ `backend` | Backend reads final Grid | Single-direction copy |
| `backend` ↔ `core` | Backend pushes input events into Signals via setter channel | One channel per app, drained on render thread |
| Meta-crate ↔ `-py` | `-py` re-exports meta-crate types as PyClasses | Newtype wrappers only |

---

## 14. Open Questions for Phase-Level Research

These don't block roadmap creation but should be flagged for the relevant phase:

1. **Memo equality semantics.** Should `Memo<T>` require `T: PartialEq`? Cost of comparison vs cost of unnecessary downstream re-runs. Decide in the reactive-core phase.
2. **Wide-character handling in Grid.** When does this become a real user request? Likely deferable past Milestone 1.
3. **Mesh loading library.** `tobj` for OBJ is well-known. STL: `stl_io` or roll our own (formats are simple). Decide in renderer phase.
4. **JSON schema versioning strategy.** Probably semver via `$version` field. Schema migrations TBD when breaking changes happen.
5. **pyo3-asyncio adoption.** Defer until v0.2 of the Python binding. Re-research at that point.
6. **Multi-window / multi-terminal.** Out of scope per Project doc; if revisited from 999.x backlog, requires breaking the "one Terminal per app" assumption in `backend-ratatui`.

---

## Sources

- [Leptos: Appendix — How Does the Reactive System Work?](https://book.leptos.dev/appendix_reactive_graph.html) — owner trees, slotmap-backed signals
- [Leptos: Appendix — Life Cycle of a Signal](https://book.leptos.dev/appendix_life_cycle.html) — signal/effect disposal model
- [Leptos: Owner type docs](https://docs.rs/leptos/latest/leptos/reactive/owner/struct.Owner.html) — thread-local owner stack
- [reactive_graph crate (standalone Leptos reactivity)](https://crates.io/crates/reactive_graph) — usable independently of Leptos UI
- [Xilem architecture blog post (Raph Levien)](https://raphlinus.github.io/rust/gui/2022/05/07/ui-architecture.html) — view tree + adapt nodes message-passing model
- [Xilem repo](https://github.com/linebender/xilem) — current reactive UI architecture in Rust
- [tachyonfx repo](https://github.com/junkdog/tachyonfx) — Effect/Shader trait, composable DSL
- [tachyonfx docs.rs](https://docs.rs/tachyonfx) — Shader trait API surface
- [Ratatui ecosystem — tachyonfx page](https://ratatui.rs/ecosystem/tachyonfx/) — integration patterns
- [Ratatui rendering under the hood](https://ratatui.rs/concepts/rendering/under-the-hood/) — double-buffer diff against previous frame
- [Ratatui Cell docs](https://docs.rs/ratatui/latest/ratatui/buffer/struct.Cell.html) — Cell layout including `skip` flag
- [Ratatui async event stream tutorial](https://ratatui.rs/tutorials/counter-async-app/async-event-stream/) — tokio::select! pattern with frame ticker
- [PyO3 docs — Buffer Protocol](https://docs.rs/pyo3/latest/pyo3/buffer/index.html) — zero-copy data sharing
- [PyO3 GIL release discussion](https://github.com/PyO3/pyo3/discussions/1912) — `Python::allow_threads` patterns
- [Alex Gaynor — Buffers on the edge: Python and Rust](https://alexgaynor.net/2022/oct/23/buffers-on-the-edge/) — buffer-protocol soundness

---
*Architecture research for: declarative reactive terminal scene manager (Rust workspace, future PyO3 binding)*
*Researched: 2026-04-14*
