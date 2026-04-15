---
phase: 00-workspace-hygiene-foundation
plan: 05
type: execute
wave: 3
depends_on: [00-01, 00-04]
files_modified:
  - .github/workflows/ci.yml
autonomous: true
requirements: [HYG-08, HYG-09]

must_haves:
  truths:
    - "`.github/workflows/ci.yml` exists and defines `fmt`, `clippy`, `test` (matrix 1.86 + stable), `docs`, and `hygiene` jobs."
    - "`clippy` job runs `cargo clippy --workspace --all-targets --all-features -- -D warnings`."
    - "`docs` job sets `RUSTDOCFLAGS=-D warnings` and runs `cargo doc --workspace --no-deps --all-features`."
    - "`hygiene` job runs `cargo tree --workspace --duplicates --edges normal` and fails on any output."
    - "`hygiene` job installs **cargo-machete** (not `cargo-udeps`) via `taiki-e/install-action@v2` and runs `cargo machete`."
    - "`hygiene` job runs `bash scripts/doc-lint.sh` as the final step (references the script shipped by plan 00-04)."
    - "Workflow declares a `concurrency` group with `cancel-in-progress: true`."
  artifacts:
    - path: ".github/workflows/ci.yml"
      provides: "CI baseline: fmt, clippy -D warnings, test, doc -D warnings, duplicate-dep scan, cargo-machete, doc-lint"
      contains: "cargo-machete"
  key_links:
    - from: ".github/workflows/ci.yml hygiene job"
      to: "scripts/doc-lint.sh"
      via: "bash scripts/doc-lint.sh step"
      pattern: "bash scripts/doc-lint.sh"
    - from: ".github/workflows/ci.yml test job matrix"
      to: "rust-toolchain.toml"
      via: "explicit `toolchain: '1.86'` + matrix stable; runner installs via dtolnay/rust-toolchain@master"
      pattern: "toolchain: \"1.86\""
---

# Plan 00-05 — GitHub Actions CI

## Goal
Drop `.github/workflows/ci.yml` — the five-job CI baseline (fmt, clippy `-D warnings`, test matrix on 1.86 + stable, doc `-D warnings`, hygiene = duplicate-dep + cargo-machete + doc-lint) — so every push and PR validates the Phase 0 guarantees automatically.

## Requirements covered
- **HYG-08** — fmt / clippy / test / doc / duplicate-dep / unused-dep CI baseline.
- **HYG-09** — doc-lint CI step that fails the build on forbidden strings (the script itself is owned by plan 00-04; **this plan wires it into CI**).

## Dependencies
- **00-01 must land first** — the CI matrix needs the finalized workspace (resolver=3, seven members, pinned `[workspace.dependencies]`). If the workspace layout is inconsistent with what CI expects, `cargo clippy --workspace` and `cargo tree --workspace` will produce noise or false failures.
- **00-04 must land first** — `scripts/doc-lint.sh` and the `docs/decisions/stack-rationale.md` allowlist file must exist before the `hygiene` job's `bash scripts/doc-lint.sh` step runs; otherwise the CI step fails on the very first push.

Soft dependency on **00-03** (`rust-toolchain.toml`): CI explicitly passes `toolchain: "1.86"` to `dtolnay/rust-toolchain@master`, so even without `rust-toolchain.toml` the jobs work. But it's cleaner to have both converged before the first CI run.

## Parallelizable with
Nothing in Phase 0 — this is the sequencing tail. If the implementer has landed 00-01 and 00-04 on `main` (or a shared integration branch), **00-02** and **00-03** can still be in flight concurrently; they do not touch `.github/`.

## Steps

> Execute from repo root. Content comes from **RESEARCH Item 7**.

1. **Create `.github/workflows/` directory** if it doesn't exist:
   ```bash
   mkdir -p .github/workflows
   ```

2. **Create `.github/workflows/ci.yml`** with the exact YAML content from **RESEARCH Item 7**. The file declares:
   - `on: push` and `on: pull_request` to `main` / `master`
   - `concurrency: group: ci-${{ github.workflow }}-${{ github.ref }}` with `cancel-in-progress: true`
   - `env:` block: `CARGO_TERM_COLOR: always`, `CARGO_INCREMENTAL: 0`, `RUST_BACKTRACE: 1`
   - **`fmt` job** — `ubuntu-latest`, `dtolnay/rust-toolchain@master` with `toolchain: "1.86"` + `components: rustfmt`, runs `cargo fmt --all -- --check`
   - **`clippy` job** — `ubuntu-latest`, toolchain 1.86 + clippy component, `Swatinem/rust-cache@v2`, runs `cargo clippy --workspace --all-targets --all-features -- -D warnings`
   - **`test` job** — matrix `rust: ["1.86", "stable"]`, `fail-fast: false`, `Swatinem/rust-cache@v2` keyed by matrix value, runs `cargo test --workspace --all-features --no-fail-fast`
   - **`docs` job** — `ubuntu-latest`, toolchain 1.86, `Swatinem/rust-cache@v2`, `env: RUSTDOCFLAGS: "-D warnings"`, runs `cargo doc --workspace --no-deps --all-features`
   - **`hygiene` job** — `ubuntu-latest`, toolchain 1.86, `Swatinem/rust-cache@v2`, with three steps:
     1. `cargo tree --workspace --duplicates --edges normal --format '{p}'` in a bash block that captures output and exits non-zero if non-empty; emits `::error::` on violations (verbatim shell from RESEARCH Item 7).
     2. Install **`cargo-machete`** via `taiki-e/install-action@v2` (NOT `cargo-udeps` — that's nightly-only and incompatible with our 1.86 pin; see RESEARCH §"2026 Convention Flags").
     3. Run `cargo machete`.
     4. Run `bash scripts/doc-lint.sh` (references the script from plan 00-04).

3. **Validate YAML syntax locally** (optional but recommended):
   ```bash
   # If yq/yamllint is available:
   yamllint -d relaxed .github/workflows/ci.yml || true
   # Otherwise, a Python fallback:
   python3 -c 'import yaml, sys; yaml.safe_load(open(".github/workflows/ci.yml"))'
   ```

4. **Cross-check:**
   - The `hygiene` job uses `cargo-machete`, **NOT** `cargo-udeps` — grep the file.
   - The job lists the `bash scripts/doc-lint.sh` step last.
   - The `test` job's matrix has exactly two entries: `"1.86"` and `"stable"`.
   - Only `ubuntu-latest` runners are used (macOS / Windows are deferred to M2 per CONTEXT.md).

5. **Commit** with the message below.

6. **Post-commit validation** (runs on GitHub once pushed, not strictly a local gate):
   - Push to a feature branch; open a PR.
   - Confirm all five CI jobs trigger and turn green.
   - If the `hygiene` job fails on `cargo tree --duplicates`, that indicates a duplicate-dep issue introduced by plan 00-01's `[workspace.dependencies]`; escalate back to 00-01 rather than widening the hygiene check.
   - If `hygiene` fails on `doc-lint`, fix the offending source file (not the allowlist).

## Files touched
**Created:**
- `.github/workflows/ci.yml`

**Modified / deleted:** none.

## Commit message
```
ci(phase0): GitHub Actions baseline — fmt, clippy -D warnings, test matrix 1.86+stable, doc -D warnings, duplicate-dep scan, cargo-machete, doc-lint (HYG-08, HYG-09)

- fmt:     cargo fmt --all -- --check
- clippy:  cargo clippy --workspace --all-targets --all-features -- -D warnings
- test:    matrix rust=[1.86, stable]; fail-fast=false; cargo test --workspace
- docs:    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
- hygiene: cargo tree --workspace --duplicates --edges normal (empty = pass);
           cargo-machete installed via taiki-e/install-action@v2 (NOT cargo-udeps
           — udeps requires nightly, incompatible with our 1.86 pin);
           bash scripts/doc-lint.sh (forbidden-string scanner from plan 00-04).

ubuntu-latest only in Phase 0; macOS/Windows runners deferred to M2.
concurrency group cancels in-flight runs on the same ref.
```

## Success criteria (shell-observable)
```bash
# 1. Workflow file exists and parses as valid YAML
test -f .github/workflows/ci.yml
python3 -c 'import yaml; yaml.safe_load(open(".github/workflows/ci.yml"))'

# 2. Exactly five jobs declared
grep -cE '^\s{2}[a-z_-]+:$' .github/workflows/ci.yml | awk '$1 >= 5 {exit 0} {exit 1}'
grep -q '^  fmt:'     .github/workflows/ci.yml
grep -q '^  clippy:'  .github/workflows/ci.yml
grep -q '^  test:'    .github/workflows/ci.yml
grep -q '^  docs:'    .github/workflows/ci.yml
grep -q '^  hygiene:' .github/workflows/ci.yml

# 3. clippy is -D warnings; docs is RUSTDOCFLAGS=-D warnings
grep -q -- '-D warnings' .github/workflows/ci.yml
grep -q 'RUSTDOCFLAGS' .github/workflows/ci.yml

# 4. cargo-machete is used; cargo-udeps is NOT
grep -q 'cargo-machete' .github/workflows/ci.yml
! grep -q 'cargo-udeps' .github/workflows/ci.yml

# 5. doc-lint script is invoked
grep -q 'bash scripts/doc-lint.sh' .github/workflows/ci.yml

# 6. cargo tree duplicate scan present
grep -q 'cargo tree' .github/workflows/ci.yml
grep -q -- '--duplicates' .github/workflows/ci.yml

# 7. Concurrency with cancel-in-progress
grep -q '^concurrency:' .github/workflows/ci.yml
grep -q 'cancel-in-progress: true' .github/workflows/ci.yml

# 8. ubuntu-latest only (no macos/windows runners in Phase 0)
! grep -qE 'runs-on:\s*(macos|windows)' .github/workflows/ci.yml

# 9. Test matrix covers 1.86 AND stable
grep -q '"1.86"' .github/workflows/ci.yml
grep -q '"stable"' .github/workflows/ci.yml
```

## Out of scope
- **Do not** add macOS or Windows runners — deferred to M2 per CONTEXT.md. Cross-OS CI costs runner minutes and provides little value until the renderer/backend ships.
- **Do not** add `cargo-udeps` as a CI step. It's nightly-only and incompatible with our 1.86 pin. A weekly opt-in udeps job can be added later (RESEARCH §"2026 Convention Flags" notes this as explicitly future scope).
- **Do not** add `cargo semver-checks`, `release-plz`, `docs.rs` feature-matrix builds, or publish automation — all deferred to M3 publish phase.
- **Do not** add deploy jobs, GitHub Pages for docs, coverage uploads, or other non-hygiene steps. Phase 0 is a validation baseline only.
- **Do not** inline the doc-lint logic in the CI YAML; the script lives in `scripts/doc-lint.sh` (plan 00-04) so developers can run the same check locally.
- **Do not** prepare any `cargo publish` / PyPI-reservation steps — HYG-06 is deferred.

## Open questions (implementer may decide at execution time)
- **Whether to also add `cargo build --workspace` as a distinct CI job.** Default: no. `cargo test` and `cargo clippy` both build, and an additional build job is redundant minutes.
- **Whether to cache cargo-machete binary installs.** `taiki-e/install-action@v2` ships prebuilts fast enough that caching typically doesn't help. Default: trust `install-action`.
- **Whether to gate the `docs` job on `clippy` passing first.** GitHub Actions doesn't express inter-job dependencies for speed concerns unless they share artifacts. Default: run all jobs in parallel as shown in RESEARCH Item 7.
- **Exact path to the `cargo tree --duplicates` shell block.** RESEARCH Item 7 provides a complete shell snippet with `set -euo pipefail`, variable capture, and `::error::` emission. Use that verbatim; do not simplify to a bare `[ -z "$(cargo tree -d)" ]` (that pattern does not handle whitespace-only output correctly).
