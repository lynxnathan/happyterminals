---
phase: 00-workspace-hygiene-foundation
plan: 02
type: execute
wave: 1
depends_on: []
files_modified:
  - LICENSE-MIT
  - LICENSE-APACHE
  - CONTRIBUTING.md
  - CHANGELOG.md
autonomous: true
requirements: [HYG-05]

must_haves:
  truths:
    - "Both `LICENSE-MIT` and `LICENSE-APACHE` exist at the repo root with canonical text and the `\"the happyterminals authors\"` copyright line."
    - "`CONTRIBUTING.md` carries the verbatim Rust-ecosystem-standard Apache-2.0 §5 contribution clause."
    - "`CHANGELOG.md` follows Keep-a-Changelog 1.1.0 format and documents the 0.0.0 workspace-scaffolding release."
  artifacts:
    - path: "LICENSE-MIT"
      provides: "MIT license text with copyright line 'Copyright (c) 2026 the happyterminals authors'"
      contains: "Copyright (c) 2026 the happyterminals authors"
    - path: "LICENSE-APACHE"
      provides: "Apache License 2.0 canonical text with Appendix attribution 'Copyright 2026 the happyterminals authors'"
      contains: "Apache License"
    - path: "CONTRIBUTING.md"
      provides: "Contributor guide + Apache-2.0 §5 dual-license clause"
      contains: "shall be dual licensed as"
    - path: "CHANGELOG.md"
      provides: "Keep-a-Changelog skeleton with [Unreleased] and [0.0.0] sections"
      contains: "Keep a Changelog"
  key_links:
    - from: "CONTRIBUTING.md License section"
      to: "LICENSE-MIT / LICENSE-APACHE"
      via: "textual cross-reference; clause matches Rust-ecosystem boilerplate verbatim"
      pattern: "MIT OR Apache-2.0"
---

# Plan 00-02 — Licensing, Contribution, Changelog

## Goal
Establish the dual-license baseline and the contributor-onboarding paper trail: drop `LICENSE-MIT` + `LICENSE-APACHE` at repo root with the locked `"the happyterminals authors"` copyright line, add `CONTRIBUTING.md` with the standard Apache-2.0 §5 clause, and open `CHANGELOG.md` with a Keep-a-Changelog 0.0.0 entry.

## Requirements covered
- **HYG-05** (licensing + contribution clause) — the LICENSE files, the CONTRIBUTING §"Contribution" clause, and the CHANGELOG all originate here. The SPDX `license = "MIT OR Apache-2.0"` fields on each crate's `Cargo.toml` are provided by plan **00-01** via `[workspace.package] license = "MIT OR Apache-2.0"` inheritance.

## Dependencies
None. Pure repo-root documentation; no coupling to Cargo or crate layout.

## Parallelizable with
- **00-01** (workspace refactor) — disjoint file set.
- **00-03** (vendor relocation + toolchain) — disjoint file set.
- **00-04** (docs rewrite + doc-lint) — disjoint file set (README.md, project.md, per-crate READMEs, scripts/, docs/ are 00-04's turf).
- **00-05** (CI) — disjoint.

This plan can run first, last, or concurrently with any other. No ordering constraint.

## Steps

> Execute from repo root. All three content blocks live in **RESEARCH.md**; copy verbatim.

1. **Create `LICENSE-MIT`** at repo root with the exact text from **RESEARCH Item 1** (the `LICENSE-MIT` fenced block). The copyright line must read **`Copyright (c) 2026 the happyterminals authors`** — user decision 2026-04-14, not "Nathan Ribeiro".

2. **Create `LICENSE-APACHE`** at repo root with the canonical Apache-2.0 text from **RESEARCH Item 1** (the `LICENSE-APACHE` fenced block). The text of the license is never modified; only the Appendix attribution line at the bottom is customized to **`Copyright 2026 the happyterminals authors`**.

3. **Create `CONTRIBUTING.md`** at repo root with the exact content from **RESEARCH Item 12**. This file includes:
   - Prerequisites (Rust 1.86, cargo-machete, ripgrep)
   - Build/test/lint commands
   - Workspace layout pointer
   - Forbidden-strings list (tui-vfx, Haskell bindings, pyo3-asyncio, cgmath, tui-rs) — references `docs/decisions/stack-rationale.md` (created by plan 00-04)
   - Commit style
   - **License §"Contribution"** — the verbatim Rust-ecosystem boilerplate paragraph (`shall be dual licensed as / MIT OR Apache-2.0 / without any additional terms or conditions`). **Do not rephrase.**

4. **Create `CHANGELOG.md`** at repo root with the content from **RESEARCH Item 13**. The skeleton has:
   - Header + Keep-a-Changelog + SemVer references
   - Pre-1.0 versioning note
   - `[Unreleased]` section (Added / Changed / Removed subsections)
   - `[0.0.0] — 2026-04-14` section that describes the Phase 0 scaffolding (dual-license files, `[workspace.dependencies]`, `rust-toolchain.toml`, new workspace members, `.gitattributes`, doc-lint script, CI, CONTRIBUTING, stack-rationale.md, plus the compositor→pipeline rename under Changed and the speculative-deps removal under Removed)
   - Compare/tag link footnotes (using `github.com/lynxnathan/happyterminals` placeholder URL)

5. **Sanity-check file contents with shell** (see Success criteria below) before committing.

6. **Commit** with the message given.

## Files touched
**Created (all repo root):**
- `LICENSE-MIT`
- `LICENSE-APACHE`
- `CONTRIBUTING.md`
- `CHANGELOG.md`

**Modified:** none.

**Deleted:** none.

## Commit message
```
docs(licensing): add LICENSE-MIT + LICENSE-APACHE + CONTRIBUTING + CHANGELOG (HYG-05)

- Dual-license files at repo root ("the happyterminals authors" copyright line,
  per user decision 2026-04-14 — avoids churn as contributors join)
- CONTRIBUTING.md with the verbatim Rust-ecosystem Apache-2.0 §5 contribution clause
- CHANGELOG.md Keep-a-Changelog skeleton opened at [0.0.0] for Phase 0 scaffolding
```

## Success criteria (shell-observable)
```bash
# 1. Both license files exist at root
test -f LICENSE-MIT
test -f LICENSE-APACHE

# 2. Copyright line is locked verbatim in MIT (NOT "Nathan Ribeiro")
grep -qx 'Copyright (c) 2026 the happyterminals authors' LICENSE-MIT
! grep -q 'Nathan Ribeiro' LICENSE-MIT
! grep -q 'Nathan Ribeiro' LICENSE-APACHE

# 3. Apache-2.0 canonical text present
grep -q 'Apache License' LICENSE-APACHE
grep -q 'Version 2.0, January 2004' LICENSE-APACHE
grep -q 'Copyright 2026 the happyterminals authors' LICENSE-APACHE

# 4. CONTRIBUTING.md contains the verbatim dual-license clause
test -f CONTRIBUTING.md
grep -q 'shall be dual licensed as' CONTRIBUTING.md
grep -q 'MIT OR Apache-2.0' CONTRIBUTING.md

# 5. CHANGELOG.md is Keep-a-Changelog format with 0.0.0 opened
test -f CHANGELOG.md
grep -q 'Keep a Changelog' CHANGELOG.md
grep -q '^## \[0.0.0\]' CHANGELOG.md
grep -q '^## \[Unreleased\]' CHANGELOG.md
```

## Out of scope
- **Do not** modify any `crates/*/Cargo.toml` to add `license = "..."` — that's plan 00-01's job via `[workspace.package]` inheritance.
- **Do not** add a `NOTICE` file. Apache-2.0 §4(d) makes NOTICE optional; we don't need one. Adding one later is cheap.
- **Do not** add a `SECURITY.md`, `CODE_OF_CONDUCT.md`, or GitHub issue templates — those are post-announcement polish, not Phase 0.
- **Do not** run any `cargo publish` / registry-reservation steps (HYG-06 is deferred).
- **Do not** rewrite `README.md` or `project.md` — those belong to plan 00-04.
- **Do not** edit the Apache-2.0 body text. Only the Appendix line at the very bottom is customized.

## Open questions (implementer may decide at execution time)
- **Exact CHANGELOG compare-tag URLs.** RESEARCH Item 13 uses `github.com/lynxnathan/happyterminals` as the repo URL. If the final owner/slug differs, update the two footnote links consistently with the README badge URLs in plan 00-04; otherwise use the Item-13 values verbatim.
- **Whether to include a `NOTICE` file.** Default: no (Apache-2.0 §4(d) does not require it). If the implementer disagrees, this is a reversible decision — add a one-line NOTICE in a follow-up commit.
