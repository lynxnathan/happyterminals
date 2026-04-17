# Code Quality Audit — happyterminals v1 publish readiness

**Analysis Date:** 2026-04-17
**Scope:** Rust workspace at `/home/lynxnathan/code/happyterminals/` — 8 member crates, 7 shipping examples + 1 utility, 449 passing tests.
**Method:** `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo publish -p <crate> --dry-run`, manual grep for vestigial code / TODO markers / example-shape consistency, targeted reads of flagged files.

---

## Headline numbers

| Metric                                                       | Count |
|--------------------------------------------------------------|------:|
| Tests passing (`cargo test --workspace`)                     |   449 |
| Library-only clippy errors (`--workspace --lib -- -D warnings`) |     0 |
| Workspace clippy errors (`--all-targets -- -D warnings`)        |   139 |
| Example-only clippy errors (`-p happyterminals --examples`)     |    25 |
| Test-only clippy errors (`--workspace --tests`)                 |   114 |
| `rustfmt --check` drift (diff blocks)                          |   175 |
| TODO / FIXME / XXX / HACK markers in `crates/`                |     0 |
| `unreachable!()` call sites in non-dead code                   |     2 |
| `#[allow(dead_code)]` on real items                           |     4 |
| Missing per-crate READMEs                                     |     1 |

**Library surface is clippy-clean. All 139 errors live in tests + examples.** That is the single most important finding in this audit.

---

## BLOCK — must fix before v1 publish to crates.io

### B-1. Internal workspace deps have no `version =` field → publish fails

**Files:**
- `Cargo.toml:79-86` (workspace-level `[workspace.dependencies]` block)
- `crates/happyterminals-renderer/Cargo.toml:17` (uses raw `path = "../happyterminals-core"` — no version at all)

**Evidence:**
```
$ cargo publish -p happyterminals-renderer --dry-run
error: all dependencies must have a version specified when publishing.
dependency `happyterminals-core` does not specify a version
```
Confirmed `-scene`, `-dsl`, `-input` all fail the same way with `no matching package named 'happyterminals-core' found in crates.io index`.

**What to do:** In `Cargo.toml:79-86`, the workspace deps already declare `version = "0.0.0"`, but (a) `0.0.0` is not on crates.io and (b) `happyterminals-renderer/Cargo.toml:17` bypasses the workspace entry with a bare `path = ...`. Fix: bump workspace version to the real publish version (e.g. `0.1.0`) and rewrite renderer line 17 to `happyterminals-core = { workspace = true }`. Publish order must be: `-core` → `-renderer` → `-pipeline` → `-input` → `-scene` → `-dsl` → `-backend-ratatui` → `happyterminals` (meta), each waiting for the previous to appear on the index.

### B-2. `happyterminals-input` is missing required `README.md`

**File:** `crates/happyterminals-input/` — directory lists only `src/` + `Cargo.toml`, no README.

**Evidence:** Every other crate has one; `ls crates/happyterminals-*/README*` returns 6 READMEs for 7 sub-crates.

**Bonus problem in the same file:** `crates/happyterminals-input/Cargo.toml` is the only sub-crate manifest that does **not** inherit `repository.workspace`, `homepage.workspace`, `keywords.workspace`, `categories.workspace`, or `readme`. `cargo publish --dry-run` reports: `warning: manifest has no documentation, homepage or repository.` That alone won't block publish, but it produces an orphan-looking crate page.

**What to do:** (1) Write `crates/happyterminals-input/README.md` matching the ~400 B format used by peers. (2) Align `crates/happyterminals-input/Cargo.toml` with the `description/version/edition/license/repository/homepage/authors/keywords/categories/readme` block used by every other sub-crate (`crates/happyterminals-pipeline/Cargo.toml:1-13` is the canonical template).

### B-3. `happyterminals-renderer/Cargo.toml` `[lints]` does not inherit workspace

**File:** `crates/happyterminals-renderer/Cargo.toml:34-42`

Every other member uses `[lints] workspace = true`. `-renderer` hand-rolls its own `[lints.rust]` and `[lints.clippy]` blocks, which silently drops the workspace-level rules (`unreachable_pub = warn`, `rust_2018_idioms = warn`, `pedantic = warn`, `module_name_repetitions = allow`, `todo = warn`, etc.). It is not a publish-gate error, but it is why `-renderer` shows 0 clippy errors standalone despite having the same raw-`as` casts the other crates caught — the pedantic group is just not enabled here.

**What to do:** Replace lines 34-42 with `[lints] workspace = true`. Expect a small burst of new warnings and fix them alongside the test-file cleanup in B-4.

### B-4. 139 workspace clippy errors with `-D warnings` (tests + examples only)

**Impact on publish:** `cargo publish` does **not** run clippy, so this does not block the raw upload. It will, however, block any CI pipeline that runs `cargo clippy --workspace --all-targets -- -D warnings` as a release gate (and the deferred-items log in `.eclusa/phases/03.4-examples-library/deferred-items.md:9-30,58-89` already calls this out as a pre-publish cleanup item). Classified as BLOCK because the user asked about "workspace-wide clippy errors that will block crates.io publish" as a gate.

**Category breakdown (from full-workspace run, 139 total):**

| Count | Category | Dominant crate |
|------:|----------|----------------|
| 42 | `clippy::unwrap_used` on `Result` | `happyterminals-dsl` tests + `-input` tests |
| 32 | `clippy::expect_used` on `Option` | `happyterminals-input` (`action.rs`, `context.rs`, `defaults.rs`) |
| 26 | `clippy::unwrap_used` on `Option` | `happyterminals-core/src/grid.rs` tests |
| 9 | `clippy::unwrap_err_used` | `happyterminals-dsl/src/json.rs` tests |
| 7 | `clippy::default_trait_access` (`Default::default()` → `HashMap::default()`) | `happyterminals-scene/tests/` |
| 6 | `clippy::doc_markdown` (missing backticks in `//!` doc headers) | `-renderer/tests/{obj,stl}_corpus.rs`, `-scene/tests/scene_graph.rs` |
| 3 | `clippy::expect_used` on `Result` | `-dsl/src/json.rs` |
| 3 | `clippy::redundant_closure` | `-dsl/src/json.rs:1017,1081,1109` |
| 2 | `clippy::single_match_else` | `-renderer/tests/{obj,stl}_corpus.rs` |
| 2 | `clippy::uninlined_format_args` | `-renderer/src/camera.rs:401,430` (note: lib file, but only triggers under `--tests`) |
| 2 | `clippy::approx_constant` (`3.14` → `PI`) | `-scene/tests/scene_types.rs:51,54` |
| 1 | `unused_imports: Effect` | `-pipeline/tests/smoke.rs:7` |
| 1 | `unused_imports: Owner` | `-scene/tests/scene_graph.rs:6` |
| 1 | `clippy::cast_possible_truncation` | `-core/src/grid.rs:144` (test) |
| 1 | `clippy::no_effect_underscore_binding` | `-core/src/grid.rs:133` (test) |
| 1 | `clippy::match_same_arms` | `-dsl/src/json.rs:999` |

**Per-crate totals (with `-p … --all-targets -- -D warnings`):**

| Crate | Errors |
|-------|------:|
| `happyterminals-dsl` | 74 |
| `happyterminals-input` | 33 |
| `happyterminals-backend-ratatui` | 33 |
| `happyterminals` (meta, counts examples) | 25 |
| `happyterminals-core` | 15 |
| `happyterminals-scene` | 12 |
| `happyterminals-renderer` | 0 ← but see B-3 |
| `happyterminals-pipeline` | 0 |

**Example-specific errors (all 25 are in `crates/happyterminals/examples/`):**

- `spinning-cube/main.rs:9-17` — 21× `doc_markdown` in the `//!` header (`Cell`, `Signal`, `Memo`, `Effect`, `Grid`, etc.).
- `model-viewer/main.rs:10` — `doc_markdown` on `InputMap`.
- `model-viewer/main.rs:56` — `clippy::too_many_lines` (146 / 100).
- `model-viewer/main.rs:64` — `clippy::let_and_return`.
- `model-viewer/main.rs:126,127` — `clippy::cast_lossless` (`grid.area.width.max(1) as f32`).
- `particles/main.rs:10,11,13` — 4× `doc_markdown` on `ParticleEmitter`, `Renderer::draw`, `Renderer::draw_particles`, `InputMap`.
- `transitions/main.rs` — 6 errors, same family.

These are exactly the items the deferred-items log (`.eclusa/phases/03.4-examples-library/deferred-items.md`) listed for "Phase 03.5 pre-publish lint cleanup." Every error it predicted is present; nothing on the list has been silently fixed.

**What to do:** One dedicated plan in Phase 03.5 ("lint-cleanup"). Strategy per category:
- Test `.unwrap()`/`.expect()` on `Option`/`Result`: either (a) add `#![allow(clippy::unwrap_used, clippy::expect_used)]` at the top of each `tests/` file, which is the idiomatic escape hatch for test code (crate lib files already do this in `-core/src/lib.rs:36`), or (b) replace with `.ok_or_else(|| …)?` in integration tests. Option (a) is ~12 allow-lines total; option (b) is ~100 mechanical edits. Recommend (a) — test panics ARE the assertion.
- `doc_markdown`: purely mechanical backtick addition, ~30 edits.
- `default_trait_access`: `Default::default()` → `HashMap::default()`, 7 edits.
- `model-viewer too_many_lines`: either decompose `main()` or `#[allow(clippy::too_many_lines)]` with a short justification comment. Given this is an example, the allow is fine.
- `cast_lossless`: `f32::from(x)` replaces `x as f32`, 2 edits.
- Unused imports: remove 2 lines.

Rough effort: ~2 hours of mechanical work, 0 architectural decisions.

### B-5. 175 rustfmt diffs across workspace

**Files (from `cargo fmt --all -- --check`):**
- `crates/happyterminals/src/lib.rs` — import-group sort drift.
- `crates/happyterminals/examples/color-test/main.rs` — multiple.
- `crates/happyterminals/examples/model-viewer/main.rs` — import ordering.
- `crates/happyterminals/examples/particles/main.rs` — drift.
- `crates/happyterminals/examples/static_grid.rs` — drift.
- `crates/happyterminals/examples/transitions/main.rs` — drift.

**Impact:** Soft block — doesn't stop publish, but a pretty-printer CI gate will fail, and the crates.io `docs.rs` render can look off if import sorts affect rustdoc group ordering.

**What to do:** Single bulk `cargo fmt --all` commit. The deferred-items log `.eclusa/phases/03.4-examples-library/deferred-items.md:50-57` scoped this exact action to Phase 03.5.

---

## FLAG — route to Phase 3.5 pre-publish cleanup, non-blocking

### F-1. Vestigial spike functions in `happyterminals-core/src/runtime.rs`

**File:** `crates/happyterminals-core/src/runtime.rs:64-90`

Three functions named `__spike_owner_current_exists`, `__spike_immediate_effect_accepts_wrapped_fnmut`, `__spike_memo_accepts_wrapped_fn` — all `pub(crate)`, all `#[doc(hidden)]`, all `#[allow(dead_code)]`, all documented as "Spike A/B/C: confirm … exists." Their body comment at line 2-30 says they are "small compile-time spikes that pin down MEDIUM-confidence API details from `reactive_graph` 0.2.13 … Nothing here is public. The spikes exist to fail fast at build time if `reactive_graph`'s shape drifts from what RESEARCH.md assumes."

**Status:** The project shipped v1, Phase 2.5, and Phase 3.4 on top of this crate. `reactive_graph` 0.2.13 has held. The spikes have never fired a build failure in anger. They are dead scaffolding from the 2026-04-14 exploration commit.

**What to do:** Delete all three functions and the module-level spike-outcome doc comment (lines 7-30). Keep `wrap_local_fnmut` and `wrap_local_fn` (they are used by `Effect::new` / `Memo::new`). Trim from ~91 lines down to ~45. This removes 3 of the 4 `#[allow(dead_code)]` attributes in the entire crate graph.

### F-2. Deprecated-but-kept `Grid::inner_mut`

**File:** `crates/happyterminals-core/src/grid.rs:98-105`

```rust
#[deprecated(note = "use buffer_mut()")]
#[allow(dead_code)]
pub(crate) fn inner_mut(&mut self) -> &mut Buffer { &mut self.inner }
```

`pub(crate)` + `#[deprecated]` is a weird combination — `deprecated` is a public-API signal, but this method is crate-private. Either there's an internal caller still living (grep across workspace returns zero hits) or it's pure dead weight.

**What to do:** Delete the method. The `buffer_mut()` method on the preceding lines is the live API. Drop the `#[allow(dead_code)]` with it.

### F-3. `CoreError::NotInitialized` is an unreachable placeholder

**File:** `crates/happyterminals-core/src/error.rs:10-23`

Comment at line 17-19: *"Placeholder variant: not constructed in v0.0.0. Future fallible surface area will populate this enum; `#[allow(dead_code)]` keeps the crate compatible with a project-wide `deny(dead_code)` upgrade."*

Publishing a single-variant enum with an explicit "not constructed" doc comment is not ideal — consumers see an `enum CoreError { NotInitialized }` in rustdoc and can't pattern-match against it meaningfully.

**What to do:** Two options. (a) Keep it, remove the "Placeholder variant" doc comment wording from the public-facing rustdoc, and re-word around "this variant is not currently constructed but is reserved for future threading-boundary errors." (b) Delete the enum entirely for v1 and re-introduce when a real error surfaces. Recommend (a) — the Python milestone (M4) will likely populate it and breaking the enum later is worse than shipping a reserved variant.

Also delete `_assert_core_error_send_sync` at `crates/happyterminals-core/src/error.rs:27-31`. It's a runtime-never-called function serving only to pin `Send + Sync` at compile time. Replace with `static_assertions::assert_impl_all!(CoreError: Send, Sync);` (the crate already depends on `static_assertions = "1.1"` per `crates/happyterminals-core/Cargo.toml:35`). One line, expresses intent directly, removes the last `#[allow(dead_code)]` in this crate.

### F-4. `static_grid.rs` violates the example-directory convention

**File:** `crates/happyterminals/examples/static_grid.rs` (flat file, not a directory)

All 7 shipping examples live in `examples/<name>/main.rs` subdirectories. `static_grid.rs` is the only one still at the flat-file layout. Header comment at line 3 calls it "Developer utility — not a demo." — but then why is it still in `examples/`? It shows up in `cargo run --example static_grid` output and Cargo lists it alongside `spinning-cube`, confusing the "what can I run?" story.

**Options:** (a) Move to `crates/happyterminals/examples/static_grid/main.rs` to match peers. (b) Move to `tools/static-grid/` or delete — it's a backend-smoke-test and its README exit-contract ("sanity-check Grid buffer behavior during backend/font changes") is rarely exercised. (c) Promote it to a real tiny "hello world" example and rename — it's actually a useful minimal `run()` demo.

**Recommendation:** (c). Rename to `hello-world`, put it in `crates/happyterminals/examples/hello-world/main.rs`, and lead the README example list with it. The content already demonstrates `run()`, `Grid::put_str`, `InputSignals::terminal_size` — everything a first-time user needs.

### F-5. `unreachable!()` in a real control-flow branch

**File:** `crates/happyterminals/examples/transitions/main.rs:51`

```rust
.unwrap_or_else(|e| unreachable!("static scene: {e}"))
```

`build()` is `fn build(&self) -> Result<Scene, SceneError>`. The failure mode is *"validation failed while constructing the scene"* — not infallible, not programmer-error. `unreachable!()` is wrong here: if a schema change in `-scene` adds a validation rule that the example's hand-rolled cube layer trips, the build panics with `internal error: entered unreachable code` instead of a readable error.

**What to do:** Replace with `.expect("transitions example: static scene failed to validate")` (note: the crate overall denies `expect_used` in lib code, but examples can `#[allow(clippy::expect_used)]` at the file level — several examples already do) or propagate with `?` if the calling closure is made fallible.

Same pattern in `crates/happyterminals/examples/json-loader/main.rs:81` but that one is inside commented-out educational code; leave it.

### F-6. Large file: `happyterminals-dsl/src/json.rs` is 1,405 lines

**File:** `crates/happyterminals-dsl/src/json.rs` (1,405 lines — by far the largest source file in the workspace; next is `-backend-ratatui/src/color.rs` at 573).

Contains the parse / IR-lowering / sandbox-validate / schema-generate / round-trip-test code for the JSON recipe format. The 74 clippy errors in `-dsl` all hit this file. 42 of them are `.unwrap()` in test code that lives in the same `#[cfg(test)] mod` in the same file.

**Impact:** Not a publish blocker, but moving the ~500 lines of inline tests into `crates/happyterminals-dsl/tests/json_*.rs` files would (a) cut the file in half, (b) let the test files opt into `#![allow(clippy::unwrap_used, clippy::expect_used)]` en masse instead of the lib crate needing to keep those in the `#[cfg(test)]` block, and (c) decouple json.rs rebuild time from test-edit churn.

**What to do:** Move inline tests to `tests/json_roundtrip.rs`, `tests/json_sandbox.rs`, `tests/json_schema.rs` as a single Phase 03.5 refactor. Verify `cargo test -p happyterminals-dsl` still shows the same test count before/after.

### F-7. Per-crate lint drift: `-renderer` has stricter-but-narrower lints

Already called out in B-3 — `crates/happyterminals-renderer/Cargo.toml:34-42` hand-rolls `[lints.rust]` and `[lints.clippy]`, which explicitly enables `pedantic = warn` and `unwrap_used/expect_used/dbg_macro = deny`, but skips `missing_docs`, `unreachable_pub`, `rust_2018_idioms`, `todo = warn`, and the `module_name_repetitions / missing_errors_doc / missing_panics_doc = allow` overrides. So `-renderer` is simultaneously stricter (on unwrap) and looser (on missing-docs) than peers.

**What to do:** Same fix as B-3 — converge to `[lints] workspace = true`.

---

## INFO — nice to know, non-action

### I-1. `TODO` / `FIXME` / `XXX` / `HACK` markers: zero hits in `crates/`

No stale follow-up markers in source. Every deferred item is tracked in `.eclusa/phases/*/deferred-items.md`, which is the intended pattern. Clean.

### I-2. `#[cfg(any())]` branches: zero hits

No always-false feature-gate tricks hiding dead code. Clean.

### I-3. `commented-out code blocks`: one intentional instance

`crates/happyterminals/examples/json-loader/main.rs:65-82` — 17-line commented block labeled *"Educational: uncomment this block to see the sandbox reject a traversal path. Left COMMENTED per D-07 so the example renders cleanly by default."* This is a documented pedagogical pattern, not forgotten scaffolding. Leave as-is.

### I-4. Example shape consistency: 7/7 match

All 7 main examples (`spinning-cube`, `model-viewer`, `particles`, `transitions`, `json-loader`, `text-reveal`, `color-test`) and `static_grid` use the identical:

```rust
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> { … }
```

plus the same `happyterminals::prelude::*` import style. No stylistic outliers in error handling, tokio flavor, or camera-setup shape. The consistency cleanup that must have happened in Phase 2.3/2.5 has held through 3.4.

### I-5. Critical-path test coverage: dense

The "render a scene → see output" pipeline has end-to-end tests:

- **Pixel-byte level:** `crates/happyterminals-backend-ratatui/tests/one_cell_change.rs` and `scene_bytes_test.rs` — signal flip → minimal ANSI bytes. These are the tests that make "one reactive cell change = one cell's worth of bytes on the wire" a provable contract instead of a wish.
- **Scene graph:** `crates/happyterminals-scene/tests/scene_graph.rs` + `scene_types.rs`.
- **Renderer fuzz corpus:** `crates/happyterminals-renderer/tests/obj_corpus.rs` + `stl_corpus.rs` + `z_fighting.rs` + `snapshot.rs`. OBJ/STL never-panics contract exercised against 11 real + synthetic files.
- **Reactive core:** 9 integration tests in `crates/happyterminals-core/tests/` covering cycle detection, disposal, diamond-shape propagation, batch semantics, Send/Sync boundaries, proptest on set/get.
- **Pipeline:** `crates/happyterminals-pipeline/tests/smoke.rs` exercises all 10 wired tachyonfx effects end-to-end.
- **DSL:** `crates/happyterminals-dsl/tests/builder_smoke.rs` + `hello_world.rs`, plus ~500 lines of inline tests in `json.rs`.
- **Color pipeline:** `crates/happyterminals-backend-ratatui/tests/color_detect.rs` + `color_snapshots.rs` + `event_signals.rs`.

**Gap (minor):** `happyterminals-input` crate has no `tests/` directory — all tests are inline `#[cfg(test)] mod`. Given the action system is 4 files × ~350 lines and has 33 clippy errors in those inline test blocks, extracting to `tests/` during the B-4 cleanup would double as coverage hygiene. Not a BLOCK — the inline tests still run and still pass. Flagging only because the "move tests out of the 1,405-line `json.rs`" refactor under F-6 applies equally here.

### I-6. `happyterminals-py` crate is commented out in workspace members

`Cargo.toml:12` — `# "crates/happyterminals-py",  # activated in Milestone 4`. Correct and intentional; mentioned here only so reviewer doesn't flag it during publish.

---

## Verdict

**Codebase has 5 blocking issues before v1 publish to crates.io:** (1) inter-workspace version deps, (2) missing `happyterminals-input` README + metadata, (3) `-renderer` lint-table drift, (4) 139 workspace clippy errors (tests+examples — soft-block, CI-gate), (5) 175 rustfmt diffs (soft-block). Libraries are clippy-clean; 449 tests pass; no TODO markers; example shape is consistent. The pre-publish cleanup phase that the deferred-items log already anticipates (Phase 03.5) is exactly the right shape — the log's predictions match today's reality with no drift.

*Audit: 2026-04-17*
