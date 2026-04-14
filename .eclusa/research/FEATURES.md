# Feature Research

**Domain:** Declarative reactive terminal scene manager (Rust core + Python bindings) for ASCII/ANSI cinematic output
**Researched:** 2026-04-14
**Confidence:** HIGH (comparator features verified against official docs and crates; user-expectation framing partly derived from training data + community discourse, marked MEDIUM where applicable)

---

## Feature Landscape

### Table Stakes (Users Expect These)

Without these, a Rust TUI animation framework feels half-built and users fall back to using `tachyonfx` + `crossterm` directly.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| `Signal<T>` / `Effect` / `Memo<T>` reactive primitives | Whole pitch is "SolidJS for terminals." Without these, framework is just a renderer. | M | SolidJS-style fine-grained graph; tracking via thread-local subscriber stack. Must support `get`/`set`/`update`, batched writes, untracked reads, cleanup on dispose. |
| `Grid` cell buffer (char + fg + bg + attributes) | Every TUI/effect lib has one (ratatui `Buffer`, tui-vfx `Grid`, tachyonfx `CellIterator`). Required for any compositor. | S | Match ratatui's cell layout closely for cheap conversion. Include modifier bits (bold/italic/underline/reverse). |
| `Pipeline` executor — ordered chain of `Grid → Grid` transforms | Composable effect chains are how every modern terminal-art lib works (tachyonfx, tui-vfx, libcaca canvas blits). | S | Must be nestable (a Pipeline is itself a transform). Must handle resize between frames. |
| Ratatui backend adapter (`Grid → ratatui::Buffer`) | Ratatui is *the* Rust TUI ecosystem. Not integrating = orphaning the framework. | S | One-way mapping into `Buffer`; bidirectional optional. Use `crossterm` as default backend. |
| tachyonfx integration (all 50+ effects callable from Pipeline) | Building on tachyonfx is a stated key decision. Users will expect every tachyonfx effect to "just work." | M | Wrap `tachyonfx::Effect` as a Pipeline stage. Preserve duration/timer semantics. |
| Terminal resize handling | Ratatui's FAQ specifically calls out resize as a thing apps must handle gracefully. Crash-on-resize = abandonment. | S | Trigger reactive invalidation of `viewport_width` / `viewport_height` signals so dependent memos recompute. |
| Event loop with FPS control | Every TUI guide (ratatui recipes, async-ratatui example) frames the app as `tick / render / input` at a target FPS. | M | Default 30 FPS, configurable. Must handle slow frames without drift. Must support pause/resume. |
| Input handling (keys, optional mouse) | crossterm provides events; users need at minimum "press q to quit," "arrow keys for camera." | S | Wire `crossterm::Event` into a `Signal<Option<Event>>` or callback. Keep mouse opt-in to preserve VT100 fallback. |
| Basic camera (orbit / pan / zoom) | voxcii, rendascii, every ASCII-3D viewer ships with these. Without them, the cube is unmoveable. | M | Look-at + perspective matrix. Orbit camera as the default for the spinning-cube demo. |
| Mesh loading: OBJ (and STL) | voxcii, rendascii, online STL→ASCII tools all do this. Loading a custom model is a "first thing users try." | M | OBJ first (simpler, ubiquitous). STL second. Triangulate non-tri faces. |
| Z-buffer rasterizer + ASCII shading ramp | Required for any non-wireframe 3D in a terminal. voxcii's headline feature. | L | 10-step ramp `` .:-=+*#%@`` configurable. Backface culling. Per-pixel depth test. |
| Working "spinning cube" example end-to-end | Every graphics/animation library has a demo proving the stack. Milestone 1 already requires this. | M | Single binary in `examples/`. <100 LOC. Signal-driven rotation + 1 effect + ratatui output. |
| Multiple runnable examples (≥5) | crates.io users skim `examples/` before reading docs. Tachyonfx, ratatui both ship many. | M | Cube, particles, mesh viewer, text reveal, JSON-recipe loader. |
| Decent error messages (no `unwrap` panics on user paths) | Rust ecosystem standard. PyO3 panics surface as Python exceptions — must be readable. | M | `thiserror` on the public surface; map to `PyErr` via PyO3 `From` impls. |
| `cargo doc` documentation on every public item | crates.io publish best-practices guides hammer this; users judge crates by their docs.rs page. | M | `#![deny(missing_docs)]` on public crates. Every public item gets `///` with at least one example. |
| README with quickstart that runs in <2 minutes | First-impression asset. Must show install + minimal scene + screenshot/GIF. | S | Cargo.toml `description`, `categories`, `keywords` populated for crates.io discoverability. |
| Cross-terminal compatibility verified (Win Terminal, GNOME, macOS, iTerm2, Kitty, tmux, SSH) | Project's stated compatibility constraint. Users will test on their terminal first. | M | CI matrix on Linux/macOS/Windows. Manual SSH test. Document VT100 (no color) fallback explicitly. |
| crates.io publish (workspace member crates) | Project goal. Without it, the framework doesn't exist for the Rust ecosystem. | S | Use Trusted Publishing (2026 crates.io feature) via GitHub Actions. Dual MIT OR Apache-2.0 in every `Cargo.toml`. |
| Scene graph with z-order layering | Implied by "Scene + transition manager" requirement. Users expect to layer background + objects + UI. | M | Render order = z-index, not declaration order. Stable sort. |
| Transition manager (scene A → scene B with effect) | Listed as Active requirement. Cinematic transitions are core to the "magic, not plumbing" pitch. | M | Wrap two scenes in a parametric transition effect (dissolve, slide, etc.). Reuse tachyonfx primitives. |
| JSON recipe loader + validator | Active requirement. LLM-generatable scenes are a stated differentiator (see below); validator/error reporting is table stakes for the loader. | M | `serde` + JSON Schema. Friendly error messages with line/column. Round-trip with the Rust DSL. |
| Declarative DSL surface (Rust API) | The framework's whole point. The imperative escape hatch is the anti-pattern. | M | Builder + macro-light surface. Mirrors mental model of the JSON recipes. |
| Python bindings (PyO3) — final milestone | Active requirement, explicitly the last milestone. Users in the creative-coding crowd require it. | XL | Wrap reactive primitives, `Scene`, `Pipeline`, `run()`. PyPI publish. Wheels for cp310–cp313, manylinux+macos+windows. |
| Getting-started tutorial (written, not just code) | Textual, ratatui both have them; crates without them lose the long tail of users. | M | "Build a spinning cube in 50 lines" walkthrough. Linked from README and docs.rs. |

### Differentiators (Competitive Advantage)

What makes happyterminals worth choosing over "just use tachyonfx" or "just use Textual."

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Declarative scene graph (objects + effects + signals as data) | tachyonfx is imperative-effects-on-a-Buffer; ratatui is widget-oriented; nobody else gives you a Solid-style scene description for terminals. | L | This is the headline. Scene is a value, not a sequence of draw calls. |
| SolidJS-style fine-grained reactivity (no VDOM, no diffing) | Textual uses class-attribute reactivity (coarse). Ratatui has none. Fine-grained = surgical re-renders, lower CPU at high FPS. | L | Stated key decision. Differentiates from React-shaped TUI frameworks. |
| 3D rendering integrated with the effect pipeline | voxcii doesn't compose with effects (it's a binary, not a lib); tachyonfx has no 3D. Combining them is novel. | XL | The cube is a Pipeline stage; `dissolve` runs *after* projection. This composability is the moat. |
| Cross-language: Rust core + Python creative surface | tachyonfx is Rust-only; Textual is Python-only; no project bridges hot-path Rust + creative Python for terminal art. | XL | PyO3 milestone. Lower the friction for Python creative-coders without giving up Rust speed. |
| LLM-friendly JSON scene recipes (extends tachyonfx DSL) | "Prompt → scene" is achievable when scene-as-data is the primary representation. Differentiates from imperative TUIs. | M | Schema published to docs site. Can paste into any LLM and get back valid recipes. |
| Universal terminal output (works on VT100, SSH, tmux) | Modern competitors (opentui, iTerm-image-protocol projects) often require special terminals. Sticking to pure ANSI + text is a positioning advantage. | M | Constraint discipline. Document tested terminals. CI smoke-tests on dumb terminals. |
| Pipeline-as-effect (Pipelines nest into other Pipelines) | tachyonfx's `parallel`/`sequence` is similar but weaker than full nesting. tui-vfx doesn't have it. | S | `impl Effect for Pipeline`. Encourages reusable scene fragments. |
| Built-in particle systems + L-systems / generative geometry | Listed in renderer scope. Creative-coding crowd (processing.py users) expects these as primitives, not as recipes-you-write-yourself. | L | Particle update on the reactive graph; emitter as a signal. L-system as a turtle on a Grid. |
| Ratatui interop both ways (use happyterminals scenes inside ratatui apps) | Most ratatui apps would love a "splash screen / loading animation / 3D background" widget. Embedding = adoption funnel. | M | Implement `Widget` for `Scene` so a user can drop a happyterminals scene into any ratatui layout. |
| Easing/motion primitives library (mixed-signals-style) | Animation without easing curves is amateur-hour. Listed in design lineage. | M | `Linear`, `EaseInOut`, `Spring`, `Bounce`, `Cubic`. Compose with signals. |
| Hot-reload of JSON recipes during development | Tachyonfx's DSL has live-reloading aspirations; we can deliver this for whole scenes. Big DX win for the demoscene crowd. | M | File watcher → re-parse → swap scene atomically. Useful for the parked "live REPL" Phase 5 item too. |
| Async / `asyncio` integration on the Python side | `asyncio` is the lingua franca of modern Python apps. Textual has it; we'd need parity. Listed in Active requirements. | L | Run the event loop in a Rust thread; bridge via `pyo3-asyncio`. |
| Color palette presets (synthwave, dracula, monokai, gruvbox, solarized) | Cited in the manifesto example. Cheap to add, big perceived value, makes screenshots pop. | S | Static `Palette` enum + apply-as-effect. |

### Anti-Features (Deliberately NOT Building)

Documented reasons to keep these out so they don't get re-added.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| GPU shaders / compute shaders | "Faster effects" / "more impressive output" | Violates universal-terminal principle. Breaks SSH, tmux, VT100. Fails the core constraint. | Stay on CPU. Effect math is cheap on a 200×60 grid. |
| LD_PRELOAD / terminal hijacking tricks | Some demoscene productions abuse terminal control sequences for impressive effects | Non-portable, breaks in tmux/screen, security/permissions issues, OS-specific | Pure ANSI escapes only. If a terminal doesn't support a sequence, gracefully degrade. |
| iTerm/Kitty inline image protocols | "We could show real images!" | Splits the audience into "real terminals" and "broken terminals," contradicts universal compatibility | Convert to ASCII via the renderer, or skip. |
| React-style VDOM / reconciliation | Familiarity for React devs | Diffing a grid is overhead for no gain. Stated key decision against. | Fine-grained signals only. Document the rationale prominently. |
| Built-in widget library (buttons, inputs, dropdowns, data tables) | "Make it a TUI framework like Textual/ratatui" | Would compete with Ratatui instead of complementing it. Bloats scope. | Interop with Ratatui widgets (we render scenes; Ratatui handles UI chrome). |
| CSS-like styling DSL (Textual-style) | Textual users will ask | Different conceptual model. Adds parser/runtime weight. Scenes-as-data already covers styling via JSON. | Style as fields on scene objects + effect parameters. |
| Haskell bindings | Originally in the manifesto for Eclusa | Removed in Active scope. Eclusa consumes via Python or Rust. New project if ever wanted. | Don't ship. Document the decision. |
| Forking voxcii or tui-vfx | "Why reimplement?" | voxcii isn't a library; tui-vfx is too immature (8★, 5 weeks). Both create dependency hazards. | Re-implement 3D fresh (voxcii-inspired). Build on tachyonfx (mature) instead of tui-vfx. |
| Audio-reactive scenes (chroma-style FFT → scene parameters) | Cited as inspiration; demo-worthy | Phase 5 (`999.x`). Audio capture is platform-specific (CoreAudio/PulseAudio/WASAPI), expands scope dramatically. Not on critical path. | Park in 999.x. Revisit post-Python milestone. |
| AI scene generation (prompt → JSON recipe → rendered scene) | Hot trend; LLM-friendly recipes enable it | Phase 5. Belongs in a separate tool that emits our JSON, not in the framework. | The JSON-recipe loader is the surface that makes this *possible* externally. We don't ship the LLM glue. |
| GLSL → ASCII shader transpiler | Demoscene wishlist | Phase 5. Massive scope (parsing GLSL, simulating shader semantics on CPU, mapping to ASCII). | Park. Custom effects in Rust/Python are the supported extension path. |
| Live coding REPL | Phase 5. Powerful but DX-only feature | Substantial work (sandbox, hot-reload of Python/Rust, error recovery). Hot-reload of *JSON recipes* delivers most of the value sooner. | JSON hot-reload (differentiator above) covers 80% of the use case. |
| Multi-monitor / multi-terminal scenes | Phase 5. Cool demo | Requires multi-terminal coordination, network sync, terminal-discovery — entire subsystem. | Park. Single terminal first; if pursued, build as separate `happyterminals-multi` crate later. |
| Imperative draw-call API (`draw_cube(x, y)`) | Familiar to processing.py / pygame users | Contradicts the declarative pitch. Once it exists, users default to it and the reactive layer rots. | Document the imperative pattern in tutorials as the "wrong way" with the declarative version next to it. |
| Forking ratatui or shipping our own backend | "More control over rendering" | Fragments the ecosystem. Loses the input/resize/cross-platform work ratatui already did. | Use ratatui as a library. Contribute upstream when needed. |

---

## Feature Dependencies

```
Signal/Effect/Memo (reactive core)
    ├──required-by──> Pipeline (effects depend on signal-driven params)
    ├──required-by──> Scene graph (object positions/rotations are signals)
    ├──required-by──> Camera controls (orbit/zoom are signals)
    ├──required-by──> Transition manager (progress is a signal)
    ├──required-by──> Hot-reload of JSON recipes (file watcher → signal)
    └──required-by──> Async / asyncio integration (Python signals must dispatch on the loop)

Grid buffer
    ├──required-by──> Pipeline (Grid → Grid transforms)
    ├──required-by──> Ratatui adapter (Grid → Buffer)
    ├──required-by──> tachyonfx integration (effects mutate Cells)
    ├──required-by──> Z-buffer rasterizer (writes shaded chars into Grid)
    └──required-by──> Particle systems / L-systems (write to Grid)

Pipeline
    ├──required-by──> Scene graph (scenes compose object renders + effects)
    ├──required-by──> Transition manager (transitions are pipelines)
    └──required-by──> JSON recipe loader (recipe deserializes to a Pipeline)

Ratatui adapter
    └──required-by──> Event loop with FPS / Input handling
                            └──required-by──> Spinning-cube example (Milestone 1 exit)

3D renderer (Z-buffer + projection + shading + camera)
    ├──requires──> Mesh loading (OBJ/STL) for non-primitive scenes
    ├──required-by──> Spinning-cube example (Milestone 1 exit)
    └──required-by──> Particle systems (3D-aware particles)

JSON recipe loader
    ├──requires──> Declarative DSL (DSL types are what JSON deserializes into)
    ├──required-by──> Hot-reload of JSON recipes
    └──enables (out-of-tree)──> AI scene generation (Phase 5)

Python bindings (PyO3)
    ├──requires──> Reactive core stable (signals exposed to Python)
    ├──requires──> Scene graph stable (declarative API mirrored in Python)
    ├──requires──> Pipeline + JSON recipes stable (Python re-exports them)
    ├──requires──> Async event loop (asyncio integration)
    └──required-by──> PyPI publish, Python tutorials, creative-coder adoption

Easing/motion primitives ──enhances──> Signal (signals get cinematic over time)
Color palettes ──enhances──> tachyonfx integration (palette as an effect)
Ratatui interop both ways ──enhances──> Ratatui adapter (Scene as a Widget)

Imperative draw API ──conflicts──> Declarative DSL (and is the anti-feature; do not add)
GPU shaders ──conflicts──> Universal terminal output (and is the anti-feature)
React-style VDOM ──conflicts──> Fine-grained reactivity (anti-feature)
```

### Dependency Notes

- **Reactive core is the foundation:** Every other table-stakes item assumes signals exist. Build it first; nothing reactive can be backfilled without rewriting consumers.
- **Grid is the universal currency:** Any feature that produces or consumes pixels touches `Grid`. Define its shape carefully early — changing the cell layout later is a breaking-change cascade.
- **Pipeline gates the effects layer:** tachyonfx integration, transitions, JSON recipes all flow through Pipeline. If Pipeline can't nest cleanly, every downstream feature gets warty.
- **Ratatui adapter unblocks the demo:** Until the adapter exists, "running the framework" means dumping bytes to stdout. Milestone 1's spinning cube needs this.
- **Python bindings depend on EVERYTHING being stable.** This is why they're the final milestone — exposing an unstable Rust API to Python amplifies churn cost.
- **Hot-reload depends on JSON recipes + reactive core:** A file watcher is trivial; what's hard is swapping the live scene atomically without losing signal subscriptions. Don't promise this until the reactive graph supports clean re-attachment.

---

## MVP Definition

### Launch With (v1 — "framework exists, cube spins, Rust crate published")

Minimum viable framework — proves the stack and ships to crates.io.

- [ ] Reactive primitives: `Signal<T>`, `Effect`, `Memo<T>` with fine-grained re-execution
- [ ] `Grid` cell buffer matching ratatui cell semantics
- [ ] `Pipeline` executor with nesting support
- [ ] Ratatui backend adapter (Grid → Buffer → crossterm)
- [ ] Event loop with FPS control + crossterm input + resize handling
- [ ] tachyonfx integration: at least 10 effects working end-to-end (dissolve, slide, coalesce, vignette, color_ramp, sweep, hsl_shift, fade, glitch, sequence/parallel)
- [ ] 3D renderer: perspective projection + Z-buffer + ASCII shading ramp + orbit camera + OBJ loading
- [ ] Scene graph with z-ordering
- [ ] Transition manager (scene A → scene B with one effect)
- [ ] JSON recipe loader + validator (basic schema)
- [ ] Declarative DSL (Rust API)
- [ ] **Spinning cube example** end-to-end (Milestone 1 exit)
- [ ] At least 4 more runnable examples (mesh viewer, particles, JSON recipe, text-reveal)
- [ ] README with quickstart + GIF
- [ ] `cargo doc` coverage on all public items
- [ ] Cross-terminal verification: Win Terminal, GNOME, macOS, iTerm2, tmux, SSH
- [ ] Published to crates.io under MIT OR Apache-2.0

### Add After Validation (v1.x — "Python crowd shows up")

Triggered by: v1 ships and Rust users start filing issues / requests.

- [ ] Python bindings (PyO3) — final milestone
- [ ] PyPI publish with wheels for cp310–cp313 across linux/macos/windows
- [ ] `asyncio` integration
- [ ] Python-side declarative DSL mirroring the Rust API
- [ ] Particle systems
- [ ] L-systems / generative geometry primitives
- [ ] Easing/motion primitive library (mixed-signals-inspired)
- [ ] Color palette presets (synthwave, dracula, monokai, gruvbox, solarized)
- [ ] STL mesh loading (after OBJ proves sufficient)
- [ ] Hot-reload of JSON recipes
- [ ] `Scene as Widget` — embed happyterminals scenes inside ratatui apps
- [ ] Getting-started tutorial (long-form)

### Future Consideration (v2+ / `999.x` backlog — Phase 5 "fun" items)

All deferred deliberately. Revisit after Python milestone ships.

- [ ] Audio-reactive scenes (chroma-style FFT → scene parameters)
- [ ] AI prompt → scene generation (probably as a separate companion tool, not in-tree)
- [ ] GLSL → ASCII shader transpiler
- [ ] Live coding REPL (beyond JSON hot-reload)
- [ ] Multi-monitor / multi-terminal scenes
- [ ] WASM build of the runtime (browser-based scene preview, mirroring tachyonfx's WASM editor)
- [ ] Visual scene editor (TUI Studio-style, but scene-graph aware)

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Reactive primitives (Signal/Effect/Memo) | HIGH | MEDIUM | P1 |
| Grid buffer | HIGH | LOW | P1 |
| Pipeline executor | HIGH | LOW | P1 |
| Ratatui adapter | HIGH | LOW | P1 |
| Event loop + FPS + input + resize | HIGH | MEDIUM | P1 |
| tachyonfx integration | HIGH | MEDIUM | P1 |
| 3D renderer (z-buffer + projection + shading) | HIGH | HIGH | P1 |
| Mesh loading (OBJ) | MEDIUM | MEDIUM | P1 |
| Camera controls (orbit) | MEDIUM | MEDIUM | P1 |
| Scene graph + z-order | HIGH | MEDIUM | P1 |
| Transition manager | MEDIUM | MEDIUM | P1 |
| JSON recipe loader | HIGH | MEDIUM | P1 |
| Declarative DSL (Rust) | HIGH | MEDIUM | P1 |
| Spinning-cube example | HIGH | MEDIUM | P1 |
| README + docs + crates.io publish | HIGH | LOW | P1 |
| Cross-terminal verification | HIGH | MEDIUM | P1 |
| Python bindings (PyO3) | HIGH | HIGH | P2 |
| asyncio integration | HIGH | MEDIUM | P2 |
| Particle systems | MEDIUM | HIGH | P2 |
| Easing primitives | MEDIUM | LOW | P2 |
| Color palettes | MEDIUM | LOW | P2 |
| Scene-as-Widget (ratatui interop) | MEDIUM | LOW | P2 |
| Hot-reload JSON recipes | MEDIUM | MEDIUM | P2 |
| STL mesh loading | LOW | LOW | P2 |
| L-systems / generative geometry | LOW | HIGH | P3 |
| Audio-reactive scenes | LOW (now) | HIGH | P3 (999.x) |
| AI prompt → scene | LOW (now) | HIGH | P3 (999.x) |
| GLSL → ASCII transpiler | LOW (now) | HIGH | P3 (999.x) |
| Live coding REPL | LOW (now) | HIGH | P3 (999.x) |
| Multi-terminal scenes | LOW (now) | HIGH | P3 (999.x) |
| WASM build | MEDIUM | MEDIUM | P3 |
| Visual scene editor | LOW | HIGH | P3 |

**Priority key:**
- P1: Must have for v1 launch (Rust crate publish + spinning cube)
- P2: v1.x — add after v1 lands and Python milestone is in flight
- P3: v2+ / `999.x` — defer until validated demand

**Complexity legend (S/M/L/XL → matrix LOW/MEDIUM/HIGH):**
- S/LOW: <1 week of focused work; well-trodden patterns
- M/MEDIUM: 1–3 weeks; some design decisions
- L/HIGH: 3–6 weeks; subsystem with novel design
- XL: 6+ weeks; multi-subsystem with cross-cutting concerns (Python bindings, integrated 3D+effects)

---

## Competitor Feature Analysis

| Feature | tachyonfx | Ratatui | Textual | voxcii | tui-vfx | happyterminals |
|---------|-----------|---------|---------|--------|---------|----------------|
| Reactive primitives | None (timeline-based) | None | Class-attribute reactive (coarse) | None | None | **Fine-grained signals (SolidJS-style)** |
| Effects library | 50+ effects with DSL | None built-in | CSS animations | None | JSON recipes | **Reuse all of tachyonfx + scene-level composition** |
| 3D rendering | None | None | None | OBJ/STL z-buffer | None | **Integrated 3D as Pipeline stage** |
| Scene graph | None | None | Widget tree (UI-oriented) | Single mesh | Cell-based composition | **Object + effect declarative scene** |
| Declarative API | DSL for effects only | Imperative | Class-based + CSS | CLI flags | JSON recipes | **Full scene-as-data (DSL + JSON)** |
| Cross-language | Rust only | Rust only | Python only | C++ binary | Rust only | **Rust core + Python bindings** |
| LLM-friendly recipes | DSL is text but not scene-level | No | No | No | Yes (JSON) | **Yes (JSON, scene-level, schema-published)** |
| Universal terminal output | Yes | Yes | Yes (mostly) | Yes | Yes | Yes (deliberate constraint) |
| WASM/browser preview | Yes | Partial | No (web mode is server-side) | No | No | Future (P3) |
| Hot-reload | Aspirational | No | App reload via dev tools | No | No | **JSON recipe hot-reload (P2)** |
| Mesh loading | N/A | N/A | N/A | OBJ + STL | N/A | OBJ (P1), STL (P2) |
| Particles / generative | None | None | None | None | None | **P2 (particles), P3 (L-systems)** |
| Maturity | 1,182★ active | de-facto standard | 25k★ standard | Niche viewer | 8★, 5wk | New project |

**Positioning takeaway:** No competitor combines (declarative scene graph + fine-grained reactivity + integrated 3D + cross-language). Each does one or two; happyterminals does all four. The risk is scope — which is exactly why the anti-features list is long and Phase 5 items are parked.

---

## Sources

- [tachyonfx (GitHub)](https://github.com/ratatui/tachyonfx) — 50+ effects, DSL, 2026 DSL completion engine
- [tachyonfx DSL docs](https://docs.rs/tachyonfx/latest/tachyonfx/dsl/index.html) — runtime config, live reload, serialization
- [ratatui ecosystem: tachyonfx](https://ratatui.rs/ecosystem/tachyonfx/) — official integration
- [voxcii (GitHub)](https://github.com/ashish0kumar/voxcii) — OBJ/STL, z-buffer, ncurses, C++17
- [LinuxLinks: voxcii overview](https://www.linuxlinks.com/voxcii-terminal-based-ascii-3d-model-viewer/) — feature confirmation
- [rendascii (GitHub)](https://github.com/Foxbud/rendascii) — comparable Python ASCII 3D engine, feature parity reference
- [Ratatui FAQ](https://ratatui.rs/faq/) — resize handling, event-loop patterns
- [Ratatui event handling](https://ratatui.rs/concepts/event-handling/) — backend (crossterm) ownership of input
- [async-ratatui (GitHub)](https://github.com/d-holguin/async-ratatui) — FPS counter + tick/render/input loop reference
- [SolidJS reactivity docs](https://docs.solidjs.com/advanced-concepts/fine-grained-reactivity) — Signal/Effect/Memo definitions
- [SolidJS memos docs](https://docs.solidjs.com/concepts/derived-values/memos) — caching semantics for our `Memo<T>`
- [Textual reactivity guide](https://textual.textualize.io/guide/reactivity/) — reactive-attribute model (coarse, contrast point)
- [Textual widgets guide](https://textual.textualize.io/guide/widgets/) — scope of a Python TUI framework (what we deliberately don't build)
- [libcaca overview (Grokipedia)](https://grokipedia.com/page/libcaca) — canvas blits, animation patterns, color/dither features as historical baseline
- [AAlib (Wikipedia)](https://en.wikipedia.org/wiki/AAlib) — image → ASCII heritage; informs differentiator framing
- [Awesome ASCII animation (GitHub)](https://github.com/mu-ct/awesome-ascii-animation) — landscape survey
- [PyO3 Buffer Protocol (DeepWiki)](https://deepwiki.com/PyO3/pyo3/4.5-capsules-and-opaque-data) — zero-copy strategy for Python bindings milestone
- [rust-numpy (GitHub)](https://github.com/PyO3/rust-numpy) — pattern for zero-copy Grid-as-array on the Python side
- [Crates.io publishing (Cargo Book)](https://doc.rust-lang.org/cargo/reference/publishing.html) — required metadata, dry-run, yanking
- [Crates.io 2026 update (Rust Blog)](https://blog.rust-lang.org/2026/01/21/crates-io-development-update/) — Trusted Publishing for our CI
- [Processing.py reference: draw()](https://py.processing.org/reference/draw) — `setup`/`draw` mental model that creative-coders bring (informs DSL ergonomics)
- [Textualize blog: 7 things building a TUI framework](https://www.textualize.io/blog/7-things-ive-learned-building-a-modern-tui-framework/) — modern user expectations (smooth animation, mouse, true color, optional features)

**Confidence by area:**
- tachyonfx / ratatui / voxcii feature lists: **HIGH** (verified from official repos and docs)
- SolidJS primitive semantics: **HIGH** (official Solid docs)
- Textual reactivity scope (used as a contrast point, not a build target): **HIGH**
- "What users expect" framing: **MEDIUM** (synthesis of Textualize blog + ratatui community discourse + general Rust crate publishing best-practices; not a survey)
- Demoscene / libcaca / AAlib historical features: **MEDIUM** (encyclopedic sources)
- PyO3 zero-copy patterns: **HIGH** (official docs + rust-numpy)
- Anti-feature reasoning: **HIGH** (anchored to PROJECT.md decisions)

---
*Feature research for: declarative reactive terminal scene manager*
*Researched: 2026-04-14*
