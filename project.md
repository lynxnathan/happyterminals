# happyterminals

**Terminal art should feel like magic, not plumbing.**

A declarative, reactive terminal scene manager with GPU-quality effects
rendered as pure text. Runs on every terminal ever made — from Windows
Terminal to GNOME Terminal to macOS Terminal.app to SSH into a Raspberry Pi.

---

## The Stack

```
┌─────────────────────────────────────────────────┐
│  Reactive Runtime (signals → re-render)          │
│  SolidJS-style fine-grained, not VDOM diffing    │
├─────────────────────────────────────────────────┤
│  Pipeline + tachyonfx (effects composition)      │
│  50+ effects, DSL, WASM editor, composable       │
├─────────────────────────────────────────────────┤
│  Fresh 3D Renderer (ASCII rasterizer)            │
│  z-buffer, lighting, OBJ/STL support             │
├─────────────────────────────────────────────────┤
│  Ratatui Backend (terminal I/O via crossterm)    │
│  cursor, colors, resize, input — the boring bits │
└─────────────────────────────────────────────────┘
```

### Why this layering?

- **Ratatui** handles the boring terminal stuff (cursor, colors, resize, input)
- **tachyonfx** handles the cinematic stuff (50+ effects, DSL, compositing)
- **Our renderer** handles the 3D stuff (mesh rendering, projection, lighting)
- **Reactive runtime** handles state management (signals, effects, memoization)
- **DSL** makes it pleasant to use (declare what you want, not how to draw it)

---

## Design Principles

### 1. Declarative, not imperative

```rust
// NOT this:
fn render(frame: &mut Frame) {
    clear_screen();
    draw_cube(40, 12, t * 0.5);
    apply_dissolve(0.7);
    flush();
}

// THIS:
let scene = scene()
    .layer(|l| l.cube().rotation(&rot).position(vec3(0., 0., 0.)))
    .effect(fx::dissolve(0.7))
    .build()?;
```

### 2. Reactive, not polling

Inspired by SolidJS, not React. No virtual DOM. No diffing.

- **Signals** hold state. When a signal changes, only the cells that read
  it re-render. Fine-grained, surgical updates.
- **Effects** run when dependencies change. Side effects are explicit.
- **Memos** cache derived computations. Expensive math runs once.

### 3. Pure text output = universal terminal support

No GPU shaders. No LD_PRELOAD hacks. No special terminal required.

The pipeline operates on a `Grid` (cells with graphemes + colors). Effects
transform grids. Output is ANSI escape sequences.

This means:
- Works over SSH
- Works in Windows Terminal, GNOME, macOS Terminal.app, iTerm2, Kitty
- Works in tmux and screen
- Degrades gracefully on limited terminals (no color → ASCII silhouette)

### 4. Composable effects pipeline

Every effect is a `Grid → Grid` transform. Chain them. Nest them:

```rust
let pipeline = Pipeline::new()
    .push(render_3d(scene, camera))
    .push(fx::vignette(0.3))
    .push(fx::color_ramp("synthwave"))
    .push(fx::typewriter(2));
```

### 5. Rust-first, Python-final

The hot path (rendering, compositing, 3D projection) lives in Rust.
The creative path (scene description, signal wiring, effect composition)
is ergonomic in Rust and optional in Python via PyO3 bindings
(final milestone). Users pick the language; the engine is the same.

### 6. JSON recipes for AI generation

Scene recipes are pure data. An LLM can generate them; a human can
hand-edit them; both are valid:

```json
{
  "scene": {
    "objects": [
      {"type": "cube", "rotation_speed": 0.5}
    ],
    "effects": [
      {"type": "vignette", "strength": 0.3},
      {"type": "color_ramp", "palette": "dracula"}
    ]
  }
}
```

---

## Components

- **`happyterminals-core`** — reactive primitives (Signal, Memo, Effect, Owner), Grid buffer.
- **`happyterminals-renderer`** — 3D projection, z-buffer, ASCII shading, OBJ/STL loading.
- **`happyterminals-pipeline`** — Effect trait, Pipeline executor, tachyonfx adapter.
- **`happyterminals-scene`** — Scene IR, scene graph, transitions.
- **`happyterminals-dsl`** — Rust builder API, JSON recipe loader.
- **`happyterminals-backend-ratatui`** — ratatui/crossterm event loop, panic-safe terminal guard.
- **`happyterminals`** — meta crate: curated public surface + prelude.
- **`happyterminals-py`** (final milestone) — Python bindings via PyO3.

---

## Roadmap

### Milestone 0 — Workspace cleanup (current)

Clean build, dual-license, vendor hygiene, CI baseline. Blocker for everything else.

### Milestone 1 — Spinning Cube Demo

Signal-driven ASCII cube with one tachyonfx effect, rendered via ratatui, verified on
Windows Terminal / GNOME / macOS Terminal / iTerm2 / Kitty / Alacritty / tmux / SSH.

### Milestone 2 — Renderer Depth

OBJ mesh loading, particle system, color-mode pipeline (truecolor → 256 → 16 → mono),
cross-terminal resize hardening.

### Milestone 3 — Scene graph + JSON + v1 crates.io release

TransitionManager, JSON recipe loader with validated schema, 5+ examples,
seven crates published to crates.io under `MIT OR Apache-2.0`.

### Milestone 4 — Python bindings (FINAL)

`pip install happyterminals` on Linux / macOS / Windows. abi3 wheels for CPython
3.10–3.13. Type stubs. Sync `run(scene, fps=30)` as the primary entry.

### Parked (post-v1)

Audio-reactive scenes, AI scene generation, live-coding REPL, shader-to-ASCII
transpiler, multi-terminal scenes, WASM runtime. Revisited after v1 ships and
user feedback lands.

---

## Related Projects

| Project | Role |
|---------|------|
| [ratatui](https://github.com/ratatui/ratatui) | Terminal I/O, buffer model, widget system |
| [tachyonfx](https://github.com/ratatui/tachyonfx) | Effects library we build on |
| [SolidJS](https://www.solidjs.com/) | Reactive signal model (not the implementation) |
| [reactive_graph](https://crates.io/crates/reactive_graph) | Leptos's reactive core — we wrap this |

---

## Name

**happyterminals** — because terminals should make you happy.

Not `sad-terminals`. Not `terminal-hell`. Not `ncurses-ptsd`.

Happy. Terminals.
