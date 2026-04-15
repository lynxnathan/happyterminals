# Phase 0 — Plan Verification

**Phase:** 00-workspace-hygiene-foundation
**Verified:** 2026-04-14
**Verifier:** eclusa-plan-checker (goal-backward)
**Plans checked:** 5
**Final verdict:** **GO**

---

## Scope of Verification

Goal-backward audit against the Phase 0 exit criteria:

1. `cargo build --workspace` clean on Rust 1.86 with zero warnings.
2. `cargo tree -d` reports zero duplicate dependencies.
3. `grep -r "tui-vfx"` matches only `vendor/_reference/tui-vfx/` and one rationale section.
4. Both `LICENSE-MIT` and `LICENSE-APACHE` exist + SPDX on every crate.
5. HYG-06 **DEFERRED** per user decision — not a Phase 0 exit criterion.
6. CI baseline (fmt, clippy `-D warnings`, tests, docs `-D warnings`, `cargo tree -d`, `cargo-machete`, doc-lint) is green.

---

## HYG Requirement → Plan Coverage Matrix

| Req    | Primary plan | Secondary | Status |
|--------|--------------|-----------|--------|
| HYG-01 | 00-04        | —         | Covered |
| HYG-02 | 00-01        | —         | Covered |
| HYG-03 | 00-01        | —         | Covered |
| HYG-04 | 00-03        | —         | Covered |
| HYG-05 | 00-02        | 00-01 (SPDX via `[workspace.package]` inheritance) | Covered |
| HYG-06 | *(deferred)* | —         | **Correctly absent** |
| HYG-07 | 00-03        | —         | Covered |
| HYG-08 | 00-05        | —         | Covered |
| HYG-09 | 00-04 (script) | 00-05 (CI wiring) | Covered |

**All eight active HYG requirements covered exactly once by a primary plan. HYG-06 correctly absent from every plan's `requirements:` field and every step.**

---

## Per-plan verdicts

### 00-01 — Workspace Deps Refactor + Crate Scaffold → **PASS**

**Goal achievable?** Yes. Steps 2–13 deliver: compositor→pipeline rename via `git mv` (history preserved), root `Cargo.toml` rewrite (resolver=3, `[workspace.package]`, `[workspace.dependencies]` verbatim from RESEARCH Item 2, `[workspace.lints]`), strip speculative deps from `-core`/`-renderer`/`-pipeline`, delete `grid.rs`/`reactive.rs`/`python.rs`, scaffold `-scene`/`-dsl`/`-backend-ratatui`/meta, commented-out `-py`. Ordering is correct: `lib.rs` reset precedes the module `git rm` (step 8 resets lib.rs first, then `git rm` — no dangling `mod foo;` mid-step that would break `cargo check`).

**Requirements:** HYG-02 (speculative-dep strip + premature file deletion) and HYG-03 (`[workspace.dependencies]` + inheritance) — both delivered in-full.

**Correctness checks:**
- `resolver = "3"` explicitly specified; `edition = "2024"`; `rust-version = "1.86"` — all 2026 conventions honored.
- `pyo3 = { version = "0.28.3", default-features = false }` pinned but NOT added to any Phase 0 member's `[dependencies]` — matches CONTEXT.md lock.
- `pyo3-async-runtimes = "0.28"` used; NO `pyo3-asyncio`. ✓
- `cgmath` and `tui-rs` nowhere in the pinned set. ✓
- `authors = ["the happyterminals authors"]` in `[workspace.package]` — locked user decision honored.
- Step 11's `rg` check correctly excludes `.eclusa/**` (where historical "compositor" references are allowlisted).
- Step 12 proves no inline version literals in member `Cargo.toml` (enforces HYG-03 inheritance).

**Minor FLAG (non-blocking):** The `-renderer` variant in RESEARCH Item 4 (and referenced by step 6) opts out of `lints.workspace = true` to set `unsafe_code = "allow"`, and therefore re-declares the clippy lint list by hand. This is correct Cargo semantics (workspace `forbid` cannot be downgraded; you must fully opt out), and the duplication is explicitly documented, but it will silently drift from the workspace lint table if the workspace list changes later. Acceptable for Phase 0; worth a note at M2/M3 lint review.

**Observable success criteria:** all 7 are shell-executable commands.

---

### 00-02 — Licensing, Contribution, Changelog → **PASS**

**Goal achievable?** Yes. Four file creations from verbatim RESEARCH content (Items 1, 12, 13). No dependencies — truly wave-1 parallel.

**Requirements:** HYG-05 — LICENSE files + contribution clause + CHANGELOG — all in-scope. Correctly notes SPDX strings come from plan 00-01 via `[workspace.package]` inheritance (not this plan's job).

**User-decision compliance:**
- Copyright line hard-coded to `"Copyright (c) 2026 the happyterminals authors"`. Step 1 + success-criteria shell explicitly `! grep -q 'Nathan Ribeiro' LICENSE-MIT` and `LICENSE-APACHE`. ✓
- CONTEXT.md Discretion #3 ("copyright holder name can be overridden by implementer") is **correctly rejected** in favor of the locked user decision. The plan explicitly says "not 'Nathan Ribeiro'".
- No `cargo publish` / `twine` / reservation steps — HYG-06 correctly excluded. ✓

**Observable success criteria:** 5 shell blocks, all executable.

---

### 00-03 — Vendor Relocation + Toolchain Pin → **PASS**

**Goal achievable?** Yes. `git mv` for each vendored dir (history preserved), STAMP.txt from RESEARCH Item 6 template, `.gitattributes` with correct linguist-vendored glob + LF eol declarations, `rust-toolchain.toml` with `channel = "1.86"`, `components = ["clippy", "rustfmt"]`, `profile = "default"`.

**Requirements:** HYG-04 (vendor relocation) + HYG-07 (toolchain pin) — both in-scope.

**Correctness checks:**
- Step 8's regex `! rg '^\s*(\w[\w-]*)\s*=\s*\{[^}]*path\s*=\s*"[^"]*vendor/'` correctly enforces the "never reference vendored copies via `path =`" invariant — this directly targets the current `happyterminals-core/Cargo.toml` anti-pattern (`ratatui = { path = "../../vendor/ratatui" }` etc.), which plan 00-01 is stripping concurrently. No race: 00-01 removes those `path =` lines in its Cargo.toml rewrite (its step 5 "strip `pyo3`, `tui-vfx`, `ratatui` entries"), and 00-03's grep runs after its own relocation. If 00-03 commits before 00-01, the grep still passes because 00-03 does not change Cargo.toml; if 00-01 commits first, the grep passes because the `path =` lines are gone. Either order is safe.
- STAMP.txt placeholder status is explicit and honest; `Captured by: Nathan Ribeiro` in the provenance field is distinct from the LICENSE copyright line (provenance = snapshot captor; LICENSE = holder) — not a user-decision violation.
- `profile = "default"` (not `"minimal"`) per CONTEXT.md lock. ✓
- No `targets = [...]` (CONTEXT.md constraint honored). ✓
- Step 9 "Do not add `.cargo/config.toml`" — correct; CONTEXT.md Discretion #4 suggested optional `.cargo/config.toml` for lints, but RESEARCH Item 2 puts all lints in `[workspace.lints]`, and the plan correctly rejects the extra file. 2026 convention honored.

**Observable success criteria:** 7 shell checks. ✓

---

### 00-04 — Docs Rewrite + Doc-Lint Script → **PASS**

**Goal achievable?** Yes. README.md rewrite (RESEARCH Item 10), `project.md` **scrubbed rewrite** (RESEARCH Item 9, ~150 lines vs current 343 — explicitly a rewrite, not a banner-archive ✓), seven per-crate READMEs (Item 11), `docs/decisions/stack-rationale.md` allowlist (Item 8), `scripts/doc-lint.sh` (Item 8 script block, chmod +x, `git add --chmod=+x`).

**Requirements:** HYG-01 (doc consistency) + HYG-09 (script asset) — both delivered. CI wiring correctly deferred to 00-05.

**Dependency correctness:** Wave 2, depends on 00-01 — correct, because per-crate READMEs need the renamed `-pipeline` dir and the four new crate dirs to exist on disk. If 00-04 ran before 00-01, `crates/happyterminals-pipeline/README.md` would land in a non-existent directory (or worse, in `-compositor/` and then get `git mv`-ed to the new location, bloating the diff). The wave-2 constraint is necessary.

**Correctness checks:**
- Doc-lint `FORBIDDEN=(tui-vfx "Haskell bindings" pyo3-asyncio cgmath tui-rs)` matches REQUIREMENTS.md HYG-09 exactly.
- `EXCLUDES` allowlist includes `!.eclusa/**` (planning artifacts), `!scripts/doc-lint.sh` (self), `!.github/workflows/ci.yml` (for CI step names), `!CHANGELOG.md` (migration notes), `!docs/decisions/stack-rationale.md` (sole prose allowlist), `!vendor/_reference/**`, `!target/**`, `!Cargo.lock`. Nothing gratuitous.
- Step 6 self-tests the script locally before commit; step 5's negative-test (append forbidden string to a temp file, confirm script exits 1, then revert) is a genuine observable check.
- `project.md` rewrite is explicitly a **scrub**, not banner-archive, per user constraint. Success criteria `! grep -qi 'tui-vfx' project.md` + `! grep -qi 'haskell bindings' project.md` prove this. ✓
- No `cgmath`, `tui-rs`, `pyo3-asyncio` references outside the stack-rationale allowlist. ✓

**Observable success criteria:** 5 shell blocks, including negative doc-lint test. ✓

---

### 00-05 — GitHub Actions CI → **PASS**

**Goal achievable?** Yes. Creates `.github/workflows/ci.yml` with five jobs — `fmt`, `clippy`, `test` (matrix 1.86 + stable, `fail-fast: false`), `docs`, `hygiene` (tree --duplicates + cargo-machete + doc-lint). All RESEARCH Item 7 conventions honored.

**Requirements:** HYG-08 (CI baseline) + HYG-09 (wire doc-lint into CI) — both delivered.

**Dependency correctness:** Wave 3, depends on 00-01 + 00-04. Correct: workspace must exist for `cargo clippy --workspace` to succeed; `scripts/doc-lint.sh` must exist for the hygiene step to find it. Soft dependency on 00-03 for `rust-toolchain.toml` (plan explicitly pins `toolchain: "1.86"` in CI as a belt-and-suspenders — CI works either way).

**Exit-criterion #6 coverage (the big one):** every required step is present —
- `cargo fmt --all -- --check` ✓ (fmt job)
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` ✓ (clippy job)
- `cargo test --workspace --all-features --no-fail-fast` ✓ (test job, matrix 1.86 + stable)
- `cargo doc --workspace --no-deps --all-features` with `RUSTDOCFLAGS: "-D warnings"` ✓ (docs job)
- `cargo tree --workspace --duplicates --edges normal --format '{p}'` with non-empty failure ✓ (hygiene step 1)
- `cargo-machete` via `taiki-e/install-action@v2` → `cargo machete` ✓ (hygiene steps 2–3)
- `bash scripts/doc-lint.sh` ✓ (hygiene step 4)

**2026 conventions:**
- `Swatinem/rust-cache@v2` ✓
- `dtolnay/rust-toolchain@master` ✓
- `taiki-e/install-action@v2` ✓
- `cargo-machete` (stable-compatible) **NOT** `cargo-udeps` (nightly-only, violates 1.86 pin). Explicitly enforced in success criteria step 4: `! grep -q 'cargo-udeps' .github/workflows/ci.yml`. ✓
- `concurrency` group with `cancel-in-progress: true` ✓
- `ubuntu-latest` only (macOS/Windows deferred to M2) ✓

**Observable success criteria:** 9 shell checks including YAML parse, job presence, anti-patterns. ✓

---

## Cross-plan checks

### File conflicts — None

Disjoint file sets across all five plans:

| Plan | Owns |
|------|------|
| 00-01 | `Cargo.toml` (root), `crates/*/Cargo.toml`, `crates/*/src/lib.rs` |
| 00-02 | `LICENSE-MIT`, `LICENSE-APACHE`, `CONTRIBUTING.md`, `CHANGELOG.md` |
| 00-03 | `vendor/**`, `.gitattributes`, `rust-toolchain.toml` |
| 00-04 | `README.md`, `project.md`, `crates/*/README.md`, `docs/**`, `scripts/doc-lint.sh` |
| 00-05 | `.github/workflows/ci.yml` |

No two plans write the same file. No conflicting intents.

### Dependency DAG — Valid and acyclic

- **Wave 1** (parallel, `depends_on: []`): 00-01, 00-02, 00-03
- **Wave 2** (`depends_on: [00-01]`): 00-04
- **Wave 3** (`depends_on: [00-01, 00-04]`): 00-05

No cycles. No forward references. Wave assignment consistent with `depends_on` (wave = max(deps.wave) + 1). ✓

### Race-condition check — Clean

- **00-01 ↔ 00-03:** both wave-1. 00-01 strips `path = "../../vendor/ratatui"` from `crates/happyterminals-core/Cargo.toml`; 00-03 moves `vendor/ratatui/` to `vendor/_reference/ratatui/`. If 00-03 lands first with the `path =` line still present in Cargo.toml, `cargo metadata` / `cargo check` would fail on the next build — but both plans commit independently without running the other's build gate, and the final unified branch has 00-01's Cargo.toml fix + 00-03's relocation. No integration window where the workspace is unbuildable if merged atomically (e.g., via a Phase 0 integration branch that lands all three wave-1 plans together). **Safe**, but the phase executor should merge wave 1 as a batch, not plan-by-plan into main.
- **00-01's `cargo check --workspace` gate:** runs during 00-01 execution. At that point, `vendor/` still has `vendor/ratatui/` (00-03 hasn't run yet), but 00-01 has already removed the `path = "../../vendor/ratatui"` line from `crates/happyterminals-core/Cargo.toml` (step 5) — so the `vendor/` path is no longer referenced. `cargo check` uses registry sources only. ✓
- **00-04 ↔ 00-01:** wave-2 sequencing correctly solves the "new crate dirs must exist" dependency.
- **00-05 ↔ 00-04:** wave-3 sequencing correctly solves the "script must exist before CI references it" dependency.

### Red-flag scan — All clear

- `cargo publish`, `twine`, `reservation`: only in explicit **"Do not"** out-of-scope blocks. ✓
- `Nathan Ribeiro` in LICENSE: negative-asserted by 00-02 success criteria. ✓
- `Nathan Ribeiro` in STAMP.txt `Captured by:` field: this is snapshot-captor provenance, not copyright holder — not a user-decision violation.
- `resolver = "2"`: only as historical "current state we're upgrading from". ✓
- `cargo-udeps` / `cargo udeps`: only in negations + "Do not" blocks. ✓
- `pyo3-asyncio`, `cgmath`, `tui-rs`: appear only in forbidden-string lists / rationale docs. ✓
- `tui-vfx` dep anywhere in pinned workspace: absent. ✓
- `pyo3` in `-core/Cargo.toml [dependencies]`: explicitly stripped by 00-01 step 5. ✓

### 2026 convention compliance — All honored

| Convention | Expected | Plan status |
|------------|----------|-------------|
| Workspace resolver | `"3"` | 00-01 verbatim; negative check in success criteria |
| Edition | `"2024"` | 00-01 via `[workspace.package]` |
| Python async runtime | `pyo3-async-runtimes` | 00-01 pin; 00-04 forbidden-string rule forbids `pyo3-asyncio` |
| CI cache action | `Swatinem/rust-cache@v2` | 00-05 ✓ |
| Tool installer | `taiki-e/install-action@v2` | 00-05 ✓ |
| Unused-dep scanner | `cargo-machete` | 00-05 ✓ |
| Graphics math | `glam` | 00-01 pin; `cgmath` forbidden |
| TUI crate | `ratatui` | 00-01 pin; `tui-rs` forbidden |
| Lint location | `[workspace.lints]` | 00-01 ✓; 00-03 explicitly rejects `.cargo/config.toml` for lints |

---

## Exit-criterion coverage (phase-level)

| Exit criterion | Achieved by |
|----------------|-------------|
| 1. `cargo build --workspace` clean on Rust 1.86, zero warnings | 00-01 (workspace builds on pinned toolchain) + 00-03 (toolchain pin). Gate lives in 00-01 step 13 (`cargo check --workspace`) and is re-verified by 00-05's CI `clippy -D warnings` + `test` jobs. |
| 2. `cargo tree -d` zero duplicates | 00-01 (single pinned version set via `[workspace.dependencies]`) + 00-05 (CI hygiene job fails on any duplicate). |
| 3. `grep -r "tui-vfx"` matches only `vendor/_reference/tui-vfx/` + one rationale section | 00-03 (vendor relocation) + 00-04 (doc scrub + `docs/decisions/stack-rationale.md` as sole prose allowlist) + 00-05 (CI doc-lint enforces on every push). |
| 4. LICENSE-MIT + LICENSE-APACHE + SPDX on every crate | 00-02 (LICENSE files) + 00-01 (SPDX via `[workspace.package] license = "MIT OR Apache-2.0"` inheritance). |
| 5. HYG-06 deferred (not an exit criterion) | *(correctly absent from all plans)* |
| 6. CI baseline green | 00-05 (all 7 required steps present: fmt, clippy `-D warnings`, test, doc `-D warnings`, `cargo tree -d`, `cargo-machete`, `bash scripts/doc-lint.sh`). |

All six exit criteria demonstrably covered by the plan set.

---

## Final Phase Verdict: **GO**

All five plans pass. Coverage is complete, dependencies are sound, 2026 conventions are honored, user decisions (copyright = "the happyterminals authors"; HYG-06 deferred; `project.md` scrubbed not archived; premature `-core` modules deleted) are all respected, and anti-recommendations (`tui-vfx`, `pyo3-asyncio`, `cgmath`, `tui-rs`, `Nathan Ribeiro` in LICENSE) are not reintroduced.

### Execution note

Phase 0 is safe to execute as three waves:

1. **Wave 1 (parallel):** 00-01, 00-02, 00-03. Recommend landing as a single integration batch onto the phase branch, not plan-by-plan into `main`, so the intermediate `vendor/ratatui/` + stripped-`path=` window never surfaces.
2. **Wave 2:** 00-04 (after 00-01 is in the branch).
3. **Wave 3:** 00-05 (after 00-01 + 00-04).

After Wave 3 lands, the CI on the phase branch is the authoritative gate for Phase 0 exit. A single post-merge local run of `cargo build --workspace`, `cargo tree --workspace --duplicates --edges normal`, and `bash scripts/doc-lint.sh` provides the same confidence as GitHub Actions for the initial push.

**Execute.**

---

*Verification performed against plan frontmatter, step text, and cross-referenced RESEARCH.md items 1–15. No code was executed; plans were statically analyzed against the phase goal and locked user decisions from CONTEXT.md (2026-04-14).*
