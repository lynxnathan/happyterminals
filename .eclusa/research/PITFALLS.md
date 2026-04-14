# Pitfalls Research — happyterminals

**Domain:** Reactive terminal scene manager (Rust core + tachyonfx + ASCII 3D + PyO3 bindings, public OSS day one)
**Researched:** 2026-04-14
**Confidence:** HIGH on portability, reactive, ASCII-3D, OSS hygiene; MEDIUM-HIGH on tachyonfx/PyO3 specifics (current crates verified, but version-pinning matters)

Phase legend used throughout:
- **early** = architecture-affecting, must be settled before broad implementation (Milestone 1 / spinning cube)
- **mid**  = surfaces during implementation of a layer (renderer, compositor, scene graph, recipes)
- **late** = pre-release polish (docs, packaging, CI matrix, Python milestone)

---

## Critical Pitfalls

### Pitfall 1: Terminal left in raw mode / alt screen after panic

**What goes wrong:**
A panic anywhere in the render loop (3D math, OBJ parser, signal cycle, tachyonfx effect) leaves the user's terminal in an unusable state: no echo, cursor invisible, alt-screen still active, mouse capture still on. Users have to blindly type `reset`. Bug reports flood in for "your library broke my shell".

**Why it happens:**
crossterm's `enable_raw_mode` / `EnterAlternateScreen` are not RAII-bound by default. The cleanup is done by application code in the happy path; a panic skips it. This is a known ratatui/crossterm gotcha — raw mode is not auto-disabled on panic.

**How to avoid:**
- Install a panic hook in `happyterminals::run()` that restores the terminal *before* the default handler runs (disable raw mode, leave alt screen, show cursor, disable mouse, flush).
- Wrap the `(raw, alt)` lifecycle in a guard struct whose `Drop` impl restores the terminal — covers both panic-via-unwind and early `?` returns.
- For the Python binding, the panic hook must also catch Rust panics that would otherwise propagate as `PanicException` and still leave the TTY trashed.

**Warning signs:**
- A bare `enable_raw_mode()` call in any example without a matching guard.
- Demos that run fine but require `stty sane` after Ctrl-C.
- No integration test that deliberately panics mid-render and asserts the TTY is sane.

**Phase to address:** **early** (the spinning-cube demo must already use the guard).

---

### Pitfall 2: README/docs drift from actual stack ("tui-vfx vs tachyonfx")

**What goes wrong:**
Public-facing docs claim a different stack than the code uses. New contributors waste hours wiring up the wrong dependency; LLM-generated PRs reference non-existent APIs; users on crates.io find a mismatched description.

**Why it happens:**
Project pivoted from tui-vfx (8★, 5 weeks old) to tachyonfx (1,182★, official ratatui-org). README.md, project.md, and the `happyterminals-compositor` README were written before the pivot and still mention tui-vfx as a core dep. The vendored `vendor/tui-vfx/` reinforces the confusion.

**How to avoid:**
- Single-source-of-truth file (`.eclusa/PROJECT.md` is already this) with a CI doc-lint step that greps READMEs for forbidden strings (`tui-vfx`, `voxcii-core` as a dep name, `Haskell bindings`).
- Move `vendor/tui-vfx/` to `vendor/_reference/tui-vfx/` and add a `README` explaining "reference reading only, not a dep".
- README.md must be rewritten in the very first implementation phase, not deferred.

**Warning signs:**
- `grep -r tui-vfx` returns hits outside `vendor/` and `CHANGELOG`.
- Cargo.toml `[dependencies]` and README "Stack" section don't match.
- Any decision rationale in PROJECT.md not yet reflected in user-facing copy.

**Phase to address:** **early** (Phase 0 cleanup before any new feature lands).

---

### Pitfall 3: Stub crates with real dependencies before any code exists

**What goes wrong:**
The workspace has three placeholder crates that already depend on `ratatui`, `tui-vfx`, `pyo3`. `cargo check` emits `unused_imports`/`dead_code`/`unused_extern_crates` warnings on every build. CI fails on `-D warnings`. New contributors can't tell which crates are real and which are scaffolding. Worse: the wrong dep (tui-vfx) is locked into Cargo.lock from day zero.

**Why it happens:**
Scaffolding pattern of "declare the shape of the workspace upfront." Reasonable for solo work; toxic for OSS because every clone produces noise and the lockfile commits to choices not yet made.

**How to avoid:**
- Stub crates should have **no** dependencies until code that uses them exists. A `lib.rs` with `// placeholder` is fine; a `[dependencies]` table with three crates is not.
- Delete `tui-vfx` from every `Cargo.toml` immediately. Add `tachyonfx` only when the compositor crate has the first real call site.
- Add `pyo3` only in the dedicated `happyterminals-py` crate when Phase N (Python milestone) starts — not in `happyterminals-core`.
- Run `cargo +nightly udeps` (or `cargo machete`) in CI to keep deps honest.

**Warning signs:**
- `cargo build` produces `warning: unused crate dependency`.
- A crate's `src/lib.rs` is one line but `Cargo.toml` has 3+ deps.
- Cargo.lock contains crates that no `use` statement in the workspace references.

**Phase to address:** **early** (must be done before Milestone 1 implementation begins, otherwise lockfile churn pollutes the entire history).

---

### Pitfall 4: Reactive system memory leaks via dangling effects

**What goes wrong:**
Effects created inside other effects (or scene transitions) are never disposed when their parent re-runs. Each scene change leaks effect closures, which keep references to old grids, old meshes, old ratatui buffers. After a few minutes of demo, RSS grows linearly; after a few hours of a long-running TUI app it OOMs.

**Why it happens:**
SolidJS's owner-tree model exists precisely to fix this — every computation registers under the owner that created it, and disposing the owner disposes its children. Naive Rust ports often skip the owner tree because `Rc<RefCell<>>` graphs are awkward to model in Rust. Without it, child computations subscribed to outer signals never unsubscribe, the signal keeps re-running them, and they keep their captured environment alive forever.

**How to avoid:**
- Implement **owner scopes** as a first-class concept in the reactive core, not an afterthought. Every `create_effect` / `create_memo` registers under the current owner; disposing the owner runs `onCleanup` callbacks and unsubscribes from all signals.
- API mirror: provide `create_root` (returns a `Dispose` handle) and require it for any non-test entry point.
- For the renderer's scene-transition manager, ensure scene B's owner replaces scene A's, then dispose A — this is the main loss vector for a scene manager specifically.
- Add a long-running memory test: 10k scene transitions, RSS must not grow more than N MB.

**Warning signs:**
- No `Owner` / `Scope` / `Root` type in the reactive crate.
- `create_effect` doesn't take or capture a parent context.
- Examples never call a `dispose` function.
- `valgrind --tool=massif` shows linear growth in `Rc::new` allocations.

**Phase to address:** **early** (the reactive primitives are foundational; retrofitting an owner tree later is a rewrite).

Source: [SolidJS pain points and pitfalls](https://vladislav-lipatov.medium.com/solidjs-pain-points-and-pitfalls-a693f62fcb4c), [Lifetime of a signal](https://github.com/solidjs/solid/discussions/1776).

---

### Pitfall 5: Reactive over-firing (thundering herd) and under-firing (stale reads)

**What goes wrong:**
- **Over-fire:** One signal change triggers a cascade because effects update other signals which trigger other effects within the same tick. The render loop runs the same effect 5–50 times per frame. CPU pegs.
- **Under-fire:** A "batch" boundary collapses several writes into one notification, but a read between the writes returns the old value, so a memo computes from inconsistent inputs.

**Why it happens:**
The reactive system has no notion of a transaction / batch / propagation phase. Writes notify synchronously, effects run synchronously, mid-cascade reads see partial state. Or the opposite: writes are deferred but reads aren't, so reads lie.

**How to avoid:**
- Two-phase propagation: writes mark dependents dirty; a single drain at the end of the tick re-runs them in topological order. Memos lazily recompute on read.
- Provide a `batch(|| { ... })` API that defers notification until the closure exits.
- Detect cycles in the dependency graph at runtime (effect re-enters its own dirty set) and panic with a clear "signal cycle detected: A → B → A" message — silent infinite loops are the worst failure mode.
- Add an internal `effect_runs_per_tick` counter exposed via a debug flag for users to spot pathological scenes.

**Warning signs:**
- An effect runs more than once per `tick()` call in tests.
- Memos compute even when no input signal changed.
- The framework hangs (no panic, no log) on a scene with mutually-dependent signals.

**Phase to address:** **early** (semantics of propagation are public API and cannot change later without breaking every user's scene).

---

### Pitfall 6: Send/Sync ambiguity for the reactive core

**What goes wrong:**
First version uses `Rc<RefCell<>>` because it's simple. Then the user wants to drive a signal from a tokio task ("tick from a websocket"), or from the Python `asyncio` loop on a different thread, and everything is `!Send`. Retrofit to `Arc<Mutex<>>` requires touching every API and every example. Or worse: ship `Arc<Mutex<>>` without thinking and pay the contention cost on every cell-level read in the hot path.

**Why it happens:**
"Should the reactive graph be thread-safe?" is a foundational question with no obviously-right answer for a TUI:
- Single-threaded `Rc` is fastest and matches SolidJS exactly, but limits async integration.
- Multi-threaded `Arc<Mutex>` is flexible but slower and complicates owner trees.

**How to avoid:**
- Decide explicitly and document: **single-threaded reactive core, run the loop on one OS thread, post messages from other threads via a channel** (this is what most UI frameworks do, including Slint and Dioxus). Document this prominently — Python users especially will assume cross-thread signal writes work.
- Provide a `Runtime::send_handle()` that returns a `Send`-able producer that posts to an mpsc; the reactive loop drains it each tick. This keeps the hot path cheap and gives async-shaped users an answer.
- For PyO3: bind signal writes from Python through this same channel, never directly. Document that holding a Python `Signal` reference across an `await` is fine; mutating it from a different OS thread requires the channel.

**Warning signs:**
- `Signal<T>` is `Sync` but uses interior mutability without atomic ordering.
- `tokio::spawn(async move { signal.set(...) })` compiles but produces racy renders.
- No public guidance on threading in the README.

**Phase to address:** **early** (this is a public-API tattoo; pick once).

---

### Pitfall 7: Unicode width math broken for emoji / CJK / combining marks

**What goes wrong:**
A single user-perceived character (家, 🤦🏼‍♂️, é as `e` + combining acute) takes 1, 2, or even 7 codepoints, has display width 0, 1, or 2, and may or may not occupy two terminal cells. If the renderer measures width wrong, every column after that emoji is offset by one cell — 3D scenes warp, effects mis-position, alignment breaks. Worse, the bug only appears when users include emoji in their text overlays.

**Why it happens:**
- `str::len()` counts bytes (wrong).
- `chars().count()` counts codepoints (still wrong — combining marks).
- `unicode-width` gives codepoint width (closer, but doesn't handle ZWJ-joined emoji correctly).
- Even *correct* width depends on the terminal's Unicode version; older terminals render new emoji as two single-width glyphs.

**How to avoid:**
- Always operate on **grapheme clusters** (`unicode-segmentation`) and compute width per-cluster, not per-codepoint. Consider `unicode-display-width` or `runefix-core` which handle ZWJ sequences and emoji presentation per Unicode 15.1.
- Define the `Cell` type as holding *one grapheme cluster + a width field (1 or 2)*. Wide cells occupy a "ghost" cell to their right that renders as nothing — same model as tmux, kitty, ghostty.
- Provide a single `Grid::put_str(x, y, &str)` that handles segmentation and width internally; *no* code outside that function should compute string widths.
- Add property tests: random-grapheme insertion never desynchronizes the cursor column.

**Warning signs:**
- Grid API takes `char` instead of `&str` for cell content.
- Any use of `s.len()` or `s.chars().count()` to compute layout.
- Demos use ASCII-only text — emoji never tested.
- Width helpers depend solely on `unicode-width` without grapheme segmentation.

**Phase to address:** **early** (Grid is a foundational type; changing the cell shape later is invasive).

Source: [unicode-width docs](https://docs.rs/unicode-width/), [unicode-display-width](https://lib.rs/crates/unicode-display-width), [runefix-core](https://crates.io/crates/runefix-core), [It's not wrong that "🤦🏼‍♂️".length == 7](https://hsivonen.fi/string-length/), [Grapheme Clusters and Terminal Emulators (Mitchell Hashimoto)](https://mitchellh.com/writing/grapheme-clusters-in-terminals).

---

### Pitfall 8: Color regression — true-color assumed everywhere

**What goes wrong:**
Library emits 24-bit `\e[38;2;R;G;Bm` sequences unconditionally. Looks great in iTerm2; in tmux without `Tc`/`RGB` capability it renders as random 8-color palette mappings; on a real VT220 over serial it dumps escape literals. Users blame the framework.

**Why it happens:**
- `$COLORTERM=truecolor` is the de-facto signal but isn't always set (e.g., inside tmux unless overridden).
- tmux requires `terminal-overrides ',*256col*:Tc'` or proper `RGB` terminfo to pass true-color through.
- Some terminals (older Windows conhost, old PuTTY) report 256 colors but mishandle them.
- Many libraries detect once at startup and never re-check after `SIGWINCH`/reattach.

**How to avoid:**
- Implement a **color-mode pipeline**: `Cell` stores RGB; output stage downsamples to 256 / 16 / monochrome based on detected capability.
- Detection order: explicit user override → `$COLORTERM` → terminfo (`tput colors`, `RGB` cap) → conservative default (256-color).
- Provide `--force-color=truecolor|256|16|none` env var and CLI flag for both the Rust binary and Python `run()`.
- Test matrix in CI: render same scene at all four color depths; snapshot each.
- Document tmux: provide a copy-paste `tmux.conf` snippet for users.

**Warning signs:**
- The output layer hardcodes `\e[38;2;...m` strings.
- No code path exists for 16-color or monochrome.
- `COLORTERM` checked once and cached.
- Demo over SSH-into-tmux looks visibly broken.

**Phase to address:** **mid** (becomes critical when the compositor lands; pre-release blocker).

Source: [termstandard/colors](https://github.com/termstandard/colors), [Adding 24-bit TrueColor RGB to tmux](https://sunaku.github.io/tmux-24bit-color.html), [tmux truecolor terminfo issues](https://github.com/tmux/tmux/issues/1236).

---

### Pitfall 9: tmux/screen DCS passthrough breakage for advanced sequences

**What goes wrong:**
Anything beyond plain SGR — kitty graphics, sixel, OSC 52 clipboard, OSC 8 hyperlinks, custom DCS — is *eaten by tmux* unless wrapped in a `\ePtmux;\e<sequence>\e\\` envelope. Library that emits these sequences directly looks fine outside tmux, broken inside.

**Why it happens:**
tmux is itself a terminal emulator with its own state machine. Sequences it doesn't recognize get dropped (or worse, partially interpreted). `allow-passthrough` (tmux ≥ 3.3) is required and not on by default.

**How to avoid:**
- The output layer must detect tmux (`$TMUX` set) and wrap any non-SGR DCS/OSC sequences in the passthrough envelope, doubling embedded `\e`.
- For happyterminals' MVP: ASCII + ANSI SGR is the universe, so most of this is moot — *but* the moment anyone wants OSC 52 (paste demos) or hyperlinks (URLs in scenes), it bites.
- Document explicit non-support: "kitty graphics, sixel, OSC 8 are not part of v0; they violate the universal-terminal principle."
- Add a "this requires `allow-passthrough on`" warning at startup if tmux is detected and any advanced feature is requested.

**Warning signs:**
- OSC/DCS sequence emitted unwrapped under `$TMUX`.
- Any feature beyond pure SGR present without a tmux test.
- "Works in iTerm2 but not in my tmux" issues filed.

**Phase to address:** **mid** (only matters if/when non-SGR sequences are used; gate the feature on detection).

Source: [tmux FAQ](https://github.com/tmux/tmux/wiki/FAQ), [tmux allow-passthrough](https://tmuxai.dev/tmux-allow-passthrough/).

---

### Pitfall 10: Terminal resize race — rendering against stale dimensions

**What goes wrong:**
User resizes window mid-frame. The render code computed cell positions against 80×24, the terminal is now 120×40, half the scene draws into the void or wraps catastrophically. On Windows Terminal, the resize signal arrives differently; on tmux-attached sessions, it's relayed with a delay.

**Why it happens:**
SIGWINCH is asynchronous. Reading the new size, reallocating the Grid, re-projecting 3D scenes, and re-running effects all take time. Without explicit ordering, you can be halfway through a frame when the resize fires.

**How to avoid:**
- Single render loop, single frame state. Resize events drain into a queue and apply *between* frames, never during.
- After every resize: clear the alt-screen buffer, re-query `crossterm::terminal::size()` (don't trust the SIGWINCH payload), reallocate grids, mark all signals dirty, redraw fully.
- Debounce rapid resizes (e.g., during continuous drag) — coalesce to 1 resize per 16ms.
- For the 3D renderer, the projection matrix is a function of viewport; recompute it after resize.
- Test on Windows Terminal specifically (its resize semantics differ; it can also report 0×0 momentarily).

**Warning signs:**
- Render code reads `terminal::size()` more than once per frame.
- No "after resize" hook.
- Demos visibly tear when resized rapidly.

**Phase to address:** **mid** (during ratatui backend integration / event loop work).

---

### Pitfall 11: Per-frame string allocation churn (the GC-less stutter)

**What goes wrong:**
Render loop allocates `String`s per cell or per row (`format!("\x1b[{};{}H{}", ...)`). At 60fps × 80×24 cells = 115k allocations/s. Heap fragments, jitter spikes, the cube "stutters." On Python via PyO3, each call across the boundary additionally allocates.

**Why it happens:**
"Just format the string" is the path of least resistance. `format!` allocates. So does `to_string()`, `String::from()`, and most `Display` impls.

**How to avoid:**
- Render into a single reused `Vec<u8>` per frame (or two double-buffered).
- Use `write!` with a borrow into the buffer, never `format!`.
- Pre-compute SGR escape strings for the 256 most common (fg, bg, attr) tuples and cache them as `&'static [u8]`.
- Diff against the previous frame; only emit cursor-move + cell content for changed cells (this is the "fine-grained" promise — make it real).
- Use `criterion` benchmarks early; alloc count regressions are commit-blocking.

**Warning signs:**
- `format!` or `.to_string()` inside per-cell loops.
- `dhat` / `heaptrack` profile shows millions of allocations per render.
- Frame time variance > 2× the median.

**Phase to address:** **mid** (during the ratatui backend / pipeline executor work). Mark this in the renderer phase explicitly as a benchmark gate.

---

### Pitfall 12: Full-buffer redraw — undermines the entire reactivity premise

**What goes wrong:**
Despite all the SolidJS-style signal plumbing, the render path calls `terminal.clear()` + redraw every frame. The fine-grained reactivity is theatre — users feel no perf benefit over a polling-based TUI. The whole architectural justification collapses.

**Why it happens:**
It's easier to wire "write the whole grid on every tick" than to (a) track which cells changed because a signal changed, (b) translate that to a minimal sequence of cursor moves + writes, and (c) handle wide cells / overdraw correctly.

**How to avoid:**
- Make the diff stage a first-class component: `Grid` + `previous Grid` → `Vec<DirtySpan>`. Output stage emits cursor-positioned writes only for dirty spans.
- Tie the dirty-tracking to the signal graph: when signal S changes, the renderer knows which Grid regions S contributed to (via the effect that wrote them). Mark only those dirty.
- Benchmark: a scene where one cell changes per frame must produce roughly 1 cursor-move + 1 cell-write of output, not a full repaint.
- This is the core thesis. Phase exit criterion for Milestone 1 should explicitly verify it via a "change one signal, count bytes written to TTY" test.

**Warning signs:**
- Output writer's per-frame byte count is constant regardless of what changed.
- `tput cup ...` (or `\e[H`) appears once per frame at position (0,0) followed by a full-grid dump.
- The "spinning cube" demo writes 80×24×ANSI bytes each frame.

**Phase to address:** **early** (the architecture must be diff-based from Milestone 1 — retrofitting later means re-doing the renderer).

---

### Pitfall 13: Z-fighting and depth precision in the ASCII rasterizer

**What goes wrong:**
Two triangles at nearly the same depth flicker between frames as the depth comparison swings. Camera dolly causes shimmering bands. With ASCII characters this is *more* visible than pixels because each cell is huge.

**Why it happens:**
- Standard perspective z-buffer concentrates precision near the near plane and wastes it far from the camera (`1/z` is non-linear).
- Floating-point `f32` z-buffer with poorly-chosen near/far gives only millimeter precision at scene scale.
- Two coplanar tris (e.g., shared edges from non-triangulated quads) tie on depth; tiebreak depends on draw order.

**How to avoid:**
- Use `f32` z-buffer with **reversed-Z** (store `1 - depth`, near=1, far=0) — distributes precision more evenly.
- Pick near/far conservatively: scene-bounding-box driven, not hardcoded `0.1..1000.0`.
- Triangulate at load time (OBJ allows quads — split into two tris with consistent winding).
- For coplanar surfaces, apply a small depth bias (polygon offset analogue).
- Render at slightly higher internal resolution than terminal cells (4× supersampling per cell), then downsample with majority-vote — smooths the staircase and hides depth ties.

**Warning signs:**
- `cargo run --example cube` shows visible flicker on faces.
- Z-buffer is hardcoded `f32::INFINITY` initialized with `0.1..100.0` near/far.
- OBJ loader treats faces with 4 vertices as a single primitive.

**Phase to address:** **mid** (during 3D renderer phase). Add a "spinning cube must not flicker" visual snapshot test.

Source: [ASCII characters are not pixels](https://alexharri.com/blog/ascii-rendering), [Z-buffer rendering article](https://waspdev.com/articles/2025-05-09/the-power-of-z-buffer).

---

### Pitfall 14: Character aspect ratio (cells are ~2:1 tall)

**What goes wrong:**
A "cube" rendered as if cells were square comes out as a tall rectangle. Spheres look like eggs. The ASCII rendering looks unprofessional and "off" even when the math is correct.

**Why it happens:**
Terminal cells are roughly 2:1 (height:width) in most fonts. Treating them as square in projection math causes vertical stretch.

**How to avoid:**
- The projection matrix takes a `cell_aspect` parameter (default ~2.0) and stretches X by that factor (or compresses Y).
- Make it user-configurable — some fonts (Pragmata Pro, Iosevka with line-height adjustments) are closer to 1.6:1.
- Document this in the camera/projection API as the *first* paragraph.
- Provide a `Camera::auto_aspect()` that queries the OSC sequence for cell pixel dimensions if the terminal supports it (most don't), else uses 2.0.

**Warning signs:**
- The first cube demo looks visibly tall.
- No `aspect_ratio` parameter anywhere in the renderer API.
- Math papers / GLSL shaders ported directly without aspect adjustment.

**Phase to address:** **early** (in the renderer phase; it's a public-API parameter that affects every example).

Source: [ASCII characters are not pixels (HN discussion)](https://news.ycombinator.com/item?id=46657122).

---

### Pitfall 15: OBJ/STL loader brittleness

**What goes wrong:**
User downloads a free `.obj` from the internet, library panics on unwrap, or silently renders a featureless blob, or rotates the model 180° because winding order was assumed CCW when it was CW. STL ASCII vs binary autodetection picks wrong on edge files.

**Why it happens:**
- OBJ has dozens of optional features: groups, smoothing groups, materials, multiple UV sets, n-gons, negative indices, comments, line continuations.
- STL files come in ASCII and binary; some binary STLs start with `solid ` (the ASCII magic), causing detection to fail.
- Real-world meshes have degenerate triangles, missing normals, non-manifold edges, NaN vertices.

**How to avoid:**
- Don't write a parser from scratch — use `obj` / `tobj` (well-tested) and `stl_io` / `mesh-loader`. Audit what they don't handle and document it.
- On load: triangulate n-gons, generate flat normals if missing, drop degenerate triangles (zero-area), normalize winding to CCW, validate finite floats.
- Return `Result<Mesh, MeshError>` with structured errors, never panic.
- Ship a small fixture set covering: quads, missing normals, ASCII STL, binary STL, large mesh, malformed mesh — each has a regression test.

**Warning signs:**
- Bespoke loader with `.unwrap()` on lines.
- No fixture file with quads or n-gons in tests.
- Loader returns `Mesh` not `Result<Mesh, _>`.

**Phase to address:** **mid** (in renderer phase, when mesh loading lands).

---

### Pitfall 16: tachyonfx integration — stateful effect lifecycle mismatches

**What goes wrong:**
tachyonfx effects are **stateful** — created once, processed every frame with elapsed time. Naive integrations create a fresh effect each frame ("it's just a function, right?") — animations never advance, or restart every tick. Conversely, holding onto a finished effect blocks the pipeline forever because tachyonfx effects have a `done()` state that must drive the next phase.

**Why it happens:**
Effects look like pure functions in the DSL (`fx::dissolve(seed)`) but carry mutable state internally. Their position in the lifecycle (`pending → in_progress → done`) is not always obvious from API shape.

**How to avoid:**
- happyterminals' `Effect` (the reactive primitive) and tachyonfx's `Effect` (the visual one) are different — **rename one of them** before any code is written. Suggested: tachyonfx ones become `Fx` or `VisualEffect` in our public API.
- The Pipeline holds tachyonfx effect *instances*, not constructors. Reactive signals can drive *parameters* but the instance lives across frames.
- When a tachyonfx effect's `done()` returns true: pipeline pops it, fires an `on_complete` callback, lets the next stage proceed. This is how transitions chain.
- Frame timing: pass real `Duration` since last frame, not nominal `1/fps`. Pause-friendly.
- Verify: a `fade_in(2s)` effect actually takes 2 seconds in a smoke test.

**Warning signs:**
- Effects re-instantiated inside the render closure.
- `process_effects(Duration::from_millis(16))` hardcoded regardless of actual elapsed time.
- Two types named `Effect` in scope without disambiguation.
- Animations "stuck" or "never start."

**Phase to address:** **early** (the naming clash is API-shaping; lifecycle model is in Milestone 1).

Source: [tachyonfx docs](https://docs.rs/tachyonfx), [tachyonfx on ratatui.rs](https://ratatui.rs/ecosystem/tachyonfx/).

---

### Pitfall 17: tachyonfx WASM vs native divergence

**What goes wrong:**
The browser-based tachyonfx editor / WASM target uses different timing, different color handling, different threading. Effects authored in the browser editor look subtly different in the native terminal. JSON recipes generated in one environment misbehave in the other.

**Why it happens:**
WASM has no real threads (without SharedArrayBuffer), no `Instant::now()` (uses `performance.now()`), and a different color compositing model when rendered to canvas vs to ANSI.

**How to avoid:**
- happyterminals targets native first; do **not** ship WASM in Milestone 1.
- If/when WASM is added (post-Python milestone), gate it behind a `wasm` feature flag and keep the recipe schema identical between targets.
- For recipes from the tachyonfx web editor: provide an importer that snapshots the version of tachyonfx the recipe was authored against and warns on schema drift.
- Treat the browser editor as an inspirational tool for now, not part of the integration contract.

**Warning signs:**
- `wasm32` in CI matrix without a clear reason.
- Recipe import path with no version field.
- "Looks different in browser vs terminal" issues.

**Phase to address:** **late** (only if WASM is ever pursued; otherwise out of scope).

---

### Pitfall 18: PyO3 — `pyo3-asyncio` is deprecated; use `pyo3-async-runtimes`

**What goes wrong:**
Project picks `pyo3-asyncio` because it's well-known. It's been moved/deprecated; the active fork is `pyo3-async-runtimes`. Six months later, a PyO3 minor version bump breaks the build, the upstream crate is unmaintained, and the migration is non-trivial because async semantics changed.

**Why it happens:**
The original `pyo3-asyncio` crate hasn't kept pace with PyO3 0.21+. The PyO3 org forked it as `pyo3-async-runtimes` and that's where development happens.

**How to avoid:**
- Use **`pyo3-async-runtimes`** from day one. Pin to a recent version compatible with the chosen `pyo3` version.
- Match runtime explicitly: `pyo3-async-runtimes` with `tokio` feature if happyterminals uses tokio (it probably shouldn't — see Pitfall 6 / single-threaded model).
- Document the architecture: Python owns the main thread (asyncio), Rust runs its loops in background threads, signal writes cross via a channel.
- Guard against future churn by pinning the dep with a tilde range and including a clear migration note in CHANGELOG.

**Warning signs:**
- `pyo3-asyncio` in `Cargo.toml`.
- `crates.io` page for the chosen async crate shows last commit > 12 months old.

**Phase to address:** **early** (architecture-affecting even before Phase N — the threading model decision is influenced by this).

Source: [pyo3-async-runtimes (active fork)](https://github.com/PyO3/pyo3-async-runtimes), [crates.io pyo3-asyncio (deprecated)](https://crates.io/crates/pyo3-asyncio).

---

### Pitfall 19: PyO3 GIL contention in the render loop

**What goes wrong:**
Rust render loop is called from Python; each frame acquires the GIL to read signal values written from Python, holds it for the entire render, then releases. Other Python tasks (asyncio I/O, websocket handlers) starve. Conversely, releasing the GIL too aggressively while reading Python objects causes UB.

**Why it happens:**
PyO3's API forces explicit thinking about the GIL, but examples often hold `Python<'_>` tokens for too long because it's syntactically convenient.

**How to avoid:**
- The render loop runs in Rust **without** the GIL. Signal values are mirrored into a Rust-side cache; Python writes go through the channel (Pitfall 6) and the render loop reads from the cache.
- Only acquire GIL when calling user-provided Python callbacks; minimize their scope with `Python::with_gil(|py| { ... })` blocks as small as possible.
- `allow_threads` around any blocking work (file I/O, sleeps).
- Be aware: as of 2026 there is no clean way to release the GIL across an `await` point; design async APIs to do async work in Rust and only cross to Python at boundaries.

**Warning signs:**
- A `Python<'_>` token threaded through the renderer.
- Python `asyncio` tasks visibly stutter while a scene runs.
- Benchmarks show similar speed with/without `allow_threads`.

**Phase to address:** **mid** (Python milestone), but **architecture decided early** (signal channel, cache).

Source: [pyo3-async-runtimes README](https://github.com/PyO3/pyo3-async-runtimes/blob/main/README.md), [PyO3 async-await docs](https://pyo3.rs/v0.23.4/async-await).

---

### Pitfall 20: PyO3 zero-copy hazards

**What goes wrong:**
"Zero-copy" Grid sharing between Rust and Python: Rust hands a `memoryview` over its grid bytes to Python. Python keeps the view across a tick during which Rust resizes the grid. Reallocation moves the buffer. Python now holds a dangling pointer. Segfault, or worse, silent data corruption.

**Why it happens:**
Buffer protocol exposes raw memory. Lifetime tracking across the FFI boundary is not enforced by Rust's borrow checker — Python doesn't know about Rust lifetimes.

**How to avoid:**
- Default to **copy** semantics for cross-boundary data. Make zero-copy opt-in with explicit `freeze()` / `lock()` semantics that pin the buffer for the duration of a Python context.
- For images / large arrays: use `numpy::PyArray` with explicit ownership transfer or a `bytes`-like wrapper that holds a `PyBuffer` whose lifetime is enforced.
- Document the pinned/movable model in the Python API reference.
- Stress test: 1M iterations of "create memoryview, hold across resize" must not segfault.

**Warning signs:**
- API hands raw `*mut u8` to Python.
- "Zero-copy" claimed without a pin/lock mechanism.
- Examples store buffer views in long-lived Python variables.

**Phase to address:** **mid** (Python milestone — but design the Grid type early to support pinning).

---

### Pitfall 21: PyPI / crates.io naming collisions and squatting

**What goes wrong:**
Project commits to `happyterminals` on crates.io, then discovers `happy-terminals` (with hyphen) is taken on PyPI by an unrelated project, or vice versa. Both registries treat hyphens, underscores, dots, and case as equivalent on lookup but the *display* and discoverability suffer. Worse: a name-squatter grabs `happyterminals-py` on PyPI between registration on crates.io and the Python milestone.

**Why it happens:**
crates.io and PyPI normalize names equivalently (`my-package` == `my_package`), but they are independent registries. Public visibility on crates.io advertises the name to squatters watching for adjacent PyPI grabs.

**How to avoid:**
- **Reserve both `happyterminals` on crates.io AND PyPI on day one**, before any public announcement. A minimal placeholder package on PyPI is acceptable per their AUP if it has *some* real content (a single function and README pointing to the Rust crate is typically fine, but check the AUP).
- Also reserve the obvious variants: `happyterminals-py`, `happy-terminals` (hyphen), `happyterminals-core`. crates.io's policy permits this for a maintainer who genuinely intends to publish.
- For the workspace, use a consistent naming scheme: `happyterminals-{core,renderer,compositor,scene,recipe,py}`.
- Document the reservation in PROJECT.md so future contributors don't accidentally publish under a different name.

**Warning signs:**
- Public announcement before PyPI reservation.
- Workspace crates with inconsistent naming (`ht-core` vs `happyterminals-renderer`).
- No CI check that `cargo publish --dry-run` and `python -m build` succeed.

**Phase to address:** **early** (Phase 0 / cleanup phase — must precede any public announcement).

Source: [crates.io policies](https://crates.io/policies), [PyPI naming and squatting analysis](https://blog.orsinium.dev/posts/py/pypi-squatting/), [Package Management Namespaces (Nesbitt 2026)](https://nesbitt.io/2026/02/14/package-management-namespaces.html).

---

### Pitfall 22: Dual-license MIT OR Apache-2.0 — file conventions

**What goes wrong:**
Project says "MIT OR Apache-2.0" in `Cargo.toml` but ships only one `LICENSE` file. crates.io accepts it; legal review at downstream companies rejects the dependency because they can't verify the license offer. Or worse: contributor adds a file under "MIT only" buried in a subdirectory and the dual-license claim becomes false.

**Why it happens:**
Rust ecosystem convention is well-established but easy to get wrong:
- `LICENSE-MIT` and `LICENSE-APACHE` (both files, both at root)
- `Cargo.toml`: `license = "MIT OR Apache-2.0"` (note the SPDX OR, not slash)
- README footer: "Licensed under either of MIT or Apache 2.0 at your option."
- Apache-2.0 contribution clause (boilerplate paragraph) so contributions inherit the dual license.

**How to avoid:**
- Put both files at root with the canonical SPDX boilerplate.
- Add the contribution paragraph to README and CONTRIBUTING.
- CI check: every workspace crate's `Cargo.toml` has matching license string; root has both files; no nested LICENSE files claim something different.
- Use `cargo deny check licenses` in CI to also constrain transitive deps.

**Warning signs:**
- Single `LICENSE` file or `license = "MIT/Apache-2.0"` (slash, not SPDX-valid).
- Subdirectory with a different LICENSE.
- No contribution clause.

**Phase to address:** **early** (before first publish; trivial to do, painful to retrofit cleanly).

---

### Pitfall 23: Semver discipline before 1.0 — "minor versions can break things"

**What goes wrong:**
Pre-1.0 ethos says "anything goes." Project ships `0.3.0` with breaking changes; a downstream user pinned `0.2` is fine; another pinned `^0.2` (which Cargo treats as `0.2.x` only — fine); a third pinned `^0.2.0` thinking it'd allow `0.3` (it doesn't in Cargo). Nobody is sure what was breaking. Multiple `0.x` series proliferate.

**Why it happens:**
Cargo's semver has subtle rules: `^0.2.0` means `>= 0.2.0, < 0.3.0`, not `>= 0.2.0, < 1.0.0`. Users coming from npm/Python guess wrong.

**How to avoid:**
- Adopt **strict semver** even pre-1.0: bump minor for breaks (matches Cargo's interpretation), bump patch for fixes.
- Maintain a `CHANGELOG.md` per [Keep a Changelog](https://keepachangelog.com/) with explicit `### Breaking` sections.
- Use `cargo semver-checks` in CI to catch accidental API breakage in patch releases.
- Document the MSRV (minimum supported Rust version) and treat MSRV bumps as breaking until 1.0.
- Don't reach 1.0 until the API has stabilized through real user feedback — but get out of the perpetual-0.x trap by setting an explicit "1.0 readiness" checklist in the roadmap.

**Warning signs:**
- No CHANGELOG.md.
- `cargo semver-checks` not in CI.
- Patch releases changing function signatures.

**Phase to address:** **late** (pre-release polish, before first public release on crates.io).

---

### Pitfall 24: MSRV policy sprawl

**What goes wrong:**
Project doesn't pick an MSRV; uses every new Rust feature; users on Debian-stable Rust (often a year behind) can't compile. Or the opposite: project pins MSRV at Rust 1.65, then a transitive dep raises *its* MSRV to 1.78, and project's build breaks for older toolchains anyway.

**Why it happens:**
MSRV is a coordination problem: yours, your deps', and your users'.

**How to avoid:**
- Pick an MSRV explicitly (e.g., "stable - 6 months" — currently around Rust 1.79 in early 2026), document it in README and `Cargo.toml` (`rust-version = "1.79"`).
- CI matrix runs MSRV in addition to stable. Use `cargo +1.79 check` to enforce.
- Use `cargo-msrv` to verify and detect drift.
- When raising MSRV, do it in a minor-version bump and announce in CHANGELOG.
- Don't take deps with looser MSRV expectations than yours; pin ranges defensively.

**Warning signs:**
- No `rust-version` in Cargo.toml.
- CI only tests `stable`.
- README says "any modern Rust."

**Phase to address:** **late** (pre-release polish), but pick MSRV early so it's not retrofitted.

---

### Pitfall 25: CI matrix cost explosion — Linux × macOS × Windows × Python × Rust

**What goes wrong:**
Naive matrix: 3 OSes × 4 Python versions × 2 Rust toolchains × 2 features = 48 jobs. Every PR runs them all; CI takes 30 minutes; contributors give up.

**Why it happens:**
"Test everything" is the safe default. The matrix multiplies fast.

**How to avoid:**
- Minimal "fast" matrix on every PR: Linux + Rust stable + Python 3.11 + default features. ~3–5 jobs.
- Full matrix only on `main` push, on releases, and on a `[ci-full]` PR label.
- Use `abi3` for Python wheels (single wheel covers Python 3.x ≥ minimum), eliminating the Python-version axis entirely on the publish path.
- For native cross-compilation, use `cargo-zigbuild` and `maturin build --target ...` — cheaper than spinning up a Mac/Windows runner per matrix cell.
- Cache aggressively: `Swatinem/rust-cache@v2`, `actions/setup-python` cache, sccache.
- Skip platform-specific tests on platforms where they don't apply rather than stubbing.

**Warning signs:**
- PRs taking > 10 minutes of CI.
- Matrix > 12 jobs on every PR.
- Contributors' first PRs blocked by flaky cross-platform tests.

**Phase to address:** **late** (but set up cheap CI from Phase 0; don't wait until release).

---

### Pitfall 26: Snapshot-test flakiness from time, randomness, locale

**What goes wrong:**
Snapshot tests of rendered terminal output diff against committed `.snap` files. Tests fail in CI because:
- The clock advanced: an effect that's "20% complete at frame 5" is now "21% complete."
- Random seed differs (dissolve effect uses `thread_rng()`).
- Locale: numbers formatted with `,` vs `.` decimal separator.
- Terminal size: tests passed at 80×24 on dev, CI runs with `TERM=dumb` and gets a different size.

**Why it happens:**
Snapshot testing assumes deterministic output. Anything that varies between runs breaks it.

**How to avoid:**
- **Fake clock**: every place that reads `Instant::now()` actually reads from an injected `Clock` trait. Tests advance it manually.
- **Seeded RNG**: every place that uses randomness takes a `&mut SmallRng` (or `StdRng`) seeded explicitly. Effects' `seed` parameters are required, not defaulted.
- **Locale-pinned formatting**: never use locale-aware formatters in renders.
- **Size-pinned tests**: tests instantiate `Grid::new(80, 24)` directly; never call `terminal::size()` from a test.
- Use **`insta`** with `INSTA_UPDATE=no` in CI (default behavior with `CI=true`); review-then-commit workflow for changes.
- For PTY-based integration tests: use `ratatui-testlib`'s headless mode in CI.

**Warning signs:**
- A snapshot test passes locally but fails in CI without code changes.
- `Instant::now()` or `SystemTime::now()` called outside a `Clock` abstraction.
- `rand::random()` or `thread_rng()` in render code paths.

**Phase to address:** **early** (the clock and rng abstractions are foundational; retrofitting is invasive).

Source: [Insta snapshot docs](https://insta.rs/), [ratatui snapshot testing recipe](https://ratatui.rs/recipes/testing/snapshots/), [ratatui-testlib](https://crates.io/crates/ratatui-testlib).

---

### Pitfall 27: API ergonomics — "hello world" requires 40 lines

**What goes wrong:**
First example in README requires importing 8 modules, building a Camera, a Pipeline, a Scene, registering a Signal scope, calling `run()` with a closure that takes a context. Beginners bounce. The "magic, not plumbing" promise is unkept on first contact.

**Why it happens:**
Rust APIs gravitate to maximum explicitness because the type system rewards it. But README examples are marketing.

**How to avoid:**
- The README example must fit in one screen (≤ 25 lines including imports).
- Provide a default-everything `quickstart::spinning_cube()` and `quickstart::run(scene)` that hides the ceremony. The ceremony is available; it's just not the front door.
- A Python example should be ≤ 10 lines.
- Have someone unfamiliar with the project try the README in a fresh terminal and time how long it takes to see something on screen. Target < 60 seconds from `cargo new` to spinning cube.

**Warning signs:**
- README example requires more than one `let` to set up the runtime.
- Examples differ from what `cargo new` users actually have in scope.
- Multiple types named similarly (`Effect`, `Effect`, `Fx`) without prose disambiguation.

**Phase to address:** **late** (pre-release polish), but design intent during API-shape phases.

---

### Pitfall 28: Panics that should be Results — error UX in scene-graph bugs

**What goes wrong:**
User's scene declaration has a typo ("dissoolve" effect), library panics with `unwrap on None` deep in the registry. Error message is in Rust's panic format, not actionable. Python user sees `pyo3_runtime.PanicException`. They think the library is broken.

**Why it happens:**
`unwrap`/`expect` are quick to write during prototyping; converting to typed errors is "polish work" that gets deferred forever.

**How to avoid:**
- Public API surface returns `Result<_, SceneError>` (or similar). Internal panics only for true logic errors (invariants) — never for user input.
- Use `thiserror` (or hand-rolled enums) with messages that say *what* failed, *what was expected*, and *how to fix* — the Rust style guide for error messages.
- For Python: convert Rust errors to dedicated Python exception classes (`SceneError`, `EffectNotFound`), not generic `RuntimeError`.
- Lint: `clippy::unwrap_used` and `clippy::expect_used` deny in non-test code.

**Warning signs:**
- `.unwrap()` count grows with project size.
- Issues filed with stack traces ending in `panicked at 'called Option::unwrap()'`.
- Python users report `PanicException`.

**Phase to address:** **mid** (in each layer's implementation), enforced by clippy lint from Phase 0.

---

### Pitfall 29: Logging in the frame path

**What goes wrong:**
`tracing::debug!("rendering {} cells", n)` inside the cell loop. With debug logging off, it's "free" — except it still evaluates the format args lazily but allocates the span context. With logging on, it floods stdout (which is also where the TUI is drawing) and corrupts the screen.

**Why it happens:**
Logging is helpful during development; once added, it stays.

**How to avoid:**
- Logs go to a file or `stderr`, *never* `stdout` when a TUI owns the screen.
- A startup hook redirects `tracing` to a file (or to a separate buffer rendered after exit) when in TUI mode.
- The frame-path code uses `tracing::trace!` only, gated by a feature flag that is off by default and stripped at compile time in release builds.
- Provide a `RUST_LOG=happyterminals=trace` workflow that writes to `~/.cache/happyterminals/last.log` for postmortem.

**Warning signs:**
- `println!` or `eprintln!` anywhere in the render loop.
- Tracing default subscriber initialized to stdout.
- Demo output garbled when logging is enabled.

**Phase to address:** **mid** (during ratatui integration / event loop).

---

### Pitfall 30: O(n²) or O(n·m) effect composition

**What goes wrong:**
Pipeline composes 10 effects, each iterating over all 1920 cells. That's 19,200 cell visits per frame — fine. Then someone composes a "for each particle, apply each effect to a sub-grid" pattern: 100 particles × 10 effects × 100 cells each = 100k visits. At 60fps, that's 6M cell-ops/sec — borderline. With 1000 particles, it's 60M/sec, frame budget blown.

**Why it happens:**
Composability invites nesting; nesting multiplies cost; the per-cell cost is small enough to hide the multiplication until the scene grows.

**How to avoid:**
- Pipeline executes effects on the *whole grid in passes*, not per-object. Particles render *into* the grid; effects then transform the grid as one.
- Profile with `criterion`: include scenes with realistic counts (1000 particles, 20 effects).
- For unavoidable nesting, document the cost model and provide a bench in `examples/perf/`.
- Allow effects to declare dirty regions (Pitfall 12) so a composition only touches changed cells.

**Warning signs:**
- Effect API exposes "apply to a sub-region for object X" patterns.
- Per-frame time scales superlinearly with object count.
- No bench for composed pipelines.

**Phase to address:** **mid** (during pipeline executor work).

---

### Pitfall 31: Cursor visibility, mouse mode, and SGR state leakage

**What goes wrong:**
On exit (clean or panic): cursor invisible, mouse capture still on, last SGR color persists into the user's shell prompt — they see a bright-red `$` for the rest of the session.

**Why it happens:**
The terminal is a global state machine. Every escape sequence emitted is "remembered" until reset.

**How to avoid:**
- Cleanup sequence on exit (and in panic hook): `\e[0m` (reset SGR), `\e[?25h` (show cursor), `\e[?1000l\e[?1003l\e[?1015l\e[?1006l` (disable all mouse modes), `\e[?1049l` (leave alt screen), `disable_raw_mode()`.
- Wrap as a `Drop` impl on a `TerminalGuard` (see Pitfall 1).
- Smoke test: run the demo, exit, verify shell prompt has default color and cursor visible.

**Warning signs:**
- After `cargo run --example cube`, the next shell prompt is colored.
- Mouse clicks in the next shell session emit weird sequences.

**Phase to address:** **early** (part of the panic-safe lifecycle).

---

### Pitfall 32: Vendored dependencies going stale

**What goes wrong:**
`vendor/pyo3/`, `vendor/ratatui/`, `vendor/tui-vfx/` are in the repo "for reference reading." Six months later they're 3 versions behind. Someone reads the vendored docs, writes code against an old API, then the build fails against the real (newer) crates.io version. Vendored copies bloat the repo and clone time.

**Why it happens:**
"I'll just keep a copy to read" is a useful trick for a single sprint, but vendored copies have no cleanup signal.

**How to avoid:**
- Move vendor copies to `vendor/_reference/` with a `STAMP.txt` recording the version snapshot date and source URL.
- Add to `.gitattributes`: `vendor/_reference/** linguist-vendored=true linguist-generated=true` (keeps GitHub stats clean).
- Set a calendar reminder / issue: refresh vendor snapshots quarterly OR delete them entirely once the team has internalized the relevant APIs.
- Never let vendor copies be referenced from `Cargo.toml` `path = ...` deps.

**Warning signs:**
- Git history shows vendor copies last touched > 6 months ago.
- Cargo.toml `path = "vendor/..."` anywhere.
- Repo `.git` size dominated by `vendor/`.

**Phase to address:** **early** (cleanup pass before Milestone 1).

---

### Pitfall 33: Workspace dependency version drift

**What goes wrong:**
Three crates each declare `ratatui = "0.27"`. One day someone bumps `happyterminals-renderer` to `"0.28"` for a feature; now Cargo resolves two ratatui versions; types don't match across crate boundaries; baffling compile errors.

**Why it happens:**
Shared deps in a workspace aren't centrally managed by default.

**How to avoid:**
- Use `[workspace.dependencies]` in the root `Cargo.toml`. Member crates write `ratatui.workspace = true`. Single source of version truth.
- Same for `tachyonfx`, `pyo3`, `crossterm`, `unicode-segmentation`, etc.
- CI: `cargo tree -d` (duplicates) check; fail on unexpected duplicates.

**Warning signs:**
- Per-crate `Cargo.toml`s with hardcoded versions.
- `cargo tree -d` shows multiple versions of foundational deps.
- Compile errors like "expected `ratatui::Buffer`, found `ratatui::Buffer`."

**Phase to address:** **early** (workspace setup; trivial when done first, surgical when retrofitted).

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Hardcode 80×24 grid in early demos | Faster to first pixel | All examples die in resize tests; users start from a broken pattern | Only in `examples/_internal/` smoke tests; never in README |
| Skip the diff renderer in v0 | Days of saved work | The "fine-grained reactivity" claim is false; refactor cost is high | Never — it's the architectural thesis; do it from Milestone 1 |
| Use `unwrap()` everywhere "for now" | Faster prototyping | Public API panics on user input; Python users see PanicException | OK in `examples/`; never in library code (clippy-enforced) |
| Single-license MIT only | Avoids dual-license boilerplate | Major OSS users won't adopt without explicit patent grant (Apache-2.0) | Never — dual-license from day one is industry standard for Rust |
| Copy tui-vfx grid trait verbatim | Saves design work | Lock-in to a crate not in the dep graph; concept drift | Never — the abstraction is small enough to design fresh |
| Ship 24-bit color only | Simpler output stage | Anything pre-2018 looks broken; tmux issues; SSH-into-old-server users complain | Acceptable as a feature flag default with explicit fallback path |
| One big `happyterminals` crate (no workspace split) | Easier dep management | Python milestone forces breaking the mono-crate; refactor pain | Acceptable only if Python milestone is reconsidered out of scope |
| `pyo3-asyncio` because it's well-known | Familiar API | Deprecated/unmaintained for current PyO3; migration churn | Never — use `pyo3-async-runtimes` from day one |
| Skip Owner/Scope in reactive primitives | Days saved | Memory leaks at every scene transition; rewriting reactive core | Never — owner tree is fundamental |
| `Rc<RefCell>` reactive + "deal with threading later" | Simpler initial code | Async / Python milestone forces `Arc<Mutex>` rewrite | OK only with documented "single-threaded by design + send_handle channel" model |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| ratatui `Buffer` ↔ happyterminals `Grid` | One-way conversion that copies on every frame | Adapter that writes Grid cells *into* a borrowed Buffer in place, no allocation |
| tachyonfx `Effect` | Re-create per frame, lose state | Hold instance across frames; pass real elapsed `Duration`; check `done()` to advance pipeline |
| crossterm | Initialize raw mode without Drop guard | RAII `TerminalGuard` + panic hook (Pitfall 1) |
| pyo3 | Hold `Python<'_>` token across blocking work | Minimal `with_gil` blocks; mirror data into Rust cache |
| pyo3-async-runtimes | Mix tokio + asyncio runtimes ad-hoc | Pick one model: Python owns main thread, Rust runs on background, channel between |
| crossterm event reading | Block forever on `read()` in render thread | Use `poll(Duration)` with frame-budget timeout |
| OBJ loader | Parse from path inside a hot loop | Load once into a `Mesh` struct, render that |
| tmux detection | Check `$TMUX` once at startup | Re-check on `SIGCONT` (re-attached session may differ) |
| Windows Terminal | Assume same SIGWINCH semantics as POSIX | Test on Windows in CI; use crossterm's resize event abstraction |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Per-frame `format!` allocations | Frame-time variance, GC-like jitter | Reusable `Vec<u8>` write buffer; pre-cached SGR strings | > 30fps on small grids; any fps on 200×60 grids |
| Full-buffer redraw | Constant per-frame byte output regardless of changes | Diff stage + dirty-cell tracking tied to signal graph | Always — undermines architecture |
| Z-buffer thrash | Visible flicker on faces; fps drops with scene complexity | Reversed-Z, scene-fit near/far, consistent winding | > ~50 visible faces; close camera distances |
| O(n·m) effect composition | Frame budget blown when particle count grows | Whole-grid passes; dirty regions; bench in CI | > 100 particles + > 10 effects |
| Logging in frame path | Stdout corruption; perf cliff with `RUST_LOG=trace` | Logs to file/stderr; trace! gated by feature, stripped in release | Always when active |
| Python GIL held across render | Async tasks starve | Render without GIL; cache signal values Rust-side | Always when GIL > frame budget |
| Mesh re-parse per frame | OBJ loader called every render | Load once → `Mesh` → render `&Mesh` | Always — pure regression |
| `Arc<Mutex>` on hot path | Lock contention; weird timing dependencies | Single-threaded reactive core + channel for cross-thread writes | > 1 thread writing signals |
| Eager memo recomputation | Memos run when irrelevant inputs change | True dependency tracking + lazy recompute on read | Scenes with > ~20 memos |
| Repeated `terminal::size()` calls | Syscall per cell = catastrophic | Read once per frame; cache; invalidate on resize | Always |

## Security Mistakes

(Limited surface — TUI library, no network, no untrusted execution. But:)

| Mistake | Risk | Prevention |
|---------|------|------------|
| JSON recipe loader executes arbitrary code | Recipes from untrusted sources (LLM output, downloads) trigger code paths via reflection | Recipes are pure data; effect names lookup in a static registry; no `eval`, no plugin loading at recipe-load time |
| OBJ/STL loader trusts file size | Malicious `.obj` claims 1B vertices, OOMs | Cap mesh sizes (configurable, sane default); validate counts before allocating |
| Panic-on-user-input | DoS by submitting weird recipe | Errors are `Result`s with structured types (Pitfall 28) |
| Path traversal in recipe `mesh: "../../../etc/passwd"` | Reads files outside intended directory | Recipes resolve mesh paths only relative to a sandbox root; reject absolute paths and `..` |
| ANSI injection from user-provided strings | Scene with attacker-controlled text emits raw escapes that reposition cursor, hide content, exfiltrate via OSC 52 | Sanitize: only allow displayable graphemes through `Grid::put_str`; strip C0/C1 control codes |
| Unsafe FFI in PyO3 layer | Crash brings down host Python process | Minimize `unsafe`; audit any `transmute` or raw pointer use; fuzz the boundary |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Hello-world is 40 lines | Bounce on first contact | Quickstart helpers + sub-25-line README example (Pitfall 27) |
| Errors as panics | Users think library is broken; can't recover | Typed errors with actionable messages |
| No way to pause/resume an animation | Demos can't be screen-recorded cleanly | `Runtime::pause()` / `resume()`; fake-clock-friendly |
| Forced async even for trivial scenes | Python hello world requires `asyncio.run` | Provide sync `run(scene)` that handles event loop internally |
| All examples assume color | Users in `TERM=dumb` or piping output see garbage | Examples include a no-color variant; library handles `NO_COLOR` env var |
| Documentation shows only happy paths | Users hit edge cases (resize, narrow grid, missing mesh) and get unhelpful errors | Each major feature has a "things that go wrong" section in its docs |
| Type names collide with Python builtins | `from happyterminals import effect` shadows `unittest.effect`? Not really, but `signal` collides with `signal` stdlib | Document the import pattern; suggest `import happyterminals as ht` |
| Scene graph debug output is opaque | "Why isn't my cube showing?" → no inspection tool | `scene.debug_dump()` prints the resolved graph + each layer's bounds + effect chain |

## "Looks Done But Isn't" Checklist

- [ ] **Spinning cube demo:** Often missing — terminal restoration on Ctrl-C. Verify: kill the process with SIGTERM, shell prompt is sane.
- [ ] **Reactive primitives:** Often missing — owner/scope cleanup. Verify: 10k scene transitions, RSS stable.
- [ ] **Grid type:** Often missing — wide-cell handling for emoji. Verify: insert "🤦🏼‍♂️" at (0,0); cells (0,0) and (1,0) are correct; cell (2,0) is unaffected.
- [ ] **3D renderer:** Often missing — aspect-ratio compensation. Verify: cube example looks like a cube, not a tower.
- [ ] **OBJ loader:** Often missing — quad triangulation. Verify: load a Blender-default `.obj` (many use quads), render successfully.
- [ ] **tachyonfx integration:** Often missing — frame timing with real elapsed Duration. Verify: `fade_in(2s)` actually takes 2 seconds when fps is uneven.
- [ ] **Color output:** Often missing — fallback chain for non-truecolor terminals. Verify: `COLORTERM=` (empty) `cargo run` still produces sensible output.
- [ ] **Tmux integration:** Often missing — `Tc`/`RGB` documentation. Verify: README includes a tmux.conf snippet.
- [ ] **Diff renderer:** Often missing — actually emitting diff. Verify: scene where one cell changes per frame writes ~10 bytes/frame, not 5000.
- [ ] **Panic safety:** Often missing — restoration in panic hook. Verify: `panic!()` inside an effect leaves terminal usable.
- [ ] **Python bindings:** Often missing — sync `run()` entry point. Verify: 10-line example with no `asyncio` works.
- [ ] **PyPI wheel:** Often missing — abi3 build. Verify: one wheel installs on 3.9, 3.10, 3.11, 3.12.
- [ ] **License files:** Often missing — both LICENSE-MIT and LICENSE-APACHE at root. Verify: `ls LICENSE*` returns both.
- [ ] **CHANGELOG:** Often missing — Unreleased section with current changes. Verify: every PR updates CHANGELOG.
- [ ] **README:** Often missing — accurate stack section (no "tui-vfx"). Verify: `grep -i tui-vfx README.md project.md` returns nothing outside of explicit "Why not tui-vfx" rationale.
- [ ] **CI:** Often missing — clippy `-D warnings` and snapshot test enforcement. Verify: a deliberately-failing snapshot makes CI red.

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Terminal trashed at runtime (no panic hook) | LOW | Add panic hook + `Drop` guard. One-off `stty sane` documented. |
| README/docs drift | LOW | Doc-lint CI, single rewrite pass. |
| Stub crates with stale deps | LOW | Delete deps; clean Cargo.lock. |
| Reactive memory leak (no owner tree) | HIGH | Architectural rework — owner type touches every public API. Best caught early. |
| Single-threaded vs Send/Sync wrong choice | HIGH | Public-API rewrite; affects every consumer. Best caught early. |
| Wide-cell math wrong | MEDIUM | Centralize through `Grid::put_str`; fix in one place; snapshot regressions catch consumers. |
| Z-fighting | MEDIUM | Switch to reversed-Z + tweak near/far; mostly internal. |
| Wrong ASCII aspect ratio | LOW | Add parameter with non-default; users fix per-scene; default change is a minor-version bump. |
| `pyo3-asyncio` deprecated | MEDIUM | Migrate to `pyo3-async-runtimes`; minor API differences. |
| PyPI name squatted before reservation | HIGH | File PyPI dispute (slow); or rename to `happyterminals-py` style; cascading doc/example updates. |
| Single-license MIT shipped, dual-license intended | MEDIUM | Add LICENSE-APACHE + boilerplate; bump minor; downstreams may need re-review. |
| MSRV too aggressive | LOW | Bump MSRV in CHANGELOG; users on old toolchain pin to last supported version. |
| CI matrix too expensive | LOW | Tier the matrix (PR-fast vs main-full). |
| Snapshots flaky | MEDIUM | Add Clock + RNG abstractions; one-time refactor. |
| Hello-world too long | LOW | Add quickstart helpers; rewrite README. |
| Performance cliff | MEDIUM | Profile, fix worst offender; usually 1–2 places. |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| 1. Panic-trashed terminal | early | Integration test: panic in effect → TTY queryable post-panic |
| 2. README/docs drift | early | CI doc-lint grep for forbidden strings |
| 3. Stub crates with deps | early | `cargo udeps` clean; Cargo.lock minimal |
| 4. Reactive memory leak | early | 10k-transition memory test in CI |
| 5. Over/under-firing | early | Effect-runs-per-tick instrumentation; cycle-detection panic test |
| 6. Send/Sync ambiguity | early | Documented threading model in README; channel API present |
| 7. Wide-cell math | early | Property test: random graphemes never desync columns |
| 8. Color regression | mid | Snapshot tests for 24/256/16/mono; matrix in CI |
| 9. tmux DCS passthrough | mid | Skipped if no advanced sequences; gated otherwise |
| 10. Resize race | mid | Resize-during-render integration test |
| 11. Allocation churn | mid | `dhat` bench in CI with allocation budget |
| 12. Full-buffer redraw | early | "1-cell change → ~10 bytes" assertion test |
| 13. Z-fighting | mid | Visual snapshot of cube; no flicker |
| 14. Aspect ratio | early | Cube example renders square-looking |
| 15. OBJ/STL brittleness | mid | Fixture test corpus (quads, missing normals, etc.) |
| 16. tachyonfx lifecycle | early | `fade_in(2s)` integration test; `Effect` naming clash resolved in API |
| 17. WASM divergence | late (if at all) | N/A unless WASM pursued |
| 18. pyo3-asyncio deprecated | early | `Cargo.toml` uses `pyo3-async-runtimes` |
| 19. GIL contention | mid | `asyncio` task latency benchmark with running scene |
| 20. Zero-copy hazards | mid | Stress test: 1M view-creations across resizes |
| 21. PyPI/crates.io naming | early | Both registered before public announcement |
| 22. Dual-license files | early | CI license-lint; both files at root |
| 23. Semver discipline | late | `cargo semver-checks` in CI; CHANGELOG enforced |
| 24. MSRV policy | late | `rust-version` in Cargo.toml; CI matrix includes MSRV |
| 25. CI matrix cost | late | PR matrix ≤ 5 jobs; full matrix on main only |
| 26. Snapshot flakiness | early | Clock + RNG abstractions present from Phase 0 |
| 27. Hello-world ergonomics | late | Quickstart helpers + ≤25-line README example |
| 28. Panics → Results | mid | clippy `unwrap_used` denied in lib code |
| 29. Logging in frame path | mid | No stdout output during render; trace gated |
| 30. O(n·m) composition | mid | criterion bench with 1000 particles × 20 effects |
| 31. SGR/cursor leakage | early | Smoke test: post-exit shell prompt is default |
| 32. Vendor stale | early | Vendor moved to `_reference/` with stamps |
| 33. Workspace dep drift | early | `[workspace.dependencies]` used; `cargo tree -d` clean |

## Sources

- [SolidJS pain points and pitfalls (Lipatov)](https://vladislav-lipatov.medium.com/solidjs-pain-points-and-pitfalls-a693f62fcb4c)
- [Lifetime of a signal — SolidJS Discussions #1776](https://github.com/solidjs/solid/discussions/1776)
- [unicode-width docs](https://docs.rs/unicode-width/)
- [unicode-display-width crate](https://lib.rs/crates/unicode-display-width)
- [runefix-core (CJK / emoji width engine)](https://crates.io/crates/runefix-core)
- [It's not wrong that "🤦🏼‍♂️".length == 7 (Hsivonen)](https://hsivonen.fi/string-length/)
- [Grapheme Clusters and Terminal Emulators (Mitchell Hashimoto)](https://mitchellh.com/writing/grapheme-clusters-in-terminals)
- [termstandard/colors](https://github.com/termstandard/colors)
- [Adding 24-bit TrueColor RGB to tmux (Sunaku)](https://sunaku.github.io/tmux-24bit-color.html)
- [tmux truecolor terminfo issue #1236](https://github.com/tmux/tmux/issues/1236)
- [tmux FAQ — passthrough wrapping](https://github.com/tmux/tmux/wiki/FAQ)
- [tmux allow-passthrough](https://tmuxai.dev/tmux-allow-passthrough/)
- [crossterm raw mode panic issue #368](https://github.com/crossterm-rs/crossterm/issues/368)
- [Ratatui Alternate Screen docs](https://ratatui.rs/concepts/backends/alternate-screen/)
- [Ratatui Insta snapshot testing recipe](https://ratatui.rs/recipes/testing/snapshots/)
- [ratatui-testlib (PTY + headless)](https://crates.io/crates/ratatui-testlib)
- [Insta snapshot framework](https://insta.rs/)
- [tachyonfx GitHub](https://github.com/ratatui/tachyonfx)
- [tachyonfx docs.rs](https://docs.rs/tachyonfx)
- [tachyonfx on ratatui.rs ecosystem](https://ratatui.rs/ecosystem/tachyonfx/)
- [pyo3-async-runtimes (active fork of pyo3-asyncio)](https://github.com/PyO3/pyo3-async-runtimes)
- [pyo3-asyncio (deprecated, on crates.io)](https://crates.io/crates/pyo3-asyncio)
- [PyO3 async-await user guide](https://pyo3.rs/v0.23.4/async-await)
- [PyO3 building & distribution](https://pyo3.rs/v0.28.0/building-and-distribution.html)
- [Maturin universal2 PR #403](https://github.com/PyO3/maturin/pull/403)
- [Maturin cross-compile mac discussion #2281](https://github.com/PyO3/maturin/discussions/2281)
- [crates.io policies](https://crates.io/policies)
- [RFC 3463 — crates.io policy update](https://rust-lang.github.io/rfcs/3463-crates-io-policy-update.html)
- [PyPI package squatting analysis (Orsinium)](https://blog.orsinium.dev/posts/py/pypi-squatting/)
- [Package management namespaces (Nesbitt 2026)](https://nesbitt.io/2026/02/14/package-management-namespaces.html)
- [ASCII characters are not pixels (Harri)](https://alexharri.com/blog/ascii-rendering)
- [ASCII rendering HN discussion](https://news.ycombinator.com/item?id=46657122)
- [Z-buffer rendering (WaspDev)](https://waspdev.com/articles/2025-05-09/the-power-of-z-buffer)
- [Keep a Changelog](https://keepachangelog.com/)

---
*Pitfalls research for: declarative reactive terminal scene manager (Rust core + tachyonfx + ASCII 3D + PyO3, public OSS)*
*Researched: 2026-04-14*
