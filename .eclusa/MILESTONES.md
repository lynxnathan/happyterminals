# Milestones

## v1.0 Renderer Depth (Shipped: 2026-04-17)

**Phases completed:** 12 phases, 37 plans, 60 tasks

**Key accomplishments:**

- Found during:
- Consolidated 13-type crate-root re-export block for happyterminals-core, serializing the Wave-2 three-way file race on `lib.rs` identified in VERIFICATION.md BLOCK 1.
- Landed the injectable-dependencies surface of the reactive core — `Clock` / `SystemClock` / `ManualClock`, `Rng` / `ThreadRng` / `SeededRng`, and the `thiserror`-derived `CoreError` — all on rand 0.9's current API (`rand::rng()`, `.random::<f32>()`) with a `test-util` feature gate for the deterministic test doubles.
- Phase 1.0 exit gate: 5 integration tests + 2 proptests + 1 compile-time Send/Sync test + 1 criterion bench (both functions under 500 ns, 1µs gate automated via `jq`) + crate README recording doc-test deferral. Two Memo-exercising tests (`diamond`, `transitions_10k`) are `#[ignore]`'d pending a discovered `ImmediateEffect + Memo` deadlock in `reactive_graph 0.2.13` — documented below as a Phase 1.0 architectural blocker.
- Grapheme-cluster-correct Cell utilities and Grid newtype over ratatui::Buffer with put_str handling ASCII/CJK/emoji/ZWJ with silent clipping
- TerminalGuard RAII with panic-safe restore, InputEvent mapping from crossterm, and FrameSpec render config at 30fps default
- run() event loop with InputSignals wiring key/mouse/resize/focus events into reactive signals, plus 1-cell-change bytes test and static_grid demo binary
- Effect trait, Pipeline executor with sequential application, Fx alias resolving tachyonfx name clash, and Grid::buffer_mut() for adapter access
- TachyonAdapter bridges tachyonfx effects into our Effect system with 10 convenience constructors, smoke tests, and O(cells) criterion benchmarks
- Four foundational renderer modules -- projection with cell aspect correction, cube primitive, ASCII shading ramp, and orbit camera -- all with 25 unit tests passing and zero clippy warnings
- Half-space edge function rasterizer with reversed-Z z-buffer, zero-alloc draw() loop rendering a visible ASCII cube at 80x24, validated by insta snapshots, z-fighting regression, and criterion bench
- Scene IR tree with type-erased reactive props (Signal/Memo via AnyReactive), layer-sorted SceneGraph, validated Scene wrapper, and TransitionManager scaffold with Owner disposal
- R3F-shaped consuming-self builder DSL with signal-bindable props, prelude for <=25 line hello world, and meta crate re-exports
- run_scene() wiring Scene->Renderer->Pipeline->Grid->ratatui, spinning cube example at 43 LOC, scene-level bytes test passing at <= 50 bytes
- Root README rewritten with spinning cube showcase, 18-line hello world matching DSL test, crate structure table, and dual license
- 9-terminal HUMAN-UAT.md checklist created for BACK-05 M1 exit gate (Windows Terminal, GNOME, Terminal.app, iTerm2, Kitty, Alacritty, tmux, screen, SSH)
- Panic-free `tobj 4.0`-backed OBJ loader delivering a concrete `Mesh` struct that unifies runtime geometry with the existing `Cube` primitive; 11-file quirk-case corpus + proptest byte-fuzz prove zero panics on any input; `Cube::mesh()` bridges const data into the new heap type without touching `Renderer::draw` (refactor deferred to Plan 02).
- Refactored `Renderer::draw` to accept `&Mesh` instead of hardcoded `Cube::VERTICES`/`INDICES`/`FACE_NORMALS` — unifying cube rendering and loaded-mesh rendering under a single hot path that preserves REND-09 zero-per-frame-allocation discipline, proven at 12-triangle (cube) and ~5000-triangle (Stanford bunny) scale via criterion bench + capacity-stability proxy tests.
- Ships the Phase 2.1 exit artifact — a 96-LOC `model-viewer` example that loads bunny/cow/teapot at startup, cycles them via Left/Right arrows with per-model auto-fit orbit camera, and renders each through the unified `Renderer::draw(&Mesh)` hot path from Wave 2. Extends `happyterminals::prelude` with `Mesh`, `LoadStats`, `MeshError`, `load_obj`, `Cube` so end-users get a single canonical import. Chooses Path A (direct `run()` usage) over DSL extension, deferring the `.mesh()` builder to post-2.1 work.
- Hand-rolled env-cascade color-mode detector + xterm 256-palette quantizer + flush-time buffer downsample pass — 60 lib tests + 11 integration tests green, zero new deps.
- Wired Plan 01's `ColorMode` pipeline into the runtime — `FrameSpec.color_mode`, cached mode on `TerminalGuard`, flush-time downsample in both `run()` and `run_scene()`, and `ColorMode` surfaced through the happyterminals prelude. Zero example churn.
- Phase 2.2 exit artifacts shipped — 8-fixture `insta` snapshot matrix (cube × bunny × 4 modes), 119-LOC `color-test` example with `--force-color` CLI, and README §"Terminal Color Support" with tmux `Tc` guidance. Zero new deps. All VALIDATION must-have gates satisfied.
- Godot/Unreal-inspired input action system with reactive signals, context stack dispatch, drag state machine, and modifier pipeline -- 62 tests passing
- Camera trait with 3 implementations (Orbit/FreeLook/FPS), elevation clamping, and polymorphic CameraConfig dispatch
- run_with_input() wiring InputMap into backend event loop, TerminalGuard focus events, prelude re-exports, and model-viewer upgraded to mouse-drag orbit + scroll zoom + WASD pan
- Pool-based particle system with Copy struct, z-buffered point projection, color-over-time lerp, and criterion bench enforcing zero per-frame allocation across all 3 renderer paths (cube, OBJ mesh, particles)
- Snow-over-bunny particle demo (114 LOC) with InputMap controls and prelude re-exports for particle types
- STL mesh loading via stl_io 0.11 with degenerate-normal fallback, proptest fuzz, and CI MSRV aligned to 1.88
- Skip-frame resize_pending flag added to run(), run_with_input(), and run_scene() preventing stale z-buffer renders during rapid terminal resize

---
