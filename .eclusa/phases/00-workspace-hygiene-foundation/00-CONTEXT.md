# Phase 0: Workspace Hygiene & Foundation — Context

**Gathered:** 2026-04-14
**Status:** Ready for planning
**Source:** ROADMAP.md (Phase 0 scope fully specified; no discuss-phase needed — this is operational cleanup, not design)

<domain>
## Phase Boundary

**Goal:** Unblock all feature work by resolving stub-crate dep rot, vendor-dir debris, README drift, and missing OSS release plumbing.

**In scope:**
- Doc sweep (`tui-vfx` → `tachyonfx`)
- Reset speculative deps in stub crates
- Establish `[workspace.dependencies]` with pinned version set
- Crate rename/scaffold (`compositor` → `pipeline`; add `-scene`, `-dsl`, `-backend-ratatui`, meta, commented `-py`)
- Vendor relocation (`vendor/*` → `vendor/_reference/*` + `STAMP.txt` + `.gitattributes`)
- Dual-license files (`LICENSE-MIT`, `LICENSE-APACHE`) + SPDX on every crate
- `rust-toolchain.toml` pinned to Rust 1.86
- Baseline CI (fmt, clippy `-D warnings`, tests, docs, `cargo tree -d`, `cargo udeps`/`cargo-machete`, doc-lint)

**Out of scope (later phases):**
- Registry reservation on crates.io / PyPI (HYG-06) — user deferred; keep local for now. Revisit before M3 publish.
- Any actual feature code (reactive primitives, Grid, Pipeline, renderer — those are Phase 1.0+)
- Maturin / PyO3 wheel infrastructure (Phase 4.x)
- Qdrant docker-compose setup (environment concern, not Phase 0)

</domain>

<decisions>
## Implementation Decisions (locked)

### Doc Sweep (HYG-01)
- Replace `tui-vfx` with `tachyonfx` in `README.md`, `project.md`, per-crate READMEs.
- Preserve exactly **one** "why not tui-vfx" rationale section (inside PROJECT.md Key Decisions or a dedicated ADR).
- Also scrub `Haskell bindings` references (user descoped them).
- Also scrub `pyo3-asyncio`, `cgmath`, `tui-rs` references.

### Stub Crate Dep Reset (HYG-02)
- `crates/happyterminals-core/Cargo.toml`: remove `pyo3`, `tui-vfx`, `ratatui` from dependencies.
- `crates/happyterminals-renderer/Cargo.toml`: same treatment — strip speculative deps.
- `crates/happyterminals-compositor/Cargo.toml`: same; also rename to `happyterminals-pipeline`.
- Stub `lib.rs` files stay minimal (empty module or single doc comment) until real call sites exist.

### Workspace Dependencies (HYG-03)
Pinned versions to add to root `Cargo.toml` `[workspace.dependencies]`:

- `ratatui-core = "0.1"` (libs consume this)
- `ratatui = { version = "0.30", features = ["crossterm"] }` (apps/examples only)
- `crossterm = "0.29"`
- `tachyonfx = "0.25"`
- `glam = "0.32.1"`
- `reactive_graph = "0.2.13"`
- `any_spawner = "0.3"`
- `pyo3 = { version = "0.28.3", default-features = false }` (only `-py` consumes)
- `pyo3-async-runtimes = "0.28"` (never `pyo3-asyncio`)
- `schemars = "1.2"`
- `jsonschema = "0.46"`
- `thiserror = "2.0"`
- `color-eyre = "0.6.5"` (bins/examples only)
- `compact_str = "0.9"`
- `bon = "3.9"`
- `tobj = "4.0"`
- `stl_io = "0.11"` (deferred to M2 but pin now)
- `serde = { version = "1", features = ["derive"] }`
- `serde_json = "1"`
- `tracing = "0.1"`
- `tracing-subscriber = "0.3"`
- `insta = "1.47"` (dev-dep)
- `proptest = "1.11"` (dev-dep)
- `criterion = "0.8"` (dev-dep)

All member crates declare dependencies via `{ workspace = true }`.

### Crate Scaffold (HYG-03 cont'd)
- Rename `compositor` → `pipeline` (directory rename + `Cargo.toml` name change + remove any references).
- Add empty crates: `happyterminals-scene`, `happyterminals-dsl`, `happyterminals-backend-ratatui`.
- Add meta crate `happyterminals` (re-exports; empty for now).
- Add commented-out `happyterminals-py` entry in workspace members (activated at M4).

### Vendor Relocation (HYG-04)
- Move `vendor/pyo3/` → `vendor/_reference/pyo3/` + `vendor/_reference/pyo3/STAMP.txt` with upstream commit SHA + date + source URL.
- Move `vendor/ratatui/` → `vendor/_reference/ratatui/` + `STAMP.txt`.
- Move `vendor/tui-vfx/` → `vendor/_reference/tui-vfx/` + `STAMP.txt`.
- Add `.gitattributes` marking `vendor/_reference/** linguist-vendored=true`.
- **Never** reference vendored copies via `path =` in any `Cargo.toml` — they are for reading only.

### Dual License (HYG-05)
- `LICENSE-MIT` at repo root (standard MIT text, **"Copyright (c) 2026 the happyterminals authors"** per user decision 2026-04-14 — avoids name churn as contributors join).
- `LICENSE-APACHE` at repo root (standard Apache-2.0 text).
- Every crate's `Cargo.toml` uses SPDX `license = "MIT OR Apache-2.0"`.
- README.md gets a License section: "Licensed under either of Apache License, Version 2.0 or MIT License at your option" + the Apache-2.0 §5 contribution clause.
- `CONTRIBUTING.md` repeats the contribution clause.

### Registry Reservation (HYG-06) — **DEFERRED OUT OF PHASE 0**
- **User decision (2026-04-14):** keep the project local for now; do not prepare or publish reservation placeholders in Phase 0.
- Revisit at a later phase (likely before M3 publish). When revived, the plan will prepare placeholder packages and hand off to the user for `cargo publish` / `twine upload`.
- HYG-06 is therefore NOT an exit criterion of Phase 0 — remove from the phase checklist.
- Project root `Cargo.toml` crate names and PyPI names are still chosen consistently (`happyterminals`, `happyterminals-core`, `happyterminals-pipeline`, `happyterminals-renderer`, `happyterminals-scene`, `happyterminals-dsl`, `happyterminals-backend-ratatui`, `happyterminals-py`) so that a future reservation step is drop-in.

### Rust Toolchain (HYG-07)
- Add `rust-toolchain.toml` at repo root:
  ```toml
  [toolchain]
  channel = "1.86"
  components = ["clippy", "rustfmt"]
  profile = "default"
  ```
- Current local toolchain is 1.92; pinning 1.86 as MSRV floor. `rustup` will auto-install.

### Baseline CI (HYG-08, HYG-09)
- GitHub Actions workflow `.github/workflows/ci.yml` triggered on `push` + `pull_request`:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
  - `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS="-D warnings"`
  - `cargo tree --workspace --duplicates` must be empty
  - `cargo udeps --workspace` OR `cargo-machete`
  - Doc-lint step: custom script that greps for forbidden strings (`tui-vfx`, `Haskell bindings`, `pyo3-asyncio`, `cgmath`, `tui-rs`) anywhere in the repo except `vendor/_reference/` and a single `docs/decisions/rationale.md` (or similar) allow-list.
- Matrix: `ubuntu-latest`, Rust 1.86 + stable. macOS/Windows runners deferred to M2 (cross-terminal matrix).

### Claude's Discretion
- Exact structure of the doc-lint script (shell vs small Rust binary — default: shell with `rg`).
- Whether to enforce `forbid(unsafe_code)` at workspace level (recommend: yes for `-core`, `-pipeline`, `-scene`, `-dsl`; allow in `-renderer` for potential SIMD later).
- Exact copyright holder name on LICENSE-MIT — defaults to git user "Nathan Ribeiro"; user can override at execution time.
- Whether to commit a `.cargo/config.toml` with `[target.*.rustflags]` lints deny list (recommend: yes, keeps lints DRY).
- Whether to set up `release-plz` or keep manual release-tag workflow (defer the choice to M3's publish phase).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project context
- `.eclusa/PROJECT.md` — identity, core value, key decisions
- `.eclusa/REQUIREMENTS.md` — HYG-01 through HYG-09 definitions
- `.eclusa/ROADMAP.md` §"Phase 0" — the authoritative phase spec
- `.eclusa/STATE.md` — current focus + open questions

### Research
- `.eclusa/research/STACK.md` — crate versions with justification, anti-recommendations, workspace Cargo.toml skeleton (§10)
- `.eclusa/research/SUMMARY.md` §"Prerequisite Cleanup Items" — the checklist form of Phase 0
- `.eclusa/research/PITFALLS.md` §2, §3, §21, §22, §32, §33 — the specific pitfalls this phase addresses

### External references
- https://doc.rust-lang.org/cargo/reference/workspaces.html (workspace.dependencies schema)
- https://spdx.org/licenses/ (SPDX license expressions)
- https://opensource.apple.com/source/apache2/apache2-32/httpd/ (Apache-2.0 text — canonical copy)
- https://www.apache.org/licenses/LICENSE-2.0 (Apache-2.0 text upstream)
- https://crates.io/policies (publishing policies, name reservation etiquette)
- https://packaging.python.org/en/latest/guides/trusted-publishing/ (PyPI Trusted Publishing — for later, but naming conventions apply now)

</canonical_refs>

<specifics>
## Specific Concerns

### Current state of the repo (verified)
- Cargo workspace exists at root, members: `happyterminals-core`, `-renderer`, `-compositor`.
- `crates/happyterminals-core/Cargo.toml` already lists `pyo3`, `tui-vfx`, `ratatui` — all MUST be stripped.
- `vendor/{pyo3,ratatui,tui-vfx}/` exist at root.
- `README.md` says "happyterminals-core depends on ratatui, tui-vfx, and pyo3" — stale, will be rewritten.
- `project.md` (manifesto) is retained as historical but PROJECT.md is the living source.
- `.tool-versions` pins `rust 1.92.0` locally (fine — higher than MSRV 1.86).
- No `LICENSE*` files.
- No `.github/` workflows.
- No `rust-toolchain.toml`.
- No `CONTRIBUTING.md`, no `CHANGELOG.md`.

### Human-executed items in this phase
- None. HYG-06 (registry reservation) is deferred; copyright holder is locked to "the happyterminals authors".

All Phase 0 work is scriptable and commit-automatable by agents.

### The Haskell-bindings reference — **SCRUB** (user decision 2026-04-14)
- `project.md` (manifesto) at repo root has a "Haskell Bindings" section and `M4: Haskell Bindings` in Phase 4 roadmap. **Rewrite it to match current scope**: remove the Haskell Bindings section entirely, replace the old Phase 4 with the current Milestone 4 (Python bindings as FINAL), drop the `tui-vfx` mention in the stack diagram (already uses `tachyonfx`), trim Phase 5 items (keep a brief mention that they're parked).
- Keep `project.md` as a shorter vision/manifesto doc — it's the "why" companion to `.eclusa/PROJECT.md`'s "what".
- Doc-lint forbidden-string rule applies to `project.md` — no exemption.

</specifics>

<deferred>
## Deferred to Later Phases

- Actual reactive primitives, Grid code, Pipeline code — Phase 1.0+
- Full `docs.rs` feature matrix builds — Phase 3.5 (first publish)
- Cross-OS CI runners (macOS, Windows) — Phase 2.4 (resize hardening + MSRV policy phase) and Phase 3.5 (publish)
- `cargo semver-checks` as a blocking CI step — Phase 3.5 (the publish phase; only meaningful after first release)
- `release-plz` vs manual release tooling decision — Phase 3.5

</deferred>

---

*Phase: 00-workspace-hygiene-foundation*
*Context gathered: 2026-04-14 (extracted from ROADMAP.md; no discuss-phase needed — operational cleanup)*
