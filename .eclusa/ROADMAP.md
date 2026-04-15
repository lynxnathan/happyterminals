# Roadmap: happyterminals

**Created:** 2026-04-14
**Granularity:** Standard (5–8 phases per milestone, 3–5 plans per phase)
**Parallelization:** Enabled — independent phases/plans run in parallel
**Core Value:** Terminal art should feel like magic, not plumbing. A tiny declarative scene description should produce cinematic, reactive, cross-terminal output.
**Ordering philosophy:** Bottom-up with vertical-slice pulls (ARCH §11.2, SUMMARY §"Milestone Sketch"). Confirmed default per user deferral.

---

## Executive Summary

happyterminals ships as a **six-crate Rust workspace** (plus a seventh `-py` cdylib at the end) where a SolidJS-style reactive core feeds a flat `SceneIr` that is rendered into a `Grid` (newtyped over `ratatui::Buffer`), transformed by a `Pipeline` of `dyn Effect` trait objects (tachyonfx adapted as one stage), and flushed through ratatui's built-in ANSI diff. Every surface — Rust builder DSL, JSON recipes, Python — compiles to the same IR. The DSL takes cues from **react-three-fiber**: tree of typed nodes with props that can be Signals, but with fine-grained reactivity instead of a VDOM.

The roadmap delivers that system in four milestones:

- **Milestone 0** — Workspace cleanup (Phase 0). Blocker for everything else.
- **Milestone 1 — Spinning Cube Demo (THE FOCUS OF THIS ROADMAP).** Six phases (1.0–1.5) in bottom-up + vertical-slice order: reactive core → Grid+ratatui → Pipeline+tachyonfx → renderer → scene IR + DSL → spinning-cube exit. Covers REACT/GRID/PIPE/REND-01..05/SCENE/DSL-01..03/BACK-01..04/DEMO-* and BACK-05.
- **Milestone 2** — Renderer depth + cross-terminal polish. OBJ, particles, color pipeline, camera modes. REND-06..10, REL-03.
- **Milestone 3** — Compositor + JSON recipes + v1 crates.io release. Transitions, JSON loader + schema + validator, 5+ examples, publish, changelog. DSL-04..08, REL-01..05.
- **Milestone 4 — Python bindings (FINAL per PROJECT.md).** abi3 wheels, PyPI, type stubs, sync `run()`. PY-01..10.

M1 is planned in full detail (per-phase goals, scope, requirements, success criteria, dependencies, pitfall notes). M2/M3/M4 are sketched — enough arc to see the destination, but per-milestone re-planning will happen after each milestone ships.

---

## Milestones Overview

| Milestone | Phases | Exit Criterion | Requirement IDs |
|-----------|--------|----------------|-----------------|
| **M0** — Workspace Cleanup | Phase 0 | Clean build, dual-license, registries reserved, CI baseline green, no forbidden strings outside rationale sections. | HYG-01 … HYG-09 |
| **M1** — Spinning Cube Demo | Phases 1.0 → 1.5 (6 phases) | `examples/spinning-cube/` (<100 LOC) — signal-driven rotation → 3D projection → one tachyonfx effect → ratatui output. Verified on Win Terminal, GNOME, macOS Terminal.app, iTerm2, Kitty, Alacritty, tmux+screen, SSH. Ctrl-C leaves sane shell. 1-cell signal change emits ~10 bytes to TTY. | REACT-*, GRID-*, PIPE-*, REND-01..05, SCENE-*, DSL-01..03, BACK-01..04, BACK-05, DEMO-* |
| **M2** — Renderer Depth | ~4 phases (to be re-planned at M1 exit) | OBJ mesh viewer works on a 10+ file real-world corpus; particle example runs; `NO_COLOR` / `--force-color` honored; resize hardened on Windows Terminal. | REND-06..10, REL-03 |
| **M3** — Compositor + JSON + Release | ~5 phases (to be re-planned at M2 exit) | 5+ examples beyond the cube; JSON recipes round-trip with Rust builder; 7 crates published to crates.io under `MIT OR Apache-2.0`; `cargo semver-checks` green. | DSL-04..08, SCENE transitions, REL-01..05 |
| **M4** — Python Bindings (FINAL) | ~4 phases (to be re-planned at M3 exit) | `pip install happyterminals` on Linux/macOS/Windows; 10-line Python spinning-cube example; `mypy --strict` clean on the example; abi3 wheels cp310–cp313. | PY-01..10 |

---

## Dependency DAG

```
Phase 0 (cleanup)
    │
    ▼
Phase 1.0 (reactive core) ──────────────────────────────┐
    │                                                    │
    ▼                                                    ▼
Phase 1.1 (Grid + ratatui backend, static)       (consumed by 1.3, 1.4)
    │
    ├──► Phase 1.2 (Pipeline + tachyonfx)  ──► 1.5
    │         │
    │         ▼
    │   Phase 1.3 (Minimal renderer: cube)  ──► 1.5
    │         │
    │         ▼
    │   Phase 1.4 (SceneIr + Rust DSL)      ──► 1.5
    │
    ▼
Phase 1.5 (spinning-cube demo + cross-terminal matrix) → M1 EXIT
    │
    ▼
Milestone 2 → Milestone 3 → Milestone 4
```

**Parallelization notes (M1):**

- **Phases 1.2 and 1.3 can run in parallel** once 1.1 ships. Pipeline+tachyonfx work is orthogonal to renderer work (one mutates a `Grid`, the other writes into one). Two parallel plan streams.
- **Phase 1.4 (SceneIr + DSL)** can start in parallel with 1.3 using a *stub* renderer, but both must converge before 1.5.
- **Within Phase 1.1**, the Grid/Buffer layout-compat spike and the `-backend-ratatui` skeleton (TerminalGuard, tokio::select loop) can proceed in parallel plans.
- **Within Phase 1.5**, the cross-terminal verification matrix (BACK-05) and the README/GIF work (DEMO-02) are independent plans.

---

## Milestone 0 — Workspace Cleanup (prerequisite)

### Phase 0: Workspace Hygiene & Foundation

**Goal:** Unblock all feature work by resolving stub-crate dep rot, vendor-dir debris, README drift, and missing OSS release plumbing.

**Scope:**
- README/docs sweep: `tui-vfx` → `tachyonfx` across `README.md`, `project.md`, per-crate READMEs; preserve one "why not tui-vfx" rationale section only.
- Stub crates (`-core`, `-renderer`, `-compositor`): strip all speculative `[dependencies]` until a real call site exists. Remove `pyo3` from core.
- `[workspace.dependencies]` block with pinned versions: `ratatui-core 0.1`, `tachyonfx 0.25`, `glam 0.32.1`, `reactive_graph 0.2.13`, `any_spawner 0.3`, `pyo3 0.28.3`, `pyo3-async-runtimes 0.28`, `schemars 1.2`, `jsonschema 0.46`, `thiserror 2.0`, `compact_str 0.9`, `bon 3.9`, `tobj 4.0`, `insta 1.47`, `proptest 1.11`, `criterion 0.8`. All member crates use `dep.workspace = true`.
- Crate rename plan: `compositor` → `pipeline`. Add empty `-scene`, `-dsl`, `-backend-ratatui`, meta `happyterminals`, commented-out `-py`.
- Vendor relocation: `vendor/{pyo3,ratatui,tui-vfx}` → `vendor/_reference/{name}/` each with `STAMP.txt` (upstream commit + date). `.gitattributes` marks all `linguist-vendored=true`. Never reference via `path =` in any Cargo.toml.
- Dual-license files: `LICENSE-MIT` + `LICENSE-APACHE` at repo root. Every crate's Cargo.toml: `license = "MIT OR Apache-2.0"`. README + CONTRIBUTING get the standard Apache-2.0 §5 contribution clause.
- Registry reservation: minimal placeholder publishes for `happyterminals`, `happyterminals-core`, `happyterminals-py`, `happy-terminals` on **both** crates.io and PyPI.
- `rust-toolchain.toml` pinned to Rust 1.86 with `clippy` + `rustfmt` components.
- Baseline CI: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --workspace`, `cargo doc -D warnings`, `cargo tree -d` (duplicates), `cargo udeps` / `cargo-machete` (unused deps), doc-lint grep for forbidden strings (`tui-vfx`, `Haskell bindings`, `pyo3-asyncio`, `cgmath`, `tui-rs`) outside approved rationale.

**Requirements covered:** HYG-01, HYG-02, HYG-03, HYG-04, HYG-05, HYG-06, HYG-07, HYG-08, HYG-09

**Success criteria (observable):**
1. `cargo build --workspace` completes clean on Rust 1.86 with zero warnings.
2. `cargo tree -d` reports zero duplicate dependencies.
3. `grep -r "tui-vfx"` at repo root only matches `vendor/_reference/tui-vfx/` and one designated rationale section; the doc-lint CI step enforces this.
4. Both `LICENSE-MIT` and `LICENSE-APACHE` exist at the repo root, and every crate's `Cargo.toml` carries the SPDX string `"MIT OR Apache-2.0"`.
5. `happyterminals`, `happyterminals-core`, `happyterminals-py`, `happy-terminals` are reserved on crates.io AND on PyPI (placeholder versions published).
6. CI baseline (fmt, clippy `-D warnings`, tests, docs `-D warnings`, duplicate-dep scan, unused-dep scan, doc-lint) is green on `main`.

**Dependencies:** None (this is the entry phase).

**Pitfalls addressed:** §2 (README drift), §3 (stub-crate dep rot), §21 (name squatting), §22 (dual-license hygiene), §32 (vendored dir staleness), §33 (workspace dep drift).

**Parallelization:** Plans within Phase 0 are mostly independent — license files, workspace-deps refactor, vendor relocation, registry reservation, CI baseline can run as 4–5 parallel plans.

**Plans:** TBD (eclusa-planner will decompose into 3–5 plans)

---

## Milestone 1 — Spinning Cube Demo (full detail)

M1 delivers the PROJECT.md exit criterion: a signal-driven spinning ASCII cube with at least one tachyonfx effect, rendered via ratatui, as a <100 LOC example, cross-terminal verified. Six phases in bottom-up vertical-slice order.

### Phase 1.0: Reactive Core

**Goal:** Ship a SolidJS-style reactive graph (`Signal`, `Memo`, `Effect`, `Owner`, `batch`, `untracked`, `SignalSetter`) in `happyterminals-core` with clean owner disposal and no cross-thread ambiguity. This is the foundation every subsequent phase consumes.

**Scope:**
- Wrap `reactive_graph 0.2` behind our own public types (`Signal<T>`, `Memo<T>`, `Effect`, `Owner`, `Root`) so the public surface is API-stable and Python-friendly. Re-export nothing from `reactive_graph`.
- `create_root(|| { ... })` + `Owner::run_in` + `on_cleanup` — when an owner is disposed, all its effects and memos are cleaned up with zero leaks.
- `batch(|| { ... })` coalesces multiple signal writes into a single downstream propagation.
- `signal.untracked()` — the only way the renderer reads scene state without registering a subscription.
- Two-phase propagation (mark dirty → recompute in topo order); **cycle detection panics with `"signal cycle A→B→A"`** (clear message, not stack overflow).
- **Single-threaded runtime + `SignalSetter` channel:** cross-thread writes go through a `Send` handle drained on the render thread each tick. This rule is tattooed onto every public reactive API docstring.
- `Clock` and `Rng` traits — injectable so snapshot/property tests are deterministic. Production defaults: `SystemClock` + `thread_rng`.
- Memo equality-skip decision: benchmark `Memo<T>: PartialEq` bound vs no bound; document the trade-off (Unresolved Question Q1 from SUMMARY.md).
- Tests: diamond-dependency test (signal fires once → each downstream runs exactly once); 10k-scene-transition RSS test (owner disposal works); cycle-detection panic test.

**Requirements covered:** REACT-01, REACT-02, REACT-03, REACT-04, REACT-05, REACT-06, REACT-07, REACT-08, REACT-09, REACT-10

**Success criteria (observable):**
1. A developer can create a signal, derive a memo from it, run an effect that reads the memo, and see the effect run exactly once per meaningful change — no missed updates, no thundering herd.
2. Disposing an owner (or completing a `create_root`) cleans up every effect and memo transitively; the 10k-transition RSS test holds under a documented ceiling.
3. Introducing a cycle (`a → b → a`) panics with a readable `"signal cycle"` message within one frame; never overflows the stack.
4. A background thread can update UI state by calling `.set()` on a `SignalSetter` handle; the render thread sees the update on the next frame tick with no data race.
5. Tests using the injected `Clock` and `Rng` produce byte-identical snapshots across runs (determinism verified).

**Dependencies:** Phase 0.

**Pitfalls addressed:** §4 (owner-tree leaks), §5 (over-fire / under-fire / cycles), §6 (Send/Sync ambiguity), §26 (snapshot determinism via Clock+Rng injection).

**Plans:** TBD

---

### Phase 1.1: Grid + Ratatui Backend (static)

**Goal:** Ship a grapheme-cluster-correct `Grid` (newtyped over `ratatui::Buffer`) and a panic-safe `-backend-ratatui` event loop that can render a static Grid and exit cleanly on Ctrl-C or panic. No animation yet — this is the "pixels on screen, safely" phase.

**Scope:**
- **1–2 day layout-compat spike (FIRST):** verify that a newtype over `ratatui::Buffer` can expose our `Cell` semantics (grapheme + width + fg/bg/modifiers) without fighting ratatui's memory layout. Gate commit to newtype on spike success. Fallback: own-struct Grid with `impl From<&Grid> for ratatui::Buffer`. (SUMMARY Q7.)
- `Cell`: holds one **grapheme cluster** (not `char`), display width (via `unicode-width` / `unicode-segmentation` or `runefix-core`), fg/bg colors (RGB + 256 + 16 + auto-fallback), modifiers (bold/italic/underline/reverse).
- `Grid::put_str(x, y, s, style)` — the only write API for text. Handles multi-byte, wide (east-asian, emoji), combining marks, ZWJ sequences correctly.
- `happyterminals-backend-ratatui` skeleton:
  - **`TerminalGuard` RAII + panic hook** that restores cursor visibility, raw mode, alternate-screen buffer, mouse capture, and SGR state. Ctrl-C and unhandled panics both leave a sane shell.
  - `run(scene, FrameSpec)` driving a `tokio::select!` loop between a frame ticker (30/60 fps) and `crossterm::EventStream`. All frame work synchronous on the render thread.
  - Input events (key, mouse, resize, focus) propagate into scene signals.
  - Resize events drained between frames (no resize during rasterization); Grid clears cleanly on resize.
- **Async runtime decision:** tokio vs smol (Unresolved Q2). Recommendation: tokio (ecosystem weight, PyO3 story), document.
- `happyterminals` meta crate: curated public re-exports (`prelude`).
- Tests: static-grid render smoke test; TerminalGuard panic test (`panic!()` mid-render leaves cursor visible, raw mode off); resize test on Windows Terminal.
- **1-cell-change bytes test (foundation):** write a smoke test that mutates exactly one cell via `Grid::put_str` and asserts `< ~10 bytes` written to the captured TTY via `ratatui::Terminal::flush()`. This is the M1 exit gate — establish it now.

**Requirements covered:** GRID-01, GRID-02, GRID-03, GRID-04, GRID-05, BACK-01, BACK-02, BACK-03, BACK-04

**Success criteria (observable):**
1. A demo binary renders a static grapheme-correct string (including emoji and CJK) to a terminal, exits on Ctrl-C, and leaves the shell in a clean state.
2. Triggering a `panic!()` mid-frame still restores the terminal (cursor visible, raw mode off, alt screen exited, mouse capture off).
3. Resizing the terminal during render produces no garbled output and no crash; Grid dimensions update on the next frame.
4. A unit test mutating one `Grid` cell produces ≤ ~10 bytes of ANSI output on flush (order-of-magnitude sane — no full-buffer repaint).
5. Key/mouse/resize events are observable as signal updates in a test scene.

**Dependencies:** Phase 1.0 (the backend event loop writes into signals).

**Pitfalls addressed:** §1 + §31 (panic-safe TerminalGuard), §7 (grapheme/width math), §10 (resize race), §12 (true-diff rendering — M1 exit gate established here), §17 (WASM divergence — explicitly not addressed; native only in M1).

**Parallelization:** Grid-newtype spike, TerminalGuard implementation, and the tokio::select loop skeleton can proceed as three parallel plans once Phase 1.0 is done.

**Plans:** 3/3 plans complete

Plans:
- [x] 01.1-01-PLAN.md -- Cell + Grid types (grapheme-cluster-correct newtype over ratatui::Buffer)
- [x] 01.1-02-PLAN.md -- TerminalGuard RAII + event mapping + FrameSpec
- [x] 01.1-03-PLAN.md -- run() event loop + 1-cell bytes test + meta prelude + static demo
**UI hint**: yes

---

### Phase 1.2: Pipeline + tachyonfx Adapter

**Goal:** Build `happyterminals-pipeline` — our `Effect` trait, the `Pipeline` executor, and a `TachyonAdapter` that makes all 50+ tachyonfx effects usable inside our system without per-object loops or name collisions.

**Scope:**
- **Resolve the `Effect` name clash FIRST (before any Pipeline consumer code):** in our public surface, `tachyonfx::Effect` is aliased to `Fx` (or `VisualEffect`). Our `Effect` trait keeps the name. This decision is committed to code + CHANGELOG before PIPE-02 lands. (PITFALLS §16.)
- Our `Effect` trait: `fn apply(&mut self, grid: &mut Grid, dt: Duration) -> EffectState` where `EffectState` is `Running` or `Done`. Effects are stateful (timers, progress).
- `Pipeline = Vec<Box<dyn Effect>>` — **trait objects, not generic chains** — so JSON recipes and Python can construct pipelines at runtime.
- `TachyonAdapter<S: tachyonfx::Shader>` wraps a tachyonfx effect as one of our trait objects, with real `Duration` dt forwarded. `impl Effect for TachyonAdapter<S>`.
- **Whole-grid composition only.** No per-object effect loops (avoid O(objects × effects)). Criterion bench establishes the performance floor and catches regression.
- **Invariant: effects mutate scene state only through the reactive channel** — never mutate `Grid` from outside a Pipeline's `apply()`. Documented in every public docstring; lint-enforced where possible (`clippy::disallowed_methods`).
- Wire ≥ 10 tachyonfx effects end-to-end with smoke tests: vignette, dissolve, fade_in, fade_out, crt, color_ramp, typewriter, sweep, slide, and at least one glitch/shader effect.
- Errors on public surface are `Result`-based; `clippy::unwrap_used` and `clippy::expect_used` denied in `-pipeline`.

**Requirements covered:** PIPE-01, PIPE-02, PIPE-03, PIPE-04, PIPE-05, PIPE-06, PIPE-07, DSL-03 (partial — clippy lints on lib crates)

**Success criteria (observable):**
1. A static `Grid` with text has `fx::vignette(0.3)` applied via our Pipeline and renders identically to tachyonfx's own example (visual parity).
2. A `fade_in(Duration::from_secs(2))` effect runs over exactly 2 seconds of frame time and then returns `EffectState::Done`.
3. Compiling code that uses both our `Effect` trait and `tachyonfx::Effect` produces no ambiguous-name errors — the alias `Fx` is in effect.
4. A criterion bench shows pipeline overhead is `O(cells)`, not `O(objects × effects)`; regression would fail CI.
5. Ten named tachyonfx effects each run without panic on a 200×60 Grid for 60 frames.

**Dependencies:** Phase 1.1 (`Grid` exists).

**Pitfalls addressed:** §16 (Effect name clash), §28 (errors not panics on public surface), §30 (O(n·m) composition).

**Parallelization:** Can run **in parallel with Phase 1.3** — both consume Grid from 1.1 and neither depends on the other's output until Phase 1.5.

**Plans:** 2 plans in 2 waves

Plans:
- [x] 01.2-01-PLAN.md — Effect trait + EffectState + Pipeline executor + Fx alias + Grid::buffer_mut()
- [x] 01.2-02-PLAN.md — TachyonAdapter + 10 tachyonfx effect wrappers + smoke tests + criterion benchmark

---

### Phase 1.3: Minimal 3D Renderer (Cube Primitive)

**Goal:** Ship `happyterminals-renderer` with a z-buffer rasterizer, perspective projection with correct cell-aspect handling, an ASCII shading ramp, a built-in `Cube` primitive, and a signal-driven orbit camera. No OBJ yet — that's M2.

**Scope:**
- Fresh (not forked, not voxcii-vendored) z-buffer rasterizer. Per-pixel depth test. Backface culling.
- Perspective projection with **cell aspect ratio as a public API parameter** (default 2:1 tall). A cube rendered with defaults looks cubic, not tower-shaped. (PITFALLS §14.)
- **Reversed-Z + scene-fit near/far planes** — no visible z-fighting on the spinning cube at default resolutions. (PITFALLS §13.)
- Configurable ASCII shading ramp (default 10 levels `` .:-=+*#%@``). Flat-shaded / per-face sufficient for M1.
- Built-in `Cube` primitive (M1 demo dependency).
- **Orbit camera** with signal-driven azimuth / elevation / distance. Camera owned by the Scene (not a global) — binding established here; SceneGraph ownership formalized in 1.4.
- **Per-frame allocation budget enforced via criterion bench** — reusable `Vec<u8>` buffers, cached SGR escape sequences, no heap churn in the hot path. (PITFALLS §11.)
- Tests: snapshot test of a static cube at a fixed camera pose; reversed-Z flicker test (no z-fighting); criterion bench for per-frame allocation.

**Requirements covered:** REND-01, REND-02, REND-03, REND-04, REND-05, REND-09 (allocation budget; the criterion harness and CI gate land here even though the full bench coverage matures in M2)

**Success criteria (observable):**
1. A cube rendered at defaults looks cubic (not a tower) in a terminal with default cell aspect ratio.
2. A rotating cube at 30 fps shows no visible z-fighting or depth flicker anywhere on the silhouette.
3. Changing the `orbit_camera.azimuth` signal causes the cube to visibly orbit; no other scene state is touched.
4. A criterion bench reports zero heap allocations inside `Renderer::draw()` after warmup (reusable buffers verified).
5. The ASCII shading ramp can be swapped at runtime and the silhouette's shading visibly changes.

**Dependencies:** Phase 1.0 (signals drive the camera); Phase 1.1 (renderer writes into `Grid`).

**Pitfalls addressed:** §11 (per-frame allocation churn), §13 (z-fighting), §14 (aspect ratio).

**Parallelization:** Can run **in parallel with Phase 1.2**.

**Plans:** 2 plans in 2 waves

Plans:
- [x] 01.3-01-PLAN.md — Crate setup + projection, cube, shading, camera modules
- [x] 01.3-02-PLAN.md — Rasterizer + Renderer::draw() + snapshot test + criterion bench
**UI hint**: yes

---

### Phase 1.4: Scene IR + Rust Builder DSL

**Goal:** Build `happyterminals-scene` (SceneIr, SceneGraph, layered z-order, Camera-on-Scene) and `happyterminals-dsl` (Rust builder producing SceneIr, react-three-fiber-shaped tree with signal-bindable props). This is the first external-facing API — the shape users see when they learn the library.

**Scope:**
- **`SceneIr`** is the one internal representation. Rust builder, JSON loader (M3), and Python front-end (M4) all produce it. Single source of truth.
- **`SceneGraph`** supports layered composition with explicit z-order. Camera owned by the scene, not a global.
- Scene node props can be `Signal<T>`, plain `T`, or `Memo<T>` — **fine-grained subscription at the prop level** (the react-three-fiber departure: R3F's VDOM becomes signal-prop reactivity).
- `TransitionManager` scaffold (full transitions ship in M3, but the type and owner-disposal semantics are defined here so scene swap is possible).
- **Scene construction is `Result<Scene, SceneError>`** — invalid scenes (missing refs, bad signal bindings) fail at build time, not render time.
- Rust builder DSL shaped after react-three-fiber — tree of typed nodes with chainable props, props can be signal references:
  ```rust
  scene()
      .layer(|l| l
          .cube()
              .rotation(&r)
              .position(vec3(0., 0., 0.)))
      .effect(fx::vignette(0.3))
      .build()?
  ```
- **"Hello world" ≤ 25 lines of Rust including imports** — open an editor, type it, spinning cube on screen. (PITFALLS §27.)
- `Result`-based errors on the entire public API surface. `clippy::unwrap_used` / `clippy::expect_used` **denied** in every library crate (not just `-dsl`).
- Tests: round-trip snapshot of a builder-produced SceneIr; build-time error on an invalid signal binding.

**Requirements covered:** SCENE-01, SCENE-02, SCENE-03, SCENE-04 (scaffold), SCENE-05, DSL-01, DSL-02, DSL-03

**Success criteria (observable):**
1. A user can write a ≤25-line Rust file that imports the prelude, constructs a scene with a cube whose rotation is bound to a signal, and calls `run(scene)`.
2. A scene that references an undeclared signal fails at `build()` time with a readable error — no panic, no render-time crash.
3. Mutating a signal bound to a node prop causes only that node's render state to invalidate (verified via a fine-grained subscription test).
4. The builder tree shape (`.layer().cube().rotation().position()`) compiles to exactly the same `SceneIr` as a manual IR construction (round-trip snapshot test).
5. `cargo clippy -D warnings` on all library crates fails if any lib code uses `.unwrap()` or `.expect()`.

**Dependencies:** Phase 1.0 (signals are scene-node prop types), Phase 1.3 (the renderer consumes `SceneIr`).

**Pitfalls addressed:** §27 (hello-world ≤25 lines), §28 (errors not panics).

**Parallelization:** Can start **in parallel with Phase 1.3** using a stub renderer; both converge before 1.5.

**Plans:** 2 plans in 2 waves

Plans:
- [x] 01.4-01-PLAN.md — Scene crate types (NodeId, SceneNode, PropValue, SceneIr, SceneGraph, CameraConfig, Scene, TransitionManager)
- [x] 01.4-02-PLAN.md — DSL builder (SceneBuilder, CubeBuilder, LayerBuilder) + prelude + meta crate update
**UI hint**: yes

---

### Phase 1.5: Spinning Cube Demo + Cross-Terminal Matrix (M1 EXIT)

**Goal:** Deliver the PROJECT.md M1 exit artifact: `examples/spinning-cube/` as a single Rust file under 100 LOC, verified across the full cross-terminal matrix, with README + GIF and the 1-cell-change bytes test as a hard gate.

**Scope:**
- `examples/spinning-cube/main.rs` — **< 100 LOC** — renders a signal-rotated cube with one tachyonfx effect (vignette), running at 30 fps, with no visible flicker and no memory growth over a 10-minute run (soak test).
- Root `README.md` showcases the cube with an **animated GIF or asciicast** plus the ≤25-line hello-world snippet inline.
- **Ctrl-C during the demo leaves the terminal in a clean state on every platform in the verification matrix** (regression coverage for §1 / §31).
- **1-cell-change output test** (HARD M1 GATE): mutating one signal produces approximately one cell of ANSI change. Order-of-magnitude bytes test asserts ≤ ~10 bytes for a 1-cell change on a typical 80×24 terminal. No full-buffer repaint.
- **Cross-terminal verification matrix (BACK-05):** spinning cube runs correctly on:
  - Windows Terminal
  - GNOME Terminal
  - macOS Terminal.app
  - iTerm2
  - Kitty
  - Alacritty
  - tmux + screen
  - SSH session (against Linux box from at least two of the above)
- 10-minute soak test: no memory growth, no frame drops > 5%.
- Documentation: the example is the canonical "read this first" surface; docstrings linked from `docs.rs`.

**Requirements covered:** DEMO-01, DEMO-02, DEMO-03, DEMO-04, BACK-05

**Success criteria (observable):**
1. `cargo run --example spinning-cube` in a fresh checkout renders a smoothly rotating, vignette-effected ASCII cube at 30 fps.
2. The example source file is under 100 LOC including imports.
3. Pressing Ctrl-C during the demo leaves a usable shell (cursor visible, prompt responsive, terminal state sane) on every platform in the verification matrix.
4. A unit test asserts that mutating one signal driving the cube's rotation produces ≤ ~10 bytes of ANSI output per frame step.
5. Running the example for 10 minutes shows no RSS growth (memory-stable) and no visible flicker on any of the listed terminals.
6. The root `README.md` displays the spinning cube (GIF/asciicast) and the ≤25-line hello-world above the fold.

**Dependencies:** Phases 1.2, 1.3, 1.4.

**Pitfalls addressed:** §1 / §31 (panic test across terminals), §12 (1-cell bytes test — hard gate), §14 (cube shape on every terminal), §27 (hello-world length), §30 (composition cost in practice).

**Parallelization:** Example implementation, README + GIF production, and the cross-terminal verification matrix are three independent plans once 1.2/1.3/1.4 have merged.

**Plans:** 3 plans

Plans:
- [ ] 01.5-01-PLAN.md — Integration wiring (run_scene, spinning cube example, bytes test)
- [ ] 01.5-02-PLAN.md — README update with spinning cube showcase
- [ ] 01.5-03-PLAN.md — Cross-terminal verification matrix (HUMAN-UAT)

**UI hint**: yes

---

## Milestone 2 — Renderer Depth + Cross-Terminal Polish (summary)

**Goal:** Harden the renderer for real-world meshes and non-truecolor terminals; add particles; document the per-frame allocation budget.

**Indicative phases (re-planned at M1 exit):**

- **Phase 2.1 — OBJ mesh loading.** `tobj 4.0.3`; triangulation of quads; winding normalization; flat-normal fallback when normals are missing. 10+ real-world `.obj` file corpus in `tests/fixtures/obj/`. `Result<Mesh, MeshError>` — **no panics on malformed input.** Covers REND-06. Pitfall §15.
- **Phase 2.2 — Color-mode pipeline.** RGB → 256 → 16 → monochrome fallback. `NO_COLOR` env var honored; `--force-color` override; tmux `Tc` truecolor guidance in docs. Covers REND-08. Pitfall §8.
- **Phase 2.3 — Particle system + camera modes.** Emitter + gravity + lifetime + color-over-time. At least one particle example runs. Free + FPS camera modes (orbit shipped in M1). Covers REND-07 (and the orbit part of REND-05 is already done). M1's criterion-bench harness (REND-09) matures into full per-frame allocation coverage here.
- **Phase 2.4 — Resize hardening + MSRV policy.** Resize-race hardening on Windows Terminal. STL loading via `stl_io 0.11` (REND-10). MSRV 1.86 pinned; CI tests MSRV + stable. Covers REND-10, REL-03.

**Requirements covered:** REND-06, REND-07, REND-08, REND-09 (matures), REND-10, REL-03.

**Pitfall notes:** §8 (color regression), §10 (resize race hardened), §15 (OBJ/STL brittleness), §29 (logging — file/stderr only, never in-frame), §30 (composition cost re-validated with real meshes).

**Re-planning trigger:** After M1 ships, re-plan M2 phases based on what cross-terminal verification exposed.

---

## Milestone 3 — Compositor + JSON Recipes + v1 Release (summary)

**Goal:** Complete the declarative surface (full transitions, JSON recipes, schema validator) and publish v1 to crates.io.

**Indicative phases (re-planned at M2 exit):**

- **Phase 3.1 — Full TransitionManager.** Scene A → Scene B via named effect (dissolve, slide, etc.). Outgoing owner disposed cleanly (integrates with Phase 1.0's owner tree). Covers SCENE-04 full.
- **Phase 3.2 — JSON recipe loader.** `-dsl::json` accepts a JSON scene, validates against a `schemars 1.2`-generated schema via `jsonschema 0.46`, binds named signal refs, produces a `SceneIr` identical to the Rust builder path (round-trip property test — DSL-06). Covers DSL-04, DSL-06, DSL-07 (versioned `$version` field).
- **Phase 3.3 — JSON sandbox + ANSI-injection.** Effect names resolved through a static registry (no eval, no shell-out); mesh paths sandboxed; ANSI-injection stripping on any user-provided string that lands in a Grid cell. Covers DSL-05, DSL-08.
- **Phase 3.4 — Examples library.** 5+ runnable examples beyond the cube: mesh viewer, particles, transitions, JSON recipe loader, text-reveal. No audio-reactive (that's parked). Covers REL-04.
- **Phase 3.5 — v1 crates.io publish.** Seven crates: `-core`, `-pipeline`, `-renderer`, `-scene`, `-dsl`, `-backend-ratatui`, `happyterminals` (meta). All have `description`, `license`, `repository`, `keywords`, `categories`, `readme`. Keep-a-Changelog `CHANGELOG.md`. `cargo semver-checks` on every PR. `docs.rs` builds every crate with every feature. Covers REL-01, REL-02, REL-05.

**Requirements covered:** SCENE-04 (full), DSL-04, DSL-05, DSL-06, DSL-07, DSL-08, REL-01, REL-02, REL-04, REL-05.

**Pitfall notes:** §5 (round-trip property tests), §9 (tmux DCS passthrough — only gate if non-SGR sequences ship), §23 (semver discipline before 1.0), §28 (errors not panics in JSON loader).

**Re-planning trigger:** After M2 ships, re-plan M3 based on real-world OBJ and color-fallback lessons.

---

## Milestone 4 — Python Bindings (FINAL per PROJECT.md)

**Goal:** Ship Python bindings as the last milestone. This is explicitly LAST because a Python surface is only worth shipping once the Rust layers are solid.

**Indicative phases (re-planned at M3 exit):**

- **Phase 4.1 — `-py` cdylib activation.** `happyterminals-py` as a PyO3 `cdylib`, **excluded from default workspace members**, built only via `maturin`. Python API mirrors the Rust builder shape (`happyterminals.signal`, `.scene`, `.fx`, `.run`). PyO3 0.28 `Bound<'py, T>` API throughout — no `&PyAny`, no `IntoPy`. `pyo3-async-runtimes 0.28` (never `pyo3-asyncio`) wired in even if asyncio doesn't ship in this milestone. Covers PY-01, PY-02, PY-03.
- **Phase 4.2 — Sync `run()` + GIL + threading.** Primary entry point is sync `run(scene, fps=30)` (ARCH §9.4 recommendation; Unresolved Q6). GIL released during the render loop via `Python::allow_threads`. Python-side signal writes go through the same `SignalSetter` channel used by cross-thread Rust writers. Default cross-boundary semantics: **copy**; zero-copy only via explicit `freeze()`/`lock()`, never implicit. Covers PY-04, PY-05, PY-06, PY-10.
- **Phase 4.3 — Wheels + PyPI + type stubs.** abi3 wheels built via `maturin 1.13` for CPython 3.10–3.13 on Linux x86_64/aarch64, macOS x86_64/aarch64 (universal2), and Windows x86_64. PyPI **Trusted Publishing** for release. Type stubs (`.pyi`) shipped. `mypy --strict` on the example code passes. Covers PY-07, PY-08.
- **Phase 4.4 — Python hello-world + v1 launch.** Python-side "hello spinning cube" is **≤10 lines** and installs via `pip install happyterminals` on every supported platform. Python README documents the single-threaded reactive model and the `SignalSetter` pattern for cross-thread updates. Covers PY-09, and completes PY-10.

**Requirements covered:** PY-01, PY-02, PY-03, PY-04, PY-05, PY-06, PY-07, PY-08, PY-09, PY-10.

**Pitfall notes:** §18 (`pyo3-async-runtimes`, never `pyo3-asyncio`), §19 (GIL contention in render loop), §20 (zero-copy hazards — default copy), §25 (CI matrix cost — abi3 tiered matrix), §6 (threading rules documented on the Python side).

**Re-planning trigger:** After M3 ships, user decides Q6 (sync vs asyncio first) definitively.

**After M4:** v1 is complete. Phase 5 "fun" items (audio-reactive, AI prompt→scene, GLSL→ASCII, live REPL, multi-terminal, WASM, visual editor) live in `999.x` backlog and are revisited only after v1.x user feedback.

---

## Coverage Matrix (v1 Requirement → Phase)

| Requirement | Phase | Milestone |
|-------------|-------|-----------|
| HYG-01 | Phase 0 | M0 |
| HYG-02 | Phase 0 | M0 |
| HYG-03 | Phase 0 | M0 |
| HYG-04 | Phase 0 | M0 |
| HYG-05 | Phase 0 | M0 |
| HYG-06 | Phase 0 | M0 |
| HYG-07 | Phase 0 | M0 |
| HYG-08 | Phase 0 | M0 |
| HYG-09 | Phase 0 | M0 |
| REACT-01 | Phase 1.0 | M1 |
| REACT-02 | Phase 1.0 | M1 |
| REACT-03 | Phase 1.0 | M1 |
| REACT-04 | Phase 1.0 | M1 |
| REACT-05 | Phase 1.0 | M1 |
| REACT-06 | Phase 1.0 | M1 |
| REACT-07 | Phase 1.0 | M1 |
| REACT-08 | Phase 1.0 | M1 |
| REACT-09 | Phase 1.0 | M1 |
| REACT-10 | Phase 1.0 | M1 |
| GRID-01 | Phase 1.1 | M1 |
| GRID-02 | Phase 1.1 | M1 |
| GRID-03 | Phase 1.1 | M1 |
| GRID-04 | Phase 1.1 | M1 |
| GRID-05 | Phase 1.1 | M1 |
| BACK-01 | Phase 1.1 | M1 |
| BACK-02 | Phase 1.1 | M1 |
| BACK-03 | Phase 1.1 | M1 |
| BACK-04 | Phase 1.1 | M1 |
| PIPE-01 | Phase 1.2 | M1 |
| PIPE-02 | Phase 1.2 | M1 |
| PIPE-03 | Phase 1.2 | M1 |
| PIPE-04 | Phase 1.2 | M1 |
| PIPE-05 | Phase 1.2 | M1 |
| PIPE-06 | Phase 1.2 | M1 |
| PIPE-07 | Phase 1.2 | M1 |
| REND-01 | Phase 1.3 | M1 |
| REND-02 | Phase 1.3 | M1 |
| REND-03 | Phase 1.3 | M1 |
| REND-04 | Phase 1.3 | M1 |
| REND-05 | Phase 1.3 | M1 |
| REND-09 | Phase 1.3 (harness) → Phase 2.3 (full coverage) | M1/M2 |
| SCENE-01 | Phase 1.4 | M1 |
| SCENE-02 | Phase 1.4 | M1 |
| SCENE-03 | Phase 1.4 | M1 |
| SCENE-04 | Phase 1.4 (scaffold) → Phase 3.1 (full) | M1/M3 |
| SCENE-05 | Phase 1.4 | M1 |
| DSL-01 | Phase 1.4 | M1 |
| DSL-02 | Phase 1.4 | M1 |
| DSL-03 | Phase 1.4 (and enforced in every lib crate) | M1 |
| DEMO-01 | Phase 1.5 | M1 |
| DEMO-02 | Phase 1.5 | M1 |
| DEMO-03 | Phase 1.5 | M1 |
| DEMO-04 | Phase 1.5 | M1 |
| BACK-05 | Phase 1.5 | M1 |
| REND-06 | Phase 2.1 | M2 |
| REND-07 | Phase 2.3 | M2 |
| REND-08 | Phase 2.2 | M2 |
| REND-10 | Phase 2.4 | M2 |
| REL-03 | Phase 2.4 | M2 |
| DSL-04 | Phase 3.2 | M3 |
| DSL-05 | Phase 3.3 | M3 |
| DSL-06 | Phase 3.2 | M3 |
| DSL-07 | Phase 3.2 | M3 |
| DSL-08 | Phase 3.3 | M3 |
| REL-01 | Phase 3.5 | M3 |
| REL-02 | Phase 3.5 | M3 |
| REL-04 | Phase 3.4 | M3 |
| REL-05 | Phase 3.5 | M3 |
| PY-01 | Phase 4.1 | M4 |
| PY-02 | Phase 4.1 | M4 |
| PY-03 | Phase 4.1 | M4 |
| PY-04 | Phase 4.2 | M4 |
| PY-05 | Phase 4.2 | M4 |
| PY-06 | Phase 4.2 | M4 |
| PY-07 | Phase 4.3 | M4 |
| PY-08 | Phase 4.3 | M4 |
| PY-09 | Phase 4.4 | M4 |
| PY-10 | Phase 4.2 + Phase 4.4 | M4 |

**Coverage stats:**
- v1 requirements total: 69
- Mapped to phases: 69 ✓
- Orphaned: 0 ✓
- Split across phases (scaffold → full): SCENE-04 (1.4 → 3.1), REND-09 (1.3 harness → 2.3 full). Both annotated above. No duplication of primary responsibility.

---

## Open Questions (inherited from SUMMARY.md, resolved or carried forward)

| # | Question | Surfaces in | Resolution |
|---|---|---|---|
| 1 | `Memo<T>: PartialEq` bound — equality-skip cost vs spurious re-runs | Phase 1.0 | Benchmark-driven decision inside Phase 1.0; documented in phase exit. |
| 2 | Async runtime: tokio vs smol | Phase 1.1 | Default: **tokio** (ecosystem weight, PyO3 story). Revisit only if render-loop contention measured. |
| 3 | `Effect` name clash — ours or theirs? | Phase 1.2 | **Resolved:** `tachyonfx::Effect` → `Fx` in our public surface; ours keeps the name. Committed before any Pipeline consumer code. |
| 4 | Wide-char display — grapheme+width fields now, defer wide-cell rendering? | Phase 1.1 | **Ship grapheme + width fields now** (GRID-01/03). Wide-cell rendering edge cases documented and snapshot-tested; full wide-cell polish deferrable if a specific terminal misbehaves. |
| 5 | JSON schema versioning — `$version` field + semver, migrations TBD | Phase 3.2 | **Ship `$version` field** in Phase 3.2; first breaking migration is the trigger to design migration format. |
| 6 | Python primary surface: sync `run()` vs asyncio from day one | Phase 4.2 planning | **Default: sync `run()` first** per ARCH §9.4 and PY-04. User re-confirms at M3 exit before M4 planning. |
| 7 | Grid-as-ratatui-Buffer newtype layout compat | Phase 1.1 (spike) | **Explicit 1–2 day spike inside Phase 1.1**, gating the newtype commit. Fallback path documented (own-struct + `From` impl). |
| 8 | Roadmapper ordering philosophy | This document | **Resolved:** bottom-up with vertical-slice pulls. |

---

## Notes for eclusa-planner

Context the plan-phase workflow needs when decomposing each phase:

### Per-phase expectations (Standard granularity, user preferences)

- **3–5 plans per phase**, each a coherent unit of work that can be implemented and verified in one plan cycle.
- **Parallelization enabled** — when decomposing a phase, call out which plans can run in parallel vs must sequence.
- **Research before each phase: YES.** Before decomposing Phase N, re-check SUMMARY.md / STACK.md / PITFALLS.md for drift; any crate version that moved in the 2026-04-14 snapshot needs re-verification.
- **Plan check + Verifier: YES** per plan, both gates active.
- **Commit planning docs to git: YES.**

### Pitfall preservation

Each M1 phase carries a specific pitfall-to-phase mapping (SUMMARY.md "Critical Pitfalls Per Phase" + the "Pitfalls addressed" annotations above). When decomposing a phase into plans, surface those pitfalls as **explicit plan must_haves** — e.g., Phase 1.1 must_have includes "TerminalGuard panic hook implemented in first commit, not end of phase."

### M1 is the deep plan; M2/M3/M4 are sketches

Only Phase 0 and Phases 1.0–1.5 are planned in full here. **Re-plan M2, M3, and M4 at each preceding milestone's exit** — lessons from M1 cross-terminal verification, M2 mesh-corpus work, and M3 JSON-sandbox design will shape the next milestone's scope.

### M1 exit gates (hard)

Beyond per-phase success criteria, M1 exits only when ALL of the following hold:

1. `examples/spinning-cube/main.rs` < 100 LOC, runs at 30 fps with no flicker (DEMO-01).
2. 10-minute soak test shows no RSS growth (DEMO-01).
3. Ctrl-C leaves a sane shell on every platform in the cross-terminal matrix (DEMO-03 + BACK-05).
4. 1-cell-change bytes test ≤ ~10 bytes per frame step (DEMO-04 + GRID-04).
5. `README.md` has GIF/asciicast + ≤25-line hello-world above the fold (DEMO-02 + DSL-02).
6. All 69 v1 requirements mapped to phases still apply; no orphans emerged during implementation.

### Parallel plan streams to watch

Highest-value parallelization opportunities inside M1:

- **Phase 0:** 4–5 parallel plans (license files, workspace deps, vendor move, registry reservation, CI baseline).
- **Phases 1.2 and 1.3:** entire phases run in parallel after 1.1.
- **Phase 1.4:** can begin in parallel with 1.3 using a stub renderer.
- **Phase 1.5:** example code, README+GIF, and the cross-terminal matrix are three independent plans.

### File locations (for downstream tools)

- `.eclusa/PROJECT.md` — core value, decisions
- `.eclusa/REQUIREMENTS.md` — REQ IDs + traceability table (will be updated with the phase mappings in this roadmap)
- `.eclusa/research/SUMMARY.md` — the authoritative research input for this roadmap
- `.eclusa/research/STACK.md`, `FEATURES.md`, `ARCHITECTURE.md`, `PITFALLS.md` — deeper references for per-phase research

---

## Phases (summary checklist)

- [ ] **Phase 0: Workspace Hygiene** — Resolve stub-crate dep rot, vendor debris, license + registry plumbing, CI baseline.
- [ ] **Phase 1.0: Reactive Core** — Signal/Memo/Effect/Owner/batch/untracked/SignalSetter with clean disposal and cycle detection.
- [x] **Phase 1.1: Grid + Ratatui Backend (static)** — Grapheme-correct Grid, panic-safe TerminalGuard, tokio::select loop, 1-cell-bytes test harness. (completed 2026-04-15)
- [ ] **Phase 1.2: Pipeline + tachyonfx** — Our Effect trait, Pipeline<Vec<Box<dyn Effect>>>, TachyonAdapter, `Fx` rename, 10+ effects smoke-tested.
- [ ] **Phase 1.3: Minimal Renderer (Cube)** — Z-buffer, cell-aspect projection, reversed-Z, shading ramp, Cube primitive, signal-driven orbit camera.
- [ ] **Phase 1.4: Scene IR + Rust DSL** — SceneIr, layered SceneGraph, react-three-fiber-shaped builder with signal-bindable props, ≤25-line hello-world.
- [ ] **Phase 1.5: Spinning Cube Demo** — <100 LOC example, README + GIF, cross-terminal matrix, soak test, 1-cell-bytes hard gate → **M1 EXIT**.
- [ ] **Milestone 2 (sketch)** — OBJ, color fallback, particles, resize hardening.
- [ ] **Milestone 3 (sketch)** — Transitions, JSON recipes, 5+ examples, crates.io v1 publish.
- [ ] **Milestone 4 (sketch)** — Python bindings. FINAL.

## Phase Details

The sections above (Milestone 0, Milestone 1 phases 1.0–1.5, Milestone 2 sketch, Milestone 3 sketch, Milestone 4 sketch) are the detailed phase records. Downstream tools (`plan-phase`, `progress`) should parse the `### Phase N.M:` headers under **Milestone 1 — Spinning Cube Demo (full detail)** for full M1 planning, and treat Milestones 2/3/4 as re-planned at each preceding milestone exit.

## Progress Table

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 0. Workspace Hygiene | 0/? | Not started | - |
| 1.0 Reactive Core | 0/? | Not started | - |
| 1.1 Grid + Ratatui Backend | 3/3 | Complete   | 2026-04-15 |
| 1.2 Pipeline + tachyonfx | 0/? | Not started | - |
| 1.3 Minimal Renderer | 0/? | Not started | - |
| 1.4 Scene IR + Rust DSL | 0/? | Not started | - |
| 1.5 Spinning Cube Demo | 0/? | Not started | - |
| M2 (sketch) | — | Re-plan at M1 exit | - |
| M3 (sketch) | — | Re-plan at M2 exit | - |
| M4 (sketch) | — | Re-plan at M3 exit | - |

*Plan counts are populated by `plan-phase` when each phase is decomposed.*

---

*Roadmap created: 2026-04-14*
*Author: eclusa-roadmapper*
*Granularity: Standard*
*Parallelization: Enabled*
