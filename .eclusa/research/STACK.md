# Stack Research — happyterminals

**Domain:** Declarative reactive TUI framework with ASCII 3D rendering, composable visual effects, optional Python bindings
**Researched:** 2026-04-14
**Researcher:** eclusa-researcher (Stack dimension)
**Overall confidence:** HIGH

All version numbers in this document were verified against the live crates.io registry API on 2026-04-14, **not** recalled from training data. Where a claim relies only on web search (no primary-source verification), it is marked LOW confidence.

---

## Headline Recommendations

| Concern | Recommendation | Version (verified 2026-04-14) | Confidence |
|---------|----------------|-------------------------------|------------|
| Terminal framework | `ratatui` (full crate, not yet `ratatui-core` for libs) | `0.30.0` | HIGH |
| Terminal I/O backend | `crossterm` (default), via `ratatui-crossterm` | `0.29.0` / `0.1.0` | HIGH |
| Effects engine | `tachyonfx` | `0.25.0` | HIGH |
| Reactivity primitives | **`reactive_graph`** (Leptos's standalone reactive core) | `0.2.13` | HIGH |
| 3D math | **`glam`** | `0.32.1` | HIGH |
| OBJ mesh loading | `tobj` | `4.0.3` | HIGH |
| STL mesh loading | `stl_io` | `0.11.0` | HIGH |
| JSON serde | `serde` + `serde_json` | `1.0.x` / `1.0.149` | HIGH |
| JSON Schema generation | `schemars` | `1.2.1` | HIGH |
| JSON Schema validation | `jsonschema` | `0.46.0` | HIGH |
| Python bindings (final milestone) | `pyo3` | `0.28.3` | HIGH |
| Python build tool | `maturin` | `1.13.1` | HIGH |
| PyO3 ↔ asyncio bridge | `pyo3-async-runtimes` (NOT `pyo3-asyncio`) | `0.28.0` | HIGH |
| Snapshot testing | `insta` | `1.47.2` | HIGH |
| Property testing | `proptest` | `1.11.0` | HIGH |
| Benchmarks | `criterion` | `0.8.2` | HIGH |
| Errors (libraries) | `thiserror` | `2.0.18` | HIGH |
| Errors (binaries / demos) | `color-eyre` | `0.6.5` | HIGH |
| Tracing / logs | `tracing` | `0.1.44` | HIGH |
| Workspace MSRV | Rust **1.86** (matches ratatui 0.30 MSRV) | stable line is 1.94 in April 2026 | HIGH |

---

## 1. Core Rust Crates

### 1.1 ratatui — the terminal framework

**Choice:** `ratatui = "0.30.0"`
**Released:** 2025-12-26
**Confidence:** HIGH (verified via crates.io API + ratatui.rs release highlights)

**Major news:** ratatui 0.30 split the monolithic crate into a workspace:

- `ratatui-core` (`0.1.0`) — types & traits only, evolves slowly, intended for **library/widget authors**
- `ratatui-widgets` (`0.3.0`) — built-in widgets
- `ratatui-crossterm` (`0.1.0`) — crossterm backend
- `ratatui-termion` (`0.1.0`) — termion backend
- `ratatui-termwiz` (`0.1.0`) — termwiz backend
- `ratatui-macros` (`0.7.0`) — utility macros
- `ratatui` (`0.30.0`) — convenience facade re-exporting the above behind feature flags

**MSRV:** 1.86.0 (ratatui's own constraint).

#### Recommendation by crate

| happyterminals crate | Should depend on | Rationale |
|----------------------|------------------|-----------|
| `happyterminals-core` (signals, Grid, Pipeline) | **`ratatui-core`** only | Core defines `Buffer`/`Cell` traits and nothing else. Stays out of widgets/backend churn. Mirrors what tachyonfx itself does (see §1.3). |
| `happyterminals-renderer` (3D → Grid) | **`ratatui-core`** only | Same — renderer emits cells, doesn't drive the terminal. |
| `happyterminals-compositor` (effects pipeline) | **`ratatui-core`** + `tachyonfx` | Compositor only needs Buffer types. |
| `happyterminals-app` / examples / demos (e.g. spinning cube binary) | **`ratatui`** with `crossterm` feature | Apps want the convenience facade and an actual backend. |
| `happyterminals-py` (PyO3 crate) | **`ratatui`** + crossterm feature | Same as a binary — needs the full driving stack. |

Anti-pattern: do **not** make library crates depend on the full `ratatui` facade. That couples them to widgets and a specific backend version they don't need.

**Sources:**
- crates.io API for `ratatui` (max_stable_version 0.30.0, updated 2025-12-26)
- https://ratatui.rs/highlights/v030/ — official release highlights
- https://github.com/ratatui/ratatui/blob/main/BREAKING-CHANGES.md

### 1.2 crossterm — terminal I/O

**Choice:** `crossterm = "0.29.0"` (transitively via `ratatui-crossterm = "0.1.0"`)
**Released:** 2025-04-05
**Confidence:** HIGH

Crossterm is the only sane default for cross-platform Rust terminal I/O in 2026. It works on Windows (without WSL), macOS, Linux, BSDs, over SSH, and inside tmux. Termion and termwiz exist (and ratatui has backends for both) but neither is a serious default for new code:

- **termion** is Unix-only — disqualified by the project's "Windows Terminal must work" constraint.
- **termwiz** is excellent but is wezterm's internal stack and pulls in a much heavier dependency tree. Reasonable as an opt-in backend later if someone wants Sixel/Kitty graphics passthrough; not a default.

In practice, application crates take `ratatui` with the `crossterm` feature; library crates do not depend on a backend at all.

### 1.3 tachyonfx — the effects layer

**Choice:** `tachyonfx = "0.25.0"`
**Released:** 2026-02-27
**Confidence:** HIGH (verified via crates.io API + dependency listing)

Confirmed dependency profile (from crates.io API):

```
ratatui-core  ^0.1.0   (normal)   ← depends on the slim core, not full ratatui
compact_str   ^0.9.0
unicode-width ^0.2.0
micromath     ^2.1.0   (fast trig approximations)
anpa          ^0.10.0  (parser combinators — for the Effect DSL)
bon           ^3.8.2   (builder macros)
thiserror     ^2.0
web-time      ^1.1
```

**Implication:** tachyonfx already follows the "library depends on `ratatui-core` only" pattern — happyterminals should adopt the same shape so the two compose without duplicate ratatui versions in the lockfile.

**Compatibility note (from web search, MEDIUM confidence):** later tachyonfx versions expose a `ratatui-next-cell` feature for downstream apps using ratatui ≥ 0.30 where `Cell::skip` was replaced by `Cell::diff_option`. We should enable it from the apps that pull in both. Verify by reading tachyonfx's `Cargo.toml` features list directly when wiring milestone 1.

### 1.4 Reactivity — fine-grained signals

This is the most interesting decision in the stack. The project is committed to **SolidJS-style fine-grained reactivity, no VDOM**. Three real options exist in Rust (2026):

| Option | Version | Verdict |
|--------|---------|---------|
| **`reactive_graph`** (Leptos's reactive core, extracted) | 0.2.13 (2026-02-16) | **RECOMMENDED** |
| `reaktiv` (pascalkuthe) | 0.1.1 (2025-12-19) | Watch list, not yet |
| `futures-signals` (Pauan) | 0.3.34 (2024-07-26) | Different model — push-based FRP, not Solid-style |
| Roll our own | — | Unjustified — see below |

#### Why `reactive_graph`

Confidence: HIGH (verified via docs.rs + crates.io + dep tree).

- It is **literally the reactive core that powers Leptos**, factored into its own crate. Leptos itself is the closest thing Rust has to SolidJS, and SolidJS is the explicit design lineage in `project.md`.
- The algorithm is documented as based on Reactively (the Solid-aligned push-pull algorithm), which gives the exact "only effects that read a changed signal re-run" semantics happyterminals wants.
- It is **runtime-agnostic by design** — docs say "can be used in the browser with `wasm-bindgen-futures`, in a native binary with `tokio`, in a GTK application with `glib`". A terminal app is exactly the same shape as the GTK case.
- It exposes the three primitives we want under names that are nearly identical to what `project.md` already uses: `Signal`, `Memo`, `Effect`, plus `RwSignal` for read-write splits and `ArcSignal` / `ReadSignal` / `WriteSignal` for ownership control.
- 958k downloads, active maintenance, 0.2.x line means breaking changes are tolerable but the API has stabilized.

**The catch:** Effects in `reactive_graph` are spawned on the next tick of an async runtime — they don't run synchronously when a signal changes. We need an `any_spawner = "0.3.0"` executor that drives them; for a TUI we provide a tiny per-frame pump or wire it to tokio. This is not a problem (it matches the SolidJS model and the per-frame render loop a TUI runs anyway), but it must be designed in from milestone 1.

**Public API choice:** wrap `reactive_graph`'s primitives in our own thin types (`happyterminals_core::Signal<T>`, `Effect`, `Memo<T>`) so the public surface is ours, not Leptos's. This:

1. lets us swap implementations later without an API break,
2. lets the Python `PyO3` layer expose `happyterminals.signal(...)` without leaking Leptos identifiers,
3. lets us add TUI-specific helpers (a `cell_signal!` macro that automatically marks a Grid cell dirty when a signal changes).

#### Why not `reaktiv`

Genuinely interesting (≈50% less memory via arena allocation, signals as metadata rather than wrapping values, skippable effects for large UIs) — but it is `0.1.1`, six months old, single maintainer, and we'd be the canary. Re-evaluate at milestone 4 or 5.

#### Why not roll our own

A correct fine-grained reactive graph is a small but subtle data structure (cycle detection, glitch-free updates, batching, drop-order safety). Two production-quality implementations already exist. Rolling our own in milestone 1 burns weeks and we'll still hit Reactively's same edge cases. Wrap, don't reinvent.

#### Why not `futures-signals`

Different model entirely — it's a push-based FRP layer over `futures::Stream`. Beautiful for some problems, but it doesn't give you `signal()` and `memo()` with implicit dependency tracking the way Solid does. Wrong shape for this project.

### 1.5 Supporting Rust crates

| Library | Version | Purpose | When to use |
|---------|---------|---------|-------------|
| `serde` | 1.0.x | Derive (de)serialization | Everywhere with `derive` feature |
| `serde_json` | 1.0.149 | JSON for scene recipes | Compositor + Python boundary |
| `schemars` | 1.2.1 | Generate JSON Schema from Rust types | Publish a `scene-schema.json` for LLM/editor authoring |
| `jsonschema` | 0.46.0 | Validate incoming JSON recipes against schema | Loader entry point |
| `thiserror` | 2.0.18 | Library error types with `derive(Error)` | All library crates |
| `color-eyre` | 0.6.5 | Pretty error reports for binaries | Demo binaries, not library code |
| `tracing` | 0.1.44 | Structured logging | Cross-crate diagnostics |
| `tracing-subscriber` | 0.3.x | Subscriber implementations | Apps only |
| `compact_str` | 0.9.0 | Stack-allocated small strings | Cell content; matches tachyonfx's choice |
| `bon` | 3.9.1 | Builder macro | Scene/effect builders; matches tachyonfx's choice |
| `slotmap` | 1.x | Stable arena keys | Scene graph node IDs (reactive_graph uses it internally too) |
| `rayon` | 1.12.0 | Data parallelism | Z-buffer rasterization at scale (post-MVP) |
| `ahash` or `rustc-hash` | latest | Faster hashmaps | Hot path lookups |

**Note on `schemars` 1.x:** the 1.0 release was a substantial breaking change from 0.8 (different attribute syntax, draft 2020-12 default). All examples in the wild from before 2025 are 0.8 syntax — read current docs, don't pattern-match old blog posts.

**Note on `jsonschema` 0.46:** API stabilized recently around `Validator::new(&schema)` with `Draft202012` as the default. Confidence MEDIUM on exact API shape — verify against current docs when implementing the loader.

---

## 2. 3D Math: glam vs nalgebra vs cgmath

**Choice:** **`glam = "0.32.1"`**
**Released:** 2026-03-06
**Confidence:** HIGH

| Library | Latest | Status | Verdict |
|---------|--------|--------|---------|
| `glam` | **0.32.1** (2026-03-06) | Active, weekly downloads in millions | **Use this** |
| `nalgebra` | 0.34.2 (2026-03-28) | Active, very capable | Use only if you need its specialty (statistics, decompositions) |
| `cgmath` | 0.18.0 (**2021-01-03**) | **Effectively unmaintained** | Do not use for new projects |
| `bevy_math` | 0.18.1 (2026-03-04) | Bevy-specific re-export of glam + extras | Don't pull Bevy in for math alone |

### Why glam

- **Right-sized for graphics.** It's exactly `Vec2`/`Vec3`/`Vec4`/`Mat3`/`Mat4`/`Quat`/`Affine3A` and the operations a renderer actually does — perspective projection, rotation composition, `look_at`, `mul_vec3`. No type-level dimension parameters, no decompositions you'll never use.
- **SIMD by default** on x86_64 (`Vec3A`, `Mat4` use SSE2 lanes). For a software ASCII rasterizer running per-frame projection over hundreds of mesh vertices, this matters.
- **Used by Bevy, rend3, wgpu's example code, and most of the modern graphics-Rust ecosystem.** Standard answer in 2026.
- **Tiny compile time** compared to nalgebra. Matters for a project that already pulls in ratatui + tachyonfx + reactive_graph + (eventually) pyo3.

### When nalgebra would be right

If happyterminals later grows scientific-style features (skeletal animation needing matrix decomposition, IK solvers, fitting / regression for generative geometry), nalgebra earns its keep. It's not wrong — it's just oversized for a perspective-projection-and-rotate renderer.

### Why not cgmath

Last release 2021-01-03. The Rust graphics ecosystem moved on. Anyone recommending it in 2026 is reciting old knowledge.

---

## 3. Mesh Loading

### 3.1 OBJ — `tobj = "4.0.3"`

**Released:** 2025-01-20
**Confidence:** HIGH

`tobj` ("tinyobjloader-rs") is the de-facto OBJ loader in Rust. 1.78M downloads, used by wgpu's examples, rend3, and most tutorials. Returns `Vec<Model>` + `Vec<Material>` with positions/normals/texcoords as flat `Vec<f32>` — the exact shape a rasterizer wants. No drama.

Alternatives considered:
- `obj` — older, less ergonomic, smaller community.
- Roll our own — OBJ has surprising edge cases (negative indices, polygon faces requiring triangulation, line continuations). Use the library.

### 3.2 STL — `stl_io = "0.11.0"`

**Released:** 2026-03-15
**Confidence:** HIGH

`stl_io` reads and writes both ASCII and binary STL, returns indexed triangle meshes (not just raw triangle soup), and is actively maintained. 2.84M downloads.

Alternatives considered:
- `stl` (the crate) — older, ASCII-only ergonomics issues, less active.

### 3.3 Other formats?

For milestone 1's spinning cube we don't even need a loader — embed the cube's 8 vertices / 12 triangles inline. Mesh loading enters at the renderer milestone. Defer GLTF (`gltf` crate) until a user actually asks; OBJ + STL covers "I have a mesh I want to see in my terminal" for ≥95% of demand.

---

## 4. PyO3 Stack (Final Milestone)

This entire section applies only to the **last** milestone, but the decisions affect the workspace shape from day one (we should not paint ourselves into corners that block PyO3 later).

### 4.1 `pyo3 = "0.28.3"`

**Released:** 2026-04-02
**Confidence:** HIGH (verified via crates.io API + PyO3 CHANGELOG)

Critical 0.28-line facts:
- **MSRV: Rust 1.83** (already covered by our 1.86 floor for ratatui).
- **Free-threaded Python (3.13+ no-GIL builds) is opt-out, not opt-in** — major reversal from 0.27.
- `__init__` support in `#[pymethods]` (no more separate `__new__` dance).
- `PyUntypedBuffer` for type-erased buffer handling — directly relevant for zero-copy Grid sharing.
- 0.27 deprecated `IntoPy` and `ToPyObject` in favour of `IntoPyObject`. Any code/example you copy from the internet older than ~mid-2025 will use deprecated APIs. **Read current pyo3 docs, don't pattern-match old blog posts.**
- The `Bound<'py, T>` API (introduced 0.21, stabilized through 0.27) is now the default — `&PyAny` / `Py<T>` patterns from old tutorials are wrong. All new code uses `Bound`.

### 4.2 `pyo3-async-runtimes = "0.28.0"` — NOT `pyo3-asyncio`

**Released:** 2026-02-04
**Confidence:** HIGH (verified via crates.io API + GitHub README)

This is the single most important "don't trust your training data" item in the whole stack:

- `pyo3-asyncio` (max version `0.20.0`, last published **2023-11-11**) is **abandoned**. It does not support pyo3 ≥ 0.21.
- `pyo3-async-runtimes` is the maintained successor — same team, fork, renamed because they couldn't reuse the crate name. Currently on `0.28.0` matching pyo3 `0.28`.
- Any tutorial, blog post, or LLM completion older than ~late-2024 will recommend `pyo3-asyncio`. **Wrong.**

Add it as: `pyo3-async-runtimes = { version = "0.28", features = ["tokio-runtime"] }` if we use tokio (we will, via reactive_graph's `any_spawner`).

### 4.3 `maturin = "1.13.1"`

**Released:** 2026-04-09
**Confidence:** HIGH

`maturin` is the standard build tool for PyO3-backed wheels. 1.x line is fully stable. `pyproject.toml` with `[build-system] requires = ["maturin>=1.13,<2"]` and `[tool.maturin] features = ["pyo3/extension-module"]`. CI: `maturin build --release --strip` for distribution wheels; `maturin develop` for local iteration.

### 4.4 Workspace shape implication

The `happyterminals-py` crate should be a **separate crate** in the same workspace, with `crate-type = ["cdylib"]` and a thin wrapper around the public APIs of `happyterminals-core` / `-renderer` / `-compositor`. Do **not** put `pyo3` as a dependency in the core crates — that forces every Rust-only consumer to compile against libpython. (This is the single most common PyO3-workspace mistake.)

---

## 5. JSON Scene Recipes — Schema + Validation

The project wants LLM-generatable, hand-editable JSON scene descriptions that extend tachyonfx's Effect DSL. Two-crate stack:

### 5.1 `schemars = "1.2.1"` — generate schema from Rust types

```rust
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SceneRecipe {
    pub objects: Vec<ObjectSpec>,
    pub effects: Vec<EffectSpec>,
}
```

Then a build-time `cargo xtask` (or a small binary) emits `scene-schema.json` shipped alongside the crate. LLMs and editors (VS Code's JSON schema support) consume it directly.

**Critical:** `schemars 1.0` (released October 2025) is a complete API rewrite vs the long-lived `0.8` line. It defaults to JSON Schema **draft 2020-12** and changed attribute syntax. Web examples from 2024 or earlier are 0.8 — they will not compile against 1.x.

### 5.2 `jsonschema = "0.46.0"` — validate at load time

```rust
let schema = serde_json::from_str(SCHEMA_JSON)?;
let validator = jsonschema::validator_for(&schema)?;
validator.validate(&recipe_json).map_err(|errors| ...)?;
```

The 0.4x line of `jsonschema` substantially rewrote the API around `Validator` (older code used `JSONSchema::compile`). Read current docs; don't pattern-match.

### Alternatives considered

- `valico` — older, less active, maintenance uncertain.
- Validate manually in `serde::Deserialize` — fine for trivial cases, but loses the ability to ship a schema for LLM/editor consumption. Use schemars + jsonschema for the public boundary.

---

## 6. Workspace, Build, and Toolchain

### 6.1 Cargo workspace layout

The repo already has `crates/` with three placeholders. Recommended final shape:

```
happyterminals/
├── Cargo.toml              # workspace root
├── rust-toolchain.toml     # pin
├── crates/
│   ├── happyterminals-core/         # Signal/Effect/Memo/Grid/Pipeline. Depends on ratatui-core.
│   ├── happyterminals-renderer/     # 3D projection, mesh loading. Depends on core + glam + tobj + stl_io.
│   ├── happyterminals-compositor/   # tachyonfx integration, scene graph, JSON recipes. Depends on core + tachyonfx.
│   ├── happyterminals-dsl/          # (optional) Rust-side scene-builder macros. Depends on core + compositor.
│   ├── happyterminals/              # Convenience facade re-exporting the above.
│   └── happyterminals-py/           # PyO3 bindings. cdylib. Final-milestone crate.
├── examples/
│   └── spinning-cube/      # Milestone 1 exit demo.
└── xtask/                  # Build helpers (schema export, etc.) — optional.
```

Workspace `Cargo.toml` defines `[workspace.dependencies]` for every shared crate version so that `ratatui-core = "0.1"`, `glam = "0.32"`, etc. are pinned in exactly one place.

Use `[workspace.lints]` to enforce a baseline (deny `rust_2018_idioms`, deny `clippy::all`, allow `clippy::module_name_repetitions`).

### 6.2 `rust-toolchain.toml`

```toml
[toolchain]
channel = "1.86"
components = ["clippy", "rustfmt"]
profile = "minimal"
```

Pin the MSRV (1.86 to match ratatui 0.30 + pyo3 0.28). CI matrix should additionally test against `stable` (currently 1.94) so we catch breakage early. Confidence on Rust 1.94 being stable in April 2026: MEDIUM (web search, not verified against forge.rust-lang.org).

### 6.3 CI baseline

GitHub Actions (matches Rust ecosystem norm; project is OSS from day one):

- **Lint:** `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`
- **Test:** `cargo test --workspace --all-features`
- **MSRV:** `cargo +1.86 check --workspace`
- **Docs:** `cargo doc --workspace --no-deps` (with `RUSTDOCFLAGS=-D warnings`)
- **Snapshots:** `cargo insta test --workspace` with `--unreferenced=reject` to catch stale snapshots
- **Coverage (optional, post-MVP):** `cargo llvm-cov`
- **Python wheels (final milestone):** `maturin build` matrix across Linux/macOS/Windows × CPython 3.10–3.13 (use the official `maturin-action`).

For releases use `cargo-release` or `release-plz` — both are mature in 2026 and remove the manual version-bump-tag-publish dance.

---

## 7. Testing Strategy

### 7.1 Snapshot testing — `insta = "1.47.2"`

Confidence: HIGH

Snapshot testing is the **right** tool for terminal output. A test renders a scene to a `Grid`, converts to a known string representation (e.g. one line per row, ANSI stripped or `[FG:R,G,B]` markers), and `insta::assert_snapshot!` captures the expected output. Reviewing diffs becomes a normal git workflow.

Use `cargo install cargo-insta` for the interactive review TUI. Pattern:

```rust
#[test]
fn cube_at_zero_rotation_renders_silhouette() {
    let grid = render_cube(rotation: 0.0, size: (40, 20));
    insta::assert_snapshot!(grid_to_string(&grid));
}
```

For ANSI-colored output, snapshot the `Grid` cells directly (`assert_yaml_snapshot!`) so colors and attributes are part of the diff.

`expect-test` (1.5.1) is a smaller alternative (inline snapshots only, no separate file) — fine for a handful of unit tests, doesn't scale to the visual-regression suite happyterminals will want.

### 7.2 Property-based testing — `proptest = "1.11.0"`

Confidence: HIGH

Use proptest for the parts where invariants matter more than specific outputs:

- `Grid::compose(a, b).compose(c) == a.compose(Grid::compose(b, c))` — pipeline associativity
- Round-trip: `parse(serialize(recipe)) == recipe` for JSON scene recipes
- `project(unproject(p)) == p` (within float tolerance) — projection invertibility
- Signal / memo: changing an input changes only memos that transitively read it

`proptest` over `quickcheck` because of its shrinking quality and the standard Rust ecosystem choice.

### 7.3 TUI integration tests

Two patterns:

1. **Headless Terminal:** `ratatui::Terminal` constructed over a `TestBackend` (in-memory buffer). Drive a frame, assert the buffer state. This is the bread-and-butter integration test pattern in the ratatui ecosystem.
2. **End-to-end binary smoke:** `assert_cmd` + `assert_fs` to spawn a demo binary with `--frames 1 --output ascii` flags and compare the captured stdout via insta.

### 7.4 Benchmarks — `criterion = "0.8.2"`

Reach for criterion when a real "is the renderer fast enough at 60fps" question shows up — likely milestone 2 or 3. Don't pre-optimize in milestone 1.

---

## 8. Anti-Recommendations — Things NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `tui-rs` | The original crate, **deprecated since 2023.** ratatui is the community-maintained continuation. | `ratatui` 0.30 |
| `tui-vfx` (despite the vendored copy) | 8★, ~5 weeks old at project start, not adopted. Manifesto already supersedes the README on this. | `tachyonfx` 0.25 |
| `termion` | Unix-only. Project requires Windows Terminal support. | `crossterm` (via `ratatui-crossterm`) |
| `cgmath` | Last release 2021. Effectively unmaintained. | `glam` 0.32 |
| `pyo3-asyncio` | **Abandoned 2023, does not support pyo3 ≥ 0.21.** Most commonly recommended-by-mistake crate in the entire stack. | `pyo3-async-runtimes` 0.28 |
| `pyo3` 0.20 / 0.21 patterns (`&PyAny`, `IntoPy`, manual `Py::new`) | Pre-`Bound` API, deprecated through 0.27, still everywhere on Stack Overflow. | `Bound<'py, T>` API + `IntoPyObject` from current pyo3 docs |
| `schemars` 0.8 syntax | Pre-1.0 API; attributes and schema dialect changed. | `schemars` 1.x current docs |
| `jsonschema::JSONSchema::compile(...)` | Old API gone in the 0.4x line. | `jsonschema::validator_for(&schema)` |
| Putting `pyo3` in core/library crates | Forces every consumer to link libpython. Standard PyO3 anti-pattern. | Keep PyO3 isolated in `happyterminals-py` cdylib |
| Library crates depending on `ratatui` (the facade) | Couples library to widgets + a backend. Makes downstream users fight feature-flag wars. | `ratatui-core` only in libraries |
| Rolling our own reactive graph in milestone 1 | Subtle data structure with documented edge cases (cycle detection, glitch-freeness, batching). Burns time for no gain. | Wrap `reactive_graph` 0.2 |
| Forking voxcii into the tree | Already decided in `project.md` — re-implement fresh. Worth restating: **do not vendor voxcii** even "for reference". The vendored copy under `vendor/` is read-only inspiration. | Fresh implementation in `happyterminals-renderer` |
| `unsafe` in the Grid hot path before measuring | Premature optimization; ratatui's `Buffer` is already efficient. | Profile first; reach for `unsafe` only with criterion data justifying it. |

---

## 9. Version Compatibility Matrix

Critical pairings to lock in workspace dependencies:

| Crate A | Crate B | Constraint | Source |
|---------|---------|------------|--------|
| `tachyonfx 0.25` | `ratatui-core 0.1` | Hard requirement | tachyonfx Cargo.toml on crates.io |
| `ratatui 0.30` | `crossterm 0.29` | Default backend pairing | ratatui-crossterm 0.1 deps |
| `pyo3 0.28` | `pyo3-async-runtimes 0.28` | Same minor | pyo3-async-runtimes README |
| `pyo3 0.28` | `maturin 1.13+` | maturin auto-detects | Standard |
| `pyo3 0.28` | Rust ≥ 1.83 | MSRV | pyo3 CHANGELOG |
| `ratatui 0.30` | Rust ≥ 1.86 | MSRV | ratatui release highlights |
| `reactive_graph 0.2` | `any_spawner 0.3` | Required for effect scheduling | reactive_graph deps |
| `schemars 1.x` | JSON Schema draft 2020-12 default | API change from 0.8 | schemars 1.0 announcement |

**Workspace MSRV: 1.86** (max of all deps).

---

## 10. Installation — `Cargo.toml` Skeleton

Workspace root `Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
    "crates/happyterminals-core",
    "crates/happyterminals-renderer",
    "crates/happyterminals-compositor",
    "crates/happyterminals",
    # "crates/happyterminals-py",     # uncomment at final milestone
    "examples/spinning-cube",
]

[workspace.package]
edition = "2024"
rust-version = "1.86"
license = "MIT OR Apache-2.0"
repository = "https://github.com/<owner>/happyterminals"

[workspace.dependencies]
# ratatui ecosystem
ratatui          = { version = "0.30",  default-features = false }
ratatui-core     = "0.1"
ratatui-crossterm = "0.1"
ratatui-widgets  = "0.3"
ratatui-macros   = "0.7"
crossterm        = "0.29"
tachyonfx        = "0.25"

# reactivity
reactive_graph   = "0.2"
any_spawner      = "0.3"

# 3D
glam             = "0.32"
tobj             = "4.0"
stl_io           = "0.11"

# serde + schema
serde            = { version = "1", features = ["derive"] }
serde_json       = "1"
schemars         = "1.2"
jsonschema       = "0.46"

# errors / logs
thiserror        = "2"
color-eyre       = "0.6"
tracing          = "0.1"
tracing-subscriber = "0.3"

# misc utils chosen to align with tachyonfx
compact_str      = "0.9"
bon              = "3"
slotmap          = "1"

# dev
insta            = "1.47"
proptest         = "1.11"
criterion        = "0.8"
rstest           = "0.26"
```

### Library crate (`crates/happyterminals-core/Cargo.toml`):

```toml
[dependencies]
ratatui-core   = { workspace = true }
reactive_graph = { workspace = true }
any_spawner    = { workspace = true }
thiserror      = { workspace = true }
tracing        = { workspace = true }
serde          = { workspace = true, optional = true }

[features]
default = []
serde   = ["dep:serde"]
```

### App crate (`examples/spinning-cube/Cargo.toml`):

```toml
[dependencies]
happyterminals = { path = "../../crates/happyterminals" }
ratatui        = { workspace = true, features = ["crossterm"] }
crossterm      = { workspace = true }
color-eyre     = { workspace = true }
```

### Final-milestone Python crate (`crates/happyterminals-py/Cargo.toml`):

```toml
[lib]
name = "happyterminals"
crate-type = ["cdylib"]

[dependencies]
happyterminals = { path = "../happyterminals" }
pyo3 = { version = "0.28", features = ["extension-module", "abi3-py310"] }
pyo3-async-runtimes = { version = "0.28", features = ["tokio-runtime"] }
```

`abi3-py310` builds a single wheel that works across Python 3.10+, dramatically simplifying the release matrix. Verify this still applies for any features we end up needing.

---

## 11. Confidence Assessment

| Area | Confidence | Why |
|------|------------|-----|
| Ratatui 0.30 + sub-crate split | HIGH | crates.io API + ratatui.rs official highlights |
| tachyonfx 0.25 dep on ratatui-core | HIGH | crates.io dependency listing |
| `reactive_graph` as the right reactivity choice | HIGH | docs.rs description + crates.io stats + match with project's stated SolidJS lineage |
| `glam` as the 2026 default for graphics math | HIGH | crates.io stats + ecosystem (Bevy/wgpu) usage + stale cgmath comparison |
| `tobj` for OBJ | HIGH | de-facto standard, 1.78M downloads, recent release |
| `stl_io` for STL | HIGH | active, 2.84M downloads, recent release |
| `pyo3 0.28` + `pyo3-async-runtimes 0.28` | HIGH | crates.io + GitHub README + PyO3 changelog cross-checked |
| `pyo3-asyncio` is abandoned | HIGH | last release 2023-11-11 verified against crates.io API; `pyo3-async-runtimes` README explicitly states it's the fork |
| `schemars` 1.x API shift | MEDIUM | Web search; surface API not exhaustively verified against current docs |
| `jsonschema` 0.46 surface API | MEDIUM | Same — read current docs when implementing |
| Rust 1.94 being current stable in April 2026 | MEDIUM | Web search only; not verified against forge.rust-lang.org |
| tachyonfx `ratatui-next-cell` feature flag for ratatui ≥ 0.30 | MEDIUM | Web search only; verify by reading tachyonfx Cargo.toml when wiring milestone 1 |
| `reaktiv` not yet ready to bet on | MEDIUM | Single source (its README + 0.1.1 version on crates.io) |

---

## 12. Implications for the Roadmap

(Pulled forward to `SUMMARY.md` for the roadmapper; restated here for completeness.)

1. **Milestone 1 dependency floor is large but well-trodden.** The spinning cube demo realistically needs: `ratatui-core`, `ratatui` (in the example), `crossterm`, `tachyonfx`, `reactive_graph`, `any_spawner`, `glam`, `color-eyre`. That's a ~10-crate stack but every crate is well-maintained and the integrations are documented.
2. **The reactivity wrapper deserves its own early sub-milestone.** Wrapping `reactive_graph` cleanly so we never expose Leptos identifiers in `happyterminals_core`'s public API is design work that should happen before the first signal-driven demo, not after.
3. **PyO3 milestone is best treated as `(N + 2)` in scope, not `N`.** The crate must exist as a separate cdylib, the public API of the Rust side must already be Bound-compatible (no Leptos-flavored types leaking out), and the schema for JSON recipes must be stable so Python users don't catch breaking changes via the JSON path.
4. **`schemars` and `jsonschema` are 1.x/0.4x territory** — anything an LLM (or our own training-era memory) suggests for these will likely be wrong. Treat the JSON-recipe milestone as needing dedicated docs research.
5. **Workspace layout matters from milestone 1.** Once `happyterminals-core` has even one external user, splitting out `happyterminals-py` later is much harder than getting the layering right now.

---

## Sources

### Primary (verified live, 2026-04-14)

- crates.io API responses for: `ratatui`, `ratatui-core`, `ratatui-widgets`, `ratatui-crossterm`, `ratatui-macros`, `crossterm`, `tachyonfx`, `pyo3`, `pyo3-async-runtimes`, `pyo3-asyncio`, `maturin`, `glam`, `nalgebra`, `cgmath`, `bevy_math`, `tobj`, `stl_io`, `serde_json`, `schemars`, `jsonschema`, `insta`, `proptest`, `criterion`, `thiserror`, `anyhow`, `color-eyre`, `tracing`, `tokio`, `rayon`, `image`, `reactive_graph`, `reactive_stores`, `leptos`, `leptos_reactive`, `any_spawner`, `futures-signals`, `compact_str`, `bon`, `micromath`, `anpa`, `ratatui-image`, `snapbox`, `expect-test`, `rstest`
- crates.io dependency listings for: `tachyonfx 0.25.0`, `reactive_graph 0.2.13`, `ratatui 0.30.0`, `pyo3 0.28.3`

### Secondary (web)

- https://ratatui.rs/highlights/v030/ — ratatui 0.30 release highlights (HIGH)
- https://docs.rs/reactive_graph/latest/reactive_graph/ — reactive_graph documentation (HIGH)
- https://github.com/PyO3/pyo3-async-runtimes — pyo3-async-runtimes README (HIGH)
- https://raw.githubusercontent.com/PyO3/pyo3/main/CHANGELOG.md — PyO3 CHANGELOG (HIGH)
- https://github.com/pascalkuthe/reaktiv — reaktiv README (MEDIUM, single source)
- https://github.com/ratatui/tachyonfx — tachyonfx README (for `ratatui-next-cell` feature flag claim, MEDIUM)
- https://book.leptos.dev/appendix_reactive_graph.html — Leptos reactive system (HIGH)
- https://blog.rust-lang.org/releases/ — Rust release notes index (MEDIUM, used for current-stable claim only)

### Not consulted (intentionally)

- Context7 was not queried for this round because the registry-API path gave us the primary-source version data we needed. For depth-of-API questions during implementation (e.g. "what's the exact `Validator::new` signature in `jsonschema 0.46`"), Context7 should be the first stop.

---

*Stack research for: declarative reactive TUI framework with ASCII 3D + effects + Python bindings*
*Researched: 2026-04-14*
