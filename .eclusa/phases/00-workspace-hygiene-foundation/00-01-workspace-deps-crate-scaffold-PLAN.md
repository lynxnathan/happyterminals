---
phase: 00-workspace-hygiene-foundation
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - Cargo.toml
  - crates/happyterminals-compositor/   # renamed -> crates/happyterminals-pipeline/
  - crates/happyterminals-pipeline/Cargo.toml
  - crates/happyterminals-pipeline/src/lib.rs
  - crates/happyterminals-core/Cargo.toml
  - crates/happyterminals-core/src/lib.rs
  - crates/happyterminals-core/src/grid.rs       # DELETED
  - crates/happyterminals-core/src/reactive.rs   # DELETED
  - crates/happyterminals-core/src/python.rs     # DELETED
  - crates/happyterminals-renderer/Cargo.toml
  - crates/happyterminals-renderer/src/lib.rs
  - crates/happyterminals-scene/Cargo.toml
  - crates/happyterminals-scene/src/lib.rs
  - crates/happyterminals-dsl/Cargo.toml
  - crates/happyterminals-dsl/src/lib.rs
  - crates/happyterminals-backend-ratatui/Cargo.toml
  - crates/happyterminals-backend-ratatui/src/lib.rs
  - crates/happyterminals/Cargo.toml
  - crates/happyterminals/src/lib.rs
autonomous: true
requirements: [HYG-02, HYG-03]

must_haves:
  truths:
    - "The workspace compiles clean on Rust 1.86 (`cargo check --workspace`) with no warnings."
    - "The root `Cargo.toml` declares `resolver = \"3\"`, `[workspace.package]`, and `[workspace.dependencies]` with every pinned version from RESEARCH Item 2."
    - "The crate `happyterminals-compositor` no longer exists; `happyterminals-pipeline` exists in its place."
    - "Every member crate declares shared deps as `{ workspace = true }` — no inline version literals."
    - "`crates/happyterminals-core/src/{grid.rs,reactive.rs,python.rs}` are removed; `-core`'s lib.rs is a doc-only stub."
    - "New stub crates `happyterminals-scene`, `-dsl`, `-backend-ratatui`, meta `happyterminals` exist and compile; `happyterminals-py` is listed as a commented-out workspace member."
  artifacts:
    - path: "Cargo.toml"
      provides: "Workspace root with resolver=3, [workspace.package], [workspace.dependencies], [workspace.lints]"
      contains: "resolver = \"3\""
    - path: "crates/happyterminals-pipeline/Cargo.toml"
      provides: "Renamed pipeline crate manifest"
      contains: "name = \"happyterminals-pipeline\""
    - path: "crates/happyterminals-scene/src/lib.rs"
      provides: "Scene crate stub with top-level docs"
    - path: "crates/happyterminals-dsl/src/lib.rs"
      provides: "DSL crate stub with top-level docs"
    - path: "crates/happyterminals-backend-ratatui/src/lib.rs"
      provides: "Backend crate stub with top-level docs"
    - path: "crates/happyterminals/src/lib.rs"
      provides: "Meta crate stub exposing empty `pub mod prelude {}`"
  key_links:
    - from: "Cargo.toml [workspace] members"
      to: "crates/happyterminals*/Cargo.toml"
      via: "member paths resolve; `cargo metadata` enumerates 7 active members + 1 commented"
      pattern: "cargo metadata --format-version 1 | jq '.workspace_members | length'"
    - from: "member Cargo.toml [package]"
      to: "[workspace.package]"
      via: "version.workspace / edition.workspace / license.workspace etc. inheritance"
      pattern: "version.workspace = true"
---

# Plan 00-01 — Workspace Deps Refactor + Crate Scaffold

## Goal
Rewrite the workspace root, rename `-compositor` → `-pipeline`, strip speculative deps from every stub crate, delete premature `-core` modules, and scaffold the four new crates (`-scene`, `-dsl`, `-backend-ratatui`, meta `happyterminals`) so every member compiles clean and inherits shared deps via `{ workspace = true }`.

## Requirements covered
- **HYG-02** — stub crates stripped of speculative deps; premature source files deleted.
- **HYG-03** — `[workspace.dependencies]` pinned version set; every member uses `dep.workspace = true`; crate rename + new scaffolds landed.

## Dependencies
None. This is a Wave 1 root-level plan; every other Phase 0 plan consumes its output (new crate names, new workspace members).

## Parallelizable with
- **00-02** (licensing, contribution, changelog) — touches LICENSE-MIT, LICENSE-APACHE, CONTRIBUTING.md, CHANGELOG.md at repo root; no `Cargo.toml` overlap.
- **00-03** (vendor relocation + toolchain pin) — touches `vendor/**`, `.gitattributes`, `rust-toolchain.toml`; no overlap.

**Sequential before:** 00-04 (per-crate READMEs need the new crate directories to exist) and 00-05 (CI needs the final workspace layout).

## Steps

> Execute from repo root (`/home/lynxnathan/code/happyterminals`). All content blocks referenced below live in **RESEARCH.md** — copy the blocks verbatim; do not rephrase.

1. **Sanity check current state.** Run `git status` and confirm only the files listed in `files_modified` will change. Abort and ask the user if unexpected modifications are present.

2. **Rename `compositor` → `pipeline` at filesystem level** (RESEARCH Item 14):
   ```bash
   git mv crates/happyterminals-compositor crates/happyterminals-pipeline
   ```

3. **Rewrite root `Cargo.toml`** — replace the current 9-line file wholesale with the full block in **RESEARCH Item 2**. This block contains:
   - `resolver = "3"` (verbatim — do NOT downgrade to `"2"`)
   - workspace `members` = the 7 active crates, with `"crates/happyterminals-py"` as a commented-out line
   - `[workspace.package]` (version="0.0.0", edition="2024", rust-version="1.86", license="MIT OR Apache-2.0", authors=["the happyterminals authors"], repository/homepage/keywords/categories/readme)
   - `[workspace.dependencies]` block verbatim from Item 2 (ratatui-core, ratatui, ratatui-widgets, ratatui-crossterm, ratatui-macros, crossterm, tachyonfx, reactive_graph, any_spawner, glam, tobj, stl_io, serde, serde_json, schemars, jsonschema, thiserror, color-eyre, tracing, tracing-subscriber, compact_str, bon, slotmap, pyo3 `{ default-features = false }`, pyo3-async-runtimes, insta, proptest, criterion, and the internal `happyterminals*` path deps)
   - `[workspace.lints.rust]` and `[workspace.lints.clippy]` blocks verbatim (forbid unsafe_code, warn missing_docs, deny unwrap_used/expect_used/dbg_macro, etc.)

4. **Write `crates/happyterminals-pipeline/Cargo.toml`** from the stub template in **RESEARCH Item 4**. Substitute `name = "happyterminals-pipeline"` and `description = "Effect pipeline (\`dyn Effect\` trait objects) and tachyonfx adapter for happyterminals."` from the Item 4 description table. `[dependencies]` empty; `[lints] workspace = true`.

5. **Rewrite `crates/happyterminals-core/Cargo.toml`** from the same template with the -core name and description. **Strip** any existing `pyo3`, `tui-vfx`, `ratatui` entries — Phase 0 stubs have zero deps. `[lints] workspace = true`.

6. **Rewrite `crates/happyterminals-renderer/Cargo.toml`** using the **`-renderer` variant** from RESEARCH Item 4 (the variant that flips `unsafe_code = "allow"` in a local `[lints.rust]` block, and re-declares the clippy lints inline because `lints.workspace = true` cannot coexist with a per-crate override in the same `[lints]` table). Strip all speculative deps.

7. **Create the four new stub crates.** For each — `crates/happyterminals-scene/`, `crates/happyterminals-dsl/`, `crates/happyterminals-backend-ratatui/`, `crates/happyterminals/` — create `Cargo.toml` from RESEARCH Item 4 template (with the matching description from Item 4's description table) and `src/lib.rs` from the per-crate scaffolds in **RESEARCH Item 15**. Each lib.rs is a single `//!` doc comment; the meta crate `happyterminals` additionally exports `pub mod prelude {}`.

8. **Reset `crates/happyterminals-core/src/lib.rs`** to the doc-only stub in **RESEARCH Item 15** (the "RESET" variant). Then delete the premature modules:
   ```bash
   git rm crates/happyterminals-core/src/grid.rs \
          crates/happyterminals-core/src/reactive.rs \
          crates/happyterminals-core/src/python.rs
   ```
   (If any of these files are not `git`-tracked, use `rm` instead. Then `git add crates/happyterminals-core/src/lib.rs`.)

9. **Replace `crates/happyterminals-pipeline/src/lib.rs`** with the pipeline scaffold from RESEARCH Item 15 (doc-only; remove any pre-existing `Compositor` struct).

10. **Replace `crates/happyterminals-renderer/src/lib.rs`** with the renderer scaffold from RESEARCH Item 15 (doc-only).

11. **Verify no lingering `compositor` references** outside allowed areas:
    ```bash
    rg --hidden --glob '!target/**' --glob '!vendor/**' --glob '!.eclusa/**' \
       --glob '!.git/**' 'compositor'
    ```
    Must return nothing. (`.eclusa/` planning artifacts are exempt.)

12. **Verify member crates declare no version literals** (every shared dep must inherit from `[workspace.dependencies]`):
    ```bash
    # Any line of the form `name = "<digit>..."` inside a member [dependencies]
    # section indicates an un-inherited version literal — fail.
    ! rg '^\w+\s*=\s*"[0-9]' crates/*/Cargo.toml
    ```

13. **Build gate:**
    ```bash
    cargo check --workspace
    cargo fmt --all -- --check
    ```
    Both must pass with zero warnings. If `cargo fmt --check` complains on a `Cargo.toml`, that's expected (`rustfmt` doesn't touch TOML); only `.rs` files are in scope.

14. **Commit atomically** with the message below.

## Files touched
**Modified / rewritten:**
- `Cargo.toml` (root)
- `crates/happyterminals-core/Cargo.toml`
- `crates/happyterminals-core/src/lib.rs`
- `crates/happyterminals-renderer/Cargo.toml`
- `crates/happyterminals-renderer/src/lib.rs`

**Moved (git mv):**
- `crates/happyterminals-compositor/` → `crates/happyterminals-pipeline/`
  - Inside the moved directory: rewrite `Cargo.toml` and `src/lib.rs` per steps 4 and 9.

**Deleted:**
- `crates/happyterminals-core/src/grid.rs`
- `crates/happyterminals-core/src/reactive.rs`
- `crates/happyterminals-core/src/python.rs`

**Created (new files):**
- `crates/happyterminals-scene/Cargo.toml`
- `crates/happyterminals-scene/src/lib.rs`
- `crates/happyterminals-dsl/Cargo.toml`
- `crates/happyterminals-dsl/src/lib.rs`
- `crates/happyterminals-backend-ratatui/Cargo.toml`
- `crates/happyterminals-backend-ratatui/src/lib.rs`
- `crates/happyterminals/Cargo.toml`
- `crates/happyterminals/src/lib.rs`

## Commit message
```
refactor(workspace): resolver=3, [workspace.deps], rename compositor→pipeline, scaffold -scene/-dsl/-backend-ratatui/meta (HYG-02, HYG-03)

- Rewrite root Cargo.toml: resolver = "3", [workspace.package], pinned [workspace.dependencies], [workspace.lints]
- Rename crate happyterminals-compositor → happyterminals-pipeline (directory + Cargo.toml + lib.rs)
- Strip speculative deps from -core, -renderer, -pipeline (removed pyo3, tui-vfx, ratatui)
- Delete premature crates/happyterminals-core/src/{grid,reactive,python}.rs
- Add empty stub crates: happyterminals-scene, -dsl, -backend-ratatui, meta happyterminals
- Commented-out happyterminals-py workspace member (activated at M4)
- Every member inherits shared deps via { workspace = true }
```

## Success criteria (shell-observable)
```bash
# 1. resolver = "3" is in root Cargo.toml
grep -q '^resolver = "3"$' Cargo.toml

# 2. Seven active workspace members; one commented; py excluded
cargo metadata --format-version 1 --no-deps \
  | jq '.workspace_members | length' \
  | grep -qx 7

# 3. happyterminals-compositor is gone; happyterminals-pipeline exists
test ! -d crates/happyterminals-compositor
test -d crates/happyterminals-pipeline
grep -q '^name = "happyterminals-pipeline"$' crates/happyterminals-pipeline/Cargo.toml

# 4. No member crate has an inline version literal
! rg '^\w+\s*=\s*"[0-9]' crates/*/Cargo.toml

# 5. Premature -core modules are gone
test ! -f crates/happyterminals-core/src/grid.rs
test ! -f crates/happyterminals-core/src/reactive.rs
test ! -f crates/happyterminals-core/src/python.rs

# 6. Workspace compiles clean
cargo check --workspace 2>&1 | tee /tmp/check.log
! grep -E 'warning:|error:' /tmp/check.log

# 7. No stray "compositor" references outside vendor/ and .eclusa/
rg --hidden --glob '!target/**' --glob '!vendor/**' --glob '!.eclusa/**' \
   --glob '!.git/**' 'compositor' || echo "clean"
```

## Out of scope
- **Do not** publish or prepare any `cargo publish` / registry-reservation work — HYG-06 is deferred.
- **Do not** add `pyo3` to any crate's `[dependencies]`. It's pinned in `[workspace.dependencies]` so a future `happyterminals-py` drop-in is trivial, but no Phase 0 crate actually depends on it.
- **Do not** implement any reactive / Grid / renderer / pipeline types. Stubs are doc-only until Phase 1.0+.
- **Do not** touch `README.md`, `project.md`, per-crate READMEs, LICENSE files, CI, vendor/, or `rust-toolchain.toml` — those belong to plans 00-02 / 00-03 / 00-04 / 00-05.
- **Do not** rewrite `.eclusa/ROADMAP.md` or other planning artifacts — historical `compositor` references there are allowlisted.

## Open questions (implementer may decide at execution time)
- **Exact `[workspace.lints]` priority values.** RESEARCH Item 2 uses `priority = -1` for the `all`/`pedantic` groups so targeted `deny` lints override them; follow that verbatim unless Cargo rejects the ordering on 1.86 (it shouldn't — it's stable since 1.74).
- **Whether to pre-create empty `tests/` dirs per crate.** Not required for Phase 0; defer to the phase that adds the first test.
- **Whether to run `cargo generate-lockfile`** after the rename. `cargo check --workspace` regenerates `Cargo.lock` automatically; an explicit regeneration step is optional.
