# happyterminals

## What This Is

A declarative, reactive terminal scene manager with GPU-quality visual effects rendered as pure text. Users describe scenes (objects, effects, signals) and the framework handles 3D projection, compositing, and ANSI output — running on every terminal ever made, from Windows Terminal to macOS Terminal.app to SSH into a Raspberry Pi. Rust core for the hot path, Python bindings for the creative path. Public open-source library from day one.

## Core Value

**Terminal art should feel like magic, not plumbing.** If everything else gets cut, the framework must let someone write a tiny declarative scene description and see a cinematic, reactive, cross-terminal result — without touching ANSI escapes, buffers, or draw calls.

## Requirements

### Validated

<!-- Shipped and confirmed valuable. -->

(None yet — ship to validate)

### Active

<!-- Current scope. Building toward these. -->

- [ ] Reactive core: `Signal`, `Effect`, `Memo` with fine-grained re-execution (SolidJS-style, no VDOM diffing)
- [ ] `Grid` buffer: cells with char + fg + bg + attributes
- [ ] `Pipeline` executor: ordered chain of `Grid → Grid` transforms, composable and nestable
- [ ] Ratatui backend adapter: `Grid → ratatui::Buffer` → terminal I/O via crossterm
- [ ] tachyonfx integration: all 50+ effects usable inside the Pipeline
- [ ] 3D ASCII renderer (re-implemented fresh, voxcii-inspired): z-buffer, perspective projection, shading ramp, OBJ/STL mesh loading, camera controls
- [ ] Scene graph + transition manager: layer objects, z-order, transition A→B with an effect
- [ ] JSON recipe loader + validator (extends tachyonfx's Effect DSL to full scenes; LLM-generatable, hand-editable)
- [ ] Declarative scene DSL (Rust API surface, later mirrored in Python)
- [ ] Python bindings via PyO3 — zero-copy where possible, `asyncio` integration — this is the **final milestone**
- [ ] Milestone 1 exit: a spinning ASCII cube demo — signal-driven rotation → 3D projection → one tachyonfx effect → ratatui output, in a single small Rust binary that proves the whole stack end-to-end
- [ ] Public release: dual-licensed MIT OR Apache-2.0, published to crates.io (and PyPI for the Python milestone)

### Out of Scope

<!-- Explicit boundaries. Includes reasoning to prevent re-adding. -->

- **Haskell bindings** — removed from scope. The manifesto mentioned them for Eclusa integration; Python bindings cover the creative layer and Eclusa will consume happyterminals via the same Python surface (or directly via the Rust crate). Parked, not backlogged — if Haskell support is ever wanted, it's a new project.
- **GPU shaders / LD_PRELOAD tricks / special-terminal requirements** — violates the "pure text output, universal terminal support" principle. Anything that breaks over SSH or in a VT100 is wrong.
- **React-style VDOM / reconciliation** — explicitly rejected. Fine-grained signals only.
- **Phase 5 "fun" items** (audio-reactive scenes, AI prompt→scene generation, GLSL→ASCII shader transpiler, live-coding REPL, multi-monitor scenes) — parked in `999.x` backlog. Revisit after the Python bindings milestone ships. Valuable but not on the critical path.
- **Forking voxcii or tui-vfx** — we re-implement the 3D renderer fresh (voxcii-inspired), and we build on tachyonfx (not tui-vfx) as the effects foundation.

## Context

- **Ecosystem state (2025):** Ratatui is the de-facto Rust TUI framework. tachyonfx (1,182★, official ratatui-org project) is the effects library — 50+ built-in effects, composable DSL, WASM + browser-based editor, active maintenance. tui-vfx (8★, ~5 weeks old) is promising but not mature; we don't build on it. voxcii is the reference ASCII 3D viewer but isn't packaged as a library — we take inspiration, not code. PyO3 is the standard Rust↔Python bridge.
- **Existing workspace state:** Cargo workspace already scaffolded with three placeholder crates — `happyterminals-core`, `happyterminals-renderer`, `happyterminals-compositor`. Vendored copies of `pyo3`, `ratatui`, and `tui-vfx` exist under `vendor/` for reference reading only (not as dependencies). README.md currently lists tui-vfx as a core dep — stale, needs correction to tachyonfx in an early phase.
- **Who uses this:** Public open-source users first (Rust TUI devs, Python creative coders, terminal art hackers). Eclusa is the workflow manager being used to build it, not the target market — the project stands on its own.
- **Design lineage:** SolidJS for reactivity (fine-grained, no diffing), tachyonfx for compositing, voxcii for 3D approach, **react-three-fiber (R3F) for the declarative scene-graph feel** (components-as-props, tree-as-scene, re-render on prop change — but adapted to fine-grained signals, not VDOM), chroma for the "audio-reactive" inspiration (parked), mixed-signals for easing primitives (future).

## Constraints

- **Tech stack**: Rust for the hot path (rendering, compositing, projection, ANSI output); Python via PyO3 for the creative path (scene description, signals, effects) — because iteration speed matters on one side, speed-of-execution on the other.
- **Compatibility**: Output is pure text + ANSI escapes. Must work on Windows Terminal, GNOME Terminal, macOS Terminal.app, iTerm2, Kitty, tmux, screen, SSH sessions, and (minus color) VT100. No GPU, no OS-specific APIs.
- **Performance**: Fine-grained reactivity — when a signal changes, only cells that depend on it re-render. No full-buffer redraws, no diffing passes. Grid operations and 3D math stay in Rust.
- **Dependencies**: ratatui (backend), tachyonfx (effects), pyo3 (Python bridge, final milestone). No tui-vfx despite the vendored copy. No voxcii as a dep.
- **License**: MIT OR Apache-2.0 (dual, Rust-ecosystem standard, explicit patent grant via Apache).

## Key Decisions

<!-- Decisions that constrain future work. Add throughout project lifecycle. -->

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Build on **tachyonfx**, not tui-vfx | Mature (1,182★), official, 50+ effects, DSL + WASM editor. tui-vfx is 5 weeks old with 8★ — too immature to stake on. | — Pending |
| **Re-implement the 3D renderer fresh**, voxcii-inspired | voxcii isn't packaged as a library. A fresh re-implementation keeps dependencies clean and the API shaped to our needs. | — Pending |
| **SolidJS-style signals**, not React-style VDOM | Terminal cells are already a grid; VDOM diffing is overhead for no gain. Fine-grained reactivity means surgical redraws. | — Pending |
| **Rust core + Python bindings**, no Haskell | Python covers the creative layer; Eclusa can consume via Python or the Rust crate directly. Haskell bindings removed as scope bloat. | — Pending |
| **Python bindings are the LAST milestone** | The Rust layers (reactive core, renderer, compositor, DSL, JSON recipes) must be solid before a Python surface is worth shipping. | — Pending |
| **Milestone 1 exit = spinning cube demo** | Smallest artifact that exercises every layer — signal, 3D, effect, ratatui. Proves the stack before we deepen it. | — Pending |
| **Public release from day one**, dual MIT OR Apache-2.0 | Project stands on its own. Rust-ecosystem standard licensing maximizes adoption. | — Pending |
| **Phase 5 "fun" items → 999.x backlog** | Audio-reactive, AI scene gen, shader transpile, live REPL, multi-terminal — all valuable, none on the critical path. Revisit post-Python milestone. | — Pending |
| **Roadmapper agent decides milestone ordering** | User deferred ordering to the eclusa-roadmapper, which will propose from research. Bottom-up vs vertical-slice to be resolved there. | — Pending |
| **Scene DSL takes cues from react-three-fiber** | R3F's declarative component/prop model ("a tree of primitives is the scene") is the best prior art for terminal 3D scene description. We adapt the shape — tree of scene nodes with props that can be signals — but replace React's VDOM with fine-grained signal reactivity. | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/eclusa:transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/eclusa:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-14 after initialization*
