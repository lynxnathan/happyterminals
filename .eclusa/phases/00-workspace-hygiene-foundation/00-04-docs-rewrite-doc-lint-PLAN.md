---
phase: 00-workspace-hygiene-foundation
plan: 04
type: execute
wave: 2
depends_on: [00-01]
files_modified:
  - README.md
  - project.md
  - crates/happyterminals/README.md
  - crates/happyterminals-core/README.md
  - crates/happyterminals-renderer/README.md
  - crates/happyterminals-pipeline/README.md
  - crates/happyterminals-scene/README.md
  - crates/happyterminals-dsl/README.md
  - crates/happyterminals-backend-ratatui/README.md
  - docs/decisions/stack-rationale.md
  - scripts/doc-lint.sh
autonomous: true
requirements: [HYG-01, HYG-09]

must_haves:
  truths:
    - "`README.md` describes the current stack (ratatui, tachyonfx, reactive_graph, glam) with no `tui-vfx`, `Haskell bindings`, `pyo3-asyncio`, `cgmath`, or `tui-rs` references; crates table uses `-pipeline` (not `-compositor`)."
    - "`project.md` is rewritten (scrubbed, not banner-archived): ≤ ~150 lines, no `tui-vfx`, no Haskell section, no `Phase 5` speculative detail beyond a brief parked mention."
    - "Every workspace member has a `crates/<name>/README.md` matching RESEARCH Item 11."
    - "`docs/decisions/stack-rationale.md` exists and is the single place forbidden terms may appear in prose."
    - "`scripts/doc-lint.sh` exists, is executable, and fails if any forbidden string appears outside the allowlist (vendor/_reference/**, docs/decisions/stack-rationale.md, .eclusa/**, scripts/doc-lint.sh itself, .github/workflows/ci.yml, CHANGELOG.md, target/, Cargo.lock)."
  artifacts:
    - path: "README.md"
      provides: "Project README with badges, crates table (using -pipeline), License section + Apache-2.0 §5 clause"
      contains: "MIT OR Apache-2.0"
    - path: "project.md"
      provides: "Rewritten manifesto (≤ ~150 lines), Haskell-scrubbed, tachyonfx throughout"
      contains: "happyterminals"
    - path: "scripts/doc-lint.sh"
      provides: "Forbidden-string scanner using rg, exits 1 on violation, 0 clean"
      contains: "FORBIDDEN="
    - path: "docs/decisions/stack-rationale.md"
      provides: "Sole allowlist file for prose mentioning forbidden terms"
      contains: "Why not"
  key_links:
    - from: "scripts/doc-lint.sh"
      to: "docs/decisions/stack-rationale.md"
      via: "EXCLUDES glob list; rationale file is explicitly excluded"
      pattern: "!docs/decisions/stack-rationale.md"
    - from: "crates/<name>/Cargo.toml `readme = \"README.md\"`"
      to: "crates/<name>/README.md"
      via: "relative path resolved by Cargo and crates.io"
      pattern: "readme = \"README.md\""
---

# Plan 00-04 — Docs Rewrite + Doc-Lint Script

## Goal
Rewrite `README.md`, scrub-rewrite `project.md`, drop a per-crate `README.md` in each workspace member, create the allowlist file `docs/decisions/stack-rationale.md`, and ship `scripts/doc-lint.sh` — the forbidden-string scanner that CI (plan 00-05) will invoke.

## Requirements covered
- **HYG-01** — README/project.md/per-crate READMEs consistently describe `tachyonfx`, no stray `tui-vfx` or `Haskell bindings` references outside the single rationale file.
- **HYG-09** (asset) — the `scripts/doc-lint.sh` script itself. The CI invocation of this script is owned by plan 00-05.

## Dependencies
- **00-01 must land before this plan.** Per-crate READMEs need the renamed `-pipeline` directory and the new `-scene`/`-dsl`/`-backend-ratatui`/meta `happyterminals` crates to exist on disk. This plan is therefore Wave 2 (after 00-01's Wave 1).

## Parallelizable with
- **00-02** (licensing) — disjoint files.
- **00-03** (vendor relocation + toolchain) — disjoint files.

**Sequential before:** 00-05 (CI workflow references `scripts/doc-lint.sh`).

## Steps

> All content lives in **RESEARCH.md**; copy verbatim. Execute from repo root.

1. **Rewrite `README.md`** at repo root with the exact content from **RESEARCH Item 10**. Confirm:
   - CI / License / Rust badges present at top (badge URLs reference `github.com/lynxnathan/happyterminals`)
   - Stack section lists `ratatui`, `tachyonfx`, `reactive_graph`, `glam` and "Fresh ASCII rasterizer"
   - Crates table uses `happyterminals-pipeline` (not `-compositor`) and includes the four new crates + meta + commented-out `-py` row
   - Quick Start code sample uses `use happyterminals::prelude::*;` + `run(scene, FrameSpec::fps(30))`
   - Development section lists `cargo build --workspace`, `cargo test`, `cargo clippy -D warnings`, `cargo fmt`, `cargo doc`, `cargo tree --duplicates`, `cargo-machete`
   - **License section** with Apache-2.0 §5 clause (verbatim Rust-ecosystem boilerplate)

2. **Rewrite `project.md`** at repo root with the content from **RESEARCH Item 9**. This is a **scrub** (rewrite), **not** a banner-archive. The new file:
   - Is ~150 lines (under half of the current 343)
   - Keeps manifesto tone
   - Contains no `Haskell Bindings` section
   - Contains no `tui-vfx` mention (the rationale for choosing tachyonfx lives in `docs/decisions/stack-rationale.md`, not here)
   - Trims Phase 5 to a brief "Parked (post-v1)" paragraph
   - Uses `tachyonfx` throughout the stack diagram
   - Lists the current 8 crates in the Components section (`-core`, `-renderer`, `-pipeline`, `-scene`, `-dsl`, `-backend-ratatui`, meta `happyterminals`, `-py` final)

3. **Create `crates/<name>/README.md` for every workspace member** using the per-crate templates in **RESEARCH Item 11**. One file per crate:
   - `crates/happyterminals/README.md` — meta crate blurb with prelude hint
   - `crates/happyterminals-core/README.md` — reactive + Grid; explicit "pyo3 is NOT a dependency of this crate" line
   - `crates/happyterminals-renderer/README.md` — fresh rasterizer, not a fork
   - `crates/happyterminals-pipeline/README.md` — Effect trait, Pipeline, TachyonAdapter; notes `tachyonfx::Effect` is aliased as `Fx`
   - `crates/happyterminals-scene/README.md` — SceneIr + scene graph
   - `crates/happyterminals-dsl/README.md` — builder + JSON recipes
   - `crates/happyterminals-backend-ratatui/README.md` — tokio::select loop + TerminalGuard
   Each file ends with `Dual-licensed under MIT OR Apache-2.0.`

4. **Create `docs/decisions/stack-rationale.md`** with the content from **RESEARCH Item 8** (the "Allowlist file" block). This is the **sole** allowlist for the doc-lint step. Sections:
   - Why not tui-vfx (chose tachyonfx)
   - Why not pyo3-asyncio (chose pyo3-async-runtimes)
   - Why not cgmath (chose glam)
   - Why not tui-rs (chose ratatui)
   - Why not Haskell bindings (chose Python-only)
   `mkdir -p docs/decisions` first.

5. **Create `scripts/doc-lint.sh`** with the content from **RESEARCH Item 8** (the shell script block). Requirements:
   - First line `#!/usr/bin/env bash`
   - Uses `set -euo pipefail`
   - `FORBIDDEN=(tui-vfx "Haskell bindings" pyo3-asyncio cgmath tui-rs)`
   - `EXCLUDES` glob list covers: `!vendor/_reference/**`, `!docs/decisions/stack-rationale.md`, `!.eclusa/**`, `!scripts/doc-lint.sh`, `!.github/workflows/ci.yml`, `!CHANGELOG.md`, `!target/**`, `!Cargo.lock`
   - Uses `rg --hidden --line-number --no-heading --fixed-strings` with `--glob` negations
   - Emits `::error::` annotations (GitHub Actions picks these up)
   - Exit 0 on clean, 1 on any violation
   - **`mkdir -p scripts` first**, then write the file, then `chmod +x scripts/doc-lint.sh`.
   - `git add --chmod=+x scripts/doc-lint.sh` so the executable bit is committed.

6. **Self-test the doc-lint script** from repo root:
   ```bash
   bash scripts/doc-lint.sh
   ```
   Must exit 0 and print `Doc-lint: clean.`. If it reports hits, investigate: most likely a leftover `tui-vfx` / `Haskell bindings` string in a README or `project.md` that wasn't fully scrubbed. Fix the source — do NOT widen the allowlist.

7. **Commit atomically** with the message below.

## Files touched

**Created:**
- `crates/happyterminals/README.md`
- `crates/happyterminals-core/README.md`
- `crates/happyterminals-renderer/README.md`
- `crates/happyterminals-pipeline/README.md`
- `crates/happyterminals-scene/README.md`
- `crates/happyterminals-dsl/README.md`
- `crates/happyterminals-backend-ratatui/README.md`
- `docs/decisions/stack-rationale.md`
- `scripts/doc-lint.sh` (mode 755)

**Rewritten (replaced wholesale):**
- `README.md`
- `project.md`

**Deleted:** none.

## Commit message
```
docs(phase0): rewrite README + project.md, add per-crate READMEs, stack-rationale, doc-lint script (HYG-01, HYG-09)

- Rewrite README.md: current stack (ratatui/tachyonfx/reactive_graph/glam), crates
  table uses -pipeline, License section with Apache-2.0 §5 clause.
- Rewrite project.md: scrub Haskell-bindings section, tui-vfx references, and
  Phase 5 speculative detail. Trimmed from 343 lines to ~150.
- Add one README.md per workspace member (seven crates).
- Add docs/decisions/stack-rationale.md — the sole allowlist for forbidden terms
  ("Why not X" rationale for tui-vfx / pyo3-asyncio / cgmath / tui-rs / Haskell).
- Add scripts/doc-lint.sh (mode 755) — rg-based forbidden-string scanner used
  by CI's hygiene job (see plan 00-05).
```

## Success criteria (shell-observable)
```bash
# 1. Root README and project.md exist with expected content markers
test -f README.md
test -f project.md
grep -q 'tachyonfx' README.md
grep -q 'MIT OR Apache-2.0' README.md
grep -q 'happyterminals-pipeline' README.md
! grep -qi 'tui-vfx' README.md
! grep -qi 'haskell bindings' README.md
! grep -qi 'tui-vfx' project.md
! grep -qi 'haskell bindings' project.md

# 2. Every workspace member has a README
for c in happyterminals happyterminals-core happyterminals-renderer \
         happyterminals-pipeline happyterminals-scene happyterminals-dsl \
         happyterminals-backend-ratatui; do
  test -f "crates/$c/README.md"
  grep -q 'Dual-licensed under MIT OR Apache-2.0' "crates/$c/README.md"
done

# 3. Rationale file exists with the five "Why not" sections
test -f docs/decisions/stack-rationale.md
grep -q 'Why not tui-vfx' docs/decisions/stack-rationale.md
grep -q 'Why not pyo3-asyncio' docs/decisions/stack-rationale.md
grep -q 'Why not cgmath' docs/decisions/stack-rationale.md
grep -q 'Why not tui-rs' docs/decisions/stack-rationale.md
grep -q 'Why not Haskell bindings' docs/decisions/stack-rationale.md

# 4. Doc-lint script exists, is executable, and passes
test -f scripts/doc-lint.sh
test -x scripts/doc-lint.sh
bash scripts/doc-lint.sh   # must exit 0; prints "Doc-lint: clean."

# 5. Intentional negative test — inject a forbidden string temporarily, verify
#    doc-lint fails, then revert.
echo 'tui-vfx' >> /tmp/doc-lint-test.md
cp /tmp/doc-lint-test.md ./__doc_lint_probe.md
! bash scripts/doc-lint.sh   # should exit 1
rm -f ./__doc_lint_probe.md
bash scripts/doc-lint.sh     # clean again
```

## Out of scope
- **Do not** add CI workflow files — plan 00-05 owns `.github/workflows/ci.yml`.
- **Do not** modify `Cargo.toml` or `[workspace.package]` — those belong to 00-01.
- **Do not** add LICENSE files, CONTRIBUTING.md, or CHANGELOG.md — those belong to 00-02.
- **Do not** modify `.eclusa/**` planning artifacts (historical `compositor`, `tui-vfx`, `Haskell` mentions in `.eclusa/` are explicitly allowlisted — scrubbing them is churn without value).
- **Do not** widen the doc-lint allowlist to accept new files. If a rewrite leaves a forbidden string in a user-facing doc, **fix the source**, not the allowlist.
- **Do not** rewrite `vendor/_reference/**` content (those are reference copies of external source; they keep their original language).

## Open questions (implementer may decide at execution time)
- **Badge URLs in README.md.** RESEARCH Item 10 uses `github.com/lynxnathan/happyterminals` as the repo URL. If the final owner differs, update the three badge URLs consistently. Default: use the Item-10 values verbatim.
- **Implementation language of `scripts/doc-lint.sh`.** CONTEXT.md allows the implementer to pick shell-with-rg or a small Rust binary. **Default (per RESEARCH Item 8): shell with `rg`**, because the CI runner already has both and the script is ~50 lines. Changing the default requires a conscious decision and a corresponding update to the CI step in plan 00-05.
- **Whether to add the `docs/decisions/` folder's own README.** Not required; defer unless/until more ADRs land.
