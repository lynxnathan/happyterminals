# happyterminals

**Terminal art should feel like magic, not plumbing.**

A declarative, reactive terminal scene manager with GPU-quality effects
rendered as pure text. Runs on every terminal ever made — from Windows
Terminal to GNOME to macOS Terminal.app to SSH into a Raspberry Pi.

---

## The Stack

```
┌─────────────────────────────────────────────────┐
│  Python / Haskell DSL (declarative scenes)       │
│  "this scene has a spinning cube + matrix rain"  │
├─────────────────────────────────────────────────┤
│  Reactive Runtime (async state → re-render)      │
│  SolidJS-like signals, not React's VDOM diffing  │
├─────────────────────────────────────────────────┤
│  tui-vfx Compositor (effects pipeline)           │
│  masks → filters → samplers → style shaders      │
├─────────────────────────────────────────────────┤
│  voxcii-style 3D Renderer (ASCII rasterizer)     │
│  z-buffer, lighting, OBJ/STL support             │
├─────────────────────────────────────────────────┤
│  Ratatui Backend (terminal I/O)                  │
│  crossterm / termion → actual pixels on screen   │
└─────────────────────────────────────────────────┘
```

### Why this layering?

- **Ratatui** handles the boring terminal stuff (cursor, colors, resize, input)
- **tui-vfx** handles the cinematic stuff (transitions, filters, shaders)
- **voxcii-core** handles the 3D stuff (mesh rendering, projection, lighting)
- **Reactive runtime** handles state management (signals, effects, memoization)
- **DSL** makes it pleasant to use (declare what you want, not how to draw it)

---

## Design Principles

### 1. Declarative, not imperative

```python
# NOT this:
def render(frame):
    clear_screen()
    draw_cube(x=40, y=12, rotation=t * 0.5)
    apply_filter("dissolve", progress=0.7)
    flush()

# THIS:
scene = Scene(
    objects=[Cube(position=center, rotation=signal("t") * 0.5)],
    effects=[Dissolve(progress=0.7)],
)
```

### 2. Reactive, not polling

Inspired by SolidJS, not React. No virtual DOM. No diffing.

- **Signals** hold state. When a signal changes, only the DOM nodes that
  read it re-render. Fine-grained, surgical updates.
- **Effects** run when dependencies change. Side effects are explicit.
- **Memos** cache derived computations. Expensive math runs once.

```python
rotation = signal(0.0)
zoom = signal(100)

# Only re-renders when rotation or zoom changes
cube = create_memo(lambda: project_cube(rotation(), zoom()))

# Side effect: plays a sound when rotation crosses threshold
create_effect(lambda: play_sound("click") if rotation() > 2 * pi else None)
```

### 3. Pure text output = universal terminal support

No GPU shaders in the terminal. No LD_PRELOAD hacks. No special terminal
required.

The compositor operates on a `Grid` (cells with characters + colors).
Effects transform grids. The output is ANSI escape sequences.

This means:
- Works over SSH
- Works in Windows Terminal, GNOME, macOS Terminal.app, iTerm2, Kitty
- Works in tmux/screen
- Works on a 30-year-old VT100 (minus the colors)

### 4. Composable effects pipeline

Every effect is a transform: Grid → Grid.

Chain them. Nest them. Mix and match:

```python
pipeline = Pipeline([
    Render3D(scene, camera),
    Mask(Dissolve(seed=42)),
    Filter(Vignette(strength=0.3)),
    Sampler(CRT(curvature=0.04)),
    StyleShader(ColorRamp(palette="synthwave")),
    ContentTransformer(Typewriter(speed=2)),
])
```

### 5. Python-first, Rust-speed

The hot path (rendering, compositing, 3D projection) lives in Rust.
The creative path (scene description, signals, effects) lives in Python.

PyO3 bindings. Zero-copy where possible.

Users never touch Rust. They write Python (or a DSL) and the framework
handles the rest.

### 6. JSON recipes for AI generation

tui-vfx already proved this works: effects as data.

Extend it to full scenes:

```json
{
  "scene": {
    "objects": [
      {"type": "cube", "position": [40, 12], "rotation_speed": 0.5},
      {"type": "particles", "count": 50, "gravity": -0.1}
    ],
    "background": {"type": "starfield", "depth": 3, "speed": 0.2},
    "effects": [
      {"type": "vignette", "strength": 0.3},
      {"type": "color_ramp", "palette": "dracula"}
    ]
  }
}
```

An LLM can generate these. A human can hand-edit them. Both are valid.

---

## Components

### Core (Rust crate: `happyterminals-core`)

- `Signal<T>` — reactive primitive (SolidJS-style)
- `Effect` — side effect runner
- `Memo<T>` — cached derived state
- `Grid` — cell buffer (char + fg + bg + attributes)
- `Pipeline` — ordered chain of Grid transforms

### Renderer (Rust crate: `happyterminals-renderer`)

- 3D projection (perspective, orthographic)
- Z-buffer rasterization
- ASCII shading ramp (10 levels: ` .:-=+*#%@`)
- Mesh loading (OBJ, STL)
- Camera controls (orbit, pan, zoom)
- Particle systems
- L-systems, fractals, generative geometry

### Compositor (wraps `tui-vfx`)

- All tui-vfx effects: masks, filters, samplers, shaders, transformers
- Scene-level compositing (layer objects, z-order)
- Transition manager (scene A → scene B with effect)
- JSON recipe loading + validation

### Python Bindings (PyO3)

- `happyterminals.signal(value)` → reactive signal
- `happyterminals.scene(objects, effects)` → declarative scene
- `happyterminals.run(scene, fps=30)` → event loop
- Full access to all effects, renderers, compositor
- Async support via `asyncio` integration

### Declarative DSL (Python)

```python
from happyterminals import signal, scene, run, effects as fx

rotation = signal(0.0)

@scene
def my_scene():
    return Cube(
        position=(40, 12),
        rotation=rotation,
        shading="ascii",
    )

@my_scene.effect
def vignette():
    return fx.Vignette(strength=0.3)

create_effect(lambda: rotation.set(rotation() + 0.02))

run(my_scene, fps=30)
```

### Haskell Bindings (FFI to Rust core)

```haskell
-- Type-safe scene description
data Scene = Scene
  { objects :: [Object]
  , effects :: [Effect]
  , camera  :: Camera
  }

-- Lens-based signal updates
rotation .= rotation + 0.02

-- Run the event loop
run scene 30  -- 30 fps
```

Because Eclusa needs this. Because Haskell types ARE the model.

---

## Architecture Decisions

### Why not just use tui-vfx directly?

tui-vfx is a compositing engine. It transforms grids. It doesn't:
- Manage state (signals/reactivity)
- Render 3D objects
- Run an event loop
- Provide Python bindings

happyterminals is the layer that makes tui-vfx *usable* without writing Rust.

### Why SolidJS-style reactivity instead of React-style?

React's model (VDOM diffing) is fundamentally wrong for terminals:
- Terminal cells are expensive to diff (no DOM)
- VDOM reconciliation is overhead for something that's already a grid
- Fine-grained reactivity means only changed cells get re-rendered

SolidJS signals: change a value → only the effect that reads it runs.
No diffing. No reconciliation. Just: "this cell changed, repaint it."

### Why Python + Rust instead of pure Rust?

- The creative layer (scene description, signal wiring, effect composition)
  should be fast to iterate on. Python wins.
- The rendering layer (3D projection, grid compositing, ANSI output)
  must be fast. Rust wins.
- PyO3 makes the boundary nearly free.

### Why Haskell bindings?

- Eclusa integration (cascade migration, type-level scene validation)
- Pattern matching on scene graphs is natural in Haskell
- The WW3 sim already uses Haskell for type-safe modeling
- Because we can.

---

## Roadmap

### Phase 0: Research (now)

- [ ] Clone tui-vfx, run recipe browser, catalog all 400+ effects
- [ ] Port voxcii's rendering core to a library (not a binary)
- [ ] Study ratatui's buffer model and widget trait
- [ ] Prototype the Signal/Effect/Memo reactivity system in Python

### Phase 1: Core (Rust)

- [ ] Reactive primitives (Signal, Effect, Memo)
- [ ] Grid buffer with cell attributes
- [ ] Pipeline executor (chain Grid transforms)
- [ ] Ratatui backend adapter (Grid → ratatui::Buffer)
- [ ] tui-vfx adapter (Grid → tui-vfx Grid trait)

### Phase 2: 3D Renderer

- [ ] Extract voxcii's projection/rasterization as a library
- [ ] Add mesh loading (OBJ, STL)
- [ ] ASCII shading ramp with configurable characters
- [ ] Camera system (orbit, FPS, free)
- [ ] Particle system
- [ ] L-system / generative geometry

### Phase 3: Python Bindings

- [ ] PyO3 wrapper for core types
- [ ] Declarative scene API
- [ ] Async event loop integration
- [ ] Signal/effect/memo Python API
- [ ] JSON recipe loader

### Phase 4: Haskell Bindings

- [ ] FFI bindings to Rust core
- [ ] Type-safe scene description
- [ ] Eclusa integration

### Phase 5: The Fun Part

- [ ] Audio-reactive scenes (chroma-style FFT → scene parameters)
- [ ] AI scene generation (prompt → JSON recipe → rendered scene)
- [ ] Shader-to-ASCII converter (GLSL → tui-vfx style shader)
- [ ] Live coding REPL (change scene, see it update instantly)
- [ ] Multi-monitor / multi-terminal scenes (one scene across N terminals)

---

## Related Projects

| Project | What it does | What we take from it |
|---------|-------------|---------------------|
| [tui-vfx](https://github.com/5ocworkshop/tui-vfx) | Cell-based compositor | Effects pipeline, Grid trait, JSON recipes |
| [voxcii](https://github.com/ashish0kumar/voxcii) | ASCII 3D viewer | Z-buffer, projection, OBJ/STL loading |
| [ratatui](https://github.com/ratatui-org/ratatui) | TUI framework | Terminal I/O, buffer model, widget system |
| [chroma](https://github.com/yuri-xyz/chroma) | GPU audio visualizer | Audio-reactive concept (but we do it in text) |
| [mixed-signals](https://github.com/5ocworkshop/mixed-signals) | Easing/motion primitives | Signal generators, easing functions |
| [SolidJS](https://www.solidjs.com/) | Reactive UI framework | Signal/Effect/Memo pattern |
| [retrovoxel](https://github.com/leonmavr/retrovoxel) | ASCII raycaster | Pure C rendering approach, zero dependencies |
| [claudelab](https://pypi.org/project/claudelab/) | Isometric TUI companion | Proof that terminal 3D + Python works |

---

## Name

**happyterminals** — because terminals should make you happy.

Not `sad-terminals`. Not `terminal-hell`. Not `ncurses-ptsd`.

Happy. Terminals.
