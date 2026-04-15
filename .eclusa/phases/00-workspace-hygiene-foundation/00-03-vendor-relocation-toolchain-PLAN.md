---
phase: 00-workspace-hygiene-foundation
plan: 03
type: execute
wave: 1
depends_on: []
files_modified:
  - vendor/pyo3/          # moved -> vendor/_reference/pyo3/
  - vendor/ratatui/       # moved -> vendor/_reference/ratatui/
  - vendor/tui-vfx/       # moved -> vendor/_reference/tui-vfx/
  - vendor/_reference/pyo3/STAMP.txt
  - vendor/_reference/ratatui/STAMP.txt
  - vendor/_reference/tui-vfx/STAMP.txt
  - .gitattributes
  - rust-toolchain.toml
autonomous: true
requirements: [HYG-04, HYG-07]

must_haves:
  truths:
    - "`vendor/pyo3/`, `vendor/ratatui/`, `vendor/tui-vfx/` are moved under `vendor/_reference/` and each directory contains a `STAMP.txt` recording upstream URL/version/commit/date."
    - "`.gitattributes` marks `vendor/_reference/**` as `linguist-vendored=true`, sets LF line endings for `*.rs`/`*.toml`/`*.md`/`*.yml`/`*.yaml`/`Cargo.lock`, and declares common binary fixtures."
    - "`rust-toolchain.toml` pins `channel = \"1.86\"` with `components = [\"clippy\", \"rustfmt\"]` and `profile = \"default\"`."
    - "No `Cargo.toml` in the workspace references any `vendor/_reference/**` via `path = \"...\"` — vendored copies are reading-only."
  artifacts:
    - path: "vendor/_reference/pyo3/STAMP.txt"
      provides: "Upstream provenance stamp for pyo3 reference copy"
      contains: "Upstream:"
    - path: "vendor/_reference/ratatui/STAMP.txt"
      provides: "Upstream provenance stamp for ratatui reference copy"
      contains: "Upstream:"
    - path: "vendor/_reference/tui-vfx/STAMP.txt"
      provides: "Upstream provenance stamp for tui-vfx reference copy (rationale anchor for tachyonfx pivot)"
      contains: "Preserved as reference"
    - path: ".gitattributes"
      provides: "LF line endings + linguist-vendored directives"
      contains: "linguist-vendored=true"
    - path: "rust-toolchain.toml"
      provides: "Pinned toolchain 1.86 + clippy + rustfmt"
      contains: "channel = \"1.86\""
  key_links:
    - from: "rustup"
      to: "rust-toolchain.toml"
      via: "rustup auto-installs 1.86 on first cargo invocation"
      pattern: "rustc --version"
    - from: "GitHub linguist"
      to: "vendor/_reference/**"
      via: ".gitattributes linguist-vendored=true glob"
      pattern: "linguist-vendored=true"
---

# Plan 00-03 — Vendor Relocation + Toolchain Pin

## Goal
Move every `vendor/<name>/` directory under `vendor/_reference/` with a `STAMP.txt` provenance file, publish `.gitattributes` so GitHub linguist ignores vendored content and source files stay on LF line endings, and pin the Rust toolchain to 1.86 via `rust-toolchain.toml`.

## Requirements covered
- **HYG-04** — vendor relocation with STAMP.txt + `.gitattributes` linguist-vendored marking; vendored copies never referenced via `path =` deps.
- **HYG-07** — `rust-toolchain.toml` pins Rust 1.86 with clippy + rustfmt components.

## Dependencies
None. No coupling to Cargo or crate layout; `rust-toolchain.toml` is a rustup-level file and does not require the new workspace to be in place.

## Parallelizable with
- **00-01** (workspace refactor) — disjoint.
- **00-02** (licensing) — disjoint.
- **00-04** (docs rewrite + doc-lint) — disjoint (doc-lint's allowlist excludes `vendor/_reference/**` and the script itself lives under `scripts/`, not `vendor/`).
- **00-05** (CI) — disjoint.

No ordering constraint against any other plan.

## Steps

> Execute from repo root. Content for each file comes from **RESEARCH.md** items noted inline.

1. **Sanity check** that `vendor/pyo3`, `vendor/ratatui`, and `vendor/tui-vfx` are all git-tracked directories (per RESEARCH §Specific Concerns they exist as local placeholder stubs, not real upstream clones):
   ```bash
   for d in pyo3 ratatui tui-vfx; do test -d "vendor/$d" || { echo "missing vendor/$d"; exit 1; }; done
   ```

2. **Relocate each vendored directory** using `git mv` so history is preserved:
   ```bash
   mkdir -p vendor/_reference
   git mv vendor/pyo3    vendor/_reference/pyo3
   git mv vendor/ratatui vendor/_reference/ratatui
   git mv vendor/tui-vfx vendor/_reference/tui-vfx
   ```

3. **Create `vendor/_reference/pyo3/STAMP.txt`** with the exact content from **RESEARCH Item 6** (the "Concrete STAMP.txt files to create" → pyo3 block). The stamp uses the **placeholder status** sentinel `placeholder — no upstream snapshot taken; local stub only.` — this is correct for the current state (the directories are local stubs, not real upstream clones). Captured-by is `Nathan Ribeiro <nribeiro@strm.com.br>` per the RESEARCH block.

4. **Create `vendor/_reference/ratatui/STAMP.txt`** using the same template from Item 6, substituting:
   - `Upstream:        https://github.com/ratatui/ratatui`
   - `Upstream name:   ratatui`
   - `Upstream version: (placeholder, workspace pins 0.30)`
   - All other fields identical to the pyo3 stamp pattern (placeholder commit, 2026-04-14 captured date, Nathan Ribeiro captured-by).
   - `Relationship to workspace:` paragraph mirrors the pyo3 version but references the ratatui pin (0.30) and mentions that the real upstream snapshot can be taken at any time when someone wants to grep real ratatui source.

5. **Create `vendor/_reference/tui-vfx/STAMP.txt`** the same way with:
   - `Upstream:        https://github.com/5ocworkshop/tui-vfx`
   - `Upstream name:   tui-vfx`
   - `Upstream version: (placeholder — project not consumed; tachyonfx replaces it)`
   - Add the extra `Purpose:` line from Item 6: **"Preserved as reference for the 'why we chose tachyonfx over tui-vfx' rationale section in PROJECT.md."** (This is the rationale anchor for the tachyonfx pivot.)

6. **Create `.gitattributes`** at repo root with the exact content from **RESEARCH Item 5**. Key lines:
   - `* text=auto eol=lf`
   - `*.rs`, `*.toml`, `*.md`, `*.yml`, `*.yaml`, `Cargo.lock` → `text eol=lf`
   - `vendor/_reference/** linguist-vendored=true`
   - `vendor/_reference/**/*.rs linguist-vendored=true -diff`
   - `vendor/_reference/** -text`
   - `*.obj binary`, `*.stl binary`, `*.png binary`, `*.gif binary`

7. **Create `rust-toolchain.toml`** at repo root with the content from **RESEARCH Item 3**:
   ```toml
   [toolchain]
   channel    = "1.86"
   components = ["clippy", "rustfmt"]
   profile    = "default"
   ```
   (Do **not** add a `targets = [...]` line — wasm/cross targets are later-milestone concerns. `profile = "default"`, not `"minimal"`, per CONTEXT.md lock.)

8. **Verify no `Cargo.toml` in the workspace references vendor/ via `path =`:**
   ```bash
   # Any path = reference that ends up under vendor/ would be an HYG-04 violation.
   ! rg '^\s*(\w[\w-]*)\s*=\s*\{[^}]*path\s*=\s*"[^"]*vendor/' . \
       --glob '!target/**' --glob '!.eclusa/**' --glob '!.git/**'
   ```

9. **Commit atomically** with the message below.

## Files touched
**Moved (via git mv — history preserved):**
- `vendor/pyo3/` → `vendor/_reference/pyo3/`
- `vendor/ratatui/` → `vendor/_reference/ratatui/`
- `vendor/tui-vfx/` → `vendor/_reference/tui-vfx/`

**Created:**
- `vendor/_reference/pyo3/STAMP.txt`
- `vendor/_reference/ratatui/STAMP.txt`
- `vendor/_reference/tui-vfx/STAMP.txt`
- `.gitattributes`
- `rust-toolchain.toml`

**Modified / deleted:** none beyond the moves above. `vendor/` itself remains as the parent directory (now containing only `_reference/`).

## Commit message
```
chore(vendor): relocate vendored copies to vendor/_reference/ + pin toolchain 1.86 (HYG-04, HYG-07)

- git mv vendor/{pyo3,ratatui,tui-vfx} -> vendor/_reference/{pyo3,ratatui,tui-vfx}/
- Each _reference/ dir carries a STAMP.txt recording upstream URL, name, placeholder
  status (current directories are local stubs, not real upstream snapshots), capture
  date 2026-04-14, and the refresh recipe for future maintainers.
- Add .gitattributes: LF line endings for *.rs/*.toml/*.md/*.yml/*.yaml/Cargo.lock,
  linguist-vendored=true for vendor/_reference/**, binary flags for common fixtures.
- Add rust-toolchain.toml pinning channel=1.86 with clippy + rustfmt + profile=default.
```

## Success criteria (shell-observable)
```bash
# 1. Each old vendor dir is gone from its original location
test ! -d vendor/pyo3
test ! -d vendor/ratatui
test ! -d vendor/tui-vfx

# 2. Each _reference dir exists with a STAMP.txt
for d in pyo3 ratatui tui-vfx; do
  test -d "vendor/_reference/$d"
  test -f "vendor/_reference/$d/STAMP.txt"
  grep -q '^Upstream:' "vendor/_reference/$d/STAMP.txt"
  grep -q '^Captured:' "vendor/_reference/$d/STAMP.txt"
done

# 3. tui-vfx STAMP calls out the tachyonfx-rationale purpose
grep -q 'Preserved as reference' vendor/_reference/tui-vfx/STAMP.txt

# 4. .gitattributes flags vendor/_reference as vendored
test -f .gitattributes
grep -q 'vendor/_reference/\*\* *linguist-vendored=true' .gitattributes
grep -q 'eol=lf' .gitattributes

# 5. rust-toolchain.toml pins 1.86
test -f rust-toolchain.toml
grep -q '^channel *= *"1.86"$' rust-toolchain.toml
grep -q 'clippy' rust-toolchain.toml
grep -q 'rustfmt' rust-toolchain.toml
grep -q 'profile *= *"default"' rust-toolchain.toml

# 6. No Cargo.toml uses path="...vendor/..."
! rg '^\s*\w+\s*=\s*\{[^}]*path\s*=\s*"[^"]*vendor/' . \
    --glob '!target/**' --glob '!.eclusa/**' --glob '!.git/**'

# 7. rustup picks up the pin (may trigger download on first run; acceptable)
rustc --version   # should install and print 1.86.x if not already present
```

## Out of scope
- **Do not** fetch real upstream snapshots for any of the three vendored crates. Current directories are local stubs; the STAMP.txt explicitly records `placeholder — no upstream snapshot taken`. Replacing them with real clones is a future maintenance task (refresh recipe is in each STAMP).
- **Do not** add any `path = "vendor/..."` entries to any `Cargo.toml`. Vendored copies are for **reading only**.
- **Do not** add `.cargo/config.toml` — RESEARCH Item 2 keeps all lint policy in `[workspace.lints]`; no `.cargo/config.toml` is required in Phase 0.
- **Do not** add `targets = [...]` in `rust-toolchain.toml` — wasm/cross-compile targets belong to later milestones.
- **Do not** edit `Cargo.toml`, `README.md`, crate sources, CI, or LICENSE files — those are other plans.

## Open questions (implementer may decide at execution time)
- **`Captured by:`** in STAMP.txt files is set to `Nathan Ribeiro <nribeiro@strm.com.br>` per RESEARCH Item 6. If the implementer is a different human / the executing agent wants to use `git config user.name / user.email`, either is acceptable — the field records the snapshot captor, not the project owner. Default: use the Item-6 string verbatim.
- **Whether to fetch real upstream snapshots now.** RESEARCH explicitly says this is optional and out of scope for Phase 0; placeholders are fine. If the implementer decides to pull real sources (e.g., `git clone ratatui` at v0.30.x), replace the placeholder commit fields with real SHAs per the refresh recipe in each STAMP. This is a judgment call — default is `don't`.
