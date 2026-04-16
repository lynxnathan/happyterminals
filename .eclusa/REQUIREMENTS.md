# Requirements: happyterminals

**Defined:** 2026-04-14
**Core Value:** Terminal art should feel like magic, not plumbing. A tiny declarative scene description should produce cinematic, reactive, cross-terminal output without touching ANSI escapes, buffers, or draw calls.

## v1 Requirements

Requirements for the public v1 release — everything up to and including Python bindings, which is the final milestone per PROJECT.md. Each maps to a roadmap phase (see Traceability). Grouped by subsystem using categories from `.eclusa/research/FEATURES.md` and `.eclusa/research/ARCHITECTURE.md`.

### Workspace Hygiene (prerequisite to any feature work)

- [ ] **HYG-01**: README.md, project.md, and per-crate READMEs consistently describe `tachyonfx` as the effects layer (no stray `tui-vfx` or "Haskell bindings" references outside an explicit rationale section)
- [ ] **HYG-02**: Stub crates (`happyterminals-core`, `-renderer`, `-compositor`) are stripped of speculative dependencies — no `tui-vfx`, no `pyo3` in core, nothing listed before a real call site exists
- [ ] **HYG-03**: Cargo workspace uses `[workspace.dependencies]` so all member crates inherit a single pinned version set (ratatui-core 0.1, tachyonfx 0.25, glam 0.32, reactive_graph 0.2, pyo3 0.28, etc.)
- [ ] **HYG-04**: Vendored reference copies moved to `vendor/_reference/{pyo3,ratatui,tui-vfx}/` with a `STAMP.txt` recording upstream commit and date; `.gitattributes` marks them `linguist-vendored=true`; never referenced via `path =` deps
- [ ] **HYG-05**: Dual LICENSE-MIT + LICENSE-APACHE files at repo root; every crate's `Cargo.toml` uses SPDX `license = "MIT OR Apache-2.0"`; README + CONTRIBUTING have the standard Apache-2.0 §5 contribution clause
- [ ] **HYG-06**: `happyterminals` crate name reserved on crates.io AND `happyterminals` package reserved on PyPI (plus `happyterminals-core`, `happyterminals-py`, `happy-terminals`) before any public announcement
- [ ] **HYG-07**: `rust-toolchain.toml` pins Rust 1.86 with `clippy` + `rustfmt` components; all crates compile clean on that toolchain
- [ ] **HYG-08**: CI baseline: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --workspace`, `cargo doc -D warnings`, `cargo tree -d` (duplicate detection), `cargo udeps`/`cargo-machete` (unused deps)
- [ ] **HYG-09**: Doc-lint CI step fails the build if forbidden strings appear outside approved rationale sections (`tui-vfx`, `Haskell bindings`, `pyo3-asyncio`, `cgmath`, `tui-rs`)

### Reactive Core (`happyterminals-core`)

- [x] **REACT-01**: `Signal<T>` primitive with fine-grained dependency tracking — reading inside an Effect or Memo registers a subscription; `signal.set(x)` triggers dependents only if the value actually changed
- [x] **REACT-02**: `Memo<T>` caches derived values; recomputes only when tracked dependencies change; downstream consumers only re-run when the memo's own value changes (PartialEq equality-skip decision documented in the phase)
- [ ] **REACT-03**: `Effect` runs an arbitrary side-effect closure whenever any tracked signal/memo read inside it changes
- [ ] **REACT-04**: Owner/scope tree: `create_root` + `Owner::run_in` + `on_cleanup` — when an owner is disposed, all its effects and memos are cleaned up, no leaks
- [x] **REACT-05**: `batch(|| { ... })` groups multiple signal updates so dependents run once with the final state
- [ ] **REACT-06**: Two-phase propagation (mark dirty → recompute in topo order); cycle detection panics with a clear "signal cycle A→B→A" message, not stack overflow
- [x] **REACT-07**: `signal.untracked()` lets the renderer read current values without registering a dependency; it's the only way the render loop reads scene state
- [x] **REACT-08**: Single-threaded runtime + `SignalSetter` channel: cross-thread writes go through a `Send` handle drained on the render thread each tick. Threading rules documented in every public reactive API docstring.
- [ ] **REACT-09**: `Clock` and `Rng` are trait-injected so snapshot/property tests are deterministic; the production default is `SystemClock` + `thread_rng`
- [ ] **REACT-10**: 10k-scene-transition test keeps RSS under a documented ceiling (proves owner disposal works); diamond-dependency test proves a signal firing once triggers each downstream exactly once

### Grid (`happyterminals-core`)

- [x] **GRID-01**: `Cell` holds one grapheme cluster (not a `char`) plus display width, fg/bg colors (RGB+256+16+auto-fallback), and modifiers (bold/italic/underline/reverse)
- [x] **GRID-02**: `Grid` is a newtype over `ratatui::Buffer` (or layout-equivalent); a 1–2 day spike in Phase M1.1 verifies layout compatibility before the newtype lands in public API
- [x] **GRID-03**: `Grid::put_str(x, y, s, style)` handles multi-byte characters, wide (east-asian, emoji) cells, combining marks, and ZWJ sequences correctly via `unicode-segmentation` + `unicode-width` (or `runefix-core`)
- [x] **GRID-04**: No custom ANSI diff layer — Grid cells blit into `ratatui::Buffer` and `terminal.flush()` emits minimal ANSI. M1 exit includes a test: "one signal change → ≤ ~10 bytes written to TTY"
- [x] **GRID-05**: Grid clears cleanly on resize; resize events are drained between frames (no resize during rasterization); Windows-Terminal resize is tested

### Pipeline + Effects (`happyterminals-pipeline`)

- [x] **PIPE-01**: `Effect` trait: `fn apply(&mut self, grid: &mut Grid, dt: Duration) -> EffectState`; returns `Running`/`Done`. Effects are stateful (hold timers, progress).
- [x] **PIPE-02**: `Pipeline` is `Vec<Box<dyn Effect>>` — trait objects, not generic chains, so JSON recipes and Python can construct pipelines at runtime
- [x] **PIPE-03**: `TachyonAdapter<S: tachyonfx::Shader>` wraps a tachyonfx effect as one of our `Effect` trait objects (with real `Duration` dt forwarded)
- [x] **PIPE-04**: `tachyonfx::Effect` is aliased to `Fx` in our public surface so our `Effect` trait name stays unambiguous (decision committed before any Pipeline consumer lands)
- [x] **PIPE-05**: Whole-grid composition passes — no per-object effect loops that go O(objects × effects); criterion bench documents the floor
- [x] **PIPE-06**: At least 10 tachyonfx effects wired end-to-end and smoke-tested (vignette, dissolve, fade_in, fade_out, crt, color_ramp, typewriter, sweep, slide, and one glitch/shader effect)
- [x] **PIPE-07**: Effects mutate scene state only through the reactive channel — **never** mutate `Grid` from outside a Pipeline apply. Documented invariant, lint-enforced where possible.

### 3D Renderer (`happyterminals-renderer`)

- [x] **REND-01**: Fresh (not forked) z-buffer rasterizer with perspective projection, configurable ASCII shading ramp (default 10 levels `` .:-=+*#%@``), flat/per-face shading sufficient for M1
- [x] **REND-02**: Cell aspect ratio is a public projection-API parameter (default 2:1 tall); a cube rendered with defaults looks cubic, not tower-shaped
- [x] **REND-03**: Reversed-Z + scene-fit near/far planes prevent visible z-fighting on the spinning cube demo at default resolutions
- [x] **REND-04**: Built-in `Cube` primitive in the renderer (M1 demo dependency); more primitives (sphere, plane, torus) are v2
- [x] **REND-05**: Orbit camera with signal-driven azimuth/elevation/distance (M1); free and FPS cameras are v2
- [x] **REND-06**: OBJ mesh loading via `tobj` with triangulation of quads, winding normalization, flat-normal fallback for missing normals; corpus of 10+ real-world OBJ files tested; load errors return `Result<Mesh, MeshError>`, never panic
- [ ] **REND-07**: Particle system infrastructure (emitter, gravity, lifetime, color over time); at least one particle example runs
- [x] **REND-08**: Color-mode pipeline: RGB → 256 → 16 → monochrome fallback; `NO_COLOR` env var honored; `--force-color` override; tmux `Tc` truecolor guidance in docs
- [x] **REND-09**: Per-frame allocation budget enforced via criterion bench — reusable string buffers, cached SGR escape sequences, no heap churn in the hot path
- [ ] **REND-10**: STL mesh loading via `stl_io` (v2 — post-OBJ)

### Scene & Transitions (`happyterminals-scene`)

- [x] **SCENE-01**: `SceneIr` is the one intermediate representation; Rust builder, JSON, and Python front-ends all produce it
- [x] **SCENE-02**: `SceneGraph` supports layered composition with explicit z-order; camera is owned by the scene (not a global)
- [x] **SCENE-03**: Scene node props can be `Signal<T>`, plain `T`, or `Memo<T>` — fine-grained subscription at the prop level
- [x] **SCENE-04**: `TransitionManager` handles scene A → scene B with a named effect (dissolve, slide, etc.); outgoing scene's owner is disposed cleanly
- [x] **SCENE-05**: Scene construction is `Result<Scene, SceneError>` — invalid scenes (missing refs, bad signal bindings) fail at build time, not render time

### Declarative DSL (`happyterminals-dsl`)

- [x] **DSL-01**: Rust builder API shaped after react-three-fiber: tree of typed nodes with props, props can be signals — e.g. `scene().layer(|l| l.cube().rotation(&r).position(vec3(0., 0., 0.))).effect(fx::vignette(0.3)).build()?`
- [x] **DSL-02**: "Hello world" for a user (open an editor → spinning cube on screen) is ≤25 lines of Rust including imports
- [x] **DSL-03**: Errors are `Result`-based on the public API surface; `clippy::unwrap_used`/`clippy::expect_used` denied in every library crate
- [ ] **DSL-04**: JSON recipe loader (`-dsl::json`) accepts a JSON scene, validates against a schemars-generated schema via jsonschema, binds named signal references, and produces a `SceneIr` identical to the Rust builder path
- [ ] **DSL-05**: JSON recipes are pure data — effect names resolve through a static registry; no `eval`, no shell-out, no mesh paths that escape a user-defined sandbox
- [ ] **DSL-06**: Round-trip property test: Rust builder → SceneIr → JSON → SceneIr produces identical render output
- [ ] **DSL-07**: JSON schema is versioned (`$version` field); schema migrations remain TBD until the first breaking change
- [ ] **DSL-08**: ANSI-injection sanitization on any user-provided string that ends up in a Grid cell

### Ratatui Backend (`happyterminals-backend-ratatui`)

- [x] **BACK-01**: `run(scene, FrameSpec)` drives a `tokio::select!` loop between a frame ticker (30/60 fps) and `crossterm::EventStream`; all frame work is synchronous on the render thread
- [x] **BACK-02**: `TerminalGuard` RAII + panic hook restore cursor, raw mode, alternate-screen buffer, mouse capture, and SGR state even on panic; Ctrl-C in any example leaves a sane shell
- [x] **BACK-03**: Input events propagate into scene signals (key, mouse, resize, focus)
- [x] **BACK-04**: Meta crate `happyterminals` re-exports a curated public surface (signals, scene builder, fx preludes, run, common types); users typically write `use happyterminals::prelude::*;`
- [x] **BACK-05**: Cross-terminal verification matrix — Windows Terminal, GNOME Terminal, macOS Terminal.app, iTerm2, Kitty, Alacritty, tmux + screen, SSH session; spinning cube runs on all of them before M1 exit

### Milestone 1 Demo Exit

- [x] **DEMO-01**: `examples/spinning-cube/` is a single Rust file under 100 LOC that renders a signal-rotated cube with one tachyonfx effect (e.g., vignette), running at 30fps, no visible flicker, no memory growth over a 10-minute run
- [ ] **DEMO-02**: Project root `README.md` shows the spinning cube with an animated GIF/asciicast + the ≤25-line hello-world
- [x] **DEMO-03**: Ctrl-C during the demo leaves the terminal in a clean state on every platform in the verification matrix
- [x] **DEMO-04**: One-cell-change output test: mutating one signal produces approximately one cell of ANSI change (bytes order-of-magnitude sane, not a full-buffer repaint)

### Public Release (crates.io + v1)

- [ ] **REL-01**: Six crates published to crates.io: `happyterminals-core`, `-pipeline`, `-renderer`, `-scene`, `-dsl`, `-backend-ratatui`, `happyterminals` (meta); all with proper `description`, `license`, `repository`, `keywords`, `categories`, `readme`
- [ ] **REL-02**: Keep-a-Changelog `CHANGELOG.md` at root; `cargo semver-checks` runs on every PR
- [ ] **REL-03**: MSRV policy documented (floor = Rust 1.86); CI tests MSRV + stable
- [ ] **REL-04**: At least 5 runnable examples beyond the cube (mesh viewer, particles, transitions, JSON recipe loader, audio-reactive stub is NOT v1)
- [ ] **REL-05**: `docs.rs` builds every crate with every feature; doc-lint CI denies broken intra-doc links and missing top-level docs

### Python Bindings (FINAL MILESTONE per PROJECT.md)

- [ ] **PY-01**: `happyterminals-py` is a `cdylib` PyO3 crate, excluded from the default workspace members, built only via maturin
- [ ] **PY-02**: PyO3 0.28 `Bound<'py, T>` API throughout — no `&PyAny`, no `IntoPy`; `pyo3-async-runtimes` 0.28 used if asyncio integration ships (never `pyo3-asyncio`)
- [ ] **PY-03**: Python API mirrors the Rust builder shape — `happyterminals.signal`, `happyterminals.scene`, `happyterminals.fx`, `happyterminals.run`
- [ ] **PY-04**: Primary entry point is sync `run(scene, fps=30)` (ARCH §9.4 recommendation); asyncio-first is a v2 decision unless user overrides before this phase
- [ ] **PY-05**: GIL released during the render loop via `Python::allow_threads`; Python-side signal writes go through the same `SignalSetter` channel used by cross-thread Rust writers
- [ ] **PY-06**: Default cross-boundary semantics are copy; zero-copy is opt-in via explicit `freeze()`/`lock()` methods, never implicit
- [ ] **PY-07**: abi3 wheels built via maturin for CPython 3.10–3.13 on Linux x86_64/aarch64, macOS x86_64/aarch64 (universal2), and Windows x86_64; PyPI Trusted Publishing for release
- [ ] **PY-08**: Type stubs (`.pyi`) shipped; `mypy --strict` on the example code passes
- [ ] **PY-09**: Python-side "hello spinning cube" is ≤10 lines and installs cleanly via `pip install happyterminals` on every supported platform
- [ ] **PY-10**: Python README explicitly documents the single-threaded reactive model and the `SignalSetter` pattern for cross-thread updates

## v2 Requirements

Deferred beyond the Python bindings milestone. Tracked but not in the current roadmap; revisit after v1 ships and user feedback lands.

### Audio-Reactive

- **AUD-01**: FFT of audio input → signal values (chroma-style)
- **AUD-02**: Built-in effect presets driven by bands

### AI Integration

- **AI-01**: Out-of-tree CLI that takes a prompt and emits a valid JSON recipe
- **AI-02**: Schema-constrained generation so LLMs produce valid scenes by construction

### Live Coding

- **REPL-01**: Stateful REPL that hot-reloads scene definitions without terminal flicker
- **REPL-02**: File-watch mode for JSON recipes

### Advanced Rendering

- **ADV-01**: GLSL → ASCII shader transpiler (subset)
- **ADV-02**: Multi-terminal scenes (single scene tiled across N terminals)
- **ADV-03**: WASM runtime — render happyterminals scenes in the browser via `xterm.js`
- **ADV-04**: Visual scene editor (GUI tool, mirrors tachyonfx's browser editor for full scenes)

### Additional Primitives

- **PRIM-01**: L-systems / generative geometry
- **PRIM-02**: Sphere, torus, plane, custom SDF primitives
- **PRIM-03**: Skeletal animation for loaded meshes

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| **Haskell bindings** | Removed from the manifesto. Python covers the creative layer; Eclusa consumes via Python or the Rust crate directly. |
| **GPU shaders / wgpu / any GPU path** | Violates the "pure text, universal terminal" principle. Anything that breaks over SSH or in VT100 is wrong. |
| **LD_PRELOAD / terminal hijacking** | Same: portability-breaking. |
| **Inline-image protocols** (Sixel, Kitty graphics, iTerm2 images) | Non-portable; undermines the "ANSI text only" thesis. Possibly reconsidered as a strictly-opt-in ADV-xx feature post-v2. |
| **React-style VDOM / reconciliation** | Fine-grained signals are the core design thesis. VDOM is explicitly rejected. |
| **Built-in widget library** (buttons, lists, inputs) | happyterminals is a scene manager, not a form framework. ratatui/Textual do that. |
| **CSS-like styling DSL** | Props are Rust/Python values with signal integration; a separate style language would duplicate the prop layer. |
| **Forking voxcii or tui-vfx** | Fresh 3D implementation; tachyonfx is the effects foundation. Forking either creates maintenance debt and API mismatch. |
| **Imperative draw-call API** (`clear_screen`, `draw_pixel(x, y)`) | Violates Design Principle 1 (declarative, not imperative). |
| **Forking ratatui** | We are a consumer; patches go upstream. |
| **Old PyO3 patterns** (`&PyAny`, `IntoPy`, `pyo3-asyncio`) | All deprecated as of PyO3 0.28 / 2025-11; using them is a known pitfall. |

## Traceability

Authoritative coverage matrix lives in `.eclusa/ROADMAP.md` §"Coverage Matrix". Summary grouping here for quick reference:

| Requirement group | Phase(s) | Milestone |
|-------------------|----------|-----------|
| HYG-01 … HYG-09 | Phase 0 | M0 — Workspace Cleanup |
| REACT-01 … REACT-10 | Phase 1.0 | M1 |
| GRID-01 … GRID-05, BACK-01 … BACK-04 | Phase 1.1 | M1 |
| PIPE-01 … PIPE-07 | Phase 1.2 | M1 |
| REND-01 … REND-05, REND-09 (harness) | Phase 1.3 | M1 |
| SCENE-01 … SCENE-03, SCENE-05, SCENE-04 (scaffold), DSL-01 … DSL-03 | Phase 1.4 | M1 |
| DEMO-01 … DEMO-04, BACK-05 | Phase 1.5 | M1 — **spinning cube exit** |
| REND-06 … REND-10, REND-09 (full), REL-03 | Phases 2.1–2.4 | M2 |
| SCENE-04 (full), DSL-04 … DSL-08, REL-01 … REL-05 | Phases 3.1–3.5 | M3 — **v1 crates.io release** |
| PY-01 … PY-10 | Phases 4.1–4.4 | M4 — **FINAL (Python bindings)** |

**Coverage:** 69 v1 requirements, 69 mapped, 0 unmapped.

Two requirements are split across phases (scaffold first, full coverage later):

- **SCENE-04** — scaffolded in Phase 1.4, full TransitionManager in Phase 3.1.
- **REND-09** — allocation-budget harness in Phase 1.3, full per-frame bench coverage in Phase 2.3.

---
*Requirements defined: 2026-04-14*
*Last updated: 2026-04-14 after initialization*
