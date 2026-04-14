# Research Summary — happyterminals

**Domain:** Declarative, reactive terminal scene manager. Rust core + PyO3 bindings (final milestone). Spinning ASCII cube = M1 exit. Public OSS, dual MIT OR Apache-2.0.
**Synthesized:** 2026-04-14
**Sources:** STACK.md, FEATURES.md, ARCHITECTURE.md, PITFALLS.md

---

## Executive Summary

The research converges on a clear, opinionated build: a **six-crate Rust workspace** where a tiny SolidJS-style reactive core feeds a flat scene-IR that is rendered into a single `Grid` (newtyped over `ratatui::Buffer`), transformed by a `Pipeline` of `dyn Effect` trait objects (tachyonfx adapted as one stage), and flushed through ratatui's built-in diff. Every surface — Rust builder DSL, JSON recipes, and (last-milestone) Python — compiles to the same `SceneIr`. react-three-fiber's "tree of primitives with props that can be signals" is the correct shape for the DSL; the key departure is **fine-grained signals instead of VDOM**.

Three decisions dominate downstream work and must be locked in before Milestone 1: (a) **single-threaded reactive core** with a `Send` setter-channel for cross-thread writes (PITFALLS.md §6), (b) **Grid-as-newtyped-ratatui-Buffer** so the tachyonfx adapter is trivial and ratatui owns the ANSI diff (ARCHITECTURE.md §5.2, §6.3), (c) **PyO3 isolated in its own `-py` cdylib crate** so libpython never touches the hot path (STACK.md §4.4, PITFALLS.md §3). Ignoring any of these forces a rewrite later.

The biggest risks are not technical novelty but **foundational discipline**: panic-safe terminal lifecycle, owner-tree reactive disposal, grapheme/width math, true-diff rendering (no full-buffer redraw), and workspace hygiene (stub crates currently depend on `tui-vfx` and `pyo3` before any real code exists). Milestone 1 cannot start until a **Phase 0 cleanup** clears that debt, relocates vendored dirs, reserves names on crates.io + PyPI, and establishes the dual-license files. After that, the architecture supports a clean bottom-up vertical slice: reactive core → Grid + ratatui backend → Pipeline + tachyonfx → renderer → scene IR + builder → spinning cube demo.

---

## Key Findings

### From STACK.md — Final Crate Picks (all versions verified on crates.io 2026-04-14)

| Concern | Crate | Version | Source |
|---|---|---|---|
| Terminal framework (apps) | `ratatui` | 0.30.0 | STACK §1.1 |
| Terminal framework (libs) | `ratatui-core` | 0.1.0 | STACK §1.1 |
| Backend | `ratatui-crossterm` + `crossterm` | 0.1.0 / 0.29.0 | STACK §1.2 |
| Effects | `tachyonfx` | 0.25.0 | STACK §1.3 |
| Reactivity | `reactive_graph` (Leptos core) — **wrapped behind our own `Signal`/`Memo`/`Effect`** | 0.2.13 | STACK §1.4 |
| Reactive scheduler | `any_spawner` | 0.3.0 | STACK §1.4 |
| 3D math | `glam` | 0.32.1 | STACK §2 |
| OBJ | `tobj` | 4.0.3 | STACK §3.1 |
| STL | `stl_io` | 0.11.0 | STACK §3.2 |
| Serde | `serde` / `serde_json` | 1.x / 1.0.149 | STACK §1.5 |
| Schema gen | `schemars` (1.x API rewrite) | 1.2.1 | STACK §5.1 |
| Schema validate | `jsonschema` (0.46 API) | 0.46.0 | STACK §5.2 |
| PyO3 | `pyo3` | 0.28.3 | STACK §4.1 |
| PyO3 ↔ asyncio | **`pyo3-async-runtimes`** (NOT `pyo3-asyncio`) | 0.28.0 | STACK §4.2 |
| Python build | `maturin` | 1.13.1 | STACK §4.3 |
| Errors (lib/bin) | `thiserror` 2.0.18 / `color-eyre` 0.6.5 | — | STACK §1.5 |
| Tests | `insta` 1.47.2 / `proptest` 1.11.0 / `criterion` 0.8.2 | — | STACK §7 |
| Small strings / builder | `compact_str` 0.9.0 / `bon` 3.9.1 | match tachyonfx | STACK §1.5 |
| **MSRV** | **Rust 1.86** (ratatui 0.30 floor) | — | STACK §6.2, §9 |

**Anti-recommendations (STACK §8):** `tui-rs` (deprecated), `tui-vfx` (immature), `termion` (Unix-only), `cgmath` (unmaintained 2021), `pyo3-asyncio` (abandoned 2023), pre-`Bound` PyO3 patterns (`&PyAny`, `IntoPy`), `schemars` 0.8 syntax, `jsonschema::JSONSchema::compile`, pyo3 in core/library crates, library crates depending on the `ratatui` facade, rolling our own reactive graph in M1, forking voxcii, `unsafe` in Grid pre-measurement.

### From FEATURES.md — Prioritization

**P0 — Must exist for M1 "spinning cube" exit:** Signal/Effect/Memo, Grid cell buffer, Pipeline executor, ratatui adapter, event loop + FPS + input + resize, tachyonfx integration (≥10 effects), 3D renderer (z-buffer + projection + shading ramp + orbit camera), scene graph + z-order, declarative Rust DSL, spinning-cube example, README + GIF.

**P1 — Still in v1 launch (before Python milestone):** OBJ mesh loading, transition manager, JSON recipe loader + validator, `cargo doc` coverage, cross-terminal verification matrix, crates.io publish, ≥4 runnable examples beyond the cube.

**P2 — Python milestone + post-v1:** PyO3 bindings (with `pyo3-async-runtimes`), PyPI + maturin wheels (abi3 across 3.10–3.13), sync `run()` first (asyncio deferred — ARCH §9.4), Python DSL mirror, particle systems, easing primitives, color palette presets, STL loading, JSON hot-reload, `Scene as ratatui::Widget` interop, long-form tutorial.

**Backlog (999.x):** L-systems, audio-reactive scenes, AI prompt→scene tool (out-of-tree), GLSL→ASCII transpiler, live-coding REPL, multi-terminal scenes, WASM runtime, visual scene editor.

**Anti-features (FEATURES §Anti-Features):** GPU shaders, LD_PRELOAD, inline image protocols, React-VDOM, built-in widget library, CSS-like styling DSL, Haskell bindings, forking voxcii/tui-vfx, imperative draw-call API, forking ratatui.

### From ARCHITECTURE.md — System Snapshot

**Six-crate split** (ARCH §1.1):

```
crates/
├── happyterminals-core/       Signal/Effect/Memo + Grid + errors. std + slotmap only. NO pyo3, NO ratatui facade.
├── happyterminals-pipeline/   Effect trait, Pipeline<Vec<Box<dyn Effect>>>, TachyonAdapter.
├── happyterminals-renderer/   3D rasterizer, projection, camera, primitives, mesh loading.
├── happyterminals-scene/      SceneIr, layered SceneGraph, TransitionManager. Camera owned by Scene.
├── happyterminals-dsl/        Rust builder + JSON loader/validator — both compile to SceneIr.
├── happyterminals-backend-ratatui/  Grid → ratatui::Buffer, crossterm EventStream, run() loop.
├── happyterminals/            META-crate: curated public re-exports.
└── happyterminals-py/         PyO3 cdylib. Excluded from default-members. Built only at Python milestone.
```

**Load-bearing data flow (ARCH §3):**

```
signal.set() → Effect re-runs → writes scene node prop + scene_dirty=true
           ↓ (next frame tick on render thread, synchronous)
grid.clear() → renderer.draw(&Scene, &Camera, &mut Grid) using untracked reads
           → pipeline.apply(&mut Grid, dt)
           → backend blits Grid cells into terminal.current_buffer_mut()
           → terminal.flush() — ratatui diffs and emits minimal ANSI
```

**Four hard rules (ARCH §12 anti-patterns):**

1. **Effects mutate scene state only. Never the Grid.** Render loop walks the scene once per frame.
2. **Renderer uses `signal.untracked()` for scene reads.** Must not subscribe; subscriptions belong to scene-node Effects.
3. **Single Grid; ratatui owns the diff.** No custom ANSI diff layer. No per-effect scratch. All effects mutate `&mut Grid` in place.
4. **One IR (`SceneIr`). Three front-ends** (Rust builder / JSON / Python) all compile to it.

**Async at perimeter only** (ARCH §3, §10): `tokio::select!` between `tick.tick()` and `crossterm::EventStream`. Inside a frame, everything synchronous on the render thread. Cross-thread signal writes go through a `Send` setter channel drained each tick.

**react-three-fiber lineage (PROJECT.md):** DSL is a tree of typed nodes with props. Departure: props can be `Signal<T>` and re-renders are surgical via fine-grained dependency tracking, not VDOM diffing. Sample in ARCH §8.2: `cube().rotation(&r)` binds a prop to a signal reference.

### From PITFALLS.md — Top Critical Gotchas

1. **§1, §31 Terminal left in raw mode on panic** — RAII `TerminalGuard` + panic hook restoring cursor, raw mode, alt screen, mouse, SGR.
2. **§4 Reactive memory leak via missing owner tree** — `create_root`/`Owner`/`onCleanup` not optional; scene transitions dispose outgoing owner. 10k-transition RSS test in CI.
3. **§5 Over-fire / under-fire** — two-phase propagation, `batch()` API, runtime cycle detection panicking with "signal cycle A→B→A".
4. **§6 Send/Sync ambiguity** — single-threaded core; `Runtime::send_handle()` returns `Send` producer to an mpsc drained by render loop.
5. **§7 Grapheme/width math** — `Cell` holds one grapheme cluster + width field; all width via `Grid::put_str`.
6. **§12 Full-buffer redraw undermines the thesis** — dirty-span output; "1-cell change → ~10 bytes" is the M1 exit gate.
7. **§16 tachyonfx `Effect` name clash** — rename theirs to `Fx`/`VisualEffect` before any code.
8. **§18 `pyo3-asyncio` is dead** — use `pyo3-async-runtimes` 0.28 from day one.
9. **§21 Name squatting** — reserve `happyterminals` on crates.io *and* PyPI plus variants before announcement.
10. **§22 Dual-license hygiene** — `LICENSE-MIT` + `LICENSE-APACHE` both at root; `license = "MIT OR Apache-2.0"` SPDX.

Full mapping in PITFALLS.md "Pitfall-to-Phase Mapping" (33 pitfalls).

---

## Critical Pitfalls Per Phase

### Phase 0 — Prerequisite Cleanup (before M1)

- **§2 README/docs drift:** `tui-vfx` → `tachyonfx` across README.md, project.md, compositor README. Doc-lint CI.
- **§3 Stub crates with stale deps:** Delete `tui-vfx` everywhere; remove `pyo3` from core; stubs have NO deps until real call site.
- **§32 Vendored dirs stale:** Move `vendor/{pyo3,ratatui,tui-vfx}` to `vendor/_reference/` with `STAMP.txt`; `linguist-vendored=true`.
- **§21 Registry naming:** Reserve `happyterminals` on **both** crates.io and PyPI + variants before any announcement.
- **§22 Dual-license files:** Both LICENSE files at root; SPDX `"MIT OR Apache-2.0"`; contribution paragraph.
- **§33 Workspace dep drift:** `[workspace.dependencies]` so all crates inherit pinned versions.

### Phase M1.0 — Reactive core

- **§4 Owner tree.** `create_root` + `Owner` + `onCleanup` in first commit.
- **§5 Propagation semantics.** Two-phase, `batch()`, cycle-detection panic.
- **§6 Send/Sync model.** Single-threaded + `SignalSetter` channel. Public-API tattoo.
- **§26 Clock + RNG injection** for deterministic snapshots.

### Phase M1.1–M1.2 — Grid + ratatui backend + Pipeline + tachyonfx

- **§1, §31 Panic-safe TerminalGuard** in first demo.
- **§7 Grapheme-cluster Cell + `Grid::put_str`.** No `char`-only API.
- **§12 True diff from day one.** M1 exit test: "1 signal change → ~10 bytes to TTY."
- **§16 `Effect` name clash resolved.** `tachyonfx::Effect` → `Fx` in our surface.

### Phase M1.3 — Minimal renderer (cube primitive)

- **§14 Aspect ratio parameter** from first cube.
- **§13 Z-fighting / reversed-Z + scene-fit near/far.** No flicker.
- **§11 Per-frame allocation churn** — reusable `Vec<u8>`, cached SGR escapes, criterion budget.

### Phase M1.4–M1.5 — Scene IR + DSL + Spinning-cube exit

- **§28 Errors not panics** on public surface; `clippy::unwrap_used` denied in lib.
- **§27 Hello-world ≤25 lines.** Quickstart helpers.
- **§30 O(n·m) composition** — whole-grid passes, no per-object effect loops.

### Phase M2.x — Renderer depth (mesh + camera + particles)

- **§15 OBJ/STL brittleness** — fixture corpus; triangulate + flat-normal + winding-normalize; `Result<Mesh, _>`.
- **§8 Color regression** — RGB → 256 → 16 → mono pipeline; `--force-color`; `NO_COLOR`; tmux `Tc` docs.
- **§10 Resize race** — drained between frames; debounce; Windows-Terminal test.
- **§29 Logging** — file/stderr only; `trace!` feature-gated.

### Phase M3.x — Compositor + Scene + JSON recipes

- **JSON sandboxing** — pure data; effect names in static registry; no eval; mesh path sandbox; ANSI-injection strip.
- **§5 Round-trip property tests.**
- **§9 tmux DCS passthrough** — only if non-SGR sequences ship; gate on detection.

### Phase M4.x — PyO3 / Python bindings (FINAL)

- **§18 `pyo3-async-runtimes`** never `pyo3-asyncio`.
- **§19 GIL contention** — render loop in `Python::allow_threads`; Python writes via setter channel; Rust-side cache.
- **§20 Zero-copy hazards** — default copy semantics; zero-copy only with explicit `freeze()`/`lock()`.
- **§25 CI matrix cost** — abi3 wheel; tiered matrix.
- **§6 Threading rules** documented in Python README.

### Pre-release polish (crosses M1 and M4 exits)

- **§23 Semver discipline** — `cargo semver-checks` CI; Keep-a-Changelog CHANGELOG.
- **§24 MSRV pinned** 1.86; CI MSRV + stable.
- **§17 WASM divergence** — not in M1; revisit only post-M4.

---

## Prerequisite Cleanup Items (Phase 0)

- [ ] **Docs sweep** — `tui-vfx` → `tachyonfx` in README.md, project.md, compositor README; keep one "why not tui-vfx" rationale.
- [ ] **Stub crate deps reset** — strip `[dependencies]` from core/renderer/compositor until real call site. `pyo3` leaves core.
- [ ] **Workspace scaffolding** — `[workspace.dependencies]` block; members use `dep.workspace = true`.
- [ ] **Crate rename plan** — `compositor` → `pipeline`; add empty `-scene`, `-dsl`, `-backend-ratatui`, meta `happyterminals`, commented-out `-py`.
- [ ] **Vendor relocation** — `vendor/{pyo3,ratatui,tui-vfx}` → `vendor/_reference/{name}/` + `STAMP.txt`; `.gitattributes` markers; never `path =` from Cargo.toml.
- [ ] **Dual-license files** — LICENSE-MIT + LICENSE-APACHE at root; SPDX license string.
- [ ] **Registry reservation** — minimal placeholder on crates.io + PyPI for `happyterminals`, `happyterminals-core`, `happyterminals-py`, `happy-terminals`.
- [ ] **Rust toolchain pin** — `rust-toolchain.toml` at 1.86, clippy + rustfmt.
- [ ] **Baseline CI** — fmt, clippy `-D warnings`, test, docs `-D warnings`, `cargo udeps`, `cargo tree -d`.
- [ ] **Doc-lint CI step** — grep forbidden strings (`tui-vfx` outside vendor/rationale, `Haskell bindings`, `pyo3-asyncio`, `cgmath`, `tui-rs`).

**Exit:** `cargo build --workspace` clean; `cargo tree -d` no duplicates; both LICENSE files; `grep -r tui-vfx` only hits vendor/rationale; both registries reserved.

---

## Unresolved Questions (for the roadmapper)

| # | Question | Surfaces in | Decide by |
|---|---|---|---|
| 1 | `Memo<T>: PartialEq` bound — equality-skip cost vs spurious downstream re-runs | ARCH §4.3 | M1.0 |
| 2 | Async runtime for backend: tokio vs smol | STACK §1.4, ARCH §10.1 | M1.1 |
| 3 | `Effect` name clash — rename ours or theirs? (Recommendation: theirs → `Fx`) | PITFALLS §16 | M1.2 |
| 4 | Wide-char display — ship grapheme+width fields now, defer wide-cell rendering? | PITFALLS §7 | M1.2 |
| 5 | JSON schema versioning — `$version` + semver, migrations TBD | ARCH §14 Q4 | M3 |
| 6 | Python primary surface: sync `run()` first vs asyncio from day one | ARCH §9.4 | M4 planning |
| 7 | Grid-as-ratatui-Buffer newtype layout compat — 1–2 day spike before committing | ARCH §5.1, §6.3 | M1.1 |
| 8 | Roadmapper ordering philosophy (bottom-up-vertical-slice default) | ARCH §11.2 | Pre-M1.0 |

---

## Top 10 Actionable Insights

1. **Six-crate workspace, not three** — isolate PyO3 in `-py` cdylib; split pipeline/scene/dsl/backend. (ARCH §1)
2. **Reactive core: wrap `reactive_graph` 0.2 behind our own types** — API stability + Python cleanliness + migration option. (STACK §1.4, ARCH §4.1)
3. **Grid = newtype over `ratatui::Buffer`** — tachyonfx adapter trivial, ratatui owns the ANSI diff. Validate layout compat in an M1.1 spike. (ARCH §5.2, §6.3)
4. **Single-threaded reactive graph + `Send` setter channel** — documented up front. Python users assume the opposite. (PITFALLS §6, ARCH §4.2)
5. **True-diff rendering from day one** — "1 signal change → ~10 bytes" is the M1 exit gate. (PITFALLS §12, ARCH §3)
6. **One `SceneIr`, three front-ends** — Rust builder, JSON, Python all compile to the same IR. react-three-fiber's tree-of-typed-nodes shape maps cleanly. (ARCH §8, PROJECT.md)
7. **Phase 0 cleanup is mandatory and low-cost** — README drift, stub-crate dep rot, vendor dirs, license files, registry reservation. Before any new-feature PR. (PITFALLS §2/§3/§21/§22/§32/§33)
8. **tachyonfx `Effect` name clash resolved before any Pipeline code** — theirs becomes `Fx`; ours keeps the name. (PITFALLS §16)
9. **Panic safety foundational, not polish** — `TerminalGuard` + panic hook in first demo commit; PyO3 panic surface to `PyErr` via same hook. (PITFALLS §1, §31)
10. **`pyo3-async-runtimes` (0.28), never `pyo3-asyncio`** — picked in Phase 0's workspace table even though PyO3 work lands at M4. (STACK §4.2, PITFALLS §18)

---

## Milestone Sketch (first-pass; roadmapper refines)

Bottom-up with vertical-slice pulls per ARCH §11.

### Phase 0 — Workspace Cleanup (prerequisite)
Scope: all Prerequisite Cleanup Items above.
Exit: clean build, zero dep duplicates, both LICENSEs, both registries reserved, READMEs match stack, CI baseline green.

### Phase M1.0 — Reactive Core
Scope: `happyterminals-core` Signal/Memo/Effect/Owner/Root/onCleanup/batch/untracked/cycle-detection/SignalSetter + Clock + RNG injection.
Exit: diamond-dep + disposal + 10k-transition RSS tests pass.
Pitfalls: §4, §5, §6, §26.

### Phase M1.1 — Grid + ratatui Backend (static)
Scope: Grid newtyped over `ratatui::Buffer`, grapheme-cluster cell, `Grid::put_str`; `-backend-ratatui` TerminalGuard + panic hook + tokio::select loop skeleton + resize.
Exit: static Grid renders; Ctrl-C leaves sane shell; resize works.
Pitfalls: §1, §7, §10, §12, §31.
Spike: Grid/Buffer layout compat, 1–2 days.

### Phase M1.2 — Pipeline + tachyonfx adapter
Scope: Effect trait (ours), Pipeline<Vec<Box<dyn Effect>>>, TachyonAdapter wrapping `tachyonfx::Shader` (renamed `Fx`), real Duration dt.
Exit: vignette applies to static Grid; `fade_in(2s)` test verifies timing; no `Effect` name collision.
Pitfalls: §16, §28, §30.

### Phase M1.3 — Minimal renderer (cube primitive)
Scope: `-renderer` Renderer3D, projection with cell-aspect, reversed-Z, ASCII shading ramp, built-in Cube primitive, orbit camera. No OBJ yet.
Exit: rotating cube no flicker; cube-shaped cube; criterion budget established.
Pitfalls: §11, §13, §14.

### Phase M1.4 — Scene IR + Rust builder DSL
Scope: `-scene` SceneIr, SceneGraph, Layer z-order, Camera on Scene; `-dsl` chainable Rust builder producing SceneIr (R3F-shaped tree).
Exit: `scene().layer(|l| l.cube().rotation(&r)).effect(fx::vignette(0.3)).build()?` compiles, binds a signal, valid IR.
Pitfalls: §27, §28.

### Phase M1.5 — Spinning Cube Demo (MILESTONE 1 EXIT)
Scope: `examples/spinning-cube/` <100 LOC end-to-end; README + GIF; cross-terminal verification (Win Terminal, GNOME, macOS, iTerm2, tmux, SSH).
Exit: PROJECT.md M1 exit criterion met.
Pitfalls: §1 (panic test), §12 (diff bytes), §14 (cube shape), §27 (hello-world length).

### Phase M2 — Renderer depth
Scope: OBJ via `tobj` (triangulate, normalize winding, flat-normal fallback); camera modes as signals; particle infrastructure; color-mode pipeline (RGB→256→16→mono).
Exit: mesh-viewer example renders a real `.obj` without panic on quads; `NO_COLOR` + `--force-color` honored.
Pitfalls: §8, §15, §30.

### Phase M3 — Compositor + Scene + JSON recipes
Scope: TransitionManager (two-scene blend via BlendEffect); multi-layer compositing; `-dsl::json` loader + schemars schema + jsonschema validator + signal binding by name; ≥5 runnable examples beyond cube; first crates.io publish.
Exit: JSON recipe loads, validates, binds signal, renders identically to Rust builder.
Pitfalls: §9, §28, JSON sandbox.

### Phase M4 — Python bindings (FINAL per PROJECT.md)
Scope: activate `-py` cdylib; PyO3 0.28 wrappers around Signal/Scene/Effect/run; maturin abi3 wheels cp310–cp313 Linux/macOS/Windows; sync `run(scene)` first (asyncio v0.2 unless Q6 overridden); Python builder mirror; PyPI Trusted Publishing.
Exit: 10-line Python cube example runs; wheels install clean all platforms.
Pitfalls: §18, §19, §20, §25.

### Phase 999.x — Backlog
Audio-reactive, AI prompt→scene, GLSL→ASCII, live-coding REPL, multi-terminal, WASM, visual editor. Revisit only after M4 + v1.x user feedback.

---

## Confidence Assessment

| Area | Confidence | Notes |
|---|---|---|
| Stack | **HIGH** | Versions verified against crates.io API 2026-04-14. |
| Features | **HIGH** | Comparator features verified; user-expectation framing MEDIUM. |
| Architecture | **HIGH** on crate split + data flow + pipeline; **MEDIUM** on reactive build-vs-reuse; **MEDIUM** on PyO3 asyncio. |
| Pitfalls | **HIGH** on portability + reactive + ASCII-3D + OSS hygiene. |

**Overall confidence: HIGH.**

### Gaps to address during planning

- Grid/Buffer layout-compat spike (M1.1) — 1–2 days before locking newtype.
- Memo equality trade-off (Q1) — M1.0, benchmark-driven.
- Python async surface (Q6) — user decision before M4.
- Roadmapper ordering philosophy (Q8) — user decision before M1.0 (default: bottom-up-vertical).

---
*Synthesized for: declarative reactive terminal scene manager (Rust core + Python bindings, public OSS)*
*Date: 2026-04-14*
